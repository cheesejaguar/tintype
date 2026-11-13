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

## External comparisons

- The CVPR 2026 NTIRE challenge measures clean and distorted images separately. Its labeled hard validation set can support a reproducible distortion benchmark. An official challenge ranking requires scoring the challenge test set with its evaluator. [Validation archive](https://huggingface.co/datasets/deepfakesMSU/NTIRE-RobustAIGenDetection-val/tree/main) · [Challenge protocol and results](https://arxiv.org/abs/2604.11487).
- AI Detector Arena v0.1 lists 2,038 images across 17 generators. The full release has a published SHA-256. The GitHub sample has only one generated image per category, all from FLUX.2 Flex, and 30 authentic images. At that size a single false positive moves the rate by 3.3 percentage points. [Full benchmark release](https://github.com/AI-Detect-Arena/benchmark-dataset/blob/main/RELEASE_v0.1.md).
- B-Free's authors report strong results with same-format controls and post-processing. Pangram Image 1.0 is a commercial comparison for the newest image generators. Establishing a broader image rank requires scores on identical inputs and at matched operating points. [B-Free](https://arxiv.org/abs/2412.17671) · [Pangram Image 1.0](https://www.pangram.com/blog/introducing-pangram-image-1-0).
- DF26 uses controlled-access, format-normalized talking-head footage from recent video generators. VidAudit provides controls for duration, codec and real-source confounds in video benchmarks. A wider video comparison should include current systems such as NVIDIA Synthetic Video Detector and Pangram's video detector, which uses Image Flash 1.0, on the same held-out clips. [DF26](https://huggingface.co/datasets/DF26/DF26) · [VidAudit](https://arxiv.org/abs/2606.31004) · [NVIDIA model card](https://build.nvidia.com/nvidia/synthetic-video-detector/modelcard) · [Pangram video detector](https://www.pangram.com/video-detector).
