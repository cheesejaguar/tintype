# Policy comparison on the confirm7 archive (exploratory)

These archived suites were used to develop the detector. Cutoffs are chosen on the cohort they are reported on: a ranking comparison, not a transfer result and not a decision about what ships. Routed shares are measured workload; end-to-end savings depend on the cohort and the hardware.

Fusion weights cross-fitted: 5 folds of whole `dataset` groups (30 groups), each fold scored with weights fitted on the other folds. Exploratory.

- Tintype default + Community Forensics, fitted for |tintype logit| <= 2, fold 1: +0.8843 x tintype logit +0.2443 x commfor logit +1.9825 from 2354 photos inside the gate; outside it +1.0249 x tintype logit +1.6652 from 19161
- Tintype default + Community Forensics, fitted for |tintype logit| <= 2, fold 2: +0.9216 x tintype logit +0.2676 x commfor logit +1.3767 from 1495 photos inside the gate; outside it +1.0701 x tintype logit +1.3975 from 15812
- Tintype default + Community Forensics, fitted for |tintype logit| <= 2, fold 3: +0.9872 x tintype logit +0.2039 x commfor logit +1.6751 from 2117 photos inside the gate; outside it +1.0009 x tintype logit +1.5065 from 18758
- Tintype default + Community Forensics, fitted for |tintype logit| <= 2, fold 4: +1.0392 x tintype logit +0.2424 x commfor logit +2.3047 from 2119 photos inside the gate; outside it +1.0572 x tintype logit +1.8539 from 17551
- Tintype default + Community Forensics, fitted for |tintype logit| <= 2, fold 5: +1.0064 x tintype logit +0.2066 x commfor logit +1.5920 from 2211 photos inside the gate; outside it +1.0460 x tintype logit +1.4111 from 20926
- Tintype default + Community Forensics, fitted full fusion, fold 1: +0.8185 x tintype logit +0.2409 x commfor logit +2.0475 from 19161 photos
- Tintype default + Community Forensics, fitted full fusion, fold 2: +0.8387 x tintype logit +0.2512 x commfor logit +1.4687 from 15812 photos
- Tintype default + Community Forensics, fitted full fusion, fold 3: +0.8225 x tintype logit +0.2055 x commfor logit +1.7820 from 18758 photos
- Tintype default + Community Forensics, fitted full fusion, fold 4: +0.8493 x tintype logit +0.2346 x commfor logit +2.2520 from 17551 photos
- Tintype default + Community Forensics, fitted full fusion, fold 5: +0.8585 x tintype logit +0.2150 x commfor logit +1.6626 from 20926 photos

