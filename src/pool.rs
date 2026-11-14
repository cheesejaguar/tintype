//! Replay of view pooling from saved per-view logits (`tintype replay-pool`).
//!
//! The detector scores up to six views of a photo and averages their logits.
//! This module asks, without running the network again, what another way of
//! combining the same logits would decide: a weighted mean, a trimmed mean,
//! the median, the mean of the top k, the maximum, and another order of
//! looking at the views. It needs score files that hold every view's logit
//! of every image, in the spec's view order (`score --all-views`, the
//! `*-views.tsv` files of `evaluate`, `calib_images.tsv` of `finalize`).
//!
//! A pool is never compared at the shipped threshold or with the shipped
//! early-exit margins: both were fitted to the mean. For every candidate the
//! threshold is refitted on separate fit items by the calibration rule
//! `calibrate` uses ([`fit_threshold`]), and the early-exit margins are
//! refitted by the criterion the shipped schedule was fitted with
//! ([`refit_margins`]). Only then is the candidate reported on the test
//! items. It runs on every platform: nothing here touches the GPU engine.
//!
//! # What "the pool of the views looked at so far" means
//!
//! Early exit looks at the views in a fixed order and stops when the pool of
//! the views seen so far is far enough from the threshold; the verdict then
//! comes from those views alone ([`Pool::value`]):
//!
//! - **mean**: the mean of the views seen (today's rule).
//! - **weighted mean**: the weighted mean of the views seen, with their
//!   weights renormalised to add up to 1. Undefined, so no exit, while every
//!   view seen has weight 0.
//! - **median**: the median of the views seen (the mean of the two middle
//!   ones when their number is even).
//! - **trimmed mean**: the mean after dropping the lowest and the highest
//!   view. Undefined, so no exit, before three views are seen.
//! - **top k** (and the maximum, k = 1): the mean of the k highest views
//!   seen. Undefined, so no exit, before k views are seen. From then on it
//!   can only rise as views are added, so stopping *above* the threshold
//!   never disagrees with all views; stopping below it is a bet, as every
//!   exit of the mean is.
use crate::detector::{EarlyExit, ExitStage};
use crate::metrics;
use anyhow::{bail, ensure, Context, Result};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// How the logits of an image's views become one score.
#[derive(Clone, Debug, PartialEq)]
pub enum Pool {
    Mean,
    /// One non-negative weight per view, in the spec's view order, adding up to 1.
    Weighted(Vec<f32>),
    /// Mean without the lowest and the highest view.
    Trimmed,
    Median,
    /// Mean of the k highest views; `TopK(1)` is the maximum.
    TopK(usize),
}

impl Pool {
    /// `mean`, `trimmed`, `median`, `max`, `top2`, `top3`, ..., or
    /// `weights:` followed by one weight per view separated by `/`
    /// (normalised to add up to 1).
    pub fn parse(s: &str, views: usize) -> Result<Self> {
        Ok(match s {
            "mean" => Pool::Mean,
            "trimmed" => Pool::Trimmed,
            "median" => Pool::Median,
            "max" => Pool::TopK(1),
            _ => {
                if let Some(k) = s.strip_prefix("top") {
                    let k: usize = k.parse().map_err(|_| anyhow::anyhow!("use topK, e.g. top2"))?;
                    ensure!(k >= 1 && k <= views, "top{k} needs between 1 and {views} views");
                    Pool::TopK(k)
                } else if let Some(w) = s.strip_prefix("weights:") {
                    let w: Vec<f32> = w.split('/').map(|x| x.trim().parse::<f32>()).collect::<Result<_, _>>().map_err(|_| anyhow::anyhow!("use weights:w0/w1/..., one number per view"))?;
                    ensure!(w.len() == views, "weights: needs {views} weights, one per view (got {})", w.len());
                    let total: f32 = w.iter().sum();
                    ensure!(w.iter().all(|x| x.is_finite() && *x >= 0.0) && total > 0.0, "weights must be non-negative and not all zero");
                    Pool::Weighted(w.iter().map(|x| x / total).collect())
                } else {
                    bail!("unknown pool `{s}` (use mean, trimmed, median, max, topK, fitted or weights:w0/w1/...)")
                }
            }
        })
    }

    pub fn name(&self) -> String {
        match self {
            Pool::Mean => "mean".into(),
            Pool::Weighted(w) => format!("weights {}", w.iter().map(|x| format!("{x:.3}")).collect::<Vec<_>>().join("/")),
            Pool::Trimmed => "trimmed mean".into(),
            Pool::Median => "median".into(),
            Pool::TopK(1) => "max".into(),
            Pool::TopK(k) => format!("top {k}"),
        }
    }

    /// The pooled logit of the views looked at so far: `seen[i]` is the
    /// logit of view `views[i]` (an index into the spec's view list), in the
    /// order they were looked at. NaN where the pool is not defined yet (see
    /// the module documentation); with every view seen it is always defined.
    ///
    /// The mean adds the logits in looking order and divides once, the same
    /// arithmetic as [`EarlyExit::decided`] and as scoring, so that a replay
    /// of the mean reproduces the detector's own decisions exactly.
    pub fn value(&self, views: &[usize], seen: &[f32]) -> f32 {
        let n = seen.len();
        let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len() as f32;
        let sorted = || {
            let mut s = seen.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            s
        };
        match self {
            Pool::Mean => mean(seen),
            Pool::Weighted(w) => {
                let total: f32 = views.iter().map(|&v| w[v]).sum();
                if total > 0.0 {
                    views.iter().zip(seen).map(|(&v, &z)| w[v] * z).sum::<f32>() / total
                } else {
                    f32::NAN
                }
            }
            Pool::Trimmed if n >= 3 => mean(&sorted()[1..n - 1]),
            Pool::Trimmed => f32::NAN,
            Pool::Median => {
                let s = sorted();
                if n % 2 == 1 {
                    s[n / 2]
                } else {
                    (s[n / 2 - 1] + s[n / 2]) / 2.0
                }
            }
            Pool::TopK(k) if n >= *k => mean(&sorted()[n - k..]),
            Pool::TopK(_) => f32::NAN,
        }
    }

    /// The pooled logit of all of an image's views (`all` in the spec's view order).
    pub fn of_all(&self, all: &[f32]) -> f32 {
        let views: Vec<usize> = (0..all.len()).collect();
        self.value(&views, all)
    }

    /// The pooled logit after each number of views, looking in `order`:
    /// element `k - 1` is the pool of the first `k` views looked at.
    pub fn partials(&self, order: &[usize], all: &[f32]) -> Vec<f32> {
        let seen: Vec<f32> = order.iter().map(|&v| all[v]).collect();
        (1..=order.len()).map(|k| self.value(&order[..k], &seen[..k])).collect()
    }
}

/// How many views early exit looks at, given the pooled logit after each
/// number of views ([`Pool::partials`]) and the threshold on it: the same
/// rule as [`EarlyExit::looked_at`], for any pool. An undefined (NaN) pooled
/// logit never stops.
pub fn views_looked_at(exit: &EarlyExit, partials: &[f32], threshold: f32) -> usize {
    for st in exit.stages.iter().filter(|st| st.views < partials.len()) {
        let margin = partials[st.views - 1] - threshold;
        if margin >= st.above || margin <= -st.below {
            return st.views;
        }
    }
    partials.len()
}

/// One image of an all-views score file.
#[derive(Clone, Debug)]
pub struct Item {
    pub path: String,
    pub label: u8,
    /// The generator column: the generator of an AI image, `real:<source>` of a real one.
    pub generator: String,
    pub dataset: String,
    /// Every view's raw logit, in the spec's view order.
    pub logits: Vec<f32>,
}

