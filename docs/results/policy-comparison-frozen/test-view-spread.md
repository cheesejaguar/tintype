# Policy comparison

35542 photos scored by every system (commfor, tintype-views).

Split: 3759 fit, 12457 calibrate, 19326 test photos; 0 scored photos are in no set and are left out.

Fusion weights fitted on the fit set (3759 photos) only:

- tintype-views + Community Forensics, fitted for |tintype-views logit| <= 2, fit set: +1.2623 x tintype-views logit +0.4165 x commfor logit +1.3039 from 352 photos inside the gate; outside it +1.1629 x tintype-views logit +1.6611 from 3759
- tintype-views + Community Forensics, fitted for |tintype-views logit| <= 3, fit set: +1.3455 x tintype-views logit +0.4786 x commfor logit +1.7232 from 684 photos inside the gate; outside it +1.1629 x tintype-views logit +1.6611 from 3759
- tintype-views + Community Forensics, fitted full fusion, fit set: +1.1288 x tintype-views logit +0.5747 x commfor logit +1.9442 from 3759 photos

Cutoffs are frozen on the `calibrate` set (12457 photos, 5857 authentic).

## Test set: 19326 photos, 9600 generated, 9726 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 9 FP), points vs tintype-views | Recall at 1% FPR (<= 97 FP), points vs tintype-views | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| tintype-views | 0/19326 (0.00%) | 8056/9600 (83.92%) +0.00 | 8755/9600 (91.20%) +0.00 | 8717, 85 (90.80%, 0.87%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 10699/19326 (55.36%) | 8159/9600 (84.99%) +1.07 | 9122/9600 (95.02%) +3.82 | 9135, 110 (95.16%, 1.13%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 2193/19326 (11.35%) | 8142/9600 (84.81%) +0.90 | 9054/9600 (94.31%) +3.11 | 9050, 92 (94.27%, 0.95%) |
| Community Forensics alone | 0/19326 (0.00%) | 6922/9600 (72.10%) -11.81 | 8099/9600 (84.36%) -6.83 | 8306, 166 (86.52%, 1.71%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 1872/19326 (9.69%) | 8514/9600 (88.69%) +4.77 | 8949/9600 (93.22%) +2.02 | 8782, 33 (91.48%, 0.34%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 3460/19326 (17.90%) | 8554/9600 (89.10%) +5.19 | 8960/9600 (93.33%) +2.14 | 8793, 36 (91.59%, 0.37%) |
| tintype-views + Community Forensics, 50:50 full fusion | 19326/19326 (100.00%) | 8276/9600 (86.21%) +2.29 | 8967/9600 (93.41%) +2.21 | 8757, 41 (91.22%, 0.42%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 2 | 1872/19326 (9.69%) | 8694/9600 (90.56%) +6.65 | 9028/9600 (94.04%) +2.84 | 8976, 64 (93.50%, 0.66%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 3 | 3460/19326 (17.90%) | 8756/9600 (91.21%) +7.29 | 9109/9600 (94.89%) +3.69 | 9080, 82 (94.58%, 0.84%) |
| tintype-views + Community Forensics, fitted full fusion | 19326/19326 (100.00%) | 8717/9600 (90.80%) +6.89 | 9130/9600 (95.10%) +3.91 | 9138, 105 (95.19%, 1.08%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| tintype-views | 0.84512 | 31/9726 (0.32%) | 8420/9600 (87.71%) | -0.46940 | 138/9726 (1.42%) | 8851/9600 (92.20%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1.62600 | 28/9726 (0.29%) | 8567/9600 (89.24%) | 0.11401 | 104/9726 (1.07%) | 9127/9600 (95.07%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1.62600 | 24/9726 (0.25%) | 8503/9600 (88.57%) | -0.23711 | 104/9726 (1.07%) | 9061/9600 (94.39%) |
| Community Forensics alone | 4.40448 | 18/9726 (0.19%) | 7174/9600 (74.73%) | 1.08658 | 90/9726 (0.93%) | 8056/9600 (83.92%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 1.37517 | 15/9726 (0.15%) | 8664/9600 (90.25%) | -1.46972 | 100/9726 (1.03%) | 8957/9600 (93.30%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 1.50465 | 11/9726 (0.11%) | 8567/9600 (89.24%) | -0.92722 | 90/9726 (0.93%) | 8935/9600 (93.07%) |
| tintype-views + Community Forensics, 50:50 full fusion | 1.50465 | 11/9726 (0.11%) | 8302/9600 (86.48%) | -0.68999 | 84/9726 (0.86%) | 8929/9600 (93.01%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 2 | 1.73208 | 20/9726 (0.21%) | 8783/9600 (91.49%) | -0.45194 | 90/9726 (0.93%) | 9021/9600 (93.97%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 3 | 2.20908 | 17/9726 (0.17%) | 8827/9600 (91.95%) | -0.14795 | 87/9726 (0.89%) | 9097/9600 (94.76%) |
| tintype-views + Community Forensics, fitted full fusion | 2.94953 | 12/9726 (0.12%) | 8726/9600 (90.90%) | 0.20086 | 88/9726 (0.90%) | 9116/9600 (94.96%) |

Gates on tintype-views with Community Forensics behind them, both at their own cutoffs. tintype-views misses 883 of 9600 generated photos (Community Forensics flags 498 of them) and flags 85 of 9726 authentic photos (Community Forensics clears 82).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 10699/19326 (55.36%) | 880 | 3 | 1 | 44 | 44 |
| -3 < tintype-views logit < 0.5 | 2193/19326 (11.35%) | 648 | 235 | 102 | 44 | 44 |
| |tintype-views logit| <= 2 | 1872/19326 (9.69%) | 483 | 400 | 196 | 5 | 5 |
| |tintype-views logit| <= 3 | 3460/19326 (17.90%) | 648 | 235 | 102 | 1 | 1 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.81, which routes 1651/19326 (8.54%); 90% of the misses needs |logit| <= 4.40, which routes 7682/19326 (39.75%); 100% of the misses needs |logit| <= 7.60, which routes 19279/19326 (99.76%).

View disagreement (largest minus smallest view logit of tintype-views) as a routing feature, analysis only:

| Gate | Confident negatives outside: generated, authentic | AUROC of view spread for generated among them | Misses inside the margin gate | Misses among as many photos with the largest view spread | In both |
|---|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 3, 24 | 0.0694 | 880 | 378 | 378 |
| -3 < tintype-views logit < 0.5 | 235, 8298 | 0.2419 | 648 | 90 | 78 |
| |tintype-views logit| <= 2 | 400, 9117 | 0.2814 | 483 | 81 | 52 |
| |tintype-views logit| <= 3 | 235, 8298 | 0.2419 | 648 | 115 | 98 |

## Test set, processing original: 14526 photos, 7200 generated, 7326 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 7 FP), points vs tintype-views | Recall at 1% FPR (<= 73 FP), points vs tintype-views | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| tintype-views | 0/14526 (0.00%) | 6656/7200 (92.44%) +0.00 | 6904/7200 (95.89%) +0.00 | 6828, 35 (94.83%, 0.48%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 7738/14526 (53.27%) | 6847/7200 (95.10%) +2.65 | 7155/7200 (99.38%) +3.49 | 7144, 48 (99.22%, 0.66%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1100/14526 (7.57%) | 6855/7200 (95.21%) +2.76 | 7092/7200 (98.50%) +2.61 | 7076, 33 (98.28%, 0.45%) |
| Community Forensics alone | 0/14526 (0.00%) | 6132/7200 (85.17%) -7.28 | 6643/7200 (92.26%) -3.62 | 6711, 107 (93.21%, 1.46%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 906/14526 (6.24%) | 6952/7200 (96.56%) +4.11 | 7027/7200 (97.60%) +1.71 | 6958, 9 (96.64%, 0.12%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 1830/14526 (12.60%) | 6963/7200 (96.71%) +4.26 | 7081/7200 (98.35%) +2.46 | 6998, 10 (97.19%, 0.14%) |
| tintype-views + Community Forensics, 50:50 full fusion | 14526/14526 (100.00%) | 6921/7200 (96.12%) +3.68 | 7122/7200 (98.92%) +3.03 | 7012, 15 (97.39%, 0.20%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 2 | 906/14526 (6.24%) | 6980/7200 (96.94%) +4.50 | 7034/7200 (97.69%) +1.81 | 7010, 21 (97.36%, 0.29%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 3 | 1830/14526 (12.60%) | 7043/7200 (97.82%) +5.38 | 7104/7200 (98.67%) +2.78 | 7087, 29 (98.43%, 0.40%) |
| tintype-views + Community Forensics, fitted full fusion | 14526/14526 (100.00%) | 7057/7200 (98.01%) +5.57 | 7163/7200 (99.49%) +3.60 | 7139, 41 (99.15%, 0.56%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| tintype-views | 0.84512 | 11/7326 (0.15%) | 6690/7200 (92.92%) | -0.46940 | 62/7326 (0.85%) | 6895/7200 (95.76%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1.62600 | 10/7326 (0.14%) | 6876/7200 (95.50%) | 0.11401 | 45/7326 (0.61%) | 7142/7200 (99.19%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1.62600 | 7/7326 (0.10%) | 6817/7200 (94.68%) | -0.23711 | 39/7326 (0.53%) | 7081/7200 (98.35%) |
| Community Forensics alone | 4.40448 | 7/7326 (0.10%) | 6006/7200 (83.42%) | 1.08658 | 52/7326 (0.71%) | 6567/7200 (91.21%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 1.37517 | 2/7326 (0.03%) | 6917/7200 (96.07%) | -1.46972 | 39/7326 (0.53%) | 7002/7200 (97.25%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 1.50465 | 1/7326 (0.01%) | 6896/7200 (95.78%) | -0.92722 | 33/7326 (0.45%) | 7043/7200 (97.82%) |
| tintype-views + Community Forensics, 50:50 full fusion | 1.50465 | 2/7326 (0.03%) | 6760/7200 (93.89%) | -0.68999 | 36/7326 (0.49%) | 7078/7200 (98.31%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 2 | 1.73208 | 5/7326 (0.07%) | 6963/7200 (96.71%) | -0.45194 | 32/7326 (0.44%) | 7021/7200 (97.51%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 3 | 2.20908 | 6/7326 (0.08%) | 7015/7200 (97.43%) | -0.14795 | 32/7326 (0.44%) | 7089/7200 (98.46%) |
| tintype-views + Community Forensics, fitted full fusion | 2.94953 | 4/7326 (0.05%) | 7005/7200 (97.29%) | 0.20086 | 33/7326 (0.45%) | 7134/7200 (99.08%) |

Gates on tintype-views with Community Forensics behind them, both at their own cutoffs. tintype-views misses 372 of 7200 generated photos (Community Forensics flags 340 of them) and flags 35 of 7326 authentic photos (Community Forensics clears 35).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 7738/14526 (53.27%) | 371 | 1 | 1 | 19 | 19 |
| -3 < tintype-views logit < 0.5 | 1100/14526 (7.57%) | 286 | 86 | 79 | 19 | 19 |
| |tintype-views logit| <= 2 | 906/14526 (6.24%) | 209 | 163 | 150 | 0 | 0 |
| |tintype-views logit| <= 3 | 1830/14526 (12.60%) | 286 | 86 | 79 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.72, which routes 743/14526 (5.11%); 90% of the misses needs |logit| <= 4.30, which routes 4517/14526 (31.10%); 100% of the misses needs |logit| <= 7.60, which routes 14486/14526 (99.72%).

View disagreement (largest minus smallest view logit of tintype-views) as a routing feature, analysis only:

| Gate | Confident negatives outside: generated, authentic | AUROC of view spread for generated among them | Misses inside the margin gate | Misses among as many photos with the largest view spread | In both |
|---|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 1, 15 | 0.2667 | 371 | 152 | 152 |
| -3 < tintype-views logit < 0.5 | 86, 6568 | 0.1516 | 286 | 9 | 9 |
| |tintype-views logit| <= 2 | 163, 7029 | 0.1985 | 209 | 9 | 9 |
| |tintype-views logit| <= 3 | 86, 6568 | 0.1516 | 286 | 19 | 19 |

## Test set, processing processed: 4800 photos, 2400 generated, 2400 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 2 FP), points vs tintype-views | Recall at 1% FPR (<= 24 FP), points vs tintype-views | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| tintype-views | 0/4800 (0.00%) | 1416/2400 (59.00%) +0.00 | 1778/2400 (74.08%) +0.00 | 1889, 50 (78.71%, 2.08%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 2961/4800 (61.69%) | 1350/2400 (56.25%) -2.75 | 1816/2400 (75.67%) +1.58 | 1991, 62 (82.96%, 2.58%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1093/4800 (22.77%) | 1347/2400 (56.12%) -2.88 | 1813/2400 (75.54%) +1.46 | 1974, 59 (82.25%, 2.46%) |
| Community Forensics alone | 0/4800 (0.00%) | 1019/2400 (42.46%) -16.54 | 1452/2400 (60.50%) -13.58 | 1595, 59 (66.46%, 2.46%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 966/4800 (20.12%) | 1518/2400 (63.25%) +4.25 | 1824/2400 (76.00%) +1.92 | 1824, 24 (76.00%, 1.00%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 1630/4800 (33.96%) | 1568/2400 (65.33%) +6.33 | 1781/2400 (74.21%) +0.12 | 1795, 26 (74.79%, 1.08%) |
| tintype-views + Community Forensics, 50:50 full fusion | 4800/4800 (100.00%) | 1346/2400 (56.08%) -2.92 | 1725/2400 (71.88%) -2.21 | 1745, 26 (72.71%, 1.08%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 2 | 966/4800 (20.12%) | 1567/2400 (65.29%) +6.29 | 1860/2400 (77.50%) +3.42 | 1966, 43 (81.92%, 1.79%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 3 | 1630/4800 (33.96%) | 1660/2400 (69.17%) +10.17 | 1885/2400 (78.54%) +4.46 | 1993, 53 (83.04%, 2.21%) |
| tintype-views + Community Forensics, fitted full fusion | 4800/4800 (100.00%) | 1549/2400 (64.54%) +5.54 | 1854/2400 (77.25%) +3.17 | 1999, 64 (83.29%, 2.67%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| tintype-views | 0.84512 | 20/2400 (0.83%) | 1730/2400 (72.08%) | -0.46940 | 76/2400 (3.17%) | 1956/2400 (81.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1.62600 | 18/2400 (0.75%) | 1691/2400 (70.46%) | 0.11401 | 59/2400 (2.46%) | 1985/2400 (82.71%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1.62600 | 17/2400 (0.71%) | 1686/2400 (70.25%) | -0.23711 | 65/2400 (2.71%) | 1980/2400 (82.50%) |
| Community Forensics alone | 4.40448 | 11/2400 (0.46%) | 1168/2400 (48.67%) | 1.08658 | 38/2400 (1.58%) | 1489/2400 (62.04%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 1.37517 | 13/2400 (0.54%) | 1747/2400 (72.79%) | -1.46972 | 61/2400 (2.54%) | 1955/2400 (81.46%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 1.50465 | 10/2400 (0.42%) | 1671/2400 (69.62%) | -0.92722 | 57/2400 (2.38%) | 1892/2400 (78.83%) |
| tintype-views + Community Forensics, 50:50 full fusion | 1.50465 | 9/2400 (0.38%) | 1542/2400 (64.25%) | -0.68999 | 48/2400 (2.00%) | 1851/2400 (77.12%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 2 | 1.73208 | 15/2400 (0.62%) | 1820/2400 (75.83%) | -0.45194 | 58/2400 (2.42%) | 2000/2400 (83.33%) |
| tintype-views + Community Forensics, fitted for |tintype-views logit| <= 3 | 2.20908 | 11/2400 (0.46%) | 1812/2400 (75.50%) | -0.14795 | 55/2400 (2.29%) | 2008/2400 (83.67%) |
| tintype-views + Community Forensics, fitted full fusion | 2.94953 | 8/2400 (0.33%) | 1721/2400 (71.71%) | 0.20086 | 55/2400 (2.29%) | 1982/2400 (82.58%) |

Gates on tintype-views with Community Forensics behind them, both at their own cutoffs. tintype-views misses 511 of 2400 generated photos (Community Forensics flags 158 of them) and flags 50 of 2400 authentic photos (Community Forensics clears 47).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 2961/4800 (61.69%) | 509 | 2 | 0 | 25 | 25 |
| -3 < tintype-views logit < 0.5 | 1093/4800 (22.77%) | 362 | 149 | 23 | 25 | 25 |
| |tintype-views logit| <= 2 | 966/4800 (20.12%) | 274 | 237 | 46 | 5 | 5 |
| |tintype-views logit| <= 3 | 1630/4800 (33.96%) | 362 | 149 | 23 | 1 | 1 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.82, which routes 869/4800 (18.10%); 90% of the misses needs |logit| <= 4.48, which routes 2941/4800 (61.27%); 100% of the misses needs |logit| <= 7.36, which routes 4778/4800 (99.54%).

View disagreement (largest minus smallest view logit of tintype-views) as a routing feature, analysis only:

| Gate | Confident negatives outside: generated, authentic | AUROC of view spread for generated among them | Misses inside the margin gate | Misses among as many photos with the largest view spread | In both |
|---|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 2, 9 | 0.0000 | 509 | 230 | 230 |
| -3 < tintype-views logit < 0.5 | 149, 1730 | 0.2867 | 362 | 90 | 73 |
| |tintype-views logit| <= 2 | 237, 2088 | 0.3259 | 274 | 83 | 48 |
| |tintype-views logit| <= 3 | 149, 1730 | 0.2867 | 362 | 126 | 103 |
