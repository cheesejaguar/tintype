# State of the art in AI-generated image and video detection (November 2026)

This is the survey that shaped Veracity's design and its choice of baselines.
Claims are tied to sources; numbers measured on this machine are marked
"measured here".

## 1. Images

### What works

Three families of detector are in use.

- **Low-level artifact detectors** (CNNSpot, NPR, PatchCraft, SAFE, FreDect,
  LGrad). Small CNNs on pixels, residuals or spectra. Very accurate on the
  generators they were trained on and on unprocessed files; they fall apart
  on unseen generator families and after recompression.
- **Frozen foundation-model features** (UnivFD, Cozzolino et al.'s CLIP
  detector, RINE, AIDE, Effort, ForgeLens, DRCT). A CLIP or DINO encoder plus a
  light head. These generalise better to diffusion and transformer generators,
  at the cost of a large backbone (usually ViT-L/14).
- **End-to-end training on very diverse data** (Community Forensics, B-Free,
  OpenFake baselines). The Community Forensics paper's central finding is that
  the number of distinct generators in the training set matters more than the
  architecture: a plain ViT-S trained on 4,803 generators beat far larger
  specialised models.

### Who leads among open detectors

The most useful independent comparison is *How well are open sourced
AI-generated image detection models out-of-the-box* (arXiv 2602.07814, Feb
2026): 16 methods and 23 released checkpoints, evaluated zero-shot at a fixed
0.5 threshold on 12 datasets. Its findings:

- **Community Forensics is the best open detector overall**, with 75–78% mean
  accuracy across datasets and first place on 8 of 12. The worst (CNNSpot)
  manages 37.5%.
- Rankings are unstable across datasets (Spearman ρ from 0.01 to 0.87), so a
  single benchmark number means little.
- Detectors trained on one generator family (AIDE, DRCT variants) swing 20–60
  points depending on the training set.

Community Forensics (Park & Owens, CVPR 2025) is therefore the image baseline
here. Its released model is a ViT-S/16 at 384 px (21.8 M parameters, MIT
licence), trained end to end from an ImageNet-21k checkpoint with heavy
augmentation. At test time it resizes the shorter side to 440, centre-crops 384
and thresholds the sigmoid output at 0.5.

A vendor-run leaderboard (WasItAIGenerated, Oct 2026) lists other open
checkpoints on brand-new generators; the best open entry there, a 200 M
parameter SwinV2, reaches AUROC 0.911 and detects 48% of AI images at a 1%
false-positive rate. Commercial systems report 97–100% on their own tests,
which cannot be reproduced.

### Where the leader is weak (measured here)

| Benchmark | Precision | Recall | False-positive rate | AUROC |
|---|---|---|---|---|
| So-Fake-OOD (15 commercial generators of 2025–26 vs. social-media photos) | 0.979 | 0.404 | 1.2% | 0.899 |
| Synthbuster+ (13 generators vs. RAISE camera photos) | 0.979 | 0.774 | 7.9% | 0.899 |
| AIGIBench CommunityAI | 0.964 | 0.554 | 2.1% | 0.932 |

Two things stand out.

1. **Recall on current generators is low.** It finds 12% of FLUX.2 images, 12%
   of Nano Banana 2, 15% of GPT-image-2 and 16% of FLUX.1 pro. The model dates
   from 2024 and the generators it never saw are the ones people use now.
2. **Its decisions hinge on resampling details.** Feeding it the same image
   resized by two bilinear implementations whose outputs differ by at most one
   intensity level in 16% of pixels moved one score from 0.92 to 0.32. Across
   So-Fake-OOD, 2% of decisions flip between Pillow's resize and a
   Pillow-compatible Rust resize. A detector that sensitive to rounding is
   reading traces that the first re-save will destroy.

### Benchmarks worth using

- **So-Fake-OOD** – social-media reals (Reddit, Tumblr, Bluesky) against 15
  commercial generators; the closest public proxy for real moderation traffic.
- **Synthbuster** and the B-Free extension – uncompressed generator output
  against RAISE camera originals; the baseline's authors report on it, which
  lets a reproduction be checked.
- **AIGIBench**, **Chameleon**, **OpenFake**, **MNW**, **AI-GenBench** – other
  in-the-wild or multi-generator sets. Chameleon needs an email request; MNW is
  fake-only and evaluation-only.
- **WasItAIGenerated frontier benchmark (2026-10)** – small, but built from
  generators released in the second half of 2026 with free-licence real
  photos, plus WhatsApp/Instagram re-shares of every file.

GenImage and the ForenSynths/UniversalFakeDetect sets are saturated and are
in-distribution for the baseline.

## 2. Video

### What works

- **Frame-level image detectors averaged over frames.** Still the strongest
  simple approach. AIGVDBench (CVPR 2026: 31 generators, 440k clips, 33
  detectors) found image detectors retrained on video frames at the top of its
  tables, ahead of video-specific architectures.
