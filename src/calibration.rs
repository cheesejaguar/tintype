//! Cutoffs fitted on authentic media only, and whether they hold elsewhere.
//!
//! The quantile rules of `tintype calibrate` aim at a false-positive rate on
//! the calibration items themselves. The order-statistic rule here
//! (`--rule bound`) instead picks the loosest cutoff whose one-sided
//! Clopper–Pearson upper bound on that rate is still within the target, so
//! the target is a statement about the population the items were drawn from,
//! at a stated confidence. That statement needs independent draws, so items
//! that repeat a parent are first reduced to one score per parent
//! ([`one_per_parent`]).
//!
//! A bound on the calibration population says nothing about another
//! population: the archived NTIRE scores are the standing example (a clean
//! cutoff at 0.96% flagged 10.50% of distorted authentic images). Transfer
//! is therefore measured, per predeclared stratum, on separate items
//! ([`transfer`]), and never assumed.
//!
//! There is deliberately no per-group cutoff here. A cutoff that depends on
//! a group is admissible only where the group can be observed at inference,
//! has enough calibration data of its own, and is shown to transfer; the
//! NTIRE operation labels are for diagnosis only, and the final JPEG
//! quantisation tables are identical across the NTIRE splits, so file
//! metadata cannot recover an operation history.
use crate::metrics::{clopper_pearson_lower, clopper_pearson_upper, order_statistic_cutoff};
use anyhow::{bail, Result};
use std::collections::BTreeMap;

/// How many independent authentic items, with none flagged, it takes to show
/// a false-positive rate of at most `target_fpr` at `confidence`: the
/// smallest `n` with `1 − (1 − confidence)^(1/n) ≤ target_fpr`. 2,995 for
/// 0.1% at 95%, 299 for 1%.
pub fn needed_with_no_flags(target_fpr: f64, confidence: f64) -> usize {
    if !(target_fpr > 0.0 && target_fpr < 1.0 && confidence > 0.0 && confidence < 1.0) {
        return usize::MAX;
    }
    let mut n = ((1.0 - confidence).ln() / (1.0 - target_fpr).ln()).ceil().max(1.0) as usize;
    // The closed form and the bound round differently at the edge: settle it with the bound itself.
    while n > 1 && clopper_pearson_upper(0, n - 1, confidence) <= target_fpr {
        n -= 1;
    }
    while clopper_pearson_upper(0, n, confidence) > target_fpr {
        n += 1;
    }
    n
}

/// Which score stands for a parent whose derivatives were scored too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerParent {
    /// The highest score among the parent and its derivatives: the parent
    /// counts as flagged when any version of it is. The bound then holds for
    /// every processing the calibration set contains, at the price of a
    /// stricter cutoff. The default.
    Max,
    /// The score of the unprocessed original only (the item with no
    /// `transform`). The bound then speaks for originals alone; parents with
    /// no original scored are left out.
    Original,
}

impl PerParent {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "max" => PerParent::Max,
            "original" => PerParent::Original,
            other => bail!("unknown --per-parent `{other}` (use max or original)"),
        })
    }
}

/// Scores reduced to one per independent unit.
#[derive(Clone, Debug, PartialEq)]
pub struct Units {
    pub scores: Vec<f32>,
    /// Items that went in.
    pub items: usize,
    /// Parents left out because no original of theirs was scored
    /// ([`PerParent::Original`] only).
    pub dropped: usize,
}

