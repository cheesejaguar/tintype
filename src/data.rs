//! Training data: balanced sampling over generators and the augmentation
//! pipeline, run on background threads that feed the GPU.
use crate::image_io::{self, Filter, Rgb};
use crate::ingest::Item;
use crate::rng::Rng;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct AugConfig {
    pub size: u32,
    /// Shorter-side target of the "fit" view (inference uses exactly this).
    pub fit_resize: u32,
    /// Probability of training on a native-resolution crop instead of a fit view.
    pub p_native: f32,
    /// Probability of JPEG-compressing before / after the geometric step.
    pub p_jpeg_pre: f32,
    pub p_jpeg_post: f32,
    pub jpeg_quality: (f32, f32),
    /// Probability of a non-bilinear resampling filter.
    pub p_alt_filter: f32,
    /// Probability of jittering the fit scale / rescaling a native crop.
    pub p_scale_jitter: f32,
    pub p_flip: f32,
    /// For video items: probability of H.264 re-encoding before frame grab.
    pub p_video_recompress: f32,
    /// Probability of a messenger-style round trip: downscale to a typical
    /// upload size, then JPEG at the qualities chat and social apps use.
    pub p_messenger: f32,
    /// Long-side cap of one messenger trip, drawn uniformly from this range
    /// (pixels). Measured platforms go down to 720 px.
    pub messenger_cap: (f32, f32),
    /// JPEG quality of one messenger trip, drawn uniformly from this range.
    pub messenger_quality: (f32, f32),
    /// How many times in a row a messenger sample is shared, drawn uniformly
    /// from this inclusive range. `(1, 1)` is one trip and draws nothing.
    pub messenger_cycles: (u32, u32),
    /// Probability of a processing chain: a few operations from
    /// `chain_kinds`, in random order, applied to real and generated items
    /// alike. 0 turns chains off, and then they draw nothing from the
    /// generator, so the stream of views is the one from before chains existed.
    pub p_chain: f32,
    /// Number of operations in a chain, drawn uniformly from this inclusive range.
    pub chain_ops: (u32, u32),
    /// How far each operation may go, from 0 (barely visible) to 1 (the
    /// heaviest setting): the top of every parameter range scales with it.
    pub chain_severity: f32,
    /// The operations a chain draws from, each equally likely (list one twice to favour it).
    pub chain_kinds: Vec<ChainOp>,
    /// Share of chains applied to the finished view instead of the source
    /// image: processing at the scale the network sees, as on a small or
    /// already-shrunk image. The rest run before the geometric step, the way
    /// a file is processed before it reaches the detector.
    pub chain_post: f32,
    /// A chain on a source image with more pixels than this first cuts a
    /// random window of this many pixels (same aspect ratio) and continues
    /// on it. That bounds the cost without resampling, and without sparing
    /// large images, which are mostly camera photos: sparing them would tie
    /// "was processed" to the label.
    pub chain_max_pixels: u64,
    /// Curriculum: from each stage's step on, its probability, largest chain
    /// length and severity replace `p_chain`, `chain_ops` and
    /// `chain_severity`. Empty keeps them constant.
    pub chain_schedule: Vec<ChainStage>,
}

/// One operation of a processing chain. The first six are the default set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainOp {
    /// Scale down (sometimes up) with a random kernel.
    Resize,
    Jpeg,
    /// Gaussian blur.
    Blur,
    /// Additive noise, independent per value.
    Noise,
    /// Unsharp mask.
    Sharpen,
    /// One messenger trip: cap the long side, then JPEG.
    Repost,
    MotionBlur,
    LensBlur,
    /// Signal-dependent (Poisson-like) noise.
    ShotNoise,
    /// Salt-and-pepper noise.
    ImpulseNoise,
    /// Shrink, then enlarge with nearest-neighbour.
    Pixelate,
    /// Gamma, contrast, brightness and saturation.
    Tone,
}

impl ChainOp {
    /// Every operation with its name on the command line.
    pub const ALL: [(ChainOp, &'static str); 12] = [
        (ChainOp::Resize, "resize"),
        (ChainOp::Jpeg, "jpeg"),
        (ChainOp::Blur, "blur"),
        (ChainOp::Noise, "noise"),
        (ChainOp::Sharpen, "sharpen"),
        (ChainOp::Repost, "repost"),
        (ChainOp::MotionBlur, "motion-blur"),
        (ChainOp::LensBlur, "lens-blur"),
        (ChainOp::ShotNoise, "shot-noise"),
        (ChainOp::ImpulseNoise, "impulse-noise"),
        (ChainOp::Pixelate, "pixelate"),
        (ChainOp::Tone, "tone"),
    ];
    /// Resizing, JPEG, blur, noise, sharpening and a simulated repost.
    pub const CORE: [ChainOp; 6] = [ChainOp::Resize, ChainOp::Jpeg, ChainOp::Blur, ChainOp::Noise, ChainOp::Sharpen, ChainOp::Repost];

    pub fn name(self) -> &'static str {
        Self::ALL.iter().find(|(op, _)| *op == self).map_or("", |(_, n)| n)
    }

    /// Parse a comma-separated list such as `resize,jpeg,blur`.
    pub fn parse_list(text: &str) -> anyhow::Result<Vec<ChainOp>> {
        let kinds = text
            .split(',')
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(|n| Self::ALL.iter().find(|(_, name)| *name == n).map(|(op, _)| *op).ok_or_else(|| anyhow::anyhow!("unknown chain operation `{n}` (use {})", Self::ALL.map(|(_, n)| n).join(", "))))
            .collect::<anyhow::Result<Vec<_>>>()?;
        anyhow::ensure!(!kinds.is_empty(), "a chain needs at least one operation");
        Ok(kinds)
    }
}

