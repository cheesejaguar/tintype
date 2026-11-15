19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**B-sensitive**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.998980 | 9 of 9726 | 0.09% | 0.16% | 67.56% (66.10%–69.10%) | 88.06% |
| 1% empirical FPR | 0.961485 | 97 of 9726 | 1.00% | 1.18% | 88.73% (87.66%–89.76%) | 47.33% |
| frozen cutoff | 0.500000 | 591 of 9726 | 6.08% | 6.49% | 96.03% (95.43%–96.56%) | 13.77% |

B-sensitive by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 97.32% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 98.20% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 100.00% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 88.02% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 91.62% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 95.81% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 40.72% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 91.02% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 88.02% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 92.81% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 98.80% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 97.01% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 339 of 5304 | 6.39% | – |
| floreview-apple | 0 | 333 | – | 6 of 333 | 1.80% | – |
| floreview-doogee | 0 | 54 | – | 22 of 54 | 40.74% | – |
| floreview-google | 0 | 161 | – | 12 of 161 | 7.45% | – |
| floreview-huawei | 0 | 333 | – | 23 of 333 | 6.91% | – |
| floreview-lenovo | 0 | 54 | – | 5 of 54 | 9.26% | – |
| floreview-lg | 0 | 174 | – | 13 of 174 | 7.47% | – |
| floreview-motorola | 0 | 332 | – | 7 of 332 | 2.11% | – |
| floreview-oneplus | 0 | 108 | – | 11 of 108 | 10.19% | – |
| floreview-samsung | 0 | 497 | – | 42 of 497 | 8.45% | – |
| floreview-sony | 0 | 52 | – | 2 of 52 | 3.85% | – |
| floreview-xiaomi | 0 | 388 | – | 29 of 388 | 7.47% | – |
| genimagepp | 1200 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 42.42% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 80 of 1936 | 4.13% | – |

Worst source slice with at least 100 items of the class: recall 40.72% (arforensics-noise-a40); FPR 10.19% (floreview-oneplus).

B-sensitive transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 223 | 5.58% | 5.00% | 6.21% | not resolved |
| floreview-apple | 249 | 4 | 1.61% | 0.55% | 3.64% | within |
| floreview-doogee | 42 | 18 | 42.86% | 29.83% | 56.67% | exceeds |
| floreview-google | 125 | 7 | 5.60% | 2.66% | 10.26% | not resolved |
| floreview-huawei | 249 | 20 | 8.03% | 5.39% | 11.46% | exceeds |
| floreview-lenovo | 42 | 3 | 7.14% | 1.98% | 17.44% | not resolved (59 needed with none flagged) |
| floreview-lg | 126 | 10 | 7.94% | 4.37% | 13.09% | not resolved |
| floreview-motorola | 248 | 6 | 2.42% | 1.06% | 4.72% | within |
| floreview-oneplus | 84 | 5 | 5.95% | 2.37% | 12.11% | not resolved |
| floreview-samsung | 377 | 26 | 6.90% | 4.88% | 9.44% | not resolved |
| floreview-sony | 40 | 2 | 5.00% | 0.90% | 14.92% | not resolved (59 needed with none flagged) |
| floreview-xiaomi | 292 | 23 | 7.88% | 5.44% | 10.97% | exceeds |
| hdrplus | 1456 | 57 | 3.91% | 3.11% | 4.86% | within |

3 of 13 authentic source strata are shown within the budget, 3 exceed it, 7 are not resolved.

