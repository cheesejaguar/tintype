14526 items scored by all 1 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**C-photo-clean**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.863768 | 7 of 7326 | 0.10% | 0.18% | 90.36% (89.65%–91.02%) | 90.52% |
| 1% empirical FPR | 0.356894 | 73 of 7326 | 1.00% | 1.21% | 96.39% (95.96%–96.82%) | 49.42% |
| frozen cutoff | 0.500000 | 51 of 7326 | 0.70% | 0.88% | 95.71% (95.23%–96.17%) | 58.14% |

C-photo-clean by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 95.00% | – | – | – |
| csafe | 0 | 3996 | – | 31 of 3996 | 0.78% | – |
| floreview | 0 | 1874 | – | 13 of 1874 | 0.69% | – |
| genimagepp | 1200 | 0 | 99.25% | – | – | – |
| hdrplus | 0 | 1456 | – | 7 of 1456 | 0.48% | – |

Worst dataset slice with at least 100 items of the class: recall 95.00% (arforensics); FPR 0.78% (csafe).