/// One stage of the chain curriculum ([`AugConfig::chain_schedule`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainStage {
    pub from_step: u64,
    pub p: f32,
    pub max_ops: u32,
    pub severity: f32,
}

impl ChainStage {
    /// Parse `step:p:max_ops:severity,...`, e.g.
    /// `0:0:1:0,1000:0.5:2:0.25,3000:0.5:3:0.5,5500:0.5:5:1` for clean, mild,
    /// moderate, heavy.
    pub fn parse_list(text: &str) -> anyhow::Result<Vec<ChainStage>> {
        let mut stages = Vec::new();
        for part in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let f: Vec<&str> = part.split(':').collect();
            anyhow::ensure!(f.len() == 4, "chain stage `{part}` is not step:p:max_ops:severity");
            let stage = ChainStage { from_step: f[0].parse()?, p: f[1].parse()?, max_ops: f[2].parse()?, severity: f[3].parse()? };
            anyhow::ensure!((0.0..=1.0).contains(&stage.p) && (0.0..=1.0).contains(&stage.severity) && stage.max_ops >= 1, "chain stage `{part}`: p and severity must be within 0..1 and max_ops at least 1");
            anyhow::ensure!(stages.last().is_none_or(|l: &ChainStage| l.from_step < stage.from_step), "chain stages must be in increasing step order");
            stages.push(stage);
        }
        Ok(stages)
    }
}

impl Default for AugConfig {
    fn default() -> Self {
        Self {
            size: 224,
            fit_resize: 256,
            p_native: 0.5,
            p_jpeg_pre: 0.25,
            p_jpeg_post: 0.25,
            jpeg_quality: (40.0, 98.0),
            p_alt_filter: 0.3,
            p_scale_jitter: 0.3,
            p_flip: 0.5,
            p_video_recompress: 0.5,
            p_messenger: 0.0,
            messenger_cap: (960.0, 1700.0),
            messenger_quality: (55.0, 90.0),
            messenger_cycles: (1, 1),
            p_chain: 0.0,
            chain_ops: (1, 3),
            chain_severity: 0.5,
            chain_kinds: ChainOp::CORE.to_vec(),
            chain_post: 0.25,
            chain_max_pixels: 4_000_000,
            chain_schedule: Vec::new(),
        }
    }
}

impl AugConfig {
    /// The configuration in force at training step `step`: this one, with
    /// the latest curriculum stage that has started laid over it.
    pub fn at_step(&self, step: u64) -> AugConfig {
        let mut cfg = self.clone();
        if let Some(stage) = self.chain_schedule.iter().rev().find(|s| s.from_step <= step) {
            cfg.p_chain = stage.p;
            cfg.chain_ops = (self.chain_ops.0.min(stage.max_ops), stage.max_ops);
            cfg.chain_severity = stage.severity;
        }
        cfg
    }

    /// The settings that differ from the recipe before chains existed, for
    /// the training log; `None` when nothing does.
    pub fn chain_summary(&self) -> Option<String> {
        let d = AugConfig::default();
        let mut parts = Vec::new();
        if (self.messenger_cap, self.messenger_quality, self.messenger_cycles) != (d.messenger_cap, d.messenger_quality, d.messenger_cycles) {
            parts.push(format!("messenger cap {:?} px, quality {:?}, cycles {:?}", self.messenger_cap, self.messenger_quality, self.messenger_cycles));
        }
        if self.p_chain > 0.0 || !self.chain_schedule.is_empty() {
            let kinds: Vec<&str> = self.chain_kinds.iter().map(|k| k.name()).collect();
            parts.push(format!("chains p {} of {}-{} operations from [{}], severity {}, {} on the finished view, window {} px", self.p_chain, self.chain_ops.0, self.chain_ops.1, kinds.join(","), self.chain_severity, self.chain_post, self.chain_max_pixels));
            for s in &self.chain_schedule {
                parts.push(format!("from step {}: p {} up to {} operations, severity {}", s.from_step, s.p, s.max_ops, s.severity));
            }
        }
        (!parts.is_empty()).then(|| parts.join("; "))
    }
}

/// One messenger trip, as WhatsApp, Instagram or a re-share does it: cap the
/// long side, then re-encode at moderate JPEG quality.
fn messenger_trip(mut img: Rgb, cfg: &AugConfig, rng: &mut Rng) -> Rgb {
    let cap = rng.range(cfg.messenger_cap.0, cfg.messenger_cap.1);
    let long = img.w.max(img.h) as f32;
    if long > cap {
        let f = cap / long;
        let filter = if rng.chance(0.5) { Filter::Bicubic } else { Filter::Bilinear };
        img = img.resize(((img.w as f32 * f) as u32).max(1), ((img.h as f32 * f) as u32).max(1), filter);
    }
    if let Ok(j) = img.jpeg_roundtrip(rng.range(cfg.messenger_quality.0, cfg.messenger_quality.1).round() as u8) {
        img = j;
    }
    img
}

