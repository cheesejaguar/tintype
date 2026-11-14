14526 items scored by all 2 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**alpha-0**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.722041 | 7 of 7326 | 0.10% | 0.18% | 92.44% (91.83%–93.01%) | 90.72% |
| 1% empirical FPR | 0.355125 | 73 of 7326 | 1.00% | 1.21% | 95.89% (95.42%–96.35%) | 49.29% |
| frozen cutoff | 0.384758 | 62 of 7326 | 0.85% | 1.04% | 95.76% (95.29%–96.22%) | 53.34% |

alpha-0 by family, at the frozen cutoff 0.3847578:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 99.50% | – | – | – |
| infinity | 1500 | 0 | 100.00% | – | – | – |
| janus-pro | 600 | 0 | 100.00% | – | – | – |
| llamagen | 600 | 0 | 99.00% | – | – | – |
| open-magvit2 | 600 | 0 | 70.83% | – | – | – |
| rar | 1500 | 0 | 99.47% | – | – | – |
| real:csafe | 0 | 3996 | – | 44 of 3996 | 1.10% | – |
| real:floreview | 0 | 1874 | – | 12 of 1874 | 0.64% | – |
| real:hdrplus | 0 | 1456 | – | 6 of 1456 | 0.41% | – |
| sd3 | 400 | 0 | 99.50% | – | – | – |
| switti | 600 | 0 | 99.83% | – | – | – |
| var | 600 | 0 | 81.83% | – | – | – |

Worst family slice with at least 100 items of the class: recall 70.83% (open-magvit2); FPR 1.10% (real:csafe).

alpha-0 by source, at the frozen cutoff 0.3847578:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 95.02% | – | – | – |
| csafe-iphone14pro | 0 | 3996 | – | 44 of 3996 | 1.10% | – |
| floreview-apple | 0 | 249 | – | 0 of 249 | 0.00% | – |
| floreview-doogee | 0 | 42 | – | 3 of 42 | 7.14% | – |
| floreview-google | 0 | 125 | – | 0 of 125 | 0.00% | – |
| floreview-huawei | 0 | 249 | – | 1 of 249 | 0.40% | – |
| floreview-lenovo | 0 | 42 | – | 0 of 42 | 0.00% | – |
| floreview-lg | 0 | 126 | – | 0 of 126 | 0.00% | – |
| floreview-motorola | 0 | 248 | – | 1 of 248 | 0.40% | – |
| floreview-oneplus | 0 | 84 | – | 1 of 84 | 1.19% | – |
| floreview-samsung | 0 | 377 | – | 0 of 377 | 0.00% | – |
| floreview-sony | 0 | 40 | – | 0 of 40 | 0.00% | – |
| floreview-xiaomi | 0 | 292 | – | 6 of 292 | 2.05% | – |
| genimagepp | 1200 | 0 | 99.50% | – | – | – |
| hdrplus | 0 | 1456 | – | 6 of 1456 | 0.41% | – |

Worst source slice with at least 100 items of the class: recall 95.02% (arforensics); FPR 2.05% (floreview-xiaomi).

**phase3-0.30**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.760167 | 7 of 7326 | 0.10% | 0.18% | 93.00% (92.42%–93.57%) | 90.77% |
| 1% empirical FPR | 0.443977 | 73 of 7326 | 1.00% | 1.21% | 96.08% (95.62%–96.52%) | 49.34% |
| frozen cutoff | 0.500000 | 52 of 7326 | 0.71% | 0.89% | 95.75% (95.29%–96.21%) | 57.67% |

phase3-0.30 by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 99.75% | – | – | – |
| infinity | 1500 | 0 | 100.00% | – | – | – |
| janus-pro | 600 | 0 | 100.00% | – | – | – |
| llamagen | 600 | 0 | 98.83% | – | – | – |
| open-magvit2 | 600 | 0 | 70.67% | – | – | – |
| rar | 1500 | 0 | 99.27% | – | – | – |
| real:csafe | 0 | 3996 | – | 36 of 3996 | 0.90% | – |
| real:floreview | 0 | 1874 | – | 11 of 1874 | 0.59% | – |
| real:hdrplus | 0 | 1456 | – | 5 of 1456 | 0.34% | – |
| sd3 | 400 | 0 | 99.75% | – | – | – |
| switti | 600 | 0 | 99.83% | – | – | – |
| var | 600 | 0 | 82.00% | – | – | – |

Worst family slice with at least 100 items of the class: recall 70.67% (open-magvit2); FPR 0.90% (real:csafe).

phase3-0.30 by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 94.95% | – | – | – |
| csafe-iphone14pro | 0 | 3996 | – | 36 of 3996 | 0.90% | – |
| floreview-apple | 0 | 249 | – | 0 of 249 | 0.00% | – |
| floreview-doogee | 0 | 42 | – | 3 of 42 | 7.14% | – |
| floreview-google | 0 | 125 | – | 0 of 125 | 0.00% | – |
| floreview-huawei | 0 | 249 | – | 1 of 249 | 0.40% | – |
| floreview-lenovo | 0 | 42 | – | 0 of 42 | 0.00% | – |
| floreview-lg | 0 | 126 | – | 0 of 126 | 0.00% | – |
| floreview-motorola | 0 | 248 | – | 1 of 248 | 0.40% | – |
| floreview-oneplus | 0 | 84 | – | 1 of 84 | 1.19% | – |
| floreview-samsung | 0 | 377 | – | 0 of 377 | 0.00% | – |
| floreview-sony | 0 | 40 | – | 0 of 40 | 0.00% | – |
| floreview-xiaomi | 0 | 292 | – | 5 of 292 | 1.71% | – |
| genimagepp | 1200 | 0 | 99.75% | – | – | – |
| hdrplus | 0 | 1456 | – | 5 of 1456 | 0.34% | – |

Worst source slice with at least 100 items of the class: recall 94.95% (arforensics); FPR 1.71% (floreview-xiaomi).
