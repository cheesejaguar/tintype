# Runbook: scoring identical views once and preparing only what is looked at

This is item 6 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
("Reuse identical view scores and avoid unnecessary crop and decode work").
It changes how much work a verdict takes, not the verdict: nothing here
touches the models, the views, the stopping rule or a threshold. Each change
is one commit with its own numbers below, so that they can be reviewed and
merged separately.

| Change | State | Where |
|---|---|---|
| Work counters (`TINTYPE_STATS`), timing and parity scripts | built | `src/stats.rs`, `scripts/view-reuse-bench.py`, `scripts/view-reuse-parity.py` |
| 1. Score identical views of an image once | built, on by default where exact | `image_io::Pixels`, `detector::ViewReuse` |

## The gate

Written before the measurements; the report's wording is "lower end-to-end
latency with equivalent decisions, reported per cohort". It is a proposed
decision criterion for merging each change, not a claim about detection
quality. Nothing is tuned here, so no cohort is held out; the archived
photos and clips are used only as realistic inputs.

**Equivalent decisions.** With no new flags, a change must leave every score
as it was:

1. One process per file (`detect --json`, where the batches a file's views
   are scored in do not depend on timing): the output is the same bytes as
   the base checkout's, on every engine, for every photo profile (default,
   `--all-views`, `--fast`, `--strict`, `--sensitive`) and for clips with
   and without `--all-views`, on ordinary photos, on images below the crop
   size, on images with few window positions, and on clips. The test
   fixtures and the bundled demo image are part of this.
2. All files in one process: the same bytes wherever the base binary gives
   the same bytes twice (it does on the GPU engines). Where it does not
   (Accelerate, see below), no more rows may differ from base than base
   differs from itself, by no more, and no verdict and no number of views
   or frames looked at may change anywhere.

**Lower latency.** Per cohort (ordinary photos, small images, images with
few window positions, clips), a change is kept if it lowers file-to-verdict
p50 or p95 or raises throughput on at least one cohort and engine, with the
work counters showing where the saving comes from, and does not make any
cohort slower by more than the two identical-work controls differ (the base
binary against the instrumented one, and the cohorts a change cannot
affect). A change that only moves numbers inside that noise is reported as
a wash; a change that is slower is reported as a loss and not kept.

## Hardware, build and load

- Apple M6, 12 cores, 32 GB, macOS 27.2 (build 26B123). `rustc` from the
  stable toolchain, `cargo build --release --locked --bin tintype`.
- Base: `b88c122` (`research/step0-frozen-eval`). The commit each binary was
  built from is given with its numbers.
- **The machine was shared.** Other jobs were running throughout. The
  numbers reported were taken at load averages between 2 and 11 on the 12
  cores (the value at the start of each block is given with it); earlier
  runs at load averages of 20 to 65 varied by a factor of two to four from
  pass to pass and were thrown away. Every timing is taken with the
  binaries taking turns on each file and is reported as each file's best of
  several passes. Counts of work (views scored, frames converted and so on)
  do not depend on load and are exact. The timings are good for the
  direction and rough size of an effect, not for its third digit; they
  should be repeated on a quiet machine before anyone quotes them.

## Cohorts

All lists are plain text files, one path per line, kept outside the
repository (the data tree is not part of it).

| Cohort | Files | What |
|---|---|---|
| Ordinary photos | 308 | 22 files from each of the 14 photo stores under `data/store` that have a manifest, picked with a fixed seed. 0.01 to 45 megapixels, median 1.05; 153 are 1024 x 1024 and 3 have a shorter side below 224 |
| Small images | 100 | 100 of those photos resampled with `sips` to 160 x 120, 200 x 150, 128 x 128, 96 x 144, 200 x 200 and 320 x 223: shorter side below the 224-pixel crop |
| Few window positions | 100 | the same 100 photos resampled to 256 x 256, 224 x 224, 224 x 300, 448 x 224, 230 x 230 and 320 x 256 |
| Synthetic edge cases | 42 | `scripts/view-reuse-parity.py --make-edge-cases DIR`: two textured pictures at each of 21 sizes from 64 x 64 to 900 x 700 (parity only) |
| Clips | 60 | 3 clips from each of the 21 video stores under `data/vstore`, fixed seed, less 3 `.avi` files that the decoders here cannot read. 35 are decided after 2 frames, 13 after 4, 12 go through all 16 |

The brief named `data/fresh8` for the photos and `data/fresh5` for the
clips. Only their manifests are on this machine (the media are not), so the
stores above stand in.

## What is counted

`TINTYPE_STATS=1` makes `detect`, `batch` and `latency` print one line on
standard error when they finish; standard output is unchanged.

