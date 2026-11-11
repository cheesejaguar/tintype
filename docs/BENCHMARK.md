# Benchmark

Veracity against three open detectors, on data none of them trained on.
Everything was measured on one machine (Apple M6, 32 GB, macOS 27.2) in
November 2026, and every system – Veracity and the reference detectors – was
run by the same Rust binary.

## Summary

Results on the five blind batches – data downloaded only after the
configuration under test and the protocol had been written down, and scored
once. Each system is at its shipped threshold.

| | Veracity | Reference |
|---|---|---|
| **Photos**, batch 7: 12,305 AI and 10,747 real | default profile: precision 0.988, recall 0.897 | Community Forensics: 0.945, 0.789 |
| | with the photo second opinion: 0.972, 0.935 | SwinV2 detector: 0.841, 0.587 |
| **Photos**, batch 6: 8,628 AI and 7,696 real | default profile, current weights: precision 0.986, recall 0.904 | Community Forensics: 0.966, 0.818 |
| | | SwinV2 detector: 0.834, 0.541 |
| **Photos**, batch 5: 7,079 AI and 6,278 real | default profile, phase-3 weights: 0.980, 0.875 | Community Forensics: 0.963, 0.762 |
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

- **Older generators, set by set.** Batches 5 and 6 are mostly GANs, GLIDE
  and identity-preserving adapters. Community Forensics was trained on
  thousands of such generators and reaches 0.96–1.00 recall on the GAN sets.
  After batch 5 the photo weights were retrained with such generators added;
  on batch 6 the default profile's recall is still below Community Forensics'
  on fourteen of nineteen sets (eleven GAN sets, GLIDE, IP-Adapter,
  PhotoMaker), equal on two and above on three, while its precision is higher
  on seventeen and equal on two. Given Community Forensics' own
  false-positive rate on each set the gap shrinks to 0.002–0.023 of recall on
  ten sets, with four ties and five wins. The pooled results are carried by
  So-Fake-OOD – current commercial generators in social-media photos – where
  Community Forensics finds 0.40 and Veracity 0.86. No single threshold puts
  Veracity ahead of Community Forensics on recall on every set.
- **The photo second opinion closes most of that, not all.** On batch 7,
  with `--photo-second-opinion`, precision and recall are both at or above
  Community Forensics' on 26 of 30 sets (the default alone: 12). The four
  others: Midjourney and WFIR, where one real photo is flagged to Community
  Forensics' none; SAN, where precision is 0.970 against 0.992 with recall
  0.735 against 0.543; and the FaceForensics++ deepfakes – face swaps – where
  recall is 0.480 against 0.573.
- **The photo second opinion is Community Forensics' own network, and it
  costs its speed.** The mode sends about half the photos of a balanced test
  set – nine real photos in ten – to that network. It is then as fast as
  Community Forensics through this engine, no faster (288 against 295
  images/s on the GPU, 41 against 42 on the CPU), and 3.5× and 1.6× faster
  than Community Forensics as released. On a stream of mostly real photos it
  is slower than Community Forensics through this engine. It also flags more
  real photos than the default: 3.1% against 1.2% on batch 7 (Community
  Forensics: 5.2%).
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
6. **Third blind photo batch** (`confirm6`). The retrained (phase-6) photo
   weights, their thresholds and the protocol were written down before this
   data was downloaded. It was scored once, with the phase-3 weights
   alongside.

In every table: precision and recall at each system's shipped threshold, with
95% bootstrap intervals; "Recall at 1% FPR" sets each system's threshold on
that set's own real items, which removes the choice of threshold.

## Photos

### Blind batch 7: the photo second opinion

Batch 6 left the default profile ahead of Community Forensics pooled and
behind it on recall on fourteen of nineteen sets. `--photo-second-opinion`
is the attempt to close that set by set: a photo the default profile does
not flag with confidence (calibrated first-stage logit between -7 and 0.5)
is also scored by the Community Forensics 384-pixel network, and is flagged
when half the first logit plus that network's logit is positive. The band
was chosen on the 48 photo sets scored before this batch and written down,
with what was expected, before the batch was fetched
([PREREGISTRATION.md](PREREGISTRATION.md)). The batch: fresh files from every
collection the earlier photo batches drew on, and five families of the
classic test set that no batch had used (plan `confirm7`).

