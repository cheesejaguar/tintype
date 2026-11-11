//! In-process video decoding on macOS: AVFoundation hands back decoded
//! 4:2:0 frames, and the YCbCr -> RGB conversion is done here in Rust so the
//! pixels do not depend on any external tool's colour pipeline.
#![cfg(target_os = "macos")]

use crate::image_io::Rgb;
use crate::video::VideoInfo;
use anyhow::{anyhow, bail, Result};
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::AnyObject;
use objc2_av_foundation::{AVAssetReader, AVAssetReaderOutput, AVAssetReaderTrackOutput, AVAssetTrack, AVMediaTypeVideo, AVURLAsset};
use objc2_core_foundation::{CFBoolean, CFString};
use objc2_core_media::{kCMFormatDescriptionExtension_FullRangeVideo, kCMTimePositiveInfinity, CMFormatDescription, CMTime, CMTimeRange};
use objc2_core_video::{
    kCVImageBufferYCbCrMatrixKey, kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_420YpCbCr8BiPlanarFullRange, kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
    CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferGetHeight, CVPixelBufferGetWidth,
    CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};
use std::path::Path;

/// One decoded frame: full-resolution luma and interleaved half-resolution CbCr.
pub struct Yuv {
    pub w: usize,
    pub h: usize,
    y: Vec<u8>,
    uv: Vec<u8>,
    bt709: bool,
    full_range: bool,
}

impl Yuv {
    /// Wrap planar 4:2:0 data (`Y`, then `Cb`, then `Cr`), e.g. from a
    /// fallback decoder.
    pub fn from_planar(w: usize, h: usize, data: &[u8], full_range: bool) -> Option<Self> {
        let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
        if data.len() < w * h + 2 * cw * ch {
            return None;
        }
        let (u, v) = (&data[w * h..][..cw * ch], &data[w * h + cw * ch..][..cw * ch]);
        let uv = u.iter().zip(v).flat_map(|(&a, &b)| [a, b]).collect();
        Some(Self { w, h, y: data[..w * h].to_vec(), uv, bt709: false, full_range })
    }
}

/// C-style division that truncates toward zero.
fn tdiv(a: i64, b: i64) -> i64 {
    a / b
}

impl Yuv {
    /// YCbCr to RGB. Chroma is replicated over its 2x2 block and the matrix
    /// follows the stream's tag (BT.601 when untagged), matching the
    /// conversion the training frames went through.
    pub fn to_rgb(&self) -> Rgb {
        // 16.16 fixed point: (luma gain, Cr->R, Cb->G, Cr->G, Cb->B, luma offset).
        let (ky, rv, gu, gv, bu, y0): (i32, i32, i32, i32, i32, i32) = match (self.full_range, self.bt709) {
            (false, false) => (76309, 104597, 25675, 53279, 132201, 16),
            (false, true) => (76309, 117489, 13975, 34925, 138438, 16),
            (true, false) => (65536, 91881, 22554, 46802, 116130, 0),
            (true, true) => (65536, 103206, 12276, 30679, 121609, 0),
        };
        self.convert(|y, u, v| {
            let l = ky * (y - y0) + 32768;
            [(l + rv * v) >> 16, (l - gu * u - gv * v) >> 16, (l + bu * u) >> 16]
        })
    }

    /// The conversion OpenCV's FFmpeg backend applies (swscale's table path:
    /// BT.601 whatever the tag, chroma contribution quantised to whole luma
    /// steps). Reproduced exactly so that reference pipelines built on
    /// `cv2.VideoCapture` see the pixels they were developed with.
    pub fn to_rgb_opencv(&self) -> Rgb {
        let cy: i64 = if self.full_range { 65536 } else { 76309 };
        let scale = |c: i64| if self.full_range { c * 224 / 255 } else { c };
        let inc = |c: i64, sign: i64| tdiv(((sign * scale(c)) << 16) + 0x8000, cy);
        let (crv, cbu, cgu, cgv) = (inc(104597, 1), inc(132201, 1), inc(25675, -1), inc(53279, -1));
        // Per-chroma-value luma shifts.
        let tab = |k: i64| -> Vec<i32> { (0..256).map(|c| (((c * k) >> 16) - (k >> 9)) as i32).collect() };
        let (t_rv, t_bu, t_gu, t_gv) = (tab(crv), tab(cbu), tab(cgu), tab(cgv));
        // Luma index -> output level.
        let luma: Vec<u8> = (-512i64..768)
            .map(|l| if self.full_range { l.clamp(0, 255) as u8 } else { ((cy * (l - 16) - 83936) >> 16).clamp(0, 255) as u8 })
            .collect();
        let lut = |l: i32| luma[(l + 512).clamp(0, 1279) as usize] as i32;
        self.convert_raw(|y, u, v| [lut(y + t_rv[v as usize]), lut(y + t_gu[u as usize] + t_gv[v as usize]), lut(y + t_bu[u as usize])])
    }

