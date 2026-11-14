//! Re-check and refit an early-exit schedule offline, from score files that
//! hold every view logit of every image (`tintype score --all-views`) or
//! every frame logit of every clip (`tintype score-video --all-frames`).
//!
//! The exit margins are in raw logits around the profile's threshold, so
//! they belong to one threshold and one score scale: when calibration moves
//! either, or the views or their pooling change, the schedule has to be
//! checked again and, if it no longer holds, refitted. [`check`] says what a
//! schedule does at a threshold: which verdicts differ from scoring every
//! view, the false-positive rate and recall per source with and without
//! early exit (the calibration target is set with every view, and deployment
//! uses early exit), and how many views are looked at. [`fit_margins`] finds
//! tight margins that keep pooled precision and recall where scoring every
//! view puts them, within a tolerance. Fit on one set of files and report on
//! another: margins fitted and reported on the same items are exploratory.
//!
//! Everything here is arithmetic on logits already scored, and runs on any
//! platform. [`fit_margins`] is the function a changed pooling rule or view
//! order needs too: give it the rows and a template with the new order.
use crate::detector::{EarlyExit, ExitStage, PhotoSecondOpinion};
use crate::report::Scored;
use anyhow::{bail, ensure, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One scored item with every unit (view, or frame of a clip) of its profile.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// 1 = AI-generated.
    pub label: u8,
    /// Where it came from: the real source (`real:<name>`) of a real item,
    /// the dataset of a generated one.
    pub source: String,
    /// Logits in the profile's unit order.
    pub units: Vec<f32>,
}

/// Rows of the given score files. Rows whose logit count is not `units` are
/// left out and counted: a file scored with early exit holds only the views
/// that were looked at, and a short clip holds fewer frames than the
/// schedule covers.
pub fn read_rows(paths: &[PathBuf], units: usize) -> Result<(Vec<Row>, usize)> {
    let (mut rows, mut skipped) = (Vec::new(), 0);
    for p in paths {
        for (_, r) in crate::report::read_scores(p)? {
            if r.logits.len() == units {
                rows.push(row_of(r));
            } else {
                skipped += 1;
            }
        }
    }
    ensure!(!rows.is_empty(), "no row of {} holds {units} logits: score with --all-views (photos) or --all-frames (clips)", paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "));
    Ok((rows, skipped))
}

fn row_of(r: Scored) -> Row {
    Row { label: r.label, source: if r.label == 0 { r.generator } else { r.dataset }, units: r.logits }
}

/// Counts at one decision rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub n_real: usize,
    pub n_fake: usize,
    /// Real items flagged.
    pub fp: usize,
    /// AI items flagged.
    pub tp: usize,
}

fn ratio(a: usize, b: usize) -> f64 {
    if b > 0 {
        a as f64 / b as f64
    } else {
        f64::NAN
    }
}

impl Counts {
    pub fn precision(&self) -> f64 {
        ratio(self.tp, self.tp + self.fp)
    }
    pub fn recall(&self) -> f64 {
        ratio(self.tp, self.n_fake)
    }
    pub fn fpr(&self) -> f64 {
        ratio(self.fp, self.n_real)
    }
    fn add(&mut self, label: u8, flagged: bool) {
        if label == 1 {
            self.n_fake += 1;
            self.tp += flagged as usize;
        } else {
            self.n_real += 1;
            self.fp += flagged as usize;
        }
    }
}

/// What a schedule does at one threshold, against scoring every unit.
#[derive(Clone, Debug, PartialEq)]
pub struct Check {
    pub all: Counts,
    pub exit: Counts,
    /// Items flagged under early exit and not with every unit, and the reverse.
    pub flips_to_ai: usize,
    pub flips_to_real: usize,
    /// Units looked at per item under early exit.
    pub mean_units: f64,
    /// `stopped[k]`: items decided after `k + 1` units.
    pub stopped: Vec<usize>,
    /// Per source: counts with every unit, and under early exit.
    pub sources: BTreeMap<String, (Counts, Counts)>,
}

