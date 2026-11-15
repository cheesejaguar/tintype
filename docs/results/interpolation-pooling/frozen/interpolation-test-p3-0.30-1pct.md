Fit items: 12457 (6600 AI, 5857 real) from 1 file(s). Target false-positive rate 1.00%, averaged over the 2 real sources with at least 100 items.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.064 | 0.564 | 6 / 3 / 2 / 1.5 / 1 | 1.69 | 0.9901 / 0.8782 | 0.9909 / 0.8789 |
| mean | 2,0,1,3,4,5 | 1.064 | 0.564 | 4.5 / 0.75 / 1 / 0.75 / 1 | 1.35 | 0.9901 / 0.8782 | 0.9892 / 0.8777 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 1.0642): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":4.5,"above":4.5},{"views":2,"below":0.75,"above":0.75},{"views":3,"below":1.0,"above":1.0},{"views":4,"below":0.75,"above":0.75},{"views":5,"below":1.0,"above":1.0}]}`

### Test: test-clean (14526 items: 7200 AI, 7326 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 96.08% | 93.00% | 52 of 7326 (0.71%) | 95.75% | +1 −0 AI, +14 −6 real |
| mean | 96.08% | 93.00% | 52 of 7326 (0.71%) | 95.75% | +1 −0 AI, +14 −6 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.42 | 96.60% | 90.58% | 44 of 7326 (0.60%) | 95.74% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.20 | 95.89% | 83.35% | 57 of 7326 (0.78%) | 95.39% | +8 −33 AI, +25 −12 real |

test-clean by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 399 (99.75%) | +0 | – | – | 1.70 |
| FLUX.1-dev | 400 | 0 | 399 (99.75%) | +0 | – | – | 1.12 |
| Infinity | 1500 | 0 | 1500 (100.00%) | +0 | – | – | 1.17 |
| Janus-Pro | 600 | 0 | 600 (100.00%) | +0 | – | – | 1.04 |
| LlamaGen | 600 | 0 | 593 (98.83%) | +0 | – | – | 1.24 |
| Open-MAGVIT2 | 600 | 0 | 424 (70.67%) | +0 | – | – | 2.41 |
| RAR | 1500 | 0 | 1489 (99.27%) | +0 | – | – | 1.15 |
| SD3 | 400 | 0 | 398 (99.50%) | +0 | – | – | 1.34 |
| Switti | 600 | 0 | 599 (99.83%) | +0 | – | – | 1.12 |
| VAR | 600 | 0 | 492 (82.00%) | +0 | – | – | 2.25 |
| real:csafe | 0 | 3996 | – | – | 26 (0.65%) | +0 | 1.40 |
| real:floreview | 0 | 1874 | – | – | 12 (0.64%) | +0 | 1.59 |
| real:hdrplus | 0 | 1456 | – | – | 6 (0.41%) | +0 | 1.48 |

test-clean by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 384 (96.00%) | -15 | – | – | 1.32 |
| FLUX.1-dev | 400 | 0 | 398 (99.50%) | -1 | – | – | 1.06 |
| Infinity | 1500 | 0 | 1498 (99.87%) | -2 | – | – | 1.08 |
| Janus-Pro | 600 | 0 | 600 (100.00%) | +0 | – | – | 1.01 |
| LlamaGen | 600 | 0 | 591 (98.50%) | -2 | – | – | 1.12 |
| Open-MAGVIT2 | 600 | 0 | 425 (70.83%) | +1 | – | – | 1.68 |
| RAR | 1500 | 0 | 1488 (99.20%) | -1 | – | – | 1.06 |
| SD3 | 400 | 0 | 396 (99.00%) | -2 | – | – | 1.20 |
| Switti | 600 | 0 | 599 (99.83%) | +0 | – | – | 1.06 |
| VAR | 600 | 0 | 489 (81.50%) | -3 | – | – | 1.59 |
| real:csafe | 0 | 3996 | – | – | 33 (0.83%) | +7 | 1.19 |
| real:floreview | 0 | 1874 | – | – | 16 (0.85%) | +4 | 1.29 |
| real:hdrplus | 0 | 1456 | – | – | 8 (0.55%) | +2 | 1.22 |

### Test: test-degraded (4800 items: 2400 AI, 2400 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 76.54% | 61.04% | 73 of 2400 (3.04%) | 82.25% | +0 −18 AI, +13 −3 real |
| mean | 76.54% | 61.04% | 73 of 2400 (3.04%) | 82.25% | +0 −18 AI, +13 −3 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.97 | 77.29% | 45.08% | 63 of 2400 (2.62%) | 83.00% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.51 | 73.79% | 43.00% | 54 of 2400 (2.25%) | 83.54% | +25 −12 AI, +13 −22 real |

