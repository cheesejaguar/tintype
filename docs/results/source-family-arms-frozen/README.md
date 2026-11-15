# Source and family arms on the frozen split: arms B and C, one seed each

The gate run of [the source and family runbook](../../runbooks/source-family-matching.md):
the current mix (the control, two seeds) against the same recipe with
family weights (arm B, "family") and with Reddit-posted generated images
drawn as often as Reddit-posted photos (arm C, "source"). Every run starts
from the phase-3 checkpoint on the 25 phase-6 manifests: 8,000 steps,
batch 32, learning rate 2e-5, warmup 300, messenger probability 0.25.
Arms B and C were trained on 2026-11-14 with `--seed 1` at commit `f59da2e`
on an Apple M6 (12 cores, 32 GB, macOS 27.2, GPU engine). What was fixed
before the first look is in
[PREREGISTRATION.md](../../PREREGISTRATION.md#source-and-family-matching-on-the-frozen-split-what-was-fixed-before-the-first-look).

**The control was not trained here.** It is the two current-mix runs that
workstream 1 trained on the same manifests from the same checkpoint with
the same budget (`--seed 1`, `--seed 2`, no chain flags), built from that
workstream's branch at `7156565`; their `calibrate` and `test` score files
are used as that workstream wrote them. The recipe is arm A of the
runbook; the build is a different one.

## Verdict

- **Arm B, family-weighted: does not pass the gate as written, by one
  image.** New-family macro recall is higher by 1.69 points at the 1%
  cutoff (95% interval +1.15 to +2.16), three times the 0.54 points the
  control moves between its own seeds, and by 1.60 points at 0.1%. Older
  families and processed copies are not lower (processed recall +2.46 and
  +2.12 points). The condition that fails is the false-positive rate on
  processed authentic copies at 0.1%: 16 of 2,400 flagged against the
  control's 15, which is what the control's second seed also flags. At 1%
  it is 71 against 73. One seed.
- **Arm C, source-matched: fails.** New-family macro recall is lower at
  both cutoffs (−1.23 points at 1%, interval −1.70 to −0.76; −0.72 at
  0.1%) and recall on processed copies is 1.75 points lower at 1%, more
  than the one point allowed. Its frozen cutoffs land at a lower
  false-positive rate than the control's (42 clean authentic photos
  flagged against 55), and at the empirical cutoffs of `test` it ranks
  like the control (91.45% against 91.49% recall at 1%), so the arm moved
  the operating point, not the ranking. The simulated repost rows, the
  only place it could have helped, are not better.

Neither arm is a ship candidate. Arm B is the one worth a second seed.

## What limits this

- One run per arm. The control's two seeds differ by 0.54 points of
  new-family macro recall, 0.62 of processed recall and 26 clean authentic
  photos flagged (55 and 81 of 7,326) at the 1% cutoff; an arm's
  difference is set beside those throughout.
- Every authentic original is a camera JPEG of 12 megapixels or more and
  every generated original a PNG of 256 to 1,024 px. The clean rows carry
  that difference; the processed copies re-encode both classes the same
  way.
- The authentic photos are few scenes and devices: FloreView is 36 scenes
  photographed by 46 phones, CSAFE is ten units of one phone model.
  Intervals resample parents and are narrower than the scenes warrant.
- The split has no real platform round trip and no learned-codec copy.
  Arm C's question is about platform-processed images; twelve simulated
  operations on 200 parents per class are a weak test of it.
- "New families" are six autoregressive and tokenizer generators from one
  collection (ARForensics). Two of them (`open-magvit2`, `var`) carry
  almost all of the difference; the other four are found 98.5% to 100% of
  the time by every model.
- `test` had been looked at before, for the released model and for
  workstream 1's arms. These are two more looks, one per arm.

## The gate quantities

Cutoffs fitted on `calibrate` (raw mean-logit thresholds), 1% and 0.1%:
control 1.077 and 3.002, control seed 2 0.855 and 2.774, family 1.046 and
2.859, source 1.243 and 2.886. [Full output](gate.md), with every family
and every authentic source.

At the 1% cutoffs:

| | Control | Control, seed 2 | Family (B) | Source (C) | Seed 2 − control | B − control | C − control |
|---|---|---|---|---|---|---|---|
| New-family macro recall | 91.54% | 91.00% | 93.23% | 90.31% | −0.54 [−1.02, −0.07] | +1.69 [+1.15, +2.16] | −1.23 [−1.70, −0.76] |
| Older-family macro recall | 99.71% | 99.83% | 99.92% | 99.42% | +0.13 [+0.00, +0.35] | +0.21 [+0.00, +0.48] | −0.29 [−0.62, +0.00] |
| Recall, processed copies | 82.08% | 81.46% | 84.54% | 80.33% | −0.62 [−1.78, +0.50] | +2.46 [+1.40, +3.61] | −1.75 [−2.83, −0.56] |
| Recall, unprocessed | 95.61% | 95.38% | 96.56% | 94.93% | −0.24 [−0.49, +0.01] | +0.94 [+0.67, +1.19] | −0.68 [−0.94, −0.43] |
| Processed authentic copies flagged (of 2,400) | 73 | 78 | 71 | 53 | | | |
| Unprocessed authentic photos flagged (of 7,326) | 55 | 81 | 48 | 42 | | | |

At the 0.1% cutoffs:

| | Control | Control, seed 2 | Family (B) | Source (C) | Seed 2 − control | B − control | C − control |
|---|---|---|---|---|---|---|---|
| New-family macro recall | 87.16% | 86.35% | 88.76% | 86.44% | −0.81 [−1.38, −0.30] | +1.60 [+1.03, +2.12] | −0.72 [−1.15, −0.23] |
| Older-family macro recall | 98.71% | 99.08% | 98.75% | 98.92% | +0.38 [+0.12, +0.67] | +0.04 [−0.24, +0.37] | +0.21 [−0.05, +0.49] |
| Recall, processed copies | 72.33% | 71.50% | 74.46% | 71.88% | −0.83 [−1.95, +0.26] | +2.12 [+1.06, +3.14] | −0.46 [−1.35, +0.47] |
| Recall, unprocessed | 92.92% | 92.62% | 93.83% | 92.62% | −0.29 [−0.58, −0.01] | +0.92 [+0.61, +1.22] | −0.29 [−0.55, −0.04] |
| Processed authentic copies flagged (of 2,400) | 15 | 16 | 16 | 12 | | | |
| Unprocessed authentic photos flagged (of 7,326) | 9 | 12 | 9 | 6 | | | |

Differences are in points, with a 95% interval from a bootstrap over
parents.

### Against the gate

