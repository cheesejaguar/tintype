3000 items scored by all 2 systems: 3000 AI, 0 real; grouping columns found for 3000
Recall intervals: 95% bootstrap over 3000 parent clusters (3000 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**alpha-0**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| frozen cutoff | 0.384758 | – | – | – | 99.73% (99.53%–99.90%) | – |

alpha-0 by dataset, at the frozen cutoff 0.3847578:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 3000 | 0 | 99.73% | – | – | – |

Worst dataset slice with at least 100 items of the class: recall 99.73% (arforensics); FPR none large enough.

**phase3-0.30**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| frozen cutoff | 0.500000 | – | – | – | 99.63% (99.40%–99.83%) | – |

phase3-0.30 by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 3000 | 0 | 99.63% | – | – | – |

Worst dataset slice with at least 100 items of the class: recall 99.63% (arforensics); FPR none large enough.
