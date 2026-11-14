//! Shallow clip heads over dumped frame features ([`crate::temporal`]), a
//! small trainer for them, and the harness that compares them with the frame
//! mean (`docs/runbooks/temporal-head.md`).
//!
//! Every head gives a clip logit of the form `a * mean(frame logits) + b +
//! h(features)` and starts as the frame mean exactly (`a = 1`, `b = 0`,
//! `h = 0`), so what it learns is a correction to the detector Tintype
//! already has. The kinds differ in what `h` may look at:
//!
//! - order-invariant: [`Kind::PoolLinear`], [`Kind::AttnPool`] and
//!   [`Kind::Patch`] see the frames as a set. Whatever they gain is a better
//!   pooled classifier, not a temporal cue.
//! - temporal: [`Kind::Diff`] and [`Kind::Concat`] see pairs of adjacent
//!   samples, cell against the same cell, with the time between them.
//! - [`Kind::Covariates`] sees no pixels at all: frame rate, cadence,
//!   repeated frames, cuts and motion. If it does well, the set can be told
//!   apart by how its clips were made, and any head might be reading that.
//!
//! The trainer is plain Rust on the CPU (hand-written gradients, Adam): the
//! heads are a few thousand to a few tens of thousands of parameters over
//! 384-wide features, and a head fitted here can be run by the same code
//! with no other runtime.
use crate::metrics;
use crate::temporal::{Clip, FeatureSet, Frame};
use crate::weights::Weights;
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Version of the head files written by [`Head::save`].
pub const FORMAT: u32 = 1;

/// When a stretch between two samples counts as holding a hard cut: its
/// largest frame-to-frame thumbnail difference is above `above` (8-bit
/// levels) and above `ratio` times the clip's median difference.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CutRule {
    pub above: f32,
    pub ratio: f32,
}

impl Default for CutRule {
    fn default() -> Self {
        Self { above: 12.0, ratio: 6.0 }
    }
}

/// What a clip looks like without looking at its pictures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Covariates {
    /// Frames per second: from the median time between the frames read,
    /// else the container's nominal rate, else 0.
    pub fps: f64,
    /// Largest over smallest time between consecutive frames (1 = constant cadence).
    pub jitter: f64,
    /// Frames read that repeat the frame before them, and frames read.
    pub repeats: usize,
    pub frames_read: usize,
    /// Stretches between samples that hold a cut, and stretches.
    pub cuts: usize,
    pub stretches: usize,
    /// Samples that are the same decoded frame as the sample before.
    pub resampled: usize,
    /// Median frame-to-frame thumbnail difference, 8-bit levels.
    pub motion: f64,
    /// Mean seconds between consecutive samples.
    pub sample_gap: f64,
}

/// Names of [`Covariates::vector`]'s entries.
pub const COVARIATES: [&str; 6] = ["log frame rate", "log cadence jitter", "share of repeated frames", "share of stretches with a cut", "log motion", "log time between samples"];

impl Covariates {
    pub fn of(clip: &Clip, frames: &[Frame], rule: CutRule) -> Self {
        let s = &clip.stream;
        let later = &frames[frames.len().min(1)..];
        let moved: Vec<&Frame> = later.iter().filter(|f| f.gap_frames != 0).collect();
        Self {
            fps: if s.gap_median > 0.0 { 1.0 / s.gap_median } else { clip.fps.max(0.0) },
            jitter: if s.gap_min > 0.0 { s.gap_max / s.gap_min } else { 1.0 },
            repeats: s.repeats,
            frames_read: s.frames_read,
            cuts: moved.iter().filter(|f| f.max_diff > rule.above && f.max_diff > rule.ratio * s.median_diff).count(),
            stretches: moved.len(),
            resampled: later.len() - moved.len(),
            motion: s.median_diff as f64,
            sample_gap: if later.is_empty() { 0.0 } else { later.iter().map(|f| f.gap_s.abs()).sum::<f64>() / later.len() as f64 },
        }
    }

    /// The covariates as numbers a linear model can use, in the order of [`COVARIATES`].
    pub fn vector(&self) -> [f32; 6] {
        [
            self.fps.max(1.0).ln() as f32,
            self.jitter.max(1.0).ln() as f32,
            self.repeats as f32 / self.frames_read.max(1) as f32,
            self.cuts as f32 / self.stretches.max(1) as f32,
            (1.0 + self.motion).ln() as f32,
            self.sample_gap.max(1e-3).ln() as f32,
        ]
    }

    /// The slice a clip falls in for each stratified table: `(covariate, slice)`.
    /// `motion_cuts` are the two motion values that split the set in thirds.
    pub fn slices(&self, motion_cuts: (f64, f64)) -> [(&'static str, &'static str); 5] {
        [
            ("cuts", if self.cuts > 0 { "one or more" } else { "none" }),
            ("repeated frames", if self.repeats > 0 || self.resampled > 0 { "some" } else { "none" }),
            ("motion", if self.motion <= motion_cuts.0 { "1 low" } else if self.motion <= motion_cuts.1 { "2 middle" } else { "3 high" }),
            ("frame rate", match self.fps {
                f if f < 20.0 => "1 under 20 fps",
                f if f < 24.5 => "2 20 to 24.5 fps",
                f if f < 27.0 => "3 24.5 to 27 fps",
                f if f < 35.0 => "4 27 to 35 fps",
                _ => "5 35 fps and over",
            }),
            ("time between samples", match self.sample_gap {
                g if g < 0.1 => "1 under 0.1 s",
                g if g < 0.5 => "2 0.1 to 0.5 s",
                g if g < 1.5 => "3 0.5 to 1.5 s",
                _ => "4 1.5 s and over",
            }),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Linear in the clip's [`Covariates`]: no pixels.
    Covariates,
    /// Linear in the mean and the maximum over frames of the class token.
    PoolLinear,
    /// Linear in an attention-weighted mean over frames of the class token.
    AttnPool,
    /// A one-hidden-layer network on every cell of every frame, averaged.
    Patch,
    /// The same network on the difference between a cell and the same cell
    /// of the next sample, with the time between them.
    Diff,
    /// The same network on a cell and the same cell of the next sample side
    /// by side, with the time between them.
    Concat,
}

impl Kind {
    pub const ALL: [Kind; 6] = [Kind::Covariates, Kind::PoolLinear, Kind::AttnPool, Kind::Patch, Kind::Diff, Kind::Concat];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Covariates => "covariates",
            Kind::PoolLinear => "pool-linear",
            Kind::AttnPool => "attn-pool",
            Kind::Patch => "patch",
            Kind::Diff => "diff",
            Kind::Concat => "concat",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        Self::ALL.into_iter().find(|k| k.name() == s).ok_or_else(|| anyhow::anyhow!("unknown head `{s}` (use {})", Self::ALL.map(|k| k.name()).join(", ")))
    }

    /// Whether the head looks at which sample follows which.
    pub fn temporal(self) -> bool {
        matches!(self, Kind::Diff | Kind::Concat)
    }

    fn reads_tokens(self) -> bool {
        self != Kind::Covariates
    }
}

/// Which per-frame logit a head corrects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Base {
    /// The mean of every view scored on the frame (six in the video profile).
    Frame,
    /// The whole-frame view alone: one view per frame.
    Fit,
}

/// How [`Kind::Diff`] scales a difference by the time `dt` it was taken over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeNorm {
    /// Not at all; the network still gets `ln dt` as an input.
    None,
    /// Divide by `sqrt(dt)`.
    Sqrt,
    /// Divide by `dt`: a rate of change per second.
    Rate,
}

impl TimeNorm {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "none" => TimeNorm::None,
            "sqrt" => TimeNorm::Sqrt,
            "rate" => TimeNorm::Rate,
            other => anyhow::bail!("unknown time normaliser `{other}` (use none, sqrt, rate)"),
        })
    }
}

/// How a head was fitted, kept with it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FitRecord {
    pub features: String,
    pub weights_sha256: String,
    pub clips: usize,
    pub ai: usize,
    pub epochs: usize,
    pub lr: f32,
    pub weight_decay: f32,
    pub batch: usize,
    pub seed: u64,
    pub final_loss: f32,
}

/// What a head is (`<name>.json`); its numbers are in `<name>.safetensors`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeadSpec {
    pub format: u32,
    pub kind: Kind,
    pub base: Base,
    /// The block whose tokens it reads.
    pub block: usize,
    pub grid: usize,
    pub dim: usize,
    /// Hidden units of the network kinds.
    pub hidden: usize,
    /// The sampling it pairs frames by ([`crate::temporal::Meta::follow`]).
    pub follow: usize,
    pub time_norm: TimeNorm,
    /// Fitted with every clip's frames in random order: an order-blind twin.
    pub trained_shuffled: bool,
    pub cut_rule: CutRule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit: Option<FitRecord>,
}

