Fit items: 3759 (2100 AI, 1659 real) from 1 file(s). Target false-positive rate 1.00%, averaged over the 2 real sources with at least 100 items.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.692 | 0.664 | 6 / 3 / 2 / 1.5 / 1 | 1.55 | 0.9953 / 0.9114 | 0.9958 / 0.9105 |
| mean | 2,0,1,3,4,5 | 0.657 | 0.686 | 5.5 / 1.75 / 1.5 / 0.75 / 0.5 | 1.44 | 0.9909 / 0.9290 | 0.9904 / 0.9290 |
| mean | 4,0,3,1,5,2 (greedy) | 0.657 | 0.686 | 4.5 / 1 / 1.5 / 0.25 / 0.5 | 1.29 | 0.9909 / 0.9290 | 0.9903 / 0.9281 |
| trimmed mean | 2,0,1,3,4,5 | 1.207 | 0.613 | – / – / 3 / 1.75 / 1 | 3.19 | 0.9907 / 0.9171 | 0.9913 / 0.9176 |
| trimmed mean | 0,1,2,4,5,3 (greedy) | 1.207 | 0.613 | – / – / 3 / 2.75 / 0.5 | 3.21 | 0.9907 / 0.9171 | 0.9913 / 0.9176 |
| median | 2,0,1,3,4,5 | 1.129 | 0.608 | 5 / 3 / – / 4.25 / 1.5 | 1.59 | 0.9907 / 0.9152 | 0.9907 / 0.9162 |
| median | 0,4,2,1,5,3 (greedy) | 1.129 | 0.608 | 9.5 / 4.75 / 2.75 / 2.5 / 2.25 | 2.12 | 0.9907 / 0.9152 | 0.9912 / 0.9162 |
| top 2 | 2,0,1,3,4,5 | 4.234 | 0.533 | – / 8.25 / 3.75 / 3.5 / 2 | 2.95 | 0.9906 / 0.8995 | 0.9911 / 0.8986 |
| top 2 | 0,1,3,2,4,5 (greedy) | 4.234 | 0.533 | – / 9 / 7 / 3.25 / 1.75 | 3.38 | 0.9906 / 0.8995 | 0.9911 / 0.8986 |
| top 3 | 2,0,1,3,4,5 | 3.363 | 0.550 | – / – / 5.25 / 2.25 / 1.75 | 3.37 | 0.9906 / 0.9052 | 0.9911 / 0.9043 |
| top 3 | 0,1,3,5,2,4 (greedy) | 3.363 | 0.550 | – / – / 5.75 / 4.25 / 1 | 3.48 | 0.9906 / 0.9052 | 0.9911 / 0.9043 |
| max | 2,0,1,3,4,5 | 6.154 | 0.480 | – / 9.5 / 8 / 8 / 2.25 | 4.09 | 0.9902 / 0.8662 | 0.9907 / 0.8652 |
| max | 1,3,4,2,5,0 (greedy) | 6.154 | 0.480 | 10.25 / 8.75 / – / 5.25 / 5.25 | 3.85 | 0.9902 / 0.8662 | 0.9907 / 0.8662 |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 0.444 | 0.685 | 6.25 / 1.5 / – / 0.5 / 0.25 | 1.53 | 0.9909 / 0.9314 | 0.9899 / 0.9319 |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 (greedy) | 0.444 | 0.685 | 2.5 / 1.5 / 0.5 / 0.5 / 0 | 1.17 | 0.9909 / 0.9314 | 0.9904 / 0.9310 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 0.6567): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":5.5,"above":5.5},{"views":2,"below":1.75,"above":1.75},{"views":3,"below":1.5,"above":1.5},{"views":4,"below":0.75,"above":0.75},{"views":5,"below":0.5,"above":0.5}]}`

`early_exit` for mean, order 4,0,3,1,5,2 (pooled-logit threshold 0.6567): `{"order":[4,0,3,1,5,2],"stages":[{"views":1,"below":4.5,"above":4.5},{"views":2,"below":1.0,"above":1.0},{"views":3,"below":1.5,"above":1.5},{"views":4,"below":0.25,"above":0.25},{"views":5,"below":0.5,"above":0.5}]}`

