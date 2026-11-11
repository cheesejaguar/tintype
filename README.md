# Tintype

Detects AI-generated photos and videos. One Rust crate and one binary cover
inference, training, dataset preparation, the reference detectors it is
compared against, and the evaluation.

```bash
tintype detect holiday.jpg clip.mp4
```

```
AI-GENERATED    98.7%  holiday.jpg
likely real      2.1%  clip.mp4
```

Two 22 M-parameter checkpoints, one for photos and one for video.

## Results

Against the strongest open detectors that ship weights, on data downloaded
only after the configuration under test and the protocol were fixed, and
scored once. Each system is at its shipped threshold.

| | Precision | Recall | Speed |
|---|---|---|---|
| **Photos** (8,628 AI, 7,696 real) | | | images/s, GPU / CPU |
| Tintype | 0.986 | 0.904 | 263 / 38.8 |
| Community Forensics | 0.966 | 0.818 | 61–67 / 25.4 (100 / 35.0 through this engine) |
| SwinV2 detector | 0.834 | 0.541 | 8.6 (30 through this engine) |
| Tintype `--strict` | 0.996 | 0.815 | 292 / 39.0 |
| Tintype `--sensitive` | 0.960 | 0.975 | 282 |
| Tintype `--fast` | 0.995 | 0.761 | 455 / 52.6 |
| **Photos, newest batch** (12,305 AI, 10,747 real) | | | images/s, GPU / CPU |
| Tintype | 0.988 | 0.897 | 530–563 / 77–78 |
| Tintype `--photo-second-opinion` | 0.972 | 0.935 | 281–288 / 39–41 |
| Community Forensics | 0.945 | 0.789 | 82 / 26 (291–295 / 39–42 through this engine at 16 bits) |
| **Video** (1,156 AI, 900 real) | | | seconds per clip |
| Tintype | 0.997 | 0.941 | 0.09 |
| Tintype `--second-opinion` | 0.997 | 0.960 | 0.26 |
| WaveRep | 0.983 | 0.912 | 4.0 (1.08 through this engine at 16 bits) |

What that table does not show:

- The photo result is pooled over nineteen sets. Most are older generators
  (GANs, GLIDE, identity adapters), and on fourteen of them Community
  Forensics finds more of the AI images than Tintype's default does (0.96–1.00
  against 0.82–1.00 on the GAN sets), at lower precision. Allowed Community
  Forensics' false-positive rate, Tintype is within 0.002–0.023 of its recall
  on those sets, level on four and ahead on five. Tintype's lead comes from
  current commercial generators in social-media photos, where Community
  Forensics finds 40% and Tintype 86%.
- `--photo-second-opinion` is the answer to that, and a partial one. It
  sends the photos the default profile does not flag with confidence – about
  half of a balanced test set, nine real photos in ten – to Community
  Forensics' own released network. On the newest batch (30 sets, fixed in
  writing and fetched after the mode was built) precision and recall are both
  at or above Community Forensics' on 26 sets, against 12 for the default
  alone. The other four: one real photo flagged to none on Midjourney and on
  WFIR, lower precision with higher recall on SAN, and lower recall on
  face-swap deepfakes (0.480 against 0.573). The mode runs at the speed of
  Community Forensics through this engine, not faster; it is 3.5× (GPU) and
  1.6× (CPU) faster than Community Forensics as released. It flags 3.1% of
  real photos where the default flags 1.2% and Community Forensics 5.2%.
- The first photo table is the third blind photo batch; the newest batch is
  the fourth, and its speeds were measured after the later speed work, which
  is why they are higher. After the first, the default
  threshold was loosened (the stricter setting is now `--strict`); after the
  second, the photo weights were retrained with older generators added. Each
  change was fixed in writing before the next batch was fetched.
- Photo speeds in the table were measured with the previous photo weights on
  an idle machine. Timed side by side later, under load, the current weights
  were faster than the previous ones (196 against 167 images/s on the GPU,
  30.3 against 28.6 on the CPU) and than Community Forensics in the same pass
  (165 and 25.5 through this engine, 50 and 18.4 as released).
