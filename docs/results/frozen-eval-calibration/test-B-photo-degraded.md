4800 items scored by all 1 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**B-photo-degraded**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.988663 | 2 of 2400 | 0.08% | 0.26% | 44.58% (40.63%–48.60%) | 84.38% |
| 1% empirical FPR | 0.830831 | 24 of 2400 | 1.00% | 1.40% | 75.12% (71.51%–78.51%) | 43.14% |
| frozen cutoff | 0.500000 | 103 of 2400 | 4.29% | 5.04% | 84.62% (82.17%–87.26%) | 16.61% |

B-photo-degraded by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
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
| csafe-blur-s1.0 | 0 | 109 | – | 1 of 109 | 0.92% | – |
| csafe-blur-s3.0 | 0 | 109 | – | 5 of 109 | 4.59% | – |
| csafe-jpeg-qv12 | 0 | 109 | – | 2 of 109 | 1.83% | – |
| csafe-jpeg-qv4 | 0 | 109 | – | 2 of 109 | 1.83% | – |
| csafe-noise-a10 | 0 | 109 | – | 23 of 109 | 21.10% | – |
| csafe-noise-a40 | 0 | 109 | – | 18 of 109 | 16.51% | – |
| csafe-repost-720-qv9x1 | 0 | 109 | – | 6 of 109 | 5.50% | – |
| csafe-repost-960-qv12x1 | 0 | 109 | – | 5 of 109 | 4.59% | – |
| csafe-repost-960-qv7x4 | 0 | 109 | – | 7 of 109 | 6.42% | – |
| csafe-resize-0.25-bicubic | 0 | 109 | – | 1 of 109 | 0.92% | – |
| csafe-resize-0.5-bicubic | 0 | 109 | – | 3 of 109 | 2.75% | – |
| csafe-sharpen-a1.0 | 0 | 109 | – | 2 of 109 | 1.83% | – |
| floreview-blur-s1.0 | 0 | 51 | – | 2 of 51 | 3.92% | – |
| floreview-blur-s3.0 | 0 | 51 | – | 6 of 51 | 11.76% | – |
| floreview-jpeg-qv12 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-jpeg-qv4 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-noise-a10 | 0 | 51 | – | 7 of 51 | 13.73% | – |
| floreview-noise-a40 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-repost-720-qv9x1 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-repost-960-qv12x1 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-repost-960-qv7x4 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-resize-0.25-bicubic | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-resize-0.5-bicubic | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-sharpen-a1.0 | 0 | 51 | – | 1 of 51 | 1.96% | – |
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
| hdrplus-blur-s1.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-blur-s3.0 | 0 | 40 | – | 2 of 40 | 5.00% | – |
| hdrplus-jpeg-qv12 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-jpeg-qv4 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-noise-a10 | 0 | 40 | – | 3 of 40 | 7.50% | – |
| hdrplus-noise-a40 | 0 | 40 | – | 1 of 40 | 2.50% | – |
| hdrplus-repost-720-qv9x1 | 0 | 40 | – | 1 of 40 | 2.50% | – |
| hdrplus-repost-960-qv12x1 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-repost-960-qv7x4 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-resize-0.25-bicubic | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-resize-0.5-bicubic | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-sharpen-a1.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |

Worst dataset slice with at least 100 items of the class: recall 28.14% (arforensics-noise-a40); FPR 21.10% (csafe-noise-a10).
