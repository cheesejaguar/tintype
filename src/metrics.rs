//! Binary detection metrics. Positive class = AI-generated.
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Metrics {
    pub n: usize,
    pub n_fake: usize,
    pub n_real: usize,
    pub threshold: f32,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub accuracy: f64,
    pub balanced_accuracy: f64,
    /// False-positive rate: real images flagged as AI.
    pub fpr: f64,
    pub auroc: f64,
    pub average_precision: f64,
}

/// `scores` are P(fake) in `[0,1]`; `labels` are 1 for fake.
pub fn compute(scores: &[f32], labels: &[u8], threshold: f32) -> Metrics {
    assert_eq!(scores.len(), labels.len());
    let (mut tp, mut fp, mut tn, mut fn_) = (0f64, 0f64, 0f64, 0f64);
    for (&s, &l) in scores.iter().zip(labels) {
        match (s > threshold, l == 1) {
            (true, true) => tp += 1.0,
            (true, false) => fp += 1.0,
            (false, false) => tn += 1.0,
            (false, true) => fn_ += 1.0,
        }
    }
    let div = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
    let (precision, recall) = (div(tp, tp + fp), div(tp, tp + fn_));
    let tnr = div(tn, tn + fp);
    Metrics {
        n: scores.len(),
        n_fake: (tp + fn_) as usize,
        n_real: (tn + fp) as usize,
        threshold,
        precision,
        recall,
        f1: div(2.0 * precision * recall, precision + recall),
        accuracy: div(tp + tn, tp + tn + fp + fn_),
        balanced_accuracy: (recall + tnr) / 2.0,
        fpr: div(fp, fp + tn),
        auroc: auroc(scores, labels),
        average_precision: average_precision(scores, labels),
    }
}

/// Area under the ROC curve via the rank-sum statistic (ties get mid-ranks).
pub fn auroc(scores: &[f32], labels: &[u8]) -> f64 {
    let mut idx: Vec<usize> = (0..scores.len()).collect();
    idx.sort_by(|&a, &b| scores[a].total_cmp(&scores[b]));
    let (mut rank_sum, mut pos) = (0f64, 0f64);
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && scores[idx[j + 1]] == scores[idx[i]] {
            j += 1;
        }
        let mid = (i + j) as f64 / 2.0 + 1.0;
        for &k in &idx[i..=j] {
            if labels[k] == 1 {
                rank_sum += mid;
                pos += 1.0;
            }
        }
        i = j + 1;
    }
    let neg = scores.len() as f64 - pos;
    if pos == 0.0 || neg == 0.0 {
        return f64::NAN;
    }
    (rank_sum - pos * (pos + 1.0) / 2.0) / (pos * neg)
}

/// Average precision (area under the precision-recall curve, step-wise),
/// matching scikit-learn's `average_precision_score`.
pub fn average_precision(scores: &[f32], labels: &[u8]) -> f64 {
    let mut idx: Vec<usize> = (0..scores.len()).collect();
    idx.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]));
    let total_pos = labels.iter().filter(|&&l| l == 1).count() as f64;
    if total_pos == 0.0 {
        return f64::NAN;
    }
    let (mut tp, mut seen, mut ap, mut prev_recall) = (0f64, 0f64, 0f64, 0f64);
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && scores[idx[j + 1]] == scores[idx[i]] {
            j += 1;
        }
        for &k in &idx[i..=j] {
            seen += 1.0;
            if labels[k] == 1 {
                tp += 1.0;
            }
        }
        let recall = tp / total_pos;
        ap += (recall - prev_recall) * (tp / seen);
        prev_recall = recall;
        i = j + 1;
    }
    ap
}

/// Recall when the threshold is set so that at most `fpr` of real items are
/// flagged (the operating point moderation teams quote).
pub fn recall_at_fpr(scores: &[f32], labels: &[u8], fpr: f64) -> f64 {
    match empirical_cutoff(scores, labels, fpr) {
        Some(thr) if labels.contains(&1) => rates_at(scores, labels, thr).1,
        _ => f64::NAN,
    }
}

