```
$ tintype report --operating-points --system control=test.control.tsv@0.5 --system control-seed2=test.control-seed2.tsv@0.5 --system family=test.family.tsv@0.5 --system source=test.source.tsv@0.5 --fpr 0.001,0.01 --prevalence 0.01 --slice-by transform,family,source --cluster-by parent --groups test-groups.tsv --min-slice 100
```

Run on the unpacked archive (`gzip -d`), from its directory. Each score file holds the probability under its own 1% calibration, so `@0.5` is each model's cutoff frozen on `calibrate` for 1%.

19326 items scored by all 4 systems: 9600 AI, 9726 real; grouping columns found for 19326
Recall intervals: 95% bootstrap over 14526 parent clusters (7200 of them hold AI items).
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**control**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.836982 | 9 of 9726 | 0.09% | 0.16% | 85.10% (83.93%–86.31%) | 90.28% |
| 1% empirical FPR | 0.550915 | 97 of 9726 | 1.00% | 1.18% | 91.49% (90.63%–92.37%) | 48.10% |
| frozen cutoff | 0.500000 | 128 of 9726 | 1.32% | 1.52% | 92.23% (91.41%–93.03%) | 41.45% |

control by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 96.50% | 1 of 200 | 0.50% | 0.997 |
| blur-s3.0 | 200 | 200 | 98.00% | 5 of 200 | 2.50% | 0.998 |
| jpeg-qv12 | 200 | 200 | 73.50% | 1 of 200 | 0.50% | 0.976 |
| jpeg-qv4 | 200 | 200 | 85.00% | 2 of 200 | 1.00% | 0.988 |
| noise-a10 | 200 | 200 | 92.50% | 31 of 200 | 15.50% | 0.951 |
| noise-a40 | 200 | 200 | 17.50% | 14 of 200 | 7.00% | 0.646 |
| none | 7200 | 7326 | 95.61% | 55 of 7326 | 0.75% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 78.50% | 5 of 200 | 2.50% | 0.964 |
| repost-960-qv12x1 | 200 | 200 | 74.00% | 3 of 200 | 1.50% | 0.951 |
| repost-960-qv7x4 | 200 | 200 | 83.00% | 4 of 200 | 2.00% | 0.977 |
| resize-0.25-bicubic | 200 | 200 | 92.00% | 1 of 200 | 0.50% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 98.00% | 1 of 200 | 0.50% | 0.998 |
| sharpen-a1.0 | 200 | 200 | 96.50% | 5 of 200 | 2.50% | 0.987 |

Worst transform slice with at least 100 items of the class: recall 17.50% (noise-a40); FPR 15.50% (noise-a10).

control by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 97.27% | – | – | – |
| infinity | 1992 | 0 | 98.24% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 95.65% | – | – | – |
| open-magvit2 | 804 | 0 | 67.66% | – | – | – |
| rar | 1992 | 0 | 94.03% | – | – | – |
| real:csafe | 0 | 5304 | – | 84 of 5304 | 1.58% | – |
| real:floreview | 0 | 2486 | – | 33 of 2486 | 1.33% | – |
| real:hdrplus | 0 | 1936 | – | 11 of 1936 | 0.57% | – |
| sd3 | 532 | 0 | 96.99% | – | – | – |
| switti | 804 | 0 | 97.14% | – | – | – |
| var | 804 | 0 | 73.51% | – | – | – |

Worst family slice with at least 100 items of the class: recall 67.66% (open-magvit2); FPR 1.58% (real:csafe).

