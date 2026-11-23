//! Image decoding and the geometric preprocessing used by every model here.
use anyhow::{Context, Result};
use fast_image_resize as fir;
use crate::rng::Rng;
use fir::images::Image as FirImage;
use std::path::Path;

pub const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
pub const STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Interleaved 8-bit RGB.
#[derive(Clone)]
pub struct Rgb {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Nearest,
    Box,
    Bilinear,
    Bicubic,
    Lanczos,
}

impl Filter {
    pub const ALL: [Filter; 5] = [Filter::Nearest, Filter::Box, Filter::Bilinear, Filter::Bicubic, Filter::Lanczos];
    fn alg(self) -> fir::ResizeAlg {
        use fir::{FilterType as F, ResizeAlg as A};
        match self {
            Filter::Nearest => A::Nearest,
            Filter::Box => A::Convolution(F::Box),
            Filter::Bilinear => A::Convolution(F::Bilinear),
            Filter::Bicubic => A::Convolution(F::CatmullRom),
            Filter::Lanczos => A::Convolution(F::Lanczos3),
        }
    }
}

/// Per-channel `(scale, offset)` that turn an 8-bit value into the
/// ImageNet-normalised one: `v * scale - offset`.
pub fn norm_consts() -> ([f32; 3], [f32; 3]) {
    (std::array::from_fn(|c| 1.0 / (255.0 * STD[c])), std::array::from_fn(|c| MEAN[c] / STD[c]))
}

pub fn decode(bytes: &[u8]) -> Result<Rgb> {
    let img = image::load_from_memory(bytes).context("decoding image")?.into_rgb8();
    Ok(Rgb { w: img.width(), h: img.height(), data: img.into_raw() })
}

pub fn load(path: impl AsRef<Path>) -> Result<Rgb> {
    let p = path.as_ref();
    let start = std::time::Instant::now();
    let img = decode(&std::fs::read(p).with_context(|| format!("reading {}", p.display()))?).with_context(|| format!("decoding {}", p.display()));
    crate::stats::DECODE_TIME.since(start);
    img
}

impl Rgb {
    pub fn new(w: u32, h: u32) -> Self {
        Self { w, h, data: vec![0; (w * h * 3) as usize] }
    }

    /// Antialiased resize to exactly `w x h` (Pillow-compatible convolution).
    pub fn resize(&self, w: u32, h: u32, filter: Filter) -> Rgb {
        if (w, h) == (self.w, self.h) {
            return self.clone();
        }
        let src = fir::images::ImageRef::new(self.w, self.h, &self.data, fir::PixelType::U8x3).expect("valid rgb buffer");
        let mut dst = FirImage::new(w, h, fir::PixelType::U8x3);
        fir::Resizer::new()
            .resize(&src, &mut dst, &fir::ResizeOptions::new().resize_alg(filter.alg()))
            .expect("resize");
        Rgb { w, h, data: dst.into_vec() }
    }

    /// Resize so the shorter side equals `short`, keeping aspect ratio
    /// (same rounding as torchvision's `Resize(int)`).
    pub fn resize_short(&self, short: u32, filter: Filter) -> Rgb {
        let (w, h) = if self.w <= self.h {
            (short, ((short as u64 * self.h as u64) / self.w as u64) as u32)
        } else {
            (((short as u64 * self.w as u64) / self.h as u64) as u32, short)
        };
        self.resize(w.max(1), h.max(1), filter)
    }

    /// Crop a `cw x ch` window whose top-left is `(x0, y0)`; areas outside the
    /// image are zero (black), as torchvision pads.
    pub fn crop(&self, x0: i64, y0: i64, cw: u32, ch: u32) -> Rgb {
        let mut out = Rgb::new(cw, ch);
        let xs = x0.max(0);
        let xe = (x0 + cw as i64).min(self.w as i64);
        if xe <= xs {
            return out;
        }
        let n = ((xe - xs) * 3) as usize;
        for oy in 0..ch as i64 {
            let sy = y0 + oy;
            if sy < 0 || sy >= self.h as i64 {
                continue;
            }
            let s = ((sy * self.w as i64 + xs) * 3) as usize;
            let d = ((oy * cw as i64 + (xs - x0)) * 3) as usize;
            out.data[d..d + n].copy_from_slice(&self.data[s..s + n]);
        }
        out
    }

    /// Top-left corner of the centre `size`-square window (negative when the
    /// image is smaller than it).
    pub fn center_origin(&self, size: u32) -> (i64, i64) {
        // torchvision rounds the offset: int(round((h - size) / 2))
        let off = |full: u32| ((full as f64 - size as f64) / 2.0).round() as i64;
        (off(self.w), off(self.h))
    }

    pub fn center_crop(&self, size: u32) -> Rgb {
        let (x, y) = self.center_origin(size);
        self.crop(x, y, size, size)
    }

    /// As [`Rgb::center_origin`] with torchvision's `CenterCrop` rounding:
    /// offsets use Python's `round`, which rounds halves to the nearest even
    /// integer.
    pub fn center_origin_torchvision(&self, size: u32) -> (i64, i64) {
        let off = |full: u32| {
            let v = (full as f64 - size as f64) / 2.0;
            let r = v.round();
            (if (v - v.trunc()).abs() == 0.5 && r as i64 % 2 != 0 { r - v.signum() } else { r }) as i64
        };
        (off(self.w), off(self.h))
    }

    /// torchvision's `CenterCrop` (see [`Rgb::center_origin_torchvision`]).
    pub fn center_crop_torchvision(&self, size: u32) -> Rgb {
        let (x, y) = self.center_origin_torchvision(size);
        self.crop(x, y, size, size)
    }

    pub fn flip_h(&mut self) {
        let w = self.w as usize;
        for row in self.data.chunks_exact_mut(w * 3) {
            for x in 0..w / 2 {
                for c in 0..3 {
                    row.swap(x * 3 + c, (w - 1 - x) * 3 + c);
                }
            }
        }
    }

    /// Write as planar, ImageNet-normalised `f32` CHW into `out` (len `3*h*w`).
    pub fn write_chw(&self, out: &mut [f32]) {
        let n = (self.w * self.h) as usize;
        debug_assert_eq!(out.len(), 3 * n);
        let (r, rest) = out.split_at_mut(n);
        let (g, b) = rest.split_at_mut(n);
        let k: [f32; 3] = [1.0 / (255.0 * STD[0]), 1.0 / (255.0 * STD[1]), 1.0 / (255.0 * STD[2])];
        let o: [f32; 3] = [MEAN[0] / STD[0], MEAN[1] / STD[1], MEAN[2] / STD[2]];
        for (i, px) in self.data.chunks_exact(3).enumerate() {
            r[i] = px[0] as f32 * k[0] - o[0];
            g[i] = px[1] as f32 * k[1] - o[1];
            b[i] = px[2] as f32 * k[2] - o[2];
        }
    }

    /// Re-encode as JPEG at `quality` (1-100) and decode again.
    pub fn jpeg_roundtrip(&self, quality: u8) -> Result<Rgb> {
        let mut buf = Vec::with_capacity(self.data.len() / 4);
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality)
            .encode(&self.data, self.w, self.h, image::ExtendedColorType::Rgb8)?;
        decode(&buf)
    }
}

/// Resampling kernels of Pillow's `Image.resize`, reproduced exactly
/// (same coefficient maths, 22-bit fixed point, horizontal then vertical pass)
/// so that reference detectors built on Pillow can be run from Rust.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PillowFilter {
    Bilinear,
    Bicubic,
}

