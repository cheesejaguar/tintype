19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**B-photo**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.986946 | 9 of 9726 | 0.09% | 0.16% | 69.19% (67.66%–70.72%) | 88.31% |
| 1% empirical FPR | 0.702153 | 97 of 9726 | 1.00% | 1.18% | 91.27% (90.30%–92.20%) | 48.04% |
| frozen cutoff | 0.500000 | 185 of 9726 | 1.90% | 2.15% | 93.26% (92.43%–94.01%) | 33.12% |

B-photo by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 95.43% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 74.25% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 84.43% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 92.22% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 28.14% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 77.84% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 74.25% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 83.23% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 92.81% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 95.81% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 124 of 5304 | 2.34% | – |
| floreview-apple | 0 | 333 | – | 1 of 333 | 0.30% | – |
| floreview-doogee | 0 | 54 | – | 6 of 54 | 11.11% | – |
| floreview-google | 0 | 161 | – | 5 of 161 | 3.11% | – |
| floreview-huawei | 0 | 333 | – | 2 of 333 | 0.60% | – |
| floreview-lenovo | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-lg | 0 | 174 | – | 2 of 174 | 1.15% | – |
| floreview-motorola | 0 | 332 | – | 2 of 332 | 0.60% | – |
| floreview-oneplus | 0 | 108 | – | 3 of 108 | 2.78% | – |
| floreview-samsung | 0 | 497 | – | 9 of 497 | 1.81% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 9 of 388 | 2.32% | – |
| genimagepp | 1200 | 0 | 99.67% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 27.27% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 20 of 1936 | 1.03% | – |

Worst source slice with at least 100 items of the class: recall 28.14% (arforensics-noise-a40); FPR 3.11% (floreview-google).

B-photo transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 87 | 2.18% | 1.81% | 2.60% | exceeds |
| floreview-apple | 249 | 1 | 0.40% | 0.02% | 1.89% | not resolved (299 needed with none flagged) |
| floreview-doogee | 42 | 5 | 11.90% | 4.81% | 23.42% | exceeds |
| floreview-google | 125 | 3 | 2.40% | 0.66% | 6.09% | not resolved (299 needed with none flagged) |
| floreview-huawei | 249 | 2 | 0.80% | 0.14% | 2.51% | not resolved (299 needed with none flagged) |
| floreview-lenovo | 42 | 1 | 2.38% | 0.12% | 10.80% | not resolved (299 needed with none flagged) |
| floreview-lg | 126 | 2 | 1.59% | 0.28% | 4.91% | not resolved (299 needed with none flagged) |
| floreview-motorola | 248 | 2 | 0.81% | 0.14% | 2.52% | not resolved (299 needed with none flagged) |
| floreview-oneplus | 84 | 2 | 2.38% | 0.42% | 7.31% | not resolved (299 needed with none flagged) |
| floreview-samsung | 377 | 6 | 1.59% | 0.70% | 3.12% | not resolved |
| floreview-sony | 40 | 0 | 0.00% | 0.00% | 7.22% | not resolved (299 needed with none flagged) |
| floreview-xiaomi | 292 | 9 | 3.08% | 1.62% | 5.32% | exceeds |
| hdrplus | 1456 | 19 | 1.30% | 0.86% | 1.91% | not resolved |

0 of 13 authentic source strata are shown within the budget, 3 exceed it, 10 are not resolved.

B-photo by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 98.00% | 3 of 200 | 1.50% | 0.998 |
| blur-s3.0 | 200 | 200 | 98.00% | 13 of 200 | 6.50% | 0.997 |
| jpeg-qv12 | 200 | 200 | 78.50% | 2 of 200 | 1.00% | 0.981 |
| jpeg-qv4 | 200 | 200 | 87.00% | 2 of 200 | 1.00% | 0.989 |
| noise-a10 | 200 | 200 | 93.00% | 33 of 200 | 16.50% | 0.954 |
| noise-a40 | 200 | 200 | 28.00% | 20 of 200 | 10.00% | 0.679 |
| none | 7200 | 7326 | 96.14% | 82 of 7326 | 1.12% | 0.995 |
| repost-720-qv9x1 | 200 | 200 | 81.00% | 8 of 200 | 4.00% | 0.964 |
| repost-960-qv12x1 | 200 | 200 | 78.00% | 6 of 200 | 3.00% | 0.954 |
| repost-960-qv7x4 | 200 | 200 | 85.50% | 8 of 200 | 4.00% | 0.971 |
| resize-0.25-bicubic | 200 | 200 | 94.00% | 2 of 200 | 1.00% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 98.00% | 3 of 200 | 1.50% | 0.999 |
| sharpen-a1.0 | 200 | 200 | 96.50% | 3 of 200 | 1.50% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 28.00% (noise-a40); FPR 16.50% (noise-a10).

B-photo transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (299 needed with none flagged) |
| blur-s3.0 | 200 | 13 | 6.50% | 3.89% | 10.14% | exceeds |
| jpeg-qv12 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |
| jpeg-qv4 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |
| noise-a10 | 200 | 33 | 16.50% | 12.33% | 21.44% | exceeds |
| noise-a40 | 200 | 20 | 10.00% | 6.73% | 14.20% | exceeds |
| none | 7326 | 82 | 1.12% | 0.92% | 1.34% | not resolved |
| repost-720-qv9x1 | 200 | 8 | 4.00% | 2.01% | 7.10% | exceeds |
| repost-960-qv12x1 | 200 | 6 | 3.00% | 1.31% | 5.83% | exceeds |
| repost-960-qv7x4 | 200 | 8 | 4.00% | 2.01% | 7.10% | exceeds |
| resize-0.25-bicubic | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |
| resize-0.5-bicubic | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (299 needed with none flagged) |
| sharpen-a1.0 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (299 needed with none flagged) |

0 of 13 authentic transform strata are shown within the budget, 6 exceed it, 7 are not resolved.

B-photo by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 98.21% | – | – | – |
| infinity | 1992 | 0 | 98.95% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 97.26% | – | – | – |
| open-magvit2 | 804 | 0 | 69.40% | – | – | – |
| rar | 1992 | 0 | 94.98% | – | – | – |
| real:csafe | 0 | 5304 | – | 124 of 5304 | 2.34% | – |
| real:floreview | 0 | 2486 | – | 41 of 2486 | 1.65% | – |
| real:hdrplus | 0 | 1936 | – | 20 of 1936 | 1.03% | – |
| sd3 | 532 | 0 | 97.56% | – | – | – |
| switti | 804 | 0 | 97.26% | – | – | – |
| var | 804 | 0 | 76.62% | – | – | – |

Worst family slice with at least 100 items of the class: recall 69.40% (open-magvit2); FPR 2.34% (real:csafe).

B-photo transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 87 | 2.18% | 1.81% | 2.60% | exceeds |
| real:floreview | 1874 | 33 | 1.76% | 1.29% | 2.35% | exceeds |
| real:hdrplus | 1456 | 19 | 1.30% | 0.86% | 1.91% | not resolved |

0 of 3 authentic family strata are shown within the budget, 2 exceed it, 1 are not resolved.
