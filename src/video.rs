//! Video support: frame sampling and clip-level aggregation.
//!
//! Frames are decoded in-process: by the system decoder on macOS
//! ([`crate::avf`]) and by a pure-Rust reader elsewhere ([`crate::swdec`],
//! MP4 and MOV with H.264 or HEVC). `TINTYPE_DECODER=rust` selects the
//! pure-Rust reader on macOS too. No external tool is called.
use crate::image_io::Rgb;
use anyhow::Result;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub fps: f64,
}

#[cfg(not(target_os = "macos"))]
fn unsupported<T>(path: &Path) -> Result<T> {
    anyhow::bail!("re-encoding needs the macOS system encoder; {} cannot be re-encoded on this platform", path.display())
}

/// Whether the pure-Rust reader was asked for in place of the system decoder.
#[cfg(target_os = "macos")]
fn pure_rust() -> bool {
    std::env::var("TINTYPE_DECODER").is_ok_and(|v| v == "rust")
}

/// Run a decode, and once more if the system decoder stalled: stalls are
/// intermittent, not a property of the file.
#[cfg(target_os = "macos")]
fn decode<T>(f: impl Fn() -> Result<T>) -> Result<T> {
    match f() {
        Err(e) if e.to_string().contains("stalled") => {
            eprintln!("warning: {e}; trying once more");
            f()
        }
        r => r,
    }
}

#[cfg(target_os = "macos")]
fn decode_with_fallback<T>(system: impl Fn() -> Result<T>, software: impl FnOnce() -> Result<T>) -> Result<T> {
    decode(system).or_else(|system_error| {
        software().map_err(|software_error| anyhow::anyhow!("system decoder: {system_error}; Rust decoder: {software_error}"))
    })
}

pub fn probe(path: &Path) -> Result<VideoInfo> {
    #[cfg(target_os = "macos")]
    if !pure_rust() {
        return decode_with_fallback(|| crate::avf::probe(path), || crate::swdec::probe(path));
    }
    crate::swdec::probe(path)
}

/// How frames are picked from a clip.
#[derive(Clone, Copy, Debug)]
pub struct Sampling {
    /// Number of frames to analyse.
    pub frames: usize,
    /// Only the first `max_seconds` of the clip are considered; zero or less
    /// means the whole clip.
    pub max_seconds: f64,
    /// Take the first `frames` consecutive frames instead of spreading them
    /// over the clip (what the WaveRep reference pipeline does).
    pub consecutive: bool,
}

impl Default for Sampling {
    fn default() -> Self {
        Self { frames: 8, max_seconds: 30.0, consecutive: false }
    }
}

/// Seconds of a clip that sampling spreads its frames over: the clip's
/// duration, capped at `max_seconds` when that is positive. A clip of unknown
/// duration is taken to be `max_seconds` long (30 s when there is no cap).
pub fn span(duration: f64, max_seconds: f64) -> f64 {
    match (duration > 0.0, max_seconds > 0.0) {
        (true, true) => duration.min(max_seconds),
        (true, false) => duration,
        (false, true) => max_seconds,
        (false, false) => 30.0,
    }
}

/// What a clip's verdict rests on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coverage {
    /// Frames that were scored.
    pub frames: usize,
    /// Frames sampling asked for.
    pub frames_expected: usize,
    /// The clip's duration in seconds, when the container states it.
    pub duration: Option<f64>,
    /// Latest sampling target represented in the score, in seconds from the start.
    /// A decoder may use the nearest frame to each target.
    pub until: Option<f64>,
    /// False when decoding stopped before the frames the verdict needed had
    /// been scored. A clip that early exit decided, and a clip shorter than
    /// the number of frames asked for, are complete.
    pub complete: bool,
}