/// The cutoff [`recall_at_fpr`] operates at: the strictest score that lets
/// at most `fpr` of the real items through (items are flagged strictly above
/// it). `None` without real items.
pub fn empirical_cutoff(scores: &[f32], labels: &[u8], fpr: f64) -> Option<f32> {
    let mut real: Vec<f32> = scores.iter().zip(labels).filter(|(_, &l)| l == 0).map(|(&s, _)| s).collect();
    if real.is_empty() {
        return None;
    }
    real.sort_by(|a, b| b.total_cmp(a));
    // Largest number of real items we may flag.
    let allowed = (fpr * real.len() as f64).floor() as usize;
    // Flag strictly above the (allowed+1)-th highest real score.
    Some(real[allowed.min(real.len() - 1)])
}

/// `(false-positive rate, recall)` when everything above `cutoff` is
/// flagged: what a cutoff frozen elsewhere does on these items. A rate
/// without items of its class is NaN.
pub fn rates_at(scores: &[f32], labels: &[u8], cutoff: f32) -> (f64, f64) {
    let (mut fp, mut tp, mut real, mut fake) = (0usize, 0usize, 0usize, 0usize);
    for (&s, &l) in scores.iter().zip(labels) {
        if l == 1 {
            fake += 1;
            tp += (s > cutoff) as usize;
        } else {
            real += 1;
            fp += (s > cutoff) as usize;
        }
    }
    let div = |a: usize, b: usize| if b > 0 { a as f64 / b as f64 } else { f64::NAN };
    (div(fp, real), div(tp, fake))
}

/// Precision when a share `prevalence` of the incoming items is
/// AI-generated, from rates measured on any class mix:
/// `recall·π / (recall·π + fpr·(1 − π))`.
pub fn precision_at_prevalence(recall: f64, fpr: f64, prevalence: f64) -> f64 {
    let (tp, fp) = (recall * prevalence, fpr * (1.0 - prevalence));
    if tp + fp > 0.0 {
        tp / (tp + fp)
    } else {
        f64::NAN
    }
}

/// P(X ≤ k) for X ~ Binomial(n, p).
fn binomial_cdf(k: u64, n: u64, p: f64) -> f64 {
    if p <= 0.0 {
        return 1.0;
    }
    if p >= 1.0 {
        return if k >= n { 1.0 } else { 0.0 };
    }
    let (lp, lq) = (p.ln(), (1.0 - p).ln());
    let mut ln_c = 0f64; // ln C(n, 0)
    let mut sum = 0f64;
    for i in 0..=k.min(n) {
        sum += (ln_c + i as f64 * lp + (n - i) as f64 * lq).exp();
        if i < n {
            ln_c += ((n - i) as f64).ln() - ((i + 1) as f64).ln();
        }
    }
    sum.min(1.0)
}

/// One-sided Clopper–Pearson upper bound on a rate after `k` events in `n`
/// independent trials: the largest rate that would still give `k` or fewer
/// events with probability `1 − confidence`. With no events it is
/// `1 − (1 − confidence)^(1/n)`: about 3/n at 95%, so showing a
/// false-positive rate below 0.1% takes about 3,000 authentic items with
/// none flagged. The trials must be independent: count clusters, not
/// repeats of one source file.
pub fn clopper_pearson_upper(k: usize, n: usize, confidence: f64) -> f64 {
    if n == 0 {
        return f64::NAN;
    }
    if k >= n {
        return 1.0;
    }
    let alpha = 1.0 - confidence;
    if k == 0 {
        return 1.0 - alpha.powf(1.0 / n as f64);
    }
    let (mut lo, mut hi) = (k as f64 / n as f64, 1.0);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if binomial_cdf(k as u64, n as u64, mid) > alpha {
            lo = mid
        } else {
            hi = mid
        }
    }
    hi
}

/// The matching one-sided lower bound (0 with no events).
pub fn clopper_pearson_lower(k: usize, n: usize, confidence: f64) -> f64 {
    if n == 0 {
        return f64::NAN;
    }
    if k == 0 {
        return 0.0;
    }
    // k or more events in n trials <=> n - k or fewer non-events.
    1.0 - clopper_pearson_upper(n - k, n, confidence)
}