pub type Params = BTreeMap<String, Vec<f32>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Head {
    pub spec: HeadSpec,
    pub p: Params,
}

/// One clip as a head sees it.
pub struct ClipInput {
    /// The base logit of each sample.
    pub z: Vec<f32>,
    pub pts: Vec<f64>,
    /// Stream position of each sample.
    pub index: Vec<usize>,
    /// `[samples, 1 + grid * grid, dim]`; empty for [`Kind::Covariates`].
    pub tok: Vec<f32>,
    pub cov: [f32; 6],
}

/// Clip `c` of `set` for a head of `spec`. With `order`, sample `k` gets the
/// tokens of sample `order[k]` while times and positions stay where they
/// were: the shuffled-order control.
pub fn input(set: &FeatureSet, c: usize, spec: &HeadSpec, order: Option<&[usize]>) -> Result<ClipInput> {
    let frames = &set.frames[c];
    let mut tok = if spec.kind.reads_tokens() { set.block_tokens(c, spec.block)? } else { Vec::new() };
    if let (Some(order), false) = (order, tok.is_empty()) {
        let per = set.meta.cells() * set.meta.dim;
        tok = order.iter().flat_map(|&k| tok[k * per..(k + 1) * per].iter().copied()).collect();
    }
    Ok(ClipInput {
        z: frames.iter().map(|f| if spec.base == Base::Fit { f.fit_logit } else { f.frame_logit }).collect(),
        pts: frames.iter().map(|f| f.pts).collect(),
        index: frames.iter().map(|f| f.index).collect(),
        tok,
        cov: Covariates::of(&set.clips[c], frames, spec.cut_rule).vector(),
    })
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    // Eight running sums, so the loop vectorises.
    let mut acc = [0f32; 8];
    let (ca, cb) = (a.chunks_exact(8), b.chunks_exact(8));
    let tail: f32 = ca.remainder().iter().zip(cb.remainder()).map(|(x, y)| x * y).sum();
    for (x, y) in ca.zip(cb) {
        for i in 0..8 {
            acc[i] += x[i] * y[i];
        }
    }
    acc.iter().sum::<f32>() + tail
}

fn axpy(alpha: f32, x: &[f32], y: &mut [f32]) {
    for (y, x) in y.iter_mut().zip(x) {
        *y += alpha * x;
    }
}

impl Head {
    /// A head that is exactly the frame mean: every output weight is zero.
    pub fn new(spec: HeadSpec, seed: u64) -> Self {
        let mut p = Params::new();
        let (d, h) = (spec.dim, spec.hidden);
        p.insert("a".into(), vec![1.0]);
        p.insert("b".into(), vec![0.0]);
        match spec.kind {
            Kind::Covariates => {
                p.insert("w".into(), vec![0.0; COVARIATES.len()]);
                p.insert("mu".into(), vec![0.0; COVARIATES.len()]);
                p.insert("sd".into(), vec![1.0; COVARIATES.len()]);
            }
            Kind::PoolLinear => {
                p.insert("w".into(), vec![0.0; 2 * d]);
            }
            Kind::AttnPool => {
                p.insert("u".into(), vec![0.0; d]);
                p.insert("w".into(), vec![0.0; d]);
            }
            Kind::Patch | Kind::Diff | Kind::Concat => {
                let inp = Self::width(&spec);
                let mut rng = crate::rng::Rng::new(seed);
                // Unit-variance pre-activations for unit-variance inputs.
                let r = (3.0 / inp as f32).sqrt();
                p.insert("w1".into(), (0..h * inp).map(|_| rng.range(-r, r)).collect());
                p.insert("b1".into(), vec![0.0; h]);
                p.insert("w2".into(), vec![0.0; h]);
            }
        }
        Self { spec, p }
    }

    /// Inputs per row of the network kinds.
    fn width(spec: &HeadSpec) -> usize {
        match spec.kind {
            Kind::Patch => spec.dim,
            Kind::Diff => spec.dim + 1,
            Kind::Concat => 2 * spec.dim + 1,
            _ => 0,
        }
    }

    /// Trained numbers in the head.
    pub fn parameters(&self) -> usize {
        self.p.iter().filter(|(k, _)| !Self::fixed(k)).map(|(_, v)| v.len()).sum()
    }

    /// Statistics kept with the head but not trained.
    fn fixed(name: &str) -> bool {
        name == "mu" || name == "sd"
    }

    /// The sample pairs a temporal head compares: every sample with the next
    /// one, or the pairs of a paired dump. Two samples that are the same
    /// decoded frame are not a pair: nothing happened between them.
    pub fn pairs(&self, c: &ClipInput) -> Vec<(usize, usize)> {
        let n = c.pts.len();
        let all: Vec<(usize, usize)> = if self.spec.follow > 0 { (0..n / 2).map(|j| (2 * j, 2 * j + 1)).collect() } else { (1..n).map(|j| (j - 1, j)).collect() };
        all.into_iter().filter(|&(i, j)| c.index[i] != c.index[j]).collect()
    }

    /// The rows the network kinds average over: `(rows, count, width)`.
    fn rows(&self, c: &ClipInput) -> (Vec<f32>, usize, usize) {
        let (d, cells, inp) = (self.spec.dim, 1 + self.spec.grid * self.spec.grid, Self::width(&self.spec));
        // Cell `g` (from 1: 0 is the class token) of sample `t`.
        let cell = |t: usize, g: usize| &c.tok[(t * cells + g) * d..][..d];
        let mut x = Vec::new();
        match self.spec.kind {
            Kind::Patch => {
                for t in 0..c.pts.len() {
                    for g in 1..cells {
                        x.extend_from_slice(cell(t, g));
                    }
                }
            }
            Kind::Diff | Kind::Concat => {
                for (i, j) in self.pairs(c) {
                    let dt = ((c.pts[j] - c.pts[i]).abs() as f32).max(1e-3);
                    let scale = match self.spec.time_norm {
                        TimeNorm::None => 1.0,
                        TimeNorm::Sqrt => dt.sqrt().recip(),
                        TimeNorm::Rate => dt.recip(),
                    };
                    for g in 1..cells {
                        if self.spec.kind == Kind::Diff {
                            x.extend(cell(j, g).iter().zip(cell(i, g)).map(|(b, a)| (b - a) * scale));
                        } else {
                            x.extend_from_slice(cell(i, g));
                            x.extend_from_slice(cell(j, g));
                        }
                        x.push(dt.ln());
                    }
                }
            }
            _ => {}
        }
        let n = if inp > 0 { x.len() / inp } else { 0 };
        (x, n, inp)
    }

    /// The clip logit.
    pub fn logit(&self, c: &ClipInput) -> f32 {
        self.pass(c, None).0
    }

