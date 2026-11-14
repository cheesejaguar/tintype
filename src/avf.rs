//! In-process video decoding on macOS: AVFoundation hands back decoded
//! 4:2:0 frames, and the YCbCr -> RGB conversion is done here in Rust so the
//! pixels do not depend on any external tool's colour pipeline.
#![cfg(target_os = "macos")]

use crate::image_io::Rgb;
use crate::video::VideoInfo;
use crate::yuv::{self, Source, Yuv};
use anyhow::{anyhow, bail, ensure, Result};
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::AnyObject;
use objc2_av_foundation::{
    AVAssetReader, AVAssetReaderOutput, AVAssetReaderTrackOutput, AVAssetTrack, AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor, AVFileTypeMPEG4,
    AVMediaTypeVideo, AVURLAsset, AVVideoAverageBitRateKey, AVVideoCodecKey, AVVideoCodecTypeH264, AVVideoCompressionPropertiesKey, AVVideoHeightKey, AVVideoWidthKey,
};
use objc2_core_foundation::{CFBoolean, CFString};
use objc2_core_media::{kCMFormatDescriptionExtension_FullRangeVideo, kCMTimePositiveInfinity, CMFormatDescription, CMTime, CMTimeRange};
use objc2_core_video::{
    kCVImageBufferYCbCrMatrixKey, kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_420YpCbCr8BiPlanarFullRange, kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
    kCVPixelFormatType_32BGRA, CVPixelBuffer, CVPixelBufferCreate, CVPixelBufferGetBaseAddress, CVPixelBufferGetBaseAddressOfPlane, CVPixelBufferGetBytesPerRow,
    CVPixelBufferGetBytesPerRowOfPlane, CVPixelBufferGetHeight, CVPixelBufferGetWidth,
    CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};
use std::path::Path;

struct Reader {
    reader: Retained<AVAssetReader>,
    output: Retained<AVAssetReaderTrackOutput>,
    info: VideoInfo,
    full_range: bool,
    /// The retagged copy being read, when the original could not be opened as it was.
    _temp: Option<tempfile::NamedTempFile>,
}

/// Byte offset of the sample-entry type in the first sample description that
/// says `hev1`, found by walking the MP4 box tree.
fn hev1_entry_offset(path: &Path) -> Result<Option<u64>> {
    use std::io::{Read, Seek, SeekFrom};
    fn walk(f: &mut std::fs::File, start: u64, end: u64, depth: usize) -> Result<Option<u64>> {
        let mut pos = start;
        while end - pos >= 8 {
            let mut h = [0u8; 16];
            f.seek(SeekFrom::Start(pos))?;
            f.read_exact(&mut h[..8])?;
            let (mut size, mut header) = (u32::from_be_bytes([h[0], h[1], h[2], h[3]]) as u64, 8u64);
            if size == 1 {
                f.read_exact(&mut h[8..])?;
                size = u64::from_be_bytes([h[8], h[9], h[10], h[11], h[12], h[13], h[14], h[15]]);
                header = 16;
            } else if size == 0 {
                size = end - pos;
            }
            if size < header || size > end - pos {
                break;
            }
            match &h[4..8] {
                b"moov" | b"trak" | b"mdia" | b"minf" | b"stbl" if depth < 8 => {
                    if let Some(at) = walk(f, pos + header, pos + size, depth + 1)? {
                        return Ok(Some(at));
                    }
                }
                b"stsd" => {
                    // Version and flags (4 bytes), entry count (4), then each entry: size (4), type (4).
                    if size - header >= 16 {
                        let entry = pos + header + 8;
                        let mut kind = [0u8; 4];
                        f.seek(SeekFrom::Start(entry + 4))?;
                        f.read_exact(&mut kind)?;
                        if &kind == b"hev1" {
                            return Ok(Some(entry + 4));
                        }
                    }
                }
                _ => {}
            }
            pos += size;
        }
        Ok(None)
    }
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    walk(&mut f, 0, len, 0)
}

/// AVFoundation opens HEVC only when the sample description is tagged `hvc1`;
/// many encoders write `hev1` for the same stream. For such a file, a
/// temporary copy with the four-byte tag changed.
fn retagged_copy(path: &Path) -> Result<Option<tempfile::NamedTempFile>> {
    use std::os::unix::fs::FileExt;
    let Some(at) = hev1_entry_offset(path)? else { return Ok(None) };
    let mut tmp = tempfile::Builder::new().prefix("tintype-").suffix(".mp4").tempfile()?;
    std::io::copy(&mut std::fs::File::open(path)?, tmp.as_file_mut())?;
    tmp.as_file().write_all_at(b"hvc1", at)?;
    Ok(Some(tmp))
}

