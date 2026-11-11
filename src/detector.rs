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
            arch: "vit_base_patch14_reg4_dinov2".into(),
        }
    }

    /// The Community Forensics baseline's own preprocessing.
    pub fn commfor(img: usize) -> Self {
        let resize = if img == 384 { 440 } else { 256 };
        Self { img, views: vec![ViewSpec::FitPillow { resize, crop: img as u32 }], logit_scale: 1.0, logit_bias: 0.0, threshold: 0.5, prefix: "vit.".into(), video: None, fast: None, sensitive: None, arch: String::new() }
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
        // Texture ranking is computed once and shared by all "rich" views.
        let need = self.views.iter().filter_map(|v| if let View::NativeRich { crop, rank } = v { Some((*crop, *rank as usize + 1)) } else { None }).max_by_key(|x| x.1);
        let rich = need.filter(|(crop, _)| img.w.min(img.h) >= *crop).map(|(crop, k)| image_io::rich_windows(img, crop, k));
        for (v, chunk) in self.views.iter().zip(out.chunks_exact_mut(self.view_len)) {
            v.apply_with(img, rich.as_deref()).write_chw(chunk);
        }
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
    /// Accelerate-backed engine (macOS).
    #[cfg(target_os = "macos")]
    Cpu { vit: crate::cpu_vit::CpuVit },
    /// Portable engine on candle.
    #[cfg(not(target_os = "macos"))]
    Cpu { vit: crate::vit::Vit },
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
            #[cfg(target_os = "macos")]
            Device::Cpu => Engine::Cpu { vit: crate::cpu_vit::CpuVit::new(w, cfg)? },
            #[cfg(not(target_os = "macos"))]
            Device::Cpu => {
                let dev = candle_core::Device::Cpu;
                let tensors = w
                    .0
                    .iter()
                    .map(|(k, (d, sh))| Ok((k.clone(), candle_core::Tensor::from_slice(d, sh.as_slice(), &dev)?)))
                    .collect::<Result<std::collections::HashMap<_, _>>>()?;
                let vb = candle_nn::VarBuilder::from_tensors(tensors, candle_core::DType::F32, &dev);
                Engine::Cpu { vit: crate::vit::Vit::new(vb, cfg, Some(1))? }
            }
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
        let s = self.spec.img;
        match &self.engine {
            #[cfg(target_os = "macos")]
            Engine::Mps { g, inp, out } => g.run(&[(inp, x, &[self.batch as isize, 3, s as isize, s as isize])], &[out.clone()], &[]).remove(0),
            #[cfg(target_os = "macos")]
            Engine::Cpu { vit } => vit.forward(x),
            #[cfg(not(target_os = "macos"))]
            Engine::Cpu { vit } => {
                use rayon::prelude::*;
                let vl = self.view_len();
                x.par_chunks(vl * 2)
                    .flat_map_iter(|c| {
                        let t = candle_core::Tensor::from_slice(c, (c.len() / vl, 3, s, s), &candle_core::Device::Cpu).expect("tensor");
                        vit.forward(&t).and_then(|o| o.flatten_all()?.to_vec1::<f32>()).expect("cpu forward")
                    })
                    .collect()
            }
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

    /// Score clips: `sampling.frames` frames each, decoded by ffmpeg workers.
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
    fn video_weights_default_to_the_photo_checkpoint() {
        let p = std::path::Path::new("/nonexistent/dir/veracity.safetensors");
        assert_eq!(video_weights_for(p), p);
    }
}
