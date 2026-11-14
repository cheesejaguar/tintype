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

## Seen on the way, not changed

- **Single-view passes pay for thread hand-offs.** One image at a time with
  Accelerate, the residual additions take 11% of file-to-verdict time
  (3.8 ms per file), softmax 9%, GELU 5.5% and layer normalisation 4.9%:
  each is a few tens of thousands of floats handed to the thread pool per
  call (`par_iter` over 75,648 values for a residual addition). In a batch
  the same additions cost a fifteenth as much per view. Doing these steps on
  the calling thread when a pass holds one view would not change a bit of
  the output (they are element-wise) and looks like the cheapest latency
  gain on the table: up to about a tenth of the median and p95, to be
  measured. It is not part of this workstream's brief and is not done here.
- **Early exit one view at a time is slower than a batch for the photos that
  need every view** (p95 84.6 ms against 65.2 ms with `--all-views`, with
  Accelerate). A schedule that puts the remaining views in one pass once
  the first two have not decided would trade a little median for the tail;
  it changes which views are looked at together, not which, so scores would
  move only by the engine's run-size rounding.