`early_exit` for trimmed mean, order 2,0,1,3,4,5 (pooled-logit threshold 1.2073): `{"order":[2,0,1,3,4,5],"stages":[{"views":3,"below":3.0,"above":3.0},{"views":4,"below":1.75,"above":1.75},{"views":5,"below":1.0,"above":1.0}]}`

`early_exit` for trimmed mean, order 0,1,2,4,5,3 (pooled-logit threshold 1.2073): `{"order":[0,1,2,4,5,3],"stages":[{"views":3,"below":3.0,"above":3.0},{"views":4,"below":2.75,"above":2.75},{"views":5,"below":0.5,"above":0.5}]}`

`early_exit` for median, order 2,0,1,3,4,5 (pooled-logit threshold 1.1294): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":5.0,"above":5.0},{"views":2,"below":3.0,"above":3.0},{"views":4,"below":4.25,"above":4.25},{"views":5,"below":1.5,"above":1.5}]}`

`early_exit` for median, order 0,4,2,1,5,3 (pooled-logit threshold 1.1294): `{"order":[0,4,2,1,5,3],"stages":[{"views":1,"below":9.5,"above":9.5},{"views":2,"below":4.75,"above":4.75},{"views":3,"below":2.75,"above":2.75},{"views":4,"below":2.5,"above":2.5},{"views":5,"below":2.25,"above":2.25}]}`

`early_exit` for top 2, order 2,0,1,3,4,5 (pooled-logit threshold 4.2339): `{"order":[2,0,1,3,4,5],"stages":[{"views":2,"below":8.25,"above":8.25},{"views":3,"below":3.75,"above":3.75},{"views":4,"below":3.5,"above":3.5},{"views":5,"below":2.0,"above":2.0}]}`

`early_exit` for top 2, order 0,1,3,2,4,5 (pooled-logit threshold 4.2339): `{"order":[0,1,3,2,4,5],"stages":[{"views":2,"below":9.0,"above":9.0},{"views":3,"below":7.0,"above":7.0},{"views":4,"below":3.25,"above":3.25},{"views":5,"below":1.75,"above":1.75}]}`

`early_exit` for top 3, order 2,0,1,3,4,5 (pooled-logit threshold 3.3625): `{"order":[2,0,1,3,4,5],"stages":[{"views":3,"below":5.25,"above":5.25},{"views":4,"below":2.25,"above":2.25},{"views":5,"below":1.75,"above":1.75}]}`

`early_exit` for top 3, order 0,1,3,5,2,4 (pooled-logit threshold 3.3625): `{"order":[0,1,3,5,2,4],"stages":[{"views":3,"below":5.75,"above":5.75},{"views":4,"below":4.25,"above":4.25},{"views":5,"below":1.0,"above":1.0}]}`

`early_exit` for max, order 2,0,1,3,4,5 (pooled-logit threshold 6.1535): `{"order":[2,0,1,3,4,5],"stages":[{"views":2,"below":9.5,"above":9.5},{"views":3,"below":8.0,"above":8.0},{"views":4,"below":8.0,"above":8.0},{"views":5,"below":2.25,"above":2.25}]}`

`early_exit` for max, order 1,3,4,2,5,0 (pooled-logit threshold 6.1535): `{"order":[1,3,4,2,5,0],"stages":[{"views":1,"below":10.25,"above":10.25},{"views":2,"below":8.75,"above":8.75},{"views":4,"below":5.25,"above":5.25},{"views":5,"below":5.25,"above":5.25}]}`

`early_exit` for fitted weights 0.438/0.000/0.135/0.173/0.108/0.146, order 2,0,1,3,4,5 (pooled-logit threshold 0.4436): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":6.25,"above":6.25},{"views":2,"below":1.5,"above":1.5},{"views":4,"below":0.5,"above":0.5},{"views":5,"below":0.25,"above":0.25}]}`