/// Run `exit` over `rows` at `threshold` (raw logit, on the mean) and
/// compare with scoring every unit. The same arithmetic as running it:
/// [`EarlyExit::looked_at`].
pub fn check(rows: &[Row], exit: &EarlyExit, threshold: f32) -> Check {
    let total = rows.first().map_or(0, |r| r.units.len());
    let mut c = Check { all: Counts::default(), exit: Counts::default(), flips_to_ai: 0, flips_to_real: 0, mean_units: 0.0, stopped: vec![0; total], sources: BTreeMap::new() };
    let mut looked = 0usize;
    for r in rows {
        let seen = exit.looked_at(&r.units, threshold);
        let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len() as f32;
        let (all, early) = (mean(&r.units) > threshold, mean(&seen) > threshold);
        c.all.add(r.label, all);
        c.exit.add(r.label, early);
        c.flips_to_ai += (early && !all) as usize;
        c.flips_to_real += (all && !early) as usize;
        looked += seen.len();
        c.stopped[seen.len() - 1] += 1;
        let s = c.sources.entry(r.source.clone()).or_default();
        s.0.add(r.label, all);
        s.1.add(r.label, early);
    }
    c.mean_units = ratio(looked, rows.len());
    c
}

/// The mean over real sources with at least `min` items of the per-source
/// false-positive rate: the quantity `tintype calibrate --rule balanced`
/// sets to its target, here `(with every unit, under early exit)`. NaN when
/// no source is large enough.
pub fn balanced_fpr(check: &Check, min: usize) -> (f64, f64) {
    let srcs: Vec<&(Counts, Counts)> = check.sources.values().filter(|(a, _)| a.n_fake == 0 && a.n_real >= min.max(1)).collect();
    let mean = |f: &dyn Fn(&(Counts, Counts)) -> f64| if srcs.is_empty() { f64::NAN } else { srcs.iter().map(|s| f(s)).sum::<f64>() / srcs.len() as f64 };
    (mean(&|s| s.0.fpr()), mean(&|s| s.1.fpr()))
}

fn pct(x: f64) -> String {
    if x.is_finite() {
        format!("{:.2}%", 100.0 * x)
    } else {
        "–".into()
    }
}

/// The printed form of a [`Check`]. With `target_fpr`, each real source's
/// rate under early exit is compared with the calibration target.
pub fn check_lines(c: &Check, unit: &str, target_fpr: Option<f64>, min_source: usize) -> Vec<String> {
    let n = c.all.n_real + c.all.n_fake;
    let mut out = vec![
        format!("{n} items ({} AI, {} real); {:.2} {unit}s looked at per item of {}", c.all.n_fake, c.all.n_real, c.mean_units, c.stopped.len()),
        format!("decided after 1..{} {unit}s: {}", c.stopped.len(), c.stopped.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(" / ")),
        format!("verdicts that differ from every {unit}: {} ({} more flagged, {} fewer)", c.flips_to_ai + c.flips_to_real, c.flips_to_ai, c.flips_to_real),
        String::new(),
        "| Pooled | Real items flagged | FPR | Recall | Precision |".into(),
        "|---|---|---|---|---|".into(),
    ];
    for (what, k) in [(format!("every {unit}"), &c.all), ("early exit".to_string(), &c.exit)] {
        out.push(format!("| {what} | {} of {} | {} | {} | {:.4} |", k.fp, k.n_real, pct(k.fpr()), pct(k.recall()), k.precision()));
    }
    out.push(format!("| difference | {:+} | {:+.2} points | {:+.2} points | {:+.4} |", c.exit.fp as i64 - c.all.fp as i64, 100.0 * (c.exit.fpr() - c.all.fpr()), 100.0 * (c.exit.recall() - c.all.recall()), c.exit.precision() - c.all.precision()));
    out.push(String::new());
    out.push(format!("| Real source | Items | FPR, every {unit} | FPR, early exit | Flagged, early exit | Against the target |"));
    out.push("|---|---|---|---|---|---|".into());
    for (name, (a, e)) in c.sources.iter().filter(|(_, (a, _))| a.n_real > 0) {
        let verdict = match target_fpr {
            None => "–".to_string(),
            Some(_) if a.n_real < min_source => format!("fewer than {min_source} items"),
            Some(t) if e.fpr() > t => "over".into(),
            Some(_) => "within".into(),
        };
        out.push(format!("| {name} | {} | {} | {} | {} of {} | {verdict} |", a.n_real, pct(a.fpr()), pct(e.fpr()), e.fp, e.n_real));
    }
    let (ball, bexit) = balanced_fpr(c, min_source);
    out.push(String::new());
    out.push(format!(
        "Mean over real sources with at least {min_source} items: {} with every {unit}, {} under early exit{}.",
        pct(ball),
        pct(bexit),
        target_fpr.map_or(String::new(), |t| format!("; the calibration target is {} ({})", pct(t), if bexit > t { "early exit is over it" } else if bexit.is_finite() { "early exit is within it" } else { "not comparable" }))
    ));
    out.push(String::new());
    out.push(format!("| Generated items from | Items | Recall, every {unit} | Recall, early exit |"));
    out.push("|---|---|---|---|".into());
    for (name, (a, e)) in c.sources.iter().filter(|(_, (a, _))| a.n_fake > 0) {
        out.push(format!("| {name} | {} | {} | {} |", a.n_fake, pct(a.recall()), pct(e.recall())));
    }
    out
}

