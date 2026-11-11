//! Scoring, calibration and whole-suite evaluation as library functions, so
//! the same code backs the individual commands and the end-to-end ones.
#![cfg(target_os = "macos")]

use crate::detector::{Detector, Device, ModelSpec, Timing};
use crate::ingest::{self, Item};
use crate::report::{self, Scored};
use anyhow::Result;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

fn threads() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8)
}

fn items_of(manifests: &[PathBuf]) -> Result<Vec<Item>> {
    let mut items = Vec::new();
    for m in manifests {
        items.extend(ingest::read_manifest(m)?);
    }
    Ok(items)
}

/// Veracity's photo operating points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Photo {
    /// Up to six views, balanced threshold.
    Default,
    /// Two views, strict threshold.
    Fast,
    /// Up to six views, looser threshold.
    Sensitive,
    /// Up to six views, strict threshold.
    Strict,
}

impl Photo {
    fn apply(self, spec: &mut ModelSpec) {
        match self {
            Photo::Default => {}
            Photo::Fast => Detector::use_fast_profile(spec),
            Photo::Sensitive => Detector::use_sensitive_profile(spec),
            Photo::Strict => Detector::use_strict_profile(spec),
        }
    }
}

/// Which system a score run uses.
#[derive(Clone, Debug)]
pub enum System {
    /// Veracity with its sidecar spec and one of its photo profiles.
    Veracity { profile: Photo },
    CommunityForensics,
    SwinV2,
    WaveRep,
}

/// Score images; writes `path label generator dataset prob logits...`.
/// Veracity's six-view profiles stop early on clear images when the
/// checkpoint has an early-exit schedule, unless `all_views` is set; the
/// logits written are then the ones looked at, in looking order.
#[allow(clippy::too_many_arguments)]
pub fn score_images(weights: &Path, sys: &System, manifests: &[PathBuf], out: &Path, device: Device, batch: usize, views: Option<&str>, all_views: bool) -> Result<Timing> {
    let items = items_of(manifests)?;
    let (mut spec, batch) = match sys {
        System::Veracity { profile } => {
            let mut s = ModelSpec::load_for(weights)?;
            profile.apply(&mut s);
            if all_views || views.is_some() || *profile == Photo::Fast {
                s.early_exit = None;
            }
            (s, batch)
        }
        System::CommunityForensics => (ModelSpec::commfor(384), batch),
        System::SwinV2 => (ModelSpec::swinv2(), batch.min(16)),
        System::WaveRep => anyhow::bail!("WaveRep is a video detector"),
    };
    if let Some(v) = views {
        spec.views = crate::detector::parse_views(v, spec.img as u32, 256)?;
    }
    let det = Detector::load(weights, Some(spec), batch, device)?;
    let paths: Vec<PathBuf> = items.iter().map(|i| i.path.clone()).collect();
    let (res, t) = match det.spec.early_exit.clone() {
        Some(exit) => det.score_files_adaptive(&paths, threads(), &exit),
        None => det.score_files(&paths, threads()),
    };
    let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
    let mut bad = 0;
    let mut looked = 0usize;
    for (it, r) in items.iter().zip(&res) {
        let Some(l) = r else {
            bad += 1;
            continue;
        };
        looked += l.len();
        let ls: Vec<String> = l.iter().map(|x| format!("{x:.5}")).collect();
        writeln!(w, "{}\t{}\t{}\t{}\t{:.6}\t{}", it.path.display(), it.label, it.generator, it.dataset, det.probability(l), ls.join("\t"))?;
    }
    eprintln!(
        "scored {} images ({} unreadable) in {:.1}s: {:.1} img/s end-to-end, {:.1} img/s model-only, {:.2} views per image",
        t.images,
        bad,
        t.wall_s,
        t.images as f64 / t.wall_s,
        t.images as f64 / t.model_s,
        looked as f64 / (t.images - bad).max(1) as f64
    );
    Ok(t)
}

/// Clip logits from the WaveRep network, run the way its authors release it:
/// every one of the first 64 frames, centre 504 px at native resolution.
pub fn waverep_logits(weights: &Path, paths: &[PathBuf], device: Device, batch: usize) -> Result<(Vec<Option<f32>>, Timing)> {
    let det = Detector::load(weights, Some(ModelSpec::waverep()), batch.min(16), device)?;
    let sampling = crate::video::Sampling { frames: 64, consecutive: true, ..Default::default() };
    let (res, t) = det.score_videos(paths, sampling, threads().min(4));
    let clip = |fr: Vec<Vec<f32>>| crate::video::aggregate(&fr.iter().map(|v| v.iter().sum::<f32>() / v.len() as f32).collect::<Vec<f32>>());
    Ok((res.into_iter().map(|r| r.map(clip)).collect(), t))
}