`early_exit` for fitted weights 0.438/0.000/0.135/0.173/0.108/0.146, order 0,3,2,4,5,1 (pooled-logit threshold 0.4436): `{"order":[0,3,2,4,5,1],"stages":[{"views":1,"below":2.5,"above":2.5},{"views":2,"below":1.5,"above":1.5},{"views":3,"below":0.5,"above":0.5},{"views":4,"below":0.5,"above":0.5},{"views":5,"below":0.0,"above":0.0}]}`

### Test: calibrate-clean (7657 items: 4200 AI, 3457 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 94.76% | 92.60% | 1 of 3457 (0.03%) | 92.29% | +11 −2 AI, +0 −1 real |
| mean | 94.76% | 92.60% | 17 of 3457 (0.49%) | 94.00% | +81 −0 AI, +16 −1 real |
| trimmed mean | 93.95% | 91.43% | 16 of 3457 (0.46%) | 92.83% | +34 −2 AI, +15 −1 real |
| median | 93.40% | 90.74% | 29 of 3457 (0.84%) | 92.90% | +40 −5 AI, +28 −1 real |
| top 2 | 91.10% | 86.55% | 34 of 3457 (0.98%) | 91.00% | +19 −64 AI, +33 −1 real |
| top 3 | 92.10% | 88.26% | 27 of 3457 (0.78%) | 91.60% | +21 −41 AI, +26 −1 real |
| max | 88.19% | 83.26% | 28 of 3457 (0.81%) | 87.83% | +16 −194 AI, +27 −1 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 95.26% | 92.52% | 18 of 3457 (0.52%) | 94.05% | +84 −1 AI, +17 −1 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.53 | 95.07% | 92.31% | 2 of 3457 (0.06%) | 92.07% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.43 | 95.02% | 72.48% | 25 of 3457 (0.72%) | 93.74% | +71 −1 AI, +23 −0 real |
| mean | 4,0,3,1,5,2 | 1.27 | 94.57% | 75.83% | 27 of 3457 (0.78%) | 93.45% | +70 −12 AI, +26 −1 real |
| trimmed mean | 2,0,1,3,4,5 | 3.17 | 94.14% | 90.26% | 16 of 3457 (0.46%) | 92.79% | +33 −3 AI, +15 −1 real |
| trimmed mean | 0,1,2,4,5,3 | 3.19 | 93.95% | 90.36% | 15 of 3457 (0.43%) | 92.79% | +34 −4 AI, +14 −1 real |
| median | 2,0,1,3,4,5 | 1.54 | 95.00% | 72.71% | 17 of 3457 (0.49%) | 92.74% | +32 −4 AI, +15 −0 real |
| median | 0,4,2,1,5,3 | 2.08 | 94.26% | 88.60% | 22 of 3457 (0.64%) | 92.90% | +40 −5 AI, +21 −1 real |
| top 2 | 2,0,1,3,4,5 | 2.91 | 91.90% | 88.21% | 20 of 3457 (0.58%) | 90.90% | +16 −65 AI, +19 −1 real |
| top 2 | 0,1,3,2,4,5 | 3.34 | 91.55% | 86.55% | 25 of 3457 (0.72%) | 90.95% | +18 −65 AI, +24 −1 real |
| top 3 | 2,0,1,3,4,5 | 3.34 | 92.62% | 89.29% | 16 of 3457 (0.46%) | 91.45% | +16 −42 AI, +15 −1 real |
| top 3 | 0,1,3,5,2,4 | 3.45 | 92.19% | 88.07% | 19 of 3457 (0.55%) | 91.48% | +18 −43 AI, +18 −1 real |
| max | 2,0,1,3,4,5 | 4.07 | 89.43% | 84.60% | 15 of 3457 (0.43%) | 87.83% | +16 −194 AI, +14 −1 real |
| max | 1,3,4,2,5,0 | 3.79 | 89.17% | 83.81% | 20 of 3457 (0.58%) | 87.81% | +15 −194 AI, +20 −2 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 1.51 | 94.29% | 75.57% | 26 of 3457 (0.75%) | 93.67% | +79 −12 AI, +24 −0 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 | 1.14 | 91.93% | 74.74% | 40 of 3457 (1.16%) | 93.31% | +79 −27 AI, +39 −1 real |