| Batch 7, pooled (12,305 AI, 10,747 real) | Precision | Recall | Real photos flagged | AUROC |
|---|---|---|---|---|
| Community Forensics | 0.945 | 0.789 | 564 (5.2%) | 0.958 |
| SwinV2 detector | 0.841 | 0.587 | 1,366 (12.7%) | 0.794 |
| **Veracity**, default | **0.988** | **0.897** | 130 (1.2%) | 0.987 |
| **Veracity + second opinion** | **0.972** | **0.935** | 329 (3.1%) | 0.987 |

Set by set, as precision / recall (real photos flagged); bold where both are
at or above Community Forensics':

| Set | AI / real | Community Forensics | Veracity | Veracity + second opinion |
|---|---|---|---|---|
| So-Fake-OOD | 2217 / 1525 | 0.981 / 0.421 (18) | **0.985 / 0.836 (28)** | **0.985 / 0.839 (29)** |
| Midjourney | 394 / 313 | 1.000 / 0.650 (0) | 0.997 / 0.970 (1) | 0.997 / 0.967 (1) |
| FLUX.1-dev | 400 / 366 | 0.945 / 0.735 (17) | **0.992 / 0.990 (3)** | **0.973 / 0.985 (11)** |
| DALL-E 3 | 400 / 365 | 0.983 / 0.855 (6) | **0.997 / 0.993 (1)** | **0.990 / 0.993 (4)** |
| Imagen 3 | 400 / 374 | 0.895 / 0.385 (18) | **0.989 / 0.922 (4)** | **0.971 / 0.912 (11)** |
| SD3 | 400 / 383 | 0.946 / 0.795 (18) | **0.993 / 1.000 (3)** | **0.973 / 0.998 (11)** |
| SDXL | 400 / 0 | – / 0.998 | **– / 1.000** | **– / 1.000** |
| GLIDE | 397 / 388 | 0.947 / 0.995 (22) | 0.995 / 0.957 (2) | **0.968 / 0.995 (13)** |
| BLIP | 400 / 400 | 0.939 / 1.000 (26) | **0.990 / 1.000 (4)** | **0.971 / 1.000 (12)** |
| IP-Adapter | 400 / 373 | 0.957 / 0.885 (16) | 0.988 / 0.833 (4) | **0.971 / 0.932 (11)** |
| InstantID | 399 / 368 | 0.929 / 0.654 (20) | **0.995 / 0.990 (2)** | **0.978 / 0.982 (9)** |
| Infinite-ID | 400 / 381 | 0.964 / 0.927 (14) | **0.995 / 1.000 (2)** | **0.988 / 1.000 (5)** |
| PhotoMaker | 399 / 373 | 0.950 / 0.712 (15) | 0.996 / 0.692 (1) | **0.978 / 0.774 (7)** |
| ProGAN | 400 / 399 | 0.920 / 1.000 (35) | 0.942 / 0.980 (24) | **0.928 / 1.000 (31)** |
| R3GAN | 400 / 365 | 0.950 / 1.000 (21) | 0.997 / 0.958 (1) | **0.971 / 1.000 (12)** |
| StyleGAN3 | 400 / 382 | 0.997 / 0.985 (1) | 0.991 / 0.870 (3) | **0.997 / 0.990 (1)** |
| StyleGAN-XL | 400 / 369 | 0.955 / 1.000 (19) | 1.000 / 0.885 (0) | **0.971 / 1.000 (12)** |
| StyleSwim | 400 / 369 | 0.957 / 1.000 (18) | 0.994 / 0.843 (2) | **0.978 / 1.000 (9)** |
| WFIR | 200 / 200 | 1.000 / 1.000 (0) | 0.995 / 0.955 (1) | 0.995 / 1.000 (1) |
| StyleGAN (classic) | 300 / 292 | 0.964 / 0.970 (11) | 0.982 / 0.927 (5) | **0.967 / 0.980 (10)** |
| StyleGAN2 (classic) | 300 / 282 | 0.961 / 0.980 (12) | 0.985 / 0.860 (4) | **0.971 / 0.993 (9)** |
| BigGAN | 300 / 300 | 0.974 / 1.000 (8) | 0.980 / 0.997 (6) | **0.980 / 1.000 (6)** |
| CycleGAN | 300 / 300 | 0.971 / 0.997 (9) | 0.974 / 0.983 (8) | **0.977 / 1.000 (7)** |
| GauGAN | 300 / 300 | 0.993 / 1.000 (2) | 1.000 / 0.983 (0) | **1.000 / 1.000 (0)** |
| StarGAN | 300 / 300 | 1.000 / 0.993 (0) | 1.000 / 0.983 (0) | **1.000 / 0.993 (0)** |
| CRN | 300 / 300 | 0.723 / 0.930 (107) | **0.964 / 0.973 (11)** | **0.853 / 0.983 (51)** |
| IMLE | 300 / 300 | 0.734 / 0.987 (107) | **0.984 / 0.997 (5)** | **0.867 / 1.000 (46)** |
| SAN | 219 / 200 | 0.992 / 0.543 (1) | 0.974 / 0.689 (4) | 0.970 / 0.735 (5) |
| SITD | 180 / 180 | 0.831 / 0.572 (21) | **1.000 / 0.817 (0)** | **0.975 / 0.856 (4)** |
| FaceForensics++ deepfakes | 300 / 300 | 0.989 / 0.573 (2) | 0.977 / 0.143 (1) | 0.993 / 0.480 (1) |

