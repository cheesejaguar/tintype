//! The detector: weights + preprocessing views + calibration, and a
//! multi-threaded decode -> batch -> GPU scoring pipeline.
use crate::image_io::{self, Rgb, View};
use crate::metrics::sigmoid;
#[cfg(target_os = "macos")]
use crate::mps::{Graph, Precision, T};
#[cfg(target_os = "macos")]
use crate::mps_vit::{build_head, build_vit, Frozen};
use crate::vit::VitConfig;
use crate::weights::Weights;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ViewSpec {
    Fit { resize: u32, crop: u32 },
    Native { crop: u32 },
    NativeAt { crop: u32, fx: u32, fy: u32 },
    NativeRich { crop: u32, rank: u32 },
    FitPillow { resize: u32, crop: u32 },
    SquashPillow { size: u32 },
    CenterPad { crop: u32 },
}

impl From<ViewSpec> for View {
    fn from(v: ViewSpec) -> Self {
        match v {
            ViewSpec::Fit { resize, crop } => View::Fit { resize, crop },
            ViewSpec::Native { crop } => View::Native { crop },
            ViewSpec::NativeAt { crop, fx, fy } => View::NativeAt { crop, fx, fy },
            ViewSpec::NativeRich { crop, rank } => View::NativeRich { crop, rank },
            ViewSpec::FitPillow { resize, crop } => View::FitPillow { resize, crop },
            ViewSpec::SquashPillow { size } => View::SquashPillow { size },
            ViewSpec::CenterPad { crop } => View::CenterPad { crop },
        }
    }
}

/// Relative centres (thousandths) of the four off-centre native crops.
const QUADS: [(u32, u32); 4] = [(280, 280), (720, 280), (280, 720), (720, 720)];

/// The default views: the whole image, the centre native-resolution crop and
/// the four most textured native-resolution crops.
pub fn thorough_views(size: u32, fit_resize: u32) -> Vec<ViewSpec> {
    let mut v = vec![ViewSpec::Fit { resize: fit_resize, crop: size }, ViewSpec::Native { crop: size }];
    v.extend((0..4).map(|rank| ViewSpec::NativeRich { crop: size, rank }));
    v
}

/// The fast views: the whole image plus the single most textured native crop.
pub fn fast_views(size: u32, fit_resize: u32) -> Vec<ViewSpec> {
    vec![ViewSpec::Fit { resize: fit_resize, crop: size }, ViewSpec::NativeRich { crop: size, rank: 0 }]
}

/// A cheaper alternative view set with its own calibration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub views: Vec<ViewSpec>,
    #[serde(default = "one")]
    pub logit_scale: f32,
    #[serde(default)]
    pub logit_bias: f32,
}

/// One checkpoint of the early exit: after `views` views, stop if their mean
/// logit is at least `above` over the decision threshold or at least `below`
/// under it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExitStage {
    pub views: usize,
    pub below: f32,
    pub above: f32,
}

/// Early exit for the six-view photo profiles: the views are looked at in
/// `order`, and an image whose answer is already clear part-way through is
/// not shown the rest. The verdict and probability then come from
/// the views that were looked at.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EarlyExit {
    /// Indices into the profile's view list, in the order they are looked at.
    pub order: Vec<usize>,
    pub stages: Vec<ExitStage>,
}

impl EarlyExit {
    /// The schedule fitted on the development sets: most textured native crop
    /// first, then the whole image, then the remaining crops, with a check
    /// after every view.
    pub fn standard() -> Self {
        Self {
            order: vec![2, 0, 1, 3, 4, 5],
            stages: vec![
                ExitStage { views: 1, below: 7.0, above: 9.0 },
                ExitStage { views: 2, below: 5.0, above: 5.0 },
                ExitStage { views: 3, below: 3.5, above: 3.5 },
                ExitStage { views: 4, below: 2.5, above: 2.5 },
                ExitStage { views: 5, below: 2.0, above: 2.0 },
            ],
        }
    }

    /// Whether to stop after `seen` (the logits of the views looked at so
    /// far), given the profile's threshold on the mean logit.
    pub fn decided(&self, seen: &[f32], threshold: f32) -> bool {
        let Some(st) = self.stages.iter().find(|st| st.views == seen.len()) else { return false };
        let margin = seen.iter().sum::<f32>() / seen.len() as f32 - threshold;
        margin >= st.above || margin <= -st.below
    }