| Counter | Meaning |
|---|---|
| `frames_decoded` | video frames a decoder handed back, sampled or not |
| `frames_converted` | decoded frames converted from YCbCr to RGB |
| `views_materialised` | views cropped or resized out of an image or frame |
| `views_scored` | views that went through the network (padding not counted) |
| `views_reused` | views that took the score of an identical view of the same image |
| `clips_reopened` | clips opened a second time for their second half |
| `decode_ms`, `convert_ms`, `views_ms`, `model_ms` | time decoding files and frames, converting frames, ranking windows and preparing views, and in the network, added up over threads |

`views_scored + views_reused` is the number of views looked at, which the
JSON output reports as `views` for a photo. With `TINTYPE_STATS` set,
`latency` also prints each image's own time (`latency_ms <index> <ms>`), so
that two runs can be compared image by image.

## How parity is checked

`BASE` is a `tintype` built from the base commit and `NEW` the one under
test; both are run from the repository root so that they load the same
`models/`. `photos.txt` and `clips.txt` are the cohort lists.

```bash
# Synthetic edge cases (sizes where views coincide), to add to the photo list.
python3 -I scripts/view-reuse-parity.py --make-edge-cases /tmp/edge >> photos.txt

# 1. One process per file: must be byte-identical on every engine.
python3 -I scripts/view-reuse-parity.py --base BASE --new NEW --list photos.txt \
    --each --require-identical --devices cpu --modes default,all-views,fast,strict,sensitive
python3 -I scripts/view-reuse-parity.py --base BASE --new NEW --list clips.txt \
    --each --require-identical --devices cpu --modes default,all-views

# 2. All files in one process; the base binary is run twice as well.
python3 -I scripts/view-reuse-parity.py --base BASE --new NEW --list photos.txt \
    --together --devices cpu,gpu,gpu-f32 --modes default,all-views,fast,strict,sensitive
python3 -I scripts/view-reuse-parity.py --base BASE --new NEW --list clips.txt \
    --together --devices cpu,gpu,gpu-f32 --modes default,all-views

# 3. Short GPU check, one process per file (each process compiles the graph: about 0.17 s per file).
python3 -I scripts/view-reuse-parity.py --base BASE --new NEW --list small-photos.txt \
    --each --require-identical --devices gpu,gpu-f32 --modes default,all-views

# 4. The bundled demo image.
for d in cpu gpu gpu-f32; do diff <(BASE demo --device $d --json) <(NEW demo --device $d --json); done
```

Each line of output is one device and profile: rows compared, rows whose
JSON differs, the largest logit and score difference among them, and how
many verdicts and how many numbers of views or frames looked at changed.
With `--together` the same figures are given for the base binary against a
second run of itself. `--new-env NAME=VALUE` sets a variable for the binary
under test only.

## How time is measured

```bash
# In-process single-image latency (photos): file to verdict, one image at a time.
python3 -I scripts/view-reuse-bench.py --bin before=OLD --bin after=NEW \
    --latency --manifest photos.tsv --n 300 --repeat 5 --device cpu

# File to verdict per process, photos or clips (includes start-up and loading the weights).
python3 -I scripts/view-reuse-bench.py --bin before=OLD --bin after=NEW \
    --list clips.txt --each --repeat 2 --device cpu

# Throughput and peak memory: all files in one process.
python3 -I scripts/view-reuse-bench.py --bin before=OLD --bin after=NEW \
    --list photos.txt --together --repeat 3 --device gpu
```

The binaries take turns (per file with `--each`, per pass otherwise). For
`--latency` and `--each` the tables give p50, p95 and mean over each file's
best pass; for `--together`, the best pass. Peak memory is the largest
resident set size of any process of that binary (`wait4`), which is what
`/usr/bin/time -l` reports. The counters are for one pass.

## Change 1: score identical views of an image once

### Where identical views come from

The six photo views are the whole image (shorter side resized to 256, centre
224 cut out), the centre 224-pixel crop at native resolution and the four
most textured 224-pixel windows. On many images some of these are the same
pixels:

| Image | Views that coincide | Distinct views of six |
|---|---|---|
| shorter side below 224 | the centre crop and all four windows are one upscaled centre crop | 2 |
| exactly 224 x 224 | one window position, which is the centre crop too | 2 |
| 224 x 300 (two window positions) | ranks 3 and 4 repeat the richest window | 4 |
| 448 x 224 (three positions, the middle one central) | one rank is the centre crop, rank 4 repeats the richest | 4 |
| shorter side exactly 256 | the whole-image view is not resized, so it is the centre crop | 5 |
| 230 x 230 and anything larger off these sizes | none | 6 |

Every frame of a clip gets the same six views, so the same holds per frame.