- With the second opinion, recall is higher than Community Forensics' on 19
  sets, equal on 10 and lower on one; precision is higher on 25, equal on two
  and lower on three. Both are at or above on 26 of the 30 sets – 25 of the
  29 that have real photos – against 12 (11) for the default alone, whose
  recall is lower on 16 sets.
- The four exceptions are the ones listed in the summary. On the deepfake
  set the second opinion recovers most of what the default misses (0.143 to
  0.480) and still ends below Community Forensics.
- The second stage looked at 48.0% of the photos. It flagged 749 photos the
  default had passed (525 AI, 224 real) and cleared 84 the default had
  flagged (59 AI, 25 real): 466 more detections and 199 more false alarms.
- SDXL has no real photos (all 400 drawn were identical to real photos used
  elsewhere and were dropped) and WFIR has 200 per class, all that earlier
  batches had left. The prediction written beforehand was "more sets than the
  default manages, not all of them", from 46 of 48 in-sample and 43 of 48 held
  out.
- The scoring run stopped once, after 25 sets, when the disk filled. No
  score had been read. It was restarted keeping the reference detectors'
  score files (each checked against its manifest) and recomputing Veracity's,
  which came out byte for byte the same.

Speed on this batch, decode to verdict, on 2,096 of its photos (every
eleventh), two rounds
([results/confirm7_second_opinion.log](results/confirm7_second_opinion.log)):

| Batch 7 sample | GPU, images/s | CPU, images/s |
|---|---|---|
| **Veracity**, default (1.7 views per image) | **530–563** | **77–78** |
| **Veracity + second opinion** (48% of photos to the second stage) | **281–288** | **39–41** |
| Community Forensics, as released (PyTorch, 32-bit) | 82 | 26 |
| Community Forensics, Rust port, 32-bit on the GPU | 112 | – |
| Community Forensics, Rust port, 16-bit on the GPU (not a released configuration) | 291–295 | 39–42 |

Raw tables: [results/confirm7.md](results/confirm7.md).

### Blind batch 6: the retrained photo weights

Batch 5 showed the photo weights finding fewer images than Community
Forensics from older GAN and diffusion families and from identity adapters.
The weights were trained on from there with such generators added (phase 6,
[DATA.md](DATA.md)), calibrated by the same rules, and tested on a batch
fixed beforehand: three more shards of So-Fake-OOD, a second sample of the
twelve AIGIBench archives of batch 5 that shares no file with the first, and
six families of the classic GAN test set of Wang et al. (CVPR 2020).