    /// View counts at which the next decision is taken: each stage, then all.
    pub fn checkpoints(&self, total: usize) -> Vec<usize> {
        let mut c: Vec<usize> = self.stages.iter().map(|s| s.views).filter(|&v| v < total).collect();
        c.push(total);
        c
    }

    /// The logits early exit would have looked at, from all of an image's
    /// view logits (in the profile's view order): the same arithmetic as
    /// running it.
    pub fn looked_at(&self, all: &[f32], threshold: f32) -> Vec<f32> {
        let mut seen = Vec::with_capacity(all.len());
        for n in self.checkpoints(all.len()) {
            while seen.len() < n {
                seen.push(all[self.order[seen.len()]]);
            }
            if self.decided(&seen, threshold) {
                break;
            }
        }
        seen
    }
}

/// Parse a comma-separated view list: `fit`, `native`, `quads` (four
/// off-centre native crops). Used to try alternative view sets.
pub fn parse_views(list: &str, size: u32, fit_resize: u32) -> Result<Vec<ViewSpec>> {
    let mut out = Vec::new();
    for tok in list.split(',').map(str::trim).filter(|t| !t.is_empty()) {
        match tok {
            "fit" => out.push(ViewSpec::Fit { resize: fit_resize, crop: size }),
            "native" => out.push(ViewSpec::Native { crop: size }),
            "quads" => out.extend(QUADS.map(|(fx, fy)| ViewSpec::NativeAt { crop: size, fx, fy })),
            t if t.starts_with("rich") => {
                let k: u32 = t[4..].parse().map_err(|_| anyhow::anyhow!("use richN, e.g. rich5"))?;
                out.extend((0..k).map(|rank| ViewSpec::NativeRich { crop: size, rank }));
            }
            "grid3" => {
                for fy in [200, 500, 800] {
                    for fx in [200, 500, 800] {
                        out.push(ViewSpec::NativeAt { crop: size, fx, fy });
                    }
                }
            }
            other => anyhow::bail!("unknown view `{other}` (use fit, native, quads, grid3)"),
        }
    }
    anyhow::ensure!(!out.is_empty(), "empty view list");
    Ok(out)
}

/// Optional second stage for video. A clip whose first-stage logit lies
/// strictly between `lo` and `hi` is also scored by the WaveRep network (its
/// released weights and pipeline: the first 64 frames at native resolution),
/// and it is flagged when the two clip logits add up to more than `threshold`.
/// Clips outside the band keep the first-stage verdict.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecondOpinion {
    pub lo: f32,
    pub hi: f32,
    pub threshold: f32,
}

impl SecondOpinion {
    pub fn wanted(&self, first_logit: f32) -> bool {
        first_logit > self.lo && first_logit < self.hi
    }

    /// P(AI-generated) for a clip inside the band, from both clip logits.
    pub fn probability(&self, first_logit: f32, second_logit: f32) -> f32 {
        sigmoid(0.5 * (first_logit + second_logit - self.threshold))
    }
}

/// How clips are analysed: which frames, which views of each frame, and the
/// clip-level calibration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoSpec {
    pub frames: usize,
    pub views: Vec<ViewSpec>,
    #[serde(default = "one")]
    pub logit_scale: f32,
    #[serde(default)]
    pub logit_bias: f32,
    /// Band and threshold of the optional second stage (`--second-opinion`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_opinion: Option<SecondOpinion>,
}

impl VideoSpec {
    /// One whole-frame view plus five native-resolution crops (centre and
    /// four quadrants) on each of 16 frames.
    pub fn standard(size: u32, fit_resize: u32) -> Self {
        Self { frames: 16, views: thorough_views(size, fit_resize), logit_scale: 1.0, logit_bias: 0.0, second_opinion: None }
    }
}

