# Processing chains on the frozen split: arms A and B, two seeds

The gate run of [the processing-chains runbook](../../runbooks/processing-chains.md):
the current recipe (arm A, "baseline") against the same recipe with pixel
chains (arm B, "chains"), each trained twice (`--seed 1`, `--seed 2`) on
all 25 phase-6 manifests from the phase-3 checkpoint, 8,000 steps, at
commit `7156565` on an Apple M6 (12 cores, 32 GB, macOS 27.2, GPU engine),
2026-11-14. Each `step_008000` checkpoint was copied under `runs/`, its
cutoff fitted on the frozen split's `calibrate` set
(`tintype calibrate --profile photo`, `--target-fpr 0.01` and `0.001`),
and the `test` set scored with every view. `tintype verify-models` printed
four `ok` lines after every step.

**The chains arm does not pass.** Recall on processed copies at the 1%
cutoff rises by 5.8 points (seed 1) and 7.0 points (seed 2); the gate asks
for 10. Recall at the 0.1% cutoff is not lower (+6.5 and +6.8 points), and
clean recall is within one point in both seeds. Item 4 of the gate cannot
be evaluated: the split has no learned-codec copies.

## What limits this

- `test` holds 200 generated and 200 authentic parents with processed
  copies; every processed figure rests on those 400 parents.
- Every authentic original is a camera JPEG of 12 megapixels or more and
  every generated original a PNG of 256 to 1,024 px, so the clean rows are
  easy. The cost the chains arm shows on confirmation 7 (below) does not
  appear on them.
- The processed copies are twelve single FFmpeg operations. No real
  platform round trip and no learned codec is in the split.
- `test` had been looked at before for the released model.

## The gate quantities

Cutoffs fitted on `calibrate` (raw mean-logit thresholds): for 1%,
baseline 1.077 and 0.855 (seeds 1 and 2), chains −0.129 and −0.222; for
0.1%, baseline 3.002 and 2.774, chains 1.310 and 1.469.

| | Baseline, seed 1 | Chains, seed 1 | Baseline, seed 2 | Chains, seed 2 |
|---|---|---|---|---|
| Processed recall, 1% cutoff | 82.1% | 87.9% | 81.5% | 88.5% |
| Processed recall, 0.1% cutoff | 72.3% | 78.9% | 71.5% | 78.3% |
| Processed authentic flagged, 1% cutoff (of 2,400) | 73 | 58 | 78 | 55 |
| Clean recall, 1% cutoff | 95.6% | 95.2% | 95.4% | 95.7% |
| Clean recall, 0.1% cutoff | 92.9% | 92.4% | 92.6% | 92.8% |
| Clean authentic flagged, 1% cutoff (of 7,326) | 55 | 90 | 81 | 61 |
| Clean authentic flagged, 0.1% cutoff (of 7,326) | 9 | 24 | 12 | 9 |

- **The processed gain holds in both seeds** and its interval excludes
  zero: +5.83 points (+4.67 to +7.25) and +7.04 (+5.75 to +8.46) at the
  1% cutoff, bootstrap over the 200 generated parents.
- **Where it comes from**: heavy noise (17.5% → 61.0% and 10.0% → 55.0%),
  then `jpeg-qv12` (+9.5 and +10.0 points) and the three reposts (+3.5 to
  +9.5). Resizing loses 1.5 to 3 points in both seeds.
- **At each row's own 1% cutoff the JPEG and repost gains mostly go**
  (`jpeg-qv12` 86.0% → 87.0% and 84.5% → 88.5%; `repost-720-qv9x1` 73.5%
  → 71.5% and 77.0% → 76.5%): the chains arm scores those copies higher in
  both classes, and flags more authentic reposts at the frozen cutoff (6
  to 9 of 200 against 1 to 5).
- **Authentic noise copies are flagged far less** by the chains arm at the
  1% cutoff: `noise-a10` 6 and 4 of 200 against 31 and 30.
- **Clean authentic photos flagged differ more between seeds than between
  arms** (baseline 55 and 81, chains 90 and 61), so the split shows no
  difference there.