test-degraded by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 120 (90.91%) | +0 | – | – | 2.60 |
| FLUX.1-dev | 132 | 0 | 126 (95.45%) | +0 | – | – | 1.39 |
| Infinity | 492 | 0 | 463 (94.11%) | +0 | – | – | 1.54 |
| Janus-Pro | 204 | 0 | 188 (92.16%) | +0 | – | – | 1.47 |
| LlamaGen | 204 | 0 | 185 (90.69%) | +0 | – | – | 1.94 |
| Open-MAGVIT2 | 204 | 0 | 108 (52.94%) | +0 | – | – | 2.88 |
| RAR | 492 | 0 | 388 (78.86%) | +0 | – | – | 2.30 |
| SD3 | 132 | 0 | 120 (90.91%) | +0 | – | – | 1.85 |
| Switti | 204 | 0 | 182 (89.22%) | +0 | – | – | 1.54 |
| VAR | 204 | 0 | 112 (54.90%) | +0 | – | – | 3.31 |
| real:csafe | 0 | 1308 | – | – | 46 (3.52%) | +0 | 1.88 |
| real:floreview | 0 | 612 | – | – | 14 (2.29%) | +0 | 1.96 |
| real:hdrplus | 0 | 480 | – | – | 3 (0.62%) | +0 | 1.73 |

test-degraded by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 118 (89.39%) | -2 | – | – | 1.88 |
| FLUX.1-dev | 132 | 0 | 126 (95.45%) | +0 | – | – | 1.22 |
| Infinity | 492 | 0 | 470 (95.53%) | +7 | – | – | 1.26 |
| Janus-Pro | 204 | 0 | 189 (92.65%) | +1 | – | – | 1.23 |
| LlamaGen | 204 | 0 | 192 (94.12%) | +7 | – | – | 1.49 |
| Open-MAGVIT2 | 204 | 0 | 107 (52.45%) | -1 | – | – | 1.98 |
| RAR | 492 | 0 | 388 (78.86%) | +0 | – | – | 1.68 |
| SD3 | 132 | 0 | 120 (90.91%) | +0 | – | – | 1.49 |
| Switti | 204 | 0 | 182 (89.22%) | +0 | – | – | 1.29 |
| VAR | 204 | 0 | 113 (55.39%) | +1 | – | – | 2.26 |
| real:csafe | 0 | 1308 | – | – | 39 (2.98%) | -7 | 1.45 |
| real:floreview | 0 | 612 | – | – | 13 (2.12%) | -1 | 1.53 |
| real:hdrplus | 0 | 480 | – | – | 2 (0.42%) | -1 | 1.38 |

### Test: test-reposts (1200 items: 600 AI, 600 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 75.00% | 60.83% | 11 of 600 (1.83%) | 79.33% | +0 −3 AI, +0 −2 real |
| mean | 75.00% | 60.83% | 11 of 600 (1.83%) | 79.33% | +0 −3 AI, +0 −2 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 2.27 | 74.33% | 61.00% | 13 of 600 (2.17%) | 79.83% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.69 | 70.50% | 37.00% | 14 of 600 (2.33%) | 80.00% | +4 −3 AI, +5 −4 real |

test-reposts by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 33 (100.00%) | +0 | – | – | 3.18 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 1.33 |
| Infinity | 123 | 0 | 123 (100.00%) | +0 | – | – | 1.56 |
| Janus-Pro | 51 | 0 | 51 (100.00%) | +0 | – | – | 1.73 |
| LlamaGen | 51 | 0 | 49 (96.08%) | +0 | – | – | 2.59 |
| Open-MAGVIT2 | 51 | 0 | 19 (37.25%) | +0 | – | – | 3.63 |
| RAR | 123 | 0 | 81 (65.85%) | +0 | – | – | 3.13 |
| SD3 | 33 | 0 | 30 (90.91%) | +0 | – | – | 1.97 |
| Switti | 51 | 0 | 48 (94.12%) | +0 | – | – | 1.55 |
| VAR | 51 | 0 | 12 (23.53%) | +0 | – | – | 4.53 |
| real:csafe | 0 | 327 | – | – | 10 (3.06%) | +0 | 2.16 |
| real:floreview | 0 | 153 | – | – | 2 (1.31%) | +0 | 1.92 |
| real:hdrplus | 0 | 120 | – | – | 1 (0.83%) | +0 | 1.82 |

