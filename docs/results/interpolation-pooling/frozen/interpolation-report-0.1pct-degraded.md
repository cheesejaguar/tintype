4800 items scored by all 2 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**alpha-0**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.918020 | 2 of 2400 | 0.08% | 0.26% | 59.00% (55.12%–62.85%) | 87.73% |
| 1% empirical FPR | 0.662264 | 24 of 2400 | 1.00% | 1.40% | 74.08% (70.66%–77.31%) | 42.80% |
| frozen cutoff | 0.699543 | 20 of 2400 | 0.83% | 1.21% | 72.08% (68.81%–75.35%) | 46.63% |

alpha-0 by family, at the frozen cutoff 0.6995432:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 75.38% | – | – | – |
| infinity | 492 | 0 | 88.01% | – | – | – |
| janus-pro | 204 | 0 | 82.84% | – | – | – |
| llamagen | 204 | 0 | 75.98% | – | – | – |
| open-magvit2 | 204 | 0 | 41.67% | – | – | – |
| rar | 492 | 0 | 67.48% | – | – | – |
| real:csafe | 0 | 1308 | – | 16 of 1308 | 1.22% | – |
| real:floreview | 0 | 612 | – | 3 of 612 | 0.49% | – |
| real:hdrplus | 0 | 480 | – | 1 of 480 | 0.21% | – |
| sd3 | 132 | 0 | 84.85% | – | – | – |
| switti | 204 | 0 | 84.31% | – | – | – |
| var | 204 | 0 | 35.78% | – | – | – |

Worst family slice with at least 100 items of the class: recall 35.78% (var); FPR 1.22% (real:csafe).

alpha-0 by source, at the frozen cutoff 0.6995432:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-blur-s1.0 | 167 | 0 | 93.41% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 92.22% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 52.69% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 70.66% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 84.43% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 7.78% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 59.88% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 52.69% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 66.47% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 80.84% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.01% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 91.62% | – | – | – |
| csafe-iphone14pro | 0 | 1308 | – | 16 of 1308 | 1.22% | – |
| floreview-apple | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-doogee | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-google | 0 | 36 | – | 0 of 36 | 0.00% | – |
| floreview-huawei | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-lenovo | 0 | 12 | – | 1 of 12 | 8.33% | – |
| floreview-lg | 0 | 48 | – | 0 of 48 | 0.00% | – |
| floreview-motorola | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-oneplus | 0 | 24 | – | 0 of 24 | 0.00% | – |
| floreview-samsung | 0 | 120 | – | 1 of 120 | 0.83% | – |
| floreview-sony | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-xiaomi | 0 | 96 | – | 1 of 96 | 1.04% | – |
| genimagepp-blur-s1.0 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 66.67% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 3.03% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 66.67% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 69.70% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 84.85% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 93.94% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 87.88% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 480 | – | 1 of 480 | 0.21% | – |

Worst source slice with at least 100 items of the class: recall 7.78% (arforensics-noise-a40); FPR 1.22% (csafe-iphone14pro).

**phase3-0.30**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.910286 | 2 of 2400 | 0.08% | 0.26% | 61.04% (57.12%–64.86%) | 88.09% |
| 1% empirical FPR | 0.682008 | 24 of 2400 | 1.00% | 1.40% | 76.54% (73.50%–79.49%) | 43.60% |
| frozen cutoff | 0.728662 | 20 of 2400 | 0.83% | 1.21% | 74.58% (71.24%–77.61%) | 47.48% |

phase3-0.30 by family, at the frozen cutoff 0.7286624:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 80.30% | – | – | – |
| infinity | 492 | 0 | 88.62% | – | – | – |
| janus-pro | 204 | 0 | 88.73% | – | – | – |
| llamagen | 204 | 0 | 79.90% | – | – | – |
| open-magvit2 | 204 | 0 | 44.12% | – | – | – |
| rar | 492 | 0 | 68.50% | – | – | – |
| real:csafe | 0 | 1308 | – | 17 of 1308 | 1.30% | – |
| real:floreview | 0 | 612 | – | 2 of 612 | 0.33% | – |
| real:hdrplus | 0 | 480 | – | 1 of 480 | 0.21% | – |
| sd3 | 132 | 0 | 87.88% | – | – | – |
| switti | 204 | 0 | 85.78% | – | – | – |
| var | 204 | 0 | 39.22% | – | – | – |

Worst family slice with at least 100 items of the class: recall 39.22% (var); FPR 1.30% (real:csafe).

phase3-0.30 by source, at the frozen cutoff 0.7286624:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-blur-s1.0 | 167 | 0 | 94.61% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 93.41% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 59.88% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 74.85% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 86.23% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 7.78% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 62.28% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 59.28% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 68.26% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 79.04% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.01% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 92.81% | – | – | – |
| csafe-iphone14pro | 0 | 1308 | – | 17 of 1308 | 1.30% | – |
| floreview-apple | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-doogee | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-google | 0 | 36 | – | 0 of 36 | 0.00% | – |
| floreview-huawei | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-lenovo | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-lg | 0 | 48 | – | 0 of 48 | 0.00% | – |
| floreview-motorola | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-oneplus | 0 | 24 | – | 0 of 24 | 0.00% | – |
| floreview-samsung | 0 | 120 | – | 1 of 120 | 0.83% | – |
| floreview-sony | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-xiaomi | 0 | 96 | – | 1 of 96 | 1.04% | – |
| genimagepp-blur-s1.0 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 72.73% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 84.85% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 3.03% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 81.82% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 90.91% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 480 | – | 1 of 480 | 0.21% | – |

Worst source slice with at least 100 items of the class: recall 7.78% (arforensics-noise-a40); FPR 1.30% (csafe-iphone14pro).
