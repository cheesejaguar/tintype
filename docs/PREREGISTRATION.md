# What was fixed before each blind batch

Four batches were scored blind: the third confirmation batch (photos, and
video with the phase-3 weights), the fourth (video, for the dedicated video
checkpoint), the fifth (photos, for the balanced default threshold and early
exit) and the sixth (photos, for the retrained phase-6 weights). Each note below was written before its data was downloaded and
is reproduced unchanged.

## Third confirmation batch: what was fixed before scoring it

Written 2026-11-10 16:02 UTC, while phase-3 training was still running and
before any of the confirm3 data had been downloaded.

- **Model**: the phase-3 checkpoint at step 7,000. No checkpoint selection.
- **Views**: whole image + centre native crop + four most textured native
  crops; video: 16 frames, same views. Decoding in-process (AVFoundation).
- **Calibration data**: validation split of the training stores plus every
  set scored before this batch (plans dev, benchmarks, confirm1, confirm2).
- **Threshold rule**: mean false-positive rate over real sources with at
  least 100 items.
  - Photo, default and fast profiles: the loosest target among 0.5%, 1% and
    2% at which, on every earlier set that has real images, Veracity's
    precision is above Community Forensics'.
  - Photo, sensitive profile: 5%.
  - Video: 1%.
- **Data**: plan `confirm3` as committed in src/plans.rs: So-Fake-OOD shards
  3, 21, 39; AIGIBench DALLE-3, SD3, SDXL, Imagen3 (1,000 per class, evenly
  spaced); AIGVDBench Gen3, CausVid, RepVideo, IPOC (100 each) against 200
  Kinetics-400 test clips; ComGenVid files 351-450 (AI) and 351-550 (real);
  GenVideo-Val, 200 real and 30 per generator not used before. Files
  byte-identical to anything trained on or scored earlier are dropped.
- **Baselines**: Community Forensics, SwinV2 detector and WaveRep run by the
  Rust harness (validated against their PyTorch pipelines), 32-bit for the
  image baselines, at their shipped thresholds.
- **Scoring**: once. Reported: per-set and pooled precision and recall at
  shipped thresholds for every profile, AUROC, recall at 1% FPR, and recall at
  each baseline's own false-positive rate.

## Fourth confirmation batch (video): what was fixed before scoring it

Written 2026-11-10 22:30 UTC, after the video checkpoint and its thresholds
were frozen and before any of the confirm4 data had been downloaded.

- **Model**: `models/veracity-video.safetensors`, the phase-5 checkpoint at
  step 4,000 (SHA-256 `77010f7cd733b23a73f2…`). No checkpoint selection.
  Photos keep the phase-3 weights; nothing about them changes.
- **Frames and views**: 16 frames, six views each, mean logit – unchanged.
- **Calibration data**: validation split of the video training stores plus
  the real clips of every video set scored before this batch (including the
  third confirmation batch, which is development data for this checkpoint).
- **Threshold**: mean false-positive rate of 0.5% over real sources with at
  least 100 clips – the same target as the photo default. Logit threshold
  1.621.
- **Second opinion** (`--second-opinion`, optional): a clip whose first-stage
  logit lies between -1 and 4 is also scored by the WaveRep network, and is
  flagged when the two clip logits add up to more than 1. Band and threshold
  were chosen on the eight earlier AIGVDBench, ComGenVid and GenVideo-Val
  sets.
- **On those eight development sets** (2,761 AI, 1,750 real clips): WaveRep
  precision 0.993, recall 0.912, 17 real clips flagged; Veracity video 0.993,
  0.937, 19; with the second opinion 0.996, 0.951, 11, with 13.5% of clips
  going to the second stage. These numbers are in-sample.
