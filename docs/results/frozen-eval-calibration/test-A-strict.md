19326 items scored by all 1 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-strict**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.567345 | 9 of 9726 | 0.09% | 0.16% | 84.52% (83.29%–85.77%) | 90.22% |
| 1% empirical FPR | 0.251665 | 97 of 9726 | 1.00% | 1.18% | 92.25% (91.35%–93.05%) | 48.30% |
| frozen cutoff | 0.500000 | 12 of 9726 | 0.12% | 0.20% | 86.12% (84.95%–87.28%) | 87.58% |

A-strict, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 9726 (0.00%) | 0 of 9600 (0.00%) | 0 of 0 | 12 of 9726 | 0.12% | 0.20% | 86.12% (84.95%–87.28%) | 86.12% | 87.58% | 0.00% |
| margin 1.5 | 88.81% | 153 of 9726 (1.57%) | 2009 of 9600 (20.93%) | 2162 of 2162 | 0 of 9726 | 0.00% | 0.03% | 72.95% (71.47%–74.44%) | 86.12% | 100.00% | 1.77% |

A-strict by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 91.07% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 93.41% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 90.42% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 50.30% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 65.87% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 82.63% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 5.99% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 55.69% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 50.30% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 64.07% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 77.25% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 96.41% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 88.02% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 10 of 5304 | 0.19% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 0 of 54 | 0.00% | – |
| floreview-google | 0 | 161 | – | 0 of 161 | 0.00% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 0 of 54 | 0.00% | – |
| floreview-lg | 0 | 174 | – | 0 of 174 | 0.00% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 0 of 108 | 0.00% | – |
| floreview-samsung | 0 | 497 | – | 0 of 497 | 0.00% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 2 of 388 | 0.52% | – |
| genimagepp | 1200 | 0 | 95.00% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 60.61% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 75.76% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 3.03% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 66.67% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 63.64% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 75.76% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 90.91% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 81.82% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 96.97% | – | – | – |
| hdrplus | 0 | 1936 | – | 0 of 1936 | 0.00% | – |

Worst source slice with at least 100 items of the class: recall 5.99% (arforensics-noise-a40); FPR 0.52% (floreview-xiaomi).

A-strict transfer by source: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| source | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 10 | 0.25% | 0.14% | 0.42% | exceeds |
| floreview-apple | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-doogee | 42 | 0 | 0.00% | 0.00% | 6.88% | not resolved (2995 needed with none flagged) |
| floreview-google | 125 | 0 | 0.00% | 0.00% | 2.37% | not resolved (2995 needed with none flagged) |
| floreview-huawei | 249 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-lenovo | 42 | 0 | 0.00% | 0.00% | 6.88% | not resolved (2995 needed with none flagged) |
| floreview-lg | 126 | 0 | 0.00% | 0.00% | 2.35% | not resolved (2995 needed with none flagged) |
| floreview-motorola | 248 | 0 | 0.00% | 0.00% | 1.20% | not resolved (2995 needed with none flagged) |
| floreview-oneplus | 84 | 0 | 0.00% | 0.00% | 3.50% | not resolved (2995 needed with none flagged) |
| floreview-samsung | 377 | 0 | 0.00% | 0.00% | 0.79% | not resolved (2995 needed with none flagged) |
| floreview-sony | 40 | 0 | 0.00% | 0.00% | 7.22% | not resolved (2995 needed with none flagged) |
| floreview-xiaomi | 292 | 2 | 0.68% | 0.12% | 2.14% | exceeds |
| hdrplus | 1456 | 0 | 0.00% | 0.00% | 0.21% | not resolved (2995 needed with none flagged) |

0 of 13 authentic source strata are shown within the budget, 2 exceed it, 11 are not resolved.

