# Benchmark

Veracity against three open detectors, on data none of them trained on.
Everything was measured on one machine (Apple M6, 32 GB, macOS 27.2) in
November 2026, and every system – Veracity and the reference detectors – was
run by the same Rust binary.

## Summary

Results on the three blind batches – data downloaded only after the
configuration under test and the protocol had been written down, and scored
once. Each system is at its shipped threshold.

| | Veracity | Reference |
|---|---|---|
| **Photos**, batch 5: 7,079 AI and 6,278 real | default profile: precision 0.980, recall 0.875 | Community Forensics: 0.963, 0.762 |
| | | SwinV2 detector: 0.838, 0.677 |
| **Photos**, batch 3: 6,172 AI and 4,541 real | `--strict` (the default at the time): 0.997, 0.865 | Community Forensics: 0.970, 0.637 |
| | `--sensitive`: 0.955, 0.972 | SwinV2 detector: 0.896, 0.918 |
| **Video**, batch 4: 1,156 AI and 900 real | 0.997, 0.941 | WaveRep: 0.983, 0.912 |
| | with the second opinion: 0.997, 0.960 | |
| **Photo throughput**, GPU, end to end | 263 images/s; 455 with `--fast` | Community Forensics: 61–67 as released, 100 through this engine; SwinV2 detector: 8.6 and 30 |
| **Photo throughput**, CPU | 38.8 images/s; 52.6 with `--fast` | Community Forensics: 25.4 as released, 35.0 through this engine |
| **Video**, time per clip | 0.09 s; 0.26 s with the second opinion | WaveRep: 4.0 s as released, 1.08 s through this engine at 16 bits |

Pooled over each blind batch, the default photo profile is ahead of both
reference photo detectors on precision and on recall, and the video
checkpoint is ahead of WaveRep on both.

Where Veracity is not ahead, or the comparison needs a qualifier:

- **Older generators, set by set.** Batch 5 is mostly AIGIBench archives of
  GANs, GLIDE and identity-preserving adapters. On nine of its thirteen sets
  Community Forensics finds more of the AI images than the default profile
  (it was trained on thousands of such generators and reaches 0.98–1.00 on
  the GAN sets, where Veracity reaches 0.73–0.99), and on PhotoMaker both
  reference detectors do. Veracity's precision is the higher on eleven of the
  thirteen. The pooled result is carried by So-Fake-OOD – current commercial
  generators in social-media photos, a third of the batch's AI images – where
  Community Forensics finds 0.41 and Veracity 0.86.
- **The default threshold was moved after batch 3.** There the default was
  what is now `--strict`: more precise than the SwinV2 detector on every set
  but with lower recall on three of five (pooled 0.865 against 0.918). The
  1.5% default was then chosen on development data and tested on batch 5.
  Re-scored on batch 3, which it has seen, it gives 0.988 / 0.932.
- **Early exit costs a little ranking quality.** Verdicts barely change, but
  scores of clear images come from fewer views, so AUROC and recall at 1% FPR
  drop slightly (0.992 to 0.986 on batch 3). `--all-views` restores them.
- **Video, one set.** On AIGVDBench the video checkpoint alone has higher
  precision than WaveRep (0.998 against 0.982) and marginally lower recall
  (0.966 against 0.970), while flagging 1 real clip to WaveRep's 9. Given
  WaveRep's false-positive rate it finds 0.988. With the second opinion it is
  ahead on both at shipped thresholds on all three sets.
- **The second opinion is WaveRep's own network.** That mode shows the two
  models are complementary and that a cascade is both more accurate and
  faster than WaveRep alone; it is not a win by Veracity's weights alone. The
  rows labelled "Veracity" use only Veracity's weights.
- **Speed against a half-precision Community Forensics.** Run through this
  engine at 16 bits – a configuration its authors do not ship – Community
  Forensics does 253 images/s on the GPU, close to the default profile's 263.
  Looking at all six views of every image (`--all-views`) Veracity does 180
  on the GPU and 19.5 on the CPU, below Community Forensics in the same
  engine (253 and 35.0).
- **The first blind video result was not a win.** The phase-3 weights had
  higher recall than WaveRep and lower precision (0.987 / 0.949 against
  0.995 / 0.924). That result is kept below; it is what led to the video
  checkpoint and the second blind batch.
- **The newest commercial video generators.** On the Frontier 2026-10 video
  set (development data) WaveRep finds 0.921 of the AI clips while flagging 7%
  of the real ones; the video checkpoint finds 0.762 with none flagged, 0.881
  at 2% with the second opinion.
- **Social-media copies** of images are hard for every detector here; see
  the development sets.

## Reference detectors

| System | What it is | Run as |
|---|---|---|
| **Community Forensics** (CVPR 2025) | ViT-S/16 at 384 px, 21.8 M parameters. Best open detector in the independent out-of-the-box study arXiv 2602.07814. | Official weights `OwensLab/commfor-model-384`, official transform (resize 440, centre crop 384), threshold 0.5, 32-bit floats. |
| **SwinV2 detector** | `haywoodsloan/ai-image-detector-deploy`, SwinV2-L at 256 px, about 200 M parameters. Best open entry on the WasItAIGenerated frontier leaderboard. | Its own preprocessing (bicubic squash to 256 px), threshold 0.5, 32-bit floats. |
| **WaveRep** (NeurIPS 2025) | DINOv2 ViT-B/14 on 504 px crops of the first 64 frames. Strongest open video detector that ships weights. | The authors' demo pipeline: mean frame logit above 0. Accuracy runs used 16-bit floats to save time; on 147 clips scored both ways no verdict changed. |

