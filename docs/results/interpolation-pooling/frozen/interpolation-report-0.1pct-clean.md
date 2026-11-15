14526 items scored by all 2 systems: 7200 AI, 7326 real; grouping columns found for 14526
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**alpha-0**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.722041 | 7 of 7326 | 0.10% | 0.18% | 92.44% (91.83%–93.01%) | 90.72% |
| 1% empirical FPR | 0.355125 | 73 of 7326 | 1.00% | 1.21% | 95.89% (95.42%–96.35%) | 49.29% |
| frozen cutoff | 0.699543 | 11 of 7326 | 0.15% | 0.25% | 92.92% (92.34%–93.48%) | 86.21% |

alpha-0 by family, at the frozen cutoff 0.6995432:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 96.62% | – | – | – |
| infinity | 1500 | 0 | 99.33% | – | – | – |
| janus-pro | 600 | 0 | 99.83% | – | – | – |
| llamagen | 600 | 0 | 97.00% | – | – | – |
| open-magvit2 | 600 | 0 | 59.00% | – | – | – |
| rar | 1500 | 0 | 98.60% | – | – | – |
| real:csafe | 0 | 3996 | – | 7 of 3996 | 0.18% | – |
| real:floreview | 0 | 1874 | – | 3 of 1874 | 0.16% | – |
| real:hdrplus | 0 | 1456 | – | 1 of 1456 | 0.07% | – |
| sd3 | 400 | 0 | 99.00% | – | – | – |
| switti | 600 | 0 | 99.00% | – | – | – |
| var | 600 | 0 | 70.50% | – | – | – |

Worst family slice with at least 100 items of the class: recall 59.00% (open-magvit2); FPR 0.18% (real:csafe).

alpha-0 by source, at the frozen cutoff 0.6995432:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 92.02% | – | – | – |
| csafe-iphone14pro | 0 | 3996 | – | 7 of 3996 | 0.18% | – |
| floreview-apple | 0 | 249 | – | 0 of 249 | 0.00% | – |
| floreview-doogee | 0 | 42 | – | 1 of 42 | 2.38% | – |
| floreview-google | 0 | 125 | – | 0 of 125 | 0.00% | – |
| floreview-huawei | 0 | 249 | – | 0 of 249 | 0.00% | – |
| floreview-lenovo | 0 | 42 | – | 0 of 42 | 0.00% | – |
| floreview-lg | 0 | 126 | – | 0 of 126 | 0.00% | – |
| floreview-motorola | 0 | 248 | – | 0 of 248 | 0.00% | – |
| floreview-oneplus | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-samsung | 0 | 377 | – | 0 of 377 | 0.00% | – |
| floreview-sony | 0 | 40 | – | 0 of 40 | 0.00% | – |
| floreview-xiaomi | 0 | 292 | – | 2 of 292 | 0.68% | – |
| genimagepp | 1200 | 0 | 97.42% | – | – | – |
| hdrplus | 0 | 1456 | – | 1 of 1456 | 0.07% | – |

Worst source slice with at least 100 items of the class: recall 92.02% (arforensics); FPR 0.68% (floreview-xiaomi).

**phase3-0.30**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.760167 | 7 of 7326 | 0.10% | 0.18% | 93.00% (92.42%–93.57%) | 90.77% |
| 1% empirical FPR | 0.443977 | 73 of 7326 | 1.00% | 1.21% | 96.08% (95.62%–96.52%) | 49.34% |
| frozen cutoff | 0.728662 | 10 of 7326 | 0.14% | 0.23% | 93.46% (92.92%–93.98%) | 87.37% |

phase3-0.30 by family, at the frozen cutoff 0.7286624:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 800 | 0 | 97.88% | – | – | – |
| infinity | 1500 | 0 | 99.47% | – | – | – |
| janus-pro | 600 | 0 | 100.00% | – | – | – |
| llamagen | 600 | 0 | 97.00% | – | – | – |
| open-magvit2 | 600 | 0 | 60.50% | – | – | – |
| rar | 1500 | 0 | 98.40% | – | – | – |
| real:csafe | 0 | 3996 | – | 7 of 3996 | 0.18% | – |
| real:floreview | 0 | 1874 | – | 3 of 1874 | 0.16% | – |
| real:hdrplus | 0 | 1456 | – | 0 of 1456 | 0.00% | – |
| sd3 | 400 | 0 | 99.00% | – | – | – |
| switti | 600 | 0 | 99.17% | – | – | – |
| var | 600 | 0 | 73.67% | – | – | – |

Worst family slice with at least 100 items of the class: recall 60.50% (open-magvit2); FPR 0.18% (real:csafe).

phase3-0.30 by source, at the frozen cutoff 0.7286624:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 92.50% | – | – | – |
| csafe-iphone14pro | 0 | 3996 | – | 7 of 3996 | 0.18% | – |
| floreview-apple | 0 | 249 | – | 0 of 249 | 0.00% | – |
| floreview-doogee | 0 | 42 | – | 1 of 42 | 2.38% | – |
| floreview-google | 0 | 125 | – | 0 of 125 | 0.00% | – |
| floreview-huawei | 0 | 249 | – | 0 of 249 | 0.00% | – |
| floreview-lenovo | 0 | 42 | – | 0 of 42 | 0.00% | – |
| floreview-lg | 0 | 126 | – | 0 of 126 | 0.00% | – |
| floreview-motorola | 0 | 248 | – | 0 of 248 | 0.00% | – |
| floreview-oneplus | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-samsung | 0 | 377 | – | 0 of 377 | 0.00% | – |
| floreview-sony | 0 | 40 | – | 0 of 40 | 0.00% | – |
| floreview-xiaomi | 0 | 292 | – | 2 of 292 | 0.68% | – |
| genimagepp | 1200 | 0 | 98.25% | – | – | – |
| hdrplus | 0 | 1456 | – | 0 of 1456 | 0.00% | – |

Worst source slice with at least 100 items of the class: recall 92.50% (arforensics); FPR 0.68% (floreview-xiaomi).