    /// `f(y, cb - 128, cr - 128)` per pixel.
    fn convert(&self, f: impl Fn(i32, i32, i32) -> [i32; 3]) -> Rgb {
        self.convert_raw(|y, u, v| f(y, u - 128, v - 128))
    }

    /// `f(y, cb, cr)` per pixel with raw sample values.
    fn convert_raw(&self, f: impl Fn(i32, i32, i32) -> [i32; 3]) -> Rgb {
        let cw = self.w.div_ceil(2);
        let mut data = vec![0u8; self.w * self.h * 3];
        for yy in 0..self.h {
            let yrow = &self.y[yy * self.w..][..self.w];
            let crow = &self.uv[(yy / 2) * cw * 2..][..cw * 2];
            let out = &mut data[yy * self.w * 3..][..self.w * 3];
            for x in 0..self.w {
                let c = f(yrow[x] as i32, crow[(x / 2) * 2] as i32, crow[(x / 2) * 2 + 1] as i32);
                out[x * 3] = c[0].clamp(0, 255) as u8;
                out[x * 3 + 1] = c[1].clamp(0, 255) as u8;
                out[x * 3 + 2] = c[2].clamp(0, 255) as u8;
            }
        }
        Rgb { w: self.w as u32, h: self.h as u32, data }
    }
}

struct Reader {
    reader: Retained<AVAssetReader>,
    output: Retained<AVAssetReaderTrackOutput>,
    info: VideoInfo,
    full_range: bool,
}

fn open(path: &Path, start: Option<f64>) -> Result<Reader> {
    unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
        let asset = AVURLAsset::URLAssetWithURL_options(&url, None);
        let media = AVMediaTypeVideo.ok_or_else(|| anyhow!("AVFoundation unavailable"))?;
        #[allow(deprecated)]
        let tracks = asset.tracksWithMediaType(media);
        let track: Retained<AVAssetTrack> = tracks.firstObject().ok_or_else(|| anyhow!("no video track in {}", path.display()))?;
        let size = track.naturalSize();
        #[allow(deprecated)]
        let duration = asset.duration().seconds();
        let info = VideoInfo {
            width: size.width.round() as u32,
            height: size.height.round() as u32,
            duration: if duration.is_finite() { duration } else { 0.0 },
            fps: track.nominalFrameRate() as f64,
        };
        let reader = AVAssetReader::assetReaderWithAsset_error(&asset).map_err(|e| anyhow!("cannot read {}: {}", path.display(), e.localizedDescription()))?;
        // Ask for the stream's own range so the decoder does no range conversion.
        let descs = track.formatDescriptions();
        let full_range = descs
            .firstObject()
            .and_then(|d| {
                let desc: &CMFormatDescription = &*(Retained::as_ptr(&d) as *const CMFormatDescription);
                desc.extension(kCMFormatDescriptionExtension_FullRangeVideo)
            })
            .and_then(|v| v.downcast_ref::<CFBoolean>().map(|b| b.as_bool()))
            .unwrap_or(false);
        let key: &NSString = &*(kCVPixelBufferPixelFormatTypeKey as *const CFString as *const NSString);
        let fmt = NSNumber::new_u32(if full_range { kCVPixelFormatType_420YpCbCr8BiPlanarFullRange } else { kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange });
        let fmt_obj: &AnyObject = &fmt;
        let settings: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(&[key], &[fmt_obj]);
        let output = AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(&track, Some(&settings));
        output.setAlwaysCopiesSampleData(false);
        let out_ref: &AVAssetReaderOutput = &output;
        reader.addOutput(out_ref);
        if let Some(t) = start {
            reader.setTimeRange(CMTimeRange { start: CMTime::with_seconds(t.max(0.0), 600_000), duration: kCMTimePositiveInfinity });
        }
        if !reader.startReading() {
            bail!("cannot decode {}", path.display());
        }
        Ok(Reader { reader, output, info, full_range })
    }
}

