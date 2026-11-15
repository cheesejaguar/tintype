19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-sensitive**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.996458 | 9 of 9726 | 0.09% | 0.16% | 68.60% (67.11%–70.12%) | 88.22% |
| 1% empirical FPR | 0.828097 | 97 of 9726 | 1.00% | 1.18% | 91.18% (90.23%–92.07%) | 48.01% |
| frozen cutoff | 0.500000 | 368 of 9726 | 3.78% | 4.12% | 94.73% (94.04%–95.34%) | 20.18% |

A-sensitive by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 96.45% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 100.00% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 81.44% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 88.62% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 94.01% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 31.74% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 86.23% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 81.44% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 88.62% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 95.81% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 98.80% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 97.01% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 228 of 5304 | 4.30% | – |
| floreview-apple | 0 | 333 | – | 3 of 333 | 0.90% | – |
| floreview-doogee | 0 | 54 | – | 15 of 54 | 27.78% | – |
| floreview-google | 0 | 161 | – | 8 of 161 | 4.97% | – |
| floreview-huawei | 0 | 333 | – | 10 of 333 | 3.00% | – |
| floreview-lenovo | 0 | 54 | – | 4 of 54 | 7.41% | – |
| floreview-lg | 0 | 174 | – | 4 of 174 | 2.30% | – |
| floreview-motorola | 0 | 332 | – | 5 of 332 | 1.51% | – |
| floreview-oneplus | 0 | 108 | – | 6 of 108 | 5.56% | – |
| floreview-samsung | 0 | 497 | – | 18 of 497 | 3.62% | – |
| floreview-sony | 0 | 52 | – | 1 of 52 | 1.92% | – |
| floreview-xiaomi | 0 | 388 | – | 18 of 388 | 4.64% | – |
| genimagepp | 1200 | 0 | 99.83% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 27.27% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 48 of 1936 | 2.48% | – |

Worst source slice with at least 100 items of the class: recall 31.74% (arforensics-noise-a40); FPR 5.56% (floreview-oneplus).

A-sensitive transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 158 | 3.95% | 3.46% | 4.50% | within |
| floreview-apple | 249 | 3 | 1.20% | 0.33% | 3.08% | within |
| floreview-doogee | 42 | 14 | 33.33% | 21.41% | 47.11% | exceeds |
| floreview-google | 125 | 5 | 4.00% | 1.59% | 8.23% | not resolved |
| floreview-huawei | 249 | 10 | 4.02% | 2.19% | 6.72% | not resolved |
| floreview-lenovo | 42 | 2 | 4.76% | 0.85% | 14.24% | not resolved (59 needed with none flagged) |
| floreview-lg | 126 | 4 | 3.17% | 1.09% | 7.12% | not resolved |
| floreview-motorola | 248 | 5 | 2.02% | 0.80% | 4.19% | within |
| floreview-oneplus | 84 | 2 | 2.38% | 0.42% | 7.31% | not resolved |
| floreview-samsung | 377 | 13 | 3.45% | 2.05% | 5.43% | not resolved |
| floreview-sony | 40 | 1 | 2.50% | 0.13% | 11.32% | not resolved (59 needed with none flagged) |
| floreview-xiaomi | 292 | 15 | 5.14% | 3.19% | 7.80% | not resolved |
| hdrplus | 1456 | 37 | 2.54% | 1.90% | 3.33% | within |

4 of 13 authentic source strata are shown within the budget, 1 exceed it, 8 are not resolved.