A view is resolved to what it does on the image at hand
(`image_io::Pixels`: a window at a position, or a resize to a given shorter
side followed by the centre crop), and views that resolve to the same thing
share one score. The key is the transform, not a hash of the pixels:
equal keys give equal pixels because the pixels are computed from the key
and the image alone. `resolved_views_give_the_pixels_the_views_gave_and_equal_ones_give_equal_pixels`
checks both halves of that against the old per-view code, byte for byte,
over 22 image sizes and 14 views.

The score is put back in every place the view has: the list of logits has
one entry per view looked at, the mean is over all of them, the early-exit
checks see the same running mean at the same checkpoints and an image stops
where it would have stopped. Only the repeated network runs go, and the
repeated crops.

### When it is on

Whether reuse is exact depends on whether an engine gives a view the same
logit whatever it is run with. Measured here:

| Engine | Does a view's logit depend on what is run with it? | Reuse with no flags |
|---|---|---|
| `gpu`, `gpu-f32` | No. The graph has a fixed batch and short runs are padded; the same view gives the same bits in any slot, beside any neighbours, in batches of 6 and of 64 (`the_gpu_engine_gives_a_view_the_same_logit_in_every_slot_and_with_reuse`, run by hand), and the base binary gives the same bytes three times over 308 files | on everywhere |
| `cpu`, pure-Rust kernels (Linux, Windows, `--features portable-cpu`) | No: a batch is run one view at a time. Checked on this machine's NEON kernels; `the_cpu_engine_depends_on_the_run_size_no_more_than_it_says` holds every platform's test run to it | on everywhere |
| `cpu`, Accelerate (macOS) | Yes. Every layer is one matrix product over all rows of the run, and the library rounds differently for different row counts: the same view alone and in a run of six differs by up to 2.4e-6 on the logit. The same run always gives the same bits | on only where every run is a single view anyway: one photo through early exit (`detect photo.jpg`, `latency`, a one-image API request), or a batch size of 1 |

With Accelerate the base binary does not reproduce itself when many files
go through one process, because which views share a run depends on the
order worker threads finish in (rows below). Reuse would add nothing worse
than that, but it could not be shown identical, so it stays off there unless
asked for.

`TINTYPE_VIEW_REUSE=off` turns reuse off everywhere, `=on` forces it on
everywhere (with Accelerate that moves logits by about 1e-6 where the
default would not reuse), anything else is the default. `Detector::reuse`
is the same switch for library callers.

### What it saves

"Before" is the counters commit, which scores exactly as the base `b88c122`
does; "after" is this change. Times in milliseconds. Load average at the
start of the blocks: 7.1 (latency), 7.7 (per process), 4.2 and 4.7
(throughput), 4.5 and 2.1 (clips).

In-process latency, one image at a time, CPU engine (Accelerate), file to
verdict (`view-reuse-bench.py --latency --device cpu`, each image's best of
6 passes; 5 for the ordinary photos):

| Cohort | Images | p50 before → after | p95 before → after | mean before → after | Views scored | Views cut out |
|---|---|---|---|---|---|---|
| Small images | 95 | 17.8 → 16.8 | 98.9 → 33.1 | 32.2 → 23.5 | 192 → 142 | 600 → 200 |
| Few window positions | 95 | 16.9 → 16.9 | 82.1 → 70.4 | 30.4 → 28.1 | 180 → 167 | 600 → 431 |
| Ordinary photos | 300 | 31.8 → 31.6 | 86.6 → 84.7 | 35.5 → 35.4 | 546 → 544 | 1830 → 1802 |

