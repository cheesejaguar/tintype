runs/frozen/ws2/bound-max.before-refit.safetensors: stored schedule, order [2, 0, 1, 3, 4, 5], margins after N views (raw logits) 1: 6, 2: 3, 3: 2, 4: 1.5, 5: 1

## photo, threshold 1.636 (raw logit), stored schedule

19326 items (9600 AI, 9726 real); 1.53 views looked at per item of 6
decided after 1..6 views: 12868 / 4743 / 737 / 305 / 241 / 432
verdicts that differ from every view: 61 (25 more flagged, 36 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 86 of 9726 | 0.88% | 90.93% | 0.9902 |
| early exit | 67 of 9726 | 0.69% | 91.01% | 0.9924 |
| difference | -19 | -0.20 points | +0.08 points | +0.0021 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:csafe | 5304 | 1.23% | 0.90% | 48 of 5304 | within |
| real:floreview | 2486 | 0.56% | 0.48% | 12 of 2486 | within |
| real:hdrplus | 1936 | 0.36% | 0.36% | 7 of 1936 | within |

Mean over real sources with at least 100 items: 0.72% with every view, 0.58% under early exit; the calibration target is 1.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 6000 | 94.10% | 94.07% |
| arforensics-blur-s1.0 | 167 | 95.21% | 95.21% |
| arforensics-blur-s3.0 | 167 | 97.01% | 97.01% |
| arforensics-jpeg-qv12 | 167 | 63.47% | 64.07% |
| arforensics-jpeg-qv4 | 167 | 79.04% | 80.24% |
| arforensics-noise-a10 | 167 | 89.82% | 89.82% |
| arforensics-noise-a40 | 167 | 14.37% | 18.56% |
| arforensics-repost-720-qv9x1 | 167 | 68.86% | 68.86% |
| arforensics-repost-960-qv12x1 | 167 | 64.67% | 64.67% |
| arforensics-repost-960-qv7x4 | 167 | 75.45% | 76.05% |
| arforensics-resize-0.25-bicubic | 167 | 86.23% | 86.23% |
| arforensics-resize-0.5-bicubic | 167 | 97.60% | 97.60% |
| arforensics-sharpen-a1.0 | 167 | 93.41% | 93.41% |
| genimagepp | 1200 | 99.08% | 98.83% |
| genimagepp-blur-s1.0 | 33 | 100.00% | 96.97% |
| genimagepp-blur-s3.0 | 33 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 33 | 87.88% | 90.91% |
| genimagepp-jpeg-qv4 | 33 | 90.91% | 90.91% |
| genimagepp-noise-a10 | 33 | 96.97% | 96.97% |
| genimagepp-noise-a40 | 33 | 3.03% | 18.18% |
| genimagepp-repost-720-qv9x1 | 33 | 96.97% | 93.94% |
| genimagepp-repost-960-qv12x1 | 33 | 90.91% | 87.88% |
| genimagepp-repost-960-qv7x4 | 33 | 96.97% | 96.97% |
| genimagepp-resize-0.25-bicubic | 33 | 96.97% | 96.97% |
| genimagepp-resize-0.5-bicubic | 33 | 96.97% | 93.94% |
| genimagepp-sharpen-a1.0 | 33 | 100.00% | 100.00% |

## strict, threshold 3.504 (raw logit), stored schedule

19326 items (9600 AI, 9726 real); 1.53 views looked at per item of 6
decided after 1..6 views: 13137 / 4318 / 810 / 332 / 268 / 461
verdicts that differ from every view: 68 (33 more flagged, 35 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 15 of 9726 | 0.15% | 86.12% | 0.9982 |
| early exit | 9 of 9726 | 0.09% | 86.17% | 0.9989 |
| difference | -6 | -0.06 points | +0.04 points | +0.0007 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:csafe | 5304 | 0.23% | 0.13% | 7 of 5304 | over |
| real:floreview | 2486 | 0.08% | 0.08% | 2 of 2486 | within |
| real:hdrplus | 1936 | 0.05% | 0.00% | 0 of 1936 | within |

Mean over real sources with at least 100 items: 0.12% with every view, 0.07% under early exit; the calibration target is 0.10% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 6000 | 91.08% | 91.05% |
| arforensics-blur-s1.0 | 167 | 93.41% | 93.41% |
| arforensics-blur-s3.0 | 167 | 90.42% | 90.42% |
| arforensics-jpeg-qv12 | 167 | 49.70% | 50.90% |
| arforensics-jpeg-qv4 | 167 | 65.87% | 66.47% |
| arforensics-noise-a10 | 167 | 82.04% | 83.23% |
| arforensics-noise-a40 | 167 | 4.79% | 6.59% |
| arforensics-repost-720-qv9x1 | 167 | 55.69% | 55.69% |
| arforensics-repost-960-qv12x1 | 167 | 49.10% | 51.50% |
| arforensics-repost-960-qv7x4 | 167 | 64.07% | 63.47% |
| arforensics-resize-0.25-bicubic | 167 | 77.25% | 77.25% |
| arforensics-resize-0.5-bicubic | 167 | 96.41% | 96.41% |
| arforensics-sharpen-a1.0 | 167 | 88.02% | 88.02% |
| genimagepp | 1200 | 95.42% | 94.67% |
| genimagepp-blur-s1.0 | 33 | 96.97% | 93.94% |
| genimagepp-blur-s3.0 | 33 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 33 | 60.61% | 60.61% |
| genimagepp-jpeg-qv4 | 33 | 78.79% | 78.79% |
| genimagepp-noise-a10 | 33 | 81.82% | 84.85% |
| genimagepp-noise-a40 | 33 | 3.03% | 3.03% |
| genimagepp-repost-720-qv9x1 | 33 | 60.61% | 66.67% |
| genimagepp-repost-960-qv12x1 | 33 | 63.64% | 69.70% |
| genimagepp-repost-960-qv7x4 | 33 | 75.76% | 78.79% |
| genimagepp-resize-0.25-bicubic | 33 | 90.91% | 90.91% |
| genimagepp-resize-0.5-bicubic | 33 | 81.82% | 84.85% |
| genimagepp-sharpen-a1.0 | 33 | 96.97% | 90.91% |

## sensitive, threshold -0.494 (raw logit), stored schedule

19326 items (9600 AI, 9726 real); 1.73 views looked at per item of 6
decided after 1..6 views: 10775 / 6052 / 1012 / 525 / 381 / 581
verdicts that differ from every view: 83 (30 more flagged, 53 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 372 of 9726 | 3.82% | 94.65% | 0.9607 |
| early exit | 336 of 9726 | 3.45% | 94.78% | 0.9644 |
| difference | -36 | -0.37 points | +0.14 points | +0.0037 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:csafe | 5304 | 4.37% | 3.92% | 208 of 5304 | within |
| real:floreview | 2486 | 3.66% | 3.54% | 88 of 2486 | within |
| real:hdrplus | 1936 | 2.53% | 2.07% | 40 of 1936 | within |

Mean over real sources with at least 100 items: 3.52% with every view, 3.18% under early exit; the calibration target is 5.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 6000 | 96.45% | 96.45% |
| arforensics-blur-s1.0 | 167 | 97.60% | 97.60% |
| arforensics-blur-s3.0 | 167 | 100.00% | 100.00% |
| arforensics-jpeg-qv12 | 167 | 81.44% | 81.44% |
| arforensics-jpeg-qv4 | 167 | 88.62% | 88.62% |
| arforensics-noise-a10 | 167 | 94.01% | 94.01% |
| arforensics-noise-a40 | 167 | 29.34% | 34.13% |
| arforensics-repost-720-qv9x1 | 167 | 86.23% | 86.83% |
| arforensics-repost-960-qv12x1 | 167 | 81.44% | 81.44% |
| arforensics-repost-960-qv7x4 | 167 | 88.02% | 88.62% |
| arforensics-resize-0.25-bicubic | 167 | 95.81% | 95.81% |
| arforensics-resize-0.5-bicubic | 167 | 98.80% | 98.80% |
| arforensics-sharpen-a1.0 | 167 | 97.01% | 97.01% |
| genimagepp | 1200 | 99.83% | 99.83% |
| genimagepp-blur-s1.0 | 33 | 100.00% | 100.00% |
| genimagepp-blur-s3.0 | 33 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 33 | 100.00% | 100.00% |
| genimagepp-jpeg-qv4 | 33 | 100.00% | 100.00% |
| genimagepp-noise-a10 | 33 | 100.00% | 100.00% |
| genimagepp-noise-a40 | 33 | 18.18% | 27.27% |
| genimagepp-repost-720-qv9x1 | 33 | 100.00% | 100.00% |
| genimagepp-repost-960-qv12x1 | 33 | 96.97% | 96.97% |
| genimagepp-repost-960-qv7x4 | 33 | 96.97% | 96.97% |
| genimagepp-resize-0.25-bicubic | 33 | 100.00% | 100.00% |
| genimagepp-resize-0.5-bicubic | 33 | 100.00% | 100.00% |
| genimagepp-sharpen-a1.0 | 33 | 100.00% | 100.00% |

photo second-opinion band (-7, 0.5) in calibrated logits is (-12.090, 2.617) in raw logits around the threshold 1.636.
  routed from every view: 9685 of 9726 real (99.58%), 1087 of 9600 AI (11.32%)
  routed from the early-exit views: 9684 of 9726 real (99.57%), 1025 of 9600 AI (10.68%)
  The band and its rule were not refitted: that takes second-stage scores on calibration data.
