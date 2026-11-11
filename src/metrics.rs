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
    let mut real: Vec<f32> = scores.iter().zip(labels).filter(|(_, &l)| l == 0).map(|(&s, _)| s).collect();
    let fake: Vec<f32> = scores.iter().zip(labels).filter(|(_, &l)| l == 1).map(|(&s, _)| s).collect();
    if real.is_empty() || fake.is_empty() {
        return f64::NAN;
    }
    real.sort_by(|a, b| b.total_cmp(a));
    // Largest number of real items we may flag.
    let allowed = (fpr * real.len() as f64).floor() as usize;
    // Flag strictly above the (allowed+1)-th highest real score.
    let thr = real[allowed.min(real.len() - 1)];
    fake.iter().filter(|&&s| s > thr).count() as f64 / fake.len() as f64
}

/// Percentile bootstrap confidence interval of a statistic over items.
pub fn bootstrap_ci<F: Fn(&[f32], &[u8]) -> f64>(scores: &[f32], labels: &[u8], stat: F, rounds: usize, seed: u64) -> (f64, f64) {
    let mut rng = crate::rng::Rng::new(seed);
    let n = scores.len();
    let mut vals = Vec::with_capacity(rounds);
    let (mut s, mut l) = (vec![0f32; n], vec![0u8; n]);
    for _ in 0..rounds {
        for i in 0..n {
            let k = rng.below(n);
            s[i] = scores[k];
            l[i] = labels[k];
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
    fn known_values() {
        // sklearn: roc_auc_score([0,0,1,1],[0.1,0.4,0.35,0.8]) = 0.75, AP = 0.8333
        let (s, l) = ([0.1, 0.4, 0.35, 0.8], [0, 0, 1, 1]);
        assert!((auroc(&s, &l) - 0.75).abs() < 1e-9);
        assert!((average_precision(&s, &l) - 0.833333333).abs() < 1e-6);
        let m = compute(&s, &l, 0.5);
        assert_eq!((m.precision, m.recall, m.fpr), (1.0, 0.5, 0.0));
    }
}