/// Score clips; writes `path label generator dataset prob frame-logits...`.
/// With `second` (the WaveRep weights) and a second-opinion band
/// in the checkpoint's video profile, clips inside the band get the second
/// stage and their probability comes from both.
#[allow(clippy::too_many_arguments)]
pub fn score_videos(weights: &Path, sys: &System, manifests: &[PathBuf], out: &Path, device: Device, batch: usize, frames: Option<usize>, views: Option<&str>, second: Option<&Path>) -> Result<Timing> {
    let items = items_of(manifests)?;
    let waverep = matches!(sys, System::WaveRep);
    let mut det = Detector::load(weights, waverep.then(ModelSpec::waverep), if waverep { batch.min(16) } else { batch }, device)?;
    let band = if waverep { None } else { det.spec.video.as_ref().and_then(|v| v.second_opinion) };
    let profile_frames = if waverep { 64 } else { det.use_video_profile() };
    if let Some(v) = views {
        det.spec.views = crate::detector::parse_views(v, det.spec.img as u32, 256)?;
    }
    let sampling = crate::video::Sampling { frames: frames.unwrap_or(profile_frames), consecutive: waverep, ..Default::default() };
    let paths: Vec<PathBuf> = items.iter().map(|i| i.path.clone()).collect();
    let (res, mut t) = det.score_videos(&paths, sampling, if waverep { threads().min(4) } else { threads() });
    // One logit per frame (views averaged), then the clip logit.
    let per_frame: Vec<Option<Vec<f32>>> = res.iter().map(|r| r.as_ref().map(|fr| fr.iter().map(|v| v.iter().sum::<f32>() / v.len() as f32).collect())).collect();
    let mut probs: Vec<Option<f32>> = per_frame.iter().map(|f| f.as_ref().map(|f| det.probability(&[crate::video::aggregate(f)]))).collect();
    let mut asked = 0;
    if let (Some(w2), Some(band)) = (second, band) {
        let pick: Vec<usize> = (0..items.len()).filter(|&i| per_frame[i].as_ref().is_some_and(|f| band.wanted(crate::video::aggregate(f)))).collect();
        asked = pick.len();
        if !pick.is_empty() {
            drop(det); // free the first network before loading the second
            let (z2, t2) = waverep_logits(w2, &pick.iter().map(|&i| paths[i].clone()).collect::<Vec<_>>(), device, 16)?;
            for (&i, z2) in pick.iter().zip(z2) {
                if let (Some(f), Some(z2)) = (&per_frame[i], z2) {
                    probs[i] = Some(band.probability(crate::video::aggregate(f), z2));
                }
            }
            t.wall_s += t2.wall_s;
            t.model_s += t2.model_s;
        }
    }
    let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
    let mut bad = 0;
    for (i, it) in items.iter().enumerate() {
        let (Some(f), Some(p)) = (&per_frame[i], probs[i]) else {
            bad += 1;
            continue;
        };
        let ls: Vec<String> = f.iter().map(|x| format!("{x:.4}")).collect();
        writeln!(w, "{}\t{}\t{}\t{}\t{:.6}\t{}", it.path.display(), it.label, it.generator, it.dataset, p, ls.join("\t"))?;
    }
    eprintln!(
        "scored {} clips ({} unreadable{}) in {:.1}s: {:.2} clips/s end-to-end ({:.1} ms/clip), model time {:.1}s",
        t.images,
        bad,
        if second.is_some() { format!(", {asked} given a second opinion") } else { String::new() },
        t.wall_s,
        t.images as f64 / t.wall_s,
        1000.0 * t.wall_s / t.images.max(1) as f64,
        t.model_s
    );
    Ok(t)
}