impl PillowFilter {
    fn support(self) -> f64 {
        match self {
            PillowFilter::Bilinear => 1.0,
            PillowFilter::Bicubic => 2.0,
        }
    }
    fn eval(self, x: f64) -> f64 {
        let x = x.abs();
        match self {
            PillowFilter::Bilinear => {
                if x < 1.0 {
                    1.0 - x
                } else {
                    0.0
                }
            }
            PillowFilter::Bicubic => {
                const A: f64 = -0.5;
                if x < 1.0 {
                    ((A + 2.0) * x - (A + 3.0)) * x * x + 1.0
                } else if x < 2.0 {
                    (((x - 5.0) * x + 8.0) * x - 4.0) * A
                } else {
                    0.0
                }
            }
        }
    }
}

const PRECISION_BITS: u32 = 32 - 8 - 2;

/// Per output position: first input index and fixed-point weights.
fn pillow_coeffs(in_size: usize, out_size: usize, f: PillowFilter) -> Vec<(usize, Vec<i32>)> {
    let scale = in_size as f64 / out_size as f64;
    let filterscale = scale.max(1.0);
    let support = f.support() * filterscale;
    let ss = 1.0 / filterscale;
    (0..out_size)
        .map(|xx| {
            let center = (xx as f64 + 0.5) * scale;
            let xmin = ((center - support + 0.5) as i64).max(0) as usize;
            let xmax = ((center + support + 0.5) as i64).min(in_size as i64) as usize;
            let mut k: Vec<f64> = (xmin..xmax).map(|x| f.eval((x as f64 - center + 0.5) * ss)).collect();
            let ww: f64 = k.iter().sum();
            if ww != 0.0 {
                k.iter_mut().for_each(|v| *v /= ww);
            }
            let fixed = k.iter().map(|&v| if v < 0.0 { (-0.5 + v * (1u32 << PRECISION_BITS) as f64) as i32 } else { (0.5 + v * (1u32 << PRECISION_BITS) as f64) as i32 }).collect();
            (xmin, fixed)
        })
        .collect()
}

#[inline]
fn clip8(v: i32) -> u8 {
    (v >> PRECISION_BITS).clamp(0, 255) as u8
}

impl Rgb {
    /// `PIL.Image.resize((w, h), filter)`, bit for bit.
    pub fn resize_pillow(&self, w: u32, h: u32, f: PillowFilter) -> Rgb {
        let (sw, sh) = (self.w as usize, self.h as usize);
        let (dw, dh) = (w as usize, h as usize);
        // Horizontal pass (skipped when the width is unchanged, as Pillow does).
        let hor = if dw != sw {
            let co = pillow_coeffs(sw, dw, f);
            let mut out = vec![0u8; dw * sh * 3];
            for y in 0..sh {
                let row = &self.data[y * sw * 3..][..sw * 3];
                let dst = &mut out[y * dw * 3..][..dw * 3];
                for (xx, (xmin, k)) in co.iter().enumerate() {
                    let mut acc = [1i32 << (PRECISION_BITS - 1); 3];
                    for (i, &kv) in k.iter().enumerate() {
                        let p = &row[(xmin + i) * 3..];
                        acc[0] += p[0] as i32 * kv;
                        acc[1] += p[1] as i32 * kv;
                        acc[2] += p[2] as i32 * kv;
                    }
                    dst[xx * 3] = clip8(acc[0]);
                    dst[xx * 3 + 1] = clip8(acc[1]);
                    dst[xx * 3 + 2] = clip8(acc[2]);
                }
            }
            out
        } else {
            self.data.clone()
        };
        if dh == sh {
            return Rgb { w, h, data: hor };
        }
        let co = pillow_coeffs(sh, dh, f);
        let mut out = vec![0u8; dw * dh * 3];
        for (yy, (ymin, k)) in co.iter().enumerate() {
            let dst = &mut out[yy * dw * 3..][..dw * 3];
            let mut acc = vec![1i32 << (PRECISION_BITS - 1); dw * 3];
            for (i, &kv) in k.iter().enumerate() {
                let row = &hor[(ymin + i) * dw * 3..][..dw * 3];
                for (a, &p) in acc.iter_mut().zip(row) {
                    *a += p as i32 * kv;
                }
            }
            for (d, &a) in dst.iter_mut().zip(&acc) {
                *d = clip8(a);
            }
        }
        Rgb { w, h, data: out }
    }

    /// Shorter side to `size` with Pillow's bicubic filter, then the centre
    /// `size` square. Sizes follow the edit model's training code:
    /// `max(size, round(side * size / short))` with Python's rounding (ties
    /// to even), and the crop origin is `(side - size) // 2`.
    pub fn short_pillow_bicubic(&self, size: u32) -> Rgb {
        let scale = size as f64 / self.w.min(self.h) as f64;
        let side = |v: u32| ((v as f64 * scale).round_ties_even() as u32).max(size);
        let (w, h) = (side(self.w), side(self.h));
        let resized = if (w, h) == (self.w, self.h) { None } else { Some(self.resize_pillow(w, h, PillowFilter::Bicubic)) };
        resized.as_ref().unwrap_or(self).crop(((w - size) / 2) as i64, ((h - size) / 2) as i64, size, size)
    }

    /// torchvision `Resize(short)` on a PIL image (bilinear, aspect kept).
    pub fn resize_short_pillow(&self, short: u32) -> Rgb {
        let (w, h) = if self.w <= self.h {
            (short, ((short as u64 * self.h as u64) / self.w as u64) as u32)
        } else {
            (((short as u64 * self.w as u64) / self.h as u64) as u32, short)
        };
        self.resize_pillow(w.max(1), h.max(1), PillowFilter::Bilinear)
    }
}

/// Fixed-point scale of the blur kernels: taps sum to exactly `1 << TAP_BITS`,
/// so a flat image stays flat.
const TAP_BITS: u32 = 14;

/// Gaussian kernel of standard deviation `sigma` pixels, cut at three sigma,
/// as fixed-point taps.
fn gaussian_taps(sigma: f32) -> Vec<i32> {
    let sigma = sigma.max(1e-3) as f64;
    let r = (3.0 * sigma).ceil().max(1.0) as i64;
    let w: Vec<f64> = (-r..=r).map(|x| (-(x * x) as f64 / (2.0 * sigma * sigma)).exp()).collect();
    let sum: f64 = w.iter().sum();
    let mut taps: Vec<i32> = w.iter().map(|v| (v / sum * (1u32 << TAP_BITS) as f64).round() as i32).collect();
    // Rounding leaves the sum a few units off; the centre tap absorbs it.
    taps[r as usize] += (1 << TAP_BITS) - taps.iter().sum::<i32>();
    taps
}

/// A zero-mean, unit-variance sample that is close to Gaussian: the sum of
/// four uniform 16-bit values from one draw (Irwin–Hall). Its tails stop at
/// 3.46 standard deviations, which is what sensor and processing noise look
/// like after clipping anyway, and it costs one draw instead of a logarithm
/// and a cosine per value — the noise step runs on whole images in the
/// training loader.
#[inline]
fn near_gaussian(rng: &mut Rng) -> f32 {
    let u = rng.u64();
    let sum = (u & 0xFFFF) + ((u >> 16) & 0xFFFF) + ((u >> 32) & 0xFFFF) + (u >> 48);
    // Mean 2 * 65535; variance 4 * (65536^2 - 1) / 12.
    (sum as f32 - 131_070.0) * (1.0 / 37_837.23)
}