fn open(path: &Path, start: Option<f64>) -> Result<Reader> {
    match retagged_copy(path).unwrap_or(None) {
        Some(tmp) => {
            let mut r = open_file(tmp.path(), start)?;
            r._temp = Some(tmp);
            Ok(r)
        }
        None => open_file(path, start),
    }
}

fn open_file(path: &Path, start: Option<f64>) -> Result<Reader> {
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
        Ok(Reader { reader, output, info, full_range, _temp: None })
    }
}

impl Source for Reader {
    fn duration(&self) -> f64 {
        self.info.duration
    }

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
                    return Some((pts, Yuv::new(w, h, y, uv, bt709, self.full_range)));
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

/// Stalled decodes one process may leave behind before it stops starting more.
const MAX_STALLED: usize = 8;

/// Decode threads that were given up on and have not come back.
static STALLED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Run a decode on its own thread and give up after [`DEADLINE`]. On some
/// files `copyNextSampleBuffer` never returns; the caller then gets an error
/// and the stuck thread is left behind, since a thread cannot be killed. Such
/// threads are counted, and once [`MAX_STALLED`] of them are outstanding no
/// further system decode is started in this process: a batch of files built
/// to stall the decoder costs a bounded number of threads, not one per file.
fn with_deadline<T: Send + 'static>(path: &Path, f: impl FnOnce(&Path) -> Result<T> + Send + 'static) -> Result<T> {
    use std::sync::atomic::Ordering::SeqCst;
    #[derive(PartialEq)]
    enum State {
        Running,
        Abandoned,
        Done,
    }
    let stalled = STALLED.load(SeqCst);
    ensure!(stalled < MAX_STALLED, "the system decoder has stalled on {stalled} files in this run; not starting it on {}", path.display());
    let (tx, rx) = std::sync::mpsc::channel();
    let owned = path.to_path_buf();
    let state = std::sync::Arc::new(std::sync::Mutex::new(State::Running));
    let theirs = state.clone();
    std::thread::Builder::new().name("avf-decode".into()).spawn(move || {
        let r = autoreleasepool(|_| f(&owned));
        let mut s = theirs.lock().unwrap();
        if *s == State::Abandoned {
            STALLED.fetch_sub(1, SeqCst);
        }
        *s = State::Done;
        let _ = tx.send(r);
    })?;
    match rx.recv_timeout(std::time::Duration::from_secs(DEADLINE)) {
        Ok(r) => r,
        Err(_) => {
            let mut s = state.lock().unwrap();
            if *s == State::Done {
                // It finished as the deadline passed: the result is on its way.
                drop(s);
                return rx.recv().map_err(|_| anyhow!("the system decoder failed on {}", path.display()))?;
            }
            *s = State::Abandoned;
            STALLED.fetch_add(1, SeqCst);
            bail!("the system decoder stalled on {}", path.display())
        }
    }
}

pub fn probe(path: &Path) -> Result<VideoInfo> {
    with_deadline(path, |p| Ok(open(p, None)?.info))
}

/// `n` frames spread evenly over the first `max_seconds` of the clip: frame
/// `k` is the one nearest to `k * span / n` after the first frame.
pub fn frames_uniform(path: &Path, n: usize, max_seconds: f64) -> Result<Vec<Rgb>> {
    frames_uniform_range(path, n, max_seconds, 0, n)
}

/// Frames `from..to` of the `n` that [`frames_uniform`] picks: the same
/// frames, but decoding stops once the last one wanted has been reached, and
/// frames before `from` are decoded without being converted.
pub fn frames_uniform_range(path: &Path, n: usize, max_seconds: f64, from: usize, to: usize) -> Result<Vec<Rgb>> {
    with_deadline(path, move |p| yuv::uniform_range(&mut open(p, None)?, n, max_seconds, from, to, p))
}

/// As [`frames_uniform_range`], with the frames left as decoded.
pub fn frames_uniform_range_unconverted(path: &Path, n: usize, max_seconds: f64, from: usize, to: usize) -> Result<Vec<Yuv>> {
    with_deadline(path, move |p| yuv::uniform_range_with(&mut open(p, None)?, n, max_seconds, from, to, p, Yuv::clone))
}

