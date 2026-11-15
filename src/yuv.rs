//! Decoded video frames and how they are picked from a clip, shared by the
//! decoders: the system one on macOS ([`crate::avf`]) and the pure-Rust one
//! ([`crate::swdec`]). The YCbCr -> RGB conversion is done here so the pixels
//! do not depend on which decoder produced the frame.
use crate::image_io::Rgb;
use anyhow::{bail, Result};
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

/// Cells along one side of [`Yuv::thumb`].
pub const THUMB: usize = 16;

/// C-style division that truncates toward zero.
fn tdiv(a: i64, b: i64) -> i64 {
    a / b
}

impl Yuv {
    /// `y` is `w * h` luma samples; `uv` is interleaved Cb, Cr at half
    /// resolution, `ceil(w / 2) * 2` bytes per row for `ceil(h / 2)` rows.
    pub fn new(w: usize, h: usize, y: Vec<u8>, uv: Vec<u8>, bt709: bool, full_range: bool) -> Self {
        Self { w, h, y, uv, bt709, full_range }
    }

    /// True when both frames hold the same samples and colour tags.
    pub fn same_as(&self, other: &Yuv) -> bool {
        (self.w, self.h, self.bt709, self.full_range) == (other.w, other.h, other.bt709, other.full_range) && self.y == other.y && self.uv == other.uv
    }

    /// `(bt709, full_range)`.
    pub fn tags(&self) -> (bool, bool) {
        (self.bt709, self.full_range)
    }

    /// Largest absolute difference between the two frames' samples, or
    /// `None` when their sizes differ.
    pub fn max_difference(&self, other: &Yuv) -> Option<u8> {
        if (self.w, self.h) != (other.w, other.h) {
            return None;
        }
        self.y.iter().zip(&other.y).chain(self.uv.iter().zip(&other.uv)).map(|(a, b)| a.abs_diff(*b)).max()
    }

    /// Mean luma of each cell of a `THUMB` x `THUMB` grid, in 8-bit levels,
    /// from at most about 128 samples a side: cheap enough to take of every
    /// frame read, and two frames with the same samples give the same values.
    pub fn thumb(&self) -> [f32; THUMB * THUMB] {
        let (sx, sy) = ((self.w / 128).max(1), (self.h / 128).max(1));
        let (mut sum, mut count) = ([0u32; THUMB * THUMB], [0u32; THUMB * THUMB]);
        for y in (0..self.h).step_by(sy) {
            let row = &self.y[y * self.w..][..self.w];
            let by = y * THUMB / self.h;
            for x in (0..self.w).step_by(sx) {
                let cell = by * THUMB + x * THUMB / self.w;
                sum[cell] += row[x] as u32;
                count[cell] += 1;
            }
        }
        std::array::from_fn(|i| sum[i] as f32 / count[i].max(1) as f32)
    }

