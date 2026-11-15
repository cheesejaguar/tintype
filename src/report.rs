//! Compare score files (as written by `tintype score` or the baseline
//! harness) with one shared metric implementation.
use crate::metrics::{self, Metrics};
use crate::review::{self, Outcome, Reason, Review};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone)]
pub struct Scored {
    pub label: u8,
    pub generator: String,
    pub dataset: String,
    pub prob: f32,
    pub logits: Vec<f32>,
}

pub fn read_scores(path: &Path) -> Result<BTreeMap<String, Scored>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = BTreeMap::new();
    for l in text.lines() {
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() < 5 {
            continue;
        }
        out.insert(
            f[0].to_string(),
            Scored { label: f[1].parse()?, generator: f[2].into(), dataset: f[3].into(), prob: f[4].parse()?, logits: f[5..].iter().filter_map(|x| x.parse().ok()).collect() },
        );
    }
    Ok(out)
}

pub fn metrics_of(rows: &[&Scored], threshold: f32) -> Metrics {
    let p: Vec<f32> = rows.iter().map(|r| r.prob).collect();
    let l: Vec<u8> = rows.iter().map(|r| r.label).collect();
    metrics::compute(&p, &l, threshold)
}

pub fn line(name: &str, m: &Metrics) -> String {
    format!(
        "{name:<34} n={:<6} P {:.4}  R {:.4}  F1 {:.4}  acc {:.4}  FPR {:.4}  AUROC {:.4}  AP {:.4}",
        m.n, m.precision, m.recall, m.f1, m.accuracy, m.fpr, m.auroc, m.average_precision
    )
}

/// Print overall, per-dataset and (optionally) per-generator results for each
/// system, restricted to the images every system scored.
pub fn compare(systems: &[(String, BTreeMap<String, Scored>, f32)], per_generator: bool) {
    let common: Vec<&String> = systems[0].1.keys().filter(|k| systems.iter().all(|s| s.1.contains_key(*k))).collect();
    println!("{} images scored by all {} systems", common.len(), systems.len());
    let datasets: std::collections::BTreeSet<&str> = common.iter().map(|k| systems[0].1[*k].dataset.as_str()).collect();
    for (name, scores, thr) in systems {
        println!("\n== {name} (threshold {thr})");
        let all: Vec<&Scored> = common.iter().map(|k| &scores[*k]).collect();
        println!("{}", line("ALL", &metrics_of(&all, *thr)));
        for d in &datasets {
            let rows: Vec<&Scored> = all.iter().copied().filter(|r| r.dataset == *d).collect();
            println!("{}", line(d, &metrics_of(&rows, *thr)));
        }
        if per_generator {
            let mut by: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
            for r in &all {
                let e = by.entry(r.generator.as_str()).or_default();
                e.0 += 1;
                e.1 += (r.prob > *thr) as usize;
            }
            for (g, (n, flagged)) in by {
                let kind = if g.starts_with("real") { "flagged (false positives)" } else { "detected (recall)" };
                println!("    {g:<28} {flagged:>5}/{n:<5} = {:.3} {kind}", flagged as f64 / n as f64);
            }
        }
    }
}

fn probs_labels(rows: &[&Scored]) -> (Vec<f32>, Vec<u8>) {
    (rows.iter().map(|r| r.prob).collect(), rows.iter().map(|r| r.label).collect())
}

