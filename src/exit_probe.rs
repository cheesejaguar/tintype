//! Could a view stop after fewer blocks? A diagnostic, not a feature: fit a
//! linear read-out of the class token as an earlier block leaves it to the
//! logit the whole network gives the same view, and see how well it ranks
//! and how many views it could let go.
//!
//! Nothing here is used to score anything. The read-out is ridge regression
//! on standardised features, solved exactly, so there is nothing to tune and
//! nothing that depends on a random seed; it is fitted on one half of the
//! items and judged on the other, and the halves are swapped.
use anyhow::{ensure, Result};

/// A linear read-out of standardised features.
pub struct Probe {
    mean: Vec<f64>,
    inv_std: Vec<f64>,
    w: Vec<f64>,
    b: f64,
}

impl Probe {
    /// Least squares from `x` (`[n, d]`) to `y` with penalty `lambda` on the
    /// weights of the standardised features (not on the intercept).
    pub fn fit(x: &[f32], y: &[f32], d: usize, lambda: f64) -> Result<Self> {
        let n = y.len();
        ensure!(n > 0 && d > 0 && x.len() == n * d, "a probe needs {d} features for each of {n} items");
        let mut mean = vec![0f64; d];
        for row in x.chunks(d) {
            row.iter().zip(&mut mean).for_each(|(&v, m)| *m += v as f64);
        }
        mean.iter_mut().for_each(|m| *m /= n as f64);
        let mut var = vec![0f64; d];
        for row in x.chunks(d) {
            row.iter().zip(&mean).zip(&mut var).for_each(|((&v, m), s)| *s += (v as f64 - m) * (v as f64 - m));
        }
        // A feature that never varies carries nothing: its weight stays zero.
        let inv_std: Vec<f64> = var.iter().map(|&s| if s > 0.0 { (n as f64 / s).sqrt() } else { 0.0 }).collect();
        let b = y.iter().map(|&v| v as f64).sum::<f64>() / n as f64;
        // Normal equations on the standardised, centred data: (Z'Z + lambda I) w = Z'(y - b).
        let mut a = vec![0f64; d * d];
        let mut rhs = vec![0f64; d];
        let mut z = vec![0f64; d];
        for (row, &target) in x.chunks(d).zip(y) {
            for k in 0..d {
                z[k] = (row[k] as f64 - mean[k]) * inv_std[k];
            }
            let t = target as f64 - b;
            for i in 0..d {
                rhs[i] += z[i] * t;
                let zi = z[i];
                for (aij, zj) in a[i * d..i * d + i + 1].iter_mut().zip(&z) {
                    *aij += zi * zj;
                }
            }
        }
        for i in 0..d {
            a[i * d + i] += lambda.max(1e-9);
        }
        let w = solve_spd(&mut a, &rhs, d)?;
        Ok(Self { mean, inv_std, w, b })
    }

    pub fn predict(&self, row: &[f32]) -> f32 {
        (self.b + row.iter().zip(&self.mean).zip(&self.inv_std).zip(&self.w).map(|(((&v, m), s), w)| (v as f64 - m) * s * w).sum::<f64>()) as f32
    }
}

/// Solve `A x = rhs` for a symmetric positive-definite `A` given by its
/// lower triangle (row-major, `d` by `d`), by Cholesky factorisation in place.
fn solve_spd(a: &mut [f64], rhs: &[f64], d: usize) -> Result<Vec<f64>> {
    for i in 0..d {
        for j in 0..=i {
            let mut sum = a[i * d + j];
            for k in 0..j {
                sum -= a[i * d + k] * a[j * d + k];
            }
            if i == j {
                ensure!(sum > 0.0, "the features are degenerate (not positive definite at {i})");
                a[i * d + i] = sum.sqrt();
            } else {
                a[i * d + j] = sum / a[j * d + j];
            }
        }
    }
    let mut x = rhs.to_vec();
    for i in 0..d {
        for k in 0..i {
            x[i] -= a[i * d + k] * x[k];
        }
        x[i] /= a[i * d + i];
    }
    for i in (0..d).rev() {
        for k in i + 1..d {
            x[i] -= a[k * d + i] * x[k];
        }
        x[i] /= a[i * d + i];
    }
    Ok(x)
}