/// One score per parent. `parents[i]` is the parent of item `i` (`None`: the
/// item is its own parent) and `original[i]` says whether it is unprocessed.
/// Derivatives of one photo are one piece of evidence about the
/// false-positive rate, not several, so the binomial bound must count
/// parents.
pub fn one_per_parent(scores: &[f32], parents: &[Option<&str>], original: &[bool], how: PerParent) -> Units {
    assert!(scores.len() == parents.len() && scores.len() == original.len());
    let mut out = Vec::new();
    // Per parent: the highest score of any version, and of an original (if one was scored).
    let mut by_parent: BTreeMap<&str, (f32, Option<f32>)> = BTreeMap::new();
    let mut dropped = 0;
    for ((&s, p), &orig) in scores.iter().zip(parents).zip(original) {
        match p {
            None => match how {
                PerParent::Max => out.push(s),
                PerParent::Original if orig => out.push(s),
                PerParent::Original => dropped += 1,
            },
            Some(p) => {
                let e = by_parent.entry(p).or_insert((f32::NEG_INFINITY, None));
                e.0 = e.0.max(s);
                if orig {
                    e.1 = Some(e.1.map_or(s, |o: f32| o.max(s)));
                }
            }
        }
    }
    for (any, orig) in by_parent.into_values() {
        match (how, orig) {
            (PerParent::Max, _) => out.push(any),
            (PerParent::Original, Some(o)) => out.push(o),
            (PerParent::Original, None) => dropped += 1,
        }
    }
    Units { scores: out, items: scores.len(), dropped }
}

/// A cutoff from the order-statistic rule and what it does on its own units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundCutoff {
    /// Flag strictly above this.
    pub cutoff: f32,
    /// Units above it.
    pub flagged: usize,
    pub n: usize,
    /// The one-sided Clopper–Pearson upper bound on the false-positive rate.
    pub upper: f64,
}

/// The loosest cutoff on `units` (one score per independent authentic unit)
/// whose one-sided Clopper–Pearson upper bound on the false-positive rate is
/// at most `target_fpr` at `confidence`. Fails, saying how many units are
/// needed, when even flagging none of them cannot show the target.
pub fn bound_cutoff(units: &[f32], target_fpr: f64, confidence: f64) -> Result<BoundCutoff> {
    if !(target_fpr > 0.0 && target_fpr < 1.0) || !(confidence > 0.0 && confidence < 1.0) {
        bail!("the target false-positive rate and the confidence are fractions strictly between 0 and 1 (got {target_fpr} and {confidence})");
    }
    let Some((cutoff, flagged)) = order_statistic_cutoff(units, target_fpr, confidence) else {
        bail!(
            "too few independent authentic items for the bound rule: showing a false-positive rate of at most {}% at {}% confidence takes at least {} with none flagged, and these scores hold {}",
            100.0 * target_fpr,
            100.0 * confidence,
            needed_with_no_flags(target_fpr, confidence),
            units.len()
        );
    };
    Ok(BoundCutoff { cutoff, flagged, n: units.len(), upper: clopper_pearson_upper(flagged, units.len(), confidence) })
}

/// What a cutoff from any rule achieves on `units`: how many lie strictly
/// above it, of how many, and the one-sided Clopper–Pearson upper bound.
pub fn achieved(units: &[f32], cutoff: f32, confidence: f64) -> (usize, usize, f64) {
    let flagged = units.iter().filter(|&&x| x > cutoff).count();
    (flagged, units.len(), clopper_pearson_upper(flagged, units.len(), confidence))
}

/// Whether a stratum's false-positive rate at a frozen cutoff is within a budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transfer {
    /// The upper bound is within the budget: shown to hold here.
    Within,
    /// The lower bound is over the budget: shown not to hold here.
    Exceeds,
    /// Neither: too few independent units to tell.
    Unresolved,
}

/// One stratum's false-positive count at a frozen cutoff, with one-sided
/// Clopper–Pearson bounds at `confidence` and the verdict against `budget`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stratum {
    pub flagged: usize,
    pub n: usize,
    pub lower: f64,
    pub upper: f64,
    pub status: Transfer,
}

/// Judge `flagged` of `n` independent authentic units against a
/// false-positive `budget`. Each bound is one-sided at `confidence`, so
/// "within" and "exceeds" are each a claim at that confidence, and anything
/// between them is left unresolved rather than read as a pass.
pub fn transfer(flagged: usize, n: usize, budget: f64, confidence: f64) -> Stratum {
    let (lower, upper) = (clopper_pearson_lower(flagged, n, confidence), clopper_pearson_upper(flagged, n, confidence));
    let status = if n > 0 && upper <= budget {
        Transfer::Within
    } else if n > 0 && lower > budget {
        Transfer::Exceeds
    } else {
        Transfer::Unresolved
    };
    Stratum { flagged, n, lower, upper, status }
}

