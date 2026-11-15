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
    2% at which, on every earlier set that has real images, Tintype's
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

- **Model**: `models/tintype-video.safetensors`, the phase-5 checkpoint at
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
  precision 0.993, recall 0.912, 17 real clips flagged; Tintype video 0.993,
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
- **Scoring**: once. Primary outcome: pooled precision and recall of Tintype
  video, alone and with the second opinion, against WaveRep. Also reported:
  the same per set, AUROC, recall at 1% FPR, the share of clips given a second
  opinion, and time per clip.

## Fifth confirmation batch (photos): what was fixed before scoring it

Written 2026-11-11 01:04 UTC, after the photo configuration below was frozen
and before any of the confirm5 data had been downloaded.

- **Model**: `models/tintype.safetensors`, unchanged (the phase-3 weights,
  SHA-256 `2d11bffd77f246416d85…`). Only how they are used changes.
- **Default threshold**: the mean false-positive target over real sources,
  chosen among 0.5%, 1%, 1.5%, 2%, 3% and 5% as the one at which Tintype is
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
  Tintype default with early exit 0.984, 0.830. On the third confirmation
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
  again, as the target at which Tintype is ahead of both reference detectors
  on both measures on the most development sets (13 of 30). Strict 0.5%
  (3.396), sensitive 5% (-0.776), fast 0.5% (4.197).
- **Early exit**: the schedule fitted for the phase-3 weights, unchanged. On
  the development sets it changes 0.19% of verdicts with these weights and
  uses 2.5 views per image; the rule was to keep it if under 0.5%.
- **What is already known and not expected to change**: on the development
  sets no single threshold puts Tintype ahead of Community Forensics on
  recall on every set. Community Forensics reaches 0.98–1.00 recall on several
  GAN sets while flagging 3–5% of their real photos; at the default threshold
  Tintype flags under 1% there and finds 0.84–0.99.
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