    /// The clip logit and, given the clip's `(label, weight)`, the gradient
    /// of `weight * binary cross-entropy` in every trained parameter.
    pub fn pass(&self, c: &ClipInput, target: Option<(u8, f32)>) -> (f32, Params) {
        let p = |name: &str| self.p[name].as_slice();
        let zbar = c.z.iter().sum::<f32>() / c.z.len().max(1) as f32;
        let mut g = Params::new();
        // The logit for a correction `h`, and d(loss)/d(logit) when there is a target.
        let close = |h: f32, g: &mut Params| -> (f32, Option<f32>) {
            let logit = p("a")[0] * zbar + p("b")[0] + h;
            let dz = target.map(|(y, w)| w * (metrics::sigmoid(logit) - y as f32));
            if let Some(dz) = dz {
                g.insert("a".into(), vec![dz * zbar]);
                g.insert("b".into(), vec![dz]);
            }
            (logit, dz)
        };
        let (d, cells) = (self.spec.dim, 1 + self.spec.grid * self.spec.grid);
        let frames = c.pts.len();
        // The class token of sample `t`.
        let cls = |t: usize| &c.tok[t * cells * d..][..d];
        match self.spec.kind {
            Kind::Covariates => {
                let x: Vec<f32> = c.cov.iter().zip(p("mu")).zip(p("sd")).map(|((v, m), s)| (v - m) / s).collect();
                let (logit, dz) = close(dot(p("w"), &x), &mut g);
                if let Some(dz) = dz {
                    g.insert("w".into(), x.iter().map(|v| dz * v).collect());
                }
                (logit, g)
            }
            Kind::PoolLinear => {
                let mut f = vec![0f32; 2 * d];
                f[d..].fill(f32::NEG_INFINITY);
                for t in 0..frames {
                    for (i, &v) in cls(t).iter().enumerate() {
                        f[i] += v / frames as f32;
                        f[d + i] = f[d + i].max(v);
                    }
                }
                if frames == 0 {
                    f.fill(0.0);
                }
                let (logit, dz) = close(dot(p("w"), &f), &mut g);
                if let Some(dz) = dz {
                    g.insert("w".into(), f.iter().map(|v| dz * v).collect());
                }
                (logit, g)
            }
            Kind::AttnPool => {
                let s: Vec<f32> = (0..frames).map(|t| dot(p("u"), cls(t))).collect();
                let top = s.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                let e: Vec<f32> = s.iter().map(|v| (v - top).exp()).collect();
                let total: f32 = e.iter().sum();
                let alpha: Vec<f32> = e.iter().map(|v| v / total).collect();
                let mut f = vec![0f32; d];
                for t in 0..frames {
                    axpy(alpha[t], cls(t), &mut f);
                }
                let (logit, dz) = close(dot(p("w"), &f), &mut g);
                if let Some(dz) = dz {
                    g.insert("w".into(), f.iter().map(|v| dz * v).collect());
                    // d(loss)/d(alpha_t) = dz * w . cls_t; back through the softmax.
                    let da: Vec<f32> = (0..frames).map(|t| dz * dot(p("w"), cls(t))).collect();
                    let mean: f32 = alpha.iter().zip(&da).map(|(a, d)| a * d).sum();
                    let mut du = vec![0f32; d];
                    for t in 0..frames {
                        axpy(alpha[t] * (da[t] - mean), cls(t), &mut du);
                    }
                    g.insert("u".into(), du);
                }
                (logit, g)
            }
            Kind::Patch | Kind::Diff | Kind::Concat => {
                let (x, n, inp) = self.rows(c);
                let (w1, b1, w2) = (p("w1"), p("b1"), p("w2"));
                let h = self.spec.hidden;
                // Hidden activations, after the ReLU.
                let mut hid = vec![0f32; n * h];
                hid.chunks_exact_mut(h.max(1)).zip(x.chunks_exact(inp.max(1))).for_each(|(o, row)| {
                    for k in 0..h {
                        o[k] = (dot(&w1[k * inp..(k + 1) * inp], row) + b1[k]).max(0.0);
                    }
                });
                let out = if n == 0 { 0.0 } else { hid.chunks_exact(h.max(1)).map(|o| dot(w2, o)).sum::<f32>() / n as f32 };
                let (logit, dz) = close(out, &mut g);
                if let Some(dz) = dz {
                    let (mut dw1, mut db1, mut dw2) = (vec![0f32; h * inp], vec![0f32; h], vec![0f32; h]);
                    let each = if n == 0 { 0.0 } else { dz / n as f32 };
                    for (o, row) in hid.chunks_exact(h.max(1)).zip(x.chunks_exact(inp.max(1))) {
                        axpy(each, o, &mut dw2);
                        for k in 0..h {
                            if o[k] > 0.0 {
                                let e = each * w2[k];
                                db1[k] += e;
                                axpy(e, row, &mut dw1[k * inp..(k + 1) * inp]);
                            }
                        }
                    }
                    g.insert("w1".into(), dw1);
                    g.insert("b1".into(), db1);
                    g.insert("w2".into(), dw2);
                }
                (logit, g)
            }
        }
    }

    fn files(stem: &Path) -> (PathBuf, PathBuf) {
        let s = stem.display().to_string();
        let s = s.strip_suffix(".json").or_else(|| s.strip_suffix(".safetensors")).unwrap_or(&s);
        (format!("{s}.json").into(), format!("{s}.safetensors").into())
    }

    /// Write `<stem>.json` and `<stem>.safetensors`.
    pub fn save(&self, stem: &Path) -> Result<()> {
        let (json, tensors) = Self::files(stem);
        if let Some(dir) = json.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&json, serde_json::to_string_pretty(&self.spec)? + "\n")?;
        let mut w = Weights::default();
        for (name, v) in &self.p {
            w.insert(name, v.clone(), vec![v.len()]);
        }
        w.save(&tensors)
    }

    pub fn load(stem: &Path) -> Result<Self> {
        let (json, tensors) = Self::files(stem);
        let spec: HeadSpec = serde_json::from_str(&std::fs::read_to_string(&json).with_context(|| format!("reading {}", json.display()))?)?;
        anyhow::ensure!(spec.format == FORMAT, "{} is head format {}, this build reads format {FORMAT}", json.display(), spec.format);
        let p: Params = Weights::load(&tensors, "")?.0.into_iter().map(|(k, (v, _))| (k, v)).collect();
        let want = Self::new(spec.clone(), 0);
        anyhow::ensure!(want.p.len() == p.len() && want.p.iter().all(|(k, v)| p.get(k).is_some_and(|x| x.len() == v.len())), "{} does not hold the tensors of a {} head", tensors.display(), spec.kind.name());
        Ok(Self { spec, p })
    }
}

/// Why `set` cannot feed a head of `spec`, if it cannot.
pub fn check(set: &FeatureSet, spec: &HeadSpec) -> Result<()> {
    let m = &set.meta;
    if spec.kind.reads_tokens() {
        anyhow::ensure!(m.blocks.contains(&spec.block), "the head reads block {} and the dump holds {:?}", spec.block, m.blocks);
        anyhow::ensure!((m.grid, m.dim) == (spec.grid, spec.dim), "the head was made for a {}x{} grid of {}-wide tokens and the dump has {}x{} of {}", spec.grid, spec.grid, spec.dim, m.grid, m.grid, m.dim);
    }
    anyhow::ensure!(!spec.kind.temporal() || m.follow == spec.follow, "the head pairs frames {} apart and the dump was sampled with --pairs {}", spec.follow, m.follow);
    Ok(())
}

/// A permutation of `0..n` that depends only on `seed` and `key`.
pub fn permutation(n: usize, seed: u64, key: &str) -> Vec<usize> {
    let h = key.bytes().fold(0xcbf2_9ce4_8422_2325u64 ^ seed, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3));
    let mut order: Vec<usize> = (0..n).collect();
    crate::rng::Rng::new(h).shuffle(&mut order);
    order
}

