# Policy comparison

16216 photos scored by every system (commfor, tintype, tintype-sensitive, tintype-strict, tintype-views).

Split: 3759 fit, 12457 calibrate, 0 test photos; 0 scored photos are in no set and are left out.

Fusion weights fitted on the fit set (3759 photos) only:

- Tintype default + Community Forensics, fitted for |tintype logit| <= 1, fit set: +1.0455 x tintype logit +0.4025 x commfor logit +1.0873 from 115 photos inside the gate; outside it +1.1551 x tintype logit +1.9287 from 3759
- Tintype default + Community Forensics, fitted for |tintype logit| <= 2, fit set: +1.4738 x tintype logit +0.4368 x commfor logit +1.3727 from 302 photos inside the gate; outside it +1.1551 x tintype logit +1.9287 from 3759
- Tintype default + Community Forensics, fitted for |tintype logit| <= 3, fit set: +1.2910 x tintype logit +0.4896 x commfor logit +1.8924 from 606 photos inside the gate; outside it +1.1551 x tintype logit +1.9287 from 3759
- Tintype default + Community Forensics, fitted full fusion, fit set: +1.0404 x tintype logit +0.5716 x commfor logit +2.0343 from 3759 photos

Cutoffs are frozen on the `calibrate` set (12457 photos, 5857 authentic).
The `calibrate` set is reported here, so these cutoffs are in-sample: this is the view for choosing on `calibrate`, not a transfer result.

## Calibration set: 12457 photos, 6600 generated, 5857 authentic

