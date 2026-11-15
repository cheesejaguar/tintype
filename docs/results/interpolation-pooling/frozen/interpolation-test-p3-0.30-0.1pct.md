Fit items: 12457 (6600 AI, 5857 real) from 1 file(s). Target false-positive rate 0.10%, averaged over the 2 real sources with at least 100 items.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 2.815 | 0.442 | 6 / 3 / 2 / 1.5 / 1 | 1.63 | 0.9991 / 0.8311 | 0.9996 / 0.8302 |
| mean | 2,0,1,3,4,5 | 2.815 | 0.442 | 5 / 3.5 / 1.75 / 2 / 1.75 | 1.58 | 0.9991 / 0.8311 | 0.9993 / 0.8302 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 2.8150): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":5.0,"above":5.0},{"views":2,"below":3.5,"above":3.5},{"views":3,"below":1.75,"above":1.75},{"views":4,"below":2.0,"above":2.0},{"views":5,"below":1.75,"above":1.75}]}`

### Test: test-clean (14526 items: 7200 AI, 7326 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 96.08% | 93.00% | 10 of 7326 (0.14%) | 93.46% | +15 −9 AI, +5 −0 real |
| mean | 96.08% | 93.00% | 10 of 7326 (0.14%) | 93.46% | +15 −9 AI, +5 −0 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.40 | 96.51% | 93.56% | 5 of 7326 (0.07%) | 93.38% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.34 | 96.49% | 91.08% | 9 of 7326 (0.12%) | 93.35% | +5 −7 AI, +4 −0 real |

test-clean by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 381 (95.25%) | +0 | – | – | 2.26 |
| FLUX.1-dev | 400 | 0 | 398 (99.50%) | +0 | – | – | 1.22 |
| Infinity | 1500 | 0 | 1493 (99.53%) | +0 | – | – | 1.38 |
| Janus-Pro | 600 | 0 | 600 (100.00%) | +0 | – | – | 1.10 |
| LlamaGen | 600 | 0 | 582 (97.00%) | +0 | – | – | 1.39 |
| Open-MAGVIT2 | 600 | 0 | 363 (60.50%) | +0 | – | – | 2.61 |
| RAR | 1500 | 0 | 1476 (98.40%) | +0 | – | – | 1.31 |
| SD3 | 400 | 0 | 393 (98.25%) | +0 | – | – | 1.62 |
| Switti | 600 | 0 | 595 (99.17%) | +0 | – | – | 1.25 |
| VAR | 600 | 0 | 442 (73.67%) | +0 | – | – | 2.48 |
| real:csafe | 0 | 3996 | – | – | 3 (0.08%) | +0 | 1.19 |
| real:floreview | 0 | 1874 | – | – | 2 (0.11%) | +0 | 1.28 |
| real:hdrplus | 0 | 1456 | – | – | 0 (0.00%) | +0 | 1.21 |

test-clean by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 379 (94.75%) | -2 | – | – | 2.17 |
| FLUX.1-dev | 400 | 0 | 397 (99.25%) | -1 | – | – | 1.18 |
| Infinity | 1500 | 0 | 1493 (99.53%) | +0 | – | – | 1.28 |
| Janus-Pro | 600 | 0 | 600 (100.00%) | +0 | – | – | 1.07 |
| LlamaGen | 600 | 0 | 582 (97.00%) | +0 | – | – | 1.35 |
| Open-MAGVIT2 | 600 | 0 | 363 (60.50%) | +0 | – | – | 2.57 |
| RAR | 1500 | 0 | 1476 (98.40%) | +0 | – | – | 1.25 |
| SD3 | 400 | 0 | 394 (98.50%) | +1 | – | – | 1.52 |
| Switti | 600 | 0 | 595 (99.17%) | +0 | – | – | 1.21 |
| VAR | 600 | 0 | 442 (73.67%) | +0 | – | – | 2.48 |
| real:csafe | 0 | 3996 | – | – | 7 (0.18%) | +4 | 1.14 |
| real:floreview | 0 | 1874 | – | – | 2 (0.11%) | +0 | 1.20 |
| real:hdrplus | 0 | 1456 | – | – | 0 (0.00%) | +0 | 1.15 |

### Test: test-degraded (4800 items: 2400 AI, 2400 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 76.54% | 61.04% | 20 of 2400 (0.83%) | 74.58% | +7 −12 AI, +8 −1 real |
| mean | 76.54% | 61.04% | 20 of 2400 (0.83%) | 74.58% | +7 −12 AI, +8 −1 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.86 | 79.58% | 66.58% | 13 of 2400 (0.54%) | 74.79% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.82 | 78.12% | 48.29% | 17 of 2400 (0.71%) | 74.71% | +2 −4 AI, +4 −0 real |

