19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**C-sensitive**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.997727 | 9 of 9726 | 0.09% | 0.16% | 67.91% (66.45%–69.43%) | 88.11% |
| 1% empirical FPR | 0.890481 | 97 of 9726 | 1.00% | 1.18% | 90.67% (89.69%–91.64%) | 47.87% |
| frozen cutoff | 0.500000 | 416 of 9726 | 4.28% | 4.63% | 95.16% (94.49%–95.73%) | 18.35% |

C-sensitive by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 96.77% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 100.00% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 83.23% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 89.22% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 95.21% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 34.13% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 89.22% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 83.23% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 89.82% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 95.81% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 98.80% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 97.01% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 252 of 5304 | 4.75% | – |
| floreview-apple | 0 | 333 | – | 5 of 333 | 1.50% | – |
| floreview-doogee | 0 | 54 | – | 16 of 54 | 29.63% | – |
| floreview-google | 0 | 161 | – | 9 of 161 | 5.59% | – |
| floreview-huawei | 0 | 333 | – | 12 of 333 | 3.60% | – |
| floreview-lenovo | 0 | 54 | – | 4 of 54 | 7.41% | – |
| floreview-lg | 0 | 174 | – | 6 of 174 | 3.45% | – |
| floreview-motorola | 0 | 332 | – | 5 of 332 | 1.51% | – |
| floreview-oneplus | 0 | 108 | – | 8 of 108 | 7.41% | – |
| floreview-samsung | 0 | 497 | – | 27 of 497 | 5.43% | – |
| floreview-sony | 0 | 52 | – | 1 of 52 | 1.92% | – |
| floreview-xiaomi | 0 | 388 | – | 18 of 388 | 4.64% | – |
| genimagepp | 1200 | 0 | 99.83% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 33.33% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 53 of 1936 | 2.74% | – |

Worst source slice with at least 100 items of the class: recall 34.13% (arforensics-noise-a40); FPR 7.41% (floreview-oneplus).

C-sensitive transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 168 | 4.20% | 3.69% | 4.76% | within |
| floreview-apple | 249 | 4 | 1.61% | 0.55% | 3.64% | within |
| floreview-doogee | 42 | 15 | 35.71% | 23.47% | 49.55% | exceeds |
| floreview-google | 125 | 6 | 4.80% | 2.11% | 9.25% | not resolved |
| floreview-huawei | 249 | 12 | 4.82% | 2.80% | 7.69% | not resolved |
| floreview-lenovo | 42 | 2 | 4.76% | 0.85% | 14.24% | not resolved (59 needed with none flagged) |
| floreview-lg | 126 | 5 | 3.97% | 1.58% | 8.16% | not resolved |
| floreview-motorola | 248 | 4 | 1.61% | 0.55% | 3.65% | within |
| floreview-oneplus | 84 | 3 | 3.57% | 0.98% | 8.97% | not resolved |
| floreview-samsung | 377 | 19 | 5.04% | 3.33% | 7.31% | not resolved |
| floreview-sony | 40 | 1 | 2.50% | 0.13% | 11.32% | not resolved (59 needed with none flagged) |
| floreview-xiaomi | 292 | 16 | 5.48% | 3.47% | 8.20% | not resolved |
| hdrplus | 1456 | 41 | 2.82% | 2.14% | 3.64% | within |

4 of 13 authentic source strata are shown within the budget, 1 exceed it, 8 are not resolved.

C-sensitive by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 98.00% | 7 of 200 | 3.50% | 0.997 |
| blur-s3.0 | 200 | 200 | 100.00% | 29 of 200 | 14.50% | 0.996 |
| jpeg-qv12 | 200 | 200 | 86.00% | 4 of 200 | 2.00% | 0.979 |
| jpeg-qv4 | 200 | 200 | 91.00% | 3 of 200 | 1.50% | 0.989 |
| noise-a10 | 200 | 200 | 96.00% | 72 of 200 | 36.00% | 0.949 |
| noise-a40 | 200 | 200 | 34.00% | 37 of 200 | 18.50% | 0.661 |
| none | 7200 | 7326 | 97.28% | 196 of 7326 | 2.68% | 0.995 |
| repost-720-qv9x1 | 200 | 200 | 91.00% | 16 of 200 | 8.00% | 0.962 |
| repost-960-qv12x1 | 200 | 200 | 85.50% | 14 of 200 | 7.00% | 0.953 |
| repost-960-qv7x4 | 200 | 200 | 91.00% | 11 of 200 | 5.50% | 0.973 |
| resize-0.25-bicubic | 200 | 200 | 96.50% | 4 of 200 | 2.00% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 99.00% | 12 of 200 | 6.00% | 0.998 |
| sharpen-a1.0 | 200 | 200 | 97.50% | 11 of 200 | 5.50% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 34.00% (noise-a40); FPR 36.00% (noise-a10).

C-sensitive transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 7 | 3.50% | 1.65% | 6.47% | not resolved |
| blur-s3.0 | 200 | 29 | 14.50% | 10.57% | 19.24% | exceeds |
| jpeg-qv12 | 200 | 4 | 2.00% | 0.69% | 4.52% | within |
| jpeg-qv4 | 200 | 3 | 1.50% | 0.41% | 3.83% | within |
| noise-a10 | 200 | 72 | 36.00% | 30.35% | 41.96% | exceeds |
| noise-a40 | 200 | 37 | 18.50% | 14.11% | 23.61% | exceeds |
| none | 7326 | 196 | 2.68% | 2.37% | 3.01% | within |
| repost-720-qv9x1 | 200 | 16 | 8.00% | 5.08% | 11.90% | exceeds |
| repost-960-qv12x1 | 200 | 14 | 7.00% | 4.28% | 10.73% | not resolved |
| repost-960-qv7x4 | 200 | 11 | 5.50% | 3.11% | 8.94% | not resolved |
| resize-0.25-bicubic | 200 | 4 | 2.00% | 0.69% | 4.52% | within |
| resize-0.5-bicubic | 200 | 12 | 6.00% | 3.50% | 9.54% | not resolved |
| sharpen-a1.0 | 200 | 11 | 5.50% | 3.11% | 8.94% | not resolved |

4 of 13 authentic transform strata are shown within the budget, 4 exceed it, 5 are not resolved.

C-sensitive by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 98.40% | – | – | – |
| infinity | 1992 | 0 | 99.10% | – | – | – |
| janus-pro | 804 | 0 | 98.38% | – | – | – |
| llamagen | 804 | 0 | 98.51% | – | – | – |
| open-magvit2 | 804 | 0 | 77.24% | – | – | – |
| rar | 1992 | 0 | 96.54% | – | – | – |
| real:csafe | 0 | 5304 | – | 252 of 5304 | 4.75% | – |
| real:floreview | 0 | 2486 | – | 111 of 2486 | 4.47% | – |
| real:hdrplus | 0 | 1936 | – | 53 of 1936 | 2.74% | – |
| sd3 | 532 | 0 | 98.31% | – | – | – |
| switti | 804 | 0 | 97.51% | – | – | – |
| var | 804 | 0 | 84.58% | – | – | – |

Worst family slice with at least 100 items of the class: recall 77.24% (open-magvit2); FPR 4.75% (real:csafe).

C-sensitive transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 168 | 4.20% | 3.69% | 4.76% | within |
| real:floreview | 1874 | 87 | 4.64% | 3.87% | 5.52% | not resolved |
| real:hdrplus | 1456 | 41 | 2.82% | 2.14% | 3.64% | within |

2 of 3 authentic family strata are shown within the budget, 0 exceed it, 1 are not resolved.
