19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**C-photo**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.975623 | 9 of 9726 | 0.09% | 0.16% | 70.29% (68.77%–71.77%) | 88.47% |
| 1% empirical FPR | 0.564770 | 97 of 9726 | 1.00% | 1.18% | 91.58% (90.67%–92.48%) | 48.12% |
| frozen cutoff | 0.500000 | 120 of 9726 | 1.23% | 1.43% | 92.34% (91.46%–93.16%) | 43.05% |

C-photo by source, at the frozen cutoff 0.5:

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

C-photo transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 61 | 1.53% | 1.22% | 1.89% | exceeds |
| floreview-apple | 249 | 1 | 0.40% | 0.02% | 1.89% | not resolved (299 needed with none flagged) |
| floreview-doogee | 42 | 2 | 4.76% | 0.85% | 14.24% | not resolved (299 needed with none flagged) |
| floreview-google | 125 | 1 | 0.80% | 0.04% | 3.74% | not resolved (299 needed with none flagged) |
| floreview-huawei | 249 | 1 | 0.40% | 0.02% | 1.89% | not resolved (299 needed with none flagged) |
| floreview-lenovo | 42 | 1 | 2.38% | 0.12% | 10.80% | not resolved (299 needed with none flagged) |
| floreview-lg | 126 | 2 | 1.59% | 0.28% | 4.91% | not resolved (299 needed with none flagged) |
| floreview-motorola | 248 | 1 | 0.40% | 0.02% | 1.90% | not resolved (299 needed with none flagged) |
| floreview-oneplus | 84 | 2 | 2.38% | 0.42% | 7.31% | not resolved (299 needed with none flagged) |
| floreview-samsung | 377 | 4 | 1.06% | 0.36% | 2.41% | not resolved |
| floreview-sony | 40 | 0 | 0.00% | 0.00% | 7.22% | not resolved (299 needed with none flagged) |
| floreview-xiaomi | 292 | 8 | 2.74% | 1.37% | 4.89% | exceeds |
| hdrplus | 1456 | 10 | 0.69% | 0.37% | 1.16% | not resolved |

0 of 13 authentic source strata are shown within the budget, 2 exceed it, 11 are not resolved.

C-photo by transform, at the frozen cutoff 0.5:

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

C-photo transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |
| blur-s3.0 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (299 needed with none flagged) |
| jpeg-qv12 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (299 needed with none flagged) |
| jpeg-qv4 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |
| noise-a10 | 200 | 24 | 12.00% | 8.41% | 16.46% | exceeds |
| noise-a40 | 200 | 18 | 9.00% | 5.90% | 13.05% | exceeds |
| none | 7326 | 51 | 0.70% | 0.54% | 0.88% | within |
| repost-720-qv9x1 | 200 | 7 | 3.50% | 1.65% | 6.47% | exceeds |
| repost-960-qv12x1 | 200 | 4 | 2.00% | 0.69% | 4.52% | not resolved (299 needed with none flagged) |
| repost-960-qv7x4 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (299 needed with none flagged) |
| resize-0.25-bicubic | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (299 needed with none flagged) |
| resize-0.5-bicubic | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (299 needed with none flagged) |
| sharpen-a1.0 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (299 needed with none flagged) |

1 of 13 authentic transform strata are shown within the budget, 3 exceed it, 9 are not resolved.

C-photo by family, at the frozen cutoff 0.5:

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

C-photo transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 61 | 1.53% | 1.22% | 1.89% | exceeds |
| real:floreview | 1874 | 23 | 1.23% | 0.84% | 1.73% | not resolved |
| real:hdrplus | 1456 | 10 | 0.69% | 0.37% | 1.16% | not resolved |

0 of 3 authentic family strata are shown within the budget, 1 exceed it, 2 are not resolved.