impl Reader {
    /// Next frame in presentation order with its timestamp in seconds.
    fn next(&mut self) -> Option<(f64, Yuv)> {
        autoreleasepool(|_| unsafe {
            loop {
                let sb = self.output.copyNextSampleBuffer()?;
                let Some(pb) = sb.image_buffer() else { continue };
                let pts = sb.presentation_time_stamp().seconds();
                let (w, h) = (CVPixelBufferGetWidth(&pb), CVPixelBufferGetHeight(&pb));
                if CVPixelBufferLockBaseAddress(&pb, CVPixelBufferLockFlags::ReadOnly) != 0 {
                    continue;
                }
                let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
                let mut y = vec![0u8; w * h];
                let mut uv = vec![0u8; cw * ch * 2];
                let copy = |plane: usize, rows: usize, row_len: usize, dst: &mut [u8]| {
                    let base = CVPixelBufferGetBaseAddressOfPlane(&pb, plane) as *const u8;
                    let stride = CVPixelBufferGetBytesPerRowOfPlane(&pb, plane);
                    if base.is_null() {
                        return false;
                    }
                    for r in 0..rows {
                        let src = std::slice::from_raw_parts(base.add(r * stride), row_len);
                        dst[r * row_len..][..row_len].copy_from_slice(src);
                    }
                    true
                };
                let ok = copy(0, h, w, &mut y) && copy(1, ch, cw * 2, &mut uv);
                let matrix = pb.attachment(kCVImageBufferYCbCrMatrixKey, std::ptr::null_mut());
                let bt709 = matrix.and_then(|m| m.downcast_ref::<CFString>().map(|s| s.to_string().contains("709"))).unwrap_or(false);
                CVPixelBufferUnlockBaseAddress(&pb, CVPixelBufferLockFlags::ReadOnly);
                if ok {
                    return Some((pts, Yuv { w, h, y, uv, bt709, full_range: self.full_range }));
                }
            }
        })
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe { self.reader.cancelReading() }
    }
}

// Every entry point runs inside its own autorelease pool: the readers hold
// hardware decoder sessions, and on worker threads nothing else would drain
// the objects AVFoundation autoreleases while opening a file.

/// Seconds a decode may take before it is abandoned.
const DEADLINE: u64 = 30;

/// Run a decode on its own thread and give up after [`DEADLINE`]. On some
/// files `copyNextSampleBuffer` never returns; the caller then gets an error
/// (and falls back to another decoder) and the stuck thread is left behind.
fn with_deadline<T: Send + 'static>(path: &Path, f: impl FnOnce(&Path) -> Result<T> + Send + 'static) -> Result<T> {
    let (tx, rx) = std::sync::mpsc::channel();
    let owned = path.to_path_buf();
    std::thread::Builder::new().name("avf-decode".into()).spawn(move || {
        let _ = tx.send(autoreleasepool(|_| f(&owned)));
    })?;
    match rx.recv_timeout(std::time::Duration::from_secs(DEADLINE)) {
        Ok(r) => r,
        Err(_) => bail!("the system decoder stalled on {}", path.display()),
    }
}

pub fn probe(path: &Path) -> Result<VideoInfo> {
    with_deadline(path, |p| Ok(open(p, None)?.info))
}

/// `n` frames spread evenly over the first `max_seconds` of the clip: frame
/// `k` is the one nearest to `k * span / n` after the first frame.
pub fn frames_uniform(path: &Path, n: usize, max_seconds: f64) -> Result<Vec<Rgb>> {
    with_deadline(path, move |p| frames_uniform_inner(p, n, max_seconds))
}

