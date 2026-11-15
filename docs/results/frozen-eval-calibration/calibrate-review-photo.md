12457 items scored by all 1 systems: 6600 AI, 5857 real; grouping columns found for 12457
Recall intervals: 95% bootstrap over 7657 parent clusters (4200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**bound-max-photo**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.657982 | 5 of 5857 | 0.09% | 0.18% | 82.97% (81.17%–84.65%) | 90.76% |
| 1% empirical FPR | 0.371171 | 58 of 5857 | 0.99% | 1.23% | 88.06% (86.57%–89.45%) | 47.32% |
| frozen cutoff | 0.500000 | 26 of 5857 | 0.44% | 0.62% | 86.15% (84.63%–87.67%) | 66.22% |

bound-max-photo, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 5857 (0.00%) | 0 of 6600 (0.00%) | 0 of 0 | 26 of 5857 | 0.44% | 0.62% | 86.15% (84.63%–87.67%) | 86.15% | 66.22% | 0.00% |
| margin 0.25 | 98.64% | 20 of 5857 (0.34%) | 150 of 6600 (2.27%) | 170 of 170 | 18 of 5857 | 0.31% | 0.46% | 84.85% (83.21%–86.43%) | 86.15% | 73.61% | 0.36% |
| margin 0.5 | 97.56% | 44 of 5857 (0.75%) | 260 of 6600 (3.94%) | 304 of 304 | 10 of 5857 | 0.17% | 0.29% | 84.06% (82.32%–85.67%) | 86.15% | 83.26% | 0.78% |
| margin 0.75 | 95.66% | 81 of 5857 (1.38%) | 460 of 6600 (6.97%) | 541 of 541 | 4 of 5857 | 0.07% | 0.16% | 82.45% (80.63%–84.21%) | 86.15% | 92.42% | 1.44% |
| margin 1 | 93.77% | 127 of 5857 (2.17%) | 649 of 6600 (9.83%) | 776 of 776 | 2 of 5857 | 0.03% | 0.11% | 80.65% (78.79%–82.45%) | 86.15% | 95.98% | 2.24% |
| margin 1.25 | 91.46% | 194 of 5857 (3.31%) | 870 of 6600 (13.18%) | 1064 of 1064 | 1 of 5857 | 0.02% | 0.08% | 78.62% (76.67%–80.46%) | 86.15% | 97.90% | 3.41% |
| margin 1.5 | 89.44% | 261 of 5857 (4.46%) | 1055 of 6600 (15.98%) | 1316 of 1316 | 1 of 5857 | 0.02% | 0.08% | 76.77% (74.78%–78.66%) | 86.15% | 97.85% | 4.57% |
| margin 1.75 | 87.72% | 315 of 5857 (5.38%) | 1215 of 6600 (18.41%) | 1530 of 1530 | 1 of 5857 | 0.02% | 0.08% | 75.12% (73.12%–77.11%) | 86.15% | 97.80% | 5.51% |
| margin 2 | 86.48% | 362 of 5857 (6.18%) | 1322 of 6600 (20.03%) | 1684 of 1684 | 1 of 5857 | 0.02% | 0.08% | 74.11% (72.08%–76.16%) | 86.15% | 97.77% | 6.32% |
| margin 2.5 | 77.84% | 864 of 5857 (14.75%) | 1897 of 6600 (28.74%) | 2761 of 2761 | 1 of 5857 | 0.02% | 0.08% | 67.12% (65.04%–69.37%) | 86.15% | 97.54% | 14.89% |
| margin 3 | 69.62% | 1468 of 5857 (25.06%) | 2316 of 6600 (35.09%) | 3784 of 3784 | 1 of 5857 | 0.02% | 0.08% | 61.83% (59.77%–64.01%) | 86.15% | 97.34% | 25.16% |

bound-max-photo by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 3000 | 0 | 89.63% | – | – | – |
| arforensics-blur-s1.0 | 143 | 0 | 90.21% | – | – | – |
| arforensics-blur-s3.0 | 143 | 0 | 92.31% | – | – | – |
| arforensics-jpeg-qv12 | 143 | 0 | 65.03% | – | – | – |
| arforensics-jpeg-qv4 | 143 | 0 | 70.63% | – | – | – |
| arforensics-noise-a10 | 143 | 0 | 79.72% | – | – | – |
| arforensics-noise-a40 | 143 | 0 | 8.39% | – | – | – |
| arforensics-repost-720-qv9x1 | 143 | 0 | 66.43% | – | – | – |
| arforensics-repost-960-qv12x1 | 143 | 0 | 65.03% | – | – | – |
| arforensics-repost-960-qv7x4 | 143 | 0 | 67.13% | – | – | – |
| arforensics-resize-0.25-bicubic | 143 | 0 | 79.72% | – | – | – |
| arforensics-resize-0.5-bicubic | 143 | 0 | 94.41% | – | – | – |
| arforensics-sharpen-a1.0 | 143 | 0 | 83.22% | – | – | – |
| floreview | 0 | 2001 | – | 0 of 2001 | 0.00% | – |
| floreview-blur-s1.0 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-blur-s3.0 | 0 | 116 | – | 3 of 116 | 2.59% | – |
| floreview-jpeg-qv12 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-jpeg-qv4 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-noise-a10 | 0 | 116 | – | 8 of 116 | 6.90% | – |
| floreview-noise-a40 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-repost-720-qv9x1 | 0 | 116 | – | 1 of 116 | 0.86% | – |
| floreview-repost-960-qv12x1 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-repost-960-qv7x4 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-resize-0.25-bicubic | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-resize-0.5-bicubic | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-sharpen-a1.0 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| genimagepp | 1200 | 0 | 99.08% | – | – | – |
| genimagepp-blur-s1.0 | 57 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 57 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 57 | 0 | 80.70% | – | – | – |
| genimagepp-jpeg-qv4 | 57 | 0 | 91.23% | – | – | – |
| genimagepp-noise-a10 | 57 | 0 | 96.49% | – | – | – |
| genimagepp-noise-a40 | 57 | 0 | 12.28% | – | – | – |
| genimagepp-repost-720-qv9x1 | 57 | 0 | 85.96% | – | – | – |
| genimagepp-repost-960-qv12x1 | 57 | 0 | 78.95% | – | – | – |
| genimagepp-repost-960-qv7x4 | 57 | 0 | 85.96% | – | – | – |
| genimagepp-resize-0.25-bicubic | 57 | 0 | 91.23% | – | – | – |
| genimagepp-resize-0.5-bicubic | 57 | 0 | 87.72% | – | – | – |
| genimagepp-sharpen-a1.0 | 57 | 0 | 98.25% | – | – | – |
| hdrplus | 0 | 1456 | – | 2 of 1456 | 0.14% | – |
| hdrplus-blur-s1.0 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-blur-s3.0 | 0 | 84 | – | 3 of 84 | 3.57% | – |
| hdrplus-jpeg-qv12 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-jpeg-qv4 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-noise-a10 | 0 | 84 | – | 5 of 84 | 5.95% | – |
| hdrplus-noise-a40 | 0 | 84 | – | 1 of 84 | 1.19% | – |
| hdrplus-repost-720-qv9x1 | 0 | 84 | – | 1 of 84 | 1.19% | – |
| hdrplus-repost-960-qv12x1 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-repost-960-qv7x4 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-resize-0.25-bicubic | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-resize-0.5-bicubic | 0 | 84 | – | 2 of 84 | 2.38% | – |
| hdrplus-sharpen-a1.0 | 0 | 84 | – | 0 of 84 | 0.00% | – |

Worst dataset slice with at least 100 items of the class: recall 8.39% (arforensics-noise-a40); FPR 6.90% (floreview-noise-a10).
