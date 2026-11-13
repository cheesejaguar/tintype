# NTIRE 2026 Public Test

Same 2,500 released JPEG inputs for both detectors. Each score table uses the default threshold of 0.5. Recall at 1% FPR ranks scores with a threshold selected from that cohort's authentic images.

**ntire-clean** (650 AI, 600 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.911 (0.883–0.935) | 0.662 (0.624–0.696) | 0.766 | 7.00% | 0.870 | 0.229 |
| Community Forensics | 0.911 (0.865–0.954) | 0.222 (0.190–0.252) | 0.356 | 2.33% | 0.773 | 0.154 |

**ntire-distorted** (650 AI, 600 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.652 (0.609–0.697) | 0.448 (0.411–0.485) | 0.531 | 25.83% | 0.636 | 0.022 |
| Community Forensics | 0.571 (0.517–0.632) | 0.228 (0.196–0.260) | 0.326 | 18.50% | 0.577 | 0.008 |

**ntire-all** (1300 AI, 1200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.785 (0.757–0.814) | 0.555 (0.528–0.582) | 0.650 | 16.42% | 0.755 | 0.045 |
| Community Forensics | 0.700 (0.656–0.744) | 0.225 (0.201–0.249) | 0.340 | 10.42% | 0.662 | 0.009 |

## At Tintype's shipped false-positive count

| Set | Tintype: real items flagged | Tintype: precision / recall | Community Forensics at that rate: precision / recall |
|---|---|---|---|
| ntire-clean | 42 of 600 (7.00%) | 0.911 / 0.662 | 0.848 / 0.360 |
| ntire-distorted | 155 of 600 (25.83%) | 0.652 / 0.448 | 0.588 / 0.340 |
| ntire-all | 197 of 1200 (16.42%) | 0.785 / 0.555 | 0.703 / 0.358 |
