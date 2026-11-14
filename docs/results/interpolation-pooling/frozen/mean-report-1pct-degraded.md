4800 items scored by all 2 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**shipped-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.978565 | 2 of 2400 | 0.08% | 0.26% | 45.50% (41.51%–49.51%) | 84.65% |
| 1% empirical FPR | 0.738506 | 24 of 2400 | 1.00% | 1.40% | 76.12% (72.69%–79.42%) | 43.47% |
| frozen cutoff | 0.500000 | 69 of 2400 | 2.88% | 3.50% | 82.25% (79.60%–85.13%) | 22.42% |

shipped-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 92.05% | – | – | – |
| infinity | 492 | 0 | 94.51% | – | – | – |
| janus-pro | 204 | 0 | 92.16% | – | – | – |
| llamagen | 204 | 0 | 90.69% | – | – | – |
| open-magvit2 | 204 | 0 | 52.94% | – | – | – |
| rar | 492 | 0 | 77.24% | – | – | – |
| real:csafe | 0 | 1308 | – | 52 of 1308 | 3.98% | – |
| real:floreview | 0 | 612 | – | 14 of 612 | 2.29% | – |
| real:hdrplus | 0 | 480 | – | 3 of 480 | 0.62% | – |
| sd3 | 132 | 0 | 90.15% | – | – | – |
| switti | 204 | 0 | 89.22% | – | – | – |
| var | 204 | 0 | 50.98% | – | – | – |

Worst family slice with at least 100 items of the class: recall 50.98% (var); FPR 3.98% (real:csafe).

**refitted-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.978565 | 2 of 2400 | 0.08% | 0.26% | 43.46% (39.63%–47.36%) | 84.05% |
| 1% empirical FPR | 0.939822 | 24 of 2400 | 1.00% | 1.40% | 61.58% (57.54%–65.38%) | 38.35% |
| frozen cutoff | 0.500000 | 69 of 2400 | 2.88% | 3.50% | 82.71% (79.97%–85.51%) | 22.52% |

refitted-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 90.15% | – | – | – |
| infinity | 492 | 0 | 96.14% | – | – | – |
| janus-pro | 204 | 0 | 92.65% | – | – | – |
| llamagen | 204 | 0 | 91.18% | – | – | – |
| open-magvit2 | 204 | 0 | 52.94% | – | – | – |
| rar | 492 | 0 | 77.64% | – | – | – |
| real:csafe | 0 | 1308 | – | 46 of 1308 | 3.52% | – |
| real:floreview | 0 | 612 | – | 17 of 612 | 2.78% | – |
| real:hdrplus | 0 | 480 | – | 6 of 480 | 1.25% | – |
| sd3 | 132 | 0 | 90.91% | – | – | – |
| switti | 204 | 0 | 89.22% | – | – | – |
| var | 204 | 0 | 52.45% | – | – | – |

Worst family slice with at least 100 items of the class: recall 52.45% (var); FPR 3.52% (real:csafe).
