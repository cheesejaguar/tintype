14526 items scored by all 1 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-strict-clean**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.427603 | 7 of 7326 | 0.10% | 0.18% | 92.85% (92.25%–93.40%) | 90.75% |
| 1% empirical FPR | 0.167591 | 73 of 7326 | 1.00% | 1.21% | 96.46% (96.01%–96.86%) | 49.44% |
| frozen cutoff | 0.500000 | 6 of 7326 | 0.08% | 0.16% | 91.72% (91.04%–92.34%) | 91.88% |

A-strict-clean, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 7326 (0.00%) | 0 of 7200 (0.00%) | 0 of 0 | 6 of 7326 | 0.08% | 0.16% | 91.72% (91.04%–92.34%) | 91.72% | 91.88% | 0.00% |
| margin 1.5 | 91.61% | 63 of 7326 (0.86%) | 1156 of 7200 (16.06%) | 1219 of 1219 | 0 of 7326 | 0.00% | 0.04% | 80.15% (79.20%–81.02%) | 91.72% | 100.00% | 1.01% |

A-strict-clean by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 91.07% | – | – | – |
| csafe | 0 | 3996 | – | 4 of 3996 | 0.10% | – |
| floreview | 0 | 1874 | – | 2 of 1874 | 0.11% | – |
| genimagepp | 1200 | 0 | 95.00% | – | – | – |
| hdrplus | 0 | 1456 | – | 0 of 1456 | 0.00% | – |

Worst dataset slice with at least 100 items of the class: recall 91.07% (arforensics); FPR 0.11% (floreview).
