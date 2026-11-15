14526 items scored by all 1 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**B-strict-clean**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.506686 | 7 of 7326 | 0.10% | 0.18% | 93.33% (92.75%–93.88%) | 90.80% |
| 1% empirical FPR | 0.207061 | 73 of 7326 | 1.00% | 1.21% | 96.40% (95.96%–96.82%) | 49.42% |
| frozen cutoff | 0.500000 | 9 of 7326 | 0.12% | 0.21% | 93.47% (92.90%–94.01%) | 88.49% |

B-strict-clean by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 92.73% | – | – | – |
| csafe | 0 | 3996 | – | 5 of 3996 | 0.13% | – |
| floreview | 0 | 1874 | – | 2 of 1874 | 0.11% | – |
| genimagepp | 1200 | 0 | 97.17% | – | – | – |
| hdrplus | 0 | 1456 | – | 2 of 1456 | 0.14% | – |

Worst dataset slice with at least 100 items of the class: recall 92.73% (arforensics); FPR 0.14% (hdrplus).
