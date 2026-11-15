19326 items scored by all 2 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**shipped-default**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.880098 | 9 of 9726 | 0.09% | 0.16% | 84.09% (82.90%–85.28%) | 90.18% |
| 1% empirical FPR | 0.342121 | 97 of 9726 | 1.00% | 1.18% | 92.39% (91.49%–93.18%) | 48.34% |
| frozen cutoff | 0.500000 | 65 of 9726 | 0.67% | 0.82% | 90.90% (89.95%–91.82%) | 57.87% |

shipped-default by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 96.05% | – | – | – |
| infinity | 1992 | 0 | 98.04% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 94.78% | – | – | – |
| open-magvit2 | 804 | 0 | 62.19% | – | – | – |
| rar | 1992 | 0 | 93.17% | – | – | – |
| real:csafe | 0 | 5304 | – | 47 of 5304 | 0.89% | – |
| real:floreview | 0 | 2486 | – | 11 of 2486 | 0.44% | – |
| real:hdrplus | 0 | 1936 | – | 7 of 1936 | 0.36% | – |
| sd3 | 532 | 0 | 96.62% | – | – | – |
| switti | 804 | 0 | 96.64% | – | – | – |
| var | 804 | 0 | 68.91% | – | – | – |

Worst family slice with at least 100 items of the class: recall 62.19% (open-magvit2); FPR 0.89% (real:csafe).

shipped-default by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 93.95% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 95.21% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 96.41% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 64.07% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 80.24% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 89.82% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 18.56% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 68.86% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 64.67% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 75.45% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 85.63% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 93.41% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 47 of 5304 | 0.89% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-google | 0 | 161 | – | 2 of 161 | 1.24% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-lg | 0 | 174 | – | 1 of 174 | 0.57% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 1 of 108 | 0.93% | – |
| floreview-samsung | 0 | 497 | – | 2 of 497 | 0.40% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 3 of 388 | 0.77% | – |
| genimagepp | 1200 | 0 | 98.83% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 18.18% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 93.94% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 7 of 1936 | 0.36% | – |

Worst source slice with at least 100 items of the class: recall 18.56% (arforensics-noise-a40); FPR 1.24% (floreview-google).

shipped-default by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 95.50% | 1 of 200 | 0.50% | 0.997 |
| blur-s3.0 | 200 | 200 | 97.00% | 1 of 200 | 0.50% | 0.999 |
| jpeg-qv12 | 200 | 200 | 68.50% | 1 of 200 | 0.50% | 0.981 |
| jpeg-qv4 | 200 | 200 | 82.00% | 1 of 200 | 0.50% | 0.987 |
| noise-a10 | 200 | 200 | 91.00% | 16 of 200 | 8.00% | 0.956 |
| noise-a40 | 200 | 200 | 18.50% | 12 of 200 | 6.00% | 0.674 |
| none | 7200 | 7326 | 94.76% | 26 of 7326 | 0.35% | 0.995 |
| repost-720-qv9x1 | 200 | 200 | 73.00% | 2 of 200 | 1.00% | 0.961 |
| repost-960-qv12x1 | 200 | 200 | 68.50% | 3 of 200 | 1.50% | 0.956 |
| repost-960-qv7x4 | 200 | 200 | 78.50% | 0 of 200 | 0.00% | 0.973 |
| resize-0.25-bicubic | 200 | 200 | 87.50% | 1 of 200 | 0.50% | 0.995 |
| resize-0.5-bicubic | 200 | 200 | 97.00% | 0 of 200 | 0.00% | 0.997 |
| sharpen-a1.0 | 200 | 200 | 94.50% | 1 of 200 | 0.50% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 18.50% (noise-a40); FPR 8.00% (noise-a10).

**shipped-all-views**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.847605 | 9 of 9726 | 0.09% | 0.16% | 83.92% (82.67%–85.16%) | 90.16% |
| 1% empirical FPR | 0.466254 | 97 of 9726 | 1.00% | 1.18% | 91.20% (90.30%–92.09%) | 48.02% |
| frozen cutoff | 0.500000 | 85 of 9726 | 0.87% | 1.05% | 90.80% (89.91%–91.71%) | 51.21% |

shipped-all-views by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 95.96% | – | – | – |
| infinity | 1992 | 0 | 97.74% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 94.40% | – | – | – |
| open-magvit2 | 804 | 0 | 62.31% | – | – | – |
| rar | 1992 | 0 | 93.17% | – | – | – |
| real:csafe | 0 | 5304 | – | 64 of 5304 | 1.21% | – |
| real:floreview | 0 | 2486 | – | 14 of 2486 | 0.56% | – |
| real:hdrplus | 0 | 1936 | – | 7 of 1936 | 0.36% | – |
| sd3 | 532 | 0 | 96.80% | – | – | – |
| switti | 804 | 0 | 96.52% | – | – | – |
| var | 804 | 0 | 68.91% | – | – | – |

Worst family slice with at least 100 items of the class: recall 62.31% (open-magvit2); FPR 1.21% (real:csafe).

shipped-all-views by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 93.98% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 95.21% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 96.41% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 63.47% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 79.04% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 89.82% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 14.37% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 68.86% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 64.67% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 74.85% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 85.63% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 93.41% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 64 of 5304 | 1.21% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-google | 0 | 161 | – | 2 of 161 | 1.24% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-lg | 0 | 174 | – | 1 of 174 | 0.57% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 0 of 108 | 0.00% | – |
| floreview-samsung | 0 | 497 | – | 3 of 497 | 0.60% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 6 of 388 | 1.55% | – |
| genimagepp | 1200 | 0 | 99.08% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 3.03% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 7 of 1936 | 0.36% | – |

Worst source slice with at least 100 items of the class: recall 14.37% (arforensics-noise-a40); FPR 1.55% (floreview-xiaomi).

shipped-all-views by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 96.00% | 1 of 200 | 0.50% | 0.997 |
| blur-s3.0 | 200 | 200 | 97.00% | 1 of 200 | 0.50% | 0.998 |
| jpeg-qv12 | 200 | 200 | 67.50% | 1 of 200 | 0.50% | 0.975 |
| jpeg-qv4 | 200 | 200 | 81.00% | 2 of 200 | 1.00% | 0.988 |
| noise-a10 | 200 | 200 | 91.00% | 22 of 200 | 11.00% | 0.948 |
| noise-a40 | 200 | 200 | 12.50% | 14 of 200 | 7.00% | 0.611 |
| none | 7200 | 7326 | 94.83% | 35 of 7326 | 0.48% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 73.50% | 2 of 200 | 1.00% | 0.961 |
| repost-960-qv12x1 | 200 | 200 | 68.50% | 3 of 200 | 1.50% | 0.947 |
| repost-960-qv7x4 | 200 | 200 | 78.00% | 0 of 200 | 0.00% | 0.976 |
| resize-0.25-bicubic | 200 | 200 | 87.50% | 1 of 200 | 0.50% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 97.50% | 0 of 200 | 0.00% | 0.998 |
| sharpen-a1.0 | 200 | 200 | 94.50% | 3 of 200 | 1.50% | 0.986 |

Worst transform slice with at least 100 items of the class: recall 12.50% (noise-a40); FPR 11.00% (noise-a10).
