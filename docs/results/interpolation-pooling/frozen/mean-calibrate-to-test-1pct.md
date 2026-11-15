Fit items: 12457 (6600 AI, 5857 real) from 1 file(s). Target false-positive rate 1.00%, averaged over the 2 real sources with at least 100 items.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 0.985 | 0.561 | 6 / 3 / 2 / 1.5 / 1 | 1.70 | 0.9901 / 0.8756 | 0.9904 / 0.8762 |
| mean | 2,0,1,3,4,5 | 0.985 | 0.561 | 4.75 / 2 / 0.25 / 0.5 / 0 | 1.39 | 0.9901 / 0.8756 | 0.9892 / 0.8750 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 0.9853): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":4.75,"above":4.75},{"views":2,"below":2.0,"above":2.0},{"views":3,"below":0.25,"above":0.25},{"views":4,"below":0.5,"above":0.5},{"views":5,"below":0.0,"above":0.0}]}`

### Test: test-clean (14526 items: 7200 AI, 7326 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 95.89% | 92.44% | 62 of 7326 (0.85%) | 95.76% | +4 −0 AI, +15 −4 real |
| mean | 95.89% | 92.44% | 62 of 7326 (0.85%) | 95.76% | +4 −0 AI, +15 −4 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.44 | 96.39% | 90.36% | 51 of 7326 (0.70%) | 95.71% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.24 | 95.99% | 81.54% | 52 of 7326 (0.71%) | 95.31% | +11 −40 AI, +19 −18 real |

test-clean by generator: shipped (mean, spec as is) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 395 (98.75%) | +0 | – | – | 1.77 |
| FLUX.1-dev | 400 | 0 | 399 (99.75%) | +0 | – | – | 1.12 |
| Infinity | 1500 | 0 | 1500 (100.00%) | +0 | – | – | 1.16 |
| Janus-Pro | 600 | 0 | 600 (100.00%) | +0 | – | – | 1.04 |
| LlamaGen | 600 | 0 | 594 (99.00%) | +0 | – | – | 1.25 |
| Open-MAGVIT2 | 600 | 0 | 425 (70.83%) | +0 | – | – | 2.44 |
| RAR | 1500 | 0 | 1492 (99.47%) | +0 | – | – | 1.14 |
| SD3 | 400 | 0 | 397 (99.25%) | +0 | – | – | 1.35 |
| Switti | 600 | 0 | 599 (99.83%) | +0 | – | – | 1.12 |
| VAR | 600 | 0 | 490 (81.67%) | +0 | – | – | 2.27 |
| real:csafe | 0 | 3996 | – | – | 31 (0.78%) | +0 | 1.43 |
| real:floreview | 0 | 1874 | – | – | 13 (0.69%) | +0 | 1.59 |
| real:hdrplus | 0 | 1456 | – | – | 7 (0.48%) | +0 | 1.49 |

test-clean by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 400 | 0 | 388 (97.00%) | -7 | – | – | 1.42 |
| FLUX.1-dev | 400 | 0 | 398 (99.50%) | -1 | – | – | 1.07 |
| Infinity | 1500 | 0 | 1499 (99.93%) | -1 | – | – | 1.09 |
| Janus-Pro | 600 | 0 | 600 (100.00%) | +0 | – | – | 1.01 |
| LlamaGen | 600 | 0 | 592 (98.67%) | -2 | – | – | 1.14 |
| Open-MAGVIT2 | 600 | 0 | 423 (70.50%) | -2 | – | – | 1.73 |
| RAR | 1500 | 0 | 1490 (99.33%) | -2 | – | – | 1.06 |
| SD3 | 400 | 0 | 396 (99.00%) | -1 | – | – | 1.22 |
| Switti | 600 | 0 | 599 (99.83%) | +0 | – | – | 1.07 |
| VAR | 600 | 0 | 477 (79.50%) | -13 | – | – | 1.67 |
| real:csafe | 0 | 3996 | – | – | 28 (0.70%) | -3 | 1.24 |
| real:floreview | 0 | 1874 | – | – | 15 (0.80%) | +2 | 1.34 |
| real:hdrplus | 0 | 1456 | – | – | 9 (0.62%) | +2 | 1.27 |

### Test: test-degraded (4800 items: 2400 AI, 2400 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 74.08% | 59.00% | 76 of 2400 (3.17%) | 81.50% | +1 −19 AI, +11 −4 real |
| mean | 74.08% | 59.00% | 76 of 2400 (3.17%) | 81.50% | +1 −19 AI, +11 −4 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.96 | 76.12% | 45.50% | 69 of 2400 (2.88%) | 82.25% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.53 | 61.58% | 43.46% | 69 of 2400 (2.88%) | 82.71% | +24 −13 AI, +18 −18 real |

