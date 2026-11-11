# Veracity

Detects AI-generated photos and videos. One Rust crate and one binary cover
inference, training, dataset preparation, the reference detectors it is
compared against, and the evaluation.

```bash
veracity detect holiday.jpg clip.mp4
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
| **Photos** (7,079 AI, 6,278 real) | | | images/s, GPU / CPU |
| Veracity | 0.980 | 0.875 | 263 / 38.8 |
| Community Forensics | 0.963 | 0.762 | 61–67 / 25.4 (100 / 35.0 through this engine) |
| SwinV2 detector | 0.838 | 0.677 | 8.6 (30 through this engine) |
| Veracity `--strict` | 0.995 | 0.713 | 292 / 39.0 |
| Veracity `--sensitive` | 0.940 | 0.957 | 282 |
| Veracity `--fast` | 0.995 | 0.658 | 455 / 52.6 |
| **Video** (1,156 AI, 900 real) | | | seconds per clip |
| Veracity | 0.997 | 0.941 | 0.09 |
| Veracity `--second-opinion` | 0.997 | 0.960 | 0.26 |
| WaveRep | 0.983 | 0.912 | 4.0 (1.08 through this engine at 16 bits) |

What that table does not show:

- The photo result is pooled. Most of that batch is older generators (GANs,
  GLIDE, identity adapters), and on nine of its thirteen sets Community
  Forensics finds more of the AI images than Veracity's default does, at
  lower precision. Veracity's lead comes from current commercial generators
  in social-media photos, where Community Forensics finds 41% and Veracity
  86%. `--sensitive` recovers the older generators at 4–10% false positives.
- The default threshold was loosened after an earlier blind batch, on which
  the stricter setting (now `--strict`) scored 0.997 / 0.865 against
  Community Forensics' 0.970 / 0.637 and the SwinV2 detector's 0.896 / 0.918.
- `--second-opinion` leans on WaveRep's own released network for the clips
  Veracity's video model is unsure about. The plain "Veracity" video row uses
  only Veracity's weights; on one of the three video sets its recall is a
  hair below WaveRep's (0.966 against 0.970) with higher precision.
- The speed lead over Community Forensics is wide against its released
  pipeline and narrow against a port of it to this engine. It depends on
  early exit; looking at all six views of every image, Veracity is the slower
  of the two in the same engine (180 against 253 images/s at 16 bits).

[docs/BENCHMARK.md](docs/BENCHMARK.md) has every set, the speed tables, how the
reference detectors were ported and checked, and the other places where
Veracity is not ahead. [docs/RESEARCH.md](docs/RESEARCH.md) is the survey
behind the design and [docs/DATA.md](docs/DATA.md) lists every dataset.

## How it works

**Several looks at every image.** A detector that only sees a downscaled image
loses the pixel-level fingerprint of the generator; one that only sees a small
native-resolution patch loses the global picture. Veracity runs the same
network on six views and averages the logits:

- a *fit* view – shorter side resized to 256, centre crop 224;
- the centre 224×224 pixels at native resolution, untouched by resampling;
- the four most *textured* 224×224 native-resolution windows. Flat regions
  such as sky or a studio backdrop carry almost no forensic signal, so the
  crops go where the detail is.

**But only as many looks as it takes.** Most images are obvious. The views are
looked at one at a time, most textured crop first, and as soon as the running
score is far enough from the threshold the rest are skipped: about three
views per image on average instead of six, with one verdict in 700 changed on
the development sets. `--all-views` turns this off.

**Video is frames, looked at the same way.** 16 frames are taken evenly across
a clip, each gets the six views, and the clip score is the mean logit. Video
has its own checkpoint: the photo weights trained on for two more phases on
clips only, including the kinds of real footage the photo weights mistook for
generated video (cartoons, animation, gameplay).

**A second opinion, if you want one.** `--second-opinion` sends the clips the
video model is unsure about – about one in eight in the tests – to the
[WaveRep](https://github.com/grip-unina/WaveRep-SyntheticVideoDetection)
network, re-implemented here in Rust and run on its authors' released
weights. The two models make different mistakes, so together they beat either
alone, and because most clips never reach the second network the pair is
still several times faster than WaveRep by itself.

**Small network, wide training set.** The network is a plain ViT-S/16 at
224 px. It starts from the Community Forensics 224 px checkpoint (MIT) and is
retrained in three phases on photos, frames and clips from about 380
generators – the commercial image generators of 2025–26 among them – against
real photos and footage from fourteen sources. The video checkpoint is
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
| `cpu` | macOS: Accelerate-backed engine written for this network. Elsewhere: [candle](https://github.com/huggingface/candle). | |

Images are decoded and resized in Rust (`image`, `zune-jpeg`,
`fast_image_resize`) on all cores and fed to the engine through a bounded
queue. Video is decoded in-process through AVFoundation, with the YCbCr→RGB
conversion done in Rust; no external tool is called anywhere.

## Install

```bash
cargo build --release
```

Needs a Rust toolchain. The trained weights are `models/veracity.safetensors`
for photos and `models/veracity-video.safetensors` for video (87 MB each),
each with its run spec in a `.json` beside it. Video, training and the
evaluation commands need macOS on Apple Silicon; `detect --device cpu` on
photos builds and runs anywhere.

## Use

```bash
veracity detect *.jpg *.mp4                 # human-readable verdicts
veracity detect --json suspicious.png       # one JSON object per file
veracity detect --sensitive inbox/*.webp    # looser threshold
veracity detect --strict evidence.jpg       # tighter threshold
veracity detect --fast --device cpu a.jpg   # two views, CPU engine

veracity data baselines                     # fetches the WaveRep weights, once
veracity detect --second-opinion baselines/waverep-g4.ckpt *.mp4
```

The WaveRep weights are its authors' and come under their licence (research
and other non-profit use); they are downloaded from the authors' site, not
shipped here.

Exit status is 0 whether or not anything is flagged; parse the output.

## Reproduce

Everything is a subcommand; there are no scripts.

```bash
veracity data train-images && veracity data train-video   # phase-1 data
veracity data phase2 && veracity data phase3              # later additions
veracity data baselines                                    # reference detectors' released weights

veracity train --init baselines/commfor-224.safetensors --init-prefix vit. \
    --manifest data/store/openfake/manifest.tsv ... --out runs/v2 --steps 14000 --p-messenger 0
veracity train --init runs/v2/step_014000.safetensors --manifest ... --out runs/v3 --steps 7000 --lr 2.5e-5 --warmup 300
veracity train --init runs/v3/step_007000.safetensors --manifest ... --out runs/v4 --steps 7000 --lr 2e-5 --warmup 300

veracity data dev && veracity data benchmarks && veracity data confirm1 && veracity data confirm2
veracity finalize runs/v4/step_007000.safetensors          # spec + calibration of every profile

veracity data confirm3 && veracity data confirm5           # the two blind photo batches
veracity evaluate confirm3 --baselines                     # all systems, all tables
veracity evaluate confirm5 --baselines

# the video checkpoint
veracity data phase4 && veracity data phase5
veracity train --init runs/v4/step_007000.safetensors --manifest <video stores> ... --out runs/v5 --steps 7000 --lr 2e-5 --warmup 300 --p-messenger 0.1
veracity train --init runs/v5/step_007000.safetensors --manifest <video stores> ... --out runs/v6 --steps 4000 --lr 1.5e-5 --warmup 200 --p-messenger 0.1
veracity finalize-video runs/v6/step_004000.safetensors --video-fpr 0.005 --second-opinion=-1,4,1
veracity data confirm4
veracity evaluate confirm4 --baselines
```

[docs/DATA.md](docs/DATA.md) lists the manifests each training run used.

## Commands

| Command | Purpose |
|---|---|
| `detect` | Verdicts for image and video files |
| `data` | Download and prepare a dataset plan |
| `train` | Fine-tune on image and video manifests |
| `finalize`, `finalize-video`, `calibrate` | Freeze a checkpoint and fit its operating points |
| `evaluate` | Score a whole suite with Veracity and the reference detectors, print tables |
| `score`, `score-video` | Score one manifest (`--reference commfor|swinv2|waverep` runs a reference detector) |
| `report` | Precision, recall, F1, FPR, AUROC, recall at 1% FPR, bootstrap intervals; `--at-rate-of` compares systems at one system's false-positive rate |
| `latency` | Single-image latency, decode to verdict |
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
- The GPU engine, training, all video decoding and the second-opinion stage
  are Apple-only. Video is read by the system decoder: MP4 and MOV with H.264
  or HEVC work, AVI and WebM do not.

## Licence

Code: MIT OR Apache-2.0 ([LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE)). Trained weights: CC BY-NC-SA 4.0, because
of the training data ([models/LICENSE.md](models/LICENSE.md)). The optional
second-opinion weights are WaveRep's, under its own non-profit licence.
Third-party notices: [NOTICE.md](NOTICE.md).
