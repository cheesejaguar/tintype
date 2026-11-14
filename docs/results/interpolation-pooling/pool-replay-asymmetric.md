Fit items: 82203 (46577 AI, 35626 real) from 1 file(s). Target false-positive rate 1.50%, averaged over the 16 real sources with at least 100 items.
745 items of the fit files are also test items (same path) and were left out of the fit. Paths are all this checks: copies and derivatives of a fit item under another path are not noticed.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views; below (↓) and above (↑) fitted separately.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.692 | 0.664 | 6 / 3 / 2 / 1.5 / 1 | 1.79 | 0.9889 / 0.8783 | 0.9887 / 0.8767 |
| mean | 2,0,1,3,4,5 | 1.631 | 0.671 | 4.75↓ 4.25↑ / 4↓ 2.5↑ / 2↓ 2.5↑ / 2↓ 1.75↑ / 1.25↓ 1↑ | 1.64 | 0.9885 / 0.8809 | 0.9875 / 0.8799 |
| top 2 | 2,0,1,3,4,5 | 4.910 | 0.634 | – / 9.25↓ 0↑ / 7.5↓ 0↑ / –↓ 0↑ / 5.5↓ 0↑ | 2.76 | 0.9884 / 0.8818 | 0.9887 / 0.8808 |
| top 3 | 2,0,1,3,4,5 | 3.925 | 0.644 | – / – / 6↓ 0↑ / 5.25↓ 0↑ / 4↓ 0↑ | 3.37 | 0.9882 / 0.8865 | 0.9885 / 0.8855 |
| max | 2,0,1,3,4,5 | 6.436 | 0.592 | –↓ 0↑ / 9.5↓ 0↑ / 8.75↓ 0.25↑ / –↓ 0↑ / 7.75↓ 0↑ | 2.53 | 0.9878 / 0.8580 | 0.9886 / 0.8570 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 1.6312): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":4.75,"above":4.25},{"views":2,"below":4.0,"above":2.5},{"views":3,"below":2.0,"above":2.5},{"views":4,"below":2.0,"above":1.75},{"views":5,"below":1.25,"above":1.0}]}`

`early_exit` for top 2, order 2,0,1,3,4,5 (pooled-logit threshold 4.9097): `{"order":[2,0,1,3,4,5],"stages":[{"views":2,"below":9.25,"above":0.0},{"views":3,"below":7.5,"above":0.0},{"views":4,"below":1000000000.0,"above":0.0},{"views":5,"below":5.5,"above":0.0}]}`

`early_exit` for top 3, order 2,0,1,3,4,5 (pooled-logit threshold 3.9254): `{"order":[2,0,1,3,4,5],"stages":[{"views":3,"below":6.0,"above":0.0},{"views":4,"below":5.25,"above":0.0},{"views":5,"below":4.0,"above":0.0}]}`

`early_exit` for max, order 2,0,1,3,4,5 (pooled-logit threshold 6.4357): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":1000000000.0,"above":0.0},{"views":2,"below":9.5,"above":0.0},{"views":3,"below":8.75,"above":0.25},{"views":4,"below":1000000000.0,"above":0.0},{"views":5,"below":7.75,"above":0.0}]}`

### Test: confirm7 (23052 items: 12305 AI, 10747 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 89.00% | 63.58% | 127 of 10747 (1.18%) | 90.18% | +112 −53 AI, +1 −4 real |
| mean | 89.00% | 63.58% | 128 of 10747 (1.19%) | 90.33% | +123 −46 AI, +2 −4 real |
| top 2 | 90.28% | 71.06% | 96 of 10747 (0.89%) | 89.78% | +271 −262 AI, +23 −57 real |
| top 3 | 90.17% | 69.27% | 109 of 10747 (1.01%) | 90.19% | +237 −177 AI, +19 −40 real |
| max | 89.28% | 64.89% | 79 of 10747 (0.74%) | 87.22% | +282 −588 AI, +34 −85 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.69 | 88.48% | 58.30% | 130 of 10747 (1.21%) | 89.70% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.56 | 87.85% | 54.43% | 139 of 10747 (1.29%) | 89.81% | +63 −50 AI, +10 −1 real |
| top 2 | 2,0,1,3,4,5 | 2.68 | 90.36% | 58.14% | 93 of 10747 (0.87%) | 89.70% | +262 −262 AI, +20 −57 real |
| top 3 | 2,0,1,3,4,5 | 3.31 | 90.28% | 63.51% | 106 of 10747 (0.99%) | 90.14% | +231 −177 AI, +16 −40 real |
| max | 2,0,1,3,4,5 | 2.42 | 89.33% | 50.36% | 76 of 10747 (0.71%) | 87.15% | +274 −588 AI, +31 −85 real |