/// Markdown table of every other system operated at the baseline's own
/// false-positive count on each dataset (and pooled): the baseline keeps its
/// shipped threshold, the others get the strictest threshold that flags no
/// more real items than the baseline did. Datasets without real items are
/// skipped.
pub fn matched_markdown(systems: &[(String, BTreeMap<String, Scored>, f32)], baseline: &str, pooled_name: Option<&str>) -> Result<()> {
    let b = systems.iter().position(|s| s.0 == baseline).with_context(|| format!("no system named `{baseline}`"))?;
    let common: Vec<&String> = systems[0].1.keys().filter(|k| systems.iter().all(|s| s.1.contains_key(*k))).collect();
    let mut datasets: Vec<&str> = common.iter().map(|k| systems[0].1[*k].dataset.as_str()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    let pooled = pooled_name.filter(|_| datasets.len() > 1);
    if let Some(p) = pooled {
        datasets.push(p);
    }
    let others: Vec<usize> = (0..systems.len()).filter(|&i| i != b).collect();
    let mut head = format!("| Set | {baseline}: real items flagged | {baseline}: precision / recall |");
    for &i in &others {
        head.push_str(&format!(" {} at that rate: precision / recall |", systems[i].0));
    }
    println!("{head}");
    println!("|---|---|---|{}", "---|".repeat(others.len()));
    for d in datasets {
        let split = |scores: &BTreeMap<String, Scored>| -> (Vec<f32>, Vec<f32>) {
            let (mut real, mut fake) = (Vec::new(), Vec::new());
            for r in common.iter().map(|k| &scores[*k]).filter(|r| Some(d) == pooled || r.dataset == d) {
                if r.label == 1 { fake.push(r.prob) } else { real.push(r.prob) }
            }
            (real, fake)
        };
        let (real, fake) = split(&systems[b].1);
        if real.is_empty() || fake.is_empty() {
            continue;
        }
        let thr = systems[b].2;
        let (fp, tp) = (real.iter().filter(|&&x| x > thr).count(), fake.iter().filter(|&&x| x > thr).count());
        let pr = |fp: usize, tp: usize, n: usize| format!("{:.3} / {:.3}", tp as f64 / (tp + fp).max(1) as f64, tp as f64 / n as f64);
        let mut row = format!("| {d} | {fp} of {} ({:.2}%) | {} |", real.len(), 100.0 * fp as f64 / real.len() as f64, pr(fp, tp, fake.len()));
        for &i in &others {
            let (r, f) = split(&systems[i].1);
            let (fp2, tp2) = metrics::at_false_positive_count(&r, &f, fp);
            row.push_str(&format!(" {} |", pr(fp2, tp2, f.len())));
        }
        println!("{row}");
    }
    Ok(())
}

/// Markdown comparison: one table per dataset (plus the pooled set), one row
/// per system, with 95% bootstrap intervals for precision and recall.
pub fn compare_markdown(systems: &[(String, BTreeMap<String, Scored>, f32)], pooled_name: Option<&str>) {
    let common: Vec<&String> = systems[0].1.keys().filter(|k| systems.iter().all(|s| s.1.contains_key(*k))).collect();
    let mut datasets: Vec<&str> = common.iter().map(|k| systems[0].1[*k].dataset.as_str()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    let pooled = pooled_name.filter(|_| datasets.len() > 1);
    if let Some(p) = pooled {
        datasets.push(p);
    }
    for d in datasets {
        let pick = |scores: &'_ BTreeMap<String, Scored>| -> Vec<(f32, u8)> {
            common.iter().map(|k| &scores[*k]).filter(|r| Some(d) == pooled || r.dataset == d).map(|r| (r.prob, r.label)).collect()
        };
        let first = pick(&systems[0].1);
        let (nf, nr) = (first.iter().filter(|x| x.1 == 1).count(), first.iter().filter(|x| x.1 == 0).count());
        println!("\n**{d}** ({nf} AI, {nr} real)\n");
        println!("| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |");
        println!("|---|---|---|---|---|---|---|");
        for (name, scores, thr) in systems {
            let rows = pick(scores);
            let (p, l): (Vec<f32>, Vec<u8>) = rows.into_iter().unzip();
            let m = metrics::compute(&p, &l, *thr);
            let t = *thr;
            let (plo, phi) = metrics::bootstrap_ci(&p, &l, |s, l| metrics::compute(s, l, t).precision, 1000, 11);
            let (rlo, rhi) = metrics::bootstrap_ci(&p, &l, |s, l| metrics::compute(s, l, t).recall, 1000, 12);
            if nr == 0 {
                // No real items: only recall means anything.
                println!("| {name} | – | {:.3} ({:.3}–{:.3}) | – | – | – | – |", m.recall, rlo, rhi);
                continue;
            }
            println!(
                "| {name} | {:.3} ({:.3}–{:.3}) | {:.3} ({:.3}–{:.3}) | {:.3} | {:.2}% | {:.3} | {:.3} |",
                m.precision, plo, phi, m.recall, rlo, rhi, m.f1, 100.0 * m.fpr, m.auroc, metrics::recall_at_fpr(&p, &l, 0.01)
            );
        }
    }
    let _ = probs_labels;
}

/// What [`operating_points_markdown`] reports beyond the scores themselves.
#[derive(Clone, Debug)]
pub struct OperatingPoints {
    /// Empirical false-positive rates to operate at (cutoffs found on the
    /// items reported, so exploratory unless those are calibration items).
    pub fprs: Vec<f64>,
    /// The predeclared share of AI-generated items among real-world inputs,
    /// for the precision column.
    pub prevalence: f64,
    /// Columns to break the results down by: `dataset`, `generator`,
    /// `source`, `family`, `transform`.
    pub slice_by: Vec<String>,
    /// Resample whole groups of this kind in the bootstrap (`parent`,
    /// `scene`, `source`, `family`); `None` resamples items.
    pub cluster_by: Option<crate::split::Key>,
    /// Slices with fewer items of a class than this are listed but do not
    /// count towards the worst slice.
    pub min_slice: usize,
    /// Manifest rows by path, for the grouping columns score files lack.
    pub groups: BTreeMap<String, crate::ingest::Item>,
    /// Review bands to account for at each system's frozen cutoff; `None`
    /// prints nothing about review.
    pub review: Option<ReviewAccounting>,
    /// A false-positive budget: with it, every slice table is followed by
    /// the authentic strata's rates at the frozen cutoff judged against it
    /// (the transfer of a cutoff frozen on other items).
    pub transfer_budget: Option<f64>,
}

impl Default for OperatingPoints {
    fn default() -> Self {
        Self { fprs: vec![0.001, 0.01], prevalence: 0.01, slice_by: vec!["dataset".into()], cluster_by: None, min_slice: 100, groups: BTreeMap::new(), review: None, transfer_budget: None }
    }
}

/// The review bands [`operating_points_markdown`] accounts for.
#[derive(Clone, Debug, Default)]
pub struct ReviewAccounting {
    /// Margins to tabulate, one row each: the half-width of the band in
    /// calibrated logits around the frozen cutoff (`logit(score) −
    /// logit(cutoff)`), as `--review-margin` applies it.
    pub margins: Vec<f32>,
    /// `(share, threshold)`: also review when at least `share` of a row's
    /// logits (the views or frames looked at) fall on the other side of
    /// `threshold`, the profile's raw threshold on their mean (`detect
    /// --json` prints it as `threshold_logit`). Rows with fewer than two
    /// logits are never reviewed for this.
    pub disagreement: Option<(f32, f32)>,
    /// Review decisions recorded elsewhere, by path (`None`: not reviewed),
    /// tabulated as a row of their own: see [`read_review_flags`].
    pub flags: Option<BTreeMap<String, Option<Reason>>>,
}

/// Read per-item review decisions: either the JSON lines `detect --json` /
/// `batch --json` print with a review band (`path`, `review`,
/// `review_reason`), or tab-separated `path review [reason]` rows where
/// `review` is `1`/`true` or `0`/`false`. A reviewed row without a known
/// reason is counted as reviewed for the margin.
pub fn read_review_flags(path: &Path) -> Result<BTreeMap<String, Option<Reason>>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let reason = |reviewed: bool, why: Option<&str>| reviewed.then(|| if why == Some(Reason::ViewDisagreement.name()) { Reason::ViewDisagreement } else { Reason::Margin });
    let mut out = BTreeMap::new();
    for l in text.lines().filter(|l| !l.trim().is_empty()) {
        if l.trim_start().starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(l).with_context(|| format!("{}: not a JSON object: {l}", path.display()))?;
            // Rows without a verdict (unreadable files) carry no review field.
            let (Some(p), Some(r)) = (v.get("path").or_else(|| v.get("name")).and_then(|p| p.as_str()), v.get("review").and_then(|r| r.as_bool())) else { continue };
            out.insert(p.to_string(), reason(r, v.get("review_reason").and_then(|w| w.as_str())));
        } else {
            let f: Vec<&str> = l.split('\t').collect();
            let r = match f.get(1).copied() {
                Some("1" | "true") => true,
                Some("0" | "false") => false,
                _ => anyhow::bail!("{}: expected `path<TAB>0|1[<TAB>reason]`, got: {l}", path.display()),
            };
            out.insert(f[0].to_string(), reason(r, f.get(2).copied()));
        }
    }
    Ok(out)
}

