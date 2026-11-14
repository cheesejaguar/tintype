14526 items scored by all 1 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**B-photo-clean**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.978595 | 7 of 7326 | 0.10% | 0.18% | 82.64% (81.69%–83.48%) | 89.73% |
| 1% empirical FPR | 0.540272 | 73 of 7326 | 1.00% | 1.21% | 95.94% (95.49%–96.39%) | 49.31% |
| frozen cutoff | 0.500000 | 82 of 7326 | 1.12% | 1.34% | 96.14% (95.70%–96.58%) | 46.46% |

B-photo-clean by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 95.43% | – | – | – |
| csafe | 0 | 3996 | – | 49 of 3996 | 1.23% | – |
| floreview | 0 | 1874 | – | 20 of 1874 | 1.07% | – |
| genimagepp | 1200 | 0 | 99.67% | – | – | – |
| hdrplus | 0 | 1456 | – | 13 of 1456 | 0.89% | – |

Worst dataset slice with at least 100 items of the class: recall 95.43% (arforensics); FPR 1.23% (csafe).