/// The cutoff an authentic-only order statistic gives: the smallest score
/// among `real` such that, flagging strictly above it, the one-sided
/// Clopper–Pearson upper bound on the false-positive rate is at most
/// `target_fpr` at `confidence`. Stricter than the plain quantile
/// ([`fpr_threshold`]), which aims at the target on these very items. Also
/// returns how many of `real` lie above the cutoff. `None` when even
/// flagging nothing here cannot show the target (too few items: the bound
/// with no events is about `3/n` at 95%).
pub fn order_statistic_cutoff(real: &[f32], target_fpr: f64, confidence: f64) -> Option<(f32, usize)> {
    let n = real.len();
    if n == 0 || clopper_pearson_upper(0, n, confidence) > target_fpr {
        return None;
    }
    let mut r = real.to_vec();
    r.sort_by(|a, b| b.total_cmp(a));
    // The largest k whose bound still meets the target; the bound grows with k.
    let (mut lo, mut hi) = (0usize, n - 1);
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if clopper_pearson_upper(mid, n, confidence) <= target_fpr {
            lo = mid
        } else {
            hi = mid - 1
        }
    }
    // Flagging strictly above the (k+1)-th highest score flags at most k (fewer on ties).
    let thr = r[lo];
    Some((thr, r.iter().filter(|&&x| x > thr).count()))
}

/// Percentile bootstrap confidence interval of a statistic over items.
pub fn bootstrap_ci<F: Fn(&[f32], &[u8]) -> f64>(scores: &[f32], labels: &[u8], stat: F, rounds: usize, seed: u64) -> (f64, f64) {
    bootstrap_ci_clustered(scores, labels, None, stat, rounds, seed)
}

/// Number the distinct values of `keys` in order of first appearance: the
/// cluster ids [`bootstrap_ci_clustered`] takes.
pub fn cluster_ids<T: Ord + Clone>(keys: &[T]) -> Vec<usize> {
    let mut seen: std::collections::BTreeMap<T, usize> = Default::default();
    keys.iter()
        .map(|k| {
            let n = seen.len();
            *seen.entry(k.clone()).or_insert(n)
        })
        .collect()
}

/// As [`bootstrap_ci`], resampling whole clusters: `clusters[i]` is the
/// cluster of item `i` (ids from [`cluster_ids`]), and each round draws as
/// many clusters as there are, with replacement, taking every item of a
/// drawn cluster. Use it when items repeat a source or a parent: derivatives
/// of one photo are not independent evidence, and the item-level interval is
/// then too narrow. `None` treats every item as its own cluster, which is
/// the item-level bootstrap, draw for draw.
pub fn bootstrap_ci_clustered<F: Fn(&[f32], &[u8]) -> f64>(scores: &[f32], labels: &[u8], clusters: Option<&[usize]>, stat: F, rounds: usize, seed: u64) -> (f64, f64) {
    let mut rng = crate::rng::Rng::new(seed);
    let n = scores.len();
    let members: Option<Vec<Vec<usize>>> = clusters.map(|c| {
        let mut m = vec![Vec::new(); c.iter().max().map_or(0, |x| x + 1)];
        c.iter().enumerate().for_each(|(i, &k)| m[k].push(i));
        m.retain(|v| !v.is_empty());
        m
    });
    let mut vals = Vec::with_capacity(rounds);
    let (mut s, mut l) = (vec![0f32; n], vec![0u8; n]);
    for _ in 0..rounds {
        match &members {
            None => {
                for i in 0..n {
                    let k = rng.below(n);
                    s[i] = scores[k];
                    l[i] = labels[k];
                }
            }
            Some(m) => {
                s.clear();
                l.clear();
                for _ in 0..m.len() {
                    for &k in &m[rng.below(m.len())] {
                        s.push(scores[k]);
                        l.push(labels[k]);
                    }
                }
            }
        }
        let v = stat(&s, &l);
        if v.is_finite() {
            vals.push(v);
        }
    }
    if vals.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    vals.sort_by(|a, b| a.total_cmp(b));
    (vals[(0.025 * vals.len() as f64) as usize], vals[((0.975 * vals.len() as f64) as usize).min(vals.len() - 1)])
}

