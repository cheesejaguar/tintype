//! Compare score files (as written by `tintype score` or the baseline
//! harness) with one shared metric implementation.
use crate::metrics::{self, Metrics};
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
