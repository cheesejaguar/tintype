# Calibration and the review outcome on the frozen split

Raw output of the run of 2026-11-14 recorded in
[the calibration runbook](../../runbooks/calibration-abstention.md#on-the-frozen-split-the-released-photo-model):
the released photo model, no new scoring, arithmetic on the all-views
score files in [`../frozen-eval-incumbent/`](../frozen-eval-incumbent/).
The gate failed on cutoff transfer; nothing here is a shipped setting.

| File | What |
|---|---|
| `arm-a.json` | Spec of arm A: cutoffs from `calibrate --rule bound` with parents counted once (the highest score of a parent's versions), and the exit margins fitted on `fit` |
| `arm-a-shipped-margins.json` | Arm A's cutoffs with the shipped exit margins |
| `arm-b.json`, `arm-c.json` | Cutoffs from the bound rule on originals only, and from the default rule |
| `calibrate-exit-stored.md`, `calibrate-exit-fitted.md` | `refit-exit` on `calibrate`: the shipped margins at arm A's thresholds, and the margins fitted on `fit` |
| `calibrate-review-photo.md`, `calibrate-review-strict.md` | Review margins tabulated on `calibrate` at the 1% and 0.1% cutoffs |
| `test-exit-A-fitted.md`, `test-exit-A-shipped-margins.md` | `refit-exit` on `test` (gate 4) |
| `test-<arm>-<profile>.md` | `report --operating-points` on `test` with transfer tables by source and transform, and review accounting for arm A (gates 1 and 2) |
| `test-<arm>-<profile>-clean.md`, `-degraded.md` | The same on the unprocessed rows and on the processed copies |

The specs are copies of `models/tintype.json` with recalibrated profiles;
they are not released models. The decided score files are not archived:
`refit-exit --apply-out` rebuilds them from the all-views files and a spec.