/// Apply one chain operation at severity `s` (0 to 1). With `keep_size` the
/// result has the size of the input (the finished view must stay the
/// training size); without it `Resize` and `Repost` may change it, because
/// the geometric step still follows.
///
/// Parameter ranges at severity 1, chosen to span what the cited robustness
/// work trains on (NTIRE 2026's distortion scales; DCPT's JPEG 30-90, blur
/// sigma 1-3, downscale 0.25-0.75): resize factor 0.25-1 (one in five an
/// enlargement up to 2x), JPEG quality 30-95, blur sigma up to 3 px, noise
/// sigma up to 16 of 255, unsharp amount up to 2.
fn apply_chain_op(img: Rgb, op: ChainOp, cfg: &AugConfig, s: f32, keep_size: bool, rng: &mut Rng) -> Rgb {
    let any_filter = |rng: &mut Rng| Filter::ALL[rng.below(Filter::ALL.len())];
    match op {
        ChainOp::Resize => {
            let f = if rng.chance(0.2) { rng.range(1.0, 1.0 + s) } else { rng.range(1.0 - 0.75 * s, 1.0) };
            let (a, b) = (any_filter(rng), any_filter(rng));
            if keep_size {
                img.rescale_roundtrip(f, a, b)
            } else {
                img.resize(((img.w as f32 * f).round() as u32).max(1), ((img.h as f32 * f).round() as u32).max(1), a)
            }
        }
        ChainOp::Jpeg => {
            let q = rng.range(100.0 - 70.0 * s, 100.0 - 5.0 * s).round().clamp(1.0, 100.0) as u8;
            img.jpeg_roundtrip(q).unwrap_or(img)
        }
        ChainOp::Blur => img.gaussian_blur(rng.range(0.3, 0.3 + 2.7 * s)),
        ChainOp::Noise => {
            let mut img = img;
            img.add_gaussian_noise(rng.range(0.5, 0.5 + 15.5 * s), rng);
            img
        }
        ChainOp::Sharpen => {
            let sigma = rng.range(0.5, 2.0);
            img.unsharp_mask(sigma, rng.range(0.2, 0.2 + 1.8 * s))
        }
        ChainOp::Repost => {
            let (w, h) = (img.w, img.h);
            let out = messenger_trip(img, cfg, rng);
            // A finished view larger than the cap comes back at its own size.
            if keep_size && (out.w, out.h) != (w, h) {
                out.resize(w, h, Filter::Bilinear)
            } else {
                out
            }
        }
        ChainOp::MotionBlur => {
            let length = rng.range(3.0, 3.0 + 12.0 * s);
            img.motion_blur(length, rng.range(0.0, std::f32::consts::PI))
        }
        ChainOp::LensBlur => img.lens_blur(rng.range(0.7, 0.7 + 3.3 * s)),
        ChainOp::ShotNoise => {
            let mut img = img;
            img.add_shot_noise(rng.range(1.0, 1.0 + 24.0 * s), rng);
            img
        }
        ChainOp::ImpulseNoise => {
            let mut img = img;
            img.add_impulse_noise(rng.range(0.001, 0.001 + 0.019 * s), rng);
            img
        }
        ChainOp::Pixelate => {
            let f = 1.0 / rng.range(1.0, 1.0 + 3.0 * s);
            img.rescale_roundtrip(f, Filter::Box, Filter::Nearest)
        }
        ChainOp::Tone => {
            let mut img = img;
            let gamma = (rng.range(-0.5, 0.5) * s).exp();
            let contrast = 1.0 + rng.range(-0.3, 0.3) * s;
            img.adjust_tone(gamma, contrast, rng.range(-25.0, 25.0) * s);
            img.adjust_saturation(1.0 + rng.range(-0.5, 0.5) * s);
            img
        }
    }
}

/// Draw the operations of one chain, in the order they will run.
fn draw_chain(cfg: &AugConfig, rng: &mut Rng) -> Vec<ChainOp> {
    let (lo, hi) = (cfg.chain_ops.0.max(1), cfg.chain_ops.1.max(cfg.chain_ops.0).max(1));
    let n = lo + rng.below((hi - lo + 1) as usize) as u32;
    (0..n).map(|_| cfg.chain_kinds[rng.below(cfg.chain_kinds.len())]).collect()
}

fn random_crop(img: &Rgb, size: u32, rng: &mut Rng) -> Rgb {
    let x = rng.below((img.w.saturating_sub(size) + 1) as usize) as i64;
    let y = rng.below((img.h.saturating_sub(size) + 1) as usize) as i64;
    img.crop(x, y, size, size)
}