/// Sidecar `<weights>.json` describing how to run a checkpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelSpec {
    pub img: usize,
    pub views: Vec<ViewSpec>,
    /// Calibration applied to the mean logit: `p = sigmoid(scale * z + bias)`.
    #[serde(default = "one")]
    pub logit_scale: f32,
    #[serde(default)]
    pub logit_bias: f32,
    #[serde(default = "half")]
    pub threshold: f32,
    /// Prefix stripped from tensor names (e.g. `vit.` for upstream checkpoints).
    #[serde(default)]
    pub prefix: String,
    /// Video profile; absent means "treat frames like photos".
    #[serde(default)]
    pub video: Option<VideoSpec>,
    /// Two-view photo profile for when throughput matters more than recall.
    #[serde(default)]
    pub fast: Option<Profile>,
    /// Same views as the default profile, calibrated to flag more: for
    /// triage, where a missed fake costs more than a false alarm.
    #[serde(default)]
    pub sensitive: Option<Profile>,
    /// Same views again, calibrated to flag less: for when a positive
    /// verdict has to stand on its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strict: Option<Profile>,
    /// Early exit for the six-view photo profiles; absent means every view
    /// of every image is looked at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub early_exit: Option<EarlyExit>,
    /// Network architecture; empty means the ViT-S/16 Veracity ships.
    #[serde(default)]
    pub arch: String,
}
fn one() -> f32 {
    1.0
}
fn half() -> f32 {
    0.5
}

impl ModelSpec {
    /// The SwinV2 reference detector (`haywoodsloan/ai-image-detector-deploy`):
    /// squash to 256x256 with Pillow's bicubic filter; class 0 is "artificial".
    pub fn swinv2() -> Self {
        Self {
            img: 256,
            views: vec![ViewSpec::SquashPillow { size: 256 }],
            logit_scale: 1.0,
            logit_bias: 0.0,
            threshold: 0.5,
            prefix: String::new(),
            video: None,
            fast: None,
            sensitive: None,
            strict: None,
            early_exit: None,
            arch: "swinv2_large_w16_256".into(),
        }
    }

    /// WaveRep's reference pipeline: 504 px centre crop (zero padded, no
    /// resizing) of each frame through a DINOv2 ViT-B/14 with registers.
    pub fn waverep() -> Self {
        Self {
            img: 504,
            views: vec![ViewSpec::CenterPad { crop: 504 }],
            logit_scale: 1.0,
            logit_bias: 0.0,
            threshold: 0.5,
            prefix: "model.".into(),
            video: None,
            fast: None,
            sensitive: None,
            strict: None,
            early_exit: None,
            arch: "vit_base_patch14_reg4_dinov2".into(),
        }
    }

    /// The Community Forensics baseline's own preprocessing.
    pub fn commfor(img: usize) -> Self {
        let resize = if img == 384 { 440 } else { 256 };
        Self { img, views: vec![ViewSpec::FitPillow { resize, crop: img as u32 }], logit_scale: 1.0, logit_bias: 0.0, threshold: 0.5, prefix: "vit.".into(), video: None, fast: None, sensitive: None, strict: None, early_exit: None, arch: String::new() }
    }

    pub fn sidecar(weights: &Path) -> PathBuf {
        weights.with_extension("json")
    }

    pub fn load_for(weights: &Path) -> Result<Self> {
        let p = Self::sidecar(weights);
        Ok(serde_json::from_str(&std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?)?)
    }

    pub fn save_for(&self, weights: &Path) -> Result<()> {
        Ok(std::fs::write(Self::sidecar(weights), serde_json::to_string_pretty(self)?)?)
    }
}

/// The video checkpoint that goes with a photo checkpoint:
/// `<name>-video.safetensors` beside it when that exists, otherwise the photo
/// checkpoint itself (its own video profile).
pub fn video_weights_for(weights: &Path) -> std::path::PathBuf {
    let stem = weights.file_stem().and_then(|s| s.to_str()).unwrap_or("veracity");
    let sibling = weights.with_file_name(format!("{stem}-video.safetensors"));
    if sibling.exists() {
        sibling
    } else {
        weights.to_path_buf()
    }
}

#[derive(Clone)]
pub struct Preprocessor {
    views: Vec<View>,
    view_len: usize,
}

impl Preprocessor {
    /// Write all views of `img` into `out` (len `views * 3 * img * img`).
    pub fn apply(&self, img: &Rgb, out: &mut [f32]) {
        for (crop, chunk) in self.crops(img).iter().zip(out.chunks_exact_mut(self.view_len)) {
            crop.write_chw(chunk);
        }
    }