#[inline]
fn to_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// The processing operations of the training chains ([`crate::data::augment`])
/// and of nothing else: inference never calls them. Each keeps the image
/// size, treats the three channels alike unless it says otherwise, and is a
/// pure function of its arguments and the generator it is handed.
impl Rgb {
    /// Convolve rows, then columns, with the same fixed-point kernel; pixels
    /// past the border repeat the border.
    fn convolve_separable(&self, taps: &[i32]) -> Rgb {
        let (w, h, r) = (self.w as usize, self.h as usize, taps.len() / 2);
        if r == 0 || w == 0 || h == 0 {
            return self.clone();
        }
        const HALF: i32 = 1 << (TAP_BITS - 1);
        let store = |dst: &mut [u8], acc: &[i32]| dst.iter_mut().zip(acc).for_each(|(d, &a)| *d = (a >> TAP_BITS).clamp(0, 255) as u8);
        let mut acc = vec![0i32; w * 3];
        let mut rows = vec![0u8; self.data.len()];
        let mut pad = vec![0u8; (w + 2 * r) * 3];
        for y in 0..h {
            let row = &self.data[y * w * 3..][..w * 3];
            for i in 0..r {
                pad[i * 3..i * 3 + 3].copy_from_slice(&row[..3]);
                pad[(w + r + i) * 3..(w + r + i) * 3 + 3].copy_from_slice(&row[(w - 1) * 3..]);
            }
            pad[r * 3..(r + w) * 3].copy_from_slice(row);
            acc.fill(HALF);
            for (t, &k) in taps.iter().enumerate() {
                acc.iter_mut().zip(&pad[t * 3..][..w * 3]).for_each(|(a, &p)| *a += p as i32 * k);
            }
            store(&mut rows[y * w * 3..][..w * 3], &acc);
        }
        let mut out = vec![0u8; self.data.len()];
        for y in 0..h {
            acc.fill(HALF);
            for (t, &k) in taps.iter().enumerate() {
                let sy = (y + t).saturating_sub(r).min(h - 1);
                acc.iter_mut().zip(&rows[sy * w * 3..][..w * 3]).for_each(|(a, &p)| *a += p as i32 * k);
            }
            store(&mut out[y * w * 3..][..w * 3], &acc);
        }
        Rgb { w: self.w, h: self.h, data: out }
    }

    /// Gaussian blur with standard deviation `sigma` pixels.
    pub fn gaussian_blur(&self, sigma: f32) -> Rgb {
        self.convolve_separable(&gaussian_taps(sigma))
    }

    /// Unsharp mask, the sharpening that editors and upload pipelines apply
    /// after a resize: `image + amount * (image - blur(image, sigma))`.
    pub fn unsharp_mask(&self, sigma: f32, amount: f32) -> Rgb {
        let mut out = self.gaussian_blur(sigma);
        let k = (amount * 256.0).round() as i32;
        for (b, &v) in out.data.iter_mut().zip(&self.data) {
            let v = v as i32;
            *b = (v + (((v - *b as i32) * k + 128) >> 8)).clamp(0, 255) as u8;
        }
        out
    }

    /// The mean of copies of the image shifted by each `(dx, dy)`; pixels
    /// past the border repeat the border. Motion and lens blur are both
    /// this, with the shifts along a line or inside a disc.
    fn mean_of_shifts(&self, shifts: &[(i32, i32)]) -> Rgb {
        let (w, h) = (self.w as usize, self.h as usize);
        if shifts.is_empty() || w == 0 || h == 0 {
            return self.clone();
        }
        let r = shifts.iter().map(|s| s.0.unsigned_abs() as usize).max().unwrap_or(0);
        let n = shifts.len() as u32;
        let mut out = vec![0u8; self.data.len()];
        let mut acc = vec![0u32; w * 3];
        let mut pad = vec![0u8; (w + 2 * r) * 3];
        for y in 0..h {
            acc.fill(n / 2);
            for &(dx, dy) in shifts {
                let sy = (y as i64 + dy as i64).clamp(0, h as i64 - 1) as usize;
                let row = &self.data[sy * w * 3..][..w * 3];
                for i in 0..r {
                    pad[i * 3..i * 3 + 3].copy_from_slice(&row[..3]);
                    pad[(w + r + i) * 3..(w + r + i) * 3 + 3].copy_from_slice(&row[(w - 1) * 3..]);
                }
                pad[r * 3..(r + w) * 3].copy_from_slice(row);
                let start = (r as i64 + dx as i64) as usize * 3;
                acc.iter_mut().zip(&pad[start..][..w * 3]).for_each(|(a, &p)| *a += p as u32);
            }
            out[y * w * 3..][..w * 3].iter_mut().zip(&acc).for_each(|(d, &a)| *d = (a / n) as u8);
        }
        Rgb { w: self.w, h: self.h, data: out }
    }

    /// Linear motion blur: the mean along a line `length` pixels long at
    /// `angle` radians.
    pub fn motion_blur(&self, length: f32, angle: f32) -> Rgb {
        let n = length.round().max(1.0) as i32;
        let (s, c) = angle.sin_cos();
        let shifts: Vec<(i32, i32)> = (0..n).map(|i| i as f32 - (n - 1) as f32 / 2.0).map(|t| ((t * c).round() as i32, (t * s).round() as i32)).collect();
        self.mean_of_shifts(&shifts)
    }

    /// Defocus blur: the mean over a disc of `radius` pixels. The cost grows
    /// with the square of the radius.
    pub fn lens_blur(&self, radius: f32) -> Rgb {
        let r = radius.max(0.0).ceil() as i32;
        let shifts: Vec<(i32, i32)> = (-r..=r).flat_map(|dy| (-r..=r).map(move |dx| (dx, dy))).filter(|&(dx, dy)| ((dx * dx + dy * dy) as f32) <= radius * radius + 0.5).collect();
        self.mean_of_shifts(&shifts)
    }

    /// Additive noise of standard deviation `sigma` (in 8-bit steps), drawn
    /// independently for every value.
    pub fn add_gaussian_noise(&mut self, sigma: f32, rng: &mut Rng) {
        for v in &mut self.data {
            *v = to_u8(*v as f32 + sigma * near_gaussian(rng));
        }
    }

    /// Signal-dependent (photon shot) noise: its standard deviation grows
    /// with the square root of the value and is `sigma` at white.
    pub fn add_shot_noise(&mut self, sigma: f32, rng: &mut Rng) {
        let scale: [f32; 256] = std::array::from_fn(|v| sigma * (v as f32 / 255.0).sqrt());
        for v in &mut self.data {
            *v = to_u8(*v as f32 + scale[*v as usize] * near_gaussian(rng));
        }
    }

    /// Salt-and-pepper noise: a fraction `p` of the values become 0 or 255.
    pub fn add_impulse_noise(&mut self, p: f32, rng: &mut Rng) {
        if p <= 0.0 || self.data.is_empty() {
            return;
        }
        // Jump straight to the next hit instead of testing every value.
        let step = (1.0 - p.min(0.999) as f64).ln();
        let mut i = 0usize;
        loop {
            let u = (rng.f32() as f64).max(1e-12);
            i = i.saturating_add((u.ln() / step) as usize);
            if i >= self.data.len() {
                return;
            }
            self.data[i] = if rng.chance(0.5) { 255 } else { 0 };
            i += 1;
        }
    }

    /// Tone curve: gamma, then contrast around mid-grey, then a brightness
    /// offset in 8-bit steps. `(1, 1, 0)` leaves the image alone.
    pub fn adjust_tone(&mut self, gamma: f32, contrast: f32, brightness: f32) {
        let lut: [u8; 256] = std::array::from_fn(|v| to_u8(((v as f32 / 255.0).powf(gamma) * 255.0 - 127.5) * contrast + 127.5 + brightness));
        for v in &mut self.data {
            *v = lut[*v as usize];
        }
    }

    /// Scale each pixel's distance from its own grey value: 0 is greyscale,
    /// 1 leaves the image alone, above 1 saturates.
    pub fn adjust_saturation(&mut self, factor: f32) {
        let k = (factor * 256.0).round() as i32;
        for px in self.data.chunks_exact_mut(3) {
            let luma = (px[0] as i32 * 77 + px[1] as i32 * 150 + px[2] as i32 * 29) >> 8;
            for v in px {
                *v = (luma + (((*v as i32 - luma) * k + 128) >> 8)).clamp(0, 255) as u8;
            }
        }
    }

