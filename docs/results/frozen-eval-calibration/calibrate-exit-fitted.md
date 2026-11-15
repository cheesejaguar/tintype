runs/frozen/ws2/bound-max.safetensors: stored schedule, order [2, 0, 1, 3, 4, 5], margins after N views (raw logits) 1: 6, 2: 3, 3: 2, 4: 1.5, 5: 1
fitted on 3759 items, pooled precision and recall within 0.001 of every view at photo, strict, sensitive: margins after N views 1: 6.5, 2: 4, 3: 4, 4: 1.75, 5: 1
  on the fit items, photo (threshold 1.636): 1.73 views per item; precision 0.9953 against 0.9953, recall 0.9124 against 0.9124
  on the fit items, strict (threshold 3.504): 1.76 views per item; precision 0.9984 against 0.9984, recall 0.8705 against 0.8714
  on the fit items, sensitive (threshold -0.494): 1.96 views per item; precision 0.9769 against 0.9779, recall 0.9471 against 0.9471

## photo, threshold 1.636 (raw logit), stored schedule

12457 items (6600 AI, 5857 real); 1.66 views looked at per item of 6
decided after 1..6 views: 7398 / 3611 / 616 / 276 / 223 / 333
verdicts that differ from every view: 45 (20 more flagged, 25 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 28 of 5857 | 0.48% | 86.17% | 0.9951 |
| early exit | 25 of 5857 | 0.43% | 86.14% | 0.9956 |
| difference | -3 | -0.05 points | -0.03 points | +0.0005 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:floreview | 3393 | 0.41% | 0.29% | 10 of 3393 | within |
| real:hdrplus | 2464 | 0.57% | 0.61% | 15 of 2464 | within |

Mean over real sources with at least 100 items: 0.49% with every view, 0.45% under early exit; the calibration target is 1.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 3000 | 89.63% | 89.63% |
| arforensics-blur-s1.0 | 143 | 90.91% | 90.21% |
| arforensics-blur-s3.0 | 143 | 92.31% | 92.31% |
| arforensics-jpeg-qv12 | 143 | 65.03% | 65.03% |
| arforensics-jpeg-qv4 | 143 | 70.63% | 71.33% |
| arforensics-noise-a10 | 143 | 79.72% | 79.72% |
| arforensics-noise-a40 | 143 | 8.39% | 9.09% |
| arforensics-repost-720-qv9x1 | 143 | 66.43% | 66.43% |
| arforensics-repost-960-qv12x1 | 143 | 65.03% | 65.03% |
| arforensics-repost-960-qv7x4 | 143 | 67.13% | 67.83% |
| arforensics-resize-0.25-bicubic | 143 | 79.72% | 79.72% |
| arforensics-resize-0.5-bicubic | 143 | 94.41% | 94.41% |
| arforensics-sharpen-a1.0 | 143 | 83.22% | 83.22% |
| genimagepp | 1200 | 99.50% | 98.67% |
| genimagepp-blur-s1.0 | 57 | 98.25% | 96.49% |
| genimagepp-blur-s3.0 | 57 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 57 | 80.70% | 80.70% |
| genimagepp-jpeg-qv4 | 57 | 91.23% | 89.47% |
| genimagepp-noise-a10 | 57 | 96.49% | 96.49% |
| genimagepp-noise-a40 | 57 | 5.26% | 15.79% |
| genimagepp-repost-720-qv9x1 | 57 | 85.96% | 87.72% |
| genimagepp-repost-960-qv12x1 | 57 | 78.95% | 80.70% |
| genimagepp-repost-960-qv7x4 | 57 | 84.21% | 85.96% |
| genimagepp-resize-0.25-bicubic | 57 | 91.23% | 91.23% |
| genimagepp-resize-0.5-bicubic | 57 | 89.47% | 87.72% |
| genimagepp-sharpen-a1.0 | 57 | 98.25% | 98.25% |

## photo, threshold 1.636 (raw logit), fitted schedule

12457 items (6600 AI, 5857 real); 1.87 views looked at per item of 6
decided after 1..6 views: 6720 / 3639 / 247 / 1083 / 383 / 385
verdicts that differ from every view: 19 (8 more flagged, 11 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 28 of 5857 | 0.48% | 86.17% | 0.9951 |
| early exit | 26 of 5857 | 0.44% | 86.15% | 0.9954 |
| difference | -2 | -0.03 points | -0.02 points | +0.0003 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:floreview | 3393 | 0.41% | 0.35% | 12 of 3393 | within |
| real:hdrplus | 2464 | 0.57% | 0.57% | 14 of 2464 | within |

Mean over real sources with at least 100 items: 0.49% with every view, 0.46% under early exit; the calibration target is 1.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 3000 | 89.63% | 89.63% |
| arforensics-blur-s1.0 | 143 | 90.91% | 90.21% |
| arforensics-blur-s3.0 | 143 | 92.31% | 92.31% |
| arforensics-jpeg-qv12 | 143 | 65.03% | 65.03% |
| arforensics-jpeg-qv4 | 143 | 70.63% | 70.63% |
| arforensics-noise-a10 | 143 | 79.72% | 79.72% |
| arforensics-noise-a40 | 143 | 8.39% | 8.39% |
| arforensics-repost-720-qv9x1 | 143 | 66.43% | 66.43% |
| arforensics-repost-960-qv12x1 | 143 | 65.03% | 65.03% |
| arforensics-repost-960-qv7x4 | 143 | 67.13% | 67.13% |
| arforensics-resize-0.25-bicubic | 143 | 79.72% | 79.72% |
| arforensics-resize-0.5-bicubic | 143 | 94.41% | 94.41% |
| arforensics-sharpen-a1.0 | 143 | 83.22% | 83.22% |
| genimagepp | 1200 | 99.50% | 99.08% |
| genimagepp-blur-s1.0 | 57 | 98.25% | 100.00% |
| genimagepp-blur-s3.0 | 57 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 57 | 80.70% | 80.70% |
| genimagepp-jpeg-qv4 | 57 | 91.23% | 91.23% |
| genimagepp-noise-a10 | 57 | 96.49% | 96.49% |
| genimagepp-noise-a40 | 57 | 5.26% | 12.28% |
| genimagepp-repost-720-qv9x1 | 57 | 85.96% | 85.96% |
| genimagepp-repost-960-qv12x1 | 57 | 78.95% | 78.95% |
| genimagepp-repost-960-qv7x4 | 57 | 84.21% | 85.96% |
| genimagepp-resize-0.25-bicubic | 57 | 91.23% | 91.23% |
| genimagepp-resize-0.5-bicubic | 57 | 89.47% | 87.72% |
| genimagepp-sharpen-a1.0 | 57 | 98.25% | 98.25% |

## strict, threshold 3.504 (raw logit), stored schedule

12457 items (6600 AI, 5857 real); 1.66 views looked at per item of 6
decided after 1..6 views: 7747 / 3144 / 639 / 276 / 228 / 423
verdicts that differ from every view: 63 (33 more flagged, 30 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 0 of 5857 | 0.00% | 79.80% | 1.0000 |
| early exit | 0 of 5857 | 0.00% | 79.85% | 1.0000 |
| difference | +0 | +0.00 points | +0.05 points | +0.0000 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:floreview | 3393 | 0.00% | 0.00% | 0 of 3393 | within |
| real:hdrplus | 2464 | 0.00% | 0.00% | 0 of 2464 | within |

Mean over real sources with at least 100 items: 0.00% with every view, 0.00% under early exit; the calibration target is 0.10% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 3000 | 84.83% | 84.80% |
| arforensics-blur-s1.0 | 143 | 85.31% | 85.31% |
| arforensics-blur-s3.0 | 143 | 83.92% | 83.92% |
| arforensics-jpeg-qv12 | 143 | 46.85% | 47.55% |
| arforensics-jpeg-qv4 | 143 | 62.24% | 62.24% |
| arforensics-noise-a10 | 143 | 66.43% | 68.53% |
| arforensics-noise-a40 | 143 | 2.10% | 2.80% |
| arforensics-repost-720-qv9x1 | 143 | 53.15% | 55.24% |
| arforensics-repost-960-qv12x1 | 143 | 46.85% | 47.55% |
| arforensics-repost-960-qv7x4 | 143 | 53.15% | 54.55% |
| arforensics-resize-0.25-bicubic | 143 | 72.73% | 72.73% |
| arforensics-resize-0.5-bicubic | 143 | 88.81% | 88.81% |
| arforensics-sharpen-a1.0 | 143 | 75.52% | 75.52% |
| genimagepp | 1200 | 97.42% | 95.75% |
| genimagepp-blur-s1.0 | 57 | 96.49% | 92.98% |
| genimagepp-blur-s3.0 | 57 | 96.49% | 96.49% |
| genimagepp-jpeg-qv12 | 57 | 57.89% | 61.40% |
| genimagepp-jpeg-qv4 | 57 | 75.44% | 77.19% |
| genimagepp-noise-a10 | 57 | 82.46% | 85.96% |
| genimagepp-noise-a40 | 57 | 0.00% | 3.51% |
| genimagepp-repost-720-qv9x1 | 57 | 64.91% | 70.18% |
| genimagepp-repost-960-qv12x1 | 57 | 61.40% | 66.67% |
| genimagepp-repost-960-qv7x4 | 57 | 71.93% | 77.19% |
| genimagepp-resize-0.25-bicubic | 57 | 89.47% | 89.47% |
| genimagepp-resize-0.5-bicubic | 57 | 84.21% | 84.21% |
| genimagepp-sharpen-a1.0 | 57 | 94.74% | 92.98% |

## strict, threshold 3.504 (raw logit), fitted schedule

12457 items (6600 AI, 5857 real); 1.86 views looked at per item of 6
decided after 1..6 views: 7149 / 3126 / 240 / 1087 / 375 / 480
verdicts that differ from every view: 30 (13 more flagged, 17 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 0 of 5857 | 0.00% | 79.80% | 1.0000 |
| early exit | 0 of 5857 | 0.00% | 79.74% | 1.0000 |
| difference | +0 | +0.00 points | -0.06 points | +0.0000 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:floreview | 3393 | 0.00% | 0.00% | 0 of 3393 | within |
| real:hdrplus | 2464 | 0.00% | 0.00% | 0 of 2464 | within |

Mean over real sources with at least 100 items: 0.00% with every view, 0.00% under early exit; the calibration target is 0.10% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 3000 | 84.83% | 84.83% |
| arforensics-blur-s1.0 | 143 | 85.31% | 85.31% |
| arforensics-blur-s3.0 | 143 | 83.92% | 83.92% |
| arforensics-jpeg-qv12 | 143 | 46.85% | 46.85% |
| arforensics-jpeg-qv4 | 143 | 62.24% | 62.24% |
| arforensics-noise-a10 | 143 | 66.43% | 67.83% |
| arforensics-noise-a40 | 143 | 2.10% | 2.80% |
| arforensics-repost-720-qv9x1 | 143 | 53.15% | 53.15% |
| arforensics-repost-960-qv12x1 | 143 | 46.85% | 46.85% |
| arforensics-repost-960-qv7x4 | 143 | 53.15% | 53.85% |
| arforensics-resize-0.25-bicubic | 143 | 72.73% | 72.73% |
| arforensics-resize-0.5-bicubic | 143 | 88.81% | 88.81% |
| arforensics-sharpen-a1.0 | 143 | 75.52% | 75.52% |
| genimagepp | 1200 | 97.42% | 96.42% |
| genimagepp-blur-s1.0 | 57 | 96.49% | 92.98% |
| genimagepp-blur-s3.0 | 57 | 96.49% | 96.49% |
| genimagepp-jpeg-qv12 | 57 | 57.89% | 59.65% |
| genimagepp-jpeg-qv4 | 57 | 75.44% | 77.19% |
| genimagepp-noise-a10 | 57 | 82.46% | 82.46% |
| genimagepp-noise-a40 | 57 | 0.00% | 1.75% |
| genimagepp-repost-720-qv9x1 | 57 | 64.91% | 64.91% |
| genimagepp-repost-960-qv12x1 | 57 | 61.40% | 66.67% |
| genimagepp-repost-960-qv7x4 | 57 | 71.93% | 73.68% |
| genimagepp-resize-0.25-bicubic | 57 | 89.47% | 89.47% |
| genimagepp-resize-0.5-bicubic | 57 | 84.21% | 84.21% |
| genimagepp-sharpen-a1.0 | 57 | 94.74% | 92.98% |

## sensitive, threshold -0.494 (raw logit), stored schedule

12457 items (6600 AI, 5857 real); 1.86 views looked at per item of 6
decided after 1..6 views: 6107 / 4450 / 726 / 386 / 287 / 501
verdicts that differ from every view: 77 (35 more flagged, 42 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 220 of 5857 | 3.76% | 91.14% | 0.9647 |
| early exit | 199 of 5857 | 3.40% | 91.35% | 0.9680 |
| difference | -21 | -0.36 points | +0.21 points | +0.0033 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:floreview | 3393 | 4.13% | 3.63% | 123 of 3393 | within |
| real:hdrplus | 2464 | 3.25% | 3.08% | 76 of 2464 | within |

Mean over real sources with at least 100 items: 3.69% with every view, 3.35% under early exit; the calibration target is 5.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 3000 | 93.80% | 93.80% |
| arforensics-blur-s1.0 | 143 | 94.41% | 94.41% |
| arforensics-blur-s3.0 | 143 | 97.20% | 97.20% |
| arforensics-jpeg-qv12 | 143 | 78.32% | 79.02% |
| arforensics-jpeg-qv4 | 143 | 79.72% | 79.72% |
| arforensics-noise-a10 | 143 | 87.41% | 87.41% |
| arforensics-noise-a40 | 143 | 22.38% | 25.17% |
| arforensics-repost-720-qv9x1 | 143 | 77.62% | 77.62% |
| arforensics-repost-960-qv12x1 | 143 | 78.32% | 79.02% |
| arforensics-repost-960-qv7x4 | 143 | 76.92% | 76.92% |
| arforensics-resize-0.25-bicubic | 143 | 88.11% | 88.11% |
| arforensics-resize-0.5-bicubic | 143 | 95.80% | 95.80% |
| arforensics-sharpen-a1.0 | 143 | 89.51% | 89.51% |
| genimagepp | 1200 | 100.00% | 100.00% |
| genimagepp-blur-s1.0 | 57 | 100.00% | 100.00% |
| genimagepp-blur-s3.0 | 57 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 57 | 91.23% | 91.23% |
| genimagepp-jpeg-qv4 | 57 | 100.00% | 100.00% |
| genimagepp-noise-a10 | 57 | 100.00% | 100.00% |
| genimagepp-noise-a40 | 57 | 19.30% | 33.33% |
| genimagepp-repost-720-qv9x1 | 57 | 94.74% | 94.74% |
| genimagepp-repost-960-qv12x1 | 57 | 91.23% | 91.23% |
| genimagepp-repost-960-qv7x4 | 57 | 98.25% | 98.25% |
| genimagepp-resize-0.25-bicubic | 57 | 96.49% | 96.49% |
| genimagepp-resize-0.5-bicubic | 57 | 96.49% | 98.25% |
| genimagepp-sharpen-a1.0 | 57 | 100.00% | 98.25% |

## sensitive, threshold -0.494 (raw logit), fitted schedule

12457 items (6600 AI, 5857 real); 2.16 views looked at per item of 6
decided after 1..6 views: 5489 / 3940 / 278 / 1645 / 515 / 590
verdicts that differ from every view: 32 (16 more flagged, 16 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 220 of 5857 | 3.76% | 91.14% | 0.9647 |
| early exit | 214 of 5857 | 3.65% | 91.23% | 0.9657 |
| difference | -6 | -0.10 points | +0.09 points | +0.0010 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:floreview | 3393 | 4.13% | 3.89% | 132 of 3393 | within |
| real:hdrplus | 2464 | 3.25% | 3.33% | 82 of 2464 | within |

Mean over real sources with at least 100 items: 3.69% with every view, 3.61% under early exit; the calibration target is 5.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 3000 | 93.80% | 93.80% |
| arforensics-blur-s1.0 | 143 | 94.41% | 94.41% |
| arforensics-blur-s3.0 | 143 | 97.20% | 97.20% |
| arforensics-jpeg-qv12 | 143 | 78.32% | 78.32% |
| arforensics-jpeg-qv4 | 143 | 79.72% | 79.72% |
| arforensics-noise-a10 | 143 | 87.41% | 87.41% |
| arforensics-noise-a40 | 143 | 22.38% | 23.08% |
| arforensics-repost-720-qv9x1 | 143 | 77.62% | 77.62% |
| arforensics-repost-960-qv12x1 | 143 | 78.32% | 78.32% |
| arforensics-repost-960-qv7x4 | 143 | 76.92% | 76.92% |
| arforensics-resize-0.25-bicubic | 143 | 88.11% | 88.11% |
| arforensics-resize-0.5-bicubic | 143 | 95.80% | 95.80% |
| arforensics-sharpen-a1.0 | 143 | 89.51% | 89.51% |
| genimagepp | 1200 | 100.00% | 100.00% |
| genimagepp-blur-s1.0 | 57 | 100.00% | 100.00% |
| genimagepp-blur-s3.0 | 57 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 57 | 91.23% | 91.23% |
| genimagepp-jpeg-qv4 | 57 | 100.00% | 100.00% |
| genimagepp-noise-a10 | 57 | 100.00% | 100.00% |
| genimagepp-noise-a40 | 57 | 19.30% | 28.07% |
| genimagepp-repost-720-qv9x1 | 57 | 94.74% | 94.74% |
| genimagepp-repost-960-qv12x1 | 57 | 91.23% | 91.23% |
| genimagepp-repost-960-qv7x4 | 57 | 98.25% | 98.25% |
| genimagepp-resize-0.25-bicubic | 57 | 96.49% | 96.49% |
| genimagepp-resize-0.5-bicubic | 57 | 96.49% | 96.49% |
| genimagepp-sharpen-a1.0 | 57 | 100.00% | 100.00% |

photo second-opinion band (-7, 0.5) in calibrated logits is (-12.090, 2.617) in raw logits around the threshold 1.636.
  routed from every view: 5848 of 5857 real (99.85%), 1107 of 6600 AI (16.77%)
  routed from the early-exit views: 5847 of 5857 real (99.83%), 1052 of 6600 AI (15.94%)
  The band and its rule were not refitted: that takes second-stage scores on calibration data.

wrote the fitted schedule to runs/frozen/ws2/bound-max.json