/// What the second-opinion stage would decide, computed from a first-stage
/// score file and a WaveRep score file of the same clips (the same arithmetic
/// as running it, without scoring the band clips twice).
pub fn second_opinion_from_scores(first: &Path, waverep: &Path, weights: &Path, out: &Path) -> Result<()> {
    let spec = ModelSpec::load_for(weights)?;
    let Some(band) = spec.video.as_ref().and_then(|v| v.second_opinion) else { return Ok(()) };
    let clip_logit = |cols: &[&str]| -> f32 { crate::video::aggregate(&cols[5..].iter().filter_map(|x| x.parse::<f32>().ok()).collect::<Vec<f32>>()) };
    let text2 = std::fs::read_to_string(waverep)?;
    let z2: std::collections::HashMap<&str, f32> = text2.lines().filter_map(|l| { let c: Vec<&str> = l.split('\t').collect(); (c.len() > 5).then(|| (c[0], clip_logit(&c))) }).collect();
    let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
    for l in std::fs::read_to_string(first)?.lines() {
        let c: Vec<&str> = l.split('\t').collect();
        if c.len() < 6 {
            continue;
        }
        let z1 = clip_logit(&c);
        match z2.get(c[0]) {
            Some(&z2) if band.wanted(z1) => writeln!(w, "{}\t{}\t{}\t{}\t{:.6}\t{}", c[0], c[1], c[2], c[3], band.probability(z1, z2), c[5..].join("\t"))?,
            _ => writeln!(w, "{l}")?,
        }
    }
    Ok(())
}

/// How the decision threshold is derived from real items.
#[derive(Clone, Copy, Debug)]
pub enum Rule {
    /// Pooled over all real items.
    Pooled,
    /// Strictest threshold over real sources with at least `min` items.
    Strictest(usize),
    /// Mean per-source rate over real sources with at least `min` items.
    Balanced(usize),
}

/// Fit `profile` ("photo", "fast" or "video") of the spec next to `weights`.
pub fn calibrate(weights: &Path, scores: &[PathBuf], target_fpr: f64, profile: &str, rule: Rule) -> Result<()> {
    let mut spec = ModelSpec::load_for(weights)?;
    let (mut real, mut fake) = (Vec::new(), Vec::new());
    let mut by_source: BTreeMap<String, Vec<f32>> = Default::default();
    for p in scores {
        for r in report::read_scores(p)?.values() {
            if r.logits.is_empty() {
                continue;
            }
            let z = r.logits.iter().sum::<f32>() / r.logits.len() as f32;
            if r.label == 1 {
                fake.push(z)
            } else {
                real.push(z);
                by_source.entry(r.generator.clone()).or_default().push(z);
            }
        }
    }
    anyhow::ensure!(real.len() >= 200, "need at least 200 real items to calibrate (have {})", real.len());
    let mut thr = crate::metrics::fpr_threshold(&real, target_fpr);
    match rule {
        Rule::Pooled => {}
        Rule::Strictest(min) => {
            for (src, z) in by_source.iter().filter(|(_, z)| z.len() >= min) {
                let t = crate::metrics::fpr_threshold(z, target_fpr);
                println!("  {src:<28} n={:<5} threshold {t:.3}", z.len());
                thr = thr.max(t);
            }
        }
        Rule::Balanced(min) => {
            let srcs: Vec<&Vec<f32>> = by_source.values().filter(|z| z.len() >= min).collect();
            anyhow::ensure!(!srcs.is_empty(), "no real source has {min} items");
            let mean_fpr = |t: f32| srcs.iter().map(|z| z.iter().filter(|&&v| v > t).count() as f64 / z.len() as f64).sum::<f64>() / srcs.len() as f64;
            let (mut lo, mut hi) = (-30f32, 30f32);
            for _ in 0..60 {
                let mid = (lo + hi) / 2.0;
                if mean_fpr(mid) > target_fpr {
                    lo = mid
                } else {
                    hi = mid
                }
            }
            thr = hi;
            for (src, z) in by_source.iter().filter(|(_, z)| z.len() >= min) {
                println!("  {src:<28} n={:<5} false positives at threshold {:.2}%", z.len(), 100.0 * z.iter().filter(|&&v| v > thr).count() as f64 / z.len() as f64);
            }
        }
    }
    let (scale, bias, thr) = crate::metrics::calibrate_at(&real, &fake, thr);
    let recall = fake.iter().filter(|&&z| z > thr).count() as f64 / fake.len().max(1) as f64;
    println!("{profile}: {} real / {} AI items; logit threshold {thr:.3} for a {:.2}% target; recall on these AI items {recall:.4}", real.len(), fake.len(), 100.0 * target_fpr);
    let size = spec.img as u32;
    match profile {
        "video" => {
            let v = spec.video.get_or_insert_with(|| crate::detector::VideoSpec::standard(size, 256));
            (v.logit_scale, v.logit_bias) = (scale, bias);
        }
        "fast" => {
            let f = spec.fast.get_or_insert_with(|| crate::detector::Profile { views: crate::detector::fast_views(size, 256), logit_scale: 1.0, logit_bias: 0.0 });
            (f.logit_scale, f.logit_bias) = (scale, bias);
        }
        "sensitive" => spec.sensitive = Some(crate::detector::Profile { views: spec.views.clone(), logit_scale: scale, logit_bias: bias }),
        "strict" => spec.strict = Some(crate::detector::Profile { views: spec.views.clone(), logit_scale: scale, logit_bias: bias }),
        "photo" => (spec.logit_scale, spec.logit_bias, spec.threshold) = (scale, bias, 0.5),
        other => anyhow::bail!("unknown profile `{other}` (use photo, strict, fast, sensitive or video)"),
    }
    spec.save_for(weights)
}

