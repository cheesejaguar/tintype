# Detection evidence

## How much generated media is found at a 1% false-positive rate?

Each detector gets the same false-positive budget on the same files. The resulting recall measures how well scores rank generated media above authentic media. It uses a threshold derived from that batch's real items for each detector. Use the shipped-threshold results to see what happens when you run Tintype with its defaults.

| Evaluation | Files | Tintype | Community Forensics | SwinV2 detector | WaveRep |
|---|---:|---:|---:|---:|---:|
| Photo confirmation batch 7 | 23,052 | **88.5%** | 63.1% | 33.8% | — |
| Video confirmation batch 4 | 2,056 | **96.1%** | — | — | 87.4% |

At the shipped threshold, Tintype found **89.7%** of the generated photos and flagged **130 of 10,747** real photos. On video it found **94.1%** of the generated clips and flagged **3 of 900** real clips. Precision was 98.8% for photos and 99.7% for video with the class mix in these batches. Precision changes with the prevalence of generated media. For example, 89.7% recall and a 1.21% false-positive rate would give about 43% precision if 1 in 100 inputs were generated and both rates carried over.

These are author-run comparisons against specific released checkpoints. Each batch used a configuration written down before the files were fetched. The [benchmark report](BENCHMARK.md) gives the preprocessing, hardware, baselines and per-set results. The [protocol](PREREGISTRATION.md) describes the model and threshold decisions, and the [data records](DATA.md#frozen-evaluation-sets) identify the exact media. Some generator families and source collections were shared with training. Several AIGIBench sets correlate file format with class. The photo and video datasets also include authentic source material from different distributions than the generated items.

## Social-media JPEGs

To see how much of the photo result holds when the file extension is shared across both classes, we filtered the saved batch 7 So-Fake-OOD scores to `.jpg` and `.jpeg` paths. Every detector scored the same **2,281 JPEGs** (878 generated and 1,403 authentic). The generated items cover seven generators. Authentic items come from Reddit, Tumblr and Bluesky. This is an analysis of scores already collected for batch 7. JPEG quality, resizing, upload paths and content can still differ across classes.

| Detector | Recall at 1% FPR | Recall at shipped threshold | Real images flagged at shipped threshold | AUROC |
|---|---:|---:|---:|---:|
| **Tintype** | **77.0%** | 77.7% | 18 / 1,403 (1.28%) | 0.958 |
| Community Forensics | 28.8% | 27.0% | 13 / 1,403 (0.93%) | 0.851 |
| SwinV2 detector | 46.7% | 68.7% | 89 / 1,403 (6.34%) | 0.895 |

With the false-positive budget fixed to Tintype's 18 real photos flagged, Community Forensics found 30.0% and SwinV2 found 49.2%. On the full 3,742-image So-Fake-OOD slice, including non-JPEG inputs, Tintype found 79.2% at 1% FPR. The full slice results are under `sofake-ood-fresh6` in the [photo tables](results/confirm7.md). This subset's rates describe those seven generators and three real sources. [So-Fake-OOD dataset](https://huggingface.co/datasets/saberzl/So-Fake-OOD).

At that same 1% false-positive budget, the default Tintype score ranks ahead of both references on each of the seven generator subsets. Performance still varies considerably. It found 85 of 94 FLUX.2 images, 61 of 65 Nano Banana 2 images and 92 of 204 FLUX.1 Pro images. The per-generator counts use the shared real pool for each detector's threshold.

## JPEG re-encoding and downscaling

We ran the released Tintype photo checkpoint and Community Forensics on the WasItAIGenerated Frontier public photo set on macOS 27.2 using the CPU. We scored the originals, a same-dimension JPEG re-encode, and a stronger JPEG re-encode with the long side capped at 960 pixels. Both JPEG passes used FFmpeg 9.0.2's MJPEG encoder in 4:2:0 with metadata stripped. The quality settings were `-q:v 4` and `-q:v 12`, respectively. These transformations are controlled approximations of common post-processing. The dataset was already used during development. SwinV2's Rust port requires a GPU, so this CPU run covered two systems. [Source dataset](https://huggingface.co/datasets/wasitaigeneratedcom/frontier-image-video-bench-2026-10/tree/e6a8b83d15fab1da958848368fcd07e943afafd0).

Both systems scored the same 745 originals and their paired copies. Two of the 747 inputs (`i90bff65232d9.jpg`, `i66dfe571bf71.jpg`, both authentic) failed to decode in at least one variant. We dropped both inputs from all three groups. Each group contains 261 generated images and 484 authentic images.

| Input | Tintype recall at 1% FPR | Community Forensics recall at 1% FPR | Tintype shipped recall | Tintype shipped FPR |
|---|---:|---:|---:|---:|
| Original files | 69.3% | 28.7% | 78.5% | 1.86% |
| Same-dimension JPEG | 53.3% | 14.2% | 44.4% | 0.21% |
| JPEG capped at 960 px | 28.0% | 8.4% | 38.7% | 2.07% |

Re-encoding lowered recall by 16 percentage points at the same false-positive budget. The stronger transform reduced it by 41.3 points, leaving almost three in four generated images undetected at that budget. The [archived scores](results/frontier-jpeg-stress-scores.tsv.gz) contain all paired rows. Actual Instagram, WhatsApp and re-shared copies are reported separately in the [development results](results/benchmarks_phase6_weights.md).

## Recompute the tables

The [photo scores](results/confirm7-scores.tsv.gz) and [video scores](results/confirm4-scores.tsv.gz) archive the scored paths, labels and scores used for both comparisons. The script checks file identities against the frozen manifests before passing the data to Tintype's original reporting code. You can reproduce the metrics without downloading media or model weights.

```bash
python3 scripts/report-archive.py docs/results/confirm7-scores.tsv.gz \
  --frozen docs/manifests/confirm7.frozen.tsv.gz --pooled confirm7

python3 scripts/report-archive.py docs/results/confirm4-scores.tsv.gz \
  --frozen docs/manifests/confirm4.frozen.tsv.gz --pooled confirm4

python3 scripts/report-archive.py docs/results/confirm7-scores.tsv.gz \
  --frozen docs/manifests/confirm7.frozen.tsv.gz \
  --dataset sofake-ood-fresh6 --jpeg-only --systems tintype commfor swinv2

python3 scripts/report-archive.py docs/results/frontier-jpeg-stress-scores.tsv.gz
```

Add `--at-rate-of tintype` to the JPEG command for the 18-false-positive comparison. To score the original media, fetch the relevant `tintype data` plan, verify it with `tintype verify-set`, and run `tintype evaluate <suite> --baselines`. The evaluation commands run on macOS. The archived score report runs wherever Tintype's `report` command is available.

To rerun the JPEG stress experiment, fetch the `benchmarks` and `baselines` plans, then use the public photo manifest at `data/vstore/wasitai/images.tsv`. Run the commands from the checkout root after installing FFmpeg. The first line fixes the sample at the same 745 paired inputs used in the table.

```bash
awk -F '\t' '$1 != "images/i90bff65232d9.jpg" && $1 != "images/i66dfe571bf71.jpg"' \
  data/vstore/wasitai/images.tsv > data/vstore/wasitai/paired-images.tsv

python3 scripts/stress-photo.py data/vstore/wasitai/paired-images.tsv /tmp/tintype-jpeg-stress

tintype score --device cpu --weights models/tintype.safetensors \
  --manifest /tmp/tintype-jpeg-stress/original.tsv --manifest /tmp/tintype-jpeg-stress/jpeg.tsv \
  --manifest /tmp/tintype-jpeg-stress/jpeg-960.tsv --out /tmp/tintype-jpeg-stress/tintype.tsv

tintype score --device cpu --reference commfor --weights baselines/commfor-384.safetensors \
  --manifest /tmp/tintype-jpeg-stress/original.tsv --manifest /tmp/tintype-jpeg-stress/jpeg.tsv \
  --manifest /tmp/tintype-jpeg-stress/jpeg-960.tsv --out /tmp/tintype-jpeg-stress/commfor.tsv

tintype report --markdown --system Tintype=/tmp/tintype-jpeg-stress/tintype.tsv \
  --system 'Community Forensics=/tmp/tintype-jpeg-stress/commfor.tsv'
```

## AI Detector Arena v0.1

We ran Tintype v0.1.0 and the Community Forensics 384-pixel reference on all
**2,038 images** in the pinned Arena release, then scored a same-dimension
JPEG copy of each image. Both systems scored all original and transformed
files. The externally curated benchmark has 1,018 generated and 1,020
authentic images. Tintype used Apple GPU fp16 and the reference used fp32.
A CPU repeat reproduced all four recall-at-1%-FPR values shown.

| Inputs | Tintype recall at 1% FPR | Community Forensics recall at 1% FPR |
| --- | ---: | ---: |
| Original | **88.5%** | 46.3% |
| Both classes re-encoded to JPEG | **73.1%** | 23.7% |

At the shipped threshold on the normalized set, Tintype found 69.5% of
generated images and flagged 7 of 1,020 authentic images. Community
Forensics found 23.9% and flagged 12 authentic images. Seven exact duplicate
authentic inputs appeared in the archive. Removing the repeated rows
preserved the displayed recall values at 1% FPR.

[Full result tables](results/arena-v0.1-gpu/report.md) ·
[Per-file scores](results/arena-v0.1-gpu/scores.tsv.gz) ·
[Input hashes](results/arena-v0.1-gpu/inputs.tsv.gz) ·
[Run parameters](results/arena-v0.1-gpu/run.json) ·
[Metadata paths absent from the ZIP](results/arena-v0.1-gpu/errors.tsv)

[CPU repeat tables](results/arena-v0.1-cpu/report.md) ·
[CPU scores](results/arena-v0.1-cpu/scores.tsv.gz) ·
[CPU run parameters](results/arena-v0.1-cpu/run.json). Both runs used the
same input hashes and had no decode or transform failures.

The Arena runner evaluates the externally curated dataset with Tintype's released photo checkpoint. Its original and normalized tables use the same image IDs, and optional Community Forensics scores use those same inputs. The runner verifies the published archive SHA-256 and uses the label metadata inside it. If the archive omits the CSV, it retrieves a fixed GitHub revision and verifies the Git blob ID. The released photo weights, model spec and optional Community Forensics checkpoint are verified before scoring. [Arena release record](https://github.com/AI-Detect-Arena/benchmark-dataset/blob/3014b54eec9d44c7ef60b4c7e0cda6aeb4735d5b/RELEASE_v0.1.md).

```bash
cargo build --release --locked --bin tintype
python3 scripts/evaluate-arena.py arena-results --tintype target/release/tintype
# On macOS, include the pinned Community Forensics reference.
python3 scripts/evaluate-arena.py arena-head-to-head --tintype target/release/tintype --with-commfor
```

The runner requires Python 3.9 or later, FFmpeg and curl. `--archive` accepts a previously downloaded copy of the pinned ZIP. `--commfor` accepts a local copy of the pinned baseline checkpoint on macOS. `report.md` contains the metrics, `run.json` records hashes, counts, and encoder version, `inputs.tsv` pairs the original and JPEG hashes, `errors.tsv` lists exclusions, and `scores.tsv.gz` contains the per-file scores. The runner drops an input from both variants if either detector could not score it in either variant. `scripts/report-archive.py` can regenerate the tables from `scores.tsv.gz`.

The originals have a format imbalance between labels. The runner counts formats from file signatures, including JPEG data stored under `.png` names. It re-encodes both classes to 4:2:0 JPEG at FFmpeg MJPEG q4 with stored dimensions and orientation, and with source metadata removed. That controls the output format and encoder. Earlier compression, image sources, and content differences remain. The public repository CSV lists 1,032 authentic images while the release lists 1,020. The runner reports the actual archive membership and any missing metadata paths. Results from this runner are author-run comparisons and the official leaderboard uses its own submissions.

### GPTTrace comparison on the same Arena images

We added the 448-pixel GPTTrace PE-Core-L flag-pec-v3 ONNX checkpoint to
the same archived input pairs. The run verified the pinned FP32 model,
config, Arena ZIP, and each original and normalized JPEG hash. It scored
all 4,076 file variants with the model card's center crop, resized full
frame and shipped offset. ONNX Runtime used the Core ML provider with CPU
fallback. A CPU spot check on 24 image IDs sampled by hash within class,
with both variants for each ID, differed by at most 0.000054 in raw logits.
[GPTTrace model and preprocessing](https://huggingface.co/nakas/gpttrace-img-flag-pec-v3-onnx/blob/197d072c7ff2e41d53f5e23b57e16c9e0d9ba042/README.md).

| Input | Tintype recall at 1% FPR | GPTTrace recall at 1% FPR | Community Forensics recall at 1% FPR |
| --- | ---: | ---: | ---: |
| Original | 88.5% | 82.0% | 46.3% |
| Same-dimension JPEG | 73.1% | 76.7% | 23.7% |

Tintype led on the originals, and GPTTrace led after JPEG normalization.
On normalized inputs at the shipped thresholds, Tintype found 69.5% and
flagged 7 of 1,020 authentic images. GPTTrace found 79.1% and flagged 13
authentic images. The 1% false-positive comparison chooses each threshold
on the cohort being measured. Prior compression and source differences
can persist after re-encoding.

[Full tables](results/arena-v0.1-gpttrace/report.md) ·
[Matched score archive](results/arena-v0.1-gpttrace/scores.tsv.gz) ·
[Raw GPTTrace logits](results/arena-v0.1-gpttrace/gpttrace-logits.tsv.gz) ·
[Run parameters](results/arena-v0.1-gpttrace/run.json) ·
[CPU comparison](results/arena-v0.1-gpttrace/mixed-provider-parity.tsv).

With Python 3.12, FFmpeg 9.0.2, NumPy 2.5.4, Pillow 13.0.0 and
ONNX Runtime 1.32.0, run the pinned comparison. The `--coreml` flag
enables Core ML on supported macOS builds. Omit it for a CPU run.

```sh
python3 scripts/score-gpttrace-arena.py arena-gpttrace --coreml
python3 scripts/report-archive.py arena-gpttrace/scores.tsv.gz --tintype tintype
```

## External comparisons

### CVPR 2026 NTIRE Hard Validation

We scored all 2,500 JPEGs in the challenge organizers' hard validation archive using Tintype's default photo checkpoint and the Community Forensics 384-pixel reference on macOS 27.2 CPU. Both systems scored every file. The clean and distorted cohorts each contain 625 generated images and 625 authentic images. The organizers applied transformations including downscaling, JPEG recompression, noise, crops, and watermark attacks to the distorted cohort. The public labels specify the distortion operations. [Validation archive](https://huggingface.co/datasets/deepfakesMSU/NTIRE-RobustAIGenDetection-val/tree/e067bdd40b973d7daff741de20b86bf551760d3f) · [Challenge protocol](https://arxiv.org/abs/2604.11487).

| Hard-validation cohort | Tintype recall at 1% FPR | Community Forensics recall at 1% FPR | Tintype AUROC | Community Forensics AUROC |
|---|---:|---:|---:|---:|
| Clean | 51.8% | 47.0% | 0.938 | 0.880 |
| Distorted | 5.4% | 3.8% | 0.734 | 0.641 |

At the shipped threshold, Tintype flagged 103 of 625 authentic distorted images (16.48%) and found 43.5% of the generated distorted images. Community Forensics flagged 74 of 625 (11.84%) and found 23.4%. At Tintype's 103-false-positive budget Community Forensics found 32.5%. Strong distortions substantially lowered recall at a low false-positive budget for both systems. This is an author-run comparison on the released validation split. [Full tables](results/ntire-2026-hard-validation/report.md) · [Per-file scores](results/ntire-2026-hard-validation/scores.tsv.gz).

To reproduce the hard-validation run on macOS, download the two files from the pinned revision and use the baseline checkpoint from `tintype data baselines`. The runner checks both dataset digests and all model digests, then requires one score from each detector for every labeled image.

```bash
ntire_hard=https://huggingface.co/datasets/deepfakesMSU/NTIRE-RobustAIGenDetection-val/resolve/e067bdd40b973d7daff741de20b86bf551760d3f
curl -fLO "$ntire_hard/val_images_hard.zip"
curl -fLO "$ntire_hard/val_hard_labels.csv"
python3 scripts/evaluate-ntire.py val_images_hard.zip val_hard_labels.csv ntire-hard \
  --tintype target/release/tintype --commfor baselines/commfor-384.safetensors
```

### CVPR 2026 NTIRE Public Test

The same two checkpoints scored all 2,500 JPEGs on the organizers' public test split after the challenge ended. Each cohort has 650 generated and 600 authentic images. Both released splits contain unique image bytes and share no exact image duplicates. [Public test archive and labels](https://huggingface.co/datasets/deepfakesMSU/NTIRE-RobustAIGenDetection-test-public/tree/7446d42c0ce38305cedc12e1a8ae7925ce2133dd).

| Public-test cohort | Tintype recall at 1% FPR | Community Forensics recall at 1% FPR | Tintype AUROC | Community Forensics AUROC |
|---|---:|---:|---:|---:|
| Clean | 22.9% | 15.4% | 0.870 | 0.773 |
| Distorted | 2.2% | 0.8% | 0.636 | 0.577 |

At the shipped threshold on distorted files, Tintype found 44.8% of generated images and flagged 155 of 600 authentic images (25.83%). Community Forensics found 22.8% and flagged 111 of 600 (18.50%). At Tintype's false-positive count, Community Forensics found 34.0%. [Full tables](results/ntire-2026-public-test/report.md) · [Per-file scores](results/ntire-2026-public-test/scores.tsv.gz).

The challenge's nine reported entrants scored between 0.9132 and 0.9978 AUROC on clean public images and between 0.8302 and 0.9738 on distorted public images. Tintype's default score fell below that range on both cohorts. The official ranking uses both the public and hidden split. These after-challenge results cover the public split. [Challenge results, Table 3](https://arxiv.org/html/2604.11487v1#S4).

```bash
ntire_test=https://huggingface.co/datasets/deepfakesMSU/NTIRE-RobustAIGenDetection-test-public/resolve/7446d42c0ce38305cedc12e1a8ae7925ce2133dd
curl -fLO "$ntire_test/test_images.zip"
curl -fLO "$ntire_test/test_labels.csv"
python3 scripts/evaluate-ntire.py test_images.zip test_labels.csv ntire-test \
  --split public-test --tintype target/release/tintype --commfor baselines/commfor-384.safetensors
```

### NTIRE threshold transfer and artifact diagnostics

We set each Tintype cutoff using only authentic hard-validation scores. We sorted the scores from highest to lowest and took the first score outside the 1% false-positive allowance. An image flags only when its score strictly exceeds that cutoff. The table applies those cutoffs unchanged to the corresponding public-test cohort. Each calibration cohort flags 0.96% of its authentic images.

| Calibration and public-test cohort | Cutoff | Public-test recall | Public-test false positives |
|---|---:|---:|---:|
| Clean | 0.941085 | 218 / 650 (33.54%) | 10 / 600 (1.67%) |
| Distorted | 0.996498 | 31 / 650 (4.77%) | 13 / 600 (2.17%) |
| Pooled | 0.994595 | 102 / 1,300 (7.85%) | 19 / 1,200 (1.58%) |

The clean cutoff applied to public distorted files flags 63 of 600 authentic images (10.50%). A stricter cutoff reduces false alarms, but the distorted score ranking remains weak. At the threshold chosen directly on the public distorted authentic scores, 14 of 650 generated images score above the 1% false-positive operating point. This public-cohort operating point describes ranking performance on that cohort.

Of the 155 public-test false positives on distorted inputs at the shipped cutoff, 110 occur in 182 authentic images that include JPEG AI, Cheng2020 learned compression or WMForger. That is 60.4%, compared with 45 of 418 (10.8%) for the other authentic distorted files. The same grouping shows a smaller gap in hard validation, where only JPEG AI appears among those three operations. Operations can co-occur, and the grouping is descriptive. Short-side resolution was a weaker separator on public distorted files. Tintype flagged 82 of 291 authentic files below 512 px (28.2%) and 73 of 309 at or above 512 px (23.6%). All 5,000 final files have identical JPEG quantization tables and 4:2:0 subsampling. Those metadata fields cannot identify this prior processing.

We also disabled early exit on hard validation, so the model scored all six views per image. Clean recall at 1% FPR rose from 51.8% to 60.5%, and clean AUROC rose from 0.938 to 0.955. On distorted files recall rose from 5.4% to 7.5%, and AUROC rose from 0.734 to 0.739. At the shipped cutoff the all-views profile found 263 generated distorted images and flagged 106 authentic ones, compared with 272 and 103 using early exit. The CPU run averaged 14.4 images/s with all views. [All-views scores](results/ntire-2026-hard-validation/all-views-scores.tsv.gz).

The shipped cutoff stays unchanged. The distorted results point to ranking robustness and calibration across processing histories. [Diagnostic counts](results/ntire-2026-hard-validation/transfer-diagnostics.txt).

With Pillow installed, reproduce the diagnostic counts from the four pinned organizer files and the archived detector scores.

```bash
python3 scripts/diagnose-ntire.py /path/to/ntire-archives
```

`python3 scripts/diagnose-ntire.py /path/to/ntire-archives --operations`
prints the same kind of counts for every operation and severity in the
labels. It reads only the two label files and the archived scores, so it
needs neither Pillow nor the image archives.

### Content-matched generated video clips

Davodi, Barrington, Farid and Cooper released 24 scene themes with a Pixabay source and synthetic variants from Seedance Pro 1, Veo 3.1, Sora 2 and Kling 2.6. We recovered the exact listed authentic rendition for 20 themes. The exact files for themes 6, 7 and 16 were unavailable at the tested CDN paths, and theme 24's page and filename list different asset IDs. All variants of those four themes are outside the comparison. The release includes Kling alongside the three generators used in the human study. [Dataset and normalization protocol](https://zenodo.org/records/19463142) · [CVPR workshop paper](https://openaccess.thecvf.com/content/CVPR2026W/APAI/html/Davodi_Perceptual_Judgments_of_Video_Authenticity_An_Examination_of_Viewing_Duration_CVPRW_2026_paper.html).

Each input uses the first four seconds, muted, encoded at 800 pixels wide with aspect ratio preserved, 24 fps, H.264 CRF 18, 4:2:0 and BT.709 tags. All 100 normalized clips have 96 frames. Aspect ratio preservation produced 800 × 452 for Seedance and 800 × 450 for the other clips. Source frame cadence, encoder and compression history can still differ. Tintype uses its shipped video profile on Apple GPU fp16. We ran the published WaveRep DINOv2 G4 checkpoint through Tintype's reference path on Apple GPU fp32. It scores the first 64 consecutive frames using a 504-pixel center crop with zero padding. Both detectors flag scores above 0.5 at their default operating point. With 20 authentic inputs, one false alarm changes the observed rate by 5 percentage points. [WaveRep checkpoint and method](https://github.com/grip-unina/WaveRep-SyntheticVideoDetection).

| Detector | Generated clips flagged | Authentic clips flagged | Pooled AUROC | Generated clips scoring above their authentic counterpart |
| --- | ---: | ---: | ---: | ---: |
| Tintype 0.1.0 | 72/80 | 3/20 | 0.9244 | 74/80 |
| WaveRep DINOv2 G4 | 34/80 | 4/20 | 0.6925 | 60/80 |

Neither detector tied any matched pair. With cutoffs selected on this cohort to flag 3 of 20 authentic clips for each detector, Tintype flagged 73 of 80 generated clips and WaveRep flagged 34 of 80. This describes the measured cohort at an equal observed false-positive count. The cutoffs were selected and evaluated on the same clips.

| Generator | Tintype flagged | WaveRep flagged | Tintype ranked above authentic | WaveRep ranked above authentic |
| --- | ---: | ---: | ---: | ---: |
| Kling 2.6 | 20/20 | 11/20 | 20/20 | 18/20 |
| Seedance Pro 1 | 17/20 | 8/20 | 18/20 | 15/20 |
| Sora 2 | 16/20 | 4/20 | 17/20 | 11/20 |
| Veo 3.1 | 19/20 | 11/20 | 19/20 | 16/20 |

Tintype's authors ran this exploratory evaluation on external media. The cohort represents the themes whose exact authentic rendition was recoverable. Each authentic clip appears in four matched pairs, so those pairs are correlated. WaveRep was run in two batches with identical scoring code and checkpoint. The run record lists the binary source for each batch.

[Exact source URLs and hashes](results/davodi-2026-video/inputs.tsv) · [Run parameters](results/davodi-2026-video/run.json) · [Clip scores](results/davodi-2026-video/scores.tsv.gz) · [Frame logits](results/davodi-2026-video/frame-scores.tsv.gz) · [Metric report](results/davodi-2026-video/report.txt).

To recreate the normalized inputs, download the Zenodo ZIP and the 20 authentic clips listed in `inputs.tsv` to an `authentic` directory. Name each authentic clip by its scene number with two digits and an `.mp4` extension. The script verifies the archive and original file SHA-256 values. Use the recorded FFmpeg version for byte-identical transforms.

```bash
python3 scripts/prepare-davodi-video.py ForZenodo.zip authentic davodi-clips \
  --source-record docs/results/davodi-2026-video/inputs.tsv

tintype score-video --device gpu --weights models/tintype-video.safetensors \
  --manifest davodi-clips/manifest.tsv --out davodi-clips/tintype.tsv
tintype score-video --device gpu-f32 --reference waverep --weights baselines/waverep-g4.ckpt \
  --manifest davodi-clips/manifest.tsv --out davodi-clips/waverep.tsv
python3 scripts/report-davodi-video.py davodi-clips/inputs.tsv davodi-clips/tintype.tsv \
  davodi-clips/scores.tsv.gz --waverep davodi-clips/waverep.tsv \
  --frame-rows davodi-clips/frame-scores.tsv.gz
```

### Comparisons still open

- B-Free's authors report strong results with same-format controls and post-processing. Pangram Image 1.0 is a commercial comparison for the newest image generators. Establishing a broader image rank requires scores on identical inputs and at matched operating points. [B-Free](https://arxiv.org/abs/2412.17671) · [Pangram Image 1.0](https://www.pangram.com/blog/introducing-pangram-image-1-0).
- DF26 uses controlled-access, format-normalized talking-head footage from recent video generators. VidAudit provides controls for duration, codec and real-source confounds in video benchmarks. A wider video comparison should include current systems such as NVIDIA Synthetic Video Detector and Pangram's video detector, which uses Image Flash 1.0, on the same held-out clips. [DF26](https://huggingface.co/datasets/DF26/DF26) · [VidAudit](https://arxiv.org/abs/2606.31004) · [NVIDIA model card](https://build.nvidia.com/nvidia/synthetic-video-detector/modelcard) · [Pangram video detector](https://www.pangram.com/video-detector).
