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
    let span = if r.duration() > 0.0 { r.duration().min(max_seconds) } else { max_seconds };
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