    /// Every view of `img` as a cropped image, in view order.
    pub fn crops(&self, img: &Rgb) -> Vec<Rgb> {
        // Texture ranking is computed once and shared by all "rich" views.
        let need = self.views.iter().filter_map(|v| if let View::NativeRich { crop, rank } = v { Some((*crop, *rank as usize + 1)) } else { None }).max_by_key(|x| x.1);
        let rich = need.filter(|(crop, _)| img.w.min(img.h) >= *crop).map(|(crop, k)| image_io::rich_windows(img, crop, k));
        self.views.iter().map(|v| v.apply_with(img, rich.as_deref())).collect()
    }
}

/// Where the network runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    /// Apple GPU through MPSGraph, half precision (fastest; macOS only).
    Gpu,
    /// Apple GPU, single precision.
    GpuF32,
    /// Portable CPU engine.
    Cpu,
}

impl Device {
    pub fn best() -> Self {
        if cfg!(target_os = "macos") {
            Device::Gpu
        } else {
            Device::Cpu
        }
    }
}

enum Engine {
    #[cfg(target_os = "macos")]
    Mps { g: Graph, inp: T, out: T },
    /// The engine in `cpu_vit`: Accelerate on macOS, pure Rust elsewhere.
    Cpu { vit: crate::cpu_vit::CpuVit },
}

pub struct Detector {
    engine: Engine,
    pub batch: usize,
    pub spec: ModelSpec,
}

#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub images: usize,
    pub wall_s: f64,
    /// Seconds spent inside GPU runs.
    pub model_s: f64,
}

impl Detector {
    pub fn new(w: &Weights, spec: ModelSpec, batch: usize, device: Device) -> Result<Self> {
        #[cfg(target_os = "macos")]
        if spec.arch == "swinv2_large_w16_256" {
            anyhow::ensure!(device != Device::Cpu, "the SwinV2 reference network runs on the GPU engine only");
            let g = Graph::new(if device == Device::Gpu { Precision::F16 } else { Precision::F32 })?;
            let s = spec.img as isize;
            let inp = g.placeholder(&[batch as isize, 3, s, s]);
            let out = g.cast_out(&crate::swin::build_swin(&g, w, &crate::swin::SwinConfig::large_w16_256(), &g.cast_in(&inp), batch)?);
            return Ok(Self { engine: Engine::Mps { g, inp, out }, batch, spec });
        }
        let cfg = VitConfig::by_name(&spec.arch, spec.img).ok_or_else(|| anyhow::anyhow!("unknown architecture `{}`", spec.arch))?;
        let engine = match device {
            #[cfg(target_os = "macos")]
            Device::Gpu | Device::GpuF32 => {
                let g = Graph::new(if device == Device::Gpu { Precision::F16 } else { Precision::F32 })?;
                let s = spec.img as isize;
                let inp = g.placeholder(&[batch as isize, 3, s, s]);
                let v = build_vit(&g, w, &mut Frozen, &cfg, &g.cast_in(&inp), batch)?;
                let out = g.cast_out(&build_head(&g, w, &mut Frozen, &v.cls)?);
                Engine::Mps { g, inp, out }
            }
            #[cfg(not(target_os = "macos"))]
            Device::Gpu | Device::GpuF32 => anyhow::bail!("the GPU engine is only available on macOS; use the CPU engine"),
            Device::Cpu => Engine::Cpu { vit: crate::cpu_vit::CpuVit::new(w, cfg)? },
        };
        Ok(Self { engine, batch, spec })
    }

    pub fn load(weights: &Path, spec: Option<ModelSpec>, batch: usize, device: Device) -> Result<Self> {
        let spec = match spec {
            Some(s) => s,
            None => ModelSpec::load_for(weights)?,
        };
        let is_torch = weights.extension().and_then(|e| e.to_str()).is_some_and(|e| e == "ckpt" || e == "pth" || e == "pt");
        let w = if is_torch { Weights::load_torch(weights, &spec.prefix)? } else { Weights::load(weights, &spec.prefix)? };
        Self::new(&w, spec, batch, device)
    }

    fn view_len(&self) -> usize {
        3 * self.spec.img * self.spec.img
    }

