//! The opt-in "needs review" outcome: a result is marked for a person to
//! look at when the evidence sits close to the decision threshold, or when
//! the views that were scored do not agree with one another.
//!
//! It never changes a verdict, a score or an exit status, and it costs no
//! inference: it reads the logits the detector has already computed. With no
//! band (no `--review-margin` / `--review-disagreement` and no `review` in
//! the model's spec) nothing here runs and the output has no `review` keys.
//!
//! The band is a policy, fitted on calibration data after everything else
//! about the system is fixed (docs/runbooks/calibration-abstention.md). An
//! AI-generated item sent to review is a miss as far as automatic detection
//! goes, and is counted as one ([`account`]).
use clap::Args;
use serde::{Deserialize, Serialize};

/// When a result is marked as needing review. Stored in a model's spec as
/// `review` (for the default photo profile, and inside `video` for clips),
/// or given on the command line.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Review {
    /// Half-width of the band around the decision threshold, in calibrated
    /// logits (the logit of the reported score, so the threshold is 0): a
    /// result whose calibrated logit is strictly inside `(-margin, margin)`
    /// needs review. 0 marks nothing.
    pub margin: f32,
    /// Also mark a result when at least this share of the views that were
    /// scored (frames, for a clip) fall on the other side of the threshold
    /// from their mean: see [`disagreement`]. Absent means the margin alone
    /// decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_disagreement: Option<f32>,
}

/// Why a result needs review. When both apply the margin is reported: it is
/// the criterion with a calibrated meaning, and the one the band is fitted on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reason {
    Margin,
    ViewDisagreement,
}

impl Reason {
    /// The value of `review_reason` in JSON output.
    pub fn name(self) -> &'static str {
        match self {
            Reason::Margin => "margin",
            Reason::ViewDisagreement => "view_disagreement",
        }
    }
}

/// The share of `logits` (one per view looked at, or per frame of a clip)
/// that fall on the other side of `threshold` from their mean, which is the
/// side the first-stage verdict is on. 0 when all agree, and 0 with fewer
/// than two: one view cannot disagree with itself, so an image that early
/// exit decided on its first view is never marked for this reason.
pub fn disagreement(logits: &[f32], threshold: f32) -> f32 {
    if logits.len() < 2 {
        return 0.0;
    }
    let flagged = logits.iter().sum::<f32>() / logits.len() as f32 > threshold;
    logits.iter().filter(|&&z| (z > threshold) != flagged).count() as f32 / logits.len() as f32
}

/// The logit of a score: what the margin is measured on when only the score
/// is known (a second-opinion blend, or a row of a score file). Scores of
/// exactly 0 or 1 give an infinite logit, which is outside every band.
pub fn logit_of(score: f32) -> f32 {
    (score as f64 / (1.0 - score as f64)).ln() as f32
}

impl Review {
    /// Whether, and why, a result needs review. `calibrated` is the logit of
    /// its reported score (0 at the threshold); `logits` are the first
    /// stage's view or frame logits that were looked at, and `threshold` is
    /// the first stage's threshold on their mean.
    pub fn reason(&self, calibrated: f32, logits: &[f32], threshold: f32) -> Option<Reason> {
        if calibrated.abs() < self.margin {
            return Some(Reason::Margin);
        }
        match self.view_disagreement {
            Some(share) if logits.len() >= 2 && disagreement(logits, threshold) >= share => Some(Reason::ViewDisagreement),
            _ => None,
        }
    }
}

/// Add the review fields to a JSON result: `review` always, `review_reason`
/// when it is true. Called only when a band is in use, so that results
/// without one keep exactly the keys they had.
pub fn annotate(object: &mut serde_json::Value, reason: Option<Reason>) {
    object["review"] = reason.is_some().into();
    if let Some(r) = reason {
        object["review_reason"] = r.name().into();
    }
}

fn parse_margin(value: &str) -> Result<f32, String> {
    value.parse::<f32>().ok().filter(|m| m.is_finite() && *m >= 0.0).ok_or_else(|| "expected a finite, non-negative margin in calibrated logits".into())
}