fn frames_uniform_inner(path: &Path, n: usize, max_seconds: f64) -> Result<Vec<Rgb>> {
    let mut r = open(path, None)?;
    let span = if r.info.duration > 0.0 { r.info.duration.min(max_seconds) } else { max_seconds };
    let mut out: Vec<Rgb> = Vec::with_capacity(n);
    let mut prev: Option<(f64, Yuv)> = None;
    let mut t0 = None;
    while out.len() < n {
        let Some((pts, f)) = r.next() else { break };
        let start = *t0.get_or_insert(pts);
        while out.len() < n {
            let target = start + out.len() as f64 * span / n as f64;
            if pts < target {
                break;
            }
            // The target lies between the previous frame and this one.
            let use_prev = prev.as_ref().is_some_and(|(pp, _)| target - pp < pts - target);
            out.push(if use_prev { prev.as_ref().unwrap().1.to_rgb() } else { f.to_rgb() });
        }
        prev = Some((pts, f));
    }
    // Targets past the last decoded frame reuse it.
    if let Some((_, last)) = &prev {
        if out.is_empty() {
            out.push(last.to_rgb());
        }
    }
    if out.is_empty() {
        bail!("no frames decoded from {}", path.display());
    }
    Ok(out)
}

/// The first `n` frames in presentation order, with the colour conversion
/// of OpenCV's reader (see [`Yuv::to_rgb_opencv`]).
pub fn frames_first(path: &Path, n: usize) -> Result<Vec<Rgb>> {
    with_deadline(path, move |p| frames_first_inner(p, n))
}

fn frames_first_inner(path: &Path, n: usize) -> Result<Vec<Rgb>> {
    let mut r = open(path, None)?;
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let Some((_, f)) = r.next() else { break };
        out.push(f.to_rgb_opencv());
    }
    if out.is_empty() {
        bail!("no frames decoded from {}", path.display());
    }
    Ok(out)
}

/// The first frame at or after `t` seconds.
pub fn frame_at(path: &Path, t: f64) -> Result<Rgb> {
    with_deadline(path, move |p| {
        let mut r = open(p, Some(t))?;
        match r.next() {
            Some((_, f)) => Ok(f.to_rgb()),
            None => bail!("no frame at {t:.2}s in {}", p.display()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(y: u8, u: u8, v: u8, full_range: bool, bt709: bool) -> Yuv {
        Yuv { w: 4, h: 2, y: vec![y; 8], uv: [u, v].repeat(2), bt709, full_range }
    }

    #[test]
    fn grey_levels_convert_exactly() {
        // Video range: 16 is black, 235 is white, neutral chroma stays grey.
        assert_eq!(&flat(16, 128, 128, false, false).to_rgb().data[..3], &[0, 0, 0]);
        assert_eq!(&flat(235, 128, 128, false, false).to_rgb().data[..3], &[255, 255, 255]);
        assert_eq!(&flat(126, 128, 128, false, true).to_rgb().data[..3], &[128, 128, 128]);
        // Full range passes luma through.
        assert_eq!(&flat(77, 128, 128, true, false).to_rgb().data[..3], &[77, 77, 77]);
    }

    #[test]
    fn primaries_have_the_right_sign() {
        // High Cr pushes red up and green down; high Cb pushes blue up.
        let r = flat(120, 128, 200, false, false).to_rgb();
        assert!(r.data[0] > 200 && r.data[1] < 80, "{:?}", &r.data[..3]);
        let b = flat(120, 200, 128, false, false).to_rgb();
        assert!(b.data[2] > 240 && b.data[0] == b.data[0].min(130), "{:?}", &b.data[..3]);
    }

    #[test]
    fn opencv_conversion_matches_recorded_values() {
        // Values read from frames decoded by cv2.VideoCapture.
        for (y, expect) in [(16u8, 0u8), (17, 0), (100, 96), (101, 97), (200, 212)] {
            assert_eq!(flat(y, 128, 128, false, false).to_rgb_opencv().data[0], expect, "Y={y}");
        }
        // Full range: luma plus a whole-step chroma shift.
        assert_eq!(&flat(90, 128, 128, true, false).to_rgb_opencv().data[..3], &[90, 90, 90]);
    }
}