/// What each item's verdict and review decision are under a band of
/// half-width `margin` (calibrated logits) around the cutoff `frozen`, with
/// the optional disagreement rule: the arithmetic of [`Review::reason`] on
/// the rows of a score file.
pub fn review_outcomes(rows: &[&Scored], frozen: f32, margin: f32, disagreement: Option<(f32, f32)>) -> Vec<Outcome> {
    let band = Review { margin, view_disagreement: disagreement.map(|d| d.0) };
    let at = review::logit_of(frozen);
    rows.iter().map(|r| Outcome { label: r.label, flagged: r.prob > frozen, review: band.reason(review::logit_of(r.prob) - at, &r.logits, disagreement.map_or(0.0, |d| d.1)) }).collect()
}

/// The value of slicing column `by` for a scored item.
pub fn slice_value(by: &str, it: &crate::ingest::Item) -> Result<String> {
    Ok(match by {
        "dataset" => it.dataset.clone(),
        "generator" => it.generator.clone(),
        "source" => crate::split::source_of(it).to_string(),
        "family" => crate::split::family_of(it).to_string(),
        "transform" => it.transform().unwrap_or("none").to_string(),
        other => anyhow::bail!("unknown slice `{other}` (use dataset, generator, source, family, transform)"),
    })
}

/// The fixed-budget tables of the frozen evaluation: for each system, on the
/// items every system scored,
///
/// - recall at each empirical false-positive rate in `opts.fprs`, and the
///   actual false-positive rate and recall at the system's own frozen
///   cutoff, each with the number of real items flagged, a one-sided 95%
///   Clopper–Pearson upper bound on the false-positive rate, a 95% bootstrap
///   interval for recall (clustered when `opts.cluster_by` is set) and the
///   precision at `opts.prevalence`;
/// - the same rates per slice at the frozen cutoff, and the worst slice.
///
/// The Clopper–Pearson bound counts items as independent trials; where
/// sources or parents repeat it is optimistic, and the clustered interval
/// is the one to read.
pub fn operating_points_markdown(systems: &[(String, BTreeMap<String, Scored>, f32)], opts: &OperatingPoints) -> Result<()> {
    let common: Vec<&String> = systems[0].1.keys().filter(|k| systems.iter().all(|s| s.1.contains_key(*k))).collect();
    // Manifest rows carry the grouping columns; a path without one gets what its score row says.
    let items: Vec<crate::ingest::Item> = common
        .iter()
        .map(|k| {
            let r = &systems[0].1[*k];
            let extra = opts.groups.get(*k).map(|it| it.extra.clone()).unwrap_or_default();
            crate::ingest::Item { path: (*k).into(), label: r.label, generator: r.generator.clone(), dataset: r.dataset.clone(), extra }
        })
        .collect();
    let labels: Vec<u8> = items.iter().map(|it| it.label).collect();
    let (n_real, n_fake) = (labels.iter().filter(|&&l| l == 0).count(), labels.iter().filter(|&&l| l == 1).count());
    let clusters: Option<Vec<usize>> = opts.cluster_by.map(|key| {
        // An item without the key is its own cluster.
        metrics::cluster_ids(&items.iter().map(|it| key.of(it).map_or_else(|| format!("path:{}", it.path.display()), |v| format!("{}:{v}", key.name()))).collect::<Vec<_>>())
    });
    let n_clusters = clusters.as_ref().map(|c| c.iter().max().map_or(0, |m| m + 1));
    let grouped = common.iter().filter(|k| opts.groups.contains_key(**k)).count();
    println!("{} items scored by all {} systems: {n_fake} AI, {n_real} real{}", common.len(), systems.len(), if opts.groups.is_empty() { String::new() } else { format!("; grouping columns found for {grouped}") });
    match (opts.cluster_by, &clusters) {
        (Some(key), Some(c)) => {
            let with_ai: std::collections::BTreeSet<usize> = c.iter().zip(&labels).filter(|(_, &l)| l == 1).map(|(&k, _)| k).collect();
            println!("Recall intervals: 95% bootstrap over {} {} clusters ({} of them hold AI items).", n_clusters.unwrap_or(0), key.name(), with_ai.len());
            if with_ai.len() < 20 {
                println!("Fewer than 20 clusters hold AI items: these intervals are too narrow to rely on. Cluster by a finer key, or read them as unknown.");
            }
        }
        _ => println!("Recall intervals: 95% bootstrap over items."),
    }
    println!("Precision is at {:.2}% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.", 100.0 * opts.prevalence);
    let pct = |x: f64| if x.is_finite() { format!("{:.2}%", 100.0 * x) } else { "–".to_string() };
    for (name, scores, frozen) in systems {
        let probs: Vec<f32> = common.iter().map(|k| scores[*k].prob).collect();
        println!("\n**{name}**\n");
        println!("| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |");
        println!("|---|---|---|---|---|---|---|");
        let mut points: Vec<(String, f32)> = Vec::new();
        for &f in &opts.fprs {
            if let Some(c) = metrics::empirical_cutoff(&probs, &labels, f) {
                points.push((format!("{}% empirical FPR", 100.0 * f), c));
            }
        }
        points.push(("frozen cutoff".into(), *frozen));
        for (what, cutoff) in &points {
            let (fpr, recall) = metrics::rates_at(&probs, &labels, *cutoff);
            let flagged = probs.iter().zip(&labels).filter(|(&p, &l)| l == 0 && p > *cutoff).count();
            let c = *cutoff;
            let (lo, hi) = metrics::bootstrap_ci_clustered(&probs, &labels, clusters.as_deref(), |s, l| metrics::rates_at(s, l, c).1, 1000, 12);
            let recall_cell = if n_fake > 0 { format!("{} ({}–{})", pct(recall), pct(lo), pct(hi)) } else { "–".into() };
            let (flag_cell, bound) = if n_real > 0 { (format!("{flagged} of {n_real}"), pct(metrics::clopper_pearson_upper(flagged, n_real, 0.95))) } else { ("–".into(), "–".into()) };
            println!("| {what} | {cutoff:.6} | {flag_cell} | {} | {bound} | {recall_cell} | {} |", pct(fpr), pct(metrics::precision_at_prevalence(recall, fpr, opts.prevalence)));
        }
        if let Some(acc) = &opts.review {
            let rows: Vec<&Scored> = common.iter().map(|k| &scores[*k]).collect();
            let mut bands: Vec<(String, Vec<Outcome>)> = vec![("none".into(), review_outcomes(&rows, *frozen, 0.0, None))];
            for &m in &acc.margins {
                let what = match acc.disagreement {
                    Some((share, _)) => format!("margin {m}, or disagreement of {share} or more"),
                    None => format!("margin {m}"),
                };
                bands.push((what, review_outcomes(&rows, *frozen, m, acc.disagreement)));
            }
            if let Some(flags) = &acc.flags {
                let missing = common.iter().filter(|k| !flags.contains_key(**k)).count();
                anyhow::ensure!(missing == 0, "{missing} of {} scored items have no row in the review flags file (paths must match the score file's)", common.len());
                bands.push(("recorded decisions".into(), common.iter().zip(&rows).map(|(k, r)| Outcome { label: r.label, flagged: r.prob > *frozen, review: flags[*k] }).collect()));
            }
            println!("\n{name}, review accounting at the frozen cutoff {frozen}. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.\n");
            println!("| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |");
            println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
            for (what, outcomes) in &bands {
                let a = review::account(outcomes);
                // Recall with reviewed items as misses, as a statistic of 0/1 scores, for the same bootstrap as above.
                let auto: Vec<f32> = outcomes.iter().map(|o| (o.flagged && o.review.is_none()) as u8 as f32).collect();
                let (lo, hi) = metrics::bootstrap_ci_clustered(&auto, &labels, clusters.as_deref(), |s, l| metrics::rates_at(s, l, 0.5).1, 1000, 12);
                let of = |k: usize, n: usize| if n > 0 { format!("{k} of {n} ({})", pct(k as f64 / n as f64)) } else { "–".into() };
                let (flag_cell, bound) = if a.n_real > 0 { (format!("{} of {}", a.real_flagged_unreviewed, a.n_real), pct(metrics::clopper_pearson_upper(a.real_flagged_unreviewed, a.n_real, 0.95))) } else { ("–".into(), "–".into()) };
                let recall_cell = if a.n_fake > 0 { format!("{} ({}–{})", pct(a.recall()), pct(lo), pct(hi)) } else { "–".into() };
                println!(
                    "| {what} | {} | {} | {} | {} of {} | {flag_cell} | {} | {bound} | {recall_cell} | {} | {} | {} |",
                    pct(a.coverage()),
                    of(a.real_reviewed, a.n_real),
                    of(a.fake_reviewed, a.n_fake),
                    a.by_margin,
                    a.real_reviewed + a.fake_reviewed,
                    pct(a.fpr()),
                    pct(a.recall_without_band()),
                    pct(metrics::precision_at_prevalence(a.recall(), a.fpr(), opts.prevalence)),
                    pct(a.review_rate_at(opts.prevalence))
                );
            }
        }
        for by in &opts.slice_by {
            let mut slices: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (i, it) in items.iter().enumerate() {
                slices.entry(slice_value(by, it)?).or_default().push(i);
            }
            println!("\n{name} by {by}, at the frozen cutoff {frozen}:\n");
            println!("| {by} | AI | Real | Recall | Real items flagged | FPR | AUROC |");
            println!("|---|---|---|---|---|---|---|");
            // (rate, slice) of the lowest recall and the highest false-positive rate among slices large enough.
            let (mut worst_recall, mut worst_fpr): (Option<(f64, &str)>, Option<(f64, &str)>) = (None, None);
            for (value, idx) in &slices {
                let (p, l): (Vec<f32>, Vec<u8>) = idx.iter().map(|&i| (probs[i], labels[i])).unzip();
                let (nr, nf) = (l.iter().filter(|&&x| x == 0).count(), l.iter().filter(|&&x| x == 1).count());
                let (fpr, recall) = metrics::rates_at(&p, &l, *frozen);
                let flagged = p.iter().zip(&l).filter(|(&p, &l)| l == 0 && p > *frozen).count();
                if nf >= opts.min_slice && worst_recall.is_none_or(|w| recall < w.0) {
                    worst_recall = Some((recall, value));
                }
                if nr >= opts.min_slice && worst_fpr.is_none_or(|w| fpr > w.0) {
                    worst_fpr = Some((fpr, value));
                }
                let auroc = metrics::auroc(&p, &l);
                println!("| {value} | {nf} | {nr} | {} | {} | {} | {} |", pct(recall), if nr > 0 { format!("{flagged} of {nr}") } else { "–".into() }, pct(fpr), if auroc.is_finite() { format!("{auroc:.3}") } else { "–".into() });
            }
            let show = |w: Option<(f64, &str)>| w.map_or_else(|| "none large enough".to_string(), |(r, v)| format!("{} ({v})", pct(r)));
            println!("\nWorst {by} slice with at least {} items of the class: recall {}; FPR {}.", opts.min_slice, show(worst_recall), show(worst_fpr));
            if let Some(budget) = opts.transfer_budget {
                let units = opts.cluster_by.map_or("items".to_string(), |k| format!("{} clusters", k.name()));
                println!("\n{name} transfer by {by}: authentic {units} flagged at the frozen cutoff {frozen}, against a false-positive budget of {}. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.\n", pct(budget));
                println!("| {by} | Authentic {units} | Flagged | FPR | Lower bound | Upper bound | Against the budget |");
                println!("|---|---|---|---|---|---|---|");
                let (mut within, mut exceeds, mut open) = (0, 0, 0);
                for (value, idx) in &slices {
                    let real: Vec<usize> = idx.iter().copied().filter(|&i| labels[i] == 0).collect();
                    if real.is_empty() {
                        continue;
                    }
                    let flags: Vec<bool> = real.iter().map(|&i| probs[i] > *frozen).collect();
                    let ids: Option<Vec<usize>> = clusters.as_ref().map(|c| real.iter().map(|&i| c[i]).collect());
                    let (k, n) = crate::calibration::flagged_units(&flags, ids.as_deref());
                    let s = crate::calibration::transfer(k, n, budget, 0.95);
                    let verdict = match s.status {
                        crate::calibration::Transfer::Within => {
                            within += 1;
                            "within".to_string()
                        }
                        crate::calibration::Transfer::Exceeds => {
                            exceeds += 1;
                            "exceeds".to_string()
                        }
                        crate::calibration::Transfer::Unresolved => {
                            open += 1;
                            let needed = crate::calibration::needed_with_no_flags(budget, 0.95);
                            if n < needed { format!("not resolved ({needed} needed with none flagged)") } else { "not resolved".to_string() }
                        }
                    };
                    println!("| {value} | {n} | {k} | {} | {} | {} | {verdict} |", pct(k as f64 / n as f64), pct(s.lower), pct(s.upper));
                }
                println!("\n{within} of {} authentic {by} strata are shown within the budget, {exceeds} exceed it, {open} are not resolved.", within + exceeds + open);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_use_grouping_columns_and_fall_back_to_the_score_row() {
        let plain = crate::ingest::Item { path: "a.png".into(), label: 1, generator: "flux".into(), dataset: "set".into(), ..Default::default() };
        let grouped = crate::ingest::Item { extra: vec!["p1".into(), "".into(), "reddit".into(), "diffusion".into(), "jpeg-q70".into()], ..plain.clone() };
        for (by, a, b) in [("dataset", "set", "set"), ("generator", "flux", "flux"), ("source", "set", "reddit"), ("family", "flux", "diffusion"), ("transform", "none", "jpeg-q70")] {
            assert_eq!((slice_value(by, &plain).unwrap().as_str(), slice_value(by, &grouped).unwrap().as_str()), (a, b), "{by}");
        }
        assert!(slice_value("prompt", &plain).is_err());
    }

    #[test]
    fn operating_points_run_on_one_class_and_on_both() {
        let row = |label: u8, prob: f32, gen: &str| Scored { label, generator: gen.into(), dataset: "d".into(), prob, logits: vec![] };
        let mut scores = BTreeMap::new();
        for i in 0..300 {
            scores.insert(format!("r{i}"), row(0, i as f32 / 400.0, "real:cam"));
            scores.insert(format!("f{i}"), row(1, 0.4 + i as f32 / 500.0, if i % 2 == 0 { "flux" } else { "sdxl" }));
        }
        let opts = OperatingPoints { slice_by: vec!["generator".into(), "transform".into()], cluster_by: Some(crate::split::Key::Source), ..Default::default() };
        operating_points_markdown(&[("a".into(), scores.clone(), 0.5)], &opts).unwrap();
        scores.retain(|_, r| r.label == 1);
        operating_points_markdown(&[("a".into(), scores, 0.5)], &OperatingPoints::default()).unwrap();
    }
}