Cutoffs chosen on the calibrate set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 5 FP), points vs Tintype default | Recall at 1% FPR (<= 58 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/12457 (0.00%) | 4076/6600 (61.76%) +0.00 | 5851/6600 (88.65%) +0.00 | 5673, 24 (85.95%, 0.41%) |
| Tintype sensitive | 0/12457 (0.00%) | 3796/6600 (57.52%) -4.24 | 5630/6600 (85.30%) -3.35 | 6067, 258 (91.92%, 4.40%) |
| Tintype strict | 0/12457 (0.00%) | 5498/6600 (83.30%) +21.55 | 5860/6600 (88.79%) +0.14 | 5302, 1 (80.33%, 0.02%) |
| tintype-views | 0/12457 (0.00%) | 5404/6600 (81.88%) +20.12 | 5779/6600 (87.56%) -1.09 | 5673, 26 (85.95%, 0.44%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 6752/12457 (54.20%) | 4364/6600 (66.12%) +4.36 | 6088/6600 (92.24%) +3.59 | 6092, 61 (92.30%, 1.04%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1363/12457 (10.94%) | 4303/6600 (65.20%) +3.44 | 5992/6600 (90.79%) +2.14 | 5974, 47 (90.52%, 0.80%) |
| Community Forensics alone | 0/12457 (0.00%) | 4255/6600 (64.47%) +2.71 | 5022/6600 (76.09%) -12.56 | 5261, 144 (79.71%, 2.46%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 454/12457 (3.64%) | 4245/6600 (64.32%) +2.56 | 5892/6600 (89.27%) +0.62 | 5722, 15 (86.70%, 0.26%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 1212/12457 (9.73%) | 4470/6600 (67.73%) +5.97 | 5918/6600 (89.67%) +1.02 | 5743, 19 (87.02%, 0.32%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 2406/12457 (19.31%) | 4748/6600 (71.94%) +10.18 | 5859/6600 (88.77%) +0.12 | 5734, 26 (86.88%, 0.44%) |
| Tintype default + Community Forensics, 50:50 full fusion | 12457/12457 (100.00%) | 5255/6600 (79.62%) +17.86 | 5908/6600 (89.52%) +0.86 | 5715, 23 (86.59%, 0.39%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 454/12457 (3.64%) | 4157/6600 (62.98%) +1.23 | 5896/6600 (89.33%) +0.68 | 5957, 99 (90.26%, 1.69%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 1212/12457 (9.73%) | 4355/6600 (65.98%) +4.23 | 5928/6600 (89.82%) +1.17 | 5898, 29 (89.36%, 0.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 2406/12457 (19.31%) | 4742/6600 (71.85%) +10.09 | 6025/6600 (91.29%) +2.64 | 6001, 45 (90.92%, 0.77%) |
| Tintype default + Community Forensics, fitted full fusion | 12457/12457 (100.00%) | 5560/6600 (84.24%) +22.48 | 6080/6600 (92.12%) +3.47 | 6089, 63 (92.26%, 1.08%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the calibrate set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (5498/6600); 1% FPR: Tintype strict (5860/6600).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 5/5857 (0.09%) | 4076/6600 (61.76%) | -0.94908 | 58/5857 (0.99%) | 5851/6600 (88.65%) |
| Tintype sensitive | 7.53853 | 5/5857 (0.09%) | 3796/6600 (57.52%) | 2.69947 | 58/5857 (0.99%) | 5630/6600 (85.30%) |
| Tintype strict | -0.52927 | 5/5857 (0.09%) | 5498/6600 (83.30%) | -1.52022 | 58/5857 (0.99%) | 5860/6600 (88.79%) |
| tintype-views | 0.84512 | 5/5857 (0.09%) | 5404/6600 (81.88%) | -0.46940 | 58/5857 (0.99%) | 5779/6600 (87.56%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 5/5857 (0.09%) | 4364/6600 (66.12%) | 0.05848 | 58/5857 (0.99%) | 6088/6600 (92.24%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 5/5857 (0.09%) | 4303/6600 (65.20%) | -0.28482 | 58/5857 (0.99%) | 5992/6600 (90.79%) |
| Community Forensics alone | 4.40448 | 5/5857 (0.09%) | 4255/6600 (64.47%) | 1.08658 | 58/5857 (0.99%) | 5022/6600 (76.09%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 5/5857 (0.09%) | 4245/6600 (64.32%) | -1.41675 | 58/5857 (0.99%) | 5892/6600 (89.27%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 5/5857 (0.09%) | 4470/6600 (67.73%) | -1.96860 | 58/5857 (0.99%) | 5918/6600 (89.67%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 5/5857 (0.09%) | 4748/6600 (71.94%) | -0.95838 | 58/5857 (0.99%) | 5859/6600 (88.77%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 5/5857 (0.09%) | 5255/6600 (79.62%) | -0.87410 | 58/5857 (0.99%) | 5908/6600 (89.52%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 5/5857 (0.09%) | 4157/6600 (62.98%) | 0.24886 | 58/5857 (0.99%) | 5896/6600 (89.33%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 5/5857 (0.09%) | 4355/6600 (65.98%) | -0.42355 | 58/5857 (0.99%) | 5928/6600 (89.82%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 5/5857 (0.09%) | 4742/6600 (71.85%) | -0.34067 | 58/5857 (0.99%) | 6025/6600 (91.29%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 5/5857 (0.09%) | 5560/6600 (84.24%) | 0.11322 | 58/5857 (0.99%) | 6080/6600 (92.12%) |

## Calibration set, processing original: 7657 photos, 4200 generated, 3457 authentic

Cutoffs chosen on the calibrate set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 3 FP), points vs Tintype default | Recall at 1% FPR (<= 34 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/7657 (0.00%) | 3877/4200 (92.31%) +0.00 | 3993/4200 (95.07%) +0.00 | 3867, 2 (92.07%, 0.06%) |
| Tintype sensitive | 0/7657 (0.00%) | 3001/4200 (71.45%) -20.86 | 3970/4200 (94.52%) -0.55 | 4026, 83 (95.86%, 2.40%) |
| Tintype strict | 0/7657 (0.00%) | 3852/4200 (91.71%) -0.60 | 3967/4200 (94.45%) -0.62 | 3708, 0 (88.29%, 0.00%) |
| tintype-views | 0/7657 (0.00%) | 3889/4200 (92.60%) +0.29 | 3980/4200 (94.76%) -0.31 | 3876, 1 (92.29%, 0.03%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 3774/7657 (49.29%) | 4060/4200 (96.67%) +4.36 | 4162/4200 (99.10%) +4.02 | 4154, 18 (98.90%, 0.52%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 560/7657 (7.31%) | 4062/4200 (96.71%) +4.40 | 4084/4200 (97.24%) +2.17 | 4071, 11 (96.93%, 0.32%) |
| Community Forensics alone | 0/7657 (0.00%) | 3280/4200 (78.10%) -14.21 | 3721/4200 (88.60%) -6.48 | 3804, 74 (90.57%, 2.14%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 204/7657 (2.66%) | 3941/4200 (93.83%) +1.52 | 4003/4200 (95.31%) +0.24 | 3935, 1 (93.69%, 0.03%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 549/7657 (7.17%) | 3984/4200 (94.86%) +2.55 | 4023/4200 (95.79%) +0.71 | 3979, 3 (94.74%, 0.09%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 1140/7657 (14.89%) | 4012/4200 (95.52%) +3.21 | 4073/4200 (96.98%) +1.90 | 4012, 4 (95.52%, 0.12%) |
| Tintype default + Community Forensics, 50:50 full fusion | 7657/7657 (100.00%) | 4063/4200 (96.74%) +4.43 | 4136/4200 (98.48%) +3.40 | 4064, 4 (96.76%, 0.12%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 204/7657 (2.66%) | 3939/4200 (93.79%) +1.48 | 4007/4200 (95.40%) +0.33 | 3995, 27 (95.12%, 0.78%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 549/7657 (7.17%) | 4011/4200 (95.50%) +3.19 | 4032/4200 (96.00%) +0.93 | 4013, 5 (95.55%, 0.14%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 1140/7657 (14.89%) | 4055/4200 (96.55%) +4.24 | 4083/4200 (97.21%) +2.14 | 4074, 11 (97.00%, 0.32%) |
| Tintype default + Community Forensics, fitted full fusion | 7657/7657 (100.00%) | 4103/4200 (97.69%) +5.38 | 4164/4200 (99.14%) +4.07 | 4153, 18 (98.88%, 0.52%) |
- Tintype default: 33 of 34 allowed FPs used at 1% FPR (tied scores at the cutoff)
- Tintype default + Community Forensics, 50:50 full fusion: 33 of 34 allowed FPs used at 1% FPR (tied scores at the cutoff)
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the calibrate set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: tintype-views (3889/4200); 1% FPR: Tintype default (3993/4200).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 1/3457 (0.03%) | 2967/4200 (70.64%) | -0.94908 | 15/3457 (0.43%) | 3946/4200 (93.95%) |
| Tintype sensitive | 7.53853 | 1/3457 (0.03%) | 2813/4200 (66.98%) | 2.69947 | 13/3457 (0.38%) | 3823/4200 (91.02%) |
| Tintype strict | -0.52927 | 0/3457 (0.00%) | 3786/4200 (90.14%) | -1.52022 | 20/3457 (0.58%) | 3930/4200 (93.57%) |
| tintype-views | 0.84512 | 0/3457 (0.00%) | 3771/4200 (89.79%) | -0.46940 | 12/3457 (0.35%) | 3920/4200 (93.33%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 1/3457 (0.03%) | 3215/4200 (76.55%) | 0.05848 | 16/3457 (0.46%) | 4154/4200 (98.90%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 1/3457 (0.03%) | 3161/4200 (75.26%) | -0.28482 | 16/3457 (0.46%) | 4073/4200 (96.98%) |
| Community Forensics alone | 4.40448 | 2/3457 (0.06%) | 3259/4200 (77.60%) | 1.08658 | 24/3457 (0.69%) | 3685/4200 (87.74%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 1/3457 (0.03%) | 3110/4200 (74.05%) | -1.41675 | 17/3457 (0.49%) | 3977/4200 (94.69%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 1/3457 (0.03%) | 3293/4200 (78.40%) | -1.96860 | 20/3457 (0.58%) | 4013/4200 (95.55%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 1/3457 (0.03%) | 3502/4200 (83.38%) | -0.95838 | 19/3457 (0.55%) | 4047/4200 (96.36%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 1/3457 (0.03%) | 3861/4200 (91.93%) | -0.87410 | 21/3457 (0.61%) | 4116/4200 (98.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 1/3457 (0.03%) | 3046/4200 (72.52%) | 0.24886 | 17/3457 (0.49%) | 3981/4200 (94.79%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 1/3457 (0.03%) | 3206/4200 (76.33%) | -0.42355 | 16/3457 (0.46%) | 4020/4200 (95.71%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 1/3457 (0.03%) | 3483/4200 (82.93%) | -0.34067 | 18/3457 (0.52%) | 4077/4200 (97.07%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 0/3457 (0.00%) | 3999/4200 (95.21%) | 0.11322 | 17/3457 (0.49%) | 4151/4200 (98.83%) |

## Calibration set, processing processed: 4800 photos, 2400 generated, 2400 authentic

Cutoffs chosen on the calibrate set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 2 FP), points vs Tintype default | Recall at 1% FPR (<= 24 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/4800 (0.00%) | 1072/2400 (44.67%) +0.00 | 1817/2400 (75.71%) +0.00 | 1806, 22 (75.25%, 0.92%) |
| Tintype sensitive | 0/4800 (0.00%) | 962/2400 (40.08%) -4.58 | 1334/2400 (55.58%) -20.12 | 2041, 175 (85.04%, 7.29%) |
| Tintype strict | 0/4800 (0.00%) | 1650/2400 (68.75%) +24.08 | 1898/2400 (79.08%) +3.38 | 1594, 1 (66.42%, 0.04%) |
| tintype-views | 0/4800 (0.00%) | 1579/2400 (65.79%) +21.12 | 1792/2400 (74.67%) -1.04 | 1797, 25 (74.88%, 1.04%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 2978/4800 (62.04%) | 1110/2400 (46.25%) +1.58 | 1875/2400 (78.12%) +2.42 | 1938, 43 (80.75%, 1.79%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 803/4800 (16.73%) | 1103/2400 (45.96%) +1.29 | 1845/2400 (76.88%) +1.17 | 1903, 36 (79.29%, 1.50%) |
| Community Forensics alone | 0/4800 (0.00%) | 927/2400 (38.62%) -6.04 | 1257/2400 (52.38%) -23.33 | 1457, 70 (60.71%, 2.92%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 250/4800 (5.21%) | 1095/2400 (45.62%) +0.96 | 1847/2400 (76.96%) +1.25 | 1787, 14 (74.46%, 0.58%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 663/4800 (13.81%) | 1137/2400 (47.38%) +2.71 | 1836/2400 (76.50%) +0.79 | 1764, 16 (73.50%, 0.67%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 1266/4800 (26.38%) | 1205/2400 (50.21%) +5.54 | 1750/2400 (72.92%) -2.79 | 1722, 22 (71.75%, 0.92%) |
| Tintype default + Community Forensics, 50:50 full fusion | 4800/4800 (100.00%) | 1187/2400 (49.46%) +4.79 | 1717/2400 (71.54%) -4.17 | 1651, 19 (68.79%, 0.79%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 250/4800 (5.21%) | 1074/2400 (44.75%) +0.08 | 1833/2400 (76.38%) +0.67 | 1962, 72 (81.75%, 3.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 663/4800 (13.81%) | 1112/2400 (46.33%) +1.67 | 1885/2400 (78.54%) +2.83 | 1885, 24 (78.54%, 1.00%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 1266/4800 (26.38%) | 1220/2400 (50.83%) +6.17 | 1889/2400 (78.71%) +3.00 | 1927, 34 (80.29%, 1.42%) |
| Tintype default + Community Forensics, fitted full fusion | 4800/4800 (100.00%) | 1314/2400 (54.75%) +10.08 | 1856/2400 (77.33%) +1.62 | 1936, 45 (80.67%, 1.88%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on the calibrate set itself (a ranking comparison, not a frozen operating point): 0.1% FPR: Tintype strict (1650/2400); 1% FPR: Tintype strict (1898/2400).

Cutoffs frozen on the calibration set (5857 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 4.00855 | 4/2400 (0.17%) | 1109/2400 (46.21%) | -0.94908 | 43/2400 (1.79%) | 1905/2400 (79.38%) |
| Tintype sensitive | 7.53853 | 4/2400 (0.17%) | 983/2400 (40.96%) | 2.69947 | 45/2400 (1.88%) | 1807/2400 (75.29%) |
| Tintype strict | -0.52927 | 5/2400 (0.21%) | 1712/2400 (71.33%) | -1.52022 | 38/2400 (1.58%) | 1930/2400 (80.42%) |
| tintype-views | 0.84512 | 5/2400 (0.21%) | 1633/2400 (68.04%) | -0.46940 | 46/2400 (1.92%) | 1859/2400 (77.46%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 4.00855 | 4/2400 (0.17%) | 1149/2400 (47.88%) | 0.05848 | 42/2400 (1.75%) | 1934/2400 (80.58%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4.00855 | 4/2400 (0.17%) | 1142/2400 (47.58%) | -0.28482 | 42/2400 (1.75%) | 1919/2400 (79.96%) |
| Community Forensics alone | 4.40448 | 3/2400 (0.12%) | 996/2400 (41.50%) | 1.08658 | 34/2400 (1.42%) | 1337/2400 (55.71%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 1 | 4.00855 | 4/2400 (0.17%) | 1135/2400 (47.29%) | -1.41675 | 41/2400 (1.71%) | 1915/2400 (79.79%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4.00855 | 4/2400 (0.17%) | 1177/2400 (49.04%) | -1.96860 | 38/2400 (1.58%) | 1905/2400 (79.38%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 3 | 4.00855 | 4/2400 (0.17%) | 1246/2400 (51.92%) | -0.95838 | 39/2400 (1.62%) | 1812/2400 (75.50%) |
| Tintype default + Community Forensics, 50:50 full fusion | 1.65208 | 4/2400 (0.17%) | 1394/2400 (58.08%) | -0.87410 | 37/2400 (1.54%) | 1792/2400 (74.67%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 1 | 6.55894 | 4/2400 (0.17%) | 1111/2400 (46.29%) | 0.24886 | 41/2400 (1.71%) | 1915/2400 (79.79%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 6.55894 | 4/2400 (0.17%) | 1149/2400 (47.88%) | -0.42355 | 42/2400 (1.75%) | 1908/2400 (79.50%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 3 | 6.55894 | 4/2400 (0.17%) | 1259/2400 (52.46%) | -0.34067 | 40/2400 (1.67%) | 1948/2400 (81.17%) |
| Tintype default + Community Forensics, fitted full fusion | 3.68072 | 5/2400 (0.21%) | 1561/2400 (65.04%) | 0.11322 | 41/2400 (1.71%) | 1929/2400 (80.38%) |
