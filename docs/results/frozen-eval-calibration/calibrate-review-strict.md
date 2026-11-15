12457 items scored by all 1 systems: 6600 AI, 5857 real; grouping columns found for 12457
Recall intervals: 95% bootstrap over 7657 parent clusters (4200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**bound-max-strict**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.401475 | 5 of 5857 | 0.09% | 0.18% | 82.79% (81.01%–84.47%) | 90.74% |
| 1% empirical FPR | 0.217456 | 58 of 5857 | 0.99% | 1.23% | 88.76% (87.33%–90.07%) | 47.52% |
| frozen cutoff | 0.500000 | 0 of 5857 | 0.00% | 0.05% | 79.74% (77.85%–81.53%) | 100.00% |

bound-max-strict, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 5857 (0.00%) | 0 of 6600 (0.00%) | 0 of 0 | 0 of 5857 | 0.00% | 0.05% | 79.74% (77.85%–81.53%) | 79.74% | 100.00% | 0.00% |
| margin 0.25 | 97.74% | 4 of 5857 (0.07%) | 278 of 6600 (4.21%) | 282 of 282 | 0 of 5857 | 0.00% | 0.05% | 77.59% (75.65%–79.44%) | 79.74% | 100.00% | 0.11% |
| margin 0.5 | 95.47% | 11 of 5857 (0.19%) | 553 of 6600 (8.38%) | 564 of 564 | 0 of 5857 | 0.00% | 0.05% | 75.39% (73.31%–77.32%) | 79.74% | 100.00% | 0.27% |
| margin 0.75 | 93.37% | 18 of 5857 (0.31%) | 808 of 6600 (12.24%) | 826 of 826 | 0 of 5857 | 0.00% | 0.05% | 73.15% (71.06%–75.14%) | 79.74% | 100.00% | 0.43% |
| margin 1 | 90.38% | 34 of 5857 (0.58%) | 1164 of 6600 (17.64%) | 1198 of 1198 | 0 of 5857 | 0.00% | 0.05% | 69.59% (67.49%–71.63%) | 79.74% | 100.00% | 0.75% |
| margin 1.25 | 87.93% | 54 of 5857 (0.92%) | 1450 of 6600 (21.97%) | 1504 of 1504 | 0 of 5857 | 0.00% | 0.05% | 66.71% (64.57%–68.90%) | 79.74% | 100.00% | 1.13% |
| margin 1.5 | 85.72% | 78 of 5857 (1.33%) | 1701 of 6600 (25.77%) | 1779 of 1779 | 0 of 5857 | 0.00% | 0.05% | 64.02% (61.90%–66.24%) | 79.74% | 100.00% | 1.58% |
| margin 1.75 | 81.54% | 162 of 5857 (2.77%) | 2138 of 6600 (32.39%) | 2300 of 2300 | 0 of 5857 | 0.00% | 0.05% | 58.95% (56.89%–61.16%) | 79.74% | 100.00% | 3.06% |
| margin 2 | 76.71% | 295 of 5857 (5.04%) | 2606 of 6600 (39.48%) | 2901 of 2901 | 0 of 5857 | 0.00% | 0.05% | 53.17% (51.15%–55.29%) | 79.74% | 100.00% | 5.38% |
| margin 2.5 | 68.50% | 608 of 5857 (10.38%) | 3316 of 6600 (50.24%) | 3924 of 3924 | 0 of 5857 | 0.00% | 0.05% | 44.05% (42.08%–46.02%) | 79.74% | 100.00% | 10.78% |
| margin 3 | 48.44% | 1760 of 5857 (30.05%) | 4663 of 6600 (70.65%) | 6423 of 6423 | 0 of 5857 | 0.00% | 0.05% | 25.79% (24.30%–27.33%) | 79.74% | 100.00% | 30.46% |

bound-max-strict by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 3000 | 0 | 84.83% | – | – | – |
| arforensics-blur-s1.0 | 143 | 0 | 85.31% | – | – | – |
| arforensics-blur-s3.0 | 143 | 0 | 83.92% | – | – | – |
| arforensics-jpeg-qv12 | 143 | 0 | 46.85% | – | – | – |
| arforensics-jpeg-qv4 | 143 | 0 | 62.24% | – | – | – |
| arforensics-noise-a10 | 143 | 0 | 67.83% | – | – | – |
| arforensics-noise-a40 | 143 | 0 | 2.80% | – | – | – |
| arforensics-repost-720-qv9x1 | 143 | 0 | 53.15% | – | – | – |
| arforensics-repost-960-qv12x1 | 143 | 0 | 46.85% | – | – | – |
| arforensics-repost-960-qv7x4 | 143 | 0 | 53.85% | – | – | – |
| arforensics-resize-0.25-bicubic | 143 | 0 | 72.73% | – | – | – |
| arforensics-resize-0.5-bicubic | 143 | 0 | 88.81% | – | – | – |
| arforensics-sharpen-a1.0 | 143 | 0 | 75.52% | – | – | – |
| floreview | 0 | 2001 | – | 0 of 2001 | 0.00% | – |
| floreview-blur-s1.0 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-blur-s3.0 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-jpeg-qv12 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-jpeg-qv4 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-noise-a10 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-noise-a40 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-repost-720-qv9x1 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-repost-960-qv12x1 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-repost-960-qv7x4 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-resize-0.25-bicubic | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-resize-0.5-bicubic | 0 | 116 | – | 0 of 116 | 0.00% | – |
| floreview-sharpen-a1.0 | 0 | 116 | – | 0 of 116 | 0.00% | – |
| genimagepp | 1200 | 0 | 96.42% | – | – | – |
| genimagepp-blur-s1.0 | 57 | 0 | 92.98% | – | – | – |
| genimagepp-blur-s3.0 | 57 | 0 | 96.49% | – | – | – |
| genimagepp-jpeg-qv12 | 57 | 0 | 59.65% | – | – | – |
| genimagepp-jpeg-qv4 | 57 | 0 | 77.19% | – | – | – |
| genimagepp-noise-a10 | 57 | 0 | 82.46% | – | – | – |
| genimagepp-noise-a40 | 57 | 0 | 1.75% | – | – | – |
| genimagepp-repost-720-qv9x1 | 57 | 0 | 64.91% | – | – | – |
| genimagepp-repost-960-qv12x1 | 57 | 0 | 66.67% | – | – | – |
| genimagepp-repost-960-qv7x4 | 57 | 0 | 73.68% | – | – | – |
| genimagepp-resize-0.25-bicubic | 57 | 0 | 89.47% | – | – | – |
| genimagepp-resize-0.5-bicubic | 57 | 0 | 84.21% | – | – | – |
| genimagepp-sharpen-a1.0 | 57 | 0 | 92.98% | – | – | – |
| hdrplus | 0 | 1456 | – | 0 of 1456 | 0.00% | – |
| hdrplus-blur-s1.0 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-blur-s3.0 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-jpeg-qv12 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-jpeg-qv4 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-noise-a10 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-noise-a40 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-repost-720-qv9x1 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-repost-960-qv12x1 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-repost-960-qv7x4 | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-resize-0.25-bicubic | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-resize-0.5-bicubic | 0 | 84 | – | 0 of 84 | 0.00% | – |
| hdrplus-sharpen-a1.0 | 0 | 84 | – | 0 of 84 | 0.00% | – |

Worst dataset slice with at least 100 items of the class: recall 2.80% (arforensics-noise-a40); FPR 0.00% (floreview).
