# Tintype Photo Benchmark v1, blind split: protocol

Registered 2026-11-25, before any blind-split file was listed, fetched or
scored. No scores from the AIGIBench full-protocol run (registered
separately the same day, ea6e4411) had been read when this was committed.

## The benchmark

The **Tintype Photo Benchmark (TPB)** turns the website's batch-9 pool into
a fixed, versioned benchmark. Version 1 has three parts.

| Split | What | Status |
|---|---|---|
| `open` | The website's pool: photo batch 9 without the OpenFake shard. 24,961 photos (13,475 generated, 11,486 real) in 27 sets from three public sources: AIGIBench (17 test archives), CNNDetection's classic test set (9 families) and So-Fake-OOD (2 shards) | Not blind: read in research before Tintype's shipped calibration was fitted |
| `openfake` | Batch 9's OpenFake test shard 7: 6,990 photos | Reported separately, as on the website |
| `blind` | Fresh files from the same 27 sets and sources, defined below | Blind for every detector scored here |

Version 1 is frozen once its manifests are committed. New generators go
into v2. Version 1 is never edited.

## Blind split: selection rule

The blind split uses the same sources, revisions and set definitions as
the open split. Within each set, it takes only files that no earlier work
has used.

**Sources:**
- **AIGIBench:** `HorizonTEL/AIGIBench@e44ec40e`, `test/<archive>.zip`,
  the same 17 archives as the open split.
- **CNNDetection:** `sywang/CNNDetection`, `CNN_synth_testset.zip`, the
  same 9 families. The revision is the one `src/fetch.rs` pins.
- **So-Fake-OOD:** `saberzl/So-Fake-OOD`, at the pinned revision. The set
  is the 46 `data/test_image-000NN-of-00046.parquet` shards. Shards
  already read by any plan in `src/plans.rs` are 00000, 00001, 00003,
  00004, 00006, 00007, 00009, 00010, 00012, 00015, 00018, 00020, 00021,
  00022, 00024, 00025, 00027, 00030, 00033, 00036, 00037, 00039, 00040,
  00042 and 00045. The blind split takes **00002 and 00044**, the lowest-
  and highest-numbered unread shards. Rows are kept as batch 9 kept them:
  label `FULL_SYNTHETIC` or `REAL`.

**Unused files (zip archives).** A member is unused if:
1. its path is in no earlier manifest:
   - the open and OpenFake splits;
   - every frozen manifest in `docs/manifests/` (`benchmarks` and
     `confirm1` to `confirm7`);
   - batch 8 (`data/fresh9`) and batch 9 (`data/fresh10`) on the data
     share;
   - every training-store manifest;
2. its CRC-32 and uncompressed size, from the zip's central directory,
   match no member used under rule 1 in any of the three sources.

Members are images under `0_real/` or `1_fake/` with an image extension.

**How many.** Per set and class, the same count as the open split.
- If fewer unused files remain, take all of them.
- If fewer than 100 remain in either class, drop the set from the blind
  split, and report the drop.

For So-Fake-OOD, take every kept row of the two shards.

**Which ones.** Sort the unused members of a set and class by path. With
`n` to take from `m` unused members, take the indices
`floor((i + 0.5) * m / n)` for `i` in `0 .. n-1`.

**Duplicates.** After download, drop:
- any file whose SHA-256 equals that of a file in the open or OpenFake
  split;
- any file whose SHA-256 equals that of a training-store file, checked
  with the same exact-overlap routine the data plans use.

Both counts are reported.

The blind manifest gets one row per file: split, set, source, archive or
shard, member path or row index, label, generator and SHA-256. It is
committed **before any detector scores a blind file**.

## Detectors

Each runs once on the blind split, at its shipped operating point.

| Detector | How it runs |
|---|---|
| Tintype v0.4.0 default | `tintype batch --json`, release build, weights byte-identical to v0.4.0 |
| GPTTrace flag-pec-v3 | Pinned `nakas/gpttrace-img-flag-pec-v3@e5e2423e` (SHA-256 `a0adec67…`), the research build's native runtime, flagged when the mean two-view raw logit exceeds 3.562 |
| Community Forensics | `tintype score --reference commfor`, official 384 px weights, cutoff 0.5 |
| SwinV2 detector | `tintype score --reference swinv2`, cutoff 0.5 |