    /// Resize by `factor` with `down`, then back to the original size with
    /// `up`: what is left of an image that was shrunk and enlarged again
    /// (or enlarged and shrunk, for a factor above 1). Nearest-neighbour
    /// enlargement gives pixelation.
    pub fn rescale_roundtrip(&self, factor: f32, down: Filter, up: Filter) -> Rgb {
        let side = |v: u32| ((v as f32 * factor).round() as u32).max(1);
        self.resize(side(self.w), side(self.h), down).resize(self.w, self.h, up)
    }
}

/// Top-left corners of the `k` most textured `crop`-sized windows, richest
/// first. Texture is the mean absolute luma gradient on a 2x-subsampled grid;
/// windows are taken from a half-crop grid and chosen greedily so that picks
/// do not overlap while non-overlapping candidates remain.
pub fn rich_windows(img: &Rgb, crop: u32, k: usize) -> Vec<(i64, i64)> {
    let start = std::time::Instant::now();
    let picks = rank_windows(img, crop, k);
    crate::stats::VIEWS_TIME.since(start);
    picks
}

fn rank_windows(img: &Rgb, crop: u32, k: usize) -> Vec<(i64, i64)> {
    let (w, h, c) = (img.w as usize, img.h as usize, crop as usize);
    if w < c || h < c {
        return vec![(0, 0); k.max(1)];
    }
    // Gradient energy on a subsampled luma plane, as an integral image.
    let (sw, sh) = (w / 2, h / 2);
    let integ = gradient_integral(img, sw, sh);
    let energy = |x0: usize, y0: usize| {
        let (a, b, cc, d) = (x0 / 2, y0 / 2, (x0 + c) / 2, (y0 + c) / 2);
        integ[d * (sw + 1) + cc] + integ[b * (sw + 1) + a] - integ[b * (sw + 1) + cc] - integ[d * (sw + 1) + a]
    };
    let grid = |full: usize| {
        let mut v: Vec<usize> = (0..=(full - c)).step_by((c / 2).max(1)).collect();
        if *v.last().unwrap() != full - c {
            v.push(full - c);
        }
        v
    };
    let mut cands: Vec<(u64, usize, usize)> = Vec::new();
    for &y in &grid(h) {
        for &x in &grid(w) {
            cands.push((energy(x, y), x, y));
        }
    }
    cands.sort_by(|a, b| b.0.cmp(&a.0).then(a.2.cmp(&b.2)).then(a.1.cmp(&b.1)));
    let mut picks: Vec<(i64, i64)> = Vec::new();
    // First pass: non-overlapping; later passes relax the spacing.
    for min_dist in [c, c / 2, 0] {
        for &(_, x, y) in &cands {
            if picks.len() >= k {
                break;
            }
            let far = picks.iter().all(|&(px, py)| (px - x as i64).unsigned_abs() as usize >= min_dist || (py - y as i64).unsigned_abs() as usize >= min_dist);
            if far && !picks.contains(&(x as i64, y as i64)) {
                picks.push((x as i64, y as i64));
            }
        }
    }
    while picks.len() < k.max(1) {
        picks.push(picks.first().copied().unwrap_or((0, 0)));
    }
    picks
}

/// The integral image of gradient energy on the luma plane subsampled by two
/// (every other pixel of every other row): entry `(y + 1) * (sw + 1) + x + 1`
/// is the sum over the samples above and to the left of `(x, y)` of
/// `|right - here| + |below - here|`, a missing neighbour counting nothing.
/// Integer arithmetic throughout, so splitting rows and columns between
/// threads, as large images do, gives the same sums.
fn gradient_integral(img: &Rgb, sw: usize, sh: usize) -> Vec<u64> {
    use rayon::prelude::*;
    let w = img.w as usize;
    if sw == 0 || sh == 0 {
        return vec![0; (sw + 1) * (sh + 1)];
    }
    let parallel = sw * sh >= 1 << 20;
    // The luma of each sample, 0..=255.
    let mut luma = vec![0u8; sw * sh];
    let luma_row = |(y, out): (usize, &mut [u8])| {
        let row = &img.data[y * 2 * w * 3..];
        for (x, o) in out.iter_mut().enumerate() {
            let i = x * 6;
            *o = ((row[i] as i32 * 77 + row[i + 1] as i32 * 150 + row[i + 2] as i32 * 29) >> 8) as u8;
        }
    };
    if parallel {
        luma.par_chunks_mut(sw).enumerate().for_each(luma_row);
    } else {
        luma.chunks_mut(sw).enumerate().for_each(luma_row);
    }
    // Each row's running sum of gradients, then the rows added down.
    let mut integ = vec![0u64; (sw + 1) * (sh + 1)];
    let row_sums = |(y, out): (usize, &mut [u64])| {
        let here = &luma[y * sw..(y + 1) * sw];
        let below = if y + 1 < sh { Some(&luma[(y + 1) * sw..(y + 2) * sw]) } else { None };
        let mut run = 0u64;
        for x in 0..sw {
            let v = here[x] as i32;
            let gx = if x + 1 < sw { (here[x + 1] as i32 - v).abs() } else { 0 };
            let gy = below.map_or(0, |b| (b[x] as i32 - v).abs());
            run += (gx + gy) as u64;
            out[x + 1] = run;
        }
    };
    if parallel {
        integ[sw + 1..].par_chunks_mut(sw + 1).enumerate().for_each(row_sums);
    } else {
        integ[sw + 1..].chunks_mut(sw + 1).enumerate().for_each(row_sums);
    }
    // Rows added down: within blocks of rows side by side, then each block
    // offset by the total of the blocks above it.
    let stride = sw + 1;
    let block_rows = if parallel { sh.div_ceil(rayon::current_num_threads().max(1) * 4).max(16) } else { sh.max(1) };
    let add_down = |block: &mut [u64]| {
        for y in 1..block.len() / stride {
            let (above, rest) = block.split_at_mut(y * stride);
            for (r, a) in rest[..stride].iter_mut().zip(&above[(y - 1) * stride..]) {
                *r += *a;
            }
        }
    };
    let rows = &mut integ[stride..];
    if parallel {
        rows.par_chunks_mut(block_rows * stride).for_each(add_down);
        let blocks = rows.len().div_ceil(block_rows * stride);
        let mut offsets = vec![vec![0u64; stride]; blocks];
        for k in 1..blocks {
            let last = &rows[(k * block_rows - 1) * stride..k * block_rows * stride];
            let next: Vec<u64> = offsets[k - 1].iter().zip(last).map(|(o, l)| o + l).collect();
            offsets[k] = next;
        }
        rows.par_chunks_mut(block_rows * stride).zip(offsets.par_iter()).skip(1).for_each(|(block, off)| {
            for row in block.chunks_mut(stride) {
                for (r, o) in row.iter_mut().zip(off) {
                    *r += *o;
                }
            }
        });
    } else {
        add_down(rows);
    }
    integ
}

/// A deterministic inference-time view of an image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// Resize the shorter side to `resize`, then centre-crop `crop`.
    Fit { resize: u32, crop: u32 },
    /// Centre-crop `crop` pixels at native resolution (no resampling). Images
    /// smaller than the crop are upscaled to fit first.
    Native { crop: u32 },
    /// Native-resolution crop centred at the relative position `(fx, fy)` in
    /// thousandths of the width / height (500, 500 is the centre).
    NativeAt { crop: u32, fx: u32, fy: u32 },
    /// Reference pipelines, reproduced exactly: torchvision `Resize(resize)` +
    /// `CenterCrop(crop)` on a PIL image.
    FitPillow { resize: u32, crop: u32 },
    /// `PIL.Image.resize((size, size), BICUBIC)` (aspect ratio not kept).
    SquashPillow { size: u32 },
    /// Shorter side to `size` with Pillow's bicubic filter (aspect kept, the
    /// longer side rounded as Python's `round`), then the centre `size`
    /// square with a floored origin: the edit model's training pipeline.
    ShortPillowBicubic { size: u32 },
    /// Centre crop with zero padding and no resampling at all.
    CenterPad { crop: u32 },
    /// The `rank`-th most textured native-resolution window (0 = richest).
    /// Flat regions such as sky or studio backdrops carry little forensic
    /// signal, so the crops go where the detail is.
    NativeRich { crop: u32, rank: u32 },
}