    /// The sample-wise mean of frames of one size (rounded to nearest): what
    /// a shutter open across all of them would have recorded, to the extent
    /// 8-bit video samples allow. Colour tags are the first frame's.
    pub fn mean_of(frames: &[&Yuv]) -> Yuv {
        let first = frames[0];
        let same: Vec<&&Yuv> = frames.iter().filter(|f| (f.w, f.h) == (first.w, first.h)).collect();
        let n = same.len() as u32;
        let avg = |plane: fn(&Yuv) -> &Vec<u8>| -> Vec<u8> { (0..plane(first).len()).map(|i| ((same.iter().map(|f| plane(f)[i] as u32).sum::<u32>() + n / 2) / n) as u8).collect() };
        Yuv { w: first.w, h: first.h, y: avg(|f| &f.y), uv: avg(|f| &f.uv), bt709: first.bt709, full_range: first.full_range }
    }

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

/// A decoder positioned in a clip.
pub trait Source {
    /// Clip duration in seconds, 0 when unknown.
    fn duration(&self) -> f64;
    /// Next frame in presentation order with its timestamp in seconds.
    fn next(&mut self) -> Option<(f64, Yuv)>;
}

/// `n` frames spread evenly over the first `max_seconds` of the clip: frame
/// `k` is the one nearest to `k * span / n` after the first frame.
pub fn uniform(r: &mut impl Source, n: usize, max_seconds: f64, path: &Path) -> Result<Vec<Rgb>> {
    uniform_range(r, n, max_seconds, 0, n, path)
}

/// Frames `from..to` of the `n` that [`uniform`] picks: the same frames, but
/// reading stops once the last one wanted has been reached, and frames
/// before `from` are read without being converted. A clip that ends before
/// frame `from` gives an empty list when `from > 0`.
pub fn uniform_range(r: &mut impl Source, n: usize, max_seconds: f64, from: usize, to: usize, path: &Path) -> Result<Vec<Rgb>> {
    let to = to.min(n);
    let span = crate::video::span(r.duration(), max_seconds);
    let mut out: Vec<Rgb> = Vec::with_capacity(to.saturating_sub(from));
    // Index of the next of the `n` targets.
    let mut k = 0;
    let mut prev: Option<(f64, Yuv)> = None;
    let mut t0 = None;
    while k < to {
        let Some((pts, f)) = r.next() else { break };
        let start = *t0.get_or_insert(pts);
        while k < to {
            let target = start + k as f64 * span / n as f64;
            if pts < target {
                break;
            }
            if k >= from {
                // The target lies between the previous frame and this one.
                let use_prev = prev.as_ref().is_some_and(|(pp, _)| target - pp < pts - target);
                out.push(if use_prev { prev.as_ref().unwrap().1.to_rgb() } else { f.to_rgb() });
            }
            k += 1;
        }
        prev = Some((pts, f));
    }
    // A clip too short to reach any target still yields its last frame.
    if let Some((_, last)) = &prev {
        if k == 0 && from == 0 {
            out.push(last.to_rgb());
        }
    }
    if out.is_empty() && from == 0 {
        bail!("no frames decoded from {}", path.display());
    }
    Ok(out)
}

/// One sampled frame and where it sits in the stream.
pub struct Timed {
    /// Presentation time in seconds, as the decoder reports it.
    pub pts: f64,
    /// Position among the frames read, from 0, in presentation order. Two
    /// samples with the same index are the same decoded frame.
    pub index: usize,
    pub rgb: Rgb,
}

/// What every frame read on the way to the samples looked like next to the
/// one before it: the evidence for cuts, repeated frames, motion and cadence.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trace {
    /// Presentation time of each frame read, in the order read.
    pub pts: Vec<f64>,
    /// Mean absolute difference between the luma thumbnails ([`Yuv::thumb`])
    /// of frame `i` and frame `i - 1`, in 8-bit levels; 0 for the first frame.
    pub diff: Vec<f32>,
}

/// The frames [`uniform`] picks, each with its presentation time and its
/// position in the stream, and a [`Trace`] of every frame read to get there.
///
/// With `follow > 0` the `n` samples are `n / 2` pairs instead: the frames
/// [`uniform`] would pick for `n / 2` samples, each followed by the frame
/// `follow` frames after it in the stream (1 = the very next frame). A pair
/// whose second frame lies past the end of the clip is left out, so the
/// result always holds whole pairs, first frame then second.
///
/// With `blur > 1` every sample is the mean of itself and the `blur - 1`
/// frames read before it ([`Yuv::mean_of`]): a synthetic long exposure, for
/// checking that motion blur on authentic clips is not what a head reads.
pub fn uniform_timed(r: &mut impl Source, n: usize, max_seconds: f64, follow: usize, blur: usize, path: &Path) -> Result<(Vec<Timed>, Trace)> {
    let span = crate::video::span(r.duration(), max_seconds);
    let anchors = if follow > 0 { n / 2 } else { n };
    let blur = blur.max(1);
    let mut slots: Vec<Option<Timed>> = (0..if follow > 0 { anchors * 2 } else { n }).map(|_| None).collect();
    // The frames a sample may still need: the one before the current frame, and those blurred into either.
    let mut ring: std::collections::VecDeque<(usize, f64, Yuv)> = std::collections::VecDeque::with_capacity(blur + 2);
    // Second frames of pairs still to come: (slot, index in the stream).
    let mut pending: Vec<(usize, usize)> = Vec::new();
    let mut trace = Trace::default();
    let mut thumb: Option<[f32; THUMB * THUMB]> = None;
    let emit = |ring: &std::collections::VecDeque<(usize, f64, Yuv)>, index: usize| -> Option<Timed> {
        let &(_, pts, ref frame) = ring.iter().find(|(i, _, _)| *i == index)?;
        let rgb = if blur > 1 {
            Yuv::mean_of(&ring.iter().filter(|(i, _, _)| *i <= index && *i + blur > index).map(|(_, _, f)| f).collect::<Vec<_>>()).to_rgb()
        } else {
            frame.to_rgb()
        };
        Some(Timed { pts, index, rgb })
    };
    let (mut k, mut i, mut t0) = (0usize, 0usize, None);
    while k < anchors || !pending.is_empty() {
        let Some((pts, f)) = r.next() else { break };
        let now = f.thumb();
        trace.pts.push(pts);
        trace.diff.push(thumb.map_or(0.0, |before| now.iter().zip(&before).map(|(a, b)| (a - b).abs()).sum::<f32>() / (THUMB * THUMB) as f32));
        thumb = Some(now);
        ring.push_back((i, pts, f));
        if ring.len() > blur + 1 {
            ring.pop_front();
        }
        let start = *t0.get_or_insert(pts);
        while k < anchors {
            let target = start + k as f64 * span / anchors as f64;
            if pts < target {
                break;
            }
            // The target lies between the previous frame and this one.
            let use_prev = ring.len() >= 2 && target - ring[ring.len() - 2].1 < pts - target;
            let pick = if use_prev { i - 1 } else { i };
            if follow > 0 {
                slots[2 * k] = emit(&ring, pick);
                pending.push((2 * k + 1, pick + follow));
            } else {
                slots[k] = emit(&ring, pick);
            }
            k += 1;
        }
        pending.retain(|&(slot, index)| {
            if index == i {
                slots[slot] = emit(&ring, i);
            }
            index != i
        });
        i += 1;
    }
    let out: Vec<Timed> = if follow > 0 {
        let mut it = slots.into_iter();
        let mut pairs = Vec::new();
        while let (Some(a), Some(b)) = (it.next(), it.next()) {
            if let (Some(a), Some(b)) = (a, b) {
                pairs.extend([a, b]);
            }
        }
        pairs
    } else {
        slots.into_iter().flatten().collect()
    };
    if out.is_empty() {
        bail!("no frames decoded from {}", path.display());
    }
    Ok((out, trace))
}

