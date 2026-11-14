Fit items: 3759 (2100 AI, 1659 real) from 1 file(s). Target false-positive rate 1.00%, averaged over the 2 real sources with at least 100 items.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views; below (↓) and above (↑) fitted separately.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.692 | 0.664 | 6 / 3 / 2 / 1.5 / 1 | 1.55 | 0.9953 / 0.9114 | 0.9958 / 0.9105 |
| mean | 2,0,1,3,4,5 | 0.657 | 0.686 | 4↓ 4.25↑ / 1.25↓ 2.25↑ / 0.75↓ 2.75↑ / 0.75↓ 0.5↑ / 0.5 | 1.31 | 0.9909 / 0.9290 | 0.9903 / 0.9281 |
| mean | 4,0,3,1,5,2 (greedy) | 0.657 | 0.686 | 3.75↓ 4.5↑ / 4.75↓ 2.25↑ / 1↓ 1.75↑ / 0.25↓ 2.75↑ / –↓ 0.75↑ | 1.38 | 0.9909 / 0.9290 | 0.9903 / 0.9281 |
| trimmed mean | 2,0,1,3,4,5 | 1.207 | 0.613 | – / – / 1.75↓ 1↑ / 0.75 / 0.5 | 3.08 | 0.9907 / 0.9171 | 0.9902 / 0.9162 |
| trimmed mean | 0,1,2,4,5,3 (greedy) | 1.207 | 0.613 | – / – / 1.25↓ 1.5↑ / 0.75↓ 0↑ / 0.25 | 3.06 | 0.9907 / 0.9171 | 0.9912 / 0.9162 |
| median | 2,0,1,3,4,5 | 1.129 | 0.608 | 2.75↓ 3.5↑ / 1.75↓ 1.25↑ / 2.5↓ 1.75↑ / 1↓ 2↑ / 0.25↓ 1.25↑ | 1.24 | 0.9907 / 0.9152 | 0.9902 / 0.9143 |
| median | 0,4,2,1,5,3 (greedy) | 1.129 | 0.608 | 6.5↓ 2↑ / 4↓ 2.25↑ / 2.75↓ 3.5↑ / 2.5↓ 1.75↑ / 2.25↓ 1.5↑ | 1.48 | 0.9907 / 0.9152 | 0.9912 / 0.9162 |
| top 2 | 2,0,1,3,4,5 | 4.234 | 0.533 | – / 8.25↓ 0↑ / 3.75↓ 0↑ / 3.5↓ 0↑ / 2↓ 0↑ | 2.34 | 0.9906 / 0.8995 | 0.9911 / 0.8986 |
| top 2 | 0,1,3,2,4,5 (greedy) | 4.234 | 0.533 | – / 9↓ 0↑ / 7↓ 0↑ / 3.25↓ 0↑ / 1.75↓ 0↑ | 2.55 | 0.9906 / 0.8995 | 0.9911 / 0.8986 |
| top 3 | 2,0,1,3,4,5 | 3.363 | 0.550 | – / – / 5.25↓ 0↑ / 2.25↓ 0↑ / 1.75↓ 0↑ | 3.16 | 0.9906 / 0.9052 | 0.9911 / 0.9043 |
| top 3 | 0,1,3,5,2,4 (greedy) | 3.363 | 0.550 | – / – / 5.75↓ 0↑ / 4.25↓ 0↑ / 1↓ 0↑ | 3.20 | 0.9906 / 0.9052 | 0.9911 / 0.9043 |
| max | 2,0,1,3,4,5 | 6.154 | 0.480 | –↓ 0↑ / 9.5↓ 0↑ / 8↓ 0↑ / –↓ 0↑ / 2.25↓ 0↑ | 2.22 | 0.9902 / 0.8662 | 0.9907 / 0.8652 |
| max | 1,3,4,2,5,0 (greedy) | 6.154 | 0.480 | 10.25↓ 0↑ / 8.75↓ 0↑ / –↓ 0↑ / 5.25↓ 0↑ / –↓ 0↑ | 2.06 | 0.9902 / 0.8662 | 0.9907 / 0.8662 |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 0.444 | 0.685 | 3.5↓ 6.25↑ / 1.5↓ 1.25↑ / – / 0.5↓ 0.25↑ / 0.25↓ –↑ | 1.33 | 0.9909 / 0.9314 | 0.9904 / 0.9305 |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 (greedy) | 0.444 | 0.685 | 3↓ 2.5↑ / 0.75↓ 1.5↑ / 0.5↓ 0.75↑ / 0.25↓ 0.5↑ / 0 | 1.18 | 0.9909 / 0.9314 | 0.9904 / 0.9310 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 0.6567): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":4.0,"above":4.25},{"views":2,"below":1.25,"above":2.25},{"views":3,"below":0.75,"above":2.75},{"views":4,"below":0.75,"above":0.5},{"views":5,"below":0.5,"above":0.5}]}`

