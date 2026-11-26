# GPTTrace on photo batch 9

Batch 9 is the photo set behind the website's benchmark table. GPTTrace
flag-pec-v3 had not been scored on it. This run adds GPTTrace and re-runs
the Tintype default, on the same files and on the same machine (Apple M6,
macOS 27.2).

The protocol was registered before any batch-9 file was read. A single
read then produced the figures below. Every check passed:
- every one of the 31,951 listed files has a GPTTrace score;
- the archived Community Forensics and SwinV2 scores reproduce the website
  exactly;
- the Tintype re-run reproduces the README's batch-9 row exactly.

## Setup

- **GPTTrace.** The weights are the publisher's pinned checkpoint,
  `nakas/gpttrace-img-flag-pec-v3@e5e2423e`: `model.safetensors`,
  1,266,302,604 bytes, SHA-256 `a0adec67…`. That is PE-Core-L/14 at
  448 px, about 317 M parameters.
  - It ran on Tintype's native Metal port of the model (research build;
    not in this release). On WildAIGC the port reproduced every flag of the
    publisher's ONNX export.
  - Each photo is scored as the model card describes: a bicubic centre
    crop and a full-frame resize, with the two raw logits averaged. A
    square photo has one view, because both views are identical.
  - A photo is flagged when the raw logit is above the publisher's
    operating point, 3.562. GPTTrace's "can't tell" band (0 to 3.562)
    counts as not flagged.
- **Tintype.** The v0.4.0 default, with weights byte-identical to the
  release: `tintype batch --json`.
- **Community Forensics and SwinV2.** The archived scores behind the
  website, at 0.5.

## Results

The website's pool: 24,961 photos across 27 sets (13,475 generated,
11,486 real). The OpenFake test shard is excluded, as on the website.

| Detector | Precision | Recall | Real photos flagged | AUROC | Recall at 1% false positives |
|---|---|---|---|---|---|
| Tintype v0.4.0 default | 98.95% | **93.31%** | 1.16% | 0.982 | **92.29%** |
| GPTTrace flag-pec-v3 | **99.11%** | 74.09% | **0.78%** | 0.976 | 76.51% |
| Community Forensics | 95.24% | 84.12% | 4.94% | 0.970 | 70.93% |
| SwinV2 detector | 83.83% | 48.84% | 11.05% | 0.748 | 28.27% |

Paired bootstrap of GPTTrace − Tintype (2,000 draws, real and generated
photos resampled separately), 95% intervals:
- **Precision: −0.09 to +0.40 points.** The two are not separated.
- **Recall: −20.01 to −18.46 points.** GPTTrace is behind.

GPTTrace's misses are concentrated in older generator families:

| Set | GPTTrace recall | Tintype recall |
|---|---|---|
| classic-deepfake | 0.0% | 27.3% |
| classic-crn | 22.0% | 98.8% |
| AIGIBench StyleGAN3 | 27.3% | 95.3% |
| classic-stargan | 29.4% | 100.0% |
| classic-imle | 37.7% | 100.0% |
| AIGIBench GLIDE | 41.6% | 99.5% |
| classic-cyclegan | 47.7% | 98.7% |

GPTTrace finds more on Imagen 3 (96.7% against 90.9%), PhotoMaker (94.0%
against 77.8%), Midjourney (98.0% against 96.0%) and DALL-E 3 (99.8% against
98.6%).

Fourteen of the 27 sets come from older generators:
- the nine `classic-*` sets (GANs, CRN, IMLE and face swaps);
- five AIGIBench GAN sets.

The pooled recall therefore leans towards older generators.

### OpenFake test shard

6,990 photos: 3,520 generated, 3,470 real, all modern text-to-image output.

| Detector | Precision | Recall | Real photos flagged | Recall at 1% false positives |
|---|---|---|---|---|
| Tintype v0.4.0 default | 99.81% | 74.18% | 0.14% | 82.67% |
| GPTTrace flag-pec-v3 | 99.96% | **76.48%** | 0.03% | **87.50%** |
| Community Forensics | 99.42% | 34.32% | 0.20% | 53.47% |
| SwinV2 detector | 97.74% | 72.39% | 1.70% | 67.05% |

On this shard GPTTrace is ahead on recall, by +0.99 to +3.72 points (95%
interval). Precision is not separated.

## Speed

Both runs read the photos from a network share, on a GPU shared with other
jobs, so these are not idle figures.
- **GPTTrace:** 0.311 s per photo.
- **Tintype default:** 13.3 ms per photo, under lighter contention.

At idle rates on the same Apple M6, Tintype is about 30 to 40 times faster
per photo:
- GPTTrace through Tintype's native runtime: 0.109 s per photo (2,500 NTIRE
  public-test photos, start-up included);
- Tintype default: 300–364 photos/s (the website's 2,096-photo speed
  sample).

## Caveats

- **Batch 9 is not blind for Tintype.** It was read in research before the
  shipped calibration was fitted.
- **Batch 9 is not shown to be blind for GPTTrace.** Its training data
  (`nakas/gpttrace-img-v2`) is described only at collection level, so it
  can't be checked file by file. Its model card reports results on
  AIGIBench and So-Fake-OOD, the sources of 18 of the 27 sets.
- **Resampling is by file.** The intervals ignore clusters of related
  images within a set.

## Recompute

The archives hold every file's path, label, generator, set and the four
detectors' scores. Each score is on a 0-to-1 scale where 0.5 is that
detector's shipped cutoff. GPTTrace's score is `sigmoid(raw logit − 3.562)`.

```sh
python3 scripts/report-archive.py docs/results/batch9-gpttrace/scores.tsv.gz --pooled batch9
python3 scripts/report-archive.py docs/results/batch9-gpttrace/openfake-scores.tsv.gz
```