test-reposts by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 32 (96.97%) | -1 | – | – | 2.33 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 1.15 |
| Infinity | 123 | 0 | 122 (99.19%) | -1 | – | – | 1.30 |
| Janus-Pro | 51 | 0 | 51 (100.00%) | +0 | – | – | 1.39 |
| LlamaGen | 51 | 0 | 51 (100.00%) | +2 | – | – | 1.84 |
| Open-MAGVIT2 | 51 | 0 | 19 (37.25%) | +0 | – | – | 2.53 |
| RAR | 123 | 0 | 81 (65.85%) | +0 | – | – | 2.11 |
| SD3 | 33 | 0 | 30 (90.91%) | +0 | – | – | 1.76 |
| Switti | 51 | 0 | 48 (94.12%) | +0 | – | – | 1.25 |
| VAR | 51 | 0 | 13 (25.49%) | +1 | – | – | 3.00 |
| real:csafe | 0 | 327 | – | – | 10 (3.06%) | +0 | 1.58 |
| real:floreview | 0 | 153 | – | – | 3 (1.96%) | +1 | 1.50 |
| real:hdrplus | 0 | 120 | – | – | 1 (0.83%) | +0 | 1.44 |

### Test: test-all (19326 items: 9600 AI, 9726 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| baseline (mean, standard schedule), all views | 91.74% | 84.23% | 125 of 9726 (1.29%) | 92.38% | +1 −18 AI, +27 −9 real |
| mean | 91.74% | 84.23% | 125 of 9726 (1.29%) | 92.38% | +1 −18 AI, +27 −9 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| baseline (mean, standard schedule) | 2,0,1,3,4,5 | 1.56 | 92.08% | 74.71% | 107 of 9726 (1.10%) | 92.55% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.28 | 91.59% | 71.79% | 111 of 9726 (1.14%) | 92.43% | +33 −45 AI, +38 −34 real |

test-all by generator: baseline (mean, standard schedule) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 519 (97.56%) | +0 | – | – | 1.92 |
| FLUX.1-dev | 532 | 0 | 525 (98.68%) | +0 | – | – | 1.19 |
| Infinity | 1992 | 0 | 1963 (98.54%) | +0 | – | – | 1.26 |
| Janus-Pro | 804 | 0 | 788 (98.01%) | +0 | – | – | 1.15 |
| LlamaGen | 804 | 0 | 778 (96.77%) | +0 | – | – | 1.41 |
| Open-MAGVIT2 | 804 | 0 | 532 (66.17%) | +0 | – | – | 2.53 |
| RAR | 1992 | 0 | 1877 (94.23%) | +0 | – | – | 1.44 |
| SD3 | 532 | 0 | 518 (97.37%) | +0 | – | – | 1.47 |
| Switti | 804 | 0 | 781 (97.14%) | +0 | – | – | 1.23 |
| VAR | 804 | 0 | 604 (75.12%) | +0 | – | – | 2.52 |
| real:csafe | 0 | 5304 | – | – | 72 (1.36%) | +0 | 1.52 |
| real:floreview | 0 | 2486 | – | – | 26 (1.05%) | +0 | 1.68 |
| real:hdrplus | 0 | 1936 | – | – | 9 (0.46%) | +0 | 1.54 |

test-all by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 502 (94.36%) | -17 | – | – | 1.46 |
| FLUX.1-dev | 532 | 0 | 524 (98.50%) | -1 | – | – | 1.10 |
| Infinity | 1992 | 0 | 1968 (98.80%) | +5 | – | – | 1.13 |
| Janus-Pro | 804 | 0 | 789 (98.13%) | +1 | – | – | 1.07 |
| LlamaGen | 804 | 0 | 783 (97.39%) | +5 | – | – | 1.22 |
| Open-MAGVIT2 | 804 | 0 | 532 (66.17%) | +0 | – | – | 1.76 |
| RAR | 1992 | 0 | 1876 (94.18%) | -1 | – | – | 1.22 |
| SD3 | 532 | 0 | 516 (96.99%) | -2 | – | – | 1.27 |
| Switti | 804 | 0 | 781 (97.14%) | +0 | – | – | 1.12 |
| VAR | 804 | 0 | 602 (74.88%) | -2 | – | – | 1.76 |
| real:csafe | 0 | 5304 | – | – | 72 (1.36%) | +0 | 1.25 |
| real:floreview | 0 | 2486 | – | – | 29 (1.17%) | +3 | 1.35 |
| real:hdrplus | 0 | 1936 | – | – | 10 (0.52%) | +1 | 1.26 |
