# Policy comparison

23052 photos scored by every system (commfor, tintype-views).
Fitted fusion is left out: it needs `--set fit=... --set test=...` or `--cross-fit K`, so that no weight is fitted on the photos it is reported on.

## All photos: 23052 photos, 12305 generated, 10747 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 10 FP), points vs tintype-views | Recall at 1% FPR (<= 107 FP), points vs tintype-views | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| tintype-views | 0/23052 (0.00%) | 7824/12305 (63.58%) +0.00 | 10951/12305 (89.00%) +0.00 | 11097, 127 (90.18%, 1.18%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 11472/23052 (49.77%) | 7480/12305 (60.79%) -2.80 | 10333/12305 (83.97%) -5.02 | 11507, 349 (93.51%, 3.25%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 2776/23052 (12.04%) | 7852/12305 (63.81%) +0.23 | 10796/12305 (87.74%) -1.26 | 11479, 220 (93.29%, 2.05%) |
| Community Forensics alone | 0/23052 (0.00%) | 5535/12305 (44.98%) -18.60 | 7766/12305 (63.11%) -25.88 | 9707, 564 (78.89%, 5.25%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 3443/23052 (14.94%) | 8290/12305 (67.37%) +3.79 | 11044/12305 (89.75%) +0.76 | 10992, 99 (89.33%, 0.92%) |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 5954/23052 (25.83%) | 8914/12305 (72.44%) +8.86 | 10724/12305 (87.15%) -1.84 | 10786, 123 (87.66%, 1.14%) |
| tintype-views + Community Forensics, 50:50 full fusion | 23052/23052 (100.00%) | 8030/12305 (65.26%) +1.67 | 10120/12305 (82.24%) -6.75 | 10565, 173 (85.86%, 1.61%) |

Sets (of 30) where precision and recall both reach Community Forensics at its own cutoff:

| Policy | at 0.1% FPR | at 1% FPR | at own cutoff |
|---|---:|---:|---:|
| tintype-views | 8/30 | 12/30 | 12/30 |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 3/30 | 12/30 | 26/30 |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 8/30 | 14/30 | 21/30 |
| Community Forensics alone | 2/30 | 6/30 | 30/30 |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 2 | 5/30 | 19/30 | 19/30 |
| tintype-views + Community Forensics, 50:50 for |tintype-views logit| <= 3 | 8/30 | 18/30 | 20/30 |
| tintype-views + Community Forensics, 50:50 full fusion | 4/30 | 22/30 | 27/30 |

Gates on tintype-views with Community Forensics behind them, both at their own cutoffs. tintype-views misses 1208 of 12305 generated photos (Community Forensics flags 579 of them) and flags 127 of 10747 authentic photos (Community Forensics clears 100).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 11472/23052 (49.77%) | 1208 | 0 | 0 | 86 | 69 |
| -3 < tintype-views logit < 0.5 | 2776/23052 (12.04%) | 1072 | 136 | 34 | 86 | 69 |
| |tintype-views logit| <= 2 | 3443/23052 (14.94%) | 851 | 357 | 122 | 20 | 13 |
| |tintype-views logit| <= 3 | 5954/23052 (25.83%) | 1072 | 136 | 34 | 6 | 3 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.26, which routes 1991/23052 (8.64%); 90% of the misses needs |logit| <= 3.08, which routes 6145/23052 (26.66%); 100% of the misses needs |logit| <= 5.80, which routes 17675/23052 (76.67%).

View disagreement (largest minus smallest view logit of tintype-views) as a routing feature, analysis only:

| Gate | Confident negatives outside: generated, authentic | AUROC of view spread for generated among them | Misses inside the margin gate | Misses among as many photos with the largest view spread | In both |
|---|---:|---:|---:|---:|---:|
| -7 < tintype-views logit < 0.5 | 0, 767 | n/a | 1208 | 886 | 886 |
| -3 < tintype-views logit < 0.5 | 136, 9327 | 0.6282 | 1072 | 385 | 370 |
| |tintype-views logit| <= 2 | 357, 10109 | 0.6354 | 851 | 450 | 374 |
| |tintype-views logit| <= 3 | 136, 9327 | 0.6282 | 1072 | 625 | 587 |

Misses of tintype-views outside the gate -3 < tintype-views logit < 0.5, by dataset (15 of 16 slices with such a miss, most first):

| dataset | Generated | Misses | Misses outside the gate | Outside, flagged by Community Forensics |
|---|---:|---:|---:|---:|
| sofake-ood-fresh6 | 2217 | 338 | 54 | 1 |
| classic-deepfake-b2 | 300 | 258 | 39 | 12 |
| classic-san-b2 | 219 | 59 | 12 | 3 |
| classic-stylegan-b2 | 300 | 22 | 5 | 3 |
| classic-stylegan2-b2 | 300 | 41 | 5 | 3 |
| aigibench-imagen3-b3 | 400 | 29 | 3 | 0 |
| aigibench-photomaker-b3 | 399 | 128 | 3 | 1 |
| aigibench-r3gan-b3 | 400 | 17 | 3 | 3 |
| classic-seeingdark-b2 | 180 | 32 | 3 | 2 |
| aigibench-flux1-dev-b3 | 400 | 3 | 2 | 0 |
| aigibench-glide-b3 | 397 | 17 | 2 | 2 |
| aigibench-ip-adapter-b3 | 400 | 51 | 1 | 0 |
| aigibench-stylegan-xl-b3 | 400 | 33 | 1 | 1 |
| aigibench-styleswim-b3 | 400 | 70 | 1 | 1 |
| classic-cyclegan-b2 | 300 | 5 | 1 | 1 |

Misses of tintype-views outside the gate |tintype-views logit| <= 2, by dataset (15 of 18 slices with such a miss, most first):

| dataset | Generated | Misses | Misses outside the gate | Outside, flagged by Community Forensics |
|---|---:|---:|---:|---:|
| classic-deepfake-b2 | 300 | 258 | 141 | 60 |
| sofake-ood-fresh6 | 2217 | 338 | 113 | 7 |
| classic-san-b2 | 219 | 59 | 20 | 3 |
| aigibench-photomaker-b3 | 399 | 128 | 17 | 7 |
| classic-seeingdark-b2 | 180 | 32 | 11 | 5 |
| classic-stylegan2-b2 | 300 | 41 | 10 | 8 |
| classic-stylegan-b2 | 300 | 22 | 9 | 7 |
| aigibench-styleswim-b3 | 400 | 70 | 7 | 7 |
| aigibench-ip-adapter-b3 | 400 | 51 | 6 | 4 |
| aigibench-imagen3-b3 | 400 | 29 | 5 | 0 |
| aigibench-r3gan-b3 | 400 | 17 | 5 | 5 |
| aigibench-glide-b3 | 397 | 17 | 4 | 3 |
| aigibench-stylegan-xl-b3 | 400 | 33 | 3 | 3 |
| aigibench-flux1-dev-b3 | 400 | 3 | 2 | 0 |
| aigibench-midjourney-b3 | 394 | 10 | 1 | 0 |

Misses of tintype-views outside the gate |tintype-views logit| <= 3, by dataset (15 of 16 slices with such a miss, most first):

| dataset | Generated | Misses | Misses outside the gate | Outside, flagged by Community Forensics |
|---|---:|---:|---:|---:|
| sofake-ood-fresh6 | 2217 | 338 | 54 | 1 |
| classic-deepfake-b2 | 300 | 258 | 39 | 12 |
| classic-san-b2 | 219 | 59 | 12 | 3 |
| classic-stylegan-b2 | 300 | 22 | 5 | 3 |
| classic-stylegan2-b2 | 300 | 41 | 5 | 3 |
| aigibench-imagen3-b3 | 400 | 29 | 3 | 0 |
| aigibench-photomaker-b3 | 399 | 128 | 3 | 1 |
| aigibench-r3gan-b3 | 400 | 17 | 3 | 3 |
| classic-seeingdark-b2 | 180 | 32 | 3 | 2 |
| aigibench-flux1-dev-b3 | 400 | 3 | 2 | 0 |
| aigibench-glide-b3 | 397 | 17 | 2 | 2 |
| aigibench-ip-adapter-b3 | 400 | 51 | 1 | 0 |
| aigibench-stylegan-xl-b3 | 400 | 33 | 1 | 1 |
| aigibench-styleswim-b3 | 400 | 70 | 1 | 1 |
| classic-cyclegan-b2 | 300 | 5 | 1 | 1 |
