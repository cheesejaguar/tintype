runs/frozen/ws2/bound-max.safetensors: stored schedule, order [2, 0, 1, 3, 4, 5], margins after N views (raw logits) 1: 6.5, 2: 4, 3: 4, 4: 1.75, 5: 1

## photo, threshold 1.636 (raw logit), stored schedule

19326 items (9600 AI, 9726 real); 1.70 views looked at per item of 6
decided after 1..6 views: 11933 / 4835 / 336 / 1320 / 400 / 502
verdicts that differ from every view: 29 (15 more flagged, 14 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 86 of 9726 | 0.88% | 90.93% | 0.9902 |
| early exit | 82 of 9726 | 0.84% | 90.98% | 0.9907 |
| difference | -4 | -0.04 points | +0.05 points | +0.0005 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:csafe | 5304 | 1.23% | 1.17% | 62 of 5304 | over |
| real:floreview | 2486 | 0.56% | 0.52% | 13 of 2486 | within |
| real:hdrplus | 1936 | 0.36% | 0.36% | 7 of 1936 | within |

Mean over real sources with at least 100 items: 0.72% with every view, 0.68% under early exit; the calibration target is 1.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 6000 | 94.10% | 94.10% |
| arforensics-blur-s1.0 | 167 | 95.21% | 95.21% |
| arforensics-blur-s3.0 | 167 | 97.01% | 97.01% |
| arforensics-jpeg-qv12 | 167 | 63.47% | 63.47% |
| arforensics-jpeg-qv4 | 167 | 79.04% | 79.64% |
| arforensics-noise-a10 | 167 | 89.82% | 89.82% |
| arforensics-noise-a40 | 167 | 14.37% | 16.77% |
| arforensics-repost-720-qv9x1 | 167 | 68.86% | 68.86% |
| arforensics-repost-960-qv12x1 | 167 | 64.67% | 64.67% |
| arforensics-repost-960-qv7x4 | 167 | 75.45% | 75.45% |
| arforensics-resize-0.25-bicubic | 167 | 86.23% | 86.23% |
| arforensics-resize-0.5-bicubic | 167 | 97.60% | 97.60% |
| arforensics-sharpen-a1.0 | 167 | 93.41% | 93.41% |
| genimagepp | 1200 | 99.08% | 98.92% |
| genimagepp-blur-s1.0 | 33 | 100.00% | 100.00% |
| genimagepp-blur-s3.0 | 33 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 33 | 87.88% | 87.88% |
| genimagepp-jpeg-qv4 | 33 | 90.91% | 90.91% |
| genimagepp-noise-a10 | 33 | 96.97% | 96.97% |
| genimagepp-noise-a40 | 33 | 3.03% | 15.15% |
| genimagepp-repost-720-qv9x1 | 33 | 96.97% | 96.97% |
| genimagepp-repost-960-qv12x1 | 33 | 90.91% | 87.88% |
| genimagepp-repost-960-qv7x4 | 33 | 96.97% | 96.97% |
| genimagepp-resize-0.25-bicubic | 33 | 96.97% | 96.97% |
| genimagepp-resize-0.5-bicubic | 33 | 96.97% | 93.94% |
| genimagepp-sharpen-a1.0 | 33 | 100.00% | 100.00% |

## strict, threshold 3.504 (raw logit), stored schedule

19326 items (9600 AI, 9726 real); 1.71 views looked at per item of 6
decided after 1..6 views: 12129 / 4529 / 297 / 1371 / 478 / 522
verdicts that differ from every view: 31 (14 more flagged, 17 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 15 of 9726 | 0.15% | 86.12% | 0.9982 |
| early exit | 12 of 9726 | 0.12% | 86.12% | 0.9986 |
| difference | -3 | -0.03 points | +0.00 points | +0.0004 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:csafe | 5304 | 0.23% | 0.19% | 10 of 5304 | over |
| real:floreview | 2486 | 0.08% | 0.08% | 2 of 2486 | within |
| real:hdrplus | 1936 | 0.05% | 0.00% | 0 of 1936 | within |

Mean over real sources with at least 100 items: 0.12% with every view, 0.09% under early exit; the calibration target is 0.10% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 6000 | 91.08% | 91.07% |
| arforensics-blur-s1.0 | 167 | 93.41% | 93.41% |
| arforensics-blur-s3.0 | 167 | 90.42% | 90.42% |
| arforensics-jpeg-qv12 | 167 | 49.70% | 50.30% |
| arforensics-jpeg-qv4 | 167 | 65.87% | 65.87% |
| arforensics-noise-a10 | 167 | 82.04% | 82.63% |
| arforensics-noise-a40 | 167 | 4.79% | 5.99% |
| arforensics-repost-720-qv9x1 | 167 | 55.69% | 55.69% |
| arforensics-repost-960-qv12x1 | 167 | 49.10% | 50.30% |
| arforensics-repost-960-qv7x4 | 167 | 64.07% | 64.07% |
| arforensics-resize-0.25-bicubic | 167 | 77.25% | 77.25% |
| arforensics-resize-0.5-bicubic | 167 | 96.41% | 96.41% |
| arforensics-sharpen-a1.0 | 167 | 88.02% | 88.02% |
| genimagepp | 1200 | 95.42% | 95.00% |
| genimagepp-blur-s1.0 | 33 | 96.97% | 93.94% |
| genimagepp-blur-s3.0 | 33 | 100.00% | 100.00% |
| genimagepp-jpeg-qv12 | 33 | 60.61% | 60.61% |
| genimagepp-jpeg-qv4 | 33 | 78.79% | 75.76% |
| genimagepp-noise-a10 | 33 | 81.82% | 81.82% |
| genimagepp-noise-a40 | 33 | 3.03% | 3.03% |
| genimagepp-repost-720-qv9x1 | 33 | 60.61% | 66.67% |
| genimagepp-repost-960-qv12x1 | 33 | 63.64% | 63.64% |
| genimagepp-repost-960-qv7x4 | 33 | 75.76% | 75.76% |
| genimagepp-resize-0.25-bicubic | 33 | 90.91% | 90.91% |
| genimagepp-resize-0.5-bicubic | 33 | 81.82% | 81.82% |
| genimagepp-sharpen-a1.0 | 33 | 96.97% | 96.97% |

## sensitive, threshold -0.494 (raw logit), stored schedule

19326 items (9600 AI, 9726 real); 1.99 views looked at per item of 6
decided after 1..6 views: 9682 / 5705 / 359 / 2232 / 641 / 707
verdicts that differ from every view: 20 (12 more flagged, 8 fewer)

| Pooled | Real items flagged | FPR | Recall | Precision |
|---|---|---|---|---|
| every view | 372 of 9726 | 3.82% | 94.65% | 0.9607 |
| early exit | 368 of 9726 | 3.78% | 94.73% | 0.9611 |
| difference | -4 | -0.04 points | +0.08 points | +0.0004 |

| Real source | Items | FPR, every view | FPR, early exit | Flagged, early exit | Against the target |
|---|---|---|---|---|---|
| real:csafe | 5304 | 4.37% | 4.30% | 228 of 5304 | within |
| real:floreview | 2486 | 3.66% | 3.70% | 92 of 2486 | within |
| real:hdrplus | 1936 | 2.53% | 2.48% | 48 of 1936 | within |

Mean over real sources with at least 100 items: 3.52% with every view, 3.49% under early exit; the calibration target is 5.00% (early exit is within it).

| Generated items from | Items | Recall, every view | Recall, early exit |
|---|---|---|---|
| arforensics | 6000 | 96.45% | 96.45% |
| arforensics-blur-s1.0 | 167 | 97.60% | 97.60% |
| arforensics-blur-s3.0 | 167 | 100.00% | 100.00% |
| arforensics-jpeg-qv12 | 167 | 81.44% | 81.44% |
| arforensics-jpeg-qv4 | 167 | 88.62% | 88.62% |
| arforensics-noise-a10 | 167 | 94.01% | 94.01% |
| arforensics-noise-a40 | 167 | 29.34% | 31.74% |
| arforensics-repost-720-qv9x1 | 167 | 86.23% | 86.23% |
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
  routed from the early-exit views: 9680 of 9726 real (99.53%), 1043 of 9600 AI (10.86%)
  The band and its rule were not refitted: that takes second-stage scores on calibration data.

runs/frozen/ws2/test.A-photo.tsv: 19326 items as photo decides them under the stored schedule
