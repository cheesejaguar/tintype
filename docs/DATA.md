# Data

Every set is public and is fetched from its original host by a named plan:

```bash
tintype data list            # the plans
tintype data train-images    # run one
tintype data confirm3 --dry-run
```

The plans live in [src/plans.rs](../src/plans.rs); that file is the exact
recipe (repositories, shards, sampling rules). A plan downloads, cuts archives
down to the selection, and writes a `manifest.tsv` per store
(`path  label  generator  dataset`, label 1 = AI-generated). Image bytes are
never re-encoded on the way in.

## What is pinned

A plan reads every repository at a fixed commit and checks every file it
downloads against a recorded SHA-256 where one exists. A file already on disk
is reused only if its SHA-256 matches; one that does not is downloaded again,
and a download that does not match is discarded with an error.
`tintype data <plan> --check` resolves a plan without downloading anything
and compares whatever of it is already on disk with those digests.

| Source | Revision | SHA-256 per file |
|---|---|---|
| 28 Hugging Face datasets ([the table](../src/fetch.rs), `HF_REVISIONS`) | pinned to a commit | yes for every file stored through LFS (all the data files): the digest Hugging Face lists at that commit. Small files kept in git itself (such as the frontier benchmark's `metadata.csv`) are pinned by the commit and checked by size only |
| AIGIBench and CNNDetection archives that a plan samples by HTTP range requests | pinned to a commit | no: only the chosen members are fetched, so the archive as a whole is never hashed; each member is checked against its zip CRC-32 |
| GenVideo-Val and GenVideo-100K (ModelScope) | pinned to a commit | yes, the digest ModelScope lists |
| Community Forensics 224 / 384 and SwinV2 reference weights (Hugging Face) | pinned to a commit | yes, the LFS digest |
| WaveRep reference weights (GRIP server) | **not pinnable**: one unversioned URL | yes: the digest of the copy the reported numbers used; a different file is refused |
| RAISE-1k originals (GRIP server, `benchmarks` plan) | **not pinned**: one unversioned URL | **no**: the server publishes none and the copy used was deleted after extraction. On 2026-11-11 the server reported 1,646,333,296 bytes, last modified 2025-12-28 |

**Whether the pins are what was downloaded.** The plans read `main` (and
ModelScope's `master`) when the data was fetched, between 2026-11-10 and
2026-11-11 (US Pacific time), going by when each store was created. The pins
were added on 2026-11-11: each is the commit at the head of the branch that
day. Every one of those head commits is older than the first download (the
newest, OpenFake's, is dated 2026-11-05; ModelScope's are from 2024 and
2025). So unless an upstream history was rewritten in between, the pinned
commits are the revisions that were downloaded. This rests on those dates,
not on hashes: most downloads (parquet shards, archives) were deleted once
their images were extracted. Where the downloaded files are still on disk,
`--check` confirms it by hash: all 18,813 such files match the pinned
digests (VidGuard-R1 8,853, the frontier benchmark 3,378, this-person-does-not-exist
3,000, ComGenVid 1,850, Rapidata 1,492, Sunain gameplay 240), as do the
four reference weights. None differ.

Pinning does not undo the differences described under the phases below: the
stores deleted between phases, and the clips the current video decoder cannot
open.

## Training

Three phases, each starting from the previous weights.

### Phase 1 (14,000 steps from the Community Forensics 224 px checkpoint)

| Plan | Store | Source | Real | AI |
|---|---|---|---|---|
| `train-images` | `openfake`, `openfake2` | [OpenFake](https://huggingface.co/datasets/ComplexDataLab/OpenFake) `core/train`, 9 shards | 24.0 k (LAION, Pexels) | 17.8 k, ~75 generators |
| | `openfake-reddit` | OpenFake `supplement/reddit/2026-11`, shards 0–4 | 3.0 k | 15.8 k posted to Reddit |
| | `frontier` | [frontier-synthetic-images-2026](https://huggingface.co/datasets/Thermostatic/frontier-synthetic-images-2026), 25 files | – | 18.1 k |
| | `nano-banana` | [bitmind/nano-banana](https://huggingface.co/datasets/bitmind/nano-banana), shards 0–4 | – | 1.5 k |
| | `defactify` | [Defactify](https://huggingface.co/datasets/Rajarshi-Roy-research/Defactify_Image_Dataset) train split | 7.0 k (COCO) | 35.0 k, 5 generators |
| | `commfor-small` | [Community Forensics (Small)](https://huggingface.co/datasets/OwensLab/CommunityForensics-Small), 20 shards | 23.9 k (COCO, FFHQ, LHQ) | 35.9 k, 223 generators |
| `train-video` | `vidguard` | [VidGuard-R1 sampled 10k](https://huggingface.co/datasets/kyoungjunpark/VidGuard-R1-sampled-video-10k) | 3.9 k ActivityNet | 2.5 k HunyuanVideo, 2.5 k CogVideoX |
| | `genbuster` | [GenBuster-200K-mini](https://huggingface.co/datasets/l8cv/GenBuster-200K-mini) | 4.8 k OpenVid | 5.0 k: EasyAnimate, CogVideoX, LTX-Video, HunyuanVideo |
| | `genvideo100k` | [GenVideo-100K](https://modelscope.cn/datasets/cccnju/GenVideo-100K) | – | 10 k OpenSora, 10 k ZeroScope |
| | `aigvd` | [AIGVDBench](https://huggingface.co/datasets/AIGVDBench/AIGVDBench) | – | 1 k each: Pika, Vidu, Luma, SVD, AnimateDiff, Pyramid-Flow |
| | `k400`, `pexels` | [Kinetics-400](https://huggingface.co/datasets/kiyoonkim/kinetics-400-targz) `val/part_0–1`; [Pexels subset](https://huggingface.co/datasets/zengxianyu/open-sora-pexels-subset) shard 0 | 2.1 k clips | – |

Six generators were kept out of phases 1 and 2 so that they would be unseen
in So-Fake-OOD: GPT-Image-1.5, gpt-image-2, Recraft v3, Seedream 3, Ideogram
(v2) and FLUX.1 pro. The benchmark report has the per-generator table from
that stage.

### Phase 2 (7,000 steps; plan `phase2`)

| Store | Source | Real | AI |
|---|---|---|---|
| `openfake-test-reals` | OpenFake `core/test` shards 1–2, real rows only | 7.1 k (ImageNet, DOCCI) | – |
| `k400b`, `pexelsb` | Kinetics-400 `val/part_2–3`; Pexels shards 1–4 | 2.3 k clips | – |
| `aigvd2` | AIGVDBench | – | 1 k Open-Sora, 1 k CogVideoX 1.5, 700 HunyuanVideo, 500 Wan, 500 Kling |

plus the messenger-style augmentation (long side capped at 960–1700 px, then
JPEG quality 55–90) on a quarter of the samples.

### Phase 3 (7,000 steps; plan `phase3`) – the released weights

| Store | Source | Real | AI |
|---|---|---|---|
| `frontier-heldout` | frontier-synthetic-images-2026: the six held-out generators and Recraft v2 | – | 3.3 k |
| `sofake-set` | [So-Fake-Set](https://huggingface.co/datasets/saberzl/So-Fake-Set) train shards 0, 200, …, 2200; real and fully synthetic rows | 3.0 k social-media | 4.3 k, ~30 generators |

Phase 3 ran with two of the original six `core/train` shards in the `openfake`
store; the other four had been deleted to free disk between phases. The
`train-images` plan fetches all six.

### Phases 4 and 5 (7,000 + 4,000 steps; plans `phase4`, `phase5`) – the released video weights

Video gets its own checkpoint, continued from the phase-3 weights on every
video store above, the three phase-2/3 image stores still on disk
(`openfake-test-reals`, `sofake-set`, `frontier-heldout`), and:

| Plan | Store | Source | Real | AI |
|---|---|---|---|---|
| `phase4` | `aigvd3` | AIGVDBench | – | 1.1 k Sora, 0.9 k Gen-2, 0.9 k Jimeng, 0.8 k VideoCrafter, 0.6 k LTX-Video, 0.6 k EasyAnimate |
| | `k400c` | Kinetics-400 `train/part_0–1`, first 1,000 clips each | 2.0 k | – |
| | `ucf101` | [UCF101](https://huggingface.co/datasets/quchenyuan/UCF101-ZIP), evenly spaced | 1.5 k (320×240) | – |
| `phase5` | `rapidata` | Rapidata text-to-video preference sets ([Marey](https://huggingface.co/datasets/Rapidata/text-2-video-human-preferences-moonvalley-marey), [Veo 3.1](https://huggingface.co/datasets/Rapidata/text-2-video-human-preferences-veo3.1)) | – | 1.5 k: Sora, Veo 2 / 3 / 3.1, Gen-3 Alpha, Kling 2.1, Seedance 1 Pro, Luma Ray 2, Pika, Pika 2.2, Mochi 1, Marey, HunyuanVideo, Wan |
| | `aigvd4` | AIGVDBench | – | 0.95 k Gen-3, 0.8 k RepVideo |
| | `animation` | [Sakugabooru 2025](https://huggingface.co/datasets/trojblue/sakugabooru2025), shards 4, 10, 102, 103 | 1.05 k hand-drawn animation clips | – |
| | `cartoons` | [Human-Animal-Cartoon](https://huggingface.co/datasets/hdong51/Human-Animal-Cartoon), cartoon videos | 1.1 k | – |
| | `gameplay` | [Sunain gameplay](https://huggingface.co/datasets/Sunain-Labs/sunain-gameplay), 20 clips of each of 12 games | 240 | – |

The AIGVDBench clips that earlier confirmation batches were scored on are
removed from these selections (247 and 54 files). Animation, cartoons and
gameplay are there because real footage that did not come from a camera was
what the phase-3 model most often mistook for generated video.

**What the current build cannot reproduce exactly.** The released video
weights were trained while the tool could still hand a file to `ffmpeg`.
Since then every external call has been removed and video is read only by
the system decoder, which opens neither AVI nor WebM. So the `phase4` plan no
longer fetches UCF101 (1,500 AVI clips) and `phase5` takes only the MP4 clips
of Sakugabooru (846 of the 1,053): 1,707 of the roughly 67,000 training clips.
The H.264 re-encoding augmentation also changed encoder, from x264 to the
system's. HEVC files tagged `hev1`, which the system decoder refuses as they
are (all of GenBuster, some AIGVDBench clips), are read through a temporary
copy with the tag corrected.

### Phase 6 (8,000 steps; plan `phase6`) – the released photo weights

Continued from the phase-3 weights on every store of phases 1–3 (the
`train-images` plan fetched again, all nine OpenFake shards this time) and:

| Store | Source | Real | AI |
|---|---|---|---|
| `commfor-small2` | Community Forensics (Small), 25 more shards | 29.0 k | 44.9 k, about 830 generators (BigGAN, GigaGAN, StyleSwin, StyleGAN3, Gansformer among them) |
| `tiny-genimage` | [Tiny-GenImage](https://huggingface.co/datasets/TheKernel01/Tiny-GenImage) train split | 14.0 k ImageNet | 14.0 k, 2.0 k each: ADM, BigGAN, GLIDE, VQDM, Wukong, SD 1.5, Midjourney |
| `stylegan3` | [StyleGAN3 1024 px](https://huggingface.co/datasets/34data/StyleGAN3_1024x1024_images) | – | 2.0 k |
| `tpdne` | [this-person-does-not-exist-10k](https://huggingface.co/datasets/javi22/this-person-does-not-exist-10k), evenly spaced | – | 3.0 k StyleGAN2 faces |
| `ip-adapter` | IP-Adapter FaceID sets on [SD 1.5 / SDXL](https://huggingface.co/datasets/34data/ip-adapter-faceid) and [SD 3.5](https://huggingface.co/datasets/34data/sd35-ip-adapter-faceid) | – | 3.0 k each |
| `celeba-hq` | [CelebA-HQ](https://huggingface.co/datasets/mattymchen/celeba-hq), three train shards | 7.5 k faces | – |

These collections share source photos (ImageNet, COCO) with evaluation sets:
916 training files that were byte-identical to an evaluation file were removed
before training, after which none of 314,282 training files matched any of
75,844 evaluation files.

Sampling: both classes equally often; within a class a source is drawn with
weight `count^0.5`. Three percent of every store, chosen by path hash, is held
out for validation and calibration.

## How the evaluation was staged

Each stage's sets were scored only after the decisions listed before it were
frozen.

1. **Phase-1 model** → scored on the *benchmark* sets (plan `benchmarks`).
   Findings: threshold too strict, weak on social-media copies, video false
   positives on flat scenes.
2. Decisions: phase-2 training, texture-ranked crops, source-balanced
   calibration. → **Phase-2 model** scored on *confirmation batch 1*
   (`confirm1`) at a 1% photo target. Finding: image precision level with
   Community Forensics.
3. Decision: photo target 0.5%. → same model scored on *confirmation batch 2*
   (`confirm2`).
4. Decisions: phase-3 training (stop holding out the six generators, add
   So-Fake-Set), in-process video decoding, a third "sensitive" operating
   point, and a threshold rule written down in advance (reproduced in the
   benchmark report). → **Phase-3 model** scored once on *confirmation
   batch 3* (`confirm3`). These are the numbers to trust for the released
   weights.

5. Findings on video: the phase-3 weights were level with WaveRep rather
   than ahead (higher recall, lower precision), and the real clips they
   flagged were mostly cartoons, 3D animation and motion graphics. Decisions:
   a dedicated video checkpoint (phases 4 and 5), the video target lowered to
   0.5%, an optional second-opinion stage, and a second written protocol.
   → **Video checkpoint** scored once on *confirmation batch 4* (`confirm4`).
   These are the numbers to trust for the released video weights.
6. Findings on photos: at the 0.5% default the model was more precise than
   the SwinV2 detector but found fewer AI images, and on the CPU the six
   views made it slower than Community Forensics in the same engine.
   Decisions, with the weights unchanged: the default threshold moved to the
   target at which Tintype is ahead of both reference detectors on the most
   development sets (1.5%; the 0.5% point stays as `--strict`), and early
   exit over the views. → scored once on *confirmation batch 5* (`confirm5`).
   These are the numbers to trust for the default photo profile with the
   phase-3 weights.
7. Finding on batch 5: on GAN-era and identity-adapter sets Community
   Forensics found more of the AI images. Decision: phase-6 training, with
   thresholds and early exit set by the same rules. → **Phase-6 photo
   weights** scored once on *confirmation batch 6* (`confirm6`), with the
   phase-3 weights alongside. These are the numbers to trust for the released
   photo weights. Batches 3 and 5 are development data for them.

For the phase-3 model every set before batch 3 is development data: it was
seen while deciding, and its real items are in the calibration set. For the
video checkpoint the same holds for every video set before batch 4, batch 3
included.

## Evaluation sets

### Plan `dev`

| Set | Use |
|---|---|
| OpenFake `core/test` shard 0, minus the held-out generators | Sanity checks, calibration |
| 400 Kinetics-400 `test/part_0` clips + 400 SEINE clips (AIGVDBench) | Video sanity checks, calibration |

### Plan `benchmarks`

| Set | Source | Real | AI |
|---|---|---|---|
| So-Fake-OOD | [saberzl/So-Fake-OOD](https://huggingface.co/datasets/saberzl/So-Fake-OOD) shards 0, 12, 24, 36; `REAL` and `FULL_SYNTHETIC` rows | 2,075 Reddit / Tumblr / Bluesky | 2,912, 15 commercial generators |
| Synthbuster+ | [synthbuster-plus](https://huggingface.co/datasets/marco-willi/synthbuster-plus) test + validation; reals are the full [RAISE-1k](https://www.grip.unina.it/download/prog/B-Free/extended_synthbuster/) | 1,000 | 4,679, 13 generators |
| AIGIBench CommunityAI | [AIGIBench](https://huggingface.co/datasets/HorizonTEL/AIGIBench) `test/CommunityAI.zip` | 6,000 | 6,000 |
| Frontier 2026-10 images | [WasItAIGenerated benchmark](https://huggingface.co/datasets/wasitaigeneratedcom/frontier-image-video-bench-2026-10), public split | 484 | 261, 19 generators; plus WhatsApp / Instagram / re-shared copies |
| Frontier 2026-10 video | same | 100 | 101, 12 video models |
| ComGenVid | [OmerXYZ/comgenvid](https://huggingface.co/datasets/OmerXYZ/comgenvid), files 1–150 per class | 150 MSVD | 150 Sora, 150 Veo 3 |
| GenVideo-Val | [GenVideo](https://modelscope.cn/datasets/cccnju/Gen-Video), evenly spaced subset | 300 MSR-VTT | 485, 10 generators |

### Plan `confirm1`

| Set | Source | Real | AI |
|---|---|---|---|
| So-Fake-OOD, batch 1 | shards 6, 18 | 953 | 1,507 |
| AIGIBench SocialRF | `test/SocialRF.zip` | 3,000 | 3,000 |
| Synthbuster+, new images | `train` shards 0–4 (no real images in them) | – | 1,725: DALL·E 2, DALL·E 3, Firefly |
| AIGVDBench commercial | 150 evenly spaced clips each of Gen-2, Sora, Jimeng; Kinetics-400 `test/part_1` | 200 | 450 |
| ComGenVid, batch 1 | files 151–250 (AI), 151–350 (real) | 200 | 100 Sora, 100 Veo 3 |
| Frontier video, messenger copies | `videos_social/messenger` | 100 | 101 |
| GenVideo-Val, batch 1 | evenly spaced clips not used before | 300 | 456 |

### Plan `confirm2`

| Set | Source | Real | AI |
|---|---|---|---|
| So-Fake-OOD, batch 2 | shards 9, 27 | 980 | 1,531 |
| AIGIBench Midjourney, FLUX.1-dev | 1,500 evenly spaced files per class from each archive | 1,500 each | 1,500 each |

### Plan `confirm3`

| Set | Source | Real | AI |
|---|---|---|---|
| So-Fake-OOD, batch 3 | shards 3, 21, 39 | 1,542 | 2,172 |
| AIGIBench DALL·E 3, SD3, SDXL, Imagen 3 | 1,000 evenly spaced files per class from each archive | 1,000 each | 1,000 each |
| AIGVDBench, batch 2 | 100 evenly spaced clips each of Gen-3, CausVid, RepVideo, IPOC; Kinetics-400 `test/part_2` | 200 | 400 |
| ComGenVid, batch 2 | files 351–450 (AI), 351–550 (real) | 200 | 100 Sora, 100 Veo 3 |
| GenVideo-Val, batch 2 | evenly spaced clips not used before (no Sora clips were left) | 200 | 270, 9 generators |

AIGIBench pairs SD3 and SDXL with the same 1,000 real photos; pooled tables
count them once.

### Plan `confirm4` (video only)

| Set | Source | Real | AI |
|---|---|---|---|
| AIGVDBench, batch 3 | 150 evenly spaced AccVideo clips; 100 each of Gen-3, CausVid, RepVideo, IPOC, half a step away from batch 2's; Kinetics-400 `test/part_3` | 300 | 496 after removing 54 clips that are in the training selection (50 Gen-3, 4 RepVideo) |
| ComGenVid, batch 3 | files 451–600 (AI), 551–850 (real) | 300 | 150 Sora, 150 Veo 3 |
| GenVideo-Val, batch 3 | evenly spaced clips not used before | 300 | 360, 9 generators |

### Plan `confirm5` (photos only)

| Set | Source | Real | AI |
|---|---|---|---|
| So-Fake-OOD, batch 4 | shards 15, 30, 42 | 1,479 | 2,279 |
| AIGIBench, the twelve test archives not used before, minus the five face-swap ones | 400 evenly spaced files per class from each of GLIDE, BLIP, IP-Adapter, InstantID, Infinite-ID, PhotoMaker, ProGAN, R3GAN, StyleGAN3, StyleGAN-XL, StyleSwim, WFIR | 4,799 | 4,800 |

The AIGIBench files were read out of the remote archives by HTTP range
requests, without downloading the archives. No real photo is shared between
these twelve sets.

### Plan `confirm6` (photos only)

| Set | Source | Real | AI |
|---|---|---|---|
| So-Fake-OOD, batch 5 | shards 1, 33, 45 | 1,361 | 2,040 |
| AIGIBench, second sample | 400 files per class from each of the twelve archives of batch 5, half a step away from that batch's files | 4,555 | 4,795 |
| Classic GAN test set | [CNNDetection](https://huggingface.co/datasets/sywang/CNNDetection) `CNN_synth_testset.zip`: 300 evenly spaced files per class of StyleGAN, StyleGAN2, BigGAN, CycleGAN, GauGAN, StarGAN | 1,780 | 1,793 |

Files identical to anything trained on or scored earlier, or to a file in an
earlier set of this batch, were dropped (291 in all, mostly real photos that
AIGIBench archives share).

### Plan `confirm7` (photos only)

Fetched after the photo second opinion was fixed in writing
([PREREGISTRATION.md](PREREGISTRATION.md)). 23,052 photos in 30 sets.

| Set | Source | Real | AI |
|---|---|---|---|
| So-Fake-OOD, batch 7 | shards 4, 20, 37 | 1,525 | 2,217 |
| AIGIBench, third sample | 400 files per class from what batches 5 and 6 left of each of their twelve archives (WFIR had 200 per class left) | 4,367 | 4,595 |
| AIGIBench, commercial generators | 400 per class from what batches 2 and 3 left of the Midjourney, FLUX.1-dev, DALL-E 3, SD3, SDXL and Imagen 3 archives | 1,801 | 2,394 |
| Classic test set, second sample | 300 per class from what batch 6 left of StyleGAN, StyleGAN2, BigGAN, CycleGAN, GauGAN, StarGAN | 1,774 | 1,800 |
| Classic test set, new families | up to 300 per class of CRN, IMLE, SAN, SITD and the FaceForensics++ deepfakes | 1,280 | 1,299 |

899 files identical to one trained on or scored earlier, or to an earlier
set of this batch, were dropped – among them all 400 real photos drawn for
SDXL, which that archive shares with others, so that set is AI only.

## Overlap checks

`tintype overlap` compares files byte for byte.

- No file in any benchmark or confirmation set is identical to a training
  file. Where a plan found a duplicate it dropped the file from the later set
  (So-Fake-OOD repeats a few files across shards) or from training (one
  So-Fake-Set photo that is also an AIGIBench real).
- Batch 3 was fetched after several training stores had been deleted. Its
  images were checked against a saved list of those stores' file hashes: no
  matches among 11,730 files.

## Frozen evaluation sets

[docs/manifests](manifests) records every evaluation suite as it was scored:
the SHA-256, size, label, generator and path of each file, and the SHA-256
of each manifest. They were written from the data on the machine that
produced the reported numbers.

| Suite | Record | Files | Size of the files |
|---|---|---|---|
| `benchmarks` | [benchmarks.frozen.tsv.gz](manifests/benchmarks.frozen.tsv.gz) | 27,078 | 29.3 GB |
| `confirm1` | [confirm1.frozen.tsv.gz](manifests/confirm1.frozen.tsv.gz) | 12,192 | 36.5 GB |
| `confirm2` | [confirm2.frozen.tsv.gz](manifests/confirm2.frozen.tsv.gz) | 8,511 | 10.5 GB |
| `confirm3` | [confirm3.frozen.tsv.gz](manifests/confirm3.frozen.tsv.gz) | 13,183 | 18.4 GB |
| `confirm4` | [confirm4.frozen.tsv.gz](manifests/confirm4.frozen.tsv.gz) | 2,056 | 6.5 GB |
| `confirm5` | [confirm5.frozen.tsv.gz](manifests/confirm5.frozen.tsv.gz) | 13,357 | 11.5 GB |
| `confirm6` | [confirm6.frozen.tsv.gz](manifests/confirm6.frozen.tsv.gz) | 16,324 | 11.3 GB |
| `confirm7` | [confirm7.frozen.tsv.gz](manifests/confirm7.frozen.tsv.gz) | 23,052 | 23.9 GB |

To check a set you fetched, run this from the directory that holds `data/`:

```bash
tintype verify-set docs/manifests/confirm7.frozen.tsv.gz
```

It lists every file that is missing or has other bytes, and every manifest
row that was added, dropped or relabelled, and fails if there is any. A
plan's output can differ from the record if an upstream file was removed or
replaced (the pins guard against the second) or if the RAISE-1k archive
changed.

The training stores and the `dev` sets are not recorded: several training
stores were deleted between phases (see above), so a record would describe
data that was not all there at once. To record any set, including your own:

```bash
tintype freeze-set --suite confirm7 --out confirm7.frozen.tsv.gz
tintype freeze-set --manifest data/store/openfake/manifest.tsv --out openfake.frozen.tsv
```

## Released model files

| File | Bytes | SHA-256 |
|---|---|---|
| models/tintype.safetensors | 86,678,036 | 9dcb21dc9800591ffd758a3c7ca6aa65fa5a9f09a223fecc952f34294ae9aeba |
| models/tintype.json | 3,450 | bfe1366bcfec28d15fb9116cb96c721de52de91d6fa4d98b6da4ec7fe6e62709 |
| models/tintype-video.safetensors | 86,678,036 | 77010f7cd733b23a73f269e275dd9c974f266b32e3c38e0e4d4a12e1723c4881 |
| models/tintype-video.json | 2,161 | 673e9040d941125691bfb80cbcf130e8efe2fdbf26a08f0b947450c4abca4e9c |

`tintype verify-models` checks a checkout against these values, which are also
in [src/integrity.rs](../src/integrity.rs); the `models` test fails if the files,
the source or this table disagree. The `.json` specs hold the thresholds and
calibration, so a recalibration changes them as surely as retraining changes
the weights.

## Continuous integration

[.github/workflows/ci.yml](../.github/workflows/ci.yml) builds and runs
`cargo test --release` on macOS on Apple Silicon (once with the default
build and once with `--features portable-cpu`) and on Linux. The tests
include runs of the built binary (`detect` in text and JSON form, on a
readable and an unreadable file) and a complete-model inference fixture: a
picture generated in the test goes through decoding, the photo model's views
and the released weights on the CPU engine, and its six logits must match
the recorded ones to within 0.05. The GPU engine, training and evaluation
are not exercised: hosted runners have no GPU that the Apple GPU engine can use.

## Known caveats

- AIGIBench CommunityAI stores every real image as JPEG and every AI image as
  PNG. A detector could exploit that; all systems face the same files.
- Generator *families* overlap between training and evaluation. The released
  weights have seen most image generators that appear in the evaluation sets,
  in the form of other images from other collections. The phase-3 weights had
  seen none of the video generators in batches 1–3 (Gen-2, Gen-3, Sora,
  Jimeng, Veo 3, CausVid, RepVideo, IPOC, the GenVideo-Val generators). The
  video checkpoint has seen Sora, Veo 3, Gen-2, Gen-3, Jimeng, RepVideo and
  VideoCrafter through other clips; AccVideo, CausVid, IPOC and most
  GenVideo-Val generators remain unseen. The baselines predate many of these
  generators: the comparison measures the systems as they exist today, not
  architectures on equal data.
- The video checkpoint's calibration and the second-opinion band were fitted
  on real clips from MSR-VTT, MSVD and Kinetics-400, the same sources batch 4
  draws its (different) real clips from.
- So-Fake-Set (training, phase 3) and So-Fake-OOD (evaluation) come from the
  same project. The authors designed OOD as the held-out test for models
  trained on Set, with disjoint generators; the social-media photos share
  platforms.
- Licences: OpenFake, Community Forensics (Small), So-Fake and GenVideo are
  non-commercial; see [models/LICENSE.md](../models/LICENSE.md). Sakugabooru,
  Human-Animal-Cartoon, Kinetics, UCF101 and the gameplay set are clips of
  third-party footage gathered by their curators; they are used here only as
  examples of real video and are not redistributed.
