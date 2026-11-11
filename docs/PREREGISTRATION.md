# What was fixed before each blind batch

Two batches were scored blind: the third confirmation batch (photos, and
video with the phase-3 weights) and the fourth (video, for the dedicated video
checkpoint). Each note below was written before its data was downloaded and
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
