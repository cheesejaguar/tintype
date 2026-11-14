# Policy comparison on the arena archive (exploratory)

These archived suites were used to develop the detector. Cutoffs are chosen on the cohort they are reported on: a ranking comparison, not a transfer result and not a decision about what ships. Routed shares are measured workload; end-to-end savings depend on the cohort and the hardware.

7 repeated authentic originals are removed from each variant.

Fusion weights cross-fitted: 5 folds of whole `parent` groups (2031 groups), each fold scored with weights fitted on the other folds of cohort arena-jpeg-q4. Exploratory.

- Tintype default + GPTTrace, fitted for |tintype logit| <= 2, fold 1: +0.5413 x tintype logit +0.5274 x gpttrace logit +3.4143 from 299 photos inside the gate; outside it +0.9855 x tintype logit +2.1874 from 1614
- Tintype default + GPTTrace, fitted for |tintype logit| <= 2, fold 2: +0.5629 x tintype logit +0.4288 x gpttrace logit +2.9763 from 308 photos inside the gate; outside it +0.9422 x tintype logit +2.0228 from 1628
- Tintype default + GPTTrace, fitted for |tintype logit| <= 2, fold 3: +0.7402 x tintype logit +0.5260 x gpttrace logit +3.5746 from 304 photos inside the gate; outside it +0.9667 x tintype logit +2.1239 from 1646
- Tintype default + GPTTrace, fitted for |tintype logit| <= 2, fold 4: +0.5698 x tintype logit +0.4661 x gpttrace logit +3.1856 from 315 photos inside the gate; outside it +0.9724 x tintype logit +2.1495 from 1609
- Tintype default + GPTTrace, fitted for |tintype logit| <= 2, fold 5: +0.7912 x tintype logit +0.4559 x gpttrace logit +3.1856 from 302 photos inside the gate; outside it +0.9424 x tintype logit +2.0641 from 1627
- Tintype default + GPTTrace, fitted full fusion, fold 1: +0.6752 x tintype logit +0.5241 x gpttrace logit +3.3890 from 1614 photos
- Tintype default + GPTTrace, fitted full fusion, fold 2: +0.6344 x tintype logit +0.4771 x gpttrace logit +3.0261 from 1628 photos
- Tintype default + GPTTrace, fitted full fusion, fold 3: +0.6705 x tintype logit +0.5272 x gpttrace logit +3.3674 from 1646 photos
- Tintype default + GPTTrace, fitted full fusion, fold 4: +0.6420 x tintype logit +0.4880 x gpttrace logit +3.1613 from 1609 photos
- Tintype default + GPTTrace, fitted full fusion, fold 5: +0.6334 x tintype logit +0.4990 x gpttrace logit +3.1791 from 1627 photos