/// Which of two halves an item is in: by a hash of its path, so that it
/// depends on neither the order of the list nor the other items.
pub fn half_of(path: &str) -> usize {
    (path.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3)) >> 17) as usize & 1
}

/// Predictions for every item from a read-out fitted on the other half.
pub fn cross_fitted(x: &[f32], y: &[f32], d: usize, half: &[usize], lambda: f64) -> Result<Vec<f32>> {
    let mut out = vec![0f32; y.len()];
    for held in 0..2 {
        let (mut fx, mut fy) = (Vec::new(), Vec::new());
        for (i, row) in x.chunks(d).enumerate() {
            if half[i] != held {
                fx.extend_from_slice(row);
                fy.push(y[i]);
            }
        }
        let probe = Probe::fit(&fx, &fy, d, lambda)?;
        for (i, row) in x.chunks(d).enumerate() {
            if half[i] == held {
                out[i] = probe.predict(row);
            }
        }
    }
    Ok(out)
}

/// The largest share of items the rule "stop when the read-out is at least
/// `m` from the threshold" lets go while at most `allowed` of those it lets
/// go end up on the other side of the threshold from the whole network:
/// `(share, m, wrong among them)`. `early` and `full` are the read-out's and
/// the whole network's logits.
pub fn exit_share(early: &[f32], full: &[f32], threshold: f32, allowed: f64) -> (f64, f32, usize) {
    let mut order: Vec<usize> = (0..early.len()).collect();
    order.sort_by(|&a, &b| (early[b] - threshold).abs().total_cmp(&(early[a] - threshold).abs()));
    let (mut wrong, mut best) = (0usize, (0.0, f32::INFINITY, 0));
    for (k, &i) in order.iter().enumerate() {
        wrong += ((early[i] > threshold) != (full[i] > threshold)) as usize;
        // Items the same distance away stop together.
        let last_of_its_distance = order.get(k + 1).is_none_or(|&j| (early[j] - threshold).abs() < (early[i] - threshold).abs());
        if last_of_its_distance && wrong as f64 <= allowed * (k + 1) as f64 {
            best = ((k + 1) as f64 / early.len() as f64, (early[i] - threshold).abs(), wrong);
        }
    }
    best
}

