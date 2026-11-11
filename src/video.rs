//! Video support: frame sampling and clip-level aggregation.
//!
//! Frames are decoded in-process by the system decoder ([`crate::avf`]), so
//! video needs macOS; no external tool is called.
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
    anyhow::bail!("video is decoded by the macOS system decoder; {} cannot be read on this platform", path.display())
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

pub fn probe(path: &Path) -> Result<VideoInfo> {
    #[cfg(target_os = "macos")]
    return decode(|| crate::avf::probe(path));
    #[cfg(not(target_os = "macos"))]
    unsupported(path)
}

/// How frames are picked from a clip.
#[derive(Clone, Copy, Debug)]
pub struct Sampling {
    /// Number of frames to analyse.
    pub frames: usize,
    /// Only the first `max_seconds` of the clip are considered.
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

/// Decode `s.frames` frames at native resolution: spread evenly over the
/// clip, or the first ones in order when `s.consecutive` is set.
pub fn sample_frames(path: &Path, s: Sampling) -> Result<Vec<Rgb>> {
    #[cfg(target_os = "macos")]
    return decode(|| if s.consecutive { crate::avf::frames_first(path, s.frames) } else { crate::avf::frames_uniform(path, s.frames, s.max_seconds) });
    #[cfg(not(target_os = "macos"))]
    {
        let _ = s;
        unsupported(path)
    }
}

/// Frames `from..to` of the ones [`sample_frames`] returns for `s` (spread
/// sampling only), without decoding past the last one wanted.
pub fn sample_frames_range(path: &Path, s: Sampling, from: usize, to: usize) -> Result<Vec<Rgb>> {
    #[cfg(target_os = "macos")]
    return decode(|| crate::avf::frames_uniform_range(path, s.frames, s.max_seconds, from, to));
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (s, from, to);
        unsupported(path)
    }
}

/// Combine per-frame logits into one clip logit. The mean is the
/// log-likelihood-ratio average, the same rule the strongest frame-level
/// video detectors use.
pub fn aggregate(frame_logits: &[f32]) -> f32 {
    frame_logits.iter().sum::<f32>() / frame_logits.len().max(1) as f32
}

/// Decode the first frame at or after time `t` (seconds).
pub fn frame_at(path: &Path, t: f64) -> Result<Rgb> {
    #[cfg(target_os = "macos")]
    return decode(|| crate::avf::frame_at(path, t));
    #[cfg(not(target_os = "macos"))]
    {
        let _ = t;
        unsupported(path)
    }
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

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// Needs a real clip: `VERACITY_TEST_VIDEO=clip.mp4 cargo test --release -- --ignored`.
    #[test]
    #[ignore]
    fn reencoded_frame_keeps_the_picture() {
        let path = std::path::PathBuf::from(std::env::var("VERACITY_TEST_VIDEO").expect("set VERACITY_TEST_VIDEO"));
        let info = probe(&path).unwrap();
        let t = (info.duration / 2.0 - 0.3).max(0.0);
        let original = frame_at(&path, t).unwrap();
        let tmp = std::env::temp_dir().join(format!("veracity-test-{}.mp4", std::process::id()));
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