The three networks and their preprocessing are re-implemented in Rust
(`veracity score --reference commfor|swinv2`, `veracity score-video
--reference waverep`) and load the authors' released weights. Each port was
checked against the authors' PyTorch pipeline on the same files:

| Port | Compared on | Decisions that differ | Precision, recall (PyTorch → Rust) |
|---|---|---|---|
| Community Forensics | 44,336 images (every development set) | 62 (0.14%) | 0.9744 → 0.9750, 0.5892 → 0.5885 |
| SwinV2 detector | 35,336 images | 93 (0.26%) | differences below 0.001 |
| WaveRep | 1,902 clips | 1 (0.05%) | 0.9878 → 0.9878, 0.8420 → 0.8411 |

The residual image differences come from the JPEG decoder (`zune-jpeg` here,
libjpeg-turbo under Pillow). Matching the references took reproducing
Pillow's fixed-point resampling, torchvision's round-half-even centre crop
and OpenCV's table-driven YCbCr conversion exactly.

**A correction to an earlier draft of this report.** Two long PyTorch runs of
WaveRep on GenVideo-Val had gone wrong partway through (from roughly the 600th
clip on, nearly every real clip scored positive), which made WaveRep appear to flag 55–57%
of real MSR-VTT clips. A sample of those clips re-run in a fresh PyTorch
process reproduces the Rust port's scores, and the port puts the figure at
1.3–1.7%. Every WaveRep number below comes from the Rust port.

## How the evaluation was staged

Thresholded precision and recall are easy to flatter by tuning on the test
set, so the data is split by what it was allowed to influence
([DATA.md](DATA.md) has the sources and the full history):

1. **Development sets** (plans `benchmarks`, `confirm1`, `confirm2`) were
   scored while the model, its views and its calibration were still
   changing. Their real items are part of the calibration set. Read their
   thresholded numbers as optimistic.
2. **Blind photo batch** (`confirm3`). The model, views, thresholds and the
   protocol were written down ([PREREGISTRATION.md](PREREGISTRATION.md))
   before any of this data was downloaded. It was scored once.
3. **First blind video result** (`confirm3`, the same protocol). The phase-3
   weights came out level with WaveRep rather than ahead. This batch then
   became development data for the video checkpoint.
4. **Blind video batch** (`confirm4`). The video checkpoint, its threshold,
   the second-opinion rule and the protocol were written down (same document)
   before this data was downloaded. It was scored once.
5. **Second blind photo batch** (`confirm5`). The photo weights did not
   change; the default threshold moved from a 0.5% to a 1.5% target and early
   exit was added, both fitted on development data and written down (same
   document) before this data was downloaded. It was scored once.

In every table: precision and recall at each system's shipped threshold, with
95% bootstrap intervals; "Recall at 1% FPR" sets each system's threshold on
that set's own real items, which removes the choice of threshold.

## Photos

### Blind batch 5: the default profile

Three more shards of So-Fake-OOD and 400 images per class from each of the
twelve AIGIBench test archives not used before (the five face-swap archives
are left out: local edits are out of scope).