control by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 94.82% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 95.81% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 70.06% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 82.63% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 91.62% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 19.16% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 74.85% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 70.06% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 80.24% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 91.02% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 95.81% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 84 of 5304 | 1.58% | – |
| floreview-apple | 0 | 333 | – | 1 of 333 | 0.30% | – |
| floreview-doogee | 0 | 54 | – | 4 of 54 | 7.41% | – |
| floreview-google | 0 | 161 | – | 3 of 161 | 1.86% | – |
| floreview-huawei | 0 | 333 | – | 1 of 333 | 0.30% | – |
| floreview-lenovo | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-lg | 0 | 174 | – | 2 of 174 | 1.15% | – |
| floreview-motorola | 0 | 332 | – | 2 of 332 | 0.60% | – |
| floreview-oneplus | 0 | 108 | – | 2 of 108 | 1.85% | – |
| floreview-samsung | 0 | 497 | – | 5 of 497 | 1.01% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 11 of 388 | 2.84% | – |
| genimagepp | 1200 | 0 | 99.58% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 90.91% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 9.09% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 11 of 1936 | 0.57% | – |

Worst source slice with at least 100 items of the class: recall 19.16% (arforensics-noise-a40); FPR 2.84% (floreview-xiaomi).

**control-seed2**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.869489 | 9 of 9726 | 0.09% | 0.16% | 82.90% (81.65%–84.15%) | 90.05% |
| 1% empirical FPR | 0.584033 | 97 of 9726 | 1.00% | 1.18% | 90.61% (89.69%–91.51%) | 47.86% |
| frozen cutoff | 0.500000 | 159 of 9726 | 1.63% | 1.86% | 91.90% (91.03%–92.71%) | 36.22% |

control-seed2 by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 96.00% | 2 of 200 | 1.00% | 0.996 |
| blur-s3.0 | 200 | 200 | 98.00% | 11 of 200 | 5.50% | 0.997 |
| jpeg-qv12 | 200 | 200 | 75.50% | 1 of 200 | 0.50% | 0.976 |
| jpeg-qv4 | 200 | 200 | 83.50% | 2 of 200 | 1.00% | 0.986 |
| noise-a10 | 200 | 200 | 91.50% | 30 of 200 | 15.00% | 0.946 |
| noise-a40 | 200 | 200 | 10.00% | 12 of 200 | 6.00% | 0.627 |
| none | 7200 | 7326 | 95.38% | 81 of 7326 | 1.11% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 79.50% | 5 of 200 | 2.50% | 0.962 |
| repost-960-qv12x1 | 200 | 200 | 75.50% | 3 of 200 | 1.50% | 0.950 |
| repost-960-qv7x4 | 200 | 200 | 83.00% | 1 of 200 | 0.50% | 0.975 |
| resize-0.25-bicubic | 200 | 200 | 91.00% | 2 of 200 | 1.00% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 98.00% | 4 of 200 | 2.00% | 0.997 |
| sharpen-a1.0 | 200 | 200 | 96.00% | 5 of 200 | 2.50% | 0.986 |

Worst transform slice with at least 100 items of the class: recall 10.00% (noise-a40); FPR 15.00% (noise-a10).

control-seed2 by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 97.74% | – | – | – |
| infinity | 1992 | 0 | 98.04% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 94.40% | – | – | – |
| open-magvit2 | 804 | 0 | 65.42% | – | – | – |
| rar | 1992 | 0 | 93.93% | – | – | – |
| real:csafe | 0 | 5304 | – | 118 of 5304 | 2.22% | – |
| real:floreview | 0 | 2486 | – | 29 of 2486 | 1.17% | – |
| real:hdrplus | 0 | 1936 | – | 12 of 1936 | 0.62% | – |
| sd3 | 532 | 0 | 96.99% | – | – | – |
| switti | 804 | 0 | 97.01% | – | – | – |
| var | 804 | 0 | 73.26% | – | – | – |

Worst family slice with at least 100 items of the class: recall 65.42% (open-magvit2); FPR 2.22% (real:csafe).

