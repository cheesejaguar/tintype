//! Video support: frame sampling and clip-level aggregation.
//!
//! On macOS frames are decoded in-process ([`crate::avf`]). Elsewhere, and for
//! the optional H.264 recompression augmentation used in training, the
//! `ffmpeg` binary is called if it is installed.
use crate::image_io::Rgb;
use anyhow::{anyhow, bail, Context, Result};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub fps: f64,
}

fn ratio(s: &str) -> Option<f64> {
    match s.split_once('/') {
        Some((a, b)) => {
            let (a, b) = (a.parse::<f64>().ok()?, b.parse::<f64>().ok()?);
            (b != 0.0).then(|| a / b)
        }
        None => s.parse().ok(),
    }
}

/// Whether to decode in-process (macOS) or through the `ffmpeg` binary.
/// `VERACITY_DECODER=ffmpeg` forces the external decoder.
fn native() -> bool {
    cfg!(target_os = "macos") && std::env::var("VERACITY_DECODER").map(|v| v != "ffmpeg").unwrap_or(true)
}

pub fn probe(path: &Path) -> Result<VideoInfo> {
    #[cfg(target_os = "macos")]
    if native() {
        // Files the system decoder rejects (HEVC tagged `hev1`, for one) fall
        // through to ffmpeg when it is installed.
        if let Ok(i) = crate::avf::probe(path) {
            return Ok(i);
        }
    }
    probe_ffmpeg(path)
}

