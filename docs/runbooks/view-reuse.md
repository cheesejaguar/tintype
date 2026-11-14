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
