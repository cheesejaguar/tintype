14526 items scored by all 1 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**C-strict-clean**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.459008 | 7 of 7326 | 0.10% | 0.18% | 93.24% (92.64%–93.79%) | 90.79% |
| 1% empirical FPR | 0.206466 | 73 of 7326 | 1.00% | 1.21% | 96.21% (95.75%–96.63%) | 49.37% |
| frozen cutoff | 0.500000 | 5 of 7326 | 0.07% | 0.14% | 92.72% (92.15%–93.29%) | 93.21% |

C-strict-clean by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 92.00% | – | – | – |
| csafe | 0 | 3996 | – | 3 of 3996 | 0.08% | – |
| floreview | 0 | 1874 | – | 1 of 1874 | 0.05% | – |
| genimagepp | 1200 | 0 | 96.33% | – | – | – |
| hdrplus | 0 | 1456 | – | 1 of 1456 | 0.07% | – |

Worst dataset slice with at least 100 items of the class: recall 92.00% (arforensics); FPR 0.08% (csafe).