/// One randomly augmented training view.
///
/// The label is not an argument, on purpose: every operation here treats
/// real and generated items alike, so none of them can become a cue for the
/// class. Everything added since the phase-6 recipe is off by default and,
/// when off, draws nothing from `rng`: the default stream of views is
/// byte-identical to the one that recipe trained on (a test holds a copy of
/// the old function to prove it).
pub fn augment(mut img: Rgb, cfg: &AugConfig, rng: &mut Rng) -> Rgb {
    let q = |rng: &mut Rng| rng.range(cfg.jpeg_quality.0, cfg.jpeg_quality.1).round() as u8;
    // A processing chain, if this sample gets one, and whether it runs on
    // the finished view (after the crop) or on the source image (now).
    let chain = if cfg.p_chain > 0.0 && !cfg.chain_kinds.is_empty() && rng.chance(cfg.p_chain) { Some((draw_chain(cfg, rng), rng.chance(cfg.chain_post))) } else { None };
    if let Some((ops, false)) = &chain {
        let pixels = img.w as u64 * img.h as u64;
        if pixels > cfg.chain_max_pixels.max(1) {
            let k = (cfg.chain_max_pixels.max(1) as f64 / pixels as f64).sqrt();
            let (w, h) = (((img.w as f64 * k) as u32).max(1), ((img.h as f64 * k) as u32).max(1));
            let x = rng.below((img.w - w + 1) as usize) as i64;
            let y = rng.below((img.h - h + 1) as usize) as i64;
            img = img.crop(x, y, w, h);
        }
        for &op in ops {
            img = apply_chain_op(img, op, cfg, cfg.chain_severity, false, rng);
        }
    }
    if rng.chance(cfg.p_messenger) {
        // What WhatsApp / Instagram / re-sharing do: cap the long side, then
        // re-encode at moderate JPEG quality; several times over when the
        // file is passed on.
        let (lo, hi) = cfg.messenger_cycles;
        let cycles = if hi > lo { lo + rng.below((hi - lo + 1) as usize) as u32 } else { lo };
        for _ in 0..cycles {
            img = messenger_trip(img, cfg, rng);
        }
    } else if rng.chance(cfg.p_jpeg_pre) && (img.w as u64 * img.h as u64) <= 6_000_000 {
        // JPEG at full resolution models "saved as JPEG, then resized by us".
        // Skipped for very large images to bound the cost.
        if let Ok(j) = img.jpeg_roundtrip(q(rng)) {
            img = j;
        }
    }
    let filter = if rng.chance(cfg.p_alt_filter) { Filter::ALL[rng.below(Filter::ALL.len())] } else { Filter::Bilinear };
    let mut out = if rng.chance(cfg.p_native) {
        // Native view: original pixels, optionally mildly rescaled.
        if rng.chance(cfg.p_scale_jitter) {
            let f = rng.range(0.5, 1.25);
            let (w, h) = (((img.w as f32 * f) as u32).max(cfg.size), ((img.h as f32 * f) as u32).max(cfg.size));
            img = img.resize(w, h, filter);
        }
        if img.w.min(img.h) < cfg.size {
            img = img.resize_short(cfg.size, filter);
        }
        random_crop(&img, cfg.size, rng)
    } else {
        let short = if rng.chance(cfg.p_scale_jitter) { rng.range(cfg.size as f32, cfg.fit_resize as f32 * 1.25) as u32 } else { cfg.fit_resize };
        random_crop(&img.resize_short(short, filter), cfg.size, rng)
    };
    if let Some((ops, true)) = &chain {
        for &op in ops {
            out = apply_chain_op(out, op, cfg, cfg.chain_severity, true, rng);
        }
    }
    if rng.chance(cfg.p_jpeg_post) {
        if let Ok(j) = out.jpeg_roundtrip(q(rng)) {
            out = j;
        }
    }
    if rng.chance(cfg.p_flip) {
        out.flip_h();
    }
    out
}

/// Draws items so that both classes are equally likely and, within a class,
/// generators are sampled with weight `count^alpha` (alpha < 1 lifts rare ones).
pub struct Sampler {
    classes: [Vec<(f64, Vec<usize>)>; 2],
}

impl Sampler {
    pub fn new(items: &[Item], alpha: f64) -> Self {
        let mut groups: [BTreeMap<&str, Vec<usize>>; 2] = [BTreeMap::new(), BTreeMap::new()];
        for (i, it) in items.iter().enumerate() {
            groups[it.label as usize].entry(it.generator.as_str()).or_default().push(i);
        }
        let build = |g: &BTreeMap<&str, Vec<usize>>| {
            let mut acc = 0.0;
            g.values()
                .map(|v| {
                    acc += (v.len() as f64).powf(alpha);
                    (acc, v.clone())
                })
                .collect::<Vec<_>>()
        };
        Self { classes: [build(&groups[0]), build(&groups[1])] }
    }

    pub fn sample(&self, label: usize, rng: &mut Rng) -> usize {
        let c = &self.classes[label];
        let r = rng.f32() as f64 * c.last().map(|x| x.0).unwrap_or(0.0);
        let g = &c[c.partition_point(|x| x.0 <= r).min(c.len() - 1)].1;
        g[rng.below(g.len())]
    }

    pub fn summary(&self) -> String {
        format!("{} real sources, {} generators", self.classes[0].len(), self.classes[1].len())
    }
}

pub struct Batch {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}

/// Spawn `threads` workers producing augmented batches forever. `step` is
/// the training step the consumer has reached; the workers read it only to
/// follow the chain curriculum ([`AugConfig::chain_schedule`]), a few batches
/// ahead of the trainer because of the queues between them.
pub fn spawn_loader(items: Arc<Vec<Item>>, sampler: Arc<Sampler>, cfg: AugConfig, batch: usize, threads: usize, seed: u64, step: Arc<AtomicU64>) -> crossbeam_channel::Receiver<Batch> {
    let (stx, srx) = crossbeam_channel::bounded::<(Vec<f32>, f32)>(batch * 4);
    for t in 0..threads {
        let (items, sampler, mut cfg, stx, step) = (items.clone(), sampler.clone(), cfg.clone(), stx.clone(), step.clone());
        std::thread::spawn(move || {
            let mut rng = Rng::new(seed.wrapping_mul(0x9E37_79B9).wrapping_add(t as u64 + 1));
            let n = (cfg.size * cfg.size * 3) as usize;
            let tmp = std::env::temp_dir().join("tintype-loader");
            let _ = std::fs::create_dir_all(&tmp);
            let base = cfg.clone();
            loop {
                if !base.chain_schedule.is_empty() {
                    cfg = base.at_step(step.load(Ordering::Relaxed));
                }
                let label = rng.below(2);
                let it = &items[sampler.sample(label, &mut rng)];
                let img = if crate::ingest::is_video(&it.path) {
                    crate::video::random_frame(&it.path, &mut rng, cfg.p_video_recompress, &tmp)
                } else {
                    image_io::load(&it.path)
                };
                let Ok(img) = img else { continue };
                let mut v = vec![0f32; n];
                augment(img, &cfg, &mut rng).write_chw(&mut v);
                if stx.send((v, label as f32)).is_err() {
                    return;
                }
            }
        });
    }
    let (btx, brx) = crossbeam_channel::bounded::<Batch>(4);
    std::thread::spawn(move || {
        let n = (cfg.size * cfg.size * 3) as usize;
        loop {
            let mut b = Batch { x: vec![0f32; batch * n], y: Vec::with_capacity(batch) };
            for k in 0..batch {
                let Ok((v, l)) = srx.recv() else { return };
                b.x[k * n..(k + 1) * n].copy_from_slice(&v);
                b.y.push(l);
            }
            if btx.send(b).is_err() {
                return;
            }
        }
    });
    brx
}

