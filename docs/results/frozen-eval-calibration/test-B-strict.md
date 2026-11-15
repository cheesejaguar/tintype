19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**B-strict**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.675646 | 9 of 9726 | 0.09% | 0.16% | 85.28% (84.08%–86.49%) | 90.30% |
| 1% empirical FPR | 0.284811 | 97 of 9726 | 1.00% | 1.18% | 92.49% (91.65%–93.28%) | 48.37% |
| frozen cutoff | 0.500000 | 29 of 9726 | 0.30% | 0.41% | 88.82% (87.80%–89.88%) | 75.06% |

B-strict by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 92.73% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 94.01% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 92.81% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 59.88% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 73.05% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 85.63% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 10.78% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 62.28% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 59.28% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 68.86% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 83.23% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.01% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 92.81% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 21 of 5304 | 0.40% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-google | 0 | 161 | – | 0 of 161 | 0.00% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-lg | 0 | 174 | – | 0 of 174 | 0.00% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 1 of 108 | 0.93% | – |
| floreview-samsung | 0 | 497 | – | 1 of 497 | 0.20% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 1 of 388 | 0.26% | – |
| genimagepp | 1200 | 0 | 97.17% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 72.73% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 9.09% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 78.79% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 87.88% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 3 of 1936 | 0.15% | – |

Worst source slice with at least 100 items of the class: recall 10.78% (arforensics-noise-a40); FPR 0.93% (floreview-oneplus).

B-strict transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 18 | 0.45% | 0.29% | 0.67% | exceeds |
| floreview-apple | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-doogee | 42 | 1 | 2.38% | 0.12% | 10.80% | exceeds |
| floreview-google | 125 | 0 | 0.00% | 0.00% | 2.37% | not resolved (2995 needed with none flagged) |
| floreview-huawei | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-lenovo | 42 | 1 | 2.38% | 0.12% | 10.80% | exceeds |
| floreview-lg | 126 | 0 | 0.00% | 0.00% | 2.35% | not resolved (2995 needed with none flagged) |
| floreview-motorola | 248 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-oneplus | 84 | 1 | 1.19% | 0.06% | 5.52% | not resolved (2995 needed with none flagged) |
| floreview-samsung | 377 | 1 | 0.27% | 0.01% | 1.25% | not resolved (2995 needed with none flagged) |
| floreview-sony | 40 | 0 | 0.00% | 0.00% | 7.22% | not resolved (2995 needed with none flagged) |
| floreview-xiaomi | 292 | 1 | 0.34% | 0.02% | 1.61% | not resolved (2995 needed with none flagged) |
| hdrplus | 1456 | 3 | 0.21% | 0.06% | 0.53% | not resolved (2995 needed with none flagged) |

0 of 13 authentic source strata are shown within the budget, 3 exceed it, 10 are not resolved.

B-strict by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 94.50% | 0 of 200 | 0.00% | 0.997 |
| blur-s3.0 | 200 | 200 | 94.00% | 0 of 200 | 0.00% | 0.999 |
| jpeg-qv12 | 200 | 200 | 62.00% | 1 of 200 | 0.50% | 0.976 |
| jpeg-qv4 | 200 | 200 | 74.50% | 0 of 200 | 0.00% | 0.988 |
| noise-a10 | 200 | 200 | 87.50% | 9 of 200 | 4.50% | 0.959 |
| noise-a40 | 200 | 200 | 10.50% | 7 of 200 | 3.50% | 0.657 |
| none | 7200 | 7326 | 93.47% | 9 of 7326 | 0.12% | 0.995 |
| repost-720-qv9x1 | 200 | 200 | 65.00% | 1 of 200 | 0.50% | 0.959 |
| repost-960-qv12x1 | 200 | 200 | 63.00% | 2 of 200 | 1.00% | 0.951 |
| repost-960-qv7x4 | 200 | 200 | 72.50% | 0 of 200 | 0.00% | 0.971 |
| resize-0.25-bicubic | 200 | 200 | 85.50% | 0 of 200 | 0.00% | 0.995 |
| resize-0.5-bicubic | 200 | 200 | 95.50% | 0 of 200 | 0.00% | 0.995 |
| sharpen-a1.0 | 200 | 200 | 94.00% | 0 of 200 | 0.00% | 0.989 |

Worst transform slice with at least 100 items of the class: recall 10.50% (noise-a40); FPR 4.50% (noise-a10).

B-strict transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| blur-s3.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| jpeg-qv12 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (2995 needed with none flagged) |
| jpeg-qv4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| noise-a10 | 200 | 9 | 4.50% | 2.37% | 7.72% | exceeds |
| noise-a40 | 200 | 7 | 3.50% | 1.65% | 6.47% | exceeds |
| none | 7326 | 9 | 0.12% | 0.06% | 0.21% | not resolved |
| repost-720-qv9x1 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (2995 needed with none flagged) |
| repost-960-qv12x1 | 200 | 2 | 1.00% | 0.18% | 3.11% | exceeds |
| repost-960-qv7x4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| resize-0.25-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| resize-0.5-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| sharpen-a1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |

0 of 13 authentic transform strata are shown within the budget, 3 exceed it, 10 are not resolved.

B-strict by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 92.58% | – | – | – |
| infinity | 1992 | 0 | 97.14% | – | – | – |
| janus-pro | 804 | 0 | 97.26% | – | – | – |
| llamagen | 804 | 0 | 93.41% | – | – | – |
| open-magvit2 | 804 | 0 | 57.21% | – | – | – |
| rar | 1992 | 0 | 91.37% | – | – | – |
| real:csafe | 0 | 5304 | – | 21 of 5304 | 0.40% | – |
| real:floreview | 0 | 2486 | – | 5 of 2486 | 0.20% | – |
| real:hdrplus | 0 | 1936 | – | 3 of 1936 | 0.15% | – |
| sd3 | 532 | 0 | 95.68% | – | – | – |
| switti | 804 | 0 | 95.65% | – | – | – |
| var | 804 | 0 | 64.18% | – | – | – |

Worst family slice with at least 100 items of the class: recall 57.21% (open-magvit2); FPR 0.40% (real:csafe).

B-strict transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 18 | 0.45% | 0.29% | 0.67% | exceeds |
| real:floreview | 1874 | 5 | 0.27% | 0.11% | 0.56% | exceeds |
| real:hdrplus | 1456 | 3 | 0.21% | 0.06% | 0.53% | not resolved (2995 needed with none flagged) |

0 of 3 authentic family strata are shown within the budget, 2 exceed it, 1 are not resolved.