/// Read all-views score files. Every row must hold exactly `views` logits:
/// a file written with early exit (fewer logits on clear images, and in
/// looking order) or one that holds probabilities only cannot be replayed.
pub fn read_items(files: &[PathBuf], views: usize) -> Result<Vec<Item>> {
    let mut out = Vec::new();
    for f in files {
        let rows = crate::report::read_scores(f).with_context(|| format!("{} is not a score file (rows of `path label generator dataset prob logits...`, no header)", f.display()))?;
        for (path, r) in rows {
            ensure!(
                r.logits.len() == views,
                "{}: {path} has {} view logits, not {views}. A replay needs every view of every image: files from `score --all-views`, `evaluate` (`*-views.tsv`) or `finalize` (`calib_images.tsv`); the archives in docs/results hold probabilities only",
                f.display(),
                r.logits.len()
            );
            out.push(Item { path, label: r.label, generator: r.generator, dataset: r.dataset, logits: r.logits });
        }
    }
    Ok(out)
}

/// The files a `--fit` or `--test` argument names: the file itself, or,
/// when its file name holds one `*`, every file of that directory whose name
/// starts and ends as given, in name order.
pub fn expand(pattern: &str) -> Result<Vec<PathBuf>> {
    let path = Path::new(pattern);
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let Some((start, end)) = name.split_once('*') else { return Ok(vec![path.to_path_buf()]) };
    ensure!(!end.contains('*'), "{pattern}: only one * is understood, in the file name");
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .with_context(|| format!("listing {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().is_some_and(|n| n.len() >= start.len() + end.len() && n.starts_with(start) && n.ends_with(end)))
        .map(|e| e.path())
        .collect();
    out.sort();
    ensure!(!out.is_empty(), "no file matches {pattern}");
    Ok(out)
}

/// How the decision threshold is derived from real items (as `calibrate --rule`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rule {
    /// Pooled over all real items.
    Pooled,
    /// Mean per-source rate over real sources with at least this many items.
    Balanced(usize),
}

/// A pool's calibration: flag above `threshold` (a pooled raw logit), and
/// report `p = sigmoid(scale * (z - threshold))`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Calibration {
    pub threshold: f32,
    pub scale: f32,
    /// Real sources that counted towards a balanced threshold.
    pub sources: usize,
}

/// Fit a pool's threshold and slope on its pooled logits `values` of
/// `items`, by the rule `tintype calibrate` applies to the mean: the logit
/// that `target_fpr` of the real items exceed, or, balanced, the logit at
/// which the false-positive rate averaged over the real sources with enough
/// items equals the target (a bisection between -30 and 30); then the slope
/// of a class-balanced logistic fit around that threshold.
///
/// This mirrors the arithmetic of `eval::calibrate` (which is compiled on
/// macOS only, with the GPU engine) step for step; a test on macOS holds the
/// two together.
pub fn fit_threshold(values: &[f32], items: &[Item], target_fpr: f64, rule: Rule) -> Result<Calibration> {
    let (mut real, mut fake) = (Vec::new(), Vec::new());
    let mut by_source: BTreeMap<&str, Vec<f32>> = Default::default();
    for (&z, it) in values.iter().zip(items) {
        if it.label == 1 {
            fake.push(z)
        } else {
            real.push(z);
            by_source.entry(it.generator.as_str()).or_default().push(z);
        }
    }
    ensure!(real.len() >= 200, "need at least 200 real items to calibrate (have {})", real.len());
    let mut thr = metrics::fpr_threshold(&real, target_fpr);
    let mut sources = 0;
    if let Rule::Balanced(min) = rule {
        let srcs: Vec<&Vec<f32>> = by_source.values().filter(|z| z.len() >= min).collect();
        ensure!(!srcs.is_empty(), "no real source has {min} items");
        sources = srcs.len();
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
    }
    let (scale, _, threshold) = metrics::calibrate_at(&real, &fake, thr);
    Ok(Calibration { threshold, scale, sources })
}

/// A margin nothing reaches: one direction of a stage that never exits
/// (finite, so that the schedule still reads and writes as JSON).
pub const NEVER: f32 = 1.0e9;

/// What [`refit_margins`] searches over and holds to.
#[derive(Clone, Debug)]
pub struct Refit {
    /// How far pooled precision and recall may move from deciding with all views.
    pub tolerance: f64,
    /// The margins a stage may take, in increasing order.
    pub grid: Vec<f32>,
    /// One margin per stage for both directions, as the shipped schedule
    /// has; `false` fits `below` and `above` separately.
    pub symmetric: bool,
}

impl Default for Refit {
    /// Within 0.001, margins from 0 to 12 in steps of 0.25, symmetric.
    fn default() -> Self {
        Self { tolerance: 0.001, grid: (0..=48).map(|i| i as f32 * 0.25).collect(), symmetric: true }
    }
}

/// Counts of one way of deciding a set of items.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Tally {
    tp: usize,
    fp: usize,
    fake: usize,
    views: usize,
}

impl Tally {
    fn precision(&self) -> f64 {
        self.tp as f64 / (self.tp + self.fp) as f64
    }
    fn recall(&self) -> f64 {
        self.tp as f64 / self.fake as f64
    }
    /// Whether precision and recall are both within `tolerance` of `other`'s
    /// (a rate that is undefined in both counts as equal).
    fn within(&self, other: &Tally, tolerance: f64) -> bool {
        let close = |a: f64, b: f64| (a.is_nan() && b.is_nan()) || (a - b).abs() <= tolerance;
        close(self.precision(), other.precision()) && close(self.recall(), other.recall())
    }
}

/// Decide every item with per-stage margins: `stages[n - 1]` is the
/// `(below, above)` pair after `n` views, `None` for no exit there.
/// `margins` holds, item after item, the pooled logit minus the threshold
/// after 1..=`total` views.
fn tally(margins: &[f32], total: usize, labels: &[u8], stages: &[Option<(f32, f32)>]) -> Tally {
    let mut t = Tally::default();
    for (m, &label) in margins.chunks_exact(total).zip(labels) {
        let mut stop = total;
        for (n, st) in stages.iter().enumerate() {
            if let Some((below, above)) = st {
                if m[n] >= *above || m[n] <= -*below {
                    stop = n + 1;
                    break;
                }
            }
        }
        let flagged = m[stop - 1] > 0.0;
        t.views += stop;
        t.fake += (label == 1) as usize;
        t.tp += (flagged && label == 1) as usize;
        t.fp += (flagged && label == 0) as usize;
    }
    t
}