    /// Raw logits for `batch` preprocessed views (CHW, concatenated).
    pub fn logits(&self, x: &[f32]) -> Vec<f32> {
        match &self.engine {
            #[cfg(target_os = "macos")]
            Engine::Mps { g, inp, out } => {
                let s = self.spec.img as isize;
                g.run(&[(inp, x, &[self.batch as isize, 3, s, s])], &[out.clone()], &[]).remove(0)
            }
            Engine::Cpu { vit } => vit.forward(x),
        }
    }

    /// The preprocessing half of the detector; unlike the GPU graph it is
    /// `Send + Sync`, so decode workers can share it.
    pub fn preprocessor(&self) -> Preprocessor {
        Preprocessor { views: self.spec.views.iter().map(|&v| v.into()).collect(), view_len: self.view_len() }
    }

    /// Switch to the fast photo profile. Call before building a detector
    /// (the profile changes how many views each image needs).
    pub fn use_fast_profile(spec: &mut ModelSpec) {
        match spec.fast.clone() {
            Some(f) => {
                spec.views = f.views;
                spec.logit_scale = f.logit_scale;
                spec.logit_bias = f.logit_bias;
            }
            None => spec.views = fast_views(spec.img as u32, 256),
        }
    }

    /// Switch to the sensitive photo profile (same views, looser threshold).
    pub fn use_sensitive_profile(spec: &mut ModelSpec) {
        if let Some(p) = spec.sensitive.clone() {
            spec.views = p.views;
            spec.logit_scale = p.logit_scale;
            spec.logit_bias = p.logit_bias;
        }
    }

    /// Switch to the strict photo profile (same views, tighter threshold).
    pub fn use_strict_profile(spec: &mut ModelSpec) {
        if let Some(p) = spec.strict.clone() {
            spec.views = p.views;
            spec.logit_scale = p.logit_scale;
            spec.logit_bias = p.logit_bias;
        }
    }

    /// The active profile's decision threshold on the mean view logit.
    pub fn threshold_logit(&self) -> f32 {
        -self.spec.logit_bias / self.spec.logit_scale
    }

    /// Logits for the first `n` views in `x`, whatever the engine's batch size.
    fn logits_n(&self, x: &[f32], n: usize) -> Vec<f32> {
        let vl = self.view_len();
        let mut out = Vec::with_capacity(n);
        match &self.engine {
            #[cfg(target_os = "macos")]
            Engine::Mps { .. } => {
                // The graph has a fixed batch: pad the last one.
                let mut buf = vec![0f32; self.batch * vl];
                for chunk in x[..n * vl].chunks(self.batch * vl) {
                    let k = chunk.len() / vl;
                    let l = if k == self.batch {
                        self.logits(chunk)
                    } else {
                        buf[..chunk.len()].copy_from_slice(chunk);
                        self.logits(&buf)
                    };
                    out.extend_from_slice(&l[..k]);
                }
            }
            Engine::Cpu { .. } => {
                for chunk in x[..n * vl].chunks(self.batch * vl) {
                    out.extend(self.logits(chunk));
                }
            }
        }
        out
    }

    /// Early exit for one image whose views are already prepared (all of
    /// them, in the profile's order): the logits of the views looked at.
    pub fn logits_adaptive(&self, all_views: &[f32], exit: &EarlyExit) -> Vec<f32> {
        let (nv, vl, thr) = (self.spec.views.len(), self.view_len(), self.threshold_logit());
        let mut seen: Vec<f32> = Vec::with_capacity(nv);
        let mut buf = Vec::new();
        for upto in exit.checkpoints(nv) {
            buf.clear();
            for k in seen.len()..upto {
                let v = exit.order[k];
                buf.extend_from_slice(&all_views[v * vl..(v + 1) * vl]);
            }
            let n = upto - seen.len();
            seen.extend(self.logits_n(&buf, n));
            if exit.decided(&seen, thr) {
                break;
            }
        }
        seen
    }