/// One evaluation set: display name, manifest, and whether it holds clips.
pub struct Set {
    pub name: &'static str,
    pub manifest: &'static str,
    pub video: bool,
    /// False when this set's real items are the same files as another set's
    /// in the suite, so the pooled table counts them once.
    pub pool_reals: bool,
}

const fn img(name: &'static str, manifest: &'static str) -> Set {
    Set { name, manifest, video: false, pool_reals: true }
}
const fn vid(name: &'static str, manifest: &'static str) -> Set {
    Set { name, manifest, video: true, pool_reals: true }
}

pub fn suite(name: &str) -> Result<Vec<Set>> {
    Ok(match name {
        "benchmarks" => vec![
            img("sofake-ood", "data/store/sofake-ood/manifest.tsv"),
            img("synthbuster", "data/store/synthbuster/manifest.tsv"),
            img("aigibench", "data/store/aigibench/manifest.tsv"),
            img("wasitai-images", "data/vstore/wasitai/images.tsv"),
            img("wasitai-images_social", "data/vstore/wasitai/images_social.tsv"),
            vid("wasitai-video", "data/vstore/wasitai/manifest.tsv"),
            vid("comgenvid", "data/vstore/comgenvid/manifest.tsv"),
            vid("genvideo-val", "data/vstore/genvideo-val/manifest.tsv"),
        ],
        "confirm1" => vec![
            img("fresh-sofake", "data/fresh/sofake-ood/manifest.tsv"),
            img("fresh-socialrf", "data/fresh/socialrf/manifest.tsv"),
            img("fresh-synthbuster", "data/fresh/synthbuster/manifest.tsv"),
            vid("fresh-aigvd", "data/fresh/video/manifest.tsv"),
            vid("fresh-comgenvid", "data/fresh/comgenvid/manifest.tsv"),
            vid("wasitai-messenger", "data/vstore/wasitai/videos_social.tsv"),
            vid("fresh-genvideo", "data/fresh/genvideo-val/manifest.tsv"),
        ],
        "confirm2" => vec![
            img("fresh3-sofake-ood", "data/fresh3/sofake-ood/manifest.tsv"),
            img("fresh3-aigibench-Midjourney", "data/fresh3/aigibench-Midjourney/manifest.tsv"),
            img("fresh3-aigibench-FLUX1-dev", "data/fresh3/aigibench-FLUX1-dev/manifest.tsv"),
        ],
        "confirm3" => vec![
            img("fresh4-sofake-ood", "data/fresh4/sofake-ood/manifest.tsv"),
            img("fresh4-aigibench-DALLE-3", "data/fresh4/aigibench-DALLE-3/manifest.tsv"),
            img("fresh4-aigibench-SD3", "data/fresh4/aigibench-SD3/manifest.tsv"),
            // AIGIBench pairs SDXL with the same real photos as SD3.
            Set { pool_reals: false, ..img("fresh4-aigibench-SDXL", "data/fresh4/aigibench-SDXL/manifest.tsv") },
            img("fresh4-aigibench-Imagen3", "data/fresh4/aigibench-Imagen3/manifest.tsv"),
            vid("fresh4-aigvd", "data/fresh4/video/manifest.tsv"),
            vid("fresh4-comgenvid", "data/fresh4/comgenvid/manifest.tsv"),
            vid("fresh4-genvideo", "data/fresh4/genvideo-val/manifest.tsv"),
        ],
        "confirm4" => vec![
            vid("fresh5-aigvd", "data/fresh5/video/manifest.tsv"),
            vid("fresh5-comgenvid", "data/fresh5/comgenvid/manifest.tsv"),
            vid("fresh5-genvideo", "data/fresh5/genvideo-val/manifest.tsv"),
        ],
        "confirm5" => {
            let mut v = vec![img("fresh6-sofake-ood", "data/fresh6/sofake-ood/manifest.tsv")];
            for (name, _) in crate::plans::CONFIRM5_AIGIBENCH {
                let (set, manifest) = (format!("fresh6-aigibench-{name}"), format!("data/fresh6/aigibench-{name}/manifest.tsv"));
                v.push(img(Box::leak(set.into_boxed_str()), Box::leak(manifest.into_boxed_str())));
            }
            v
        }
        other => anyhow::bail!("unknown suite `{other}` (benchmarks, confirm1 to confirm5)"),
    })
}