A-strict by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 93.50% | 0 of 200 | 0.00% | 0.996 |
| blur-s3.0 | 200 | 200 | 92.00% | 0 of 200 | 0.00% | 0.999 |
| jpeg-qv12 | 200 | 200 | 52.00% | 0 of 200 | 0.00% | 0.974 |
| jpeg-qv4 | 200 | 200 | 67.50% | 0 of 200 | 0.00% | 0.987 |
| noise-a10 | 200 | 200 | 82.50% | 1 of 200 | 0.50% | 0.960 |
| noise-a40 | 200 | 200 | 5.50% | 5 of 200 | 2.50% | 0.649 |
| none | 7200 | 7326 | 91.72% | 6 of 7326 | 0.08% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 57.50% | 0 of 200 | 0.00% | 0.959 |
| repost-960-qv12x1 | 200 | 200 | 52.50% | 0 of 200 | 0.00% | 0.952 |
| repost-960-qv7x4 | 200 | 200 | 66.00% | 0 of 200 | 0.00% | 0.970 |
| resize-0.25-bicubic | 200 | 200 | 79.50% | 0 of 200 | 0.00% | 0.995 |
| resize-0.5-bicubic | 200 | 200 | 94.00% | 0 of 200 | 0.00% | 0.995 |
| sharpen-a1.0 | 200 | 200 | 89.50% | 0 of 200 | 0.00% | 0.991 |

Worst transform slice with at least 100 items of the class: recall 5.50% (noise-a40); FPR 2.50% (noise-a40).

A-strict transfer by transform: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| transform | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| blur-s3.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| jpeg-qv12 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| jpeg-qv4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| noise-a10 | 200 | 1 | 0.50% | 0.03% | 2.35% | not resolved (2995 needed with none flagged) |
| noise-a40 | 200 | 5 | 2.50% | 0.99% | 5.18% | exceeds |
| none | 7326 | 6 | 0.08% | 0.04% | 0.16% | not resolved |
| repost-720-qv9x1 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| repost-960-qv12x1 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| repost-960-qv7x4 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| resize-0.25-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| resize-0.5-bicubic | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |
| sharpen-a1.0 | 200 | 0 | 0.00% | 0.00% | 1.49% | not resolved (2995 needed with none flagged) |

0 of 13 authentic transform strata are shown within the budget, 1 exceed it, 12 are not resolved.

A-strict by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 87.50% | – | – | – |
| infinity | 1992 | 0 | 95.88% | – | – | – |
| janus-pro | 804 | 0 | 95.15% | – | – | – |
| llamagen | 804 | 0 | 90.17% | – | – | – |
| open-magvit2 | 804 | 0 | 51.37% | – | – | – |
| rar | 1992 | 0 | 89.71% | – | – | – |
| real:csafe | 0 | 5304 | – | 10 of 5304 | 0.19% | – |
| real:floreview | 0 | 2486 | – | 2 of 2486 | 0.08% | – |
| real:hdrplus | 0 | 1936 | – | 0 of 1936 | 0.00% | – |
| sd3 | 532 | 0 | 94.55% | – | – | – |
| switti | 804 | 0 | 94.78% | – | – | – |
| var | 804 | 0 | 58.71% | – | – | – |

Worst family slice with at least 100 items of the class: recall 51.37% (open-magvit2); FPR 0.19% (real:csafe).

A-strict transfer by family: authentic parent clusters flagged at the frozen cutoff 0.5, against a false-positive budget of 0.10%. Bounds are one-sided 95% Clopper–Pearson; a stratum is within the budget when its upper bound is, exceeds it when its lower bound does, and is otherwise not resolved.

| family | Authentic parent clusters | Flagged | FPR | Lower bound | Upper bound | Against the budget |
|---|---|---|---|---|---|---|
| real:csafe | 3996 | 10 | 0.25% | 0.14% | 0.42% | exceeds |
| real:floreview | 1874 | 2 | 0.11% | 0.02% | 0.34% | not resolved (2995 needed with none flagged) |
| real:hdrplus | 1456 | 0 | 0.00% | 0.00% | 0.21% | not resolved (2995 needed with none flagged) |

0 of 3 authentic family strata are shown within the budget, 1 exceed it, 2 are not resolved.