/// Refit the early-exit margins of a pool: the tightest margins, for looking
/// at the views in `order`, that keep pooled precision and recall on the
/// given items within `opts.tolerance` of deciding from all views at
/// `threshold`. This is the criterion the shipped schedule was fitted with
/// (docs/BENCHMARK.md, "Five speed changes"), made into a procedure:
///
/// 1. Start with no early exit at all (which satisfies the criterion).
/// 2. A step tightens one stage: from where it is (at first "no exit") down
///    the grid to the first margin that lets at least one more item out
///    early. Margins that would change nothing are passed over, so a stage
///    is never tighter than the items made it.
/// 3. Among the steps that keep the criterion, take the one that saves the
///    most views over the items (ties go to the earliest stage); repeat
///    until no step keeps it.
///
/// `logits[i]` are all of item `i`'s view logits in the spec's view order
/// and `labels[i]` its class. The result has a stage after every number of
/// views short of all at which an exit survived; a stage where the pool is
/// not defined yet never exits and is left out. The margins are in pooled
/// logits, like the threshold. With `opts.symmetric` unset, `below` and
/// `above` are stepped separately, and a direction with no exit gets the
/// margin [`NEVER`].
///
/// The search is greedy, so "tightest" is local: no single further step is
/// possible, though another path could end elsewhere. The margins hold on
/// the items given and nowhere else by construction: check them on separate
/// data, as the replay does.
pub fn refit_margins(pool: &Pool, order: &[usize], logits: &[Vec<f32>], labels: &[u8], threshold: f32, opts: &Refit) -> EarlyExit {
    let total = order.len();
    let margins: Vec<f32> = logits.iter().flat_map(|all| pool.partials(order, all).into_iter().map(|z| z - threshold)).collect();
    // Per stage, the index into the grid of (below, above); None is no exit.
    let mut state: Vec<(Option<usize>, Option<usize>)> = vec![(None, None); total.saturating_sub(1)];
    let as_stages = |state: &[(Option<usize>, Option<usize>)]| -> Vec<Option<(f32, f32)>> {
        let m = |i: Option<usize>| i.map_or(NEVER, |i| opts.grid[i]);
        state.iter().map(|&(b, a)| (b.is_some() || a.is_some()).then(|| (m(b), m(a)))).collect()
    };
    let reference = tally(&margins, total, labels, &vec![None; state.len()]);
    let tighter = |i: Option<usize>| match i {
        None => opts.grid.len().checked_sub(1),
        Some(0) => None,
        Some(i) => Some(i - 1),
    };
    let moves: &[(bool, bool)] = if opts.symmetric { &[(true, true)] } else { &[(true, false), (false, true)] };
    let mut views = reference.views;
    loop {
        let mut best: Option<(usize, Vec<(Option<usize>, Option<usize>)>)> = None;
        for n in 0..state.len() {
            for &(below, above) in moves {
                // Walk this stage down the grid to the first margin that lets any item out earlier.
                let mut trial = state.clone();
                let saved = loop {
                    let (b, a) = trial[n];
                    let (nb, na) = (if below { tighter(b) } else { b }, if above { tighter(a) } else { a });
                    if (below && nb.is_none()) || (above && na.is_none()) {
                        break None;
                    }
                    trial[n] = (nb, na);
                    let t = tally(&margins, total, labels, &as_stages(&trial));
                    if t.views < views {
                        break Some(t);
                    }
                };
                if let Some(t) = saved {
                    if t.within(&reference, opts.tolerance) && best.as_ref().is_none_or(|(v, _)| t.views < *v) {
                        best = Some((t.views, trial));
                    }
                }
            }
        }
        match best {
            Some((v, next)) => {
                views = v;
                state = next;
            }
            None => break,
        }
    }
    let stages = as_stages(&state).into_iter().enumerate().filter_map(|(n, st)| st.map(|(below, above)| ExitStage { views: n + 1, below, above })).collect();
    EarlyExit { order: order.to_vec(), stages }
}

/// An order of looking at the views chosen from the data: at each step the
/// view whose addition makes the pool of the views so far agree with the
/// all-views verdict (at `threshold`) on the most items; ties go to the
/// lower view index.
pub fn greedy_order(pool: &Pool, logits: &[Vec<f32>], threshold: f32) -> Vec<usize> {
    let total = logits.first().map_or(0, |l| l.len());
    let verdicts: Vec<bool> = logits.iter().map(|all| pool.of_all(all) > threshold).collect();
    let mut order: Vec<usize> = Vec::new();
    while order.len() < total {
        let mut best = (0usize, usize::MAX);
        for v in (0..total).filter(|v| !order.contains(v)) {
            let mut trial = order.clone();
            trial.push(v);
            let agree = logits.iter().zip(&verdicts).filter(|(all, &want)| (pool.value(&trial, &trial.iter().map(|&i| all[i]).collect::<Vec<f32>>()) > threshold) == want).count();
            if best.1 == usize::MAX || agree > best.0 {
                best = (agree, v);
            }
        }
        order.push(best.1);
    }
    order
}

/// Non-negative per-view weights, adding up to 1, from a class-balanced
/// logistic regression of the label on the view logits (with an intercept).
/// A view whose coefficient comes out negative is dropped (weight 0) and the
/// rest refitted, until none is negative. Only the direction is kept: the
/// scale and the intercept are refitted with the threshold afterwards.
pub fn fit_weights(logits: &[Vec<f32>], labels: &[u8]) -> Result<Vec<f32>> {
    let total = logits.first().map_or(0, |l| l.len());
    let (n_fake, n_real) = (labels.iter().filter(|&&l| l == 1).count(), labels.iter().filter(|&&l| l == 0).count());
    ensure!(total > 0 && n_fake > 0 && n_real > 0, "fitting view weights needs items of both classes");
    let (w_fake, w_real) = (0.5 / n_fake as f64, 0.5 / n_real as f64);
    let mut active: Vec<usize> = (0..total).collect();
    loop {
        ensure!(!active.is_empty(), "no view has a positive weight");
        let d = active.len() + 1; // coefficients, then the intercept
        let mut beta = vec![0f64; d];
        let loss_grad = |beta: &[f64]| -> (f64, Vec<f64>, Vec<f64>) {
            let (mut loss, mut g, mut h) = (0f64, vec![0f64; d], vec![0f64; d * d]);
            let mut x = vec![1f64; d];
            for (all, &label) in logits.iter().zip(labels) {
                for (k, &v) in active.iter().enumerate() {
                    x[k] = all[v] as f64;
                }
                let z: f64 = x.iter().zip(beta).map(|(a, b)| a * b).sum();
                let (y, w) = if label == 1 { (1.0, w_fake) } else { (0.0, w_real) };
                let p = 1.0 / (1.0 + (-z).exp());
                // log(1 + e^z) - y z, computed without overflow.
                loss += w * (z.max(0.0) + (-z.abs()).exp().ln_1p() - y * z);
                for i in 0..d {
                    g[i] += w * (p - y) * x[i];
                    for j in 0..d {
                        h[i * d + j] += w * p * (1.0 - p) * x[i] * x[j];
                    }
                }
            }
            // A little ridge on the coefficients keeps a separable fit finite.
            for i in 0..d - 1 {
                loss += 0.5e-6 * beta[i] * beta[i];
                g[i] += 1e-6 * beta[i];
                h[i * d + i] += 1e-6;
            }
            (loss, g, h)
        };
        let (mut loss, mut g, mut h) = loss_grad(&beta);
        for _ in 0..100 {
            let Some(step) = solve(&h, &g, d) else { break };
            let mut scale = 1.0;
            let mut moved = false;
            for _ in 0..30 {
                let trial: Vec<f64> = beta.iter().zip(&step).map(|(b, s)| b - scale * s).collect();
                let (l2, g2, h2) = loss_grad(&trial);
                if l2 <= loss {
                    moved = (loss - l2) > 1e-12;
                    (beta, loss, g, h) = (trial, l2, g2, h2);
                    break;
                }
                scale /= 2.0;
            }
            if !moved {
                break;
            }
        }
        let worst = (0..active.len()).min_by(|&a, &b| beta[a].total_cmp(&beta[b])).unwrap();
        if beta[worst] < 0.0 {
            active.remove(worst);
            continue;
        }
        let sum: f64 = beta[..d - 1].iter().sum();
        ensure!(sum > 0.0, "no view has a positive weight");
        let mut w = vec![0f32; total];
        for (k, &v) in active.iter().enumerate() {
            w[v] = (beta[k] / sum) as f32;
        }
        return Ok(w);
    }
}

/// Solve `a x = b` for a small dense system (Gaussian elimination with
/// partial pivoting); `None` when it is singular.
fn solve(a: &[f64], b: &[f64], d: usize) -> Option<Vec<f64>> {
    let (mut a, mut b) = (a.to_vec(), b.to_vec());
    for c in 0..d {
        let p = (c..d).max_by(|&i, &j| a[i * d + c].abs().total_cmp(&a[j * d + c].abs()))?;
        if a[p * d + c].abs() < 1e-300 {
            return None;
        }
        if p != c {
            for k in 0..d {
                a.swap(c * d + k, p * d + k);
            }
            b.swap(c, p);
        }
        for r in c + 1..d {
            let f = a[r * d + c] / a[c * d + c];
            for k in c..d {
                a[r * d + k] -= f * a[c * d + k];
            }
            b[r] -= f * b[c];
        }
    }
    let mut x = vec![0f64; d];
    for r in (0..d).rev() {
        let s: f64 = (r + 1..d).map(|k| a[r * d + k] * x[k]).sum();
        x[r] = (b[r] - s) / a[r * d + r];
    }
    Some(x)
}