A-sensitive by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 98.00% | 6 of 200 | 3.00% | 0.998 |
| blur-s3.0 | 200 | 200 | 100.00% | 27 of 200 | 13.50% | 0.997 |
| jpeg-qv12 | 200 | 200 | 84.50% | 3 of 200 | 1.50% | 0.979 |
| jpeg-qv4 | 200 | 200 | 90.50% | 4 of 200 | 2.00% | 0.990 |
| noise-a10 | 200 | 200 | 95.00% | 62 of 200 | 31.00% | 0.951 |
| noise-a40 | 200 | 200 | 31.00% | 36 of 200 | 18.00% | 0.654 |
| none | 7200 | 7326 | 97.01% | 175 of 7326 | 2.39% | 0.995 |
| repost-720-qv9x1 | 200 | 200 | 88.50% | 14 of 200 | 7.00% | 0.965 |
| repost-960-qv12x1 | 200 | 200 | 84.00% | 10 of 200 | 5.00% | 0.952 |
| repost-960-qv7x4 | 200 | 200 | 90.00% | 7 of 200 | 3.50% | 0.976 |
| resize-0.25-bicubic | 200 | 200 | 96.50% | 3 of 200 | 1.50% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 99.00% | 10 of 200 | 5.00% | 0.998 |
| sharpen-a1.0 | 200 | 200 | 97.50% | 11 of 200 | 5.50% | 0.990 |

Worst transform slice with at least 100 items of the class: recall 31.00% (noise-a40); FPR 31.00% (noise-a10).

A-sensitive transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 6 | 3.00% | 1.31% | 5.83% | not resolved |
| blur-s3.0 | 200 | 27 | 13.50% | 9.70% | 18.13% | exceeds |
| jpeg-qv12 | 200 | 3 | 1.50% | 0.41% | 3.83% | within |
| jpeg-qv4 | 200 | 4 | 2.00% | 0.69% | 4.52% | within |
| noise-a10 | 200 | 62 | 31.00% | 25.61% | 36.82% | exceeds |
| noise-a40 | 200 | 36 | 18.00% | 13.66% | 23.07% | exceeds |
| none | 7326 | 175 | 2.39% | 2.10% | 2.70% | within |
| repost-720-qv9x1 | 200 | 14 | 7.00% | 4.28% | 10.73% | not resolved |
| repost-960-qv12x1 | 200 | 10 | 5.00% | 2.74% | 8.33% | not resolved |
| repost-960-qv7x4 | 200 | 7 | 3.50% | 1.65% | 6.47% | not resolved |
| resize-0.25-bicubic | 200 | 3 | 1.50% | 0.41% | 3.83% | within |
| resize-0.5-bicubic | 200 | 10 | 5.00% | 2.74% | 8.33% | not resolved |
| sharpen-a1.0 | 200 | 11 | 5.50% | 3.11% | 8.94% | not resolved |

4 of 13 authentic transform strata are shown within the budget, 3 exceed it, 6 are not resolved.

A-sensitive by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 98.31% | – | – | – |
| infinity | 1992 | 0 | 98.95% | – | – | – |
| janus-pro | 804 | 0 | 98.13% | – | – | – |
| llamagen | 804 | 0 | 98.13% | – | – | – |
| open-magvit2 | 804 | 0 | 75.50% | – | – | – |
| rar | 1992 | 0 | 96.18% | – | – | – |
| real:csafe | 0 | 5304 | – | 228 of 5304 | 4.30% | – |
| real:floreview | 0 | 2486 | – | 92 of 2486 | 3.70% | – |
| real:hdrplus | 0 | 1936 | – | 48 of 1936 | 2.48% | – |
| sd3 | 532 | 0 | 98.12% | – | – | – |
| switti | 804 | 0 | 97.51% | – | – | – |
| var | 804 | 0 | 83.33% | – | – | – |

Worst family slice with at least 100 items of the class: recall 75.50% (open-magvit2); FPR 4.30% (real:csafe).

A-sensitive transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 158 | 3.95% | 3.46% | 4.50% | within |
| real:floreview | 1874 | 74 | 3.95% | 3.24% | 4.77% | within |
| real:hdrplus | 1456 | 37 | 2.54% | 1.90% | 3.33% | within |

3 of 3 authentic family strata are shown within the budget, 0 exceed it, 0 are not resolved.
