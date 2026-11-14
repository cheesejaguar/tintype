19326 items scored by all 2 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**recal-1pct**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.975623 | 9 of 9726 | 0.09% | 0.16% | 70.29% (68.77%–71.77%) | 88.47% |
| 1% empirical FPR | 0.564770 | 97 of 9726 | 1.00% | 1.18% | 91.58% (90.67%–92.48%) | 48.12% |
| frozen cutoff | 0.500000 | 120 of 9726 | 1.23% | 1.43% | 92.34% (91.46%–93.16%) | 43.05% |

recal-1pct by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 97.46% | – | – | – |
| infinity | 1992 | 0 | 98.64% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 96.89% | – | – | – |
| open-magvit2 | 804 | 0 | 66.29% | – | – | – |
| rar | 1992 | 0 | 93.98% | – | – | – |
| real:csafe | 0 | 5304 | – | 83 of 5304 | 1.56% | – |
| real:floreview | 0 | 2486 | – | 27 of 2486 | 1.09% | – |
| real:hdrplus | 0 | 1936 | – | 10 of 1936 | 0.52% | – |
| sd3 | 532 | 0 | 96.99% | – | – | – |
| switti | 804 | 0 | 97.14% | – | – | – |
| var | 804 | 0 | 73.88% | – | – | – |

Worst family slice with at least 100 items of the class: recall 66.29% (open-magvit2); FPR 1.56% (real:csafe).

recal-1pct by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 95.00% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 95.81% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 69.46% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 82.63% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 91.62% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 23.35% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 73.65% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 68.86% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 80.24% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 89.82% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 94.61% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 83 of 5304 | 1.56% | – |
| floreview-apple | 0 | 333 | – | 1 of 333 | 0.30% | – |
| floreview-doogee | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-google | 0 | 161 | – | 3 of 161 | 1.86% | – |
| floreview-huawei | 0 | 333 | – | 1 of 333 | 0.30% | – |
| floreview-lenovo | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-lg | 0 | 174 | – | 2 of 174 | 1.15% | – |
| floreview-motorola | 0 | 332 | – | 1 of 332 | 0.30% | – |
| floreview-oneplus | 0 | 108 | – | 2 of 108 | 1.85% | – |
| floreview-samsung | 0 | 497 | – | 5 of 497 | 1.01% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 8 of 388 | 2.06% | – |
| genimagepp | 1200 | 0 | 99.25% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 21.21% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 10 of 1936 | 0.52% | – |

Worst source slice with at least 100 items of the class: recall 23.35% (arforensics-noise-a40); FPR 2.06% (floreview-xiaomi).

recal-1pct by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 96.50% | 2 of 200 | 1.00% | 0.998 |
| blur-s3.0 | 200 | 200 | 98.00% | 3 of 200 | 1.50% | 0.998 |
| jpeg-qv12 | 200 | 200 | 73.50% | 1 of 200 | 0.50% | 0.981 |
| jpeg-qv4 | 200 | 200 | 85.50% | 2 of 200 | 1.00% | 0.989 |
| noise-a10 | 200 | 200 | 92.50% | 24 of 200 | 12.00% | 0.955 |
| noise-a40 | 200 | 200 | 23.00% | 18 of 200 | 9.00% | 0.672 |
| none | 7200 | 7326 | 95.71% | 51 of 7326 | 0.70% | 0.995 |
| repost-720-qv9x1 | 200 | 200 | 77.50% | 7 of 200 | 3.50% | 0.963 |
| repost-960-qv12x1 | 200 | 200 | 73.50% | 4 of 200 | 2.00% | 0.953 |
| repost-960-qv7x4 | 200 | 200 | 83.00% | 3 of 200 | 1.50% | 0.973 |
| resize-0.25-bicubic | 200 | 200 | 91.00% | 1 of 200 | 0.50% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 97.50% | 1 of 200 | 0.50% | 0.997 |
| sharpen-a1.0 | 200 | 200 | 95.50% | 3 of 200 | 1.50% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 23.00% (noise-a40); FPR 12.00% (noise-a10).

