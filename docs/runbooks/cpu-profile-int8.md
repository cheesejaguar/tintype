# Runbook: where CPU time goes, INT8 and a shallower student

This is item 8 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
("Profile a shallower student and portable INT8"). It asks three things in
order: where a CPU run spends its time, whether the linear layers survive
eight bits, and what a model with fewer blocks would cost. Nothing here
changes a default: `--device cpu`, `gpu` and `gpu-f32` give the bytes they
gave before, the shipped models are untouched, and no model, threshold or
exit rule is chosen.

| Step | State | Where |
|---|---|---|
| 1. Timers inside the CPU forward pass, and the table of where time goes | built, measured on ARM64 (Accelerate and pure-Rust kernels) | `src/stats.rs`, `src/cpu_vit.rs`, `scripts/cpu-profile.py` |
| The same on x86-64 and on Linux | **not run**: no such machine here; commands below | [Ready to run elsewhere](#ready-to-run-elsewhere) |
| 2. INT8 simulation behind `--device cpu-int8` | built | `cpu_vit::Int8`, `detector::Device::CpuInt8` |
| 3. Agreement of INT8 with FP32 on 3,148 labelled store images | measured (training data: a parity check, not an evaluation) | `scripts/int8-agreement.py` |
| The same on the frozen split and the suites | **not run**: the files are not on this machine; commands below | [Ready to run elsewhere](#ready-to-run-elsewhere) |
| 4. An integer kernel timed against the FP32 products | benchmarked on ARM64, test-only, **not wired into scoring**; no x86-64 kernel | `src/cpu_vit/int8_gemm.rs` |
| 5a. What 6, 7 and 8 blocks would cost to run | timed with the released weights cut short (timing only) | `vit_small_patch16_d<N>` in a spec |
| 5b. Exits part-way with the weights as shipped | measured with a fitted linear read-out | `tintype probe-exits`, `src/exit_probe.rs` |
| 5c. Training a student | code and commands, **no training run** | `train --blocks`, `--init-blocks`, `--distill`; `src/distill.rs` |

## The gate

Written before the measurements. The report's wording: "The initial target
is 1.3× CPU throughput or 25% lower p95, with at most one point of recall
loss on each clean and degraded suite at equal FPR." It is a proposed
decision criterion, not a release decision: passing it here would justify
building and then evaluating a real kernel or training a student, nothing
more.

1. **Speed.** File-to-verdict throughput on the CPU engine at least 1.3× the
   FP32 engine's on the same files, **or** single-image file-to-verdict p95
   at least 25% lower, on the hardware that will run it, binaries taking
   turns.
2. **Accuracy.** At equal false-positive rate, recall no more than one point
   below FP32 on each clean and each degraded suite, at 1% and at 0.1% FPR.

What this machine can decide of that, and what it cannot:

- The named suites (confirmation 7, Arena, Frontier, NTIRE, every
  `data/fresh*` set) are manifests without their files here, and the archived
  score files hold FP32 scores only. **Recall on the suites cannot be
  measured on this machine for anything in this runbook.** The accuracy
  clause is therefore not decided here.
- What stands in is agreement with FP32 on labelled images from the training
  stores: logit differences, verdict flips at the shipped threshold, changed
  early-exit stages, and recall at 1% and 0.1% empirical FPR with the cutoff
  chosen on the same sample. **Those images are training data**, so the
  recall figures are a check that INT8 tracks FP32, not an evaluation, and
  a cutoff chosen and reported on one cohort is exploratory. A variant that
  moves recall by more than a point *even there* fails; one that does not
  has only earned the suite run.
- Speed is measured for ARM64 only. A fake-quantised simulation is slower
  than FP32 by construction, so the speed clause is bounded from a kernel
  micro-benchmark and the profile (Amdahl), not measured end to end.

## Hardware, build and load

- Apple M6, 12 cores (three performance levels), 32 GB, macOS 27.2. `rustc`
  1.99.1 from the stable toolchain. "Accelerate" is `cargo build --release
  --locked --bin tintype`; "pure-Rust kernels" is the same with `--features
  portable-cpu`, which is the code Linux and Windows run (the `gemm` crate's
  NEON kernels here).
- There is no x86-64 machine and no Linux machine here. The pure-Rust
  kernels on this Mac are the ARM64 half of the portable engine, under
  macOS; the documented 35.2 photos/s came from an aarch64 Ubuntu virtual
  machine on 8 of this Mac's cores, and is not reproduced here.
- Nothing else was running. The load average quoted with each block is the
  machine's one-minute figure before and after it; most of it is the
  measurement itself (the engine's worker threads spin between runs).
  Binaries take turns in every timing, as in
  [the view-reuse runbook](view-reuse.md#how-time-is-measured), whose
  scripts and cohorts are reused.

## Cohorts

| Cohort | Files | What |
|---|---|---|
| Ordinary photos | 308 | the timing cohort of [the view-reuse runbook](view-reuse.md#cohorts): 22 files from each of 14 photo stores under `data/store`, fixed seed, 0.01 to 45 megapixels. `photos.txt` lists them, `photos.tsv` is the same as a manifest |

| Labelled store sample | 3,148 | `scripts/int8-agreement.py --make-cohort data/store --per-store 300 --seed 1`: up to 150 authentic and 150 generated files from each of the 13 stores whose files are on this machine; 1,350 authentic, 1,798 generated. `cohort.tsv`. **Training data** |

The lists are kept outside the repository, as the data is.

## Step 1: where the time goes

### What is timed

`TINTYPE_STATS=1` already made `detect`, `batch` and `latency` print one
line of counters on standard error ([view-reuse runbook](view-reuse.md#what-is-counted)).
The line now goes on with the inside of the CPU forward pass; standard
output is unchanged and so is every score (checked below).

| Counter | What it times |
|---|---|
| `decode_ms` | reading and decoding image files (and video frames) |
| `views_ms` | ranking windows, resizing, cropping, putting views in the engine's form |
| `model_ms` | the scoring thread inside the network |
| `forward_ms` | CPU forward passes, each on the thread that ran it. Equal to `model_ms` when a batch is one pass (Accelerate); larger where the views of a batch run side by side, one per thread (the pure-Rust kernels) |
| `patch_ms` | unfolding and normalising pixels, the patch-embedding product, class token and positions |
| `qkv_ms`, `proj_ms` | the two linear layers of attention (bias and product) |
| `fc1_ms`, `fc2_ms` | the two linear layers of the MLP |
| `attention_ms` | the query-key and attention-value products (activation times activation: no weights) |
| `softmax_ms`, `gelu_ms`, `norm_ms` | softmax, GELU, layer normalisation |
| `gather_ms` | the rest of attention: copying each head's rows together and its output back |
| `residual_ms` | adding each half-block's output to the residual stream |
| `head_ms` | the classification head |

What the parts leave of `forward_ms` is allocating buffers and gathering
the class tokens. Attention runs one task per view; the time of its
products and softmax is measured inside the tasks and the wall time of the
section is divided in those proportions. With `TINTYPE_STATS`, `latency`
also prints the warm-up images' times (`warmup_ms`), because the counters
include them.

### How it was measured

```bash
ACC=target/release/tintype            # cargo build --release --locked --bin tintype
PORT=target2/release/tintype          # CARGO_TARGET_DIR=target2 cargo build --release --locked --bin tintype --features portable-cpu

# One image at a time, file to verdict (early exit: mostly one view per network run).
python3 -I scripts/view-reuse-bench.py --bin accelerate=$ACC --bin portable=$PORT \
    --latency --manifest photos.tsv --n 300 --repeat 5 --device cpu > latency.jsonl
# All files in one process (throughput).
python3 -I scripts/view-reuse-bench.py --bin accelerate=$ACC --bin portable=$PORT \
    --together --list photos.txt --repeat 5 --device cpu > together.jsonl
# The same two with --extra=--all-views --repeat 3 (six views of every file).
python3 -I scripts/cpu-profile.py latency.jsonl      # and together.jsonl: the tables below
```

Milliseconds are per file, averaged over the passes. In the latency tables
everything happens on one thread after another and the rows add up to the
images' own times. In the throughput tables files are decoded and views
prepared on worker threads while the network runs, so those two rows are
processor time on other threads and have no share; the shares are of the
wall time of the process. Where views run side by side (pure-Rust kernels,
throughput) the parts of the forward pass are scaled to the time the scoring
thread waited for them.

### Many files in one process (throughput), default profile

| Part | accelerate, throughput, `cpu`: ms per file | share | portable, throughput, `cpu`: ms per file | share |
|---|---:|---:|---:|---:|
| Reading and decoding the file (worker threads) | 7.99 |  | 12.01 |  |
| Ranking windows, resizing and cropping views (worker threads) | 3.62 |  | 4.68 |  |
| In the network | 13.39 | 97.9% | 23.41 | 98.9% |
| - Patch embedding (unfold, normalise, linear, positions) | 0.56 | 4.1% | 0.33 | 1.4% |
| - Attention: query/key/value linear | 2.82 | 20.6% | 4.82 | 20.4% |
| - Attention: output linear | 0.87 | 6.4% | 1.50 | 6.4% |
| - MLP: first linear | 3.53 | 25.8% | 5.89 | 24.9% |
| - MLP: second linear | 2.53 | 18.5% | 5.87 | 24.8% |
| - Attention: query-key and attention-value products | 0.55 | 4.0% | 1.64 | 6.9% |
| - Softmax | 0.58 | 4.3% | 0.73 | 3.1% |
| - Attention: copying heads together and back | 0.22 | 1.6% | 0.16 | 0.7% |
| - GELU | 0.72 | 5.2% | 1.29 | 5.5% |
| - Layer normalisation | 0.72 | 5.3% | 1.09 | 4.6% |
| - Head | 0.00 | 0.0% | 0.00 | 0.0% |
| - Residual additions | 0.26 | 1.9% | 0.07 | 0.3% |
| - Buffers, gathering class tokens | 0.05 | 0.3% | 0.02 | 0.1% |
| - All four linear layers of the blocks together | 9.74 | 71.3% | 18.08 | 76.4% |
| Outside the network (start-up, weights, waiting for files, output) | 0.28 | 2.1% | 0.27 | 1.1% |
| Wall time per file | 13.67 | 100.0% | 23.68 | 100.0% |

- accelerate, throughput, `cpu`: 308 files, 72.2 files/s (median of 5), best 78.3, 1.786 views per file, peak memory 1398.6 MB, load average 5.26 before and 7.27 after
- portable, throughput, `cpu`: 308 files, 42.3 files/s (median of 5), best 43.9, 1.786 views per file, peak memory 1288.0 MB, load average 5.26 before and 7.27 after

### One image at a time (latency), default profile

| Part | accelerate, latency, `cpu`: ms per file | share | portable, latency, `cpu`: ms per file | share |
|---|---:|---:|---:|---:|
| Reading and decoding the file | 5.13 | 14.7% | 5.28 | 9.0% |
| Ranking windows, resizing and cropping views | 1.23 | 3.5% | 1.28 | 2.2% |
| In the network | 28.51 | 81.7% | 52.37 | 88.9% |
| - Patch embedding (unfold, normalise, linear, positions) | 0.47 | 1.3% | 0.77 | 1.3% |
| - Attention: query/key/value linear | 3.71 | 10.6% | 7.43 | 12.6% |
| - Attention: output linear | 2.53 | 7.3% | 5.16 | 8.8% |
| - MLP: first linear | 4.47 | 12.8% | 8.03 | 13.6% |
| - MLP: second linear | 4.53 | 13.0% | 9.73 | 16.5% |
| - Attention: query-key and attention-value products | 1.39 | 4.0% | 10.82 | 18.4% |
| - Softmax | 3.22 | 9.2% | 3.60 | 6.1% |
| - Attention: copying heads together and back | 0.69 | 2.0% | 0.87 | 1.5% |
| - GELU | 1.90 | 5.5% | 1.78 | 3.0% |
| - Layer normalisation | 1.71 | 4.9% | 1.77 | 3.0% |
| - Head | 0.00 | 0.0% | 0.00 | 0.0% |
| - Residual additions | 3.83 | 11.0% | 2.33 | 4.0% |
| - Buffers, gathering class tokens | 0.06 | 0.2% | 0.07 | 0.1% |
| - All four linear layers of the blocks together | 15.25 | 43.7% | 30.35 | 51.5% |
| Everything else | 0.01 | 0.0% | 0.01 | 0.0% |
| File to verdict | 34.88 | 100.0% | 58.93 | 100.0% |

- accelerate, latency, `cpu`: 300 files, p50 31.1 / p95 84.6 / mean 34.8 ms (each image's best of 5), 544 views scored, peak memory 665.4 MB, load average 7.54 before and 7.85 after
- portable, latency, `cpu`: 300 files, p50 46.2 / p95 149.1 / mean 58.8 ms (each image's best of 5), 544 views scored, peak memory 675.7 MB, load average 7.54 before and 7.85 after

### All eight runs in one view

| Path | Kernels | Speed | In the network | The four linear layers of the blocks | Decoding and preparing views |
|---|---|---|---:|---:|---:|
| Throughput, default | Accelerate | 72.2 files/s | 96.7% | **70.4%** | other threads: 11.6 ms of processor time per file |
| Throughput, default | pure Rust | 42.3 files/s | 99.1% | **76.5%** | other threads: 16.7 ms |
| Throughput, `--all-views` | Accelerate | 24.0 files/s | 100% | **75.2%** | other threads: 10.3 ms |
| Throughput, `--all-views` | pure Rust | 12.9 files/s | 98.4% | **75.5%** | other threads: 15.1 ms |
| Latency, default | Accelerate | p50 31.1 / p95 84.6 ms | 81.7% | **43.7%** | 18.2% |
| Latency, default | pure Rust | p50 46.2 / p95 149.1 ms | 88.9% | **51.5%** | 11.2% |
| Latency, `--all-views` | Accelerate | p50 49.1 / p95 65.2 ms | 86.8% | **58.9%** | 13.2% |
| Latency, `--all-views` | pure Rust | p50 111.1 / p95 126.0 ms | 93.6% | **78.9%** | 6.4% |

Commit: the one that adds the timers (step 1 of this branch). Load average
5.3 to 11.7 over the throughput runs and 7.5 to 8.4 over the latency runs,
nothing else running. `--all-views` runs: 3 passes; the others: 5. Speeds
are the median pass (throughput) and each image's best pass (latency).

A control for the timers themselves: the base binary (`624fa39`) and the
instrumented one, taking turns over the 300 photos in a latency run, gave
p50 30.9 and 30.9 ms, p95 84.3 and 83.8 ms, mean 34.8 and 34.8 ms (each
image's best of 5). With no new flags `detect --json` on the demo image and
the fixture clip and `demo --json` are the same bytes as the base binary's
on `cpu`, `gpu` and `gpu-f32`.

### What the profile says

1. **With many files the network is the whole cost, and three quarters of
   it is the four linear layers of the blocks**: 70% of wall time with
   Accelerate, 76% with the pure-Rust kernels. Decoding and view preparation
   run on other threads and do not hold the network up on either (the
   scoring thread is outside the network for 3.3% and 0.9% of the time).
   This is the case for looking at INT8, and for fewer blocks: both act on
   exactly this share.
2. **One image at a time, the linear layers are about half** (44% with
   Accelerate, 52% pure Rust), decoding and views 18% and 11%. Much of the
   rest is not arithmetic: with Accelerate, adding the residual costs 3.8 ms
   per file (11%), softmax 3.2 ms, GELU 1.9 ms, layer normalisation 1.7 ms.
   Per view the residual addition costs 2.15 ms in a single-view pass and
   0.14 ms in a batch (1167 ms over 544 views against 79 ms over 550): these
   steps hand 75,000 floats to the thread pool one pass at a time, and the
   hand-off costs more than the work. That is a saving available without
   touching a number (the additions are element-wise), separate from INT8;
   it is listed under [Seen on the way](#seen-on-the-way-not-changed).
3. **The pure-Rust kernels are 1.7 times slower than Accelerate** on this
   chip in the throughput path (42.3 against 72.2 files/s) and 1.5 times at
   the median image (46.2 against 31.1 ms), and their attention products are
   a visible share one image at a time (18%): the 197 x 197 x 64 products
   are small for a general kernel.
4. **A p95 photo is slower through early exit than through one batch.** With
   Accelerate the p95 of the default profile is 84.6 ms, of `--all-views`
   65.2 ms: a photo that needs all six views runs them one or two at a time
   in five passes, and a pass of one view costs about 16 ms where six in one
   pass cost 45. The median goes the other way (31.1 against 49.1 ms).

### What a faster linear layer could buy (Amdahl)

If the four linear layers take a share `f` of the time and a kernel makes
them `k` times faster, the whole gets `1 / (1 - f + f / k)` times faster.
From the shares above (arithmetic, not a measurement):

| Path | Kernels | `f` | `k` needed for 1.3x | `k` needed for 25% less time | Whole speed-up at `k` = 2 | at `k` = 3 | Ceiling |
|---|---|---:|---:|---:|---:|---:|---:|
| Throughput, default | Accelerate | 0.704 | 1.49 | 1.55 | 1.54x | 1.88x | 3.38x |
| Throughput, default | pure Rust | 0.765 | 1.43 | 1.49 | 1.62x | 2.04x | 4.26x |
| Latency, default | Accelerate | 0.437 | 2.12 | 2.34 | 1.28x | 1.41x | 1.78x |
| Latency, default | pure Rust | 0.515 | 1.81 | 1.94 | 1.35x | 1.52x | 2.06x |
| Latency, `--all-views` (what a p95 photo is made of) | Accelerate | 0.589 | 1.64 | 1.74 | 1.42x | 1.65x | 2.43x |
| Latency, `--all-views` | pure Rust | 0.789 | 1.41 | 1.46 | 1.65x | 2.11x | 4.74x |

So the linear layers are most of the time where throughput is measured, and
a kernel about 1.5 times faster than the FP32 one it replaces would meet
the speed clause there. Whether eight bits keep the scores is steps 2 and
3; whether an integer kernel is that much faster is step 4.

## Step 2: simulating INT8 before writing a kernel

`--device cpu-int8` is the `cpu` engine with some values rounded to what an
eight-bit kernel would hold, kept as `f32` and put through the same
products ("fake quantisation"). It answers one question, what INT8 does to
the scores, and is slower than `cpu` while doing so. It is never chosen by
`--device auto`, `serve` does not offer it, it takes every input `cpu`
takes, and `cpu`, `gpu` and `gpu-f32` are not touched by it: the rounding
is one `if` that the engine proper never enters, the unit tests hold a
network built with nothing to round to the engine's bits, and the parity
checks of step 1 were repeated on this commit.

### What is rounded, and what is not

| Kept in single precision, always | Why |
|---|---|
| Patch embedding | the first layer; its input is already eight-bit pixels, and it is 1 to 4% of the time |
| Layer normalisation | divides by a per-token deviation: small errors are amplified |
| Query-key and attention-value products, softmax | activation times activation, and the values after softmax span many orders of magnitude; PTQ4ViT names these as one of the two places where eight uniform levels fail. 4 to 7% of throughput time (products), 3 to 4% (softmax) |
| GELU itself, the residual stream | the residual stream carries every block's output and is never re-rounded |
| Head | one product per view |

| `TINTYPE_INT8=` | Weights rounded | Inputs rounded |
|---|---|---|
| `weights` | qkv, proj, fc1, fc2 | none |
| `mlp` | qkv, proj, fc1, fc2 | fc1, fc2 |
| `all` (the default) | qkv, proj, fc1, fc2 | qkv, proj, fc1, fc2 |
| `qkv`, `proj`, `fc1`, `fc2` | all four | that layer's only (to find the sensitive one) |
| `inputs` | none | all four |

The table is `cpu_vit::Int8::VARIANTS`. A variant may be followed by
`:token` (the default) or `:tensor`, then by the number of bits of an input
and then of a weight (eight each by default), as in `all:tensor`,
`all:token:7` or `all:token:8:7`.

**Weights** are rounded once, when the model is loaded: each output channel
(row of the weight matrix) to 255 levels symmetric about zero, the step set
by the row's largest magnitude. This is the usual per-channel scheme and
costs an integer kernel nothing, because a row's step multiplies that
row's sum.

**Inputs** are rounded as they arrive, to 256 evenly spaced levels between
the smallest and the largest value of their range (an unsigned byte, a step
and an offset). Two choices were made here, and one alternative is left
for whoever runs the calibrated comparison:

- *Ranges are taken from the values themselves, not from a calibration
  pass.* A calibrated (static) range clips whatever falls outside what the
  calibration images showed, and a detector's errors live in exactly the
  unusual images; a range taken from the tensor at hand cannot clip. It
  also needs no extra file beside the model, whose files are pinned by
  hash. The price is one pass over each input to find its extremes, which
  step 4 times, and resolution spent on outliers, which is what the
  per-token ranges are for.
- *A range per token by default* (`:token`), not per view (`:tensor`). A
  token is a row of the product, so an integer kernel can give every token
  its own step and offset for free: they multiply and shift that row's
  sums. A few tokens with very large values then cost only themselves
  their resolution. A range per input *channel* would need calibration
  (it has to be folded into the weights beforehand) and is not simulated.
- *The range is lopsided where the values are.* After GELU (the input of
  fc2) values lie between about -0.17 and a long positive tail; PTQ4ViT's
  second finding is that a symmetric range wastes half the levels there.
  An unsigned byte between the smallest and largest value does not, though
  it still spends most levels on the tail (PTQ4ViT's twin ranges, one for
  the negative side and one for the positive, are not simulated).

`all:token:7` rounds inputs to seven bits and `all:token:8:7` weights. The
AVX2 multiply-add (`vpmaddubsw`) adds two products in sixteen bits, which
holds without saturating when one of the two has seven bits, so these two
stand for an x86-64 machine without VNNI.

### What an integer kernel would compute

For a token with rounded input `x[k] = lo + step * q[k]` (`q` from 0 to
255) and weights `w[j][k] = s[j] * v[j][k]` (`v` from -127 to 127):

```text
y[j] = b[j] + s[j] * (step * sum_k q[k] * v[j][k] + lo * sum_k v[j][k])
```

The first sum is the integer product, at most 255 x 127 x 1536, well inside
32 bits; the second is a constant per output channel. The simulation
computes the same `y` from the rounded values in single precision, so it
agrees with an integer kernel up to the rounding of the sums; step 4
checks that on a real integer product.

## Step 3: agreement with FP32

### How it was measured

```bash
BIN=target/release/tintype                    # Accelerate build of this branch
# A labelled sample of the stores: up to 150 authentic and 150 generated files from each.
python3 -I scripts/int8-agreement.py --make-cohort data/store --per-store 300 --seed 1 > cohort.tsv
# FP32, then each variant, on the same files: every view, and with early exit.
python3 -I scripts/int8-agreement.py --run $BIN --cohort cohort.tsv --device cpu --extra=--all-views --out fp32-all.jsonl
python3 -I scripts/int8-agreement.py --run $BIN --cohort cohort.tsv --device cpu-int8 --extra=--all-views --env TINTYPE_INT8=all --out all-all.jsonl
python3 -I scripts/int8-agreement.py --run $BIN --cohort cohort.tsv --device cpu --out fp32-default.jsonl
python3 -I scripts/int8-agreement.py --run $BIN --cohort cohort.tsv --device cpu-int8 --env TINTYPE_INT8=all --out all-default.jsonl
# ... one run per variant, then:
python3 -I scripts/int8-agreement.py --cohort cohort.tsv --fp32 fp32-all.jsonl --int8 all=all-all.jsonl --int8 weights=weights-all.jsonl --tail 10
```

The cohort is 3,148 files from the 13 stores under `data/store` whose files
are on this machine: 1,350 authentic and 1,798 generated (150 of each from a
store that has both; stores of one class give 150). **Every one of them is
training data for the model being measured.** What follows says how closely
INT8 tracks FP32 on images the model knows; it says nothing about held-out
recall, and the "recall at 1% FPR" rows use a cutoff chosen on these same
files, which makes them exploratory twice over. With 1,350 authentic files a
1% false-positive rate is 13 files and 0.1% is one, so the 0.1% row moves by
whole detections when a single authentic file moves.

The logit is the JSON `logit`: the mean over the views looked at, before
calibration (the shipped threshold is 1.692 on it). Each run is a few
hundred files per process on the Accelerate build; FP32 itself is not
bit-reproducible in that setting, by about 1e-7 on a logit
([view-reuse runbook](view-reuse.md#seen-on-the-way-not-changed)), five
orders of magnitude below anything here. Commit: `08b7a80` (the step-2 commit)
for every column but `all:token:8:7`, and `5a4763e`, which adds weight bits,
for that one. Load average 5 to 13 during the runs (the runs themselves and,
for some, a build beside them; nothing here is a timing).

### Every view of every file (`--all-views`)

The same six views on both sides, so the difference is the arithmetic alone.

| | weights | mlp | all | all:tensor | all:token:7 | all:token:8:7 |
|---|---:|---:|---:|---:|---:|---:|
| Files compared (authentic / generated) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) |
| Scored by one run only | 0 | 0 | 0 | 0 | 0 | 0 |
| Absolute logit difference: median | 0.0478 | 0.0870 | 0.0955 | 0.4675 | 0.2103 | 0.1238 |
| Absolute logit difference: 99th percentile | 0.3698 | 0.5166 | 0.5972 | 2.5637 | 1.6531 | 0.7023 |
| Absolute logit difference: largest | 0.7996 | 0.9925 | 1.5308 | 6.1781 | 5.0189 | 1.7952 |
| Mean signed difference (INT8 minus FP32) | +0.0389 | +0.0786 | +0.0859 | +0.1236 | +0.2256 | -0.0207 |
| Shipped threshold, recall: FP32 → INT8 | 91.43% → 91.38% | 91.43% → 91.32% | 91.43% → 91.38% | 91.43% → 91.16% | 91.43% → 91.66% | 91.43% → 91.05% |
| Shipped threshold, false-positive rate: FP32 → INT8 | 0.15% → 0.07% | 0.15% → 0.07% | 0.15% → 0.15% | 0.15% → 0.22% | 0.15% → 0.15% | 0.15% → 0.15% |
| Generated: detected by FP32, missed by INT8 | 1 | 4 | 2 | 13 | 3 | 9 |
| Generated: missed by FP32, detected by INT8 | 0 | 2 | 1 | 8 | 7 | 2 |
| Authentic: newly flagged by INT8 | 0 | 0 | 0 | 2 | 1 | 0 |
| Authentic: flagged by FP32 only | 1 | 1 | 0 | 1 | 1 | 0 |
| Files stopping at a different stage | 0 | 0 | 0 | 0 | 0 | 0 |
| Views per file: FP32 → INT8 | 6.000 → 6.000 | 6.000 → 6.000 | 6.000 → 6.000 | 6.000 → 6.000 | 6.000 → 6.000 | 6.000 → 6.000 |
| Recall at 1% empirical FPR (own cutoff, this cohort): FP32 → INT8 | 97.50% → 97.22% (-0.28 points) | 97.50% → 97.16% (-0.33 points) | 97.50% → 97.27% (-0.22 points) | 97.50% → 95.55% (-1.95 points) | 97.50% → 96.22% (-1.28 points) | 97.50% → 97.44% (-0.06 points) |
| Cutoff logit at 1% FPR: FP32 → INT8 | -0.981 → -0.727 | -0.981 → -0.675 | -0.981 → -0.749 | -0.981 → 0.133 | -0.981 → -0.189 | -0.981 → -0.902 |
| Recall at 0.1% empirical FPR (own cutoff, this cohort): FP32 → INT8 | 91.27% → 91.94% (+0.67 points) | 91.27% → 91.77% (+0.50 points) | 91.27% → 91.21% (-0.06 points) | 91.27% → 86.26% (-5.01 points) | 91.27% → 90.88% (-0.39 points) | 91.27% → 90.21% (-1.06 points) |
| Cutoff logit at 0.1% FPR: FP32 → INT8 | 1.744 → 1.338 | 1.744 → 1.496 | 1.744 → 1.821 | 1.744 → 2.835 | 1.744 → 1.997 | 1.744 → 1.945 |

### With early exit (the default profile)

Here a file may stop at a different stage, and then the two logits are means
over different views.

| | weights | mlp | all | all:token:8:7 |
|---|---:|---:|---:|---:|
| Files compared (authentic / generated) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) |
| Scored by one run only | 0 | 0 | 0 | 0 |
| Absolute logit difference: median | 0.0619 | 0.1347 | 0.1425 | 0.1778 |
| Absolute logit difference: 99th percentile | 0.8028 | 1.4733 | 1.4503 | 1.5206 |
| Absolute logit difference: largest | 3.2141 | 3.3170 | 7.9300 | 9.7254 |
| Mean signed difference (INT8 minus FP32) | +0.0343 | +0.0694 | +0.0686 | -0.0380 |
| Shipped threshold, recall: FP32 → INT8 | 91.43% → 91.55% | 91.43% → 91.38% | 91.43% → 91.55% | 91.43% → 90.93% |
| Shipped threshold, false-positive rate: FP32 → INT8 | 0.15% → 0.07% | 0.15% → 0.07% | 0.15% → 0.15% | 0.15% → 0.15% |
| Generated: detected by FP32, missed by INT8 | 1 | 4 | 2 | 11 |
| Generated: missed by FP32, detected by INT8 | 3 | 3 | 4 | 2 |
| Authentic: newly flagged by INT8 | 0 | 0 | 0 | 0 |
| Authentic: flagged by FP32 only | 1 | 1 | 0 | 0 |
| Files stopping at a different stage | 63 | 133 | 138 | 175 |
| Views per file: FP32 → INT8 | 1.627 → 1.623 | 1.627 → 1.624 | 1.627 → 1.619 | 1.627 → 1.641 |
| Recall at 1% empirical FPR (own cutoff, this cohort): FP32 → INT8 | 96.55% → 96.50% (-0.06 points) | 96.55% → 96.50% (-0.06 points) | 96.55% → 96.66% (+0.11 points) | 96.55% → 96.50% (-0.06 points) |
| Cutoff logit at 1% FPR: FP32 → INT8 | -1.263 → -1.106 | -1.263 → -1.029 | -1.263 → -1.088 | -1.263 → -1.094 |
| Recall at 0.1% empirical FPR (own cutoff, this cohort): FP32 → INT8 | 91.27% → 91.88% (+0.61 points) | 91.27% → 91.71% (+0.44 points) | 91.27% → 91.43% (+0.17 points) | 91.27% → 90.55% (-0.72 points) |
| Cutoff logit at 0.1% FPR: FP32 → INT8 | 1.744 → 1.338 | 1.744 → 1.496 | 1.744 → 1.821 | 1.744 → 1.945 |

Stage by stage for `all`: of 3,148 files, 3,010 stop where FP32 stops and
138 do not (60 later, 78 earlier); FP32 decides 1,953 files on the first
view and INT8 1,955. One file that FP32 decided on its first view went
through all six with INT8.

### Which input is the sensitive one

All four layers' weights rounded, and one layer's input at a time; `inputs`
is all four inputs with the weights left alone. Every view of every file.

| | qkv | proj | fc1 | fc2 | inputs |
|---|---:|---:|---:|---:|---:|
| Files compared (authentic / generated) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) | 3148 (1350 / 1798) |
| Scored by one run only | 0 | 0 | 0 | 0 | 0 |
| Absolute logit difference: median | 0.0503 | 0.0516 | 0.0527 | 0.0867 | 0.0689 |
| Absolute logit difference: 99th percentile | 0.3572 | 0.4160 | 0.3889 | 0.5345 | 0.3416 |
| Absolute logit difference: largest | 0.9463 | 0.8949 | 0.9347 | 1.0392 | 1.3379 |
| Mean signed difference (INT8 minus FP32) | +0.0397 | +0.0447 | +0.0418 | +0.0777 | +0.0455 |
| Shipped threshold, recall: FP32 → INT8 | 91.43% → 91.38% | 91.43% → 91.38% | 91.43% → 91.38% | 91.43% → 91.49% | 91.43% → 91.55% |
| Shipped threshold, false-positive rate: FP32 → INT8 | 0.15% → 0.07% | 0.15% → 0.07% | 0.15% → 0.07% | 0.15% → 0.07% | 0.15% → 0.15% |
| Generated: detected by FP32, missed by INT8 | 1 | 1 | 1 | 1 | 0 |
| Generated: missed by FP32, detected by INT8 | 0 | 0 | 0 | 2 | 2 |
| Authentic: newly flagged by INT8 | 0 | 0 | 0 | 0 | 0 |
| Authentic: flagged by FP32 only | 1 | 1 | 1 | 1 | 0 |
| Files stopping at a different stage | 0 | 0 | 0 | 0 | 0 |
| Views per file: FP32 → INT8 | 6.000 → 6.000 | 6.000 → 6.000 | 6.000 → 6.000 | 6.000 → 6.000 | 6.000 → 6.000 |
| Recall at 1% empirical FPR (own cutoff, this cohort): FP32 → INT8 | 97.50% → 97.05% (-0.44 points) | 97.50% → 97.22% (-0.28 points) | 97.50% → 97.16% (-0.33 points) | 97.50% → 97.16% (-0.33 points) | 97.50% → 97.33% (-0.17 points) |
| Cutoff logit at 1% FPR: FP32 → INT8 | -0.981 → -0.705 | -0.981 → -0.702 | -0.981 → -0.695 | -0.981 → -0.686 | -0.981 → -0.879 |
| Recall at 0.1% empirical FPR (own cutoff, this cohort): FP32 → INT8 | 91.27% → 92.32% (+1.06 points) | 91.27% → 91.71% (+0.44 points) | 91.27% → 91.94% (+0.67 points) | 91.27% → 92.49% (+1.22 points) | 91.27% → 90.66% (-0.61 points) |
| Cutoff logit at 0.1% FPR: FP32 → INT8 | 1.744 → 1.208 | 1.744 → 1.442 | 1.744 → 1.394 | 1.744 → 1.174 | 1.744 → 1.975 |

### The largest authentic scores, side by side

The tail is where a false positive comes from. Every view, `all` against
FP32 (the shipped threshold is 1.692):

| File | FP32 | all |
|---|---:|---:|
| train-00003-of-00032-00000/0003583.jpg | 2.436 (6) | 2.643 (6) |
| test-00001-of-00013/0003478.jpg | 1.744 (6) | 1.821 (6) |
| supplement_reddit-2026-11-00003/0000007.jpg | 0.482 (6) | 0.654 (6) |
| train-00016-of-00032-00000/0000328.jpg | 0.141 (6) | 0.646 (6) |
| supplement_reddit-2026-11-00003/0000370.jpg | -0.092 (6) | -0.132 (6) |
| train-00000-of-00032-00000/0003480.jpg | -0.136 (6) | -0.210 (6) |
| train-00013-of-00032-00000/0000606.jpg | -0.408 (6) | -0.348 (6) |
| train-00028-of-00032-00000/0000706.jpg | -0.430 (6) | -0.412 (6) |
| supplement_reddit-2026-11-00004/0001023.jpg | -0.434 (6) | -0.716 (6) |
| train-00007-of-00032-00000/0000894.jpg | -0.473 (6) | -0.525 (6) |

No single input is the sensitive one: each alone adds little to the weights'
0.048 median, the input of fc2 (the values after GELU) the most (0.087),
which is the direction PTQ4ViT predicts and a small effect here. Inputs
alone (0.069) cost about what weights alone do.

### What this says

1. **Eight bits with a range per token track FP32 closely here.** With all
   four linear layers' weights and inputs rounded (`all`), the median file's
   logit moves by 0.10 and the 99th percentile by 0.60 on a scale where the
   early-exit margins are 1 to 6; 3 verdicts of 3,148 change with every view
   looked at (2 generated files lost, 1 gained, no authentic file newly
   flagged) and 6 with early exit (2 lost, 4 gained, none newly flagged).
   Recall at each run's own 1% FPR cutoff moves by -0.22 points (every
   view) and +0.11 (early exit); at 0.1% by -0.06 and +0.17. On this
   training-data cohort that is inside the gate's one point at both
   operating points.
2. **The range has to be per token.** One range for the whole view
   (`all:tensor`) loses 1.95 points at 1% FPR and 5.0 points at 0.1%, moves
   one logit by 6.2 and newly flags two authentic files: a few tokens with
   large values take the resolution from all the others. This variant fails
   the gate even here.
3. **Seven bits are not clean on either side.** The AVX2 multiply-add
   needs the inputs or the weights at seven bits. Seven-bit inputs
   (`all:token:7`) lose 1.28 points at 1% FPR. Seven-bit weights
   (`all:token:8:7`) lose 0.06 at 1% but 1.06 points at 0.1% with every
   view (0.72 with early exit) and 9 generated files at the shipped
   threshold against 2 gained: at the edge of the bound on a row that
   turns on single files. A plain-AVX2 kernel would need its own answer;
   kernels with full eight-bit products (`SDOT`, VNNI) do not.
4. **Logits drift upwards.** The mean signed difference is positive in every
   variant (+0.04 for weights alone, +0.09 for `all`), and the cutoffs that
   give 1% FPR move with it (-0.98 to -0.75). An INT8 engine could not reuse
   the FP32 calibration unchanged: it would need its own `calibrate` run,
   and its own early-exit check, since 4.4% of files change stage.
5. **Early exit amplifies single-view differences.** With early exit the
   largest logit difference is 7.9 (a file that stops at a different view
   is averaged over different views) against 1.5 with every view, though
   verdicts barely move.

None of this is a result on the suites. What it earns is the run on the
frozen split, under [Ready to run elsewhere](#ready-to-run-elsewhere).

## Step 4: is a real INT8 kernel worth writing?

Both conditions for asking hold: the linear layers are most of the time
where throughput is measured (step 1), and a variant stays inside the
accuracy bound on what could be measured (step 3). So the question is how
much faster an integer product is than the single-precision one it would
replace, at this network's shapes.

### The benchmark

`src/cpu_vit/int8_gemm.rs` is compiled into the test binary only; `tintype`
does not contain it and nothing scores with it. It holds:

- an `i8 x i8 -> i32` product, `c[m, n] = a[m, k] * w[n, k]^T`: on ARM64
  with the dot-product extension, four rows by four with sixteen `SDOT`
  accumulators (`std::arch::aarch64`, stable Rust, no new crate); anywhere
  else a plain loop. **There is no x86-64 kernel in it.**
- the linear layer around it, as step 2 describes it: each token's input
  rounded to a byte with its own step and offset, the integer product, the
  sums scaled back and the bias added;
- `the_integer_product_matches_the_definition` and
  `an_integer_linear_layer_gives_what_the_simulation_gives`: the layer's
  output equals the `cpu-int8` simulation's to 2e-5 of the output's size, so
  what step 3 measured is what such a kernel would produce;
- `int8_against_f32_at_the_networks_shapes` (ignored by default), which
  times the four linear layers of one view's eleven full blocks, 3.83
  billion multiply-adds, with weights of their own per block as in the
  network (77.9 MB in single precision, 19.5 MB in bytes), three ways:
  single precision exactly as the engine runs it, the integer product
  alone, and the whole integer layer.

```bash
cargo test --release --locked --lib int8_against_f32 -- --ignored --nocapture                          # against Accelerate
cargo test --release --locked --lib --features portable-cpu int8_against_f32 -- --ignored --nocapture  # against the pure-Rust kernels
VECLIB_MAXIMUM_THREADS=1 cargo test --release --locked --lib int8_against_f32 -- --ignored --nocapture # Accelerate held to one thread
```

### Results (Apple M6, 12 threads, best of 7 passes; load average 3.5 to 4.9)

Milliseconds for the block linear layers of one view.

| Situation | FP32 product | FP32 | INT8 product alone | INT8 layer (rounding, product, scaling) | FP32 / INT8 layer |
|---|---|---:|---:|---:|---:|
| One view, all threads (single-image path) | Accelerate | 9.69 | 4.59 | 4.89 | **1.98x** |
| 48 views at once, per view (throughput path) | Accelerate | 4.80 | 2.34 | 2.68 | **1.79x** |
| One view, all threads | pure-Rust `gemm` (NEON) | 16.30 | 4.72 | 4.90 | **3.33x** |
| 48 views at once, per view | pure-Rust `gemm` (NEON) | 8.99 | 2.42 | 2.67 | **3.37x** |
| One view on one thread | pure-Rust `gemm` (NEON) | 71.59 | | 18.12 | 3.95x |
| One view on one thread | Accelerate, held to one thread | 9.11 | | 18.49 | 0.49x |

Per layer, one view with all threads, against Accelerate: qkv 2.26 ms
against 1.26 (1.79x), proj 1.10 against 0.48 (2.27x), fc1 3.03 against 1.55
(1.96x), fc2 2.86 against 1.39 (2.06x); against the pure-Rust kernels 3.15x,
4.39x, 3.02x and 3.89x. In multiply-adds per second with every thread busy:
Accelerate 799 billion, the pure-Rust FP32 kernels 426 billion, the integer
layer 1,432 to 1,436 billion.

Two things to read from the single-thread rows. One thread of this chip does
single-precision products through Accelerate twice as fast as it does `SDOT`
(Apple's matrix unit), but that unit does not multiply with the cores:
twelve threads of `SDOT` end up 1.8 to 2 times ahead of Accelerate using
the whole machine. And rounding the input and scaling the output cost 4 to
15% on top of the integer product; the per-token ranges are cheap.

### Against the gate, by Amdahl

The share `f` of the block linear layers from step 1 and the measured `k`
give a bound on the whole: `1 / (1 - f + f / k)`. This is arithmetic on two
measurements, not an end-to-end timing; no integer kernel is wired into the
scoring path.

| Path | FP32 products | `f` | `k` | Bound on the whole | Gate (1.3x, or 25% less p95) |
|---|---|---:|---:|---:|---|
| Throughput, default | Accelerate | 0.704 | 1.79 | **1.45x** | clears 1.3x |
| Throughput, default | pure-Rust (NEON) | 0.765 | 3.37 | **2.16x** | clears 1.3x |
| Throughput, `--all-views` | Accelerate | 0.752 | 1.79 | 1.50x | clears |
| Throughput, `--all-views` | pure-Rust (NEON) | 0.755 | 3.37 | 2.13x | clears |
| Latency, default (p50 and p95 are both single-view passes) | Accelerate | 0.437 | 1.98 | 1.28x (22% less time) | just short of both |
| Latency, default | pure-Rust (NEON) | 0.515 | 3.33 | 1.56x (36% less time) | clears both |

So on ARM64 the speed clause looks reachable by the INT8 route on
throughput with either set of FP32 kernels, and on latency with the
pure-Rust kernels (Linux and Windows on ARM64). On macOS one image at a
time it falls just short, because there the linear layers are only 44% of
the time; the thread hand-offs under
[Seen on the way](#seen-on-the-way-not-changed) are the larger share to go
after first on that path.

What the bound leaves out: the attention products and everything else stay
FP32 and were measured with the FP32 linear layers beside them; a real
engine would also keep the weights four times smaller in the cache, which
the benchmark includes for the linear layers only. And all of it is ARM64:
**nothing here says what x86-64 gains.** There the pure-Rust FP32 kernels
are AVX2 or AVX-512 FMA, and the integer kernel would be `vpmaddubsw` +
`vpmaddwd` (AVX2, with the seven-bit question of step 3) or `vpdpbusd`
(AVX-512 VNNI and AVX-VNNI). How those compare with FMA at these shapes has
to be measured on such a machine; no figure is offered here.

### Design for the real kernel (not built)

If the frozen-split run of step 3 passes, the engine to build is small:

1. **Storage.** At load, quantise each block's four weight matrices per
   output channel to `i8` and keep the step and the step-times-row-sum per
   channel (`LinI8::new` in the benchmark is this). The FP32 weights of
   those layers need not be kept: memory for them falls from 78 MB to
   20 MB.
2. **Per layer, per run.** Round each token's input to a byte with its own
   step and offset (one pass for the extremes, one to round), run the
   integer product, scale the sums back with the bias. Everything else in
   `CpuVit::run` stays as it is.
3. **Kernels.** `SDOT` on ARM64 (run-time detection, as here; ARMv8.2
   cores without it fall back to FP32), `vpdpbusd` where VNNI is present,
   `vpmaddubsw` on plain AVX2 only if the seven-bit variant passes;
   otherwise FP32. One view per thread for batches, rows split over threads
   for a single view, as the benchmark does.
4. **A device of its own**, `cpu-int8`, replacing the simulation behind the
   same name, with its own calibration in the spec (a new optional field,
   so the shipped specs load unchanged) and its own early-exit check: the
   logits drift (step 3), so the FP32 threshold must not be reused.
5. **Parity tests**: the integer engine against the simulation on the
   fixture (they agree to rounding), and the agreement script of step 3 on
   the frozen split before anything is said about accuracy.

## Step 5: fewer blocks, and exits part-way

Token merging is not on the table: it already cost more than six points of
recall here ([benchmark](../BENCHMARK.md#five-speed-changes-before-and-after)).
What is left is a network with fewer blocks, trained for it, or a way out of
the network part-way for views that are already clear. Both are training
experiments; what could be done here is to measure what they would save and
to get the code and commands ready.

### What a shallower network would cost to run (timing only)

A spec may now name a ViT-S/16 with fewer blocks: `"arch":
"vit_small_patch16_d8"` (`VitConfig::by_name`; empty still means the
twelve-block network, and the shipped specs are unchanged). Both engines
build it. For timing, copies of the released weights and spec were given
that name, so that the engine runs their first 6, 7 or 8 blocks:

```bash
mkdir -p runs/ws8-depth-proxy
for n in 6 7 8 12; do
  cp models/tintype.safetensors runs/ws8-depth-proxy/d$n.safetensors
  python3 -c "import json,sys; d=json.load(open('models/tintype.json')); n=int(sys.argv[1]); d.update({'arch': 'vit_small_patch16_d%d' % n} if n != 12 else {}); json.dump(d, open('runs/ws8-depth-proxy/d%d.json' % n, 'w'), indent=2)" $n
done
tintype verify-models
A=target/release/tintype; W=runs/ws8-depth-proxy
python3 -I scripts/view-reuse-bench.py --bin acc-12=$A --bin acc-8=$A --bin acc-7=$A --bin acc-6=$A \
    --weights-of acc-12=$W/d12.safetensors --weights-of acc-8=$W/d8.safetensors \
    --weights-of acc-7=$W/d7.safetensors --weights-of acc-6=$W/d6.safetensors \
    --together --list photos.txt --repeat 3 --device cpu --extra=--all-views
# and the same with --latency --manifest photos.tsv --n 150, and with the portable-cpu binary
```

**These are the first blocks of a network trained to use twelve. Their
scores mean nothing, and no accuracy is claimed or was looked at.** They
are run with `--all-views` because their logits would otherwise decide how
many views each file gets. The twelve-block copy is the control: it is the
released model under another path.

| Kernels | Blocks | Files/s, six views each (median of 3) | against 12 | Network ms per view | Latency p50 (ms) | Latency p95 (ms) | p95 against 12 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Accelerate | 12 | 24.0 | 1.00x | 6.90 | 47.8 | 59.9 | +0% |
| Accelerate | 8 | 36.0 | 1.50x | 4.58 | 32.2 | 44.1 | -26% |
| Accelerate | 7 | 41.6 | 1.73x | 3.99 | 28.3 | 40.2 | -33% |
| Accelerate | 6 | 48.8 | 2.03x | 3.44 | 24.4 | 36.2 | -40% |
| pure Rust | 12 | 13.4 | 1.00x | 12.84 | 107.7 | 118.7 | +0% |
| pure Rust | 8 | 20.0 | 1.49x | 8.48 | 72.0 | 83.3 | -30% |
| pure Rust | 7 | 22.9 | 1.71x | 7.35 | 62.5 | 74.7 | -37% |
| pure Rust | 6 | 27.4 | 2.04x | 6.17 | 53.3 | 65.4 | -45% |

All 308 photos, six views each, three passes, the eight configurations taking
turns; latency is each image's best of three over 150 photos, all six views
in one pass. Load average 5.2 before the throughput run and 13.5 after it,
12.7 before the latency run and 9.5 after (nothing else running; the
latency run followed the throughput run directly). "Network ms per view"
is the scoring thread's time in the network over the views scored.

So against twelve blocks the same engine runs eight in two thirds of the
time: 1.50x the throughput with Accelerate and 1.49x with the pure-Rust
kernels, and a p95 26% and 30% lower. Seven blocks give 1.73x and 1.71x,
six 2.03x and 2.04x. Every depth from eight down clears the gate's speed
clause on both counts, **if a student of that depth keeps the recall**,
which nothing here says.

A trained student's early exit would look at fewer views of most files, as
the released model does, so its file-to-verdict gain depends on its own
logits; the per-view cost above is what is known.

### Could a view stop part-way with the weights as shipped?

`tintype probe-exits` (hidden from `--help`; a diagnostic) takes the view
early exit looks at first (the most textured crop) of each image, runs the
released network cut after 4, 6, 8 and 10 blocks, and fits a linear
read-out from the class token at that depth to the logit the whole network
gives the same view: ridge regression on standardised features, solved
exactly, penalty 10 (not tuned), fitted on one half of the images and
judged on the other, then the halves swapped. Beside it, the shipped final
normalisation and head applied to the same class token with no fitting.

```bash
tintype probe-exits --weights models/tintype.safetensors --manifest cohort.tsv --blocks 4,6,8,10
```

On the 3,148-file cohort of step 3 (training data, so every row flatters
the model alike; the cutoff for "recall at 1% FPR" is this cohort's own):

| Blocks | Read-out | AUROC | Recall at 1% FPR | Recall at 0.1% FPR | RMS distance from the whole network's logit | Views it could stop with at most 1.0% of them on the other side | of the authentic | margin | Blocks per view then | Network time then |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 12 | the network itself | 0.9894 | 83.9% | 70.0% | 0.000 | | | | | |
| 4 | fitted linear read-out of the class token | 0.8769 | 22.6% | 9.0% | 5.404 | 0.1% | 0.1% | 19.40 | 11.24 | 100% |
| 4 | the shipped final normalisation and head, untrained for this depth | 0.6155 | 6.7% | 2.4% | 7.303 | 0.0% | 0.1% | 1.10 | 11.25 | 100% |
| 6 | fitted linear read-out of the class token | 0.9384 | 52.3% | 34.3% | 4.247 | 26.4% | 35.9% | 7.56 | 9.86 | 88% |
| 6 | the shipped final normalisation and head, untrained for this depth | 0.7102 | 15.7% | 8.5% | 7.229 | 0.6% | 0.0% | 1.39 | 11.22 | 100% |
| 8 | fitted linear read-out of the class token | 0.9696 | 72.3% | 27.4% | 3.112 | 60.9% | 77.2% | 4.59 | 9.27 | 82% |
| 8 | the shipped final normalisation and head, untrained for this depth | 0.9224 | 54.7% | 35.0% | 6.486 | 11.4% | 24.9% | 2.94 | 10.88 | 97% |
| 10 | fitted linear read-out of the class token | 0.9825 | 83.2% | 73.3% | 2.018 | 83.6% | 94.2% | 2.33 | 10.20 | 91% |
| 10 | the shipped final normalisation and head, untrained for this depth | 0.9731 | 74.5% | 53.6% | 5.210 | 46.4% | 88.3% | 2.99 | 10.67 | 95% |

3,150 rows of the cohort file (two files are listed twice), 1,350 authentic.

"Views it could stop" is the largest share that a rule "stop when the
read-out is at least `m` from the threshold" lets go while at most 1% of
those it lets go end on the other side of the threshold from the whole
network; `m` is chosen on the same held-out predictions, so the share is an
upper estimate. A view that stops has run that many whole blocks; the whole
network costs 11.25 (only the class token goes through the last block).

What it says: the class token part-way through does carry the answer for
many views (after eight blocks a fitted read-out ranks at AUROC 0.970
against the network's 0.989 on this single view), but not cheaply enough.
The best row lets 61% of first views go after eight blocks and brings the
network to 82% of its time on the first view; after six blocks 26% go
(88%), after ten 84% (91%). That is short of the 23% the speed clause
needs (1.3x), before counting that the margin was chosen on the data it is
judged on, that a stopped view's logit is the read-out's and not the
network's (so calibration and the view-level early exit would have to be
refitted), and that these are training images. The shipped head applied
part-way without fitting is far worse (11% of views after eight blocks).
So exits bolted onto the released weights do not reach the gate; a network
*trained* with exits might, and that is the multi-exit distillation the
report cites, a training experiment like the student.

### Training a student (commands, not run)

`train` gained three options, all off by default (with none of them it
trains the twelve-block network on labels alone, as before, and writes the
same spec):

| Option | What it does |
|---|---|
| `--blocks N` | train a ViT-S/16 with `N` blocks; it starts from the first `N` blocks of `--init` and keeps `--init`'s embeddings, final normalisation and head. The saved spec names the depth (`vit_small_patch16_dN`), so `detect`, `score` and `latency` load it on either engine |
| `--init-blocks 0,1,2,3,5,7,9,11` | start from these blocks of `--init` instead of the first ones (`distill::take_blocks`) |
| `--distill TEACHER.safetensors --distill-weight W --distill-temperature T` | a frozen teacher sees every batch in the same graph; the target of the cross-entropy becomes `(1 - W) * label + W * sigmoid(teacher logit / T)` (`distill::soft_target`), which is `1 - W` of the loss against the label plus `W` of the loss against the teacher's softened probability |

Tested here without a training run: the block selection, the target and the
loss as pure functions (unit tests, every platform); the graph's loss with a
teacher against those functions on a two-block network, the student's own
logits untouched by the teacher, only the student exported, and the loss
falling over twenty steps (`trainer::tests`, needs the Apple GPU, run by
hand: passed); a spec naming eight blocks loading on the CPU engine
(`tests/inference.rs`) and, by hand, the same logit to five decimals on the
GPU engine (0.40838 and 0.40837 on the demo image). **`tintype train` itself
was not started with these options**, so the command line path through
`train_cmd` is compiled but not exercised.

**Before running.** The report's condition is "distil only from a teacher
that improves the target slices". The released model is not known to be
that teacher: it is the incumbent. The teacher should be the candidate that
[the frozen evaluation](frozen-eval.md) selects from the other workstreams
(matched processing chains, source matching); if none is selected yet, a run
against the released model is a rehearsal of the pipeline and says how much
of the incumbent a student keeps, not whether to ship one. Either way the
teacher is copied under `runs/` first; nothing below writes to `models/`.

```bash
# From the training tree (the directory holding data/ and runs/).
mkdir -p runs/ws8-teacher
cp <teacher>.safetensors runs/ws8-teacher/teacher.safetensors    # e.g. models/tintype.safetensors for a rehearsal
cp <teacher>.json        runs/ws8-teacher/teacher.json
T=runs/ws8-teacher/teacher.safetensors
RECIPE="--steps 8000 --batch 32 --lr 2e-5 --warmup 300 --seed 1"   # the phase-6 recipe
M=$(cat runs/v7/manifests.txt)                                      # the phase-6 manifests, as --manifest flags

# 1. The main runs: eight, seven and six blocks, spread over the teacher's depth, with the teacher.
tintype train --init $T --init-blocks 0,1,2,3,5,7,9,11 --distill $T --distill-weight 0.5 --distill-temperature 2 $M --out runs/ws8-student-8 $RECIPE
tintype train --init $T --init-blocks 0,1,2,4,6,8,11   --distill $T --distill-weight 0.5 --distill-temperature 2 $M --out runs/ws8-student-7 $RECIPE
tintype train --init $T --init-blocks 0,2,4,6,8,11     --distill $T --distill-weight 0.5 --distill-temperature 2 $M --out runs/ws8-student-6 $RECIPE

# 2. Controls at eight blocks: labels alone (what the teacher adds), and the first eight blocks (what the choice of blocks adds).
tintype train --init $T --init-blocks 0,1,2,3,5,7,9,11 $M --out runs/ws8-student-8-labels $RECIPE
tintype train --init $T --blocks 8 --distill $T --distill-weight 0.5 --distill-temperature 2 $M --out runs/ws8-student-8-first $RECIPE

# 3. The reference the students are compared with at equal steps: the phase-6 run itself,
#    from the phase-3 checkpoint, twelve blocks, no teacher (this reproduces the released weights' recipe).
tintype train --init runs/v4/step_007000.safetensors $M --out runs/ws8-reference-12 $RECIPE

tintype verify-models        # the released files are untouched
```

Each run writes `step_00N000.safetensors`, `last.safetensors` and their
specs under its `--out`, and `train.log` with an `eval @` line every 1,000
steps (validation AUC, accuracy, precision and recall for the whole-image
view, the native crop and their mean). Expected on this Mac's GPU: the
twelve-block recipe's duration scaled by depth, plus the teacher's forward
pass. Things to check before any evaluation: the `student:` and `teacher`
lines at the top of `train.log` name the blocks and the teacher meant; the
first `eval @` is not at chance (a student cut from twelve blocks starts far
from its teacher: the cut network's own read-out ranks poorly, see the
table above); and the loss is still falling at 8,000 steps. If it is, the
schedule is too short for a student and the first thing to vary is its
length, on the fit set, not on the test set. The block lists, the weight
0.5 and the temperature 2 are starting values, not tuned ones.

**Then**, for each student, the candidate procedure of
[the frozen evaluation runbook](frozen-eval.md#what-is-measured) with
`W=runs/ws8-student-N/last.safetensors`: score `calibrate`, `calibrate
--weights $W`, score `test`, `report --operating-points` against the
teacher and the incumbent, and `latency` on the GPU and the CPU. The gate
for a student is the one at the top of this page: 1.3x CPU throughput or
25% lower p95 than the twelve-block model **on the same files with each
model's own early exit**, with at most one point of recall lost at 1% and
at 0.1% FPR on each clean and each degraded slice. The timing table above
says the speed half is within reach of eight blocks and below; the
accuracy half is unknown until a student exists.

## Against the gate

| Route | Speed clause (1.3x throughput, or 25% lower p95) | Accuracy clause (at most 1 point of recall at equal FPR, each clean and degraded suite) |
|---|---|---|
| INT8 linear layers, eight bits, a range per token (`all`) | **Bounded, not measured end to end, ARM64 only.** Kernel benchmark times profile share: 1.45x throughput against Accelerate and 2.16x against the pure-Rust NEON kernels; latency 22% lower against Accelerate (just short) and 36% lower against pure Rust. x86-64 unknown: no kernel, no machine | **Not decided.** On 3,148 training-store images: -0.22 and -0.06 points at 1% and 0.1% FPR with every view, +0.11 and +0.17 with early exit, 3 and 6 verdicts changed, no authentic file newly flagged. The suites' files are not here |
| INT8 with one range per view, or seven-bit inputs | as above | **Fails even on the store images**: -1.95 / -5.01 points and -1.28 / -0.39 |
| INT8 with seven-bit weights (plain AVX2) | not measured | at the edge on the store images: -0.06 / -1.06 points |
| A student with 8, 7 or 6 blocks | **Timed with untrained stand-ins**: 1.50x, 1.73x, 2.03x throughput with Accelerate (1.49x, 1.71x, 2.04x pure Rust), p95 26% to 45% lower, six views per file | **Unknown**: no student exists. Code and commands are ready; needs the GPU, the training stores and a teacher chosen on the frozen split |
| Exits part-way with the released weights | **Short**: at most 18% off the network on the first view (61% of views could stop after eight blocks), an upper estimate | would need refitting calibration and early exit; not pursued |
| Token merging | not tried again | already lost more than six points |

Read together: the linear layers are where CPU time goes (step 1), eight
bits with a range per token keep the scores on everything that could be
measured (step 3), and an `SDOT` kernel is 1.8 to 3.4 times faster than the
FP32 products it would replace on this chip (step 4), so **INT8 looks able
to reach the speed clause on ARM64 by throughput without retraining**, and
the next step for it is the frozen-split run, then an x86-64 kernel. **A
student of eight blocks or fewer reaches the speed clause by construction**
and by a wider margin, on every platform at once, but its accuracy is an
open training question. The two compound: a quantised student is both.
Neither is a decision to ship anything.

## Ready to run elsewhere

### The profile on x86-64 and on Linux (not run here)

Nothing in step 1 is specific to this machine; the numbers are. On Linux and
Windows the default build is the pure-Rust kernels (AVX2 or AVX-512 on
x86-64, NEON on ARM64), so there is one binary to time. On an x86-64 Linux
machine and on an ARM64 Linux machine (the documented 35.2 photos/s was an
aarch64 Ubuntu virtual machine with 8 cores), with nothing else running:

```bash
git checkout research/ws8-cpu-int8
cargo build --release --locked --bin tintype
# Any few hundred ordinary photos; the 308 used here are 22 from each store.
ls /path/to/photos/*.jpg | head -300 > photos.txt
awk '{print $0 "\t0\tcohort\tphotos"}' photos.txt > photos.tsv
uname -srm; nproc; grep -m1 'model name' /proc/cpuinfo; cat /proc/loadavg

python3 -I scripts/view-reuse-bench.py --bin linux=target/release/tintype \
    --together --list photos.txt --repeat 5 --device cpu > together.jsonl
python3 -I scripts/view-reuse-bench.py --bin linux=target/release/tintype \
    --latency --manifest photos.tsv --n 295 --repeat 5 --device cpu > latency.jsonl
python3 -I scripts/view-reuse-bench.py --bin linux=target/release/tintype \
    --together --list photos.txt --repeat 3 --device cpu --extra=--all-views > together-all.jsonl
python3 -I scripts/cpu-profile.py together.jsonl latency.jsonl together-all.jsonl
```

Expected output: the tables of step 1 for that machine. What to read off:
the share of the four linear layers in the throughput table (`f` above; the
decision to go on with INT8 or a student needs it to stay near three
quarters), whether the scoring thread waits for files there (the "outside
the network" row; on a machine with few cores decoding may hold the network
up, which this Mac does not show), and the attention products' share, which
is the one row where the pure-Rust kernels are weak here. On macOS on Intel
the same commands time Accelerate's x86-64 build; add `--features
portable-cpu` to the build for the AVX2 kernels.

### INT8 against FP32 on the frozen split (not run: the files are not here)

This is the run that decides the accuracy clause. It needs the frozen split
of [the frozen evaluation runbook](frozen-eval.md) on disk, and macOS
(`score` and `calibrate` are built there only), but no GPU: both engines
are the CPU's. About 13 files a second with every view on this Mac.

```bash
# Two copies of the released model under runs/: each is calibrated on its own scores.
mkdir -p runs/ws8-int8
for e in fp32 int8; do cp models/tintype.safetensors runs/ws8-int8/$e.safetensors; cp models/tintype.json runs/ws8-int8/$e.json; done
export TINTYPE_INT8=all            # read by --device cpu-int8 only

tintype score --weights runs/ws8-int8/fp32.safetensors --device cpu      --manifest data/frozen-eval/calibrate.tsv --out runs/ws8-int8/calibrate-fp32.tsv --all-views
tintype score --weights runs/ws8-int8/int8.safetensors --device cpu-int8 --manifest data/frozen-eval/calibrate.tsv --out runs/ws8-int8/calibrate-int8.tsv --all-views
tintype calibrate --weights runs/ws8-int8/fp32.safetensors --scores runs/ws8-int8/calibrate-fp32.tsv --profile photo --target-fpr 0.01
tintype calibrate --weights runs/ws8-int8/int8.safetensors --scores runs/ws8-int8/calibrate-int8.tsv --profile photo --target-fpr 0.01
tintype verify-models                                   # the released files are untouched

tintype score --weights runs/ws8-int8/fp32.safetensors --device cpu      --manifest data/frozen-eval/test.tsv --out runs/ws8-int8/test-fp32.tsv
tintype score --weights runs/ws8-int8/int8.safetensors --device cpu-int8 --manifest data/frozen-eval/test.tsv --out runs/ws8-int8/test-int8.tsv
tintype report --operating-points \
  --system "fp32=runs/ws8-int8/test-fp32.tsv@0.5" --system "int8=runs/ws8-int8/test-int8.tsv@0.5" \
  --fpr 0.001,0.01 --prevalence 0.01 --slice-by family,source,transform --cluster-by parent \
  --groups data/frozen-eval/test.tsv --min-slice 100
```

Expected output: the operating-point tables for the two systems side by
side. The gate: INT8's recall no more than one point under FP32's at 1%
and at 0.1% FPR, pooled and on every clean and every degraded (transform)
slice of at least 100 items, at the cutoffs frozen on `calibrate` and at
the empirical cutoffs on `test`; and its false-positive rate at its own
frozen cutoff inside the same Clopper–Pearson bound. The test scores above
go through early exit, as `detect` does; add `--all-views` to both test
runs for the every-view comparison. The archived suites, where their files
are restored, can be compared the same way or with
`scripts/int8-agreement.py` on a cohort made from their manifests; they
are diagnostics, not the decision.

### An x86-64 integer kernel (not written)

`src/cpu_vit/int8_gemm.rs` has a scalar loop where an x86-64 kernel would
go. On such a machine: add the kernel beside `gemm_i8_dotprod` behind
`cfg(target_arch = "x86_64")` and run-time detection (`avx512vnni` or
`avxvnni`: `vpdpbusd`; plain `avx2`: `vpmaddubsw` + `vpmaddwd`, which needs
seven-bit weights or inputs, or sixteen-bit products at half the density),
have `the_integer_product_matches_the_definition` hold it to the scalar
loop, and run

```bash
cargo test --release --locked --lib int8_against_f32 -- --ignored --nocapture
```

for the table of step 4 against the AVX2 or AVX-512 FP32 kernels. Put `k`
from its "views at once" row and `f` from that machine's throughput profile
into `1 / (1 - f + f / k)`: at or above 1.3 clears the speed clause there.

### A student

The training commands, their controls and what to check are under
[Training a student](#training-a-student-commands-not-run). They need the
Apple GPU and the training stores; nothing was started here.

## Seen on the way, not changed

- **Single-view passes pay for thread hand-offs.** One image at a time with
  Accelerate, the residual additions take 11% of file-to-verdict time
  (3.8 ms per file), softmax 9%, GELU 5.5% and layer normalisation 4.9%:
  each is a few tens of thousands of floats handed to the thread pool per
  call (`par_iter` over 75,648 values for a residual addition). In a batch
  the same additions cost a fifteenth as much per view. A check without
  changing any code: the same binary with the thread pool held to one thread
  (`RAYON_NUM_THREADS=1`; Accelerate keeps its own threads), taking turns
  with the default over the 300 photos, five passes, load average 5.3 before
  and 4.3 after:

  | Per file (ms) | Default | Pool of one thread |
  |---|---:|---:|
  | p50 / p95 / mean, each image's best of 5 | 31.0 / 83.9 / 34.8 | 30.2 / 79.7 / 33.7 |
  | Residual additions | 3.82 | 0.37 |
  | The four linear layers (their bias is copied in per row by the pool) | 15.25 | 12.38 |
  | GELU | 1.90 | 4.07 |
  | Layer normalisation | 1.71 | 4.76 |

  So the hand-offs cost 3.4 ms a file in the residual additions and 2.9 ms
  in the bias copies, about 18% of file-to-verdict time between them, while
  GELU and layer normalisation do gain from the pool (one thread costs them
  5.2 ms more). Holding the whole pool to one thread nets 5% off p95; doing
  only the additions and the bias copies on the calling thread when a pass
  holds one view should keep both sides of that, and would not change a bit
  of the output (they are element-wise). It is not part of this
  workstream's brief and is not done here; it is the first thing to do for
  single-image latency on macOS, ahead of INT8.
- **Early exit one view at a time is slower than a batch for the photos that
  need every view** (p95 84.6 ms against 65.2 ms with `--all-views`, with
  Accelerate). A schedule that puts the remaining views in one pass once
  the first two have not decided would trade a little median for the tail;
  it changes which views are looked at together, not which, so scores would
  move only by the engine's run-size rounding.