/// The first `n` frames in presentation order, with the colour conversion
/// of OpenCV's reader (see [`Yuv::to_rgb_opencv`]).
pub fn frames_first(path: &Path, n: usize) -> Result<Vec<Rgb>> {
    with_deadline(path, move |p| frames_first_inner(p, n))
}

fn frames_first_inner(path: &Path, n: usize) -> Result<Vec<Rgb>> {
    yuv::first(&mut open(path, None)?, n, true, path)
}

/// The first `n` frames in presentation order, with the standard colour
/// conversion (the one training and scoring use).
pub fn frames_first_plain(path: &Path, n: usize) -> Result<Vec<Rgb>> {
    with_deadline(path, move |p| yuv::first(&mut open(p, None)?, n, false, p))
}

/// Re-encode up to `frames` frames starting at `start` seconds as H.264 into
/// `out` (an `.mp4` path), scaled by `scale` and at `bits_per_pixel` average
/// bitrate. Returns the number of frames written.
pub fn recompress(path: &Path, start: f64, frames: usize, scale: f64, bits_per_pixel: f64, out: &Path) -> Result<usize> {
    let out = out.to_path_buf();
    with_deadline(path, move |p| unsafe {
        let mut r = open(p, Some(start))?;
        let fps = if r.info.fps > 0.0 { r.info.fps } else { 25.0 };
        let mut src = Vec::with_capacity(frames);
        while src.len() < frames {
            let Some((_, f)) = r.next() else { break };
            src.push(f.to_rgb());
        }
        drop(r);
        if src.is_empty() {
            bail!("no frames decoded from {}", p.display());
        }
        // Even dimensions, as 4:2:0 needs.
        let even = |v: f64| ((v / 2.0).floor() as u32 * 2).max(16);
        let (w, h) = (even(src[0].w as f64 * scale), even(src[0].h as f64 * scale));
        let _ = std::fs::remove_file(&out);
        let url = NSURL::fileURLWithPath(&NSString::from_str(&out.to_string_lossy()));
        let (file_type, media) = (AVFileTypeMPEG4.ok_or_else(|| anyhow!("AVFoundation unavailable"))?, AVMediaTypeVideo.ok_or_else(|| anyhow!("AVFoundation unavailable"))?);
        let writer = AVAssetWriter::assetWriterWithURL_fileType_error(&url, file_type).map_err(|e| anyhow!("cannot write {}: {}", out.display(), e.localizedDescription()))?;
        let key = |k: Option<&'static NSString>| k.ok_or_else(|| anyhow!("AVFoundation unavailable"));
        let bitrate = NSNumber::new_f64((bits_per_pixel * w as f64 * h as f64 * fps).max(20_000.0).round());
        let compression: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(&[key(AVVideoAverageBitRateKey)?], &[&*bitrate as &AnyObject]);
        let codec: &NSString = AVVideoCodecTypeH264.ok_or_else(|| anyhow!("AVFoundation unavailable"))?;
        let (nw, nh) = (NSNumber::new_u32(w), NSNumber::new_u32(h));
        let settings: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(
            &[key(AVVideoCodecKey)?, key(AVVideoWidthKey)?, key(AVVideoHeightKey)?, key(AVVideoCompressionPropertiesKey)?],
            &[codec as &AnyObject, &*nw as &AnyObject, &*nh as &AnyObject, &*compression as &AnyObject],
        );
        let input = AVAssetWriterInput::assetWriterInputWithMediaType_outputSettings(media, Some(&settings));
        input.setExpectsMediaDataInRealTime(false);
        let adaptor = AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(&input, None);
        writer.addInput(&input);
        if !writer.startWriting() {
            bail!("cannot start encoding {}", out.display());
        }
        writer.startSessionAtSourceTime(CMTime::with_seconds(0.0, 600_000));
        let mut written = 0;
        for (k, frame) in src.iter().enumerate() {
            let frame = if (frame.w, frame.h) == (w, h) { frame.crop(0, 0, w, h) } else { frame.resize(w, h, crate::image_io::Filter::Bilinear) };
            let mut raw: *mut CVPixelBuffer = std::ptr::null_mut();
            if CVPixelBufferCreate(None, w as usize, h as usize, kCVPixelFormatType_32BGRA, None, std::ptr::NonNull::from(&mut raw)) != 0 {
                bail!("cannot allocate a frame buffer");
            }
            let Some(raw) = std::ptr::NonNull::new(raw) else { bail!("cannot allocate a frame buffer") };
            let pb = objc2_core_foundation::CFRetained::from_raw(raw);
            CVPixelBufferLockBaseAddress(&pb, CVPixelBufferLockFlags::empty());
            let (base, stride) = (CVPixelBufferGetBaseAddress(&pb) as *mut u8, CVPixelBufferGetBytesPerRow(&pb));
            for y in 0..h as usize {
                let row = std::slice::from_raw_parts_mut(base.add(y * stride), w as usize * 4);
                for (px, rgb) in row.chunks_exact_mut(4).zip(frame.data[y * w as usize * 3..].chunks_exact(3)) {
                    px.copy_from_slice(&[rgb[2], rgb[1], rgb[0], 255]);
                }
            }
            CVPixelBufferUnlockBaseAddress(&pb, CVPixelBufferLockFlags::empty());
            // The encoder takes frames at its own pace.
            let mut waited = 0;
            while !input.isReadyForMoreMediaData() && waited < 5000 {
                std::thread::sleep(std::time::Duration::from_millis(1));
                waited += 1;
            }
            if !adaptor.appendPixelBuffer_withPresentationTime(&pb, CMTime::with_seconds(k as f64 / fps, 600_000)) {
                break;
            }
            written += 1;
        }
        input.markAsFinished();
        #[allow(deprecated)]
        let ok = writer.finishWriting();
        if !ok || written == 0 {
            bail!("encoding {} failed", out.display());
        }
        Ok(written)
    })
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

/// How the pure-Rust reader's frames compare with the system decoder's.
#[derive(Debug, Default, Clone)]
pub struct Comparison {
    /// Frames compared (the shorter of the two streams, up to the limit).
    pub frames: usize,
    /// Frames each reader produced within the limit: (system, pure Rust).
    pub produced: (usize, usize),
    /// Frames whose samples and colour tags are identical.
    pub identical: usize,
    /// Frames whose colour tags differ.
    pub tag_mismatches: usize,
    /// Largest difference between samples, over frames of equal size.
    pub max_sample_difference: u8,
    /// Largest difference between presentation times, seconds.
    pub max_time_difference: f64,
    /// (duration, system) and (duration, pure Rust).
    pub durations: (f64, f64),
    /// `(bt709, full_range)` of the first frame from each: (system, pure Rust).
    pub tags: Option<((bool, bool), (bool, bool))>,
    /// Matrix coefficients and range the file states, and the frame size.
    pub declared: (Option<u16>, Option<bool>),
    pub size: (usize, usize),
}

/// Decode up to `limit` frames with both readers and compare them.
pub fn compare_with_pure_rust(path: &Path, limit: usize) -> Result<Comparison> {
    with_deadline(path, move |p| {
        let mut a = open(p, None)?;
        let mut b = crate::swdec::open(p, None)?;
        let mut c = Comparison { durations: (a.duration(), b.duration()), declared: b.declared(), ..Default::default() };
        loop {
            if c.frames >= limit {
                break;
            }
            let (fa, fb) = (a.next(), Source::next(&mut b));
            c.produced.0 += fa.is_some() as usize;
            c.produced.1 += fb.is_some() as usize;
            let (Some((ta, fa)), Some((tb, fb))) = (fa, fb) else { break };
            c.frames += 1;
            c.max_time_difference = c.max_time_difference.max((ta - tb).abs());
            if fa.same_as(&fb) {
                c.identical += 1;
            }
            c.tags.get_or_insert((fa.tags(), fb.tags()));
            c.size = (fa.w, fa.h);
            if fa.tags() != fb.tags() {
                c.tag_mismatches += 1;
            }
            c.max_sample_difference = c.max_sample_difference.max(fa.max_difference(&fb).unwrap_or(255));
        }
        Ok(c)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decoder-continuation benchmark for the system decoder. Needs real clips:
    /// `TINTYPE_TEST_VIDEOS=clips.txt cargo test --release --lib avf -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn reading_on_against_reopening_for_the_second_half() {
        yuv::report_continuation("system decoder", |path| with_deadline(path, |p| yuv::continuation_times(|| open(p, None), 16, 30.0)));
    }
}
