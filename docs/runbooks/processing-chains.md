# Runbook: training photos through matched processing chains

Workstream 1 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
(section 1). The question: does the current ViT, trained through
label-symmetric processing chains, hold its ranking on processed files it
has not seen, without flagging more authentic ones?

**Status.** The code, the audit tooling and this runbook exist and are
tested. **No arm has been trained.** The audit numbers at the end come
from archived scores and CPU re-scoring of development sets; they are
exploratory. The split that decides between the arms
([frozen-eval.md](frozen-eval.md)) has not been built yet.

## The gate, stated before anything is trained

The report's target is a *proposed* decision criterion. An arm passes if,
against the current recipe re-run at the same budget (arm A below), on the
`test` set of the frozen split, at cutoffs frozen on its `calibrate` set:

1. **+10 recall points at the 1% false-positive operating point on the
   processed copies** (every `transform` other than none). The processed
   copies are rendered by FFmpeg (`scripts/stress-photo.py`), not by the
   training augmentation, and their parents are in no training manifest.
2. **Recall at the 0.1% operating point on the same copies is not lower.**
   It is always reported beside the 1% figure. An arm that gains at 1% and
   loses at 0.1% does not pass.
3. **Clean recall within 1 point** of arm A, on unprocessed generated
   items, at both operating points.
4. **Fewer learned-codec false alarms at a frozen cutoff**: on the
   authentic `codec-*` copies of `test`, the count flagged at the cutoff
   frozen on `calibrate` is lower than arm A's, and the false-positive
   rate over every other transform slice is not higher.

