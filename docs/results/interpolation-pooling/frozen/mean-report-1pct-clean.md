14526 items scored by all 2 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**shipped-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.863768 | 7 of 7326 | 0.10% | 0.18% | 90.36% (89.65%–91.02%) | 90.52% |
| 1% empirical FPR | 0.356894 | 73 of 7326 | 1.00% | 1.21% | 96.39% (95.96%–96.82%) | 49.42% |
| frozen cutoff | 0.500000 | 51 of 7326 | 0.70% | 0.88% | 95.71% (95.23%–96.17%) | 58.14% |

shipped-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 99.25% | – | – | – |
| infinity | 1500 | 0 | 100.00% | – | – | – |
| janus-pro | 600 | 0 | 100.00% | – | – | – |
| llamagen | 600 | 0 | 99.00% | – | – | – |
| open-magvit2 | 600 | 0 | 70.83% | – | – | – |
| rar | 1500 | 0 | 99.47% | – | – | – |
| real:csafe | 0 | 3996 | – | 31 of 3996 | 0.78% | – |
| real:floreview | 0 | 1874 | – | 13 of 1874 | 0.69% | – |
| real:hdrplus | 0 | 1456 | – | 7 of 1456 | 0.48% | – |
| sd3 | 400 | 0 | 99.25% | – | – | – |
| switti | 600 | 0 | 99.83% | – | – | – |
| var | 600 | 0 | 81.67% | – | – | – |

Worst family slice with at least 100 items of the class: recall 70.83% (open-magvit2); FPR 0.78% (real:csafe).

**refitted-margins**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.961783 | 7 of 7326 | 0.10% | 0.18% | 81.54% (80.61%–82.38%) | 89.61% |
| 1% empirical FPR | 0.392033 | 73 of 7326 | 1.00% | 1.21% | 95.99% (95.54%–96.45%) | 49.32% |
| frozen cutoff | 0.500000 | 52 of 7326 | 0.71% | 0.89% | 95.31% (94.83%–95.77%) | 57.56% |

refitted-margins by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 98.25% | – | – | – |
| infinity | 1500 | 0 | 99.93% | – | – | – |
| janus-pro | 600 | 0 | 100.00% | – | – | – |
| llamagen | 600 | 0 | 98.67% | – | – | – |
| open-magvit2 | 600 | 0 | 70.50% | – | – | – |
| rar | 1500 | 0 | 99.33% | – | – | – |
| real:csafe | 0 | 3996 | – | 28 of 3996 | 0.70% | – |
| real:floreview | 0 | 1874 | – | 15 of 1874 | 0.80% | – |
| real:hdrplus | 0 | 1456 | – | 9 of 1456 | 0.62% | – |
| sd3 | 400 | 0 | 99.00% | – | – | – |
| switti | 600 | 0 | 99.83% | – | – | – |
| var | 600 | 0 | 79.50% | – | – | – |

Worst family slice with at least 100 items of the class: recall 70.50% (open-magvit2); FPR 0.80% (real:floreview).
