# Frontier photo development set: paired per-operation audit

Exploratory. The 745 paired inputs of the Frontier JPEG stress table
([EVIDENCE.md](../../EVIDENCE.md#jpeg-re-encoding-and-downscaling): 261
generated, 484 authentic; the set was used during development) and twelve
FFmpeg 9.0.2 renderings of each, one transform at a time:

```sh
python3 scripts/stress-photo.py paired-images.tsv suite --jobs 4 --transform <name>
tintype score --device cpu --weights runs/ws1-audit/tintype.safetensors --manifest suite/<name>.tsv --out <name>.tsv
```

`runs/ws1-audit/tintype.safetensors` is a copy of the released photo
weights with its sidecar; the default profile with early exit, as
released. Rendered at commit `d55d07e` (`stress-photo.py` unchanged
since); scored 2026-11-14 on the CPU engine of an Apple M6 (macOS 27.2)
with binaries built from this branch between `0e2e73c` and `40816c8`,
whose scoring code is that of the base commit `f684678`. `jpeg-qv4` and `repost-960-qv12x1` are the two
transforms of the archived table and reproduce its counts exactly (139 and
73 of 261 at 1% FPR). The lossless renderings were deleted after scoring
(tens of megabytes each); [scores.tsv.gz](scores.tsv.gz) keeps every
score as `<transform>/<original file name>`.

```sh
python3 scripts/report-stress.py --scores docs/results/frontier-operation-stress/scores.tsv.gz --pair-by-name
```

Frozen cutoff 0.5 (the shipped default). Empirical cutoffs are taken on
the 484 authentic copies of each row, so the 1% point allows four flagged
and the 0.1% point none: both describe this cohort only.

| Transform | Pairs: AI / real | Real flagged, original → processed | FPR shift (newly flagged, cleared) | Recall, original → processed | Recall at 1% FPR | Recall at 0.1% FPR | AUROC | Originals' 1% cutoff on processed: FPR, recall |
|---|---|---|---|---|---|---|---|---|
| original | 261 / 484 | 9/484 (1.9%) → 9/484 (1.9%) | +0.00 pt (+0, −0) | 205/261 (78.5%) → 205/261 (78.5%) | 181/261 (69.3%), 4 FP | 137/261 (52.5%), 0 FP | 0.960 | 4/484 (0.8%), 181/261 (69.3%) |
| resize-0.25-bicubic | 261 / 484 | 9/484 (1.9%) → 16/484 (3.3%) | +1.45 pt (+14, −7) | 205/261 (78.5%) → 189/261 (72.4%) | 143/261 (54.8%), 4 FP | 34/261 (13.0%), 0 FP | 0.944 | 11/484 (2.3%), 167/261 (64.0%) |
| resize-0.5-bicubic | 261 / 484 | 9/484 (1.9%) → 20/484 (4.1%) | +2.27 pt (+15, −4) | 205/261 (78.5%) → 198/261 (75.9%) | 113/261 (43.3%), 4 FP | 27/261 (10.3%), 0 FP | 0.933 | 8/484 (1.7%), 173/261 (66.3%) |
| jpeg-qv4 | 261 / 484 | 9/484 (1.9%) → 1/484 (0.2%) | -1.65 pt (+0, −8) | 205/261 (78.5%) → 116/261 (44.4%) | 139/261 (53.3%), 4 FP | 99/261 (37.9%), 0 FP | 0.922 | 0/484 (0.0%), 97/261 (37.2%) |
| jpeg-qv12 | 261 / 484 | 9/484 (1.9%) → 3/484 (0.6%) | -1.24 pt (+1, −7) | 205/261 (78.5%) → 102/261 (39.1%) | 137/261 (52.5%), 4 FP | 71/261 (27.2%), 0 FP | 0.932 | 1/484 (0.2%), 75/261 (28.7%) |
| blur-s1.0 | 261 / 484 | 9/484 (1.9%) → 13/484 (2.7%) | +0.83 pt (+7, −3) | 205/261 (78.5%) → 201/261 (77.0%) | 141/261 (54.0%), 4 FP | 39/261 (14.9%), 0 FP | 0.944 | 6/484 (1.2%), 167/261 (64.0%) |
| blur-s3.0 | 261 / 484 | 9/484 (1.9%) → 30/484 (6.2%) | +4.34 pt (+25, −4) | 205/261 (78.5%) → 213/261 (81.6%) | 125/261 (47.9%), 4 FP | 13/261 (5.0%), 0 FP | 0.935 | 12/484 (2.5%), 173/261 (66.3%) |
| noise-a10 | 261 / 484 | 9/484 (1.9%) → 65/484 (13.4%) | +11.57 pt (+57, −1) | 205/261 (78.5%) → 180/261 (69.0%) | 29/261 (11.1%), 4 FP | 10/261 (3.8%), 0 FP | 0.849 | 33/484 (6.8%), 156/261 (59.8%) |
| noise-a40 | 261 / 484 | 9/484 (1.9%) → 27/484 (5.6%) | +3.72 pt (+27, −9) | 205/261 (78.5%) → 8/261 (3.1%) | 1/261 (0.4%), 4 FP | 0/261 (0.0%), 0 FP | 0.575 | 20/484 (4.1%), 6/261 (2.3%) |
| sharpen-a1.0 | 261 / 484 | 9/484 (1.9%) → 9/484 (1.9%) | +0.00 pt (+4, −4) | 205/261 (78.5%) → 202/261 (77.4%) | 178/261 (68.2%), 4 FP | 45/261 (17.2%), 0 FP | 0.951 | 4/484 (0.8%), 177/261 (67.8%) |
| repost-720-qv9x1 | 261 / 484 | 9/484 (1.9%) → 13/484 (2.7%) | +0.83 pt (+13, −9) | 205/261 (78.5%) → 105/261 (40.2%) | 58/261 (22.2%), 4 FP | 27/261 (10.3%), 0 FP | 0.897 | 7/484 (1.4%), 71/261 (27.2%) |
| repost-960-qv7x4 | 261 / 484 | 9/484 (1.9%) → 13/484 (2.7%) | +0.83 pt (+12, −8) | 205/261 (78.5%) → 120/261 (46.0%) | 91/261 (34.9%), 4 FP | 6/261 (2.3%), 0 FP | 0.907 | 4/484 (0.8%), 90/261 (34.5%) |
| repost-960-qv12x1 | 261 / 484 | 9/484 (1.9%) → 10/484 (2.1%) | +0.21 pt (+9, −8) | 205/261 (78.5%) → 101/261 (38.7%) | 73/261 (28.0%), 4 FP | 9/261 (3.4%), 0 FP | 0.899 | 6/484 (1.2%), 79/261 (30.3%) |

Transform names are recipes (`python3 scripts/stress-photo.py --help`):
resize factor and kernel, FFmpeg MJPEG `-q:v`, blur sigma in pixels,
FFmpeg `noise` strength, unsharp amount, and for a repost the cap in
pixels, `-q:v` and the number of cycles.