control-seed2 by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 94.50% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 95.21% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 71.26% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 80.84% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 89.82% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 11.38% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 76.05% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 71.26% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 80.24% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 89.22% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 95.21% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 118 of 5304 | 2.22% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 3 of 54 | 5.56% | – |
| floreview-google | 0 | 161 | – | 2 of 161 | 1.24% | – |
| floreview-huawei | 0 | 333 | – | 2 of 333 | 0.60% | – |
| floreview-lenovo | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-lg | 0 | 174 | – | 1 of 174 | 0.57% | – |
| floreview-motorola | 0 | 332 | – | 1 of 332 | 0.30% | – |
| floreview-oneplus | 0 | 108 | – | 2 of 108 | 1.85% | – |
| floreview-samsung | 0 | 497 | – | 5 of 497 | 1.01% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 11 of 388 | 2.84% | – |
| genimagepp | 1200 | 0 | 99.75% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 3.03% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 12 of 1936 | 0.62% | – |

Worst source slice with at least 100 items of the class: recall 11.38% (arforensics-noise-a40); FPR 2.84% (floreview-xiaomi).

**family**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.841346 | 9 of 9726 | 0.09% | 0.16% | 86.66% (85.48%–87.72%) | 90.44% |
| 1% empirical FPR | 0.526856 | 97 of 9726 | 1.00% | 1.18% | 93.22% (92.43%–93.93%) | 48.56% |
| frozen cutoff | 0.500000 | 119 of 9726 | 1.22% | 1.42% | 93.55% (92.76%–94.24%) | 43.58% |

family by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 97.00% | 2 of 200 | 1.00% | 0.998 |
| blur-s3.0 | 200 | 200 | 99.00% | 14 of 200 | 7.00% | 0.998 |
| jpeg-qv12 | 200 | 200 | 80.00% | 1 of 200 | 0.50% | 0.982 |
| jpeg-qv4 | 200 | 200 | 88.50% | 2 of 200 | 1.00% | 0.989 |
| noise-a10 | 200 | 200 | 92.00% | 19 of 200 | 9.50% | 0.959 |
| noise-a40 | 200 | 200 | 18.00% | 13 of 200 | 6.50% | 0.676 |
| none | 7200 | 7326 | 96.56% | 48 of 7326 | 0.66% | 0.997 |
| repost-720-qv9x1 | 200 | 200 | 83.50% | 5 of 200 | 2.50% | 0.969 |
| repost-960-qv12x1 | 200 | 200 | 79.50% | 3 of 200 | 1.50% | 0.960 |
| repost-960-qv7x4 | 200 | 200 | 86.50% | 5 of 200 | 2.50% | 0.979 |
| resize-0.25-bicubic | 200 | 200 | 96.00% | 1 of 200 | 0.50% | 0.998 |
| resize-0.5-bicubic | 200 | 200 | 98.50% | 3 of 200 | 1.50% | 0.999 |
| sharpen-a1.0 | 200 | 200 | 96.00% | 3 of 200 | 1.50% | 0.989 |

Worst transform slice with at least 100 items of the class: recall 18.00% (noise-a40); FPR 9.50% (noise-a10).

family by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 97.65% | – | – | – |
| infinity | 1992 | 0 | 98.19% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 96.89% | – | – | – |
| open-magvit2 | 804 | 0 | 73.13% | – | – | – |
| rar | 1992 | 0 | 95.73% | – | – | – |
| real:csafe | 0 | 5304 | – | 85 of 5304 | 1.60% | – |
| real:floreview | 0 | 2486 | – | 25 of 2486 | 1.01% | – |
| real:hdrplus | 0 | 1936 | – | 9 of 1936 | 0.46% | – |
| sd3 | 532 | 0 | 97.37% | – | – | – |
| switti | 804 | 0 | 97.26% | – | – | – |
| var | 804 | 0 | 77.61% | – | – | – |

Worst family slice with at least 100 items of the class: recall 73.13% (open-magvit2); FPR 1.60% (real:csafe).

