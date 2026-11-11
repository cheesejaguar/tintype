use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use veracity::detector::{Detector, Device, ModelSpec};
use veracity::ingest;

#[derive(Parser)]
#[command(name = "veracity", version, about = "AI-generated image and video detection")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Decide whether images or videos are AI-generated.
    Detect {
        /// Image or video files.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Photo model weights.
        #[arg(long, default_value = "models/veracity.safetensors")]
        weights: PathBuf,
        /// Video model weights (default: `<weights>-video.safetensors` beside the photo weights, if present).
        #[arg(long)]
        video_weights: Option<PathBuf>,
        /// WaveRep weights (`veracity data baselines` fetches them; research-only licence).
        /// Videos the video model is unsure about are also scored by that network.
        #[arg(long)]
        second_opinion: Option<PathBuf>,
        /// Frames analysed per video (default: the model's video profile).
        #[arg(long)]
        frames: Option<usize>,
        /// Two views per image at the strict threshold: the highest throughput, lower recall.
        #[arg(long)]
        fast: bool,
        /// Looser threshold: finds more AI images, flags more real ones.
        #[arg(long, conflicts_with = "fast")]
        sensitive: bool,
        /// Tighter threshold: a positive verdict that stands on its own, at lower recall.
        #[arg(long, conflicts_with_all = ["fast", "sensitive"])]
        strict: bool,
        /// Look at all six views of every image instead of stopping early on clear ones.
        #[arg(long)]
        all_views: bool,
        /// Print one JSON object per file.
        #[arg(long)]
        json: bool,
        /// Engine: auto, gpu (Apple GPU, 16-bit), gpu-f32, or cpu.
        #[arg(long, default_value = "auto")]
        device: String,
    },
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
    },
    /// Score a whole evaluation suite and print the comparison tables.
    Evaluate {
        /// benchmarks, confirm1, confirm2, confirm3 or confirm4.
        suite: String,
        #[arg(long, default_value = "models/veracity.safetensors")]
        weights: PathBuf,
        /// Video model weights (default: `<weights>-video.safetensors` beside the photo weights, if present).
        #[arg(long)]
        video_weights: Option<PathBuf>,
        /// Suffix of the score files written for Veracity.
        #[arg(long, default_value = "veracity")]
        tag: String,
        #[arg(long, default_value = "runs/scores")]
        scores: PathBuf,
        /// Also run the three reference detectors (weights from `veracity data baselines`).
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
        #[arg(long, default_value = "models/veracity.safetensors")]
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
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Freeze a checkpoint as the video model: write its spec and calibrate the video threshold.
    FinalizeVideo {
        checkpoint: PathBuf,
        #[arg(long, default_value = "models/veracity-video.safetensors")]
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
    /// Download and prepare a dataset plan (`veracity data list` shows them).
    Data {
        plan: String,
        /// Resolve and print the downloads without fetching anything.
        #[arg(long)]
        dry_run: bool,
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
fn system(reference: &Option<String>, fast: bool, sensitive: bool, strict: bool) -> Result<veracity::eval::System> {
    use veracity::eval::{Photo, System};
    Ok(match reference.as_deref() {
        None => System::Veracity { profile: if fast { Photo::Fast } else if sensitive { Photo::Sensitive } else if strict { Photo::Strict } else { Photo::Default } },
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
        Cmd::Detect { paths, weights, video_weights, second_opinion, frames, fast, sensitive, strict, all_views, json, device } => {
            let device = parse_device(&device)?;
            let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
            let (vids, imgs): (Vec<PathBuf>, Vec<PathBuf>) = paths.into_iter().partition(|p| ingest::is_video(p));
            let mut rows: Vec<(PathBuf, &str, Option<f32>, usize)> = Vec::new();
            if !imgs.is_empty() {
                let mut spec = ModelSpec::load_for(&weights)?;
                if fast {
                    Detector::use_fast_profile(&mut spec);
                }
                if sensitive {
                    Detector::use_sensitive_profile(&mut spec);
                }
                if strict {
                    Detector::use_strict_profile(&mut spec);
                }
                let exit = spec.early_exit.clone().filter(|_| !fast && !all_views);
                let batch = 64.min(imgs.len() * spec.views.len());
                // On the GPU one batch costs about the same whatever is in it: when every
                // view of every image fits in one, run them all and apply the stopping
                // rule afterwards (the same verdicts as stopping early).
                let one_batch = !matches!(device, Device::Cpu) && imgs.len() * spec.views.len() <= 64;
                let det = Detector::load(&weights, Some(spec), batch, device)?;
                let (res, _) = match &exit {
                    Some(e) if one_batch => {
                        let (all, t) = det.score_files(&imgs, threads);
                        (all.into_iter().map(|r| r.map(|l| e.looked_at(&l, det.threshold_logit()))).collect(), t)
                    }
                    Some(e) => det.score_files_adaptive(&imgs, threads, e),
                    None => det.score_files(&imgs, threads),
                };
                rows.extend(imgs.iter().zip(res).map(|(p, r)| (p.clone(), "image", r.map(|l| det.probability(&l)), 1)));
            }
            if !vids.is_empty() {
                let video_weights = video_weights.unwrap_or_else(|| veracity::detector::video_weights_for(&weights));
                let mut det = Detector::load(&video_weights, None, 64, device)?;
                let profile_frames = det.use_video_profile();
                let sampling = veracity::video::Sampling { frames: frames.unwrap_or(profile_frames), ..Default::default() };
                let (res, _) = det.score_videos(&vids, sampling, threads);
                // Clip logit (mean over frames of the mean over views) and frame count.
                let clips: Vec<Option<(f32, usize)>> = res
                    .iter()
                    .map(|r| r.as_ref().map(|fr| (veracity::video::aggregate(&fr.iter().map(|v| v.iter().sum::<f32>() / v.len() as f32).collect::<Vec<f32>>()), fr.len())))
                    .collect();
                let mut probs: Vec<Option<f32>> = clips.iter().map(|c| c.map(|(z, _)| det.probability(&[z]))).collect();
                if let Some(w2) = &second_opinion {
                    let band = det.spec.video.as_ref().and_then(|v| v.second_opinion).ok_or_else(|| anyhow::anyhow!("{} has no second-opinion band in its video profile", video_weights.display()))?;
                    let pick: Vec<usize> = (0..vids.len()).filter(|&i| clips[i].is_some_and(|(z, _)| band.wanted(z))).collect();
                    if !pick.is_empty() {
                        drop(det);
                        #[cfg(target_os = "macos")]
                        {
                            let (z2, _) = veracity::eval::waverep_logits(w2, &pick.iter().map(|&i| vids[i].clone()).collect::<Vec<_>>(), device, 16)?;
                            for (&i, z2) in pick.iter().zip(z2) {
                                if let (Some((z1, _)), Some(z2)) = (clips[i], z2) {
                                    probs[i] = Some(band.probability(z1, z2));
                                }
                            }
                        }
                        #[cfg(not(target_os = "macos"))]
                        anyhow::bail!("--second-opinion runs on the Apple GPU engine only (weights: {})", w2.display());
                    }
                }
                rows.extend(vids.iter().enumerate().map(|(i, p)| (p.clone(), "video", probs[i], clips[i].map_or(0, |(_, n)| n))));
            }
            for (path, kind, prob, n) in rows {
                match (prob, json) {
                    (Some(p), true) => println!("{}", serde_json::json!({"path": path, "kind": kind, "ai_probability": p, "verdict": if p > 0.5 { "ai-generated" } else { "likely-real" }, "frames": n})),
                    (Some(p), false) => println!("{:<14} {:>5.1}%  {}", if p > 0.5 { "AI-GENERATED" } else { "likely real" }, 100.0 * p, path.display()),
                    (None, true) => println!("{}", serde_json::json!({"path": path, "kind": kind, "error": "could not decode"})),
                    (None, false) => println!("{:<14} {:>6}  {}", "unreadable", "-", path.display()),
                }
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Score { weights, reference, manifest, out, batch, device, views, fast, sensitive, strict, all_views } => {
            veracity::eval::score_images(&weights, &system(&reference, fast, sensitive, strict)?, &manifest, &out, parse_device(&device)?, batch, views.as_deref(), all_views).map(|_| ())
        }
        #[cfg(target_os = "macos")]
        Cmd::ScoreVideo { weights, reference, manifest, out, frames, views, second_opinion, batch, device } => {
            veracity::eval::score_videos(&weights, &system(&reference, false, false, false)?, &manifest, &out, parse_device(&device)?, batch, frames, views.as_deref(), second_opinion.as_deref()).map(|_| ())
        }
        Cmd::Report { systems, per_generator, markdown, pooled } => {
            let mut loaded = Vec::new();
            for s in &systems {
                let (name, rest) = s.split_once('=').ok_or_else(|| anyhow::anyhow!("use name=scores.tsv[@threshold]"))?;
                let (path, thr) = match rest.rsplit_once('@') {
                    Some((p, t)) => (p, t.parse::<f32>()?),
                    None => (rest, 0.5),
                };
                loaded.push((name.to_string(), veracity::report::read_scores(std::path::Path::new(path))?, thr));
            }
            if markdown {
                veracity::report::compare_markdown(&loaded, pooled.as_deref());
            } else {
                veracity::report::compare(&loaded, per_generator);
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Evaluate { suite, weights, video_weights, tag, scores, baselines, reuse, device } => {
            let b = veracity::eval::Baselines::default();
            let video_weights = video_weights.unwrap_or_else(|| veracity::detector::video_weights_for(&weights));
            veracity::eval::evaluate(&suite, &weights, &video_weights, &tag, &scores, baselines.then_some(&b), reuse, parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::Finalize { checkpoint, weights, scores, photo_fpr, strict_fpr, sensitive_fpr, video_fpr, device } => {
            veracity::eval::finalize(&checkpoint, &weights, &scores, photo_fpr, strict_fpr, sensitive_fpr, video_fpr, parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::FinalizeVideo { checkpoint, weights, scores, video_fpr, second_opinion, device } => {
            let band = match second_opinion {
                Some(s) => {
                    let v: Vec<f32> = s.split(',').map(|x| x.trim().parse::<f32>()).collect::<Result<_, _>>()?;
                    anyhow::ensure!(v.len() == 3 && v[0] < v[1], "--second-opinion takes lo,hi,threshold with lo < hi");
                    Some(veracity::detector::SecondOpinion { lo: v[0], hi: v[1], threshold: v[2] })
                }
                None => None,
            };
            veracity::eval::finalize_video(&checkpoint, &weights, &scores, video_fpr, band, parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::Calibrate { weights, scores, target_fpr, profile, early_exit, rule, min_source } => {
            if early_exit {
                let mut spec = ModelSpec::load_for(&weights)?;
                spec.early_exit = Some(veracity::detector::EarlyExit::standard());
                spec.save_for(&weights)?;
            }
            use veracity::eval::Rule;
            let rule = match rule.as_str() {
                "pooled" => Rule::Pooled,
                "strictest" => Rule::Strictest(min_source),
                "balanced" => Rule::Balanced(min_source),
                other => anyhow::bail!("unknown rule `{other}` (pooled, strictest, balanced)"),
            };
            veracity::eval::calibrate(&weights, &scores, target_fpr, &profile, rule)
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
            let mut buf = vec![0f32; nv * 3 * det.spec.img * det.spec.img];
            let mut ms = Vec::new();
            for (k, it) in items.iter().take(n + 5).enumerate() {
                let t = std::time::Instant::now();
                let Ok(img) = veracity::image_io::load(&it.path) else { continue };
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
        Cmd::Train { init, init_prefix, manifest, out, steps, batch, lr, warmup, val_pct, val_max, eval_every, threads, seed, p_native, layer_decay, weight_decay, smoothing, sampler_alpha, p_messenger } => {
            veracity::train_cmd::run(veracity::train_cmd::TrainArgs {
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
                aug: veracity::data::AugConfig { p_native, p_messenger, ..Default::default() },
                opt: veracity::trainer::OptConfig { layer_decay, weight_decay, smoothing, ..Default::default() },
                sampler_alpha,
            })
        }
        #[cfg(not(target_os = "macos"))]
        Cmd::Score { .. } | Cmd::ScoreVideo { .. } | Cmd::Evaluate { .. } | Cmd::Finalize { .. } | Cmd::FinalizeVideo { .. } | Cmd::Calibrate { .. } | Cmd::Latency { .. } | Cmd::Train { .. } => {
            anyhow::bail!("this command uses the Apple GPU engine and needs macOS on Apple Silicon; `detect --device cpu` works everywhere")
        }
        Cmd::Data { plan, dry_run, connections } => {
            if plan == "list" {
                for (name, what) in veracity::plans::PLANS {
                    println!("{name:<14} {what}");
                }
                return Ok(());
            }
            veracity::plans::run(&plan, dry_run, connections)
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
        Cmd::CheckVideos { manifest, prune, recompress } => {
            use rayon::prelude::*;
            for m in &manifest {
                let items = ingest::read_manifest(m)?;
                let bad: Vec<(PathBuf, String)> = items
                    .par_iter()
                    .filter_map(|it| {
                        let t = std::time::Instant::now();
                        let mut r = veracity::video::middle_frame(&it.path);
                        if recompress && r.is_ok() {
                            let seed = it.path.to_string_lossy().bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3));
                            let mut rng = veracity::rng::Rng::new(seed);
                            r = veracity::video::recompressed_only(&it.path, &mut rng, &std::env::temp_dir());
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
    }
}