/// Paths of the reference detectors' released weights.
pub struct Baselines {
    pub commfor: PathBuf,
    pub swinv2: PathBuf,
    pub waverep: PathBuf,
}

impl Default for Baselines {
    fn default() -> Self {
        Self { commfor: "baselines/commfor-384.safetensors".into(), swinv2: "baselines/swinv2-detector.safetensors".into(), waverep: "baselines/waverep-g4.ckpt".into() }
    }
}

/// What a six-view photo profile decides, from a score file holding all six
/// view logits of every image: early exit (when the checkpoint has a
/// schedule) and the profile's calibration, the same arithmetic as scoring
/// with that profile.
fn apply_profile(src: &Path, dst: &Path, weights: &Path, profile: Photo) -> Result<()> {
    let mut spec = ModelSpec::load_for(weights)?;
    if (profile == Photo::Sensitive && spec.sensitive.is_none()) || (profile == Photo::Strict && spec.strict.is_none()) {
        return Ok(());
    }
    profile.apply(&mut spec);
    let thr = -spec.logit_bias / spec.logit_scale;
    let text = std::fs::read_to_string(src)?;
    let mut w = std::io::BufWriter::new(std::fs::File::create(dst)?);
    for l in text.lines() {
        let c: Vec<&str> = l.split('\t').collect();
        if c.len() < 6 {
            continue;
        }
        let all: Vec<f32> = c[5..].iter().filter_map(|x| x.parse().ok()).collect();
        let used = match &spec.early_exit {
            Some(exit) if all.len() == spec.views.len() => exit.looked_at(&all, thr),
            _ => all,
        };
        let z = used.iter().sum::<f32>() / used.len() as f32;
        let prob = crate::metrics::sigmoid(spec.logit_scale * z + spec.logit_bias);
        let ls: Vec<String> = used.iter().map(|x| format!("{x:.5}")).collect();
        writeln!(w, "{}\t{}\t{}\t{}\t{prob:.6}\t{}", c[0], c[1], c[2], c[3], ls.join("\t"))?;
    }
    Ok(())
}

fn load(path: &Path) -> Result<BTreeMap<String, Scored>> {
    report::read_scores(path)
}