/// How [`fit_margins`] searches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FitOptions {
    /// Pooled precision and pooled recall under early exit must each stay
    /// within this of their values with every unit (0.001 is the rule the
    /// shipped schedule was fitted to).
    pub tolerance: f64,
    /// Margins are multiples of this.
    pub step: f32,
    /// The widest finite margin tried.
    pub max_margin: f32,
    /// Fit `below` and `above` separately instead of keeping them equal.
    pub asymmetric: bool,
}

impl Default for FitOptions {
    fn default() -> Self {
        Self { tolerance: 0.001, step: 0.25, max_margin: 12.0, asymmetric: false }
    }
}

/// Per row, what the search needs: the label, the mean over the first
/// `views` units in looking order at each stage, and the mean over all.
struct Prefix {
    label: u8,
    stage_mean: Vec<f32>,
    full_mean: f32,
}

fn prefixes(rows: &[Row], template: &EarlyExit) -> Vec<Prefix> {
    rows.iter()
        .map(|r| {
            let stage_mean = template.stages.iter().map(|st| template.order[..st.views].iter().map(|&i| r.units[i]).sum::<f32>() / st.views as f32).collect();
            Prefix { label: r.label, stage_mean, full_mean: r.units.iter().sum::<f32>() / r.units.len() as f32 }
        })
        .collect()
}

/// Margins per stage as `(below, above)`; infinite means the stage never stops.
type Margins = Vec<(f32, f32)>;

/// The stage (index) at which a row stops under `margins`, or `None` for all units.
fn stops_at(p: &Prefix, margins: &Margins, threshold: f32) -> Option<usize> {
    p.stage_mean.iter().zip(margins).position(|(&m, &(below, above))| {
        let d = m - threshold;
        d >= above || d <= -below
    })
}

/// Counts under early exit and units looked at in total.
fn evaluate(pre: &[Prefix], views: &[usize], total: usize, margins: &Margins, threshold: f32) -> (Counts, usize) {
    let (mut c, mut looked) = (Counts::default(), 0usize);
    for p in pre {
        match stops_at(p, margins, threshold) {
            Some(k) => {
                c.add(p.label, p.stage_mean[k] > threshold);
                looked += views[k];
            }
            None => {
                c.add(p.label, p.full_mean > threshold);
                looked += total;
            }
        }
    }
    (c, looked)
}