B-sensitive by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 98.50% | 10 of 200 | 5.00% | 0.997 |
| blur-s3.0 | 200 | 200 | 100.00% | 45 of 200 | 22.50% | 0.996 |
| jpeg-qv12 | 200 | 200 | 90.00% | 5 of 200 | 2.50% | 0.980 |
| jpeg-qv4 | 200 | 200 | 93.00% | 6 of 200 | 3.00% | 0.989 |
| noise-a10 | 200 | 200 | 96.50% | 90 of 200 | 45.00% | 0.946 |
| noise-a40 | 200 | 200 | 41.00% | 44 of 200 | 22.00% | 0.664 |
| none | 7200 | 7326 | 97.76% | 288 of 7326 | 3.93% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 92.50% | 26 of 200 | 13.00% | 0.961 |
| repost-960-qv12x1 | 200 | 200 | 90.00% | 22 of 200 | 11.00% | 0.953 |
| repost-960-qv7x4 | 200 | 200 | 94.00% | 15 of 200 | 7.50% | 0.973 |
| resize-0.25-bicubic | 200 | 200 | 98.00% | 10 of 200 | 5.00% | 0.994 |
| resize-0.5-bicubic | 200 | 200 | 99.00% | 15 of 200 | 7.50% | 0.998 |
| sharpen-a1.0 | 200 | 200 | 97.50% | 15 of 200 | 7.50% | 0.990 |

Worst transform slice with at least 100 items of the class: recall 41.00% (noise-a40); FPR 45.00% (noise-a10).

B-sensitive transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 10 | 5.00% | 2.74% | 8.33% | not resolved |
| blur-s3.0 | 200 | 45 | 22.50% | 17.72% | 27.90% | exceeds |
| jpeg-qv12 | 200 | 5 | 2.50% | 0.99% | 5.18% | not resolved |
| jpeg-qv4 | 200 | 6 | 3.00% | 1.31% | 5.83% | not resolved |
| noise-a10 | 200 | 90 | 45.00% | 39.05% | 51.06% | exceeds |
| noise-a40 | 200 | 44 | 22.00% | 17.27% | 27.37% | exceeds |
| none | 7326 | 288 | 3.93% | 3.56% | 4.33% | within |
| repost-720-qv9x1 | 200 | 26 | 13.00% | 9.27% | 17.58% | exceeds |
| repost-960-qv12x1 | 200 | 22 | 11.00% | 7.57% | 15.33% | exceeds |
| repost-960-qv7x4 | 200 | 15 | 7.50% | 4.68% | 11.31% | not resolved |
| resize-0.25-bicubic | 200 | 10 | 5.00% | 2.74% | 8.33% | not resolved |
| resize-0.5-bicubic | 200 | 15 | 7.50% | 4.68% | 11.31% | not resolved |
| sharpen-a1.0 | 200 | 15 | 7.50% | 4.68% | 11.31% | not resolved |

1 of 13 authentic transform strata are shown within the budget, 5 exceed it, 7 are not resolved.

B-sensitive by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 98.59% | – | – | – |
| infinity | 1992 | 0 | 99.30% | – | – | – |
| janus-pro | 804 | 0 | 98.63% | – | – | – |
| llamagen | 804 | 0 | 98.51% | – | – | – |
| open-magvit2 | 804 | 0 | 80.47% | – | – | – |
| rar | 1992 | 0 | 97.34% | – | – | – |
| real:csafe | 0 | 5304 | – | 339 of 5304 | 6.39% | – |
| real:floreview | 0 | 2486 | – | 172 of 2486 | 6.92% | – |
| real:hdrplus | 0 | 1936 | – | 80 of 1936 | 4.13% | – |
| sd3 | 532 | 0 | 99.25% | – | – | – |
| switti | 804 | 0 | 97.76% | – | – | – |
| var | 804 | 0 | 87.94% | – | – | – |

Worst family slice with at least 100 items of the class: recall 80.47% (open-magvit2); FPR 6.92% (real:floreview).

B-sensitive transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 223 | 5.58% | 5.00% | 6.21% | not resolved |
| real:floreview | 1874 | 124 | 6.62% | 5.70% | 7.64% | exceeds |
| real:hdrplus | 1456 | 57 | 3.91% | 3.11% | 4.86% | within |

1 of 3 authentic family strata are shown within the budget, 1 exceed it, 1 are not resolved.