impl View {
    pub fn size(self) -> u32 {
        match self {
            View::Fit { crop, .. } | View::Native { crop } | View::NativeAt { crop, .. } | View::NativeRich { crop, .. } => crop,
            View::FitPillow { crop, .. } | View::CenterPad { crop } => crop,
            View::SquashPillow { size } | View::ShortPillowBicubic { size } => size,
        }
    }

    pub fn apply(self, img: &Rgb) -> Rgb {
        self.apply_with(img, None)
    }

    /// As [`View::apply`], reusing precomputed [`rich_windows`] when given.
    pub fn apply_with(self, img: &Rgb, rich: Option<&[(i64, i64)]>) -> Rgb {
        self.resolve(img, rich).render(img)
    }

    /// What this view comes to on `img`: where its window lies, or how the
    /// image is resized first. Two views that resolve to the same thing give
    /// the same pixels (see [`Pixels`]).
    pub fn resolve(self, img: &Rgb, rich: Option<&[(i64, i64)]>) -> Pixels {
        let short = img.w.min(img.h);
        let centre = |size: u32| {
            let (x, y) = img.center_origin(size);
            Pixels::Window { x, y, size }
        };
        // Resizing an image whose shorter side is already `to` returns it as
        // it is, so what follows is a window of the image itself.
        let resized = |to: u32, size: u32| if short == to { centre(size) } else { Pixels::Resized { short: to, size } };
        match self {
            View::NativeRich { crop, rank } => {
                if short < crop {
                    return resized(crop, crop);
                }
                let owned;
                let wins = match rich {
                    Some(w) => w,
                    None => {
                        owned = rich_windows(img, crop, rank as usize + 1);
                        &owned
                    }
                };
                let (x, y) = wins[(rank as usize).min(wins.len() - 1)];
                Pixels::Window { x, y, size: crop }
            }
            View::Fit { resize, crop } => resized(resize, crop),
            View::FitPillow { resize, crop } => Pixels::FitPillow { resize, crop },
            View::SquashPillow { size } => Pixels::SquashPillow { size },
            View::ShortPillowBicubic { size } => Pixels::ShortPillowBicubic { size },
            View::CenterPad { crop } => {
                let (x, y) = img.center_origin_torchvision(crop);
                Pixels::Window { x, y, size: crop }
            }
            View::Native { crop } => {
                if short < crop {
                    resized(crop, crop)
                } else {
                    centre(crop)
                }
            }
            View::NativeAt { crop, fx, fy } => {
                if short < crop {
                    return resized(crop, crop);
                }
                // Keep the window inside the image.
                let pos = |full: u32, f: u32| ((full as i64 * f as i64 / 1000) - crop as i64 / 2).clamp(0, (full - crop) as i64);
                Pixels::Window { x: pos(img.w, fx), y: pos(img.h, fy), size: crop }
            }
        }
    }
}

/// A [`View`] resolved on one particular image: the pixels it takes, with
/// the view's name for them left out. Different views often come to the same
/// thing: below the crop size every native view is the same upscaled centre
/// crop, the texture-ranked windows repeat when an image has fewer positions
/// than ranks, the centre crop can be one of those windows, and the
/// whole-image view of an image whose shorter side already has its size is
/// the centre crop too. Equal values give equal pixels for the same image,
/// since [`Pixels::render`] reads nothing else, so a score computed for one
/// can stand for the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pixels {
    /// The `size`-square window of the image itself whose top-left is
    /// `(x, y)`: no resampling, black outside the image.
    Window { x: i64, y: i64, size: u32 },
    /// Shorter side resized to `short` (bilinear), then the centre `size` square.
    Resized { short: u32, size: u32 },
    /// torchvision `Resize(resize)` + `CenterCrop(crop)` on a PIL image.
    FitPillow { resize: u32, crop: u32 },
    /// `PIL.Image.resize((size, size), BICUBIC)`.
    SquashPillow { size: u32 },
    /// Shorter side to `size` (Pillow bicubic), then the centre `size` square.
    ShortPillowBicubic { size: u32 },
}

impl Pixels {
    /// Cut these pixels out of `img` (the image the view was resolved on).
    pub fn render(self, img: &Rgb) -> Rgb {
        crate::stats::MATERIALISED_VIEWS.add(1);
        let start = std::time::Instant::now();
        let out = match self {
            Pixels::Window { x, y, size } => img.crop(x, y, size, size),
            Pixels::Resized { short, size } => img.resize_short(short, Filter::Bilinear).center_crop(size),
            Pixels::FitPillow { resize, crop } => img.resize_short_pillow(resize).center_crop_torchvision(crop),
            Pixels::SquashPillow { size } => img.resize_pillow(size, size, PillowFilter::Bicubic),
            Pixels::ShortPillowBicubic { size } => img.short_pillow_bicubic(size),
        };
        crate::stats::VIEWS_TIME.since(start);
        out
    }
}