File to verdict per process (`--each`, one `detect` process per file, each
file's best of 3 passes; 2 for the ordinary photos and the clips). Going by
the medians of the two tables, about 20 ms of each CPU photo figure is
starting the process and loading the weights; on the GPU engine a process
also compiles the graph, which is why a small image takes 173 ms there:

| Engine | Cohort | Files | p50 before → after | p95 before → after | mean before → after |
|---|---|---|---|---|---|
| `cpu` | Small images | 100 | 40.3 → 39.2 | 119.9 → 55.0 | 54.5 → 45.6 |
| `cpu` | Few window positions | 100 | 39.4 → 39.4 | 103.2 → 87.6 | 51.8 → 49.7 |
| `cpu` | Ordinary photos | 308 | 49.6 → 49.7 | 108.1 → 106.6 | 57.9 → 57.8 |
| `cpu` | Clips (reuse off: runs of 12 views) | 60 | 263.3 → 262.4 | 839.3 → 837.0 | 374.9 → 374.4 |
| `gpu` | Clips | 60 | 375.3 → 372.2 | 632.9 → 622.1 | 422.8 → 414.2 |
| `gpu` | Small images | 100 | 173.1 → 172.0 | 174.0 → 173.1 | 173.1 → 172.0 |
| `cpu` | Control: small images, `b88c122` → counters commit | 100 | 40.2 → 40.3 | 119.9 → 120.3 | 54.4 → 54.5 |

The median image is decided after its first view, so there is nothing to
reuse and the median does not move; the saving is in the images that go on,
which is where p95 sits. A small image that needs all six views takes two
network runs instead of six. On the GPU a single small image gains nothing:
its six views go in one padded batch either way (600 views scored become
200, in the same number of runs). Of the clips' 1884 views looked at on the
GPU, 106 repeat another view of the same frame.

Throughput, all files in one process (`--together`, best of 3 passes; 2 for
clips on the CPU), and the largest resident set:

| Engine | Cohort | Files/s before → after | Peak memory before → after (MB) | Views scored |
|---|---|---|---|---|
| `gpu` | Small images | 187.1 → 304.0 | 603 → 461 | 192 → 142 |
| `gpu` | Few window positions | 192.4 → 210.9 | 602 → 522 | 181 → 168 |
| `gpu` | Ordinary photos | 330.2 → 337.6 | 1646 → 1589 | 550 → 548 |
| `gpu` | Clips | 21.0 → 21.2 | 1025 → 1076 | 1884 → 1778 |
| `cpu` (Accelerate: reuse off with several files) | Small images | 74.4 → 74.3 | 619 → 618 | 192 → 192 |
| `cpu` | Few window positions | 79.5 → 79.5 | 611 → 599 | 180 → 180 |
| `cpu` | Ordinary photos | 74.4 → 72.0 | 1418 → 1404 | 550 → 550 |
| `cpu` | Clips | 4.5 → 4.4 | 1081 → 1136 | 1884 → 1884 |
| `cpu`, `TINTYPE_VIEW_REUSE=on` (final binary against the counters commit) | Small images | 74.4 → 101.5 | 619 → 484 | 192 → 142 |
| `cpu`, `TINTYPE_VIEW_REUSE=on` | Few window positions | 79.5 → 85.5 | 611 → 561 | 180 → 167 |

The CPU rows without the switch are the same work before and after, so
they show the noise: up to 3% on the best pass (ordinary photos: 4.14 s
against 4.28 s; the medians of three passes go the other way, 4.40 s
against 4.33 s). The 2% on the GPU for ordinary photos is inside that: two
views of 550 are reused there. The gains are the small images on the GPU
(+62%) and the images with few window positions (+10%), with less memory
held; clips gain 1%.

**Against the gate.** Small images: file-to-verdict p95 on the CPU falls
from 98.9 to 33.1 ms in process and GPU throughput rises by 62%. Images
with few window positions: p95 82.1 to 70.4 ms, GPU throughput +10%.
Ordinary photos and clips: no change beyond the noise. Nothing is slower.
Parity is below.

### Parity

In the test suite, `tests/view_reuse.rs` compares reuse off, default and
forced on through every scoring path on the CPU engine, bit for bit where
reuse claims to be exact, across batch sizes and image sizes where views
coincide; `tests/cli.rs` does the same through the binary for a photo below
the crop size and checks the counters; `tests/inference.rs` (unchanged)
pins the fixture's logits. `--new-env TINTYPE_VIEW_REUSE=on` (or `=off`)
runs the parity script's binary under test with reuse forced on (or off).

This change alone against `b88c122`. "Differing" counts rows whose JSON line is not the same
bytes.

| Engine | Scheme | Files | Profiles | Rows compared | Differing |
|---|---|---|---|---|---|
| `cpu` | one process per file | 550 photos (308 ordinary, 100 small, 100 few positions, 42 synthetic) | default, `--all-views` | 1100 | 0 |
| `gpu`, `gpu-f32` | all files in one process | the same 550 | default, `--all-views`, `--fast`, `--strict`, `--sensitive` | 5500 | 0 (base against itself: 0) |
| `cpu`, `gpu` | all files in one process | 60 clips | default, `--all-views` | 240 | 0 (base against itself: 0) |

`tintype demo --json` gives the same line as the base binary on `cpu`,
`gpu` and `gpu-f32`.

Forcing reuse on with Accelerate (`TINTYPE_VIEW_REUSE=on`, final binary),
which is what the default avoids:

| Scheme | Files | Profile | Rows | Differing | Largest logit difference | Verdicts or view counts changed |
|---|---|---|---|---|---|---|
| one process per file | 242 (small, few positions, synthetic) | default | 242 | 0 | | 0 |
| one process per file | the same 242 | `--all-views` | 242 | 2 | 4.8e-7 | 0 |
| one process per file | the same 242 | `--fast` | 242 | 0 | | 0 |
| one process per file | 60 clips | default | 60 | 0 | | 0 |
| all files in one process | 550 photos | default | 550 | 5 (base against itself in that run: 0) | 4.8e-7 | 0 |
| all files in one process | 550 photos | `--all-views` | 550 | 0 | | 0 |

So the opt-in costs a few rows in the seventh decimal of the logit on this
engine and buys the CPU throughput in the table above.
