19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-fast**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.645697 | 9 of 9726 | 0.09% | 0.16% | 85.93% (84.83%–87.06%) | 90.37% |
| 1% empirical FPR | 0.364293 | 97 of 9726 | 1.00% | 1.18% | 92.06% (91.10%–92.94%) | 48.25% |
| frozen cutoff | 0.500000 | 37 of 9726 | 0.38% | 0.50% | 89.47% (88.42%–90.49%) | 70.38% |

A-fast by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 92.87% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 94.01% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 94.01% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 61.68% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 75.45% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 89.82% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 26.35% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 67.07% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 62.28% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 75.45% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 82.63% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 92.22% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 19 of 5304 | 0.36% | – |
| floreview-apple | 0 | 333 | – | 1 of 333 | 0.30% | – |
| floreview-doogee | 0 | 54 | – | 0 of 54 | 0.00% | – |
| floreview-google | 0 | 161 | – | 1 of 161 | 0.62% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-lg | 0 | 174 | – | 2 of 174 | 1.15% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 3 of 108 | 2.78% | – |
| floreview-samsung | 0 | 497 | – | 4 of 497 | 0.80% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 2 of 388 | 0.52% | – |
| genimagepp | 1200 | 0 | 94.83% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 30.30% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 84.85% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 84.85% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 90.91% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 93.94% | – | – | – |
| hdrplus | 0 | 1936 | – | 3 of 1936 | 0.15% | – |

Worst source slice with at least 100 items of the class: recall 26.35% (arforensics-noise-a40); FPR 2.78% (floreview-oneplus).

A-fast transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.50%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 17 | 0.43% | 0.27% | 0.64% | not resolved |
| floreview-apple | 249 | 1 | 0.40% | 0.02% | 1.89% | not resolved (598 needed with none flagged) |
| floreview-doogee | 42 | 0 | 0.00% | 0.00% | 6.88% | not resolved (598 needed with none flagged) |
| floreview-google | 125 | 1 | 0.80% | 0.04% | 3.74% | not resolved (598 needed with none flagged) |
| floreview-huawei | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (598 needed with none flagged) |
| floreview-lenovo | 42 | 1 | 2.38% | 0.12% | 10.80% | not resolved (598 needed with none flagged) |
| floreview-lg | 126 | 2 | 1.59% | 0.28% | 4.91% | not resolved (598 needed with none flagged) |
| floreview-motorola | 248 | 0 | 0.00% | 0.00% | 1.20% | not resolved (598 needed with none flagged) |
| floreview-oneplus | 84 | 2 | 2.38% | 0.42% | 7.31% | not resolved (598 needed with none flagged) |
| floreview-samsung | 377 | 3 | 0.80% | 0.22% | 2.04% | not resolved (598 needed with none flagged) |
| floreview-sony | 40 | 0 | 0.00% | 0.00% | 7.22% | not resolved (598 needed with none flagged) |
| floreview-xiaomi | 292 | 2 | 0.68% | 0.12% | 2.14% | not resolved (598 needed with none flagged) |
| hdrplus | 1456 | 3 | 0.21% | 0.06% | 0.53% | not resolved |

0 of 13 authentic source strata are shown within the budget, 0 exceed it, 13 are not resolved.

A-fast by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 94.50% | 0 of 200 | 0.00% | 0.998 |
| blur-s3.0 | 200 | 200 | 95.00% | 2 of 200 | 1.00% | 0.999 |
| jpeg-qv12 | 200 | 200 | 67.00% | 0 of 200 | 0.00% | 0.979 |
| jpeg-qv4 | 200 | 200 | 78.00% | 0 of 200 | 0.00% | 0.988 |
| noise-a10 | 200 | 200 | 90.50% | 6 of 200 | 3.00% | 0.960 |
| noise-a40 | 200 | 200 | 27.00% | 3 of 200 | 1.50% | 0.734 |
| none | 7200 | 7326 | 93.19% | 22 of 7326 | 0.30% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 70.00% | 3 of 200 | 1.50% | 0.959 |
| repost-960-qv12x1 | 200 | 200 | 66.00% | 1 of 200 | 0.50% | 0.951 |
| repost-960-qv7x4 | 200 | 200 | 77.50% | 0 of 200 | 0.00% | 0.973 |
| resize-0.25-bicubic | 200 | 200 | 85.00% | 0 of 200 | 0.00% | 0.994 |
| resize-0.5-bicubic | 200 | 200 | 96.50% | 0 of 200 | 0.00% | 0.999 |
| sharpen-a1.0 | 200 | 200 | 92.50% | 0 of 200 | 0.00% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 27.00% (noise-a40); FPR 3.00% (noise-a10).

A-fast transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.50%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (598 needed with none flagged) |
| blur-s3.0 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (598 needed with none flagged) |
| jpeg-qv12 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (598 needed with none flagged) |
| jpeg-qv4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (598 needed with none flagged) |
| noise-a10 | 200 | 6 | 3.00% | 1.31% | 5.83% | exceeds |
| noise-a40 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (598 needed with none flagged) |
| none | 7326 | 22 | 0.30% | 0.20% | 0.43% | within |
| repost-720-qv9x1 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (598 needed with none flagged) |
| repost-960-qv12x1 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (598 needed with none flagged) |
| repost-960-qv7x4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (598 needed with none flagged) |
| resize-0.25-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (598 needed with none flagged) |
| resize-0.5-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (598 needed with none flagged) |
| sharpen-a1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (598 needed with none flagged) |

1 of 13 authentic transform strata are shown within the budget, 1 exceed it, 11 are not resolved.

A-fast by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 92.11% | – | – | – |
| infinity | 1992 | 0 | 98.19% | – | – | – |
| janus-pro | 804 | 0 | 97.89% | – | – | – |
| llamagen | 804 | 0 | 94.65% | – | – | – |
| open-magvit2 | 804 | 0 | 59.08% | – | – | – |
| rar | 1992 | 0 | 92.02% | – | – | – |
| real:csafe | 0 | 5304 | – | 19 of 5304 | 0.36% | – |
| real:floreview | 0 | 2486 | – | 15 of 2486 | 0.60% | – |
| real:hdrplus | 0 | 1936 | – | 3 of 1936 | 0.15% | – |
| sd3 | 532 | 0 | 94.55% | – | – | – |
| switti | 804 | 0 | 95.77% | – | – | – |
| var | 804 | 0 | 65.17% | – | – | – |

Worst family slice with at least 100 items of the class: recall 59.08% (open-magvit2); FPR 0.60% (real:floreview).

A-fast transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.50%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 17 | 0.43% | 0.27% | 0.64% | not resolved |
| real:floreview | 1874 | 12 | 0.64% | 0.37% | 1.04% | not resolved |
| real:hdrplus | 1456 | 3 | 0.21% | 0.06% | 0.53% | not resolved |

0 of 3 authentic family strata are shown within the budget, 0 exceed it, 3 are not resolved.