The full tables, one per seed and cutoff (`scripts/compare-arms.py`):

### Seed 1, cutoffs fitted for 1%

| Rows | chains: authentic flagged | chains: recall | chains: recall at the rows' own 1% cutoff | baseline: authentic flagged | baseline: recall | baseline: recall at the rows' own 1% cutoff |
|---|---|---|---|---|---|---|
| processed, every transform | 58 of 2,400 | 2110 of 2,400 (87.9%) | 81.7% | 73 of 2,400 | 1970 of 2,400 (82.1%) | 74.2% |
| clean | 90 of 7,326 | 6851 of 7,200 (95.2%) | 94.9% | 55 of 7,326 | 6884 of 7,200 (95.6%) | 96.1% |
| blur-s1.0 | 1 of 200 | 192 of 200 (96.0%) | 97.0% | 1 of 200 | 193 of 200 (96.5%) | 97.5% |
| blur-s3.0 | 8 of 200 | 196 of 200 (98.0%) | 98.0% | 5 of 200 | 196 of 200 (98.0%) | 97.5% |
| jpeg-qv12 | 1 of 200 | 166 of 200 (83.0%) | 87.0% | 1 of 200 | 147 of 200 (73.5%) | 86.0% |
| jpeg-qv4 | 2 of 200 | 177 of 200 (88.5%) | 90.0% | 2 of 200 | 170 of 200 (85.0%) | 91.0% |
| noise-a10 | 6 of 200 | 187 of 200 (93.5%) | 90.5% | 31 of 200 | 185 of 200 (92.5%) | 84.0% |
| noise-a40 | 7 of 200 | 122 of 200 (61.0%) | 32.0% | 14 of 200 | 35 of 200 (17.5%) | 1.5% |
| repost-720-qv9x1 | 8 of 200 | 173 of 200 (86.5%) | 71.5% | 5 of 200 | 157 of 200 (78.5%) | 73.5% |
| repost-960-qv12x1 | 7 of 200 | 164 of 200 (82.0%) | 72.5% | 3 of 200 | 148 of 200 (74.0%) | 67.0% |
| repost-960-qv7x4 | 7 of 200 | 173 of 200 (86.5%) | 79.5% | 4 of 200 | 166 of 200 (83.0%) | 81.5% |
| resize-0.25-bicubic | 2 of 200 | 178 of 200 (89.0%) | 91.0% | 1 of 200 | 184 of 200 (92.0%) | 96.5% |
| resize-0.5-bicubic | 5 of 200 | 192 of 200 (96.0%) | 96.0% | 1 of 200 | 196 of 200 (98.0%) | 98.0% |
| sharpen-a1.0 | 4 of 200 | 190 of 200 (95.0%) | 93.5% | 5 of 200 | 193 of 200 (96.5%) | 94.0% |

Processed recall at the frozen cutoffs, chains − baseline: +5.83 points (95% bootstrap over 200 generated parents: +4.67 to +7.25).

### Seed 1, cutoffs fitted for 0.1%

