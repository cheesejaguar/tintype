# Policy comparison

4076 photos scored by every system (commfor, gpttrace, tintype).

Split: 1624 fit, 814 calibrate, 1624 test photos; 14 scored photos are in no set and are left out.

Fusion weights fitted on the fit set (1624 photos) only:

- Tintype default + GPTTrace, fitted for |tintype logit| <= 2, fit set: +0.9804 x tintype logit +0.5147 x gpttrace logit +3.2014 from 238 photos inside the gate; outside it +0.9453 x tintype logit +1.8108 from 1624
- Tintype default + GPTTrace, fitted full fusion, fit set: +0.6538 x tintype logit +0.5120 x gpttrace logit +3.1560 from 1624 photos

Cutoffs are frozen on the `calibrate` set (814 photos, 406 authentic).

## Test set: 1624 photos, 814 generated, 810 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 8 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/1624 (0.00%) | 514/814 (63.14%) +0.00 | 633/814 (77.76%) +0.00 | 636, 9 (78.13%, 1.11%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 964/1624 (59.36%) | 397/814 (48.77%) -14.37 | 610/814 (74.94%) -2.83 | 621, 10 (76.29%, 1.23%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 311/1624 (19.15%) | 397/814 (48.77%) -14.37 | 618/814 (75.92%) -1.84 | 621, 9 (76.29%, 1.11%) |
| GPTTrace alone | 0/1624 (0.00%) | 418/814 (51.35%) -11.79 | 630/814 (77.40%) -0.37 | 663, 15 (81.45%, 1.85%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | 281/1624 (17.30%) | 539/814 (66.22%) +3.07 | 693/814 (85.14%) +7.37 | 649, 6 (79.73%, 0.74%) |
| Tintype default + GPTTrace, 50:50 full fusion | 1624/1624 (100.00%) | 512/814 (62.90%) -0.25 | 689/814 (84.64%) +6.88 | 638, 6 (78.38%, 0.74%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 281/1624 (17.30%) | 338/814 (41.52%) -21.62 | 676/814 (83.05%) +5.28 | 730, 19 (89.68%, 2.35%) |
| Tintype default + GPTTrace, fitted full fusion | 1624/1624 (100.00%) | 510/814 (62.65%) -0.49 | 678/814 (83.29%) +5.53 | 768, 44 (94.35%, 5.43%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Sets (of 2) where precision and recall both reach Community Forensics at its own cutoff:

| Policy | at 0.1% FPR | at 1% FPR | at own cutoff |
|---|---:|---:|---:|
| Tintype default | 2/2 | 2/2 | 2/2 |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 2/2 | 2/2 | 2/2 |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 2/2 | 2/2 | 2/2 |
| GPTTrace alone | 2/2 | 2/2 | 2/2 |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | 2/2 | 2/2 | 2/2 |
| Tintype default + GPTTrace, 50:50 full fusion | 2/2 | 2/2 | 2/2 |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 2/2 | 2/2 | 2/2 |
| Tintype default + GPTTrace, fitted full fusion | 2/2 | 2/2 | 1/2 |

Cutoffs frozen on the calibration set (406 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | -0.04913 | 9/810 (1.11%) | 641/814 (78.75%) | -0.83647 | 19/810 (2.35%) | 682/814 (83.78%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1.00331 | 7/810 (0.86%) | 595/814 (73.10%) | -0.93567 | 15/810 (1.85%) | 630/814 (77.40%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1.00331 | 7/810 (0.86%) | 595/814 (73.10%) | -1.57579 | 15/810 (1.85%) | 637/814 (78.26%) |
| GPTTrace alone | 0.66243 | 6/810 (0.74%) | 605/814 (74.32%) | -0.74512 | 23/810 (2.84%) | 693/814 (85.14%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | -1.22219 | 10/810 (1.23%) | 706/814 (86.73%) | -1.92295 | 14/810 (1.73%) | 723/814 (88.82%) |
| Tintype default + GPTTrace, 50:50 full fusion | -1.22219 | 12/810 (1.48%) | 716/814 (87.96%) | -1.50266 | 18/810 (2.22%) | 734/814 (90.17%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 1.92039 | 10/810 (1.23%) | 702/814 (86.24%) | 0.57137 | 14/810 (1.73%) | 726/814 (89.19%) |
| Tintype default + GPTTrace, fitted full fusion | 1.89749 | 10/810 (1.23%) | 712/814 (87.47%) | 1.12515 | 20/810 (2.47%) | 742/814 (91.15%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 178 of 814 generated photos (Community Forensics flags 11 of them) and flags 9 of 810 authentic photos (Community Forensics clears 8).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 964/1624 (59.36%) | 178 | 0 | 0 | 6 | 5 |
| -3 < tintype logit < 0.5 | 311/1624 (19.15%) | 138 | 40 | 1 | 6 | 5 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.53, which routes 196/1624 (12.07%); 90% of the misses needs |logit| <= 4.38, which routes 863/1624 (53.14%); 100% of the misses needs |logit| <= 6.92, which routes 1555/1624 (95.75%).

Gates on Tintype default with GPTTrace behind them, both at their own cutoffs. Tintype default misses 178 of 814 generated photos (GPTTrace flags 94 of them) and flags 9 of 810 authentic photos (GPTTrace clears 5).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by GPTTrace | False alarms outside | Outside, cleared by GPTTrace |
|---|---:|---:|---:|---:|---:|---:|
| |tintype logit| <= 2 | 281/1624 (17.30%) | 105 | 73 | 26 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.53, which routes 196/1624 (12.07%); 90% of the misses needs |logit| <= 4.38, which routes 863/1624 (53.14%); 100% of the misses needs |logit| <= 6.92, which routes 1555/1624 (95.75%).

## Test set, transform original: 812 photos, 407 generated, 405 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 4 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/812 (0.00%) | 293/407 (71.99%) +0.00 | 355/407 (87.22%) +0.00 | 356, 5 (87.47%, 1.23%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 449/812 (55.30%) | 293/407 (71.99%) +0.00 | 352/407 (86.49%) -0.74 | 349, 4 (85.75%, 0.99%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 120/812 (14.78%) | 293/407 (71.99%) +0.00 | 354/407 (86.98%) -0.25 | 349, 3 (85.75%, 0.74%) |
| GPTTrace alone | 0/812 (0.00%) | 219/407 (53.81%) -18.18 | 327/407 (80.34%) -6.88 | 340, 8 (83.54%, 1.98%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | 119/812 (14.66%) | 307/407 (75.43%) +3.44 | 373/407 (91.65%) +4.42 | 353, 3 (86.73%, 0.74%) |
| Tintype default + GPTTrace, 50:50 full fusion | 812/812 (100.00%) | 290/407 (71.25%) -0.74 | 371/407 (91.15%) +3.93 | 347, 3 (85.26%, 0.74%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 119/812 (14.66%) | 207/407 (50.86%) -21.13 | 374/407 (91.89%) +4.67 | 382, 9 (93.86%, 2.22%) |
| Tintype default + GPTTrace, fitted full fusion | 812/812 (100.00%) | 291/407 (71.50%) -0.49 | 378/407 (92.87%) +5.65 | 393, 20 (96.56%, 4.94%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Cutoffs frozen on the calibration set (406 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | -0.04913 | 5/405 (1.23%) | 357/407 (87.71%) | -0.83647 | 12/405 (2.96%) | 370/407 (90.91%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1.00331 | 3/405 (0.74%) | 339/407 (83.29%) | -0.93567 | 7/405 (1.73%) | 355/407 (87.22%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1.00331 | 3/405 (0.74%) | 339/407 (83.29%) | -1.57579 | 7/405 (1.73%) | 358/407 (87.96%) |
| GPTTrace alone | 0.66243 | 3/405 (0.74%) | 311/407 (76.41%) | -0.74512 | 8/405 (1.98%) | 355/407 (87.22%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | -1.22219 | 4/405 (0.99%) | 373/407 (91.65%) | -1.92295 | 7/405 (1.73%) | 378/407 (92.87%) |
| Tintype default + GPTTrace, 50:50 full fusion | -1.22219 | 5/405 (1.23%) | 373/407 (91.65%) | -1.50266 | 9/405 (2.22%) | 380/407 (93.37%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 1.92039 | 4/405 (0.99%) | 374/407 (91.89%) | 0.57137 | 7/405 (1.73%) | 380/407 (93.37%) |
| Tintype default + GPTTrace, fitted full fusion | 1.89749 | 4/405 (0.99%) | 375/407 (92.14%) | 1.12515 | 10/405 (2.47%) | 384/407 (94.35%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 51 of 407 generated photos (Community Forensics flags 5 of them) and flags 5 of 405 authentic photos (Community Forensics clears 5).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 449/812 (55.30%) | 51 | 0 | 0 | 3 | 3 |
| -3 < tintype logit < 0.5 | 120/812 (14.78%) | 37 | 14 | 1 | 3 | 3 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.47, which routes 78/812 (9.61%); 90% of the misses needs |logit| <= 4.10, which routes 357/812 (43.97%); 100% of the misses needs |logit| <= 5.35, which routes 588/812 (72.41%).

Gates on Tintype default with GPTTrace behind them, both at their own cutoffs. Tintype default misses 51 of 407 generated photos (GPTTrace flags 26 of them) and flags 5 of 405 authentic photos (GPTTrace clears 3).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by GPTTrace | False alarms outside | Outside, cleared by GPTTrace |
|---|---:|---:|---:|---:|---:|---:|
| |tintype logit| <= 2 | 119/812 (14.66%) | 28 | 23 | 8 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.47, which routes 78/812 (9.61%); 90% of the misses needs |logit| <= 4.10, which routes 357/812 (43.97%); 100% of the misses needs |logit| <= 5.35, which routes 588/812 (72.41%).

## Test set, transform jpeg-q4: 812 photos, 407 generated, 405 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 0 FP), points vs Tintype default | Recall at 1% FPR (<= 4 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/812 (0.00%) | 221/407 (54.30%) +0.00 | 295/407 (72.48%) +0.00 | 280, 4 (68.80%, 0.99%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 515/812 (63.42%) | 161/407 (39.56%) -14.74 | 265/407 (65.11%) -7.37 | 272, 6 (66.83%, 1.48%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 191/812 (23.52%) | 161/407 (39.56%) -14.74 | 265/407 (65.11%) -7.37 | 272, 6 (66.83%, 1.48%) |
| GPTTrace alone | 0/812 (0.00%) | 227/407 (55.77%) +1.47 | 307/407 (75.43%) +2.95 | 323, 7 (79.36%, 1.73%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | 162/812 (19.95%) | 234/407 (57.49%) +3.19 | 323/407 (79.36%) +6.88 | 296, 3 (72.73%, 0.74%) |
| Tintype default + GPTTrace, 50:50 full fusion | 812/812 (100.00%) | 224/407 (55.04%) +0.74 | 324/407 (79.61%) +7.13 | 291, 3 (71.50%, 0.74%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 162/812 (19.95%) | 137/407 (33.66%) -20.64 | 310/407 (76.17%) +3.69 | 348, 10 (85.50%, 2.47%) |
| Tintype default + GPTTrace, fitted full fusion | 812/812 (100.00%) | 222/407 (54.55%) +0.25 | 315/407 (77.40%) +4.91 | 375, 24 (92.14%, 5.93%) |
- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.

Cutoffs frozen on the calibration set (406 authentic photos there), applied here unchanged:

| Policy | target 0.1% FPR: cutoff (logit) | actual FP (FPR) | recall | target 1% FPR: cutoff (logit) | actual FP (FPR) | recall |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | -0.04913 | 4/405 (0.99%) | 284/407 (69.78%) | -0.83647 | 7/405 (1.73%) | 312/407 (76.66%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1.00331 | 4/405 (0.99%) | 256/407 (62.90%) | -0.93567 | 8/405 (1.98%) | 275/407 (67.57%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1.00331 | 4/405 (0.99%) | 256/407 (62.90%) | -1.57579 | 8/405 (1.98%) | 279/407 (68.55%) |
| GPTTrace alone | 0.66243 | 3/405 (0.74%) | 294/407 (72.24%) | -0.74512 | 15/405 (3.70%) | 338/407 (83.05%) |
| Tintype default + GPTTrace, 50:50 for |tintype logit| <= 2 | -1.22219 | 6/405 (1.48%) | 333/407 (81.82%) | -1.92295 | 7/405 (1.73%) | 345/407 (84.77%) |
| Tintype default + GPTTrace, 50:50 full fusion | -1.22219 | 7/405 (1.73%) | 343/407 (84.28%) | -1.50266 | 9/405 (2.22%) | 354/407 (86.98%) |
| Tintype default + GPTTrace, fitted for |tintype logit| <= 2 | 1.92039 | 6/405 (1.48%) | 328/407 (80.59%) | 0.57137 | 7/405 (1.73%) | 346/407 (85.01%) |
| Tintype default + GPTTrace, fitted full fusion | 1.89749 | 6/405 (1.48%) | 337/407 (82.80%) | 1.12515 | 10/405 (2.47%) | 358/407 (87.96%) |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 127 of 407 generated photos (Community Forensics flags 6 of them) and flags 4 of 405 authentic photos (Community Forensics clears 3).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 515/812 (63.42%) | 127 | 0 | 0 | 3 | 2 |
| -3 < tintype logit < 0.5 | 191/812 (23.52%) | 101 | 26 | 0 | 3 | 2 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.53, which routes 118/812 (14.53%); 90% of the misses needs |logit| <= 4.41, which routes 460/812 (56.65%); 100% of the misses needs |logit| <= 6.92, which routes 773/812 (95.20%).

Gates on Tintype default with GPTTrace behind them, both at their own cutoffs. Tintype default misses 127 of 407 generated photos (GPTTrace flags 68 of them) and flags 4 of 405 authentic photos (GPTTrace clears 2).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by GPTTrace | False alarms outside | Outside, cleared by GPTTrace |
|---|---:|---:|---:|---:|---:|---:|
| |tintype logit| <= 2 | 162/812 (19.95%) | 77 | 50 | 18 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.53, which routes 118/812 (14.53%); 90% of the misses needs |logit| <= 4.41, which routes 460/812 (56.65%); 100% of the misses needs |logit| <= 6.92, which routes 773/812 (95.20%).