/// The first `n` frames in presentation order, with the standard colour
/// conversion or OpenCV's (see [`Yuv::to_rgb_opencv`]).
pub fn first(r: &mut impl Source, n: usize, opencv: bool, path: &Path) -> Result<Vec<Rgb>> {
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let Some((_, f)) = r.next() else { break };
        out.push(if opencv { f.to_rgb_opencv() } else { f.to_rgb() });
    }
    if out.is_empty() {
        bail!("no frames decoded from {}", path.display());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(y: u8, u: u8, v: u8, full_range: bool, bt709: bool) -> Yuv {
        Yuv::new(4, 2, vec![y; 8], [u, v].repeat(2), bt709, full_range)
    }

    /// A clip whose frame `i` is flat grey of level `levels[i]`, shown at `times[i]`.
    struct Fake {
        times: Vec<f64>,
        levels: Vec<u8>,
        at: usize,
        duration: f64,
    }

    impl Fake {
        /// `n` frames at `fps`, frame `i` at level `10 + 5 i` (wrapping at 256).
        fn steady(n: usize, fps: f64) -> Self {
            Self { times: (0..n).map(|i| i as f64 / fps).collect(), levels: (0..n).map(|i| ((10 + 5 * i) % 256) as u8).collect(), at: 0, duration: n as f64 / fps }
        }
    }

    impl Source for Fake {
        fn duration(&self) -> f64 {
            self.duration
        }
        fn next(&mut self) -> Option<(f64, Yuv)> {
            let i = self.at;
            self.at += 1;
            Some((*self.times.get(i)?, Yuv::new(32, 32, vec![self.levels[i]; 32 * 32], vec![128; 16 * 16 * 2], false, true)))
        }
    }

    #[test]
    fn timed_sampling_picks_the_frames_uniform_sampling_picks() {
        let p = Path::new("fake.mp4");
        for (frames, fps, n) in [(40, 10.0, 16), (40, 10.0, 7), (5, 25.0, 16), (400, 30.0, 16)] {
            let plain = uniform(&mut Fake::steady(frames, fps), n, 30.0, p).unwrap();
            let (timed, trace) = uniform_timed(&mut Fake::steady(frames, fps), n, 30.0, 0, 1, p).unwrap();
            assert_eq!(timed.len(), plain.len(), "{frames} frames at {fps} fps, {n} samples");
            for (t, f) in timed.iter().zip(&plain) {
                assert_eq!(t.rgb.data, f.data);
                // Level 10 + 5 i is frame i, shown at i / fps.
                assert_eq!(t.rgb.data[0] as usize, (10 + 5 * t.index) % 256);
                assert!((t.pts - t.index as f64 / fps).abs() < 1e-9);
            }
            // Every frame up to the last sample was read, each five levels from the one before.
            let last = timed.last().unwrap().index;
            assert!(trace.pts.len() == last + 1 || trace.pts.len() == last + 2, "{} frames read for a last sample at {last}", trace.pts.len());
            assert_eq!(trace.diff[0], 0.0);
            assert!(trace.diff[1..].iter().take(40).all(|&d| (d - 5.0).abs() < 1e-4), "{:?}", trace.diff);
        }
    }

    #[test]
    fn paired_sampling_follows_each_sample_with_a_later_frame_of_the_stream() {
        let p = Path::new("fake.mp4");
        // 16 samples as 8 pairs: the 8 frames uniform sampling picks, each with the next frame.
        let anchors = uniform_timed(&mut Fake::steady(40, 10.0), 8, 30.0, 0, 1, p).unwrap().0;
        for follow in [1, 3] {
            let (pairs, _) = uniform_timed(&mut Fake::steady(40, 10.0), 16, 30.0, follow, 1, p).unwrap();
            assert_eq!(pairs.len(), 16);
            for (pair, anchor) in pairs.chunks(2).zip(&anchors) {
                assert_eq!(pair[0].index, anchor.index);
                assert_eq!(pair[1].index, anchor.index + follow);
                assert!((pair[1].pts - pair[0].pts - follow as f64 / 10.0).abs() < 1e-9);
            }
        }
        // A pair whose second frame the clip does not have is left out whole.
        let (pairs, _) = uniform_timed(&mut Fake::steady(4, 10.0), 8, 30.0, 2, 1, p).unwrap();
        assert_eq!(pairs.iter().map(|t| t.index).collect::<Vec<_>>(), vec![0, 2, 1, 3]);
    }

    #[test]
    fn blurred_samples_average_the_frames_before_them() {
        let p = Path::new("fake.mp4");
        let (sharp, _) = uniform_timed(&mut Fake::steady(40, 10.0), 4, 30.0, 0, 1, p).unwrap();
        let (blurred, _) = uniform_timed(&mut Fake::steady(40, 10.0), 4, 30.0, 0, 3, p).unwrap();
        for (s, b) in sharp.iter().zip(&blurred) {
            assert_eq!((s.index, s.pts), (b.index, b.pts));
            // Frame i with the two before it: levels 5 apart, so the mean is one frame back (the first frame has none before it).
            assert_eq!(b.rgb.data[0] as usize, 10 + 5 * b.index - if b.index >= 2 { 5 } else { 0 });
        }
    }

    #[test]
    fn trace_shows_repeated_frames_and_cuts() {
        // Frames 2 and 3 repeat frame 1; frame 6 jumps by 100 levels.
        let levels = vec![10, 12, 12, 12, 14, 16, 116, 118, 120, 122];
        let mut clip = Fake { times: (0..10).map(|i| i as f64 / 25.0).collect(), levels, at: 0, duration: 0.4 };
        let (_, trace) = uniform_timed(&mut clip, 10, 30.0, 0, 1, Path::new("fake.mp4")).unwrap();
        assert_eq!(trace.diff, vec![0.0, 2.0, 0.0, 0.0, 2.0, 2.0, 100.0, 2.0, 2.0, 2.0]);
        assert!(trace.pts.windows(2).all(|w| (w[1] - w[0] - 0.04).abs() < 1e-9));
    }

    #[test]
    fn thumbnails_average_cells_and_frames_average_samples() {
        // Left half dark, right half bright.
        let y: Vec<u8> = (0..64 * 32).map(|i| if i % 64 < 32 { 20 } else { 220 }).collect();
        let f = Yuv::new(64, 32, y, vec![128; 32 * 16 * 2], false, true);
        let t = f.thumb();
        assert!(t.iter().enumerate().all(|(i, &v)| v == if i % THUMB < THUMB / 2 { 20.0 } else { 220.0 }));
        let m = Yuv::mean_of(&[&flat(10, 100, 100, true, false), &flat(13, 104, 100, true, false)]);
        // (10 + 13) / 2 rounds to 12; chroma (100 + 104) / 2 = 102.
        assert_eq!((m.y[0], m.uv[0], m.uv[1]), (12, 102, 100));
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