fn parse_share(value: &str) -> Result<f32, String> {
    value.parse::<f32>().ok().filter(|s| *s > 0.0 && *s <= 1.0).ok_or_else(|| "expected a share of views above 0 and at most 1".into())
}

/// The command-line form of [`Review`], shared by `detect`, `batch` and
/// `serve`. Off unless one of the flags is given or the model's spec has a
/// band.
#[derive(Args, Clone, Copy, Debug, Default)]
pub struct ReviewArgs {
    /// Mark results whose calibrated logit is within this distance of the
    /// threshold as needing review (adds `review` to the output; verdicts
    /// and exit status do not change).
    #[arg(long, value_name = "CALIBRATED_LOGIT", value_parser = parse_margin)]
    pub review_margin: Option<f32>,
    /// Also mark results when at least this share (above 0, at most 1) of
    /// the views or frames scored fall on the other side of the threshold.
    #[arg(long, value_name = "SHARE", value_parser = parse_share)]
    pub review_disagreement: Option<f32>,
}

impl ReviewArgs {
    /// The band in use: the flags, each falling back to the band stored with
    /// the model for the profile being run (`stored`). `None` when neither
    /// gives one, which is today's output.
    pub fn resolve(&self, stored: Option<Review>) -> Option<Review> {
        if self.review_margin.is_none() && self.review_disagreement.is_none() {
            return stored;
        }
        Some(Review {
            margin: self.review_margin.or(stored.map(|s| s.margin)).unwrap_or(0.0),
            view_disagreement: self.review_disagreement.or(stored.and_then(|s| s.view_disagreement)),
        })
    }
}

/// One scored, labelled item as the accounting sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outcome {
    /// 1 = AI-generated.
    pub label: u8,
    /// The verdict, which the band does not change.
    pub flagged: bool,
    pub review: Option<Reason>,
}

/// What a review band costs and buys on labelled items, as the frozen
/// evaluation counts it (docs/runbooks/frozen-eval.md): an item sent to
/// review gets no automatic verdict, so a reviewed AI item is a miss and a
/// reviewed real item is neither a false alarm nor cleared.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Accounting {
    pub n_real: usize,
    pub n_fake: usize,
    pub real_reviewed: usize,
    pub fake_reviewed: usize,
    /// Reviewed for the margin; the rest of the reviewed are for disagreement.
    pub by_margin: usize,
    /// Real items flagged and not sent to review: automatic false alarms.
    pub real_flagged_unreviewed: usize,
    /// AI items flagged and not sent to review: automatic detections.
    pub fake_flagged_unreviewed: usize,
    /// AI items flagged, reviewed or not: detections without the band.
    pub fake_flagged: usize,
    pub real_flagged: usize,
}

fn share(k: usize, n: usize) -> f64 {
    if n > 0 {
        k as f64 / n as f64
    } else {
        f64::NAN
    }
}

impl Accounting {
    /// The share of items that get an automatic verdict.
    pub fn coverage(&self) -> f64 {
        1.0 - share(self.real_reviewed + self.fake_reviewed, self.n_real + self.n_fake)
    }
    /// The share of real items sent to review.
    pub fn real_review_rate(&self) -> f64 {
        share(self.real_reviewed, self.n_real)
    }
    /// The share of AI items sent to review.
    pub fn fake_review_rate(&self) -> f64 {
        share(self.fake_reviewed, self.n_fake)
    }
    /// Recall with every reviewed AI item counted as a miss.
    pub fn recall(&self) -> f64 {
        share(self.fake_flagged_unreviewed, self.n_fake)
    }
    /// Recall of the verdicts alone, as if there were no band.
    pub fn recall_without_band(&self) -> f64 {
        share(self.fake_flagged, self.n_fake)
    }
    /// Real items flagged without review, over all real items.
    pub fn fpr(&self) -> f64 {
        share(self.real_flagged_unreviewed, self.n_real)
    }
    /// The share of a stream sent to review when `prevalence` of it is
    /// AI-generated (undefined without items of both classes).
    pub fn review_rate_at(&self, prevalence: f64) -> f64 {
        self.fake_review_rate() * prevalence + self.real_review_rate() * (1.0 - prevalence)
    }
}