| Rows | chains: authentic flagged | chains: recall | chains: recall at the rows' own 1% cutoff | baseline: authentic flagged | baseline: recall | baseline: recall at the rows' own 1% cutoff |
|---|---|---|---|---|---|---|
| processed, every transform | 18 of 2,400 | 1893 of 2,400 (78.9%) | 81.7% | 15 of 2,400 | 1736 of 2,400 (72.3%) | 74.2% |
| clean | 24 of 7,326 | 6653 of 7,200 (92.4%) | 94.9% | 9 of 7,326 | 6690 of 7,200 (92.9%) | 96.1% |
| blur-s1.0 | 1 of 200 | 188 of 200 (94.0%) | 97.0% | 0 of 200 | 189 of 200 (94.5%) | 97.5% |
| blur-s3.0 | 0 of 200 | 185 of 200 (92.5%) | 98.0% | 0 of 200 | 187 of 200 (93.5%) | 97.5% |
| jpeg-qv12 | 1 of 200 | 139 of 200 (69.5%) | 87.0% | 1 of 200 | 112 of 200 (56.0%) | 86.0% |
| jpeg-qv4 | 1 of 200 | 158 of 200 (79.0%) | 90.0% | 1 of 200 | 146 of 200 (73.0%) | 91.0% |
| noise-a10 | 2 of 200 | 176 of 200 (88.0%) | 90.5% | 6 of 200 | 172 of 200 (86.0%) | 84.0% |
| noise-a40 | 3 of 200 | 79 of 200 (39.5%) | 32.0% | 6 of 200 | 15 of 200 (7.5%) | 1.5% |
| repost-720-qv9x1 | 4 of 200 | 148 of 200 (74.0%) | 71.5% | 0 of 200 | 121 of 200 (60.5%) | 73.5% |
| repost-960-qv12x1 | 2 of 200 | 139 of 200 (69.5%) | 72.5% | 1 of 200 | 114 of 200 (57.0%) | 67.0% |
| repost-960-qv7x4 | 1 of 200 | 155 of 200 (77.5%) | 79.5% | 0 of 200 | 139 of 200 (69.5%) | 81.5% |
| resize-0.25-bicubic | 1 of 200 | 160 of 200 (80.0%) | 91.0% | 0 of 200 | 165 of 200 (82.5%) | 96.5% |
| resize-0.5-bicubic | 1 of 200 | 186 of 200 (93.0%) | 96.0% | 0 of 200 | 190 of 200 (95.0%) | 98.0% |
| sharpen-a1.0 | 1 of 200 | 180 of 200 (90.0%) | 93.5% | 0 of 200 | 186 of 200 (93.0%) | 94.0% |

Processed recall at the frozen cutoffs, chains − baseline: +6.54 points (95% bootstrap over 200 generated parents: +4.83 to +8.33).

### Seed 2, cutoffs fitted for 1%

| Rows | chains: authentic flagged | chains: recall | chains: recall at the rows' own 1% cutoff | baseline: authentic flagged | baseline: recall | baseline: recall at the rows' own 1% cutoff |
|---|---|---|---|---|---|---|
| processed, every transform | 55 of 2,400 | 2124 of 2,400 (88.5%) | 83.4% | 78 of 2,400 | 1955 of 2,400 (81.5%) | 74.4% |
| clean | 61 of 7,326 | 6890 of 7,200 (95.7%) | 96.0% | 81 of 7,326 | 6867 of 7,200 (95.4%) | 95.2% |
| blur-s1.0 | 2 of 200 | 192 of 200 (96.0%) | 96.5% | 2 of 200 | 192 of 200 (96.0%) | 96.0% |
| blur-s3.0 | 8 of 200 | 195 of 200 (97.5%) | 95.5% | 11 of 200 | 196 of 200 (98.0%) | 96.0% |
| jpeg-qv12 | 1 of 200 | 171 of 200 (85.5%) | 88.5% | 1 of 200 | 151 of 200 (75.5%) | 84.5% |
| jpeg-qv4 | 2 of 200 | 181 of 200 (90.5%) | 94.5% | 2 of 200 | 167 of 200 (83.5%) | 88.5% |
| noise-a10 | 4 of 200 | 190 of 200 (95.0%) | 91.5% | 30 of 200 | 183 of 200 (91.5%) | 79.0% |
| noise-a40 | 7 of 200 | 110 of 200 (55.0%) | 38.5% | 12 of 200 | 20 of 200 (10.0%) | 0.5% |
| repost-720-qv9x1 | 9 of 200 | 172 of 200 (86.0%) | 76.5% | 5 of 200 | 159 of 200 (79.5%) | 77.0% |
| repost-960-qv12x1 | 7 of 200 | 170 of 200 (85.0%) | 70.5% | 3 of 200 | 151 of 200 (75.5%) | 69.0% |
| repost-960-qv7x4 | 6 of 200 | 178 of 200 (89.0%) | 83.5% | 1 of 200 | 166 of 200 (83.0%) | 84.5% |
| resize-0.25-bicubic | 3 of 200 | 179 of 200 (89.5%) | 89.5% | 2 of 200 | 182 of 200 (91.0%) | 94.0% |
| resize-0.5-bicubic | 4 of 200 | 193 of 200 (96.5%) | 96.5% | 4 of 200 | 196 of 200 (98.0%) | 98.0% |
| sharpen-a1.0 | 2 of 200 | 193 of 200 (96.5%) | 97.0% | 5 of 200 | 192 of 200 (96.0%) | 93.5% |