**Pooled** (8,628 AI, 7,696 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.966 (0.962–0.970) | 0.818 (0.810–0.826) | 0.886 | 3.26% | 0.970 | 0.712 |
| SwinV2 detector | 0.834 (0.824–0.844) | 0.541 (0.530–0.551) | 0.656 | 12.10% | 0.772 | 0.227 |
| Veracity | 0.986 (0.983–0.989) | 0.904 (0.897–0.910) | 0.943 | 1.44% | 0.990 | 0.877 |
| Veracity (strict) | 0.996 (0.994–0.997) | 0.815 (0.806–0.823) | 0.896 | 0.40% | 0.987 | 0.869 |
| Veracity (fast) | 0.995 (0.993–0.997) | 0.761 (0.751–0.769) | 0.862 | 0.43% | 0.990 | 0.856 |
| Veracity (sensitive) | 0.960 (0.956–0.964) | 0.975 (0.972–0.978) | 0.967 | 4.57% | 0.993 | 0.882 |
| Veracity, phase-3 weights (default) | 0.977 | 0.882 | – | 2.34% | – | – |

By set – precision / recall, with the share of real photos flagged in
brackets. "At CF's rate" is Veracity's recall when it is allowed to flag as
many real photos of that set as Community Forensics did.

| Set | AI / real | Community Forensics | SwinV2 detector | Veracity | Veracity recall at CF's rate | Phase-3 weights |
|---|---|---|---|---|---|---|
| So-Fake-OOD, batch 5 | 2040 / 1361 | 0.976 / 0.401 (1.47%) | 0.945 / 0.812 (7.13%) | 0.980 / 0.855 (2.57%) | 0.813 | 0.985 / 0.852 |
| BLIP-Diffusion | 400 / 400 | 0.957 / 1.000 (4.50%) | 0.915 / 1.000 (9.25%) | 0.993 / 1.000 (0.75%) | 1.000 | 0.976 / 1.000 |
| GLIDE | 397 / 387 | 0.945 / 0.997 (5.94%) | 0.797 / 0.544 (14.21%) | 0.992 / 0.970 (0.78%) | 0.995 | 0.972 / 0.947 |
| Infinite-ID | 400 / 372 | 0.979 / 0.915 (2.15%) | 0.861 / 0.743 (12.90%) | 0.988 / 1.000 (1.34%) | 1.000 | 0.973 / 1.000 |
| InstantID | 400 / 368 | 0.934 / 0.667 (5.16%) | 0.860 / 0.953 (16.85%) | 0.990 / 0.985 (1.09%) | 1.000 | 0.978 / 0.990 |
| IP-Adapter | 400 / 368 | 0.957 / 0.885 (4.35%) | 0.779 / 0.698 (21.47%) | 0.991 / 0.873 (0.82%) | 0.985 | 0.975 / 0.860 |
| PhotoMaker | 400 / 374 | 0.963 / 0.713 (2.94%) | 0.778 / 0.578 (17.65%) | 0.992 / 0.608 (0.53%) | 0.965 | 0.978 / 0.453 |
| ProGAN | 400 / 400 | 0.920 / 1.000 (8.75%) | 0.615 / 0.080 (5.00%) | 0.943 / 0.998 (6.00%) | 0.998 | 0.935 / 0.975 |
| R3GAN | 400 / 366 | 0.966 / 1.000 (3.83%) | 0.752 / 0.560 (20.22%) | 0.997 / 0.968 (0.27%) | 0.998 | 0.966 / 0.983 |
| StyleGAN-XL | 400 / 375 | 0.964 / 1.000 (4.00%) | 0.847 / 0.845 (16.27%) | 0.989 / 0.892 (1.07%) | 0.993 | 0.967 / 0.892 |
| StyleGAN3 | 400 / 373 | 0.990 / 0.985 (1.07%) | 0.622 / 0.255 (16.62%) | 0.997 / 0.845 (0.27%) | 0.983 | 0.986 / 0.713 |
| StyleSwim | 400 / 372 | 0.952 / 1.000 (5.38%) | 0.783 / 0.560 (16.67%) | 0.994 / 0.815 (0.54%) | 0.993 | 0.979 / 0.917 |
| WFIR (StyleGAN faces) | 398 / 400 | 1.000 / 1.000 (0.00%) | 0.582 / 0.545 (39.00%) | 1.000 / 0.920 (0.00%) | 1.000 | 0.994 / 0.784 |
| BigGAN (classic set) | 299 / 299 | 0.984 / 1.000 (1.67%) | 0.682 / 0.050 (2.34%) | 0.987 / 0.997 (1.34%) | 0.997 | 0.977 / 0.987 |
| CycleGAN (classic set) | 298 / 298 | 0.974 / 0.997 (2.68%) | 0.467 / 0.047 (5.37%) | 0.983 / 0.963 (1.68%) | 0.977 | 0.968 / 0.923 |
| GauGAN (classic set) | 299 / 299 | 0.980 / 1.000 (2.01%) | 0.583 / 0.023 (1.67%) | 0.990 / 0.997 (1.00%) | 1.000 | 0.990 / 0.990 |
| StarGAN (classic set) | 299 / 299 | 1.000 / 1.000 (0.00%) | 0.429 / 0.010 (1.34%) | 1.000 / 1.000 (0.00%) | 1.000 | 1.000 / 1.000 |
| StyleGAN (classic set) | 299 / 299 | 0.963 / 0.963 (3.68%) | 0.686 / 0.080 (3.68%) | 0.968 / 0.923 (3.01%) | 0.940 | 0.962 / 0.923 |
| StyleGAN2 (classic set) | 299 / 286 | 0.943 / 0.990 (6.29%) | 0.357 / 0.017 (3.15%) | 0.988 / 0.836 (1.05%) | 0.970 | 0.987 / 0.773 |

Pooled, the default profile is ahead of both reference detectors on precision
and on recall, and the retrained weights are ahead of the phase-3 weights on
both. Set by set:

- Against the SwinV2 detector Veracity is ahead on both measures on eighteen
  of the nineteen sets and level on recall on the other (BLIP-Diffusion, both
  1.000).
- Against Community Forensics Veracity's precision is the higher on seventeen
  sets and equal on two. Its recall is higher on three (So-Fake-OOD,
  InstantID, Infinite-ID), equal on two (BLIP-Diffusion and StarGAN, both
  1.000) and lower on fourteen: eleven of the twelve GAN sets, GLIDE,
  IP-Adapter and PhotoMaker. Community Forensics finds 0.96–1.00 of the GAN
  images; Veracity finds 0.82–1.00 of them and flags fewer of those sets'
  real photos on every one where either detector flags any.
- Given Community Forensics' own false-positive rate on each set, Veracity's
  recall is higher on five sets, equal on four and lower on ten, by 0.002 to
  0.023. On images from GANs Community Forensics remains marginally the
  better detector; on current commercial generators and identity adapters
  Veracity is far ahead (0.855 against 0.401 on So-Fake-OOD).
- Retraining helped where it was aimed and cost a little elsewhere: StyleGAN3
  0.713 → 0.845, WFIR 0.784 → 0.920, PhotoMaker 0.453 → 0.608, StyleGAN2
  0.773 → 0.836; StyleSwim 0.917 → 0.815 and R3GAN 0.983 → 0.968, with the
  share of real photos flagged on those two falling from 2.2% and 3.8% to
  0.5% and 0.3%.

**Could a second opinion close the set-by-set gap?** Not fully. An estimate
from the score files of all 49 photo sets scored so far (seen data, so
optimistic): sending the photos the default profile rejects without being
sure – about one in five – to the Community Forensics network, and flagging
those where the two scores add up high enough, would put Veracity at or
above Community Forensics on both precision and recall on 39 of the 49 sets,
against 25 for the default alone, and move the pooled result from
0.986 / 0.876 to 0.983 / 0.905. Ten sets would still go to Community
Forensics on one measure or the other, because a second opinion can only add
detections: it cannot repair the sets where Veracity's precision is the
lower one. This mode is not built.

Raw tables: [results/confirm6.md](results/confirm6.md),
[results/confirm6_phase3_weights.md](results/confirm6_phase3_weights.md),
[results/confirm6_at_reference_rates.md](results/confirm6_at_reference_rates.md).

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
The photo rows in the table were measured with the phase-3 weights. The
phase-6 weights were timed later, with the machine in use for other things,
in a pass that alternated them with the phase-3 weights and Community
Forensics on the same files three times on the GPU and twice on the CPU
([results/speed_interleaved.log](results/speed_interleaved.log)); every
figure repeated within 3%:

| Same pass, machine under load | GPU, images/s | CPU, images/s |
|---|---|---|
| **Veracity, phase-6 weights** (default; 2.8 views per image here) | **196** | **30.3** |
| Veracity, phase-3 weights (default; 3.0 views per image) | 167 | 28.6 |
| Community Forensics, Rust port (16-bit on the GPU, 32-bit on the CPU) | 165 | 25.5 |
| Community Forensics, as released | 50 | 18.4 |

The absolute numbers are lower than on the idle machine; the order is the
same. The retrained weights are a little faster than the ones they replace
and faster than Community Forensics in every configuration measured.
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

### Five speed changes, before and after

A later pass looked for speed alone. The build before it and the build after
it were run alternately on the same files, three times on the GPU and twice
on the CPU, while the machine was also running a virtual machine on six of
its cores ([results/speed_improvements.log](results/speed_improvements.log)).
The absolute figures are therefore not comparable with the idle-machine table
above; the ratios are the result.

| Same pass, machine under load | Before | After | Ratio |
|---|---|---|---|
| Photos, default, GPU (5,000 images) | 288 images/s | 464 images/s | 1.61× |
| Photos, default, CPU (1,000 images) | 36.3 images/s | 53.2 images/s | 1.47× |
| Photos, `--fast`, GPU | 453 images/s | 500 images/s | 1.11× |
| Photos, `--all-views`, GPU | 162 images/s | 179 images/s | 1.10× |
| Photos, `--all-views`, CPU | 20.8 images/s | 24.6 images/s | 1.18× |
| Video, GPU (147 clips), second pass | 6.1 clips/s | 21.4 clips/s | 3.5× |
| Video, CPU (147 clips), second pass | 0.89 clips/s | 3.8 clips/s | 4.3× |
| Blind batch 6 photos, GPU (16,324 images) | 357 images/s | 580 images/s | 1.62× |
| Blind batch video, GPU (2,056 clips), second pass | 10.7 clips/s | 31.7 clips/s | 3.0× |

The video rows come from a second pass, run after the last of the changes
to video (decoding that stops half-way). Another job was using the GPU for
part of that pass, so its absolute figures are lower still and they moved
between repeats: the blind batch took 321 s and 193 s before the changes and
67 s and 65 s after them, and the table uses the closer pair. Video is about
three times faster (four on the CPU). Photos are about 1.6 times faster in
the default profile, short of the factor of two that was the aim. What
changed:

1. **Clips stop early.** Frames are looked at in stages: the first and last
   of the eight frames in the clip's first half, then four of them, then all
   eight, and only then the second half. A clip whose mean logit is 4, 3 and
   3 away from the threshold at those stages is not shown the rest: 4.8
   frames per clip on the blind batch instead of 16. Because the first half
   is looked at first, decoding stops at the half-way mark as well, and only
   the clips still undecided there (17% on the blind batch) are opened again
   for their second half. Simulated from the stored frame logits this changed
   no verdict on 4,770 development clips or the 2,013 blind ones; re-run for
   real on the blind batch it gives the same 0.997 / 0.941 with no verdict
   changed. A second opinion turns it off, because its band is defined on the
   logit of all frames.
2. **Tighter early exit for photos.** The margins went from 7–9 / 5 / 3.5 /
   2.5 / 2 to 6 / 3 / 2 / 1.5 / 1, fitted on the 44,336 development images
   under the condition that pooled precision and recall stay within 0.001 of
   the previous schedule. Views per image fall from 2.64 to 1.93 there. Blind
   batch 6, re-scored with the real pipeline: 1.71 views instead of 2.30, and
   the same pooled result to four decimals (0.9860 / 0.9036, 111 real images
   flagged both times). 116 of the 16,324 verdicts differ, in both directions;
   per set, recall moves by between −0.033 (IP-Adapter) and +0.028. This is a
   re-score of a batch already seen, and the choice among four candidate
   schedules was made with the blind numbers visible, so it is not a blind
   result.
3. **Every batch full.** With more images leaving early, the later stages
   seldom had a full batch waiting, and the fixed-size GPU batch ran half
   empty. A batch is now filled with the next views of whichever images are
   waiting, whatever stage they are at.
4. **Pixels handed over in the engine's own form.** Crops stay 8-bit until
   the last moment. For the GPU the decode workers turn them into
   half-precision planar values by table lookup, so the main thread only
   copies, the GPU receives half the bytes and no longer casts them. The CPU
   engine normalises while it cuts patches. Scores are unchanged (no verdict
   differs on the 1,000 timing images with all views).
5. **The last block for the class token only.** Only the class token's
   output of the last transformer block is read, so only its query is
   attended and only that token goes through the projection and the MLP. The
   result is the same; the block costs about a quarter of what it did.

Tried and not kept:

- **Token merging** (Bolya et al., ICLR 2023), which the literature reports
  as 1.6–1.9× on this network without retraining. Here it removes the signal:
  on the 1,000 timing images recall fell from 0.848 to 0.786 at 8 merged
  tokens per block and to 0.708 at 13, and dropping tokens instead of
  averaging them was no better. Merging only in the last four blocks costs
  much less (6 verdicts in 1,000) but saves only about 15% of the network.
- **Fused attention** (`scaledDotProductAttention`) and **8-bit input cast on
  the GPU**: the first was no faster at this sequence length, the second was
  several times slower than the float input it replaced.

The GPU now spends its time in the twelve blocks themselves (about 4 ms each
for a batch of 64), so a further large gain on photos would need fewer tokens
or fewer blocks, which means retraining with that in mind.

**On Linux** – everything above was measured on macOS. On Linux `detect`
runs on the CPU engine, with pure-Rust matrix kernels in place of Accelerate
and a pure-Rust video reader in place of the system decoder. Both whole
blind batches of the current comparison – the third blind photo batch and
the blind video batch – were scored in an aarch64 Ubuntu virtual machine on
the same Mac (8 of its 12 cores), next to Community Forensics as its authors
ship it (PyTorch 2.15) and its network through this engine
([results/linux.log](results/linux.log)):

| On Linux | 16,324 photos | 2,056 clips |
|---|---|---|
| **Veracity**, precision / recall | **0.986 / 0.904** | **0.997 / 0.942** on the 1,756 clips it could read |
| The same files on macOS (GPU engine) | 0.986 / 0.904 | 0.997 / 0.941 |
| Verdicts that differ from the macOS CPU engine | 0 (largest difference in probability 0.00003) | – |
| Verdicts that differ from the macOS GPU scores used above | 7 | 1 |
| Files Linux could not read | 0 | 300 (lossless H.264: the real clips of ComGenVid) |
| Community Forensics as released, precision / recall | 0.965 / 0.819 | – |
| Community Forensics network, Rust port | 0.966 / 0.818 (no verdict differs from macOS) | – |

| On Linux, CPU, decode to verdict | Photos/s, all 16,324 | By set (19) |
|---|---|---|
| **Veracity**, default | **24.9** | 18.9–40.1 |
| Community Forensics, as released | 14.8 (16.1 by its own timer, which starts once the model is loaded) | 14.4–15.3 |
| Community Forensics network, Rust port | 16.3 | 15.2–17.7 |
| Veracity, video | 1.4 s per clip (2,915 s for the batch) | |

| On Linux, video | Precision / recall | Real clips flagged | Seconds per clip, CPU |
|---|---|---|---|
| **Veracity**, the 1,756 clips it could read | **0.997 / 0.942** | 3 of 600 | 1.4 |
| WaveRep network (Rust port, macOS scores), same 1,756 | 0.989 / 0.912 | 12 of 600 | – |
| **Veracity**, 130-clip sample (127 read) | **1.000 / 0.948** | 0 of 50 | 1.6–2.0 |
| WaveRep as released, run on Linux, same 127 | 0.986 / 0.909 | 1 of 50 | 26.4 |

Those tables predate the five speed changes. The build with them was run in
the same virtual machine afterwards, alternating with the build before them
on each set ([results/linux_after_speed.log](results/linux_after_speed.log)):

| On Linux, after the speed changes | Before | After |
|---|---|---|
| Photos, batch 6, photos/s | 24.9 | **35.2** (faster on 19 of 19 sets, 1.3–1.5×) |
| Photos, precision / recall | 0.986 / 0.904 | 0.986 / 0.905; no verdict differs from the same build's macOS CPU engine |
| Video, seconds per clip | 1.41 | **0.43**; two frames were enough for 1,176 of the 1,756 clips read |
| Video, precision / recall | 0.997 / 0.942 | 0.997 / 0.942; no verdict differs from the build before or from the macOS CPU engine |

The photo second opinion was run there too, on batch 6, which is seen data
for that mode: Community Forensics as released, the default, the second
opinion and the Community Forensics port, one after another on each set.

| On Linux, batch 6 (in-sample for the second opinion) | Precision / recall | Photos/s |
|---|---|---|
| Veracity, default | 0.986 / 0.905 | 31.1 |
| Veracity + second opinion | 0.978 / 0.947 | 15.2 |
| Community Forensics, as released | 0.965 / 0.819 | 11.7 |
| Community Forensics network, Rust port | 0.966 / 0.818 | 15.3 |

With the second opinion, precision and recall were both at or above
Community Forensics' on 18 of the 19 sets (StyleGAN3: recall 0.980 against
0.985), no verdict differed from the macOS CPU engine, and the mode was 1.3×
faster than Community Forensics as released pooled – faster on 17 sets and
within timing noise of it on PhotoMaker and StyleGAN-XL, where two thirds of
the photos went to the second stage – and level with the port. On the CPU
the mode costs the default plus the 384-pixel network on the photos sent to
it; that network's matrix products already run at the core's peak rate.

