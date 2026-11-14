Fit items: 12457 (6600 AI, 5857 real) from 1 file(s). Target false-positive rate 0.10%, averaged over the 2 real sources with at least 100 items.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 2.965 | 0.425 | 6 / 3 / 2 / 1.5 / 1 | 1.64 | 0.9991 / 0.8188 | 0.9994 / 0.8179 |
| mean | 2,0,1,3,4,5 | 2.965 | 0.425 | 6.75 / 4.25 / 3.25 / 2.5 / 1.25 | 1.90 | 0.9991 / 0.8188 | 0.9993 / 0.8179 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 2.9650): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":6.75,"above":6.75},{"views":2,"below":4.25,"above":4.25},{"views":3,"below":3.25,"above":3.25},{"views":4,"below":2.5,"above":2.5},{"views":5,"below":1.25,"above":1.25}]}`

### Test: test-clean (14526 items: 7200 AI, 7326 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 95.89% | 92.44% | 11 of 7326 (0.15%) | 92.92% | +22 −8 AI, +6 −0 real |
| mean | 95.89% | 92.44% | 11 of 7326 (0.15%) | 92.92% | +22 −8 AI, +6 −0 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.41 | 96.21% | 93.24% | 5 of 7326 (0.07%) | 92.72% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.59 | 96.53% | 92.38% | 10 of 7326 (0.14%) | 92.82% | +13 −6 AI, +5 −0 real |

test-clean by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 366 (91.50%) | +0 | – | – | 2.43 |
| FLUX.1-dev | 400 | 0 | 397 (99.25%) | +0 | – | – | 1.26 |
| Infinity | 1500 | 0 | 1490 (99.33%) | +0 | – | – | 1.41 |
| Janus-Pro | 600 | 0 | 599 (99.83%) | +0 | – | – | 1.11 |
| LlamaGen | 600 | 0 | 581 (96.83%) | +0 | – | – | 1.44 |
| Open-MAGVIT2 | 600 | 0 | 355 (59.17%) | +0 | – | – | 2.66 |
| RAR | 1500 | 0 | 1479 (98.60%) | +0 | – | – | 1.32 |
| SD3 | 400 | 0 | 393 (98.25%) | +0 | – | – | 1.67 |
| Switti | 600 | 0 | 594 (99.00%) | +0 | – | – | 1.29 |
| VAR | 600 | 0 | 422 (70.33%) | +0 | – | – | 2.52 |
| real:csafe | 0 | 3996 | – | – | 3 (0.08%) | +0 | 1.18 |
| real:floreview | 0 | 1874 | – | – | 1 (0.05%) | +0 | 1.27 |
| real:hdrplus | 0 | 1456 | – | – | 1 (0.07%) | +0 | 1.20 |

test-clean by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 371 (92.75%) | +5 | – | – | 3.02 |
| FLUX.1-dev | 400 | 0 | 397 (99.25%) | +0 | – | – | 1.38 |
| Infinity | 1500 | 0 | 1489 (99.27%) | -1 | – | – | 1.63 |
| Janus-Pro | 600 | 0 | 599 (99.83%) | +0 | – | – | 1.17 |
| LlamaGen | 600 | 0 | 582 (97.00%) | +1 | – | – | 1.67 |
| Open-MAGVIT2 | 600 | 0 | 354 (59.00%) | -1 | – | – | 3.17 |
| RAR | 1500 | 0 | 1479 (98.60%) | +0 | – | – | 1.50 |
| SD3 | 400 | 0 | 395 (98.75%) | +2 | – | – | 1.95 |
| Switti | 600 | 0 | 594 (99.00%) | +0 | – | – | 1.44 |
| VAR | 600 | 0 | 423 (70.50%) | +1 | – | – | 3.02 |
| real:csafe | 0 | 3996 | – | – | 6 (0.15%) | +3 | 1.27 |
| real:floreview | 0 | 1874 | – | – | 3 (0.16%) | +2 | 1.37 |
| real:hdrplus | 0 | 1456 | – | – | 1 (0.07%) | +0 | 1.29 |

### Test: test-degraded (4800 items: 2400 AI, 2400 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 74.08% | 59.00% | 20 of 2400 (0.83%) | 72.08% | +5 −18 AI, +8 −1 real |
| mean | 74.08% | 59.00% | 20 of 2400 (0.83%) | 72.08% | +5 −18 AI, +8 −1 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.86 | 78.67% | 63.62% | 13 of 2400 (0.54%) | 72.62% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 2.19 | 75.71% | 63.12% | 19 of 2400 (0.79%) | 72.00% | +2 −17 AI, +7 −1 real |