/// Score a suite and print the comparison tables. Score files are written to
/// `scores/<set>.<system>.tsv` and reused when `reuse` is set and they exist.
pub fn evaluate(suite_name: &str, weights: &Path, video_weights: &Path, tag: &str, scores: &Path, baselines: Option<&Baselines>, reuse: bool, device: Device) -> Result<()> {
    std::fs::create_dir_all(scores)?;
    let sets = suite(suite_name)?;
    let file = |set: &str, sys: &str| scores.join(format!("{set}.{sys}.tsv"));
    let need = |p: &Path| !(reuse && p.exists());
    for s in &sets {
        let m = [PathBuf::from(s.manifest)];
        if s.video {
            score_videos(video_weights, &System::Veracity { profile: Photo::Default }, &m, &file(s.name, tag), device, 64, None, None, None)?;
            if let Some(b) = baselines {
                let p = file(s.name, "waverep");
                if need(&p) {
                    score_videos(&b.waverep, &System::WaveRep, &m, &p, device, 16, None, None, None)?;
                }
                second_opinion_from_scores(&file(s.name, tag), &p, video_weights, &file(s.name, &format!("{tag}-second")))?;
            }
        } else {
            // The three six-view profiles differ only in threshold: score every view once, then apply each.
            let views = file(s.name, &format!("{tag}-views"));
            score_images(weights, &System::Veracity { profile: Photo::Default }, &m, &views, device, 64, None, true)?;
            apply_profile(&views, &file(s.name, tag), weights, Photo::Default)?;
            apply_profile(&views, &file(s.name, &format!("{tag}-strict")), weights, Photo::Strict)?;
            apply_profile(&views, &file(s.name, &format!("{tag}-sensitive")), weights, Photo::Sensitive)?;
            score_images(weights, &System::Veracity { profile: Photo::Fast }, &m, &file(s.name, &format!("{tag}-fast")), device, 64, None, false)?;
            if let Some(b) = baselines {
                for (sys, w, name) in [(System::CommunityForensics, &b.commfor, "commfor"), (System::SwinV2, &b.swinv2, "swinv2")] {
                    let p = file(s.name, name);
                    if need(&p) {
                        // The reference pipelines run in 32-bit floats, as released.
                        score_images(w, &sys, &m, &p, Device::GpuF32, 64, None, true)?;
                    }
                }
            }
        }
    }
    for (video, title) in [(false, "Images"), (true, "Video")] {
        let these: Vec<&Set> = sets.iter().filter(|s| s.video == video).collect();
        if these.is_empty() {
            continue;
        }
        println!("\n## {title}");
        let mut systems: Vec<(String, BTreeMap<String, Scored>, f32)> = Vec::new();
        let mut pooled: Vec<(String, BTreeMap<String, Scored>, f32)> = Vec::new();
        let names: Vec<(&str, String)> = if video {
            vec![("WaveRep", "waverep".into()), ("Veracity", tag.to_string()), ("Veracity + second opinion", format!("{tag}-second"))]
        } else {
            vec![
                ("Community Forensics", "commfor".into()),
                ("SwinV2 detector", "swinv2".into()),
                ("Veracity", tag.to_string()),
                ("Veracity (strict)", format!("{tag}-strict")),
                ("Veracity (fast)", format!("{tag}-fast")),
                ("Veracity (sensitive)", format!("{tag}-sensitive")),
            ]
        };
        let pooled_name = format!("{suite_name}, pooled");
        for (label, sys) in names {
            let (mut all, mut pool) = (BTreeMap::new(), BTreeMap::new());
            let mut complete = true;
            for s in &these {
                let p = file(s.name, &sys);
                if !p.exists() {
                    complete = false;
                    continue;
                }
                for (k, r) in load(&p)? {
                    if s.pool_reals || r.label == 1 {
                        pool.insert(k.clone(), Scored { dataset: pooled_name.clone(), ..r.clone() });
                    }
                    all.insert(k, r);
                }
            }
            if complete && !all.is_empty() {
                systems.push((label.to_string(), all, 0.5));
                pooled.push((label.to_string(), pool, 0.5));
            }
        }
        report::compare_markdown(&systems, None);
        if these.len() > 1 {
            report::compare_markdown(&pooled, None);
        }
    }
    Ok(())
}

fn standard_spec(size: u32) -> ModelSpec {
    ModelSpec {
        img: size as usize,
        views: crate::detector::thorough_views(size, 256),
        logit_scale: 1.0,
        logit_bias: 0.0,
        threshold: 0.5,
        prefix: String::new(),
        video: Some(crate::detector::VideoSpec::standard(size, 256)),
        fast: Some(crate::detector::Profile { views: crate::detector::fast_views(size, 256), logit_scale: 1.0, logit_bias: 0.0 }),
        sensitive: None,
        strict: None,
        early_exit: Some(crate::detector::EarlyExit::standard()),
        arch: String::new(),
    }
}

/// The validation split (by path hash) of the given training manifests, as one manifest.
fn write_val_split(list: &[&str], only_video: bool, out: &Path) -> Result<()> {
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
    for m in list.iter().filter(|m| Path::new(m).exists()) {
        for it in ingest::read_manifest(Path::new(m))? {
            if ingest::is_video(&it.path) == only_video && crate::data::is_val(&it, 3) {
                let abs = std::fs::canonicalize(&it.path).unwrap_or(it.path.clone());
                writeln!(w, "{}\t{}\t{}\t{}", abs.display(), it.label, it.generator, it.dataset)?;
            }
        }
    }
    Ok(())
}