/// Count what a band does on labelled items.
pub fn account(outcomes: &[Outcome]) -> Accounting {
    let mut a = Accounting::default();
    for o in outcomes {
        let reviewed = o.review.is_some();
        a.by_margin += (o.review == Some(Reason::Margin)) as usize;
        if o.label == 1 {
            a.n_fake += 1;
            a.fake_reviewed += reviewed as usize;
            a.fake_flagged += o.flagged as usize;
            a.fake_flagged_unreviewed += (o.flagged && !reviewed) as usize;
        } else {
            a.n_real += 1;
            a.real_reviewed += reviewed as usize;
            a.real_flagged += o.flagged as usize;
            a.real_flagged_unreviewed += (o.flagged && !reviewed) as usize;
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_margin_band_is_open_and_zero_marks_nothing() {
        let r = Review { margin: 1.0, view_disagreement: None };
        assert_eq!(r.reason(0.99, &[], 0.0), Some(Reason::Margin));
        assert_eq!(r.reason(-0.99, &[], 0.0), Some(Reason::Margin));
        assert_eq!(r.reason(0.0, &[], 0.0), Some(Reason::Margin));
        // The edge of the band is outside it, on both sides.
        assert_eq!(r.reason(1.0, &[], 0.0), None);
        assert_eq!(r.reason(-1.0, &[], 0.0), None);
        assert_eq!(r.reason(f32::INFINITY, &[], 0.0), None);
        let off = Review { margin: 0.0, view_disagreement: None };
        assert_eq!(off.reason(0.0, &[1.0, -1.0], 0.0), None);
    }

    #[test]
    fn disagreement_is_the_share_of_views_against_the_verdict() {
        // Mean 2.25 is over the threshold of 1; one view of four is under it.
        assert_eq!(disagreement(&[4.0, 3.0, 2.0, 0.0], 1.0), 0.25);
        // Mean -1 is under; two of four are over.
        assert_eq!(disagreement(&[2.0, 2.0, -4.0, -4.0], 1.0), 0.5);
        assert_eq!(disagreement(&[5.0, 6.0, 7.0], 1.0), 0.0);
        // One view, or none, cannot disagree.
        assert_eq!(disagreement(&[0.3], 1.0), 0.0);
        assert_eq!(disagreement(&[], 1.0), 0.0);
        // A view exactly on the threshold is on the "not flagged" side, as a mean there is.
        assert_eq!(disagreement(&[1.0, 5.0], 1.0), 0.5);
    }

    #[test]
    fn the_margin_takes_precedence_and_one_view_never_disagrees() {
        let r = Review { margin: 1.0, view_disagreement: Some(0.5) };
        // Inside the band and split: reported as the margin.
        assert_eq!(r.reason(0.2, &[3.0, -2.0], 0.0), Some(Reason::Margin));
        // Outside the band and split.
        assert_eq!(r.reason(1.5, &[5.0, -2.0], 0.0), Some(Reason::ViewDisagreement));
        // Outside the band, one dissenting view of three is under a half.
        assert_eq!(r.reason(1.5, &[5.0, 4.0, -2.0], 0.0), None);
        assert_eq!(r.reason(4.0, &[6.0], 0.0), None);
        assert_eq!(Reason::Margin.name(), "margin");
        assert_eq!(Reason::ViewDisagreement.name(), "view_disagreement");
    }

    #[test]
    fn logit_of_a_score_inverts_the_sigmoid() {
        for z in [-6.0f32, -1.0, 0.0, 0.5, 4.0] {
            assert!((logit_of(crate::metrics::sigmoid(z)) - z).abs() < 1e-3, "{z}");
        }
        assert!(logit_of(1.0).is_infinite() && logit_of(0.0).is_infinite());
    }

    #[test]
    fn flags_override_the_stored_band_field_by_field() {
        let stored = Review { margin: 2.0, view_disagreement: Some(0.5) };
        let none = ReviewArgs::default();
        assert_eq!(none.resolve(None), None);
        assert_eq!(none.resolve(Some(stored)), Some(stored));
        let margin = ReviewArgs { review_margin: Some(0.5), review_disagreement: None };
        assert_eq!(margin.resolve(None), Some(Review { margin: 0.5, view_disagreement: None }));
        assert_eq!(margin.resolve(Some(stored)), Some(Review { margin: 0.5, view_disagreement: Some(0.5) }));
        let share = ReviewArgs { review_margin: None, review_disagreement: Some(0.34) };
        assert_eq!(share.resolve(None), Some(Review { margin: 0.0, view_disagreement: Some(0.34) }));
        assert!(parse_margin("-1").is_err() && parse_margin("nan").is_err() && parse_margin("0").is_ok());
        assert!(parse_share("0").is_err() && parse_share("1.5").is_err() && parse_share("0.5").is_ok());
    }

    #[test]
    fn annotate_adds_only_the_review_keys() {
        let mut o = serde_json::json!({"verdict": "ai-generated"});
        annotate(&mut o, None);
        assert_eq!(o, serde_json::json!({"verdict": "ai-generated", "review": false}));
        annotate(&mut o, Some(Reason::ViewDisagreement));
        assert_eq!(o, serde_json::json!({"verdict": "ai-generated", "review": true, "review_reason": "view_disagreement"}));
    }

    #[test]
    fn stored_bands_parse_with_and_without_the_disagreement_share() {
        let r: Review = serde_json::from_str(r#"{"margin":1.5}"#).unwrap();
        assert_eq!(r, Review { margin: 1.5, view_disagreement: None });
        assert_eq!(serde_json::to_string(&r).unwrap(), r#"{"margin":1.5}"#);
        let r: Review = serde_json::from_str(r#"{"margin":1.5,"view_disagreement":0.5}"#).unwrap();
        assert_eq!(r.view_disagreement, Some(0.5));
    }

    #[test]
    fn reviewed_ai_items_count_as_misses() {
        let o = |label, flagged, review| Outcome { label, flagged, review };
        let m = Some(Reason::Margin);
        let d = Some(Reason::ViewDisagreement);
        // 4 AI: caught, caught but reviewed, missed, missed and reviewed. 4 real: clear, flagged, flagged but reviewed, reviewed.
        let a = account(&[o(1, true, None), o(1, true, m), o(1, false, None), o(1, false, d), o(0, false, None), o(0, true, None), o(0, true, m), o(0, false, m)]);
        assert_eq!((a.n_fake, a.n_real, a.fake_reviewed, a.real_reviewed, a.by_margin), (4, 4, 2, 2, 3));
        assert_eq!(a.coverage(), 0.5);
        assert_eq!(a.recall_without_band(), 0.5);
        // The reviewed detection no longer counts: one automatic detection of four AI items.
        assert_eq!(a.recall(), 0.25);
        // One real item is flagged without review, of four.
        assert_eq!(a.fpr(), 0.25);
        assert_eq!((a.fake_review_rate(), a.real_review_rate()), (0.5, 0.5));
        assert!((a.review_rate_at(0.01) - 0.5).abs() < 1e-12);
        // No band: full coverage, and recall is the verdicts' recall.
        let a = account(&[o(1, true, None), o(1, false, None), o(0, false, None)]);
        assert_eq!((a.coverage(), a.recall(), a.recall_without_band(), a.fpr()), (1.0, 0.5, 0.5, 0.0));
        // One class only: the other class's rates are undefined, and so is the stream's review rate.
        let a = account(&[o(1, true, m), o(1, true, None)]);
        assert!(a.fpr().is_nan() && a.real_review_rate().is_nan() && a.review_rate_at(0.1).is_nan());
        assert_eq!(a.recall(), 0.5);
    }
}