`early_exit` for mean, order 4,0,3,1,5,2 (pooled-logit threshold 0.6567): `{"order":[4,0,3,1,5,2],"stages":[{"views":1,"below":3.75,"above":4.5},{"views":2,"below":4.75,"above":2.25},{"views":3,"below":1.0,"above":1.75},{"views":4,"below":0.25,"above":2.75},{"views":5,"below":1000000000.0,"above":0.75}]}`

`early_exit` for trimmed mean, order 2,0,1,3,4,5 (pooled-logit threshold 1.2073): `{"order":[2,0,1,3,4,5],"stages":[{"views":3,"below":1.75,"above":1.0},{"views":4,"below":0.75,"above":0.75},{"views":5,"below":0.5,"above":0.5}]}`

`early_exit` for trimmed mean, order 0,1,2,4,5,3 (pooled-logit threshold 1.2073): `{"order":[0,1,2,4,5,3],"stages":[{"views":3,"below":1.25,"above":1.5},{"views":4,"below":0.75,"above":0.0},{"views":5,"below":0.25,"above":0.25}]}`

`early_exit` for median, order 2,0,1,3,4,5 (pooled-logit threshold 1.1294): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":2.75,"above":3.5},{"views":2,"below":1.75,"above":1.25},{"views":3,"below":2.5,"above":1.75},{"views":4,"below":1.0,"above":2.0},{"views":5,"below":0.25,"above":1.25}]}`

`early_exit` for median, order 0,4,2,1,5,3 (pooled-logit threshold 1.1294): `{"order":[0,4,2,1,5,3],"stages":[{"views":1,"below":6.5,"above":2.0},{"views":2,"below":4.0,"above":2.25},{"views":3,"below":2.75,"above":3.5},{"views":4,"below":2.5,"above":1.75},{"views":5,"below":2.25,"above":1.5}]}`

`early_exit` for top 2, order 2,0,1,3,4,5 (pooled-logit threshold 4.2339): `{"order":[2,0,1,3,4,5],"stages":[{"views":2,"below":8.25,"above":0.0},{"views":3,"below":3.75,"above":0.0},{"views":4,"below":3.5,"above":0.0},{"views":5,"below":2.0,"above":0.0}]}`

`early_exit` for top 2, order 0,1,3,2,4,5 (pooled-logit threshold 4.2339): `{"order":[0,1,3,2,4,5],"stages":[{"views":2,"below":9.0,"above":0.0},{"views":3,"below":7.0,"above":0.0},{"views":4,"below":3.25,"above":0.0},{"views":5,"below":1.75,"above":0.0}]}`

`early_exit` for top 3, order 2,0,1,3,4,5 (pooled-logit threshold 3.3625): `{"order":[2,0,1,3,4,5],"stages":[{"views":3,"below":5.25,"above":0.0},{"views":4,"below":2.25,"above":0.0},{"views":5,"below":1.75,"above":0.0}]}`

`early_exit` for top 3, order 0,1,3,5,2,4 (pooled-logit threshold 3.3625): `{"order":[0,1,3,5,2,4],"stages":[{"views":3,"below":5.75,"above":0.0},{"views":4,"below":4.25,"above":0.0},{"views":5,"below":1.0,"above":0.0}]}`

`early_exit` for max, order 2,0,1,3,4,5 (pooled-logit threshold 6.1535): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":1000000000.0,"above":0.0},{"views":2,"below":9.5,"above":0.0},{"views":3,"below":8.0,"above":0.0},{"views":4,"below":1000000000.0,"above":0.0},{"views":5,"below":2.25,"above":0.0}]}`

`early_exit` for max, order 1,3,4,2,5,0 (pooled-logit threshold 6.1535): `{"order":[1,3,4,2,5,0],"stages":[{"views":1,"below":10.25,"above":0.0},{"views":2,"below":8.75,"above":0.0},{"views":3,"below":1000000000.0,"above":0.0},{"views":4,"below":5.25,"above":0.0},{"views":5,"below":1000000000.0,"above":0.0}]}`

