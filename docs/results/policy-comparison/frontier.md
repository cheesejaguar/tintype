# Policy comparison on the frontier archive (exploratory)

These archived suites were used to develop the detector. Cutoffs are chosen on the cohort they are reported on: a ranking comparison, not a transfer result and not a decision about what ships. Routed shares are measured workload; end-to-end savings depend on the cohort and the hardware.

## original: 745 photos, 261 generated, 484 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 4 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/745 (0.00%) | 137/261 (52.49%) +0.00 | 181/261 (69.35%) +0.00 | 205, 9 (78.54%, 1.86%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 540/745 (72.48%) | 125/261 (47.89%) -4.60 | 177/261 (67.82%) -1.53 | 204, 13 (78.16%, 2.69%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 174/745 (23.36%) | 125/261 (47.89%) -4.60 | 177/261 (67.82%) -1.53 | 204, 13 (78.16%, 2.69%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 56 of 261 generated photos (Community Forensics flags 11 of them) and flags 9 of 484 authentic photos (Community Forensics clears 6).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 540/745 (72.48%) | 56 | 0 | 0 | 6 | 3 |
| -3 < tintype logit < 0.5 | 174/745 (23.36%) | 44 | 12 | 3 | 6 | 3 |
| |tintype logit| <= 1 (sweep) | 55/745 (7.38%) | 18 | 38 | 6 | 4 | 1 |
| |tintype logit| <= 2 (sweep) | 142/745 (19.06%) | 35 | 21 | 3 | 1 | 0 |
| |tintype logit| <= 3 (sweep) | 262/745 (35.17%) | 44 | 12 | 3 | 0 | 0 |
| |tintype logit| <= 4 (sweep) | 360/745 (48.32%) | 50 | 6 | 1 | 0 | 0 |
| |tintype logit| <= 6 (sweep) | 692/745 (92.89%) | 53 | 3 | 1 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 743/745 (99.73%) | 56 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.44, which routes 96/745 (12.89%); 90% of the misses needs |logit| <= 4.93, which routes 553/745 (74.23%); 100% of the misses needs |logit| <= 6.91, which routes 736/745 (98.79%).

## jpeg: 745 photos, 261 generated, 484 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 4 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/745 (0.00%) | 99/261 (37.93%) +0.00 | 139/261 (53.26%) +0.00 | 116, 1 (44.44%, 0.21%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 633/745 (84.97%) | 47/261 (18.01%) -19.92 | 116/261 (44.44%) -8.81 | 110, 2 (42.15%, 0.41%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 201/745 (26.98%) | 47/261 (18.01%) -19.92 | 116/261 (44.44%) -8.81 | 110, 2 (42.15%, 0.41%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 145 of 261 generated photos (Community Forensics flags 7 of them) and flags 1 of 484 authentic photos (Community Forensics clears 1).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 633/745 (84.97%) | 145 | 0 | 0 | 1 | 1 |
| -3 < tintype logit < 0.5 | 201/745 (26.98%) | 111 | 34 | 0 | 1 | 1 |
| |tintype logit| <= 1 (sweep) | 63/745 (8.46%) | 37 | 108 | 5 | 0 | 0 |
| |tintype logit| <= 2 (sweep) | 148/745 (19.87%) | 83 | 62 | 2 | 0 | 0 |
| |tintype logit| <= 3 (sweep) | 265/745 (35.57%) | 111 | 34 | 0 | 0 | 0 |
| |tintype logit| <= 4 (sweep) | 344/745 (46.17%) | 122 | 23 | 0 | 0 | 0 |
| |tintype logit| <= 6 (sweep) | 674/745 (90.47%) | 142 | 3 | 0 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 745/745 (100.00%) | 145 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.70, which routes 125/745 (16.78%); 90% of the misses needs |logit| <= 4.47, which routes 452/745 (60.67%); 100% of the misses needs |logit| <= 6.38, which routes 711/745 (95.44%).

## jpeg-960: 745 photos, 261 generated, 484 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 4 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/745 (0.00%) | 9/261 (3.45%) +0.00 | 73/261 (27.97%) +0.00 | 101, 10 (38.70%, 2.07%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 648/745 (86.98%) | 10/261 (3.83%) +0.38 | 67/261 (25.67%) -2.30 | 95, 12 (36.40%, 2.48%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 284/745 (38.12%) | 10/261 (3.83%) +0.38 | 67/261 (25.67%) -2.30 | 95, 11 (36.40%, 2.27%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 160 of 261 generated photos (Community Forensics flags 10 of them) and flags 10 of 484 authentic photos (Community Forensics clears 7).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 648/745 (86.98%) | 160 | 0 | 0 | 8 | 6 |
| -3 < tintype logit < 0.5 | 284/745 (38.12%) | 131 | 29 | 0 | 8 | 6 |
| |tintype logit| <= 1 (sweep) | 93/745 (12.48%) | 50 | 110 | 5 | 6 | 5 |
| |tintype logit| <= 2 (sweep) | 210/745 (28.19%) | 103 | 57 | 0 | 2 | 2 |
| |tintype logit| <= 3 (sweep) | 354/745 (47.52%) | 131 | 29 | 0 | 1 | 1 |
| |tintype logit| <= 4 (sweep) | 458/745 (61.48%) | 146 | 14 | 0 | 1 | 1 |
| |tintype logit| <= 6 (sweep) | 712/745 (95.57%) | 160 | 0 | 0 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 745/745 (100.00%) | 160 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.43, which routes 144/745 (19.33%); 90% of the misses needs |logit| <= 3.67, which routes 424/745 (56.91%); 100% of the misses needs |logit| <= 5.68, which routes 694/745 (93.15%).