## Metrics

These are fixed for every version of the benchmark and computed by the one
scorer, `scripts/benchmark/score.py`.

- **Precision, recall, and the false-positive rate on real photos**, at
  the detector's shipped cutoff. Each score is mapped to 0–1, where 0.5
  is the shipped cutoff, and a file is flagged when its score is above 0.5.
- **Recall at 1% false positives.** Sort the real files' scores in
  descending order. The cutoff `t` is the score at index
  `floor(0.01 × number of real files)`, and a generated file counts when
  its score is above `t`.
- **Time per image.** Wall-clock time from start-up to the last verdict,
  divided by the number of images, with the hardware stated. The blind
  read reports it as measured, with the machine's other load noted. It is
  not a speed claim.
- **Intervals.** Bootstrap with 2,000 draws and seed 20261125, resampling
  real and generated files separately, with 95% percentile intervals.
  - Detector against detector on the same files: paired.
  - Tintype's blind split against the open split: unpaired, each split
    resampled on its own.

Pooled figures are over every file of the split. Per-set figures are
reported too.

## Questions this read answers

1. **Blind against the website.** For Tintype, the difference between
   the blind split and the open split in precision, recall, real photos
   flagged and recall at 1% false positives, with intervals. Pooled
   figures are reported, and also restricted to the sets present in both
   splits.
2. **Do the site's claims hold?** The website states, from the open
   split: 99.0% precision, 93.3% recall, 1.2% false positives and 92.3%
   recall at 1% false positives. Tintype is first on recall and on recall
   at 1% false positives, and the four detectors are ranked
   Tintype / GPTTrace / Community Forensics / SwinV2 on recall.
   - A figure **needs changing** when the blind 95% interval excludes the
     site's figure by more than its rounding (0.05 points).
   - A ranking **needs changing** when the paired interval for the blind
     difference contradicts it.
   - Otherwise the claim holds, and the site gains a sentence citing the
     blind split.

## What is reported and when

- **Before the read:** the blind manifest, the counts per set, and the
  sets dropped with the reason.
- **After the read:** every metric for every detector, pooled and per
  set; the two answers above; and per-file scores in the score-archive
  format.

## Caveats stated with the result

- **Not new generators.** The blind split comes from the same archives as
  the open split, so it measures the same generators on unseen files. It
  does not measure new generators; that is what v2 is for.
- **AIGIBench was scored in full the same day.** The AIGIBench
  full-protocol comparison (ea6e4411) scored every AIGIBench test file
  with the same v0.4.0 default. Nothing in Tintype was trained,
  calibrated or tuned on those scores. Its aggregate results will be
  public, so the blind split's AIGIBench part is blind with respect to
  tuning, not to aggregate knowledge.
- **GPTTrace's training data cannot be checked against these files.** The
  split is blind for GPTTrace only to the extent its training excludes
  them.

## Amendment 1 (2026-11-26, after scoring stopped on the last set, before any score was read)

The scoring run stopped on the So-Fake-OOD set, the last of 27. No score
from any set had been read.

1. **Three files were misnamed.** They are GIFs: rows 444 of shard 00002,
   and 844 and 1860 of shard 00044. The builder's table of formats had no
   GIF, so it named them `.bin`; the data plans' ingest names them `.gif`.
   Neither `tintype batch` run picked them up.
   - Their manifest paths now end in `.gif`.
   - Bytes and SHA-256 are unchanged.
2. **One file is too large for GPTTrace's decoder.** Row 1221 of shard
   00044 is an 8000 × 12000 JPEG. The research build that runs GPTTrace
   refused it, because the estimated decode memory (576 MB) is over its
   512 MB limit. The v0.4.0 default scored it.
3. **Rule for unreadable files, fixed now for every detector:** a file a
   detector reports it cannot decode counts as **not flagged** for that
   detector (score 0). The count per detector is reported with the
   results. A file with no output row at all still stops the read.
4. **The So-Fake-OOD set is scored again** with the corrected paths. The
   rows from the stopped run are discarded without being read. The other
   26 sets keep their scores.