/// Calibration `p = sigmoid(scale * z + bias)` such that `p = 0.5` sits at the
/// logit exceeded by only `target_fpr` of real items, and the slope makes `p`
/// track the class-balanced posterior (Platt scaling around that anchor).
/// Returns `(scale, bias, logit_threshold)`.
pub fn calibrate(real: &[f32], fake: &[f32], target_fpr: f64) -> (f32, f32, f32) {
    calibrate_at(real, fake, fpr_threshold(real, target_fpr))
}

/// The logit exceeded by `target_fpr` of `real`.
pub fn fpr_threshold(real: &[f32], target_fpr: f64) -> f32 {
    let mut r = real.to_vec();
    r.sort_by(|a, b| a.total_cmp(b));
    let k = (((1.0 - target_fpr) * r.len() as f64).ceil() as usize).clamp(1, r.len()) - 1;
    r[k]
}

/// Operate a system at another system's false-positive count: the threshold
/// is the strictest one that lets at most `allowed` real items through.
/// Returns (real items flagged, AI items flagged).
pub fn at_false_positive_count(real: &[f32], fake: &[f32], allowed: usize) -> (usize, usize) {
    let mut r = real.to_vec();
    r.sort_by(|a, b| b.total_cmp(a));
    // Everything above the (allowed+1)-th highest real score is flagged.
    let thr = r.get(allowed).copied().unwrap_or(f32::NEG_INFINITY);
    (real.iter().filter(|&&x| x > thr).count(), fake.iter().filter(|&&x| x > thr).count())
}

