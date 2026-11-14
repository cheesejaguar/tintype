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

## Note on the later speed changes

The early-exit margins for photos were tightened, and early exit over frames
was added for clips, after every batch above had been unsealed. Both
schedules were fitted on development data only. The figures quoted for them
on blind batches (photo batch 6, the video batch) are re-scores of data
already seen, and for photos the choice among four candidate schedules was
made with those re-scores visible. They are reported as such in
[BENCHMARK.md](BENCHMARK.md#five-speed-changes-before-and-after) and do not
replace the blind results.