**What happened when it was run** (added afterwards): the plan ran as
written and no set was skipped. Two sets came out smaller than 400 per
class: WFIR (200 left per class) and SDXL, whose real photos were all
duplicates of real photos already used and were dropped, leaving it AI only.
The scoring run stopped once when the disk filled, before any score had been
read, and was restarted with the reference score files kept; see
[BENCHMARK.md](BENCHMARK.md#blind-batch-7-the-photo-second-opinion).

## The frozen independent split: what was fixed before the first download

Written on 2026-11-14, before any file of the split was downloaded and
before any model was scored on it. It fills in what
[the frozen evaluation runbook](runbooks/frozen-eval.md) left open: the
sources, the sampling, the grouping and what is held out. The measures and
the selection rule are the runbook's and are not changed here.

- **Two proposed sources were dropped before anything was fetched.** The
  survey in the runbook proposed the Forchheim Image Database and VISION as
  authentic sources. Both are real-image sources of Community Forensics
  (its dataset card lists `coco, forchheim, imagenet, imd2020, laion,
  landscapesHQ, vision` and FFHQ), which is the checkpoint the phase-1
  weights started from, and the `commfor-small2` training store holds VISION
  photos. They are not independent of the model, so neither goes in. The
  Forchheim licence also forbids modification, which the processed copies
  would be. RAISE needs a registration form and was not requested.
- **Authentic sources** (none is in a training manifest, a calibration set,
  an archived suite or the Community Forensics source list):

  | Source | What is taken | Terms as read at the source on 2026-11-14 |
  |---|---|---|
  | [HDR+ burst photography dataset](https://hdrplusdata.org/dataset.html), bucket `hdrplusdata`, folder `20171106/results_20171023` | Every `final.jpg`: 3,640 files, 16.4 GB as listed. One per burst, from Nexus 5, 6, 5X, 6P, Pixel and Pixel XL phones; the output of the HDR+ pipeline snapshot of 2017-10-23 at JPEG quality 95, not the file the phone saved | CC BY-SA 4.0 |
  | [FloreView](https://lesc.dinfo.unifi.it/FloreView/) | Every `.jpg` and `.jpeg` under a device's `Nat/jpeg-h264` folder: 4,806 files listed, 46 phones of 11 brands. Flat-field images, the `Extra-Data` folders, HEIC files and videos are left out | CC BY-SA 4.0 |
  | [CSAFE Multi-camera Smartphone Image Database](https://doi.org/10.25380/iastate.26932084.v1), `iPhone14.zip` (20.4 GB, MD5 `dfc01c89b14356141f53d253b72e946c` as listed) | Every JPEG under a `natural` folder: ten iPhone 14 Pro phones, four cameras each, roughly 100 photos per camera. The `blank` (flat-field) folders are left out | CC BY 4.0 |

- **Generated sources.**

  | Source | What is taken | Terms |
  |---|---|---|
  | [ARForensics](https://huggingface.co/datasets/Yanran21/ARForensics) at commit `828c2ee260361574f1c7f24ea50a1698128d6baf`, the four `ARForensics_updated_test_part00N.zip` (20.9 GB) | From each generator's `1_fake` folder, 1,500 files evenly spaced over the sorted file names: LlamaGen, VAR, Infinity, Janus-Pro, RAR, Switti, Open-MAGVIT2. The `0_real` folders are ImageNet and are not extracted | MIT |
  | [GenImage++](https://huggingface.co/datasets/Lunahera/genimagepp) at commit `4160fbc103730771d3641139d3b995a3c2dbc719`, archives `flux_realistic`, `stable_diffusion_v_3_0_realistic` and `flux_krea_amateur` (27.6 GB) | 1,000 files from each, evenly spaced over the sorted file names | CC BY-NC 4.0 |

  No local manifest lists LlamaGen, VAR, Infinity, RAR, Switti or
  Open-MAGVIT2. `janus-7b` appears 95 times, so Janus-Pro is not a new
  family and is marked as seen. FLUX.1-dev and SD3 are trained on; the
  GenImage++ items are an independent source of seen families, and FLUX.1
  Krea is a variant no manifest lists.
- **Grouping columns.** `parent_id` is `<collection>/<path inside the
  collection>`. `scene_id`: for FloreView the location and scene of the
  file name (`floreview-L1S1`), shared by every phone that photographed it;
  empty elsewhere (an HDR+ burst is one photo; no scene or prompt id is
  published for the others). `source_id`: `hdrplus`, `floreview-<brand>`,
  `csafe-iphone14pro`. `generator_family`: `llamagen`, `var`, `infinity`,
  `janus-pro`, `rar`, `switti`, `open-magvit2`, `flux`, `sd3`.
- **Held out to the test set whole**: the families `infinity` and `rar`,
  and the source `csafe-iphone14pro`.
- **Shares**: `--fractions 0.2,0.4,0.4 --seed 1 --together parent,scene`.
  The runbook proposed 30 / 35 / 35. With 8,446 authentic parents to divide
  (HDR+ and FloreView), 35% is 2,956, short of the 3,000 the 0.1% operating
  point needs, and 40% is about 3,380. `fit` gets less; nothing planned
  needs more of it.
- **What the 0.1% point can and cannot show.** Calibration and test each
  get over 3,000 authentic photos, but FloreView's are about 100 photos per
  phone of shared scenes, and CSAFE's are ten units of one model
  photographed in one town. The photos are distinct; the scenes and devices
  are far fewer than 3,000. The 0.1% figures are reported with intervals
  resampled over parents and are to be read with that in mind.
- **Processed copies.** The twelve single operations of the Frontier
  diagnostic (`resize-0.5-bicubic`, `resize-0.25-bicubic`, `jpeg-qv4`,
  `jpeg-qv12`, `blur-s1.0`, `blur-s3.0`, `noise-a10`, `noise-a40`,
  `sharpen-a1.0`, `repost-720-qv9x1`, `repost-960-qv7x4`,
  `repost-960-qv12x1`), rendered by `scripts/stress-photo.py` (FFmpeg), on
  a sample of the parents of `calibrate` and of `test`: the same number of
  authentic and generated parents in each set, evenly spaced over the
  sorted paths within each source and each family, and placed with
  `--existing`. Lossless copies of a 12-megapixel photo are tens of
  megabytes each, so the number per class and set, between 200 and 300, is
  fixed after the originals are on disk and before anything is scored, and
  is written below. It depends on disk space only.
- **Known gaps, stated now.** (a) No real platform round trips: the one
  open collection that has them was dropped above. Reposts are simulated.
  (b) No learned-codec copies: no licence is stated for the CompressAI
  weights, so item 4 of the processing-chains gate cannot be evaluated on
  this split. (c) Encoding and size differ by class: the authentic photos
  are 12-megapixel and larger camera JPEGs, the generated images are 256 to
  1,024 px and their formats are recorded when they are unpacked. The
  processed copies re-encode both classes the same way, which removes the
  encoding difference on those rows but not the size difference; the
  `resize` and `repost` rows bring sizes closer. Clean-row results are to be
  read with this confound in mind.
- **Leakage check**: `tintype overlap` of the new manifests against every
  training store whose files are on disk. The archived evaluation sets are
  manifests without files on this machine, so the byte check cannot cover
  them; none of the sources above is one of their sources.
- **First look**: the released photo model (`models/tintype.safetensors`,
  copied under `runs/`), at its shipped cutoff and at a cutoff refitted on
  `calibrate`, as the incumbent every later candidate is compared with.
  Every look at `test` is logged in the runbook.

**Added on 2026-11-14 after the originals were on disk, before anything
was scored.** Nothing above changed.

- **What arrived.** HDR+ 3,640 files (each checked against the MD5 the
  bucket lists); FloreView 4,806 (the server publishes no checksums; the
  frozen record below is the first); CSAFE 3,996 natural-scene photos from
  an archive whose MD5 matched; ARForensics 10,500 from archives whose
  SHA-256 matched (1,500 of 6,000 per generator); GenImage++ 3,000 (1,000
  of 6,000 per archive, SHA-256 matched). The generated images are all PNG:
  256 px (VAR, RAR, Open-MAGVIT2), 384 px (LlamaGen, Janus-Pro), 512 px
  (Switti), 1,024 px (Infinity and the three GenImage++ sets).
- **FloreView has 36 scenes**, each photographed by every phone, so its
  4,806 photos are 36 groups that move between sets whole.
- **The sets**: `fit` 1,659 authentic and 2,100 generated; `calibrate`
  3,457 and 4,200; `test` 7,326 (3,996 of them CSAFE) and 7,200.
- **Leakage check**: 0 of 25,942 files are byte-identical to one of the
  314,282 files of the 16 training stores on disk.
- **Processed copies: 200 parents per class in `calibrate` and in `test`.**
  The twelve copies of a 12-megapixel photo take about 115 MB, so 400
  authentic parents take about 46 GB. Each class's 200 are divided over
  its sources (authentic) or families (generated) in proportion to their
  sizes and taken evenly spaced over each group's sorted paths
  (`scripts/build-frozen-eval.py stress-sample`). `fit` gets none.
- **Cutoffs are fitted on all authentic rows of `calibrate`**, originals
  and processed copies together, with `tintype calibrate`'s default rule.

## Photo policy comparison on the frozen split: what was fixed before the test set was looked at

Written on 2026-11-14 at 21:40 UTC, on top of commit `6a2c258`. The `fit`
and `calibrate` sets were being scored while this was written; no number
from them had been read, and nothing had been scored or computed on `test`
for this comparison. `test` is not blind: it has been looked at for the
released model and for the two processing-chain arms. The gate is the one
of [the policy-comparison runbook](runbooks/policy-comparison.md), with the
bounds it left open filled in here.

- **What is scored** (Apple M6, GPU engine; the released weights as copies
  under `runs/`, never `models/`): the released photo model with its
  default, sensitive and strict profiles; its six-view scores, taken from
  the incumbent's files of the frozen-evaluation run; Community Forensics
  384 px alone; and, on `test` only, the real two-stage route at the shipped
  band `(-7, 0.5, 0.5, 0)` and at the narrowed band `(-3, 0.5, 0.5, 0)` on a
  copied spec. GPTTrace and the encoder probes (PE, DINOv3, MetaCLIP 2) are
  not on disk and are not downloaded: they are not part of this run.
- **Operating points**: 0.1% and 1% false-positive rate, both always.
  Every policy's cutoff is the empirical one on **all authentic rows of
  `calibrate`**, originals and processed copies together, flagged strictly
  above (`floor(rate x rows)` authentic rows may be flagged), and is applied
  to `test` unchanged. This is the pooled rule of
  `scripts/compare-policies.py`, the same for every policy; it is not the
  balanced-over-sources rule `tintype calibrate` used for the incumbent's
  refitted cutoffs, so the baseline's numbers here need not equal those.
  Recall at empirical cutoffs chosen on `test` itself is printed beside it
  and labelled as a ranking measure.
- **The degraded slice** is every row of `test` whose `transform` is not
  none: the processed copies (2,400 authentic, 2,400 generated).
- **Baseline**: the default profile of the released model at its two
  cutoffs frozen as above.
- **Candidates**, all fixed now:
  1. retuned single profiles: sensitive, strict, and all six views;
  2. Community Forensics alone;
  3. the shipped band, at retuned cutoffs and at its shipped decision;
  4. the narrowed band, likewise;
  5. one fusion with Community Forensics as the second model, chosen on
     `calibrate` from this family: a 50:50 blend of the two logits for
     `abs(first logit) <= m` with `m` in 1, 2, 3; a 50:50 blend for every
     photo; and the same four with weights fitted by plain logistic
     regression on `fit` (inside the gate on both logits, outside it on
     the first logit alone).
  `fit` holds originals only, and there every authentic photo is a large
  camera JPEG and every generated one a small PNG, so the fitted weights
  are fitted under that confound.
- **Choosing on `calibrate`**, with the cutoffs above (in-sample there):
  the member of family 5 with the highest recall on `calibrate`'s
  processed rows at the 1% cutoff, among the members whose recall on those
  rows at the 0.1% cutoff is not below the baseline's. Ties: the smaller
  routed share on `calibrate`, then a 50:50 blend before a fitted one, then
  the smaller margin. If no member is at or above the baseline at 0.1%, the
  member with the highest processed-row recall at 1% is carried to `test`
  and reported as the family's best. The harness prints every member on
  `test`; only the chosen one is a candidate, and none can be promoted from
  what `test` shows.
- **The gate**, per candidate, on `test` at the cutoffs frozen on
  `calibrate`, against the baseline:
  1. recall on the degraded slice at the 1% cutoff is higher by **at least
     5.00 points**;
  2. recall on the degraded slice at the 0.1% cutoff is not lower;
  3. authentic photos hold: at each of the two cutoffs it flags no more of
     `test`'s authentic rows than the baseline does at its cutoff for the
     same target, and its worst authentic source and worst transform
     (slices of 100 authentic rows or more) are not flagged at a higher
     rate than the baseline's worst;
  4. recall on unprocessed generated rows is within one point of the
     baseline's at both cutoffs;
  5. cost: the share of `test` photos sent to the second model is no higher
     than the shipped band's share on `calibrate` (the number is added
     below, before `test` is looked at), and p95 single-image latency and
     peak memory are no higher than the shipped two-stage route's on the
     same hardware. Latency and memory are **not measured in this run** (the
     machine is shared), so that half of item 5 cannot be evaluated now; a
     candidate that meets everything else is reported as such and no
     further.
  A candidate that gains at 1% and loses at 0.1% fails.
- **The narrowed band as an opt-in speed candidate** is judged separately,
  on `test` at the shipped decision (probability 0.5), against the shipped
  band: (a) it sends fewer photos to the second model; (b) it reaches
  Community Forensics at its own cutoff on at least as many slices as the
  shipped band does, a slice being a generator family of 100 generated
  rows or more (it flags at least as many) or an authentic source of 100
  rows or more (it flags no more). The sets of this split hold one class
  each, so the per-set precision-and-recall count of confirmation 7 is not
  defined here and this count stands in for it; (c) its recall on the
  degraded slice is not more than one point below the shipped band's, and
  it flags no more authentic rows. Meeting all three does not ship it: that
  still needs selection on development data and its own entry.
- **Checks and analyses, not candidates**: the two real two-stage runs on
  `test` against the same policy computed from the separate score files
  (verdicts that differ are counted); the first-stage misses each gate
  leaves outside; and the spread of the six view logits as a routing
  feature. No learned gate is fitted.
- **Intervals**: `tintype report --operating-points --cluster-by parent`
  on the harness's score files, at the frozen cutoffs.
- **Looks at `test`**: one scoring pass per system and one evaluation with
  the commands of the runbook, logged there.

**Added on 2026-11-14 at 21:46 UTC, after choosing on `calibrate` and
before anything was scored or computed on `test`.** Nothing above changed.
The choice was made at commit `6fc5ecb` with
`scripts/compare-policies.py ... --report-on calibrate --cohort-by processing`
(the command and its output are in the runbook).

- **The fusion carried to `test`: weights fitted on `fit`, for
  `abs(first logit) <= 3`.** On `calibrate`'s processed rows, at the cutoffs
  frozen on `calibrate`, the baseline finds 1,109 of 2,400 at 0.1% and
  1,905 at 1%. All eight members are at or above 1,109 at 0.1%; at 1% they
  find 1,915 (50:50, margin 1), 1,905 (margin 2), 1,812 (margin 3), 1,792
  (every photo), and fitted 1,915, 1,908, **1,948** and 1,929. The chosen
  one gains 1.79 points at 1% on these rows, in-sample: below the 5 points
  the gate asks for on `test`.
- **The route bound of gate item 5**: the shipped band sends 6,752 of
  12,457 `calibrate` photos (54.20%) to the second model. The chosen fusion
  sends 2,406 (19.31%) there, the narrowed band 1,363 (10.94%).
- For the record, among the single profiles on those rows: strict finds
  1,712 at 0.1% and 1,930 at 1%, six views 1,633 and 1,859, sensitive 983
  and 1,807.

## Calibration and the review outcome on the frozen split: what was fixed before the look at `test`

Written on 2026-11-14, after the cutoffs, the review band and the exit
margins below were fitted on `fit` and `calibrate`, and before anything of
this workstream was computed on `test`. It is the entry
[the calibration runbook](runbooks/calibration-abstention.md) asks for.
Nothing here changes a released file: every spec is a copy under `runs/`.

- **The system is the released photo model, unchanged**: weights, six
  views, mean pooling, GPU engine scores. No new scoring was done. Every
  step is arithmetic on the all-views score files of the incumbent's
  logged look (`fit.all.tsv`, `calibrate.all.tsv`, `test.all.tsv`). The
  fast profile's two views are views 1 and 3 of the six (the whole image
  and the most textured crop), so its score file is those two columns,
  taken offline. There is no frozen video data: the video steps of the
  runbook are not run.
- **`test` is not blind.** The incumbent's look is published, and I have
  read it: the cutoffs refitted with the default rule flag 120 of 9,726
  authentic test files (1.23%) at the 1% target and 18 (0.19%) at 0.1%,
  the held-out phone source carries most of it, and the shipped cutoff
  (raw logit 1.692) flags 16 and 12 of 200 authentic `noise-a10` and
  `noise-a40` copies. Arm A's 1% cutoff below is 1.636, looser than the
  shipped one, so I expect gate 1 to fail on the noise strata at 1%. The
  look measures it; the expectation is written here so that it cannot be
  rewritten afterwards.
- **Three arms, each a set of cutoffs fitted on the authentic rows of
  `calibrate`** (3,457 parents; 200 of them also have twelve processed
  copies, 5,857 rows in all). Raw-logit thresholds on the mean of six views:

  | Arm | Rule | 1% (photo profile) | 0.1% (strict profile) | 5% (sensitive) | 0.5% (fast, two views) |
  |---|---|---|---|---|---|
  | **A, primary** | `--rule bound`, 95%, parents counted once, the highest score of a parent's versions (`--per-parent max`) | 1.636 (24 of 3,457 parents flagged) | 3.504 (0 flagged) | −0.494 (151) | 2.280 (10) |
  | B | `--rule bound`, 95%, the unprocessed original only (`--per-parent original`) | 0.450 (24) | 2.563 (0) | −1.384 (151) | not fitted |
  | C | the default rule (`balanced`), all 5,857 rows as items: the incumbent's refit, identical thresholds | 0.985 (58 of 5,857 rows) | 2.965 (5) | −0.795 (297) | not fitted |

  The operating points are **1% and 0.1%**; the sensitive and fast
  profiles are reported beside them at their shipped targets.
- **Exit margins (arm A).** Fitted on `fit` (3,759 items) for the three
  six-view profiles at arm A's thresholds, tolerance 0.001: after 1 to 5
  views **6.5, 4, 4, 1.75, 1** raw logits (the shipped margins are 6, 3, 2,
  1.5, 1). On `calibrate` the fitted schedule looks at 1.87 views per
  image at the 1% cutoff and changes 19 verdicts of 12,457. Arm A is
  reported under the fitted schedule, with the shipped margins beside it.
  Arms B and C keep the shipped margins.
- **Review band (arm A).** The rule of the runbook: the widest margin, in
  steps of 0.25 calibrated logits, whose share of a stream reviewed at 1%
  prevalence is at most 2% on `calibrate`. That gives **0.75** at the 1%
  cutoff (1.44% reviewed; 1.0 would be 2.24%) and **1.5** at the 0.1%
  cutoff (1.58%; 1.75 would be 3.06%). On `calibrate` these cost 3.70 and
  15.72 points of recall counted with reviewed AI items as misses (86.15%
  to 82.45%, and 79.74% to 64.02%); at the 0.1% cutoff no authentic row is
  flagged with or without the band, so there the band has no false alarm
  to remove. The rule is applied as declared all the same.
- **View disagreement is not used.** Gate 3 already fails on `calibrate`:
  at the 1% cutoff, a margin of 0.25 with "half the views disagree" covers
  97.30%, leaves 16 authentic rows flagged without review and has 83.85%
  recall, and a margin of 0.5 alone covers more (97.56%), leaves fewer
  flagged (10) and has higher recall (84.06%). It is not computed on `test`.
- **What is measured on `test`, once per arm** (9,726 authentic rows of
  7,326 parents; 9,600 generated rows), at the cutoffs above and, labelled
  as a ranking measure, at the empirical 1% and 0.1% cutoffs of `test`:
  recall and authentic files flagged, over all rows, on the clean rows and
  on the degraded slice (every `transform` other than none); for arm A
  also coverage, review rate per class, and recall with every reviewed
  generated item counted as a miss. Intervals resample parents.
- **Strata, fixed now.** Authentic `source` (HDR+, each FloreView brand,
  and `csafe-iphone14pro`, which is held out to `test` whole) and
  `transform` (none and the twelve operations), authentic parents counted
  once within a stratum, a parent being flagged when any of its rows in
  the stratum is.
- **The gate**, as the runbook states it, numbers unchanged:
  1. *Cutoffs transfer*: at a frozen cutoff no stratum is shown to exceed
     the budget (one-sided 95% lower bound over it), and the held-out
     source `csafe-iphone14pro` is shown within twice the budget (upper
     bound at most 2 × budget). Evaluated for arm A at 1% and at 0.1%;
     arms B and C are reported the same way for comparison.
  2. *Coverage is useful*: with the band, at most 2% of a stream at 1%
     prevalence goes to review on `test`.
  3. *Disagreement*: failed on `calibrate`, above.
  4. *Early exit holds*: for the photo, strict and sensitive profiles of
     arm A under the fitted schedule, pooled precision and pooled recall on
     `test` are each within 0.001 of scoring every view, and the mean
     false-positive rate over real sources with at least 100 rows is within
     the profile's target.

  The calibration passes only if 1, 2 and 4 hold at both operating points.
  A stratum the data cannot resolve is reported as not resolved and is not
  a pass: a `transform` stratum has 200 authentic rows, so at 1% it can
  only be shown to exceed, never to be within.
- **Cost.** Nothing is added to inference. Views looked at per image are
  reported. Latency is not measured in this run: the machine is shared
  with a training job.
- **Nothing is chosen on `test`**, and nothing here is a ship decision:
  whatever the verdict, the released cutoffs, margins and specs stay as
  they are.

## Source and family matching on the frozen split: what was fixed before the first look

Written on 2026-11-14 while the two arms below were training, before any
of their checkpoints existed and so before anything of theirs was scored,
on `calibrate`, on `test` or anywhere else. It applies the gate of
[the source and family runbook](runbooks/source-family-matching.md#the-gate)
to [the frozen split](runbooks/frozen-eval.md#as-built-on-2026-11-14) and
fixes what that gate left open.

- **Candidates**, each trained once (`--seed 1`) from the phase-3
  checkpoint on the 25 phase-6 manifests with 8,000 steps, batch 32,
  learning rate 2e-5, warmup 300 and messenger probability 0.25:
  - **B, family-weighted**: `--family-map runs/ws4/families.tsv
    --weights-file runs/ws4/family-weights.tsv --family-weight other=1`.
    Each AI family's share of AI draws is moved to the square root of what
    it is (latent diffusion 72.4% → 39.6%, VQ and autoregressive 1.65% →
    5.99%, hybrid 0.57% → 3.53%; printed by `--print-sampling` before the
    run).
  - **C, source-matched**: `--family-weight reddit-wild=3.85`, which draws
    Reddit-posted generated images as often in the AI class (4.26%) as
    Reddit-posted photos are drawn in the real class (4.25%).
- **Control.** Arm A of the runbook, the current mix, is the recipe
  workstream 1 has already trained twice on the same manifests from the
  same checkpoint with the same budget (`--seed 1` and `--seed 2`, no
  chain flags, built from that workstream's branch at `7156565`). It is
  not trained a third time: those two runs are the control, with their
  `calibrate` and `test` score files as that workstream wrote them. The
  default sampler of this branch draws what that build's sampler draws, so the
  recipes are the same; the builds are not, and that is said wherever the
  comparison is reported. Seed 1 is the control the gate is read against;
  seed 2 shows how much one recipe moves between seeds.
- **Not run.** Arm D needs reconstruction stores, which do not exist (the
  licence checks and the generation come first). The refresh arms need
  stores of new generators that are neither in training nor in the frozen
  split; none is on disk, and the split's own families may not be trained
  on while it is in use.
- **Operating points**: 1% and 0.1%. Each model is read at its own
  cutoffs, fitted on all authentic rows of `calibrate` with `tintype
  calibrate --profile photo --target-fpr 0.01` and `--target-fpr 0.001` on
  copies of its checkpoint. Recall at the empirical 1% and 0.1% cutoffs of
  `test` itself is reported beside it and labelled as a ranking measure.
- **Fitted on `fit`**: nothing. **Selected on `calibrate`**: nothing but
  the two cutoffs. The checkpoint is the last step of each run; no
  checkpoint, weight or share is chosen after seeing a score.
- **Measures**, printed by `scripts/family-gate.py` from the `test` score
  files and manifest:
  - *New families*: `llamagen`, `var`, `infinity`, `rar`, `switti`,
    `open-magvit2`, which no training manifest lists. `infinity` and `rar`
    are also absent from `calibrate` and are reported by name. *Older
    families*: `janus-pro`, `flux`, `sd3`.
  - A family's recall is over its unprocessed test items; macro recall is
    the unweighted mean over families (every family here has more than 100
    such items).
  - *Processed slice*: every `transform` other than none: 2,400 generated
    and 2,400 authentic rows from 200 parents per class.
  - *Authentic sources*: unprocessed photos flagged per `source_id`,
    `csafe-iphone14pro` (held out whole) among them.
  - Differences are candidate minus control seed 1, with a 95% interval
    from a bootstrap over parents, and are set beside control seed 2 minus
    control seed 1.
- **The gate.** A candidate passes if, at the cutoffs frozen on
  `calibrate`:
  1. new-family macro recall is higher than the control's at 1%, the
     interval of the difference is above zero, **and the difference is
     larger than the absolute difference between the two control seeds**
     on that measure; and it is not lower at 0.1%;
  2. at both rates, older-family macro recall and recall on processed
     copies are not lower than the control's by more than one point;
  3. at both rates, the false-positive rate on processed authentic copies
     is not higher than the control's.
  The seed condition in item 1 is added here: each candidate is one run,
  and a difference no larger than what the control shows between its own
  seeds is reported as within seed-to-seed movement, not as a pass. A
  difference in items 2 and 3 that is inside the two control seeds' range
  is reported as such.
- **Cost.** Neither arm changes the network, the views or the spec, so
  latency, memory and the route rate are the control's by construction and
  are not measured.
- **What is expected, said now.** Neither arm adds an image of any new
  family. Arm B draws the VQ, autoregressive and hybrid generators that
  training does have about four times as often and latent diffusion about
  half as often; if family weighting helps anywhere on this split it is on
  the autoregressive families, and if it costs anywhere it is on `flux`
  and `sd3`. Arm C changes one generator's share; the split has no real
  platform round trips, so the simulated repost rows are the only place it
  could show, and little is expected.
- **Diagnostic, which chooses nothing**: both arms scored on confirmation
  batch 7 with every view and compared with the two control runs at
  empirical false-positive rates. The frozen split's clean rows are camera
  originals against small PNGs and cannot show a loss of ranking on
  ordinary photo sets; batch 7 can. It is archived development data.
- **Looks at `test`**: one per candidate, logged in the runbook. `test`
  has been looked at before for the released model and for workstream 1's
  arms. A pass makes an arm a ship *candidate* under
  [the selection rule](runbooks/frozen-eval.md#the-selection-rule), which
  still asks for a second seed, a re-calibration, an early-exit re-check
  and a fresh blind batch.

**Added on 2026-11-14 at about 19:20, before the run it describes.** A
second seed of arm B (family-weighted): the same command with `--seed 2`,
calibrated on `calibrate` at 1% and 0.1% and looked at once on `test` with
every view, then confirmation 7 as a diagnostic. Nothing in the gate or
its conditions changes. It is read beside the control's second seed and
beside the two-seed means, and the question it answers is whether the
seed-1 differences (new-family macro recall, processed recall, authentic
copies flagged) repeat.

## View pooling and weight interpolation on the frozen split: what was fixed before the look at `test`

Written on 2026-11-14 for workstream 10
([runbook](runbooks/interpolation-pooling.md)), after the incumbent's
tables on the frozen split existed and before anything of this workstream
was computed on `test`. The selection rules below were written before the
`calibrate` results they select on were read. The gate is the runbook's, a
proposed criterion: *interpolation recovers legacy-GAN recall, or pooling
recovers clean recall, at a fixed false-positive rate; neither may weaken
robust (distorted) performance, current generators or reposted images.*
Nothing ships from either part whatever the outcome.

Common to both parts:

- **Operating points**: 1% and 0.1% false-positive rate. The result is at
  cutoffs frozen on `calibrate` (all authentic rows, originals and
  processed copies together, `calibrate`'s default rule: balanced over the
  real sources with at least 100 items). Recall at empirical cutoffs found
  on the reported rows themselves is given beside it and labelled.
- **Slices**: *clean* (`transform` none); *degraded* (every other
  `transform`); *reposts* (the three `repost-*` transforms); the seen
  families `flux` and `sd3` as "current generators"; the held-out families
  `infinity` and `rar`; the authentic sources.
- **Intervals**: 95% bootstrap over parents
  (`report --operating-points --cluster-by parent`); for authentic rows
  the one-sided 95% Clopper–Pearson upper bound.
- **"Not worse"** on a slice means: recall not below the lower end of the
  reference's interval on that slice, and authentic rows flagged not above
  the reference's upper bound. **"Better"** means recall above the
  reference's at both operating points, and at one or both of them the
  reference's recall below the lower end of the candidate's interval.
- **Cost**: views per image with early exit, replayed. Latency is not
  measured now (the machine is shared); it stays a step in the runbook.
- The caveats of the frozen split apply: authentic originals are camera
  JPEGs of 12 megapixels or more and generated originals are PNGs of 256
  to 1,024 px; no real platform round trips and no learned-codec copies;
  `test` has been looked at before (the incumbent, the processing-chain
  arms), and each look of this workstream is logged in the runbook.

**Part B: pooling of the released model's six view logits.** No scoring:
the incumbent's all-views score files of `fit`, `calibrate` and `test`.

- **Candidates**: trimmed mean, median, top 2, top 3, maximum, and
  per-view weights fitted on `fit`; each looking at the views in the
  shipped order (2,0,1,3,4,5) and in an order chosen greedily on `fit`;
  each with one early-exit margin per stage and with margins fitted
  separately above and below. The **reference** is the mean in the shipped
  order with one margin per stage, refitted exactly as the candidates are.
- **On `fit`**: the weights, the greedy order, a threshold for a 1% target
  and the early-exit margins (tightest on a grid of 0 to 12 in steps of
  0.25 that keep pooled precision and recall within 0.001 of all views).
- **On `calibrate`, to choose at most one candidate**: with early exit,
  recall at 1% and at 0.1% empirical false-positive rate on the clean rows
  and on the degraded rows separately, and views per image over all rows.
  A candidate is eligible when (a) its clean recall is above the
  reference's at both rates, (b) its degraded recall is not below the
  reference's at either rate, and (c) it needs no more than twice the
  reference's views per image. Among the eligible, the one with the
  highest clean recall at 1% is chosen (then at 0.1%, then fewer views).
  If none is eligible, no candidate goes to `test`, the pooling half of
  the gate has failed on `calibrate`, and `test` is read once for the
  refitted mean alone, as the early-exit re-check of the refit itself.
- **On `test`, once**: for the chosen candidate and the mean, threshold
  and margins refitted on `calibrate` for 1% and for 0.1%.
- **The gate passes for pooling** when, at both frozen cutoffs and with
  early exit, the candidate is better than the mean on the clean rows, not
  worse on the degraded rows, the reposts, the current generators and the
  held-out families, flags no more authentic rows than the mean's upper
  bound on clean and on degraded rows, and stays within twice the mean's
  views per image. Otherwise it fails.

**Part A: interpolation from the released weights towards an earlier
checkpoint.** Candidates `(1 − alpha) · released + alpha · B` under
`runs/`, written by `tintype interpolate`, run through Tintype's six
views with the mean of all views (an interpolant has no fitted early
exit).

- **Anchors `B`**: Community Forensics 224 (the phase-1 initialiser, the
  one the report names; tensor prefix `vit.`), and the phase-3 weights
  (the checkpoint phase 6 continued from; SHA-256 `2d11bffd…`). The second
  anchor is an addition to the runbook's grid, made here, before any score.
- **`alpha`**: 0.10, 0.20, 0.30, 0.50 for each anchor: eight candidates.
  `alpha` 0 is the incumbent.
- **Legacy-GAN recall cannot be measured on the frozen split**: it holds
  no GAN family. It is measured on the twelve GAN sets of Confirmation 7
  (4,000 generated, 3,858 authentic; BigGAN, CycleGAN, GauGAN, StarGAN,
  StyleGAN, StyleGAN2, ProGAN, R3GAN, StyleGAN-XL, StyleGAN3, StyleSwin,
  WFIR), which are development data that has been read many times. So the
  "recovers legacy-GAN recall" half of the gate can be shown on
  development data only; the protective half is read on the frozen split.
- **On `calibrate`, to choose at most one candidate**: each candidate is
  scored on `calibrate` and on the GAN sets with all six views. Its
  cutoffs for 1% and 0.1% are fitted on the authentic rows of `calibrate`.
  A candidate is eligible when, at both cutoffs, (a) its recall on the GAN
  sets is above `alpha` 0's at its own cutoffs, and (b) its recall on the
  clean generated rows of `calibrate` and on the degraded generated rows
  of `calibrate` are each no more than 0.5 percentage points below
  `alpha` 0's. Among the eligible, the one with the highest GAN recall at
  the 1% cutoff is chosen. If none is eligible, `test` is not read for
  Part A and the gate has failed on `calibrate`.
- **On `test`, once**, for the chosen candidate, at its cutoffs frozen on
  `calibrate`, against `alpha` 0 from the incumbent's all-views scores
  with cutoffs refitted the same way.
- **The gate passes for interpolation** when the chosen candidate's GAN
  recall is above `alpha` 0's at both cutoffs (development data), and on
  `test` it is not worse than `alpha` 0 on the clean rows, the degraded
  rows, the reposts, the current generators and the held-out families,
  and flags no more authentic rows than `alpha` 0's upper bound on clean
  and on degraded rows. Otherwise it fails; a pass would rest on
  development data for its first half and is to be read so.

**Added on 2026-11-14 after the `calibrate` stage of Part B, before the
look at `test`.** Nothing above changed. On `calibrate`, with thresholds,
weights, orders and margins fitted on `fit`, the reference (the refitted
mean) has clean recall 95.02% at 1% and 72.48% at 0.1% empirical
false-positive rate, degraded recall 72.25% and 41.00%, and 1.54 views
per image. **No candidate is eligible**: none of the 24 has clean recall
above 95.02% at 1% (the highest is the median in the shipped order,
95.00%), so condition (a) fails for every one. By the rule above no pool
goes to `test`, the pooling half of the gate has failed on `calibrate`,
and `test` is read once for the mean alone: threshold and margins
refitted on `calibrate` for 1% and for 0.1%, beside the same thresholds
with the shipped margins (the incumbent's recalibrated specs, replayed)
and with all six views.

**Added on 2026-11-14 after the `calibrate` stage of Part A, before the
look at `test`.** Nothing above changed. Eligible by the rule: the
phase-3 anchor at `alpha` 0.10, 0.20 and 0.30. Not eligible: the phase-3
anchor at 0.50 (GAN recall below `alpha` 0's at both cutoffs) and every
Community Forensics candidate (degraded recall on `calibrate` more than
0.5 points below `alpha` 0's: −0.63 points at the 1% cutoff for `alpha`
0.10, more for the others). **Chosen: the phase-3 anchor at `alpha`
0.30**, the eligible candidate with the highest GAN recall at the 1%
cutoff (95.67% against 95.45% for `alpha` 0; 88.00% against 87.38% at
the 0.1% cutoff). It goes to `test` once.

Stated now, before that look, because it bears on how a pass would read:
condition (a) compares GAN recall at each candidate's own cutoff frozen
on `calibrate`, and that cutoff does not hold the false-positive rate on
the GAN sets' own authentic images fixed. The chosen candidate flags 81
of their 3,858 authentic images at its 1% cutoff where `alpha` 0 flags
74, and at empirical cutoffs on the GAN sets its recall is *lower* than
`alpha` 0's (89.03% against 91.15% at 1%, 60.62% against 65.25% at
0.1%). The Community Forensics candidates are the ones whose GAN recall
rises at a matched rate (93.40% to 97.30% at 1%), and they are excluded
by condition (b). The rule is applied as written; the write-up reports
both readings.

## Note on the later speed changes

The early-exit margins for photos were tightened, and early exit over frames
was added for clips, after every batch above had been unsealed. Both
schedules were fitted on development data only. The figures quoted for them
on blind batches (photo batch 6, the video batch) are re-scores of data
already seen, and for photos the choice among four candidate schedules was
made with those re-scores visible. They are reported as such in
[BENCHMARK.md](BENCHMARK.md#five-speed-changes-before-and-after) and do not
replace the blind results.