`early_exit` for fitted weights 0.438/0.000/0.135/0.173/0.108/0.146, order 2,0,1,3,4,5 (pooled-logit threshold 0.4436): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":3.5,"above":6.25},{"views":2,"below":1.5,"above":1.25},{"views":4,"below":0.5,"above":0.25},{"views":5,"below":0.25,"above":1000000000.0}]}`

`early_exit` for fitted weights 0.438/0.000/0.135/0.173/0.108/0.146, order 0,3,2,4,5,1 (pooled-logit threshold 0.4436): `{"order":[0,3,2,4,5,1],"stages":[{"views":1,"below":3.0,"above":2.5},{"views":2,"below":0.75,"above":1.5},{"views":3,"below":0.5,"above":0.75},{"views":4,"below":0.25,"above":0.5},{"views":5,"below":0.0,"above":0.0}]}`

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
| mean | 2,0,1,3,4,5 | 1.30 | 94.55% | 71.33% | 24 of 3457 (0.69%) | 93.52% | +66 −5 AI, +22 −0 real |
| mean | 4,0,3,1,5,2 | 1.35 | 94.62% | 75.83% | 22 of 3457 (0.64%) | 93.52% | +69 −8 AI, +21 −1 real |
| trimmed mean | 2,0,1,3,4,5 | 3.07 | 93.74% | 85.88% | 27 of 3457 (0.78%) | 92.69% | +35 −9 AI, +26 −1 real |
| trimmed mean | 0,1,2,4,5,3 | 3.05 | 93.40% | 86.05% | 31 of 3457 (0.90%) | 92.76% | +46 −17 AI, +30 −1 real |
| median | 2,0,1,3,4,5 | 1.22 | 94.14% | 71.14% | 23 of 3457 (0.67%) | 92.40% | +32 −18 AI, +21 −0 real |
| median | 0,4,2,1,5,3 | 1.42 | 92.14% | 75.67% | 37 of 3457 (1.07%) | 92.98% | +43 −5 AI, +36 −1 real |
| top 2 | 2,0,1,3,4,5 | 2.30 | 91.90% | 82.14% | 20 of 3457 (0.58%) | 90.90% | +16 −65 AI, +19 −1 real |
| top 2 | 0,1,3,2,4,5 | 2.52 | 91.55% | 81.33% | 25 of 3457 (0.72%) | 90.95% | +18 −65 AI, +24 −1 real |
| top 3 | 2,0,1,3,4,5 | 3.13 | 92.62% | 87.52% | 16 of 3457 (0.46%) | 91.45% | +16 −42 AI, +15 −1 real |
| top 3 | 0,1,3,5,2,4 | 3.17 | 92.19% | 85.29% | 19 of 3457 (0.55%) | 91.48% | +18 −43 AI, +18 −1 real |
| max | 2,0,1,3,4,5 | 2.23 | 89.43% | 77.05% | 15 of 3457 (0.43%) | 87.83% | +16 −194 AI, +14 −1 real |
| max | 1,3,4,2,5,0 | 2.02 | 89.17% | 75.14% | 20 of 3457 (0.58%) | 87.81% | +15 −194 AI, +20 −2 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 1.32 | 94.60% | 75.55% | 21 of 3457 (0.61%) | 93.50% | +77 −17 AI, +19 −0 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 | 1.15 | 92.14% | 74.74% | 39 of 3457 (1.13%) | 93.31% | +76 −24 AI, +38 −1 real |

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
| mean | 2,0,1,3,4,5 | 1.52 | 55.04% | 40.00% | 57 of 2400 (2.38%) | 78.96% | +95 −6 AI, +36 −1 real |
| mean | 4,0,3,1,5,2 | 1.68 | 68.75% | 37.29% | 56 of 2400 (2.33%) | 78.25% | +92 −20 AI, +39 −5 real |
| trimmed mean | 2,0,1,3,4,5 | 3.13 | 70.83% | 63.12% | 52 of 2400 (2.17%) | 77.08% | +48 −4 AI, +36 −6 real |
| trimmed mean | 0,1,2,4,5,3 | 3.10 | 71.50% | 63.21% | 53 of 2400 (2.21%) | 77.17% | +61 −15 AI, +38 −7 real |
| median | 2,0,1,3,4,5 | 1.40 | 54.79% | 40.00% | 62 of 2400 (2.58%) | 76.92% | +60 −20 AI, +40 −0 real |
| median | 0,4,2,1,5,3 | 1.70 | 75.79% | 63.58% | 43 of 2400 (1.79%) | 79.88% | +116 −5 AI, +30 −9 real |
| top 2 | 2,0,1,3,4,5 | 2.57 | 64.33% | 45.67% | 62 of 2400 (2.58%) | 73.33% | +34 −80 AI, +40 −0 real |
| top 2 | 0,1,3,2,4,5 | 2.87 | 63.50% | 43.12% | 62 of 2400 (2.58%) | 73.42% | +36 −80 AI, +42 −2 real |
| top 3 | 2,0,1,3,4,5 | 3.29 | 64.92% | 53.08% | 50 of 2400 (2.08%) | 74.29% | +23 −46 AI, +30 −2 real |
| top 3 | 0,1,3,5,2,4 | 3.36 | 65.08% | 48.25% | 58 of 2400 (2.42%) | 74.29% | +24 −47 AI, +39 −3 real |
| max | 2,0,1,3,4,5 | 2.75 | 59.00% | 42.50% | 59 of 2400 (2.46%) | 69.46% | +39 −178 AI, +43 −6 real |
| max | 1,3,4,2,5,0 | 2.61 | 55.79% | 35.08% | 60 of 2400 (2.50%) | 68.54% | +21 −182 AI, +46 −8 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 1.52 | 78.29% | 45.96% | 31 of 2400 (1.29%) | 80.46% | +134 −9 AI, +19 −10 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 | 1.22 | 79.17% | 63.38% | 29 of 2400 (1.21%) | 81.67% | +162 −8 AI, +23 −16 real |

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
| mean | 2,0,1,3,4,5 | 1.38 | 83.91% | 57.48% | 81 of 5857 (1.38%) | 88.23% | +161 −11 AI, +58 −1 real |
| mean | 4,0,3,1,5,2 | 1.48 | 86.33% | 58.91% | 78 of 5857 (1.33%) | 87.97% | +161 −28 AI, +60 −6 real |
| trimmed mean | 2,0,1,3,4,5 | 3.09 | 84.73% | 77.27% | 79 of 5857 (1.35%) | 87.02% | +83 −13 AI, +62 −7 real |
| trimmed mean | 0,1,2,4,5,3 | 3.07 | 85.65% | 77.35% | 84 of 5857 (1.43%) | 87.09% | +107 −32 AI, +68 −8 real |
| median | 2,0,1,3,4,5 | 1.29 | 77.11% | 57.39% | 85 of 5857 (1.45%) | 86.77% | +92 −38 AI, +61 −0 real |
| median | 0,4,2,1,5,3 | 1.53 | 86.18% | 69.70% | 80 of 5857 (1.37%) | 88.21% | +159 −10 AI, +66 −10 real |
| top 2 | 2,0,1,3,4,5 | 2.40 | 81.92% | 65.58% | 82 of 5857 (1.40%) | 84.52% | +50 −145 AI, +59 −1 real |
| top 2 | 0,1,3,2,4,5 | 2.66 | 80.83% | 62.68% | 87 of 5857 (1.49%) | 84.58% | +54 −145 AI, +66 −3 real |
| top 3 | 2,0,1,3,4,5 | 3.19 | 84.35% | 70.55% | 66 of 5857 (1.13%) | 85.21% | +39 −88 AI, +45 −3 real |
| top 3 | 0,1,3,5,2,4 | 3.25 | 83.86% | 71.33% | 77 of 5857 (1.31%) | 85.23% | +42 −90 AI, +57 −4 real |
| max | 2,0,1,3,4,5 | 2.43 | 78.38% | 59.29% | 74 of 5857 (1.26%) | 81.15% | +55 −372 AI, +57 −7 real |
| max | 1,3,4,2,5,0 | 2.25 | 77.67% | 53.12% | 80 of 5857 (1.37%) | 80.80% | +36 −376 AI, +66 −10 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 2,0,1,3,4,5 | 1.40 | 89.30% | 61.82% | 52 of 5857 (0.89%) | 88.76% | +211 −26 AI, +38 −10 real |
| fitted weights 0.438/0.000/0.135/0.173/0.108/0.146 | 0,3,2,4,5,1 | 1.18 | 87.41% | 69.06% | 68 of 5857 (1.16%) | 89.08% | +238 −32 AI, +61 −17 real |
