# Tintype Photo Benchmark (TPB)

TPB is a fixed, versioned benchmark for detectors of generated photos. It
is built from public research sets. Each version is frozen as manifests
of every image, giving its source, label and SHA-256, plus one script
that rebuilds the images from their sources and scores any detector the
same way.

The current version is **v1**, frozen on 2026-11-25.

## Splits of v1

| Split | Files | Generated | Real | Sets | Blind for Tintype? |
|---|---|---|---|---|---|
| [`open`](v1/open.tsv.gz) | 24,961 | 13,475 | 11,486 | 27 | No: read in research before Tintype's shipped calibration was fitted |
| [`openfake`](v1/openfake.tsv.gz) | 6,990 | 3,520 | 3,470 | 1 | No; Tintype trains on other shards of the same collection |
| [`blind`](v1/blind.tsv.gz) | see [the blind read](v1/BLIND-RESULT.md) | | | | Yes: registered in [BLIND-PROTOCOL.md](v1/BLIND-PROTOCOL.md) before any detector scored it |

- **`open`** is the pool behind the website's table: Tintype's photo
  batch 9 without its OpenFake shard. Its 27 sets come from three
  sources:
  - 17 AIGIBench test archives;
  - 9 families of CNNDetection's classic test set;
  - 2 So-Fake-OOD shards.
- **`openfake`** is batch 9's OpenFake test shard. It is reported
  separately.
- **`blind`** holds files from the same sets that no earlier work had
  used. It is never used to train, calibrate or tune any Tintype model.

## Sources and licences

| Source | Revision | Licence on the dataset card | Images in the benchmark |
|---|---|---|---|
| [HorizonTEL/AIGIBench](https://huggingface.co/datasets/HorizonTEL/AIGIBench) | `e44ec40e` | CC BY-NC-SA 4.0 | 17 test archives |
| [sywang/CNNDetection](https://huggingface.co/datasets/sywang/CNNDetection) | `f287cf3c` | CC BY-NC-SA 4.0 | `CNN_synth_testset.zip`, 9 families |
| [saberzl/So-Fake-OOD](https://huggingface.co/datasets/saberzl/So-Fake-OOD) | `b4b34d16` | CC BY-NC 4.0 | 2 test shards per split |
| [ComplexDataLab/OpenFake](https://huggingface.co/datasets/ComplexDataLab/OpenFake) | `00b2f0ae` | CC BY-NC 4.0 | test shard 7 |

All four licences are non-commercial. The real photos in these sets come
from third parties: FFHQ, CelebA-HQ, Open Images, LSUN, ImageNet, COCO and
social-media posts. The dataset licences do not clear those rights. **This
repository therefore ships the recipe, not the images.** It provides
manifests with source, revision, archive member or parquet row and
SHA-256, and a script that fetches each file from its source and checks
it. Use of the images is governed by each source's terms.

## Rebuild

```sh
python3 scripts/benchmark/tpb.py rebuild docs/benchmark/v1/open.tsv.gz --root ~/tpb/v1/open
python3 scripts/benchmark/tpb.py verify  docs/benchmark/v1/open.tsv.gz --root ~/tpb/v1/open
```

- **Zip archives** are read with HTTP range requests, so only the listed
  members are downloaded.
- **Parquet shards** are downloaded whole (about 3 GB each), read row by
  row and deleted. This needs `pyarrow`.
- **Hashes.** Every file must match its SHA-256, or the rebuild reports
  it.

## Metrics (fixed for every version)

`tpb.py score` computes these for each detector, pooled and per set.

| Metric | Definition |
|---|---|
| Precision, recall, real-photo false-positive rate | At the detector's shipped cutoff. Scores are on a 0–1 scale where 0.5 is that cutoff, and a file is flagged when its score is above 0.5 |
| Recall at 1% false positives | Sort the real files' scores in descending order. The cutoff `t` is the score at index `floor(0.01 × real files)`, and a generated file counts when its score is above `t` |
| Time per image | Wall-clock seconds from start-up to the last verdict, divided by the number of files, with the hardware and other load stated |
| Intervals | Bootstrap with 2,000 draws and seed 20261125, real and generated files resampled separately, 95% percentile intervals. Paired between detectors on the same split; unpaired between splits (`tpb.py diff`) |

A detector's output for `score` is either:
- a TSV of `path<TAB>score`, or
- JSON lines with `path` and `score`, as `tintype batch --json` writes
  them.

Paths are matched to the manifest by their longest suffix.

## Versions

- **A version is never edited** once its manifests are committed. A
  correction, a new set or a new generator goes into the next version,
  with its own manifests and, where it has one, its own blind split.
- **v2** collects the blind sets of generators added through the
  [new generator playbook](../NEW_GENERATOR_PLAYBOOK.md), starting with
  Imagen 3.
- **Blind splits** are scored only under a protocol committed before the
  read. They are never used for training, calibration or threshold
  fitting.

## Results

Each detector is at its own shipped cutoff. All runs used an Apple M6.

### v1 `open` (24,961 files)

| Detector | Precision | Recall | Real flagged | Recall at 1% FPR |
|---|---|---|---|---|
| Tintype v0.4.0 default | 98.95% | 93.31% | 1.16% | 92.29% |
| GPTTrace flag-pec-v3 | 99.11% | 74.09% | 0.78% | 76.51% |
| Community Forensics (384) | 95.24% | 84.12% | 4.94% | 70.93% |
| SwinV2 detector | 83.83% | 48.84% | 11.05% | 28.27% |

### v1 `blind`

See [v1/BLIND-RESULT.md](v1/BLIND-RESULT.md).
