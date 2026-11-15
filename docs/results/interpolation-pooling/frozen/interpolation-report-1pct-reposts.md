1200 items scored by all 2 systems: 600 AI, 600 real; grouping columns found for 1200
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**alpha-0**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.740839 | 0 of 600 | 0.00% | 0.50% | 59.83% (53.54%–65.76%) | 100.00% |
| 1% empirical FPR | 0.459885 | 6 of 600 | 1.00% | 1.96% | 74.83% (69.07%–80.45%) | 43.05% |
| frozen cutoff | 0.384758 | 11 of 600 | 1.83% | 3.02% | 77.50% (72.22%–82.87%) | 29.92% |

alpha-0 by family, at the frozen cutoff 0.3847578:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 98.48% | – | – | – |
| infinity | 123 | 0 | 99.19% | – | – | – |
| janus-pro | 51 | 0 | 100.00% | – | – | – |
| llamagen | 51 | 0 | 92.16% | – | – | – |
| open-magvit2 | 51 | 0 | 37.25% | – | – | – |
| rar | 123 | 0 | 61.79% | – | – | – |
| real:csafe | 0 | 327 | – | 9 of 327 | 2.75% | – |
| real:floreview | 0 | 153 | – | 1 of 153 | 0.65% | – |
| real:hdrplus | 0 | 120 | – | 1 of 120 | 0.83% | – |
| sd3 | 33 | 0 | 90.91% | – | – | – |
| switti | 51 | 0 | 94.12% | – | – | – |
| var | 51 | 0 | 13.73% | – | – | – |

Worst family slice with at least 100 items of the class: recall 61.79% (rar); FPR 2.75% (real:csafe).

alpha-0 by source, at the frozen cutoff 0.3847578:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-repost-720-qv9x1 | 167 | 0 | 73.65% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 68.26% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 79.64% | – | – | – |
| csafe-iphone14pro | 0 | 327 | – | 9 of 327 | 2.75% | – |
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
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| hdrplus | 0 | 120 | – | 1 of 120 | 0.83% | – |

Worst source slice with at least 100 items of the class: recall 68.26% (arforensics-repost-960-qv12x1); FPR 2.75% (csafe-iphone14pro).

**phase3-0.30**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.786248 | 0 of 600 | 0.00% | 0.50% | 60.83% (54.67%–66.51%) | 100.00% |
| 1% empirical FPR | 0.593005 | 6 of 600 | 1.00% | 1.96% | 75.00% (69.65%–80.62%) | 43.10% |
| frozen cutoff | 0.500000 | 11 of 600 | 1.83% | 3.02% | 79.33% (74.52%–84.43%) | 30.42% |

phase3-0.30 by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 66 | 0 | 100.00% | – | – | – |
| infinity | 123 | 0 | 99.19% | – | – | – |
| janus-pro | 51 | 0 | 100.00% | – | – | – |
| llamagen | 51 | 0 | 92.16% | – | – | – |
| open-magvit2 | 51 | 0 | 37.25% | – | – | – |
| rar | 123 | 0 | 65.85% | – | – | – |
| real:csafe | 0 | 327 | – | 9 of 327 | 2.75% | – |
| real:floreview | 0 | 153 | – | 1 of 153 | 0.65% | – |
| real:hdrplus | 0 | 120 | – | 1 of 120 | 0.83% | – |
| sd3 | 33 | 0 | 90.91% | – | – | – |
| switti | 51 | 0 | 94.12% | – | – | – |
| var | 51 | 0 | 23.53% | – | – | – |

Worst family slice with at least 100 items of the class: recall 65.85% (rar); FPR 2.75% (real:csafe).

phase3-0.30 by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-repost-720-qv9x1 | 167 | 0 | 74.85% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 70.66% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 82.04% | – | – | – |
| csafe-iphone14pro | 0 | 327 | – | 9 of 327 | 2.75% | – |
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
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| hdrplus | 0 | 120 | – | 1 of 120 | 0.83% | – |

Worst source slice with at least 100 items of the class: recall 70.66% (arforensics-repost-960-qv12x1); FPR 2.75% (csafe-iphone14pro).