- **Video architectures** (DeMamba, DeCoF, AIGVDet, DuB3D, UNITE). They model
  temporal inconsistency directly. DeMamba is the most cited, but it has no
  official released weights and third-party tests put it near 84% balanced
  accuracy.
- **Training-free statistics** (D3, STALL, ReStraV, SPLIT). They measure how
  frame embeddings move over time. Attractive, but the headline numbers have
  not held up: an independent reproduction puts D3 at 0.72 AUROC on GenVideo
  against the 0.98 it reported.
- **Multimodal LLM detectors** (VideoVeritas, BusterX++, VidGuard-R1, MM-Det).
  They explain their verdicts but need 10–18 GB of weights and data-centre
  GPUs, and AIGVDBench measured general-purpose VLMs near chance.

### Who leads among open video detectors

**WaveRep** (GRIP-UNINA, NeurIPS 2025) is the strongest detector that actually
ships weights and runs outside a CUDA cluster. It is a DINOv2 ViT-B/14 applied
to 504 px centre crops of the first 64 frames, with the frame logits averaged.
Its wavelet-rescaling augmentation makes it generalise from just four training
generators: the authors report 94.3% balanced accuracy averaged over 15
generators, against 84.0 for DeMamba and 73.9 for AIGVDet. It is the video
baseline here. Its licence is research-only.

### Benchmarks worth using

- **GenVideo-Val** – 10 generators against MSR-VTT, the standard held-out
  split for the field.
- **ComGenVid** – Sora and Veo 3 against MSVD.
- **WasItAIGenerated frontier benchmark** – 12 video models of late 2026
  (Veo 3, Kling 3, Seedance 2, Wan 2.6, ...) with messenger-compressed copies.
- **AIGVDBench**, **GenVidBench** – large and current, but their real halves
  ship as single 80–94 GB archives.

## 3. What this implies for a new system

1. **Data recency and diversity decide accuracy.** The architecture can stay
   small if the training set covers many generators, including this year's.
2. **Look at more than one scale.** Resizing a 1024 px image to a few hundred
   pixels throws away the pixel-level fingerprint; a native-resolution crop
   keeps it. The two views fail on different images.
3. **Train against the transformations that happen in the wild** – JPEG at
   varying quality before and after resizing, several resampling kernels,
   and for video random H.264 re-encoding – so that the decision does not rest
   on fragile traces.
4. **Speed comes from resolution and from the pipeline around the network.**
   A ViT-S at 224 px costs 3.4× less than the same network at 384 px, and
   decoding and resizing are a large share of end-to-end time.
5. **Frame-level video detection is competitive**, and it lets images and
   video share one engine.

## 4. What building it added

Three things the survey did not predict, found while testing
([BENCHMARK.md](BENCHMARK.md) has the numbers):

- **Real footage that never came from a camera** – cartoons, 3D animation,
  motion graphics, gameplay – was the main source of false alarms on video.
  The usual real sets (Kinetics, MSR-VTT, MSVD) contain some of it; the usual
  training sets almost none. Adding about 2,400 such clips to training fixed
  most of it.
- **A frame-level ViT-S and WaveRep miss different clips.** Scoring only the
  uncertain clips with the second model beats either alone and costs a
  fraction of running the second model on everything.
- **A long PyTorch run on the Metal backend can go silently wrong.** Two
  reference runs produced plausible but false scores after several hundred
  clips. Re-implementing the reference detectors in the same engine as the
  system under test, and checking the ports against short fresh runs, removed
  that risk.

## Sources

- Community Forensics: <https://arxiv.org/abs/2411.04125>, <https://github.com/JeongsooP/Community-Forensics>
- Out-of-the-box benchmark of open detectors: <https://arxiv.org/abs/2602.07814>
- Raising the bar of AI-generated image detection with CLIP: <https://openaccess.thecvf.com/content/CVPR2024W/WMF/papers/Cozzolino_Raising_the_Bar_of_AI-generated_Image_Detection_with_CLIP_CVPRW_2024_paper.pdf>
- OpenFake: <https://huggingface.co/datasets/ComplexDataLab/OpenFake>
- So-Fake-OOD: <https://huggingface.co/datasets/saberzl/So-Fake-OOD>
- Synthbuster: <https://zenodo.org/records/10066460>
- AIGIBench: <https://huggingface.co/datasets/HorizonTEL/AIGIBench>
- WasItAIGenerated frontier benchmark: <https://huggingface.co/datasets/wasitaigeneratedcom/frontier-image-video-bench-2026-10>
- AIGVDBench: <https://arxiv.org/abs/2601.11035>
- WaveRep: <https://github.com/grip-unina/WaveRep-SyntheticVideoDetection>
- GenVideo / DeMamba: <https://github.com/chenhaoxing/DeMamba>
- ComGenVid / STALL: <https://huggingface.co/datasets/OmerXYZ/comgenvid>
- Awesome AIGC image/video detection list: <https://github.com/ant-research/Awesome-AIGC-Image-Video-Detection>