/// Freeze a checkpoint as the video model: copy it into place, write its
/// spec, score the held-out real and generated clips and fit the video
/// threshold.
pub fn finalize_video(checkpoint: &Path, weights: &Path, scores: &Path, video_fpr: f64, second: Option<crate::detector::SecondOpinion>, device: Device) -> Result<()> {
    if let Some(p) = weights.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::create_dir_all(scores)?;
    std::fs::copy(checkpoint, weights)?;
    let mut spec = standard_spec(224);
    if let Some(v) = spec.video.as_mut() {
        v.second_opinion = second;
    }
    spec.save_for(weights)?;
    let vv = PathBuf::from("data/splits/val_videos_phase4.tsv");
    let train: Vec<&str> = crate::plans::TRAIN_VIDEO_MANIFESTS.iter().chain(crate::plans::PHASE4_VIDEO_MANIFESTS.iter()).copied().collect();
    write_val_split(&train, true, &vv)?;
    let mut videos = vec![vv];
    videos.extend(crate::plans::DEV_VIDEO_MANIFESTS.iter().map(PathBuf::from).filter(|p| p.exists()));
    let cv = scores.join("calib_videos_phase4.tsv");
    score_videos(weights, &System::Veracity { profile: Photo::Default }, &videos, &cv, device, 64, None, None, None)?;
    calibrate(weights, &[cv], video_fpr, "video", Rule::Balanced(100))
}

/// Freeze a checkpoint: copy it into place, write its spec, score the
/// calibration data and fit the three profiles.
#[allow(clippy::too_many_arguments)]
pub fn finalize(checkpoint: &Path, weights: &Path, scores: &Path, photo_fpr: f64, strict_fpr: f64, sensitive_fpr: f64, video_fpr: f64, device: Device) -> Result<()> {
    if let Some(p) = weights.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::create_dir_all(scores)?;
    std::fs::copy(checkpoint, weights)?;
    standard_spec(224).save_for(weights)?;

    // Calibration data: the validation split of every training store, the
    // development shards, and the development-visible benchmarks.
    let (vi, vv) = (PathBuf::from("data/splits/val_images.tsv"), PathBuf::from("data/splits/val_videos.tsv"));
    write_val_split(&crate::plans::TRAIN_IMAGE_MANIFESTS, false, &vi)?;
    write_val_split(&crate::plans::TRAIN_VIDEO_MANIFESTS, true, &vv)?;
    let existing = |v: Vec<&str>| -> Vec<PathBuf> { v.into_iter().map(PathBuf::from).filter(|p| p.exists()).collect() };
    let mut images = vec![vi];
    images.extend(existing(crate::plans::EVAL_IMAGE_MANIFESTS.to_vec()));
    let mut videos = vec![vv];
    videos.extend(existing(vec![
        "data/vstore/vdev/manifest.tsv",
        "data/vstore/wasitai/manifest.tsv",
        "data/vstore/comgenvid/manifest.tsv",
        "data/vstore/genvideo-val/manifest.tsv",
        "data/fresh/video/manifest.tsv",
        "data/fresh/comgenvid/manifest.tsv",
        "data/fresh/genvideo-val/manifest.tsv",
    ]));
    let (ci, cf, cv) = (scores.join("calib_images.tsv"), scores.join("calib_images_fast.tsv"), scores.join("calib_videos.tsv"));
    score_images(weights, &System::Veracity { profile: Photo::Default }, &images, &ci, device, 64, None, true)?;
    score_images(weights, &System::Veracity { profile: Photo::Fast }, &images, &cf, device, 64, None, true)?;
    score_videos(weights, &System::Veracity { profile: Photo::Default }, &videos, &cv, device, 64, None, None, None)?;
    calibrate(weights, &[ci.clone()], photo_fpr, "photo", Rule::Balanced(100))?;
    calibrate(weights, &[ci.clone()], strict_fpr, "strict", Rule::Balanced(100))?;
    calibrate(weights, &[ci], sensitive_fpr, "sensitive", Rule::Balanced(100))?;
    calibrate(weights, &[cf], strict_fpr, "fast", Rule::Balanced(100))?;
    calibrate(weights, &[cv], video_fpr, "video", Rule::Balanced(100))
}