test-degraded by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 80 (60.61%) | +0 | – | – | 3.41 |
| FLUX.1-dev | 132 | 0 | 122 (92.42%) | +0 | – | – | 1.62 |
| Infinity | 492 | 0 | 435 (88.41%) | +0 | – | – | 1.92 |
| Janus-Pro | 204 | 0 | 171 (83.82%) | +0 | – | – | 1.86 |
| LlamaGen | 204 | 0 | 159 (77.94%) | +0 | – | – | 2.44 |
| Open-MAGVIT2 | 204 | 0 | 85 (41.67%) | +0 | – | – | 2.63 |
| RAR | 492 | 0 | 332 (67.48%) | +0 | – | – | 2.56 |
| SD3 | 132 | 0 | 114 (86.36%) | +0 | – | – | 2.16 |
| Switti | 204 | 0 | 172 (84.31%) | +0 | – | – | 1.70 |
| VAR | 204 | 0 | 73 (35.78%) | +0 | – | – | 2.85 |
| real:csafe | 0 | 1308 | – | – | 10 (0.76%) | +0 | 1.44 |
| real:floreview | 0 | 612 | – | – | 3 (0.49%) | +0 | 1.46 |
| real:hdrplus | 0 | 480 | – | – | 0 (0.00%) | +0 | 1.33 |

test-degraded by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 78 (59.09%) | -2 | – | – | 4.17 |
| FLUX.1-dev | 132 | 0 | 122 (92.42%) | +0 | – | – | 1.86 |
| Infinity | 492 | 0 | 432 (87.80%) | -3 | – | – | 2.34 |
| Janus-Pro | 204 | 0 | 169 (82.84%) | -2 | – | – | 2.10 |
| LlamaGen | 204 | 0 | 155 (75.98%) | -4 | – | – | 2.90 |
| Open-MAGVIT2 | 204 | 0 | 85 (41.67%) | +0 | – | – | 3.23 |
| RAR | 492 | 0 | 332 (67.48%) | +0 | – | – | 3.04 |
| SD3 | 132 | 0 | 110 (83.33%) | -4 | – | – | 2.76 |
| Switti | 204 | 0 | 172 (84.31%) | +0 | – | – | 2.07 |
| VAR | 204 | 0 | 73 (35.78%) | +0 | – | – | 3.54 |
| real:csafe | 0 | 1308 | – | – | 16 (1.22%) | +6 | 1.65 |
| real:floreview | 0 | 612 | – | – | 2 (0.33%) | -1 | 1.70 |
| real:hdrplus | 0 | 480 | – | – | 1 (0.21%) | +1 | 1.47 |

### Test: test-reposts (1200 items: 600 AI, 600 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 74.83% | 59.83% | 2 of 600 (0.33%) | 62.00% | +0 −9 AI, +0 −0 real |
| mean | 74.83% | 59.83% | 2 of 600 (0.33%) | 62.00% | +0 −9 AI, +0 −0 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 2.10 | 76.00% | 56.50% | 2 of 600 (0.33%) | 63.50% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 2.52 | 73.67% | 60.50% | 2 of 600 (0.33%) | 62.17% | +0 −8 AI, +0 −0 real |

test-reposts by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 16 (48.48%) | +0 | – | – | 4.36 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 1.76 |
| Infinity | 123 | 0 | 114 (92.68%) | +0 | – | – | 2.20 |
| Janus-Pro | 51 | 0 | 41 (80.39%) | +0 | – | – | 2.61 |
| LlamaGen | 51 | 0 | 33 (64.71%) | +0 | – | – | 3.65 |
| Open-MAGVIT2 | 51 | 0 | 9 (17.65%) | +0 | – | – | 2.94 |
| RAR | 123 | 0 | 57 (46.34%) | +0 | – | – | 3.22 |
| SD3 | 33 | 0 | 30 (90.91%) | +0 | – | – | 2.52 |
| Switti | 51 | 0 | 45 (88.24%) | +0 | – | – | 1.80 |
| VAR | 51 | 0 | 3 (5.88%) | +0 | – | – | 2.67 |
| real:csafe | 0 | 327 | – | – | 1 (0.31%) | +0 | 1.50 |
| real:floreview | 0 | 153 | – | – | 1 (0.65%) | +0 | 1.42 |
| real:hdrplus | 0 | 120 | – | – | 0 (0.00%) | +0 | 1.38 |