fn probe_ffmpeg(path: &Path) -> Result<VideoInfo> {
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height,avg_frame_rate,duration:format=duration", "-of", "json"])
        .arg(path)
        .output()
        .context("running ffprobe (is ffmpeg installed?)")?;
    if !out.status.success() {
        bail!("ffprobe failed on {}", path.display());
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let s = v["streams"].get(0).ok_or_else(|| anyhow!("no video stream in {}", path.display()))?;
    let num = |x: &serde_json::Value| x.as_str().and_then(|t| t.parse::<f64>().ok()).or_else(|| x.as_f64());
    let duration = num(&s["duration"]).or_else(|| num(&v["format"]["duration"])).unwrap_or(0.0);
    Ok(VideoInfo {
        width: s["width"].as_u64().unwrap_or(0) as u32,
        height: s["height"].as_u64().unwrap_or(0) as u32,
        duration,
        fps: s["avg_frame_rate"].as_str().and_then(ratio).unwrap_or(0.0),
    })
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

/// Decode `s.frames` frames spread evenly over the clip, at native resolution.
pub fn sample_frames(path: &Path, s: Sampling) -> Result<Vec<Rgb>> {
    #[cfg(target_os = "macos")]
    if native() {
        match if s.consecutive { crate::avf::frames_first(path, s.frames) } else { crate::avf::frames_uniform(path, s.frames, s.max_seconds) } {
            Ok(f) => return Ok(f),
            Err(e) => note_stall(&e),
        }
    }
    let info = probe_ffmpeg(path)?;
    if info.width == 0 || info.height == 0 {
        bail!("unknown frame size for {}", path.display());
    }
    if s.consecutive {
        return first_frames(path, &info, s.frames);
    }
    let span = if info.duration > 0.0 { info.duration.min(s.max_seconds) } else { s.max_seconds };
    // `fps=n/span` emits one frame per span/n seconds, centred in each slot.
    let rate = (s.frames as f64 / span.max(0.05)).min(if info.fps > 0.0 { info.fps } else { 60.0 });
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-noautorotate", "-t", &format!("{span:.3}"), "-i"])
        .arg(path)
        .args(["-map", "0:v:0", "-vf", &format!("fps={rate:.6}"), "-frames:v", &s.frames.to_string(), "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("running ffmpeg")?;
    let mut raw = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut raw)?;
    child.wait()?;
    let n = (info.width * info.height * 3) as usize;
    let frames: Vec<Rgb> = raw.chunks_exact(n).map(|c| Rgb { w: info.width, h: info.height, data: c.to_vec() }).collect();
    if frames.is_empty() {
        bail!("no frames decoded from {}", path.display());
    }
    Ok(frames)
}

/// The first `n` frames of a clip, in order, at native resolution, decoded
/// by ffmpeg to raw 4:2:0 and converted here (the reference-pipeline rule on
/// macOS, plain BT.601 elsewhere).
pub fn first_frames(path: &Path, info: &VideoInfo, n: usize) -> Result<Vec<Rgb>> {
    let range = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=pix_fmt,color_range", "-of", "csv=p=0"])
        .arg(path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let full = range.contains("yuvj") || range.contains(",pc");
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-noautorotate", "-i"])
        .arg(path)
        .args(["-map", "0:v:0", "-frames:v", &n.to_string(), "-fps_mode", "passthrough", "-f", "rawvideo", "-pix_fmt", if full { "yuvj420p" } else { "yuv420p" }, "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("running ffmpeg")?;
    let mut raw = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut raw)?;
    child.wait()?;
    let (w, h) = (info.width as usize, info.height as usize);
    let len = w * h + 2 * w.div_ceil(2) * h.div_ceil(2);
    let frames: Vec<Rgb> = raw.chunks_exact(len).filter_map(|c| planar_to_rgb(w, h, c, full)).collect();
    if frames.is_empty() {
        bail!("no frames decoded from {}", path.display());
    }
    Ok(frames)
}

#[cfg(target_os = "macos")]
fn planar_to_rgb(w: usize, h: usize, data: &[u8], full: bool) -> Option<Rgb> {
    crate::avf::Yuv::from_planar(w, h, data, full).map(|y| y.to_rgb_opencv())
}

#[cfg(not(target_os = "macos"))]
fn planar_to_rgb(w: usize, h: usize, data: &[u8], full: bool) -> Option<Rgb> {
    let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
    if data.len() < w * h + 2 * cw * ch {
        return None;
    }
    let (u, v) = (&data[w * h..], &data[w * h + cw * ch..]);
    let (ky, rv, gu, gv, bu, y0) = if full { (65536, 91881, 22554, 46802, 116130, 0) } else { (76309, 104597, 25675, 53279, 132201, 16) };
    let mut out = vec![0u8; w * h * 3];
    for yy in 0..h {
        for x in 0..w {
            let l = ky * (data[yy * w + x] as i32 - y0) + 32768;
            let (cb, cr) = (u[(yy / 2) * cw + x / 2] as i32 - 128, v[(yy / 2) * cw + x / 2] as i32 - 128);
            let o = &mut out[(yy * w + x) * 3..][..3];
            o[0] = ((l + rv * cr) >> 16).clamp(0, 255) as u8;
            o[1] = ((l - gu * cb - gv * cr) >> 16).clamp(0, 255) as u8;
            o[2] = ((l + bu * cb) >> 16).clamp(0, 255) as u8;
        }
    }
    Some(Rgb { w: w as u32, h: h as u32, data: out })
}

/// Combine per-frame logits into one clip logit. The mean is the
/// log-likelihood-ratio average, the same rule the strongest frame-level
/// video detectors use.
pub fn aggregate(frame_logits: &[f32]) -> f32 {
    frame_logits.iter().sum::<f32>() / frame_logits.len().max(1) as f32
}

/// Say so when the system decoder gave up on a file (the caller then tries ffmpeg).
#[cfg(target_os = "macos")]
fn note_stall(e: &anyhow::Error) {
    let msg = e.to_string();
    if msg.contains("stalled") {
        eprintln!("warning: {msg}; trying ffmpeg");
    }
}

/// Decode the single frame nearest to time `t` (seconds).
pub fn frame_at(path: &Path, info: &VideoInfo, t: f64) -> Result<Rgb> {
    #[cfg(target_os = "macos")]
    if native() {
        match crate::avf::frame_at(path, t) {
            Ok(f) => return Ok(f),
            Err(e) => note_stall(&e),
        }
    }
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-noautorotate", "-ss", &format!("{t:.3}"), "-i"])
        .arg(path)
        .args(["-map", "0:v:0", "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("running ffmpeg")?;
    let mut raw = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut raw)?;
    child.wait()?;
    let n = (info.width * info.height * 3) as usize;
    if n == 0 || raw.len() < n {
        bail!("no frame decoded from {} at {t:.2}s", path.display());
    }
    raw.truncate(n);
    Ok(Rgb { w: info.width, h: info.height, data: raw })
}

/// A random training frame. With probability `recompress_p` a short segment
/// around the chosen instant is first re-encoded with random H.264 quality and
/// scale, so bitrate and resolution stop being class cues.
pub fn random_frame(path: &Path, rng: &mut crate::rng::Rng, recompress_p: f32, tmp_dir: &Path) -> Result<Rgb> {
    let info = probe(path)?;
    let t = rng.range(0.0, (info.duration - 0.7).max(0.0) as f32) as f64;
    if !rng.chance(recompress_p) {
        return frame_at(path, &info, t);
    }
    let tmp = tmp_dir.join(format!("seg_{:016x}.mp4", rng.u64()));
    let (crf, scale) = (rng.range(20.0, 36.0) as u32, if rng.chance(0.5) { 1.0 } else { rng.range(0.5, 1.0) });
    let vf = format!("scale=trunc(iw*{scale:.3}/2)*2:trunc(ih*{scale:.3}/2)*2");
    let st = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-y", "-ss", &format!("{t:.3}"), "-t", "0.5", "-i"])
        .arg(path)
        .args(["-map", "0:v:0", "-an", "-vf", &vf, "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p", "-crf", &crf.to_string()])
        .arg(&tmp)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("running ffmpeg")?;
    let out = if st.success() {
        probe(&tmp).and_then(|i| {
            // Take a frame past the first keyframe so inter-frame coding is represented.
            let k = rng.range(0.0, (i.duration - 0.1).max(0.0) as f32) as f64;
            frame_at(&tmp, &i, k)
        })
    } else {
        Err(anyhow!("re-encode failed"))
    };
    let _ = std::fs::remove_file(&tmp);
    out.or_else(|_| frame_at(path, &info, t))
}

/// The middle frame of a clip (deterministic; used for validation).
pub fn middle_frame(path: &Path) -> Result<Rgb> {
    let info = probe(path)?;
    frame_at(path, &info, (info.duration / 2.0 - 0.05).max(0.0))
}