/// Fit the margins of `template` (its `order` and stage view counts are
/// kept) on `rows`, for every threshold in `thresholds` at once: the three
/// six-view photo profiles share one schedule, each applying it around its
/// own threshold.
///
/// The search starts from "never stop early", which changes nothing, and
/// repeatedly tightens the one margin whose next step saves the most units
/// while pooled precision and pooled recall stay within `opts.tolerance` of
/// their every-unit values at every threshold. It stops when no margin can
/// be tightened. That is a greedy descent, not a global optimum: it gives
/// the same answer every time on the same rows, and it never returns a
/// schedule that breaks the tolerance on them.
///
/// A stage that never paid for itself is written with `opts.max_margin`,
/// and the fit fails if even that margin leaves the tolerance.
pub fn fit_margins(rows: &[Row], template: &EarlyExit, thresholds: &[f32], opts: &FitOptions) -> Result<EarlyExit> {
    let total = rows.first().map_or(0, |r| r.units.len());
    ensure!(!rows.is_empty() && !thresholds.is_empty(), "nothing to fit on");
    ensure!(rows.iter().all(|r| r.units.len() == total), "rows differ in how many logits they hold");
    ensure!(template.covers(total), "the schedule's order does not cover the {total} logits of each row");
    ensure!(opts.step > 0.0 && opts.max_margin >= opts.step && opts.tolerance >= 0.0, "the step and the widest margin must be positive, and the tolerance not negative");
    let pre = prefixes(rows, template);
    let views: Vec<usize> = template.stages.iter().map(|s| s.views).collect();
    // What every unit decides, per threshold.
    let base: Vec<Counts> = thresholds
        .iter()
        .map(|&t| {
            let mut c = Counts::default();
            pre.iter().for_each(|p| c.add(p.label, p.full_mean > t));
            c
        })
        .collect();
    let close = |a: f64, b: f64| (a.is_nan() && b.is_nan()) || (a - b).abs() <= opts.tolerance;
    let feasible = |m: &Margins| -> Option<usize> {
        let mut looked = 0;
        for (&t, b) in thresholds.iter().zip(&base) {
            let (c, l) = evaluate(&pre, &views, total, m, t);
            if !close(c.precision(), b.precision()) || !close(c.recall(), b.recall()) {
                return None;
            }
            looked += l;
        }
        Some(looked)
    };
    let grid = |x: f32| -> f32 { ((x / opts.step).floor() * opts.step).min(opts.max_margin) };
    let mut margins: Margins = vec![(f32::INFINITY, f32::INFINITY); views.len()];
    let mut looked = feasible(&margins).expect("never stopping early changes nothing");
    loop {
        // (units saved, stage, side, new margin): side 0 = both, 1 = below, 2 = above.
        let mut best: Option<(usize, usize, u8, f32)> = None;
        for k in 0..views.len() {
            for side in if opts.asymmetric { vec![1u8, 2] } else { vec![0u8] } {
                // The widest grid margin under the current one at which another row would stop
                // here: rows that reach this stage undecided, by how far they are from a threshold.
                let (below, above) = margins[k];
                let mut next = f32::NEG_INFINITY;
                for &t in thresholds {
                    for p in &pre {
                        if stops_at(p, &margins, t).is_some_and(|s| s <= k) {
                            continue;
                        }
                        let d = p.stage_mean[k] - t;
                        let reach = match side {
                            0 => d.abs(),
                            1 if d < 0.0 => -d,
                            2 if d >= 0.0 => d,
                            _ => continue,
                        };
                        let g = grid(reach);
                        let current = if side == 2 { above } else { below };
                        if g < current && g > next {
                            next = g;
                        }
                    }
                }
                if next < 0.0 {
                    continue;
                }
                let mut trial = margins.clone();
                match side {
                    0 => trial[k] = (next, next),
                    1 => trial[k].0 = next,
                    _ => trial[k].1 = next,
                }
                if let Some(l) = feasible(&trial) {
                    if l < looked && best.is_none_or(|b| looked - l > b.0) {
                        best = Some((looked - l, k, side, next));
                    }
                }
            }
        }
        let Some((saved, k, side, value)) = best else { break };
        match side {
            0 => margins[k] = (value, value),
            1 => margins[k].0 = value,
            _ => margins[k].1 = value,
        }
        looked -= saved;
    }
    // A stage that never paid for itself is written with the widest margin, which must itself hold.
    let finite = |x: f32| if x.is_finite() { x } else { opts.max_margin };
    let written: Margins = margins.iter().map(|&(b, a)| (finite(b), finite(a))).collect();
    ensure!(feasible(&written).is_some(), "a stage cannot stop at a margin of {} without leaving the tolerance: fit again with a wider --max-margin", opts.max_margin);
    let margins = written;
    Ok(EarlyExit { order: template.order.clone(), stages: views.iter().zip(&margins).map(|(&v, &(b, a))| ExitStage { views: v, below: b, above: a }).collect() })
}