/// What [`bench_augment`] measured, in CPU milliseconds per image.
#[derive(Clone, Debug, Default)]
pub struct AugmentTiming {
    pub images: usize,
    pub megapixels: f64,
    pub decode_ms: f64,
    pub augment_ms: f64,
    /// Median and 95th percentile of the augmentation time alone.
    pub augment_p50_ms: f64,
    pub augment_p95_ms: f64,
}

impl AugmentTiming {
    /// Images per second one loader thread can supply: decoding plus augmentation.
    pub fn per_thread(&self) -> f64 {
        1000.0 / (self.decode_ms + self.augment_ms).max(1e-9)
    }
}

/// Time the CPU side of the training loader on real files: decode `n`
/// photos of `items` and augment each `rounds` times with `cfg`. The photos
/// are drawn the way training draws them (both classes equally often,
/// sources by `count^alpha`, with replacement; videos are left out), so
/// large and small files appear in training proportions. Timings are per
/// image and add up thread time, so they do not depend on how many of
/// `threads` workers run, as long as the machine is not oversubscribed.
pub fn bench_augment(items: &[Item], cfg: &AugConfig, n: usize, rounds: usize, threads: usize, seed: u64, alpha: f64) -> anyhow::Result<AugmentTiming> {
    use rayon::prelude::*;
    use std::time::Instant;
    let pool: Vec<Item> = items.iter().filter(|it| !crate::ingest::is_video(&it.path)).cloned().collect();
    let sampler = Sampler::new(&pool, alpha);
    let classes: Vec<usize> = (0..2).filter(|&c| !sampler.classes[c].is_empty()).collect();
    anyhow::ensure!(!classes.is_empty(), "the manifests list no photos");
    let mut rng = Rng::new(seed);
    let photos: Vec<&Item> = (0..n).map(|i| &pool[sampler.sample(classes[i % classes.len()], &mut rng)]).collect();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads.max(1)).build()?;
    // (megapixels, decode seconds, one augmentation time per round)
    let timed: Vec<(f64, f64, Vec<f64>)> = pool.install(|| {
        photos
            .par_iter()
            .enumerate()
            .filter_map(|(i, it)| {
                let t = Instant::now();
                let img = image_io::load(&it.path).ok()?;
                let decode = t.elapsed().as_secs_f64();
                let mut rng = Rng::new(seed.wrapping_mul(0x9E37_79B9).wrapping_add(i as u64 + 1));
                let rounds = (0..rounds.max(1))
                    .map(|_| {
                        let copy = img.clone();
                        let t = Instant::now();
                        std::hint::black_box(augment(copy, cfg, &mut rng));
                        t.elapsed().as_secs_f64()
                    })
                    .collect();
                Some((img.w as f64 * img.h as f64 / 1e6, decode, rounds))
            })
            .collect()
    });
    anyhow::ensure!(!timed.is_empty(), "no photo in the manifest could be decoded");
    let images = timed.len();
    let mut all: Vec<f64> = timed.iter().flat_map(|t| t.2.iter().copied()).collect();
    all.sort_by(f64::total_cmp);
    let at = |q: f64| 1e3 * all[((all.len() - 1) as f64 * q).round() as usize];
    Ok(AugmentTiming {
        images,
        megapixels: timed.iter().map(|t| t.0).sum::<f64>() / images as f64,
        decode_ms: 1e3 * timed.iter().map(|t| t.1).sum::<f64>() / images as f64,
        augment_ms: 1e3 * all.iter().sum::<f64>() / all.len() as f64,
        augment_p50_ms: at(0.5),
        augment_p95_ms: at(0.95),
    })
}

