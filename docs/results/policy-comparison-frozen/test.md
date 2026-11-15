# Policy comparison

35542 photos scored by every system (commfor, tintype, tintype-sensitive, tintype-strict, tintype-views).

Split: 3759 fit, 12457 calibrate, 19326 test photos; 0 scored photos are in no set and are left out.

Fusion weights fitted on the fit set (3759 photos) only:

- Tintype default + Community Forensics, fitted for |tintype logit| <= 1, fit set: +1.0455 x tintype logit +0.4025 x commfor logit +1.0873 from 115 photos inside the gate; outside it +1.1551 x tintype logit +1.9287 from 3759
- Tintype default + Community Forensics, fitted for |tintype logit| <= 2, fit set: +1.4738 x tintype logit +0.4368 x commfor logit +1.3727 from 302 photos inside the gate; outside it +1.1551 x tintype logit +1.9287 from 3759
- Tintype default + Community Forensics, fitted for |tintype logit| <= 3, fit set: +1.2910 x tintype logit +0.4896 x commfor logit +1.8924 from 606 photos inside the gate; outside it +1.1551 x tintype logit +1.9287 from 3759
- Tintype default + Community Forensics, fitted full fusion, fit set: +1.0404 x tintype logit +0.5716 x commfor logit +2.0343 from 3759 photos

Cutoffs are frozen on the `calibrate` set (12457 photos, 5857 authentic).

