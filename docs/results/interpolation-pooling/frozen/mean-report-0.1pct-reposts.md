1200 items scored by all 2 systems: 600 AI, 600 real; grouping columns found for 1200
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**shipped-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.628111 | 0 of 600 | 0.00% | 0.50% | 56.50% (50.00%–62.52%) | 100.00% |
| 1% empirical FPR | 0.316936 | 6 of 600 | 1.00% | 1.96% | 76.00% (70.79%–81.45%) | 43.43% |
| frozen cutoff | 0.500000 | 2 of 600 | 0.33% | 1.05% | 63.50% (57.22%–69.56%) | 65.80% |

shipped-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 74.24% | – | – | – |
| infinity | 123 | 0 | 92.68% | – | – | – |
| janus-pro | 51 | 0 | 80.39% | – | – | – |
| llamagen | 51 | 0 | 64.71% | – | – | – |
| open-magvit2 | 51 | 0 | 17.65% | – | – | – |
| rar | 123 | 0 | 46.34% | – | – | – |
| real:csafe | 0 | 327 | – | 1 of 327 | 0.31% | – |
| real:floreview | 0 | 153 | – | 1 of 153 | 0.65% | – |
| real:hdrplus | 0 | 120 | – | 0 of 120 | 0.00% | – |
| sd3 | 33 | 0 | 90.91% | – | – | – |
| switti | 51 | 0 | 88.24% | – | – | – |
| var | 51 | 0 | 5.88% | – | – | – |

Worst family slice with at least 100 items of the class: recall 46.34% (rar); FPR 0.65% (real:floreview).

**refitted-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.532785 | 0 of 600 | 0.00% | 0.50% | 60.50% (54.27%–66.50%) | 100.00% |
| 1% empirical FPR | 0.352052 | 6 of 600 | 1.00% | 1.96% | 73.67% (68.15%–79.29%) | 42.66% |
| frozen cutoff | 0.500000 | 2 of 600 | 0.33% | 1.05% | 62.17% (56.08%–68.31%) | 65.32% |

refitted-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 69.70% | – | – | – |
| infinity | 123 | 0 | 91.87% | – | – | – |
| janus-pro | 51 | 0 | 78.43% | – | – | – |
| llamagen | 51 | 0 | 62.75% | – | – | – |
| open-magvit2 | 51 | 0 | 17.65% | – | – | – |
| rar | 123 | 0 | 46.34% | – | – | – |
| real:csafe | 0 | 327 | – | 1 of 327 | 0.31% | – |
| real:floreview | 0 | 153 | – | 1 of 153 | 0.65% | – |
| real:hdrplus | 0 | 120 | – | 0 of 120 | 0.00% | – |
| sd3 | 33 | 0 | 84.85% | – | – | – |
| switti | 51 | 0 | 88.24% | – | – | – |
| var | 51 | 0 | 5.88% | – | – | – |

Worst family slice with at least 100 items of the class: recall 46.34% (rar); FPR 0.65% (real:floreview).
