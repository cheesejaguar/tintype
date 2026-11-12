<div align="center">

# Tintype

**Detects AI-generated photos and videos. Small, fast, and written in Rust.**

[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-dea584?logo=rust&logoColor=white)](Cargo.toml)
[![Code: MIT or Apache-2.0](https://img.shields.io/badge/code-MIT%20OR%20Apache--2.0-2a78d6)](#-licence)
[![Weights: CC BY-NC-SA 4.0](https://img.shields.io/badge/weights-CC%20BY--NC--SA%204.0-eb6834)](models/LICENSE.md)
[![Platforms: macOS and Linux](https://img.shields.io/badge/platform-macOS%20%C2%B7%20Linux-52514e)](#-install)
[![Model: ViT-S/16, 22M parameters](https://img.shields.io/badge/model-ViT--S%2F16%20%C2%B7%2022M%20params-1baf7a)](#-how-it-works)
[![GitHub stars](https://img.shields.io/github/stars/cheesejaguar/tintype?style=flat&color=898781)](https://github.com/cheesejaguar/tintype/stargazers)

[Quick start](#-quick-start) · [Results](#-results) · [How it works](#-how-it-works) · [Use](#-use) · [Limits](#%EF%B8%8F-limits) · [Benchmark](docs/BENCHMARK.md)

</div>

One Rust crate and one binary cover inference, training, dataset preparation,
the reference detectors it is compared against, and the evaluation. It needs
no Python and calls no external tool.

```bash
tintype detect holiday.jpg clip.mp4
```

```
AI-GENERATED    98.7%  holiday.jpg
likely real      2.1%  clip.mp4
```

| 📷 Photos | 🎬 Video | ⚡ Speed |
|:---:|:---:|:---:|
| **0.988** precision · **0.897** recall | **0.997** precision · **0.941** recall | **530–563** photos/s on an Apple GPU |
| 23,052 photos in 30 blind sets | 2,056 clips in 3 blind sets | 77–78 photos/s on the CPU |

A tintype is a photograph made directly on a metal plate: one exposure, one
original. This project is about telling that kind of picture from the other
kind.

## 🚀 Quick start

```bash
git clone https://github.com/cheesejaguar/tintype && cd tintype
cargo build --release
./target/release/tintype detect path/to/photo.jpg path/to/clip.mp4
```

The trained weights ship in the repository (two 87 MB checkpoints, one for
photos and one for video), so there is nothing else to download.

## 📊 Results

Tintype is compared with the strongest open detectors that ship weights:
[Community Forensics](https://github.com/JeongsooP/Community-Forensics) for
photos and
[WaveRep](https://github.com/grip-unina/WaveRep-SyntheticVideoDetection) for
video. Every number here is from a *blind batch*: test data downloaded only
after the configuration under test and the protocol had been written down and
committed, and scored once. Each system runs at its shipped threshold.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/errors-dark.svg">
  <img alt="Bar chart of error rates. Photos: AI photos missed – Tintype 10.3%, Tintype with second opinion 6.5%, Community Forensics 21.1%; real photos flagged – 1.2%, 3.1%, 5.2%. Video: AI clips missed – Tintype 5.9%, with second opinion 4.0%, WaveRep 8.8%; real clips flagged – 0.3%, 0.3%, 2.0%." src="docs/assets/errors-light.svg" width="820">
</picture>

| | Precision | Recall |
|---|---|---|
| **Photos**, newest blind batch (12,305 AI, 10,747 real) | | |
| Tintype | **0.988** | 0.897 |
| Tintype `--photo-second-opinion` | 0.972 | **0.935** |
| Community Forensics | 0.945 | 0.789 |
| SwinV2 detector | 0.841 | 0.587 |
| **Video**, blind batch (1,156 AI, 900 real) | | |
| Tintype | **0.997** | 0.941 |
| Tintype `--second-opinion` | **0.997** | **0.960** |
| WaveRep | 0.983 | 0.912 |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/speed-dark.svg">
  <img alt="Bar chart of photo throughput in photos per second. Apple GPU: Tintype 530–563, Tintype with second opinion 281–288, Community Forensics as released 82, Community Forensics ported to this engine 291–295. CPU: Tintype 77–78, with second opinion 39–41, Community Forensics as released 26, ported 39–42." src="docs/assets/speed-light.svg" width="820">
</picture>

Video runs at about 0.03 s per clip on the GPU (the blind batch of 2,056
clips in 65–67 s); WaveRep as released takes 4.0 s per clip.

### 🧾 What the charts do not show

These results are pooled, and pooled results hide things. The ones that
matter:

- **Not every set is a win.** Community Forensics was trained on thousands
  of older generators and is the stronger detector on GAN images; Tintype is
  far ahead on current commercial generators (on social-media photos from
  them it finds 84% where Community Forensics finds 42%). On the newest
  batch the default profile is at or above Community Forensics on *both*
  precision and recall on 12 of the 30 sets.
- **The photo second opinion closes most of that gap, not all of it.** With
  `--photo-second-opinion` it is 26 of 30. The other four: one real photo
  flagged to none on Midjourney and on WFIR, lower precision with higher
  recall on SAN, and lower recall on face-swap deepfakes (0.480 against
  0.573).
- **The second opinions lean on the other detectors' own networks.**
  `--photo-second-opinion` sends the photos Tintype is not sure about – about
  half of a balanced test set, nine real photos in ten – to Community
  Forensics' released network, and `--second-opinion` sends about one clip in
  eight to WaveRep's. Rows labelled plain "Tintype" use only Tintype's
  weights.
- **Speed depends on what you compare with.** The default profile is 6.5–7×
  (GPU) and 3× (CPU) faster than Community Forensics as its authors release
  it. The grey bars are Community Forensics' network ported to
  Tintype's engine at 16 bits, a configuration nobody ships: against that the
  default is still about twice as fast, and the photo second-opinion mode is
  level, not faster. The default's lead also depends on early exit; looking
  at all six views of every image it is the slower of the two in the same
  engine.
- **The second opinion costs false alarms.** It flags 3.1% of real photos
  where the default flags 1.2% (Community Forensics: 5.2%).
- **Video, one set.** On one of the three video sets the plain video model's
  recall is a hair below WaveRep's (0.966 against 0.970), with higher
  precision.

<details>
<summary><b>Recall on each of the 30 photo sets</b></summary>
<br>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/recall-by-set-dark.svg">
  <img alt="Dot plot of recall for Community Forensics, Tintype and Tintype with the second opinion on each of the 30 blind photo sets. Tintype leads by a wide margin on commercial generators such as Midjourney, FLUX.1-dev, DALL-E 3 and Imagen 3; Community Forensics leads the default profile on most GAN sets, where the second opinion catches up; on face-swap deepfakes Community Forensics leads both." src="docs/assets/recall-by-set-light.svg" width="820">
</picture>

Precision, real photos flagged and the exact values for every set are in
[docs/BENCHMARK.md](docs/BENCHMARK.md#blind-batch-7-the-photo-second-opinion).

</details>

[docs/BENCHMARK.md](docs/BENCHMARK.md) has every set of every batch, the
speed tables, how the reference detectors were ported and checked, the Linux
runs, and the other places where Tintype is not ahead.
[docs/RESEARCH.md](docs/RESEARCH.md) is the survey behind the design,
[docs/DATA.md](docs/DATA.md) lists every dataset, and
[docs/PREREGISTRATION.md](docs/PREREGISTRATION.md) is what was fixed in
writing before each blind batch.

```mermaid
flowchart LR
    A["✍️ Fix the configuration<br/>and protocol in writing"] --> B["📥 Download test data<br/>nobody has seen"]
    B --> C["🎯 Score once,<br/>every system"]
    C --> D["📒 Report all of it,<br/>losses included"]
```

## 🧠 How it works

```mermaid
flowchart LR
    P["📷 Photo"] --> V["Six views<br/>1 fit + 5 native-resolution crops"]
    V --> N["ViT-S/16<br/>one view at a time"]
    N -- "score already clear" --> D["Verdict"]
    N -- "not yet" --> N
    N -. "optional:<br/>still unsure" .-> S["Second opinion<br/>Community Forensics, 384 px"]
    S --> D
```

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
the 6,783 development and blind clips it changed no verdict. Video has its
own checkpoint: the photo weights trained on for two more phases on clips
only, including the kinds of real footage the photo weights mistook for
generated video (cartoons, animation, gameplay).

**A second opinion, if you want one.** `--second-opinion` sends the clips the
video model is unsure about to the
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
commercial generators; together they find more than either at a precision
between the two.

**Small network, wide training set.** The network is a plain ViT-S/16 at
224 px. It starts from the Community Forensics 224 px checkpoint (MIT) and is
retrained in four phases on photos, frames and clips from about 1,200
generators – the commercial image generators of 2025–26 and the GAN families
before them – against real photos and footage from sixteen sources. The video
checkpoint is trained on clips from thirty video generators, from open models
to Sora, Veo 3.1 and Gen-3.

**Trained against what happens to media in the wild.** Every training sample is
randomly JPEG-compressed before and/or after resizing, resized with a randomly
chosen kernel and cropped at a random scale; a quarter go through a
messenger-style round trip (long side capped at 960–1700 px, JPEG quality
55–90). Video frames are drawn from clips re-encoded with random H.264 quality
and resolution, so bitrate cannot stand in for the label.

### 🎚️ Four operating points

Each is calibrated on held-out real media so that a set share of real items,
averaged over real sources, scores above 50%. Precision and recall are from
the newest blind photo batch.

| | Views | Calibrated false-positive rate | Precision / recall | Use |
|---|---|---|---|---|
| default | up to 6 | 1.5% (photos), 0.5% (video) | 0.988 / 0.897 | The balance |
| `--strict` | up to 6 | 0.5% | 0.995 / 0.824 | A positive verdict that has to stand on its own |
| `--sensitive` | up to 6 | 5% | 0.963 / 0.960 | Triage, where a missed fake costs more than a false alarm |
| `--fast` | 2 | 0.5% | 0.995 / 0.767 | Bulk scanning at the highest throughput |

## ⚙️ Engines

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
is done in Rust either way.

## 📦 Install

```bash
cargo build --release
```

Needs a Rust toolchain. The trained weights are `models/tintype.safetensors`
for photos and `models/tintype-video.safetensors` for video (87 MB each),
each with its run spec in a `.json` beside it.

| | macOS on Apple Silicon | Linux |
|---|---|---|
| `detect`, photos and video | ✅ GPU and CPU | ✅ CPU |
| Second opinions | ✅ | ✅ photos · ❌ video |
| Training and evaluation commands | ✅ | ❌ |

On Linux (aarch64 Ubuntu, eight virtual cores) `detect` gave the same verdict
as the macOS CPU engine on every one of 16,324 photos of a blind batch, at
35 photos/s against 14–15 for Community Forensics' released pipeline on the
same machine, and scored the blind video batch at 0.43 s per clip with the
same precision and recall as on macOS – on the 1,756 of 2,056 clips its video
reader can open
([the Linux section of the benchmark](docs/BENCHMARK.md#five-speed-changes-before-and-after)).

## 🔍 Use

```bash
tintype detect *.jpg *.mp4                 # human-readable verdicts
tintype detect --json suspicious.png       # one JSON object per file
tintype detect --sensitive inbox/*.webp    # looser threshold
tintype detect --strict evidence.jpg       # tighter threshold
tintype detect --fast --device cpu a.jpg   # two views, CPU engine

tintype data baselines                     # fetches the reference detectors' weights, once
tintype detect --photo-second-opinion baselines/commfor-384.safetensors *.jpg
tintype detect --second-opinion baselines/waverep-g4.ckpt *.mp4
```

The WaveRep weights are its authors' and come under their licence (research
and other non-profit use); they are downloaded from the authors' site, not
shipped here. The Community Forensics weights are MIT.

Exit status is 0 whether or not anything is flagged; parse the output.

## 🧰 Commands

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

## 🔁 Reproduce

Everything is a subcommand; there are no scripts.

<details>
<summary><b>The full sequence, from training data to the blind batches</b></summary>

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

# the photo second opinion
tintype set-photo-second-opinion --band=-7,0.5,0.5,0
tintype data confirm7
tintype evaluate confirm7 --baselines

# the video checkpoint
tintype data phase4 && tintype data phase5
tintype train --init runs/v4/step_007000.safetensors --manifest <video stores> ... --out runs/v5 --steps 7000 --lr 2e-5 --warmup 300 --p-messenger 0.1
tintype train --init runs/v5/step_007000.safetensors --manifest <video stores> ... --out runs/v6 --steps 4000 --lr 1.5e-5 --warmup 200 --p-messenger 0.1
tintype finalize-video runs/v6/step_004000.safetensors --video-fpr 0.005 --second-opinion=-1,4,1
tintype data confirm4
tintype evaluate confirm4 --baselines
```

</details>

[docs/DATA.md](docs/DATA.md) lists the manifests each training run used.

## ⚠️ Limits

- A detector's verdict is evidence, not proof. Precision below 100% means real
  photos do get flagged; do not act on a single score where the stakes are high.
- Generators released after the training data was collected (November 2026)
  will be detected less reliably. Retrain as new generators appear.
- Local edits (inpainting a region of a real photo) and face swaps are out of
  scope; the model is trained on fully generated images. On a face-swap
  deepfake set the default profile finds 14%.
- Media that has been through a messenger or social app is much harder. On
  WhatsApp and Instagram copies of 2026 images the default profile finds
  27–36% of the AI images (flagging under 1% of real ones); `--sensitive`
  finds 55–59% while flagging 2–3%.
- Older generators are a relative weak spot of the default profile: on
  several GAN sets it finds 84–89% where a detector trained on thousands of
  such generators finds nearly all. `--photo-second-opinion` is there for
  that.
- The GPU engine, training, the evaluation commands and the video second
  opinion are Apple-only.
- Video containers and codecs: on macOS the system decoder reads MP4 and MOV
  with H.264, HEVC or MPEG-4 Part 2; AVI and WebM do not work. On Linux the
  pure-Rust reader takes MP4 and MOV with H.264 or HEVC in 4:2:0 only – not
  lossless H.264, MPEG-4 Part 2 or fragmented files – and such clips are
  reported as unreadable. `TINTYPE_DECODER=rust` selects that reader on
  macOS too.

## 📜 Licence

| What | Licence |
|---|---|
| Code | MIT OR Apache-2.0 ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)) |
| Trained weights | CC BY-NC-SA 4.0, because of the training data ([models/LICENSE.md](models/LICENSE.md)). **Non-commercial.** |
| Optional second-opinion weights | Community Forensics: MIT. WaveRep: its authors' non-profit licence. Neither is shipped here. |

Third-party notices: [NOTICE.md](NOTICE.md).
