19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-photo**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.788886 | 9 of 9726 | 0.09% | 0.16% | 84.71% (83.52%–85.92%) | 90.24% |
| 1% empirical FPR | 0.468322 | 97 of 9726 | 1.00% | 1.18% | 91.47% (90.58%–92.36%) | 48.09% |
| frozen cutoff | 0.500000 | 82 of 9726 | 0.84% | 1.01% | 90.98% (90.06%–91.89%) | 52.15% |

A-photo, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 9726 (0.00%) | 0 of 9600 (0.00%) | 0 of 0 | 82 of 9726 | 0.84% | 1.01% | 90.98% (90.06%–91.89%) | 90.98% | 52.15% | 0.00% |
| margin 0.75 | 96.44% | 159 of 9726 (1.63%) | 529 of 9600 (5.51%) | 688 of 688 | 24 of 9726 | 0.25% | 0.35% | 88.00% (86.96%–89.12%) | 90.98% | 78.27% | 1.67% |

A-photo by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 94.10% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 95.21% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 97.01% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 63.47% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 79.64% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 89.82% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 16.77% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 68.86% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 64.67% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 75.45% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 86.23% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 93.41% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 62 of 5304 | 1.17% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-google | 0 | 161 | – | 2 of 161 | 1.24% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 1 of 54 | 1.85% | – |
| floreview-lg | 0 | 174 | – | 1 of 174 | 0.57% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 1 of 108 | 0.93% | – |
| floreview-samsung | 0 | 497 | – | 3 of 497 | 0.60% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 4 of 388 | 1.03% | – |
| genimagepp | 1200 | 0 | 98.92% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 15.15% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 93.94% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 7 of 1936 | 0.36% | – |

Worst source slice with at least 100 items of the class: recall 16.77% (arforensics-noise-a40); FPR 1.24% (floreview-google).

A-photo transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 46 | 1.15% | 0.89% | 1.47% | not resolved |
| floreview-apple | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (299 needed with none flagged) |
| floreview-doogee | 42 | 1 | 2.38% | 0.12% | 10.80% | not resolved (299 needed with none flagged) |
| floreview-google | 125 | 1 | 0.80% | 0.04% | 3.74% | not resolved (299 needed with none flagged) |
| floreview-huawei | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (299 needed with none flagged) |
| floreview-lenovo | 42 | 1 | 2.38% | 0.12% | 10.80% | not resolved (299 needed with none flagged) |
| floreview-lg | 126 | 1 | 0.79% | 0.04% | 3.71% | not resolved (299 needed with none flagged) |
| floreview-motorola | 248 | 0 | 0.00% | 0.00% | 1.20% | not resolved (299 needed with none flagged) |
| floreview-oneplus | 84 | 1 | 1.19% | 0.06% | 5.52% | not resolved (299 needed with none flagged) |
| floreview-samsung | 377 | 2 | 0.53% | 0.09% | 1.66% | not resolved |
| floreview-sony | 40 | 0 | 0.00% | 0.00% | 7.22% | not resolved (299 needed with none flagged) |
| floreview-xiaomi | 292 | 4 | 1.37% | 0.47% | 3.11% | not resolved (299 needed with none flagged) |
| hdrplus | 1456 | 7 | 0.48% | 0.23% | 0.90% | within |

1 of 13 authentic source strata are shown within the budget, 0 exceed it, 12 are not resolved.

A-photo by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 96.00% | 1 of 200 | 0.50% | 0.998 |
| blur-s3.0 | 200 | 200 | 97.50% | 1 of 200 | 0.50% | 0.998 |
| jpeg-qv12 | 200 | 200 | 67.50% | 1 of 200 | 0.50% | 0.981 |
| jpeg-qv4 | 200 | 200 | 81.50% | 2 of 200 | 1.00% | 0.988 |
| noise-a10 | 200 | 200 | 91.00% | 21 of 200 | 10.50% | 0.956 |
| noise-a40 | 200 | 200 | 16.50% | 13 of 200 | 6.50% | 0.673 |
| none | 7200 | 7326 | 94.90% | 35 of 7326 | 0.48% | 0.995 |
| repost-720-qv9x1 | 200 | 200 | 73.50% | 2 of 200 | 1.00% | 0.963 |
| repost-960-qv12x1 | 200 | 200 | 68.50% | 3 of 200 | 1.50% | 0.957 |
| repost-960-qv7x4 | 200 | 200 | 79.00% | 0 of 200 | 0.00% | 0.974 |
| resize-0.25-bicubic | 200 | 200 | 88.00% | 1 of 200 | 0.50% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 97.00% | 0 of 200 | 0.00% | 0.997 |
| sharpen-a1.0 | 200 | 200 | 94.50% | 2 of 200 | 1.00% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 16.50% (noise-a40); FPR 10.50% (noise-a10).

A-photo transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (299 needed with none flagged) |
| blur-s3.0 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (299 needed with none flagged) |
| jpeg-qv12 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (299 needed with none flagged) |
| jpeg-qv4 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |
| noise-a10 | 200 | 21 | 10.50% | 7.14% | 14.77% | exceeds |
| noise-a40 | 200 | 13 | 6.50% | 3.89% | 10.14% | exceeds |
| none | 7326 | 35 | 0.48% | 0.35% | 0.63% | within |
| repost-720-qv9x1 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |
| repost-960-qv12x1 | 200 | 3 | 1.50% | 0.41% | 3.83% | not resolved (299 needed with none flagged) |
| repost-960-qv7x4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (299 needed with none flagged) |
| resize-0.25-bicubic | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (299 needed with none flagged) |
| resize-0.5-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (299 needed with none flagged) |
| sharpen-a1.0 | 200 | 2 | 1.00% | 0.18% | 3.11% | not resolved (299 needed with none flagged) |

1 of 13 authentic transform strata are shown within the budget, 2 exceed it, 10 are not resolved.

A-photo by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 96.15% | – | – | – |
| infinity | 1992 | 0 | 97.89% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 94.78% | – | – | – |
| open-magvit2 | 804 | 0 | 62.56% | – | – | – |
| rar | 1992 | 0 | 93.27% | – | – | – |
| real:csafe | 0 | 5304 | – | 62 of 5304 | 1.17% | – |
| real:floreview | 0 | 2486 | – | 13 of 2486 | 0.52% | – |
| real:hdrplus | 0 | 1936 | – | 7 of 1936 | 0.36% | – |
| sd3 | 532 | 0 | 96.80% | – | – | – |
| switti | 804 | 0 | 96.77% | – | – | – |
| var | 804 | 0 | 69.28% | – | – | – |

Worst family slice with at least 100 items of the class: recall 62.56% (open-magvit2); FPR 1.17% (real:csafe).

A-photo transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 1.00%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 46 | 1.15% | 0.89% | 1.47% | not resolved |
| real:floreview | 1874 | 11 | 0.59% | 0.33% | 0.97% | within |
| real:hdrplus | 1456 | 7 | 0.48% | 0.23% | 0.90% | within |

2 of 3 authentic family strata are shown within the budget, 0 exceed it, 1 are not resolved.