- `--second-opinion` leans on WaveRep's own released network for the clips
  Tintype's video model is unsure about. The plain "Tintype" video row uses
  only Tintype's weights; on one of the three video sets its recall is a
  hair below WaveRep's (0.966 against 0.970) with higher precision.
- The speed lead over Community Forensics is wide against its released
  pipeline and narrow against a port of it to this engine. It depends on
  early exit; looking at all six views of every image, Tintype is the slower
  of the two in the same engine (180 against 253 images/s at 16 bits).

- The speed column was measured before a later round of speed work, which
  made the default photo profile about 1.6 times faster and video about
  three times faster on the same files, with the same pooled precision and
  recall when the latest blind batches were re-scored
  ([the before-and-after table](docs/BENCHMARK.md#five-speed-changes-before-and-after)).

[docs/BENCHMARK.md](docs/BENCHMARK.md) has every set, the speed tables, how the
reference detectors were ported and checked, and the other places where
Tintype is not ahead. [docs/RESEARCH.md](docs/RESEARCH.md) is the survey
behind the design and [docs/DATA.md](docs/DATA.md) lists every dataset.

## How it works

**Several looks at every image.** A detector that only sees a downscaled image
loses the pixel-level fingerprint of the generator; one that only sees a small
native-resolution patch loses the global picture. Tintype runs the same
network on six views and averages the logits:

- a *fit* view – shorter side resized to 256, centre crop 224;
- the centre 224×224 pixels at native resolution, untouched by resampling;
- the four most *textured* 224×224 native-resolution windows. Flat regions
  such as sky or a studio backdrop carry almost no forensic signal, so the
  crops go where the detail is.

**But only as many looks as it takes.** Most images are obvious. The views are
looked at one at a time, most textured crop first, and as soon as the running
score is far enough from the threshold the rest are skipped: about two views
per image on average instead of six. The margins were fitted on the
development sets, where they left pooled precision and recall within 0.002 of
looking at everything. `--all-views` turns this off.

**Video is frames, looked at the same way.** 16 frames are taken evenly across
a clip, each gets the six views, and the clip score is the mean logit. Clips
stop early too: two frames from the first half of the clip, then four, then
all eight of that half, and only then the second half, stopping as soon as
the score is clear. That is about five frames per clip on average, decoding
stops at the half-way mark for the four clips in five decided by then, and on
the 6,783 development and blind clips it changed no verdict. Video has its own checkpoint: the photo weights
trained on for two more phases on clips only, including the kinds of real
footage the photo weights mistook for generated video (cartoons, animation,
gameplay).

**A second opinion, if you want one.** `--second-opinion` sends the clips the
video model is unsure about – about one in eight in the tests – to the
[WaveRep](https://github.com/grip-unina/WaveRep-SyntheticVideoDetection)
network, re-implemented here in Rust and run on its authors' released
weights. The two models make different mistakes, so together they beat either
alone, and because most clips never reach the second network the pair is
still several times faster than WaveRep by itself. With a second opinion every
frame of every clip is looked at.

**And one for photos.** `--photo-second-opinion` does the same with the
[Community Forensics](https://github.com/JeongsooP/Community-Forensics)
384-pixel network (MIT): a photo the default profile is not sure about is
also scored by it, and the two scores are combined. Community Forensics is
the stronger of the two on images from older GANs, Tintype on current
commercial generators; together they find more than either (recall 0.935
against 0.897 and 0.789 on the newest blind batch) at a precision between
the two. `tintype data baselines` fetches the weights:

```bash
tintype detect --photo-second-opinion baselines/commfor-384.safetensors *.jpg
```

**Small network, wide training set.** The network is a plain ViT-S/16 at
224 px. It starts from the Community Forensics 224 px checkpoint (MIT) and is
retrained in four phases on photos, frames and clips from about 1,200
generators – the commercial image generators of 2025–26 and the GAN families
before them – against real photos and footage from sixteen sources. The video checkpoint is
trained on clips from thirty video generators, from open models to Sora,
Veo 3.1 and Gen-3.

**Trained against what happens to media in the wild.** Every training sample is
randomly JPEG-compressed before and/or after resizing, resized with a randomly
chosen kernel and cropped at a random scale; a quarter go through a
messenger-style round trip (long side capped at 960–1700 px, JPEG quality
55–90). Video frames are drawn from clips re-encoded with random H.264 quality
and resolution, so bitrate cannot stand in for the label.

**Four operating points**, each calibrated on held-out real media so that a
set share of real items, averaged over real sources, scores above 50%:

| | Views | Calibrated false-positive rate | Use |
|---|---|---|---|
| default | up to 6 | 1.5% (photos), 0.5% (video) | The balance: ahead of both reference photo detectors on precision and on recall |
| `--strict` | up to 6 | 0.5% | A positive verdict that has to stand on its own |
| `--sensitive` | up to 6 | 5% | Triage, where a missed fake costs more than a false alarm |
| `--fast` | 2 | 0.5% | Bulk scanning at the highest throughput |

## Engines

| Engine | Where | Notes |
|---|---|---|
| `gpu` (default on macOS) | Apple GPU, 16-bit | The whole network is compiled into one MPSGraph, driven from Rust through `objc2`. Training uses the same graph builder, with autodiff and AdamW inside the graph. |
| `gpu-f32` | Apple GPU, 32-bit | Matches PyTorch output to five decimals. |
| `cpu` | Anywhere | An engine written for this network. The matrix products go to Accelerate on macOS and to pure-Rust kernels (NEON, AVX2 or AVX-512, chosen at run time) on Linux. |

Images are decoded and resized in Rust (`image`, `zune-jpeg`,
`fast_image_resize`) on all cores and fed to the engine through a bounded
queue. Video is decoded in-process: by AVFoundation on macOS and by a
pure-Rust reader elsewhere (an MP4/MOV parser written for this project
feeding the `rust_h264` and `rust_h265` decoders). The YCbCr→RGB conversion
is done in Rust either way, and no external tool is called anywhere.

## Install

```bash
cargo build --release
```

Needs a Rust toolchain. The trained weights are `models/tintype.safetensors`
for photos and `models/tintype-video.safetensors` for video (87 MB each),
each with its run spec in a `.json` beside it. The
GPU engine, training and the evaluation commands need macOS on Apple Silicon.
`detect` also runs on Linux, on the CPU engine, for photos and for video:
built and run on aarch64 Ubuntu (and, for photos, as an x86-64 binary under
translation). There it scored the third blind photo batch with the verdict
the macOS CPU engine gives on every one of 16,324 photos, at 25 photos/s on
eight virtual cores against 15 for Community Forensics' released pipeline
on the same machine, and the blind video batch at 1.4 s per clip with the
same precision and recall as on macOS – on the 1,756 of 2,056 clips its
video reader can open
([docs/results/linux.log](docs/results/linux.log)).

## Use

```bash
tintype detect *.jpg *.mp4                 # human-readable verdicts
tintype detect --json suspicious.png       # one JSON object per file
tintype detect --sensitive inbox/*.webp    # looser threshold
tintype detect --strict evidence.jpg       # tighter threshold
tintype detect --fast --device cpu a.jpg   # two views, CPU engine

tintype data baselines                     # fetches the WaveRep weights, once
tintype detect --second-opinion baselines/waverep-g4.ckpt *.mp4
```

The WaveRep weights are its authors' and come under their licence (research
and other non-profit use); they are downloaded from the authors' site, not
shipped here.

Exit status is 0 whether or not anything is flagged; parse the output.

## Reproduce

Everything is a subcommand; there are no scripts.

```bash
tintype data train-images && tintype data train-video   # phase-1 data
tintype data phase2 && tintype data phase3              # later additions
tintype data baselines                                    # reference detectors' released weights

tintype train --init baselines/commfor-224.safetensors --init-prefix vit. \
    --manifest data/store/openfake/manifest.tsv ... --out runs/v2 --steps 14000 --p-messenger 0
tintype train --init runs/v2/step_014000.safetensors --manifest ... --out runs/v3 --steps 7000 --lr 2.5e-5 --warmup 300
tintype train --init runs/v3/step_007000.safetensors --manifest ... --out runs/v4 --steps 7000 --lr 2e-5 --warmup 300

tintype data dev && tintype data benchmarks && tintype data confirm1 && tintype data confirm2
tintype finalize runs/v4/step_007000.safetensors          # spec + calibration of every profile

tintype data confirm3 && tintype data confirm5           # the first two blind photo batches
tintype evaluate confirm3 --baselines                     # all systems, all tables
tintype evaluate confirm5 --baselines

# the current photo weights
tintype data phase6
tintype train --init runs/v4/step_007000.safetensors --manifest <photo and phase-1 video stores> ... --out runs/v7 --steps 8000 --lr 2e-5 --warmup 300
tintype finalize runs/v7/step_008000.safetensors --also-trained-on <phase-6 manifests> --also-scored-on <batch 3 and 5 manifests>
tintype data confirm6
tintype evaluate confirm6 --baselines

# the video checkpoint
tintype data phase4 && tintype data phase5
tintype train --init runs/v4/step_007000.safetensors --manifest <video stores> ... --out runs/v5 --steps 7000 --lr 2e-5 --warmup 300 --p-messenger 0.1
tintype train --init runs/v5/step_007000.safetensors --manifest <video stores> ... --out runs/v6 --steps 4000 --lr 1.5e-5 --warmup 200 --p-messenger 0.1
tintype finalize-video runs/v6/step_004000.safetensors --video-fpr 0.005 --second-opinion=-1,4,1
tintype data confirm4
tintype evaluate confirm4 --baselines
```

[docs/DATA.md](docs/DATA.md) lists the manifests each training run used.

## Commands

| Command | Purpose |
|---|---|
| `detect` | Verdicts for image and video files |
| `data` | Download and prepare a dataset plan |
| `train` | Fine-tune on image and video manifests |
| `finalize`, `finalize-video`, `calibrate` | Freeze a checkpoint and fit its operating points |
| `evaluate` | Score a whole suite with Tintype and the reference detectors, print tables |
| `score`, `score-video` | Score one manifest (`--reference commfor|swinv2|waverep` runs a reference detector) |
| `report` | Precision, recall, F1, FPR, AUROC, recall at 1% FPR, bootstrap intervals; `--at-rate-of` compares systems at one system's false-positive rate |
| `latency` | Single-image latency, decode to verdict |
| `profile` | Decoding, cropping and the network timed separately, the network at several batch sizes |
| `ingest`, `index-folder`, `overlap`, `check-videos`, `parquet-info` | Dataset tooling |

## Limits

- A detector's verdict is evidence, not proof. Precision below 100% means real
  photos do get flagged; do not act on a single score where the stakes are high.
- Generators released after the training data was collected (November 2026)
  will be detected less reliably. Retrain as new generators appear.
- Local edits (inpainting a region of a real photo) are out of scope; the
  model is trained on fully generated images.
- Media that has been through a messenger or social app is much harder. On
  WhatsApp and Instagram copies of 2026 images the default profile finds
  27–36% of the AI images (flagging under 1% of real ones); `--sensitive`
  finds 55–59% while flagging 2–3%.
- Older generators are a relative weak spot: on StyleGAN3 and
  thispersondoesnotexist-style faces the default profile finds 73–79%, where
  a detector trained on thousands of such generators finds nearly all.
- The GPU engine, training and the second-opinion stage are Apple-only.
- Video containers and codecs: on macOS the system decoder reads MP4 and MOV
  with H.264, HEVC or MPEG-4 Part 2; AVI and WebM do not work. On Linux the
  pure-Rust reader takes MP4 and MOV with H.264 or HEVC in 4:2:0 only – not
  lossless H.264, MPEG-4 Part 2 or fragmented files – and such clips are
  reported as unreadable. `TINTYPE_DECODER=rust` selects that reader on
  macOS too.

## Licence

Code: MIT OR Apache-2.0 ([LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE)). Trained weights: CC BY-NC-SA 4.0, because
of the training data ([models/LICENSE.md](models/LICENSE.md)). The optional
second-opinion weights are WaveRep's, under its own non-profit licence.
Third-party notices: [NOTICE.md](NOTICE.md).