family by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 95.90% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 96.41% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 98.80% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 76.65% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 86.23% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 91.02% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 20.36% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 80.84% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 76.65% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 84.43% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 95.21% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 98.20% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 95.21% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 85 of 5304 | 1.60% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 4 of 54 | 7.41% | – |
| floreview-google | 0 | 161 | – | 3 of 161 | 1.86% | – |
| floreview-huawei | 0 | 333 | – | 1 of 333 | 0.30% | – |
| floreview-lenovo | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-lg | 0 | 174 | – | 1 of 174 | 0.57% | – |
| floreview-motorola | 0 | 332 | – | 0 of 332 | 0.00% | – |
| floreview-oneplus | 0 | 108 | – | 2 of 108 | 1.85% | – |
| floreview-samsung | 0 | 497 | – | 3 of 497 | 0.60% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 9 of 388 | 2.32% | – |
| genimagepp | 1200 | 0 | 99.83% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 6.06% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 9 of 1936 | 0.46% | – |

Worst source slice with at least 100 items of the class: recall 20.36% (arforensics-noise-a40); FPR 2.32% (floreview-xiaomi).

**source**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.764051 | 9 of 9726 | 0.09% | 0.16% | 85.92% (84.71%–87.05%) | 90.36% |
| 1% empirical FPR | 0.490750 | 97 of 9726 | 1.00% | 1.18% | 91.45% (90.57%–92.28%) | 48.08% |
| frozen cutoff | 0.500000 | 95 of 9726 | 0.98% | 1.16% | 91.28% (90.40%–92.13%) | 48.56% |

source by transform, at the frozen cutoff 0.5:

| transform | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| blur-s1.0 | 200 | 200 | 98.00% | 1 of 200 | 0.50% | 0.998 |
| blur-s3.0 | 200 | 200 | 98.50% | 8 of 200 | 4.00% | 0.997 |
| jpeg-qv12 | 200 | 200 | 70.50% | 1 of 200 | 0.50% | 0.973 |
| jpeg-qv4 | 200 | 200 | 82.00% | 1 of 200 | 0.50% | 0.988 |
| noise-a10 | 200 | 200 | 91.00% | 19 of 200 | 9.50% | 0.950 |
| noise-a40 | 200 | 200 | 10.50% | 10 of 200 | 5.00% | 0.624 |
| none | 7200 | 7326 | 94.93% | 42 of 7326 | 0.57% | 0.994 |
| repost-720-qv9x1 | 200 | 200 | 77.00% | 5 of 200 | 2.50% | 0.958 |
| repost-960-qv12x1 | 200 | 200 | 71.00% | 3 of 200 | 1.50% | 0.945 |
| repost-960-qv7x4 | 200 | 200 | 80.50% | 2 of 200 | 1.00% | 0.972 |
| resize-0.25-bicubic | 200 | 200 | 91.50% | 1 of 200 | 0.50% | 0.996 |
| resize-0.5-bicubic | 200 | 200 | 98.00% | 1 of 200 | 0.50% | 0.999 |
| sharpen-a1.0 | 200 | 200 | 95.50% | 1 of 200 | 0.50% | 0.987 |

Worst transform slice with at least 100 items of the class: recall 10.50% (noise-a40); FPR 9.50% (noise-a10).

source by family, at the frozen cutoff 0.5:

| family | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| flux | 1064 | 0 | 96.33% | – | – | – |
| infinity | 1992 | 0 | 97.79% | – | – | – |
| janus-pro | 804 | 0 | 98.01% | – | – | – |
| llamagen | 804 | 0 | 95.15% | – | – | – |
| open-magvit2 | 804 | 0 | 63.81% | – | – | – |
| rar | 1992 | 0 | 93.57% | – | – | – |
| real:csafe | 0 | 5304 | – | 61 of 5304 | 1.15% | – |
| real:floreview | 0 | 2486 | – | 26 of 2486 | 1.05% | – |
| real:hdrplus | 0 | 1936 | – | 8 of 1936 | 0.41% | – |
| sd3 | 532 | 0 | 96.05% | – | – | – |
| switti | 804 | 0 | 97.01% | – | – | – |
| var | 804 | 0 | 70.77% | – | – | – |