/// Write the rows of `src` as a profile decides them: early exit under
/// `exit` (for rows that hold all `units` logits; others keep the logits
/// they have) and the calibration `p = sigmoid(scale · mean + bias)`. The
/// output has the layout of a score file (`path label generator dataset
/// prob logits...`, the logits being those looked at, in looking order), so
/// `tintype report` reads it. This is the arithmetic of scoring with the
/// profile, done offline: it lets a cutoff or a schedule fitted on one
/// all-views file be reported on another without scoring again. Returns the
/// number of rows written.
pub fn apply(src: &[PathBuf], dst: &Path, exit: Option<&EarlyExit>, units: usize, scale: f32, bias: f32) -> Result<usize> {
    use std::io::Write;
    let threshold = -bias / scale;
    let mut w = std::io::BufWriter::new(std::fs::File::create(dst)?);
    let mut n = 0;
    for p in src {
        let text = std::fs::read_to_string(p)?;
        for l in text.lines() {
            let c: Vec<&str> = l.split('\t').collect();
            if c.len() < 6 {
                continue;
            }
            let all: Vec<f32> = c[5..].iter().filter_map(|x| x.parse().ok()).collect();
            if all.is_empty() {
                continue;
            }
            let used = match exit {
                Some(exit) if all.len() == units => exit.looked_at(&all, threshold),
                _ => all,
            };
            let z = used.iter().sum::<f32>() / used.len() as f32;
            let prob = crate::metrics::sigmoid(scale * z + bias);
            let ls: Vec<String> = used.iter().map(|x| format!("{x:.5}")).collect();
            writeln!(w, "{}\t{}\t{}\t{}\t{prob:.6}\t{}", c[0], c[1], c[2], c[3], ls.join("\t"))?;
            n += 1;
        }
    }
    Ok(n)
}

/// The photo second-opinion band and rule expressed in a new calibration of
/// the first stage, so that exactly the same photos are routed and the
/// second stage decides them exactly as before.
///
/// The band is stored in calibrated logits (`scale · raw + bias`), so
/// recalibrating the default profile silently moves it. With
/// `z_old = a_o·r + b_o` and `z_new = a_n·r + b_n`, the old interval and the
/// old rule `weight·z_old + z2 > threshold` become, in `z_new`: the same raw
/// interval, `weight·a_o/a_n`, and `threshold − weight·(b_o − a_o·b_n/a_n)`.
///
/// This keeps the routing fixed while the first-stage threshold moves. It
/// is the neutral starting point, not a refit: whether the band should move
/// with the threshold is a routing decision that needs second-stage scores
/// on calibration data.
pub fn rebase_photo_band(band: &PhotoSecondOpinion, old: (f32, f32), new: (f32, f32)) -> PhotoSecondOpinion {
    let ((ao, bo), (an, bn)) = ((old.0 as f64, old.1 as f64), (new.0 as f64, new.1 as f64));
    let to_new = |z_old: f64| an * (z_old - bo) / ao + bn;
    let k = ao / an;
    PhotoSecondOpinion { lo: to_new(band.lo as f64) as f32, hi: to_new(band.hi as f64) as f32, weight: (band.weight as f64 * k) as f32, threshold: (band.threshold as f64 - band.weight as f64 * (bo - k * bn)) as f32 }
}

