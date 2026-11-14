# Processing-chain arms on the Frontier photo pairs: a development diagnostic

Exploratory, and it decides nothing. The arms of
[processing-chains.md](../../runbooks/processing-chains.md#3-the-three-arms-same-budget)
are compared for a ship decision only on the frozen split, which does not
exist yet. This is what could be measured before it: arm A (the current
recipe) and arm B (pixel chains), each trained twice, scored on the 745
Frontier photo pairs (261 generated, 484 authentic; used during
development) and twelve FFmpeg renderings of each.

Read it with these limits in mind:

- The Frontier pairs are development data, and every cutoff below is taken
  on the cohort it is reported on. With 484 authentic images the 1% point
  allows four flagged and the 0.1% point none, so the 0.1% column is the
  recall above the single highest authentic score.
- `data/store/tiny-genimage` and `data/store/stylegan3` have manifests but
  no files on this machine, so all four runs leave those two manifests
  out. The arms are comparable to each other; arm A is not the phase-6
  recipe, and none of these checkpoints is the released model.
- The checkpoints are uncalibrated (`logit_scale` 1, `logit_bias` 0), so a
  fixed cutoff means nothing across them. Scores are the mean of the six
  view logits (`--all-views`, no early exit), which is also not what the
  released default profile computes: do not compare these rows with
  [frontier-operation-stress](../frontier-operation-stress/report.md).
- The 745 files were compared by SHA-256 with the files of the `frontier`,
  `frontier-heldout`, `nano-banana` and `openfake-test-reals` training
  stores: none is byte-identical. Re-encoded copies or the same scenes
  would not show up in that check.
- Two runs per arm give one seed-to-seed gap per arm, not a distribution.
- Twelve single operations from one tool; no chains, no learned codecs.

## What ran

Commit `10568b1`, 2026-11-14, Apple M6 (12 cores, 32 GB), macOS 27.2,
Apple GPU engine, FFmpeg 9.0.2. `$INIT` is `runs/v4/step_007000.safetensors`
(phase 3; SHA-256 begins `2d11bffd77f24641`), and `"${MANIFESTS[@]}"` is
`--manifest` for 14 photo stores (openfake, openfake2, openfake-reddit,
frontier, frontier-heldout, nano-banana, defactify, commfor-small,
openfake-test-reals, sofake-set, commfor-small2, tpdne, ip-adapter,
celeba-hq) and 9 video stores (vidguard, genbuster, genvideo100k, aigvd,
k400, pexels, k400b, pexelsb, aigvd2): 326,955 training images.

```sh
tintype train --init "$INIT" "${MANIFESTS[@]}" --out runs/ws1-baseline \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --seed 1
tintype train --init "$INIT" "${MANIFESTS[@]}" --out runs/ws1-chains \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --seed 1 \
  --p-chain 0.5 --chain-ops 1,3 --chain-kinds resize,jpeg,blur,noise,sharpen,repost \
  --chain-schedule 0:0.5:1:0.25,1000:0.5:2:0.5,3000:0.5:3:0.75 \
  --messenger-cap 720,2048 --messenger-cycles 1,4
# and both again with --seed 2 into runs/ws1-baseline-seed2 and runs/ws1-chains-seed2
tintype verify-models   # after each run: four files ok
```

Each run took about 55 minutes at 75 to 82 images/s, and every logged
interval of all four showed 0% data wait: the chains did not slow the
loader. Each `step_008000` checkpoint and the starting checkpoint were
copied with their sidecars to `runs/ws1-eval/<name>.safetensors`, then, one
transform at a time:

```sh
python3 -I scripts/stress-photo.py paired-images.tsv suite --jobs 6 --transform <name>
tintype score --weights runs/ws1-eval/<checkpoint>.safetensors --manifest suite/<name>.tsv --out <name>.tsv --all-views
```

The seed-2 checkpoints were scored on a second rendering by the same
commands (the renderings are deleted after scoring; the two manifests are
identical). [scores.tsv.gz](scores.tsv.gz) keeps every score as
`<transform>/<original file name>`, one column per checkpoint (`init`,
`baseline`, `baseline-seed2`, `chains`, `chains-seed2`), each the mean of
the six view logits. The tables are from:

```sh
python3 -I scripts/report-stress.py --scores docs/results/processing-chain-arms/scores.tsv.gz --pair-by-name --frozen 0 --column chains --versus baseline
```

and the same with the other columns. The fixed cutoff that command also
prints (logit 0) is not reported here.

## Validation images from the training stores

The trainer's own last evaluation, on 1,999 held-out, unprocessed images
of the training stores (`train.log`, "eval @8000"). The starting
checkpoint scored 0.9927, 0.9856 and 0.9949 on the same images.

| Run | Fit-view AUC | Native-view AUC | Fused AUC |
|---|---|---|---|
| Baseline, seed 1 | 0.9954 | 0.9926 | 0.9980 |
| Baseline, seed 2 | 0.9949 | 0.9930 | 0.9975 |
| Chains, seed 1 | 0.9947 | 0.9901 | 0.9967 |
| Chains, seed 2 | 0.9942 | 0.9914 | 0.9969 |

Both chains runs end below both baseline runs on clean in-distribution
images, by 0.0006 to 0.0013 fused AUC.

## Frontier pairs

Counts are of 261 generated images. "Start" is the starting checkpoint.

### Recall at 1% FPR

| Transform | Start | Baseline, seed 1 | Baseline, seed 2 | Chains, seed 1 | Chains, seed 2 | Chains − baseline, mean of two seeds | Seed 2 − seed 1: baseline, chains |
|---|---|---|---|---|---|---|---|
| original | 186 (71.3%) | 191 (73.2%) | 181 (69.3%) | 184 (70.5%) | 188 (72.0%) | +0.0 pt | -3.8 pt, +1.5 pt |
| resize-0.5-bicubic | 139 (53.3%) | 145 (55.6%) | 130 (49.8%) | 135 (51.7%) | 139 (53.3%) | -0.2 pt | -5.7 pt, +1.5 pt |
| resize-0.25-bicubic | 150 (57.5%) | 153 (58.6%) | 151 (57.9%) | 153 (58.6%) | 157 (60.2%) | +1.1 pt | -0.8 pt, +1.5 pt |
| jpeg-qv4 | 144 (55.2%) | 129 (49.4%) | 116 (44.4%) | 118 (45.2%) | 110 (42.1%) | -3.3 pt | -5.0 pt, -3.1 pt |
| jpeg-qv12 | 132 (50.6%) | 113 (43.3%) | 123 (47.1%) | 118 (45.2%) | 113 (43.3%) | -1.0 pt | +3.8 pt, -1.9 pt |
| blur-s1.0 | 169 (64.8%) | 154 (59.0%) | 134 (51.3%) | 152 (58.2%) | 159 (60.9%) | +4.4 pt | -7.7 pt, +2.7 pt |
| blur-s3.0 | 144 (55.2%) | 112 (42.9%) | 103 (39.5%) | 125 (47.9%) | 136 (52.1%) | +8.8 pt | -3.4 pt, +4.2 pt |
| noise-a10 | 117 (44.8%) | 98 (37.5%) | 82 (31.4%) | 144 (55.2%) | 151 (57.9%) | +22.0 pt | -6.1 pt, +2.7 pt |
| noise-a40 | 3 (1.1%) | 1 (0.4%) | 2 (0.8%) | 34 (13.0%) | 28 (10.7%) | +11.3 pt | +0.4 pt, -2.3 pt |
| sharpen-a1.0 | 199 (76.2%) | 192 (73.6%) | 191 (73.2%) | 196 (75.1%) | 196 (75.1%) | +1.7 pt | -0.4 pt, +0.0 pt |
| repost-720-qv9x1 | 74 (28.4%) | 58 (22.2%) | 57 (21.8%) | 60 (23.0%) | 54 (20.7%) | -0.2 pt | -0.4 pt, -2.3 pt |
| repost-960-qv7x4 | 86 (33.0%) | 91 (34.9%) | 88 (33.7%) | 84 (32.2%) | 86 (33.0%) | -1.7 pt | -1.1 pt, +0.8 pt |
| repost-960-qv12x1 | 65 (24.9%) | 55 (21.1%) | 57 (21.8%) | 56 (21.5%) | 57 (21.8%) | +0.2 pt | +0.8 pt, +0.4 pt |

### Paired bootstrap of the difference at 1% FPR

| Transform | Chains − baseline, seed 1 | Chains − baseline, seed 2 | Baseline, seed 2 − seed 1 | Chains, seed 2 − seed 1 |
|---|---|---|---|---|
| original | -2.7 pt (-13.0 to +5.0) | +2.7 pt (-7.7 to +14.6) | -3.8 pt (-13.4 to +3.4) | +1.5 pt (-2.7 to +8.8) |
| resize-0.5-bicubic | -3.8 pt (-9.6 to +6.9) | +3.4 pt (-10.3 to +14.2) | -5.7 pt (-11.1 to +4.6) | +1.5 pt (-5.7 to +10.0) |
| resize-0.25-bicubic | +0.0 pt (-14.2 to +9.2) | +2.3 pt (-7.7 to +11.1) | -0.8 pt (-8.4 to +5.7) | +1.5 pt (-7.3 to +5.4) |
| jpeg-qv4 | -4.2 pt (-16.5 to +4.2) | -2.3 pt (-13.0 to +8.4) | -5.0 pt (-15.7 to +3.1) | -3.1 pt (-10.0 to +6.1) |
| jpeg-qv12 | +1.9 pt (-14.2 to +6.1) | -3.8 pt (-11.5 to +4.2) | +3.8 pt (-3.1 to +8.4) | -1.9 pt (-8.4 to +7.3) |
| blur-s1.0 | -0.8 pt (-10.3 to +14.2) | +9.6 pt (-4.6 to +26.4) | -7.7 pt (-18.4 to +1.9) | +2.7 pt (-1.9 to +11.1) |
| blur-s3.0 | +5.0 pt (-13.4 to +22.2) | +12.6 pt (-9.6 to +27.6) | -3.4 pt (-18.4 to +10.3) | +4.2 pt (-10.3 to +10.7) |
| noise-a10 | +17.6 pt (+3.4 to +33.3) | +26.4 pt (+14.2 to +37.9) | -6.1 pt (-15.3 to +2.7) | +2.7 pt (-4.6 to +10.0) |
| noise-a40 | +12.6 pt (+1.9 to +25.7) | +10.0 pt (+0.4 to +26.4) | +0.4 pt (-1.5 to +1.1) | -2.3 pt (-8.0 to +10.0) |
| sharpen-a1.0 | +1.5 pt (-5.7 to +16.9) | +1.9 pt (-9.2 to +19.9) | -0.4 pt (-6.9 to +1.9) | +0.0 pt (-9.2 to +2.7) |
| repost-720-qv9x1 | +0.8 pt (-13.8 to +5.7) | -1.1 pt (-6.5 to +5.4) | -0.4 pt (-13.0 to +5.7) | -2.3 pt (-8.4 to +6.1) |
| repost-960-qv7x4 | -2.7 pt (-10.3 to +7.7) | -0.8 pt (-8.0 to +5.0) | -1.1 pt (-6.5 to +5.0) | +0.8 pt (-8.8 to +6.9) |
| repost-960-qv12x1 | +0.4 pt (-10.0 to +11.5) | +0.0 pt (-8.8 to +6.1) | +0.8 pt (-9.2 to +8.4) | +0.4 pt (-10.7 to +7.7) |

### Recall at 0.1% FPR

| Transform | Start | Baseline, seed 1 | Baseline, seed 2 | Chains, seed 1 | Chains, seed 2 |
|---|---|---|---|---|---|
| original | 138 (52.9%) | 110 (42.1%) | 136 (52.1%) | 158 (60.5%) | 161 (61.7%) |
| resize-0.5-bicubic | 91 (34.9%) | 90 (34.5%) | 104 (39.8%) | 75 (28.7%) | 78 (29.9%) |
| resize-0.25-bicubic | 113 (43.3%) | 127 (48.7%) | 97 (37.2%) | 92 (35.2%) | 98 (37.5%) |
| jpeg-qv4 | 84 (32.2%) | 75 (28.7%) | 95 (36.4%) | 83 (31.8%) | 85 (32.6%) |
| jpeg-qv12 | 34 (13.0%) | 30 (11.5%) | 35 (13.4%) | 38 (14.6%) | 48 (18.4%) |
| blur-s1.0 | 92 (35.2%) | 88 (33.7%) | 108 (41.4%) | 144 (55.2%) | 141 (54.0%) |
| blur-s3.0 | 70 (26.8%) | 59 (22.6%) | 42 (16.1%) | 64 (24.5%) | 62 (23.8%) |
| noise-a10 | 77 (29.5%) | 64 (24.5%) | 54 (20.7%) | 100 (38.3%) | 118 (45.2%) |
| noise-a40 | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 6 (2.3%) | 3 (1.1%) |
| sharpen-a1.0 | 173 (66.3%) | 133 (51.0%) | 137 (52.5%) | 132 (50.6%) | 140 (53.6%) |
| repost-720-qv9x1 | 23 (8.8%) | 40 (15.3%) | 36 (13.8%) | 26 (10.0%) | 27 (10.3%) |
| repost-960-qv7x4 | 67 (25.7%) | 57 (21.8%) | 60 (23.0%) | 61 (23.4%) | 62 (23.8%) |
| repost-960-qv12x1 | 46 (17.6%) | 29 (11.1%) | 30 (11.5%) | 28 (10.7%) | 34 (13.0%) |

### AUROC

| Transform | Start | Baseline, seed 1 | Baseline, seed 2 | Chains, seed 1 | Chains, seed 2 |
|---|---|---|---|---|---|
| original | 0.975 | 0.974 | 0.972 | 0.968 | 0.969 |
| resize-0.5-bicubic | 0.948 | 0.949 | 0.938 | 0.944 | 0.950 |
| resize-0.25-bicubic | 0.948 | 0.956 | 0.946 | 0.952 | 0.958 |
| jpeg-qv4 | 0.953 | 0.947 | 0.944 | 0.942 | 0.940 |
| jpeg-qv12 | 0.953 | 0.941 | 0.941 | 0.937 | 0.938 |
| blur-s1.0 | 0.963 | 0.960 | 0.951 | 0.953 | 0.955 |
| blur-s3.0 | 0.941 | 0.921 | 0.899 | 0.930 | 0.932 |
| noise-a10 | 0.896 | 0.889 | 0.863 | 0.953 | 0.950 |
| noise-a40 | 0.668 | 0.632 | 0.600 | 0.854 | 0.841 |
| sharpen-a1.0 | 0.971 | 0.969 | 0.964 | 0.966 | 0.969 |
| repost-720-qv9x1 | 0.913 | 0.908 | 0.899 | 0.905 | 0.893 |
| repost-960-qv7x4 | 0.924 | 0.925 | 0.919 | 0.921 | 0.910 |
| repost-960-qv12x1 | 0.910 | 0.902 | 0.897 | 0.900 | 0.885 |

### The originals' 1% cutoff applied to the processed copies

| Transform | Start | Baseline, seed 1 | Baseline, seed 2 | Chains, seed 1 | Chains, seed 2 |
|---|---|---|---|---|---|
| original | 4, 186 | 4, 191 | 4, 181 | 4, 184 | 4, 188 |
| resize-0.5-bicubic | 10, 177 | 12, 186 | 11, 174 | 14, 178 | 13, 187 |
| resize-0.25-bicubic | 8, 162 | 9, 177 | 11, 171 | 12, 186 | 14, 190 |
| jpeg-qv4 | 0, 78 | 1, 86 | 0, 78 | 2, 103 | 3, 106 |
| jpeg-qv12 | 1, 42 | 2, 53 | 1, 44 | 2, 74 | 2, 90 |
| blur-s1.0 | 7, 175 | 7, 170 | 7, 161 | 7, 164 | 6, 177 |
| blur-s3.0 | 16, 201 | 10, 168 | 15, 154 | 10, 173 | 10, 172 |
| noise-a10 | 9, 132 | 28, 167 | 27, 144 | 6, 167 | 8, 171 |
| noise-a40 | 4, 0 | 14, 6 | 6, 2 | 3, 23 | 3, 23 |
| sharpen-a1.0 | 1, 176 | 4, 189 | 4, 185 | 2, 180 | 2, 187 |
| repost-720-qv9x1 | 3, 55 | 5, 59 | 3, 53 | 10, 75 | 13, 88 |
| repost-960-qv7x4 | 2, 70 | 1, 77 | 1, 68 | 6, 93 | 12, 97 |
| repost-960-qv12x1 | 2, 51 | 5, 60 | 4, 48 | 7, 78 | 9, 88 |

In the last table each cell is "authentic flagged of 484, generated
detected of 261" when the cutoff that flags four authentic originals is
applied unchanged to the processed copies.

The bootstrap interval covers the choice of test images only; the last
two columns of the first two tables show what a second training run of
the same recipe changes.

## What it shows

- **Additive noise is the one clear gain.** Chains beat the baseline at
  both strengths in both seeds, by 22.0 and 11.3 points on average, with
  every bootstrap interval above zero and gaps well beyond the seed-to-seed
  differences (at most 6.1 points). AUROC at strength 10 goes from 0.889
  and 0.863 to 0.953 and 0.950. With the originals' cutoff, the baseline
  flags 28 and 27 of 484 authentic images after mild noise and the chains
  arm 6 and 8.
- **Blur leans the same way but is not established.** Strong blur is 8.8
  points better on average and better in both seeds; both intervals
  include zero.
- **JPEG and reposts did not improve at their own cutoffs.** JPEG is 1.0
  to 3.3 points lower on average and the three reposts are within 1.7
  points either way, all inside the seed-to-seed spread. The chains
  contain both operations.
- **Chains move processed scores up, for both classes.** With the
  originals' cutoff the chains arm keeps more detections after JPEG and
  reposts (JPEG quality 12: 74 and 90 against 53 and 44 of 261) and flags
  more authentic images there (repost to 720 px: 10 and 13 against 5 and
  3 of 484). The ranking within each processed cohort is no better, which
  is the previous point.
- **Clean recall on this cohort is level** at 1% FPR (71.3% on average
  for both arms; the single-seed differences are −2.7 and +2.7 points).
  The clean validation AUC above is slightly lower for chains.
- **Eight thousand more steps of either recipe sit below the starting
  checkpoint on several operations** (JPEG quality 12: 132 at the start,
  then 113, 123, 118 and 113; strong blur: 144, then 112, 103, 125, 136).
  The baseline-minus-start intervals include zero, so this is a pattern
  across rows, not a measured loss on any one.
- **The 0.1% column is not stable enough to read**: the baseline's clean
  figure is 110 in one seed and 136 in the other. Chains are higher there
  in both seeds on the originals, JPEG quality 12, both blurs and both
  noise strengths, and lower in both on the half-size resize and the
  repost to 720 px.

Against the gate (10 points at 1% FPR on processed copies, 0.1% not
lower, clean within 1 point, fewer codec false alarms): on this cohort
only the noise rows reach 10 points, and nothing here measures learned
codecs. That is a reason to expect arm B alone to fall short of the gate
on JPEG and reposts, and to look at why the repost and JPEG operations in
the chains do not transfer to these renderings, before spending the
frozen split on it. It is not a gate result.
