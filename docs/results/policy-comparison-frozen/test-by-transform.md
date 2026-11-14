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

## Test set, transform -: 14526 photos, 7200 generated, 7326 authentic

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

## Test set, transform resize-0.5-bicubic: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 194/200 (97.00%) +0.00 | 195/200 (97.50%) +0.00 | 194, 0 (97.00%, 0.00%) |
| Tintype sensitive | 0/400 (0.00%) | 181/200 (90.50%) -6.50 | 197/200 (98.50%) +1.00 | 198, 12 (99.00%, 6.00%) |
| Tintype strict | 0/400 (0.00%) | 193/200 (96.50%) -0.50 | 194/200 (97.00%) -0.50 | 189, 0 (94.50%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 195/200 (97.50%) +0.50 | 196/200 (98.00%) +0.50 | 195, 0 (97.50%, 0.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 205/400 (51.25%) | 187/200 (93.50%) -3.50 | 196/200 (98.00%) +0.50 | 196, 1 (98.00%, 0.50%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 35/400 (8.75%) | 187/200 (93.50%) -3.50 | 196/200 (98.00%) +0.50 | 196, 1 (98.00%, 0.50%) |
| Community Forensics alone | 0/400 (0.00%) | 134/200 (67.00%) -30.00 | 159/200 (79.50%) -18.00 | 168, 4 (84.00%, 2.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 8/400 (2.00%) | 189/200 (94.50%) -2.50 | 197/200 (98.50%) +1.00 | 195, 1 (97.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 19/400 (4.75%) | 191/200 (95.50%) -1.50 | 195/200 (97.50%) +0.00 | 193, 1 (96.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 52/400 (13.00%) | 186/200 (93.00%) -4.00 | 192/200 (96.00%) -1.50 | 191, 1 (95.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 174/200 (87.00%) -10.00 | 191/200 (95.50%) -2.00 | 187, 1 (93.50%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 8/400 (2.00%) | 193/200 (96.50%) -0.50 | 197/200 (98.50%) +1.00 | 197, 3 (98.50%, 1.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 19/400 (4.75%) | 193/200 (96.50%) -0.50 | 196/200 (98.00%) +0.50 | 196, 1 (98.00%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 52/400 (13.00%) | 191/200 (95.50%) -1.50 | 196/200 (98.00%) +0.50 | 196, 2 (98.00%, 1.00%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 186/200 (93.00%) -4.00 | 196/200 (98.00%) +0.50 | 196, 2 (98.00%, 1.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: tintype-views (195/200); 1% FPR: Tintype sensitive (197/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 165/200 (82.50%) | -0.94908 | 4/200 (2.00%) | 195/200 (97.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 160/200 (80.00%) | 2.69947 | 1/200 (0.50%) | 194/200 (97.00%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 190/200 (95.00%) | -1.52022 | 2/200 (1.00%) | 194/200 (97.00%) |
| tintype-views | 0.84512 | 0/200 (0.00%) | 191/200 (95.50%) | -0.46940 | 1/200 (0.50%) | 196/200 (98.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 167/200 (83.50%) | 0.05848 | 1/200 (0.50%) | 196/200 (98.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 167/200 (83.50%) | -0.28482 | 3/200 (1.50%) | 196/200 (98.00%) |
| Community Forensics alone | 4.40448 | 0/200 (0.00%) | 130/200 (65.00%) | 1.08658 | 3/200 (1.50%) | 160/200 (80.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 166/200 (83.00%) | -1.41675 | 3/200 (1.50%) | 197/200 (98.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 170/200 (85.00%) | -1.96860 | 2/200 (1.00%) | 195/200 (97.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 170/200 (85.00%) | -0.95838 | 3/200 (1.50%) | 193/200 (96.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 175/200 (87.50%) | -0.87410 | 3/200 (1.50%) | 191/200 (95.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 165/200 (82.50%) | 0.24886 | 3/200 (1.50%) | 197/200 (98.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 167/200 (83.50%) | -0.42355 | 2/200 (1.00%) | 196/200 (98.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 172/200 (86.00%) | -0.34067 | 2/200 (1.00%) | 196/200 (98.00%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/200 (0.00%) | 186/200 (93.00%) | 0.11322 | 2/200 (1.00%) | 196/200 (98.00%) |

## Test set, transform resize-0.25-bicubic: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 173/200 (86.50%) +0.00 | 190/200 (95.00%) +0.00 | 175, 1 (87.50%, 0.50%) |
| Tintype sensitive | 0/400 (0.00%) | 175/200 (87.50%) +1.00 | 190/200 (95.00%) +0.00 | 193, 4 (96.50%, 2.00%) |
| Tintype strict | 0/400 (0.00%) | 173/200 (86.50%) +0.00 | 185/200 (92.50%) -2.50 | 160, 0 (80.00%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 173/200 (86.50%) +0.00 | 190/200 (95.00%) +0.00 | 175, 1 (87.50%, 0.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 218/400 (54.50%) | 137/200 (68.50%) -18.00 | 173/200 (86.50%) -8.50 | 172, 2 (86.00%, 1.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 53/400 (13.25%) | 137/200 (68.50%) -18.00 | 173/200 (86.50%) -8.50 | 172, 2 (86.00%, 1.00%) |
| Community Forensics alone | 0/400 (0.00%) | 45/200 (22.50%) -64.00 | 93/200 (46.50%) -48.50 | 89, 2 (44.50%, 1.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 26/400 (6.50%) | 170/200 (85.00%) -1.50 | 177/200 (88.50%) -6.50 | 165, 0 (82.50%, 0.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 63/400 (15.75%) | 143/200 (71.50%) -15.00 | 166/200 (83.00%) -12.00 | 148, 1 (74.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 97/400 (24.25%) | 134/200 (67.00%) -19.50 | 159/200 (79.50%) -15.50 | 142, 1 (71.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 100/200 (50.00%) -36.50 | 157/200 (78.50%) -16.50 | 133, 1 (66.50%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 26/400 (6.50%) | 169/200 (84.50%) -2.00 | 175/200 (87.50%) -7.50 | 175, 4 (87.50%, 2.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 63/400 (15.75%) | 154/200 (77.00%) -9.50 | 176/200 (88.00%) -7.00 | 170, 1 (85.00%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 97/400 (24.25%) | 142/200 (71.00%) -15.50 | 179/200 (89.50%) -5.50 | 173, 1 (86.50%, 0.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 134/200 (67.00%) -19.50 | 173/200 (86.50%) -8.50 | 169, 2 (84.50%, 1.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype sensitive (175/200); 1% FPR: Tintype default (190/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 116/200 (58.00%) | -0.94908 | 2/200 (1.00%) | 188/200 (94.00%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 114/200 (57.00%) | 2.69947 | 0/200 (0.00%) | 173/200 (86.50%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 172/200 (86.00%) | -1.52022 | 3/200 (1.50%) | 186/200 (93.00%) |
| tintype-views | 0.84512 | 0/200 (0.00%) | 166/200 (83.00%) | -0.46940 | 1/200 (0.50%) | 182/200 (91.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 116/200 (58.00%) | 0.05848 | 2/200 (1.00%) | 172/200 (86.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 116/200 (58.00%) | -0.28482 | 2/200 (1.00%) | 172/200 (86.00%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 49/200 (24.50%) | 1.08658 | 2/200 (1.00%) | 77/200 (38.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 116/200 (58.00%) | -1.41675 | 2/200 (1.00%) | 174/200 (87.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 116/200 (58.00%) | -1.96860 | 3/200 (1.50%) | 168/200 (84.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 119/200 (59.50%) | -0.95838 | 2/200 (1.00%) | 149/200 (74.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 103/200 (51.50%) | -0.87410 | 2/200 (1.00%) | 143/200 (71.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 116/200 (58.00%) | 0.24886 | 2/200 (1.00%) | 173/200 (86.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 116/200 (58.00%) | -0.42355 | 1/200 (0.50%) | 175/200 (87.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 120/200 (60.00%) | -0.34067 | 2/200 (1.00%) | 177/200 (88.50%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 1/200 (0.50%) | 135/200 (67.50%) | 0.11322 | 2/200 (1.00%) | 169/200 (84.50%) |

## Test set, transform jpeg-qv4: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 148/200 (74.00%) +0.00 | 184/200 (92.00%) +0.00 | 164, 1 (82.00%, 0.50%) |
| Tintype sensitive | 0/400 (0.00%) | 164/200 (82.00%) +8.00 | 181/200 (90.50%) -1.50 | 182, 3 (91.00%, 1.50%) |
| Tintype strict | 0/400 (0.00%) | 172/200 (86.00%) +12.00 | 180/200 (90.00%) -2.00 | 140, 0 (70.00%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 144/200 (72.00%) -2.00 | 181/200 (90.50%) -1.50 | 162, 2 (81.00%, 1.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 224/400 (56.00%) | 145/200 (72.50%) -1.50 | 190/200 (95.00%) +3.00 | 188, 2 (94.00%, 1.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 49/400 (12.25%) | 145/200 (72.50%) -1.50 | 186/200 (93.00%) +1.00 | 185, 2 (92.50%, 1.00%) |
| Community Forensics alone | 0/400 (0.00%) | 115/200 (57.50%) -16.50 | 167/200 (83.50%) -8.50 | 156, 1 (78.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 31/400 (7.75%) | 179/200 (89.50%) +15.50 | 181/200 (90.50%) -1.50 | 169, 0 (84.50%, 0.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 57/400 (14.25%) | 160/200 (80.00%) +6.00 | 181/200 (90.50%) -1.50 | 171, 1 (85.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 94/400 (23.50%) | 160/200 (80.00%) +6.00 | 187/200 (93.50%) +1.50 | 173, 1 (86.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 142/200 (71.00%) -3.00 | 191/200 (95.50%) +3.50 | 167, 1 (83.50%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 31/400 (7.75%) | 179/200 (89.50%) +15.50 | 181/200 (90.50%) -1.50 | 177, 0 (88.50%, 0.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 57/400 (14.25%) | 171/200 (85.50%) +11.50 | 180/200 (90.00%) -2.00 | 176, 1 (88.00%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 94/400 (23.50%) | 171/200 (85.50%) +11.50 | 192/200 (96.00%) +4.00 | 184, 1 (92.00%, 0.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 161/200 (80.50%) +6.50 | 193/200 (96.50%) +4.50 | 185, 1 (92.50%, 0.50%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (172/200); 1% FPR: Tintype default (184/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 90/200 (45.00%) | -0.94908 | 1/200 (0.50%) | 177/200 (88.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 70/200 (35.00%) | 2.69947 | 0/200 (0.00%) | 157/200 (78.50%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 149/200 (74.50%) | -1.52022 | 1/200 (0.50%) | 175/200 (87.50%) |
| tintype-views | 0.84512 | 1/200 (0.50%) | 145/200 (72.50%) | -0.46940 | 2/200 (1.00%) | 169/200 (84.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 103/200 (51.50%) | 0.05848 | 2/200 (1.00%) | 188/200 (94.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 103/200 (51.50%) | -0.28482 | 2/200 (1.00%) | 185/200 (92.50%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 124/200 (62.00%) | 1.08658 | 1/200 (0.50%) | 152/200 (76.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 99/200 (49.50%) | -1.41675 | 0/200 (0.00%) | 174/200 (87.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 105/200 (52.50%) | -1.96860 | 1/200 (0.50%) | 180/200 (90.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 119/200 (59.50%) | -0.95838 | 1/200 (0.50%) | 179/200 (89.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 143/200 (71.50%) | -0.87410 | 1/200 (0.50%) | 178/200 (89.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 91/200 (45.50%) | 0.24886 | 0/200 (0.00%) | 174/200 (87.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 96/200 (48.00%) | -0.42355 | 1/200 (0.50%) | 178/200 (89.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 118/200 (59.00%) | -0.34067 | 1/200 (0.50%) | 185/200 (92.50%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/200 (0.00%) | 157/200 (78.50%) | 0.11322 | 1/200 (0.50%) | 185/200 (92.50%) |

## Test set, transform jpeg-qv12: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 126/200 (63.00%) +0.00 | 179/200 (89.50%) +0.00 | 137, 1 (68.50%, 0.50%) |
| Tintype sensitive | 0/400 (0.00%) | 143/200 (71.50%) +8.50 | 166/200 (83.00%) -6.50 | 172, 4 (86.00%, 2.00%) |
| Tintype strict | 0/400 (0.00%) | 160/200 (80.00%) +17.00 | 170/200 (85.00%) -4.50 | 108, 0 (54.00%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 109/200 (54.50%) -8.50 | 166/200 (83.00%) -6.50 | 135, 1 (67.50%, 0.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 269/400 (67.25%) | 98/200 (49.00%) -14.00 | 144/200 (72.00%) -17.50 | 142, 2 (71.00%, 1.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 78/400 (19.50%) | 98/200 (49.00%) -14.00 | 144/200 (72.00%) -17.50 | 142, 2 (71.00%, 1.00%) |
| Community Forensics alone | 0/400 (0.00%) | 44/200 (22.00%) -41.00 | 116/200 (58.00%) -31.50 | 94, 1 (47.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 37/400 (9.25%) | 157/200 (78.50%) +15.50 | 162/200 (81.00%) -8.50 | 133, 0 (66.50%, 0.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 83/400 (20.75%) | 116/200 (58.00%) -5.00 | 158/200 (79.00%) -10.50 | 126, 1 (63.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 124/400 (31.00%) | 107/200 (53.50%) -9.50 | 152/200 (76.00%) -13.50 | 123, 1 (61.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 87/200 (43.50%) -19.50 | 152/200 (76.00%) -13.50 | 118, 1 (59.00%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 37/400 (9.25%) | 156/200 (78.00%) +15.00 | 163/200 (81.50%) -8.00 | 154, 0 (77.00%, 0.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 83/400 (20.75%) | 131/200 (65.50%) +2.50 | 155/200 (77.50%) -12.00 | 145, 1 (72.50%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 124/400 (31.00%) | 126/200 (63.00%) +0.00 | 169/200 (84.50%) -5.00 | 148, 1 (74.00%, 0.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 113/200 (56.50%) -6.50 | 163/200 (81.50%) -8.00 | 146, 1 (73.00%, 0.50%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (160/200); 1% FPR: Tintype default (179/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 56/200 (28.00%) | -0.94908 | 1/200 (0.50%) | 159/200 (79.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 43/200 (21.50%) | 2.69947 | 0/200 (0.00%) | 138/200 (69.00%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 123/200 (61.50%) | -1.52022 | 0/200 (0.00%) | 159/200 (79.50%) |
| tintype-views | 0.84512 | 1/200 (0.50%) | 110/200 (55.00%) | -0.46940 | 1/200 (0.50%) | 145/200 (72.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 57/200 (28.50%) | 0.05848 | 2/200 (1.00%) | 142/200 (71.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 57/200 (28.50%) | -0.28482 | 2/200 (1.00%) | 142/200 (71.00%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 50/200 (25.00%) | 1.08658 | 1/200 (0.50%) | 85/200 (42.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 58/200 (29.00%) | -1.41675 | 0/200 (0.00%) | 149/200 (74.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 58/200 (29.00%) | -1.96860 | 1/200 (0.50%) | 150/200 (75.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 60/200 (30.00%) | -0.95838 | 1/200 (0.50%) | 134/200 (67.00%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 90/200 (45.00%) | -0.87410 | 1/200 (0.50%) | 131/200 (65.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 56/200 (28.00%) | 0.24886 | 0/200 (0.00%) | 146/200 (73.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 56/200 (28.00%) | -0.42355 | 1/200 (0.50%) | 147/200 (73.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 68/200 (34.00%) | -0.34067 | 1/200 (0.50%) | 152/200 (76.00%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/200 (0.00%) | 110/200 (55.00%) | 0.11322 | 1/200 (0.50%) | 146/200 (73.00%) |

## Test set, transform blur-s1.0: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 187/200 (93.50%) +0.00 | 195/200 (97.50%) +0.00 | 191, 1 (95.50%, 0.50%) |
| Tintype sensitive | 0/400 (0.00%) | 172/200 (86.00%) -7.50 | 195/200 (97.50%) +0.00 | 196, 7 (98.00%, 3.50%) |
| Tintype strict | 0/400 (0.00%) | 195/200 (97.50%) +4.00 | 195/200 (97.50%) +0.00 | 187, 0 (93.50%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 189/200 (94.50%) +1.00 | 194/200 (97.00%) -0.50 | 192, 1 (96.00%, 0.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 199/400 (49.75%) | 189/200 (94.50%) +1.00 | 199/200 (99.50%) +2.00 | 199, 2 (99.50%, 1.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 29/400 (7.25%) | 189/200 (94.50%) +1.00 | 198/200 (99.00%) +1.50 | 198, 2 (99.00%, 1.00%) |
| Community Forensics alone | 0/400 (0.00%) | 167/200 (83.50%) -10.00 | 187/200 (93.50%) -4.00 | 189, 3 (94.50%, 1.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 11/400 (2.75%) | 195/200 (97.50%) +4.00 | 196/200 (98.00%) +0.50 | 195, 0 (97.50%, 0.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 21/400 (5.25%) | 195/200 (97.50%) +4.00 | 196/200 (98.00%) +0.50 | 196, 1 (98.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 51/400 (12.75%) | 194/200 (97.00%) +3.50 | 198/200 (99.00%) +1.50 | 198, 1 (99.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 187/200 (93.50%) +0.00 | 198/200 (99.00%) +1.50 | 198, 1 (99.00%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 11/400 (2.75%) | 195/200 (97.50%) +4.00 | 196/200 (98.00%) +0.50 | 196, 3 (98.00%, 1.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 21/400 (5.25%) | 196/200 (98.00%) +4.50 | 196/200 (98.00%) +0.50 | 196, 2 (98.00%, 1.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 51/400 (12.75%) | 197/200 (98.50%) +5.00 | 198/200 (99.00%) +1.50 | 198, 2 (99.00%, 1.00%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 195/200 (97.50%) +4.00 | 199/200 (99.50%) +2.00 | 199, 3 (99.50%, 1.50%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (195/200); 1% FPR: Tintype default (195/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 158/200 (79.00%) | -0.94908 | 3/200 (1.50%) | 195/200 (97.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 149/200 (74.50%) | 2.69947 | 1/200 (0.50%) | 190/200 (95.00%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 188/200 (94.00%) | -1.52022 | 1/200 (0.50%) | 195/200 (97.50%) |
| tintype-views | 0.84512 | 0/200 (0.00%) | 188/200 (94.00%) | -0.46940 | 2/200 (1.00%) | 193/200 (96.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 163/200 (81.50%) | 0.05848 | 2/200 (1.00%) | 199/200 (99.50%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 163/200 (81.50%) | -0.28482 | 3/200 (1.50%) | 198/200 (99.00%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 170/200 (85.00%) | 1.08658 | 1/200 (0.50%) | 186/200 (93.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 164/200 (82.00%) | -1.41675 | 2/200 (1.00%) | 195/200 (97.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 168/200 (84.00%) | -1.96860 | 3/200 (1.50%) | 196/200 (98.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 178/200 (89.00%) | -0.95838 | 3/200 (1.50%) | 198/200 (99.00%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 188/200 (94.00%) | -0.87410 | 3/200 (1.50%) | 198/200 (99.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 159/200 (79.50%) | 0.24886 | 2/200 (1.00%) | 196/200 (98.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 164/200 (82.00%) | -0.42355 | 2/200 (1.00%) | 196/200 (98.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 175/200 (87.50%) | -0.34067 | 2/200 (1.00%) | 198/200 (99.00%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/200 (0.00%) | 195/200 (97.50%) | 0.11322 | 2/200 (1.00%) | 199/200 (99.50%) |

## Test set, transform blur-s3.0: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 192/200 (96.00%) +0.00 | 195/200 (97.50%) +0.00 | 194, 1 (97.00%, 0.50%) |
| Tintype sensitive | 0/400 (0.00%) | 145/200 (72.50%) -23.50 | 189/200 (94.50%) -3.00 | 200, 29 (100.00%, 14.50%) |
| Tintype strict | 0/400 (0.00%) | 192/200 (96.00%) +0.00 | 194/200 (97.00%) -0.50 | 185, 0 (92.50%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 192/200 (96.00%) +0.00 | 195/200 (97.50%) +0.00 | 194, 1 (97.00%, 0.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 212/400 (53.00%) | 181/200 (90.50%) -5.50 | 200/200 (100.00%) +2.50 | 200, 2 (100.00%, 1.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 57/400 (14.25%) | 181/200 (90.50%) -5.50 | 200/200 (100.00%) +2.50 | 200, 1 (100.00%, 0.50%) |
| Community Forensics alone | 0/400 (0.00%) | 155/200 (77.50%) -18.50 | 189/200 (94.50%) -3.00 | 190, 5 (95.00%, 2.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 18/400 (4.50%) | 182/200 (91.00%) -5.00 | 199/200 (99.50%) +2.00 | 197, 1 (98.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 36/400 (9.00%) | 192/200 (96.00%) +0.00 | 200/200 (100.00%) +2.50 | 200, 1 (100.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 85/400 (21.25%) | 192/200 (96.00%) +0.00 | 200/200 (100.00%) +2.50 | 200, 1 (100.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 183/200 (91.50%) -4.50 | 199/200 (99.50%) +2.00 | 199, 1 (99.50%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 18/400 (4.50%) | 196/200 (98.00%) +2.00 | 199/200 (99.50%) +2.00 | 200, 6 (100.00%, 3.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 36/400 (9.00%) | 198/200 (99.00%) +3.00 | 200/200 (100.00%) +2.50 | 200, 1 (100.00%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 85/400 (21.25%) | 198/200 (99.00%) +3.00 | 200/200 (100.00%) +2.50 | 200, 3 (100.00%, 1.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 194/200 (97.00%) +1.00 | 199/200 (99.50%) +2.00 | 200, 4 (100.00%, 2.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype default (192/200); 1% FPR: Tintype default (195/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 131/200 (65.50%) | -0.94908 | 8/200 (4.00%) | 197/200 (98.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 121/200 (60.50%) | 2.69947 | 3/200 (1.50%) | 191/200 (95.50%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 191/200 (95.50%) | -1.52022 | 4/200 (2.00%) | 196/200 (98.00%) |
| tintype-views | 0.84512 | 0/200 (0.00%) | 187/200 (93.50%) | -0.46940 | 5/200 (2.50%) | 196/200 (98.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 135/200 (67.50%) | 0.05848 | 2/200 (1.00%) | 200/200 (100.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 135/200 (67.50%) | -0.28482 | 3/200 (1.50%) | 200/200 (100.00%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 160/200 (80.00%) | 1.08658 | 2/200 (1.00%) | 188/200 (94.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 134/200 (67.00%) | -1.41675 | 2/200 (1.00%) | 199/200 (99.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 141/200 (70.50%) | -1.96860 | 5/200 (2.50%) | 200/200 (100.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 152/200 (76.00%) | -0.95838 | 3/200 (1.50%) | 200/200 (100.00%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 190/200 (95.00%) | -0.87410 | 4/200 (2.00%) | 199/200 (99.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 131/200 (65.50%) | 0.24886 | 2/200 (1.00%) | 199/200 (99.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 136/200 (68.00%) | -0.42355 | 2/200 (1.00%) | 200/200 (100.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 153/200 (76.50%) | -0.34067 | 4/200 (2.00%) | 200/200 (100.00%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 1/200 (0.50%) | 197/200 (98.50%) | 0.11322 | 4/200 (2.00%) | 199/200 (99.50%) |

## Test set, transform noise-a10: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 103/200 (51.50%) +0.00 | 164/200 (82.00%) +0.00 | 182, 16 (91.00%, 8.00%) |
| Tintype sensitive | 0/400 (0.00%) | 88/200 (44.00%) -7.50 | 130/200 (65.00%) -17.00 | 192, 72 (96.00%, 36.00%) |
| Tintype strict | 0/400 (0.00%) | 140/200 (70.00%) +18.50 | 167/200 (83.50%) +1.50 | 168, 3 (84.00%, 1.50%) |
| tintype-views | 0/400 (0.00%) | 147/200 (73.50%) +22.00 | 164/200 (82.00%) +0.00 | 182, 22 (91.00%, 11.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 212/400 (53.00%) | 115/200 (57.50%) +6.00 | 155/200 (77.50%) -4.50 | 195, 27 (97.50%, 13.50%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 90/400 (22.50%) | 112/200 (56.00%) +4.50 | 152/200 (76.00%) -6.00 | 190, 23 (95.00%, 11.50%) |
| Community Forensics alone | 0/400 (0.00%) | 83/200 (41.50%) -10.00 | 107/200 (53.50%) -28.50 | 151, 26 (75.50%, 13.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 33/400 (8.25%) | 112/200 (56.00%) +4.50 | 165/200 (82.50%) +0.50 | 183, 10 (91.50%, 5.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 75/400 (18.75%) | 118/200 (59.00%) +7.50 | 158/200 (79.00%) -3.00 | 184, 9 (92.00%, 4.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 138/400 (34.50%) | 121/200 (60.50%) +9.00 | 168/200 (84.00%) +2.00 | 180, 10 (90.00%, 5.00%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 110/200 (55.00%) +3.50 | 138/200 (69.00%) -13.00 | 168, 9 (84.00%, 4.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 33/400 (8.25%) | 104/200 (52.00%) +0.50 | 173/200 (86.50%) +4.50 | 188, 31 (94.00%, 15.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 75/400 (18.75%) | 109/200 (54.50%) +3.00 | 179/200 (89.50%) +7.50 | 187, 16 (93.50%, 8.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 138/400 (34.50%) | 52/200 (26.00%) -25.50 | 175/200 (87.50%) +5.50 | 189, 23 (94.50%, 11.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 110/200 (55.00%) +3.50 | 155/200 (77.50%) -4.50 | 190, 26 (95.00%, 13.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: tintype-views (147/200); 1% FPR: Tintype strict (167/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 1/200 (0.50%) | 108/200 (54.00%) | -0.94908 | 25/200 (12.50%) | 185/200 (92.50%) |
| Tintype sensitive | 7.53853 | 1/200 (0.50%) | 91/200 (45.50%) | 2.69947 | 14/200 (7.00%) | 182/200 (91.00%) |
| Tintype strict | -0.52927 | 7/200 (3.50%) | 176/200 (88.00%) | -1.52022 | 23/200 (11.50%) | 185/200 (92.50%) |
| tintype-views | 0.84512 | 9/200 (4.50%) | 171/200 (85.50%) | -0.46940 | 31/200 (15.50%) | 185/200 (92.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 1/200 (0.50%) | 120/200 (60.00%) | 0.05848 | 27/200 (13.50%) | 195/200 (97.50%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 1/200 (0.50%) | 117/200 (58.50%) | -0.28482 | 27/200 (13.50%) | 190/200 (95.00%) |
| Community Forensics alone | 4.40448 | 3/200 (1.50%) | 113/200 (56.50%) | 1.08658 | 18/200 (9.00%) | 145/200 (72.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 1/200 (0.50%) | 117/200 (58.50%) | -1.41675 | 19/200 (9.50%) | 187/200 (93.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 1/200 (0.50%) | 123/200 (61.50%) | -1.96860 | 26/200 (13.00%) | 188/200 (94.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 2/200 (1.00%) | 135/200 (67.50%) | -0.95838 | 23/200 (11.50%) | 185/200 (92.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 3/200 (1.50%) | 149/200 (74.50%) | -0.87410 | 24/200 (12.00%) | 180/200 (90.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 1/200 (0.50%) | 109/200 (54.50%) | 0.24886 | 22/200 (11.00%) | 188/200 (94.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 1/200 (0.50%) | 114/200 (57.00%) | -0.42355 | 22/200 (11.00%) | 188/200 (94.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 2/200 (1.00%) | 134/200 (67.00%) | -0.34067 | 27/200 (13.50%) | 190/200 (95.00%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 3/200 (1.50%) | 165/200 (82.50%) | 0.11322 | 26/200 (13.00%) | 190/200 (95.00%) |

## Test set, transform noise-a40: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 0/200 (0.00%) +0.00 | 3/200 (1.50%) +0.00 | 37, 12 (18.50%, 6.00%) |
| Tintype sensitive | 0/400 (0.00%) | 0/200 (0.00%) +0.00 | 3/200 (1.50%) +0.00 | 68, 37 (34.00%, 18.50%) |
| Tintype strict | 0/400 (0.00%) | 5/200 (2.50%) +2.50 | 8/200 (4.00%) +2.50 | 13, 3 (6.50%, 1.50%) |
| tintype-views | 0/400 (0.00%) | 0/200 (0.00%) +0.00 | 2/200 (1.00%) -0.50 | 25, 14 (12.50%, 7.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 356/400 (89.00%) | 0/200 (0.00%) +0.00 | 3/200 (1.50%) +0.00 | 40, 8 (20.00%, 4.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 131/400 (32.75%) | 0/200 (0.00%) +0.00 | 3/200 (1.50%) +0.00 | 37, 8 (18.50%, 4.00%) |
| Community Forensics alone | 0/400 (0.00%) | 37/200 (18.50%) +18.50 | 56/200 (28.00%) +26.50 | 30, 0 (15.00%, 0.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 32/400 (8.00%) | 1/200 (0.50%) +0.50 | 4/200 (2.00%) +0.50 | 34, 8 (17.00%, 4.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 85/400 (21.25%) | 2/200 (1.00%) +1.00 | 5/200 (2.50%) +1.00 | 32, 3 (16.00%, 1.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 160/400 (40.00%) | 2/200 (1.00%) +1.00 | 5/200 (2.50%) +1.00 | 25, 3 (12.50%, 1.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 41/200 (20.50%) +20.50 | 66/200 (33.00%) +31.50 | 25, 0 (12.50%, 0.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 32/400 (8.00%) | 0/200 (0.00%) +0.00 | 3/200 (1.50%) +0.00 | 59, 9 (29.50%, 4.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 85/400 (21.25%) | 1/200 (0.50%) +0.50 | 4/200 (2.00%) +0.50 | 37, 3 (18.50%, 1.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 160/400 (40.00%) | 1/200 (0.50%) +0.50 | 5/200 (2.50%) +1.00 | 37, 4 (18.50%, 2.00%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 32/200 (16.00%) +16.00 | 38/200 (19.00%) +17.50 | 38, 2 (19.00%, 1.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (5/200); 1% FPR: Tintype strict (8/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 2/200 (1.00%) | 3/200 (1.50%) | -0.94908 | 16/200 (8.00%) | 50/200 (25.00%) |
| Tintype sensitive | 7.53853 | 2/200 (1.00%) | 3/200 (1.50%) | 2.69947 | 20/200 (10.00%) | 56/200 (28.00%) |
| Tintype strict | -0.52927 | 3/200 (1.50%) | 17/200 (8.50%) | -1.52022 | 12/200 (6.00%) | 64/200 (32.00%) |
| tintype-views | 0.84512 | 7/200 (3.50%) | 14/200 (7.00%) | -0.46940 | 17/200 (8.50%) | 34/200 (17.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 2/200 (1.00%) | 3/200 (1.50%) | 0.05848 | 8/200 (4.00%) | 40/200 (20.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 2/200 (1.00%) | 3/200 (1.50%) | -0.28482 | 8/200 (4.00%) | 38/200 (19.00%) |
| Community Forensics alone | 4.40448 | 0/200 (0.00%) | 6/200 (3.00%) | 1.08658 | 0/200 (0.00%) | 20/200 (10.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 2/200 (1.00%) | 4/200 (2.00%) | -1.41675 | 9/200 (4.50%) | 52/200 (26.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 2/200 (1.00%) | 5/200 (2.50%) | -1.96860 | 5/200 (2.50%) | 48/200 (24.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 2/200 (1.00%) | 5/200 (2.50%) | -0.95838 | 3/200 (1.50%) | 33/200 (16.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 0/200 (0.00%) | 12/200 (6.00%) | -0.87410 | 0/200 (0.00%) | 34/200 (17.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 2/200 (1.00%) | 3/200 (1.50%) | 0.24886 | 9/200 (4.50%) | 52/200 (26.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 2/200 (1.00%) | 4/200 (2.00%) | -0.42355 | 7/200 (3.50%) | 40/200 (20.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 2/200 (1.00%) | 5/200 (2.50%) | -0.34067 | 6/200 (3.00%) | 43/200 (21.50%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/200 (0.00%) | 15/200 (7.50%) | 0.11322 | 2/200 (1.00%) | 37/200 (18.50%) |

## Test set, transform sharpen-a1.0: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 188/200 (94.00%) +0.00 | 194/200 (97.00%) +0.00 | 189, 1 (94.50%, 0.50%) |
| Tintype sensitive | 0/400 (0.00%) | 189/200 (94.50%) +0.50 | 192/200 (96.00%) -1.00 | 195, 9 (97.50%, 4.50%) |
| Tintype strict | 0/400 (0.00%) | 187/200 (93.50%) -0.50 | 193/200 (96.50%) -0.50 | 179, 0 (89.50%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 188/200 (94.00%) +0.00 | 188/200 (94.00%) -3.00 | 189, 3 (94.50%, 1.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 208/400 (52.00%) | 185/200 (92.50%) -1.50 | 199/200 (99.50%) +2.50 | 198, 1 (99.00%, 0.50%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 31/400 (7.75%) | 184/200 (92.00%) -2.00 | 196/200 (98.00%) +1.00 | 196, 1 (98.00%, 0.50%) |
| Community Forensics alone | 0/400 (0.00%) | 151/200 (75.50%) -18.50 | 175/200 (87.50%) -9.50 | 174, 1 (87.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 8/400 (2.00%) | 194/200 (97.00%) +3.00 | 194/200 (97.00%) +0.00 | 192, 0 (96.00%, 0.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 27/400 (6.75%) | 192/200 (96.00%) +2.00 | 194/200 (97.00%) +0.00 | 193, 1 (96.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 64/400 (16.00%) | 188/200 (94.00%) +0.00 | 194/200 (97.00%) +0.00 | 192, 1 (96.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 178/200 (89.00%) -5.00 | 197/200 (98.50%) +1.50 | 188, 1 (94.00%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 8/400 (2.00%) | 194/200 (97.00%) +3.00 | 194/200 (97.00%) +0.00 | 194, 2 (97.00%, 1.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 27/400 (6.75%) | 195/200 (97.50%) +3.50 | 196/200 (98.00%) +1.00 | 195, 1 (97.50%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 64/400 (16.00%) | 194/200 (97.00%) +3.00 | 196/200 (98.00%) +1.00 | 196, 1 (98.00%, 0.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 193/200 (96.50%) +2.50 | 198/200 (99.00%) +2.00 | 197, 1 (98.50%, 0.50%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype sensitive (189/200); 1% FPR: Tintype default (194/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 139/200 (69.50%) | -0.94908 | 1/200 (0.50%) | 192/200 (96.00%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 129/200 (64.50%) | 2.69947 | 0/200 (0.00%) | 186/200 (93.00%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 183/200 (91.50%) | -1.52022 | 1/200 (0.50%) | 193/200 (96.50%) |
| tintype-views | 0.84512 | 0/200 (0.00%) | 186/200 (93.00%) | -0.46940 | 5/200 (2.50%) | 191/200 (95.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 143/200 (71.50%) | 0.05848 | 1/200 (0.50%) | 198/200 (99.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 143/200 (71.50%) | -0.28482 | 1/200 (0.50%) | 196/200 (98.00%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 151/200 (75.50%) | 1.08658 | 1/200 (0.50%) | 168/200 (84.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 145/200 (72.50%) | -1.41675 | 2/200 (1.00%) | 194/200 (97.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 151/200 (75.50%) | -1.96860 | 1/200 (0.50%) | 193/200 (96.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 166/200 (83.00%) | -0.95838 | 1/200 (0.50%) | 193/200 (96.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 0/200 (0.00%) | 177/200 (88.50%) | -0.87410 | 1/200 (0.50%) | 194/200 (97.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 142/200 (71.00%) | 0.24886 | 2/200 (1.00%) | 194/200 (97.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 146/200 (73.00%) | -0.42355 | 1/200 (0.50%) | 195/200 (97.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 164/200 (82.00%) | -0.34067 | 1/200 (0.50%) | 196/200 (98.00%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/200 (0.00%) | 185/200 (92.50%) | 0.11322 | 1/200 (0.50%) | 196/200 (98.00%) |

## Test set, transform repost-720-qv9x1: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 130/200 (65.00%) +0.00 | 146/200 (73.00%) +0.00 | 146, 2 (73.00%, 1.00%) |
| Tintype sensitive | 0/400 (0.00%) | 122/200 (61.00%) -4.00 | 131/200 (65.50%) -7.50 | 182, 16 (91.00%, 8.00%) |
| Tintype strict | 0/400 (0.00%) | 123/200 (61.50%) -3.50 | 150/200 (75.00%) +2.00 | 117, 0 (58.50%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 123/200 (61.50%) -3.50 | 147/200 (73.50%) +0.50 | 147, 2 (73.50%, 1.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 263/400 (65.75%) | 141/200 (70.50%) +5.50 | 144/200 (72.00%) -1.00 | 157, 5 (78.50%, 2.50%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 105/400 (26.25%) | 140/200 (70.00%) +5.00 | 143/200 (71.50%) -1.50 | 156, 5 (78.00%, 2.50%) |
| Community Forensics alone | 0/400 (0.00%) | 90/200 (45.00%) -20.00 | 93/200 (46.50%) -26.50 | 117, 7 (58.50%, 3.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 43/400 (10.75%) | 139/200 (69.50%) +4.50 | 147/200 (73.50%) +0.50 | 144, 1 (72.00%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 97/400 (24.25%) | 130/200 (65.00%) +0.00 | 148/200 (74.00%) +1.00 | 138, 2 (69.00%, 1.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 154/400 (38.50%) | 124/200 (62.00%) -3.00 | 132/200 (66.00%) -7.00 | 133, 3 (66.50%, 1.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 115/200 (57.50%) -7.50 | 127/200 (63.50%) -9.50 | 128, 3 (64.00%, 1.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 43/400 (10.75%) | 140/200 (70.00%) +5.00 | 147/200 (73.50%) +0.50 | 163, 15 (81.50%, 7.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 97/400 (24.25%) | 137/200 (68.50%) +3.50 | 146/200 (73.00%) +0.00 | 156, 4 (78.00%, 2.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 154/400 (38.50%) | 138/200 (69.00%) +4.00 | 142/200 (71.00%) -2.00 | 164, 7 (82.00%, 3.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 122/200 (61.00%) -4.00 | 145/200 (72.50%) -0.50 | 163, 10 (81.50%, 5.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype default (130/200); 1% FPR: Tintype strict (150/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 70/200 (35.00%) | -0.94908 | 8/200 (4.00%) | 165/200 (82.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 57/200 (28.50%) | 2.69947 | 4/200 (2.00%) | 145/200 (72.50%) |
| Tintype strict | -0.52927 | 1/200 (0.50%) | 134/200 (67.00%) | -1.52022 | 11/200 (5.50%) | 166/200 (83.00%) |
| tintype-views | 0.84512 | 0/200 (0.00%) | 122/200 (61.00%) | -0.46940 | 6/200 (3.00%) | 155/200 (77.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 73/200 (36.50%) | 0.05848 | 4/200 (2.00%) | 156/200 (78.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 73/200 (36.50%) | -0.28482 | 5/200 (2.50%) | 158/200 (79.00%) |
| Community Forensics alone | 4.40448 | 0/200 (0.00%) | 73/200 (36.50%) | 1.08658 | 5/200 (2.50%) | 100/200 (50.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 74/200 (37.00%) | -1.41675 | 11/200 (5.50%) | 163/200 (81.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 76/200 (38.00%) | -1.96860 | 12/200 (6.00%) | 165/200 (82.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 83/200 (41.50%) | -0.95838 | 9/200 (4.50%) | 149/200 (74.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 0/200 (0.00%) | 107/200 (53.50%) | -0.87410 | 9/200 (4.50%) | 147/200 (73.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 70/200 (35.00%) | 0.24886 | 10/200 (5.00%) | 159/200 (79.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 72/200 (36.00%) | -0.42355 | 8/200 (4.00%) | 163/200 (81.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 85/200 (42.50%) | -0.34067 | 10/200 (5.00%) | 166/200 (83.00%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/200 (0.00%) | 113/200 (56.50%) | 0.11322 | 10/200 (5.00%) | 161/200 (80.50%) |

## Test set, transform repost-960-qv7x4: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 160/200 (80.00%) +0.00 | 166/200 (83.00%) +0.00 | 157, 0 (78.50%, 0.00%) |
| Tintype sensitive | 0/400 (0.00%) | 88/200 (44.00%) -36.00 | 158/200 (79.00%) -4.00 | 182, 11 (91.00%, 5.50%) |
| Tintype strict | 0/400 (0.00%) | 158/200 (79.00%) -1.00 | 162/200 (81.00%) -2.00 | 132, 0 (66.00%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 159/200 (79.50%) -0.50 | 165/200 (82.50%) -0.50 | 156, 0 (78.00%, 0.00%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 250/400 (62.50%) | 118/200 (59.00%) -21.00 | 184/200 (92.00%) +9.00 | 177, 2 (88.50%, 1.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 87/400 (21.75%) | 118/200 (59.00%) -21.00 | 183/200 (91.50%) +8.50 | 176, 2 (88.00%, 1.00%) |
| Community Forensics alone | 0/400 (0.00%) | 73/200 (36.50%) -43.50 | 132/200 (66.00%) -17.00 | 143, 3 (71.50%, 1.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 35/400 (8.75%) | 167/200 (83.50%) +3.50 | 171/200 (85.50%) +2.50 | 162, 0 (81.00%, 0.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 74/400 (18.50%) | 138/200 (69.00%) -11.00 | 178/200 (89.00%) +6.00 | 165, 1 (82.50%, 0.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 139/400 (34.75%) | 131/200 (65.50%) -14.50 | 174/200 (87.00%) +4.00 | 162, 2 (81.00%, 1.00%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 111/200 (55.50%) -24.50 | 172/200 (86.00%) +3.00 | 157, 2 (78.50%, 1.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 35/400 (8.75%) | 169/200 (84.50%) +4.50 | 174/200 (87.00%) +4.00 | 176, 5 (88.00%, 2.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 74/400 (18.50%) | 164/200 (82.00%) +2.00 | 177/200 (88.50%) +5.50 | 176, 2 (88.00%, 1.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 139/400 (34.75%) | 154/200 (77.00%) -3.00 | 179/200 (89.50%) +6.50 | 181, 4 (90.50%, 2.00%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 138/200 (69.00%) -11.00 | 176/200 (88.00%) +5.00 | 182, 4 (91.00%, 2.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype default (160/200); 1% FPR: Tintype default (166/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 77/200 (38.50%) | -0.94908 | 7/200 (3.50%) | 173/200 (86.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 67/200 (33.50%) | 2.69947 | 2/200 (1.00%) | 158/200 (79.00%) |
| Tintype strict | -0.52927 | 0/200 (0.00%) | 147/200 (73.50%) | -1.52022 | 6/200 (3.00%) | 173/200 (86.50%) |
| tintype-views | 0.84512 | 0/200 (0.00%) | 139/200 (69.50%) | -0.46940 | 2/200 (1.00%) | 165/200 (82.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 77/200 (38.50%) | 0.05848 | 2/200 (1.00%) | 177/200 (88.50%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 77/200 (38.50%) | -0.28482 | 2/200 (1.00%) | 180/200 (90.00%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 90/200 (45.00%) | 1.08658 | 2/200 (1.00%) | 125/200 (62.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 79/200 (39.50%) | -1.41675 | 4/200 (2.00%) | 174/200 (87.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 81/200 (40.50%) | -1.96860 | 5/200 (2.50%) | 181/200 (90.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 92/200 (46.00%) | -0.95838 | 3/200 (1.50%) | 174/200 (87.00%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 121/200 (60.50%) | -0.87410 | 3/200 (1.50%) | 172/200 (86.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 77/200 (38.50%) | 0.24886 | 2/200 (1.00%) | 173/200 (86.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 78/200 (39.00%) | -0.42355 | 4/200 (2.00%) | 178/200 (89.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 95/200 (47.50%) | -0.34067 | 5/200 (2.50%) | 181/200 (90.50%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 1/200 (0.50%) | 140/200 (70.00%) | 0.11322 | 4/200 (2.00%) | 180/200 (90.00%) |

## Test set, transform repost-960-qv12x1: 400 photos, 200 generated, 200 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 2 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/400 (0.00%) | 111/200 (55.50%) +0.00 | 135/200 (67.50%) +0.00 | 137, 3 (68.50%, 1.50%) |
| Tintype sensitive | 0/400 (0.00%) | 110/200 (55.00%) -0.50 | 127/200 (63.50%) -4.00 | 171, 14 (85.50%, 7.00%) |
| Tintype strict | 0/400 (0.00%) | 112/200 (56.00%) +0.50 | 135/200 (67.50%) +0.00 | 111, 0 (55.50%, 0.00%) |
| tintype-views | 0/400 (0.00%) | 108/200 (54.00%) -1.50 | 135/200 (67.50%) +0.00 | 137, 3 (68.50%, 1.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 269/400 (67.25%) | 81/200 (40.50%) -15.00 | 116/200 (58.00%) -9.50 | 142, 4 (71.00%, 2.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 111/400 (27.75%) | 81/200 (40.50%) -15.00 | 116/200 (58.00%) -9.50 | 142, 4 (71.00%, 2.00%) |
| Community Forensics alone | 0/400 (0.00%) | 30/200 (15.00%) -40.50 | 85/200 (42.50%) -25.00 | 94, 6 (47.00%, 3.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 41/400 (10.25%) | 118/200 (59.00%) +3.50 | 134/200 (67.00%) -0.50 | 132, 2 (66.00%, 1.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 97/400 (24.25%) | 93/200 (46.50%) -9.00 | 128/200 (64.00%) -3.50 | 127, 2 (63.50%, 1.00%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 157/400 (39.25%) | 98/200 (49.00%) -6.50 | 124/200 (62.00%) -5.50 | 123, 2 (61.50%, 1.00%) |
| Tintype default + Community Forensics, 50:50 full fusion | 400/400 (100.00%) | 68/200 (34.00%) -21.50 | 118/200 (59.00%) -8.50 | 116, 2 (58.00%, 1.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 41/400 (10.25%) | 115/200 (57.50%) +2.00 | 133/200 (66.50%) -1.00 | 154, 14 (77.00%, 7.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 97/400 (24.25%) | 117/200 (58.50%) +3.00 | 128/200 (64.00%) -3.50 | 144, 4 (72.00%, 2.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 157/400 (39.25%) | 112/200 (56.00%) +0.50 | 129/200 (64.50%) -3.00 | 149, 5 (74.50%, 2.50%) |
| Tintype default + Community Forensics, fitted full fusion | 400/400 (100.00%) | 93/200 (46.50%) -9.00 | 127/200 (63.50%) -4.00 | 148, 6 (74.00%, 3.00%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the test set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (112/200); 1% FPR: Tintype default (135/200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 0/200 (0.00%) | 62/200 (31.00%) | -0.94908 | 6/200 (3.00%) | 159/200 (79.50%) |
| Tintype sensitive | 7.53853 | 0/200 (0.00%) | 45/200 (22.50%) | 2.69947 | 3/200 (1.50%) | 136/200 (68.00%) |
| Tintype strict | -0.52927 | 2/200 (1.00%) | 128/200 (64.00%) | -1.52022 | 9/200 (4.50%) | 158/200 (79.00%) |
| tintype-views | 0.84512 | 2/200 (1.00%) | 111/200 (55.50%) | -0.46940 | 3/200 (1.50%) | 145/200 (72.50%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 63/200 (31.50%) | 0.05848 | 4/200 (2.00%) | 142/200 (71.00%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 0/200 (0.00%) | 63/200 (31.50%) | -0.28482 | 5/200 (2.50%) | 142/200 (71.00%) |
| Community Forensics alone | 4.40448 | 1/200 (0.50%) | 52/200 (26.00%) | 1.08658 | 2/200 (1.00%) | 83/200 (41.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 0/200 (0.00%) | 64/200 (32.00%) | -1.41675 | 11/200 (5.50%) | 149/200 (74.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 0/200 (0.00%) | 64/200 (32.00%) | -1.96860 | 7/200 (3.50%) | 151/200 (75.50%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 0/200 (0.00%) | 66/200 (33.00%) | -0.95838 | 5/200 (2.50%) | 137/200 (68.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/200 (0.50%) | 89/200 (44.50%) | -0.87410 | 5/200 (2.50%) | 133/200 (66.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 0/200 (0.00%) | 62/200 (31.00%) | 0.24886 | 11/200 (5.50%) | 147/200 (73.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 0/200 (0.00%) | 62/200 (31.00%) | -0.42355 | 6/200 (3.00%) | 147/200 (73.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 0/200 (0.00%) | 73/200 (36.50%) | -0.34067 | 7/200 (3.50%) | 151/200 (75.50%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 1/200 (0.50%) | 107/200 (53.50%) | 0.11322 | 6/200 (3.00%) | 148/200 (74.00%) |
