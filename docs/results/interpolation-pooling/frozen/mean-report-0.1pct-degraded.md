4800 items scored by all 2 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**shipped-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.731974 | 2 of 2400 | 0.08% | 0.26% | 63.62% (59.49%–67.40%) | 88.52% |
| 1% empirical FPR | 0.355329 | 24 of 2400 | 1.00% | 1.40% | 78.67% (75.75%–81.62%) | 44.28% |
| frozen cutoff | 0.500000 | 13 of 2400 | 0.54% | 0.86% | 72.62% (69.19%–75.93%) | 57.52% |

shipped-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 76.52% | – | – | – |
| infinity | 492 | 0 | 88.41% | – | – | – |
| janus-pro | 204 | 0 | 83.82% | – | – | – |
| llamagen | 204 | 0 | 77.94% | – | – | – |
| open-magvit2 | 204 | 0 | 41.67% | – | – | – |
| rar | 492 | 0 | 67.48% | – | – | – |
| real:csafe | 0 | 1308 | – | 10 of 1308 | 0.76% | – |
| real:floreview | 0 | 612 | – | 3 of 612 | 0.49% | – |
| real:hdrplus | 0 | 480 | – | 0 of 480 | 0.00% | – |
| sd3 | 132 | 0 | 86.36% | – | – | – |
| switti | 204 | 0 | 84.31% | – | – | – |
| var | 204 | 0 | 35.78% | – | – | – |

Worst family slice with at least 100 items of the class: recall 35.78% (var); FPR 0.76% (real:csafe).

**refitted-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.699829 | 2 of 2400 | 0.08% | 0.26% | 63.12% (59.32%–66.87%) | 88.44% |
| 1% empirical FPR | 0.428476 | 24 of 2400 | 1.00% | 1.40% | 75.71% (72.41%–78.75%) | 43.33% |
| frozen cutoff | 0.500000 | 19 of 2400 | 0.79% | 1.16% | 72.00% (68.61%–75.35%) | 47.88% |

refitted-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 75.76% | – | – | – |
| infinity | 492 | 0 | 87.80% | – | – | – |
| janus-pro | 204 | 0 | 82.84% | – | – | – |
| llamagen | 204 | 0 | 75.98% | – | – | – |
| open-magvit2 | 204 | 0 | 41.67% | – | – | – |
| rar | 492 | 0 | 67.48% | – | – | – |
| real:csafe | 0 | 1308 | – | 16 of 1308 | 1.22% | – |
| real:floreview | 0 | 612 | – | 2 of 612 | 0.33% | – |
| real:hdrplus | 0 | 480 | – | 1 of 480 | 0.21% | – |
| sd3 | 132 | 0 | 83.33% | – | – | – |
| switti | 204 | 0 | 84.31% | – | – | – |
| var | 204 | 0 | 35.78% | – | – | – |

Worst family slice with at least 100 items of the class: recall 35.78% (var); FPR 1.22% (real:csafe).