test-reposts by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 13 (39.39%) | -3 | – | – | 5.18 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 2.00 |
| Infinity | 123 | 0 | 113 (91.87%) | -1 | – | – | 2.68 |
| Janus-Pro | 51 | 0 | 40 (78.43%) | -1 | – | – | 2.90 |
| LlamaGen | 51 | 0 | 32 (62.75%) | -1 | – | – | 4.29 |
| Open-MAGVIT2 | 51 | 0 | 9 (17.65%) | +0 | – | – | 3.59 |
| RAR | 123 | 0 | 57 (46.34%) | +0 | – | – | 3.92 |
| SD3 | 33 | 0 | 28 (84.85%) | -2 | – | – | 3.45 |
| Switti | 51 | 0 | 45 (88.24%) | +0 | – | – | 2.31 |
| VAR | 51 | 0 | 3 (5.88%) | +0 | – | – | 3.47 |
| real:csafe | 0 | 327 | – | – | 1 (0.31%) | +0 | 1.79 |
| real:floreview | 0 | 153 | – | – | 1 (0.65%) | +0 | 1.65 |
| real:hdrplus | 0 | 120 | – | – | 0 (0.00%) | +0 | 1.53 |

### Test: test-all (19326 items: 9600 AI, 9726 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 91.20% | 83.92% | 31 of 9726 (0.32%) | 87.71% | +27 −26 AI, +14 −1 real |
| mean | 91.20% | 83.92% | 31 of 9726 (0.32%) | 87.71% | +27 −26 AI, +14 −1 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.52 | 92.50% | 85.00% | 18 of 9726 (0.19%) | 87.70% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.74 | 92.25% | 84.62% | 29 of 9726 (0.30%) | 87.61% | +15 −23 AI, +12 −1 real |

test-all by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 446 (83.83%) | +0 | – | – | 2.67 |
| FLUX.1-dev | 532 | 0 | 519 (97.56%) | +0 | – | – | 1.35 |
| Infinity | 1992 | 0 | 1925 (96.64%) | +0 | – | – | 1.53 |
| Janus-Pro | 804 | 0 | 770 (95.77%) | +0 | – | – | 1.30 |
| LlamaGen | 804 | 0 | 740 (92.04%) | +0 | – | – | 1.69 |
| Open-MAGVIT2 | 804 | 0 | 440 (54.73%) | +0 | – | – | 2.65 |
| RAR | 1992 | 0 | 1811 (90.91%) | +0 | – | – | 1.63 |
| SD3 | 532 | 0 | 507 (95.30%) | +0 | – | – | 1.79 |
| Switti | 804 | 0 | 766 (95.27%) | +0 | – | – | 1.39 |
| VAR | 804 | 0 | 495 (61.57%) | +0 | – | – | 2.61 |
| real:csafe | 0 | 5304 | – | – | 13 (0.25%) | +0 | 1.25 |
| real:floreview | 0 | 2486 | – | – | 4 (0.16%) | +0 | 1.31 |
| real:hdrplus | 0 | 1936 | – | – | 1 (0.05%) | +0 | 1.24 |

test-all by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 449 (84.40%) | +3 | – | – | 3.30 |
| FLUX.1-dev | 532 | 0 | 519 (97.56%) | +0 | – | – | 1.49 |
| Infinity | 1992 | 0 | 1921 (96.44%) | -4 | – | – | 1.80 |
| Janus-Pro | 804 | 0 | 768 (95.52%) | -2 | – | – | 1.41 |
| LlamaGen | 804 | 0 | 737 (91.67%) | -3 | – | – | 1.98 |
| Open-MAGVIT2 | 804 | 0 | 439 (54.60%) | -1 | – | – | 3.19 |
| RAR | 1992 | 0 | 1811 (90.91%) | +0 | – | – | 1.88 |
| SD3 | 532 | 0 | 505 (94.92%) | -2 | – | – | 2.15 |
| Switti | 804 | 0 | 766 (95.27%) | +0 | – | – | 1.60 |
| VAR | 804 | 0 | 496 (61.69%) | +1 | – | – | 3.15 |
| real:csafe | 0 | 5304 | – | – | 22 (0.41%) | +9 | 1.36 |
| real:floreview | 0 | 2486 | – | – | 5 (0.20%) | +1 | 1.45 |
| real:hdrplus | 0 | 1936 | – | – | 2 (0.10%) | +1 | 1.33 |