/// A pool with everything that was fitted for it on the fit items.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub pool: Pool,
    pub calibration: Calibration,
    /// The order of looking and the margins; no stages means every view is looked at.
    pub exit: EarlyExit,
    /// How this row is named in the tables.
    pub label: String,
}

/// What a candidate decides for one set of items.
#[derive(Clone, Debug)]
pub struct Decisions {
    /// Pooled logit of all views.
    pub all: Vec<f32>,
    /// Pooled logit of the views early exit looked at.
    pub exit: Vec<f32>,
    /// How many views early exit looked at.
    pub views: Vec<usize>,
}

impl Candidate {
    pub fn decide(&self, items: &[Item]) -> Decisions {
        let (mut all, mut exit, mut views) = (Vec::with_capacity(items.len()), Vec::with_capacity(items.len()), Vec::with_capacity(items.len()));
        for it in items {
            let partials = self.pool.partials(&self.exit.order, &it.logits);
            let n = views_looked_at(&self.exit, &partials, self.calibration.threshold);
            all.push(self.pool.of_all(&it.logits));
            exit.push(partials[n - 1]);
            views.push(n);
        }
        Decisions { all, exit, views }
    }

    /// The probability the calibration gives a pooled logit.
    pub fn probability(&self, z: f32) -> f32 {
        metrics::sigmoid(self.calibration.scale * (z - self.calibration.threshold))
    }
}

/// How one way of deciding differs from another on the same items:
/// `(AI gained, AI lost, real newly flagged, real cleared)`.
pub fn flips(labels: &[u8], new: &[bool], old: &[bool]) -> (usize, usize, usize, usize) {
    let mut f = (0, 0, 0, 0);
    for ((&l, &n), &o) in labels.iter().zip(new).zip(old) {
        match (l == 1, n, o) {
            (true, true, false) => f.0 += 1,
            (true, false, true) => f.1 += 1,
            (false, true, false) => f.2 += 1,
            (false, false, true) => f.3 += 1,
            _ => {}
        }
    }
    f
}

fn order_name(order: &[usize]) -> String {
    order.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
}