**Pooled** (7,079 AI, 6,278 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.963 (0.958–0.968) | 0.762 (0.752–0.772) | 0.851 | 3.30% | 0.965 | 0.621 |
| SwinV2 detector | 0.838 (0.829–0.847) | 0.677 (0.666–0.689) | 0.749 | 14.72% | 0.845 | 0.250 |
| Veracity | 0.980 (0.977–0.983) | 0.875 (0.867–0.882) | 0.924 | 2.02% | 0.981 | 0.800 |
| Veracity (strict) | 0.995 (0.993–0.996) | 0.713 (0.703–0.723) | 0.831 | 0.43% | 0.973 | 0.790 |
| Veracity (fast) | 0.995 (0.993–0.997) | 0.658 (0.646–0.668) | 0.792 | 0.37% | 0.981 | 0.769 |
| Veracity (sensitive) | 0.940 (0.935–0.946) | 0.957 (0.952–0.962) | 0.949 | 6.85% | 0.985 | 0.794 |

Precision / recall by set:

| Set | Community Forensics | SwinV2 detector | Veracity | `--strict` | `--sensitive` | `--fast` |
|---|---|---|---|---|---|---|
| So-Fake-OOD, batch 4 (2,279 AI, 1,479 real) | 0.983 / 0.412 | 0.944 / 0.812 | **0.989 / 0.858** | 0.996 / 0.733 | 0.971 / 0.931 | 0.994 / 0.616 |
| AIGIBench GLIDE (400 AI, 400 real) | 0.968 / 0.998 | 0.770 / 0.520 | **0.982 / 0.943** | 0.986 / 0.897 | 0.933 / 0.973 | 0.991 / 0.858 |
| AIGIBench BLIP-Diffusion (400 AI, 400 real) | 0.955 / 1.000 | 0.930 / 0.998 | **0.969 / 1.000** | 1.000 / 0.998 | 0.917 / 1.000 | 1.000 / 0.998 |
| AIGIBench IP-Adapter (400 AI, 400 real) | 0.955 / 0.905 | 0.793 / 0.660 | **0.968 / 0.843** | 0.995 / 0.540 | 0.919 / 0.968 | 0.990 / 0.258 |
| AIGIBench InstantID (400 AI, 399 real) | 0.945 / 0.682 | 0.823 / 0.968 | **0.980 / 0.998** | 1.000 / 0.955 | 0.917 / 1.000 | 1.000 / 0.797 |
| AIGIBench Infinite-ID (400 AI, 400 real) | 0.966 / 0.915 | 0.822 / 0.795 | **0.993 / 0.995** | 0.997 / 0.960 | 0.926 / 1.000 | 1.000 / 0.840 |
| AIGIBench PhotoMaker (400 AI, 400 real) | 0.939 / 0.657 | 0.748 / 0.570 | **0.972 / 0.517** | 1.000 / 0.260 | 0.920 / 0.777 | 1.000 / 0.275 |
| AIGIBench ProGAN (400 AI, 400 real) | 0.895 / 1.000 | 0.629 / 0.055 | **0.943 / 0.960** | 0.979 / 0.940 | 0.904 / 0.993 | 0.979 / 0.932 |
| AIGIBench R3GAN (400 AI, 400 real) | 0.966 / 1.000 | 0.758 / 0.580 | **0.975 / 0.985** | 0.995 / 0.953 | 0.915 / 0.998 | 0.997 / 0.917 |
| AIGIBench StyleGAN3 (400 AI, 400 real) | 0.990 / 0.980 | 0.663 / 0.300 | **0.980 / 0.730** | 1.000 / 0.412 | 0.960 / 0.950 | 1.000 / 0.370 |
| AIGIBench StyleGAN-XL (400 AI, 400 real) | 0.959 / 1.000 | 0.815 / 0.795 | **0.973 / 0.892** | 0.996 / 0.573 | 0.928 / 0.995 | 1.000 / 0.590 |
| AIGIBench StyleSwim (400 AI, 400 real) | 0.969 / 1.000 | 0.747 / 0.552 | **0.979 / 0.938** | 0.991 / 0.555 | 0.930 / 0.998 | 0.993 / 0.700 |
| AIGIBench WFIR (400 AI, 400 real) | 0.998 / 1.000 | 0.637 / 0.570 | **0.997 / 0.792** | 1.000 / 0.400 | 0.959 / 0.985 | 1.000 / 0.593 |

- Against the SwinV2 detector the default profile is ahead on both measures
  on twelve of the thirteen sets; on PhotoMaker the SwinV2 detector finds
  more (0.570 against 0.517) at much lower precision.
- Against Community Forensics it is ahead on both on three sets (So-Fake-OOD,
  InstantID, Infinite-ID), level on recall on BLIP-Diffusion, and behind on
  recall on the other nine: GLIDE, IP-Adapter, PhotoMaker and all six GAN
  sets. Its precision is the higher on eleven sets, the lower on StyleGAN3 and
  WFIR.
- `--sensitive` closes most of that recall gap (0.95–1.00 on the GAN sets) at
  4–10% false positives.

The images averaged 2.8 views each. Per-set tables with intervals, AUROC and
recall at 1% FPR are in [results/confirm5.md](results/confirm5.md).

### Blind batch 3: the strict and sensitive profiles

This batch was scored when the default threshold was the 0.5% target. The
rows labelled "Veracity" below are therefore what is now `--strict`, looking
at all six views of every image. The current default, re-scored on this batch
after the fact, gives precision 0.988 and recall 0.932 pooled
([results/confirm3_new_default.md](results/confirm3_new_default.md)).

**AIGIBench DALL·E 3** (1000 AI, 1000 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.958 (0.944–0.970) | 0.839 (0.817–0.862) | 0.894 | 3.70% | 0.980 | 0.651 |
| SwinV2 detector | 0.844 (0.822–0.863) | 0.992 (0.986–0.997) | 0.912 | 18.40% | 0.992 | 0.871 |
| Veracity | 0.997 (0.993–1.000) | 0.943 (0.929–0.957) | 0.969 | 0.30% | 0.997 | 0.965 |
| Veracity (fast) | 0.997 (0.993–1.000) | 0.925 (0.908–0.941) | 0.960 | 0.30% | 0.998 | 0.972 |
| Veracity (sensitive) | 0.939 (0.924–0.953) | 0.994 (0.989–0.998) | 0.966 | 6.50% | 0.997 | 0.965 |

**AIGIBench Imagen 3** (1000 AI, 999 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.903 (0.873–0.932) | 0.373 (0.345–0.403) | 0.528 | 4.00% | 0.928 | 0.075 |
| SwinV2 detector | 0.829 (0.806–0.852) | 0.897 (0.878–0.916) | 0.862 | 18.52% | 0.927 | 0.488 |
| Veracity | 0.999 (0.996–1.000) | 0.825 (0.800–0.850) | 0.904 | 0.10% | 0.994 | 0.916 |
| Veracity (fast) | 1.000 (1.000–1.000) | 0.671 (0.643–0.702) | 0.803 | 0.00% | 0.985 | 0.805 |
| Veracity (sensitive) | 0.925 (0.908–0.940) | 0.981 (0.972–0.989) | 0.952 | 8.01% | 0.994 | 0.916 |

**AIGIBench SD3** (1000 AI, 1000 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.967 (0.955–0.979) | 0.771 (0.744–0.797) | 0.858 | 2.60% | 0.976 | 0.510 |
| SwinV2 detector | 0.844 (0.822–0.865) | 0.981 (0.972–0.988) | 0.907 | 18.20% | 0.987 | 0.890 |
| Veracity | 0.993 (0.987–0.998) | 0.992 (0.986–0.997) | 0.992 | 0.70% | 1.000 | 0.996 |
| Veracity (fast) | 0.997 (0.993–1.000) | 0.957 (0.944–0.969) | 0.977 | 0.30% | 0.999 | 0.983 |
| Veracity (sensitive) | 0.943 (0.930–0.957) | 1.000 (1.000–1.000) | 0.971 | 6.00% | 1.000 | 0.996 |

**AIGIBench SDXL** (1000 AI, 1000 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.975 (0.965–0.983) | 0.995 (0.990–0.999) | 0.985 | 2.60% | 0.998 | 0.954 |
| SwinV2 detector | 0.846 (0.825–0.867) | 0.998 (0.995–1.000) | 0.916 | 18.20% | 0.995 | 0.887 |
| Veracity | 0.993 (0.987–0.998) | 1.000 (1.000–1.000) | 0.997 | 0.70% | 1.000 | 1.000 |
| Veracity (fast) | 0.997 (0.993–1.000) | 0.999 (0.997–1.000) | 0.998 | 0.30% | 1.000 | 0.999 |
| Veracity (sensitive) | 0.943 (0.930–0.957) | 1.000 (1.000–1.000) | 0.971 | 6.00% | 1.000 | 1.000 |

**So-Fake-OOD, batch 3** (2172 AI, 1542 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.982 (0.974–0.991) | 0.438 (0.416–0.460) | 0.606 | 1.10% | 0.910 | 0.421 |
| SwinV2 detector | 0.945 (0.935–0.956) | 0.827 (0.810–0.842) | 0.882 | 6.74% | 0.947 | 0.616 |
| Veracity | 0.997 (0.994–0.999) | 0.727 (0.708–0.745) | 0.841 | 0.32% | 0.984 | 0.807 |
| Veracity (fast) | 0.992 (0.987–0.997) | 0.608 (0.588–0.629) | 0.754 | 0.65% | 0.970 | 0.680 |
| Veracity (sensitive) | 0.962 (0.954–0.971) | 0.931 (0.920–0.941) | 0.946 | 5.12% | 0.984 | 0.807 |

**Pooled** (6172 AI, 4541 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.970 (0.965–0.976) | 0.637 (0.624–0.649) | 0.769 | 2.64% | 0.953 | 0.471 |
| SwinV2 detector | 0.896 (0.889–0.904) | 0.918 (0.911–0.924) | 0.907 | 14.42% | 0.956 | 0.679 |
| Veracity | 0.997 (0.996–0.998) | 0.865 (0.857–0.874) | 0.926 | 0.35% | 0.992 | 0.912 |
| Veracity (fast) | 0.997 (0.995–0.998) | 0.789 (0.780–0.799) | 0.881 | 0.35% | 0.986 | 0.857 |
| Veracity (sensitive) | 0.955 (0.949–0.960) | 0.972 (0.968–0.976) | 0.963 | 6.25% | 0.992 | 0.912 |

On all five sets and pooled, the default profile has higher precision and
higher recall than Community Forensics, and the sensitive profile has higher
precision and higher recall than the SwinV2 detector. Against the SwinV2
detector the default profile is the more precise on every set and finds fewer
AI images on three of the five (So-Fake-OOD, DALL·E 3, Imagen 3). The two-view
fast profile beats Community Forensics on both measures on every set and
pooled.

AIGIBench pairs SD3 and SDXL with the same 1,000 real photos; the pooled table
counts them once.

Recall by generator on So-Fake-OOD batch 3, each system at its shipped
threshold:

| Generator | Images | Community Forensics | SwinV2 detector | Veracity | Veracity (sensitive) |
|---|---|---|---|---|---|
| FLUX.2 | 70 | 0.09 | 0.89 | 0.90 | 0.99 |
| FLUX.1 pro | 200 | 0.19 | 0.46 | 0.32 | 0.61 |
| GPT-image-1.5 | 188 | 0.28 | 0.74 | 0.80 | 0.97 |
| GPT-image-2 | 95 | 0.20 | 0.68 | 0.58 | 0.94 |
| GPT-4o image | 195 | 0.53 | 0.99 | 0.80 | 0.97 |
| HiDream | 221 | 0.67 | 0.94 | 0.95 | 0.99 |
| Ideogram 2 | 89 | 0.36 | 0.73 | 0.72 | 0.98 |
| Ideogram 3 | 180 | 0.87 | 0.97 | 0.91 | 1.00 |
| Imagen 4 | 191 | 0.47 | 0.98 | 0.85 | 0.97 |
| Recraft v3 | 215 | 0.47 | 0.94 | 0.60 | 0.96 |
| Seedream 3.0 | 184 | 0.29 | 0.61 | 0.52 | 0.86 |
| Imagen 3 | 148 | 0.53 | 0.89 | 0.80 | 0.96 |
| Nano Banana | 76 | 0.59 | 0.91 | 0.59 | 0.92 |
| Nano Banana 2 | 60 | 0.27 | 0.73 | 0.75 | 1.00 |
| Seedream 4.5 | 60 | 0.20 | 0.87 | 0.98 | 1.00 |

| Real source | Images | Community Forensics | SwinV2 detector | Veracity | Veracity (sensitive) |
|---|---|---|---|---|---|
| Bluesky | 98 | 3.1% | 13.3% | 1.0% | 15.3% |
| Reddit | 1280 | 0.9% | 5.6% | 0.2% | 3.1% |
| Tumblr | 164 | 1.8% | 11.6% | 1.2% | 14.6% |

The second table is the share of real photos flagged.

### Blind batch 3, at each reference detector's own false-positive rate

Shipped thresholds differ, so a detector can lead on recall simply by flagging
more of everything. Here the reference detector keeps its shipped threshold
and Veracity is given the strictest threshold that flags no more real photos
than the reference did on that set (`veracity report --at-rate-of`). The
threshold is set on the set's own real photos, so this compares the models
rather than their operating points.

| Set | SwinV2 detector: real photos flagged | SwinV2 detector: precision / recall | Veracity at that rate: precision / recall |
|---|---|---|---|
| AIGIBench DALL·E 3 | 184 of 1000 (18.40%) | 0.844 / 0.992 | 0.844 / 0.997 |
| AIGIBench Imagen 3 | 185 of 999 (18.52%) | 0.829 / 0.897 | 0.844 / 0.995 |
| AIGIBench SD3 | 182 of 1000 (18.20%) | 0.844 / 0.981 | 0.846 / 1.000 |
| AIGIBench SDXL | 182 of 1000 (18.20%) | 0.846 / 0.998 | 0.846 / 1.000 |
| So-Fake-OOD, batch 3 | 104 of 1542 (6.74%) | 0.945 / 0.827 | 0.952 / 0.950 |

| Set | Community Forensics: real photos flagged | Community Forensics: precision / recall | Veracity at that rate: precision / recall | Veracity (fast) at that rate: precision / recall |
|---|---|---|---|---|
| AIGIBench DALL·E 3 | 37 of 1000 (3.70%) | 0.958 / 0.839 | 0.964 / 0.991 | 0.964 / 0.993 |
| AIGIBench Imagen 3 | 40 of 999 (4.00%) | 0.903 / 0.373 | 0.960 / 0.963 | 0.958 / 0.920 |
| AIGIBench SD3 | 26 of 1000 (2.60%) | 0.967 / 0.771 | 0.975 / 1.000 | 0.975 / 0.994 |
| AIGIBench SDXL | 26 of 1000 (2.60%) | 0.975 / 0.995 | 0.975 / 1.000 | 0.975 / 1.000 |
| So-Fake-OOD, batch 3 | 17 of 1542 (1.10%) | 0.982 / 0.438 | 0.991 / 0.829 | 0.989 / 0.692 |

At either reference detector's false-positive rate Veracity finds more AI
images on every set of this batch. The three sets where the SwinV2 detector
leads the strict profile on recall are a difference of threshold, not of
model.

### Development sets

Pooled results on the three earlier suites, with the released weights. The
thresholds were fitted with these sets' real photos in the calibration data.
Per-set tables are in [DEV_RESULTS.md](DEV_RESULTS.md). As for batch 3, the
rows labelled "Veracity" are the 0.5% profile, now `--strict`. The current
default on the same three suites (tables in `results/*_new_default.md`):

| Suite | Community Forensics | SwinV2 detector | Veracity, current default |
|---|---|---|---|
| Benchmark sets | 0.971 / 0.568 | 0.937 / 0.769 | 0.983 / 0.797 |
| Confirmation batch 1 | 0.984 / 0.620 | 0.924 / 0.834 | 0.979 / 0.823 |
| Confirmation batch 2 | 0.974 / 0.612 | 0.825 / 0.925 | 0.991 / 0.945 |

On confirmation batch 1 the current default is behind Community Forensics on
precision (0.979 against 0.984) and just behind the SwinV2 detector on recall
(0.823 against 0.834); AIGIBench SocialRF is the set responsible.

**Benchmark sets (plan `benchmarks`), pooled** (14635 AI, 11005 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.971 (0.967–0.974) | 0.568 (0.560–0.576) | 0.717 | 2.27% | 0.904 | 0.488 |
| SwinV2 detector | 0.937 (0.932–0.941) | 0.769 (0.763–0.776) | 0.845 | 6.89% | 0.927 | 0.530 |
| Veracity | 0.994 (0.992–0.995) | 0.634 (0.627–0.642) | 0.774 | 0.53% | 0.976 | 0.718 |
| Veracity (fast) | 0.994 (0.992–0.996) | 0.571 (0.563–0.579) | 0.726 | 0.45% | 0.966 | 0.651 |
| Veracity (sensitive) | 0.947 (0.944–0.951) | 0.918 (0.914–0.923) | 0.933 | 6.80% | 0.976 | 0.718 |

**Confirmation batch 1, pooled** (6232 AI, 3953 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.984 (0.981–0.988) | 0.620 (0.609–0.632) | 0.761 | 1.54% | 0.937 | 0.549 |
| SwinV2 detector | 0.924 (0.917–0.931) | 0.834 (0.824–0.843) | 0.876 | 10.80% | 0.932 | 0.548 |
| Veracity | 0.990 (0.987–0.993) | 0.684 (0.672–0.696) | 0.809 | 1.11% | 0.974 | 0.676 |
| Veracity (fast) | 0.989 (0.985–0.992) | 0.603 (0.590–0.615) | 0.749 | 1.06% | 0.962 | 0.583 |
| Veracity (sensitive) | 0.944 (0.938–0.949) | 0.925 (0.918–0.931) | 0.934 | 8.70% | 0.974 | 0.676 |

**Confirmation batch 2, pooled** (4531 AI, 3980 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.974 (0.968–0.980) | 0.612 (0.598–0.625) | 0.752 | 1.83% | 0.965 | 0.454 |
| SwinV2 detector | 0.825 (0.814–0.835) | 0.925 (0.917–0.932) | 0.872 | 22.39% | 0.943 | 0.644 |
| Veracity | 0.998 (0.996–0.999) | 0.882 (0.873–0.892) | 0.936 | 0.25% | 0.992 | 0.947 |
| Veracity (fast) | 0.997 (0.995–0.999) | 0.760 (0.747–0.773) | 0.863 | 0.25% | 0.985 | 0.892 |
| Veracity (sensitive) | 0.942 (0.935–0.948) | 0.973 (0.968–0.977) | 0.957 | 6.81% | 0.992 | 0.947 |

Places in the development sets where a reference detector is ahead at shipped
thresholds:

- AIGIBench CommunityAI: Community Forensics finds more AI images than the
  default profile (0.554 against 0.505) at 1.90% false positives against
  0.52%; the SwinV2 detector's precision is level with the sensitive
  profile's (0.920 against 0.918) at lower recall (0.881 against 0.929).
- AIGIBench SocialRF: the SwinV2 detector finds more than the sensitive
  profile (0.948 against 0.927) at lower precision (0.888 against 0.904).
- Synthbuster+ new images (AI only, so no precision): Community Forensics
  finds 0.788, the default profile 0.667, the sensitive profile 0.910. On the
  RAISE camera photos that go with this benchmark Community Forensics flags
  7.9%, the default profile 0.9%.
- Frontier 2026-10 images: the SwinV2 detector is slightly ahead of the
  default profile on recall (0.674 against 0.655) and well ahead of the
  sensitive profile on precision (0.951 against 0.872).
- Copies that went through Instagram, WhatsApp or a re-share: the SwinV2
  detector finds 38–44% at about 1% false positives; the default profile
  finds 12–17% with almost none, the sensitive profile 55–59% at 2–3%.

## Video

"Veracity" is the video checkpoint by itself. "Veracity + second opinion"
adds the optional stage: clips whose score falls in an uncertain band (13% of
the blind batch) are also scored by the WaveRep network and the two scores are
added.

### Blind batch

The tables below were scored once, when a handful of clips (HEVC tagged
`hev1`) were still decoded by an external tool. Re-scoring the whole batch
with the final build, which decodes everything through the system decoder,
reproduces every number in them.

**AIGVDBench, batch 3 (AccVideo, Gen-3, CausVid, RepVideo, IPOC)** (496 AI, 300 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.982 (0.970–0.992) | 0.970 (0.954–0.984) | 0.976 | 3.00% | 0.996 | 0.954 |
| Veracity | 0.998 (0.993–1.000) | 0.966 (0.949–0.981) | 0.982 | 0.33% | 0.999 | 0.984 |
| Veracity + second opinion | 0.998 (0.994–1.000) | 0.986 (0.974–0.994) | 0.992 | 0.33% | 1.000 | 0.986 |

**ComGenVid, batch 3 (Sora, Veo 3)** (300 AI, 300 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.976 (0.955–0.992) | 0.817 (0.773–0.861) | 0.889 | 2.00% | 0.986 | 0.790 |
| Veracity | 1.000 (1.000–1.000) | 0.903 (0.870–0.935) | 0.949 | 0.00% | 0.996 | 0.927 |
| Veracity + second opinion | 0.996 (0.989–1.000) | 0.930 (0.898–0.957) | 0.962 | 0.33% | 0.997 | 0.973 |

**GenVideo-Val, batch 3** (360 AI, 300 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.991 (0.980–1.000) | 0.911 (0.882–0.940) | 0.949 | 1.00% | 0.971 | 0.914 |
| Veracity | 0.994 (0.985–1.000) | 0.939 (0.915–0.963) | 0.966 | 0.67% | 0.988 | 0.953 |
| Veracity + second opinion | 0.997 (0.991–1.000) | 0.950 (0.928–0.973) | 0.973 | 0.33% | 0.984 | 0.956 |

**Pooled** (1156 AI, 900 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.983 (0.976–0.991) | 0.912 (0.894–0.927) | 0.946 | 2.00% | 0.986 | 0.874 |
| Veracity | 0.997 (0.994–1.000) | 0.941 (0.927–0.953) | 0.968 | 0.33% | 0.995 | 0.961 |
| Veracity + second opinion | 0.997 (0.994–1.000) | 0.960 (0.948–0.970) | 0.978 | 0.33% | 0.996 | 0.969 |

Pooled, the video checkpoint is ahead of WaveRep on precision and on recall,
and flags 3 of the 900 real clips where WaveRep flags 18. By set it is ahead
on both measures on ComGenVid and GenVideo-Val; on AIGVDBench its precision is
higher and its recall 0.004 lower. With the second opinion it is ahead on both
measures on all three sets.

The same comparison at WaveRep's own false-positive rate on each set (the
video checkpoint alone, threshold set on the set's real clips):

| Set | WaveRep: real clips flagged | WaveRep: precision / recall | Veracity at that rate: precision / recall |
|---|---|---|---|
| AIGVDBench, batch 3 | 9 of 300 (3.00%) | 0.982 / 0.970 | 0.982 / 0.988 |
| ComGenVid, batch 3 | 6 of 300 (2.00%) | 0.976 / 0.817 | 0.980 / 0.970 |
| GenVideo-Val, batch 3 | 3 of 300 (1.00%) | 0.991 / 0.911 | 0.991 / 0.953 |
| Pooled | 18 of 900 (2.00%) | 0.983 / 0.912 | 0.984 / 0.964 |

So the 0.004 of recall WaveRep has on AIGVDBench at shipped thresholds comes
with nine times the false alarms (9 real clips flagged against 1); allowed
the same nine, the video checkpoint finds 0.988.

Share detected by generator, and share of real clips flagged by source:

| Generator | Clips | WaveRep | Veracity | Veracity + second opinion |
|---|---|---|---|---|
| AccVideo (AIGVDBench) | 150 | 0.95 | 0.99 | 0.99 |
| CausVid (AIGVDBench) | 100 | 1.00 | 1.00 | 1.00 |
| Gen-3 (AIGVDBench) | 50 | 0.84 | 0.82 | 0.88 |
| IPOC (AIGVDBench) | 100 | 1.00 | 1.00 | 1.00 |
| RepVideo (AIGVDBench) | 96 | 1.00 | 0.94 | 1.00 |
| Sora (ComGenVid) | 150 | 0.65 | 0.89 | 0.89 |
| Veo 3 (ComGenVid) | 150 | 0.98 | 0.92 | 0.97 |
| VideoCrafter (GenVideo-Val) | 40 | 1.00 | 1.00 | 1.00 |
| Gen-2 (GenVideo-Val) | 40 | 1.00 | 1.00 | 1.00 |
| HotShot (GenVideo-Val) | 40 | 0.68 | 0.97 | 0.97 |
| Lavie (GenVideo-Val) | 40 | 1.00 | 0.95 | 1.00 |
| ModelScope (GenVideo-Val) | 40 | 0.97 | 1.00 | 1.00 |
| MoonValley (GenVideo-Val) | 40 | 1.00 | 1.00 | 1.00 |
| MorphStudio (GenVideo-Val) | 40 | 0.97 | 1.00 | 1.00 |
| Show-1 (GenVideo-Val) | 40 | 0.95 | 0.95 | 0.95 |
| WildScrape (GenVideo-Val) | 40 | 0.62 | 0.57 | 0.62 |

| Real source | Clips | WaveRep | Veracity | Veracity + second opinion |
|---|---|---|---|---|
| Kinetics-400 | 300 | 3.0% | 0.3% | 0.3% |
| MSVD | 300 | 2.0% | 0.0% | 0.3% |
| MSR-VTT | 300 | 1.0% | 0.7% | 0.3% |

AccVideo, CausVid, IPOC, HotShot, Lavie, ModelScope, MorphStudio, Show-1 and
the WildScrape clips are in none of Veracity's training data. WaveRep is ahead
on Gen-3, RepVideo, Veo 3, Lavie and WildScrape; Veracity on AccVideo, Sora,
HotShot, ModelScope and MorphStudio. That the two disagree on which clips are
hard is why the second opinion helps.

### The earlier blind result (phase-3 weights)

One checkpoint served photos and video until this batch. Its video result,
scored under the first protocol:

**AIGVDBench, batch 2 (Gen-3, CausVid, RepVideo, IPOC)** (400 AI, 200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.997 (0.992–1.000) | 0.983 (0.969–0.995) | 0.990 | 0.50% | 0.999 | 0.988 |
| Veracity (phase-3 weights) | 0.995 (0.987–1.000) | 0.955 (0.934–0.974) | 0.974 | 1.00% | 0.999 | 0.965 |

**ComGenVid, batch 2 (Sora, Veo 3)** (200 AI, 200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.994 (0.981–1.000) | 0.835 (0.787–0.885) | 0.908 | 0.50% | 0.986 | 0.890 |
| Veracity (phase-3 weights) | 0.990 (0.972–1.000) | 0.955 (0.925–0.981) | 0.972 | 1.00% | 0.995 | 0.970 |

**GenVideo-Val, batch 2** (270 AI, 200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.992 (0.979–1.000) | 0.904 (0.866–0.937) | 0.946 | 1.00% | 0.990 | 0.911 |
| Veracity (phase-3 weights) | 0.973 (0.952–0.992) | 0.937 (0.908–0.965) | 0.955 | 3.50% | 0.988 | 0.915 |

**Pooled** (870 AI, 600 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.995 (0.990–0.999) | 0.924 (0.906–0.941) | 0.958 | 0.67% | 0.993 | 0.936 |
| Veracity (phase-3 weights) | 0.987 (0.978–0.994) | 0.949 (0.934–0.963) | 0.968 | 1.83% | 0.995 | 0.932 |

Higher recall than WaveRep pooled and on two sets, lower precision on all
three. Looking at the real clips it flagged across all earlier sets, most
were not camera footage: cartoons, 3D animation, motion graphics. The video
checkpoint was trained on from these weights with real animation, cartoons
and gameplay added, along with more generators (see [DATA.md](DATA.md)), and
its threshold target was lowered from 1% to 0.5%.

### Development sets

Pooled results with the video checkpoint; its calibration and the
second-opinion rule were fitted on these sets' clips, so these are in-sample.
Per-set tables are in [DEV_RESULTS.md](DEV_RESULTS.md).

**Benchmark sets, pooled** (886 AI, 550 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.982 (0.973–0.991) | 0.878 (0.855–0.901) | 0.927 | 2.55% | 0.976 | 0.835 |
| Veracity | 0.994 (0.988–0.999) | 0.897 (0.876–0.918) | 0.943 | 0.91% | 0.989 | 0.897 |
| Veracity + second opinion | 0.994 (0.988–0.999) | 0.922 (0.904–0.940) | 0.957 | 0.91% | 0.990 | 0.929 |

**Confirmation batch 1, pooled** (1207 AI, 800 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.993 (0.988–0.998) | 0.857 (0.837–0.878) | 0.920 | 0.88% | 0.952 | 0.860 |
| Veracity | 0.995 (0.991–0.999) | 0.877 (0.859–0.896) | 0.933 | 0.62% | 0.986 | 0.901 |
| Veracity + second opinion | 0.996 (0.992–0.999) | 0.886 (0.867–0.905) | 0.938 | 0.50% | 0.982 | 0.892 |

Two sets in there go against Veracity or against everyone. On the Frontier
2026-10 video set (101 clips from twelve current commercial models) WaveRep
finds 0.921 with 7 of 100 real clips flagged, the video checkpoint 0.762 with
none, and 0.881 with 2 when the second opinion is on. On copies of those clips
that went through a messenger app nobody does well: WaveRep finds 0.059, the
video checkpoint 0.158.

## Speed

Same machine, idle, final build ([results/speed_final.log](results/speed_final.log)).
"As released" rows are the authors' own PyTorch pipelines on the same files
(PyTorch 2.15, Metal backend, 32-bit floats;
[results/speed_released.log](results/speed_released.log)); the other
reference rows are the Rust ports, which share Veracity's engine and so
isolate the cost of the model and its preprocessing. The SwinV2 port and the
32-bit WaveRep port were timed in the earlier pass
([results/speed.log](results/speed.log)); nothing in them changed since.

**Photos** – 1,000 files drawn from the test sets (mixed JPEG and PNG, up to
4K), decode to verdict.

| System | GPU, images/s | CPU, images/s | Median latency, one image, GPU |
|---|---|---|---|
| **Veracity** (default: early exit, 3.0 views per image here) | **263** | **38.8** | **10.9 ms** |
| Veracity `--strict` / `--sensitive` (early exit) | 292 / 282 | 39.0 / – | – |
| **Veracity `--fast`** (two views) | **455** | **52.6** | **8.3 ms** |
| Veracity `--all-views` (six views of every image) | 180 | 19.5 | 10.9 ms |
| Community Forensics, as released | 61–67 | 25.4 | 19.8 ms |
| Community Forensics, Rust port, 32-bit | 100 | 35.0 | 18.1 ms |
| Community Forensics, Rust port, 16-bit (not a released configuration) | 253 | – | – |
| SwinV2 detector, as released | 8.6 | – | – |
| SwinV2 detector, Rust port, 32-bit | 30 | – | 39.4 ms |

Veracity's GPU engine runs at 16 bits; the CPU engine at 32. Six 224 px views
push about twice as many tokens through the same size of network as
Community Forensics' one 384 px view, so with every view looked at Veracity
is the slower of the two on equal engines. Early exit halves the work: the
default profile is then faster than Community Forensics in every
configuration measured, by a wide margin against the released pipeline (4×
on the GPU, 1.5× on the CPU) and a narrow one against the Rust port (263
against 253 at 16 bits on the GPU, 38.8 against 35.0 on the CPU). How many
views an image needs depends on the image: batches of obvious images run
faster than this, batches of borderline ones slower. For one image at a
time on the GPU all six views go in a single batch, so latency is the same
with and without early exit; on the CPU the median is 49 ms against 61 ms
for all views and 67 ms for the Community Forensics port.

**Video** – 147 clips, every fourteenth of the blind batch.

| System | Seconds per clip | Clips/s |
|---|---|---|
| **Veracity** (GPU, 16 frames × 6 views) | **0.089** | **11.2** |
| **Veracity + second opinion** (GPU; 11% of these clips took the second stage) | **0.26** | **3.9** |
| Veracity (CPU) | 0.76 | 1.3 |
| WaveRep, as released | 4.05 | 0.25 |
| WaveRep, Rust port, 32-bit (the released precision) | 3.72 | 0.27 |
| WaveRep, Rust port, 16-bit | 1.08 | 0.92 |

Video frames always get all six views; early exit applies to photos only.

## Reproduce

```bash
veracity data baselines
veracity data confirm3 && veracity evaluate confirm3 --baselines
veracity data confirm4 && veracity evaluate confirm4 --baselines
veracity data confirm5 && veracity evaluate confirm5 --baselines
veracity data benchmarks && veracity evaluate benchmarks --baselines
```

`evaluate` writes per-item scores for every system to `runs/scores/` and
prints the tables above. `veracity report` recomputes any table from score
files.
