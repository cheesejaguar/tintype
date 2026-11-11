# Benchmark

Veracity against three open detectors, on data none of them trained on.
Everything was measured on one machine (Apple M6, 32 GB, macOS 27.2) in
November 2026, and every system – Veracity and the reference detectors – was
run by the same Rust binary.

## Summary

Results on the two blind batches – data downloaded only after the model, its
thresholds and the protocol had been written down, and scored once. Each
system is at its shipped threshold.

| | Veracity | Reference |
|---|---|---|
| **Photos**, 6,172 AI and 4,541 real | default profile: precision 0.997, recall 0.865 | Community Forensics: 0.970, 0.637 |
| | sensitive profile: 0.955, 0.972 | SwinV2 detector: 0.896, 0.918 |
| **Video**, 1,156 AI and 900 real | 0.997, 0.941 | WaveRep: 0.983, 0.912 |
| | with the second opinion: 0.997, 0.960 | |
| **Photo throughput**, GPU, end to end | 178 images/s; 454 with `--fast` | Community Forensics: 59 as released, 100 through this engine; SwinV2 detector: 8.7 and 30 |
| **Video**, time per clip | 0.09 s; 0.22 s with the second opinion | WaveRep: 3.7 s as released, 1.06 s through this engine at 16 bits |

Where Veracity is not ahead, or the comparison needs a qualifier:

- **SwinV2 detector, recall.** It trades precision for recall, flagging 14% of
  the real photos in the blind batch. Veracity's default profile is more
  precise on all five photo sets but finds fewer AI images on three of them
  (pooled recall 0.865 against 0.918). The sensitive profile is the
  like-for-like comparison: it is ahead on both measures on all five sets
  while flagging 6% of real photos.
- **Video, one set.** On AIGVDBench the video checkpoint alone has higher
  precision than WaveRep (0.998 against 0.982) and marginally lower recall
  (0.966 against 0.970). With the second opinion it is ahead on both on all
  three sets.
- **The second opinion is WaveRep's own network.** That mode shows the two
  models are complementary and that a cascade is both more accurate and
  faster than WaveRep alone; it is not a win by Veracity's weights alone. The
  rows labelled "Veracity" use only Veracity's weights.
- **Speed against a half-precision Community Forensics.** Run through this
  engine at 16 bits – a configuration its authors do not ship – Community
  Forensics does 255 images/s on the GPU and 36 on the CPU, more than the
  six-view default profile (178 and 20). The two-view fast profile is faster
  than Community Forensics in every configuration (454 and 54) and is still
  ahead of it on precision and recall on every blind photo set.
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

In every table: precision and recall at each system's shipped threshold, with
95% bootstrap intervals; "Recall at 1% FPR" sets each system's threshold on
that set's own real items, which removes the choice of threshold.

## Photos

### Blind batch

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

### Development sets

Pooled results on the three earlier suites, with the released weights. The
thresholds were fitted with these sets' real photos in the calibration data.
Per-set tables are in [DEV_RESULTS.md](DEV_RESULTS.md).

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

Same machine, idle. "As released" rows are the authors' own PyTorch pipelines
(PyTorch 2.15, Metal backend, 32-bit floats); the other reference rows are the
Rust ports, which share Veracity's engine and so isolate the cost of the
model and its preprocessing.

**Photos** – 1,000 files drawn from the test sets (mixed JPEG and PNG, up to
4K), decode to verdict.

| System | GPU, images/s | CPU, images/s | Median latency, one image |
|---|---|---|---|
| **Veracity** (six views) | **178** | **20.0** | **10.7 ms** |
| **Veracity `--fast`** (two views) | **454** | **53.6** | **8.3 ms** |
| Community Forensics, as released | 59 | 18.4 | 19.6 ms |
| Community Forensics, Rust port, 32-bit | 100 | 36.1 | 18.2 ms |
| Community Forensics, Rust port, 16-bit (not a released configuration) | 255 | – | – |
| SwinV2 detector, as released | 8.7 | – | – |
| SwinV2 detector, Rust port, 32-bit | 30 | – | 39.4 ms |

Veracity's GPU engine runs at 16 bits; the CPU engine at 32. The default
profile pushes about twice as many tokens through the same size of network as
Community Forensics does (six 224 px views against one 384 px view), so on
equal engines it is the slower of the two and the fast profile the faster.
Against the reference detectors as their authors ship them, both profiles
are faster on the GPU and on the CPU.

**Video** – 147 clips, every fourteenth of the blind batch.

| System | Seconds per clip | Clips/s |
|---|---|---|
| **Veracity** (GPU, 16 frames × 6 views) | **0.091** | **11.0** |
| **Veracity + second opinion** (GPU; 11% of these clips took the second stage) | **0.22** | **4.5** |
| Veracity (CPU) | 0.77 | 1.3 |
| WaveRep, Rust port, 32-bit (the released precision) | 3.72 | 0.27 |
| WaveRep, Rust port, 16-bit | 1.06 | 0.95 |
| WaveRep, as released (measured on a different 40-clip sample) | 3.76 | 0.27 |

Over the whole blind batch (2,056 clips) Veracity took 0.091 s per clip and
0.227 s with the second opinion, 13% of clips taking the second stage. Running
the second-opinion mode end to end gave the same verdict as the evaluation on
every clip.

## Reproduce

```bash
veracity data baselines
veracity data confirm3 && veracity evaluate confirm3 --baselines
veracity data confirm4 && veracity evaluate confirm4 --baselines
veracity data benchmarks && veracity evaluate benchmarks --baselines
```

`evaluate` writes per-item scores for every system to `runs/scores/` and
prints the tables above. `veracity report` recomputes any table from score
files.
