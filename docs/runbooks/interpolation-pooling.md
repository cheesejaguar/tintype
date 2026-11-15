# Runbook: weight interpolation and view pooling

Workstream 10 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
("Test weight interpolation and view pooling as bounded offline
experiments"). Two experiments that cost no training: move the released
weights part of the way back towards the checkpoint they were initialised
from, and combine the six view logits some other way than their mean.

**Status.** The tools exist and are tested: `tintype interpolate` and
`tintype replay-pool`, both pure Rust and platform independent. The
exploratory numbers below were measured on development data. The gate
was then run on the frozen independent split (2026-11-14, "Frozen split"
below): **pooling failed on `calibrate`, and no interpolant passed.**
**Nothing here ships.** A changed pool or changed
weights needs a new calibration, an early-exit re-check, the frozen split
of [frozen-eval.md](frozen-eval.md) and a new entry in
[PREREGISTRATION.md](../PREREGISTRATION.md) first. The released files in
`models/` are never written: `interpolate` refuses any output inside a
`models` directory, `replay-pool` only reads, and every command below
names its outputs under `runs/`.

## The gate

Written before the commands that test it. It is a *proposed* decision
criterion from the report, not a decision.

> **Interpolation recovers legacy-GAN recall, or pooling recovers clean
> recall, at a fixed false-positive rate; and neither weakens robust
> (distorted) performance, current generators or reposted images.**

Read at both operating points, **1% and 0.1% FPR**, on the `test` set of
the frozen split, at cutoffs frozen on its `calibrate` set:

| | Must improve | Must not get worse |
|---|---|---|
| Interpolation, each `alpha` | Recall on legacy-GAN families | Recall on current generators, on reposted images and on distorted images; false-positive rate on every authentic slice |
| Pooling, each pool and view order | Recall on clean images | Recall and false-positive rate on distorted and reposted images, and on current generators |

"Not worse" is judged with the parent-clustered intervals of
`report --operating-points --cluster-by parent`. A candidate that passes
is a candidate for a pre-registered confirmation, no more.

The baselines the report quotes
([EVIDENCE.md](../EVIDENCE.md#ntire-threshold-transfer-and-artifact-diagnostics)):
on NTIRE hard validation, looking at all six views instead of stopping
early raised clean recall at 1% FPR from 51.8% to 60.5%, against 5.4% to
7.5% on distorted files; at the shipped threshold the extra views lost
nine generated detections (272 to 263) and added three authentic flags
(103 to 106) on distorted input.

## Part A: interpolation towards the initialiser

### What the tool does

```bash
tintype interpolate --a models/tintype.safetensors \
  --b baselines/commfor-224.safetensors --b-prefix vit. \
  --alpha 0,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1 \
  --out-dir runs/ws10-interp/
```

writes, for each `alpha`, `(1 − alpha) · A + alpha · B` as
`runs/ws10-interp/alpha-0.10.safetensors`, with two files beside it:
`alpha-0.10.json` (the spec) and `alpha-0.10.interpolation.json` (both
sources with their SHA-256, prefixes and weights).

| Flag | Meaning |
|---|---|
| `--a`, `--a-prefix` | The fine-tuned checkpoint (`alpha` 0) and a prefix to strip from its tensor names. Only read. |
| `--b`, `--b-prefix` | The checkpoint to move towards (`alpha` 1). `vit.` for the Community Forensics release. Only read. |
| `--alpha` | Shares of `--b`, each between 0 and 1. |
| `--out-dir` | Where a sweep is written. |
| `--average A B … [--weights w1,w2,…] --out FILE` | Instead of a sweep: a weighted average of sibling fine-tunes (below). |
| `--spec` | The spec whose views the outputs get (default: the one beside `--a`). |
| `--overwrite` | Replace outputs that already exist. Never anything in a `models` directory. |

What it checks and guarantees:

- **Layout.** The two checkpoints must agree on every tensor's name
  (after the prefixes are stripped), shape and type, and every tensor must
  be F32. Otherwise it stops and lists each difference. The released
  weights and Community Forensics 224 agree on all **152 tensors**
  (21,666,049 values).
- **Arithmetic.** The sum is taken in double precision and rounded once
  to F32. A source with weight exactly 0 is left out and a single source
  with weight exactly 1 is copied, so the end points are bit for bit
  their sources (negative zeros included). What is written is read back
  and compared with what was computed.
- **No released file is touched.** An output in a directory called
  `models`, or anywhere below a `models` directory that holds
  `tintype.safetensors`, is refused, also through a symbolic link and
  also with `--overwrite`; so is an output that is one of the inputs.
- **The spec is honest.** It copies the *views* of the source spec, so
  every candidate of a sweep is preprocessed the same way, and nothing
  that was fitted: the calibration of the photo, fast and video profiles
  is reset to the identity (`logit_scale` 1, `logit_bias` 0), and the
  strict and sensitive profiles, the early exit and the second-opinion
  bands are removed. The spec carries `"uncalibrated": ["photo", "fast",
  "video"]`. While a profile is listed there, `detect`, `batch` and
  `score` print a warning on standard error and `serve` refuses to start;
  `calibrate --profile <p>` removes `<p>` from the list. A finished
  model's spec has no such field, and the released specs load and write
  back byte for byte as before.

### One preprocessing path

Community Forensics' own release is run differently from Tintype
(`ModelSpec::commfor(224)`: Pillow resize to 256, centre 224, one view;
Tintype: six views, its own resampling). A sweep must not change the
preprocessing along the way, or the weights and the pipeline are
confounded. So **every candidate, including `alpha` 1, runs through
Tintype's six views**. `alpha` 1 here is therefore *not* the Community
Forensics baseline of the benchmark tables: it is that network's weights
under Tintype's views.

### End-point parity

`alpha` 0 must be the released weights and `alpha` 1 the initialiser, both
as tensors and as scores. The tool prints the first; the second is checked
by running each source through the sweep's own spec:

```bash
mkdir -p runs/ws10-interp/parity
cp models/tintype.safetensors runs/ws10-interp/parity/tintype.safetensors
cp runs/ws10-interp/alpha-0.00.json runs/ws10-interp/parity/tintype.json
cp baselines/commfor-224.safetensors runs/ws10-interp/parity/commfor-224.safetensors
sed 's/"prefix": ""/"prefix": "vit."/' runs/ws10-interp/alpha-1.00.json > runs/ws10-interp/parity/commfor-224.json

for w in alpha-0.00 parity/tintype alpha-1.00 parity/commfor-224; do
  tintype detect --device cpu --json --weights runs/ws10-interp/$w.safetensors <images> \
    > runs/ws10-interp/parity/$(basename $w).jsonl
done
cmp runs/ws10-interp/parity/alpha-0.00.jsonl runs/ws10-interp/parity/tintype.jsonl
cmp runs/ws10-interp/parity/alpha-1.00.jsonl runs/ws10-interp/parity/commfor-224.jsonl
tintype verify-models
```

Measured (commit `0704bcc`, Apple M6, 32 GB, macOS 27.2, CPU engine;
`<images>` were the bundled `site/assets/demo.png` and 20 files of the
validation split, one of each class from each training store on disk):

- The sweep printed "agree on all 152 tensors (21666049 F32 values)",
  "alpha 0: every tensor is bit for bit that of
  models/tintype.safetensors" and "alpha 1: every tensor is bit for bit
  that of baselines/commfor-224.safetensors".
- `alpha-0.00.safetensors` is byte-identical to
  `models/tintype.safetensors` as a file (SHA-256 `9dcb21dc…e9aeba`).
  `alpha-1.00.safetensors` differs from the Community Forensics file only
  in its header, where the names lose the `vit.` prefix.
- Both `cmp` commands found the 21 JSON rows identical: each end point
  scores exactly as its source under the same spec. The two end points
  differ from each other, as they should (4 of the 21 verdicts, logits up
  to 10.5 apart).
- On the GPU, `alpha` 0 reproduced the view logits that the released
  weights' own calibration run stored for the same 4,819 files
  (`runs/scores-v7/calib_images.tsv`) to within 0.014 per view, the
  half-precision run-to-run difference.
- `tintype verify-models` reported the four released files unchanged.

The CLI test `interpolate_end_points_score_like_their_sources_and_nothing_is_written_to_models`
repeats the parity check on every platform with the two released
checkpoints (photo and video), which share one layout.

### Calibrate each candidate on separate data

An interpolant is a different network: the released threshold means
nothing for it. Each one is fitted on the `calibrate` set of the frozen
split, and never on the files it is then reported on.

```bash
for a in 0.00 0.10 0.20 0.30 0.50; do
  W=runs/ws10-interp/alpha-$a.safetensors
  tintype score --weights $W --manifest data/frozen-eval/calibrate.tsv \
    --out runs/ws10-interp/calibrate.alpha-$a.tsv --all-views
  tintype calibrate --weights $W --scores runs/ws10-interp/calibrate.alpha-$a.tsv --profile photo --target-fpr 0.015
  tintype calibrate --weights $W --scores runs/ws10-interp/calibrate.alpha-$a.tsv --profile strict --target-fpr 0.005
  tintype calibrate --weights $W --scores runs/ws10-interp/calibrate.alpha-$a.tsv --profile sensitive --target-fpr 0.05
done
tintype verify-models
```

`calibrate` rewrites `runs/ws10-interp/alpha-$a.json` only. The early
exit stays off (the spec has none) until its margins are refitted for
that network: `replay-pool --pool mean` on the candidate's own all-views
scores does that (Part B), and prints the `early_exit` block to put in
the spec. The fast profile needs its own two-view scores
(`score --fast --all-views`) and `calibrate --profile fast`.

### Evaluate

```bash
for a in 0.00 0.10 0.20 0.30 0.50; do
  W=runs/ws10-interp/alpha-$a.safetensors
  tintype score --weights $W --manifest data/frozen-eval/test.tsv \
    --out runs/ws10-interp/test.alpha-$a.tsv --all-views
done
tintype report --operating-points \
  --system "alpha-0.00=runs/ws10-interp/test.alpha-0.00.tsv@0.5" \
  --system "alpha-0.10=runs/ws10-interp/test.alpha-0.10.tsv@0.5" \
  --system "alpha-0.30=runs/ws10-interp/test.alpha-0.30.tsv@0.5" \
  --fpr 0.001,0.01 --prevalence 0.01 \
  --slice-by family,source,transform --cluster-by parent \
  --groups data/frozen-eval/test.tsv --min-slice 100
```

`alpha` 0.00 is the incumbent, refitted on the same calibration set, so
that the comparison is like for like. The gate reads the family slices
(legacy GANs against current generators) and the transform slices
(originals against reposted and distorted copies).

The archived suites are diagnostics beside that, never the basis of a
choice. Their image files are not kept on the development machine (only
the manifests and score files are): fetch and verify them first
(`tintype data confirm7`, `tintype verify-set docs/manifests/confirm7.frozen.tsv.gz`).

| Suite | What it says | Command |
|---|---|---|
| Legacy GANs | Confirmation 7's `classic-*` and AIGIBench GAN sets | `tintype evaluate confirm7 --weights runs/ws10-interp/alpha-$a.safetensors --tag alpha-$a --scores runs/ws10-interp/confirm7` |
| Current generators | Confirmation 7's AIGIBench DALL-E 3, FLUX.1-dev, Imagen 3, Midjourney, SD3, SDXL sets (the same run), and the frontier originals | `tintype score --weights runs/ws10-interp/alpha-$a.safetensors --manifest data/vstore/wasitai/images.tsv --out runs/ws10-interp/wasitai-images.alpha-$a.tsv --all-views` |
| Reposted | The same frontier images after WhatsApp, Instagram and re-sharing | `tintype score --weights runs/ws10-interp/alpha-$a.safetensors --manifest data/vstore/wasitai/images_social.tsv --out runs/ws10-interp/wasitai-social.alpha-$a.tsv --all-views` |
| Distorted | NTIRE hard validation and public test, and the Frontier JPEG stress pairs | Score a manifest of the unpacked files with `tintype score --weights runs/ws10-interp/alpha-$a.safetensors … --all-views`. `scripts/evaluate-ntire.py` pins the released weights' digest and cannot be pointed at a candidate; it needs a `--weights` that skips the pin, or a manifest built from the labels file. Not done here. |

`evaluate` writes `<set>.alpha-$a-views.tsv` (all six view logits) for
every photo set. With an uncalibrated or photo-only spec it skips the
strict and sensitive tables.

### Checkpoint averaging

```bash
tintype interpolate --average runs/a/last.safetensors runs/b/last.safetensors runs/c/last.safetensors \
  --weights 0.5,0.25,0.25 --out runs/ws10-avg/soup.safetensors
```

The weights must be non-negative and add up to 1 (default: equal). **Only
sibling fine-tunes** belong in an average: checkpoints trained from the
same initialiser, with the same layout. Two networks from different
initialisations do not lie in one basin and their average is noise; the
layout check cannot tell, so this is the operator's responsibility. The
only siblings that exist today are the photo and video checkpoints (both
continue from phase 3), which are fitted to different tasks; a real test
waits for the fine-tunes of workstreams 1 and 4.

Replay and averaging answer different questions and are **measured
independently**: a refresh trained *with replay* of older data is one
candidate, the *average* of a refresh trained without replay and its
predecessor is another, and each is compared with the predecessor on the
frozen split by the commands above. Averaging two checkpoints that both
used replay says nothing about either.

### Exploratory diagnostic (development data, uncalibrated)

A short GPU run to see the shape of the sweep, within the 2,000-item
limit for a shared machine. **This is not the gate.**

```bash
python3 -I scripts/alpha-sweep-diagnostic.py manifests \
  --val data/splits/val_images.tsv --data data --out-dir runs/ws10-interp/dev
for a in 0.00 0.10 0.30 0.50 1.00; do for s in legacy-gan current reposted; do
  tintype score --weights runs/ws10-interp/alpha-$a.safetensors \
    --manifest runs/ws10-interp/dev/$s.tsv --all-views --out runs/ws10-interp/scores/$s.alpha-$a.tsv
done; done
for s in legacy-gan current reposted; do
  python3 -I scripts/alpha-sweep-diagnostic.py report \
    $(for a in 0.00 0.10 0.30 0.50 1.00; do echo alpha-$a=runs/ws10-interp/scores/$s.alpha-$a.tsv; done)
done
```

The three manifests are cut from the validation split that `finalize`
wrote for the released weights (the 3% of each training store that
training never saw, by path hash; `$SCRATCH/data/splits/val_images.tsv`,
where `$SCRATCH` is the folder that holds the development data and runs),
keeping the files still on disk:

| Set | AI | Real |
|---|---|---|
| `legacy-gan` | 787 GAN images of the Community Forensics (Small) shards, this-person-does-not-exist and So-Fake-Set (GigaGAN 187, StyleGAN2 134, BigGAN 98, StyleSwin 89, StyleSAN-XL 84, StyleGAN-XL 52, ProGAN 51, StyleGAN3 47, Gansformer 31, others 14) | 1,213 from the same shards and CelebA-HQ |
| `current` | 1,279 from `frontier`, `frontier-heldout`, `nano-banana` and OpenFake | 717 from OpenFake and `openfake-test-reals` |
| `reposted` | 615 from `openfake-reddit` and So-Fake-Set | 208 from the same |

Measured (scoring at commit `317f398`, table at `0704bcc`; Apple M6,
32 GB, Apple GPU engine, half precision, all six views; mean view logit;
the cutoff of each row is found on that row's own real items; in brackets
the number of real items above it; archived in
[results/interpolation-pooling/alpha-sweep.md](../results/interpolation-pooling/alpha-sweep.md)):

| Set | `alpha` | AUROC | Recall at 1% FPR | Recall at 0.1% FPR |
|---|---|---|---|---|
| legacy-gan | 0 (released) | 0.9972 | 97.84% (12) | 96.32% (1) |
| | 0.1 | 0.9977 | 98.48% (12) | 96.44% (1) |
| | 0.3 | 0.9986 | 98.60% (12) | 97.08% (1) |
| | 0.5 | 0.9990 | 98.73% (12) | 96.32% (1) |
| | 1 (initialiser) | 0.9967 | 95.55% (12) | 85.01% (1) |
| current | 0 (released) | 0.9973 | 95.31% (7) | 86.79% (0) |
| | 0.1 | 0.9964 | 94.29% (7) | 82.17% (0) |
| | 0.3 | 0.9927 | 92.96% (7) | 73.65% (0) |
| | 0.5 | 0.9832 | 83.97% (7) | 60.59% (0) |
| | 1 (initialiser) | 0.8912 | 41.67% (7) | 29.55% (0) |
| reposted | 0 (released) | 0.9863 | 62.93% (2) | 37.56% (0) |
| | 0.1 | 0.9835 | 66.18% (2) | 33.50% (0) |
| | 0.3 | 0.9729 | 44.72% (2) | 26.83% (0) |
| | 0.5 | 0.9483 | 47.97% (2) | 24.39% (0) |
| | 1 (initialiser) | 0.7819 | 20.49% (2) | 11.54% (0) |

How far this can be read:

- On these files a small step towards the initialiser adds under one
  point of legacy-GAN recall at 1% FPR (97.84% to 98.48% at `alpha` 0.1,
  98.73% at 0.5), where the released weights are already near the
  ceiling, and takes recall on current generators down at every step
  (95.31% to 94.29% at 0.1, 92.96% at 0.3, 83.97% at 0.5; at 0.1% FPR
  86.79% to 82.17%, 73.65%, 60.59%). Reposted recall at 0.1% FPR falls at
  every step; at 1% FPR it moves both ways on 208 real items, where two
  real items set the cutoff.
- **The legacy-GAN set flatters both ends.** These GAN families are in
  the released weights' training stores (other files of the same shards).
  Community Forensics (Small) is published by the authors of the
  initialiser as a part of their own dataset; whether these files were in
  the initialiser's training split was not checked, and if they were,
  `alpha` 1 has seen them. The GAN weakness the README reports (84–89% on
  several GAN sets) is on other collections; it is not visible here, so
  this set cannot show a recovery. The Confirmation 7 GAN sets would,
  once their files are fetched again.
- The cutoffs are empirical on each row's own items; nothing is
  calibrated; the files are the released model's own calibration data;
  0.1% of 717 or 208 real items allows none above the cutoff, so those
  columns are recall with no real item flagged.
- As far as it goes, the diagnostic shows no sign of the gate's first
  half and clear signs against its second (current generators, reposts).
  It does not decide the gate: that takes the frozen split.

## Part B: pooling replay

### Which files can be replayed

A replay needs every view's logit of every image, in the spec's view
order (fit, native centre, then the four texture-ranked crops):

| Files | Hold | Replay |
|---|---|---|
| `tintype score --all-views` output; `evaluate`'s `<set>.<tag>-views.tsv`; `finalize`'s `calib_images.tsv` | `path label generator dataset prob` and six view logits | Yes |
| `score` output without `--all-views`, `<set>.<tag>.tsv` | The logits of the views early exit looked at, in looking order | No: refused ("has 2 view logits, not 6") |
| `docs/results/*-scores.tsv.gz`, including `ntire-2026-hard-validation/all-views-scores.tsv.gz` | Probabilities only | No. The NTIRE all-views archive holds two probabilities per image (early exit and all views of the *mean*): it supports the comparison already in EVIDENCE.md and nothing about any other pool, order or margin. Replaying NTIRE needs a local re-score with `--all-views`. |

On the development machine, the all-views files of the released weights
are `runs/scores-v7/calib_images.tsv` (its calibration data, 82,948
images), `runs/scores-c7/*.veracity-views.tsv` (Confirmation 7, 23,052),
`runs/scores-c6/*.veracity-views.tsv` (Confirmation 6, 16,324) and
`runs/scores-next/*.v7-views.tsv` (the development benchmarks, almost all
of them inside the calibration file), all under
`$SCRATCH/`. Every probability in them equals
the released calibration applied to the mean of the six logits.

### What the tool does

```bash
tintype replay-pool --fit <all-views files> --spec models/tintype.json \
  --test name=<all-views files> [--test other=…] \
  --pool mean,trimmed,median,top2,top3,max,fitted \
  --order shipped --order greedy --order 0,2,1,3,4,5
```

| Flag | Meaning |
|---|---|
| `--fit` | Files that thresholds, margins and weights are fitted on. A `*` in a file name matches. |
| `--test name=file[,file…]` | Files to report on, by name; repeatable. Items that are also in `--fit` (same path) are **left out of the fit**. |
| `--spec` | The spec the files were scored with. Its threshold and early exit are "the shipped pool", the first row of every table, against which verdict changes are counted. Only read. Without it the baseline is the mean with the standard schedule and a refitted threshold. |
| `--pool` | `mean`; `trimmed` (without the lowest and highest view); `median`; `topK` (mean of the K highest); `max`; `fitted` (per-view weights fitted on `--fit`); `weights:w0/w1/…` (fixed weights). |
| `--order` | Orders of looking at the views: `shipped`; `greedy` (chosen on `--fit`: at each step the view that makes the pool so far agree with all views on the most items); or indices such as `0,2,1,3,4,5`. |
| `--target-fpr`, `--rule`, `--min-source` | The calibration rule, as for `calibrate`: default 1.5%, balanced over real sources with at least 100 items. |
| `--tolerance`, `--margin-step`, `--margin-max`, `--asymmetric-margins` | The margin refit: default within 0.001, a grid from 0 to 12 in steps of 0.25, one margin per stage for both directions. |
| `--slice-by dataset\|generator` | Per-slice tables for every candidate. |
| `--write-scores DIR` | One ordinary score file per test set and candidate, for `report --operating-points` (intervals, clustering, prevalence). |
| `--in-sample` | Keep fit items that are also test items in the fit, and mark the output as in-sample: for reading a cutoff on slices of the very set it was fitted on. |

For every pool and order it **refits before it reports**
(`apply_profile` in `src/eval.rs`, which `evaluate` uses, does neither):

1. **Threshold and slope**, on the fit items, by the rule `calibrate`
   uses: the pooled logit at which the false-positive rate averaged over
   the real sources with at least 100 items equals the target, then the
   class-balanced logistic slope around it (`pool::fit_threshold`, which
   mirrors `eval::calibrate` step for step; a test holds the two
   together). On the released weights' calibration file the mean comes
   out at raw threshold 1.692 and slope 0.664, the released spec's own
   values.
2. **Early-exit margins**, on the fit items, by the criterion the shipped
   schedule was fitted with ([BENCHMARK.md](../BENCHMARK.md), "Five speed
   changes"): the tightest margins that keep pooled precision and recall
   within 0.001 of deciding from all views. `pool::refit_margins` makes
   that a procedure: start with no early exit; a step tightens one stage
   down the grid to the first margin that lets another item out early;
   among the steps that keep the criterion, take the one that saves the
   most views; stop when none keeps it. The search is greedy, so the
   result is tight locally, and it holds on the fit items only.
3. **Report** on the test items: recall at 1% and 0.1% empirical FPR
   (cutoffs on the test items themselves: a ranking measure), the actual
   false-positive count and recall at the refitted cutoff, average views
   per image, and the verdicts that differ from the shipped pool (`+a −b
   AI`: generated images gained and lost; `+c −d real`: real images newly
   flagged and cleared). Once for all six views and once with early exit.

**What early exit means for another pool.** The detector stops when "the
pool of the views looked at so far" is far enough from the threshold, and
the verdict comes from those views. That phrase is defined per pool, and
early exit is restricted to where it is defined:

| Pool | Of the views seen so far | Exits allowed |
|---|---|---|
| mean | Their mean | After every view |
| weighted mean | Their weighted mean, the weights renormalised over the views seen | Once a view with non-zero weight has been seen |
| median | Their median (mean of the two middle ones when even) | After every view |
| trimmed mean | Their mean without the lowest and the highest | From three views |
| top K, max | The mean of the K highest | From K views. From then on the value can only rise as views are added, so stopping *above* the threshold never disagrees with all views; stopping below is a bet. With `--asymmetric-margins` the margin above goes to 0. |

Replaying the mean with the spec's own schedule reproduces the detector:
the tool's shipped row on Confirmation 7 is 130 of 10,747 real images
flagged, recall 89.70%, 1.69 views per image, the published figures.

### Procedure for the gate

```bash
# For the incumbent or any candidate W under runs/:
tintype score --weights $W --manifest data/frozen-eval/fit.tsv       --out runs/ws10-pool/fit.tsv       --all-views
tintype score --weights $W --manifest data/frozen-eval/calibrate.tsv --out runs/ws10-pool/calibrate.tsv --all-views
tintype score --weights $W --manifest data/frozen-eval/test.tsv      --out runs/ws10-pool/test.tsv      --all-views

# Weights and margins on fit, reported on calibrate: choose at most one candidate here.
tintype replay-pool --fit runs/ws10-pool/fit.tsv --test calibrate=runs/ws10-pool/calibrate.tsv \
  --spec $(dirname $W)/$(basename $W .safetensors).json \
  --pool mean,trimmed,median,top2,top3,max,fitted --order shipped --order greedy --slice-by dataset

# Then once, for that candidate and the mean: threshold and margins on calibrate, numbers on test.
tintype replay-pool --fit runs/ws10-pool/calibrate.tsv --test test=runs/ws10-pool/test.tsv \
  --spec $(dirname $W)/$(basename $W .safetensors).json \
  --pool mean,<candidate> --order shipped --order <candidate order> \
  --write-scores runs/ws10-pool/test-scores
tintype report --operating-points \
  --system "mean=runs/ws10-pool/test-scores/test.mean.201345.tsv@0.5" \
  --system "candidate=runs/ws10-pool/test-scores/test.<pool>.<order>.tsv@0.5" \
  --fpr 0.001,0.01 --prevalence 0.01 --slice-by family,source,transform \
  --cluster-by parent --groups data/frozen-eval/test.tsv
```

The score files written by `--write-scores` are named
`<test>.<pool>.<order digits>.tsv`; their probability is the refitted
calibration and their logits are the views looked at. A pool that passes
still cannot ship from a replay: the detector computes the mean
(`Detector::probability`, `EarlyExit::decided`), so a new pool is a code
change behind a new optional spec field, re-scored for real, with its
`early_exit` block (printed by the tool) and calibration in a spec under
`runs/`.

### Exploratory replay (development data)

```bash
R=$SCRATCH/runs; C7=$R/scores-c7; v=veracity-views.tsv
tintype replay-pool --fit $R/scores-v7/calib_images.tsv --spec models/tintype.json \
  --test "confirm7=$C7/*.$v" --test "confirm6=$R/scores-c6/*.$v" \
  --test "c7-legacy-gan=$C7/fresh8-classic-biggan.$v,$C7/fresh8-classic-cyclegan.$v,$C7/fresh8-classic-gaugan.$v,$C7/fresh8-classic-stargan.$v,$C7/fresh8-classic-stylegan.$v,$C7/fresh8-classic-stylegan2.$v,$C7/fresh8-aigibench-ProGAN.$v,$C7/fresh8-aigibench-R3GAN.$v,$C7/fresh8-aigibench-StyleGAN-XL.$v,$C7/fresh8-aigibench-StyleGAN3.$v,$C7/fresh8-aigibench-StyleSwim.$v,$C7/fresh8-aigibench-WFIR.$v" \
  --test "c7-current=$C7/fresh8-aigibench-DALLE-3.$v,$C7/fresh8-aigibench-FLUX1-dev.$v,$C7/fresh8-aigibench-Imagen3.$v,$C7/fresh8-aigibench-Midjourney.$v,$C7/fresh8-aigibench-SD3.$v,$C7/fresh8-aigibench-SDXL.$v" \
  --test "wasitai-original=$R/scores-next/wasitai-images.v7-views.tsv" \
  --test "wasitai-reposted=$R/scores-next/wasitai-images_social.v7-views.tsv" \
  --pool mean,trimmed,median,top2,top3,max,fitted \
  --order shipped --order greedy --order 0,2,1,3,4,5 --order 2,1,3,4,5,0
```

Commit `0704bcc`, Apple M6, 32 GB, CPU only, 67 s. Full output:
[results/interpolation-pooling/pool-replay.md](../results/interpolation-pooling/pool-replay.md);
with `--asymmetric-margins` for mean, top 2, top 3 and max:
[pool-replay-asymmetric.md](../results/interpolation-pooling/pool-replay-asymmetric.md).

All of it is the released weights. The fit file is their calibration
data, less the 745 frontier originals that are a test set here (so the
refitted mean threshold is 1.631, not 1.692): 82,203 images, 16 real
sources. The test sets share no path with it. **They are development
data all the same**: Confirmation 6 and 7 have been read many times, the
reposted files are copies of frontier originals, and no parent or source
grouping exists for any of them. The cutoffs behind "recall at 1% / 0.1%
FPR" are found on each test set itself.

**All six views** (ranking of the pool itself):

| Test set | Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at refitted cutoff | Recall at refitted cutoff |
|---|---|---|---|---|---|
| Confirmation 7 (12,305 AI, 10,747 real) | mean | 89.00% | 63.58% | 128 (1.19%) | 90.33% |
| | trimmed mean | 88.16% | 65.66% | 132 (1.23%) | 89.21% |
| | median | 87.29% | 63.96% | 133 (1.24%) | 88.88% |
| | top 2 | 90.28% | 71.06% | 96 (0.89%) | 89.78% |
| | top 3 | 90.17% | 69.27% | 109 (1.01%) | 90.19% |
| | max | 89.28% | 64.89% | 79 (0.74%) | 87.22% |
| | fitted weights | 89.15% | 66.38% | 124 (1.15%) | 90.23% |
| Confirmation 6 (8,628 AI, 7,696 real) | mean | 87.67% | 61.37% | 115 (1.49%) | 90.81% |
| | top 2 | 90.58% | 62.63% | 75 (0.97%) | 90.48% |
| | top 3 | 90.53% | 62.00% | 81 (1.05%) | 90.98% |
| | max | 88.73% | 64.58% | 60 (0.78%) | 87.61% |
| | fitted weights | 88.38% | 63.62% | 109 (1.42%) | 90.96% |
| Confirmation 7, legacy GAN sets (4,000 AI, 3,858 real) | mean | 91.15% | 65.25% | 54 (1.40%) | 93.35% |
| | top 2 | 95.03% | 78.00% | 21 (0.54%) | 92.85% |
| | top 3 | 94.47% | 73.95% | 27 (0.70%) | 93.40% |
| | max | 94.75% | 78.12% | 18 (0.47%) | 89.65% |
| | fitted weights | 92.05% | 66.27% | 49 (1.27%) | 93.97% |
| Confirmation 7, current generators (2,394 AI, 1,801 real) | mean | 98.87% | 88.35% | 12 (0.67%) | 98.20% |
| | top 2 | 98.58% | 89.77% | 12 (0.67%) | 97.99% |
| | top 3 | 98.83% | 89.26% | 13 (0.72%) | 98.16% |
| | max | 98.04% | 85.59% | 10 (0.56%) | 97.24% |
| | fitted weights | 98.83% | 85.63% | 11 (0.61%) | 98.08% |
| Frontier originals (261 AI, 484 real) | mean | 72.41% | 45.21% | 12 (2.48%) | 78.93% |
| | top 2 | 60.15% | 39.08% | 13 (2.69%) | 77.39% |
| | top 3 | 60.54% | 43.68% | 19 (3.93%) | 77.01% |
| | max | 44.83% | 21.84% | 16 (3.31%) | 69.73% |
| | fitted weights | 73.95% | 43.68% | 9 (1.86%) | 77.78% |
| Frontier, reposted (783 AI, 1,446 real) | mean | 41.38% | 20.95% | 4 (0.28%) | 34.10% |
| | top 2 | 42.66% | 25.67% | 3 (0.21%) | 32.57% |
| | top 3 | 44.06% | 21.07% | 5 (0.35%) | 33.59% |
| | max | 31.55% | 11.62% | 10 (0.69%) | 30.01% |
| | fitted weights | 42.78% | 21.20% | 3 (0.21%) | 35.38% |

The fitted weights are 0.241 for the fit view and 0.133 to 0.164 for the
five native crops: close to the mean.

**With early exit, shipped view order** (what the detector would run; the
first row is the released spec untouched, the others are refitted):

| Test set | Pool | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged | Recall | Verdicts changed against shipped |
|---|---|---|---|---|---|---|---|
| Confirmation 7 | shipped | 1.69 | 88.48% | 58.30% | 130 (1.21%) | 89.70% | – |
| | mean, refitted | 1.66 | 88.41% | 57.35% | 138 (1.28%) | 89.94% | +74 −45 AI, +10 −2 real |
| | trimmed mean | 3.28 | 88.19% | 60.24% | 132 (1.23%) | 89.17% | +116 −182 AI, +10 −8 real |
| | median | 1.39 | 86.07% | 54.64% | 144 (1.34%) | 88.20% | +67 −252 AI, +25 −11 real |
| | top 2 | 4.22 | 90.36% | 71.04% | 93 (0.87%) | 89.70% | +262 −262 AI, +20 −57 real |
| | top 3 | 3.98 | 90.28% | 69.26% | 106 (0.99%) | 90.14% | +231 −177 AI, +16 −40 real |
| | max | 4.50 | 89.33% | 64.86% | 76 (0.71%) | 87.15% | +274 −588 AI, +31 −85 real |
| | fitted weights | 1.71 | 88.41% | 61.54% | 130 (1.21%) | 89.91% | +107 −82 AI, +11 −11 real |
| Confirmation 6 | shipped | 1.71 | 88.27% | 57.59% | 111 (1.44%) | 90.36% | – |
| | mean, refitted | 1.67 | 88.43% | 55.52% | 115 (1.49%) | 90.36% | +46 −46 AI, +5 −1 real |
| | top 2 | 4.20 | 90.51% | 62.63% | 75 (0.97%) | 90.41% | +207 −202 AI, +13 −49 real |
| | top 3 | 4.04 | 90.46% | 62.00% | 80 (1.04%) | 90.91% | +175 −127 AI, +5 −36 real |
| | fitted weights | 1.73 | 88.80% | 60.69% | 107 (1.39%) | 90.65% | +92 −67 AI, +4 −8 real |
| Frontier originals | shipped | 2.12 | 69.35% | 50.57% | 9 (1.86%) | 78.16% | – |
| | mean, refitted | 2.09 | 68.97% | 57.85% | 8 (1.65%) | 78.54% | +2 −1 AI, +0 −1 real |
| | top 2 | 4.38 | 68.20% | 39.08% | 12 (2.48%) | 77.39% | +8 −10 AI, +6 −3 real |
| | top 3 | 4.22 | 63.22% | 45.98% | 17 (3.51%) | 77.01% | +7 −10 AI, +10 −2 real |
| | fitted weights | 2.14 | 74.71% | 53.26% | 8 (1.65%) | 78.93% | +5 −3 AI, +1 −2 real |
| Frontier, reposted | shipped | 2.02 | 39.85% | 24.27% | 6 (0.41%) | 34.74% | – |
| | mean, refitted | 1.98 | 40.23% | 17.37% | 7 (0.48%) | 36.14% | +15 −4 AI, +1 −0 real |
| | top 2 | 3.77 | 43.17% | 25.67% | 3 (0.21%) | 32.57% | +35 −52 AI, +0 −3 real |
| | top 3 | 3.91 | 43.81% | 21.07% | 5 (0.35%) | 33.59% | +26 −35 AI, +1 −2 real |
| | fitted weights | 2.03 | 43.04% | 23.24% | 5 (0.35%) | 38.44% | +30 −1 AI, +0 −1 real |

Refitted margins for the mean in the shipped order on this fit file:
5.5 / 4 / 1.25 / 2 / 1.25 (shipped: 6 / 3 / 2 / 1.5 / 1), 1.75 views per
image there against 1.79.

How far this can be read:

- **All views against early exit, for the mean.** On Confirmation 7,
  looking at all six views gives 63.58% recall at 0.1% FPR against 58.30%
  with the shipped early exit (89.00% against 88.48% at 1%); on
  Confirmation 6, 61.37% against 57.59% at 0.1%, and 87.67% against
  88.27% at 1%, where early exit is ahead. The same direction as the
  NTIRE clean result at the strict point, smaller.
- **Top 2 and top 3** rank better than the mean on the Confirmation
  sets, most on the legacy-GAN sets (top 2: 95.03% against 91.15% at 1%
  FPR, 78.00% against 65.25% at 0.1%), and are level on Confirmation 7's
  current generators. They rank **worse on the frontier originals**
  (top 2: 60.15% against 72.41% at 1% FPR, 39.08% against 45.21% at
  0.1%, on 261 generated and 484 real images) and about level on their
  reposted copies. At the refitted cutoff they flag fewer real images
  and exchange hundreds of detections in both directions (+262 −262 on
  Confirmation 7). They need about four views per image where the mean
  needs 1.7; with margins fitted separately above and below, top 2 runs
  on 2.68 views on Confirmation 7, and then loses the gain at 0.1% FPR
  (58.14%), because an image that stops as soon as two views are over
  the threshold is no longer ranked by the rest.
- **Median, trimmed mean and maximum** are not better than the mean
  where it counts: the maximum loses 588 generated detections on
  Confirmation 7 at its refitted cutoff, the median 252.
- **Fitted weights** are nearly the mean and behave like it.
- **View order.** For the mean, on Confirmation 7: the shipped order
  refitted needs 1.66 views; the greedy order (4,1,0,3,2,5) 1.59 with
  63.63% recall at 0.1% FPR; fit view first (0,2,1,3,4,5) 1.61 with
  61.24%; fit view last (2,1,3,4,5,0) 2.22 with 61.63%. The orders
  differ by more in views than in recall, and the refit is greedy, so
  differences of this size between orders are within what another fit
  file could reverse.
- **Refitting on other data moves the result as much as the pool does.**
  The mean with margins refitted on this file changes 119 generated and
  12 real verdicts on Confirmation 7 against the shipped schedule, and
  drops recall at 0.1% FPR on the reposted frontier files from 24.27% to
  17.37%. The 0.001 criterion holds on the fit items only.
- **No distorted files.** No all-views score file of NTIRE, Arena or the
  JPEG stress pairs exists locally, so the half of the gate that protects
  robust performance is untested here. The reposted frontier copies are
  the only degraded slice.
- Against the gate: top 2 / top 3 show the "recovers clean recall" half
  on the Confirmation sets and fail the "current generators" half on the
  frontier originals, at more than twice the compute. Nothing here
  passes; nothing here could, on development data.

### A smooth or text-aware replacement view (design slot, not built)

The six views are the whole image, the centre crop and the four *most*
textured crops (`native_rich`). A replacement view at the same compute
budget would swap the fourth texture-ranked crop (rank 3) for a crop
chosen differently:

- **Smooth.** The *least* textured window on the same half-crop grid:
  `rich_windows` in `src/image_io.rs` already ranks the candidates by
  gradient energy, and the smooth view is the other end of that ranking
  (a new `View::NativeSmooth { crop, rank }` and `ViewSpec` variant,
  reachable only through `score --views fit,native,rich3,smooth1`; the
  default views unchanged).
- **Text-aware.** A window chosen by a text or stroke measure (dense
  short horizontal and vertical edges), for rendered text and overlays.
  It needs a measure that is cheap and deterministic; none exists in the
  source.

Neither is implemented: a new view cannot be replayed, it has to be
scored, and the files to score it on are the frozen split's. The
experiment, when it runs: score `fit`, `calibrate` and `test` with
`--views` for the baseline six and for the swapped six (`--views`
already turns early exit off and writes every logit), then run the
replay procedure above on each, comparing the swapped view set's mean
with the baseline's at the same number of views. The pool, the
threshold and the margins are refitted for the new view set by the same
tool.

## Frozen split

Run on 2026-11-14 on the frozen independent split of
[frozen-eval.md](frozen-eval.md), after the entry "View pooling and weight
interpolation on the frozen split" was written in
[PREREGISTRATION.md](../PREREGISTRATION.md) (commit `e128973`; the
calibrate-stage outcomes were added to it before each look at `test`).
Apple M6, 12 cores, 32 GB, macOS 27.2. Raw output:
[results/interpolation-pooling/frozen/](../results/interpolation-pooling/frozen/).

Read every figure with the split's caveats: every authentic original is a
camera JPEG of 12 megapixels or more and every generated original is a
PNG of 256 to 1,024 px, so the clean rows carry that difference and only
the processed copies re-encode both classes alike; FloreView is 36 scenes
on 46 phones and CSAFE is ten units of one phone model, so the intervals,
which resample parents, are narrower than the number of scenes and
devices warrants; the split has no real platform round trips and no
learned-codec copies; and `test` had been looked at before (the
incumbent, the processing-chain arms).

### Verdicts

| Part | Gate verdict | Why |
|---|---|---|
| B, view pooling | **Failed on `calibrate`.** No pool went to `test`. | None of the 24 candidates has clean recall above the refitted mean's at 1% FPR. |
| B, the refit itself | Not a gate; recorded. The refitted margins are **not better than the shipped ones** on `test`. | Same cutoffs: 0.40 points less clean recall at 1% for 0.26 fewer views; at 0.1%, 10 instead of 5 clean authentic photos flagged and 0.22 more views. |
| A, interpolation, Community Forensics anchor | **Failed on `calibrate`.** No candidate went to `test`. | All four steps recover GAN recall on the development sets and all four lose more than 0.5 points of degraded recall on `calibrate` (0.63 points at `alpha` 0.10, 8.5 at 0.50, at the 1% cutoff). |
| A, interpolation, phase-3 anchor | **Not passed.** One candidate (`alpha` 0.30) went to `test`; the protective half holds there, the improvement half is not shown. | It is not worse than the incumbent on any slice of `test`. Its GAN recall is higher only at its own cutoff, which flags more of the GAN sets' authentic images; at a matched rate it is lower (89.03% against 91.15% at 1%). |

### Part B: pooling

Inputs: the incumbent's all-views score files (`fit.all.tsv`,
`calibrate.all.tsv`, `test.all.tsv`, scored at `aa2f695` on the GPU
engine), copied under `runs/ws10-frozen/b/`. No network was run. CPU,
commit `e128973`.

```bash
B=runs/ws10-frozen/b; F=<frozen split>     # F holds fit.tsv, calibrate.tsv, test.tsv
# Clean rows (transform none) and degraded rows (every other transform), by the manifest's column 9.
awk -F'\t' 'NR==FNR{t[$1]=$9; next} {print > ((t[$1]=="-"||t[$1]=="") ? "calibrate.clean.tsv" : "calibrate.degraded.tsv")}' \
  $F/calibrate.tsv $B/calibrate.all.tsv       # run inside $B; the same for test, plus test.reposts.tsv for transforms starting repost-

# Stage 1: weights, orders, threshold (1%) and margins on fit; read on calibrate. Twice: one margin per stage, and --asymmetric-margins.
tintype replay-pool --fit $B/fit.all.tsv --spec models/tintype.json \
  --test calibrate-clean=$B/calibrate.clean.tsv --test calibrate-degraded=$B/calibrate.degraded.tsv \
  --test calibrate-all=$B/calibrate.all.tsv \
  --pool mean,trimmed,median,top2,top3,max,fitted --order shipped --order greedy --target-fpr 0.01

# Stage 2, once, for the mean alone (no pool was eligible): threshold and margins on calibrate, read on test.
# --spec is the incumbent's recalibrated spec (recal.json; for 0.1% its strict profile moved to the photo
# fields), so the first row replays the incumbent at the same cutoff with the shipped margins.
tintype replay-pool --fit $B/calibrate.all.tsv --spec $B/recal-1pct.json \
  --test test-clean=$B/test.clean.tsv --test test-degraded=$B/test.degraded.tsv \
  --test test-reposts=$B/test.reposts.tsv --test test-all=$B/test.all.tsv \
  --pool mean --order shipped --target-fpr 0.01 --slice-by generator --write-scores $B/test-scores-1pct
# ... and with recal-0.1pct.json, --target-fpr 0.001, --write-scores $B/test-scores-0.1pct
tintype report --operating-points \
  --system "shipped-margins=$B/test-scores-1pct/test-clean.shipped.201345.tsv@0.5" \
  --system "refitted-margins=$B/test-scores-1pct/test-clean.mean.201345.tsv@0.5" \
  --fpr 0.001,0.01 --prevalence 0.01 --slice-by family --cluster-by parent --groups $F/test.tsv --min-slice 100
```

**Stage 1, on `calibrate` (7,657 clean rows, 4,800 degraded rows).**
Everything fitted on `fit` (1,659 authentic, 2,100 generated; two real
sources). Recall at empirical cutoffs found on each slice's own authentic
rows, with early exit. The first row is the released spec untouched, for
orientation; it is not a candidate.

| Pool | Order | Margins | Clean, 1% | Clean, 0.1% | Degraded, 1% | Degraded, 0.1% | Views per image |
|---|---|---|---|---|---|---|---|
| shipped spec, as is | 2,0,1,3,4,5 | 6 / 3 / 2 / 1.5 / 1 | 95.07% | 92.31% | 75.71% | 44.67% | 1.66 |
| **mean, refitted (reference)** | 2,0,1,3,4,5 | one per stage | 95.02% | 72.48% | 72.25% | 41.00% | 1.54 |
| mean | greedy 4,0,3,1,5,2 | one per stage | 94.57% | 75.83% | 69.33% | 37.29% | 1.36 |
| trimmed mean | 2,0,1,3,4,5 | one per stage | 94.14% | 90.26% | 71.54% | 61.25% | 3.24 |
| trimmed mean | greedy 0,1,2,4,5,3 | one per stage | 93.95% | 90.36% | 72.54% | 63.71% | 3.26 |
| median | 2,0,1,3,4,5 | one per stage | 95.00% | 72.71% | 63.29% | 41.12% | 1.70 |
| median | greedy 0,4,2,1,5,3 | one per stage | 94.26% | 88.60% | 69.88% | 58.42% | 2.26 |
| top 2 | 2,0,1,3,4,5 | one per stage | 91.90% | 88.21% | 65.83% | 53.42% | 3.00 |
| top 2 | greedy 0,1,3,2,4,5 | one per stage | 91.55% | 86.55% | 68.08% | 52.88% | 3.44 |
| top 3 | 2,0,1,3,4,5 | one per stage | 92.62% | 89.29% | 67.33% | 53.21% | 3.43 |
| top 3 | greedy 0,1,3,5,2,4 | one per stage | 92.19% | 88.07% | 66.17% | 57.17% | 3.55 |
| max | 2,0,1,3,4,5 | one per stage | 89.43% | 84.60% | 61.46% | 50.08% | 4.08 |
| max | greedy 1,3,4,2,5,0 | one per stage | 89.17% | 83.81% | 60.96% | 45.38% | 3.85 |
| fitted weights | 2,0,1,3,4,5 | one per stage | 94.29% | 75.57% | 78.25% | 45.96% | 1.61 |
| fitted weights | greedy 0,3,2,4,5,1 | one per stage | 91.93% | 74.74% | 80.42% | 63.38% | 1.18 |
| trimmed mean | 2,0,1,3,4,5 | separate | 93.74% | 85.88% | 70.83% | 63.12% | 3.09 |
| trimmed mean | greedy 0,1,2,4,5,3 | separate | 93.40% | 86.05% | 71.50% | 63.21% | 3.07 |
| median | 2,0,1,3,4,5 | separate | 94.14% | 71.14% | 54.79% | 40.00% | 1.29 |
| median | greedy 0,4,2,1,5,3 | separate | 92.14% | 75.67% | 75.79% | 63.58% | 1.53 |
| top 2 | 2,0,1,3,4,5 | separate | 91.90% | 82.14% | 64.33% | 45.67% | 2.40 |
| top 2 | greedy 0,1,3,2,4,5 | separate | 91.55% | 81.33% | 63.50% | 43.12% | 2.66 |
| top 3 | 2,0,1,3,4,5 | separate | 92.62% | 87.52% | 64.92% | 53.08% | 3.19 |
| top 3 | greedy 0,1,3,5,2,4 | separate | 92.19% | 85.29% | 65.08% | 48.25% | 3.25 |
| max | 2,0,1,3,4,5 | separate | 89.43% | 77.05% | 59.00% | 42.50% | 2.43 |
| max | greedy 1,3,4,2,5,0 | separate | 89.17% | 75.14% | 55.79% | 35.08% | 2.25 |
| fitted weights | 2,0,1,3,4,5 | separate | 94.60% | 75.55% | 78.29% | 45.96% | 1.40 |
| fitted weights | greedy 0,3,2,4,5,1 | separate | 92.14% | 74.74% | 79.17% | 63.38% | 1.18 |

The weights fitted on `fit` are 0.438 for the whole-image view, 0 for the
centre crop and 0.108 to 0.173 for the four textured crops.

- **No candidate is eligible.** Condition (a) asks for clean recall above
  the reference's at both rates; at 1% the reference has 95.02% and the
  best candidate (median, shipped order) 95.00%. So no pool goes to
  `test`, and the pooling half of the gate has failed on `calibrate`.
- Top 2, top 3 and the maximum, which ranked better than the mean on the
  Confirmation sets in the exploratory replay, rank **worse** here at 1%
  on clean rows (91.90%, 92.62%, 89.43%) and on degraded rows.
- The fitted weights are the one candidate that helps the degraded rows
  (78.25% against 72.25% at 1%); they lose 0.73 points of clean recall at
  1%, which the rule does not allow.
- **The refit costs the strict point.** The reference's margins were
  fitted for the 1% threshold on `fit` (5.5 / 1.75 / 1.5 / 0.75 / 0.5).
  Read at a 0.1% empirical cutoff they give 72.48% clean recall where the
  shipped margins give 92.31%: an image that stops as soon as it is
  barely over a low threshold is not ranked above the authentic images
  that set a strict cutoff. Margins fitted for one operating point do not
  serve another; the trimmed mean and the top-k pools, which cannot exit
  before two or three views, keep 84% to 90% there for the same reason.

**Stage 2, one look at `test`, the mean alone.** Threshold and margins
refitted on all 12,457 rows of `calibrate`: for 1% the raw threshold is
0.985 and the margins 4.75 / 2 / 0.25 / 0.5 / 0; for 0.1%, 2.965 and
6.75 / 4.25 / 3.25 / 2.5 / 1.25. The thresholds are the ones
`tintype calibrate` gave the incumbent. The "shipped margins" rows replay
the incumbent's recalibrated specs and reproduce its tables exactly (51
of 7,326 and 95.71%; 5 of 7,326 and 92.72%; 120 and 18 of 9,726 over all
rows).

| Cutoff frozen on `calibrate` | Slice | Margins | Authentic flagged (upper bound) | Recall (95% interval over parents) | Views per image |
|---|---|---|---|---|---|
| 1% | clean | shipped | 51 of 7,326, 0.70% (0.88%) | 95.71% (95.23%–96.17%) | 1.44 |
| | | refitted | 52 of 7,326, 0.71% (0.89%) | 95.31% (94.83%–95.77%) | 1.24 |
| | | all six views | 62 of 7,326, 0.85% | 95.76% | 6 |
| | degraded | shipped | 69 of 2,400, 2.88% (3.50%) | 82.25% (79.60%–85.13%) | 1.96 |
| | | refitted | 69 of 2,400, 2.88% (3.50%) | 82.71% (79.97%–85.51%) | 1.53 |
| | | all six views | 76 of 2,400, 3.17% | 81.50% | 6 |
| | reposts | shipped | 14 of 600, 2.33% (3.62%) | 78.00% (72.94%–83.49%) | 2.28 |
| | | refitted | 18 of 600, 3.00% (4.42%) | 78.50% (73.40%–83.51%) | 1.72 |
| | all rows | shipped | 120 of 9,726, 1.23% | 92.34% | 1.57 |
| | | refitted | 121 of 9,726, 1.24% | 92.16% | 1.31 |
| 0.1% | clean | shipped | 5 of 7,326, 0.07% (0.14%) | 92.72% (92.15%–93.29%) | 1.41 |
| | | refitted | 10 of 7,326, 0.14% (0.23%) | 92.82% (92.25%–93.36%) | 1.59 |
| | | all six views | 11 of 7,326, 0.15% | 92.92% | 6 |
| | degraded | shipped | 13 of 2,400, 0.54% (0.86%) | 72.62% (69.19%–75.93%) | 1.86 |
| | | refitted | 19 of 2,400, 0.79% (1.16%) | 72.00% (68.61%–75.35%) | 2.19 |
| | | all six views | 20 of 2,400, 0.83% | 72.08% | 6 |
| | reposts | shipped | 2 of 600, 0.33% (1.05%) | 63.50% (57.22%–69.56%) | 2.10 |
| | | refitted | 2 of 600, 0.33% (1.05%) | 62.17% (56.08%–68.31%) | 2.52 |
| | all rows | shipped | 18 of 9,726, 0.19% | 87.70% | 1.52 |
| | | refitted | 29 of 9,726, 0.30% | 87.61% | 1.74 |

Recall at empirical cutoffs on `test` itself (a ranking measure, not a
transfer result), 1% / 0.1%: clean rows, all six views 95.89% / 92.44%,
shipped margins 96.39% / 90.36% (1% schedule) and 96.21% / 93.24% (0.1%
schedule), refitted margins 95.99% / 81.54% and 96.53% / 92.38%;
degraded rows, all six views 74.08% / 59.00%, shipped margins 76.12% /
45.50% and 78.67% / 63.62%, refitted margins 61.58% / 43.46% and 75.71% /
63.12%.

- **The refitted margins are not an improvement.** At the 1% cutoff they
  save 0.26 views per image over all rows (1.31 against 1.57) and lose
  0.40 points of clean recall (the refitted figure, 95.31%, is inside the
  shipped interval, 95.23%–96.17%); 88 generated and 73 authentic
  verdicts change in total. At the 0.1% cutoff they need *more* views
  (1.74 against 1.52), flag 10 clean authentic photos where the shipped
  margins flag 5 and 19 degraded ones against 13, for the same recall.
- **All six views are not better than early exit at a frozen cutoff
  here.** They flag more authentic photos at both cutoffs (62 against 51
  clean at 1%; 11 against 5 at 0.1%) for 0.05 to 0.2 points of clean
  recall, and lose recall on the degraded rows (81.50% against 82.25%;
  72.08% against 72.62%). At empirical cutoffs all six views rank the
  degraded rows better at 0.1% under the 1% schedule (59.00% against
  45.50%) and worse under the 0.1% schedule (59.00% against 63.62%): the
  NTIRE all-views gain does not carry over as a rule.
- The 0.001 criterion held where it was fitted and moved more than that
  on `test` at 1% (clean recall −0.0040). A margin schedule needs a check
  on data it was not fitted on, at every cutoff it will be used with.

### Part A: interpolation

Eight candidates under `runs/`, each `(1 − alpha) · released + alpha · B`,
for two anchors `B` and `alpha` 0.10, 0.20, 0.30, 0.50: Community
Forensics 224 (the phase-1 initialiser, `baselines/commfor-224.safetensors`,
prefix `vit.`) and the phase-3 weights (`runs/v4/step_007000.safetensors`,
SHA-256 `2d11bffd…`, the checkpoint phase 6 continued from; it agrees
with the released weights on all 152 tensors). The second anchor is not
in the grid written above; it was added in the pre-registration before
any score. Every candidate runs through Tintype's six views with the mean
of all views. `alpha` 0 is the incumbent's all-views scores.

```bash
tintype interpolate --a models/tintype.safetensors --b <phase-3 weights> \
  --alpha 0.1,0.2,0.3,0.5 --out-dir runs/ws10-interp-p3/        # the Community Forensics sweep is runs/ws10-interp/, above
A=runs/ws10-frozen/a; F=<frozen split>
# Each candidate: calibrate, and the twelve GAN sets of Confirmation 7 (development data), all six views. GPU.
tintype score --weights runs/ws10-interp-p3/alpha-0.30.safetensors --manifest $F/calibrate.tsv --all-views --out $A/p3-0.30.calibrate.tsv
tintype score --weights runs/ws10-interp-p3/alpha-0.30.safetensors \
  $(for s in classic-biggan classic-cyclegan classic-gaugan classic-stargan classic-stylegan classic-stylegan2 \
     aigibench-ProGAN aigibench-R3GAN aigibench-StyleGAN-XL aigibench-StyleGAN3 aigibench-StyleSwim aigibench-WFIR; do
     echo --manifest data/fresh8/$s/manifest.tsv; done) --all-views --out $A/p3-0.30.gan.tsv
# Cutoffs for 1% and 0.1% fitted on calibrate, read on its own clean and degraded rows (in-sample, marked) and on the GAN sets.
tintype replay-pool --fit $A/p3-0.30.calibrate.tsv --in-sample \
  --test calibrate-clean=$A/p3-0.30.calibrate.clean.tsv --test calibrate-degraded=$A/p3-0.30.calibrate.degraded.tsv \
  --test legacy-gan=$A/p3-0.30.gan.tsv --pool mean --order shipped --target-fpr 0.01      # and 0.001

# The chosen candidate only: its cutoffs into its own spec, then test, once.
tintype calibrate --weights runs/ws10-interp-p3/alpha-0.30.safetensors --scores $A/p3-0.30.calibrate.tsv --profile photo --target-fpr 0.01
tintype calibrate --weights runs/ws10-interp-p3/alpha-0.30.safetensors --scores $A/p3-0.30.calibrate.tsv --profile strict --target-fpr 0.001
tintype verify-models
tintype score --weights runs/ws10-interp-p3/alpha-0.30.safetensors --manifest $F/test.tsv --all-views --out $A/p3-0.30.test.tsv
tintype replay-pool --fit $A/p3-0.30.calibrate.tsv --test test-clean=… --test test-degraded=… --test test-reposts=… --test test-all=… \
  --pool mean --order shipped --target-fpr 0.01 --slice-by generator       # "all views" rows; and 0.001; the same for alpha 0
tintype report --operating-points --system "alpha-0=$A/alpha0.test.clean.tsv@0.3847578" \
  --system "phase3-0.30=$A/p3-0.30.test.clean.tsv@0.5" --slice-by family,source --cluster-by parent --groups $F/test.tsv
```

The cutoffs after `@` are the frozen logit cutoffs as probabilities of
each score file (`alpha` 0: 0.3847578 for 1% and 0.6995432 for 0.1% under
the shipped calibration; the candidate: 0.5 and 0.7286624 under its own).
`tintype calibrate` and the replay tool gave the same cutoffs for the
candidate (raw 1.064 and 2.815), and `verify-models` reported the four
released files unchanged after each step. Scoring: GPU engine, binary
of commit `217edfc`. Replay and tables: the code of `e128973` (the
commits after it up to the looks change documents only).

**Calibrate stage.** Each candidate at its own cutoffs fitted on
`calibrate`'s 5,857 authentic rows; recall on `calibrate`'s 4,200 clean
and 2,400 degraded generated rows (in-sample for the cutoff, the same for
every candidate) and on the GAN sets (4,000 generated, 3,858 authentic).
The last column is the GAN sets' recall at cutoffs found on their own
authentic images.

| Candidate | Cutoff | Raw threshold | Clean recall | Degraded recall | GAN recall | GAN authentic flagged | GAN recall at 1% / 0.1% empirical FPR | (a) GAN above `alpha` 0 | (b) within 0.5 points |
|---|---|---|---|---|---|---|---|---|---|
| `alpha` 0 (released) | 1% | 0.985 | 93.33% | 77.46% | 95.45% | 74 (1.92%) | 91.15% / 65.25% | – | – |
| | 0.1% | 2.965 | 89.79% | 68.04% | 87.38% | 25 (0.65%) | | – | – |
| phase 3, 0.10 | 1% | 0.993 | 93.36% | 77.75% | 95.55% | 79 (2.05%) | 90.75% / 63.68% | yes | yes |
| | 0.1% | 2.849 | 90.26% | 69.08% | 87.95% | 30 (0.78%) | | yes | yes |
| phase 3, 0.20 | 1% | 1.021 | 93.57% | 77.92% | 95.60% | 80 (2.07%) | 89.90% / 62.32% | yes | yes |
| | 0.1% | 2.849 | 90.38% | 69.42% | 87.88% | 30 (0.78%) | | yes | yes |
| **phase 3, 0.30** | 1% | 1.064 | 93.52% | 77.83% | 95.67% | 81 (2.10%) | 89.03% / 60.62% | yes | yes |
| | 0.1% | 2.815 | 90.55% | 70.08% | 88.00% | 33 (0.86%) | | yes | yes |
| phase 3, 0.50 | 1% | 1.246 | 93.38% | 77.83% | 95.35% | 89 (2.31%) | 87.48% / 57.35% | no | yes |
| | 0.1% | 2.996 | 90.33% | 69.50% | 86.72% | 35 (0.91%) | | no | yes |
| Community Forensics, 0.10 | 1% | 0.989 | 93.52% | 76.83% | 95.95% | 59 (1.53%) | 93.40% / 66.77% | yes | no |
| | 0.1% | 2.980 | 89.71% | 66.92% | 88.50% | 21 (0.54%) | | yes | no |
| Community Forensics, 0.20 | 1% | 0.998 | 93.67% | 75.58% | 96.35% | 49 (1.27%) | 95.03% / 74.20% | yes | no |
| | 0.1% | 2.961 | 89.60% | 65.21% | 89.20% | 17 (0.44%) | | yes | no |
| Community Forensics, 0.30 | 1% | 1.124 | 93.40% | 73.83% | 96.65% | 41 (1.06%) | 95.95% / 80.80% | yes | no |
| | 0.1% | 2.909 | 89.48% | 63.62% | 90.05% | 14 (0.36%) | | yes | no |
| Community Forensics, 0.50 | 1% | 1.503 | 92.33% | 68.92% | 96.15% | 28 (0.73%) | 97.30% / 87.22% | yes | no |
| | 0.1% | 2.984 | 88.14% | 59.67% | 90.38% | 8 (0.21%) | | yes | no |

- **Towards Community Forensics, legacy-GAN recall does come back**, at a
  matched rate on the GAN sets' own authentic images: 91.15% to 93.40%,
  95.03%, 95.95% and 97.30% at 1% for `alpha` 0.1 to 0.5, and 65.25% to
  66.77%, 74.20%, 80.80% and 87.22% at 0.1%, with fewer of those authentic
  images flagged at the frozen cutoffs (74 down to 28). **It is paid for
  on degraded files**: recall on `calibrate`'s degraded rows falls from
  77.46% to 76.83%, 75.58%, 73.83% and 68.92% at the 1% cutoff, and from
  68.04% to 59.67% at the 0.1% cutoff. Every one of the four is outside
  the half-point allowance, the smallest step by 0.63 points. Clean
  recall on `calibrate` is flat up to 0.3 and falls at 0.5.
- **Towards phase 3, nothing much moves.** Clean and degraded recall on
  `calibrate` are level or slightly up. GAN recall at the candidate's own
  cutoff is a fraction of a point higher (95.45% to 95.67% at `alpha`
  0.30) only because that cutoff lets more of the GAN sets' authentic
  images through (74 to 81); at a matched rate on the GAN sets it is
  lower at every step (91.15% to 89.03% at 1%, 65.25% to 60.62% at 0.1%).
  Phase 3 is weaker on GANs than the released weights, which phase 6
  trained on GAN data.
- **Chosen by the rule: phase 3 at `alpha` 0.30**, with the weakness of
  condition (a) written into the pre-registration before the look at
  `test`.

**One look at `test`: phase 3 at `alpha` 0.30 against `alpha` 0**, all
six views, each at its own cutoffs frozen on `calibrate`.

| Cutoff | Slice | | Authentic flagged (upper bound) | Recall (95% interval over parents) |
|---|---|---|---|---|
| 1% | clean | `alpha` 0 | 62 of 7,326, 0.85% (1.04%) | 95.76% (95.29%–96.22%) |
| | | phase 3, 0.30 | 52 of 7,326, 0.71% (0.89%) | 95.75% (95.29%–96.21%) |
| | degraded | `alpha` 0 | 76 of 2,400, 3.17% (3.82%) | 81.50% (78.80%–84.31%) |
| | | phase 3, 0.30 | 73 of 2,400, 3.04% (3.68%) | 82.25% (79.65%–84.91%) |
| | reposts | `alpha` 0 | 11 of 600, 1.83% (3.02%) | 77.50% (72.22%–82.87%) |
| | | phase 3, 0.30 | 11 of 600, 1.83% (3.02%) | 79.33% (74.52%–84.43%) |
| | current generators (flux, sd3), clean | `alpha` 0 | – | 99.50% (99.08%–99.83%) |
| | | phase 3, 0.30 | – | 99.75% (99.42%–100.00%) |
| | current generators, degraded | `alpha` 0 | – | 90.15% (87.37%–92.17%) |
| | | phase 3, 0.30 | – | 90.66% (88.38%–92.17%) |
| | held-out families (infinity, rar), clean | `alpha` 0 | – | 99.73% (99.53%–99.90%) |
| | | phase 3, 0.30 | – | 99.63% (99.40%–99.83%) |
| | held-out families, degraded | `alpha` 0 | – | 85.06% (81.61%–88.52%) |
| | | phase 3, 0.30 | – | 85.67% (82.42%–89.02%) |
| 0.1% | clean | `alpha` 0 | 11 of 7,326, 0.15% (0.25%) | 92.92% (92.34%–93.48%) |
| | | phase 3, 0.30 | 10 of 7,326, 0.14% (0.23%) | 93.46% (92.92%–93.98%) |
| | degraded | `alpha` 0 | 20 of 2,400, 0.83% (1.21%) | 72.08% (68.81%–75.35%) |
| | | phase 3, 0.30 | 20 of 2,400, 0.83% (1.21%) | 74.58% (71.24%–77.61%) |
| | reposts | `alpha` 0 | 2 of 600, 0.33% (1.05%) | 62.00% (55.88%–67.95%) |
| | | phase 3, 0.30 | 3 of 600, 0.50% (1.29%) | 66.83% (60.64%–72.64%) |
| | current generators, clean | `alpha` 0 | – | 97.42% (96.50%–98.25%) |
| | | phase 3, 0.30 | – | 98.25% (97.50%–99.00%) |
| | current generators, degraded | `alpha` 0 | – | 78.54% (71.21%–84.85%) |
| | | phase 3, 0.30 | – | 82.83% (76.26%–87.88%) |
| | held-out families, clean | `alpha` 0 | – | 98.97% (98.60%–99.30%) |
| | | phase 3, 0.30 | – | 98.93% (98.57%–99.27%) |
| | held-out families, degraded | `alpha` 0 | – | 77.74% (73.48%–82.11%) |
| | | phase 3, 0.30 | – | 78.56% (74.39%–82.62%) |

Over all of `test` the candidate flags 125 of 9,726 authentic files at
the 1% cutoff and 30 at the 0.1% cutoff (`alpha` 0: 138 and 31), with
recall 92.38% and 88.74% (92.20% and 87.71%). At empirical cutoffs on
`test` itself, recall at 1% / 0.1%: clean 96.08% / 93.00% (`alpha` 0:
95.89% / 92.44%), degraded 76.54% / 61.04% (74.08% / 59.00%), reposts
75.00% / 60.83% (74.83% / 59.83%).

- **The protective half of the gate holds on `test`.** On every slice the
  candidate's recall is at or above the lower end of `alpha` 0's
  interval, and it flags no more authentic files than `alpha` 0's upper
  bound. Its recall is level with or slightly above `alpha` 0's
  everywhere except clean rows at the 1% cutoff (−0.01 points) and the
  held-out families on clean rows (−0.10 and −0.04 points), and it flags
  one more reposted authentic file at the 0.1% cutoff (3 against 2);
  every difference is inside the intervals.
- **The improvement half is not shown.** By the rule as it was written,
  GAN recall at the candidate's own cutoffs is above `alpha` 0's (95.67%
  against 95.45%; 88.00% against 87.38%), and the rule is met. But the
  gate asks for legacy-GAN recall at a fixed false-positive rate, and at
  a matched rate on the GAN sets the candidate is lower (89.03% against
  91.15% at 1%; 60.62% against 65.25% at 0.1%): the difference at its own
  cutoffs is seven more of the GAN sets' authentic images flagged, not
  more generated images found. **The gate is not passed**: no interpolant
  recovers legacy-GAN recall without giving up recall on degraded files.
  The two anchors split the gate between them: Community Forensics
  recovers GANs and loses degraded recall; phase 3 keeps degraded recall
  and does not recover GANs.
- GAN recall rests on development data throughout (Confirmation 7), as
  the pre-registration says: the frozen split holds no GAN family.
- Not measured: early exit for any interpolant (none is fitted), the fast
  profile, latency, and the frontier and reposted development sets.

### Log of looks at the frozen split (workstream 10)

| Date | Commit | What was read on which set |
|---|---|---|
| 2026-11-14 | `e128973` | Part B stage 1: 24 pools and orders plus the mean, fitted on `fit` (incumbent's all-views scores), read on `calibrate`. No look at `test` |
| 2026-11-14 | `c75d92e` | Part B stage 2, the one look at `test`: the mean with threshold and margins refitted on `calibrate` for 1% and 0.1%, beside the incumbent's recalibrated specs replayed (shipped margins) and all six views. Incumbent's `test.all.tsv`; no scoring |
| 2026-11-14 | `217edfc` (scoring), `c75d92e` (tables) | Part A calibrate stage: eight interpolants scored on `calibrate` and on Confirmation 7's twelve GAN sets, all six views, GPU engine. No look at `test` |
| 2026-11-14 | `7b95c50` | Part A, the one look at `test`: phase 3 at `alpha` 0.30, scored on `test` with all six views, against `alpha` 0 from the incumbent's `test.all.tsv` |

## Ready to run

Run on the frozen split on 2026-11-14 (above): the pooling procedure, and
the interpolation grid for both anchors with one candidate read on
`test`. What is left was not run; each needs data or machine time that
was not available.

| What | Needs | Commands |
|---|---|---|
| Latency and throughput of any candidate | An unshared machine (the GPU was shared during the frozen-split run) | `tintype latency --weights runs/… --manifest <test manifest> --n 500 --device gpu`, and `--device cpu` |
| Early exit for an interpolant | All-views scores of the candidate on `calibrate` (they exist for the eight under `runs/ws10-frozen/a/`), then a real run to re-check | `tintype replay-pool --fit <candidate calibrate scores> --pool mean --order shipped`, put the printed `early_exit` block in the candidate's spec under `runs/`, then `tintype score` without `--all-views` |
| Frontier originals and reposted copies for each `alpha` (development data) | The files are on disk again (`data/vstore/wasitai`); about 3,000 images per candidate | The suite table under "Evaluate" |
| Distorted diagnostics for the sweep and for pooling | NTIRE archives unpacked into a manifest; a re-score with `--all-views` | `tintype score --weights runs/… --manifest <ntire manifest> --all-views --out runs/…`, then `replay-pool --test ntire-clean=… --test ntire-distorted=…` |
| Learned-codec and real platform round trips | Not in the frozen split (no licence stated for the codec weights; the one open repost collection was dropped) | – |
| Averaging of sibling fine-tunes | The fine-tunes of workstreams 1 and 4 | "Checkpoint averaging", then the sweep's calibrate and evaluate commands on the average |
| Replacement view | A new view kind | The design slot above |

## Before anything changes

A changed pool, view order, margin schedule or set of weights is a new
model. It needs, in this order: a pre-registration entry naming the one
candidate and the gate; calibration on the frozen `calibrate` set; an
early-exit refit and re-check by a real run (`score` without
`--all-views`, compared with the replay); the fixed-budget tables on the
frozen `test` set at 1% and 0.1% FPR with parent-clustered intervals;
the regression suites as diagnostics; and `tintype verify-models`
showing the released files untouched throughout.