test-degraded by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 89 (67.42%) | +0 | – | – | 3.40 |
| FLUX.1-dev | 132 | 0 | 123 (93.18%) | +0 | – | – | 1.57 |
| Infinity | 492 | 0 | 438 (89.02%) | +0 | – | – | 1.84 |
| Janus-Pro | 204 | 0 | 181 (88.73%) | +0 | – | – | 1.82 |
| LlamaGen | 204 | 0 | 166 (81.37%) | +0 | – | – | 2.35 |
| Open-MAGVIT2 | 204 | 0 | 90 (44.12%) | +0 | – | – | 2.63 |
| RAR | 492 | 0 | 337 (68.50%) | +0 | – | – | 2.51 |
| SD3 | 132 | 0 | 116 (87.88%) | +0 | – | – | 2.06 |
| Switti | 204 | 0 | 175 (85.78%) | +0 | – | – | 1.71 |
| VAR | 204 | 0 | 80 (39.22%) | +0 | – | – | 2.98 |
| real:csafe | 0 | 1308 | – | – | 11 (0.84%) | +0 | 1.47 |
| real:floreview | 0 | 612 | – | – | 2 (0.33%) | +0 | 1.53 |
| real:hdrplus | 0 | 480 | – | – | 0 (0.00%) | +0 | 1.36 |

test-degraded by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 90 (68.18%) | +1 | – | – | 3.40 |
| FLUX.1-dev | 132 | 0 | 122 (92.42%) | -1 | – | – | 1.55 |
| Infinity | 492 | 0 | 438 (89.02%) | +0 | – | – | 1.81 |
| Janus-Pro | 204 | 0 | 181 (88.73%) | +0 | – | – | 1.79 |
| LlamaGen | 204 | 0 | 166 (81.37%) | +0 | – | – | 2.35 |
| Open-MAGVIT2 | 204 | 0 | 90 (44.12%) | +0 | – | – | 2.72 |
| RAR | 492 | 0 | 337 (68.50%) | +0 | – | – | 2.51 |
| SD3 | 132 | 0 | 114 (86.36%) | -2 | – | – | 2.10 |
| Switti | 204 | 0 | 175 (85.78%) | +0 | – | – | 1.64 |
| VAR | 204 | 0 | 80 (39.22%) | +0 | – | – | 3.10 |
| real:csafe | 0 | 1308 | – | – | 14 (1.07%) | +3 | 1.41 |
| real:floreview | 0 | 612 | – | – | 2 (0.33%) | +0 | 1.44 |
| real:hdrplus | 0 | 480 | – | – | 1 (0.21%) | +1 | 1.27 |

### Test: test-reposts (1200 items: 600 AI, 600 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 75.00% | 60.83% | 3 of 600 (0.50%) | 66.83% | +2 −4 AI, +0 −0 real |
| mean | 75.00% | 60.83% | 3 of 600 (0.50%) | 66.83% | +2 −4 AI, +0 −0 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 2.13 | 74.17% | 58.67% | 3 of 600 (0.50%) | 67.17% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 2.12 | 73.83% | 63.00% | 3 of 600 (0.50%) | 67.00% | +0 −1 AI, +0 −0 real |

test-reposts by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 21 (63.64%) | +0 | – | – | 4.45 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 1.64 |
| Infinity | 123 | 0 | 115 (93.50%) | +0 | – | – | 2.14 |
| Janus-Pro | 51 | 0 | 46 (90.20%) | +0 | – | – | 2.57 |
| LlamaGen | 51 | 0 | 38 (74.51%) | +0 | – | – | 3.47 |
| Open-MAGVIT2 | 51 | 0 | 11 (21.57%) | +0 | – | – | 3.00 |
| RAR | 123 | 0 | 61 (49.59%) | +0 | – | – | 3.15 |
| SD3 | 33 | 0 | 30 (90.91%) | +0 | – | – | 2.39 |
| Switti | 51 | 0 | 45 (88.24%) | +0 | – | – | 1.80 |
| VAR | 51 | 0 | 3 (5.88%) | +0 | – | – | 3.04 |
| real:csafe | 0 | 327 | – | – | 2 (0.61%) | +0 | 1.57 |
| real:floreview | 0 | 153 | – | – | 1 (0.65%) | +0 | 1.51 |
| real:hdrplus | 0 | 120 | – | – | 0 (0.00%) | +0 | 1.43 |