Worst family slice with at least 100 items of the class: recall 63.81% (open-magvit2); FPR 1.15% (real:csafe).

source by source, at the frozen cutoff 0.5:

| source | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| arforensics | 6000 | 0 | 94.07% | – | – | – |
| arforensics-blur-s1.0 | 167 | 0 | 97.60% | – | – | – |
| arforensics-blur-s3.0 | 167 | 0 | 98.20% | – | – | – |
| arforensics-jpeg-qv12 | 167 | 0 | 67.66% | – | – | – |
| arforensics-jpeg-qv4 | 167 | 0 | 80.84% | – | – | – |
| arforensics-noise-a10 | 167 | 0 | 89.82% | – | – | – |
| arforensics-noise-a40 | 167 | 0 | 11.98% | – | – | – |
| arforensics-repost-720-qv9x1 | 167 | 0 | 73.65% | – | – | – |
| arforensics-repost-960-qv12x1 | 167 | 0 | 67.66% | – | – | – |
| arforensics-repost-960-qv7x4 | 167 | 0 | 77.84% | – | – | – |
| arforensics-resize-0.25-bicubic | 167 | 0 | 90.42% | – | – | – |
| arforensics-resize-0.5-bicubic | 167 | 0 | 97.60% | – | – | – |
| arforensics-sharpen-a1.0 | 167 | 0 | 94.61% | – | – | – |
| csafe-iphone14pro | 0 | 5304 | – | 61 of 5304 | 1.15% | – |
| floreview-apple | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-doogee | 0 | 54 | – | 5 of 54 | 9.26% | – |
| floreview-google | 0 | 161 | – | 2 of 161 | 1.24% | – |
| floreview-huawei | 0 | 333 | – | 0 of 333 | 0.00% | – |
| floreview-lenovo | 0 | 54 | – | 2 of 54 | 3.70% | – |
| floreview-lg | 0 | 174 | – | 0 of 174 | 0.00% | – |
| floreview-motorola | 0 | 332 | – | 1 of 332 | 0.30% | – |
| floreview-oneplus | 0 | 108 | – | 2 of 108 | 1.85% | – |
| floreview-samsung | 0 | 497 | – | 5 of 497 | 1.01% | – |
| floreview-sony | 0 | 52 | – | 0 of 52 | 0.00% | – |
| floreview-xiaomi | 0 | 388 | – | 9 of 388 | 2.32% | – |
| genimagepp | 1200 | 0 | 99.25% | – | – | – |
| genimagepp-blur-s1.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-blur-s3.0 | 33 | 0 | 100.00% | – | – | – |
| genimagepp-jpeg-qv12 | 33 | 0 | 84.85% | – | – | – |
| genimagepp-jpeg-qv4 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-noise-a10 | 33 | 0 | 96.97% | – | – | – |
| genimagepp-noise-a40 | 33 | 0 | 3.03% | – | – | – |
| genimagepp-repost-720-qv9x1 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-repost-960-qv12x1 | 33 | 0 | 87.88% | – | – | – |
| genimagepp-repost-960-qv7x4 | 33 | 0 | 93.94% | – | – | – |
| genimagepp-resize-0.25-bicubic | 33 | 0 | 96.97% | – | – | – |
| genimagepp-resize-0.5-bicubic | 33 | 0 | 100.00% | – | – | – |
| genimagepp-sharpen-a1.0 | 33 | 0 | 100.00% | – | – | – |
| hdrplus | 0 | 1936 | – | 8 of 1936 | 0.41% | – |

Worst source slice with at least 100 items of the class: recall 11.98% (arforensics-noise-a40); FPR 2.32% (floreview-xiaomi).
