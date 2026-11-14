14526 items scored by all 2 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**shipped-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.459008 | 7 of 7326 | 0.10% | 0.18% | 93.24% (92.64%–93.79%) | 90.79% |
| 1% empirical FPR | 0.206466 | 73 of 7326 | 1.00% | 1.21% | 96.21% (95.75%–96.63%) | 49.37% |
| frozen cutoff | 0.500000 | 5 of 7326 | 0.07% | 0.14% | 92.72% (92.15%–93.29%) | 93.21% |

shipped-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 95.38% | – | – | – |
| infinity | 1500 | 0 | 99.33% | – | – | – |
| janus-pro | 600 | 0 | 99.83% | – | – | – |
| llamagen | 600 | 0 | 96.83% | – | – | – |
| open-magvit2 | 600 | 0 | 59.17% | – | – | – |
| rar | 1500 | 0 | 98.60% | – | – | – |
| real:csafe | 0 | 3996 | – | 3 of 3996 | 0.08% | – |
| real:floreview | 0 | 1874 | – | 1 of 1874 | 0.05% | – |
| real:hdrplus | 0 | 1456 | – | 1 of 1456 | 0.07% | – |
| sd3 | 400 | 0 | 98.25% | – | – | – |
| switti | 600 | 0 | 99.00% | – | – | – |
| var | 600 | 0 | 70.33% | – | – | – |

Worst family slice with at least 100 items of the class: recall 59.17% (open-magvit2); FPR 0.08% (real:csafe).

**refitted-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.517510 | 7 of 7326 | 0.10% | 0.18% | 92.38% (91.74%–92.96%) | 90.71% |
| 1% empirical FPR | 0.190889 | 73 of 7326 | 1.00% | 1.21% | 96.53% (96.10%–96.93%) | 49.46% |
| frozen cutoff | 0.500000 | 10 of 7326 | 0.14% | 0.23% | 92.82% (92.25%–93.36%) | 87.29% |

refitted-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 96.00% | – | – | – |
| infinity | 1500 | 0 | 99.27% | – | – | – |
| janus-pro | 600 | 0 | 99.83% | – | – | – |
| llamagen | 600 | 0 | 97.00% | – | – | – |
| open-magvit2 | 600 | 0 | 59.00% | – | – | – |
| rar | 1500 | 0 | 98.60% | – | – | – |
| real:csafe | 0 | 3996 | – | 6 of 3996 | 0.15% | – |
| real:floreview | 0 | 1874 | – | 3 of 1874 | 0.16% | – |
| real:hdrplus | 0 | 1456 | – | 1 of 1456 | 0.07% | – |
| sd3 | 400 | 0 | 98.75% | – | – | – |
| switti | 600 | 0 | 99.00% | – | – | – |
| var | 600 | 0 | 70.50% | – | – | – |

Worst family slice with at least 100 items of the class: recall 59.00% (open-magvit2); FPR 0.16% (real:floreview).
