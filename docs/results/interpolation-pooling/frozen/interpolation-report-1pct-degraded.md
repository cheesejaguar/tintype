4800 items scored by all 2 systems: 2400 AI, 2400 real; grouping columns found for 4800
Recall intervals: 95% bootstrap over 400 parent clusters (200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**alpha-0**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.918020 | 2 of 2400 | 0.08% | 0.26% | 59.00% (55.12%–62.85%) | 87.73% |
| 1% empirical FPR | 0.662264 | 24 of 2400 | 1.00% | 1.40% | 74.08% (70.66%–77.31%) | 42.80% |
| frozen cutoff | 0.384758 | 76 of 2400 | 3.17% | 3.82% | 81.50% (78.80%–84.31%) | 20.63% |

alpha-0 by family, at the frozen cutoff 0.3847578:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 90.53% | – | – | – |
| infinity | 492 | 0 | 92.89% | – | – | – |
| janus-pro | 204 | 0 | 92.16% | – | – | – |
| llamagen | 204 | 0 | 88.24% | – | – | – |
| open-magvit2 | 204 | 0 | 52.94% | – | – | – |
| rar | 492 | 0 | 77.24% | – | – | – |
| real:csafe | 0 | 1308 | – | 53 of 1308 | 4.05% | – |
| real:floreview | 0 | 612 | – | 18 of 612 | 2.94% | – |
| real:hdrplus | 0 | 480 | – | 5 of 480 | 1.04% | – |
| sd3 | 132 | 0 | 89.39% | – | – | – |
| switti | 204 | 0 | 89.22% | – | – | – |
| var | 204 | 0 | 50.98% | – | – | – |

Worst family slice with at least 100 items of the class: recall 50.98% (var); FPR 4.05% (real:csafe).

alpha-0 by source, at the frozen cutoff 0.3847578:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-blur-s1.0 | 167 | 0 | 95.81% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 68.26% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 82.04% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 91.62% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 18.56% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 73.65% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 68.26% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 79.64% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 89.82% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 94.61% | – | – | – |
| csafe-iphone14pro | 0 | 1308 | – | 53 of 1308 | 4.05% | – |
| floreview-apple | 0 | 84 | – | 1 of 84 | 1.19% | – |
| floreview-doogee | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-google | 0 | 36 | – | 4 of 36 | 11.11% | – |
| floreview-huawei | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-lenovo | 0 | 12 | – | 2 of 12 | 16.67% | – |
| floreview-lg | 0 | 48 | – | 1 of 48 | 2.08% | – |
| floreview-motorola | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-oneplus | 0 | 24 | – | 1 of 24 | 4.17% | – |
| floreview-samsung | 0 | 120 | – | 4 of 120 | 3.33% | – |
| floreview-sony | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-xiaomi | 0 | 96 | – | 5 of 96 | 5.21% | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 9.09% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 480 | – | 5 of 480 | 1.04% | – |

Worst source slice with at least 100 items of the class: recall 18.56% (arforensics-noise-a40); FPR 4.05% (csafe-iphone14pro).

**phase3-0.30**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.910286 | 2 of 2400 | 0.08% | 0.26% | 61.04% (57.12%–64.86%) | 88.09% |
| 1% empirical FPR | 0.682008 | 24 of 2400 | 1.00% | 1.40% | 76.54% (73.50%–79.49%) | 43.60% |
| frozen cutoff | 0.500000 | 73 of 2400 | 3.04% | 3.68% | 82.25% (79.65%–84.91%) | 21.45% |

phase3-0.30 by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 264 | 0 | 91.67% | – | – | – |
| infinity | 492 | 0 | 92.48% | – | – | – |
| janus-pro | 204 | 0 | 92.16% | – | – | – |
| llamagen | 204 | 0 | 89.22% | – | – | – |
| open-magvit2 | 204 | 0 | 52.94% | – | – | – |
| rar | 492 | 0 | 78.86% | – | – | – |
| real:csafe | 0 | 1308 | – | 51 of 1308 | 3.90% | – |
| real:floreview | 0 | 612 | – | 18 of 612 | 2.94% | – |
| real:hdrplus | 0 | 480 | – | 4 of 480 | 0.83% | – |
| sd3 | 132 | 0 | 88.64% | – | – | – |
| switti | 204 | 0 | 89.22% | – | – | – |
| var | 204 | 0 | 54.90% | – | – | – |

Worst family slice with at least 100 items of the class: recall 52.94% (open-magvit2); FPR 3.90% (real:csafe).

phase3-0.30 by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics-blur-s1.0 | 167 | 0 | 96.41% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 98.20% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 70.66% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 84.43% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 91.62% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 17.37% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 74.85% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 70.66% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 82.04% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 88.02% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 95.21% | – | – | – |
| csafe-iphone14pro | 0 | 1308 | – | 51 of 1308 | 3.90% | – |
| floreview-apple | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-doogee | 0 | 12 | – | 1 of 12 | 8.33% | – |
| floreview-google | 0 | 36 | – | 3 of 36 | 8.33% | – |
| floreview-huawei | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-lenovo | 0 | 12 | – | 2 of 12 | 16.67% | – |
| floreview-lg | 0 | 48 | – | 1 of 48 | 2.08% | – |
| floreview-motorola | 0 | 84 | – | 0 of 84 | 0.00% | – |
| floreview-oneplus | 0 | 24 | – | 1 of 24 | 4.17% | – |
| floreview-samsung | 0 | 120 | – | 5 of 120 | 4.17% | – |
| floreview-sony | 0 | 12 | – | 0 of 12 | 0.00% | – |
| floreview-xiaomi | 0 | 96 | – | 5 of 96 | 5.21% | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 3.03% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 480 | – | 4 of 480 | 0.83% | – |

Worst source slice with at least 100 items of the class: recall 17.37% (arforensics-noise-a40); FPR 4.17% (floreview-samsung).