test-degraded by generator: shipped (mean, spec as is) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 118 (89.39%) | +0 | – | – | 2.67 |
| FLUX.1-dev | 132 | 0 | 125 (94.70%) | +0 | – | – | 1.44 |
| Infinity | 492 | 0 | 465 (94.51%) | +0 | – | – | 1.54 |
| Janus-Pro | 204 | 0 | 188 (92.16%) | +0 | – | – | 1.50 |
| LlamaGen | 204 | 0 | 185 (90.69%) | +0 | – | – | 1.97 |
| Open-MAGVIT2 | 204 | 0 | 108 (52.94%) | +0 | – | – | 2.81 |
| RAR | 492 | 0 | 380 (77.24%) | +0 | – | – | 2.28 |
| SD3 | 132 | 0 | 119 (90.15%) | +0 | – | – | 1.83 |
| Switti | 204 | 0 | 182 (89.22%) | +0 | – | – | 1.60 |
| VAR | 204 | 0 | 104 (50.98%) | +0 | – | – | 3.31 |
| real:csafe | 0 | 1308 | – | – | 52 (3.98%) | +0 | 1.87 |
| real:floreview | 0 | 612 | – | – | 14 (2.29%) | +0 | 1.93 |
| real:hdrplus | 0 | 480 | – | – | 3 (0.62%) | +0 | 1.71 |

test-degraded by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 132 | 0 | 112 (84.85%) | -6 | – | – | 1.89 |
| FLUX.1-dev | 132 | 0 | 126 (95.45%) | +1 | – | – | 1.21 |
| Infinity | 492 | 0 | 473 (96.14%) | +8 | – | – | 1.29 |
| Janus-Pro | 204 | 0 | 189 (92.65%) | +1 | – | – | 1.28 |
| LlamaGen | 204 | 0 | 186 (91.18%) | +1 | – | – | 1.59 |
| Open-MAGVIT2 | 204 | 0 | 108 (52.94%) | +0 | – | – | 1.99 |
| RAR | 492 | 0 | 382 (77.64%) | +2 | – | – | 1.67 |
| SD3 | 132 | 0 | 120 (90.91%) | +1 | – | – | 1.52 |
| Switti | 204 | 0 | 182 (89.22%) | +0 | – | – | 1.33 |
| VAR | 204 | 0 | 107 (52.45%) | +3 | – | – | 2.15 |
| real:csafe | 0 | 1308 | – | – | 46 (3.52%) | -6 | 1.50 |
| real:floreview | 0 | 612 | – | – | 17 (2.78%) | +3 | 1.56 |
| real:hdrplus | 0 | 480 | – | – | 6 (1.25%) | +3 | 1.42 |

### Test: test-reposts (1200 items: 600 AI, 600 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 74.83% | 59.83% | 11 of 600 (1.83%) | 77.50% | +0 −3 AI, +0 −3 real |
| mean | 74.83% | 59.83% | 11 of 600 (1.83%) | 77.50% | +0 −3 AI, +0 −3 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 2.28 | 73.17% | 59.00% | 14 of 600 (2.33%) | 78.00% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.72 | 68.50% | 39.50% | 18 of 600 (3.00%) | 78.50% | +8 −5 AI, +8 −4 real |

test-reposts by generator: shipped (mean, spec as is) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 33 (100.00%) | +0 | – | – | 3.27 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 1.39 |
| Infinity | 123 | 0 | 122 (99.19%) | +0 | – | – | 1.59 |
| Janus-Pro | 51 | 0 | 51 (100.00%) | +0 | – | – | 1.86 |
| LlamaGen | 51 | 0 | 49 (96.08%) | +0 | – | – | 2.71 |
| Open-MAGVIT2 | 51 | 0 | 19 (37.25%) | +0 | – | – | 3.65 |
| RAR | 123 | 0 | 76 (61.79%) | +0 | – | – | 3.18 |
| SD3 | 33 | 0 | 30 (90.91%) | +0 | – | – | 1.97 |
| Switti | 51 | 0 | 48 (94.12%) | +0 | – | – | 1.65 |
| VAR | 51 | 0 | 7 (13.73%) | +0 | – | – | 4.39 |
| real:csafe | 0 | 327 | – | – | 10 (3.06%) | +0 | 2.13 |
| real:floreview | 0 | 153 | – | – | 3 (1.96%) | +0 | 1.86 |
| real:hdrplus | 0 | 120 | – | – | 1 (0.83%) | +0 | 1.82 |