/// Refuse to write a spec that belongs to the released models: anything in a
/// directory called `models`. A refitted schedule is an experiment until the
/// frozen evaluation and a pre-registration say otherwise, and the released
/// specs are pinned by digest.
pub fn refuse_released(weights: &Path) -> Result<()> {
    let spec = crate::detector::ModelSpec::sidecar(weights);
    let abs = std::path::absolute(&spec)?;
    // Follow symbolic links where the file or its directory exists.
    let real = abs.canonicalize().ok().or_else(|| Some(abs.parent()?.canonicalize().ok()?.join(abs.file_name()?))).unwrap_or_else(|| abs.clone());
    for p in [&abs, &real] {
        if p.components().any(|c| c.as_os_str() == "models") {
            bail!("refusing to write {}: it is under a models/ directory, where the released specs live. Copy the weights and the spec to runs/<experiment>/ and pass that copy as --weights", spec.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(label: u8, source: &str, units: [f32; 6]) -> Row {
        Row { label, source: source.into(), units: units.to_vec() }
    }

    /// Deterministic rows: reals around -4, AI around +6, with per-view noise
    /// and a share of hard items near the threshold.
    fn synthetic(n: usize) -> Vec<Row> {
        let mut rng = crate::rng::Rng::new(5);
        let mut noise = |scale: f32| (rng.below(2001) as f32 / 1000.0 - 1.0) * scale;
        (0..n)
            .map(|i| {
                let label = (i % 2) as u8;
                let hard = i % 7 == 0;
                let centre = if hard { noise(2.5) + 1.0 } else if label == 1 { 6.0 + noise(3.0) } else { -4.0 + noise(3.0) };
                let mut units = [0f32; 6];
                units.iter_mut().for_each(|u| *u = centre + noise(if hard { 4.0 } else { 2.0 }));
                row(label, if label == 0 { if i % 4 == 0 { "real:a" } else { "real:b" } } else { "set" }, units)
            })
            .collect()
    }

    #[test]
    fn check_matches_the_detector_arithmetic_row_by_row() {
        let (e, thr) = (EarlyExit::standard(), 2.0);
        let rows = vec![
            // One look, far below.
            row(0, "real:a", [0.0, 0.0, -6.0, 0.0, 0.0, 0.0]),
            // First view far above, the rest low: early exit flags it, every view does not.
            row(0, "real:a", [0.0, 0.0, 11.5, 0.0, 0.0, 0.0]),
            // Near the threshold throughout: all six, mean 2.92 flagged either way.
            row(1, "set", [2.0, 3.0, 1.0, 2.5, 4.0, 5.0]),
            // First view far below, the rest high: early exit misses it.
            row(1, "set", [9.0, 9.0, -5.0, 9.0, 9.0, 9.0]),
        ];
        let c = check(&rows, &e, thr);
        assert_eq!(c.all, Counts { n_real: 2, n_fake: 2, fp: 0, tp: 2 });
        assert_eq!(c.exit, Counts { n_real: 2, n_fake: 2, fp: 1, tp: 1 });
        assert_eq!((c.flips_to_ai, c.flips_to_real), (1, 1));
        assert_eq!(c.stopped, vec![3, 0, 0, 0, 0, 1]);
        assert_eq!(c.mean_units, 9.0 / 4.0);
        assert_eq!(c.sources["real:a"].1.fp, 1);
        assert_eq!(c.sources["set"].1.tp, 1);
        // The same decisions as the detector's own stopping rule.
        for r in &rows {
            let seen = e.looked_at(&r.units, thr);
            assert!(seen.len() == 1 || seen.len() == 6);
        }
        let lines = check_lines(&c, "view", Some(0.015), 1).join("\n");
        for want in ["4 items (2 AI, 2 real); 2.25 views looked at per item of 6", "verdicts that differ from every view: 2 (1 more flagged, 1 fewer)", "| real:a | 2 | 0.00% | 50.00% | 1 of 2 | over |", "| set | 2 | 100.00% | 50.00% |", "early exit is over it"] {
            assert!(lines.contains(want), "missing `{want}` in:\n{lines}");
        }
    }

    #[test]
    fn balanced_rate_averages_real_sources_that_are_large_enough() {
        let mut rows = Vec::new();
        for i in 0..100 {
            // real:a: 10 of 100 flagged; real:b: 0 of 100; real:tiny: 1 of 1.
            rows.push(row(0, "real:a", [if i < 10 { 9.0 } else { -9.0 }; 6]));
            rows.push(row(0, "real:b", [-9.0; 6]));
        }
        rows.push(row(0, "real:tiny", [9.0; 6]));
        let c = check(&rows, &EarlyExit::standard(), 0.0);
        let (all, exit) = balanced_fpr(&c, 100);
        assert!((all - 0.05).abs() < 1e-12 && (exit - 0.05).abs() < 1e-12, "{all} {exit}");
        assert!(balanced_fpr(&c, 1000).0.is_nan());
    }

    #[test]
    fn fitted_margins_stay_within_the_tolerance_and_save_views() {
        let rows = synthetic(4000);
        let thresholds = [1.0f32, 2.5];
        let opts = FitOptions::default();
        let fitted = fit_margins(&rows, &EarlyExit::standard(), &thresholds, &opts).unwrap();
        assert_eq!(fitted.order, EarlyExit::standard().order);
        assert!(fitted.covers(6));
        assert!(fitted.stages.iter().all(|s| s.below == s.above && s.below >= 0.0 && s.below <= opts.max_margin && (s.below / opts.step).fract() == 0.0), "{fitted:?}");
        for &t in &thresholds {
            let c = check(&rows, &fitted, t);
            assert!((c.exit.precision() - c.all.precision()).abs() <= opts.tolerance + 1e-12, "{c:?}");
            assert!((c.exit.recall() - c.all.recall()).abs() <= opts.tolerance + 1e-12, "{c:?}");
            assert!(c.mean_units < 4.0, "early exit should save views here: {}", c.mean_units);
        }
        // The same rows give the same schedule.
        assert_eq!(fit_margins(&rows, &EarlyExit::standard(), &thresholds, &opts).unwrap(), fitted);
        // A zero tolerance still returns a schedule, one that changes no pooled rate at all.
        let exact = fit_margins(&rows, &EarlyExit::standard(), &thresholds, &FitOptions { tolerance: 0.0, ..opts }).unwrap();
        for &t in &thresholds {
            let c = check(&rows, &exact, t);
            assert_eq!((c.exit.precision(), c.exit.recall()), (c.all.precision(), c.all.recall()));
        }
        // A looser tolerance never looks at more views.
        let loose = fit_margins(&rows, &EarlyExit::standard(), &thresholds[..1], &FitOptions { tolerance: 0.01, ..opts }).unwrap();
        let tight = fit_margins(&rows, &EarlyExit::standard(), &thresholds[..1], &opts).unwrap();
        assert!(check(&rows, &loose, 1.0).mean_units <= check(&rows, &tight, 1.0).mean_units);
    }

    #[test]
    fn asymmetric_margins_are_fitted_per_side() {
        let rows = synthetic(3000);
        let opts = FitOptions { asymmetric: true, ..Default::default() };
        let fitted = fit_margins(&rows, &EarlyExit::standard(), &[1.5], &opts).unwrap();
        let c = check(&rows, &fitted, 1.5);
        assert!((c.exit.precision() - c.all.precision()).abs() <= opts.tolerance + 1e-12 && (c.exit.recall() - c.all.recall()).abs() <= opts.tolerance + 1e-12);
        let sym = fit_margins(&rows, &EarlyExit::standard(), &[1.5], &FitOptions::default()).unwrap();
        assert!(c.mean_units <= check(&rows, &sym, 1.5).mean_units + 1e-9);
    }

    #[test]
    fn fitting_rejects_rows_the_schedule_does_not_cover() {
        let rows = vec![Row { label: 0, source: "real:a".into(), units: vec![0.0; 4] }];
        assert!(fit_margins(&rows, &EarlyExit::standard(), &[0.0], &FitOptions::default()).is_err());
        assert!(fit_margins(&[], &EarlyExit::standard(), &[0.0], &FitOptions::default()).is_err());
        // The clip schedule fits sixteen frame logits.
        let clip = Row { label: 1, source: "set".into(), units: vec![5.0; 16] };
        let fitted = fit_margins(&[clip], &EarlyExit::video_standard(), &[1.6], &FitOptions::default()).unwrap();
        assert_eq!(fitted.stages.iter().map(|s| s.views).collect::<Vec<_>>(), vec![2, 4, 8]);
    }

    #[test]
    fn a_rebased_band_routes_and_decides_the_same_photos() {
        let band = PhotoSecondOpinion { lo: -7.0, hi: 0.5, weight: 0.5, threshold: 0.0 };
        let (old, new) = ((0.66400504f32, -1.1236415f32), (0.81f32, -2.4f32));
        let moved = rebase_photo_band(&band, old, new);
        for raw in [-12.0f32, -8.9, -8.8, -3.0, 0.0, 1.69, 2.4, 2.5, 6.0] {
            let (z_old, z_new) = (old.0 * raw + old.1, new.0 * raw + new.1);
            assert_eq!(band.wanted(z_old), moved.wanted(z_new), "raw {raw}");
            for z2 in [-3.0f32, -0.4, 0.3, 2.0] {
                assert!((band.probability(z_old, z2) - moved.probability(z_new, z2)).abs() < 1e-5, "raw {raw}, second {z2}");
            }
        }
        // The same calibration leaves the band where it was.
        let same = rebase_photo_band(&band, old, old);
        assert!((same.lo - band.lo).abs() < 1e-5 && (same.hi - band.hi).abs() < 1e-5 && (same.weight - band.weight).abs() < 1e-6 && same.threshold.abs() < 1e-6);
    }

    #[test]
    fn released_specs_are_never_written() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(refuse_released(&repo.join("models/tintype.safetensors")).is_err());
        assert!(refuse_released(std::path::Path::new("models/tintype-video.safetensors")).is_err());
        assert!(refuse_released(std::path::Path::new("/somewhere/else/models/copy.safetensors")).is_err());
        let e = refuse_released(std::path::Path::new("models/x.safetensors")).unwrap_err().to_string();
        assert!(e.contains("runs/<experiment>/"), "{e}");
        assert!(refuse_released(std::path::Path::new("runs/exp/candidate.safetensors")).is_ok());
        assert!(refuse_released(&std::env::temp_dir().join("tintype-refit/candidate.safetensors")).is_ok());
    }

    #[test]
    fn apply_writes_what_the_profile_decides() {
        let dir = std::env::temp_dir().join(format!("tintype-exit-apply-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (src, dst) = (dir.join("views.tsv"), dir.join("decided.tsv"));
        // Threshold 2 on the mean (scale 0.5, bias -1). The first row stops after one view, the second needs all six.
        std::fs::write(&src, "a.jpg\t0\treal:cam\tset\t0.5\t0\t0\t-6\t0\t0\t0\nb.jpg\t1\tgen\tset\t0.5\t2\t3\t1\t2.5\t4\t5\nshort.jpg\t1\tgen\tset\t0.5\t4\t6\n").unwrap();
        assert_eq!(apply(&[src.clone()], &dst, Some(&EarlyExit::standard()), 6, 0.5, -1.0).unwrap(), 3);
        let out = std::fs::read_to_string(&dst).unwrap();
        let rows: Vec<Vec<&str>> = out.lines().map(|l| l.split('\t').collect()).collect();
        // One view looked at: sigmoid(0.5 * -6 - 1).
        assert_eq!(rows[0], vec!["a.jpg", "0", "real:cam", "set", "0.017986", "-6.00000"]);
        // All six, in looking order 2, 0, 1, 3, 4, 5: mean 2.91667.
        assert_eq!(rows[1][4], format!("{:.6}", crate::metrics::sigmoid(0.5 * 17.5 / 6.0 - 1.0)));
        assert_eq!(rows[1][5..], ["1.00000", "2.00000", "3.00000", "2.50000", "4.00000", "5.00000"]);
        // A row scored with fewer views keeps them.
        assert_eq!(rows[2][4..], ["0.817574", "4.00000", "6.00000"]);
        // Without a schedule every logit is used.
        apply(&[src], &dst, None, 6, 0.5, -1.0).unwrap();
        assert!(std::fs::read_to_string(&dst).unwrap().starts_with("a.jpg\t0\treal:cam\tset\t0.182426\t0.00000\t"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rows_without_every_logit_are_left_out_and_counted() {
        let dir = std::env::temp_dir().join(format!("tintype-exit-fit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("scores.tsv");
        std::fs::write(&p, "a.jpg\t0\treal:cam\tset\t0.1\t-1\t-2\t-3\t-4\t-5\t-6\nb.jpg\t1\tgen\tset\t0.9\t5\t6\nc.jpg\t1\tgen\tset\t0.9\t1\t2\t3\t4\t5\t6\n").unwrap();
        let (rows, skipped) = read_rows(&[p.clone()], 6).unwrap();
        assert_eq!((rows.len(), skipped), (2, 1));
        // A real item's source is its real source; a generated item's is its dataset.
        assert_eq!((rows[0].source.as_str(), rows[1].source.as_str()), ("real:cam", "set"));
        assert!(read_rows(&[p], 16).unwrap_err().to_string().contains("--all-views"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