### Test: confirm6 (16324 items: 8628 AI, 7696 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 87.67% | 61.37% | 112 of 7696 (1.46%) | 90.51% | +74 −61 AI, +2 −1 real |
| mean | 87.67% | 61.37% | 115 of 7696 (1.49%) | 90.81% | +96 −57 AI, +5 −1 real |
| top 2 | 90.58% | 62.63% | 75 of 7696 (0.97%) | 90.48% | +213 −202 AI, +13 −49 real |
| top 3 | 90.53% | 62.00% | 81 of 7696 (1.05%) | 90.98% | +181 −127 AI, +6 −36 real |
| max | 88.73% | 64.58% | 60 of 7696 (0.78%) | 87.61% | +229 −466 AI, +20 −71 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.71 | 88.27% | 57.59% | 111 of 7696 (1.44%) | 90.36% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.58 | 87.53% | 52.20% | 119 of 7696 (1.55%) | 90.13% | +45 −65 AI, +8 −0 real |
| top 2 | 2,0,1,3,4,5 | 2.64 | 90.51% | 60.08% | 75 of 7696 (0.97%) | 90.41% | +207 −202 AI, +13 −49 real |
| top 3 | 2,0,1,3,4,5 | 3.31 | 89.60% | 61.89% | 80 of 7696 (1.04%) | 90.91% | +175 −127 AI, +5 −36 real |
| max | 2,0,1,3,4,5 | 2.40 | 88.78% | 50.83% | 58 of 7696 (0.75%) | 87.53% | +222 −466 AI, +18 −71 real |

### Test: wasitai-original (745 items: 261 AI, 484 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 72.41% | 45.21% | 12 of 484 (2.48%) | 78.54% | +3 −2 AI, +4 −1 real |
| mean | 72.41% | 45.21% | 12 of 484 (2.48%) | 78.93% | +4 −2 AI, +4 −1 real |
| top 2 | 60.15% | 39.08% | 13 of 484 (2.69%) | 77.39% | +8 −10 AI, +7 −3 real |
| top 3 | 60.54% | 43.68% | 19 of 484 (3.93%) | 77.01% | +7 −10 AI, +12 −2 real |
| max | 44.83% | 21.84% | 16 of 484 (3.31%) | 69.73% | +7 −29 AI, +13 −6 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 2.12 | 69.35% | 50.57% | 9 of 484 (1.86%) | 78.16% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.98 | 45.21% | 37.93% | 13 of 484 (2.69%) | 79.69% | +4 −0 AI, +4 −0 real |
| top 2 | 2,0,1,3,4,5 | 3.51 | 58.62% | 32.95% | 12 of 484 (2.48%) | 77.39% | +8 −10 AI, +6 −3 real |
| top 3 | 2,0,1,3,4,5 | 3.68 | 55.94% | 32.95% | 17 of 484 (3.51%) | 77.01% | +7 −10 AI, +10 −2 real |
| max | 2,0,1,3,4,5 | 3.68 | 39.46% | 33.33% | 14 of 484 (2.89%) | 69.73% | +7 −29 AI, +11 −6 real |

### Test: wasitai-reposted (2229 items: 783 AI, 1446 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 41.38% | 20.95% | 4 of 1446 (0.28%) | 33.33% | +3 −14 AI, +0 −2 real |
| mean | 41.38% | 20.95% | 4 of 1446 (0.28%) | 34.10% | +8 −13 AI, +0 −2 real |
| top 2 | 42.66% | 25.67% | 3 of 1446 (0.21%) | 32.57% | +35 −52 AI, +0 −3 real |
| top 3 | 44.06% | 21.07% | 5 of 1446 (0.35%) | 33.59% | +26 −35 AI, +1 −2 real |
| max | 31.55% | 11.62% | 10 of 1446 (0.69%) | 30.01% | +43 −80 AI, +8 −4 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 2.02 | 39.85% | 24.27% | 6 of 1446 (0.41%) | 34.74% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.90 | 40.23% | 10.86% | 8 of 1446 (0.55%) | 36.27% | +13 −1 AI, +3 −1 real |
| top 2 | 2,0,1,3,4,5 | 3.43 | 43.17% | 22.61% | 3 of 1446 (0.21%) | 32.57% | +35 −52 AI, +0 −3 real |
| top 3 | 2,0,1,3,4,5 | 3.67 | 43.81% | 20.95% | 5 of 1446 (0.35%) | 33.59% | +26 −35 AI, +1 −2 real |
| max | 2,0,1,3,4,5 | 3.61 | 32.31% | 8.68% | 9 of 1446 (0.62%) | 30.01% | +43 −80 AI, +7 −4 real |
