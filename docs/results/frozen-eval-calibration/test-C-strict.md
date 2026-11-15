19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**C-strict**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.631828 | 9 of 9726 | 0.09% | 0.16% | 85.00% (83.80%–86.20%) | 90.27% |
| 1% empirical FPR | 0.268534 | 97 of 9726 | 1.00% | 1.18% | 92.50% (91.63%–93.26%) | 48.37% |
| frozen cutoff | 0.500000 | 18 of 9726 | 0.19% | 0.27% | 87.70% (86.61%–88.83%) | 82.72% |

C-strict by source, at the frozen cutoff 0.5:

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

C-strict transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 12 | 0.30% | 0.17% | 0.49% | exceeds |
| floreview-apple | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-doogee | 42 | 0 | 0.00% | 0.00% | 6.88% | not resolved (2995 needed with none flagged) |
| floreview-google | 125 | 0 | 0.00% | 0.00% | 2.37% | not resolved (2995 needed with none flagged) |
| floreview-huawei | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-lenovo | 42 | 1 | 2.38% | 0.12% | 10.80% | exceeds |
| floreview-lg | 126 | 0 | 0.00% | 0.00% | 2.35% | not resolved (2995 needed with none flagged) |
| floreview-motorola | 248 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-oneplus | 84 | 1 | 1.19% | 0.06% | 5.52% | not resolved (2995 needed with none flagged) |
| floreview-samsung | 377 | 1 | 0.27% | 0.01% | 1.25% | not resolved (2995 needed with none flagged) |
| floreview-sony | 40 | 0 | 0.00% | 0.00% | 7.22% | not resolved (2995 needed with none flagged) |
| floreview-xiaomi | 292 | 1 | 0.34% | 0.02% | 1.61% | not resolved (2995 needed with none flagged) |
| hdrplus | 1456 | 1 | 0.07% | 0.00% | 0.33% | not resolved (2995 needed with none flagged) |

0 of 13 authentic source strata are shown within the budget, 2 exceed it, 11 are not resolved.

C-strict by transform, at the frozen cutoff 0.5:

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

C-strict transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| blur-s3.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| jpeg-qv12 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| jpeg-qv4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| noise-a10 | 200 | 7 | 3.50% | 1.65% | 6.47% | exceeds |
| noise-a40 | 200 | 4 | 2.00% | 0.69% | 4.52% | exceeds |
| none | 7326 | 5 | 0.07% | 0.03% | 0.14% | not resolved |
| repost-720-qv9x1 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| repost-960-qv12x1 | 200 | 2 | 1.00% | 0.18% | 3.11% | exceeds |
| repost-960-qv7x4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| resize-0.25-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| resize-0.5-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| sharpen-a1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |

0 of 13 authentic transform strata are shown within the budget, 3 exceed it, 10 are not resolved.

C-strict by family, at the frozen cutoff 0.5:

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

C-strict transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 12 | 0.30% | 0.17% | 0.49% | exceeds |
| real:floreview | 1874 | 4 | 0.21% | 0.07% | 0.49% | not resolved (2995 needed with none flagged) |
| real:hdrplus | 1456 | 1 | 0.07% | 0.00% | 0.33% | not resolved (2995 needed with none flagged) |

0 of 3 authentic family strata are shown within the budget, 1 exceed it, 2 are not resolved.