The same quantities at the empirical cutoffs of `test` itself are printed
beside them as a ranking measure and labelled as such. Passing makes an arm
a ship candidate under the [selection rule](frozen-eval.md#the-selection-rule),
nothing more: it still needs its calibration redone, its early exit
re-checked, a written [PREREGISTRATION.md](../PREREGISTRATION.md) entry and
a fresh blind batch. Confirmation 7, Arena, the Frontier pairs and the
released NTIRE sets are diagnostics for every arm and choose nothing.

Arms are compared only on the frozen split. A model initialised from the
phase-3 or phase-6 weights was trained on the path-hash partition (3% of
each store held out by a hash of the path); a group-hash split of those
same stores is not held-out data for it, so neither calibrate nor select
on one.

## What was built

| Piece | Where | Default |
|---|---|---|
| Processing chains in the training augmentation | `augment()` in `src/data.rs`; operations on `Rgb` in `src/image_io.rs` | Off. With no new flag the stream of training views is byte-identical to the phase-6 recipe (test `default_augmentation_is_byte_identical_to_the_recipe_before_chains`) |
| Loader timing on the CPU | `tintype augment-bench` | – |
| Paired per-operation suites | `scripts/stress-photo.py --operations / --transform / --external` | The three Frontier JPEG inputs, byte for byte |
| Per-operation table with paired columns | `scripts/report-stress.py`; `tintype report --operating-points --slice-by transform` | – |
| Every NTIRE operation and severity | `scripts/diagnose-ntire.py --operations` | The transfer report, unchanged (needs Pillow) |
| Offline codec and enhancement stores | `scripts/build-codec-store.py`, `tintype hash-split` | – |

### The new `tintype train` flags

`augment()` takes no label: every operation is applied to real and
generated items alike.

| Flag | Default | Meaning |
|---|---|---|
| `--p-chain` | `0` | Probability that a view goes through a processing chain. `0` draws nothing from the generator. |
| `--chain-ops lo,hi` | `1,3` | Operations per chain, uniform, inclusive. |
| `--chain-severity` | `0.5` | 0 to 1; scales the top of every operation's range. At 1: resize factor 0.25–1 (one in five an enlargement up to 2×), JPEG quality 30–95, blur sigma up to 3 px, noise sigma up to 16 of 255, unsharp amount up to 2. |
| `--chain-kinds` | `resize,jpeg,blur,noise,sharpen,repost` | Operations drawn from, equally likely. Also available: `motion-blur`, `lens-blur`, `shot-noise`, `impulse-noise`, `pixelate`, `tone`. |
| `--chain-post` | `0.25` | Share of chains applied to the finished 224 px view (processing at the scale the network sees). The rest run on the source image, before the crop, as real processing does. |
| `--chain-max-pixels` | `4000000` | A chain on a larger source first cuts a random window of this many pixels (same aspect ratio), and again after a step that enlarged it. Skipping large images instead would make how often an item is processed depend on its size, and size differs between sources and so between classes. |
| `--chain-schedule step:p:max_ops:severity,...` | empty | Curriculum: from each step on, replaces `--p-chain`, the upper end of `--chain-ops` and `--chain-severity`. |
| `--messenger-cap lo,hi` | `960,1700` | Long-side cap of a messenger trip, px. |
| `--messenger-quality lo,hi` | `55,90` | JPEG quality of a messenger trip. |
| `--messenger-cycles lo,hi` | `1,1` | Trips in a row for a messenger sample. `1,1` draws nothing extra. |

Order of draws with everything on: chain (if it runs on the source image),
messenger trips or JPEG, resampling kernel, view geometry, chain (if it
runs on the finished view), JPEG, flip. The codecs the `image` crate could
add (lossy WebP, JPEG 2000) are not available as configured, and lossy
AVIF would pull an encoder into the runtime: those and every learned codec
stay offline (below).

## 0. Before anything runs

Run from a checkout that has `data/` (the stores) and `runs/` beside it.
On the development machine the stores and earlier checkpoints are under
`/Users/aaron/Code/veracity-scratch`; link `data/store` and `data/vstore`
from there and keep `runs/` your own.

```bash
cargo build --release --locked --bin tintype
./target/release/tintype verify-models            # the released files are untouched; repeat after every step below

# The phase-3 checkpoint all three arms start from.
INIT=/Users/aaron/Code/veracity-scratch/runs/v4/step_007000.safetensors
shasum -a 256 "$INIT"     # 2d11bffd77f246416d85a85bb2ce06cee60f28789e76aaa3e4d6e6e739ce7197

# The phase-6 manifests, in the order of the phase-6 run (arrays: the same in bash and zsh).
PHOTO_MANIFESTS=(); MANIFESTS=()
for s in openfake openfake2 openfake-reddit frontier frontier-heldout nano-banana defactify commfor-small \
         openfake-test-reals sofake-set commfor-small2 tiny-genimage stylegan3 tpdne ip-adapter celeba-hq; do
  PHOTO_MANIFESTS+=(--manifest data/store/$s/manifest.tsv)
done
MANIFESTS=("${PHOTO_MANIFESTS[@]}")
for s in vidguard genbuster genvideo100k aigvd k400 pexels k400b pexelsb aigvd2; do
  MANIFESTS+=(--manifest data/vstore/$s/manifest.tsv)
done

# Every listed file must exist: the loader skips unreadable files silently.
python3 -c 'import os, sys
for m in sys.argv[1:]:
    if m == "--manifest": continue
    rows = [l.split("\t")[0] for l in open(m)]
    gone = sum(not os.path.exists(os.path.join(os.path.dirname(m), r)) for r in rows)
    if gone: print(f"{m}: {gone} of {len(rows)} files missing")' "${MANIFESTS[@]}"
# Expect no output.
```

`README.md` and [DATA.md](../DATA.md#training) give the phase-6 recipe:
8,000 steps from the phase-3 weights at `--lr 2e-5 --warmup 300`, batch 32
and every other flag at its default (so `--p-messenger 0.25`). The phase-6
log (`runs/v7/train.log`) shows the same schedule, 356,038 training items
from those 25 manifests and about 80 images/s, which is roughly 55 minutes
per arm.

**Two things found while writing this (2026-11-14):**

- The phase-3 checkpoint is `runs/v4/step_007000.safetensors` (README:
  `runs/v2` 14,000 steps, `runs/v3` 7,000 at 2.5e-5, `runs/v4` 7,000 at
  2e-5). `runs/v3/final_v3.safetensors` is byte-identical to
  `runs/v3/step_007000.safetensors`, the *phase-2* weights. A zero-step
  `tintype train` from each printed, on 1,812 readable validation images:
  from `v4/step_007000` fused AUC 0.9944 (fit 0.9933, native 0.9832); from
  `v3/final_v3` fused AUC 0.9901. The phase-6 log's initial evaluation is
  0.9932 on 2,000 images.
- On the development machine the files of two phase-6 stores are gone:
  `data/store/tiny-genimage` (27,979 listed) and `data/store/stylegan3`
  (1,999 listed) have manifests and empty directories. Restore them
  (`tintype data phase6`) before training, or all three arms silently
  train without Tiny-GenImage and StyleGAN3 and arm A is not the phase-6
  recipe.

## 1. Audit first: paired originals and derivatives

Measure, for every operation and severity, false flags on authentic media
beside recall and AUROC, before training anything.

```bash
# Render the per-operation suite from a manifest of originals (FFmpeg on PATH).
python3 scripts/stress-photo.py <originals.tsv> runs/ws1-audit/suite --operations all --jobs 4
#   -> original.tsv, one <transform>.tsv per severity, paired.tsv (everything);
#      parent_id in column 5 of every row, the recipe in column 9.
#   Lossless operations are stored as PNG and can be large: a 10-megapixel
#   set needs tens of megabytes per image and variant. Use --transform NAME
#   (repeatable) for a subset, or render, score and delete one at a time.

# Score with a copy of the weights under runs/ (CPU works at any size; drop --device for the GPU).
mkdir -p runs/ws1-audit && cp models/tintype.safetensors models/tintype.json runs/ws1-audit/
tintype score --device cpu --weights runs/ws1-audit/tintype.safetensors \
  --manifest runs/ws1-audit/suite/paired.tsv --out runs/ws1-audit/scores.tsv

# Per-transform rates at the frozen cutoff, AUROC, worst slice, intervals over parents.
tintype report --operating-points --system "tintype=runs/ws1-audit/scores.tsv@0.5" \
  --fpr 0.001,0.01 --slice-by transform --cluster-by parent \
  --groups runs/ws1-audit/suite/paired.tsv --min-slice 100

# The paired columns: flips against the original, recall at 1% and 0.1% FPR
# per transform, and what the originals' 1% cutoff does after processing.
python3 scripts/report-stress.py --scores runs/ws1-audit/scores.tsv --manifest runs/ws1-audit/suite/paired.tsv
```

The transform names are the recipe (`resize-0.5-bicubic`, `jpeg-qv4`,
`blur-s1.5`, `noise-a10`, `sharpen-a1.0`, `repost-960-qv7x2`, and chains
joined with `+`); `python3 scripts/stress-photo.py --help` has the grammar.
`-q:v` is FFmpeg's quantiser scale, not libjpeg quality. `jpeg-qv4` and
`repost-960-qv12x1` are the two legacy Frontier variants.

**Real upload round trips** cannot be scripted. They go into the same
manifests:

```bash
python3 scripts/stress-photo.py <originals.tsv> runs/ws1-audit/kit-suite --upload-kit runs/ws1-audit/upload-kit
#   upload-kit/000000.jpg, 000001.png, ... and kit.tsv. Send them through the
#   platform by hand, save what comes back under the same six-digit name
#   (the extension may change) in e.g. runs/ws1-audit/whatsapp-ios/.
python3 scripts/stress-photo.py <originals.tsv> runs/ws1-audit/upload-suite \
  --external upload-whatsapp-ios=runs/ws1-audit/whatsapp-ios
#   -> upload-whatsapp-ios.tsv with the parents' ids; record the app, its
#      version, the device and the date beside it.
```

**Every NTIRE operation.** From the two pinned label files
([EVIDENCE.md](../EVIDENCE.md#external-comparisons)) and the archived
scores; standard library only:

```bash
python3 scripts/diagnose-ntire.py /path/to/ntire-archives --operations
```

For another checkpoint, unpack a split and score it:

```bash
python3 scripts/diagnose-ntire.py /path/to/ntire-archives --extract public-test=runs/ws1-audit/ntire-public
tintype score --weights runs/ws1-<arm>/candidate.safetensors \
  --manifest runs/ws1-audit/ntire-public/manifest.tsv --out runs/ws1-<arm>/ntire-public.tsv
python3 scripts/diagnose-ntire.py /path/to/ntire-archives --operations --scores public-test=runs/ws1-<arm>/ntire-public.tsv
```

The default report of that script (threshold transfer and image
subgroups) reads the images and needs Pillow, which is not in the standard
library; install it in a virtual environment, not the system Python.

## 2. Loader cost, on the CPU, before a GPU run

`tintype train` prints "data wait %": the share of a step the GPU spent
waiting for the loader. Chains add CPU work per view. Time it first:

```bash
tintype augment-bench "${PHOTO_MANIFESTS[@]}" --images 768 --rounds 4 --threads 4            # the phase-6 recipe
tintype augment-bench "${PHOTO_MANIFESTS[@]}" --images 768 --rounds 4 --threads 4 <chain flags of the arm>
```

It draws
photos the way training does and prints decode and augmentation time per
view and views/s per loader thread. The trainer consumes about 80 images/s
(phase-6 log) from 11 loader threads, so an arm needs more than about
7.3 views/s per thread; if it has less, lower `--chain-max-pixels` or raise
`--threads`. Measured values are in [the results below](#measured-so-far).

## 3. The three arms, same budget

Same initial weights, manifests, steps, batch, learning-rate schedule and
seed. `tintype train` writes only under `--out`; it has no `--weights`.
One arm at a time (the GPU is shared).

```bash
# Arm A: the current recipe.
tintype train --init "$INIT" "${MANIFESTS[@]}" --out runs/ws1-baseline \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --seed 1

# Arm B: pixel chains. Mild chains through warm-up, then longer and stronger;
# messenger trips down to 720 px and up to four in a row.
tintype train --init "$INIT" "${MANIFESTS[@]}" --out runs/ws1-chains \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --seed 1 \
  --p-chain 0.5 --chain-ops 1,3 --chain-kinds resize,jpeg,blur,noise,sharpen,repost \
  --chain-schedule 0:0.5:1:0.25,1000:0.5:2:0.5,3000:0.5:3:0.75 \
  --messenger-cap 720,2048 --messenger-cycles 1,4

# Arm C: arm B plus the offline codec and enhancement stores (section 5).
tintype train --init "$INIT" "${MANIFESTS[@]}" "${CODEC_MANIFESTS[@]}" --out runs/ws1-chains-codec \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --seed 1 \
  --p-chain 0.5 --chain-ops 1,3 --chain-kinds resize,jpeg,blur,noise,sharpen,repost \
  --chain-schedule 0:0.5:1:0.25,1000:0.5:2:0.5,3000:0.5:3:0.75 \
  --messenger-cap 720,2048 --messenger-cycles 1,4
```

Expected in each `--out`: `train.log` (arms B and C start with an
"augmentation beyond the phase-6 recipe" line; check it), `step_00N000`
checkpoints with sidecars, `last.safetensors`. Read "data wait" in the
log: well above 0% means the loader, not the GPU, set the pace.

Arm B's setting is one predeclared point, not a tuned one. The loader
threads make a run non-deterministic, so a single run per arm cannot
separate a small effect from run-to-run variation: if the budget allows,
train arm A twice (`--seed 2 --out runs/ws1-baseline-seed2`) and read the
gap between the two as the noise floor. Optional ablations, each one more
run and each reported whatever it shows: arm B without the curriculum
(`--chain-severity 0.5`, no `--chain-schedule`); arm B with the extra
operations (`--chain-kinds` plus `motion-blur,lens-blur,shot-noise,impulse-noise,pixelate,tone`).

## 4. Comparing the arms

On the frozen split only, with the commands and tables of
[frozen-eval.md](frozen-eval.md#what-is-measured). For each arm:

```bash
ARM=ws1-chains      # then ws1-baseline, ws1-chains-codec
cp runs/$ARM/step_008000.safetensors runs/$ARM/candidate.safetensors
cp runs/$ARM/step_008000.json        runs/$ARM/candidate.json
W=runs/$ARM/candidate.safetensors

# Cutoff from the calibration set's authentic items; calibrate rewrites
# candidate.json, which is why W is a copy under runs/.
tintype score --weights $W --manifest data/frozen-eval/calibrate.tsv --out runs/$ARM/calibrate.tsv --all-views
tintype calibrate --weights $W --scores runs/$ARM/calibrate.tsv --profile photo --target-fpr 0.01
tintype verify-models

# Score the test set after calibrating, so that 0.5 is the frozen cutoff.
tintype score --weights $W --manifest data/frozen-eval/test.tsv --out runs/$ARM/test.tsv --all-views
```

Then one report over the three arms, and the paired table per arm:

```bash
tintype report --operating-points \
  --system "baseline=runs/ws1-baseline/test.tsv@0.5" \
  --system "chains=runs/ws1-chains/test.tsv@0.5" \
  --system "chains+codec=runs/ws1-chains-codec/test.tsv@0.5" \
  --fpr 0.001,0.01 --prevalence 0.01 \
  --slice-by transform,family,source --cluster-by parent \
  --groups data/frozen-eval/test.tsv --min-slice 100

python3 scripts/report-stress.py --scores runs/$ARM/test.tsv --manifest data/frozen-eval/test.tsv
```

The gate reads: the `transform` slices other than `none` (item 1 and 2),
the `none` slice (item 3), the `codec-*` slices' "real items flagged"
(item 4). The frozen split's 0.1% point for a second cutoff:
`--target-fpr 0.001 --profile strict` on a second copy of the checkpoint.
Log every look at `test` in the [frozen-eval log](frozen-eval.md#log).

Diagnostics for every arm, reported beside arm A and never used to choose:
the NTIRE operation tables (section 1), the Frontier pairs
(`scripts/stress-photo.py` with no options on
`data/vstore/wasitai/paired-images.tsv`, then `tintype score` and
`scripts/report-stress.py --pair-by-name`; see
[EVIDENCE.md](../EVIDENCE.md#recompute-the-tables)), confirmation 7 and
Arena ([EVIDENCE.md](../EVIDENCE.md), [BENCHMARK.md](../BENCHMARK.md)).
`tintype evaluate confirm7` applies the default, strict, sensitive and fast
profiles, which a training checkpoint's sidecar does not have; for an arm,
score the batch's manifests with `tintype score --all-views` and compare
at empirical false-positive rates with `tintype report --operating-points`.
The confirmation 7 and Arena commands were not exercised for this runbook.

Report for each arm, losses as plainly as gains: recall at 1% and at 0.1%
on processed and on clean items, at frozen and at empirical cutoffs;
authentic items flagged per transform slice; the worst slice; the clean
AUROC; the "data wait" of the run; the commit, command and hardware.

## 5. The offline codec arm

Nothing here runs inside `tintype`. A learned codec or an enhancement tool
runs in its own environment and leaves decoded images; the store builder
keeps the Tintype side honest.

**Before any download, for each implementation: read its licence at the
source, and record the licence, where it is stated, the commit or release
and the date in [DATA.md](../DATA.md) and [NOTICE.md](../../NOTICE.md).
If the terms do not cover producing training images for a model with
Tintype's licence, drop it and write the replacement into the
pre-registration.** Nothing was downloaded for this runbook. What is
recorded so far is the survey in
[frozen-eval.md](frozen-eval.md#what-goes-in), which is not a licence
review:

| Implementation | Use | Licence as surveyed (re-read before download) | Still to check |
|---|---|---|---|
| CompressAI (Cheng2020 attention and anchor, hyperprior, mean-scale) | learned codec, several quality levels | BSD 3-Clause Clear per the survey | No licence was found for its pretrained weights; patent grant excluded; which weights and training data each model used |
| JPEG AI reference software | learned codec, the ISO/IEC 6048 bitstream | BSD per the survey | Exact licence text and patent terms; where the official model weights are and under what terms |
| A JPEG 2000 and a lossy WebP / AVIF encoder (OpenJPEG, libwebp, libaom or similar, through FFmpeg) | classical codecs the Rust runtime cannot encode | not surveyed | Licence of each library and of the FFmpeg build |
| A restoration or enhancement tool (denoiser, deblocker, super-resolution, face restoration) | benign-processing negatives | not surveyed | Licence of code and weights; whether its output counts as generated (below) |

**Policy to settle in the pre-registration first.** A processed copy keeps
its parent's label: an authentic photo through a codec or a denoiser is
authentic, a generated image through the same is generated. That is clear
for codecs and regression-style restoration. It is not clear for
generative enhancement (diffusion upscalers, generative fill): decide
whether such a copy of an authentic photo is "authentic", "generated" or
left out, write it down, and leave those tools out until then.

```bash
# 1. The training side of the path-hash split: copies of held-out photos must not be trained on.
tintype hash-split "${PHOTO_MANIFESTS[@]}" --out-dir runs/ws1-codec/sides
#    (pass the manifests exactly as `tintype train` gets them: the hash is over the path as spelled)

# 2. The same share of every generator and real source, in both classes.
python3 scripts/build-codec-store.py plan --manifest runs/ws1-codec/sides/train.tsv \
  --fraction 0.25 --seed 1 --out runs/ws1-codec/plan
#    -> worklist.tsv: id, absolute source path, label, generator, dataset, parent_id, ...

# 3. Offline, per codec and quality level: read worklist.tsv, write one decoded
#    image per id at the source's size to runs/ws1-codec/out/<name>/<id>.png.
#    Spread the ids over the codecs and levels (for example by id modulo), so
#    that each photo gets one processed copy, not one per codec.

# 4. Check and register each output folder as a store.
python3 scripts/build-codec-store.py collect --worklist runs/ws1-codec/plan/worklist.tsv \
  --outputs runs/ws1-codec/out/cheng2020-q3 --transform codec-cheng2020-q3 \
  --store runs/ws1-codec/stores/cheng2020-q3
#    -> manifest.tsv (the copies: parent's label and generator, parent_id, transform)
#       and parents.tsv (their originals with the same parent_id).
#    It refuses a store whose two classes came back at different rates.

CODEC_MANIFESTS=(); for m in runs/ws1-codec/stores/*/manifest.tsv; do CODEC_MANIFESTS+=(--manifest $m); done
```

Why this shape:

- **Label symmetry.** `tintype train` draws the classes equally and a
  generator, within its class, by `count^0.5`. A copy keeps its parent's
  generator column, so processing a fraction *f* of every key puts the
  processing in about *f / (1 + f)* of the draws of both classes (0.25
  gives 20%, the share the JPEG AI study retrained with). A codec seen
  mostly on one class would become a cue for that class, and a laundering
  route if that class is "real".
- **Parents.** `tintype split-groups` keeps items that share a
  `parent_id` together. The training manifests have no such column, so
  `collect` writes the originals again with it (`parents.tsv`). To place
  codec copies in an existing group split:
  `tintype split-groups --manifest <store>/parents.tsv --manifest <store>/manifest.tsv --existing <split dir> --out-dir <new dir>`
  sends every copy where its parent went, and `tintype verify-split`
  fails if one crosses. `parents.tsv` is for splitting and checking, not
  for `tintype train` (the training manifests already list those files).
- **The path-hash split.** `tintype train` holds out 3% of every manifest
  by path hash, copies included, so about 3% of the copies land in the
  run's own validation pool although their parents were trained on. That
  pool only feeds the log lines of the run. Do not pass the codec stores
  to `finalize --also-trained-on`; calibrate on the frozen split.
- **Unseen codecs.** Keep at least one learned codec out of the training
  stores and in the frozen split's processed copies, so that gate item 4
  is read separately for a codec the model trained on and one it did not.
- The evaluation side is the same script on the frozen split's own
  originals, placed with `--existing data/frozen-eval`.

## 6. Clean-to-processed consistency: a later ablation, not built

Not implemented. Do it only after the three arms above are compared.
It would be a new opt-in path: geometry split from degradation in the
loader so that a clean and a processed view of one item are spatially
aligned, a second forward pass, and a KL term with the clean view as a
stop-gradient teacher (about twice the training compute, none at
inference). Expect little. TeleAI's KL-only arm gained 0.15 points of
AUROC on mixed clean and distorted inputs; DCPT's degraded AUROC rose from
78.5% to 79.4% while its clean AUROC fell from 89.1% to 87.9%. If it is
run, the clean AUROC and clean recall at both operating points are
reported explicitly beside the processed ones, and gate item 3 applies
unchanged.

## Measured so far

All exploratory: archived scores, or CPU re-scoring of development sets
with the released weights. Cutoffs taken on a cohort and reported on it
describe that cohort. Hardware: Apple M6, 12 cores, 32 GB, macOS 27.2,
shared with other jobs while these ran.

### Archived baselines, reproduced

`python3 scripts/replay-research.py --hard-labels val_hard_labels.csv --public-labels test_labels.csv`
(commit `40816c8`; the label files are the pinned ones) printed, unchanged
from the base commit:

| Cohort | Reproduced |
|---|---|
| Confirmation 7, Tintype default | 10,887 of 12,305 generated (88.5%) at 107 authentic flagged (1% of 10,747); 11,038 (89.7%) and 130 authentic at the shipped cutoff |
| Frontier pairs, recall at 1% empirical FPR | 181, 139 and 73 of 261 (69.3%, 53.3%, 28.0%) for originals, same-size JPEG and JPEG capped at 960 px |
| NTIRE public test, distorted | AUROC 0.636; 14 of 650 (2.2%) at 1% FPR; 155 of 600 authentic flagged at the shipped cutoff (291 of 650 generated, 44.8%, from `diagnose-ntire.py --operations`) |
| NTIRE public test, chains with JPEG AI, Cheng2020 or WMForger | AUROC 0.549; 110 of 182 authentic flagged. Without them: 0.690; 45 of 418 |

### Frontier pairs, paired columns (archive)

`python3 scripts/report-stress.py --scores docs/results/frontier-jpeg-stress-scores.tsv.gz --pair-by-name`
(commit `40816c8`). 261 generated and 484 authentic pairs; the released
weights; cutoffs on the cohort itself.

| Transform | Authentic flagged at 0.5, original → processed | Recall at 0.5, original → processed | Recall at 1% FPR | Recall at 0.1% FPR | AUROC | Originals' 1% cutoff on processed: FPR, recall |
|---|---|---|---|---|---|---|
| original | 9 → 9 of 484 | 205 → 205 of 261 | 181 (69.3%) | 137 (52.5%) | 0.960 | 4 of 484, 181 of 261 |
| jpeg (`jpeg-qv4`) | 9 → 1 (none new, 8 cleared) | 205 → 116 | 139 (53.3%) | 99 (37.9%) | 0.922 | 0 of 484, 97 of 261 (37.2%) |
| jpeg-960 (`repost-960-qv12x1`) | 9 → 10 (9 new, 8 cleared) | 205 → 101 | 73 (28.0%) | 9 (3.4%) | 0.899 | 6 of 484, 79 of 261 (30.3%) |

With 484 authentic items the 0.1% point allows no false positive, so it is
the recall above the highest authentic score: a coarse figure.

### Every NTIRE operation (archive)

`python3 scripts/diagnose-ntire.py <dir with the two label files> --operations`
(commit `40816c8`) wrote [ntire-2026-operations.md](../results/ntire-2026-operations.md):
both splits, every operation, chain length and severity. A row covers the
distorted images whose chain contains the operation (chains hold one to
five), so rows overlap. From the public test, at the shipped cutoff; clean
authentic images are flagged at 42 of 600 (7.0%):

| Operation in the chain | Authentic flagged | Without it | Recall | AUROC |
|---|---|---|---|---|
| wmforger | 40 of 53 (75.5%) | 115 of 547 (21.0%) | 29 of 38 | 0.525 |
| jpeg_ai | 44 of 74 (59.5%) | 111 of 526 (21.1%) | 47 of 80 | 0.548 |
| cheng2020 | 33 of 65 (50.8%) | 122 of 535 (22.8%) | 55 of 87 | 0.590 |
| lensblur | 34 of 110 (30.9%) | 121 of 490 (24.7%) | 72 of 156 | 0.613 |
| downscale | 112 of 425 (26.4%) | 43 of 175 (24.6%) | 231 of 501 | 0.642 |
| jpeg2000 | 15 of 59 (25.4%) | 140 of 541 (25.9%) | 43 of 83 | 0.724 |
| watermark (six algorithms) | 49 of 209 (23.4%) | 106 of 391 (27.1%) | 96 of 239 | 0.614 |
| jpeg | 8 of 70 (11.4%) | 147 of 530 (27.7%) | 22 of 80 | 0.628 |
| shotnoise | 1 of 42 (2.4%) | 154 of 558 (27.6%) | 3 of 45 | 0.543 |

What it shows, for these development images: the learned codecs and the
watermark attack raise authentic flags by 28 to 55 points over the other
distorted images, and JPEG AI at the organizers' scale value 0 alone is flagged at
32 of 41 (public) and 40 of 55 (hard validation). Classical JPEG and shot
noise go the other way: few authentic flags and few detections (recall
27.5% and 6.7%), a ranking loss. Pixel chains can address the second kind;
only the offline codec arm covers the first. Unpacking the public test
(`--extract`), re-scoring it on the CPU with a copy of the released
weights and passing the scores back (`--scores`) reproduced the archived
public-test tables line for line.

### Starting checkpoint (GPU, zero training steps)

`tintype train --init <checkpoint> <the 25 phase-6 manifests> --out runs/ws1-smoke/init-<name> --steps 0 --batch 32 --lr 2e-5 --warmup 300`
(binary of this branch at `0e2e73c` plus the then-uncommitted `hash-split`
command; Apple GPU). 1,812 of the 2,000 validation images were
readable (two stores' files are missing, section 0).

| `--init` | fit AUC | native AUC | fused AUC |
|---|---|---|---|
| `runs/v4/step_007000.safetensors` (phase 3) | 0.9933 | 0.9832 | 0.9944 |
| `runs/v3/final_v3.safetensors` (phase 2) | 0.9881 | 0.9785 | 0.9901 |
| phase-6 log, initial evaluation on 2,000 images | 0.9922 | 0.9824 | 0.9932 |

### Frontier photo pairs, one operation at a time (CPU re-score)

The 745 Frontier pairs through twelve FFmpeg transforms, scored on the CPU
with a copy of the released weights; commands, commits and every score in
[frontier-operation-stress](../results/frontier-operation-stress/report.md).
261 generated, 484 authentic; cutoffs on the cohort itself; the shipped
cutoff is 0.5. `jpeg-qv4` and `repost-960-qv12x1` reproduce the archived
table exactly.

| Transform | Authentic flagged at 0.5 (was 9 of 484) | Recall at 0.5 (was 205 of 261) | Recall at 1% FPR | Recall at 0.1% FPR | AUROC |
|---|---|---|---|---|---|
| original | 9 | 205 (78.5%) | 181 (69.3%) | 137 (52.5%) | 0.960 |
| resize-0.5-bicubic | 20 (15 new, 4 cleared) | 198 | 113 (43.3%) | 27 (10.3%) | 0.933 |
| resize-0.25-bicubic | 16 (14 new, 7 cleared) | 189 | 143 (54.8%) | 34 (13.0%) | 0.944 |
| jpeg-qv4 | 1 (0 new, 8 cleared) | 116 | 139 (53.3%) | 99 (37.9%) | 0.922 |
| jpeg-qv12 | 3 (1 new, 7 cleared) | 102 | 137 (52.5%) | 71 (27.2%) | 0.932 |
| blur-s1.0 | 13 (7 new, 3 cleared) | 201 | 141 (54.0%) | 39 (14.9%) | 0.944 |
| blur-s3.0 | 30 (25 new, 4 cleared) | 213 | 125 (47.9%) | 13 (5.0%) | 0.935 |
| noise-a10 | 65 (57 new, 1 cleared) | 180 | 29 (11.1%) | 10 (3.8%) | 0.849 |
| noise-a40 | 27 (27 new, 9 cleared) | 8 | 1 (0.4%) | 0 | 0.575 |
| sharpen-a1.0 | 9 (4 new, 4 cleared) | 202 | 178 (68.2%) | 45 (17.2%) | 0.951 |
| repost-720-qv9x1 | 13 (13 new, 9 cleared) | 105 | 58 (22.2%) | 27 (10.3%) | 0.897 |
| repost-960-qv7x4 | 13 (12 new, 8 cleared) | 120 | 91 (34.9%) | 6 (2.3%) | 0.907 |
| repost-960-qv12x1 | 10 (9 new, 8 cleared) | 101 | 73 (28.0%) | 9 (3.4%) | 0.899 |

What it shows, on this development set: no operation leaves recall at 1%
FPR where it was, and each family fails differently. JPEG clears
authentic flags and loses detections, and a capped repost loses as many
detections while flagging slightly more authentic files than before (a
ranking loss on the generated side). Blur, resizing and above all additive
noise raise authentic flags:
noise at strength 10 takes them from 9 to 65 of 484 and recall at 1% FPR
to 11.1%. Mild sharpening barely moves the 1% point (68.2%) but costs most
of the 0.1% point (52.5% to 17.2%). These are single operations at a few
severities, rendered by one tool; `sharpen-a2.0`, `resize-2.0-bicubic`,
the other kernels and chains were not run.

### Loader cost of chains (CPU)

`tintype augment-bench <the 16 phase-6 photo manifests> --images 768 --rounds 4 --threads 4 <flags>`,
commit `40816c8`, five runs per setting, interleaved, on a machine other
jobs were loading (load average 15 to 40 on 12 cores), so the spread is
wide and the medians are pessimistic. 719 of the 768 drawn photos were
readable (2.04 megapixels on average); decoding took a median of about
9 ms per photo in every setting (8.9 to 9.1).

| Setting | Augmentation, ms per view: median (min–max) | Views/s per loader thread at the medians |
|---|---|---|
| Phase-6 recipe (no flags) | 6.7 (6.2–8.1) | 64 |
| Arm B, first stage: `--p-chain 0.5 --chain-ops 1,1 --chain-severity 0.25 --messenger-cap 720,2048 --messenger-cycles 1,4` | 11.9 (9.5–13.4) | 48 |
| Arm B, last stage: `--p-chain 0.5 --chain-ops 1,3 --chain-severity 0.75`, same messenger flags | 13.6 (9.9–16.3) | 44 |
| Everything on: `--p-chain 1 --chain-ops 1,5 --chain-severity 1`, all twelve `--chain-kinds`, `--messenger-cycles 1,6` | 22.0 (15.5–27.5) | 32 |

The trainer needs about 7.3 views/s per thread (80 images/s over 11
threads, from the phase-6 log), so arm B's loader has roughly six times
the headroom on this measurement.

Two 100-step GPU smoke runs of the section 3 commands (commit `40816c8`,
`--steps 100`, arm B with its schedule compressed to steps 0, 40 and 70,
the Frontier audit rendering on the same CPU at the time, two stores
missing): the phase-6 recipe logged 57.4 images/s with 4% data wait for
steps 1–50 and 65.2 images/s with 0% for steps 51–100; arm B logged
52.0 images/s with 11% and 66.1 images/s with 0%. Both runs wrote only
under `runs/ws1-smoke/`. One hundred steps say nothing about accuracy.

## Log

| Date | Commit | What ran | Result |
|---|---|---|---|
| 2026-11-14 | `40816c8` and earlier, as stated with each result | Audit tooling on archives and on the CPU; loader timing; a zero-step initial evaluation of two candidate starting checkpoints; two 100-step smoke runs | Above. No arm trained. |