/// The head's logit for every clip of `set`. With `shuffle`, each clip's
/// tokens are first put in a random order (see [`input`]).
pub fn logits(set: &FeatureSet, head: &Head, shuffle: Option<u64>) -> Result<Vec<f32>> {
    check(set, &head.spec)?;
    (0..set.clips.len())
        .into_par_iter()
        .map(|c| {
            let order = shuffle.map(|seed| permutation(set.clips[c].frames, seed, &set.clips[c].item.path.to_string_lossy()));
            Ok(head.logit(&input(set, c, &head.spec, order.as_deref())?))
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct FitOptions {
    pub epochs: usize,
    pub lr: f32,
    pub weight_decay: f32,
    pub batch: usize,
    pub seed: u64,
    /// Fit with every clip's frames in a fresh random order each epoch.
    pub shuffle_frames: bool,
}

impl Default for FitOptions {
    fn default() -> Self {
        Self { epochs: 20, lr: 1e-3, weight_decay: 1e-4, batch: 64, seed: 1, shuffle_frames: false }
    }
}

/// Fit a head of `spec` to the clips of `set` (the `fit` part of a split:
/// never the clips it will be judged on). The two classes weigh the same
/// whatever their sizes. `log` gets one line per epoch; with `val`, the
/// line also says how the head does on that set, which is for watching the
/// fit, not for choosing anything.
pub fn fit(set: &FeatureSet, mut spec: HeadSpec, o: &FitOptions, val: Option<&FeatureSet>, mut log: impl FnMut(String)) -> Result<Head> {
    check(set, &spec)?;
    spec.trained_shuffled = o.shuffle_frames;
    let labels = set.labels();
    let (n, ai) = (labels.len(), labels.iter().filter(|&&l| l == 1).count());
    anyhow::ensure!(ai > 0 && ai < n, "fitting needs clips of both classes ({ai} AI of {n})");
    let weight = |l: u8| if l == 1 { n as f32 / (2.0 * ai as f32) } else { n as f32 / (2.0 * (n - ai) as f32) };
    let mut head = Head::new(spec, o.seed);
    if head.spec.kind == Kind::Covariates {
        // Standardise by this set's own statistics, kept with the head.
        let rows: Vec<[f32; 6]> = (0..n).map(|c| Covariates::of(&set.clips[c], &set.frames[c], head.spec.cut_rule).vector()).collect();
        for k in 0..COVARIATES.len() {
            let mean = rows.iter().map(|r| r[k]).sum::<f32>() / n as f32;
            let var = rows.iter().map(|r| (r[k] - mean).powi(2)).sum::<f32>() / n as f32;
            head.p.get_mut("mu").unwrap()[k] = mean;
            // A covariate that does not vary here is left as it is (and then says nothing).
            head.p.get_mut("sd").unwrap()[k] = if var.sqrt() > 1e-4 { var.sqrt() } else { 1.0 };
        }
    }
    let (b1, b2, eps) = (0.9f32, 0.999f32, 1e-8f32);
    let mut moments: BTreeMap<String, (Vec<f32>, Vec<f32>)> = head.p.iter().filter(|(k, _)| !Head::fixed(k)).map(|(k, v)| (k.clone(), (vec![0.0; v.len()], vec![0.0; v.len()]))).collect();
    let batch = o.batch.max(1);
    let total_steps = (o.epochs * n.div_ceil(batch)).max(1);
    let (mut step, mut final_loss) = (0usize, f32::NAN);
    for epoch in 0..o.epochs {
        let mut order: Vec<usize> = (0..n).collect();
        crate::rng::Rng::new(o.seed.wrapping_mul(0x9E37_79B9).wrapping_add(epoch as u64)).shuffle(&mut order);
        let (mut loss_sum, mut seen_logits, mut seen_labels) = (0f64, Vec::with_capacity(n), Vec::with_capacity(n));
        for chunk in order.chunks(batch) {
            let passes: Vec<Result<(f32, Params)>> = chunk
                .par_iter()
                .map(|&c| {
                    let shuffled = o.shuffle_frames.then(|| permutation(set.clips[c].frames, o.seed ^ (epoch as u64 + 1), &set.clips[c].item.path.to_string_lossy()));
                    Ok(head.pass(&input(set, c, &head.spec, shuffled.as_deref())?, Some((labels[c], weight(labels[c])))))
                })
                .collect();
            // Summed in clip order, so a fit is the same numbers every time.
            let mut grad: Params = moments.keys().map(|k| (k.clone(), vec![0.0; head.p[k].len()])).collect();
            for (pass, &c) in passes.into_iter().zip(chunk) {
                let (z, g) = pass?;
                let y = labels[c] as f32;
                // Weighted binary cross-entropy from the logit, stable for large |z|.
                loss_sum += (weight(labels[c]) * (z.max(0.0) - z * y + (-z.abs()).exp().ln_1p())) as f64;
                seen_logits.push(z);
                seen_labels.push(labels[c]);
                for (k, v) in g {
                    axpy(1.0 / chunk.len() as f32, &v, grad.get_mut(&k).unwrap());
                }
            }
            step += 1;
            // Cosine decay to a tenth of the starting rate.
            let lr = o.lr * (0.1 + 0.45 * (1.0 + (std::f32::consts::PI * step as f32 / total_steps as f32).cos()));
            for (k, g) in &grad {
                let (m, v) = moments.get_mut(k).unwrap();
                let decay = if matches!(k.as_str(), "a" | "b" | "b1") { 0.0 } else { o.weight_decay };
                let w = head.p.get_mut(k).unwrap();
                for i in 0..w.len() {
                    m[i] = b1 * m[i] + (1.0 - b1) * g[i];
                    v[i] = b2 * v[i] + (1.0 - b2) * g[i] * g[i];
                    let (mh, vh) = (m[i] / (1.0 - b1.powi(step as i32)), v[i] / (1.0 - b2.powi(step as i32)));
                    w[i] -= lr * (mh / (vh.sqrt() + eps) + decay * w[i]);
                }
            }
        }
        final_loss = (loss_sum / n as f64) as f32;
        let mut line = format!("epoch {:>3}: loss {:.4}, AUROC on the clips fitted {:.4}", epoch + 1, final_loss, metrics::auroc(&seen_logits, &seen_labels));
        if let Some(v) = val {
            let (z, l) = (logits(v, &head, None)?, v.labels());
            let _ = write!(line, "; watch set: AUROC {:.4}, recall at 1% FPR {:.4}, at 0.1% FPR {:.4}", metrics::auroc(&z, &l), metrics::recall_at_fpr(&z, &l, 0.01), metrics::recall_at_fpr(&z, &l, 0.001));
        }
        log(line);
    }
    head.spec.fit = Some(FitRecord { features: set.meta.weights.clone(), weights_sha256: set.meta.weights_sha256.clone(), clips: n, ai, epochs: o.epochs, lr: o.lr, weight_decay: o.weight_decay, batch, seed: o.seed, final_loss });
    Ok(head)
}

/// A system scored outside this harness (`score-video` with other frames or
/// views), to sit in the same tables: score files of the test clips and,
/// for its cutoffs, of the calibration clips.
#[derive(Clone, Debug)]
pub struct External {
    pub name: String,
    pub test: PathBuf,
    pub calibrate: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct EvalOptions {
    /// False-positive rates to operate at.
    pub fprs: Vec<f64>,
    /// Also evaluate every head with each clip's frames shuffled by this seed.
    pub shuffle_seed: Option<u64>,
    pub cut_rule: CutRule,
    /// Where to write one score file per arm, for `tintype report`.
    pub out_dir: Option<PathBuf>,
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self { fprs: vec![0.01, 0.001], shuffle_seed: Some(1), cut_rule: CutRule::default(), out_dir: None }
    }
}

struct Arm {
    name: String,
    /// What it costs per clip, in words.
    cost: String,
    /// Clip logit per test clip, `None` where the arm has no score.
    test: Vec<Option<f32>>,
    /// Logits and labels of the calibration clips.
    cal: Option<(Vec<f32>, Vec<u8>)>,
}

fn frame_mean(set: &FeatureSet, fit: bool) -> Vec<f32> {
    set.frames.iter().map(|fr| fr.iter().map(|f| if fit { f.fit_logit } else { f.frame_logit }).sum::<f32>() / fr.len().max(1) as f32).collect()
}

/// Clip logits by path from a `score-video` file: the mean of its frame logits.
fn external_logits(path: &Path) -> Result<BTreeMap<String, (f32, u8)>> {
    Ok(crate::report::read_scores(path)?.into_iter().filter(|(_, r)| !r.logits.is_empty()).map(|(k, r)| (k, (r.logits.iter().sum::<f32>() / r.logits.len() as f32, r.label))).collect())
}

fn pct(x: f64) -> String {
    if x.is_finite() { format!("{:.2}%", 100.0 * x) } else { "–".into() }
}

/// Compare the frame mean, each head and each external system on the clips
/// of `test`, at cutoffs taken from `cal` (the `calibrate` part of the
/// split). Without `cal` the cutoffs come from the test clips themselves and
/// every rate is exploratory; the text says so. Returns the report as
/// markdown.
///
/// The report has: the operating points of every arm on the clips all arms
/// scored; what each head does with shuffled frames; the same rates per
/// slice of cuts, repeated frames, motion, frame rate and sample spacing;
/// and how well each covariate alone separates the classes.
pub fn evaluate(test: &FeatureSet, cal: Option<&FeatureSet>, heads: &[(String, Head)], external: &[External], o: &EvalOptions) -> Result<String> {
    let m = &test.meta;
    if let Some(c) = cal {
        anyhow::ensure!((c.meta.frames, c.meta.follow, c.meta.blur, &c.meta.views) == (m.frames, m.follow, 1, &m.views), "the calibration dump was sampled differently from the test dump (or is a blurred one)");
    }
    let mut arms: Vec<Arm> = Vec::new();
    let views = m.views.len();
    let some = |v: Vec<f32>| v.into_iter().map(Some).collect::<Vec<_>>();
    if views > 1 {
        arms.push(Arm { name: "frame-mean".into(), cost: format!("{} frames x {views} views", m.frames), test: some(frame_mean(test, false)), cal: cal.map(|c| (frame_mean(c, false), c.labels())) });
    }
    arms.push(Arm { name: "fit-mean".into(), cost: format!("{} frames x 1 view", m.frames), test: some(frame_mean(test, true)), cal: cal.map(|c| (frame_mean(c, true), c.labels())) });
    for (name, head) in heads {
        anyhow::ensure!(head.spec.base == Base::Fit || views > 1, "head `{name}` corrects the all-view frame logit, and this dump scored the whole-frame view only");
        let cost = format!("{} frames x {} + {} head parameters", m.frames, if head.spec.base == Base::Fit { "1 view".to_string() } else { format!("{views} views") }, head.parameters());
        let on_cal = match cal {
            Some(c) => Some((logits(c, head, None)?, c.labels())),
            None => None,
        };
        arms.push(Arm { name: name.clone(), cost: cost.clone(), test: some(logits(test, head, None)?), cal: on_cal.clone() });
        if let Some(seed) = o.shuffle_seed {
            // Cutoffs stay those of the head in order: the question is what shuffling does to the head as fitted and frozen.
            arms.push(Arm { name: format!("{name}+shuffled"), cost, test: some(logits(test, head, Some(seed))?), cal: on_cal });
        }
    }
    for e in external {
        let scores = external_logits(&e.test)?;
        let on_cal = match &e.calibrate {
            Some(p) => Some(external_logits(p)?.into_values().unzip()),
            None => None,
        };
        arms.push(Arm { name: e.name.clone(), cost: "see its score-video run".into(), test: test.clips.iter().map(|c| scores.get(&c.item.path.display().to_string()).map(|s| s.0)).collect(), cal: on_cal });
    }
    anyhow::ensure!(arms.iter().map(|a| &a.name).collect::<std::collections::BTreeSet<_>>().len() == arms.len(), "two arms share a name");

    // The clips every arm scored.
    let common: Vec<usize> = (0..test.clips.len()).filter(|&c| arms.iter().all(|a| a.test[c].is_some())).collect();
    let labels: Vec<u8> = common.iter().map(|&c| test.clips[c].item.label).collect();
    let (n_ai, n_real) = (labels.iter().filter(|&&l| l == 1).count(), labels.iter().filter(|&&l| l == 0).count());
    let scores: Vec<Vec<f32>> = arms.iter().map(|a| common.iter().map(|&c| a.test[c].unwrap()).collect()).collect();
    // cutoffs[arm][fpr]: the cutoff and whether it came from calibration clips.
    let cutoffs: Vec<Vec<Option<(f32, bool)>>> = arms
        .iter()
        .zip(&scores)
        .map(|(a, s)| {
            o.fprs
                .iter()
                .map(|&f| match &a.cal {
                    Some((z, l)) => metrics::empirical_cutoff(z, l, f).map(|c| (c, true)),
                    None => metrics::empirical_cutoff(s, &labels, f).map(|c| (c, false)),
                })
                .collect()
        })
        .collect();
    // Real clips each arm's cutoffs were set on.
    let set_on: Vec<usize> = arms.iter().map(|a| a.cal.as_ref().map_or(n_real, |(_, l)| l.iter().filter(|&&x| x == 0).count())).collect();

    // Clips that share a scene (or else a parent) are one piece of evidence: resample them together.
    let key_of = |c: usize| {
        let it = &test.clips[c].item;
        it.scene_id().map(|s| format!("scene:{s}")).or_else(|| it.parent_id().map(|p| format!("parent:{p}"))).unwrap_or_else(|| format!("path:{}", it.path.display()))
    };
    let clusters = metrics::cluster_ids(&common.iter().map(|&c| key_of(c)).collect::<Vec<_>>());
    let n_clusters = clusters.iter().max().map_or(0, |m| m + 1);

    let mut out = String::new();
    let w = &mut out;
    writeln!(w, "{} clips scored by all {} arms: {n_ai} AI, {n_real} real ({} in the dump, {} unreadable when it was made).", common.len(), arms.len(), test.clips.len(), m.unreadable)?;
    writeln!(w, "Sampling: {} frames per clip{}{}, over the first {} s; tokens of block {:?} pooled to {}x{}.", m.frames, if m.follow > 0 { format!(" as pairs {} frame(s) apart", m.follow) } else { String::new() }, if m.blur > 1 { format!(", each the mean of {} frames (synthetic blur)", m.blur) } else { String::new() }, m.max_seconds, m.blocks, m.grid, m.grid)?;
    let read = common.iter().map(|&c| test.clips[c].stream.frames_read as f64).sum::<f64>() / common.len().max(1) as f64;
    writeln!(w, "Decode cost, the same for every arm of this dump: {read:.1} frames read per clip on average to reach the last sample.")?;
    if arms.iter().any(|a| a.cal.is_none()) {
        writeln!(w, "\n**Exploratory.** Arms without calibration scores take their cutoffs from the clips they are reported on: their rates are not what a frozen cutoff would give.")?;
    }
    let base = 0usize;
    for (fi, &f) in o.fprs.iter().enumerate() {
        writeln!(w, "\n### Operating at a {}% false-positive cutoff\n", 100.0 * f)?;
        writeln!(w, "| Arm | Cost per clip | Cutoff (logit) | Cutoff from | Real clips flagged | FPR | FPR upper bound | AI clips caught | Recall (95% interval) | Against {} | Recall difference (95% interval, clustered) | AUROC |", arms[base].name)?;
        writeln!(w, "|---|---|---|---|---|---|---|---|---|---|---|---|")?;
        for (ai, (a, s)) in arms.iter().zip(&scores).enumerate() {
            let Some((cut, frozen)) = cutoffs[ai][fi] else {
                writeln!(w, "| {} | {} | – | no real clips to set it on | – | – | – | – | – | – | – | – |", a.name, a.cost)?;
                continue;
            };
            let flagged = s.iter().zip(&labels).filter(|(&z, &l)| l == 0 && z > cut).count();
            let caught = s.iter().zip(&labels).filter(|(&z, &l)| l == 1 && z > cut).count();
            let (against, difference) = match cutoffs[base][fi] {
                Some((bcut, _)) if ai != base && n_ai > 0 => {
                    let (gained, lost) = s.iter().zip(&scores[base]).zip(&labels).filter(|(_, &l)| l == 1).fold((0, 0), |(g, lo), ((&z, &bz), _)| (g + (z > cut && bz <= bcut) as usize, lo + (z <= cut && bz > bcut) as usize));
                    // Per clip: +1 caught by this arm only, -1 by the baseline only. Its mean over AI clips is the recall difference.
                    let paired: Vec<f32> = s.iter().zip(&scores[base]).map(|(&z, &bz)| (z > cut) as i32 as f32 - (bz > bcut) as i32 as f32).collect();
                    let mean = |v: &[f32], l: &[u8]| {
                        let (sum, n) = v.iter().zip(l).filter(|(_, &y)| y == 1).fold((0f64, 0usize), |(a, n), (&x, _)| (a + x as f64, n + 1));
                        if n > 0 { sum / n as f64 } else { f64::NAN }
                    };
                    let (lo, hi) = metrics::bootstrap_ci_clustered(&paired, &labels, Some(&clusters), mean, 1000, 12);
                    let points = |x: f64| format!("{:+.2}", 100.0 * x);
                    (format!("+{gained} / -{lost} AI clips"), format!("{} ({} to {}) points", points(mean(&paired, &labels)), points(lo), points(hi)))
                }
                _ => ("–".into(), "–".into()),
            };
            let real_cells = if n_real > 0 { (format!("{flagged} of {n_real}"), pct(flagged as f64 / n_real as f64), pct(metrics::clopper_pearson_upper(flagged, n_real, 0.95))) } else { ("–".into(), "–".into(), "–".into()) };
            let ai_cells = if n_ai > 0 { (format!("{caught} of {n_ai}"), format!("{} ({}–{})", pct(caught as f64 / n_ai as f64), pct(metrics::clopper_pearson_lower(caught, n_ai, 0.975)), pct(metrics::clopper_pearson_upper(caught, n_ai, 0.975)))) } else { ("–".into(), "–".into()) };
            let auroc = metrics::auroc(s, &labels);
            // How many of the real clips the cutoff was set on may lie above it at this rate.
            let allowed = (f * set_on[ai] as f64).floor() as usize;
            let from = format!("{} ({allowed} of {} real allowed above)", if frozen { "calibration clips" } else { "these clips (exploratory)" }, set_on[ai]);
            writeln!(w, "| {} | {} | {cut:.4} | {from} | {} | {} | {} | {} | {} | {against} | {difference} | {} |", a.name, a.cost, real_cells.0, real_cells.1, real_cells.2, ai_cells.0, ai_cells.1, if auroc.is_finite() { format!("{auroc:.4}") } else { "–".into() })?;
        }
        if let Some(most) = set_on.iter().max().filter(|&&n| (f * n as f64) < 1.0) {
            writeln!(w, "\n**Not resolved.** A {}% rate allows no false alarm among {most} real clips (it takes {} for one): these cutoffs sit at the highest real score they were set on, and the rows say what that cutoff does, not what a {}% cutoff would.", 100.0 * f, (1.0 / f).ceil(), 100.0 * f)?;
        }
    }
    writeln!(w, "\nFPR upper bound: one-sided 95% Clopper–Pearson; recall interval: two-sided 95% Clopper–Pearson. Both take clips as independent, which clips that share a source or a scene are not. The recall difference is this arm's recall minus {}'s on the same AI clips, each at its own cutoff, with a 95% bootstrap interval over {n_clusters} clusters (clips of one `scene_id`, or else one `parent_id`, together){}. For clustered intervals on each arm's own recall, run `tintype report --operating-points --cluster-by` on the score files.", arms[base].name, if n_clusters < 20 { ": fewer than 20 clusters, too few for the interval to mean much" } else { "" })?;

    if o.shuffle_seed.is_some() && !heads.is_empty() {
        writeln!(w, "\n### Shuffled-order control\n")?;
        writeln!(w, "Each head as fitted, on the same clips with every clip's frames in a random order and its cutoffs unchanged. A head that reads what happens between frames should lose what it gained; an order-invariant head cannot change at all.\n")?;
        writeln!(w, "| Head | Reads order | Mean absolute change in logit | Largest change | AUROC in order | AUROC shuffled |{}", o.fprs.iter().map(|f| format!(" AI caught at {}% cutoff, in order -> shuffled |", 100.0 * f)).collect::<String>())?;
        writeln!(w, "|---|---|---|---|---|---|{}", "---|".repeat(o.fprs.len()))?;
        for (name, head) in heads {
            let at = |n: &str| arms.iter().position(|a| a.name == n).unwrap();
            let (i, j) = (at(name), at(&format!("{name}+shuffled")));
            let change: Vec<f32> = scores[i].iter().zip(&scores[j]).map(|(a, b)| (a - b).abs()).collect();
            let caught = |k: usize, fi: usize| cutoffs[i][fi].map_or("–".to_string(), |(cut, _)| scores[k].iter().zip(&labels).filter(|(&z, &l)| l == 1 && z > cut).count().to_string());
            let cells: String = (0..o.fprs.len()).map(|fi| format!(" {} -> {} of {n_ai} |", caught(i, fi), caught(j, fi))).collect();
            writeln!(w, "| {name} | {} | {:.4} | {:.4} | {:.4} | {:.4} |{cells}", if head.spec.kind.temporal() { "yes" } else { "no" }, change.iter().sum::<f32>() / change.len().max(1) as f32, change.iter().copied().fold(0.0, f32::max), metrics::auroc(&scores[i], &labels), metrics::auroc(&scores[j], &labels))?;
        }
    }

    // Covariates of the common clips, and the motion values that split them in thirds.
    let cov: Vec<Covariates> = common.iter().map(|&c| Covariates::of(&test.clips[c], &test.frames[c], o.cut_rule)).collect();
    let mut motion: Vec<f64> = cov.iter().map(|c| c.motion).collect();
    motion.sort_by(f64::total_cmp);
    let thirds = if motion.is_empty() { (0.0, 0.0) } else { (motion[motion.len() / 3], motion[2 * motion.len() / 3]) };
    writeln!(w, "\n### By cuts, repeated frames, motion and cadence\n")?;
    writeln!(w, "Each arm at its own cutoff, clips grouped by what their frames did. A cut is a stretch between samples whose largest frame-to-frame thumbnail difference is over {} levels and over {} times the clip's median; a repeated frame differs from the one before by at most {} levels; motion is the clip's median frame-to-frame difference (thirds of these clips: up to {:.2}, up to {:.2}, above). A head whose gain sits in one slice of cadence or repeats is reading how the clips were made.", o.cut_rule.above, o.cut_rule.ratio, crate::temporal::REPEAT_BELOW, thirds.0, thirds.1)?;
    // The shuffled arms have their own table above.
    let sliced: Vec<usize> = (0..arms.len()).filter(|&ai| !arms[ai].name.ends_with("+shuffled")).collect();
    for (fi, &f) in o.fprs.iter().enumerate() {
        for k in 0..5 {
            let mut slices: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
            for (i, c) in cov.iter().enumerate() {
                slices.entry(c.slices(thirds)[k].1).or_default().push(i);
            }
            let Some(title) = cov.first().map(|c| c.slices(thirds)[k].0) else { continue };
            writeln!(w, "\nBy {title}, at the {}% cutoff (AI caught of AI; real flagged of real):\n", 100.0 * f)?;
            writeln!(w, "| {title} | AI | Real |{}", sliced.iter().map(|&ai| format!(" {} |", arms[ai].name)).collect::<String>())?;
            writeln!(w, "|---|---|---|{}", "---|".repeat(sliced.len()))?;
            for (slice, idx) in &slices {
                let (nf, nr) = (idx.iter().filter(|&&i| labels[i] == 1).count(), idx.iter().filter(|&&i| labels[i] == 0).count());
                let cells: String = sliced
                    .iter()
                    .map(|&ai| match cutoffs[ai][fi] {
                        Some((cut, _)) => {
                            let hit = |l: u8| idx.iter().filter(|&&i| labels[i] == l && scores[ai][i] > cut).count();
                            format!(" {}; {} |", if nf > 0 { format!("{} ({})", hit(1), pct(hit(1) as f64 / nf as f64)) } else { "–".into() }, if nr > 0 { format!("{} ({})", hit(0), pct(hit(0) as f64 / nr as f64)) } else { "–".into() })
                        }
                        None => " – |".into(),
                    })
                    .collect();
                // Slices sort by a leading digit that is not part of their name.
                writeln!(w, "| {} | {nf} | {nr} |{cells}", slice.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start())?;
            }
        }
    }
    writeln!(w, "\n### What the covariates alone say\n")?;
    writeln!(w, "AUROC of each covariate used as a score by itself on these clips (0.5: the classes do not differ in it; near 0 or 1: they do, and a head can score by it without looking at a frame). Fit `--kind covariates` for the joint version.\n")?;
    writeln!(w, "| Covariate | AUROC | Median, AI | Median, real |")?;
    writeln!(w, "|---|---|---|---|")?;
    for (k, name) in COVARIATES.iter().enumerate() {
        let v: Vec<f32> = cov.iter().map(|c| c.vector()[k]).collect();
        let median = |l: u8| {
            let mut x: Vec<f32> = v.iter().zip(&labels).filter(|(_, &y)| y == l).map(|(&v, _)| v).collect();
            x.sort_by(f32::total_cmp);
            if x.is_empty() { "–".to_string() } else { format!("{:.3}", x[x.len() / 2]) }
        };
        let auroc = metrics::auroc(&v, &labels);
        writeln!(w, "| {name} | {} | {} | {} |", if auroc.is_finite() { format!("{auroc:.3}") } else { "–".into() }, median(1), median(0))?;
    }

    if let Some(dir) = &o.out_dir {
        std::fs::create_dir_all(dir)?;
        let mut systems = String::new();
        for (ai, a) in arms.iter().enumerate() {
            let file = dir.join(format!("{}.tsv", a.name.replace(['/', ' '], "_")));
            let rows: String = common.iter().zip(&scores[ai]).map(|(&c, &z)| {
                let it = &test.clips[c].item;
                format!("{}\t{}\t{}\t{}\t{}\t{z}\n", it.path.display(), it.label, it.generator, it.dataset, metrics::sigmoid(z))
            }).collect();
            std::fs::write(&file, rows)?;
            // The first rate's cutoff as a probability, which is what `report` thresholds.
            let cut = cutoffs[ai].first().copied().flatten().map_or(String::new(), |(c, _)| format!("@{}", metrics::sigmoid(c)));
            let _ = write!(systems, " --system {}={}{cut}", a.name, file.display());
        }
        writeln!(w, "\nScore files written to {}. For clustered intervals and slices by source or family (cutoffs are those of the {}% rate):\n\n```sh\ntintype report --operating-points{systems} --groups <test manifest> --cluster-by parent --slice-by source,family\n```", dir.display(), o.fprs.first().map_or(0.0, |f| 100.0 * f))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::ViewSpec;
    use crate::ingest::Item;
    use crate::temporal::{Meta, Stream};

    const DIM: usize = 6;
    const GRID: usize = 2;

    fn spec(kind: Kind) -> HeadSpec {
        HeadSpec { format: FORMAT, kind, base: Base::Frame, block: 12, grid: GRID, dim: DIM, hidden: 5, follow: 0, time_norm: TimeNorm::None, trained_shuffled: false, cut_rule: CutRule::default(), fit: None }
    }

    fn meta(clips: usize, frames: usize) -> Meta {
        Meta {
            format: crate::temporal::FORMAT,
            weights: "runs/ws7/video.safetensors".into(),
            weights_sha256: "0".repeat(64),
            device: "cpu".into(),
            img: 224,
            dim: DIM,
            blocks: vec![12],
            grid: GRID,
            record: (1 + GRID * GRID) * DIM,
            frames,
            max_seconds: 30.0,
            follow: 0,
            blur: 1,
            views: vec![ViewSpec::Fit { resize: 256, crop: 224 }, ViewSpec::Native { crop: 224 }],
            fit_view: 0,
            clips,
            frame_records: (clips * frames) as u64,
            unreadable: 0,
            wall_s: 0.0,
            model_s: 0.0,
            decode_s: 0.0,
            threads: 1,
            repeat_below: crate::temporal::REPEAT_BELOW,
        }
    }

    /// A set whose classes differ only in the order of their frames: every
    /// clip holds the same eight token patterns, each cell's first value
    /// rising steadily through an AI clip and visiting the same levels in a
    /// scrambled order in a real one. The frame logits say nothing.
    fn ordered_set(clips: usize, seed: u64) -> FeatureSet {
        let frames = 8;
        let mut rng = crate::rng::Rng::new(seed);
        let (mut rows, mut all_frames, mut tokens) = (Vec::new(), Vec::new(), Vec::new());
        for c in 0..clips {
            let label = (c % 2) as u8;
            let mut level: Vec<usize> = (0..frames).collect();
            if label == 0 {
                // Any order but the rising one.
                while level.windows(2).all(|w| w[0] < w[1]) {
                    rng.shuffle(&mut level);
                }
            }
            let mut t = Vec::new();
            for &l in &level {
                for cell in 0..1 + GRID * GRID {
                    t.push(l as f32 * 0.5 - 1.75);
                    t.extend((1..DIM).map(|k| ((l * 7 + cell * 3 + k) % 5) as f32 * 0.2 - 0.4));
                }
            }
            tokens.push(t);
            rows.push(Clip { item: Item { path: format!("/clips/{c}.mp4").into(), label, generator: if label == 1 { "gen".into() } else { "real:cam".into() }, dataset: "toy".into(), extra: Vec::new() }, first_record: 0, frames, width: 640, height: 360, fps: 25.0, duration: 8.0, stream: Stream { frames_read: 176, gap_median: 0.04, gap_min: 0.04, gap_max: 0.04, median_diff: 1.0, repeats: 0 } });
            all_frames.push((0..frames).map(|k| Frame { index: k * 25, pts: k as f64, fit_logit: rng.range(-1.0, 1.0), frame_logit: rng.range(-1.0, 1.0), view_logits: vec![0.0, 0.0], gap_frames: if k > 0 { 25 } else { 0 }, gap_s: if k > 0 { 1.0 } else { 0.0 }, fit_diff: 3.0, max_diff: 2.0, sum_diff: 25.0, repeats: 0 }).collect());
        }
        FeatureSet::in_memory(meta(clips, frames), rows, all_frames, tokens)
    }

    #[test]
    fn a_new_head_is_the_frame_mean() {
        let set = ordered_set(4, 3);
        for kind in Kind::ALL {
            let head = Head::new(spec(kind), 7);
            for c in 0..4 {
                let x = input(&set, c, &head.spec, None).unwrap();
                let mean = x.z.iter().sum::<f32>() / x.z.len() as f32;
                assert_eq!(head.logit(&x), mean, "{}", kind.name());
            }
        }
    }

    #[test]
    fn gradients_match_finite_differences() {
        let set = ordered_set(2, 5);
        for kind in Kind::ALL {
            for time_norm in [TimeNorm::None, TimeNorm::Rate] {
                let mut head = Head::new(HeadSpec { time_norm, ..spec(kind) }, 11);
                // Away from the all-zero start, where half the gradients vanish.
                let mut rng = crate::rng::Rng::new(13);
                for (name, v) in head.p.iter_mut() {
                    if name != "sd" {
                        v.iter_mut().for_each(|x| *x += rng.range(-0.3, 0.3));
                    }
                }
                // The clip with scrambled frames: its pairs differ from one another, so one
                // hidden unit switching inside a step moves one row, not all of them.
                let x = input(&set, 0, &head.spec, None).unwrap();
                let (target, weight) = (1u8, 1.7f32);
                let loss = |h: &Head| {
                    let z = h.logit(&x) as f64;
                    weight as f64 * (z.max(0.0) - z * target as f64 + (-z.abs()).exp().ln_1p())
                };
                let (_, grad) = head.pass(&x, Some((target, weight)));
                assert_eq!(grad.keys().collect::<Vec<_>>(), head.p.keys().filter(|k| !Head::fixed(k)).collect::<Vec<_>>());
                let (mut checked, mut close) = (0, 0);
                for (name, g) in &grad {
                    for i in (0..g.len()).step_by(g.len() / 7 + 1) {
                        let eps = 4e-3f32;
                        let (mut up, mut down) = (head.clone(), head.clone());
                        up.p.get_mut(name).unwrap()[i] += eps;
                        down.p.get_mut(name).unwrap()[i] -= eps;
                        let numeric = (loss(&up) - loss(&down)) / (2.0 * eps as f64);
                        checked += 1;
                        // A ReLU that switches inside the step makes the odd entry disagree; nearly all must not.
                        close += ((numeric - g[i] as f64).abs() <= 2e-3 + 0.03 * numeric.abs()) as usize;
                    }
                }
                assert!(close * 10 >= checked * 9, "{} with {time_norm:?}: {close} of {checked} gradient entries agree", kind.name());
            }
        }
    }

    #[test]
    fn order_invariant_heads_ignore_the_order_and_temporal_heads_do_not() {
        let set = ordered_set(2, 5);
        for kind in Kind::ALL {
            let mut head = Head::new(spec(kind), 11);
            let mut rng = crate::rng::Rng::new(17);
            for (name, v) in head.p.iter_mut() {
                if name != "sd" {
                    v.iter_mut().for_each(|x| *x += rng.range(-0.5, 0.5));
                }
            }
            let in_order = head.logit(&input(&set, 1, &head.spec, None).unwrap());
            let shuffled = head.logit(&input(&set, 1, &head.spec, Some(&[3, 0, 6, 1, 7, 2, 5, 4])).unwrap());
            if kind.temporal() {
                assert!((in_order - shuffled).abs() > 1e-3, "{} did not notice the shuffle", kind.name());
            } else {
                assert!((in_order - shuffled).abs() < 1e-5, "{} changed by {}", kind.name(), in_order - shuffled);
            }
        }
    }

    #[test]
    fn repeated_samples_are_not_pairs_and_paired_dumps_pair_within_pairs() {
        let head = Head::new(spec(Kind::Diff), 1);
        let clip = |index: Vec<usize>| ClipInput { z: vec![0.0; index.len()], pts: index.iter().map(|&i| i as f64 / 25.0).collect(), index, tok: vec![0.5; 6 * 5 * DIM], cov: [0.0; 6] };
        assert_eq!(head.pairs(&clip(vec![0, 3, 3, 9])), vec![(0, 1), (2, 3)]);
        let paired = Head::new(HeadSpec { follow: 1, ..spec(Kind::Diff) }, 1);
        assert_eq!(paired.pairs(&clip(vec![0, 1, 10, 11, 20, 21])), vec![(0, 1), (2, 3), (4, 5)]);
        // A one-frame clip has no pairs, and the head falls back to the frame mean.
        let one = clip(vec![4]);
        assert!(head.pairs(&one).is_empty());
        assert_eq!(head.logit(&one), 0.0);
    }

    #[test]
    fn a_temporal_head_learns_an_order_only_cue_that_order_invariant_heads_cannot() {
        let (train, test) = (ordered_set(160, 1), ordered_set(80, 2));
        let o = FitOptions { epochs: 30, lr: 5e-3, weight_decay: 0.0, batch: 32, seed: 1, shuffle_frames: false };
        let auroc = |head: &Head, shuffle: Option<u64>| metrics::auroc(&logits(&test, head, shuffle).unwrap(), &test.labels());
        // The difference head separates the classes on clips it has not seen ...
        let diff = fit(&train, HeadSpec { hidden: 16, ..spec(Kind::Diff) }, &o, None, |_| {}).unwrap();
        assert!(auroc(&diff, None) > 0.95, "difference head AUROC {}", auroc(&diff, None));
        // ... and loses that with the frames shuffled.
        assert!(auroc(&diff, Some(9)) < 0.75, "shuffled difference head AUROC {}", auroc(&diff, Some(9)));
        // The same network without pairing sees the same tokens in both classes.
        let patch = fit(&train, HeadSpec { hidden: 16, ..spec(Kind::Patch) }, &o, None, |_| {}).unwrap();
        assert!(auroc(&patch, None) < 0.7, "patch head AUROC {}", auroc(&patch, None));
        // So does a difference head fitted on shuffled frames: the order-blind twin.
        let twin = fit(&train, HeadSpec { hidden: 16, ..spec(Kind::Diff) }, &FitOptions { shuffle_frames: true, ..o.clone() }, None, |_| {}).unwrap();
        assert!(twin.spec.trained_shuffled && auroc(&twin, None) < 0.75, "order-blind twin AUROC {}", auroc(&twin, None));
        // A fit is the same numbers every time.
        let again = fit(&train, HeadSpec { hidden: 16, ..spec(Kind::Diff) }, &o, None, |_| {}).unwrap();
        assert_eq!(again, diff);
    }

    #[test]
    fn heads_save_and_load_and_refuse_a_dump_they_do_not_fit() {
        let set = ordered_set(8, 4);
        let dir = std::env::temp_dir().join(format!("tintype-test-head-{}", std::process::id()));
        let head = fit(&set, spec(Kind::Concat), &FitOptions { epochs: 2, ..Default::default() }, Some(&set), |_| {}).unwrap();
        head.save(&dir.join("heads/concat.v1")).unwrap();
        assert!(dir.join("heads/concat.v1.json").exists() && dir.join("heads/concat.v1.safetensors").exists());
        let back = Head::load(&dir.join("heads/concat.v1.json")).unwrap();
        assert_eq!(back, head);
        assert_eq!(back.spec.fit.as_ref().map(|f| (f.clips, f.ai, f.epochs)), Some((8, 4, 2)));
        // 2 * 6 + 1 inputs to 5 hidden units, their biases and output weights, and a, b.
        assert_eq!(back.parameters(), 13 * 5 + 5 + 5 + 2);
        let _ = std::fs::remove_dir_all(&dir);
        // Another grid, another block, or pairs the dump was not sampled as.
        assert!(check(&set, &HeadSpec { grid: 7, ..spec(Kind::Patch) }).is_err());
        assert!(check(&set, &HeadSpec { block: 9, ..spec(Kind::Patch) }).is_err());
        assert!(check(&set, &HeadSpec { follow: 1, ..spec(Kind::Diff) }).is_err());
        assert!(check(&set, &HeadSpec { follow: 1, grid: 7, block: 9, ..spec(Kind::Covariates) }).is_ok());
        // One class only cannot be fitted.
        let mut reals = ordered_set(4, 4);
        reals.clips.iter_mut().for_each(|c| c.item.label = 0);
        assert!(fit(&reals, spec(Kind::Patch), &FitOptions::default(), None, |_| {}).is_err());
    }

    #[test]
    fn covariates_count_cuts_repeats_and_cadence() {
        let stream = Stream { frames_read: 100, gap_median: 1.0 / 30.0, gap_min: 1.0 / 30.0, gap_max: 0.1, median_diff: 1.5, repeats: 4 };
        let clip = Clip { item: Item::default(), first_record: 0, frames: 4, width: 640, height: 360, fps: 25.0, duration: 4.0, stream };
        let f = |gap_frames: i64, max_diff: f32| Frame { gap_frames, gap_s: gap_frames as f64 / 30.0, max_diff, ..Default::default() };
        // The first sample has no stretch before it; then a quiet stretch, a cut, and a sample of the same frame again.
        let frames = vec![f(0, 0.0), f(30, 4.0), f(30, 40.0), f(0, 0.0)];
        let c = Covariates::of(&clip, &frames, CutRule::default());
        assert!((c.fps - 30.0).abs() < 1e-9 && (c.jitter - 3.0).abs() < 1e-9, "{c:?}");
        assert_eq!((c.cuts, c.stretches, c.resampled, c.repeats, c.frames_read), (1, 2, 1, 4, 100));
        assert!((c.sample_gap - 2.0 / 3.0).abs() < 1e-9);
        // A large difference in a clip that moves that much everywhere is not a cut.
        let busy = Clip { stream: Stream { median_diff: 10.0, ..stream }, ..clip.clone() };
        assert_eq!(Covariates::of(&busy, &frames, CutRule::default()).cuts, 0);
        let v = c.vector();
        assert!((v[0] - 30f32.ln()).abs() < 1e-5 && (v[2] - 0.04).abs() < 1e-6 && (v[3] - 0.5).abs() < 1e-6);
        let s = c.slices((1.0, 2.0));
        assert_eq!((s[0].1, s[1].1, s[2].1, s[3].1, s[4].1), ("one or more", "some", "2 middle", "4 27 to 35 fps", "3 0.5 to 1.5 s"));
        // Without timestamps the container's rate stands in.
        assert_eq!(Covariates::of(&Clip { stream: Stream::default(), ..clip }, &frames[..1], CutRule::default()).fps, 25.0);
    }

    #[test]
    fn evaluation_reports_every_arm_the_controls_and_the_slices() {
        let (train, cal, test) = (ordered_set(120, 1), ordered_set(60, 2), ordered_set(60, 3));
        let o = FitOptions { epochs: 25, lr: 5e-3, weight_decay: 0.0, batch: 32, seed: 1, shuffle_frames: false };
        let heads = vec![("diff".to_string(), fit(&train, HeadSpec { hidden: 16, ..spec(Kind::Diff) }, &o, None, |_| {}).unwrap()), ("patch".to_string(), fit(&train, HeadSpec { hidden: 16, ..spec(Kind::Patch) }, &o, None, |_| {}).unwrap())];
        let dir = std::env::temp_dir().join(format!("tintype-test-eval-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // An outside system that scored only the first 50 test clips.
        let ext = dir.join("more-frames.tsv");
        std::fs::write(&ext, test.clips.iter().take(50).map(|c| format!("{}\t{}\tg\td\t0.5\t{}\t{}\n", c.item.path.display(), c.item.label, c.item.label as f32 * 2.0 - 1.0, 0.25)).collect::<String>()).unwrap();
        let report = evaluate(&test, Some(&cal), &heads, &[External { name: "more-frames".into(), test: ext, calibrate: None }], &EvalOptions { out_dir: Some(dir.join("scores")), ..Default::default() }).unwrap();
        assert!(report.starts_with("50 clips scored by all 7 arms: 25 AI, 25 real"), "{report}");
        for needle in ["| frame-mean |", "| fit-mean |", "| diff |", "| diff+shuffled |", "| patch+shuffled |", "| more-frames |", "### Operating at a 1% false-positive cutoff", "### Operating at a 0.1% false-positive cutoff", "### Shuffled-order control", "By cuts", "By repeated frames", "By motion", "By frame rate", "By time between samples", "calibration clips", "these clips (exploratory)", "**Exploratory.**", "**Not resolved.**", "(0 of 30 real allowed above)", "| log frame rate |", "Recall difference (95% interval, clustered)", "bootstrap interval over 50 clusters", " points |"] {
            assert!(report.contains(needle), "missing `{needle}` in:\n{report}");
        }
        // The outside system is perfect on these clips; the temporal head catches the AI clips at a cutoff frozen on other clips.
        let row = |name: &str| report.lines().find(|l| l.starts_with(&format!("| {name} |"))).unwrap().to_string();
        assert!(row("more-frames").contains("25 of 25"), "{}", row("more-frames"));
        assert!(row("diff").contains("| 25 of 25 |") || row("diff").contains("| 24 of 25 |"), "{}", row("diff"));
        // The order-invariant head's shuffled row is its own row again.
        let patch = report.lines().find(|l| l.starts_with("| patch | no |")).unwrap();
        assert!(patch.contains("| 0.0000 | 0.0000 |"), "{patch}");
        // One score file per arm, readable by `report`, holding the common clips.
        for arm in ["frame-mean", "fit-mean", "diff", "diff+shuffled", "patch", "patch+shuffled", "more-frames"] {
            let scores = crate::report::read_scores(&dir.join("scores").join(format!("{arm}.tsv"))).unwrap();
            assert_eq!(scores.len(), 50, "{arm}");
            assert!(scores.values().all(|r| r.logits.len() == 1 && (metrics::sigmoid(r.logits[0]) - r.prob).abs() < 1e-5));
        }
        // Without calibration clips everything is exploratory, and a real-only set (the blur control) still reports.
        let mut reals = ordered_set(20, 6);
        reals.clips.iter_mut().for_each(|c| c.item.label = 0);
        let only_real = evaluate(&reals, Some(&cal), &heads, &[], &EvalOptions::default()).unwrap();
        assert!(only_real.contains("0 AI, 20 real") && only_real.contains("of 20 |"), "{only_real}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