- **Data**: plan `confirm4` as committed in src/plans.rs: AIGVDBench AccVideo
  (150 clips), Gen3, CausVid, RepVideo, IPOC (100 each, the selection half a
  step away from batch 2's) against 300 Kinetics-400 `test/part_3` clips;
  ComGenVid files 451–600 (AI) and 551–850 (real); GenVideo-Val, 300 real and
  40 per generator not used before. Clips byte-identical to anything trained
  on or scored earlier are dropped. AccVideo, CausVid and IPOC are in no
  training data; Gen-3, RepVideo, Sora, Veo 3, Gen-2 and VideoCrafter are, as
  other clips from the same or other collections.
- **Baseline**: WaveRep run by the Rust port at its shipped threshold.
- **Scoring**: once. Primary outcome: pooled precision and recall of Veracity
  video, alone and with the second opinion, against WaveRep. Also reported:
  the same per set, AUROC, recall at 1% FPR, the share of clips given a second
  opinion, and time per clip.

## Fifth confirmation batch (photos): what was fixed before scoring it

Written 2026-11-11 01:04 UTC, after the photo configuration below was frozen
and before any of the confirm5 data had been downloaded.

- **Model**: `models/veracity.safetensors`, unchanged (the phase-3 weights,
  SHA-256 `2d11bffd77f246416d85…`). Only how they are used changes.
- **Default threshold**: the mean false-positive target over real sources,
  chosen among 0.5%, 1%, 1.5%, 2%, 3% and 5% as the one at which Veracity is
  ahead of both reference detectors on both precision and recall on the most
  development sets (ties to the lower target). That is 1.5% (8 of 13 sets),
  logit threshold 2.312. The previous default (0.5%, logit 4.212) stays
  available as `--strict`; `--sensitive` (5%) and `--fast` (two views, 0.5%)
  are unchanged.
- **Early exit**: views are looked at in the order most-textured crop, whole
  image, centre crop, remaining crops, and an image is decided as soon as the
  mean logit of the views seen is far enough from the threshold: by 7 below
  or 9 above after one view, 5 after two, 3.5 after three, 2.5 after four,
  2 after five. Margins were fitted on the development sets, where they
  change 0.14% of verdicts and use 2.8 views per image on average.
- **On the development sets** (pooled, 44,336 images, in-sample): Community
  Forensics precision 0.975, recall 0.588; SwinV2 detector 0.909, 0.813;
  Veracity default with early exit 0.984, 0.830. On the third confirmation
  batch, which did not inform these choices but had been seen: 0.985, 0.932
  against 0.964, 0.637 and 0.871, 0.918.
- **Data**: plan `confirm5` as committed in src/plans.rs: So-Fake-OOD shards
  15, 30, 42; and 400 evenly spaced images per class from every AIGIBench
  test archive not used before except the five face-swap ones (local edits
  are out of scope): GLIDE, BLIP, IP-Adapter, InstantID, Infinite-ID,
  PhotoMaker, ProGAN, R3GAN, StyleGAN3, StyleGAN-XL, StyleSwim, WFIR. Files
  byte-identical to anything trained on or scored earlier are dropped. Real
  photos that several archives share are counted once in the pooled table.
- **Baselines**: Community Forensics and the SwinV2 detector run by the Rust
  ports, 32-bit, at their shipped thresholds.
- **Scoring**: once. Primary outcome: pooled precision and recall of the
  default profile against both reference detectors. Also reported: the same
  per set, the strict, sensitive and fast profiles, AUROC, recall at 1% FPR,
  and views per image.

## Sixth confirmation batch (photos): what was fixed before scoring it

Written 2026-11-11 05:00 UTC, after the phase-6 photo weights and their thresholds were
frozen and before any of the confirm6 data had been downloaded.

- **Model**: the phase-6 checkpoint at step 8,000 (SHA-256 `9dcb21dc9800591ffd75…`). No
  checkpoint selection. It continues the phase-3 weights (`2d11bffd77f246416d85…`) on the
  original training mix plus the plan `phase6` stores: 15 more generated and
  10 more real shards of Community Forensics (Small), Tiny-GenImage, StyleGAN2
  and StyleGAN3 faces, two IP-Adapter FaceID sets and CelebA-HQ faces.
  Training files identical to any evaluation file were removed first (916).
- **Adoption**: the phase-6 weights replace the phase-3 photo weights. The
  test was that pooled recall on batch 5 rises and pooled precision and
  recall on batch 3 do not fall by more than 0.01. In-sample they give
  0.992 / 0.892 on batch 5 (phase 3: 0.980 / 0.875) and 0.994 / 0.932 on
  batch 3 (0.988 / 0.932). Video keeps its own checkpoint, unchanged.
- **Calibration data**: validation split of every photo training store,
  phase 6 included, plus every photo set scored before this batch, batches 3
  and 5 included.
- **Thresholds**: mean false-positive rate over real sources with at least
  100 items. Default 1.5% (logit 1.692) – the same rule as before picks it
  again, as the target at which Veracity is ahead of both reference detectors
  on both measures on the most development sets (13 of 30). Strict 0.5%
  (3.396), sensitive 5% (-0.776), fast 0.5% (4.197).
- **Early exit**: the schedule fitted for the phase-3 weights, unchanged. On
  the development sets it changes 0.19% of verdicts with these weights and
  uses 2.5 views per image; the rule was to keep it if under 0.5%.
- **What is already known and not expected to change**: on the development
  sets no single threshold puts Veracity ahead of Community Forensics on
  recall on every set. Community Forensics reaches 0.98–1.00 recall on several
  GAN sets while flagging 3–5% of their real photos; at the default threshold
  Veracity flags under 1% there and finds 0.84–0.99.
- **Data**: plan `confirm6` as committed in src/plans.rs: So-Fake-OOD shards
  1, 33, 45; a second sample of 400 per class from each of the twelve
  AIGIBench archives of batch 5, half a step away from that batch's files;
  300 per class from six families of the classic GAN test set of Wang et al.
  (StyleGAN, StyleGAN2, BigGAN, CycleGAN, GauGAN, StarGAN). Files identical
  to anything trained on or scored earlier, or to an earlier set of this
  batch, are dropped. StyleGAN2, StyleGAN3, BigGAN, GLIDE and IP-Adapter are
  now in the training data through other collections; no AIGIBench or classic
  test-set file is.
- **Baselines**: Community Forensics and the SwinV2 detector run by the Rust
  ports, 32-bit, at their shipped thresholds.
- **Scoring**: once. Primary outcome: pooled precision and recall of the
  default profile against both reference detectors. Also reported: the same
  per set with the number of sets on which each reference detector has the
  higher recall; the strict, sensitive and fast profiles; the phase-3 weights
  on the same batch; AUROC; recall at 1% FPR; recall at each reference
  detector's own false-positive rate; views per image.

## Seventh confirmation batch (photos): what was fixed before scoring it

Written 2026-11-11 18:20 UTC, after the photo second opinion below was built
and its band fixed, and before any of the confirm7 data had been downloaded.

- **Why**: on batch 6 the default profile is ahead of Community Forensics
  pooled, but at default thresholds it finds fewer of the AI images on 14 of
  the 19 sets. The second opinion is the attempt to close that set by set.
- **Model**: the phase-6 photo weights, unchanged (SHA-256
  `9dcb21dc9800591ffd75…`), with the default profile, its threshold and the
  early-exit schedule as they stand after the speed changes.
- **Second opinion** (`--photo-second-opinion`, optional): a photo whose
  first-stage calibrated logit lies between -7 and 0.5 (a reported
  probability between 0.09% and 62%) is also scored by the Community
  Forensics network – its released 384-pixel weights (SHA-256
  `b89f36275f3bf5e2b040…`) and preprocessing – and is flagged when half the
  first logit plus that network's logit is positive. Photos outside the band
  keep the first-stage verdict. The stage can add a detection and, for a
  photo the first stage only just flagged, take one away.
- **How the band was chosen**: on the 48 photo sets scored so far (all seen;
  batches 3, 5 and 6 included), over a grid of weight, band and threshold,
  the setting with the most sets on which precision and recall are both at
  or above Community Forensics', among settings that send at most half of
  those photos to the second stage; ties by pooled recall. The 224-pixel
  Community Forensics network was tried as a cheaper second stage and
  rejected (28 of 48 sets at best).
- **On those 48 sets** (47,277 AI, 38,453 real photos; in-sample):
  Community Forensics 0.969 / 0.663, 988 real photos flagged; the default
  profile 0.986 / 0.874, 575, at or above Community Forensics on both
  measures on 24 sets; with the second opinion 0.980 / 0.897, 859, on 46
  sets (the exceptions: SocialRF, precision 0.009 lower; batch 6's StyleGAN3,
  recall 0.005 lower). 48.8% of the photos go to the second stage: 91% of the
  real ones and 15% of the AI ones. Choosing the setting on four batches and
  counting on the fifth gives 43 of 48 (17 of 19 on batch 6).
- **What is expected**: more sets at or above Community Forensics than the
  default manages, not all of them; pooled recall above the default's and
  pooled precision between the default's and Community Forensics'. The mode
  costs the first stage plus the second network on about half of a balanced
  test set: on batch 6 it ran at 311 photos a second on the GPU, against 634
  for the default and 310 for Community Forensics through the same engine.
  On mostly real photos nearly all go to the second stage, and it is slower
  than Community Forensics alone through this engine.
- **Data**: plan `confirm7` as committed in src/plans.rs. So-Fake-OOD shards
  4, 20, 37. From each AIGIBench archive used before, 400 files per class
  evenly spaced over the files earlier batches did not take: the twelve
  archives of batches 5 and 6, and Midjourney, FLUX.1-dev, DALL-E 3, SD3,
  SDXL and Imagen 3 from batches 2 and 3. From the classic test set of Wang
  et al., 300 per class from what batch 6 left of its six families, and from
  five families no batch has used: CRN, IMLE, SAN, SITD and the
  FaceForensics++ deepfakes (face swaps, which neither detector is built
  for; included so that no family is left out by choice). A set with fewer
  than 100 unused files in either class is skipped. Files identical to
  anything trained on or scored earlier, or to an earlier set of this batch,
  are dropped.
- **Baseline**: Community Forensics run by the Rust port, 32-bit, at its
  shipped threshold. The second-opinion result is computed from the default
  profile's scores and those baseline scores, which is the same arithmetic
  as running the mode (checked on batch 6: no verdict differs).
- **Scoring**: once. Primary outcome: for the default profile with the
  second opinion against Community Forensics, pooled precision and recall,
  and the number of sets on which both are at or above Community Forensics'.
  Also reported: the same for the default profile alone and per set; the
  share of photos given a second opinion; real photos flagged; and photos
  per second on this batch for the default, the second-opinion mode and
  Community Forensics (through this engine, and as released in PyTorch), on
  the GPU and the CPU.

## Note on the later speed changes

The early-exit margins for photos were tightened, and early exit over frames
was added for clips, after every batch above had been unsealed. Both
schedules were fitted on development data only. The figures quoted for them
on blind batches (photo batch 6, the video batch) are re-scores of data
already seen, and for photos the choice among four candidate schedules was
made with those re-scores visible. They are reported as such in
[BENCHMARK.md](BENCHMARK.md#five-speed-changes-before-and-after) and do not
replace the blind results.
