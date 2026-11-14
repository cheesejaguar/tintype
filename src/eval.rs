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

/// Tintype's photo operating points.
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
    /// Tintype with its sidecar spec and one of its photo profiles.
    Tintype { profile: Photo },
    CommunityForensics,
    SwinV2,
    WaveRep,
}

/// Score images; writes `path label generator dataset prob logits...`.
/// Tintype's six-view profiles stop early on clear images when the
/// checkpoint has an early-exit schedule, unless `all_views` is set; the
/// logits written are then the ones looked at, in looking order.
///
/// With `second` (the Community Forensics weights) and a second-opinion band
/// in the checkpoint's spec, photos inside the band get the second stage and
/// their probability comes from both networks; the logits written stay the
/// first stage's.
#[allow(clippy::too_many_arguments)]
pub fn score_images(weights: &Path, sys: &System, manifests: &[PathBuf], out: &Path, device: Device, batch: usize, views: Option<&str>, all_views: bool, second: Option<&Path>) -> Result<Timing> {
    let items = items_of(manifests)?;
    let (mut spec, batch) = match sys {
        System::Tintype { profile } => {
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
    let (res, mut t) = match det.spec.early_exit.clone() {
        Some(exit) => det.score_files_adaptive(&paths, threads(), &exit),
        None => det.score_files(&paths, threads()),
    };
    let mut probs: Vec<Option<f32>> = res.iter().map(|r| r.as_ref().map(|l| det.probability(l))).collect();
    let mut asked = 0;
    if let Some(w2) = second {
        let band = det.spec.second_opinion.ok_or_else(|| anyhow::anyhow!("{} has no second-opinion band for photos", weights.display()))?;
        let first: Vec<Option<f32>> = res.iter().map(|r| r.as_ref().map(|l| det.calibrated_logit(l))).collect();
        drop(det); // free the first network before loading the second
        let (second, n, t2) = crate::detector::photo_second_opinion(&band, w2, &paths, &first, device, threads())?;
        for (i, p) in second {
            probs[i] = Some(p);
        }
        asked = n;
        t.wall_s += t2.wall_s;
        t.model_s += t2.model_s;
    }
    let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
    let mut bad = 0;
    let mut looked = 0usize;
    for ((it, r), p) in items.iter().zip(&res).zip(&probs) {
        let (Some(l), Some(p)) = (r, p) else {
            bad += 1;
            continue;
        };
        looked += l.len();
        let ls: Vec<String> = l.iter().map(|x| format!("{x:.5}")).collect();
        writeln!(w, "{}\t{}\t{}\t{}\t{:.6}\t{}", it.path.display(), it.label, it.generator, it.dataset, p, ls.join("\t"))?;
    }
    eprintln!(
        "scored {} images ({} unreadable{}) in {:.1}s: {:.1} img/s end-to-end, {:.1} img/s model-only, {:.2} views per image",
        t.images,
        bad,
        if second.is_some() { format!(", {asked} given a second opinion") } else { String::new() },
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
///
/// Clips stop early when the checkpoint's video profile has an early-exit
/// schedule: the frame logits written are then those of the frames looked
/// at, in looking order. `all_frames` turns that off, and so does a second
/// opinion, whose band is defined on the logit of all frames.
#[allow(clippy::too_many_arguments)]
pub fn score_videos(weights: &Path, sys: &System, manifests: &[PathBuf], out: &Path, device: Device, batch: usize, frames: Option<usize>, views: Option<&str>, second: Option<&Path>, all_frames: bool) -> Result<Timing> {
    let items = items_of(manifests)?;
    let waverep = matches!(sys, System::WaveRep);
    let exit = if waverep || all_frames || second.is_some() { None } else { ModelSpec::load_for(weights).ok().and_then(|s| s.video).and_then(|v| v.early_exit) };
    // Early exit runs 2, 4 or 8 frames of six views at a time: 96 holds whole clips at every stage.
    let batch = if exit.is_some() { 96 } else { batch };
    let mut det = Detector::load(weights, waverep.then(ModelSpec::waverep), if waverep { batch.min(16) } else { batch }, device)?;
    let band = if waverep { None } else { det.spec.video.as_ref().and_then(|v| v.second_opinion) };
    let profile_frames = if waverep { 64 } else { det.use_video_profile() };
    if let Some(v) = views {
        det.spec.views = crate::detector::parse_views(v, det.spec.img as u32, 256)?;
    }
    let sampling = crate::video::Sampling { frames: frames.unwrap_or(profile_frames), consecutive: waverep, ..Default::default() };
    let paths: Vec<PathBuf> = items.iter().map(|i| i.path.clone()).collect();
    let (res, mut t) = match &exit {
        Some(e) => det.score_videos_adaptive(&paths, sampling, threads(), e),
        None => det.score_videos(&paths, sampling, if waverep { threads().min(4) } else { threads() }),
    };
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

/// What the photo second-opinion stage would decide, computed from a
/// default-profile score file and a Community Forensics score file of the
/// same photos (the same arithmetic as running it, without scoring the band
/// photos twice). Writes nothing when the checkpoint has no band.
pub fn photo_second_opinion_from_scores(first: &Path, commfor: &Path, weights: &Path, out: &Path) -> Result<()> {
    let spec = ModelSpec::load_for(weights)?;
    let Some(band) = spec.second_opinion else { return Ok(()) };
    let mean = |cols: &[&str]| -> f32 {
        let v: Vec<f32> = cols[5..].iter().filter_map(|x| x.parse::<f32>().ok()).collect();
        v.iter().sum::<f32>() / v.len().max(1) as f32
    };
    let text2 = std::fs::read_to_string(commfor)?;
    let z2: std::collections::HashMap<&str, f32> = text2.lines().filter_map(|l| { let c: Vec<&str> = l.split('\t').collect(); (c.len() > 5).then(|| (c[0], mean(&c))) }).collect();
    let mut w = std::io::BufWriter::new(std::fs::File::create(out)?);
    for l in std::fs::read_to_string(first)?.lines() {
        let c: Vec<&str> = l.split('\t').collect();
        if c.len() < 6 {
            continue;
        }
        let z1 = spec.logit_scale * mean(&c) + spec.logit_bias;
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
    /// The loosest threshold whose one-sided Clopper–Pearson upper bound on
    /// the false-positive rate, over independent real items, is within the
    /// target ([`crate::calibration::bound_cutoff`]). One global budget: no
    /// source gets a threshold of its own.
    Bound,
}

/// What [`calibrate_with`] takes beyond the rule. The defaults are what
/// [`calibrate`] has always done.
#[derive(Clone, Debug)]
pub struct CalibrateOptions {
    /// Confidence of the Clopper–Pearson bound: the one [`Rule::Bound`]
    /// fits to, and the one printed for every rule.
    pub confidence: f64,
    /// Manifest rows by path. With them, real items that share a
    /// `parent_id` count once towards the bound.
    pub groups: BTreeMap<String, Item>,
    /// Which score stands for a parent with derivatives.
    pub per_parent: crate::calibration::PerParent,
    /// Print, and never store, what each group of this kind (`source`,
    /// `transform`, `dataset`, `generator`, `family`) would get as a
    /// threshold of its own.
    pub group_diagnostic: Option<String>,
    /// Smallest group the diagnostic lists.
    pub min_group: usize,
}

impl Default for CalibrateOptions {
    fn default() -> Self {
        Self { confidence: 0.95, groups: BTreeMap::new(), per_parent: crate::calibration::PerParent::Max, group_diagnostic: None, min_group: 100 }
    }
}

/// Fit `profile` ("photo", "fast" or "video") of the spec next to `weights`.
pub fn calibrate(weights: &Path, scores: &[PathBuf], target_fpr: f64, profile: &str, rule: Rule) -> Result<()> {
    calibrate_with(weights, scores, target_fpr, profile, rule, &CalibrateOptions::default())
}

/// As [`calibrate`], with the options of `tintype calibrate`.
pub fn calibrate_with(weights: &Path, scores: &[PathBuf], target_fpr: f64, profile: &str, rule: Rule, opts: &CalibrateOptions) -> Result<()> {
    let mut spec = ModelSpec::load_for(weights)?;
    let (mut real, mut fake) = (Vec::new(), Vec::new());
    let mut by_source: BTreeMap<String, Vec<f32>> = Default::default();
    // The manifest row of each real item (grouping columns from `opts.groups`), in step with `real`.
    let mut real_items: Vec<Item> = Vec::new();
    for p in scores {
        for (path, r) in report::read_scores(p)?.iter() {
            if r.logits.is_empty() {
                continue;
            }
            let z = r.logits.iter().sum::<f32>() / r.logits.len() as f32;
            if r.label == 1 {
                fake.push(z)
            } else {
                real.push(z);
                by_source.entry(r.generator.clone()).or_default().push(z);
                let extra = opts.groups.get(path).map(|it| it.extra.clone()).unwrap_or_default();
                real_items.push(Item { path: path.into(), label: 0, generator: r.generator.clone(), dataset: r.dataset.clone(), extra });
            }
        }
    }
    anyhow::ensure!(real.len() >= 200, "need at least 200 real items to calibrate (have {})", real.len());
    // One score per independent real item: derivatives of one parent count once.
    let units = if opts.groups.is_empty() {
        crate::calibration::Units { scores: real.clone(), items: real.len(), dropped: 0 }
    } else {
        let parents: Vec<Option<&str>> = real_items.iter().map(|it| it.parent_id()).collect();
        let original: Vec<bool> = real_items.iter().map(|it| it.transform().is_none()).collect();
        crate::calibration::one_per_parent(&real, &parents, &original, opts.per_parent)
    };
    let mut thr = crate::metrics::fpr_threshold(&real, target_fpr);
    match rule {
        Rule::Pooled => {}
        Rule::Bound => thr = crate::calibration::bound_cutoff(&units.scores, target_fpr, opts.confidence)?.cutoff,
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
    // What the threshold achieves, whatever rule set it: the bound counts independent items.
    let (flagged, n, upper) = crate::calibration::achieved(&units.scores, thr, opts.confidence);
    let counted = if opts.groups.is_empty() {
        "real items, taken as independent".to_string()
    } else {
        format!(
            "real parents ({} items, one score per parent: {}{})",
            units.items,
            match opts.per_parent {
                crate::calibration::PerParent::Max => "the highest of its versions",
                crate::calibration::PerParent::Original => "its unprocessed original",
            },
            if units.dropped > 0 { format!("; {} without an original left out", units.dropped) } else { String::new() }
        )
    };
    println!("  flagged at the threshold: {flagged} of {n} {counted}; one-sided {}% Clopper–Pearson upper bound on the false-positive rate {:.3}%", 100.0 * opts.confidence, 100.0 * upper);
    if !matches!(rule, Rule::Balanced(_)) {
        for (src, z) in &by_source {
            println!("  {src:<28} n={:<5} false positives at threshold {:.2}%", z.len(), 100.0 * z.iter().filter(|&&v| v > thr).count() as f64 / z.len() as f64);
        }
    }
    if let Some(by) = &opts.group_diagnostic {
        println!("  DIAGNOSTIC ONLY, by {by}: the threshold each group would get on its own at this target. Nothing below is stored or used at inference. A group threshold is admissible only where the group can be observed at inference, has enough calibration data and is shown to transfer.");
        let mut by_group: BTreeMap<String, Vec<f32>> = Default::default();
        for (it, &z) in real_items.iter().zip(&real) {
            by_group.entry(report::slice_value(by, it)?).or_default().push(z);
        }
        for (group, z) in by_group.iter().filter(|(_, z)| z.len() >= opts.min_group) {
            let own = crate::metrics::fpr_threshold(z, target_fpr);
            let (k, n, up) = crate::calibration::achieved(z, thr, opts.confidence);
            println!("  [diagnostic] {group:<28} n={n:<5} own threshold {own:.3} ({:+.3} from the global one); at the global threshold {k} flagged, upper bound {:.3}%", own - thr, 100.0 * up);
        }
    }
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

/// The suites and their sets live with the dataset recipes.
pub use crate::plans::{suite, Set};

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
            score_videos(video_weights, &System::Tintype { profile: Photo::Default }, &m, &file(s.name, tag), device, 64, None, None, None, false)?;
            if let Some(b) = baselines {
                let p = file(s.name, "waverep");
                if need(&p) {
                    score_videos(&b.waverep, &System::WaveRep, &m, &p, device, 16, None, None, None, false)?;
                }
                second_opinion_from_scores(&file(s.name, tag), &p, video_weights, &file(s.name, &format!("{tag}-second")))?;
            }
        } else {
            // The three six-view profiles differ only in threshold: score every view once, then apply each.
            let views = file(s.name, &format!("{tag}-views"));
            score_images(weights, &System::Tintype { profile: Photo::Default }, &m, &views, device, 64, None, true, None)?;
            apply_profile(&views, &file(s.name, tag), weights, Photo::Default)?;
            apply_profile(&views, &file(s.name, &format!("{tag}-strict")), weights, Photo::Strict)?;
            apply_profile(&views, &file(s.name, &format!("{tag}-sensitive")), weights, Photo::Sensitive)?;
            score_images(weights, &System::Tintype { profile: Photo::Fast }, &m, &file(s.name, &format!("{tag}-fast")), device, 64, None, false, None)?;
            if let Some(b) = baselines {
                for (sys, w, name) in [(System::CommunityForensics, &b.commfor, "commfor"), (System::SwinV2, &b.swinv2, "swinv2")] {
                    let p = file(s.name, name);
                    if need(&p) {
                        // The reference pipelines run in 32-bit floats, as released.
                        score_images(w, &sys, &m, &p, Device::GpuF32, 64, None, true, None)?;
                    }
                }
                photo_second_opinion_from_scores(&file(s.name, tag), &file(s.name, "commfor"), weights, &file(s.name, &format!("{tag}-second")))?;
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
            vec![("WaveRep", "waverep".into()), ("Tintype", tag.to_string()), ("Tintype + second opinion", format!("{tag}-second"))]
        } else {
            vec![
                ("Community Forensics", "commfor".into()),
                ("SwinV2 detector", "swinv2".into()),
                ("Tintype", tag.to_string()),
                ("Tintype + second opinion", format!("{tag}-second")),
                ("Tintype (strict)", format!("{tag}-strict")),
                ("Tintype (fast)", format!("{tag}-fast")),
                ("Tintype (sensitive)", format!("{tag}-sensitive")),
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
        second_opinion: None,
        review: None,
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
                writeln!(w, "{}", it.manifest_line(&abs.display().to_string()))?;
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
        v.early_exit = Some(crate::detector::EarlyExit::video_standard());
    }
    spec.save_for(weights)?;
    let vv = PathBuf::from("data/splits/val_videos_phase4.tsv");
    let train: Vec<&str> = crate::plans::TRAIN_VIDEO_MANIFESTS.iter().chain(crate::plans::PHASE4_VIDEO_MANIFESTS.iter()).copied().collect();
    write_val_split(&train, true, &vv)?;
    let mut videos = vec![vv];
    videos.extend(crate::plans::DEV_VIDEO_MANIFESTS.iter().map(PathBuf::from).filter(|p| p.exists()));
    let cv = scores.join("calib_videos_phase4.tsv");
    score_videos(weights, &System::Tintype { profile: Photo::Default }, &videos, &cv, device, 64, None, None, None, true)?;
    calibrate(weights, &[cv], video_fpr, "video", Rule::Balanced(100))
}

/// Freeze a checkpoint: copy it into place, write its spec, score the
/// calibration data and fit the three profiles.
#[allow(clippy::too_many_arguments)]
/// `later` adds what came after the phase-3 weights: `later.0` are training
/// manifests whose validation split joins the calibration data, `later.1`
/// are evaluation manifests scored since (their real photos join it too).
#[allow(clippy::too_many_arguments)]
pub fn finalize(checkpoint: &Path, weights: &Path, scores: &Path, photo_fpr: f64, strict_fpr: f64, sensitive_fpr: f64, video_fpr: f64, later: (&[PathBuf], &[PathBuf]), device: Device) -> Result<()> {
    if let Some(p) = weights.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::create_dir_all(scores)?;
    std::fs::copy(checkpoint, weights)?;
    standard_spec(224).save_for(weights)?;

    // Calibration data: the validation split of every training store, the
    // development shards, and the development-visible benchmarks.
    let (vi, vv) = (PathBuf::from("data/splits/val_images.tsv"), PathBuf::from("data/splits/val_videos.tsv"));
    let later_train: Vec<String> = later.0.iter().map(|p| p.to_string_lossy().to_string()).collect();
    let train_images: Vec<&str> = crate::plans::TRAIN_IMAGE_MANIFESTS.iter().copied().chain(later_train.iter().map(String::as_str)).collect();
    write_val_split(&train_images, false, &vi)?;
    write_val_split(&crate::plans::TRAIN_VIDEO_MANIFESTS, true, &vv)?;
    let existing = |v: Vec<&str>| -> Vec<PathBuf> { v.into_iter().map(PathBuf::from).filter(|p| p.exists()).collect() };
    let mut images = vec![vi];
    images.extend(existing(crate::plans::EVAL_IMAGE_MANIFESTS.to_vec()));
    images.extend(later.1.iter().filter(|p| p.exists()).cloned());
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
    score_images(weights, &System::Tintype { profile: Photo::Default }, &images, &ci, device, 64, None, true, None)?;
    score_images(weights, &System::Tintype { profile: Photo::Fast }, &images, &cf, device, 64, None, true, None)?;
    score_videos(weights, &System::Tintype { profile: Photo::Default }, &videos, &cv, device, 64, None, None, None, true)?;
    calibrate(weights, &[ci.clone()], photo_fpr, "photo", Rule::Balanced(100))?;
    calibrate(weights, &[ci.clone()], strict_fpr, "strict", Rule::Balanced(100))?;
    calibrate(weights, &[ci], sensitive_fpr, "sensitive", Rule::Balanced(100))?;
    calibrate(weights, &[cf], strict_fpr, "fast", Rule::Balanced(100))?;
    calibrate(weights, &[cv], video_fpr, "video", Rule::Balanced(100))
}