Batch 7, which the mode had not seen, was then timed there the same way,
all 30 sets:

| On Linux, batch 7 (23,052 photos) | Precision / recall | Photos/s | Faster than Community Forensics as released on |
|---|---|---|---|
| Veracity, default | 0.988 / 0.897 | 35.5 | 30 of 30 sets (1.7–4.0×) |
| Veracity + second opinion | 0.972 / 0.935 | 17.5 | 28 of 30 sets |
| Community Forensics, as released | 0.945 / 0.789 | 14.3 | – |
| Community Forensics network, Rust port | 0.945 / 0.789 | 17.4 | – |

Pooled the mode is 1.2× faster than Community Forensics as released and
level with the port. It was slower than the released pipeline on PhotoMaker
(0.88×) and on the deepfake set (0.75×), and faster than the port on 18 of
the 30 sets; each cell is a single run. Precision, recall and the four sets
on which the mode falls short are the same as on macOS; against the macOS
GPU scores above, 6 default verdicts and 1 second-opinion verdict of 23,052
differ.

The rest of this section describes the earlier tables. The three photo
detectors were run one after another on each set, and
Veracity was the fastest on all 19: 1.3–2.8× the released pipeline and
1.2–2.6× the port. Precision and recall are what they are on macOS,
including the per-set picture: the default profile is behind Community
Forensics on recall on the same 14 of 19 sets. Six of the seven photo
verdicts that differ from the GPU scores are within 0.005 of the threshold;
the seventh is an image on which the 16-bit and 32-bit engines stop at
different views. On Linux the engine runs one view per thread; splitting
each matrix product across the threads instead gave 17.4 photos/s on this
batch with the same output to the last bit. An x86-64 Linux binary, run
under translation, gave the macOS verdict on each of 738 photos (checked
before that threading change, and not a speed measurement).