### Test: calibrate-degraded (4800 items: 2400 AI, 2400 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 74.67% | 65.79% | 25 of 2400 (1.04%) | 74.88% | +6 −15 AI, +8 −5 real |
| mean | 74.67% | 65.79% | 71 of 2400 (2.96%) | 78.75% | +89 −5 AI, +50 −1 real |
| trimmed mean | 71.71% | 60.29% | 46 of 2400 (1.92%) | 76.25% | +42 −18 AI, +29 −5 real |
| median | 69.58% | 58.00% | 63 of 2400 (2.62%) | 76.46% | +51 −22 AI, +45 −4 real |
| top 2 | 63.08% | 53.67% | 89 of 2400 (3.71%) | 73.58% | +40 −80 AI, +67 −0 real |
| top 3 | 65.96% | 55.67% | 69 of 2400 (2.88%) | 74.62% | +30 −45 AI, +48 −1 real |
| max | 60.42% | 43.21% | 78 of 2400 (3.25%) | 69.50% | +39 −177 AI, +62 −6 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 80.79% | 73.17% | 29 of 2400 (1.21%) | 81.17% | +145 −3 AI, +20 −13 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.86 | 75.71% | 44.67% | 22 of 2400 (0.92%) | 75.25% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.71 | 72.25% | 41.00% | 48 of 2400 (2.00%) | 79.71% | +107 −0 AI, +26 −0 real |
| mean | 4,0,3,1,5,2 | 1.49 | 69.33% | 37.29% | 44 of 2400 (1.83%) | 78.71% | +105 −22 AI, +33 −11 real |
| trimmed mean | 2,0,1,3,4,5 | 3.35 | 71.54% | 61.25% | 41 of 2400 (1.71%) | 76.46% | +42 −13 AI, +23 −4 real |
| trimmed mean | 0,1,2,4,5,3 | 3.38 | 72.54% | 63.71% | 35 of 2400 (1.46%) | 76.58% | +45 −13 AI, +18 −5 real |
| median | 2,0,1,3,4,5 | 1.96 | 63.29% | 41.12% | 52 of 2400 (2.17%) | 77.12% | +52 −7 AI, +30 −0 real |
| median | 0,4,2,1,5,3 | 2.56 | 69.88% | 58.42% | 53 of 2400 (2.21%) | 76.54% | +51 −20 AI, +35 −4 real |
| top 2 | 2,0,1,3,4,5 | 3.15 | 65.83% | 53.42% | 62 of 2400 (2.58%) | 73.33% | +34 −80 AI, +40 −0 real |
| top 2 | 0,1,3,2,4,5 | 3.60 | 68.08% | 52.88% | 62 of 2400 (2.58%) | 73.42% | +36 −80 AI, +42 −2 real |
| top 3 | 2,0,1,3,4,5 | 3.56 | 67.33% | 53.21% | 50 of 2400 (2.08%) | 74.29% | +23 −46 AI, +30 −2 real |
| top 3 | 0,1,3,5,2,4 | 3.71 | 66.17% | 57.17% | 58 of 2400 (2.42%) | 74.29% | +24 −47 AI, +39 −3 real |
| max | 2,0,1,3,4,5 | 4.10 | 61.46% | 50.08% | 59 of 2400 (2.46%) | 69.46% | +39 −178 AI, +43 −6 real |
| max | 1,3,4,2,5,0 | 3.94 | 60.96% | 45.38% | 60 of 2400 (2.50%) | 68.54% | +21 −182 AI, +46 −8 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 1.75 | 78.25% | 45.96% | 35 of 2400 (1.46%) | 81.50% | +155 −5 AI, +23 −10 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 | 1.22 | 80.42% | 63.38% | 31 of 2400 (1.29%) | 81.83% | +165 −7 AI, +25 −16 real |