/// Work out the coverage of a clip from the number of frames that were
/// scored. `order` is the early-exit looking order when early exit ran, and
/// `decided` says whether one of its stages settled the clip.
pub fn coverage(frames: usize, s: Sampling, info: Option<VideoInfo>, order: Option<&[usize]>, decided: bool) -> Coverage {
    let nf = s.frames.max(1);
    let duration = info.map(|i| i.duration).filter(|d| d.is_finite() && *d > 0.0);
    let span = duration.map(|d| span(d, s.max_seconds));
    // Frames the sampled part of the clip holds, when the container says.
    let available = info.zip(span).map(|(i, sp)| (sp * i.fps).round() as usize).filter(|&n| n > 0);
    let complete = frames > 0 && (decided || frames >= nf || available.is_some_and(|a| frames.saturating_add(1) >= nf.min(a)));
    // Index, among the frames asked for, of the latest one scored.
    let last = match order {
        Some(o) => o.iter().take(frames).copied().max(),
        _ => frames.checked_sub(1),
    };
    let until = span.zip(last).map(|(sp, k)| sp * k.min(nf - 1) as f64 / nf as f64);
    Coverage { frames, frames_expected: nf, duration, until, complete }
}

/// Decode `s.frames` frames at native resolution: spread evenly over the
/// clip, or the first ones in order when `s.consecutive` is set.
pub fn sample_frames(path: &Path, s: Sampling) -> Result<Vec<Rgb>> {
    let software = || if s.consecutive {
        crate::swdec::frames_first(path, s.frames)
    } else {
        crate::swdec::frames_uniform(path, s.frames, s.max_seconds)
    };
    #[cfg(target_os = "macos")]
    if !pure_rust() {
        return decode_with_fallback(|| if s.consecutive { crate::avf::frames_first(path, s.frames) } else { crate::avf::frames_uniform(path, s.frames, s.max_seconds) }, software);
    }
    software()
}

/// Frames `from..to` of the ones [`sample_frames`] returns for `s` (spread
/// sampling only), without decoding past the last one wanted.
pub fn sample_frames_range(path: &Path, s: Sampling, from: usize, to: usize) -> Result<Vec<Rgb>> {
    let software = || crate::swdec::frames_uniform_range(path, s.frames, s.max_seconds, from, to);
    #[cfg(target_os = "macos")]
    if !pure_rust() {
        return decode_with_fallback(|| crate::avf::frames_uniform_range(path, s.frames, s.max_seconds, from, to), software);
    }
    software()
}

/// A picture on its way to being scored: decoded already, or a video frame
/// still as the decoder gave it. Converting a frame to RGB costs about as
/// much as cutting all its views out, so a frame that early exit may never
/// look at is kept unconverted until it is.
pub enum Frame {
    Rgb(Rgb),
    Yuv(crate::yuv::Yuv),
}

impl Frame {
    /// The picture, converted the first time it is asked for (the conversion
    /// [`sample_frames`] applies).
    pub fn rgb(&mut self) -> &Rgb {
        if let Frame::Yuv(f) = self {
            *self = Frame::Rgb(f.to_rgb());
        }
        match self {
            Frame::Rgb(im) => im,
            Frame::Yuv(_) => unreachable!("converted above"),
        }
    }

    /// Bytes the picture takes as RGB.
    pub fn rgb_bytes(&self) -> usize {
        match self {
            Frame::Rgb(im) => im.data.len(),
            Frame::Yuv(f) => f.w * f.h * 3,
        }
    }
}

impl From<Rgb> for Frame {
    fn from(im: Rgb) -> Self {
        Frame::Rgb(im)
    }
}

/// As [`sample_frames_range`], with the frames left unconverted (see [`Frame`]).
pub fn sample_frames_range_unconverted(path: &Path, s: Sampling, from: usize, to: usize) -> Result<Vec<Frame>> {
    let wrap = |frames: Vec<crate::yuv::Yuv>| frames.into_iter().map(Frame::Yuv).collect();
    let software = || crate::swdec::frames_uniform_range_unconverted(path, s.frames, s.max_seconds, from, to).map(wrap);
    #[cfg(target_os = "macos")]
    if !pure_rust() {
        return decode_with_fallback(|| crate::avf::frames_uniform_range_unconverted(path, s.frames, s.max_seconds, from, to).map(wrap), software);
    }
    software()
}