Processed recall at the frozen cutoffs, chains − baseline: +7.04 points (95% bootstrap over 200 generated parents: +5.75 to +8.46).

### Seed 2, cutoffs fitted for 0.1%

| Rows | chains: authentic flagged | chains: recall | chains: recall at the rows' own 1% cutoff | baseline: authentic flagged | baseline: recall | baseline: recall at the rows' own 1% cutoff |
|---|---|---|---|---|---|---|
| processed, every transform | 11 of 2,400 | 1880 of 2,400 (78.3%) | 83.4% | 16 of 2,400 | 1716 of 2,400 (71.5%) | 74.4% |
| clean | 9 of 7,326 | 6682 of 7,200 (92.8%) | 96.0% | 12 of 7,326 | 6669 of 7,200 (92.6%) | 95.2% |
| blur-s1.0 | 1 of 200 | 184 of 200 (92.0%) | 96.5% | 1 of 200 | 188 of 200 (94.0%) | 96.0% |
| blur-s3.0 | 0 of 200 | 186 of 200 (93.0%) | 95.5% | 0 of 200 | 187 of 200 (93.5%) | 96.0% |
| jpeg-qv12 | 1 of 200 | 140 of 200 (70.0%) | 88.5% | 1 of 200 | 112 of 200 (56.0%) | 84.5% |
| jpeg-qv4 | 1 of 200 | 154 of 200 (77.0%) | 94.5% | 1 of 200 | 145 of 200 (72.5%) | 88.5% |
| noise-a10 | 1 of 200 | 171 of 200 (85.5%) | 91.5% | 7 of 200 | 168 of 200 (84.0%) | 79.0% |
| noise-a40 | 2 of 200 | 66 of 200 (33.0%) | 38.5% | 6 of 200 | 6 of 200 (3.0%) | 0.5% |
| repost-720-qv9x1 | 2 of 200 | 151 of 200 (75.5%) | 76.5% | 0 of 200 | 121 of 200 (60.5%) | 77.0% |
| repost-960-qv12x1 | 2 of 200 | 139 of 200 (69.5%) | 70.5% | 0 of 200 | 113 of 200 (56.5%) | 69.0% |
| repost-960-qv7x4 | 0 of 200 | 158 of 200 (79.0%) | 83.5% | 0 of 200 | 136 of 200 (68.0%) | 84.5% |
| resize-0.25-bicubic | 1 of 200 | 161 of 200 (80.5%) | 89.5% | 0 of 200 | 164 of 200 (82.0%) | 94.0% |
| resize-0.5-bicubic | 0 of 200 | 188 of 200 (94.0%) | 96.5% | 0 of 200 | 192 of 200 (96.0%) | 98.0% |
| sharpen-a1.0 | 0 of 200 | 182 of 200 (91.0%) | 97.0% | 0 of 200 | 184 of 200 (92.0%) | 93.5% |

Processed recall at the frozen cutoffs, chains − baseline: +6.83 points (95% bootstrap over 200 generated parents: +5.17 to +8.62).

## Diagnostics on development data

None of these chooses anything. Each arm's `candidate` copy, with the 1%
cutoff fitted on the frozen `calibrate` set, scored with every view.

**Confirmation 7** (23,052 photos, 30 sets; [table](confirmation7-report.md)).
Recall at the batch's own empirical cutoffs:

| | Released model, six views | Baseline, seeds 1 and 2 | Chains, seeds 1 and 2 |
|---|---|---|---|
| Recall at 1% empirical FPR | 89.00% | 88.58%, 88.83% | 84.88%, 84.40% |
| Recall at 0.1% empirical FPR | 63.58% | 64.56%, 62.72% | 57.91%, 57.82% |
| Authentic flagged at the cutoff frozen on the split (of 10,747) | 127 at the shipped cutoff | 184, 194 | 319, 338 |

The chains arm ranks these photos worse than the baseline arm in both
seeds, by about 4 points at 1% and 5 to 7 at 0.1%, and the cutoff fitted
on the frozen split flags about 3% of the batch's authentic photos for it
against 1.7% to 1.8%. **This is the cost of the chains, and the frozen
split's clean rows do not show it.**

**NTIRE 2026**, per operation ([baseline, seed 1](ntire-operations.baseline.md),
[seed 2](ntire-operations.baseline-seed2.md), [chains, seed 1](ntire-operations.chains.md),
[seed 2](ntire-operations.chains-seed2.md)):

| | Baseline, seeds 1 and 2 | Chains, seeds 1 and 2 |
|---|---|---|
| Public test, clean: AUROC | 0.883, 0.883 | 0.885, 0.883 |
| Public test, distorted: AUROC | 0.634, 0.635 | 0.648, 0.647 |
| Public test, chains with JPEG AI, Cheng2020 or WMForger: AUROC | 0.541, 0.537 | 0.554, 0.547 |
| Hard validation, clean: AUROC | 0.953, 0.954 | 0.953, 0.954 |
| Hard validation, distorted: AUROC | 0.738, 0.746 | 0.772, 0.767 |
| Public test, clean authentic flagged at the split's 1% cutoff (of 600) | 59, 58 | 100, 109 |

The chains arm ranks the distorted images slightly better and the clean
ones the same. The learned-codec rows stay near chance for both arms. At
the cutoff fitted on the frozen split it flags 17% to 18% of the public
test's clean authentic images against 10%: that cutoff does not carry
over to these images for either arm.

**Frontier pairs**, seed 1 only ([table](frontier-seed1.md); 261
generated and 484 authentic pairs, cutoff on each cohort): the chains arm
gains on `noise-a10` (+24.9 points, +10.3 to +41.8) and `noise-a40`
(+16.9, +3.1 to +30.7) and on nothing else beyond its interval, as in the
[earlier diagnostic](../processing-chain-arms/report.md) on 23 manifests.

## Commands

```bash
# Per arm (ARM: ws1-full-baseline, ws1-full-chains, and the same with -seed2):
tintype train --init <phase-3 checkpoint> <the 25 phase-6 manifests> --out runs/$ARM \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --seed <1|2>     # chains: plus the arm B flags of the runbook
cp runs/$ARM/step_008000.safetensors runs/$ARM/candidate.safetensors; cp runs/$ARM/step_008000.json runs/$ARM/candidate.json
tintype score --weights runs/$ARM/candidate.safetensors --manifest data/frozen-eval/calibrate.tsv --out runs/$ARM/calibrate.tsv --all-views
tintype calibrate --weights runs/$ARM/candidate.safetensors --scores runs/$ARM/calibrate.tsv --profile photo --target-fpr 0.01
tintype verify-models
tintype score --weights runs/$ARM/candidate.safetensors --manifest data/frozen-eval/test.tsv --out runs/$ARM/test.tsv --all-views

# The tables above, from the archive beside this file:
python3 -I scripts/compare-arms.py test-groups.tsv.gz chains=test.chains.tsv.gz baseline=test.baseline.tsv.gz
python3 -I scripts/compare-arms.py test-groups.tsv.gz chains=test.chains.tsv.gz@1.310 baseline=test.baseline.tsv.gz@3.002
```

The archive holds each arm's `calibrate` and `test` score files
(`path label generator dataset prob` and six view logits; paths relative
to the directory that holds `data/`) and `test-groups.tsv.gz` (path and
the five grouping columns of the frozen `test` set). The 0.1% rows use
the raw cutoffs listed above on the same score files. All four training
runs logged 0% data wait in every interval.