/// Count flagged units: items when `clusters` is `None`, otherwise whole
/// clusters (ids from [`crate::metrics::cluster_ids`]), a cluster being
/// flagged when any of its items is. Returns `(flagged, units)`.
pub fn flagged_units(flags: &[bool], clusters: Option<&[usize]>) -> (usize, usize) {
    match clusters {
        None => (flags.iter().filter(|&&f| f).count(), flags.len()),
        Some(c) => {
            assert_eq!(c.len(), flags.len());
            let mut any: BTreeMap<usize, bool> = BTreeMap::new();
            for (&k, &f) in c.iter().zip(flags) {
                *any.entry(k).or_default() |= f;
            }
            (any.values().filter(|&&f| f).count(), any.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{empirical_cutoff, rates_at};

    #[test]
    fn no_flags_among_n_gives_the_closed_form_bound() {
        // 0 flagged of n: the bound is 1 − 0.05^(1/n).
        for n in [1usize, 10, 299, 2995, 3000, 30_000] {
            let want = 1.0 - 0.05f64.powf(1.0 / n as f64);
            assert!((clopper_pearson_upper(0, n, 0.95) - want).abs() < 1e-15, "{n}");
        }
        // About 3,000 independent items for 0.10% at 95%: 2,995 exactly.
        assert_eq!(needed_with_no_flags(0.001, 0.95), 2995);
        assert_eq!(needed_with_no_flags(0.01, 0.95), 299);
        assert_eq!(needed_with_no_flags(0.01, 0.99), 459);
        assert_eq!(needed_with_no_flags(0.5, 0.5), 1);
        for (t, c) in [(0.001, 0.95), (0.01, 0.95), (0.005, 0.9), (0.015, 0.99)] {
            let n = needed_with_no_flags(t, c);
            assert!(clopper_pearson_upper(0, n, c) <= t && clopper_pearson_upper(0, n - 1, c) > t, "{t} {c} {n}");
        }
        assert_eq!(needed_with_no_flags(0.0, 0.95), usize::MAX);
    }

    #[test]
    fn the_bound_rule_fails_with_the_count_it_needs() {
        let few: Vec<f32> = (0..2000).map(|i| i as f32).collect();
        let e = bound_cutoff(&few, 0.001, 0.95).unwrap_err().to_string();
        assert!(e.contains("at least 2995 with none flagged") && e.contains("these scores hold 2000") && e.contains("0.1%") && e.contains("95%"), "{e}");
        assert!(bound_cutoff(&few, 0.0, 0.95).is_err() && bound_cutoff(&few, 0.01, 1.0).is_err());
        // With exactly enough, the cutoff is the top score and nothing is flagged.
        let enough: Vec<f32> = (0..2995).map(|i| i as f32).collect();
        let b = bound_cutoff(&enough, 0.001, 0.95).unwrap();
        assert_eq!((b.cutoff, b.flagged, b.n), (2994.0, 0, 2995));
        assert!(b.upper <= 0.001);
    }

    #[test]
    fn the_bound_cutoff_is_never_looser_than_the_quantile_and_meets_its_target() {
        let real: Vec<f32> = (0..10_000).map(|i| (i as f32 * 0.37).sin() * 5.0).collect();
        for target in [0.005, 0.01, 0.015, 0.05] {
            let b = bound_cutoff(&real, target, 0.95).unwrap();
            assert!(b.cutoff >= crate::metrics::fpr_threshold(&real, target), "{target}");
            assert!(b.upper <= target && clopper_pearson_upper(b.flagged + 1, b.n, 0.95) > target, "{target}");
            assert_eq!(achieved(&real, b.cutoff, 0.95), (b.flagged, b.n, b.upper));
            // The plain quantile flags its target share here, so its bound is over the target.
            assert!(achieved(&real, crate::metrics::fpr_threshold(&real, target), 0.95).2 > target);
        }
    }

    #[test]
    fn derivatives_count_once_per_parent() {
        // Three parents with an original and two derivatives each, and one item with no parent.
        let scores = [0.1, 0.9, 0.2, 0.3, 0.2, 0.1, 0.5, 0.6, 0.7, 0.4];
        let parents = [Some("a"), Some("a"), Some("a"), Some("b"), Some("b"), Some("b"), Some("c"), Some("c"), Some("c"), None];
        let original = [true, false, false, true, false, false, false, false, false, true];
        let mut u = one_per_parent(&scores, &parents, &original, PerParent::Max);
        u.scores.sort_by(|a, b| a.total_cmp(b));
        // The highest score of each parent, and the lone item.
        assert_eq!((u.scores.as_slice(), u.items, u.dropped), ([0.3, 0.4, 0.7, 0.9].as_slice(), 10, 0));
        let mut u = one_per_parent(&scores, &parents, &original, PerParent::Original);
        u.scores.sort_by(|a, b| a.total_cmp(b));
        // Parent c has no original scored and is left out.
        assert_eq!((u.scores.as_slice(), u.dropped), ([0.1, 0.3, 0.4].as_slice(), 1));
        assert!(PerParent::parse("max").is_ok() && PerParent::parse("original").is_ok() && PerParent::parse("mean").is_err());
    }

    #[test]
    fn repeats_of_a_parent_do_not_buy_a_tighter_bound() {
        // 300 parents, ten copies each, none flagged: 3,000 items look like enough for 0.1%, 300 parents are not.
        let scores = vec![0.0f32; 3000];
        let names: Vec<String> = (0..3000).map(|i| format!("p{}", i / 10)).collect();
        let parents: Vec<Option<&str>> = names.iter().map(|n| Some(n.as_str())).collect();
        let original: Vec<bool> = (0..3000).map(|i| i % 10 == 0).collect();
        assert!(bound_cutoff(&scores, 0.001, 0.95).is_ok());
        let units = one_per_parent(&scores, &parents, &original, PerParent::Max);
        assert_eq!(units.scores.len(), 300);
        let e = bound_cutoff(&units.scores, 0.001, 0.95).unwrap_err().to_string();
        assert!(e.contains("these scores hold 300"), "{e}");
        assert!(bound_cutoff(&units.scores, 0.01, 0.95).is_ok());
    }

    #[test]
    fn transfer_is_within_exceeds_or_unresolved() {
        // 0 of 3,000 shows a 0.1% budget; 0 of 500 cannot.
        assert_eq!(transfer(0, 3000, 0.001, 0.95).status, Transfer::Within);
        assert_eq!(transfer(0, 500, 0.001, 0.95).status, Transfer::Unresolved);
        // 63 of 600 is over a 1% budget beyond doubt; 8 of 600 (1.33%) is not resolved either way.
        let s = transfer(63, 600, 0.01, 0.95);
        assert_eq!(s.status, Transfer::Exceeds);
        assert!(s.lower > 0.08 && s.upper < 0.13, "{s:?}");
        assert_eq!(transfer(8, 600, 0.01, 0.95).status, Transfer::Unresolved);
        assert_eq!(transfer(1, 600, 0.01, 0.95).status, Transfer::Within);
        assert_eq!(transfer(0, 0, 0.01, 0.95).status, Transfer::Unresolved);
    }

    #[test]
    fn flagged_units_count_a_cluster_once() {
        let flags = [true, true, false, false, true, false];
        assert_eq!(flagged_units(&flags, None), (3, 6));
        // Clusters {0,1}, {2,3}, {4}, {5}: two are flagged.
        assert_eq!(flagged_units(&flags, Some(&[0, 0, 1, 1, 2, 3])), (2, 4));
        assert_eq!(flagged_units(&[], Some(&[])), (0, 0));
    }

    /// `(score, label, distorted)` of every row of an archived NTIRE score file.
    fn ntire(split: &str) -> Vec<(f32, u8, bool)> {
        use std::io::Read;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("docs/results/ntire-2026-{split}/scores.tsv.gz"));
        let mut text = String::new();
        flate2::read::MultiGzDecoder::new(std::fs::File::open(&path).unwrap()).read_to_string(&mut text).unwrap();
        text.lines().skip(1).map(|l| l.split('\t').collect::<Vec<_>>()).map(|f| (f[4].parse().unwrap(), f[1].parse().unwrap(), f[3] == "ntire-distorted")).collect()
    }

    /// The regression case of docs/EVIDENCE.md ("NTIRE threshold transfer"):
    /// a low false-positive rate on the calibration cohort says little about
    /// another one. Diagnostic data, not a blind result.
    #[test]
    fn the_archived_ntire_cutoffs_do_not_transfer() {
        let (hard, public) = (ntire("hard-validation"), ntire("public-test"));
        assert_eq!((hard.len(), public.len()), (2500, 2500));
        let cohort = |rows: &[(f32, u8, bool)], distorted: bool| -> (Vec<f32>, Vec<u8>) { rows.iter().filter(|r| r.2 == distorted).map(|r| (r.0, r.1)).unzip() };
        let (hard_clean, hard_clean_l) = cohort(&hard, false);
        let (hard_dist, hard_dist_l) = cohort(&hard, true);
        let (pub_dist, pub_dist_l) = cohort(&public, true);
        let count = |s: &[f32], l: &[u8], label: u8, c: f32| s.iter().zip(l).filter(|(&s, &x)| x == label && s > c).count();

        // The clean validation cutoff flags 0.96% of its own authentic images ...
        let clean = empirical_cutoff(&hard_clean, &hard_clean_l, 0.01).unwrap();
        assert!((clean - 0.941085).abs() < 1e-6, "{clean}");
        assert_eq!((count(&hard_clean, &hard_clean_l, 0, clean), hard_clean_l.iter().filter(|&&l| l == 0).count()), (6, 625));
        // ... and 63 of 600 (10.50%) of the public distorted ones: shown to exceed a 1% budget.
        let fp = count(&pub_dist, &pub_dist_l, 0, clean);
        assert_eq!((fp, pub_dist_l.iter().filter(|&&l| l == 0).count()), (63, 600));
        assert_eq!(transfer(fp, 600, 0.01, 0.95).status, Transfer::Exceeds);

        // The distorted validation cutoff transfers at 2.17% and 4.77% recall.
        let dist = empirical_cutoff(&hard_dist, &hard_dist_l, 0.01).unwrap();
        assert!((dist - 0.996498).abs() < 1e-6, "{dist}");
        assert_eq!((count(&pub_dist, &pub_dist_l, 0, dist), count(&pub_dist, &pub_dist_l, 1, dist), pub_dist_l.iter().filter(|&&l| l == 1).count()), (13, 31, 650));
        let (fpr, recall) = rates_at(&pub_dist, &pub_dist_l, dist);
        assert!((fpr - 0.0217).abs() < 5e-5 && (recall - 0.0477).abs() < 5e-5, "{fpr} {recall}");
        // Even at that cost in recall, 13 of 600 is shown to exceed a 1% budget.
        assert_eq!(transfer(13, 600, 0.01, 0.95).status, Transfer::Exceeds);

        // The order-statistic rule on the same 625 clean authentic images is stricter than the
        // quantile, and still does not transfer: a bound on one cohort is not a bound on another.
        let reals: Vec<f32> = hard_clean.iter().zip(&hard_clean_l).filter(|(_, &l)| l == 0).map(|(&s, _)| s).collect();
        let b = bound_cutoff(&reals, 0.01, 0.95).unwrap();
        assert!(b.cutoff > clean && b.flagged == 1 && b.upper <= 0.01, "{b:?}");
        let fp = count(&pub_dist, &pub_dist_l, 0, b.cutoff);
        assert_eq!(transfer(fp, 600, 0.01, 0.95).status, Transfer::Exceeds, "{fp} of 600");
        // 0.1% cannot be shown from 625 images at all.
        assert!(bound_cutoff(&reals, 0.001, 0.95).is_err());
    }
}