fn margins_cell(exit: &EarlyExit, total: usize) -> String {
    (1..total)
        .map(|n| match exit.stages.iter().find(|s| s.views == n) {
            None => "–".to_string(),
            Some(s) if s.below == s.above => format!("{}", s.below),
            Some(s) => format!("{}↓ {}↑", if s.below < NEVER { s.below.to_string() } else { "–".into() }, if s.above < NEVER { s.above.to_string() } else { "–".into() }),
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn pct(x: f64) -> String {
    if x.is_finite() {
        format!("{:.2}%", 100.0 * x)
    } else {
        "–".to_string()
    }
}

/// What `tintype replay-pool` was asked to do.
pub struct Args {
    /// All-views score files the thresholds, margins and weights are fitted on.
    pub fit: Vec<String>,
    /// `name=file[,file...]`: all-views score files to report on (several per name are pooled).
    pub test: Vec<String>,
    /// The spec the files were scored with: its threshold and early exit are
    /// "the shipped pool" every candidate is compared with. Only read.
    pub spec: Option<PathBuf>,
    pub pools: Vec<String>,
    /// `shipped`, `greedy`, or view indices such as `0,2,1,3,4,5`.
    pub orders: Vec<String>,
    pub target_fpr: f64,
    pub rule: Rule,
    pub refit: Refit,
    /// `dataset` or `generator`: also break the early-exit results down.
    pub slice_by: Option<String>,
    /// Directory for one score file per test set and candidate.
    pub write_scores: Option<PathBuf>,
}

fn files_of(patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for p in patterns {
        out.extend(expand(p)?);
    }
    Ok(out)
}

fn view_count(file: &Path) -> Result<usize> {
    let text = std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let first = text.lines().next().with_context(|| format!("{} is empty", file.display()))?;
    Ok(first.split('\t').count().saturating_sub(5))
}

pub fn run(args: Args, out: &mut impl Write) -> Result<()> {
    ensure!((0.0..1.0).contains(&args.target_fpr), "--target-fpr is a fraction, e.g. 0.015");
    ensure!(!args.refit.grid.is_empty() && args.refit.tolerance >= 0.0, "the margin grid is empty or the tolerance negative");
    let fit_files = files_of(&args.fit)?;
    let spec: Option<crate::detector::ModelSpec> = match &args.spec {
        Some(p) => Some(serde_json::from_str(&std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?)?),
        None => None,
    };
    let total = match &spec {
        Some(s) => s.views.len(),
        None => view_count(&fit_files[0])?,
    };
    ensure!(total >= 2, "{}: a replay needs at least two view logits per image (found {total})", fit_files[0].display());
    let mut fit = read_items(&fit_files, total)?;
    let mut tests: Vec<(String, Vec<Item>)> = Vec::new();
    for t in &args.test {
        let (name, patterns) = t.split_once('=').ok_or_else(|| anyhow::anyhow!("use --test name=scores.tsv"))?;
        let items = read_items(&files_of(&patterns.split(',').map(String::from).collect::<Vec<_>>())?, total)?;
        match tests.iter_mut().find(|(n, _)| n == name) {
            Some((_, v)) => v.extend(items),
            None => tests.push((name.to_string(), items)),
        }
    }
    // Fitting and reporting stay apart: a file that is in both is reported on, not fitted on.
    let before = fit.len();
    {
        let tested: std::collections::HashSet<&str> = tests.iter().flat_map(|(_, items)| items.iter().map(|it| it.path.as_str())).collect();
        fit.retain(|it| !tested.contains(it.path.as_str()));
    }
    let left_out = before - fit.len();
    let fit_logits: Vec<Vec<f32>> = fit.iter().map(|it| it.logits.clone()).collect();
    let fit_labels: Vec<u8> = fit.iter().map(|it| it.label).collect();

    // The shipped pool: the mean, with the spec's threshold and schedule when a spec is given.
    let standard = EarlyExit::standard();
    let shipped_exit = match &spec {
        Some(s) => s.early_exit.clone().filter(|e| e.covers(total)).unwrap_or(EarlyExit { order: (0..total).collect(), stages: vec![] }),
        None if standard.covers(total) => standard,
        None => EarlyExit { order: (0..total).collect(), stages: vec![] },
    };
    let mean_all: Vec<f32> = fit.iter().map(|it| Pool::Mean.of_all(&it.logits)).collect();
    let mean_fit = fit_threshold(&mean_all, &fit, args.target_fpr, args.rule)?;
    let shipped = match &spec {
        Some(s) => Candidate { pool: Pool::Mean, calibration: Calibration { threshold: -s.logit_bias / s.logit_scale, scale: s.logit_scale, sources: 0 }, exit: shipped_exit.clone(), label: "shipped (mean, spec as is)".into() },
        None => Candidate { pool: Pool::Mean, calibration: mean_fit, exit: shipped_exit.clone(), label: "baseline (mean, standard schedule)".into() },
    };

    // Orders to try; `greedy` depends on the pool, so it is resolved per pool.
    let mut orders: Vec<Option<Vec<usize>>> = Vec::new();
    for o in &args.orders {
        orders.push(match o.as_str() {
            "shipped" => Some(shipped_exit.order.clone()),
            "greedy" => None,
            list => {
                let v: Vec<usize> = list.split(',').map(|x| x.trim().parse::<usize>()).collect::<Result<_, _>>().map_err(|_| anyhow::anyhow!("--order takes shipped, greedy or view indices such as 0,2,1,3,4,5"))?;
                let mut sorted = v.clone();
                sorted.sort_unstable();
                ensure!(sorted.into_iter().eq(0..total), "--order {list} must name each of the {total} views once");
                Some(v)
            }
        });
    }

    let n_real = fit_labels.iter().filter(|&&l| l == 0).count();
    writeln!(out, "Fit items: {} ({} AI, {} real) from {} file(s). Target false-positive rate {}, {}.", fit.len(), fit.len() - n_real, n_real, fit_files.len(), pct(args.target_fpr), match args.rule {
        Rule::Pooled => "pooled over real items".to_string(),
        Rule::Balanced(min) => format!("averaged over the {} real sources with at least {min} items", mean_fit.sources),
    })?;
    if left_out > 0 {
        writeln!(out, "{left_out} items of the fit files are also test items (same path) and were left out of the fit. Paths are all this checks: copies and derivatives of a fit item under another path are not noticed.")?;
    }
    writeln!(out, "Margins: refitted per pool and order on the fit items, the tightest on a grid of {} to {} that keep pooled precision and recall within {} of all views{}.", args.refit.grid[0], args.refit.grid[args.refit.grid.len() - 1], args.refit.tolerance, if args.refit.symmetric { "" } else { "; below (↓) and above (↑) fitted separately" })?;
    writeln!(out, "Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.")?;

    let mut candidates: Vec<Candidate> = Vec::new();
    writeln!(out, "\n### Fitted on the fit items\n")?;
    writeln!(out, "| Pool | Order | Raw threshold | Slope | Margins after 1 to {} views | Views per image | Precision / recall, all views | Precision / recall, early exit |", total - 1)?;
    writeln!(out, "|---|---|---|---|---|---|---|---|")?;
    {
        // The shipped row first, for reference: nothing about it is refitted.
        let d = shipped.decide(&fit);
        let thr = shipped.calibration.threshold;
        let rates = |z: &[f32]| -> (f64, f64) {
            let (tp, fp) = z.iter().zip(&fit_labels).fold((0usize, 0usize), |(tp, fp), (&z, &l)| (tp + (z > thr && l == 1) as usize, fp + (z > thr && l == 0) as usize));
            (tp as f64 / (tp + fp) as f64, tp as f64 / (fit.len() - n_real) as f64)
        };
        let ((pa, ra), (pe, re)) = (rates(&d.all), rates(&d.exit));
        writeln!(out, "| {} | {} | {thr:.3} | {:.3} | {} | {:.2} | {pa:.4} / {ra:.4} | {pe:.4} / {re:.4} |", shipped.label, order_name(&shipped.exit.order), shipped.calibration.scale, margins_cell(&shipped.exit, total), d.views.iter().sum::<usize>() as f64 / fit.len() as f64)?;
    }
    for name in &args.pools {
        let pool = if name == "fitted" { Pool::Weighted(fit_weights(&fit_logits, &fit_labels)?) } else { Pool::parse(name, total)? };
        let all: Vec<f32> = fit.iter().map(|it| pool.of_all(&it.logits)).collect();
        let calibration = fit_threshold(&all, &fit, args.target_fpr, args.rule)?;
        let mut seen_orders: Vec<Vec<usize>> = Vec::new();
        for o in &orders {
            let (order, how) = match o {
                Some(v) => (v.clone(), ""),
                None => (greedy_order(&pool, &fit_logits, calibration.threshold), " (greedy)"),
            };
            if seen_orders.contains(&order) {
                continue;
            }
            seen_orders.push(order.clone());
            let exit = refit_margins(&pool, &order, &fit_logits, &fit_labels, calibration.threshold, &args.refit);
            let label = if name == "fitted" { format!("fitted {}", pool.name()) } else { pool.name() };
            let c = Candidate { pool: pool.clone(), calibration, exit, label };
            let d = c.decide(&fit);
            let thr = calibration.threshold;
            let rates = |z: &[f32]| -> (f64, f64) {
                let (tp, fp) = z.iter().zip(&fit_labels).fold((0usize, 0usize), |(tp, fp), (&z, &l)| (tp + (z > thr && l == 1) as usize, fp + (z > thr && l == 0) as usize));
                (tp as f64 / (tp + fp) as f64, tp as f64 / (fit.len() - n_real) as f64)
            };
            let ((pa, ra), (pe, re)) = (rates(&d.all), rates(&d.exit));
            writeln!(out, "| {} | {}{how} | {thr:.3} | {:.3} | {} | {:.2} | {pa:.4} / {ra:.4} | {pe:.4} / {re:.4} |", c.label, order_name(&order), calibration.scale, margins_cell(&c.exit, total), d.views.iter().sum::<usize>() as f64 / fit.len() as f64)?;
            candidates.push(c);
        }
    }
    for c in &candidates {
        writeln!(out, "\n`early_exit` for {}, order {} (pooled-logit threshold {:.4}): `{}`", c.label, order_name(&c.exit.order), c.calibration.threshold, serde_json::to_string(&c.exit)?)?;
    }

    if let Some(dir) = &args.write_scores {
        std::fs::create_dir_all(dir)?;
    }
    for (name, items) in &tests {
        let labels: Vec<u8> = items.iter().map(|it| it.label).collect();
        let (n_fake, n_real) = (labels.iter().filter(|&&l| l == 1).count(), labels.iter().filter(|&&l| l == 0).count());
        writeln!(out, "\n### Test: {name} ({} items: {n_fake} AI, {n_real} real)", items.len())?;
        let base = shipped.decide(items);
        let base_flags: Vec<bool> = base.exit.iter().map(|&z| z > shipped.calibration.threshold).collect();
        let at = |z: &[f32], thr: f32| -> (String, String, String) {
            let (fpr, recall) = metrics::rates_at(z, &labels, thr);
            let flagged = z.iter().zip(&labels).filter(|(&z, &l)| l == 0 && z > thr).count();
            let f = flips(&labels, &z.iter().map(|&z| z > thr).collect::<Vec<_>>(), &base_flags);
            (if n_real > 0 { format!("{flagged} of {n_real} ({})", pct(fpr)) } else { "–".into() }, pct(recall), format!("+{} −{} AI, +{} −{} real", f.0, f.1, f.2, f.3))
        };
        let r_at = |z: &[f32], fpr: f64| pct(metrics::recall_at_fpr(z, &labels, fpr));

        writeln!(out, "\nAll {total} views of every image (the order does not matter). Flips are against the first row of the next table.\n")?;
        writeln!(out, "| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |")?;
        writeln!(out, "|---|---|---|---|---|---|")?;
        let mut shown: Vec<&Pool> = Vec::new();
        for c in std::iter::once(&shipped).chain(&candidates) {
            if shown.contains(&&c.pool) && !std::ptr::eq(c, &shipped) {
                continue;
            }
            if !std::ptr::eq(c, &shipped) {
                shown.push(&c.pool);
            }
            let d = c.decide(items);
            let (flagged, recall, changed) = at(&d.all, c.calibration.threshold);
            writeln!(out, "| {} | {} | {} | {flagged} | {recall} | {changed} |", if std::ptr::eq(c, &shipped) { format!("{}, all views", c.label) } else { c.label.clone() }, r_at(&d.all, 0.01), r_at(&d.all, 0.001))?;
        }

        writeln!(out, "\nWith early exit (what the detector would run). Flips are against the first row.\n")?;
        writeln!(out, "| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |")?;
        writeln!(out, "|---|---|---|---|---|---|---|---|")?;
        let mut sliced: Vec<(String, Vec<bool>, Vec<usize>)> = Vec::new();
        for c in std::iter::once(&shipped).chain(&candidates) {
            let d = c.decide(items);
            let (flagged, recall, changed) = at(&d.exit, c.calibration.threshold);
            let row = format!("{} | {}", c.label, order_name(&c.exit.order));
            writeln!(out, "| {row} | {:.2} | {} | {} | {flagged} | {recall} | {changed} |", d.views.iter().sum::<usize>() as f64 / items.len().max(1) as f64, r_at(&d.exit, 0.01), r_at(&d.exit, 0.001))?;
            sliced.push((row, d.exit.iter().map(|&z| z > c.calibration.threshold).collect(), d.views.clone()));
            if let Some(dir) = &args.write_scores {
                let file = dir.join(format!("{name}.{}.{}.tsv", c.label.split([' ', '(']).next().unwrap_or("pool"), order_name(&c.exit.order).replace(',', "")));
                let mut w = std::io::BufWriter::new(std::fs::File::create(&file).with_context(|| format!("writing {}", file.display()))?);
                for ((it, &z), &n) in items.iter().zip(&d.exit).zip(&d.views) {
                    let looked: Vec<String> = c.exit.order[..n].iter().map(|&v| format!("{:.5}", it.logits[v])).collect();
                    writeln!(w, "{}\t{}\t{}\t{}\t{:.6}\t{}", it.path, it.label, it.generator, it.dataset, c.probability(z), looked.join("\t"))?;
                }
            }
        }

        if let Some(by) = &args.slice_by {
            let key = |it: &Item| match by.as_str() {
                "generator" => it.generator.clone(),
                _ => it.dataset.clone(),
            };
            let mut slices: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            for (i, it) in items.iter().enumerate() {
                slices.entry(key(it)).or_default().push(i);
            }
            for (row, flags, views) in &sliced {
                writeln!(out, "\n{name} by {by}: {row}, with early exit at the fitted cutoff. Changes are against the shipped row.\n")?;
                writeln!(out, "| {by} | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |")?;
                writeln!(out, "|---|---|---|---|---|---|---|---|")?;
                for (value, idx) in &slices {
                    let count = |f: &[bool], class: u8| idx.iter().filter(|&&i| labels[i] == class && f[i]).count();
                    let (nf, nr) = (idx.iter().filter(|&&i| labels[i] == 1).count(), idx.iter().filter(|&&i| labels[i] == 0).count());
                    let (tp, fp) = (count(flags, 1), count(flags, 0));
                    let cell = |k: usize, n: usize| if n > 0 { format!("{k} ({})", pct(k as f64 / n as f64)) } else { "–".into() };
                    let delta = |new: usize, old: usize, n: usize| if n > 0 { format!("{:+}", new as i64 - old as i64) } else { "–".into() };
                    writeln!(out, "| {value} | {nf} | {nr} | {} | {} | {} | {} | {:.2} |", cell(tp, nf), delta(tp, count(&base_flags, 1), nf), cell(fp, nr), delta(fp, count(&base_flags, 0), nr), idx.iter().map(|&i| views[i]).sum::<usize>() as f64 / idx.len() as f64)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    /// Six view logits around a per-image level: real images near -4, AI
    /// images near +4, with view-to-view scatter and some hard images near 1.
    fn synthetic(n: usize, seed: u64) -> (Vec<Vec<f32>>, Vec<u8>) {
        let mut rng = Rng::new(seed);
        let mut unit = move || rng.below(10_000) as f32 / 10_000.0;
        let (mut logits, mut labels) = (Vec::new(), Vec::new());
        for i in 0..n {
            let label = (i % 2) as u8;
            let hard = i % 7 == 0;
            let level = if hard { 0.5 + 2.0 * (unit() - 0.5) } else if label == 1 { 4.0 + 3.0 * unit() } else { -4.0 - 3.0 * unit() };
            logits.push((0..6).map(|v| level + (if v == 0 { 5.0 } else { 2.0 }) * (unit() - 0.5)).collect());
            labels.push(label);
        }
        (logits, labels)
    }

    fn items(logits: &[Vec<f32>], labels: &[u8]) -> Vec<Item> {
        logits.iter().zip(labels).enumerate().map(|(i, (l, &label))| Item { path: format!("{i:05}.png"), label, generator: if label == 1 { "gen".into() } else { format!("real:src{}", i % 3) }, dataset: if i % 4 == 0 { "a".into() } else { "b".into() }, logits: l.clone() }).collect()
    }

    #[test]
    fn pools_of_all_views_and_of_the_views_seen_so_far() {
        let all = [1.0f32, 5.0, -3.0, 2.0, 0.0, 7.0];
        assert_eq!(Pool::Mean.of_all(&all), 2.0);
        assert_eq!(Pool::Median.of_all(&all), 1.5);
        assert_eq!(Pool::Trimmed.of_all(&all), 2.0); // without -3 and 7
        assert_eq!(Pool::TopK(1).of_all(&all), 7.0);
        assert_eq!(Pool::TopK(2).of_all(&all), 6.0);
        assert_eq!(Pool::Weighted(vec![0.5, 0.0, 0.5, 0.0, 0.0, 0.0]).of_all(&all), -1.0);
        // Looking at views 2, 0, 1, ...: logits -3, 1, 5, 2, 0, 7.
        let order = [2usize, 0, 1, 3, 4, 5];
        assert_eq!(Pool::Mean.partials(&order, &all), vec![-3.0, -1.0, 1.0, 1.25, 1.0, 2.0]);
        assert_eq!(Pool::Median.partials(&order, &all), vec![-3.0, -1.0, 1.0, 1.5, 1.0, 1.5]);
        // The trimmed mean is not defined before three views, top 2 before two.
        let t = Pool::Trimmed.partials(&order, &all);
        assert!(t[0].is_nan() && t[1].is_nan());
        assert_eq!(t[2..], [1.0, 1.5, 1.0, 2.0]);
        let top2 = Pool::TopK(2).partials(&order, &all);
        assert!(top2[0].is_nan());
        assert_eq!(top2[1..], [-1.0, 3.0, 3.5, 3.5, 6.0]);
        // Once defined, top k never falls as views are added.
        assert!(top2[1..].windows(2).all(|w| w[1] >= w[0]));
        assert_eq!(Pool::TopK(1).partials(&order, &all), vec![-3.0, 1.0, 5.0, 5.0, 5.0, 7.0]);
        // Weights are renormalised over the views seen; all-zero so far is undefined.
        let w = Pool::Weighted(vec![0.5, 0.0, 0.0, 0.5, 0.0, 0.0]).partials(&order, &all);
        assert!(w[0].is_nan());
        assert_eq!(w[1..], [1.0, 1.0, 1.5, 1.5, 1.5]);
    }

    #[test]
    fn pool_names_parse() {
        assert_eq!(Pool::parse("max", 6).unwrap(), Pool::TopK(1));
        assert_eq!(Pool::parse("top3", 6).unwrap(), Pool::TopK(3));
        assert_eq!(Pool::parse("weights:1/1/2/0/0/0", 6).unwrap(), Pool::Weighted(vec![0.25, 0.25, 0.5, 0.0, 0.0, 0.0]));
        assert!(Pool::parse("top7", 6).is_err() && Pool::parse("weights:1/1", 6).is_err() && Pool::parse("weights:0/0/0/0/0/0", 6).is_err() && Pool::parse("mode", 6).is_err());
        assert_eq!(Pool::parse("trimmed", 6).unwrap().name(), "trimmed mean");
    }

    #[test]
    fn the_mean_replays_the_detectors_early_exit_exactly() {
        let (logits, _) = synthetic(3000, 3);
        let exit = EarlyExit::standard();
        for thr in [-0.776f32, 1.692, 3.397] {
            for all in &logits {
                let seen = exit.looked_at(all, thr);
                let partials = Pool::Mean.partials(&exit.order, all);
                let n = views_looked_at(&exit, &partials, thr);
                assert_eq!(n, seen.len());
                // The same sum in the same order: equal to the last bit.
                assert_eq!(partials[n - 1].to_bits(), (seen.iter().sum::<f32>() / seen.len() as f32).to_bits());
            }
        }
        // No stages: every view, whatever the pool.
        let none = EarlyExit { order: (0..6).collect(), stages: vec![] };
        assert_eq!(views_looked_at(&none, &Pool::Mean.partials(&none.order, &logits[0]), 0.0), 6);
    }

    #[test]
    fn refitted_margins_keep_precision_and_recall_and_save_views() {
        let (logits, labels) = synthetic(4000, 11);
        let order = vec![2usize, 0, 1, 3, 4, 5];
        let thr = 1.0f32;
        for (pool, symmetric) in [(Pool::Mean, true), (Pool::Median, true), (Pool::Trimmed, true), (Pool::TopK(2), true), (Pool::TopK(1), false), (Pool::Weighted(vec![0.0, 0.2, 0.4, 0.2, 0.1, 0.1]), true)] {
            let opts = Refit { symmetric, ..Default::default() };
            let exit = refit_margins(&pool, &order, &logits, &labels, thr, &opts);
            assert!(exit.order == order && exit.stages.windows(2).all(|w| w[0].views < w[1].views) && exit.stages.iter().all(|s| s.views < 6), "{exit:?}");
            // Decide with all views and with the refitted schedule.
            let (mut reference, mut refit) = (Tally::default(), Tally::default());
            for (all, &label) in logits.iter().zip(&labels) {
                let partials = pool.partials(&order, all);
                let n = views_looked_at(&exit, &partials, thr);
                for (t, z, views) in [(&mut reference, partials[5], 6), (&mut refit, partials[n - 1], n)] {
                    t.fake += (label == 1) as usize;
                    t.tp += (z > thr && label == 1) as usize;
                    t.fp += (z > thr && label == 0) as usize;
                    t.views += views;
                }
            }
            assert!(refit.within(&reference, 0.001), "{pool:?}: {refit:?} vs {reference:?}");
            assert!(refit.views < reference.views * 2 / 3, "{pool:?} saved too little: {} of {} views, {exit:?}", refit.views, reference.views);
            // The pool decides nothing where it is not defined.
            match pool {
                Pool::Trimmed => assert!(exit.stages.iter().all(|s| s.views >= 3), "{exit:?}"),
                Pool::TopK(k) => assert!(exit.stages.iter().all(|s| s.views >= k), "{exit:?}"),
                Pool::Weighted(_) => assert!(exit.stages.iter().all(|s| s.views >= 1), "{exit:?}"),
                _ => {}
            }
            // Tightest: one more grid step at any stage breaks the criterion, or lets nothing more out.
            if symmetric {
                for i in 0..exit.stages.len() {
                    let mut tighter = exit.clone();
                    let m = tighter.stages[i].below - 0.25;
                    if m < 0.0 {
                        continue;
                    }
                    tighter.stages[i] = ExitStage { views: tighter.stages[i].views, below: m, above: m };
                    let mut t = Tally::default();
                    for (all, &label) in logits.iter().zip(&labels) {
                        let partials = pool.partials(&order, all);
                        let n = views_looked_at(&tighter, &partials, thr);
                        t.fake += (label == 1) as usize;
                        t.tp += (partials[n - 1] > thr && label == 1) as usize;
                        t.fp += (partials[n - 1] > thr && label == 0) as usize;
                        t.views += n;
                    }
                    assert!(!t.within(&reference, 0.001) || t.views == refit.views, "{pool:?}: stage {i} of {exit:?} could be tightened");
                }
            }
        }
    }

    #[test]
    fn a_zero_tolerance_refit_changes_no_verdict_and_the_maximum_stops_above_for_free() {
        let (logits, labels) = synthetic(2000, 5);
        let order: Vec<usize> = (0..6).collect();
        let exit = refit_margins(&Pool::TopK(1), &order, &logits, &labels, 1.0, &Refit { tolerance: 0.0, symmetric: false, ..Default::default() });
        // The maximum can only rise: stopping as soon as it is over the threshold is exact,
        // so the margin above goes all the way down while the one below cannot.
        assert!(exit.stages[0].views == 1 && exit.stages[0].above == 0.0 && exit.stages.iter().all(|s| s.above < s.below), "{exit:?}");
        assert!(serde_json::from_str::<EarlyExit>(&serde_json::to_string(&exit).unwrap()).unwrap() == exit);
        for all in &logits {
            let partials = Pool::TopK(1).partials(&order, all);
            let n = views_looked_at(&exit, &partials, 1.0);
            assert_eq!(partials[n - 1] > 1.0, partials[5] > 1.0);
        }
        // Nothing to gain, nothing fitted: with no items there is no exit.
        assert!(refit_margins(&Pool::Mean, &order, &[], &[], 0.0, &Refit::default()).stages.is_empty());
    }

    #[test]
    fn threshold_follows_the_pooled_and_the_balanced_rule() {
        // Two real sources: 900 items spread over 0..0.9, and 100 items over 0..10.
        let mut its = Vec::new();
        for i in 0..900 {
            its.push(Item { path: format!("a{i}"), label: 0, generator: "real:big".into(), dataset: "d".into(), logits: vec![i as f32 / 1000.0; 6] });
        }
        for i in 0..100 {
            its.push(Item { path: format!("b{i}"), label: 0, generator: "real:small".into(), dataset: "d".into(), logits: vec![i as f32 / 10.0; 6] });
        }
        for i in 0..500 {
            its.push(Item { path: format!("f{i}"), label: 1, generator: "gen".into(), dataset: "d".into(), logits: vec![5.0 + i as f32 / 50.0; 6] });
        }
        let values: Vec<f32> = its.iter().map(|it| Pool::Mean.of_all(&it.logits)).collect();
        let reals: Vec<f32> = values[..1000].to_vec();
        // Pooled at 5%: 50 of the 1,000 reals above, all of them from the small source.
        let pooled = fit_threshold(&values, &its, 0.05, Rule::Pooled).unwrap();
        assert_eq!(pooled.threshold, metrics::fpr_threshold(&reals, 0.05));
        assert_eq!(reals.iter().filter(|&&z| z > pooled.threshold).count(), 50);
        // Balanced over both sources: the mean of the two rates is 5%, so 10% of the small source and none of the big one.
        let balanced = fit_threshold(&values, &its, 0.05, Rule::Balanced(100)).unwrap();
        assert_eq!(balanced.sources, 2);
        assert_eq!(values[900..1000].iter().filter(|&&z| z > balanced.threshold).count(), 10);
        assert!(balanced.threshold > pooled.threshold && balanced.scale > 0.0);
        // A source below the minimum does not count.
        assert_eq!(fit_threshold(&values, &its, 0.05, Rule::Balanced(101)).unwrap().sources, 1);
        assert!(fit_threshold(&values, &its, 0.05, Rule::Balanced(1000)).is_err());
        assert!(fit_threshold(&values[..100], &its[..100], 0.05, Rule::Pooled).unwrap_err().to_string().contains("at least 200 real items"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn the_threshold_is_the_one_calibrate_writes() {
        // `eval::calibrate` is the rule of record; this module mirrors it.
        let (logits, labels) = synthetic(3000, 9);
        let its = items(&logits, &labels);
        let dir = tempfile::tempdir().unwrap();
        let (weights, scores) = (dir.path().join("w.safetensors"), dir.path().join("scores.tsv"));
        crate::detector::ModelSpec::commfor(224).save_for(&weights).unwrap();
        let text: String = its.iter().map(|it| format!("{}\t{}\t{}\t{}\t0.5\t{}\n", it.path, it.label, it.generator, it.dataset, it.logits.iter().map(|x| format!("{x:.5}")).collect::<Vec<_>>().join("\t"))).collect();
        std::fs::write(&scores, text).unwrap();
        let read = read_items(&[scores.clone()], 6).unwrap();
        let values: Vec<f32> = read.iter().map(|it| Pool::Mean.of_all(&it.logits)).collect();
        for (rule, theirs) in [(Rule::Balanced(100), crate::eval::Rule::Balanced(100)), (Rule::Pooled, crate::eval::Rule::Pooled)] {
            crate::eval::calibrate(&weights, &[scores.clone()], 0.015, "photo", theirs).unwrap();
            let spec = crate::detector::ModelSpec::load_for(&weights).unwrap();
            let ours = fit_threshold(&values, &read, 0.015, rule).unwrap();
            assert_eq!(ours.scale, spec.logit_scale, "{rule:?}");
            assert_eq!((-(ours.scale as f64) * ours.threshold as f64) as f32, spec.logit_bias, "{rule:?}");
        }
    }

    #[test]
    fn fitted_weights_find_the_informative_views_and_drop_a_misleading_one() {
        let mut rng = Rng::new(21);
        let mut unit = move || rng.below(10_000) as f32 / 10_000.0 - 0.5;
        let (mut logits, mut labels) = (Vec::new(), Vec::new());
        for i in 0..4000 {
            let label = (i % 2) as u8;
            let level = if label == 1 { 1.5 } else { -1.5 };
            // View 0 is sharp, view 1 is noisy, view 2 is pure noise, view 3 points the wrong way.
            logits.push(vec![level + 2.0 * unit(), level + 8.0 * unit(), 6.0 * unit(), -0.3 * level + 6.0 * unit()]);
            labels.push(label);
        }
        let w = fit_weights(&logits, &labels).unwrap();
        assert!((w.iter().sum::<f32>() - 1.0).abs() < 1e-5 && w.iter().all(|&x| x >= 0.0), "{w:?}");
        assert!(w[0] > 0.7 && w[0] > 5.0 * w[1] && w[2] < 0.1 && w[3] == 0.0, "{w:?}");
        assert!(fit_weights(&logits, &vec![1; logits.len()]).is_err());
    }

    #[test]
    fn greedy_order_starts_with_the_view_that_agrees_most_with_all_views() {
        let (mut logits, _) = synthetic(1500, 2);
        // Make view 4 the pooled answer itself and view 1 a coin.
        let mut rng = Rng::new(8);
        for l in &mut logits {
            l[4] = l.iter().sum::<f32>() / 6.0;
            l[1] = if rng.below(2) == 0 { 9.0 } else { -9.0 };
        }
        let order = greedy_order(&Pool::Mean, &logits, 1.0);
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..6).collect::<Vec<_>>());
        assert_eq!(order[0], 4, "{order:?}");
        assert_ne!(order[0], 1);
    }

    #[test]
    fn flips_count_each_direction() {
        let labels = [1, 1, 1, 0, 0, 0];
        assert_eq!(flips(&labels, &[true, false, true, true, false, false], &[false, true, true, false, true, false]), (1, 1, 1, 1));
    }

    #[test]
    fn files_expand_and_rows_must_hold_every_view() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["a.x-views.tsv", "b.x-views.tsv", "b.x.tsv"] {
            std::fs::write(dir.path().join(name), "p.png\t1\tgen\td\t0.9\t1\t2\t3\t4\t5\t6\n").unwrap();
        }
        let found = expand(dir.path().join("*.x-views.tsv").to_str().unwrap()).unwrap();
        assert_eq!(found.iter().map(|p| p.file_name().unwrap().to_str().unwrap()).collect::<Vec<_>>(), ["a.x-views.tsv", "b.x-views.tsv"]);
        assert_eq!(expand("plain.tsv").unwrap(), vec![PathBuf::from("plain.tsv")]);
        assert!(expand(dir.path().join("*.none").to_str().unwrap()).is_err());
        assert_eq!(read_items(&found, 6).unwrap().len(), 2);
        // An early-exit file (fewer logits) or a probabilities-only archive is refused, with the reason.
        std::fs::write(dir.path().join("short.tsv"), "p.png\t1\tgen\td\t0.9\t1\t2\n").unwrap();
        let e = read_items(&[dir.path().join("short.tsv")], 6).unwrap_err().to_string();
        assert!(e.contains("has 2 view logits, not 6") && e.contains("--all-views"), "{e}");
        std::fs::write(dir.path().join("probs.tsv"), "p.png\t1\tgen\td\t0.9\n").unwrap();
        assert!(read_items(&[dir.path().join("probs.tsv")], 6).unwrap_err().to_string().contains("probabilities only"));
    }

    #[test]
    fn a_replay_fits_on_one_file_and_reports_on_another() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, seed: u64, n: usize| {
            let (logits, labels) = synthetic(n, seed);
            let text: String = items(&logits, &labels).iter().map(|it| format!("{}\t{}\t{}\t{}\t0.5\t{}\n", it.path, it.label, it.generator, it.dataset, it.logits.iter().map(|x| format!("{x:.5}")).collect::<Vec<_>>().join("\t"))).collect();
            let p = dir.path().join(name);
            std::fs::write(&p, text).unwrap();
            p.display().to_string()
        };
        let (fit, test) = (write("fit.tsv", 1, 3000), write("test.tsv", 2, 2000));
        // The paths of the two files coincide (00000.png ...): rename the fit file's, but for 40 shared ones.
        let renamed: String = std::fs::read_to_string(&fit).unwrap().lines().enumerate().map(|(i, l)| if i < 40 { format!("{l}\n") } else { format!("fit-{l}\n") }).collect();
        std::fs::write(&fit, renamed).unwrap();
        let args = Args {
            fit: vec![fit],
            test: vec![format!("held-out={test}")],
            spec: None,
            pools: ["mean", "trimmed", "median", "top2", "max", "fitted", "weights:0/1/1/1/1/1"].map(String::from).to_vec(),
            orders: vec!["shipped".into(), "greedy".into(), "0,1,2,3,4,5".into()],
            target_fpr: 0.015,
            rule: Rule::Balanced(100),
            refit: Refit::default(),
            slice_by: Some("dataset".into()),
            write_scores: Some(dir.path().join("out")),
        };
        let mut out = Vec::new();
        run(args, &mut out).unwrap();
        let out = String::from_utf8(out).unwrap();
        for want in ["Fit items: 2960 (1480 AI, 1480 real)", "40 items of the fit files are also test items (same path) and were left out of the fit", "averaged over the 3 real sources with at least 100 items", "### Fitted on the fit items", "| baseline (mean, standard schedule) | 2,0,1,3,4,5 |", "| trimmed mean |", "| max |", "| fitted weights ", "(greedy)", "### Test: held-out (2000 items: 1000 AI, 1000 real)", "held-out by dataset: mean | 2,0,1,3,4,5", "`early_exit` for median, order 0,1,2,3,4,5"] {
            assert!(out.contains(want), "missing `{want}` in:\n{out}");
        }
        // The baseline row changes no verdict against itself.
        assert!(out.lines().any(|l| l.starts_with("| baseline (mean, standard schedule) | 2,0,1,3,4,5 |") && l.ends_with("| +0 −0 AI, +0 −0 real |")), "{out}");
        // One score file per candidate, in the usual format, holding the views looked at.
        let written = std::fs::read_to_string(dir.path().join("out/held-out.median.012345.tsv")).unwrap();
        assert_eq!(written.lines().count(), 2000);
        let cols = written.lines().next().unwrap().split('\t').count();
        assert!((6..=11).contains(&cols), "{cols}");
    }
}
