14526 items scored by all 1 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-photo-clean**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.746518 | 7 of 7326 | 0.10% | 0.18% | 91.46% (90.78%–92.07%) | 90.63% |
| 1% empirical FPR | 0.340124 | 73 of 7326 | 1.00% | 1.21% | 96.08% (95.62%–96.54%) | 49.34% |
| frozen cutoff | 0.500000 | 35 of 7326 | 0.48% | 0.63% | 94.90% (94.40%–95.39%) | 66.74% |

A-photo-clean, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 7326 (0.00%) | 0 of 7200 (0.00%) | 0 of 0 | 35 of 7326 | 0.48% | 0.63% | 94.90% (94.40%–95.39%) | 94.90% | 66.74% | 0.00% |
| margin 0.75 | 97.81% | 70 of 7326 (0.96%) | 248 of 7200 (3.44%) | 318 of 318 | 9 of 7326 | 0.12% | 0.21% | 92.78% (92.19%–93.34%) | 94.90% | 88.41% | 0.98% |

A-photo-clean by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 94.10% | – | – | – |
| csafe | 0 | 3996 | – | 26 of 3996 | 0.65% | – |
| floreview | 0 | 1874 | – | 4 of 1874 | 0.21% | – |
| genimagepp | 1200 | 0 | 98.92% | – | – | – |
| hdrplus | 0 | 1456 | – | 5 of 1456 | 0.34% | – |

Worst dataset slice with at least 100 items of the class: recall 94.10% (arforensics); FPR 0.65% (csafe).