/// Deterministic train/validation split by path hash (`val_pct` percent held out).
pub fn is_val(item: &Item, val_pct: u64) -> bool {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in item.path.to_string_lossy().bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }
    (h >> 20) % 100 < val_pct
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn item(label: u8, gen: &str, i: usize) -> Item {
        Item { path: PathBuf::from(format!("{gen}/{i}.png")), label, generator: gen.into(), dataset: "t".into(), ..Default::default() }
    }

    #[test]
    fn sampler_respects_class_and_lifts_rare_generators() {
        let mut items = Vec::new();
        items.extend((0..900).map(|i| item(1, "big", i)));
        items.extend((0..100).map(|i| item(1, "small", i)));
        items.extend((0..50).map(|i| item(0, "real:a", i)));
        let s = Sampler::new(&items, 0.5);
        let mut rng = Rng::new(3);
        let mut small = 0;
        for _ in 0..4000 {
            let it = &items[s.sample(1, &mut rng)];
            assert_eq!(it.label, 1);
            small += (it.generator == "small") as usize;
        }
        // sqrt weighting: 10 / (30 + 10) = 25% instead of the raw 10%
        assert!((800..1200).contains(&small), "small sampled {small} times");
        assert_eq!(items[s.sample(0, &mut rng)].label, 0);
    }

    /// `augment` exactly as it was before processing chains were added (the
    /// phase-6 recipe), kept verbatim so the default stream of training views
    /// can be compared with it byte for byte on whatever machine runs the
    /// tests. A stored hash would not do: SIMD resizing differs between CPUs.
    fn augment_reference(mut img: Rgb, cfg: &AugConfig, rng: &mut Rng) -> Rgb {
        let q = |rng: &mut Rng| rng.range(cfg.jpeg_quality.0, cfg.jpeg_quality.1).round() as u8;
        if rng.chance(cfg.p_messenger) {
            // What WhatsApp / Instagram / re-sharing do: cap the long side, then
            // re-encode at moderate JPEG quality.
            let cap = rng.range(960.0, 1700.0);
            let long = img.w.max(img.h) as f32;
            if long > cap {
                let f = cap / long;
                let filter = if rng.chance(0.5) { Filter::Bicubic } else { Filter::Bilinear };
                img = img.resize(((img.w as f32 * f) as u32).max(1), ((img.h as f32 * f) as u32).max(1), filter);
            }
            if let Ok(j) = img.jpeg_roundtrip(rng.range(55.0, 90.0).round() as u8) {
                img = j;
            }
        } else if rng.chance(cfg.p_jpeg_pre) && (img.w as u64 * img.h as u64) <= 6_000_000 {
            // JPEG at full resolution models "saved as JPEG, then resized by us".
            // Skipped for very large images to bound the cost.
            if let Ok(j) = img.jpeg_roundtrip(q(rng)) {
                img = j;
            }
        }
        let filter = if rng.chance(cfg.p_alt_filter) { Filter::ALL[rng.below(Filter::ALL.len())] } else { Filter::Bilinear };
        let mut out = if rng.chance(cfg.p_native) {
            // Native view: original pixels, optionally mildly rescaled.
            if rng.chance(cfg.p_scale_jitter) {
                let f = rng.range(0.5, 1.25);
                let (w, h) = (((img.w as f32 * f) as u32).max(cfg.size), ((img.h as f32 * f) as u32).max(cfg.size));
                img = img.resize(w, h, filter);
            }
            if img.w.min(img.h) < cfg.size {
                img = img.resize_short(cfg.size, filter);
            }
            random_crop(&img, cfg.size, rng)
        } else {
            let short = if rng.chance(cfg.p_scale_jitter) { rng.range(cfg.size as f32, cfg.fit_resize as f32 * 1.25) as u32 } else { cfg.fit_resize };
            random_crop(&img.resize_short(short, filter), cfg.size, rng)
        };
        if rng.chance(cfg.p_jpeg_post) {
            if let Ok(j) = out.jpeg_roundtrip(q(rng)) {
                out = j;
            }
        }
        if rng.chance(cfg.p_flip) {
            out.flip_h();
        }
        out
    }

    /// A photo-like test image: smooth structure plus seeded texture, so
    /// that JPEG, resampling and the chain operations all have something to change.
    fn textured(w: u32, h: u32, seed: u64) -> Rgb {
        let mut rng = Rng::new(seed ^ 0x5EED);
        let mut im = Rgb::new(w, h);
        for (i, px) in im.data.chunks_exact_mut(3).enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let base = [(x * 255 / w.max(1)) as u8, (y * 255 / h.max(1)) as u8, ((x + y) % 256) as u8];
            for c in 0..3 {
                px[c] = (base[c] / 2).wrapping_add(rng.below(128) as u8);
            }
        }
        im
    }

    /// Every chain operation at full severity, long messenger chains and the widest caps.
    fn everything_on() -> AugConfig {
        AugConfig {
            p_messenger: 0.5,
            messenger_cap: (720.0, 2048.0),
            messenger_cycles: (1, 6),
            p_chain: 1.0,
            chain_ops: (1, 5),
            chain_severity: 1.0,
            chain_kinds: ChainOp::ALL.map(|(op, _)| op).to_vec(),
            chain_post: 0.5,
            ..Default::default()
        }
    }

    #[test]
    fn default_augmentation_is_byte_identical_to_the_recipe_before_chains() {
        // `AugConfig::default()`, and what `tintype train` builds when no
        // augmentation flag is given (main.rs has a test that it builds exactly this).
        let configs = [AugConfig::default(), AugConfig { p_native: 0.5, p_messenger: 0.25, ..Default::default() }];
        // Sizes on both sides of every branch: smaller than a view, exactly a
        // view, below and above the messenger caps, and above the 6-megapixel
        // limit of the JPEG-before-resize step.
        let sizes = [(64, 48), (224, 224), (300, 200), (1024, 768), (1800, 1350), (3008, 2000)];
        let mut views = 0;
        for cfg in &configs {
            for (k, &(w, h)) in sizes.iter().enumerate() {
                let img = textured(w, h, k as u64);
                let per_size = if w * h > 2_000_000 { 6 } else { 24 };
                for seed in 0..per_size {
                    let (mut a, mut b) = (Rng::new(1000 * k as u64 + seed), Rng::new(1000 * k as u64 + seed));
                    // Several views from one generator, as a loader thread draws them.
                    for view in 0..3 {
                        let (new, old) = (augment(img.clone(), cfg, &mut a), augment_reference(img.clone(), cfg, &mut b));
                        assert_eq!((new.w, new.h), (old.w, old.h));
                        assert!(new.data == old.data, "{w}x{h} seed {seed} view {view}: pixels differ from the reference");
                        views += 1;
                    }
                    assert_eq!(a.u64(), b.u64(), "{w}x{h} seed {seed}: the generator ended in a different state");
                }
            }
        }
        assert_eq!(views, 2 * 3 * (4 * 24 + 2 * 6));
    }

    #[test]
    fn switched_off_chains_and_single_messenger_trips_draw_nothing() {
        // Every new knob at a non-default value that still means "off":
        // the generator must be left exactly where the reference leaves it.
        let cfg = AugConfig { p_messenger: 0.25, p_chain: 0.0, chain_ops: (2, 5), chain_severity: 1.0, chain_kinds: ChainOp::ALL.map(|(op, _)| op).to_vec(), chain_post: 1.0, chain_max_pixels: 50_000, messenger_cycles: (1, 1), ..Default::default() };
        let img = textured(1500, 1100, 3);
        for seed in 0..12 {
            let (mut a, mut b) = (Rng::new(seed), Rng::new(seed));
            assert!(augment(img.clone(), &cfg, &mut a).data == augment_reference(img.clone(), &cfg, &mut b).data, "seed {seed}");
            assert_eq!(a.u64(), b.u64());
        }
        // An empty operation list also means off, whatever the probability says.
        let none = AugConfig { p_chain: 1.0, chain_kinds: Vec::new(), ..Default::default() };
        let (mut a, mut b) = (Rng::new(5), Rng::new(5));
        assert!(augment(img.clone(), &none, &mut a).data == augment_reference(img.clone(), &none, &mut b).data);
        assert_eq!(a.u64(), b.u64());
    }

    #[test]
    fn every_chain_operation_follows_the_seed_and_keeps_the_view_size() {
        let cfg = AugConfig { messenger_cap: (100.0, 160.0), ..Default::default() };
        let img = textured(224, 224, 1);
        for (op, name) in ChainOp::ALL {
            for severity in [0.0, 0.25, 1.0] {
                for seed in 0..4 {
                    let run = |keep: bool| apply_chain_op(img.clone(), op, &cfg, severity, keep, &mut Rng::new(seed));
                    let (a, b) = (run(true), run(true));
                    assert_eq!((a.w, a.h), (224, 224), "{name} changed the size of a finished view");
                    assert!(a.data == b.data, "{name} gave two results for one seed");
                    // On a source image only resizing and reposting may change the size.
                    let free = run(false);
                    assert!(matches!(op, ChainOp::Resize | ChainOp::Repost) || (free.w, free.h) == (224, 224), "{name}");
                    assert!(free.data == run(false).data, "{name}");
                }
            }
            // At full severity each operation changes a textured image for at least one seed.
            assert!((0..4).any(|seed| apply_chain_op(img.clone(), op, &cfg, 1.0, true, &mut Rng::new(seed)).data != img.data), "{name} never changed the image");
            assert_eq!(ChainOp::parse_list(name).unwrap(), vec![op]);
            assert_eq!(op.name(), name);
        }
    }

    #[test]
    fn augment_with_every_operation_on_returns_the_training_size_and_follows_the_seed() {
        let mut cfg = everything_on();
        for (k, &(w, h)) in [(1, 1), (64, 48), (224, 224), (300, 200), (1024, 768), (2600, 1900)].iter().enumerate() {
            let img = textured(w, h, k as u64);
            for seed in 0..if w * h > 2_000_000 { 4 } else { 16 } {
                let (mut a, mut b) = (Rng::new(seed), Rng::new(seed));
                let (x, y) = (augment(img.clone(), &cfg, &mut a), augment(img.clone(), &cfg, &mut b));
                assert_eq!((x.w, x.h, x.data.len()), (224, 224, 224 * 224 * 3), "{w}x{h} seed {seed}");
                assert!(x.data == y.data, "{w}x{h} seed {seed}: two results for one seed");
                assert_eq!(a.u64(), b.u64());
            }
        }
        // A window far smaller than the image still yields a full view.
        cfg.chain_max_pixels = 224 * 224;
        cfg.chain_post = 0.0;
        let big = textured(1024, 768, 9);
        for seed in 0..8 {
            let o = augment(big.clone(), &cfg, &mut Rng::new(seed));
            assert_eq!((o.w, o.h), (224, 224));
        }
    }

    #[test]
    fn chains_change_views_and_do_so_for_both_placements() {
        let img = textured(640, 480, 4);
        for post in [0.0, 1.0] {
            let cfg = AugConfig { p_chain: 1.0, chain_post: post, chain_severity: 1.0, chain_kinds: vec![ChainOp::Blur], p_jpeg_pre: 0.0, p_jpeg_post: 0.0, ..Default::default() };
            let plain = AugConfig { p_chain: 0.0, ..cfg.clone() };
            let changed = (0..8).filter(|&seed| augment(img.clone(), &cfg, &mut Rng::new(seed)).data != augment(img.clone(), &plain, &mut Rng::new(seed)).data).count();
            assert_eq!(changed, 8, "chain_post {post}");
        }
    }

    #[test]
    fn chain_lists_and_curricula_parse_and_reject_nonsense() {
        assert_eq!(ChainOp::parse_list("resize,jpeg,blur,noise,sharpen,repost").unwrap(), ChainOp::CORE.to_vec());
        assert_eq!(ChainOp::parse_list(" tone , lens-blur ").unwrap(), vec![ChainOp::Tone, ChainOp::LensBlur]);
        assert!(ChainOp::parse_list("webp").is_err());
        assert!(ChainOp::parse_list("").is_err());
        assert_eq!(AugConfig::default().chain_kinds, ChainOp::CORE.to_vec());

        assert!(ChainStage::parse_list("").unwrap().is_empty());
        let stages = ChainStage::parse_list("0:0:1:0,1000:0.5:2:0.25, 3000:0.5:3:0.5,5500:0.5:5:1").unwrap();
        assert_eq!(stages.len(), 4);
        assert_eq!(stages[2], ChainStage { from_step: 3000, p: 0.5, max_ops: 3, severity: 0.5 });
        for bad in ["0:0.5:2", "0:1.5:2:0.5", "0:0.5:0:0.5", "0:0.5:2:2", "500:0.5:2:0.5,100:0.5:2:0.5", "x:0.5:2:0.5"] {
            assert!(ChainStage::parse_list(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_curriculum_takes_over_from_its_first_stage_on() {
        let cfg = AugConfig { p_chain: 0.1, chain_ops: (2, 4), chain_severity: 0.3, chain_schedule: ChainStage::parse_list("100:0:1:0,200:0.5:3:0.5,300:0.7:5:1").unwrap(), ..Default::default() };
        let at = |step: u64| {
            let c = cfg.at_step(step);
            (c.p_chain, c.chain_ops, c.chain_severity)
        };
        assert_eq!(at(0), (0.1, (2, 4), 0.3));
        assert_eq!(at(99), (0.1, (2, 4), 0.3));
        assert_eq!(at(100), (0.0, (1, 1), 0.0));
        assert_eq!(at(250), (0.5, (2, 3), 0.5));
        assert_eq!(at(1_000_000), (0.7, (2, 5), 1.0));
        // Without a schedule the step changes nothing.
        let plain = AugConfig { p_chain: 0.4, ..Default::default() };
        assert_eq!(plain.at_step(12345), plain);
    }

    #[test]
    fn the_training_log_mentions_chains_only_when_something_is_switched_on() {
        assert_eq!(AugConfig::default().chain_summary(), None);
        assert_eq!(AugConfig { p_native: 0.5, p_messenger: 0.25, ..Default::default() }.chain_summary(), None);
        let on = everything_on().chain_summary().unwrap();
        assert!(on.contains("cycles (1, 6)") && on.contains("chains p 1") && on.contains("lens-blur"), "{on}");
        let scheduled = AugConfig { chain_schedule: ChainStage::parse_list("10:0.5:2:0.5").unwrap(), ..Default::default() }.chain_summary().unwrap();
        assert!(scheduled.contains("from step 10"), "{scheduled}");
    }

    #[test]
    fn the_loader_benchmark_times_decoding_and_augmentation_of_real_files() {
        let dir = std::env::temp_dir().join(format!("tintype-augment-bench-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut items = Vec::new();
        for i in 0..3u32 {
            let img = textured(320 + 40 * i, 240, i as u64);
            let mut buf = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 90).encode(&img.data, img.w, img.h, image::ExtendedColorType::Rgb8).unwrap();
            let path = dir.join(format!("{i}.jpg"));
            std::fs::write(&path, buf).unwrap();
            items.push(Item { path, label: (i % 2) as u8, generator: "g".into(), dataset: "t".into(), ..Default::default() });
        }
        // A file that does not decode and a video are skipped, not fatal.
        items.push(Item { path: dir.join("missing.jpg"), ..items[0].clone() });
        items.push(Item { path: dir.join("clip.mp4"), ..items[0].clone() });
        let t = bench_augment(&items, &everything_on(), 40, 2, 2, 1, 0.5).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        // 40 draws with replacement; the quarter that hit the missing file are dropped.
        assert!((20..40).contains(&t.images), "{}", t.images);
        assert!(t.megapixels > 0.07 && t.megapixels < 0.1, "{}", t.megapixels);
        assert!(t.decode_ms > 0.0 && t.augment_ms > 0.0 && t.augment_p50_ms <= t.augment_p95_ms && t.per_thread() > 0.0);
        assert!(bench_augment(&[], &AugConfig::default(), 10, 1, 1, 1, 0.5).is_err());
        assert!(bench_augment(&items[4..], &AugConfig::default(), 10, 1, 1, 1, 0.5).is_err());
    }

    #[test]
    fn augment_always_returns_training_size() {
        let cfg = AugConfig::default();
        let mut rng = Rng::new(9);
        for (w, h) in [(1024, 768), (300, 200), (64, 48), (224, 224)] {
            for _ in 0..20 {
                let mut im = Rgb::new(w, h);
                im.data.iter_mut().enumerate().for_each(|(i, p)| *p = (i * 7 % 251) as u8);
                let o = augment(im, &cfg, &mut rng);
                assert_eq!((o.w, o.h), (224, 224));
            }
        }
    }

    #[test]
    fn val_split_is_deterministic_and_roughly_sized() {
        let n = (0..10_000).filter(|&i| is_val(&item(0, "g", i), 3)).count();
        assert!((200..400).contains(&n), "{n}");
        assert_eq!(is_val(&item(0, "g", 5), 3), is_val(&item(0, "g", 5), 3));
    }
}