test-reposts by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 33 | 0 | 29 (87.88%) | -4 | – | – | 2.27 |
| FLUX.1-dev | 33 | 0 | 33 (100.00%) | +0 | – | – | 1.18 |
| Infinity | 123 | 0 | 123 (100.00%) | +1 | – | – | 1.34 |
| Janus-Pro | 51 | 0 | 51 (100.00%) | +0 | – | – | 1.51 |
| LlamaGen | 51 | 0 | 48 (94.12%) | -1 | – | – | 2.04 |
| Open-MAGVIT2 | 51 | 0 | 20 (39.22%) | +1 | – | – | 2.41 |
| RAR | 123 | 0 | 78 (63.41%) | +2 | – | – | 2.16 |
| SD3 | 33 | 0 | 31 (93.94%) | +1 | – | – | 1.76 |
| Switti | 51 | 0 | 48 (94.12%) | +0 | – | – | 1.35 |
| VAR | 51 | 0 | 10 (19.61%) | +3 | – | – | 2.67 |
| real:csafe | 0 | 327 | – | – | 14 (4.28%) | +4 | 1.62 |
| real:floreview | 0 | 153 | – | – | 3 (1.96%) | +0 | 1.58 |
| real:hdrplus | 0 | 120 | – | – | 1 (0.83%) | +0 | 1.48 |

### Test: test-all (19326 items: 9600 AI, 9726 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 91.20% | 83.92% | 138 of 9726 (1.42%) | 92.20% | +5 −19 AI, +26 −8 real |
| mean | 91.20% | 83.92% | 138 of 9726 (1.42%) | 92.20% | +5 −19 AI, +26 −8 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.57 | 91.58% | 70.29% | 120 of 9726 (1.23%) | 92.34% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.31 | 91.17% | 68.33% | 121 of 9726 (1.24%) | 92.16% | +35 −53 AI, +37 −36 real |

test-all by generator: shipped (mean, spec as is) | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 513 (96.43%) | +0 | – | – | 1.99 |
| FLUX.1-dev | 532 | 0 | 524 (98.50%) | +0 | – | – | 1.20 |
| Infinity | 1992 | 0 | 1965 (98.64%) | +0 | – | – | 1.25 |
| Janus-Pro | 804 | 0 | 788 (98.01%) | +0 | – | – | 1.16 |
| LlamaGen | 804 | 0 | 779 (96.89%) | +0 | – | – | 1.43 |
| Open-MAGVIT2 | 804 | 0 | 533 (66.29%) | +0 | – | – | 2.53 |
| RAR | 1992 | 0 | 1872 (93.98%) | +0 | – | – | 1.42 |
| SD3 | 532 | 0 | 516 (96.99%) | +0 | – | – | 1.47 |
| Switti | 804 | 0 | 781 (97.14%) | +0 | – | – | 1.24 |
| VAR | 804 | 0 | 594 (73.88%) | +0 | – | – | 2.54 |
| real:csafe | 0 | 5304 | – | – | 83 (1.56%) | +0 | 1.54 |
| real:floreview | 0 | 2486 | – | – | 27 (1.09%) | +0 | 1.67 |
| real:hdrplus | 0 | 1936 | – | – | 10 (0.52%) | +0 | 1.54 |

test-all by generator: mean | 2,0,1,3,4,5, with early exit at the fitted cutoff. Changes are against the shipped row.

| generator | AI | Real | AI flagged (recall) | Change | Real flagged (FPR) | Change | Views per image |
|---|---|---|---|---|---|---|---|
| FLUX.1-Krea | 532 | 0 | 500 (93.98%) | -13 | – | – | 1.54 |
| FLUX.1-dev | 532 | 0 | 524 (98.50%) | +0 | – | – | 1.11 |
| Infinity | 1992 | 0 | 1972 (99.00%) | +7 | – | – | 1.14 |
| Janus-Pro | 804 | 0 | 789 (98.13%) | +1 | – | – | 1.08 |
| LlamaGen | 804 | 0 | 778 (96.77%) | -1 | – | – | 1.25 |
| Open-MAGVIT2 | 804 | 0 | 531 (66.04%) | -2 | – | – | 1.79 |
| RAR | 1992 | 0 | 1872 (93.98%) | +0 | – | – | 1.21 |
| SD3 | 532 | 0 | 516 (96.99%) | +0 | – | – | 1.30 |
| Switti | 804 | 0 | 781 (97.14%) | +0 | – | – | 1.13 |
| VAR | 804 | 0 | 584 (72.64%) | -10 | – | – | 1.79 |
| real:csafe | 0 | 5304 | – | – | 74 (1.40%) | -9 | 1.30 |
| real:floreview | 0 | 2486 | – | – | 32 (1.29%) | +5 | 1.39 |
| real:hdrplus | 0 | 1936 | – | – | 15 (0.77%) | +5 | 1.30 |
