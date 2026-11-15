1200 items scored by all 2 systems: 600 AI, 600 real; grouping columns found for 1200
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**alpha-0**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.740839 | 0 of 600 | 0.00% | 0.50% | 59.83% (53.54%–65.76%) | 100.00% |
| 1% empirical FPR | 0.459885 | 6 of 600 | 1.00% | 1.96% | 74.83% (69.07%–80.45%) | 43.05% |
| frozen cutoff | 0.699543 | 2 of 600 | 0.33% | 1.05% | 62.00% (55.88%–67.95%) | 65.26% |

alpha-0 by family, at the frozen cutoff 0.6995432:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 68.18% | – | – | – |
| infinity | 123 | 0 | 91.87% | – | – | – |
| janus-pro | 51 | 0 | 78.43% | – | – | – |
| llamagen | 51 | 0 | 62.75% | – | – | – |
| open-magvit2 | 51 | 0 | 17.65% | – | – | – |
| rar | 123 | 0 | 46.34% | – | – | – |
| real:csafe | 0 | 327 | – | 1 of 327 | 0.31% | – |
| real:floreview | 0 | 153 | – | 1 of 153 | 0.65% | – |
| real:hdrplus | 0 | 120 | – | 0 of 120 | 0.00% | – |
| sd3 | 33 | 0 | 84.85% | – | – | – |
| switti | 51 | 0 | 88.24% | – | – | – |
| var | 51 | 0 | 5.88% | – | – | – |

Worst family slice with at least 100 items of the class: recall 46.34% (rar); FPR 0.65% (real:floreview).

alpha-0 by source, at the frozen cutoff 0.6995432:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-repost-720-qv9x1 | 167 | 0 | 59.88% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 52.69% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 66.47% | – | – | – |
| csafe-iphone14pro | 0 | 327 | – | 1 of 327 | 0.31% | – |
| floreview-apple | 0 | 21 | – | 0 of 21 | 0.00% | – |
| floreview-doogee | 0 | 3 | – | 0 of 3 | 0.00% | – |
| floreview-google | 0 | 9 | – | 0 of 9 | 0.00% | – |
| floreview-huawei | 0 | 21 | – | 0 of 21 | 0.00% | – |
| floreview-lenovo | 0 | 3 | – | 0 of 3 | 0.00% | – |
| floreview-lg | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-motorola | 0 | 21 | – | 0 of 21 | 0.00% | – |
| floreview-oneplus | 0 | 6 | – | 0 of 6 | 0.00% | – |
| floreview-samsung | 0 | 30 | – | 1 of 30 | 3.33% | – |
| floreview-sony | 0 | 3 | – | 0 of 3 | 0.00% | – |
| floreview-xiaomi | 0 | 24 | – | 0 of 24 | 0.00% | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 66.67% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 69.70% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 84.85% | – | – | – |
| hdrplus | 0 | 120 | – | 0 of 120 | 0.00% | – |

Worst source slice with at least 100 items of the class: recall 52.69% (arforensics-repost-960-qv12x1); FPR 0.31% (csafe-iphone14pro).

**phase3-0.30**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.786248 | 0 of 600 | 0.00% | 0.50% | 60.83% (54.67%–66.51%) | 100.00% |
| 1% empirical FPR | 0.593005 | 6 of 600 | 1.00% | 1.96% | 75.00% (69.65%–80.62%) | 43.10% |
| frozen cutoff | 0.728662 | 3 of 600 | 0.50% | 1.29% | 66.83% (60.64%–72.64%) | 57.45% |

phase3-0.30 by family, at the frozen cutoff 0.7286624:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 81.82% | – | – | – |
| infinity | 123 | 0 | 92.68% | – | – | – |
| janus-pro | 51 | 0 | 90.20% | – | – | – |
| llamagen | 51 | 0 | 72.55% | – | – | – |
| open-magvit2 | 51 | 0 | 21.57% | – | – | – |
| rar | 123 | 0 | 49.59% | – | – | – |
| real:csafe | 0 | 327 | – | 2 of 327 | 0.61% | – |
| real:floreview | 0 | 153 | – | 1 of 153 | 0.65% | – |
| real:hdrplus | 0 | 120 | – | 0 of 120 | 0.00% | – |
| sd3 | 33 | 0 | 90.91% | – | – | – |
| switti | 51 | 0 | 88.24% | – | – | – |
| var | 51 | 0 | 5.88% | – | – | – |

Worst family slice with at least 100 items of the class: recall 49.59% (rar); FPR 0.65% (real:floreview).

phase3-0.30 by source, at the frozen cutoff 0.7286624:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-repost-720-qv9x1 | 167 | 0 | 62.28% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 59.28% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 68.26% | – | – | – |
| csafe-iphone14pro | 0 | 327 | – | 2 of 327 | 0.61% | – |
| floreview-apple | 0 | 21 | – | 0 of 21 | 0.00% | – |
| floreview-doogee | 0 | 3 | – | 0 of 3 | 0.00% | – |
| floreview-google | 0 | 9 | – | 0 of 9 | 0.00% | – |
| floreview-huawei | 0 | 21 | – | 0 of 21 | 0.00% | – |
| floreview-lenovo | 0 | 3 | – | 0 of 3 | 0.00% | – |
| floreview-lg | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-motorola | 0 | 21 | – | 0 of 21 | 0.00% | – |
| floreview-oneplus | 0 | 6 | – | 0 of 6 | 0.00% | – |
| floreview-samsung | 0 | 30 | – | 1 of 30 | 3.33% | – |
| floreview-sony | 0 | 3 | – | 0 of 3 | 0.00% | – |
| floreview-xiaomi | 0 | 24 | – | 0 of 24 | 0.00% | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 90.91% | – | – | – |
| hdrplus | 0 | 120 | – | 0 of 120 | 0.00% | – |

Worst source slice with at least 100 items of the class: recall 59.28% (arforensics-repost-960-qv12x1); FPR 0.61% (csafe-iphone14pro).