/// Combine per-frame logits into one clip logit. The mean is the
/// log-likelihood-ratio average, the same rule the strongest frame-level
/// video detectors use.
pub fn aggregate(frame_logits: &[f32]) -> f32 {
    frame_logits.iter().sum::<f32>() / frame_logits.len().max(1) as f32
}

/// Decode the first frame at or after time `t` (seconds).
pub fn frame_at(path: &Path, t: f64) -> Result<Rgb> {
    let software = || crate::swdec::frame_at(path, t);
    #[cfg(target_os = "macos")]
    if !pure_rust() {
        return decode_with_fallback(|| crate::avf::frame_at(path, t), software);
    }
    software()
}

/// A random training frame. With probability `recompress_p` a short segment
/// around the chosen instant is first re-encoded as H.264 at a random bitrate
/// and scale, so bitrate and resolution stop being class cues.
pub fn random_frame(path: &Path, rng: &mut crate::rng::Rng, recompress_p: f32, tmp_dir: &Path) -> Result<Rgb> {
    let info = probe(path)?;
    let t = rng.range(0.0, (info.duration - 0.7).max(0.0) as f32) as f64;
    if !rng.chance(recompress_p) {
        return frame_at(path, t);
    }
    let tmp = tmp_dir.join(format!("seg_{:016x}.mp4", rng.u64()));
    // Bits per pixel from visually clean down to heavily compressed, log-uniform.
    let bits_per_pixel = 0.012 * (rng.range(0.0, 1.0) as f64 * (0.25f64 / 0.012).ln()).exp();
    let scale = if rng.chance(0.5) { 1.0 } else { rng.range(0.5, 1.0) as f64 };
    let frames = ((if info.fps > 0.0 { info.fps } else { 25.0 }) * 0.5).ceil().clamp(2.0, 15.0) as usize;
    // Past the first frame of the segment, so inter-frame coding is represented.
    let pick = 1 + (rng.u64() % (frames as u64 - 1)) as usize;
    let out = recompressed_frame(path, t, frames, scale, bits_per_pixel, pick, &tmp);
    let _ = std::fs::remove_file(&tmp);
    out.or_else(|_| frame_at(path, t))
}

/// A re-encoded frame or an error, with no fallback to the original frame
/// (for checking that the augmentation works on a clip).
pub fn recompressed_only(path: &Path, rng: &mut crate::rng::Rng, tmp_dir: &Path) -> Result<Rgb> {
    let info = probe(path)?;
    let tmp = tmp_dir.join(format!("seg_{:016x}.mp4", rng.u64()));
    let frames = ((if info.fps > 0.0 { info.fps } else { 25.0 }) * 0.5).ceil().clamp(2.0, 15.0) as usize;
    let out = recompressed_frame(path, (info.duration / 2.0 - 0.3).max(0.0), frames, 0.75, 0.05, 1, &tmp);
    let _ = std::fs::remove_file(&tmp);
    out
}

#[cfg(target_os = "macos")]
fn recompressed_frame(path: &Path, t: f64, frames: usize, scale: f64, bits_per_pixel: f64, pick: usize, tmp: &Path) -> Result<Rgb> {
    let written = crate::avf::recompress(path, t, frames, scale, bits_per_pixel, tmp)?;
    let mut all = crate::avf::frames_first_plain(tmp, pick.min(written.saturating_sub(1)) + 1)?;
    all.pop().ok_or_else(|| anyhow::anyhow!("re-encoded segment has no frames"))
}