## All photos: 23052 photos, 12305 generated, 10747 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 10 FP), points vs Tintype default | Recall at 1% FPR (<= 107 FP), points vs Tintype default | Recall at 107 FP (1.00% FPR) | Recall at 329 FP (3.06% FPR) | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 0/23052 (0.00%) | 7174/12305 (58.30%) +0.00 | 10887/12305 (88.48%) +0.00 | 10887/12305 (88.48%) | 11468/12305 (93.20%) | 11038, 130 (89.70%, 1.21%) |
| Tintype sensitive | 0/23052 (0.00%) | 6572/12305 (53.41%) -4.89 | 10608/12305 (86.21%) -2.27 | 10608/12305 (86.21%) | 11670/12305 (94.84%) | 11812, 459 (95.99%, 4.27%) |
| Tintype strict | 0/23052 (0.00%) | 7759/12305 (63.06%) +4.75 | 10647/12305 (86.53%) -1.95 | 10647/12305 (86.53%) | 11238/12305 (91.33%) | 10135, 48 (82.36%, 0.45%) |
| Shipped CF blend (stored scores) | 0/23052 (0.00%) | 6754/12305 (54.89%) -3.41 | 10486/12305 (85.22%) -3.26 | 10486/12305 (85.22%) | 11504/12305 (93.49%) | 11504, 329 (93.49%, 3.06%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 11068/23052 (48.01%) | 6754/12305 (54.89%) -3.41 | 10486/12305 (85.22%) -3.26 | 10486/12305 (85.22%) | 11504/12305 (93.49%) | 11504, 329 (93.49%, 3.06%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 2417/23052 (10.48%) | 7382/12305 (59.99%) +1.69 | 10872/12305 (88.35%) -0.12 | 10872/12305 (88.35%) | 11584/12305 (94.14%) | 11413, 213 (92.75%, 1.98%) |
| Community Forensics alone | 0/23052 (0.00%) | 5535/12305 (44.98%) -13.32 | 7766/12305 (63.11%) -25.36 | 7766/12305 (63.11%) | 9059/12305 (73.62%) | 9707, 564 (78.89%, 5.25%) |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 2574/23052 (11.17%) | 7413/12305 (60.24%) +1.94 | 11014/12305 (89.51%) +1.03 | 11014/12305 (89.51%) | 11455/12305 (93.09%) | 10994, 102 (89.35%, 0.95%) |
| Tintype default + Community Forensics, 50:50 full fusion | 23052/23052 (100.00%) | 8110/12305 (65.91%) +7.61 | 10272/12305 (83.48%) -5.00 | 10272/12305 (83.48%) | 11223/12305 (91.21%) | 10668, 162 (86.70%, 1.51%) |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 2574/23052 (11.17%) | 7471/12305 (60.72%) +2.41 | 11029/12305 (89.63%) +1.15 | 11029/12305 (89.63%) | 11547/12305 (93.84%) | 11480, 270 (93.30%, 2.51%) |
| Tintype default + Community Forensics, fitted full fusion | 23052/23052 (100.00%) | 8219/12305 (66.79%) +8.49 | 11003/12305 (89.42%) +0.94 | 11003/12305 (89.42%) | 11552/12305 (93.88%) | 11663, 405 (94.78%, 3.77%) |
- Tintype default + Community Forensics, 50:50 full fusion: 328 of 329 allowed FPs used at 329 FP (tied scores at the cutoff)
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Strongest single-model profile at a re-chosen cutoff, chosen on this cohort, so exploratory: 0.1% FPR: Tintype strict (7759/12305); 1% FPR: Tintype default (10887/12305); 107 FP: Tintype default (10887/12305); 329 FP: Tintype sensitive (11670/12305).

Sets (of 30) where precision and recall both reach Community Forensics at its own cutoff:

| Policy | at 0.1% FPR | at 1% FPR | at 107 FP | at 329 FP | at own cutoff |
|---|---:|---:|---:|---:|---:|
| Tintype default | 4/30 | 12/30 | 12/30 | 14/30 | 12/30 |
| Tintype sensitive | 3/30 | 13/30 | 13/30 | 14/30 | 14/30 |
| Tintype strict | 5/30 | 12/30 | 12/30 | 12/30 | 12/30 |
| Shipped CF blend (stored scores) | 3/30 | 13/30 | 13/30 | 26/30 | 26/30 |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 3/30 | 13/30 | 13/30 | 26/30 | 26/30 |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4/30 | 14/30 | 14/30 | 20/30 | 21/30 |
| Community Forensics alone | 2/30 | 6/30 | 6/30 | 9/30 | 30/30 |
| Tintype default + Community Forensics, 50:50 for |tintype logit| <= 2 | 4/30 | 16/30 | 16/30 | 14/30 | 16/30 |
| Tintype default + Community Forensics, 50:50 full fusion | 3/30 | 22/30 | 22/30 | 28/30 | 25/30 |
| Tintype default + Community Forensics, fitted for |tintype logit| <= 2 | 4/30 | 15/30 | 15/30 | 15/30 | 15/30 |
| Tintype default + Community Forensics, fitted full fusion | 4/30 | 18/30 | 18/30 | 20/30 | 21/30 |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 1267 of 12305 generated photos (Community Forensics flags 598 of them) and flags 130 of 10747 authentic photos (Community Forensics clears 103).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 11068/23052 (48.01%) | 1266 | 1 | 0 | 99 | 78 |
| -3 < tintype logit < 0.5 | 2417/23052 (10.48%) | 938 | 329 | 120 | 99 | 78 |
| |tintype logit| <= 2 | 2574/23052 (11.17%) | 635 | 632 | 233 | 32 | 22 |
| |tintype logit| <= 1 (sweep) | 984/23052 (4.27%) | 335 | 932 | 389 | 73 | 57 |
| |tintype logit| <= 3 (sweep) | 5206/23052 (22.58%) | 938 | 329 | 120 | 14 | 11 |
| |tintype logit| <= 4 (sweep) | 7489/23052 (32.49%) | 1069 | 198 | 80 | 10 | 8 |
| |tintype logit| <= 6 (sweep) | 17513/23052 (75.97%) | 1256 | 11 | 3 | 1 | 1 |
| |tintype logit| <= 8 (sweep) | 22908/23052 (99.38%) | 1267 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.98, which routes 2541/23052 (11.02%); 90% of the misses needs |logit| <= 4.36, which routes 9307/23052 (40.37%); 100% of the misses needs |logit| <= 7.00, which routes 21529/23052 (93.39%).