**recal-0.1pct**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.631828 | 9 of 9726 | 0.09% | 0.16% | 85.00% (83.80%–86.20%) | 90.27% |
| 1% empirical FPR | 0.268534 | 97 of 9726 | 1.00% | 1.18% | 92.50% (91.63%–93.26%) | 48.37% |
| frozen cutoff | 0.500000 | 18 of 9726 | 0.19% | 0.27% | 87.70% (86.61%–88.83%) | 82.72% |

recal-0.1pct by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 90.70% | – | – | – |
| infinity | 1992 | 0 | 96.64% | – | – | – |
| janus-pro | 804 | 0 | 95.77% | – | – | – |
| llamagen | 804 | 0 | 92.04% | – | – | – |
| open-magvit2 | 804 | 0 | 54.73% | – | – | – |
| rar | 1992 | 0 | 90.91% | – | – | – |
| real:csafe | 0 | 5304 | – | 13 of 5304 | 0.25% | – |
| real:floreview | 0 | 2486 | – | 4 of 2486 | 0.16% | – |
| real:hdrplus | 0 | 1936 | – | 1 of 1936 | 0.05% | – |
| sd3 | 532 | 0 | 95.30% | – | – | – |
| switti | 804 | 0 | 95.27% | – | – | – |
| var | 804 | 0 | 61.57% | – | – | – |

Worst family slice with at least 100 items of the class: recall 54.73% (open-magvit2); FPR 0.25% (real:csafe).

recal-0.1pct by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 92.00% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 93.41% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 92.22% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 53.89% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 70.66% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 85.03% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 8.98% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 59.88% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 54.49% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 66.47% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 80.84% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.01% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 91.62% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 13 of 5304 | 0.25% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 0 of 54 | 0.00% | – |
| floreview-google | 0 | 161 | – | 0 of 161 | 0.00% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-lg | 0 | 174 | – | 0 of 174 | 0.00% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 1 of 108 | 0.93% | – |
| floreview-samsung | 0 | 497 | – | 1 of 497 | 0.20% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 1 of 388 | 0.26% | – |
| genimagepp | 1200 | 0 | 96.33% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 66.67% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 78.79% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 9.09% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 72.73% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 75.76% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 93.94% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 84.85% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 96.97% | – | – | – |
| hdrplus | 0 | 1936 | – | 1 of 1936 | 0.05% | – |

Worst source slice with at least 100 items of the class: recall 8.98% (arforensics-noise-a40); FPR 0.93% (floreview-oneplus).

recal-0.1pct by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 93.50% | 0 of 200 | 0.00% | 0.997 |
| blur-s3.0 | 200 | 200 | 93.50% | 0 of 200 | 0.00% | 0.999 |
| jpeg-qv12 | 200 | 200 | 56.00% | 0 of 200 | 0.00% | 0.975 |
| jpeg-qv4 | 200 | 200 | 72.00% | 0 of 200 | 0.00% | 0.987 |
| noise-a10 | 200 | 200 | 86.50% | 7 of 200 | 3.50% | 0.960 |
| noise-a40 | 200 | 200 | 9.00% | 4 of 200 | 2.00% | 0.655 |
| none | 7200 | 7326 | 92.72% | 5 of 7326 | 0.07% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 62.00% | 0 of 200 | 0.00% | 0.957 |
| repost-960-qv12x1 | 200 | 200 | 58.00% | 2 of 200 | 1.00% | 0.949 |
| repost-960-qv7x4 | 200 | 200 | 70.50% | 0 of 200 | 0.00% | 0.970 |
| resize-0.25-bicubic | 200 | 200 | 83.00% | 0 of 200 | 0.00% | 0.995 |
| resize-0.5-bicubic | 200 | 200 | 95.00% | 0 of 200 | 0.00% | 0.995 |
| sharpen-a1.0 | 200 | 200 | 92.50% | 0 of 200 | 0.00% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 9.00% (noise-a40); FPR 3.50% (noise-a10).