#[cfg(not(target_os = "macos"))]
fn recompressed_frame(path: &Path, _t: f64, _frames: usize, _scale: f64, _bits_per_pixel: f64, _pick: usize, _tmp: &Path) -> Result<Rgb> {
    unsupported(path)
}

/// The middle frame of a clip (deterministic; used for validation).
pub fn middle_frame(path: &Path) -> Result<Rgb> {
    let info = probe(path)?;
    frame_at(path, (info.duration / 2.0 - 0.05).max(0.0))
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn clip(duration: f64, fps: f64) -> Option<VideoInfo> {
        Some(VideoInfo { width: 640, height: 360, duration, fps })
    }

    #[test]
    fn span_is_capped_only_when_asked() {
        assert_eq!(span(312.0, 30.0), 30.0);
        assert_eq!(span(12.0, 30.0), 12.0);
        assert_eq!(span(312.0, 0.0), 312.0);
        assert_eq!(span(0.0, 30.0), 30.0);
    }

    #[test]
    fn early_exit_and_short_clips_are_complete_and_truncated_ones_are_not() {
        let s = Sampling { frames: 16, max_seconds: 30.0, consecutive: false };
        let order: Vec<usize> = vec![0, 7, 3, 5, 1, 2, 4, 6, 8, 12, 10, 14, 9, 13, 11, 15];
        // Decided after two frames: complete, and it looked no further than frame 7 of 16.
        let c = coverage(2, s, clip(312.0, 30.0), Some(&order), true);
        assert!(c.complete);
        assert_eq!(c.until, Some(30.0 * 7.0 / 16.0));
        assert_eq!(c.duration, Some(312.0));
        // Every frame scored.
        assert!(coverage(16, s, clip(10.0, 30.0), Some(&order), false).complete);
        // A ten-frame clip yields fifteen of the sixteen sampled positions: complete.
        assert!(coverage(15, s, clip(0.4, 25.0), None, false).complete);
        // Undecided at the half-way mark and the second half could not be decoded.
        let c = coverage(8, s, clip(10.0, 30.0), Some(&order), false);
        assert!(!c.complete);
        assert_eq!(c.until, Some(10.0 * 7.0 / 16.0));
        // Decoding stopped a third of the way in.
        assert!(!coverage(6, s, clip(2.0, 30.0), None, false).complete);
        // Nothing known about the clip: only a full set of frames counts.
        assert!(!coverage(9, s, None, None, false).complete);
        assert_eq!(coverage(9, s, None, None, false).until, None);
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// Needs a real clip: `TINTYPE_TEST_VIDEO=clip.mp4 cargo test --release -- --ignored`.
    #[test]
    #[ignore]
    fn reencoded_frame_keeps_the_picture() {
        let path = std::path::PathBuf::from(std::env::var("TINTYPE_TEST_VIDEO").expect("set TINTYPE_TEST_VIDEO"));
        let info = probe(&path).unwrap();
        let t = (info.duration / 2.0 - 0.3).max(0.0);
        let original = frame_at(&path, t).unwrap();
        let tmp = std::env::temp_dir().join(format!("tintype-test-{}.mp4", std::process::id()));
        let written = crate::avf::recompress(&path, t, 4, 1.0, 0.2, &tmp).unwrap();
        assert!(written >= 1);
        let again = crate::avf::frames_first_plain(&tmp, 1).unwrap().remove(0);
        let _ = std::fs::remove_file(&tmp);
        // Same size up to the rounding to even dimensions, and the same colours channel by channel.
        assert!(original.w - again.w <= 1 && original.h - again.h <= 1);
        let mean = |im: &Rgb, c: usize| im.data.iter().skip(c).step_by(3).map(|&v| v as f64).sum::<f64>() / (im.w * im.h) as f64;
        for c in 0..3 {
            assert!((mean(&original, c) - mean(&again, c)).abs() < 6.0, "channel {c}: {} vs {}", mean(&original, c), mean(&again, c));
        }
    }
}
