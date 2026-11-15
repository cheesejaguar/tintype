4800 items scored by all 1 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**C-strict-degraded**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.731974 | 2 of 2400 | 0.08% | 0.26% | 63.62% (59.49%–67.40%) | 88.52% |
| 1% empirical FPR | 0.355329 | 24 of 2400 | 1.00% | 1.40% | 78.67% (75.75%–81.62%) | 44.28% |
| frozen cutoff | 0.500000 | 13 of 2400 | 0.54% | 0.86% | 72.62% (69.19%–75.93%) | 57.52% |

C-strict-degraded by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
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
| csafe-blur-s1.0 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-blur-s3.0 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-jpeg-qv12 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-jpeg-qv4 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-noise-a10 | 0 | 109 | – | 5 of 109 | 4.59% | – |
| csafe-noise-a40 | 0 | 109 | – | 4 of 109 | 3.67% | – |
| csafe-repost-720-qv9x1 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-repost-960-qv12x1 | 0 | 109 | – | 1 of 109 | 0.92% | – |
| csafe-repost-960-qv7x4 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-resize-0.25-bicubic | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-resize-0.5-bicubic | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-sharpen-a1.0 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| floreview-blur-s1.0 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-blur-s3.0 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-jpeg-qv12 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-jpeg-qv4 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-noise-a10 | 0 | 51 | – | 2 of 51 | 3.92% | – |
| floreview-noise-a40 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-repost-720-qv9x1 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-repost-960-qv12x1 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-repost-960-qv7x4 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-resize-0.25-bicubic | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-resize-0.5-bicubic | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-sharpen-a1.0 | 0 | 51 | – | 0 of 51 | 0.00% | – |
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
| hdrplus-blur-s1.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-blur-s3.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-jpeg-qv12 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-jpeg-qv4 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-noise-a10 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-noise-a40 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-repost-720-qv9x1 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-repost-960-qv12x1 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-repost-960-qv7x4 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-resize-0.25-bicubic | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-resize-0.5-bicubic | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-sharpen-a1.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |

Worst dataset slice with at least 100 items of the class: recall 8.98% (arforensics-noise-a40); FPR 4.59% (csafe-noise-a10).
