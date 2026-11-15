4800 items scored by all 1 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**A-strict-degraded**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.647138 | 2 of 2400 | 0.08% | 0.26% | 62.88% (59.10%–66.67%) | 88.40% |
| 1% empirical FPR | 0.341946 | 24 of 2400 | 1.00% | 1.40% | 77.62% (74.52%–80.74%) | 43.95% |
| frozen cutoff | 0.500000 | 6 of 2400 | 0.25% | 0.49% | 69.33% (65.87%–72.90%) | 73.69% |

A-strict-degraded, review accounting at the frozen cutoff 0.5. Verdicts are unchanged; an item sent to review gets no automatic verdict, so a reviewed AI item counts as a miss and a reviewed real item is not a false alarm.

| Review band | Coverage | Real items reviewed | AI items reviewed | Reviewed for the margin | Real items flagged without review | FPR | FPR upper bound | Recall, reviewed AI items as misses | Recall without the band | Precision at prevalence | Share of a stream reviewed at prevalence |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 100.00% | 0 of 2400 (0.00%) | 0 of 2400 (0.00%) | 0 of 0 | 6 of 2400 | 0.25% | 0.49% | 69.33% (65.87%–72.90%) | 69.33% | 73.69% | 0.00% |
| margin 1.5 | 80.35% | 90 of 2400 (3.75%) | 853 of 2400 (35.54%) | 943 of 943 | 0 of 2400 | 0.00% | 0.12% | 51.33% (47.06%–55.53%) | 69.33% | 100.00% | 4.07% |

A-strict-degraded by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
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
| csafe-blur-s1.0 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-blur-s3.0 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-jpeg-qv12 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-jpeg-qv4 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-noise-a10 | 0 | 109 | – | 1 of 109 | 0.92% | – |
| csafe-noise-a40 | 0 | 109 | – | 5 of 109 | 4.59% | – |
| csafe-repost-720-qv9x1 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-repost-960-qv12x1 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-repost-960-qv7x4 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-resize-0.25-bicubic | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-resize-0.5-bicubic | 0 | 109 | – | 0 of 109 | 0.00% | – |
| csafe-sharpen-a1.0 | 0 | 109 | – | 0 of 109 | 0.00% | – |
| floreview-blur-s1.0 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-blur-s3.0 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-jpeg-qv12 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-jpeg-qv4 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-noise-a10 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-noise-a40 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-repost-720-qv9x1 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-repost-960-qv12x1 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-repost-960-qv7x4 | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-resize-0.25-bicubic | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-resize-0.5-bicubic | 0 | 51 | – | 0 of 51 | 0.00% | – |
| floreview-sharpen-a1.0 | 0 | 51 | – | 0 of 51 | 0.00% | – |
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

Worst dataset slice with at least 100 items of the class: recall 5.99% (arforensics-noise-a40); FPR 4.59% (csafe-noise-a40).