For video, WaveRep as its authors ship it (PyTorch on the CPU, 504 px crops
of the first 64 frames) was run in the same virtual machine on a 130-clip
sample of the blind batch, between two Veracity passes over the same clips:
26.4 s per clip against 1.6–2.0, so Veracity is 13–16× faster there, and on
the 127 clips it could read it found 73 of the 77 AI clips to WaveRep's 70
and flagged no real clip to WaveRep's one. The released pipeline agreed with
this report's Rust port of WaveRep on 129 of the 130 verdicts. Over the
1,756 readable clips of the whole batch, Veracity's Linux verdicts are ahead
of the port's on precision and recall pooled and on GenVideo-val, ahead on
recall on ComGenVid's AI clips (0.907 against 0.817), and behind on recall
on AIGVDBench by the same 0.004 as on macOS. Two limits: WaveRep
as released was not run over the whole batch on Linux (about 15 hours at
that rate), and the 300 clips the pure-Rust reader cannot open are every
real clip of one of the three video sets – clips WaveRep's reader opens –
so the Linux video figures rest on the other two sets for real clips.

**The pure-Rust video reader against the system decoder**, on macOS, where
both can be run on the same files (`check-videos --compare-decoders`). Over
the whole blind video batch, 2,056 clips:

| | |
|---|---|
| Clips both read | 1,751 (5 more timed out in the system decoder under load) |
| Frames compared, every frame of every clip | 215,090 |
| Frames identical, sample for sample and in colour tags and time | 214,848 |
| Frames that differ | 242, all in two 10-bit HEVC clips, by at most 20 levels |
| Clips whose reported duration differs | 20, by at most 0.47 s |
| Clips the pure-Rust reader cannot decode | 300: the real clips of one set, which are lossless H.264 |

On the 10-bit clips the pure-Rust decoder's output equals FFmpeg's sample
for sample (checked on one of them, FFmpeg used only as a referee); the
system decoder is the one that deviates. Scoring the batch through the
pure-Rust reader on the GPU engine gave the same verdict as the system
decoder on all 1,756 clips it read, and the same precision and recall on
each of the three sets (0.998 / 0.966, 1.000 / 0.903 with that set's real
clips missing, 0.994 / 0.939). Two more differences showed up on other
material: where a file begins with an empty edit the system decoder emits a
blank first frame that the pure-Rust reader does not, and MPEG-4 Part 2
clips are read by the system decoder only.

## Reproduce

```bash
veracity data baselines
veracity data confirm3 && veracity evaluate confirm3 --baselines
veracity data confirm4 && veracity evaluate confirm4 --baselines
veracity data confirm5 && veracity evaluate confirm5 --baselines
veracity data confirm6 && veracity evaluate confirm6 --baselines
veracity data confirm7 && veracity evaluate confirm7 --baselines
veracity data benchmarks && veracity evaluate benchmarks --baselines
```

`evaluate` writes per-item scores for every system to `runs/scores/` and
prints the tables above. `veracity report` recomputes any table from score
files.
