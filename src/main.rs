use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use tintype::detector::Detector;
use tintype::detector::{Device, ModelSpec};
use tintype::ingest;

mod scan;
use scan::ScanOptions;

#[derive(Parser)]
#[command(name = "tintype", version, about = "AI-generated image and video detection")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan image and video files.
    Detect {
        /// Image or video files.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        options: ScanOptions,
        /// WaveRep weights for an optional video second opinion (macOS only).
        #[arg(long)]
        second_opinion: Option<PathBuf>,
        /// Community Forensics 384 px weights for an optional photo second opinion.
        #[arg(long, conflicts_with_all = ["fast", "sensitive", "strict"])]
        photo_second_opinion: Option<PathBuf>,
    },
    /// Recursively scan directories with bounded memory and stream results.
    Batch {
        /// Files or directory roots to scan, in the supplied order.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        options: ScanOptions,
    },
    /// Score the included labeled sample offline.
    Demo(scan::DemoOptions),
    /// Start the versioned HTTP API for photos and video.
    Serve(tintype::server::ServeArgs),
    /// Score every image in a manifest: `path label generator dataset prob logits...`.
    Score {
        #[arg(long)]
        weights: PathBuf,
        /// Run a reference detector instead: `commfor`, `swinv2`.
        #[arg(long)]
        reference: Option<String>,
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 64)]
        batch: usize,
        #[arg(long, default_value = "auto")]
        device: String,
        /// Override the model's views, e.g. `fit,native,rich4`.
        #[arg(long)]
        views: Option<String>,
        /// Use the two-view fast profile.
        #[arg(long)]
        fast: bool,
        /// Use the sensitive profile.
        #[arg(long, conflicts_with = "fast")]
        sensitive: bool,
        /// Use the strict profile.
        #[arg(long, conflicts_with_all = ["fast", "sensitive"])]
        strict: bool,
        /// Look at every view of every image (no early exit).
        #[arg(long)]
        all_views: bool,
        /// Community Forensics weights: give the photos inside the model's band a second opinion (see `detect --photo-second-opinion`).
        #[arg(long, conflicts_with_all = ["reference", "views", "fast", "sensitive", "strict"])]
        photo_second_opinion: Option<PathBuf>,
    },
    /// Score every clip in a manifest (frame sampling + aggregation).
    ScoreVideo {
        #[arg(long)]
        weights: PathBuf,
        /// Run a reference detector instead: `waverep`.
        #[arg(long)]
        reference: Option<String>,
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        /// Frames analysed per clip (default: the model's video profile).
        #[arg(long)]
        frames: Option<usize>,
        /// Override the views, e.g. `fit,rich5`.
        #[arg(long)]
        views: Option<String>,
        /// WaveRep weights: give uncertain clips a second opinion (see `detect --second-opinion`).
        #[arg(long)]
        second_opinion: Option<PathBuf>,
        /// Look at every frame of every clip (no early exit).
        #[arg(long)]
        all_frames: bool,
        #[arg(long, default_value_t = 64)]
        batch: usize,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Compare score files: `--system name=scores.tsv[@threshold]` (repeatable).
    Report {
        #[arg(long = "system", required = true)]
        systems: Vec<String>,
        #[arg(long)]
        per_generator: bool,
        /// Print markdown tables with bootstrap confidence intervals.
        #[arg(long)]
        markdown: bool,
        /// With --markdown: also print a pooled table under this name.
        #[arg(long)]
        pooled: Option<String>,
        /// Name of one of the systems: print the others operated at its false-positive count on each set.
        #[arg(long)]
        at_rate_of: Option<String>,
    },
    /// Score a whole evaluation suite and print the comparison tables.
    Evaluate {
        /// benchmarks, or confirm1 to confirm7.
        suite: String,
        #[arg(long, default_value = "models/tintype.safetensors")]
        weights: PathBuf,
        /// Video model weights (default: `<weights>-video.safetensors` beside the photo weights, if present).
        #[arg(long)]
        video_weights: Option<PathBuf>,
        /// Suffix of the score files written for Tintype.
        #[arg(long, default_value = "tintype")]
        tag: String,
        #[arg(long, default_value = "runs/scores")]
        scores: PathBuf,
        /// Also run the three reference detectors (weights from `tintype data baselines`).
        #[arg(long)]
        baselines: bool,
        /// Keep reference score files that already exist.
        #[arg(long)]
        reuse: bool,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Freeze a checkpoint: write its spec and calibrate photo, fast and video profiles.
    Finalize {
        checkpoint: PathBuf,
        #[arg(long, default_value = "models/tintype.safetensors")]
        weights: PathBuf,
        #[arg(long, default_value = "runs/scores")]
        scores: PathBuf,
        /// False-positive target for photos, averaged over real sources.
        #[arg(long, default_value_t = 0.015)]
        photo_fpr: f64,
        /// False-positive target for the strict and fast photo profiles.
        #[arg(long, default_value_t = 0.005)]
        strict_fpr: f64,
        /// False-positive target for the sensitive photo profile.
        #[arg(long, default_value_t = 0.05)]
        sensitive_fpr: f64,
        /// False-positive target for video, averaged over real sources.
        #[arg(long, default_value_t = 0.01)]
        video_fpr: f64,
        /// Training manifests added after phase 3: their validation split joins the calibration data.
        #[arg(long)]
        also_trained_on: Vec<PathBuf>,
        /// Evaluation manifests scored since phase 3: their items join the calibration data.
        #[arg(long)]
        also_scored_on: Vec<PathBuf>,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Freeze a checkpoint as the video model: write its spec and calibrate the video threshold.
    FinalizeVideo {
        checkpoint: PathBuf,
        #[arg(long, default_value = "models/tintype-video.safetensors")]
        weights: PathBuf,
        #[arg(long, default_value = "runs/scores")]
        scores: PathBuf,
        /// False-positive target, averaged over real sources.
        #[arg(long, default_value_t = 0.01)]
        video_fpr: f64,
        /// Second-opinion stage as `lo,hi,threshold` in first-stage logits (see `detect --second-opinion`).
        #[arg(long, allow_hyphen_values = true)]
        second_opinion: Option<String>,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Store the band of the photo second opinion in a checkpoint's spec
    /// (see `detect --photo-second-opinion`), or remove it with `--band none`.
    SetPhotoSecondOpinion {
        #[arg(long, default_value = "models/tintype.safetensors")]
        weights: PathBuf,
        /// `lo,hi,weight,threshold` in calibrated first-stage logits, or `none`.
        #[arg(long, allow_hyphen_values = true)]
        band: String,
    },
    /// Fit the decision calibration of one profile from scored held-out data.
    Calibrate {
        #[arg(long)]
        weights: PathBuf,
        /// Score files produced by `score` / `score-video` with these weights.
        #[arg(long, required = true)]
        scores: Vec<PathBuf>,
        /// False-positive rate on real items that probability 0.5 should correspond to.
        #[arg(long, default_value_t = 0.005)]
        target_fpr: f64,
        /// photo (default), strict, fast, sensitive or video.
        #[arg(long, default_value = "photo")]
        profile: String,
        /// Also store the standard early-exit schedule in the checkpoint's spec.
        #[arg(long)]
        early_exit: bool,
        /// Threshold rule: pooled, strictest or balanced (over real sources).
        #[arg(long, default_value = "balanced")]
        rule: String,
        /// Smallest real source that counts for the strictest / balanced rules.
        #[arg(long, default_value_t = 100)]
        min_source: usize,
    },
    /// Time the parts of the pipeline separately: decoding, cropping, and the
    /// network alone at several batch sizes.
    Profile {
        #[arg(long)]
        weights: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
        /// Number of images to decode.
        #[arg(long, default_value_t = 1000)]
        n: usize,
        #[arg(long, default_value = "auto")]
        device: String,
        /// Batch sizes to time the network at.
        #[arg(long, value_delimiter = ',', default_value = "16,32,64,128")]
        batches: Vec<usize>,
        /// Skip the decoding and cropping measurements.
        #[arg(long)]
        model_only: bool,
    },
    /// Single-item latency: one image at a time, decode to verdict.
    Latency {
        #[arg(long)]
        weights: PathBuf,
        /// Time a reference detector instead: `commfor`, `swinv2`.
        #[arg(long)]
        reference: Option<String>,
        #[arg(long)]
        manifest: PathBuf,
        /// Number of images to time (after 5 warm-up images).
        #[arg(long, default_value_t = 200)]
        n: usize,
        #[arg(long, default_value = "auto")]
        device: String,
        #[arg(long)]
        fast: bool,
        /// Look at every view of every image (no early exit).
        #[arg(long)]
        all_views: bool,
    },
    /// Prepare an untrained ViT-S/16 checkpoint for detector training.
    InitTraining {
        /// Local timm-layout ViT-S/16 safetensors backbone.
        #[arg(long)]
        backbone: PathBuf,
        /// Expected SHA-256. Defaults to the pinned timm DINO checkpoint.
        #[arg(long, default_value = tintype::training_init::DINO_SHA256)]
        sha256: String,
        /// New safetensors file. A matching JSON sidecar is written beside it.
        #[arg(long)]
        out: PathBuf,
    },
    /// Fine-tune the detector (Apple GPU).
    Train {
        /// Starting weights (safetensors).
        #[arg(long)]
        init: PathBuf,
        /// Prefix stripped from tensor names in `--init`.
        #[arg(long, default_value = "")]
        init_prefix: String,
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 20000)]
        steps: u64,
        #[arg(long, default_value_t = 32)]
        batch: usize,
        #[arg(long, default_value_t = 5e-5)]
        lr: f32,
        #[arg(long, default_value_t = 500)]
        warmup: u64,
        #[arg(long, default_value_t = 3)]
        val_pct: u64,
        #[arg(long, default_value_t = 2000)]
        val_max: usize,
        #[arg(long, default_value_t = 1000)]
        eval_every: u64,
        #[arg(long, default_value_t = 11)]
        threads: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 0.5)]
        p_native: f32,
        #[arg(long, default_value_t = 0.8)]
        layer_decay: f64,
        #[arg(long, default_value_t = 0.05)]
        weight_decay: f64,
        #[arg(long, default_value_t = 0.0)]
        smoothing: f64,
        /// Generator sampling weight exponent (count^alpha).
        #[arg(long, default_value_t = 0.5)]
        sampler_alpha: f64,
        /// Probability of the messenger-style downscale + JPEG augmentation.
        #[arg(long, default_value_t = 0.25)]
        p_messenger: f32,
    },
    /// Download and prepare a dataset plan (`tintype data list` shows them).
    Data {
        plan: String,
        /// Resolve and print the downloads without fetching anything.
        #[arg(long)]
        dry_run: bool,
        /// Resolve the downloads without fetching anything, and compare the
        /// files already on disk with their pinned SHA-256; fails if any differs.
        #[arg(long, conflicts_with = "dry_run")]
        check: bool,
        /// Parallel connections per large file.
        #[arg(long, default_value_t = 12)]
        connections: usize,
    },
    /// Extract images from parquet shards into an image store with a manifest.
    Ingest {
        /// Parquet shards.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        /// Dataset name recorded in the manifest.
        #[arg(long)]
        dataset: String,
        #[arg(long, default_value = "image")]
        image_col: String,
        #[arg(long)]
        label_col: Option<String>,
        #[arg(long, value_delimiter = ',', default_value = "fake,1")]
        fake_values: Vec<String>,
        #[arg(long, value_delimiter = ',', default_value = "real,0")]
        real_values: Vec<String>,
        /// Label for every row when there is no label column (0 or 1).
        #[arg(long)]
        const_label: Option<u8>,
        #[arg(long)]
        gen_col: Option<String>,
        #[arg(long)]
        real_src_col: Option<String>,
        #[arg(long)]
        const_gen: Option<String>,
        #[arg(long)]
        max_per_file: Option<usize>,
    },
    /// Index an extracted folder tree. Rules are `substring=label:generator`.
    IndexFolder {
        root: PathBuf,
        #[arg(long)]
        dataset: String,
        #[arg(long = "rule", required = true)]
        rules: Vec<String>,
        /// Index video files instead of images.
        #[arg(long)]
        video: bool,
    },
    /// Decode a frame from the middle of every clip in the manifests; list the
    /// clips that cannot be read or take longer than five seconds.
    CheckVideos {
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        /// Rewrite each manifest without the listed clips.
        #[arg(long)]
        prune: bool,
        /// Also draw a re-encoded training frame from every clip (the H.264 augmentation).
        #[arg(long)]
        recompress: bool,
        /// Instead: decode this many frames of every clip with both the system decoder and the pure-Rust reader, and report how they differ (macOS)
        #[arg(long)]
        compare_decoders: Option<usize>,
    },
    /// List test files that are byte-identical to a training file.
    Overlap {
        #[arg(long, required = true)]
        train: Vec<PathBuf>,
        #[arg(long, required = true)]
        test: Vec<PathBuf>,
    },
    /// Print the schema and first rows of a parquet shard.
    ParquetInfo {
        path: PathBuf,
        #[arg(long, default_value_t = 3)]
        rows: usize,
    },
    /// Record a prepared evaluation set: the SHA-256, size and label of every
    /// file its manifests list, and the SHA-256 of each manifest. Run from the
    /// directory that holds `data/`.
    FreezeSet {
        /// A whole evaluation suite: benchmarks, or confirm1 to confirm7.
        #[arg(long, required_unless_present = "manifest")]
        suite: Option<String>,
        /// Manifests to record (instead of, or as well as, a suite's).
        #[arg(long)]
        manifest: Vec<PathBuf>,
        /// Where to write it; a name ending in `.gz` is compressed.
        #[arg(long)]
        out: PathBuf,
        /// Paths are stored relative to this directory.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Check a local evaluation set against a record written by `freeze-set`:
    /// every file present with the same bytes, every manifest unchanged.
    /// Exits with an error listing every difference.
    VerifySet {
        /// Records written by `freeze-set`.
        #[arg(required = true)]
        frozen: Vec<PathBuf>,
        /// The directory the paths in the records are relative to.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Check the released model files (weights and specs) against the
    /// SHA-256 values recorded in the source and in docs/DATA.md.
    VerifyModels {
        /// The repository checkout that holds `models/`.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
}

fn parse_device(s: &str) -> Result<Device> {
    Ok(match s {
        "auto" => Device::best(),
        "gpu" => Device::Gpu,
        "gpu-f32" => Device::GpuF32,
        "cpu" => Device::Cpu,
        other => anyhow::bail!("unknown device `{other}` (use auto, gpu, gpu-f32 or cpu)"),
    })
}

#[cfg(target_os = "macos")]
fn system(reference: &Option<String>, fast: bool, sensitive: bool, strict: bool) -> Result<tintype::eval::System> {
    use tintype::eval::{Photo, System};
    Ok(match reference.as_deref() {
        None => System::Tintype { profile: if fast { Photo::Fast } else if sensitive { Photo::Sensitive } else if strict { Photo::Strict } else { Photo::Default } },
        Some("commfor") => System::CommunityForensics,
        Some("swinv2") => System::SwinV2,
        Some("waverep") => System::WaveRep,
        Some(other) => anyhow::bail!("unknown reference detector `{other}` (commfor, swinv2, waverep)"),
    })
}

fn load_manifests(ms: &[PathBuf]) -> Result<Vec<ingest::Item>> {
    let mut v = Vec::new();
    for m in ms {
        v.extend(ingest::read_manifest(m)?);
    }
    Ok(v)
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Detect { paths, options, second_opinion, photo_second_opinion } => {
            scan::detect(paths, options, second_opinion, photo_second_opinion)
        }
        Cmd::Batch { paths, options } => scan::batch(paths, options),
        Cmd::Demo(options) => scan::demo(options),
        Cmd::Serve(args) => tintype::server::run(args, scan::resolve_weights),
        Cmd::InitTraining { backbone, sha256, out } => {
            tintype::training_init::initialize(&backbone, &sha256, &out)
        }
        #[cfg(target_os = "macos")]
        Cmd::Score { weights, reference, manifest, out, batch, device, views, fast, sensitive, strict, all_views, photo_second_opinion } => {
            tintype::eval::score_images(&weights, &system(&reference, fast, sensitive, strict)?, &manifest, &out, parse_device(&device)?, batch, views.as_deref(), all_views, photo_second_opinion.as_deref()).map(|_| ())
        }
        #[cfg(target_os = "macos")]
        Cmd::ScoreVideo { weights, reference, manifest, out, frames, views, second_opinion, all_frames, batch, device } => {
            tintype::eval::score_videos(&weights, &system(&reference, false, false, false)?, &manifest, &out, parse_device(&device)?, batch, frames, views.as_deref(), second_opinion.as_deref(), all_frames).map(|_| ())
        }
        Cmd::Report { systems, per_generator, markdown, pooled, at_rate_of } => {
            let mut loaded = Vec::new();
            for s in &systems {
                let (name, rest) = s.split_once('=').ok_or_else(|| anyhow::anyhow!("use name=scores.tsv[@threshold]"))?;
                let (path, thr) = match rest.rsplit_once('@') {
                    Some((p, t)) => (p, t.parse::<f32>()?),
                    None => (rest, 0.5),
                };
                loaded.push((name.to_string(), tintype::report::read_scores(std::path::Path::new(path))?, thr));
            }
            if let Some(baseline) = at_rate_of {
                tintype::report::matched_markdown(&loaded, &baseline, pooled.as_deref())?;
            } else if markdown {
                tintype::report::compare_markdown(&loaded, pooled.as_deref());
            } else {
                tintype::report::compare(&loaded, per_generator);
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Evaluate { suite, weights, video_weights, tag, scores, baselines, reuse, device } => {
            let b = tintype::eval::Baselines::default();
            let video_weights = video_weights.unwrap_or_else(|| tintype::detector::video_weights_for(&weights));
            tintype::eval::evaluate(&suite, &weights, &video_weights, &tag, &scores, baselines.then_some(&b), reuse, parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::Finalize { checkpoint, weights, scores, photo_fpr, strict_fpr, sensitive_fpr, video_fpr, also_trained_on, also_scored_on, device } => {
            tintype::eval::finalize(&checkpoint, &weights, &scores, photo_fpr, strict_fpr, sensitive_fpr, video_fpr, (&also_trained_on, &also_scored_on), parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::FinalizeVideo { checkpoint, weights, scores, video_fpr, second_opinion, device } => {
            let band = match second_opinion {
                Some(s) => {
                    let v: Vec<f32> = s.split(',').map(|x| x.trim().parse::<f32>()).collect::<Result<_, _>>()?;
                    anyhow::ensure!(v.len() == 3 && v[0] < v[1], "--second-opinion takes lo,hi,threshold with lo < hi");
                    Some(tintype::detector::SecondOpinion { lo: v[0], hi: v[1], threshold: v[2] })
                }
                None => None,
            };
            tintype::eval::finalize_video(&checkpoint, &weights, &scores, video_fpr, band, parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::Calibrate { weights, scores, target_fpr, profile, early_exit, rule, min_source } => {
            if early_exit {
                let mut spec = ModelSpec::load_for(&weights)?;
                spec.early_exit = Some(tintype::detector::EarlyExit::standard());
                spec.save_for(&weights)?;
            }
            use tintype::eval::Rule;
            let rule = match rule.as_str() {
                "pooled" => Rule::Pooled,
                "strictest" => Rule::Strictest(min_source),
                "balanced" => Rule::Balanced(min_source),
                other => anyhow::bail!("unknown rule `{other}` (pooled, strictest, balanced)"),
            };
            tintype::eval::calibrate(&weights, &scores, target_fpr, &profile, rule)
        }
        #[cfg(target_os = "macos")]
        Cmd::Latency { weights, reference, manifest, n, device, fast, all_views } => {
            let items = ingest::read_manifest(&manifest)?;
            let mut spec = match reference.as_deref() {
                Some("commfor") => ModelSpec::commfor(384),
                Some("swinv2") => ModelSpec::swinv2(),
                Some(other) => anyhow::bail!("unknown reference detector `{other}`"),
                None => ModelSpec::load_for(&weights)?,
            };
            if fast {
                Detector::use_fast_profile(&mut spec);
            }
            let nv = spec.views.len();
            let exit = spec.early_exit.clone().filter(|_| reference.is_none() && !fast && !all_views);
            // On the CPU early exit runs the views one or two at a time; on the GPU all
            // six go in one batch and the stopping rule is applied afterwards.
            let device = parse_device(&device)?;
            let stepwise = exit.is_some() && matches!(device, Device::Cpu);
            let det = Detector::load(&weights, Some(spec), if stepwise { 2 } else { nv }, device)?;
            let pre = det.preprocessor();
            let mut buf = vec![0u8; nv * 3 * det.spec.img * det.spec.img];
            let mut ms = Vec::new();
            for (k, it) in items.iter().take(n + 5).enumerate() {
                let t = std::time::Instant::now();
                let Ok(img) = tintype::image_io::load(&it.path) else { continue };
                pre.apply(&img, &mut buf);
                match &exit {
                    Some(e) if stepwise => std::hint::black_box(det.probability(&det.logits_adaptive(&buf, e))),
                    Some(e) => std::hint::black_box(det.probability(&e.looked_at(&det.logits(&buf)[..nv], det.threshold_logit()))),
                    None => std::hint::black_box(det.probability(&det.logits(&buf)[..nv])),
                };
                if k >= 5 {
                    ms.push(t.elapsed().as_secs_f64() * 1e3);
                }
            }
            ms.sort_by(|a, b| a.total_cmp(b));
            let q = |f: f64| ms[((ms.len() - 1) as f64 * f) as usize];
            println!("single-image latency over {} images: median {:.1} ms, p90 {:.1} ms, mean {:.1} ms", ms.len(), q(0.5), q(0.9), ms.iter().sum::<f64>() / ms.len() as f64);
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Profile { weights, manifest, n, device, batches, model_only } => {
            use rayon::prelude::*;
            use std::time::Instant;
            let spec = ModelSpec::load_for(&weights)?;
            let device = parse_device(&device)?;
            if !model_only {
                let items = ingest::read_manifest(&manifest)?;
                let paths: Vec<PathBuf> = items.iter().take(n).map(|i| i.path.clone()).collect();
                let t = Instant::now();
                let imgs: Vec<tintype::image_io::Rgb> = paths.par_iter().filter_map(|p| tintype::image_io::load(p).ok()).collect();
                let s = t.elapsed().as_secs_f64();
                let mp: f64 = imgs.iter().map(|i| i.w as f64 * i.h as f64 / 1e6).sum();
                println!("decode: {} images, {:.0} megapixels in {:.2}s: {:.0} img/s, {:.0} MP/s", imgs.len(), mp, s, imgs.len() as f64 / s, mp / s);
                let pre = Detector::load(&weights, Some(spec.clone()), 1, Device::Cpu)?.preprocessor();
                let t = Instant::now();
                let k: usize = imgs.par_iter().map(|im| pre.crops(im).len()).sum();
                let s = t.elapsed().as_secs_f64();
                println!("crops: {} views of {} images in {:.2}s: {:.0} img/s", k, imgs.len(), s, imgs.len() as f64 / s);
            }
            let vl = 3 * spec.img * spec.img;
            for &b in &batches {
                let det = Detector::load(&weights, Some(spec.clone()), b, device)?;
                let mut rng = tintype::rng::Rng::new(7);
                let x: Vec<u8> = (0..b * vl).map(|_| rng.below(256) as u8).collect();
                for _ in 0..3 {
                    std::hint::black_box(det.logits(&x));
                }
                let (t, mut runs) = (Instant::now(), 0usize);
                while t.elapsed().as_secs_f64() < 3.0 {
                    std::hint::black_box(det.logits(&x));
                    runs += 1;
                }
                let s = t.elapsed().as_secs_f64();
                println!("network batch {b}: {:.1} ms per batch, {:.0} views/s", s * 1e3 / runs as f64, (runs * b) as f64 / s);
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Train { init, init_prefix, manifest, out, steps, batch, lr, warmup, val_pct, val_max, eval_every, threads, seed, p_native, layer_decay, weight_decay, smoothing, sampler_alpha, p_messenger } => {
            tintype::train_cmd::run(tintype::train_cmd::TrainArgs {
                init,
                init_prefix,
                manifests: manifest,
                out,
                steps,
                batch,
                lr,
                warmup,
                val_pct,
                val_max,
                eval_every,
                threads,
                seed,
                aug: tintype::data::AugConfig { p_native, p_messenger, ..Default::default() },
                opt: tintype::trainer::OptConfig { layer_decay, weight_decay, smoothing, ..Default::default() },
                sampler_alpha,
            })
        }
        #[cfg(not(target_os = "macos"))]
        Cmd::Score { .. } | Cmd::ScoreVideo { .. } | Cmd::Evaluate { .. } | Cmd::Finalize { .. } | Cmd::FinalizeVideo { .. } | Cmd::Calibrate { .. } | Cmd::Latency { .. } | Cmd::Profile { .. } | Cmd::Train { .. } => {
            anyhow::bail!("this command uses the Apple GPU engine and needs macOS on Apple Silicon; `detect --device cpu` works everywhere")
        }
        Cmd::SetPhotoSecondOpinion { weights, band } => {
            let mut spec = ModelSpec::load_for(&weights)?;
            spec.second_opinion = if band == "none" {
                None
            } else {
                let v: Vec<f32> = band.split(',').map(|x| x.trim().parse::<f32>()).collect::<Result<_, _>>()?;
                anyhow::ensure!(v.len() == 4 && v[0] < v[1], "--band takes lo,hi,weight,threshold with lo < hi");
                Some(tintype::detector::PhotoSecondOpinion { lo: v[0], hi: v[1], weight: v[2], threshold: v[3] })
            };
            spec.save_for(&weights)
        }
        Cmd::Data { plan, dry_run, check, connections } => {
            if plan == "list" {
                for (name, what) in tintype::plans::PLANS {
                    println!("{name:<14} {what}");
                }
                return Ok(());
            }
            tintype::plans::run(&plan, dry_run, check, connections)
        }
        Cmd::Ingest { inputs, out, dataset, image_col, label_col, fake_values, real_values, const_label, gen_col, real_src_col, const_gen, max_per_file } => {
            let spec = ingest::ParquetSpec { image_col, label_col, fake_values, real_values, const_label, gen_col, real_src_col, const_gen, max_per_file };
            let (n, skipped) = ingest::ingest_parquet(&inputs, &out, &dataset, &spec)?;
            println!("{dataset}: wrote {n} images, skipped {skipped} rows -> {}", out.display());
            Ok(())
        }
        Cmd::IndexFolder { root, dataset, rules, video } => {
            let rules = rules
                .iter()
                .map(|r| {
                    let (pat, rest) = r.split_once('=').ok_or_else(|| anyhow::anyhow!("rule must be substring=label:generator"))?;
                    let (label, gen) = rest.split_once(':').ok_or_else(|| anyhow::anyhow!("rule must be substring=label:generator"))?;
                    Ok((pat.to_string(), label.parse()?, gen.to_string()))
                })
                .collect::<Result<Vec<_>>>()?;
            let n = ingest::index_folder(&root, &dataset, &rules, video)?;
            println!("{dataset}: indexed {n} files under {}", root.display());
            Ok(())
        }
        Cmd::CheckVideos { manifest, prune, recompress, compare_decoders } => {
            use rayon::prelude::*;
            #[cfg(target_os = "macos")]
            if let Some(limit) = compare_decoders {
                for m in &manifest {
                    let items = ingest::read_manifest(m)?;
                    let rows: Vec<(PathBuf, Result<tintype::avf::Comparison>, Option<String>)> = items
                        .par_iter()
                        .map(|it| {
                            let c = tintype::avf::compare_with_pure_rust(&it.path, limit);
                            // When the comparison fails, say which reader could not open the clip.
                            let rust_only = c.is_err().then(|| tintype::swdec::open(&it.path, None).err().map(|e| e.to_string())).flatten();
                            (it.path.clone(), c, rust_only)
                        })
                        .collect();
                    let (mut clips, mut exact, mut frames, mut same, mut failed) = (0usize, 0usize, 0usize, 0usize, 0usize);
                    for (p, c, rust_error) in &rows {
                        match c {
                            Ok(c) => {
                                clips += 1;
                                frames += c.frames;
                                same += c.identical;
                                let ok = c.identical == c.frames && c.produced.0 == c.produced.1 && c.max_time_difference < 1e-3 && (c.durations.0 - c.durations.1).abs() < 1e-3;
                                exact += ok as usize;
                                if !ok {
                                    println!(
                                        "{}\tframes {} (system {}, rust {})\tidentical {}\ttag mismatches {}\tmax sample difference {}\tmax time difference {:.4}s\tdurations {:.3} / {:.3}\ttags (bt709, full range) {:?}\tdeclared {:?}\tsize {:?}",
                                        p.display(), c.frames, c.produced.0, c.produced.1, c.identical, c.tag_mismatches, c.max_sample_difference, c.max_time_difference, c.durations.0, c.durations.1, c.tags, c.declared, c.size
                                    );
                                }
                            }
                            Err(e) => {
                                failed += 1;
                                println!("{}\tnot compared: {}", p.display(), rust_error.as_ref().map_or_else(|| format!("system decoder: {e}"), |r| format!("pure-Rust reader: {r}")));
                            }
                        }
                    }
                    println!("{}: {clips} clips compared, {exact} identical in every frame, time and duration; {same} of {frames} frames identical; {failed} not compared", m.display());
                }
                return Ok(());
            }
            #[cfg(not(target_os = "macos"))]
            anyhow::ensure!(compare_decoders.is_none(), "--compare-decoders needs the macOS system decoder to compare against");
            for m in &manifest {
                let items = ingest::read_manifest(m)?;
                let bad: Vec<(PathBuf, String)> = items
                    .par_iter()
                    .filter_map(|it| {
                        let t = std::time::Instant::now();
                        let mut r = tintype::video::middle_frame(&it.path);
                        if recompress && r.is_ok() {
                            let seed = it.path.to_string_lossy().bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3));
                            let mut rng = tintype::rng::Rng::new(seed);
                            r = tintype::video::recompressed_only(&it.path, &mut rng, &std::env::temp_dir());
                        }
                        let secs = t.elapsed().as_secs_f64();
                        match r {
                            Err(e) => Some((it.path.clone(), format!("unreadable: {e}"))),
                            Ok(_) if secs > 5.0 => Some((it.path.clone(), format!("slow: {secs:.0}s"))),
                            Ok(_) => None,
                        }
                    })
                    .collect();
                for (p, why) in &bad {
                    println!("{}\t{why}", p.display());
                }
                println!("{}: {} of {} clips listed", m.display(), bad.len(), items.len());
                if prune && !bad.is_empty() {
                    let root = m.parent().unwrap_or(std::path::Path::new(""));
                    let drop: std::collections::HashSet<PathBuf> = bad.into_iter().map(|(p, _)| p).collect();
                    let text = std::fs::read_to_string(m)?;
                    let kept: Vec<&str> = text.lines().filter(|l| l.split('\t').next().is_some_and(|rel| !drop.contains(&root.join(rel)))).collect();
                    std::fs::write(m, kept.join("\n") + "\n")?;
                }
            }
            Ok(())
        }
        Cmd::Overlap { train, test } => {
            let (tr, te) = (load_manifests(&train)?, load_manifests(&test)?);
            let dups = ingest::exact_overlap(&tr, &te);
            for (a, b) in dups.iter().take(20) {
                println!("{} == {}", a.display(), b.display());
            }
            println!("{} of {} test files are byte-identical to one of {} training files", dups.len(), te.len(), tr.len());
            Ok(())
        }
        Cmd::ParquetInfo { path, rows } => ingest::parquet_info(&path, rows),
        Cmd::FreezeSet { suite, manifest, out, root } => {
            use tintype::integrity::Frozen;
            let mut manifests: Vec<PathBuf> = match &suite {
                Some(s) => tintype::plans::suite(s)?.iter().map(|set| root.join(set.manifest)).collect(),
                None => Vec::new(),
            };
            manifests.extend(manifest);
            let frozen = Frozen::build(&manifests, &root)?;
            frozen.write(&out)?;
            let bytes: u64 = frozen.rows.iter().map(|r| r.size).sum();
            println!("{}: {} files ({:.2} GB) from {} manifests", out.display(), frozen.rows.len(), bytes as f64 / 1e9, frozen.manifests.len());
            Ok(())
        }
        Cmd::VerifySet { frozen, root } => {
            let mut failed = 0;
            for f in &frozen {
                let record = tintype::integrity::Frozen::read(f)?;
                let problems = record.verify(&root);
                for p in &problems {
                    println!("{p}");
                }
                println!("{}: {} files, {} manifests: {}", f.display(), record.rows.len(), record.manifests.len(), if problems.is_empty() { "identical".to_string() } else { format!("{} differences", problems.len()) });
                failed += problems.len();
            }
            anyhow::ensure!(failed == 0, "{failed} differences from the frozen evaluation sets");
            Ok(())
        }
        Cmd::VerifyModels { root } => {
            let (lines, ok) = tintype::integrity::check_models(&root);
            for l in lines {
                println!("{l}");
            }
            anyhow::ensure!(ok, "the model files differ from the released ones");
            Ok(())
        }
    }
}