/// For each entry, the index of the first entry equal to it (its own index
/// when it is the first): which earlier view, if any, has the same pixels.
pub fn first_uses(plan: &[Pixels]) -> Vec<usize> {
    (0..plan.len()).map(|i| (0..i).find(|&j| plan[j] == plan[i]).unwrap_or(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The integral image as first written, one sample at a time.
    fn gradient_integral_reference(img: &Rgb, sw: usize, sh: usize) -> Vec<u64> {
        let w = img.w as usize;
        let luma = |x: usize, y: usize| {
            let i = (y * 2 * w + x * 2) * 3;
            (img.data[i] as i32 * 77 + img.data[i + 1] as i32 * 150 + img.data[i + 2] as i32 * 29) >> 8
        };
        let mut integ = vec![0u64; (sw + 1) * (sh + 1)];
        for y in 0..sh {
            let mut row = 0u64;
            for x in 0..sw {
                let v = luma(x, y);
                let gx = if x + 1 < sw { (luma(x + 1, y) - v).abs() } else { 0 };
                let gy = if y + 1 < sh { (luma(x, y + 1) - v).abs() } else { 0 };
                row += (gx + gy) as u64;
                integ[(y + 1) * (sw + 1) + x + 1] = integ[y * (sw + 1) + x + 1] + row;
            }
        }
        integ
    }

    #[test]
    fn the_gradient_integral_matches_the_sample_by_sample_one() {
        let mut rng = crate::rng::Rng::new(3);
        // Small ones take the single-threaded path, the last two the parallel one.
        for (w, h) in [(1u32, 1u32), (2, 2), (3, 7), (225, 224), (640, 481), (2049, 2050), (4031, 3025)] {
            let mut img = Rgb::new(w, h);
            for (i, v) in img.data.iter_mut().enumerate() {
                *v = ((i as u32 / 3 % w) * 7 + (i as u32 / 3 / w) * 3) as u8 ^ rng.below(32) as u8;
            }
            let (sw, sh) = (w as usize / 2, h as usize / 2);
            assert_eq!(gradient_integral(&img, sw, sh), gradient_integral_reference(&img, sw, sh), "{w}x{h}");
        }
    }

    fn gradient(w: u32, h: u32) -> Rgb {
        let mut im = Rgb::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 3) as usize;
                im.data[i] = (x % 256) as u8;
                im.data[i + 1] = (y % 256) as u8;
                im.data[i + 2] = ((x + y) % 256) as u8;
            }
        }
        im
    }

    #[test]
    fn resize_short_keeps_aspect() {
        let r = gradient(640, 480).resize_short(256, Filter::Bilinear);
        assert_eq!((r.w, r.h), (341, 256));
        let r = gradient(300, 900).resize_short(256, Filter::Bilinear);
        assert_eq!((r.w, r.h), (256, 768));
    }

    #[test]
    fn center_crop_matches_source_pixels() {
        let im = gradient(100, 60);
        let c = im.center_crop(20);
        assert_eq!((c.w, c.h), (20, 20));
        // top-left of the crop is source pixel (40, 20)
        assert_eq!(&c.data[..3], &[40, 20, 60]);
    }

    #[test]
    fn crop_pads_with_black_outside() {
        let c = gradient(10, 10).center_crop(16);
        assert_eq!(&c.data[..3], &[0, 0, 0]);
        let i = ((3 * 16 + 3) * 3) as usize;
        assert_eq!(&c.data[i..i + 3], &[0, 0, 0]);
    }

    #[test]
    fn views_have_requested_size() {
        for im in [gradient(1024, 768), gradient(120, 90)] {
            for v in [View::Fit { resize: 256, crop: 224 }, View::Native { crop: 224 }] {
                let o = v.apply(&im);
                assert_eq!((o.w, o.h), (224, 224));
            }
        }
    }

    #[test]
    fn native_view_does_not_resample_large_images() {
        let im = gradient(1024, 768);
        let v = View::Native { crop: 224 }.apply(&im);
        assert_eq!(&v.data[..3], &im.crop(400, 272, 1, 1).data[..3]);
    }

    #[test]
    fn offset_views_stay_inside_and_differ() {
        let im = gradient(1280, 720);
        let a = View::NativeAt { crop: 224, fx: 300, fy: 300 }.apply(&im);
        let b = View::NativeAt { crop: 224, fx: 700, fy: 700 }.apply(&im);
        assert_eq!((a.w, a.h, b.w, b.h), (224, 224, 224, 224));
        assert_ne!(a.data, b.data);
        // extreme positions clamp to the border instead of padding
        let c = View::NativeAt { crop: 224, fx: 1000, fy: 1000 }.apply(&im);
        assert_eq!(&c.data[..3], &im.crop(1280 - 224, 720 - 224, 1, 1).data[..3]);
        // centre offset equals the plain native view
        assert_eq!(View::NativeAt { crop: 224, fx: 500, fy: 500 }.apply(&im).data, View::Native { crop: 224 }.apply(&im).data);
    }

    #[test]
    fn rich_windows_prefer_texture() {
        // Left half flat, right half noisy: every pick should be on the right.
        let mut im = Rgb::new(1024, 512);
        let mut rng = crate::rng::Rng::new(5);
        for y in 0..512 {
            for x in 512..1024 {
                let i = ((y * 1024 + x) * 3) as usize;
                let v = rng.below(256) as u8;
                im.data[i..i + 3].copy_from_slice(&[v, v, v]);
            }
        }
        let w = rich_windows(&im, 224, 4);
        assert_eq!(w.len(), 4);
        // every pick touches the textured half and stays inside the image;
        // the richest one lies entirely within it
        assert!(w.iter().all(|&(x, y)| x + 224 > 512 && x + 224 <= 1024 && y >= 0 && y + 224 <= 512), "{w:?}");
        assert!(w[0].0 >= 512, "{w:?}");
        // distinct, non-overlapping picks
        for i in 0..4 {
            for j in 0..i {
                assert!((w[i].0 - w[j].0).abs() >= 224 || (w[i].1 - w[j].1).abs() >= 224, "{w:?}");
            }
        }
        // small images still yield a view of the right size
        let v = View::NativeRich { crop: 224, rank: 3 }.apply(&gradient(300, 240));
        assert_eq!((v.w, v.h), (224, 224));
    }

    /// A picture with grain everywhere, so that different windows differ.
    fn grainy(w: u32, h: u32, seed: u64) -> Rgb {
        let mut im = gradient(w, h);
        let mut rng = crate::rng::Rng::new(seed);
        for v in im.data.iter_mut() {
            *v = v.wrapping_add(rng.below(64) as u8);
        }
        im
    }

    /// Every view the way it was written before views were resolved first:
    /// the reference the resolved form has to reproduce byte for byte.
    fn apply_directly(v: View, img: &Rgb, rich: Option<&[(i64, i64)]>) -> Rgb {
        match v {
            View::NativeRich { crop, rank } => {
                if img.w.min(img.h) < crop {
                    return img.resize_short(crop, Filter::Bilinear).center_crop(crop);
                }
                let owned;
                let wins = match rich {
                    Some(w) => w,
                    None => {
                        owned = rich_windows(img, crop, rank as usize + 1);
                        &owned
                    }
                };
                let (x, y) = wins[(rank as usize).min(wins.len() - 1)];
                img.crop(x, y, crop, crop)
            }
            View::Fit { resize, crop } => img.resize_short(resize, Filter::Bilinear).center_crop(crop),
            View::FitPillow { resize, crop } => img.resize_short_pillow(resize).center_crop_torchvision(crop),
            View::SquashPillow { size } => img.resize_pillow(size, size, PillowFilter::Bicubic),
            View::ShortPillowBicubic { size } => img.short_pillow_bicubic(size),
            View::CenterPad { crop } => img.center_crop_torchvision(crop),
            View::Native { crop } => {
                if img.w.min(img.h) < crop {
                    img.resize_short(crop, Filter::Bilinear).center_crop(crop)
                } else {
                    img.center_crop(crop)
                }
            }
            View::NativeAt { crop, fx, fy } => {
                if img.w.min(img.h) < crop {
                    return img.resize_short(crop, Filter::Bilinear).center_crop(crop);
                }
                let pos = |full: u32, f: u32| ((full as i64 * f as i64 / 1000) - crop as i64 / 2).clamp(0, (full - crop) as i64);
                img.crop(pos(img.w, fx), pos(img.h, fy), crop, crop)
            }
        }
    }

    const SHAPES: [(u32, u32); 22] = [
        (64, 64), (120, 90), (200, 300), (223, 223), (100, 400), (223, 500), (224, 224), (224, 225), (225, 225), (224, 300), (230, 230),
        (448, 224), (335, 224), (336, 336), (256, 256), (320, 256), (256, 400), (255, 257), (300, 240), (448, 448), (640, 480), (1024, 768),
    ];

    fn all_views() -> Vec<View> {
        let mut v = vec![View::Fit { resize: 256, crop: 224 }, View::Native { crop: 224 }];
        v.extend((0..4).map(|rank| View::NativeRich { crop: 224, rank }));
        v.extend([(280, 280), (500, 500), (720, 720), (1000, 0)].map(|(fx, fy)| View::NativeAt { crop: 224, fx, fy }));
        v.extend([View::Fit { resize: 224, crop: 224 }, View::FitPillow { resize: 256, crop: 224 }, View::SquashPillow { size: 224 }, View::ShortPillowBicubic { size: 224 }, View::CenterPad { crop: 224 }]);
        v
    }

    #[test]
    fn resolved_views_give_the_pixels_the_views_gave_and_equal_ones_give_equal_pixels() {
        for (k, &(w, h)) in SHAPES.iter().enumerate() {
            let im = grainy(w, h, k as u64 + 1);
            let rich = (w.min(h) >= 224).then(|| rich_windows(&im, 224, 4));
            let views = all_views();
            let plan: Vec<Pixels> = views.iter().map(|v| v.resolve(&im, rich.as_deref())).collect();
            let crops: Vec<Rgb> = views.iter().map(|&v| apply_directly(v, &im, rich.as_deref())).collect();
            for (i, v) in views.iter().enumerate() {
                assert_eq!(plan[i].render(&im).data, crops[i].data, "{w}x{h} {v:?} resolved to {:?}", plan[i]);
                assert_eq!(v.apply_with(&im, rich.as_deref()).data, crops[i].data, "{w}x{h} {v:?}");
                // Without the shared ranking a rich view ranks for itself and lands on the same window.
                assert_eq!(v.resolve(&im, None), plan[i], "{w}x{h} {v:?}");
                for j in 0..i {
                    if plan[i] == plan[j] {
                        assert_eq!(crops[i].data, crops[j].data, "{w}x{h}: {:?} and {:?} both resolve to {:?}", views[i], views[j], plan[i]);
                    }
                }
            }
        }
    }

    /// `first_uses` of the six default views on an image of this size.
    fn default_first_uses(w: u32, h: u32) -> Vec<usize> {
        let im = grainy(w, h, 9);
        let rich = (w.min(h) >= 224).then(|| rich_windows(&im, 224, 4));
        first_uses(&all_views()[..6].iter().map(|v| v.resolve(&im, rich.as_deref())).collect::<Vec<_>>())
    }

    #[test]
    fn views_that_coincide_are_found_and_others_are_not() {
        // Below the crop size the centre crop and every ranked window are one upscaled crop.
        for (w, h) in [(64, 64), (120, 90), (223, 500)] {
            assert_eq!(default_first_uses(w, h), [0, 1, 1, 1, 1, 1], "{w}x{h}");
        }
        // Exactly the crop size: one window position, which is the centre crop too.
        assert_eq!(default_first_uses(224, 224), [0, 1, 1, 1, 1, 1]);
        // Two positions for four ranks: the last two ranks repeat the richest window.
        assert_eq!(default_first_uses(224, 300), [0, 1, 2, 3, 2, 2]);
        // Three positions, the middle one being the centre crop: one of the first three
        // ranks is the centre crop and the fourth repeats the richest.
        let u = default_first_uses(448, 224);
        assert_eq!(u[5], u[2], "{u:?}");
        assert_eq!(u[2..5].iter().filter(|&&i| i == 1).count(), 1, "{u:?}");
        // Four positions and an off-grid centre: nothing coincides.
        assert_eq!(default_first_uses(230, 230), [0, 1, 2, 3, 4, 5]);
        // Shorter side 256: the whole-image view is not resized, so it is the centre crop.
        for (w, h) in [(256, 256), (320, 256), (256, 400)] {
            let u = default_first_uses(w, h);
            assert_eq!(&u[..2], [0, 0], "{w}x{h} {u:?}");
        }
        // An ordinary photo: six different views.
        assert_eq!(default_first_uses(1024, 768), [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn rich_windows_repeat_only_when_positions_run_out() {
        for &(w, h) in &SHAPES {
            let im = grainy(w, h, 3);
            let wins = rich_windows(&im, 224, 4);
            assert_eq!(wins.len(), 4);
            if w < 224 || h < 224 {
                continue;
            }
            // Positions on the half-crop grid, the last one flush with the far edge.
            let positions = |full: u32| ((full - 224) / 112 + 1 + u32::from((full - 224) % 112 != 0)) as usize;
            let distinct = wins.iter().collect::<std::collections::HashSet<_>>().len();
            assert_eq!(distinct, (positions(w) * positions(h)).min(4), "{w}x{h} {wins:?}");
            // Whatever is left over repeats the richest window.
            assert!(wins[distinct..].iter().all(|p| *p == wins[0]), "{w}x{h} {wins:?}");
        }
    }

    #[test]
    fn torchvision_crop_rounds_half_to_even() {
        // (549 - 384) / 2 = 82.5 -> 82 ; (551 - 384) / 2 = 83.5 -> 84
        let a = gradient(549, 440).center_crop_torchvision(384);
        assert_eq!(a.data[0], 82);
        let b = gradient(551, 440).center_crop_torchvision(384);
        assert_eq!(b.data[0], 84);
        // smaller than the crop: (10 - 16) / 2 = -3 -> three black pixels of padding
        let c = gradient(10, 10).center_crop_torchvision(16);
        assert_eq!(&c.data[..3], &[0, 0, 0]);
        assert_eq!(c.data[(3 * 16 + 3) * 3 + 2], 0);
    }

    /// The edit model's view against Pillow itself: weighted byte sums of
    /// `resize(BICUBIC)` + centre crop computed with Pillow 13 on the same
    /// synthetic images (short side larger, equal and smaller than 224, odd
    /// crop margins, and a `round` tie).
    #[test]
    fn short_pillow_bicubic_matches_pillow() {
        let expected: [((u32, u32), u64); 8] = [((300, 200), 2417198948), ((200, 300), 2418871654), ((224, 224), 2419823239), ((225, 300), 2422807372), ((1000, 333), 2421300421), ((624, 416), 2423270018), ((100, 160), 2417782400), ((513, 224), 2417172167)];
        for ((w, h), sum) in expected {
            let mut img = Rgb { w, h, data: vec![0; (w * h * 3) as usize] };
            for y in 0..h {
                for x in 0..w {
                    for c in 0..3 {
                        img.data[((y * w + x) * 3 + c) as usize] = ((x * 7 + y * 13 + c * 29 + x * y) % 256) as u8;
                    }
                }
            }
            let v = View::ShortPillowBicubic { size: 224 }.apply(&img);
            assert_eq!((v.w, v.h), (224, 224));
            let got: u64 = v.data.iter().enumerate().map(|(i, &b)| b as u64 * (i as u64 % 251 + 1)).sum();
            assert_eq!(got, sum, "{w}x{h}");
        }
    }

    #[test]
    fn pillow_resize_is_identity_at_same_size_and_averages_on_downscale() {
        let im = gradient(64, 32);
        assert_eq!(im.resize_pillow(64, 32, PillowFilter::Bilinear).data, im.data);
        // 2x box-like downscale of a constant image stays constant
        let mut flat = Rgb::new(40, 40);
        flat.data.fill(173);
        assert!(flat.resize_pillow(17, 23, PillowFilter::Bicubic).data.iter().all(|&v| v == 173));
        let r = im.resize_pillow(16, 8, PillowFilter::Bilinear);
        assert_eq!((r.w, r.h), (16, 8));
    }

    #[test]
    fn chw_normalisation() {
        let mut im = Rgb::new(1, 1);
        im.data.copy_from_slice(&[255, 0, 128]);
        let mut o = [0f32; 3];
        im.write_chw(&mut o);
        assert!((o[0] - (1.0 - MEAN[0]) / STD[0]).abs() < 1e-5);
        assert!((o[1] - (0.0 - MEAN[1]) / STD[1]).abs() < 1e-5);
        assert!((o[2] - (128.0 / 255.0 - MEAN[2]) / STD[2]).abs() < 1e-5);
    }

    fn textured(w: u32, h: u32, seed: u64) -> Rgb {
        let mut rng = Rng::new(seed);
        let mut im = gradient(w, h);
        im.data.iter_mut().for_each(|v| *v = (*v / 2).wrapping_add(rng.below(128) as u8));
        im
    }

    fn flat(w: u32, h: u32, v: u8) -> Rgb {
        let mut im = Rgb::new(w, h);
        im.data.fill(v);
        im
    }

    fn mean_std(im: &Rgb) -> (f64, f64) {
        let n = im.data.len() as f64;
        let mean = im.data.iter().map(|&v| v as f64).sum::<f64>() / n;
        (mean, (im.data.iter().map(|&v| (v as f64 - mean).powi(2)).sum::<f64>() / n).sqrt())
    }

    /// Sum of absolute differences between horizontal neighbours.
    fn edge_energy(im: &Rgb) -> u64 {
        im.data.chunks_exact(im.w as usize * 3).map(|row| row.windows(4).map(|p| (p[0] as i64 - p[3] as i64).unsigned_abs()).sum::<u64>()).sum()
    }

    #[test]
    fn gaussian_kernels_sum_to_one_and_blur_keeps_flat_images_flat() {
        for sigma in [0.1, 0.3, 0.8, 1.5, 3.0, 5.0] {
            let taps = gaussian_taps(sigma);
            assert_eq!(taps.iter().sum::<i32>(), 1 << TAP_BITS, "sigma {sigma}");
            assert!(taps.len() % 2 == 1 && taps.iter().all(|&t| t >= 0));
            assert!(flat(37, 23, 173).gaussian_blur(sigma).data.iter().all(|&v| v == 173), "sigma {sigma}");
        }
    }

    #[test]
    fn blur_smooths_and_sharpening_does_the_opposite() {
        let im = textured(96, 64, 1);
        let (mean, std) = mean_std(&im);
        let blurred = im.gaussian_blur(1.5);
        let (bmean, bstd) = mean_std(&blurred);
        assert!((mean - bmean).abs() < 1.0, "{mean} {bmean}");
        assert!(bstd < 0.8 * std, "{std} {bstd}");
        assert!(edge_energy(&blurred) < edge_energy(&im) / 2);
        // A wider kernel smooths more.
        assert!(edge_energy(&im.gaussian_blur(3.0)) < edge_energy(&blurred));
        // Sharpening a soft image restores edge contrast; amount 0 changes nothing.
        assert!(edge_energy(&blurred.unsharp_mask(1.0, 1.5)) > edge_energy(&blurred));
        assert_eq!(im.unsharp_mask(1.0, 0.0).data, im.data);
    }

    #[test]
    fn motion_blur_smears_along_its_direction_only() {
        // Vertical stripes: horizontal motion blurs them, vertical motion leaves them.
        let mut im = Rgb::new(64, 32);
        for (i, px) in im.data.chunks_exact_mut(3).enumerate() {
            px.fill(if (i % 64) / 4 % 2 == 0 { 0 } else { 255 });
        }
        assert!(edge_energy(&im.motion_blur(9.0, 0.0)) < edge_energy(&im) / 2);
        assert_eq!(im.motion_blur(9.0, std::f32::consts::FRAC_PI_2).data, im.data);
        // One sample is the image itself, and so is a disc of radius 0.
        assert_eq!(im.motion_blur(1.0, 0.7).data, im.data);
        assert_eq!(im.lens_blur(0.0).data, im.data);
        assert!(edge_energy(&im.lens_blur(3.0)) < edge_energy(&im) * 2 / 3);
        assert!(flat(20, 20, 91).lens_blur(2.5).data.iter().all(|&v| v == 91));
    }

    #[test]
    fn gaussian_noise_has_the_requested_spread_and_follows_the_seed() {
        let mut im = flat(128, 128, 128);
        im.add_gaussian_noise(8.0, &mut Rng::new(3));
        let (mean, std) = mean_std(&im);
        assert!((mean - 128.0).abs() < 0.2 && (std - 8.0).abs() < 0.3, "{mean} {std}");
        let mut again = flat(128, 128, 128);
        again.add_gaussian_noise(8.0, &mut Rng::new(3));
        assert_eq!(im.data, again.data);
        let mut other = flat(128, 128, 128);
        other.add_gaussian_noise(8.0, &mut Rng::new(4));
        assert_ne!(im.data, other.data);
        // The near-Gaussian samples themselves: zero mean, unit variance, bounded.
        let mut rng = Rng::new(9);
        let z: Vec<f32> = (0..200_000).map(|_| near_gaussian(&mut rng)).collect();
        let m = z.iter().sum::<f32>() / z.len() as f32;
        let v = z.iter().map(|x| (x - m).powi(2)).sum::<f32>() / z.len() as f32;
        assert!(m.abs() < 0.01 && (v - 1.0).abs() < 0.02 && z.iter().all(|x| x.abs() < 3.47), "{m} {v}");
    }

    #[test]
    fn shot_noise_grows_with_brightness_and_impulse_noise_hits_the_requested_share() {
        let (mut dark, mut bright, mut black) = (flat(96, 96, 16), flat(96, 96, 200), flat(96, 96, 0));
        let mut rng = Rng::new(5);
        dark.add_shot_noise(20.0, &mut rng);
        bright.add_shot_noise(20.0, &mut rng);
        black.add_shot_noise(20.0, &mut rng);
        assert!(mean_std(&bright).1 > 2.0 * mean_std(&dark).1);
        assert!(black.data.iter().all(|&v| v == 0));

        let mut im = flat(200, 200, 128);
        im.add_impulse_noise(0.02, &mut Rng::new(6));
        let hit = im.data.iter().filter(|&&v| v != 128).count();
        assert!(im.data.iter().all(|&v| v == 128 || v == 0 || v == 255));
        assert!((1800..3000).contains(&hit), "{hit} of {}", im.data.len());
        let mut none = flat(8, 8, 7);
        none.add_impulse_noise(0.0, &mut Rng::new(6));
        assert!(none.data.iter().all(|&v| v == 7));
    }

    #[test]
    fn tone_and_saturation_have_an_identity_and_move_the_right_way() {
        let im = textured(40, 30, 2);
        let mut same = im.clone();
        same.adjust_tone(1.0, 1.0, 0.0);
        same.adjust_saturation(1.0);
        assert_eq!(same.data, im.data);
        let mut darker = im.clone();
        darker.adjust_tone(1.6, 1.0, 0.0);
        assert!(mean_std(&darker).0 < mean_std(&im).0);
        let mut flatter = im.clone();
        flatter.adjust_tone(1.0, 0.5, 0.0);
        assert!(mean_std(&flatter).1 < mean_std(&im).1);
        let mut grey = im.clone();
        grey.adjust_saturation(0.0);
        assert!(grey.data.chunks_exact(3).all(|p| p[0] == p[1] && p[1] == p[2]));
    }

    #[test]
    fn processing_operations_keep_the_image_size() {
        for (w, h) in [(1, 1), (3, 2), (17, 31), (224, 224), (301, 97)] {
            let im = textured(w, h, 7);
            let mut rng = Rng::new(1);
            let mut noisy = im.clone();
            noisy.add_gaussian_noise(6.0, &mut rng);
            noisy.add_shot_noise(6.0, &mut rng);
            noisy.add_impulse_noise(0.05, &mut rng);
            noisy.adjust_tone(0.8, 1.2, 5.0);
            noisy.adjust_saturation(1.3);
            for out in [
                im.gaussian_blur(0.2),
                im.gaussian_blur(4.0),
                im.unsharp_mask(1.2, 1.0),
                im.motion_blur(7.0, 0.6),
                im.lens_blur(2.5),
                im.rescale_roundtrip(0.4, Filter::Bicubic, Filter::Bilinear),
                im.rescale_roundtrip(1.7, Filter::Lanczos, Filter::Box),
                im.rescale_roundtrip(0.3, Filter::Box, Filter::Nearest),
                noisy,
            ] {
                assert_eq!((out.w, out.h, out.data.len()), (w, h, (w * h * 3) as usize));
            }
        }
    }

    #[test]
    fn flip_and_jpeg_roundtrip() {
        let mut im = gradient(32, 16);
        let first = im.data[..3].to_vec();
        im.flip_h();
        assert_eq!(&im.data[31 * 3..32 * 3], &first[..]);
        let j = im.jpeg_roundtrip(90).unwrap();
        assert_eq!((j.w, j.h), (32, 16));
    }
}
