4800 items scored by all 1 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-photo-degraded**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.902796 | 2 of 2400 | 0.08% | 0.26% | 60.08% (55.85%–63.95%) | 87.93% |
| 1% empirical FPR | 0.641412 | 24 of 2400 | 1.00% | 1.40% | 75.04% (71.67%–78.28%) | 43.12% |
| frozen cutoff | 0.500000 | 47 of 2400 | 1.96% | 2.49% | 79.21% (76.28%–82.30%) | 29.01% |

A-photo-degraded, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 2400 (0.00%) | 0 of 2400 (0.00%) | 0 of 0 | 47 of 2400 | 1.96% | 2.49% | 79.21% (76.28%–82.30%) | 79.21% | 29.01% | 0.00% |
| margin 0.75 | 92.29% | 89 of 2400 (3.71%) | 281 of 2400 (11.71%) | 370 of 370 | 15 of 2400 | 0.62% | 0.96% | 73.67% (70.16%–76.99%) | 79.21% | 54.35% | 3.79% |

A-photo-degraded by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
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
| csafe-blur-s1.0 | 0 | 109 | – | 1 of 109 | 0.92% | – |
| csafe-blur-s3.0 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-jpeg-qv12 | 0 | 109 | – | 1 of 109 | 0.92% | – |
| csafe-jpeg-qv4 | 0 | 109 | – | 2 of 109 | 1.83% | – |
| csafe-noise-a10 | 0 | 109 | – | 15 of 109 | 13.76% | – |
| csafe-noise-a40 | 0 | 109 | – | 11 of 109 | 10.09% | – |
| csafe-repost-720-qv9x1 | 0 | 109 | – | 2 of 109 | 1.83% | – |
| csafe-repost-960-qv12x1 | 0 | 109 | – | 2 of 109 | 1.83% | – |
| csafe-repost-960-qv7x4 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-resize-0.25-bicubic | 0 | 109 | – | 1 of 109 | 0.92% | – |
| csafe-resize-0.5-bicubic | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-sharpen-a1.0 | 0 | 109 | – | 1 of 109 | 0.92% | – |
| floreview-blur-s1.0 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-blur-s3.0 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-jpeg-qv12 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-jpeg-qv4 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-noise-a10 | 0 | 51 | – | 5 of 51 | 9.80% | – |
| floreview-noise-a40 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-repost-720-qv9x1 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-repost-960-qv12x1 | 0 | 51 | – | 1 of 51 | 1.96% | – |
| floreview-repost-960-qv7x4 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-resize-0.25-bicubic | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-resize-0.5-bicubic | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-sharpen-a1.0 | 0 | 51 | – | 1 of 51 | 1.96% | – |
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
| hdrplus-blur-s1.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-blur-s3.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-jpeg-qv12 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-jpeg-qv4 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-noise-a10 | 0 | 40 | – | 1 of 40 | 2.50% | – |
| hdrplus-noise-a40 | 0 | 40 | – | 1 of 40 | 2.50% | – |
| hdrplus-repost-720-qv9x1 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-repost-960-qv12x1 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-repost-960-qv7x4 | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-resize-0.25-bicubic | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-resize-0.5-bicubic | 0 | 40 | – | 0 of 40 | 0.00% | – |
| hdrplus-sharpen-a1.0 | 0 | 40 | – | 0 of 40 | 0.00% | – |

Worst dataset slice with at least 100 items of the class: recall 16.77% (arforensics-noise-a40); FPR 13.76% (csafe-noise-a10).