## Test set: 19326 photos, 9600 generated, 9726 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 9 FP), points vs Tintype default | Recall at 1% FPR (<= 97 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/19326 (0.00%) | 8073/9600 (84.09%) +0.00 | 8869/9600 (92.39%) +0.00 | 8726, 65 (90.90%, 0.67%) |
| Tintype sensitive | 0/19326 (0.00%) | 6521/9600 (67.93%) -16.17 | 8705/9600 (90.68%) -1.71 | 9134, 413 (95.15%, 4.25%) |
| Tintype strict | 0/19326 (0.00%) | 8285/9600 (86.30%) +2.21 | 8880/9600 (92.50%) +0.11 | 8304, 10 (86.50%, 0.10%) |
| tintype-views | 0/19326 (0.00%) | 8056/9600 (83.92%) -0.18 | 8755/9600 (91.20%) -1.19 | 8717, 85 (90.80%, 0.87%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 10379/19326 (53.70%) | 7650/9600 (79.69%) -4.41 | 9149/9600 (95.30%) +2.92 | 9152, 101 (95.33%, 1.04%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1775/19326 (9.18%) | 7592/9600 (79.08%) -5.01 | 9072/9600 (94.50%) +2.11 | 9060, 83 (94.38%, 0.85%) |
| Community Forensics alone | 0/19326 (0.00%) | 6922/9600 (72.10%) -11.99 | 8099/9600 (84.36%) -8.02 | 8306, 166 (86.52%, 1.71%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 589/19326 (3.05%) | 8030/9600 (83.65%) -0.45 | 8949/9600 (93.22%) +0.83 | 8797, 37 (91.64%, 0.38%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 1459/19326 (7.55%) | 8010/9600 (83.44%) -0.66 | 8965/9600 (93.39%) +1.00 | 8825, 36 (91.93%, 0.37%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 2959/19326 (15.31%) | 8413/9600 (87.64%) +3.54 | 8977/9600 (93.51%) +1.12 | 8842, 40 (92.10%, 0.41%) |
| Tintype default + Community Forensics, 50:50 full fusion | 19326/19326 (100.00%) | 8230/9600 (85.73%) +1.64 | 9009/9600 (93.84%) +1.46 | 8817, 36 (91.84%, 0.37%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 589/19326 (3.05%) | 8284/9600 (86.29%) +2.20 | 8931/9600 (93.03%) +0.65 | 9009, 163 (93.84%, 1.68%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 1459/19326 (7.55%) | 8641/9600 (90.01%) +5.92 | 9013/9600 (93.89%) +1.50 | 8973, 59 (93.47%, 0.61%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 2959/19326 (15.31%) | 8597/9600 (89.55%) +5.46 | 9109/9600 (94.89%) +2.50 | 9091, 83 (94.70%, 0.85%) |
| Tintype default + Community Forensics, fitted full fusion | 19326/19326 (100.00%) | 8663/9600 (90.24%) +6.15 | 9149/9600 (95.30%) +2.92 | 9162, 104 (95.44%, 1.07%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (8285/9600); 1% FPR: Tintype strict (8880/9600).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 7/9726 (0.07%) | 6831/9600 (71.16%) | -0.94908 | 131/9726 (1.35%) | 8941/9600 (93.14%) |
| Tintype sensitive | 7.53853 | 7/9726 (0.07%) | 6420/9600 (66.88%) | 2.69947 | 93/9726 (0.96%) | 8680/9600 (90.42%) |
| Tintype strict | -0.52927 | 19/9726 (0.20%) | 8520/9600 (88.75%) | -1.52022 | 127/9726 (1.31%) | 8931/9600 (93.03%) |
| tintype-views | 0.84512 | 31/9726 (0.32%) | 8420/9600 (87.71%) | -0.46940 | 138/9726 (1.42%) | 8851/9600 (92.20%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 7/9726 (0.07%) | 7167/9600 (74.66%) | 0.05848 | 99/9726 (1.02%) | 9150/9600 (95.31%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 7/9726 (0.07%) | 7118/9600 (74.15%) | -0.28482 | 99/9726 (1.02%) | 9072/9600 (94.50%) |
| Community Forensics alone | 4.40448 | 18/9726 (0.19%) | 7174/9600 (74.73%) | 1.08658 | 90/9726 (0.93%) | 8056/9600 (83.92%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 7/9726 (0.07%) | 7035/9600 (73.28%) | -1.41675 | 109/9726 (1.12%) | 8957/9600 (93.30%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 7/9726 (0.07%) | 7274/9600 (75.77%) | -1.96860 | 121/9726 (1.24%) | 9010/9600 (93.85%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 8/9726 (0.08%) | 7650/9600 (79.69%) | -0.95838 | 91/9726 (0.94%) | 8965/9600 (93.39%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 13/9726 (0.13%) | 8311/9600 (86.57%) | -0.87410 | 92/9726 (0.95%) | 8999/9600 (93.74%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 7/9726 (0.07%) | 6934/9600 (72.23%) | 0.24886 | 107/9726 (1.10%) | 8948/9600 (93.21%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 7/9726 (0.07%) | 7131/9600 (74.28%) | -0.42355 | 97/9726 (1.00%) | 9010/9600 (93.85%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 8/9726 (0.08%) | 7626/9600 (79.44%) | -0.34067 | 106/9726 (1.09%) | 9115/9600 (94.95%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 9/9726 (0.09%) | 8660/9600 (90.21%) | 0.11322 | 98/9726 (1.01%) | 9151/9600 (95.32%) |

Worst slices at the frozen cutoffs (slices of 100 photos of the class or more): the authentic source and the transform with the highest flagged share, and the generator family with the lowest recall.

| Policy | target 0.1% FPR: worst authentic source | worst transform (authentic) | weakest family | target 1% FPR: worst authentic source | worst transform (authentic) | weakest family |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 221/804 (27.49%) | floreview-oneplus 3/108 (2.78%) | noise-a10 25/200 (12.50%) | open-magvit2 554/804 (68.91%) |
| Tintype sensitive | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 206/804 (25.62%) | floreview-oneplus 4/108 (3.70%) | noise-a40 20/200 (10.00%) | open-magvit2 482/804 (59.95%) |
| Tintype strict | floreview-oneplus 1/108 (0.93%) | noise-a10 7/200 (3.50%) | open-magvit2 466/804 (57.96%) | floreview-oneplus 5/108 (4.63%) | noise-a10 23/200 (11.50%) | open-magvit2 558/804 (69.40%) |
| tintype-views | floreview-xiaomi 3/388 (0.77%) | noise-a10 9/200 (4.50%) | open-magvit2 439/804 (54.60%) | floreview-xiaomi 11/388 (2.84%) | noise-a10 31/200 (15.50%) | open-magvit2 533/804 (66.29%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 353/804 (43.91%) | floreview-samsung 13/497 (2.62%) | noise-a10 27/200 (13.50%) | open-magvit2 679/804 (84.45%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 334/804 (41.54%) | floreview-oneplus 4/108 (3.70%) | noise-a10 27/200 (13.50%) | open-magvit2 635/804 (78.98%) |
| Community Forensics alone | floreview-samsung 8/497 (1.61%) | noise-a10 3/200 (1.50%) | flux 224/1064 (21.05%) | floreview-samsung 15/497 (3.02%) | noise-a10 18/200 (9.00%) | flux 505/1064 (47.46%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 284/804 (35.32%) | floreview-oneplus 5/108 (4.63%) | noise-a10 19/200 (9.50%) | open-magvit2 577/804 (71.77%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 384/804 (47.76%) | floreview-oneplus 6/108 (5.56%) | noise-a10 26/200 (13.00%) | open-magvit2 609/804 (75.75%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | floreview-oneplus 1/108 (0.93%) | noise-a40 2/200 (1.00%) | open-magvit2 467/804 (58.08%) | floreview-oneplus 5/108 (4.63%) | noise-a10 23/200 (11.50%) | open-magvit2 631/804 (78.48%) |
| Tintype default + Community Forensics, 50:50 full fusion | floreview-samsung 8/497 (1.61%) | noise-a10 3/200 (1.50%) | flux 673/1064 (63.25%) | floreview-oneplus 5/108 (4.63%) | noise-a10 24/200 (12.00%) | open-magvit2 679/804 (84.45%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 249/804 (30.97%) | floreview-oneplus 5/108 (4.63%) | noise-a10 22/200 (11.00%) | open-magvit2 577/804 (71.77%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | csafe-iphone14pro 4/5304 (0.08%) | noise-a40 2/200 (1.00%) | open-magvit2 326/804 (40.55%) | floreview-samsung 15/497 (3.02%) | noise-a10 22/200 (11.00%) | open-magvit2 600/804 (74.63%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | floreview-oneplus 1/108 (0.93%) | noise-a40 2/200 (1.00%) | open-magvit2 422/804 (52.49%) | floreview-oneplus 5/108 (4.63%) | noise-a10 27/200 (13.50%) | open-magvit2 642/804 (79.85%) |
| Tintype default + Community Forensics, fitted full fusion | floreview-oneplus 1/108 (0.93%) | noise-a10 3/200 (1.50%) | open-magvit2 606/804 (75.37%) | floreview-oneplus 5/108 (4.63%) | noise-a10 26/200 (13.00%) | open-magvit2 680/804 (84.58%) |

Slice by slice against Community Forensics at its own cutoff, each policy at its own cutoff (slices of 100 photos of the class or more): generator families on which the policy flags at least as many generated photos, and authentic sources on which it flags no more.

| Policy | Families reached | Sources reached | Together |
|---|---:|---:|---:|
| Tintype default | 7/9 | 10/10 | 17/19 |
| Tintype sensitive | 7/9 | 2/10 | 9/19 |
| Tintype strict | 4/9 | 10/10 | 14/19 |
| tintype-views | 7/9 | 9/10 | 16/19 |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 9/9 | 9/10 | 18/19 |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 7/9 | 10/10 | 17/19 |
| Community Forensics alone | 9/9 | 10/10 | 19/19 |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 7/9 | 10/10 | 17/19 |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 7/9 | 10/10 | 17/19 |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 7/9 | 10/10 | 17/19 |
| Tintype default + Community Forensics, 50:50 full fusion | 7/9 | 10/10 | 17/19 |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 7/9 | 7/10 | 14/19 |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 7/9 | 10/10 | 17/19 |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 7/9 | 10/10 | 17/19 |
| Tintype default + Community Forensics, fitted full fusion | 9/9 | 9/10 | 18/19 |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 874 of 9600 generated photos (Community Forensics flags 493 of them) and flags 65 of 9726 authentic photos (Community Forensics clears 62).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 10379/19326 (53.70%) | 871 | 3 | 1 | 43 | 42 |
| -3 < tintype logit < 0.5 | 1775/19326 (9.18%) | 625 | 249 | 114 | 43 | 42 |
| |tintype logit| <= 1 | 589/19326 (3.05%) | 222 | 652 | 352 | 26 | 25 |
| |tintype logit| <= 2 | 1459/19326 (7.55%) | 469 | 405 | 215 | 9 | 8 |
| |tintype logit| <= 3 | 2959/19326 (15.31%) | 625 | 249 | 114 | 8 | 8 |
| |tintype logit| <= 4 (sweep) | 4648/19326 (24.05%) | 705 | 169 | 79 | 7 | 7 |
| |tintype logit| <= 6 (sweep) | 14283/19326 (73.91%) | 852 | 22 | 6 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 19296/19326 (99.84%) | 874 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.85, which routes 1329/19326 (6.88%); 90% of the misses needs |logit| <= 4.70, which routes 7902/19326 (40.89%); 100% of the misses needs |logit| <= 7.46, which routes 18994/19326 (98.28%).

## Test set, processing original: 14526 photos, 7200 generated, 7326 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 7 FP), points vs Tintype default | Recall at 1% FPR (<= 73 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/14526 (0.00%) | 6629/7200 (92.07%) +0.00 | 6954/7200 (96.58%) +0.00 | 6823, 26 (94.76%, 0.35%) |
| Tintype sensitive | 0/14526 (0.00%) | 5829/7200 (80.96%) -11.11 | 6886/7200 (95.64%) -0.94 | 7003, 195 (97.26%, 2.66%) |
| Tintype strict | 0/14526 (0.00%) | 6741/7200 (93.62%) +1.56 | 6924/7200 (96.17%) -0.42 | 6615, 4 (91.88%, 0.05%) |
| tintype-views | 0/14526 (0.00%) | 6656/7200 (92.44%) +0.38 | 6904/7200 (95.89%) -0.69 | 6828, 35 (94.83%, 0.48%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 7494/14526 (51.59%) | 6830/7200 (94.86%) +2.79 | 7159/7200 (99.43%) +2.85 | 7146, 43 (99.25%, 0.59%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 919/14526 (6.33%) | 6832/7200 (94.89%) +2.82 | 7085/7200 (98.40%) +1.82 | 7070, 30 (98.19%, 0.41%) |
| Community Forensics alone | 0/14526 (0.00%) | 6132/7200 (85.17%) -6.90 | 6643/7200 (92.26%) -4.32 | 6711, 107 (93.21%, 1.46%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 266/14526 (1.83%) | 6800/7200 (94.44%) +2.38 | 6980/7200 (96.94%) +0.36 | 6896, 14 (95.78%, 0.19%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 725/14526 (4.99%) | 6914/7200 (96.03%) +3.96 | 7006/7200 (97.31%) +0.72 | 6952, 12 (96.56%, 0.16%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 1644/14526 (11.32%) | 6941/7200 (96.40%) +4.33 | 7069/7200 (98.18%) +1.60 | 7000, 13 (97.22%, 0.18%) |
| Tintype default + Community Forensics, 50:50 full fusion | 14526/14526 (100.00%) | 6940/7200 (96.39%) +4.32 | 7135/7200 (99.10%) +2.51 | 7033, 13 (97.68%, 0.18%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 266/14526 (1.83%) | 6813/7200 (94.62%) +2.56 | 6981/7200 (96.96%) +0.38 | 6976, 71 (96.89%, 0.97%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 725/14526 (4.99%) | 6940/7200 (96.39%) +4.32 | 7017/7200 (97.46%) +0.88 | 6995, 22 (97.15%, 0.30%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 1644/14526 (11.32%) | 7003/7200 (97.26%) +5.19 | 7090/7200 (98.47%) +1.89 | 7076, 29 (98.28%, 0.40%) |
| Tintype default + Community Forensics, fitted full fusion | 14526/14526 (100.00%) | 7072/7200 (98.22%) +6.15 | 7162/7200 (99.47%) +2.89 | 7149, 42 (99.29%, 0.57%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (6741/7200); 1% FPR: Tintype default (6954/7200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 4/7326 (0.05%) | 5656/7200 (78.56%) | -0.94908 | 49/7326 (0.67%) | 6906/7200 (95.92%) |
| Tintype sensitive | 7.53853 | 4/7326 (0.05%) | 5371/7200 (74.60%) | 2.69947 | 45/7326 (0.61%) | 6774/7200 (94.08%) |
| Tintype strict | -0.52927 | 6/7326 (0.08%) | 6722/7200 (93.36%) | -1.52022 | 54/7326 (0.74%) | 6887/7200 (95.65%) |
| tintype-views | 0.84512 | 11/7326 (0.15%) | 6690/7200 (92.92%) | -0.46940 | 62/7326 (0.85%) | 6895/7200 (95.76%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 4/7326 (0.05%) | 5947/7200 (82.60%) | 0.05848 | 42/7326 (0.57%) | 7145/7200 (99.24%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 4/7326 (0.05%) | 5901/7200 (81.96%) | -0.28482 | 36/7326 (0.49%) | 7075/7200 (98.26%) |
| Community Forensics alone | 4.40448 | 7/7326 (0.10%) | 6006/7200 (83.42%) | 1.08658 | 52/7326 (0.71%) | 6567/7200 (91.21%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 4/7326 (0.05%) | 5815/7200 (80.76%) | -1.41675 | 44/7326 (0.60%) | 6950/7200 (96.53%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 4/7326 (0.05%) | 6016/7200 (83.56%) | -1.96860 | 50/7326 (0.68%) | 6995/7200 (97.15%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 4/7326 (0.05%) | 6305/7200 (87.57%) | -0.95838 | 34/7326 (0.46%) | 7041/7200 (97.79%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 2/7326 (0.03%) | 6767/7200 (93.99%) | -0.87410 | 36/7326 (0.49%) | 7099/7200 (98.60%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 4/7326 (0.05%) | 5753/7200 (79.90%) | 0.24886 | 42/7326 (0.57%) | 6950/7200 (96.53%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 4/7326 (0.05%) | 5920/7200 (82.22%) | -0.42355 | 40/7326 (0.55%) | 7007/7200 (97.32%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 4/7326 (0.05%) | 6264/7200 (87.00%) | -0.34067 | 38/7326 (0.52%) | 7080/7200 (98.33%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 2/7326 (0.03%) | 6955/7200 (96.60%) | 0.11322 | 37/7326 (0.51%) | 7145/7200 (99.24%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 377 of 7200 generated photos (Community Forensics flags 344 of them) and flags 26 of 7326 authentic photos (Community Forensics clears 26).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 7494/14526 (51.59%) | 376 | 1 | 1 | 17 | 17 |
| -3 < tintype logit < 0.5 | 919/14526 (6.33%) | 277 | 100 | 88 | 17 | 17 |
| |tintype logit| <= 1 | 266/14526 (1.83%) | 89 | 288 | 262 | 10 | 10 |
| |tintype logit| <= 2 | 725/14526 (4.99%) | 189 | 188 | 169 | 4 | 4 |
| |tintype logit| <= 3 | 1644/14526 (11.32%) | 277 | 100 | 88 | 4 | 4 |
| |tintype logit| <= 4 (sweep) | 2728/14526 (18.78%) | 312 | 65 | 58 | 4 | 4 |
| |tintype logit| <= 6 (sweep) | 10220/14526 (70.36%) | 372 | 5 | 5 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 14500/14526 (99.82%) | 377 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.96, which routes 701/14526 (4.83%); 90% of the misses needs |logit| <= 4.46, which routes 4212/14526 (29.00%); 100% of the misses needs |logit| <= 7.24, which routes 13975/14526 (96.21%).

## Test set, processing processed: 4800 photos, 2400 generated, 2400 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 2 FP), points vs Tintype default | Recall at 1% FPR (<= 24 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/4800 (0.00%) | 1145/2400 (47.71%) +0.00 | 1811/2400 (75.46%) +0.00 | 1903, 39 (79.29%, 1.62%) |
| Tintype sensitive | 0/4800 (0.00%) | 1030/2400 (42.92%) -4.79 | 1443/2400 (60.12%) -15.33 | 2131, 218 (88.79%, 9.08%) |
| Tintype strict | 0/4800 (0.00%) | 1560/2400 (65.00%) +17.29 | 1903/2400 (79.29%) +3.83 | 1689, 6 (70.38%, 0.25%) |
| tintype-views | 0/4800 (0.00%) | 1416/2400 (59.00%) +11.29 | 1778/2400 (74.08%) -1.38 | 1889, 50 (78.71%, 2.08%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 2885/4800 (60.10%) | 1190/2400 (49.58%) +1.88 | 1859/2400 (77.46%) +2.00 | 2006, 58 (83.58%, 2.42%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 856/4800 (17.83%) | 1187/2400 (49.46%) +1.75 | 1849/2400 (77.04%) +1.58 | 1990, 53 (82.92%, 2.21%) |
| Community Forensics alone | 0/4800 (0.00%) | 1019/2400 (42.46%) -5.25 | 1452/2400 (60.50%) -14.96 | 1595, 59 (66.46%, 2.46%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 323/4800 (6.73%) | 1189/2400 (49.54%) +1.83 | 1908/2400 (79.50%) +4.04 | 1901, 23 (79.21%, 0.96%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 734/4800 (15.29%) | 1227/2400 (51.12%) +3.42 | 1876/2400 (78.17%) +2.71 | 1873, 24 (78.04%, 1.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 1315/4800 (27.40%) | 1241/2400 (51.71%) +4.00 | 1827/2400 (76.12%) +0.67 | 1842, 27 (76.75%, 1.12%) |
| Tintype default + Community Forensics, 50:50 full fusion | 4800/4800 (100.00%) | 1338/2400 (55.75%) +8.04 | 1794/2400 (74.75%) -0.71 | 1784, 23 (74.33%, 0.96%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 323/4800 (6.73%) | 1151/2400 (47.96%) +0.25 | 1899/2400 (79.12%) +3.67 | 2033, 92 (84.71%, 3.83%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 734/4800 (15.29%) | 1180/2400 (49.17%) +1.46 | 1890/2400 (78.75%) +3.29 | 1978, 37 (82.42%, 1.54%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 1315/4800 (27.40%) | 1245/2400 (51.88%) +4.17 | 1884/2400 (78.50%) +3.04 | 2015, 54 (83.96%, 2.25%) |
| Tintype default + Community Forensics, fitted full fusion | 4800/4800 (100.00%) | 1593/2400 (66.38%) +18.67 | 1903/2400 (79.29%) +3.83 | 2013, 62 (83.88%, 2.58%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (1560/2400); 1% FPR: Tintype strict (1903/2400).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 3/2400 (0.12%) | 1175/2400 (48.96%) | -0.94908 | 82/2400 (3.42%) | 2035/2400 (84.79%) |
| Tintype sensitive | 7.53853 | 3/2400 (0.12%) | 1049/2400 (43.71%) | 2.69947 | 48/2400 (2.00%) | 1906/2400 (79.42%) |
| Tintype strict | -0.52927 | 13/2400 (0.54%) | 1798/2400 (74.92%) | -1.52022 | 73/2400 (3.04%) | 2044/2400 (85.17%) |
| tintype-views | 0.84512 | 20/2400 (0.83%) | 1730/2400 (72.08%) | -0.46940 | 76/2400 (3.17%) | 1956/2400 (81.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 3/2400 (0.12%) | 1220/2400 (50.83%) | 0.05848 | 57/2400 (2.38%) | 2005/2400 (83.54%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 3/2400 (0.12%) | 1217/2400 (50.71%) | -0.28482 | 63/2400 (2.62%) | 1997/2400 (83.21%) |
| Community Forensics alone | 4.40448 | 11/2400 (0.46%) | 1168/2400 (48.67%) | 1.08658 | 38/2400 (1.58%) | 1489/2400 (62.04%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 3/2400 (0.12%) | 1220/2400 (50.83%) | -1.41675 | 65/2400 (2.71%) | 2007/2400 (83.62%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 3/2400 (0.12%) | 1258/2400 (52.42%) | -1.96860 | 71/2400 (2.96%) | 2015/2400 (83.96%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 4/2400 (0.17%) | 1345/2400 (56.04%) | -0.95838 | 57/2400 (2.38%) | 1924/2400 (80.17%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 11/2400 (0.46%) | 1544/2400 (64.33%) | -0.87410 | 56/2400 (2.33%) | 1900/2400 (79.17%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 3/2400 (0.12%) | 1181/2400 (49.21%) | 0.24886 | 65/2400 (2.71%) | 1998/2400 (83.25%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 3/2400 (0.12%) | 1211/2400 (50.46%) | -0.42355 | 57/2400 (2.38%) | 2003/2400 (83.46%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 4/2400 (0.17%) | 1362/2400 (56.75%) | -0.34067 | 68/2400 (2.83%) | 2035/2400 (84.79%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 7/2400 (0.29%) | 1705/2400 (71.04%) | 0.11322 | 61/2400 (2.54%) | 2006/2400 (83.58%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 497 of 2400 generated photos (Community Forensics flags 149 of them) and flags 39 of 2400 authentic photos (Community Forensics clears 36).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 2885/4800 (60.10%) | 495 | 2 | 0 | 26 | 25 |
| -3 < tintype logit < 0.5 | 856/4800 (17.83%) | 348 | 149 | 26 | 26 | 25 |
| |tintype logit| <= 1 | 323/4800 (6.73%) | 133 | 364 | 90 | 16 | 15 |
| |tintype logit| <= 2 | 734/4800 (15.29%) | 280 | 217 | 46 | 5 | 4 |
| |tintype logit| <= 3 | 1315/4800 (27.40%) | 348 | 149 | 26 | 4 | 4 |
| |tintype logit| <= 4 (sweep) | 1920/4800 (40.00%) | 393 | 104 | 21 | 3 | 3 |
| |tintype logit| <= 6 (sweep) | 4063/4800 (84.65%) | 480 | 17 | 1 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 4796/4800 (99.92%) | 497 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.80, which routes 665/4800 (13.85%); 90% of the misses needs |logit| <= 4.89, which routes 3047/4800 (63.48%); 100% of the misses needs |logit| <= 7.46, which routes 4763/4800 (99.23%).

Wrote 15 score files to runs/ws3/policies (`path label generator dataset prob`).
