# Runbook: a small temporal head on the frame baseline

Research workstream 7 of
[the detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
("Add a small temporal head to the frame baseline"). Tintype scores a clip
as the mean of its frame logits: 16 frames spread over the first 30
seconds, six views each, nothing between frames
(`src/video.rs::aggregate`). This runbook is the experiment that asks
whether a shallow head over adjacent frames adds anything to that, and it
is built so that "no" is as easy to report as "yes".

**Status.** The tooling exists and is tested: patch tokens from both
engines, a feature dump with actual timestamps, six head kinds with a CPU
trainer, and a comparison harness with the controls below. **No head has
been fitted on data that could support a conclusion, and none of the data
the gate needs exists yet.** The only numbers in this file are engine
parity measurements and the timings and file sizes of a pipeline check on
development clips ([Measured here](#measured-here)). Nothing here selects a
model, a head, a cutoff or a sampling scheme to ship; that needs the frozen
independent split ([frozen-eval.md](frozen-eval.md)), re-calibration, an
early-exit re-check and a new entry in
[PREREGISTRATION.md](../PREREGISTRATION.md).

## The gate, before any command

Proposed decision criteria, written before any head is fitted.

1. **Whether to run this at all** is decided by the paired video audit
   ([video-audit.md](video-audit.md), branch `research/ws5-video-audit`),
   not here. Its decision rule sorts the frame baseline's errors on
   controlled cohorts into coverage, source or processing shift, and "no
   frame signal". This experiment is next only when "no frame signal" is the
   largest bin on at least two controlled cohorts and the evidence for
   source shift is absent. If source shift or coverage leads, do that
   experiment first: a temporal head fitted on clips whose classes differ
   in source or cadence learns the source or the cadence.
2. **The fast whole-clip path stays the default** until a temporal head
   gives **better recall at a low false-positive rate than the frame mean at
   equal decode and inference cost, on both**
   - **fresh matched clips**: a controlled cohort of the video audit
     (held-out authentic collections, classes matched on cadence,
     resolution, duration, bitrate and motion, one common encode), and
   - **a second, reposted corpus**: clips that have been through real
     platform round trips or the audit's `fixed-gop+repost-*` variants.

   "Better" means, on each of the two, at cutoffs frozen on calibration
   clips: more AI clips caught at the 1% cutoff **and** at the 0.1% cutoff,
   with the actual false-positive rate on the test clips no higher than the
   frame mean's, and a clustered 95% interval for the recall difference
   (clusters: scene or parent) that excludes zero at the 1% cutoff. An
   operating point the authentic clips cannot resolve (fewer than 100
   authentic clips for 1%, fewer than 1,000 for 0.1%; 2,995 with none
   flagged to bound a rate below 0.1%) is reported as not resolved, not as
   a pass.
3. **The gain has to be temporal.** All of these, or the head is a better
   pooled classifier or a shortcut and the conclusion is "no temporal
   gain":
   - the temporal head beats the best **order-invariant** head fitted on
     the same features at the same cost;
   - with **frames shuffled** the temporal head loses the gain (back to the
     order-invariant level or below), and its **order-blind twin** (the same
     head fitted on shuffled frames) does not reach it;
   - on **blurred authentic clips** (synthetic temporal blur, and authentic
     low-shutter footage where a collection has it) its false-positive rate
     at the frozen cutoffs is no higher than the frame mean's;
   - the gain is **not confined to one slice** of cuts, repeated frames,
     motion, frame rate or sample spacing, and a head that sees only those
     covariates (`--kind covariates`) does not separate the classes
     (AUROC interval including 0.5 on the test clips). If the covariates
     alone separate the classes, fix the cohort before reading any head.
4. **More frame views is the cheaper alternative.** If spending the same
   cost on more frames with fewer views matches the temporal head, prefer
   it: it needs no new parameters and no new inference path.

A negative result is a result. If the gate fails, record the tables below
as they are, keep the frame mean, and say which criterion failed.

## Why the evidence is mixed

The report's sources disagree, which is why this is a comparison and not a
port. RPG found several standalone temporal methods weak against frame
baselines; V-PVP reports gains from local patch motion, with
video-pretrained features; a motion-shortcut audit shows temporal
detectors keying on cadence and motion statistics of their training sets.
Tintype's backbone is a ViT-S/16 trained on single frames (one random frame
per training draw, `src/video.rs::random_frame`), so its patch tokens were
never asked to be stable or comparable across frames. Whether their
frame-to-frame differences carry a generator signal is an open question
that only this comparison answers for this backbone.

## Data this needs, none of which exists yet

| Need | What it is | State |
|---|---|---|
| Fresh matched clips | A controlled cohort from the video audit, with `parent_id`, `scene_id` and `source_id` in its manifests | **Does not exist.** The audit's collections have not been chosen or downloaded; licences are unchecked ([video-audit.md](video-audit.md), steps 1 and 2) |
| A second, reposted corpus | The same or other clips after platform round trips, or the audit's `fixed-gop+repost-*` variants, each derivative carrying its parent's `parent_id` | **Does not exist** |
| Fit, calibrate and test parts | `tintype split-groups` over those manifests, by parent, scene and source | Tooling exists (step 0); the split does not |
| Authentic blur control | Authentic low-shutter or motion-blurred footage from a held-out collection | **Does not exist.** The synthetic control (`--temporal-blur`) needs no new data |

The clips available locally are development data: the training and
calibration stores, confirmation 4, and the 100 DaVODi clips (20 scenes, one
authentic source). A head fitted or judged on them supports no conclusion;
they are used below only to show that the pipeline runs.

**Sizes.** The 1% point needs at least 100 authentic test clips to allow
one false alarm and the 0.1% point at least 1,000, per part and per corpus,
counted in independent scenes or parents; calibration needs the same. The
fit part needs both classes and enough scenes that a head of a few
thousand to a few tens of thousands of parameters is not memorising them:
plan for thousands of clips, and say how many there were.

## What was built

All of it is offline tooling. `detect`, `batch`, `serve`, `score` and
`score-video` are unchanged, and no model file or spec has a new field.

| Piece | Where | What it does |
|---|---|---|
| Patch tokens, CPU engine | `CpuVit::forward_tokens` (`src/cpu_vit.rs`) | Tokens after any blocks (through the final layer norm), next to logits that are bit-identical to `forward` |
| Patch tokens, GPU engine | `build_vit_tokens` (`src/mps_vit.rs`) | The same from an MPSGraph that evaluates every block in full; the scoring graph (`build_vit_cls`) is untouched |
| One interface over both | `tintype::embed::Embedder`, `pool_grid` (`src/embed.rs`) | Logits and tokens for any number of views; average pooling of the 14×14 token map to a grid |
| Timed sampling | `yuv::uniform_timed`, `video::sample_frames_timed` | The frames `score-video` samples, each with its actual presentation time and its position in the stream; adjacent-frame pairs; synthetic temporal blur; a trace of every frame read |
| `tintype dump-video-features` | `src/temporal.rs` | Per clip and sampled frame: timestamp, stream position, logits of every view, pooled whole-frame-view tokens, and what the frames in between did |
| `tintype fit-temporal-head` | `src/temporal_head.rs` | Fits one head on a dump, on the CPU |
| `tintype eval-temporal-head` | `src/temporal_head.rs` | Frame mean against heads and outside score files, at cutoffs from a calibration dump, with the controls |

**Why a Rust trainer and not a script.** The heads are small (a linear
layer, or one hidden layer over 384-wide tokens), so hand-written gradients
and Adam on the CPU are enough, and the trainer is checked against finite
differences in the unit tests. In return nothing outside the repository is
needed to fit or run a head, and a head that ever ships is run by the code
that fitted it, with no second implementation to keep in step.

**Spatial alignment.** Tokens come from the profile's whole-frame view
(`fit`: shorter side to 256, centre 224) and from nothing else. That view
has one geometry for every frame of a clip, so cell *g* of one frame and
cell *g* of the next cover the same part of the picture. The four
texture-ranked native crops are re-chosen on every frame and do not line
up; they are scored (their logits are in the dump) but never give tokens.
A native-resolution variant would have to fix one window per clip first;
that is not built.

**Timestamps.** Every sample carries the presentation time the decoder
reported for the frame actually picked, and its index among the frames
read, not the nominal `k · span / 16`. Two samples with the same index are
the same decoded frame (a clip shorter than its sample count); a temporal
head does not treat them as a pair.

### The feature dump

A directory of four files (layout version 1, described in `src/temporal.rs`):

- `meta.json`: checkpoint path and SHA-256, engine, sampling, views, the
  record layout, counts and timings.
- `clips.tsv`: one row per clip that decoded, with a header. The manifest's
  columns including `parent_id` to `transform`; where the clip's records
  start; width, height, container frame rate and duration; frames read;
  median, smallest and largest time between consecutive frames; median
  frame-to-frame difference; repeated frames.
- `frames.tsv`: one row per sampled frame, with a header: slot, index in
  the stream, presentation time, whole-frame-view logit, frame logit (mean
  over the views scored, as `score-video` writes), every view's logit, and
  for the stretch since the previous sample: frames and seconds apart,
  difference between the two whole-frame views, largest and summed
  frame-to-frame difference, repeated frames.
- `features.f32`: little-endian `f32`, one record per sampled frame. For
  each stored block: the class token (384 values), then the pooled patch
  tokens row by row (384 values per cell).
- `unreadable.tsv`: clips that did not decode, with the reason.

Frame-to-frame differences are mean absolute differences between 16×16
luma thumbnails of consecutive decoded frames, in 8-bit levels, taken of
every frame the decoder reads on the way to the samples (the reader is
sequential, so it reads them anyway).

Size: `frames × blocks × (1 + grid²) × 384 × 4` bytes per clip. With the
defaults (16 frames, the last block, a 7×7 grid) that is 1.23 MB per clip;
`--grid 2` is 123 kB, `--grid 14` (unpooled) 4.84 MB. Heads read a dump
clip by clip, so a dump larger than memory is fine.

**Decoder.** A sample's index is its position among the frames the
decoder handed over. On a heavily loaded machine the macOS system decoder
was seen to skip one frame of one clip in one of two otherwise identical
runs ([Measured here](#measured-here)): the samples and their timestamps
were the same, but the indices after the skip were one lower, the clip
showed one frame fewer read and a doubled gap. That is a decode artefact,
and it would look like cadence jitter to the covariates. For the dumps a
result rests on, use a quiet machine, or the pure-Rust reader
(`TINTYPE_DECODER=rust`), and check that `frames_read` and `gap_max` agree
between two runs.

**Precision.** `--device auto` runs the GPU in single precision for the
dump. Half precision (`--device gpu`, what scoring uses) keeps the logit
and most tokens close but moves a few patch tokens a long way
([Measured here](#measured-here)), and a head that reads differences
between tokens would be fitted partly to that noise. If a head is ever run
in half precision, measure it there.

### The heads

Every head outputs `a · mean(frame logits) + b + h(features)` and starts
as the frame mean exactly (`a = 1`, `b = 0`, `h = 0`), so what it learns is
a correction to the baseline. `--base frame` corrects the all-view frame
logit (same inference cost as today: the whole-frame view is one of the
six, so its tokens are free); `--base fit` corrects the whole-frame-view
logit alone (one view per frame, a sixth of the inference).

| `--kind` | Sees | Order | Parameters (384-wide, 7×7 grid, 32 hidden) |
|---|---|---|---|
| `pool-linear` | mean and maximum over frames of the class token | invariant | 770 |
| `attn-pool` | attention-weighted mean over frames of the class token | invariant | 770 |
| `patch` | one hidden layer on every cell of every frame, averaged | invariant | 12,354 |
| `diff` | the same network on (cell of sample *j*) − (same cell of sample *i*) for adjacent samples, with `ln Δt` | **temporal** | 12,386 |
| `concat` | the same network on both cells side by side, with `ln Δt` | **temporal** | 24,674 |
| `covariates` | frame rate, cadence jitter, repeated frames, cuts, motion, sample spacing: no pixels | – | 8 |

`Δt` is the actual time between the two samples. `diff` can also scale the
difference by it: `--time-norm sqrt` or `rate` (per second); the default
leaves the difference as it is and gives the network `ln Δt`.

Two samplings give "adjacent" two meanings, and both are worth a row:

- the default dump: the 16 frames scoring uses. Adjacent samples are up to
  1.9 s apart on a clip of 30 s or more, a quarter of a second on a
  four-second clip.
- `--pairs 1`: eight positions (the even-numbered ones of the 16), each
  followed by the very next frame of the stream. The same number of frames
  converted and scored, and truly adjacent frames. `--pairs N` takes the
  frame N frames later instead.

Both read the stream sequentially up to their last sample, so the decode
cost is the same to within one sample spacing; `eval-temporal-head` prints
the frames read per clip for the dump it is given.

### The controls

| Control | How | What it shows |
|---|---|---|
| Order-invariant heads | `pool-linear`, `attn-pool`, `patch` on the same dump | Whether a gain is temporal or a better pooled classifier |
| Shuffled order, at test | `eval-temporal-head` evaluates every head again with each clip's tokens permuted (times and positions stay), cutoffs unchanged | A temporal head should lose its gain; an order-invariant head cannot change (the table shows a change of 0) |
| Shuffled order, at fit | `fit-temporal-head --shuffle-frames`: the order-blind twin | What the same head reaches without order |
| Synthetic blur | `dump-video-features --temporal-blur N` on authentic clips: every sample is the mean of itself and the N−1 frames before it | Whether motion blur alone moves authentic clips over the cutoff |
| Authentic blur | Authentic low-shutter footage, scored like any test clip | The same, without the synthetic step |
| Cuts, repeats, motion, cadence | Slices in `eval-temporal-head`, from the trace of every frame read | A gain confined to one slice |
| Covariates alone | `--kind covariates`, and the single-covariate AUROC table | Whether the set can be told apart by how its clips were made |
| More frame views | `score-video --frames/--views` at the same view count, via `--scores` | The cheaper alternative |

Defaults of the slice rules, both adjustable (`--cut-above`,
`--cut-ratio`): a stretch between samples holds a **cut** when its largest
frame-to-frame thumbnail difference is above 12 levels and above 6 times
the clip's median; a frame is a **repeat** when it differs from the one
before by at most 0.01 levels. These are detection rules for slicing, not
tuned on anything; look at a few clips of each slice before trusting them
on a new corpus.

## Commands

Run from the repository root on macOS (the dump and `score-video` use the
GPU engine; everything also runs with `--device cpu`, and the fit and the
comparison are CPU-only everywhere). `data/`, `runs/` and the manifests
named here are placeholders for the cohort the video audit produces.

### 0. A copy of the video model

Nothing below writes beside the weights, but every command still names a
copy, never `models/`:

```sh
mkdir -p runs/ws7
cp models/tintype-video.safetensors runs/ws7/video.safetensors
cp models/tintype-video.json        runs/ws7/video.json
tintype verify-models
```

### 1. Split by parent, scene and source

One split per corpus, with whole sources kept together and at least one
authentic source held out for the test part:

```sh
tintype split-groups --manifest data/audit/matched/manifest.tsv \
  --out-dir data/splits/ws7-matched --together parent,scene,source \
  --hold-out-source <an authentic collection> --seed 1
tintype verify-split --together parent,scene,source \
  --set fit=data/splits/ws7-matched/fit.tsv \
  --set calibrate=data/splits/ws7-matched/calibrate.tsv \
  --set test=data/splits/ws7-matched/test.tsv
```

For the reposted corpus, pass `--existing data/splits/ws7-matched` so that
derivatives follow their parents. Never split by clip hash: reposts and
sibling generations of one scene would land on both sides.

### 2. Feature dumps

Three dumps per corpus and sampling (fit, calibrate, test):

```sh
for part in fit calibrate test; do
  tintype dump-video-features --weights runs/ws7/video.safetensors \
    --manifest data/splits/ws7-matched/$part.tsv --out runs/ws7/features/matched-$part
  tintype dump-video-features --weights runs/ws7/video.safetensors \
    --manifest data/splits/ws7-matched/$part.tsv --out runs/ws7/features/matched-pairs-$part --pairs 1
done
```

Each prints clips, unreadable clips, records, megabytes and milliseconds
per clip (end to end, in the network, decoding). Expect 1.23 MB per clip.
More than about 2,000 clips per run is a large GPU job: schedule it.

### 3. Fit heads on the fit part only

```sh
F=runs/ws7/features
for kind in pool-linear attn-pool patch diff concat covariates; do
  tintype fit-temporal-head --features $F/matched-fit --kind $kind --out runs/ws7/heads/$kind
done
# order-blind twins of the temporal heads
for kind in diff concat; do
  tintype fit-temporal-head --features $F/matched-fit --kind $kind --shuffle-frames --out runs/ws7/heads/$kind-blind
done
# truly adjacent frames
for kind in diff concat; do
  tintype fit-temporal-head --features $F/matched-pairs-fit --kind $kind --out runs/ws7/heads/$kind-pairs
done
```

Defaults: 20 epochs of Adam at 1e-3 with cosine decay, weight decay 1e-4,
batches of 64 clips, both classes weighted equally, seed 1; a fit is
deterministic. Hyperparameters are chosen on the **fit** part (hold some
scenes of it out with a second `split-groups` and pass that dump as
`--watch`), never on calibrate or test. Write down the settings before
looking at the test tables; every head fitted is reported, not the best
one.

### 4. More frame views at the same cost

The baseline is 16 frames × 6 views = 96 views per clip. The same 96
spent on more frames:

```sh
for part in calibrate test; do
  tintype score-video --weights runs/ws7/video.safetensors --all-frames \
    --manifest data/splits/ws7-matched/$part.tsv --frames 32 --views fit,native,rich1 \
    --out runs/ws7/scores/f32v3-$part.tsv
  tintype score-video --weights runs/ws7/video.safetensors --all-frames \
    --manifest data/splits/ws7-matched/$part.tsv --frames 48 --views fit,rich1 \
    --out runs/ws7/scores/f48v2-$part.tsv
done
```

`score-video` prints clips per second and milliseconds per clip for each.

### 5. Compare on the test part, at cutoffs from the calibrate part

```sh
H=runs/ws7/heads
tintype eval-temporal-head --features $F/matched-test --calibrate $F/matched-calibrate \
  --head pool-linear=$H/pool-linear --head attn-pool=$H/attn-pool --head patch=$H/patch \
  --head diff=$H/diff --head concat=$H/concat \
  --head diff-blind=$H/diff-blind --head concat-blind=$H/concat-blind \
  --head covariates=$H/covariates \
  --scores f32v3=runs/ws7/scores/f32v3-test.tsv,runs/ws7/scores/f32v3-calibrate.tsv \
  --scores f48v2=runs/ws7/scores/f48v2-test.tsv,runs/ws7/scores/f48v2-calibrate.tsv \
  --out-dir runs/ws7/scores/matched-test > runs/ws7/matched-test.md
tintype eval-temporal-head --features $F/matched-pairs-test --calibrate $F/matched-pairs-calibrate \
  --head diff-pairs=$H/diff-pairs --head concat-pairs=$H/concat-pairs \
  --out-dir runs/ws7/scores/matched-pairs-test > runs/ws7/matched-pairs-test.md
```

The report has, for the 1% and the 0.1% cutoff: each arm's cost, cutoff,
where the cutoff came from and how many false alarms that rate allows
there, authentic clips flagged with a Clopper–Pearson bound, AI clips
caught with an interval, clips gained and lost against the frame mean, the
recall difference against the frame mean with a bootstrap interval over
scenes (or parents), and AUROC; then the shuffled-order control; then every arm by cuts, repeated
frames, motion, frame rate and sample spacing; then what each covariate
alone says. A rate the clips cannot resolve is marked **Not resolved**.

For clustered intervals on each arm's own recall and for slices by source
and generator family, use the score files it wrote; the command is printed
at the end of the report:

```sh
tintype report --operating-points --cluster-by scene --slice-by source,family \
  --groups data/splits/ws7-matched/test.tsv \
  --system frame-mean=runs/ws7/scores/matched-test/frame-mean.tsv@<cutoff> \
  --system diff=runs/ws7/scores/matched-test/diff.tsv@<cutoff>
```

Repeat steps 2 to 5 on the reposted corpus with the **same heads and the
same cutoffs**: dump its test part and pass the matched corpus's calibrate
dump as `--calibrate`. Refitting or recalibrating on the second corpus
would make it a second first corpus.

### 6. Blur controls

```sh
awk -F'\t' '$2 == 0' data/splits/ws7-matched/test.tsv > runs/ws7/test-authentic.tsv
tintype dump-video-features --weights runs/ws7/video.safetensors \
  --manifest runs/ws7/test-authentic.tsv --out $F/matched-test-blur3 --temporal-blur 3
tintype eval-temporal-head --features $F/matched-test-blur3 --calibrate $F/matched-calibrate \
  --head patch=$H/patch --head diff=$H/diff --head concat=$H/concat --no-shuffle
```

A manifest written outside its data directory needs absolute paths in its
first column (`split-groups` writes them). On authentic clips only, the
report gives the clips flagged at the frozen cutoffs per arm; compare with
the same clips' rows in the unblurred test report. `--temporal-blur 3`
averages three consecutive frames (a tenth of a second at 30 fps); also run
5. Authentic low-shutter footage, when a collection has it, goes through
steps 2 and 5 as its own test manifest.

### 7. Cost accounting

Report these for every arm, from the runs above, on the hardware that would
run it:

| Quantity | Where it comes from |
|---|---|
| Frames read per clip | `eval-temporal-head`, second line of the report (per dump) |
| Frames sampled and views scored per clip | the "Cost per clip" column |
| Head parameters | the "Cost per clip" column |
| Wall time per clip, score path | `score-video`: "ms/clip", for the 16×6 baseline and each `--frames/--views` arm |
| Wall, network and decode time per clip, dump path | `dump-video-features` summary line and `meta.json` (`wall_s`, `model_s`, `decode_s`) |

The dump path is not the cost of a deployed head: it runs every block in
full for the whole-frame view (scoring skips most of the last block), in
single precision, with no early exit. A head that corrects the all-view
frame logit also gives up frame-level early exit as it stands, which today
stops at about 4.8 frames per clip on the blind batch
([BENCHMARK.md](../BENCHMARK.md)); an honest comparison is against
`score-video --all-frames` for accuracy and against the default
`score-video` for cost, and the write-up has to state both.

## Reading the result

| What the tables show | Conclusion |
|---|---|
| Temporal head ≤ frame mean at the frozen cutoffs on either corpus | No gain. Keep the frame mean |
| Temporal head > frame mean, but an order-invariant head or the order-blind twin does as well | The gain is pooling, not time. Consider the order-invariant head as its own, simpler proposal; it still needs the frozen split |
| Temporal head > order-invariant, and shuffling removes the gain, on one corpus only | Not passed: the gate needs both corpora |
| Gain confined to one cadence, repeat or cut slice, or `covariates` separates the classes | Shortcut. Fix the cohort (match cadence, re-encode to one frame rate), then repeat |
| Blurred authentic clips flagged more by the temporal head than by the frame mean | The head reads blur. Not passed |
| `--frames 32 --views fit,native,rich1` (or 48×2) matches the temporal head | Prefer more frames: no new parameters or path |
| All of gate items 2 and 3 hold on both corpora | Passed as a proposed criterion: write the pre-registration entry, then the frozen split, calibration and the early-exit re-check before anything ships |

## Measured here

Everything in this section was run on 2026-11-14 with the build of commit
`c116382` on an Apple M6 (12 cores, 32 GB, macOS 27.2). The machine and
its GPU were shared with other jobs throughout (load average between 21
and 96 during these runs), so **the timings are upper bounds on a noisy
machine, not benchmarks**: the same GPU dump took 28.2 s once and 56.8 s
once.

### Engine parity

| Check | Command | Result |
|---|---|---|
| CPU logits with tokens read out, small random network | `cargo test --release --locked --lib cpu_vit` | bit-identical to `forward` for 1 and 3 views and every set of blocks tried |
| CPU logits with tokens read out, released photo model on the fixture | `cargo test --release --locked --test inference reading_patch_tokens` | bit-identical to `Detector::logits` for the six views |
| GPU against CPU tokens, fixture (6 views × 197 tokens, blocks 12 and 6) | `cargo test --release --locked --test inference -- --ignored gpu_and_cpu --nocapture` | below |
| GPU against CPU, 1,600 frames of the 100 DaVODi clips (class token and 7×7 pooled tokens of block 12) | two dumps compared value by value | below |

GPU against the CPU engine, single precision (`gpu-f32`):

| | Logit, largest difference | Token values, mean absolute difference | Per token, relative difference: median / 99th percentile / largest | Smallest cosine |
|---|---|---|---|---|
| Fixture, block 12 | 0.0000145 | 0.0000129 | 0.0000064 / 0.000029 / 0.00028 | 0.9999991 |
| Fixture, block 6 | – | 0.0000074 | 0.0000036 / 0.000013 / 0.000047 | 0.9999993 |
| 1,600 DaVODi frames, pooled | 0.00009 (whole-frame view), 0.00003 (frame logit), as written to five decimals | 0.0000082 | largest 0.00085 | – |

GPU against the CPU engine, half precision (`gpu`, what scoring uses):

| | Logit, largest difference | Token values, mean absolute difference | Per token, relative difference: median / 99th percentile / largest | Smallest cosine |
|---|---|---|---|---|
| Fixture, block 12 | 0.0125 | 0.0124 | 0.0056 / 0.043 / 0.30 | 0.969 |
| Fixture, block 6 | – | 0.0072 | 0.0030 / 0.015 / 0.10 | 0.995 |
| 1,600 DaVODi frames, pooled | 0.098 (whole-frame view), 0.035 (frame logit) | 0.0091 | largest 0.35 | – |

So the two engines agree to rounding in single precision, and half
precision is fine for the logit and for the typical token but not for
every token: the worst patch token is off by about a third of its norm.
The test's bounds are several times the single-precision measurements
(largest relative difference 0.002, 99th percentile 0.0003, cosine
0.99999, logit 0.001) and deliberately loose for half precision (median
0.03, 99th percentile 0.2, mean absolute 0.05, cosine 0.9, logit 0.05).
The GPU test needs a Metal device and is `#[ignore]`d in the default run.

The default path did not move: `detect --json` on three images and the two
fixture clips gives byte-identical output to the binary of the base branch
on the CPU and on the GPU engine, and `python3 scripts/replay-research.py`
prints the same 39 lines.

### Pipeline check on development clips

The 100 DaVODi clips (20 scenes × one Pixabay clip and four generators,
normalised to four seconds at 24 fps, about 800×450; development data, one
authentic source). `split-groups --together parent,scene --seed 1` gave
60 / 20 / 20 clips with 12 / 4 / 4 authentic ones. **With four authentic
clips in the calibration part and four in the test part, neither the 1% nor
the 0.1% operating point is resolved, and the harness says so in its
output. No accuracy number from these runs is reported here or anywhere:
they show that every step runs and what it costs.**

| Step | Command (inputs under the scratch directory) | Measured |
|---|---|---|
| Dump, GPU single precision, 100 clips | `dump-video-features --weights runs/ws7/video.safetensors --manifest davodi/manifest.tsv` | 1,600 records, 122.9 MB (1.23 MB per clip), 0 unreadable; 28.2 s (282 ms per clip, 278 ms in the network) in one run and 56.8 s (568 ms per clip) in another |
| Dump, GPU half precision, 100 clips | the same with `--device gpu` | 26.2 s (262 ms per clip, 258 ms in the network) |
| Dump, CPU engine (Accelerate), 100 clips, 4 decode threads | the same with `--device cpu` | 280.5 s (2.81 s per clip) at load average about 40; 623.7 s (6.24 s per clip) at load average about 90 |
| Dump of the fit part, spread and `--pairs 1` | 60 clips each | 73.7 MB each; 15.2 s and 16.8 s |
| Frames read per clip, spread and `--pairs 1` | from `eval-temporal-head` on the 20 test clips | 91.0 and 86.0 of the 96 each clip has |
| Dump with `--blocks 6,12 --grid 2`, 20 clips | | 4.9 MB (3,840 values per record), 6.3 s |
| Dump with `--temporal-blur 3`, 4 authentic clips | | 4.9 MB, 1.7 s |
| Baseline scoring, 20 clips | `score-video --all-frames` (16 frames × 6 views, half precision) | 137.5 and 124.7 ms per clip (two parts) |
| The same in single precision | `score-video --all-frames --device gpu-f32` | 293.5 ms per clip |
| More frames at 96 views, 20 clips | `--frames 32 --views fit,native,rich1` / `--frames 48 --views fit,rich1` | 127.0 and 127.1 / 125.3 and 120.7 ms per clip |
| Fit, 60 clips, 20 epochs, CPU | `fit-temporal-head` | pool-linear 0.5 s, attn-pool 0.6 s, patch 1.1 s, diff 1.1 s, concat 2.1 s, covariates under 0.1 s; with `--shuffle-frames` diff 1.2 s, concat 2.7 s |
| Comparison, 20 test clips, 21 arms | `eval-temporal-head` with eight heads, their shuffled runs and three score files | 0.4 s; 21 score files written and read by `report --operating-points --cluster-by scene` |

What the check showed beyond "it runs":

- The dump's frame logits are `score-video`'s: against the archived
  DaVODi run (`docs/results/davodi-2026-video/frame-scores.tsv.gz`: release
  0.1.0, half precision, frames in early-exit looking order), all 966
  archived frame logits of the 100 clips are within 0.035 of the CPU
  dump's.
- On the 20 test clips the cadence covariates are constant, as they should
  be for a set normalised to one frame rate and length: frame rate, cadence
  jitter, repeats, cuts and sample spacing each give AUROC 0.500 alone.
- One decode skip: in the CPU dump made at load average about 90, one clip
  (`15-pixabay.mp4`) shows 90 frames read instead of 91 and a largest gap
  of 0.0833 s instead of 0.0417 s, with the same samples and timestamps.
  The other four dumps of the 100 clips, three single-clip repeats with the
  system decoder and one with the pure-Rust reader show 91. See **Decoder**
  above.
- A `diff` fit of 60 clips for 20 epochs took 1.1 s: about a millisecond
  per clip and epoch on this machine. Larger fits were not timed.
- In its faster run the single-precision dump cost about what
  single-precision scoring cost (282 against 294 ms per clip), and a little
  over twice half-precision scoring (125 to 138 ms per clip).

## Not built, on purpose

**The opt-in inference path** (`detect --temporal-head`, a
`temporal_head` field in the video spec). The brief for this workstream
made it conditional on everything else being complete, and a flag without
a fitted, gated head behind it would be half-wired. No spec field and no
flag were added: the shipped specs load and serialise as before because
nothing about them changed. The intended design, for when a head passes
the gate:

- **Spec.** `VideoSpec.temporal_head: Option<TemporalHead>` with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`, like
  `second_opinion` and `early_exit`, holding the head's `HeadSpec` (kind,
  block, grid, hidden, pairing, time normaliser), the name of its tensor
  file beside the weights, and its own `logit_scale` and `logit_bias`
  (a head changes the score distribution, so it needs its own calibration;
  `calibrate --profile video` would gain a `--temporal-head` variant). A
  test asserts that both shipped specs round-trip byte-identical.
- **Flag.** `detect --temporal-head`, off by default, an error when the
  spec has no head. With it: all frames of the profile are scored (no
  frame-level early exit; the head needs every sample), the whole-frame
  view goes through the token graph (`Embedder`) and the other views
  through the scoring graph, timestamps come from
  `video::sample_frames_timed`, and the clip logit is `Head::logit`.
  Incomplete decode coverage stays `inconclusive` as today.
- **Output.** No new `verdict` value. New additive JSON fields, documented
  in `docs/API.md`: `temporal_head: true`, `frame_mean_logit` (what the
  default path would have used) and `head_logit`.
- **Engines.** Half precision moves some patch tokens (see above): the
  flag would use single precision for the token graph, or the head would
  be fitted on half-precision dumps, and parity between the two would be
  measured before either is chosen.
- **Cost.** The full last block for one view per frame, and no early exit:
  measured against today's default with `latency`-style timing before the
  flag is offered.

**Also not built:** native-resolution tokens with one fixed window per
clip; optical flow or any motion estimate beyond thumbnail differences;
fine-tuning the backbone with a temporal objective (the trainer sees one
frame per draw); a time-reversal control (the shuffled-order controls
cover order; reversal would ask about direction).