## arena-original: 2031 photos, 1018 generated, 1013 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 1 FP), points vs Tintype default | Recall at 1% FPR (<= 10 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/2031 (0.00%) | 750/1018 (73.67%) +0.00 | 901/1018 (88.51%) +0.00 | 898, 10 (88.21%, 0.99%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1110/2031 (54.65%) | 753/1018 (73.97%) +0.29 | 890/1018 (87.43%) -1.08 | 883, 8 (86.74%, 0.79%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 286/2031 (14.08%) | 753/1018 (73.97%) +0.29 | 891/1018 (87.52%) -0.98 | 883, 7 (86.74%, 0.69%) |
| GPTTrace alone | 0/2031 (0.00%) | 563/1018 (55.30%) -18.37 | 835/1018 (82.02%) -6.48 | 841, 13 (82.61%, 1.28%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | 264/2031 (13.00%) | 781/1018 (76.72%) +3.05 | 930/1018 (91.36%) +2.85 | 890, 5 (87.43%, 0.49%) |
| Tintype default + GPTTrace, 50:50 full fusion | 2031/2031 (100.00%) | 828/1018 (81.34%) +7.66 | 935/1018 (91.85%) +3.34 | 865, 4 (84.97%, 0.39%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 264/2031 (13.00%) | 658/1018 (64.64%) -9.04 | 931/1018 (91.45%) +2.95 | 957, 50 (94.01%, 4.94%) |
| Tintype default + GPTTrace, fitted full fusion | 2031/2031 (100.00%) | 825/1018 (81.04%) +7.37 | 946/1018 (92.93%) +4.42 | 977, 50 (95.97%, 4.94%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 120 of 1018 generated photos (Community Forensics flags 12 of them) and flags 10 of 1013 authentic photos (Community Forensics clears 8).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 1110/2031 (54.65%) | 120 | 0 | 0 | 5 | 3 |
| -3 < tintype logit < 0.5 | 286/2031 (14.08%) | 87 | 33 | 2 | 5 | 3 |
| |tintype logit| <= 1 (sweep) | 99/2031 (4.87%) | 31 | 89 | 6 | 4 | 3 |
| |tintype logit| <= 2 (sweep) | 264/2031 (13.00%) | 52 | 68 | 3 | 1 | 0 |
| |tintype logit| <= 3 (sweep) | 554/2031 (27.28%) | 87 | 33 | 2 | 0 | 0 |
| |tintype logit| <= 4 (sweep) | 794/2031 (39.09%) | 97 | 23 | 2 | 0 | 0 |
| |tintype logit| <= 6 (sweep) | 1712/2031 (84.29%) | 119 | 1 | 0 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 2030/2031 (99.95%) | 120 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 2.13, which routes 310/2031 (15.26%); 90% of the misses needs |logit| <= 4.47, which routes 1053/2031 (51.85%); 100% of the misses needs |logit| <= 6.63, which routes 1899/2031 (93.50%).

Gates on Tintype default with GPTTrace behind them, both at their own cutoffs. Tintype default misses 120 of 1018 generated photos (GPTTrace flags 62 of them) and flags 10 of 1013 authentic photos (GPTTrace clears 7).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by GPTTrace | False alarms outside | Outside, cleared by GPTTrace |
|---|---:|---:|---:|---:|---:|---:|
| |tintype logit| <= 2 | 264/2031 (13.00%) | 52 | 68 | 29 | 1 | 1 |
| |tintype logit| <= 1 (sweep) | 99/2031 (4.87%) | 31 | 89 | 41 | 4 | 3 |
| |tintype logit| <= 3 (sweep) | 554/2031 (27.28%) | 87 | 33 | 11 | 0 | 0 |
| |tintype logit| <= 4 (sweep) | 794/2031 (39.09%) | 97 | 23 | 8 | 0 | 0 |
| |tintype logit| <= 6 (sweep) | 1712/2031 (84.29%) | 119 | 1 | 0 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 2030/2031 (99.95%) | 120 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 2.13, which routes 310/2031 (15.26%); 90% of the misses needs |logit| <= 4.47, which routes 1053/2031 (51.85%); 100% of the misses needs |logit| <= 6.63, which routes 1899/2031 (93.50%).

## arena-jpeg-q4: 2031 photos, 1018 generated, 1013 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 1 FP), points vs Tintype default | Recall at 1% FPR (<= 10 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/2031 (0.00%) | 553/1018 (54.32%) +0.00 | 744/1018 (73.08%) +0.00 | 708, 7 (69.55%, 0.69%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1277/2031 (62.88%) | 395/1018 (38.80%) -15.52 | 694/1018 (68.17%) -4.91 | 690, 10 (67.78%, 0.99%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 431/2031 (21.22%) | 395/1018 (38.80%) -15.52 | 695/1018 (68.27%) -4.81 | 690, 10 (67.78%, 0.99%) |
| GPTTrace alone | 0/2031 (0.00%) | 586/1018 (57.56%) +3.24 | 781/1018 (76.72%) +3.63 | 805, 13 (79.08%, 1.28%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | 382/2031 (18.81%) | 585/1018 (57.47%) +3.14 | 825/1018 (81.04%) +7.96 | 730, 5 (71.71%, 0.49%) |
| Tintype default + GPTTrace, 50:50 full fusion | 2031/2031 (100.00%) | 563/1018 (55.30%) +0.98 | 834/1018 (81.93%) +8.84 | 722, 5 (70.92%, 0.49%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 382/2031 (18.81%) | 373/1018 (36.64%) -17.68 | 813/1018 (79.86%) +6.78 | 882, 48 (86.64%, 4.74%) |
| Tintype default + GPTTrace, fitted full fusion | 2031/2031 (100.00%) | 528/1018 (51.87%) -2.46 | 827/1018 (81.24%) +8.15 | 943, 57 (92.63%, 5.63%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 310 of 1018 generated photos (Community Forensics flags 11 of them) and flags 7 of 1013 authentic photos (Community Forensics clears 5).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 1277/2031 (62.88%) | 310 | 0 | 0 | 6 | 4 |
| -3 < tintype logit < 0.5 | 431/2031 (21.22%) | 231 | 79 | 0 | 6 | 4 |
| |tintype logit| <= 1 (sweep) | 164/2031 (8.07%) | 86 | 224 | 4 | 5 | 3 |
| |tintype logit| <= 2 (sweep) | 382/2031 (18.81%) | 170 | 140 | 0 | 1 | 0 |
| |tintype logit| <= 3 (sweep) | 704/2031 (34.66%) | 231 | 79 | 0 | 1 | 0 |
| |tintype logit| <= 4 (sweep) | 951/2031 (46.82%) | 253 | 57 | 0 | 1 | 0 |
| |tintype logit| <= 6 (sweep) | 1729/2031 (85.13%) | 307 | 3 | 0 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 2017/2031 (99.31%) | 310 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.71, which routes 336/2031 (16.54%); 90% of the misses needs |logit| <= 4.48, which routes 1161/2031 (57.16%); 100% of the misses needs |logit| <= 6.92, which routes 1944/2031 (95.72%).

Gates on Tintype default with GPTTrace behind them, both at their own cutoffs. Tintype default misses 310 of 1018 generated photos (GPTTrace flags 162 of them) and flags 7 of 1013 authentic photos (GPTTrace clears 4).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by GPTTrace | False alarms outside | Outside, cleared by GPTTrace |
|---|---:|---:|---:|---:|---:|---:|
| |tintype logit| <= 2 | 382/2031 (18.81%) | 170 | 140 | 51 | 1 | 1 |
| |tintype logit| <= 1 (sweep) | 164/2031 (8.07%) | 86 | 224 | 106 | 5 | 3 |
| |tintype logit| <= 3 (sweep) | 704/2031 (34.66%) | 231 | 79 | 21 | 1 | 1 |
| |tintype logit| <= 4 (sweep) | 951/2031 (46.82%) | 253 | 57 | 14 | 1 | 1 |
| |tintype logit| <= 6 (sweep) | 1729/2031 (85.13%) | 307 | 3 | 0 | 0 | 0 |
| |tintype logit| <= 8 (sweep) | 2017/2031 (99.31%) | 310 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.71, which routes 336/2031 (16.54%); 90% of the misses needs |logit| <= 4.48, which routes 1161/2031 (57.16%); 100% of the misses needs |logit| <= 6.92, which routes 1944/2031 (95.72%).
