# NTIRE 2026 Hard Validation

Same 2,500 released JPEG inputs for both detectors. Each score table uses the default threshold of 0.5. Recall at 1% FPR ranks scores with a threshold selected from that cohort's authentic images.

**ntire-clean** (625 AI, 625 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.921 (0.897–0.944) | 0.806 (0.775–0.837) | 0.860 | 6.88% | 0.938 | 0.518 |
| Community Forensics | 0.966 (0.943–0.984) | 0.496 (0.459–0.535) | 0.655 | 1.76% | 0.880 | 0.470 |

**ntire-distorted** (625 AI, 625 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.725 (0.682–0.772) | 0.435 (0.396–0.471) | 0.544 | 16.48% | 0.734 | 0.054 |
| Community Forensics | 0.664 (0.600–0.721) | 0.234 (0.200–0.265) | 0.346 | 11.84% | 0.641 | 0.038 |

**ntire-all** (1250 AI, 1250 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.842 (0.817–0.865) | 0.621 (0.591–0.648) | 0.715 | 11.68% | 0.839 | 0.122 |
| Community Forensics | 0.843 (0.810–0.875) | 0.365 (0.339–0.393) | 0.509 | 6.80% | 0.745 | 0.082 |

## At Tintype's shipped false-positive count

| Set | Tintype: real items flagged | Tintype: precision / recall | Community Forensics at that rate: precision / recall |
|---|---|---|---|
| ntire-clean | 43 of 625 (6.88%) | 0.921 / 0.806 | 0.899 / 0.610 |
| ntire-distorted | 103 of 625 (16.48%) | 0.725 / 0.435 | 0.663 / 0.325 |
| ntire-all | 146 of 1250 (11.68%) | 0.842 / 0.621 | 0.801 / 0.470 |