| Condition | Arm B, family | Arm C, source |
|---|---|---|
| 1. New-family macro recall higher at 1%, interval above zero, and more than the control's seed difference (0.54) | **met**: +1.69 [+1.15, +2.16] | **not met**: −1.23 |
| 1. Not lower at 0.1% | **met**: +1.60 | **not met**: −0.72 (the control's second seed: −0.81) |
| 2. Older-family macro recall not lower by more than one point, both rates | **met**: +0.21, +0.04 | **met**: −0.29, +0.21 |
| 2. Recall on processed copies not lower by more than one point, both rates | **met**: +2.46, +2.12 | **not met at 1%**: −1.75; −0.46 at 0.1% |
| 3. False-positive rate on processed authentic copies not higher, both rates | met at 1% (71 against 73); **not met at 0.1%** (16 against 15; the control's second seed flags 16) | **met**: 53 against 73, 12 against 15 |
| Passes | **no**, on condition 3 at 0.1% by one image, inside the control's seed range | **no** |

### New families

Recall on unprocessed items, at the 1% cutoffs (0.1% in brackets):

| Family | Items | Control | Control, seed 2 | Family (B) | Source (C) |
|---|---|---|---|---|---|
| `infinity` (held out whole) | 1,500 | 100.00% (99.33%) | 100.00% (99.40%) | 100.00% (99.33%) | 99.93% (99.20%) |
| `rar` (held out whole) | 1,500 | 99.27% (98.27%) | 99.33% (98.20%) | 99.73% (99.20%) | 99.27% (98.27%) |
| `llamagen` | 600 | 98.50% (96.50%) | 98.17% (96.17%) | 99.00% (98.00%) | 98.50% (96.67%) |
| `switti` | 600 | 99.83% (98.83%) | 100.00% (99.33%) | 100.00% (99.17%) | 99.83% (99.33%) |
| `var` | 600 | 80.50% (70.50%) | 79.33% (67.50%) | 84.00% (72.17%) | 76.83% (67.50%) |
| `open-magvit2` | 600 | 71.17% (59.50%) | 69.17% (57.50%) | 76.67% (64.67%) | 67.50% (57.67%) |

The two held-out families are found almost completely by every model, so
they say nothing about the arms. The difference is in `var` and
`open-magvit2`: arm B finds 3.5 and 5.5 points more of them than the
control at 1% (the control's seeds differ by 1.2 and 2.0), arm C 3.7 and
3.7 points fewer. The older families (`flux`, `sd3`, `janus-pro`) are at
98.75% or more for every model at 1%; halving the share of latent
diffusion draws in arm B cost nothing that this split can see.

### Processed copies

Recall and authentic copies flagged at each model's 1% cutoff, 200 parents
per class and transform ([full tables](operating-points.md)):

| Transform | Recall: control | control, seed 2 | family (B) | source (C) | Authentic flagged of 200: control | control, seed 2 | family (B) | source (C) |
|---|---|---|---|---|---|---|---|---|
| blur-s1.0 | 96.50% | 96.00% | 97.00% | 98.00% | 1 | 2 | 2 | 1 |
| blur-s3.0 | 98.00% | 98.00% | 99.00% | 98.50% | 5 | 11 | 14 | 8 |
| jpeg-qv12 | 73.50% | 75.50% | 80.00% | 70.50% | 1 | 1 | 1 | 1 |
| jpeg-qv4 | 85.00% | 83.50% | 88.50% | 82.00% | 2 | 2 | 2 | 1 |
| noise-a10 | 92.50% | 91.50% | 92.00% | 91.00% | 31 | 30 | 19 | 19 |
| noise-a40 | 17.50% | 10.00% | 18.00% | 10.50% | 14 | 12 | 13 | 10 |
| repost-720-qv9x1 | 78.50% | 79.50% | 83.50% | 77.00% | 5 | 5 | 5 | 5 |
| repost-960-qv12x1 | 74.00% | 75.50% | 79.50% | 71.00% | 3 | 3 | 3 | 3 |
| repost-960-qv7x4 | 83.00% | 83.00% | 86.50% | 80.50% | 4 | 1 | 5 | 2 |
| resize-0.25-bicubic | 92.00% | 91.00% | 96.00% | 91.50% | 1 | 2 | 1 | 1 |
| resize-0.5-bicubic | 98.00% | 98.00% | 98.50% | 98.00% | 1 | 4 | 3 | 1 |
| sharpen-a1.0 | 96.50% | 96.00% | 96.00% | 95.50% | 5 | 5 | 3 | 1 |

- Arm B's gain is on the JPEG, repost and strong-resize copies (3.5 to
  6.5 points over the control, 3.5 to 5 over its second seed) and nowhere
  on heavy noise, which stays near 18%.
- Arm B flags more authentic `blur-s3.0` copies than either control seed
  (14 against 5 and 11) and fewer `noise-a10` copies (19 against 31 and
  30).
- Arm C's repost rows are 1.5 to 3 points below the control's at its
  frozen cutoff, with as many or fewer authentic reposts flagged.

### Authentic sources

Unprocessed photos flagged at the 1% cutoffs:

| Source | Photos | Control | Control, seed 2 | Family (B) | Source (C) |
|---|---|---|---|---|---|
| `csafe-iphone14pro` (held out whole) | 3,996 | 34 | 62 | 36 | 24 |
| `hdrplus` | 1,456 | 6 | 8 | 4 | 6 |
| FloreView, eleven brands together | 1,874 | 15 | 11 | 8 | 12 |
| of which `floreview-xiaomi` | 292 | 6 | 6 | 4 | 6 |
| of which `floreview-doogee` | 42 | 4 | 2 | 3 | 4 |

No source is flagged more by arm B than by the control, except the
held-out phone by two photos (36 against 34), and that source moves far
more between the control's own seeds (34 and 62). At the 0.1% cutoffs the
four models flag 9, 12, 9 and 6 photos, 6, 12, 6 and 5 of them from the
held-out phone.

## At the empirical cutoffs of `test` (a ranking measure, not a transfer result)

Recall over all generated rows of `test`, clean and processed together, at
the cutoff that flags 1% and 0.1% of its authentic rows:

| | Control | Control, seed 2 | Family (B) | Source (C) |
|---|---|---|---|---|
| Recall at 1% empirical FPR | 91.49% | 90.61% | 93.22% | 91.45% |
| Recall at 0.1% empirical FPR | 85.10% | 82.90% | 86.66% | 85.92% |
| Authentic rows flagged at the frozen 1% cutoff (of 9,726) | 128 | 159 | 119 | 95 |

Arm B ranks better than both control seeds at both rates. Arm C ranks
like the control: its lower recall at the frozen cutoff comes with fewer
authentic rows flagged (95 against 128).

## Diagnostic: confirmation batch 7 (archived development data; chooses nothing)

23,052 photos in 30 sets, every view, compared at empirical cutoffs
([output](confirm7.md)):

| | Control | Control, seed 2 | Family (B) | Source (C) |
|---|---|---|---|---|
| Recall at 1% empirical FPR | 88.58% | 88.83% | 88.48% | 87.87% |
| Recall at 0.1% empirical FPR | 64.56% | 62.72% | 67.73% | 63.60% |

Arm B does not lose ranking on batch 7: level with the control at 1% (0.1
and 0.35 points below its two seeds, inside the intervals) and 3.2 and 5.0
points above them at 0.1%. Arm C is 0.7 and 1.0 points below the control's
seeds at 1% and between them at 0.1%.

## Training

| | Control | Control, seed 2 | Family (B) | Source (C) |
|---|---|---|---|---|
| Validation, fused AUROC at step 8,000 | 0.9952 | 0.9971 | 0.9961 | 0.9957 |
| Wall time | – | – | 63.5 min | 58.4 min |
| Data wait | – | – | 0% in all 160 intervals | 0% in all 160 intervals |

The validation pool is the same 2,000 files for every run. The GPU was
shared with another job's scoring for part of arm B's run and with arm
B's own scoring at the start of arm C's.

## Not run

- **Arm D** (reconstructions): the reconstruction stores do not exist.
- **The refresh arms** (R0, R1, R2): they need stores of new generators
  that are neither in training nor in the frozen split. None is on disk.
- **A second seed of either arm.**
- **Latency and throughput**: neither arm changes the network, the views
  or the spec, so they are the control's.

## Commands

From the directory that holds `data/` and `runs/` (a worktree whose `data/`
links to the local dataset tree); `<phase-3 checkpoint>` is
`runs/v4/step_007000.safetensors` of that tree and `runs/v7/manifests.txt`
holds the 25 phase-6 manifests as `--manifest <path>` pairs.

```bash
# The family map and the weights of arm B (CPU).
python3 -I scripts/group-manifest.py $(sed "s/--manifest //g" runs/v7/manifests.txt) \
  --families docs/groups/generator-families.tsv --dry-run --write-map runs/ws4/families.tsv
tintype sampling-shares $(cat runs/v7/manifests.txt) --family-map runs/ws4/families.tsv \
  --rebalance-families 0.5 --write-weights runs/ws4/family-weights.tsv

# Arm B.
tintype train --init <phase-3 checkpoint> $(cat runs/v7/manifests.txt) \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --p-messenger 0.25 --seed 1 \
  --family-map runs/ws4/families.tsv --weights-file runs/ws4/family-weights.tsv --family-weight other=1 \
  --out runs/ws4-family
# Arm C.
tintype train --init <phase-3 checkpoint> $(cat runs/v7/manifests.txt) \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --p-messenger 0.25 --seed 1 \
  --family-weight reddit-wild=3.85 \
  --out runs/ws4-source

# Per arm (ARM: ws4-family, ws4-source): two copies of the last checkpoint, one per cutoff.
cp runs/$ARM/step_008000.safetensors runs/$ARM/candidate.safetensors;        cp runs/$ARM/step_008000.json runs/$ARM/candidate.json
cp runs/$ARM/step_008000.safetensors runs/$ARM/candidate-strict.safetensors; cp runs/$ARM/step_008000.json runs/$ARM/candidate-strict.json
tintype score --weights runs/$ARM/candidate.safetensors --manifest data/frozen-eval/calibrate.tsv --out runs/$ARM/calibrate.tsv --all-views
tintype calibrate --weights runs/$ARM/candidate.safetensors        --scores runs/$ARM/calibrate.tsv --profile photo --target-fpr 0.01
tintype calibrate --weights runs/$ARM/candidate-strict.safetensors --scores runs/$ARM/calibrate.tsv --profile photo --target-fpr 0.001
tintype verify-models                                              # four ok lines, every time
tintype score --weights runs/$ARM/candidate.safetensors --manifest data/frozen-eval/test.tsv --out runs/$ARM/test.tsv --all-views
tintype score --weights runs/$ARM/candidate.safetensors $(printf -- '--manifest %s ' data/fresh8/*/manifest.tsv) \
  --out runs/$ARM/confirm7.tsv --all-views
tintype verify-models
```

A model's raw cutoff is `-logit_bias / logit_scale` of its calibrated
spec. The tables are written from the archive beside this file by the
commands at the top of [gate.md](gate.md),
[operating-points.md](operating-points.md) and [confirm7.md](confirm7.md).

## The archive

| File | What |
|---|---|
| `test.{control,control-seed2,family,source}.tsv.gz` | `test` scored with every view: `path label generator dataset prob` and six view logits. `prob` is under the model's own 1% calibration |
| `calibrate.{family,source}.tsv.gz` | `calibrate` scored with every view, before calibration (the control's are in workstream 1's archive) |
| `confirm7.{control,control-seed2,family,source}.tsv.gz` | Confirmation batch 7, the first five columns |
| `test-groups.tsv.gz` | The frozen `test` manifest: path, label, generator, dataset and the five grouping columns |

Paths are relative to the directory that holds `data/` (the frozen split's
files are under `data/frozen-src` there).