    /// Score image files with early exit: per file, the logits of the views
    /// that were looked at, in the order they were looked at. Each image is
    /// decoded and cropped once; its crops wait in memory while it moves from
    /// stage to stage, and every stage runs in full batches drawn from all
    /// the images currently waiting at it.
    pub fn score_files_adaptive(&self, paths: &[PathBuf], threads: usize, exit: &EarlyExit) -> (Vec<Option<Vec<f32>>>, Timing) {
        use std::collections::{HashMap, VecDeque};
        let (nv, vl, thr) = (self.spec.views.len(), self.view_len(), self.threshold_logit());
        let pre = self.preprocessor();
        let checkpoints = exit.checkpoints(nv);
        let in_flight_cap = 4 * self.batch.max(16);
        let start = std::time::Instant::now();
        let mut model_s = 0f64;
        let mut results: Vec<Option<Vec<f32>>> = vec![None; paths.len()];
        let next = AtomicUsize::new(0);
        std::thread::scope(|s| {
            let (tx, rx) = crossbeam_channel::bounded::<(usize, Vec<Rgb>)>(threads.max(1) * 2);
            for _ in 0..threads.max(1) {
                let tx = tx.clone();
                let (next, pre) = (&next, &pre);
                s.spawn(move || loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= paths.len() {
                        break;
                    }
                    let Ok(img) = image_io::load(&paths[i]) else { continue };
                    if tx.send((i, pre.crops(&img))).is_err() {
                        break;
                    }
                });
            }
            drop(tx);
            let mut crops: HashMap<usize, Vec<Rgb>> = HashMap::new();
            let mut seen: HashMap<usize, Vec<f32>> = HashMap::new();
            // Images waiting for stage k (their next views are order[checkpoints[k-1]..checkpoints[k]]).
            let mut waiting: Vec<VecDeque<usize>> = vec![VecDeque::new(); checkpoints.len()];
            let mut buf = vec![0f32; self.batch * vl];
            let mut open = true;
            loop {
                let pending = |k: usize, q: &VecDeque<usize>| q.len() * (checkpoints[k] - if k == 0 { 0 } else { checkpoints[k - 1] });
                // The stage with the most views waiting; later stages win ties, to finish images.
                let best = (0..waiting.len()).rev().max_by_key(|&k| pending(k, &waiting[k])).unwrap();
                let ready = pending(best, &waiting[best]);
                if ready < self.batch && open && crops.len() < in_flight_cap {
                    match rx.recv() {
                        Ok((i, c)) => {
                            crops.insert(i, c);
                            seen.insert(i, Vec::with_capacity(nv));
                            waiting[0].push_back(i);
                        }
                        Err(_) => open = false,
                    }
                    continue;
                }
                if ready == 0 {
                    break;
                }
                let from = if best == 0 { 0 } else { checkpoints[best - 1] };
                let per = checkpoints[best] - from;
                let take = (self.batch / per).max(1).min(waiting[best].len());
                let ids: Vec<usize> = waiting[best].drain(..take).collect();
                {
                    use rayon::prelude::*;
                    let (crops, ids) = (&crops, &ids);
                    buf[..take * per * vl].par_chunks_mut(vl).enumerate().for_each(|(slot, out)| {
                        crops[&ids[slot / per]][exit.order[from + slot % per]].write_chw(out);
                    });
                }
                let t = std::time::Instant::now();
                let logits = self.logits_n(&buf, take * per);
                model_s += t.elapsed().as_secs_f64();
                for (j, i) in ids.into_iter().enumerate() {
                    let l = seen.get_mut(&i).unwrap();
                    l.extend_from_slice(&logits[j * per..(j + 1) * per]);
                    if l.len() == nv || exit.decided(l, thr) {
                        results[i] = seen.remove(&i);
                        crops.remove(&i);
                    } else {
                        waiting[best + 1].push_back(i);
                    }
                }
            }
        });
        (results, Timing { images: paths.len(), wall_s: start.elapsed().as_secs_f64(), model_s })
    }

    /// Switch this detector to its video profile (views and calibration).
    /// Returns the number of frames the profile asks for.
    pub fn use_video_profile(&mut self) -> usize {
        match self.spec.video.clone() {
            Some(v) => {
                self.spec.views = v.views;
                self.spec.logit_scale = v.logit_scale;
                self.spec.logit_bias = v.logit_bias;
                v.frames
            }
            None => 8,
        }
    }

    /// Calibrated P(AI-generated) from per-view logits.
    pub fn probability(&self, view_logits: &[f32]) -> f32 {
        let z = view_logits.iter().sum::<f32>() / view_logits.len() as f32;
        sigmoid(self.spec.logit_scale * z + self.spec.logit_bias)
    }

    /// Score already-decoded images (e.g. video frames). Returns per-image
    /// per-view logits.
    pub fn score_images(&self, imgs: &[Rgb]) -> Vec<Vec<f32>> {
        let (nv, vl) = (self.spec.views.len(), self.view_len());
        let mut flat = vec![0f32; imgs.len() * nv * vl];
        use rayon::prelude::*;
        let pre = self.preprocessor();
        flat.par_chunks_exact_mut(nv * vl).zip(imgs.par_iter()).for_each(|(o, im)| pre.apply(im, o));
        let mut logits = Vec::with_capacity(imgs.len() * nv);
        let mut buf = vec![0f32; self.batch * vl];
        for chunk in flat.chunks(self.batch * vl) {
            buf[..chunk.len()].copy_from_slice(chunk);
            logits.extend_from_slice(&self.logits(&buf)[..chunk.len() / vl]);
        }
        logits.chunks_exact(nv).map(|c| c.to_vec()).collect()
    }

    /// Score image files end to end. Decoding and preprocessing run on
    /// `threads` workers while the GPU consumes batches. Returns per-file
    /// per-view logits (`None` for unreadable files) and timing.
    pub fn score_files(&self, paths: &[PathBuf], threads: usize) -> (Vec<Option<Vec<f32>>>, Timing) {
        let (res, t) = self.score_stream(paths.len(), threads, |i| image_io::load(&paths[i]).ok().map(|im| vec![im]));
        (res.into_iter().map(|r| r.map(|mut v| v.remove(0))).collect(), t)
    }

    /// Score clips: `sampling.frames` frames each, decoded on worker threads.
    /// Returns per-clip, per-frame, per-view logits.
    pub fn score_videos(&self, paths: &[PathBuf], sampling: crate::video::Sampling, threads: usize) -> (Vec<Option<Vec<Vec<f32>>>>, Timing) {
        self.score_stream(paths.len(), threads, |i| crate::video::sample_frames(&paths[i], sampling).ok())
    }

    /// Generic pipeline: `load(i)` yields the images of item `i` (one for a
    /// photo, several frames for a clip) on worker threads; views are batched
    /// across items on the GPU. Returns per-item, per-image, per-view logits.
    pub fn score_stream<F>(&self, n_items: usize, threads: usize, load: F) -> (Vec<Option<Vec<Vec<f32>>>>, Timing)
    where
        F: Fn(usize) -> Option<Vec<Rgb>> + Sync,
    {
        let (nv, vl) = (self.spec.views.len(), self.view_len());
        let next = AtomicUsize::new(0);
        // (item, number of images, all views of all images)
        let (tx, rx) = crossbeam_channel::bounded::<(usize, usize, Vec<f32>)>(threads.max(1) * 2);
        let mut results: Vec<Option<Vec<Vec<f32>>>> = vec![None; n_items];
        let start = std::time::Instant::now();
        let mut model_s = 0f64;
        let pre = self.preprocessor();
        std::thread::scope(|s| {
            for _ in 0..threads.max(1) {
                let tx = tx.clone();
                let (next, pre, load) = (&next, &pre, &load);
                s.spawn(move || loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= n_items {
                        break;
                    }
                    let Some(imgs) = load(i) else { continue };
                    let mut v = vec![0f32; imgs.len() * nv * vl];
                    for (im, o) in imgs.iter().zip(v.chunks_exact_mut(nv * vl)) {
                        pre.apply(im, o);
                    }
                    if tx.send((i, imgs.len(), v)).is_err() {
                        break;
                    }
                });
            }
            drop(tx);
            let mut buf = vec![0f32; self.batch * vl];
            // Slots filled in `buf`: which item each view belongs to.
            let mut owners: Vec<usize> = Vec::with_capacity(self.batch);
            let mut acc: std::collections::HashMap<usize, Vec<f32>> = std::collections::HashMap::new();
            let mut flush = |owners: &mut Vec<usize>, buf: &[f32], acc: &mut std::collections::HashMap<usize, Vec<f32>>| {
                if owners.is_empty() {
                    return;
                }
                let t = std::time::Instant::now();
                let logits = self.logits(buf);
                model_s += t.elapsed().as_secs_f64();
                for (k, &i) in owners.iter().enumerate() {
                    acc.entry(i).or_default().push(logits[k]);
                }
                owners.clear();
            };
            let mut expect: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
            for (i, n_img, v) in rx {
                expect.insert(i, n_img * nv);
                for view in v.chunks_exact(vl) {
                    let k = owners.len();
                    buf[k * vl..(k + 1) * vl].copy_from_slice(view);
                    owners.push(i);
                    if owners.len() == self.batch {
                        flush(&mut owners, &buf, &mut acc);
                    }
                }
                // Hand back any item whose views are all scored.
                let done: Vec<usize> = acc.iter().filter(|(k, l)| expect.get(k) == Some(&l.len())).map(|(k, _)| *k).collect();
                for k in done {
                    let l = acc.remove(&k).unwrap();
                    results[k] = Some(l.chunks_exact(nv).map(|c| c.to_vec()).collect());
                }
            }
            flush(&mut owners, &buf, &mut acc);
            for (k, l) in acc.drain() {
                results[k] = Some(l.chunks_exact(nv).map(|c| c.to_vec()).collect());
            }
        });
        (results, Timing { images: n_items, wall_s: start.elapsed().as_secs_f64(), model_s })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_opinion_band_and_verdict() {
        let so = SecondOpinion { lo: -1.0, hi: 4.0, threshold: 1.0 };
        // Only clips strictly inside the band are sent on.
        assert!(so.wanted(0.0) && so.wanted(3.9));
        assert!(!so.wanted(-1.0) && !so.wanted(4.0) && !so.wanted(7.5));
        // Flagged exactly when the two logits add up to more than the threshold.
        assert!(so.probability(2.0, -0.5) > 0.5);
        assert!(so.probability(2.0, -1.5) < 0.5);
        assert!(so.probability(-0.5, 2.0) > 0.5);
    }

    #[test]
    fn video_spec_without_second_opinion_still_parses() {
        let old = r#"{"frames":16,"views":[{"native":{"crop":224}}],"logit_scale":1.0,"logit_bias":0.0}"#;
        let v: VideoSpec = serde_json::from_str(old).unwrap();
        assert!(v.second_opinion.is_none());
        assert!(!serde_json::to_string(&v).unwrap().contains("second_opinion"));
    }

    #[test]
    fn early_exit_stops_only_when_clear() {
        let e = EarlyExit::standard();
        let thr = 2.0;
        // Order is view 2, 0, 1, 3, 4, 5. First view far below the threshold: one look.
        assert_eq!(e.looked_at(&[0.0, 0.0, -6.0, 0.0, 0.0, 0.0], thr), vec![-6.0]);
        // First view far above: one look.
        assert_eq!(e.looked_at(&[0.0, 0.0, 11.5, 0.0, 0.0, 0.0], thr).len(), 1);
        // Undecided after one, clear after two (mean 8 is 6 over).
        assert_eq!(e.looked_at(&[9.0, 0.0, 7.0, 0.0, 0.0, 0.0], thr), vec![7.0, 9.0]);
        // Clear after four (mean -1 is 3 under).
        assert_eq!(e.looked_at(&[0.0, -2.0, 0.0, -2.0, 5.0, 5.0], thr).len(), 4);
        // Near the threshold throughout: all six, in looking order.
        assert_eq!(e.looked_at(&[2.0, 3.0, 1.0, 2.5, 4.0, 5.0], thr), vec![1.0, 2.0, 3.0, 2.5, 4.0, 5.0]);
    }

    #[test]
    fn specs_without_the_new_fields_still_parse() {
        let old = r#"{"img":224,"views":[{"native":{"crop":224}}]}"#;
        let m: ModelSpec = serde_json::from_str(old).unwrap();
        assert!(m.early_exit.is_none() && m.strict.is_none());
    }

    #[test]
    fn video_weights_default_to_the_photo_checkpoint() {
        let p = std::path::Path::new("/nonexistent/dir/veracity.safetensors");
        assert_eq!(video_weights_for(p), p);
    }
}