/// Print, for each number of blocks in `blocks`, how a read-out of the
/// class token after that many blocks compares with the whole network on
/// the view early exit looks at first, over the first `n` readable images
/// of `manifest` (see docs/runbooks/cpu-profile-int8.md).
pub fn report(weights: &std::path::Path, manifest: &std::path::Path, n: usize, blocks: &[usize], lambda: f64, allowed: f64) -> Result<()> {
    use crate::cpu_vit::CpuVit;
    use crate::detector::{Detector, Device, ModelSpec};
    use crate::metrics::{auroc, recall_at_fpr};
    use crate::vit::VitConfig;
    use rayon::prelude::*;
    let spec = ModelSpec::load_for(weights)?;
    let full_cfg = VitConfig::by_name(&spec.arch, spec.img).filter(|c| c.dim == 384 && c.reg_tokens == 0).ok_or_else(|| anyhow::anyhow!("exit probes are for the ViT-S/16 photo model"))?;
    ensure!(blocks.iter().all(|&b| b >= 1 && b <= full_cfg.depth), "--blocks takes numbers from 1 to {}", full_cfg.depth);
    let w = crate::weights::Weights::load(weights, &spec.prefix)?;
    let first_view = spec.early_exit.as_ref().and_then(|e| e.order.first().copied()).unwrap_or(0);
    let det = Detector::load(weights, Some(spec.clone()), 1, Device::Cpu)?;
    let (pre, threshold, d) = (det.preprocessor(), det.threshold_logit(), full_cfg.dim);
    let full = CpuVit::new(&w, full_cfg)?;
    let cut: Vec<CpuVit> = blocks.iter().map(|&b| CpuVit::new(&w, VitConfig { depth: b, ..full_cfg })).collect::<Result<_>>()?;

    let items: Vec<crate::ingest::Item> = crate::ingest::read_manifest(manifest)?.into_iter().take(n).collect();
    let (mut labels, mut half, mut full_logit) = (Vec::new(), Vec::new(), Vec::new());
    let mut features: Vec<Vec<f32>> = vec![Vec::new(); blocks.len()];
    for chunk in items.chunks(48) {
        // The view early exit looks at first, of every image that decodes.
        let views: Vec<(u8, usize, Vec<u8>)> = chunk
            .par_iter()
            .filter_map(|it| {
                let img = crate::image_io::load(&it.path).ok()?;
                Some((it.label, half_of(&it.path.to_string_lossy()), pre.plan(&img)[first_view].render(&img).data))
            })
            .collect();
        let batch: Vec<u8> = views.iter().flat_map(|v| v.2.iter().copied()).collect();
        labels.extend(views.iter().map(|v| v.0));
        half.extend(views.iter().map(|v| v.1));
        full_logit.extend(full.forward(&batch));
        for (vit, f) in cut.iter().zip(&mut features) {
            f.extend(vit.class_tokens(&batch));
        }
    }
    ensure!(labels.contains(&0) && labels.contains(&1), "the probe needs authentic and generated images");
    let (count, real) = (labels.len(), labels.iter().filter(|&&l| l == 0).count());
    println!("{count} images ({real} authentic, {} generated), view {first_view} of each; threshold logit {threshold:.3}; ridge penalty {lambda}; read-outs fitted on one half and judged on the other", count - real);
    println!();
    println!("| Blocks | Read-out | AUROC | Recall at 1% FPR | Recall at 0.1% FPR | RMS distance from the whole network's logit | Views it could stop with at most {:.1}% of them on the other side | of the authentic | margin | Blocks per view then | Network time then |", 100.0 * allowed);
    println!("|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|");
    let line = |blocks: String, name: &str, score: &[f32], stops: bool| {
        let rms = (score.iter().zip(&full_logit).map(|(a, b)| ((a - b) as f64).powi(2)).sum::<f64>() / count as f64).sqrt();
        let mut row = format!("| {blocks} | {name} | {:.4} | {:.1}% | {:.1}% | {rms:.3} |", auroc(score, &labels), 100.0 * recall_at_fpr(score, &labels, 0.01), 100.0 * recall_at_fpr(score, &labels, 0.001));
        if stops {
            let (share, margin, _) = exit_share(score, &full_logit, threshold, allowed);
            let among_real = score.iter().zip(&labels).filter(|(&a, &l)| l == 0 && (a - threshold).abs() >= margin).count() as f64 / real as f64;
            // The whole network costs its blocks less three quarters of the last (only the class token goes through it); a view that stops has run `b` whole blocks.
            let whole = full_cfg.depth as f64 - 0.75;
            let b: f64 = blocks.parse::<f64>().unwrap_or(whole).min(whole);
            let per_view = whole - share * (whole - b);
            row += &format!(" {:.1}% | {:.1}% | {margin:.2} | {per_view:.2} | {:.0}% |", 100.0 * share, 100.0 * among_real, 100.0 * per_view / whole);
        } else {
            row += " | | | | |";
        }
        println!("{row}");
    };
    line(full_cfg.depth.to_string(), "the network itself", &full_logit, false);
    for ((&b, vit), f) in blocks.iter().zip(&cut).zip(&features) {
        line(b.to_string(), "fitted linear read-out of the class token", &cross_fitted(f, &full_logit, d, &half, lambda)?, true);
        line(b.to_string(), "the shipped final normalisation and head, untrained for this depth", &vit.read_out(f), true);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_probe_recovers_a_linear_map_and_shrinks_with_the_penalty() {
        let mut rng = crate::rng::Rng::new(4);
        let (n, d) = (400, 6);
        let truth = [1.5f32, -2.0, 0.0, 0.25, 3.0, -0.5];
        // Features of very different sizes and offsets, one of them constant.
        let x: Vec<f32> = (0..n * d).map(|i| if i % d == 2 { 7.0 } else { rng.range(-1.0, 1.0) * [1.0, 10.0, 1.0, 0.1, 2.0, 5.0][i % d] + [0.0, 3.0, 0.0, -1.0, 20.0, 0.0][i % d] }).collect();
        let y: Vec<f32> = x.chunks(d).map(|r| 0.75 + r.iter().zip(truth).map(|(a, b)| a * b).sum::<f32>()).collect();
        let exact = Probe::fit(&x, &y, d, 0.0).unwrap();
        for (row, &want) in x.chunks(d).zip(&y) {
            assert!((exact.predict(row) - want).abs() < 2e-3, "{} against {want}", exact.predict(row));
        }
        // A heavy penalty pulls every prediction towards the mean target.
        let mean = y.iter().sum::<f32>() / n as f32;
        let heavy = Probe::fit(&x, &y, d, 1e9).unwrap();
        assert!(x.chunks(d).all(|row| (heavy.predict(row) - mean).abs() < 1e-2));
        assert!(Probe::fit(&x[..d], &y, d, 1.0).is_err());
    }

    #[test]
    fn cross_fitting_predicts_each_half_from_the_other() {
        let mut rng = crate::rng::Rng::new(8);
        let (n, d) = (300, 3);
        let x: Vec<f32> = (0..n * d).map(|_| rng.range(-1.0, 1.0)).collect();
        let y: Vec<f32> = x.chunks(d).map(|r| 2.0 * r[0] - r[1] + 0.5 * r[2] + 1.0).collect();
        let half: Vec<usize> = (0..n).map(|i| half_of(&format!("store/{i}.jpg"))).collect();
        assert!(half.iter().filter(|&&h| h == 0).count() > n / 3 && half.iter().filter(|&&h| h == 1).count() > n / 3);
        assert_eq!(half_of("a/b.jpg"), half_of("a/b.jpg"));
        let got = cross_fitted(&x, &y, d, &half, 1e-6).unwrap();
        assert!(got.iter().zip(&y).all(|(a, b)| (a - b).abs() < 1e-3));
    }

    #[test]
    fn the_exit_share_is_the_most_that_stays_within_the_allowed_disagreement() {
        // Threshold 0. Sorted by distance: 9 (agrees), -8 (agrees), 5 (the whole network says the other side), 4, -3, 1 (agree).
        let early = [1.0f32, 9.0, -3.0, 5.0, -8.0, 4.0];
        let full = [2.0f32, 7.0, -1.0, -0.5, -9.0, 3.0];
        // No disagreement allowed: only the two farthest may stop.
        assert_eq!(exit_share(&early, &full, 0.0, 0.0), (2.0 / 6.0, 8.0, 0));
        // One in five allowed: five of six stop with one wrong among them; all six, one wrong in six, too.
        assert_eq!(exit_share(&early, &full, 0.0, 0.2), (1.0, 1.0, 1));
        assert_eq!(exit_share(&early, &full, 0.0, 0.25).2, 1);
        // A threshold elsewhere moves the distances and the sides.
        let (share, margin, wrong) = exit_share(&[3.0, 0.0], &[3.0, 0.5], 1.0, 0.0);
        assert_eq!((share, margin, wrong), (1.0, 1.0, 0));
        assert_eq!(exit_share(&[], &[], 0.0, 0.01).0, 0.0);
    }
}