test-reposts by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 21 (63.64%) | +0 | – | – | 4.30 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 1.58 |
| Infinity | 123 | 0 | 114 (92.68%) | -1 | – | – | 2.15 |
| Janus-Pro | 51 | 0 | 46 (90.20%) | +0 | – | – | 2.59 |
| LlamaGen | 51 | 0 | 38 (74.51%) | +0 | – | – | 3.57 |
| Open-MAGVIT2 | 51 | 0 | 11 (21.57%) | +0 | – | – | 3.22 |
| RAR | 123 | 0 | 61 (49.59%) | +0 | – | – | 3.27 |
| SD3 | 33 | 0 | 30 (90.91%) | +0 | – | – | 2.42 |
| Switti | 51 | 0 | 45 (88.24%) | +0 | – | – | 1.78 |
| VAR | 51 | 0 | 3 (5.88%) | +0 | – | – | 3.20 |
| real:csafe | 0 | 327 | – | – | 2 (0.61%) | +0 | 1.53 |
| real:floreview | 0 | 153 | – | – | 1 (0.65%) | +0 | 1.37 |
| real:hdrplus | 0 | 120 | – | – | 0 (0.00%) | +0 | 1.33 |

### Test: test-all (19326 items: 9600 AI, 9726 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 91.74% | 84.23% | 30 of 9726 (0.31%) | 88.74% | +22 −21 AI, +13 −1 real |
| mean | 91.74% | 84.23% | 30 of 9726 (0.31%) | 88.74% | +22 −21 AI, +13 −1 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.51 | 92.78% | 85.84% | 18 of 9726 (0.19%) | 88.73% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.46 | 92.56% | 82.56% | 26 of 9726 (0.27%) | 88.69% | +7 −11 AI, +8 −0 real |

test-all by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 470 (88.35%) | +0 | – | – | 2.54 |
| FLUX.1-dev | 532 | 0 | 521 (97.93%) | +0 | – | – | 1.31 |
| Infinity | 1992 | 0 | 1931 (96.94%) | +0 | – | – | 1.49 |
| Janus-Pro | 804 | 0 | 781 (97.14%) | +0 | – | – | 1.28 |
| LlamaGen | 804 | 0 | 748 (93.03%) | +0 | – | – | 1.64 |
| Open-MAGVIT2 | 804 | 0 | 453 (56.34%) | +0 | – | – | 2.62 |
| RAR | 1992 | 0 | 1813 (91.01%) | +0 | – | – | 1.61 |
| SD3 | 532 | 0 | 509 (95.68%) | +0 | – | – | 1.73 |
| Switti | 804 | 0 | 770 (95.77%) | +0 | – | – | 1.37 |
| VAR | 804 | 0 | 522 (64.93%) | +0 | – | – | 2.61 |
| real:csafe | 0 | 5304 | – | – | 14 (0.26%) | +0 | 1.26 |
| real:floreview | 0 | 2486 | – | – | 4 (0.16%) | +0 | 1.34 |
| real:hdrplus | 0 | 1936 | – | – | 0 (0.00%) | +0 | 1.25 |

test-all by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 469 (88.16%) | -1 | – | – | 2.47 |
| FLUX.1-dev | 532 | 0 | 519 (97.56%) | -2 | – | – | 1.27 |
| Infinity | 1992 | 0 | 1931 (96.94%) | +0 | – | – | 1.41 |
| Janus-Pro | 804 | 0 | 781 (97.14%) | +0 | – | – | 1.25 |
| LlamaGen | 804 | 0 | 748 (93.03%) | +0 | – | – | 1.60 |
| Open-MAGVIT2 | 804 | 0 | 453 (56.34%) | +0 | – | – | 2.61 |
| RAR | 1992 | 0 | 1813 (91.01%) | +0 | – | – | 1.56 |
| SD3 | 532 | 0 | 508 (95.49%) | -1 | – | – | 1.66 |
| Switti | 804 | 0 | 770 (95.77%) | +0 | – | – | 1.32 |
| VAR | 804 | 0 | 522 (64.93%) | +0 | – | – | 2.64 |
| real:csafe | 0 | 5304 | – | – | 21 (0.40%) | +7 | 1.21 |
| real:floreview | 0 | 2486 | – | – | 4 (0.16%) | +0 | 1.26 |
| real:hdrplus | 0 | 1936 | – | – | 1 (0.05%) | +1 | 1.18 |