/// As [`calibrate`], with the logit threshold given.
pub fn calibrate_at(real: &[f32], fake: &[f32], thr: f32) -> (f32, f32, f32) {
    // One-parameter logistic fit of the slope, classes weighted equally.
    let (wr, wf) = (0.5 / real.len().max(1) as f64, 0.5 / fake.len().max(1) as f64);
    let mut scale = 1.0f64;
    for _ in 0..200 {
        let (mut g, mut h) = (0f64, 0f64);
        for (zs, y, w) in [(real, 0.0, wr), (fake, 1.0, wf)] {
            for &z in zs {
                let d = (z - thr) as f64;
                let p = 1.0 / (1.0 + (-scale * d).exp());
                g += w * (p - y) * d;
                h += w * p * (1.0 - p) * d * d;
            }
        }
        let step = g / (h + 1e-9);
        scale = (scale - step).clamp(0.05, 20.0);
        if step.abs() < 1e-7 {
            break;
        }
    }
    (scale as f32, (-scale * thr as f64) as f32, thr)
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_and_chance() {
        let m = compute(&[0.9, 0.8, 0.2, 0.1], &[1, 1, 0, 0], 0.5);
        assert_eq!((m.precision, m.recall, m.auroc, m.average_precision), (1.0, 1.0, 1.0, 1.0));
        assert_eq!(auroc(&[0.5, 0.5, 0.5, 0.5], &[1, 0, 1, 0]), 0.5);
    }

    #[test]
    fn recall_at_fixed_fpr() {
        // 10 reals, 4 fakes; allowing 10% FPR lets one real be flagged.
        let mut s = vec![0.1f32; 10];
        s[0] = 0.9;
        s[1] = 0.6;
        let mut l = vec![0u8; 10];
        s.extend([0.95, 0.7, 0.5, 0.2]);
        l.extend([1, 1, 1, 1]);
        assert_eq!(recall_at_fpr(&s, &l, 0.0), 0.25); // only 0.95 beats the top real
        assert_eq!(recall_at_fpr(&s, &l, 0.10), 0.5); // threshold drops to 0.6
        let (lo, hi) = bootstrap_ci(&s, &l, |s, l| compute(s, l, 0.5).recall, 200, 1);
        assert!(lo <= 0.5 && hi >= 0.5 && lo >= 0.0 && hi <= 1.0);
    }

    #[test]
    fn stricter_operating_points_and_frozen_cutoffs() {
        // 2,000 reals with scores 0..1 in steps of 1/2000; fakes spread over the top.
        let mut s: Vec<f32> = (0..2000).map(|i| i as f32 / 2000.0).collect();
        let mut l = vec![0u8; 2000];
        s.extend((0..100).map(|i| 0.99 + i as f32 / 10_000.0));
        l.extend(vec![1u8; 100]);
        // 0.1% of 2,000 lets two reals through: the cutoff is the third-highest real score.
        assert_eq!(empirical_cutoff(&s, &l, 0.001), Some(1997.0 / 2000.0));
        assert_eq!(empirical_cutoff(&s, &l, 0.01), Some(1979.0 / 2000.0));
        assert_eq!(empirical_cutoff(&[0.5], &[1], 0.01), None);
        let (fpr, recall) = rates_at(&s, &l, 1997.0 / 2000.0);
        assert_eq!(fpr, 2.0 / 2000.0);
        assert_eq!(recall, recall_at_fpr(&s, &l, 0.001));
        assert!(recall < recall_at_fpr(&s, &l, 0.01) && recall_at_fpr(&s, &l, 0.01) == 1.0);
        // A cutoff frozen elsewhere: the rates here are whatever they are.
        assert_eq!(rates_at(&s, &l, 0.5), (999.0 / 2000.0, 1.0));
        assert!(rates_at(&[0.9], &[1], 0.5).0.is_nan());
    }

    #[test]
    fn precision_depends_on_prevalence() {
        // Confirmation 7's shipped rates (recall 0.897, 130 of 10,747 flagged) at 1% prevalence: about 43%.
        let p = precision_at_prevalence(0.897, 130.0 / 10_747.0, 0.01);
        assert!((p - 0.428).abs() < 0.005, "{p}");
        // 90% precision at 90% recall and 1% prevalence takes about 0.10% FPR.
        assert!((precision_at_prevalence(0.9, 0.00101, 0.01) - 0.9).abs() < 0.001);
        assert_eq!(precision_at_prevalence(0.9, 0.0, 0.01), 1.0);
        assert!(precision_at_prevalence(0.0, 0.0, 0.5).is_nan());
    }

    #[test]
    fn clopper_pearson_bounds() {
        // No flags among 3,000 independent authentic items: 95% upper bound just under 0.10%.
        let u = clopper_pearson_upper(0, 3000, 0.95);
        assert!((u - 0.000998).abs() < 2e-6, "{u}");
        assert!(clopper_pearson_upper(0, 2994, 0.95) > 0.001 && clopper_pearson_upper(0, 2995, 0.95) <= 0.001);
        // The rule of three.
        assert!((clopper_pearson_upper(0, 100, 0.95) - 0.0295).abs() < 1e-3);
        // Reference values (R: binom.test(k, n, alternative = "less")$conf.int[2]).
        assert!((clopper_pearson_upper(3, 20, 0.95) - 0.3439).abs() < 1e-3, "{}", clopper_pearson_upper(3, 20, 0.95));
        assert!((clopper_pearson_upper(130, 10_747, 0.95) - 0.01398).abs() < 2e-5, "{}", clopper_pearson_upper(130, 10_747, 0.95));
        assert!((clopper_pearson_lower(3, 20, 0.95) - 0.0422).abs() < 1e-3, "{}", clopper_pearson_lower(3, 20, 0.95));
        assert_eq!(clopper_pearson_upper(5, 5, 0.95), 1.0);
        assert_eq!(clopper_pearson_lower(0, 5, 0.95), 0.0);
        assert!(clopper_pearson_upper(0, 0, 0.95).is_nan());
        // The bound grows with the count and shrinks with the sample.
        assert!(clopper_pearson_upper(1, 3000, 0.95) > u && clopper_pearson_upper(0, 6000, 0.95) < u);
    }

    #[test]
    fn order_statistic_cutoff_is_stricter_than_the_quantile() {
        let real: Vec<f32> = (0..10_000).map(|i| i as f32).collect();
        // The plain 1% quantile flags 100 of these; the bound-respecting cutoff flags fewer.
        let q = fpr_threshold(&real, 0.01);
        assert_eq!(real.iter().filter(|&&x| x > q).count(), 100);
        let (thr, flagged) = order_statistic_cutoff(&real, 0.01, 0.95).unwrap();
        assert!(thr > q && flagged == 83, "{thr} {flagged}");
        assert!(clopper_pearson_upper(flagged, real.len(), 0.95) <= 0.01);
        assert!(clopper_pearson_upper(flagged + 1, real.len(), 0.95) > 0.01);
        // 0.1% cannot be shown with 2,000 items, and is shown with none flagged at 3,000.
        assert!(order_statistic_cutoff(&real[..2000], 0.001, 0.95).is_none());
        assert_eq!(order_statistic_cutoff(&real[..3000], 0.001, 0.95), Some((2999.0, 0)));
        assert!(order_statistic_cutoff(&[], 0.5, 0.95).is_none());
    }

    #[test]
    fn clustered_bootstrap_is_wider_when_items_repeat() {
        // 40 parents with 25 copies each; a parent's copies are all caught or all missed.
        let (mut s, mut l, mut parent) = (Vec::new(), Vec::new(), Vec::new());
        for p in 0..40 {
            for _ in 0..25 {
                s.push(if p % 4 == 0 { 0.2f32 } else { 0.9 });
                l.push(1u8);
                parent.push(format!("p{p}"));
            }
        }
        let recall = |s: &[f32], l: &[u8]| compute(s, l, 0.5).recall;
        let ids = cluster_ids(&parent);
        assert_eq!((ids[0], ids[24], ids[25], ids[999]), (0, 0, 1, 39));
        let (lo, hi) = bootstrap_ci(&s, &l, recall, 500, 7);
        let (clo, chi) = bootstrap_ci_clustered(&s, &l, Some(&ids), recall, 500, 7);
        assert!(clo <= 0.75 && chi >= 0.75 && lo <= 0.75 && hi >= 0.75);
        assert!(chi - clo > 2.0 * (hi - lo), "item ({lo}, {hi}), clustered ({clo}, {chi})");
        // One cluster per item is the item-level bootstrap, draw for draw.
        let single: Vec<usize> = (0..s.len()).collect();
        assert_eq!(bootstrap_ci_clustered(&s, &l, Some(&single), recall, 200, 3), bootstrap_ci(&s, &l, recall, 200, 3));
    }

    #[test]
    fn known_values() {
        // sklearn: roc_auc_score([0,0,1,1],[0.1,0.4,0.35,0.8]) = 0.75, AP = 0.8333
        let (s, l) = ([0.1, 0.4, 0.35, 0.8], [0, 0, 1, 1]);
        assert!((auroc(&s, &l) - 0.75).abs() < 1e-9);
        assert!((average_precision(&s, &l) - 0.833333333).abs() < 1e-6);
        let m = compute(&s, &l, 0.5);
        assert_eq!((m.precision, m.recall, m.fpr), (1.0, 0.5, 0.0));
    }

    #[test]
    fn matched_false_positive_count() {
        let real = [0.1, 0.2, 0.9, 0.8, 0.3];
        let fake = [0.95, 0.85, 0.5, 0.25];
        // No real item allowed through: only AI items above the top real score count.
        assert_eq!(at_false_positive_count(&real, &fake, 0), (0, 1));
        // One allowed: threshold drops to the second-highest real score.
        assert_eq!(at_false_positive_count(&real, &fake, 1), (1, 2));
        // Ties never let more real items through than allowed.
        assert_eq!(at_false_positive_count(&[0.5, 0.5, 0.1], &[0.6, 0.5], 1).0, 0);
        // Allowing every real item flags everything.
        assert_eq!(at_false_positive_count(&real, &fake, 5), (5, 4));
    }
}
