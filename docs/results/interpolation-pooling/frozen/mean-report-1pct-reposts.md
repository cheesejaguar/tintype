1200 items scored by all 2 systems: 600 AI, 600 real; grouping columns found for 1200
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**shipped-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.860900 | 0 of 600 | 0.00% | 0.50% | 59.00% (52.56%–64.68%) | 100.00% |
| 1% empirical FPR | 0.648501 | 6 of 600 | 1.00% | 1.96% | 73.17% (67.70%–78.84%) | 42.50% |
| frozen cutoff | 0.500000 | 14 of 600 | 2.33% | 3.62% | 78.00% (72.94%–83.49%) | 25.24% |

shipped-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 100.00% | – | – | – |
| infinity | 123 | 0 | 99.19% | – | – | – |
| janus-pro | 51 | 0 | 100.00% | – | – | – |
| llamagen | 51 | 0 | 96.08% | – | – | – |
| open-magvit2 | 51 | 0 | 37.25% | – | – | – |
| rar | 123 | 0 | 61.79% | – | – | – |
| real:csafe | 0 | 327 | – | 10 of 327 | 3.06% | – |
| real:floreview | 0 | 153 | – | 3 of 153 | 1.96% | – |
| real:hdrplus | 0 | 120 | – | 1 of 120 | 0.83% | – |
| sd3 | 33 | 0 | 90.91% | – | – | – |
| switti | 51 | 0 | 94.12% | – | – | – |
| var | 51 | 0 | 13.73% | – | – | – |

Worst family slice with at least 100 items of the class: recall 61.79% (rar); FPR 3.06% (real:csafe).

**refitted-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.961461 | 0 of 600 | 0.00% | 0.50% | 39.50% (33.51%–45.41%) | 100.00% |
| 1% empirical FPR | 0.748173 | 6 of 600 | 1.00% | 1.96% | 68.50% (62.64%–74.27%) | 40.90% |
| frozen cutoff | 0.500000 | 18 of 600 | 3.00% | 4.42% | 78.50% (73.40%–83.51%) | 20.91% |

refitted-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 93.94% | – | – | – |
| infinity | 123 | 0 | 100.00% | – | – | – |
| janus-pro | 51 | 0 | 100.00% | – | – | – |
| llamagen | 51 | 0 | 94.12% | – | – | – |
| open-magvit2 | 51 | 0 | 39.22% | – | – | – |
| rar | 123 | 0 | 63.41% | – | – | – |
| real:csafe | 0 | 327 | – | 14 of 327 | 4.28% | – |
| real:floreview | 0 | 153 | – | 3 of 153 | 1.96% | – |
| real:hdrplus | 0 | 120 | – | 1 of 120 | 0.83% | – |
| sd3 | 33 | 0 | 93.94% | – | – | – |
| switti | 51 | 0 | 94.12% | – | – | – |
| var | 51 | 0 | 19.61% | – | – | – |

Worst family slice with at least 100 items of the class: recall 63.41% (rar); FPR 4.28% (real:csafe).