### Test: calibrate-all (12457 items: 6600 AI, 5857 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 87.56% | 81.88% | 26 of 5857 (0.44%) | 85.95% | +17 −17 AI, +8 −6 real |
| mean | 87.56% | 81.88% | 88 of 5857 (1.50%) | 88.45% | +170 −5 AI, +66 −2 real |
| trimmed mean | 86.48% | 80.00% | 62 of 5857 (1.06%) | 86.80% | +76 −20 AI, +44 −6 real |
| median | 85.36% | 77.44% | 92 of 5857 (1.57%) | 86.92% | +91 −27 AI, +73 −5 real |
| top 2 | 81.73% | 72.85% | 123 of 5857 (2.10%) | 84.67% | +59 −144 AI, +100 −1 real |
| top 3 | 83.42% | 74.24% | 96 of 5857 (1.64%) | 85.42% | +51 −86 AI, +74 −2 real |
| max | 77.86% | 67.27% | 106 of 5857 (1.81%) | 81.17% | +55 −371 AI, +89 −7 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 89.92% | 84.44% | 47 of 5857 (0.80%) | 89.36% | +229 −4 AI, +37 −14 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.66 | 88.65% | 61.76% | 24 of 5857 (0.41%) | 85.95% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.54 | 87.42% | 58.32% | 73 of 5857 (1.25%) | 88.64% | +178 −1 AI, +49 −0 real |
| mean | 4,0,3,1,5,2 | 1.36 | 86.70% | 58.91% | 71 of 5857 (1.21%) | 88.09% | +175 −34 AI, +59 −12 real |
| trimmed mean | 2,0,1,3,4,5 | 3.24 | 86.88% | 76.97% | 57 of 5857 (0.97%) | 86.85% | +75 −16 AI, +38 −5 real |
| trimmed mean | 0,1,2,4,5,3 | 3.26 | 87.33% | 79.94% | 50 of 5857 (0.85%) | 86.89% | +79 −17 AI, +32 −6 real |
| median | 2,0,1,3,4,5 | 1.70 | 85.44% | 58.42% | 69 of 5857 (1.18%) | 87.06% | +84 −11 AI, +45 −0 real |
| median | 0,4,2,1,5,3 | 2.26 | 86.17% | 75.53% | 75 of 5857 (1.28%) | 86.95% | +91 −25 AI, +56 −5 real |
| top 2 | 2,0,1,3,4,5 | 3.00 | 83.38% | 72.73% | 82 of 5857 (1.40%) | 84.52% | +50 −145 AI, +59 −1 real |
| top 2 | 0,1,3,2,4,5 | 3.44 | 83.44% | 72.73% | 87 of 5857 (1.49%) | 84.58% | +54 −145 AI, +66 −3 real |
| top 3 | 2,0,1,3,4,5 | 3.43 | 84.76% | 75.12% | 66 of 5857 (1.13%) | 85.21% | +39 −88 AI, +45 −3 real |
| top 3 | 0,1,3,5,2,4 | 3.55 | 84.21% | 75.55% | 77 of 5857 (1.31%) | 85.23% | +42 −90 AI, +57 −4 real |
| max | 2,0,1,3,4,5 | 4.08 | 80.11% | 70.71% | 74 of 5857 (1.26%) | 81.15% | +55 −372 AI, +57 −7 real |
| max | 1,3,4,2,5,0 | 3.85 | 79.67% | 67.61% | 80 of 5857 (1.37%) | 80.80% | +36 −376 AI, +66 −10 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 1.61 | 88.85% | 61.82% | 61 of 5857 (1.04%) | 89.24% | +234 −17 AI, +47 −10 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 | 1.18 | 87.94% | 69.06% | 71 of 5857 (1.21%) | 89.14% | +244 −34 AI, +64 −17 real |
