# Runbook: real-source video audit

Research workstream 5 of `reports/Tintype detection improvement research.md`:
match video quality and content, and audit authentic sources and watermarks.
The audit answers one question: **which video failure remains after source,
codec and visible marks are controlled?** Its answer picks the next video
experiment among source shift, temporal cues (workstream 7) and coverage
(workstream 9).

Nothing here selects a model, a cutoff or a routing rule to ship. Every
archived suite named below is development data, and every cutoff other than
the shipped one is chosen on the clips it is reported on. A ship decision
needs the frozen independent split, re-calibration, an early-exit re-check
and a new entry in `docs/PREREGISTRATION.md`.

## The gate, before any command

These are proposed decision criteria, written down before the runs.

1. **WaveRep route.** The WaveRep second-opinion route is widened (a wider
   band, a different threshold, or on by default) only after it adds
   detections at an equal authentic-flag count on **at least two controlled
   cohorts**. A controlled cohort is defined in [Cohort design](#cohort-design).
   Confirmation 4 is not one: its authentic clips come from the sources the
   video cutoff and the band were calibrated on (`docs/DATA.md`).
2. **A cohort counts only if it is clean.** A cohort whose label can be read
   from metadata alone (step 5) is fixed or dropped before any detector
   result on it is used.
3. **Both operating points.** Recall is reported at 0.1% and at 1% empirical
   false-positive rate wherever the authentic clips allow one false alarm at
   that rate (1,000 and 100 clips). Where they do not, the report says so
   instead of printing a number. Showing a rate below 0.1% with a 95% bound
   takes 2,995 independent authentic clips with none flagged.
4. **The next experiment** is chosen by the [decision rule](#decision-rule),
   from paired results on the controlled cohorts.

## What the archives already say

Reproduced on 2026-11-14 at commit `4682ea0` (scripts; the Rust sources are
those of `b88c122`). These read archived scores; no model was run.

```sh
python3 scripts/replay-research.py
python3 scripts/report-video-audit.py --scores docs/results/confirm4-scores.tsv.gz \
  --system tintype --compare tintype,tintype-second
python3 scripts/report-video-audit.py --scores docs/results/davodi-2026-video/frame-scores.tsv.gz \
  --system tintype --spec models/tintype-video.json
```

| Cohort | Result at the shipped cutoff |
| --- | --- |
| Confirmation 4 | 1,088/1,156 generated flagged, 3/900 authentic flagged (95% upper bound 0.86%, clips counted as independent). By source: Kinetics-400 1/300, MSR-VTT 2/300, MSVD 0/300. |
| Confirmation 4, three allowed false positives | Tintype 1,099/1,156; Tintype with the WaveRep second opinion 1,112/1,156 (+13). |
| Confirmation 4, 1% empirical FPR | 1,111/1,156 with 9/900 authentic flagged (cutoff chosen on these clips). 0.1% is not reportable: 900 authentic clips allow no false alarm at that rate. |
| Confirmation 4, authentic against authentic | Tintype scores separate MSR-VTT from Kinetics-400 (AUROC 0.312, 95% interval 0.271–0.353, Kinetics above MSR-VTT) and from MSVD (0.702, 0.660–0.745, MSR-VTT above MSVD). Kinetics-400 against MSVD is 0.519 (0.472–0.567). |
| DaVODi, content-matched | 72/80 generated flagged, 3/20 authentic flagged (95% bounds 4.22% to 34.37%; each false alarm is 5 points). Neither 0.1% nor 1% FPR is reportable with 20 authentic clips. |
| DaVODi, frames scored | All 3 false alarms and 7 of 8 misses used all 16 frames; the other miss used 8. Of the 8 misses, 5 had at least one frame above the raw cutoff (1.621) and 1 had a quarter of its frames above it. |
| WaveRep alone at 0.5 | Catches 39 of 68 Tintype misses on confirmation 4 and 0 of 8 on DaVODi (`replay-research.py`). |

The authentic-against-authentic numbers are new here. They say the detector
already ranks one authentic source above another by a wide margin, on
sources it was calibrated on: source identity is in the score.

## Cohort design

A **cohort** is a set of authentic clips from one or more collections and
generated clips matched to them, in one processing variant. A cohort is
**controlled** when all of these hold:

- its authentic clips come from collections that are **held out**: not a
  source of training negatives, of calibration, or of an earlier scored
  suite;
- classes are **matched** on cadence, resolution, duration, bitrate and
  motion in that variant (step 4), and the metadata-only probe does not
  predict the label (step 5);
- every clip went through the **same encode** (the `fixed-gop` variant or a
  variant made from it).

### Which authentic collections are held out

`scripts/video_audit.py collection` refuses a source the weights have seen
unless `--not-held-out` is passed, and then records it as an
in-distribution comparison. The sources it knows (from `docs/DATA.md`):

| Not held out | Why |
| --- | --- |
| Pexels | training negatives (`pexels`, `pexelsb`) |
| OpenVid | training negatives (GenBuster-200K-mini) |
| ActivityNet | training negatives (VidGuard-R1) |
| Kinetics-400, UCF101 | training negatives; Kinetics also calibration and confirmation 4 |
| Sakugabooru, Human-Animal-Cartoon | training negatives (hand-drawn animation, cartoons) |
| Sunain gameplay | training negatives (game footage) |
| MSR-VTT, MSVD | calibration and confirmation 4 |

So "stock footage" is not held out as a type: Pexels and OpenVid are in
training. Pixabay is absent from training and is the only authentic source
in DaVODi. Animation, cartoons and gameplay are in training too, so a
held-out animation, CGI or screen collection must come from a different
source than those three.

### Collection types and candidates

The audit wants at least one held-out collection of each type. **None has
been downloaded and no licence below has been verified.** The candidates
are starting points for the licence check in step 1; the choice is open.

| Type | Why it is in the audit | Candidates to check | Notes |
| --- | --- | --- | --- |
| `stock` | DaVODi's 3/20 false alarms are all Pixabay stock | more Pixabay clips beyond the 20; Mixkit; Coverr | Prefer the full-size rendition and record the rendition URL: two of the three flagged DaVODi clips were 960×540 `_small` CDN renditions. |
| `cctv` | fixed camera, low light, heavy compression | VIRAT; MEVA; UCF-Crime | Long recordings: cut clips and keep one sub-directory per recording so they count as one source video. |
| `handheld` | phone capture, stabilisation, platform re-encodes | VISION (native and shared versions of the same phone videos); clips recorded for this audit | Recording new clips avoids every licence question. |
| `animation` | real footage that did not come from a camera was the phase-3 model's most common false alarm | Blender open movies (Sintel, Big Buck Bunny); other hand-drawn sources than Sakugabooru | A feature film is one source video however many clips are cut from it. |
| `cgi` | rendered, not generated | Blender open movies (Cosmos Laundromat, Spring); engine captures other than the Sunain games | |
| `screen` | desktop and app recordings | recordings made for this audit; a screen-content video set | |

Generated clips are matched to the authentic ones in one of two ways:

- **Content-matched** (DaVODi's way): a generated clip of the same scene
  per authentic clip, sharing `scene_id`.
- **Image-to-video from matched starting frames**: export the first frame of
  each authentic clip (step 3) and start the generators from it. The
  generated clip records that frame in `start_frame` and shares the
  `scene_id`; the report counts those pairs.

Which generators to run, and who pays for them, is a decision for a person.

### Variants of every clip

| `transform` | What it is | Exact settings |
| --- | --- | --- |
| `original` | the file as obtained | linked, not re-encoded |
| `trim-copy` | first 4 s of the original bitstream | `ffmpeg -i IN -t 4 -map 0:v:0 -an -c copy -map_metadata -1 OUT` |
| `davodi-norm` | the existing DaVODi normalisation | `ffmpeg -nostdin -hide_banner -loglevel error -threads 1 -i IN -t 4 -map 0:v:0 -an -vf scale=800:-2:flags=bicubic,setsar=1,fps=24,setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709 -map_metadata -1 -c:v libx264 -threads 1 -crf 18 -preset veryfast -pix_fmt yuv420p OUT` |
| `fixed-gop` | the common re-encode | the line above with `-g 48 -keyint_min 48 -sc_threshold 0` after `-preset veryfast`: a closed two-second GOP, no scene-cut keyframes |
| `repost-g1` to `-g3` | simulated reposts of the original | generation *k* re-encodes generation *k*−1: width capped at 720, 540, 360 px (never upscaled, bicubic), CRF 28, 30, 32, `-preset medium`, 4:2:0, first 4 s, source cadence kept |
| `fixed-gop+repost-g1` to `-g3` | the same ladder starting from the `fixed-gop` clip | `--repost-from fixed-gop` |
| `wm-none` | watermark control | identical to `fixed-gop` |
| `wm-mark-P` | sham mark added at corner P | a synthetic ring-and-bars mark drawn by the script, 14% of the frame width, 3% from the edges, 60% opacity, overlaid on the normalised frames before the one encode |
| `wm-crop-P`, `wm-fill-P` | placebo removal where there is no mark | the full-width strip holding the mark's box is cropped away, or the box is filled with FFmpeg `delogo` |
| `wm-mark-P-crop`, `wm-mark-P-fill` | the mark added and then removed | the same crop or fill after the overlay |

Each variant keeps the `parent_id` of its original. `variants` and
`watermark` write `run.json` with the FFmpeg version and these settings.
The `davodi-norm` variant is byte-identical to the archived normalised clips
when FFmpeg 9.0.2 is used (checked for all 100; see the pipeline check).

Reposts of the original keep its cadence, and with it any cadence
difference between the classes: use `fixed-gop+repost-*` for controlled
cohorts and `repost-*` to see what a real repost does.

## Steps

Paths: `AUDIT=/path/to/audit-work` is a scratch directory outside the
repository. FFmpeg and FFprobe must be on `PATH`. Every script is standard
library only; `python3 scripts/video_audit.py --self-test` and
`python3 scripts/report-video-audit.py --self-test` check their arithmetic.

### 1. Licence check, before any download

For each candidate collection, before fetching a single file:

1. Open the licence at its source (the dataset card, the site's licence
   page, or the paper's data statement). Record the URL and the date.
2. Confirm it permits: downloading for research, automated processing,
   making derived files (re-encodes, crops, overlays) that stay local, and
   publishing scores and aggregate statistics. Note whether it permits
   redistributing clips; the audit does not need to.
3. Check for a terms-of-service layer on top of the licence (rate limits,
   bans on automated download, bans on use for machine-learning
   evaluation). Stock sites have them.
4. Check that the footage is what it claims: no generated clips in an
   "authentic" collection (stock sites now carry AI-labelled uploads, which
   must be filtered out and the filter recorded).
5. Write the outcome into `docs/DATA.md` (the source, the selection rule
   and the licence) and `NOTICE.md` if attribution is required, in the same
   change that adds the collection's results.

Only then download, into `$AUDIT/raw/<source_id>/`, one sub-directory per
long recording. `collection` will not run without `--licence` and
`--licence-source`, and stores both in `collections.tsv`.

### 2. Register the collections

```sh
python3 scripts/video_audit.py collection $AUDIT/cohort \
  --source-id pixabay-2026b --type stock --clips $AUDIT/raw/pixabay-2026b \
  --licence "<licence name as read>" --licence-source "<URL>, read <date>"
# repeat per collection: --type cctv | handheld | animation | cgi | screen
```

For generated clips, give an index with a `file` column and any of
`scene_id`, `start_frame`, `generator`, `generator_family`, `source_video`:

```sh
python3 scripts/video_audit.py collection $AUDIT/cohort \
  --source-id i2v-2026-11 --type generated --clips $AUDIT/raw/i2v-2026-11 \
  --index $AUDIT/raw/i2v-2026-11.tsv \
  --licence "<generator terms>" --licence-source "<URL>, read <date>"
```

This writes `$AUDIT/cohort/manifest.tsv` (nine columns, `transform` =
`original`), `audit.tsv` (collection type, source video, starting frame,
SHA-256 per clip) and `collections.tsv` (licence and held-out status).

To extend DaVODi itself, normalise the registered collections with the
matched clips:

```sh
python3 scripts/prepare-davodi-video.py ForZenodo.zip authentic $AUDIT/davodi \
  --source-record docs/results/davodi-2026-video/inputs.tsv \
  --collection $AUDIT/cohort
```

`--grouped` (implied by `--collection`) writes `manifest-grouped.tsv` (the
normalised clips), `originals-grouped.tsv` (the files they were made from,
same `parent_id`) and `audit.tsv`. Without the flags the script's outputs
are unchanged. Collection clips shorter than four seconds are left out and
counted.

### 3. Starting frames for image-to-video

```sh
python3 scripts/video_audit.py start-frames $AUDIT/cohort/manifest.tsv $AUDIT/start-frames
```

One PNG per authentic clip and `start-frames.tsv` (`start_frame`,
`scene_id`). Generate from those frames outside this repository, then
register the results with step 2, copying `start_frame` and `scene_id` into
the index.

### 4. Variants, probe and class matching

```sh
python3 scripts/video_audit.py variants $AUDIT/davodi/originals-grouped.tsv $AUDIT/variants \
  --variants original,trim-copy,repost-g1,repost-g2,repost-g3
python3 scripts/video_audit.py variants $AUDIT/davodi/originals-grouped.tsv $AUDIT/controlled \
  --variants fixed-gop,repost-g1,repost-g2,repost-g3 --repost-from fixed-gop
python3 scripts/video_audit.py watermark $AUDIT/davodi/originals-grouped.tsv $AUDIT/watermark \
  --positions br,tl

python3 scripts/video_audit.py probe $AUDIT/variants/manifest.tsv $AUDIT/controlled/manifest.tsv \
  --out $AUDIT/probe.tsv
python3 scripts/video_audit.py balance $AUDIT/probe.tsv
```

`probe` records container, codec, profile, pixel format, size, cadence,
duration, bitrate, bits per pixel, audio, keyframe count, mean and longest
GOP, B-frames, and a motion statistic: the mean absolute difference between
frames 1/8 s apart at 64×36 grey over the first 30 s (the span the detector
samples). `balance` prints, per variant, each feature for the two classes
with a standardized mean difference or a total-variation distance, and
marks mismatches.

Where a variant is mismatched, keep the matched subset:

```sh
awk -F'\t' '$9 == "fixed-gop"' $AUDIT/controlled/manifest.tsv > $AUDIT/controlled/fixed-gop.tsv
python3 scripts/video_audit.py match $AUDIT/controlled/fixed-gop.tsv $AUDIT/probe.tsv \
  $AUDIT/controlled/fixed-gop-matched.tsv --features fps,log2_pixels,motion,log2_bits_per_pixel
```

(`fixed-gop.tsv`, the `fixed-gop` rows of the controlled manifest, is also
what the all-frames and second-opinion runs of step 6 score.) The
matching is greedy nearest-neighbour within a caliper, inside a scene when
scenes are recorded; it prints the balance table of what it kept.

Where a generator leaves a visible mark of its own, add its region so the
crop and fill become real removals on the marked clips and placebos on the
rest: `--region veo=0.83,0.88,0.14,0.08` (x, y, width, height as fractions
of the frame; these numbers are an example, measure the mark first). A mark
that moves between frames cannot be covered by one region.

### 5. Metadata-only probe

```sh
python3 scripts/video_audit.py metadata-probe $AUDIT/probe.tsv
```

A logistic regression on the probe's container, codec, cadence, size,
duration, bitrate and GOP fields, cross-validated with whole groups held
out (sources, or scenes when a class has one source). An AUROC whose
interval excludes 0.5 means the cohort has a shortcut; the heaviest
features say which. Fix it (a different variant, the matched subset, a
common frame size) before step 7.

### 6. Scoring

Scoring uses the Apple GPU (`score-video`), which is shared. Never point
`--weights` at `models/`: copy the video model first and verify the shipped
files afterwards.

```sh
mkdir -p runs/ws5
cp models/tintype-video.safetensors runs/ws5/tintype-video.safetensors
cp models/tintype-video.json        runs/ws5/tintype-video.json

W=runs/ws5/tintype-video.safetensors
./target/release/tintype score-video --weights $W --manifest $AUDIT/variants/manifest.tsv   --out $AUDIT/scores-variants.tsv
./target/release/tintype score-video --weights $W --manifest $AUDIT/controlled/manifest.tsv --out $AUDIT/scores-controlled.tsv
./target/release/tintype score-video --weights $W --manifest $AUDIT/watermark/manifest.tsv  --out $AUDIT/scores-watermark.tsv

# Coverage: the same clips with every frame scored.
./target/release/tintype score-video --weights $W --manifest $AUDIT/controlled/fixed-gop.tsv \
  --all-frames --out $AUDIT/scores-fixed-gop-all-frames.tsv

# The WaveRep route, one controlled cohort at a time (WaveRep weights from `tintype data baselines`).
./target/release/tintype score-video --weights $W --manifest $AUDIT/controlled/fixed-gop.tsv \
  --second-opinion baselines/waverep-g4.ckpt --out $AUDIT/scores-fixed-gop-cascade.tsv

./target/release/tintype verify-models
```

`score-video` neither calibrates nor writes a spec; the copy is so that no
later `calibrate` or `finalize-video` on the same path can touch the
shipped files. Each manifest of more than about 2,000 clips is a run to
schedule, not to start alongside other work. The second-opinion run turns
early exit off and scores every band clip with WaveRep at 64 frames: on the
pipeline check it took 75 s for 102 clips.

### 7. Report

```sh
cd $AUDIT
python3 $REPO/scripts/report-video-audit.py \
  --scores tintype=scores-variants.tsv --scores tintype=scores-controlled.tsv --scores tintype=scores-watermark.tsv \
  --manifest variants/manifest.tsv --manifest controlled/manifest.tsv --manifest watermark/manifest.tsv \
  --sidecar davodi/audit.tsv --sidecar variants/audit.tsv --sidecar controlled/audit.tsv --sidecar watermark/audit.tsv \
  --spec $REPO/runs/ws5/tintype-video.json > report.md

python3 $REPO/scripts/report-video-audit.py \
  --scores tintype=scores-controlled.tsv --scores cascade=scores-fixed-gop-cascade.tsv \
  --manifest controlled/manifest.tsv --compare tintype,cascade > report-cascade.md

python3 $REPO/scripts/report-video-audit.py \
  --scores early-exit=scores-controlled.tsv --scores all-frames=scores-fixed-gop-all-frames.tsv \
  --manifest controlled/manifest.tsv --spec $REPO/runs/ws5/tintype-video.json \
  --compare early-exit,all-frames > report-all-frames.md
```

Make each variant once (step 4 puts `fixed-gop` in the controlled set
only): a variant scored under two paths is counted twice. The report
prints, per detector:

- **Authentic collections**: flags/n, FPR, one-sided 95% Clopper–Pearson
  lower and upper bounds per collection and variant, and the same by source
  video, where a video is flagged when any of its clips is. Clips cut from
  one recording are one trial; read the bound by video for those.
- **Generators**: recall per generator and variant.
- **Recall at 0.1% and 1% empirical FPR**, or the reason it is not
  reportable.
- **Frames scored before the early exit**, by outcome, and how many errors
  had frames on the other side of the raw cutoff.
- **Paired shifts**: each variant against its baseline over the same
  originals, for authentic clips (false-positive count before and after)
  and generated clips (detections before and after), with the newly flagged
  and no-longer-flagged counts, an exact sign test and the mean logit shift
  with an interval that resamples source videos.
- **Sham mark against its placebos**: the marked clip minus the placebo of
  the same original, in logits.
- **Score separation between authentic collections**: pairwise AUROC with a
  clustered interval.
- **Matched scenes**, and with `--compare` the detections of a second
  detector or route at the first one's authentic-flag count, per variant.

`tintype report --operating-points --groups <manifest> --slice-by
source,transform --cluster-by parent` gives the Rust-side fixed-budget
tables on the same score files.

## Decision rule

Proposed. Apply it to controlled cohorts only, and to at least two of them.
Sort every error at the shipped cutoff on the `fixed-gop` variant into one
bin, in this order:

| Bin | An error belongs here when | Read from |
| --- | --- | --- |
| **Coverage** | a miss or a false alarm reverses with `--all-frames`; or a miss was decided before all 16 frames; or a miss has at least a quarter of its frames above the raw cutoff (the signal is in part of the clip and the mean loses it) | frames table, frame-cutoff line, `--compare early-exit,all-frames` |
| **Source or processing shift** | a false alarm that is not a coverage error; or a miss that was detected in `original` and lost after the common encode or a repost (a paired loss) | authentic collections table, paired shifts |
| **No frame signal** | a miss that used all 16 frames, has fewer than a quarter of them above the raw cutoff, and was already missed in `original` | frames table, paired shifts |

Then:

- **Source shift is next** when the second bin is the largest on at least
  two controlled cohorts, and the collection evidence agrees: a held-out
  collection whose 95% lower bound on FPR is above the video calibration
  target (0.5%), or an authentic-against-authentic AUROC whose interval
  excludes 0.5 under the common encode, or authentic clips newly flagged by
  a re-encode or repost. The experiment is matched authentic sources and
  matched processing in video training (the video form of workstreams 1
  and 4).
- **Coverage is next** (workstream 9: temporal or spatial supervision, and
  the sampling and early-exit schedule) when the first bin is the largest
  on at least two controlled cohorts.
- **Temporal cues are next** (workstream 7) when the third bin is the
  largest on at least two controlled cohorts and the collection evidence
  for source shift is absent: the frame baseline sees every frame, the
  cohort has no shortcut, and the frames carry no signal. Detections lost
  under `fixed-gop+repost-*` with authentic flags unchanged point the same
  way: a frame cue that compression removes.
- **No decision** when no bin leads on two cohorts, or the cohorts are too
  small to tell bins apart. Say so, and enlarge the cohorts; do not pick
  the bin that leads by one clip.

The watermark table is a veto on the cohort, not a bin. If the sham mark
moves scores (marked minus placebo, interval excluding zero) by an amount
that changes verdicts, a visible mark is a shortcut in one direction or the
other: remove or equalise visible marks across the classes before reading
the bins, and report the mark effect as a finding of its own.

The WaveRep gate is separate: run `--compare tintype,cascade` on each
controlled cohort and count the cohorts with a gain. Fewer than two: the
route stays as it is.

## Pipeline check on development data

This exercised every step on the 100 archived DaVODi clips plus two
gameplay clips from a training store (registered with `--not-held-out` to
exercise a second collection and the source-video clustering). **It is a
check that the tooling runs, on development data; nothing below is a blind
result, the cohort is not controlled (one authentic source, 20 scenes), and
no cutoff or route was chosen from it.**

Run on 2026-11-13/14: scripts at `4682ea0`, binary built from `b88c122`
(`cg build --release --locked --bin tintype`), Apple M6, 32 GB, macOS 27.2,
FFmpeg 9.0.2, `score-video` on the default device with a copy of the
shipped video model at `runs/ws5/`. Commands are those of steps 2 to 7 with
`--positions br`.

- **Preparation.** `prepare-davodi-video.py … --source-record … --collection`
  reproduced `inputs.tsv` and the four-column `manifest.tsv` byte for byte;
  the `davodi-norm` variant of `video_audit.py variants` matched the
  archived `normalized_sha256` for 100 of 100 clips; `fixed-gop` equals
  `wm-none` byte for byte. Scoring the `davodi-norm` variant gave 72/80 and
  3/20, as archived.
- **Metadata-only probe** (22 authentic, 80 generated, held out by scene):
  `original` AUROC 1.000 (1.000–1.000); `repost-g1` 0.985 (0.964–1.000);
  `davodi-norm` 0.603 (0.523–0.685); `fixed-gop` 0.672 (0.607–0.747);
  `fixed-gop+repost-g1` 0.504 (0.395–0.620), `-g2` 0.457 (0.352–0.574),
  `-g3` 0.459 (0.356–0.571). The originals are trivially separable by
  metadata, reposts of the originals stay so through cadence, and the
  normalised clips keep a smaller shortcut: Seedance clips are 800×452 and
  all others 800×450.
- **Paired shifts, generated clips (80 pairs).** Against `original` (65
  flagged): `fixed-gop` 72 (8 newly flagged, 1 lost, sign test p 0.039);
  `trim-copy` 60; `repost-g1` 59, `-g2` 48, `-g3` 44. Against `fixed-gop`
  (72): `fixed-gop+repost-g1` 60, `-g2` 62, `-g3` 56, mean logit shifts
  −1.09, −1.09 and −1.88.
- **Paired shifts, authentic clips (22 pairs; 20 Pixabay).** Flagged: 2 in
  `original`, 3 in `fixed-gop`, 3/5/3 in `repost-g1/2/3`, 4/6/4 in
  `fixed-gop+repost-g1/2/3`. No sign test on authentic clips was below
  0.25.
- **Sham mark, bottom right.** Against `wm-none`, the mark left authentic
  flags at 3/22 and took generated detections from 72 to 66 of 80 (0 newly
  flagged, 6 lost, p 0.031). Marked minus placebo fill: −0.46 logits
  (−0.64 to −0.31) on authentic clips and −0.88 (−0.98 to −0.77) on
  generated clips. The placebo fill moved neither class (+0.02 and +0.01
  logits, no verdict changed); the placebo crop moved authentic clips by
  +0.25 (−0.04 to +0.54). On these clips the mark itself lowers the score.
- **Coverage.** `--all-frames` on `davodi-norm`: 72/80 and 3/22, the same
  counts as with early exit, and 73 detections each at three allowed false
  positives.
- **WaveRep route.** On `fixed-gop` (22 authentic, 80 generated), 43 clips
  went to WaveRep; the cascade flagged 56/80 and 3/22 at 0.5 against
  Tintype's 72/80 and 3/22. At three allowed false positives: Tintype 74,
  cascade 56 (−18).
- **Frames scored** over all 1,632 scored variants: missed generated clips
  used 13.3 frames on average (209 of 270 used all 16), flagged authentic
  clips 14.3 (47 of 54 used all 16).

## Ready to run

Nothing above needs a decision except the collections. Once step 1 is done
for at least one collection per type and the matched generated clips
exist:

1. Steps 2 to 5 (CPU; FFmpeg).
2. Step 6 scoring. For a cohort of *n* originals this is 15·*n* clips with
   one mark position and 20·*n* with two (five variants of the original,
   the fixed-GOP clip and its three reposts, `wm-none` and five watermark
   variants per position), plus the all-frames and second-opinion passes
   over the fixed-GOP clips. The pipeline check scored 714 clips in 61 s
   and 612 in 58 s.
3. Step 7, then the decision rule, written up with the cohort sizes beside
   every count.

## Not done here

- No collection was chosen or downloaded, and no licence was verified.
- No generated clips were made; image-to-video matching is supported and
  untested beyond its self-test and the export of two starting frames.
- The normalised frame height still differs by generator (452 against
  450 px). A common frame size is not implemented.
- The repost ladder is three libx264 steps. Platform encoders (hardware
  H.264, HEVC, AV1, platform-specific filters) are not simulated.
- The sham mark is one design at fixed opacity. Real generator marks need
  `--region`, per generator, and a moving mark is out of scope.
