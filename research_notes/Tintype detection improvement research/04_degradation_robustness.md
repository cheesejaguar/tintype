# Robustness of AI-generated image detectors to degradations, laundering and adversarial attacks (what can fix Tintype's weak spots)

Scope note. Tintype is a ViT-S/16 at 224 px. It scores six views: a fit view plus the centre crop and four most-textured native-resolution crops. Its training augmentation is random JPEG before and/or after resize, random kernels, random-scale crops, and a 25% "messenger" round trip (long side 960–1700 px, JPEG QF 55–90). The weak spots this note targets are below. Each comes from the repo's own docs (README "Limits", docs/EVIDENCE.md).
- NTIRE 2026 public test, distorted cohort: 2.2% recall at 1% FPR, AUROC 0.636. At the shipped cutoff it flagged 155 of 600 authentic images (25.8%). 110 of those 155 false positives sit in the 182 authentic images that went through JPEG AI, Cheng2020 learned compression or WMForger (60.4% flagged), against 10.8% for the other distorted authentic images. The nine challenge entrants scored 0.830–0.974 robust AUROC on the same public split.
- Hard validation, distorted: 5.4% recall at 1% FPR, AUROC 0.734.
- WhatsApp/Instagram copies of 2026 AI images: 27–36% recall at <1% FPR (default), 55–59% at 2–3% FPR (`--sensitive`).
- Arena: 88.5% falls to 73.1% recall at 1% FPR after JPEG re-encoding both classes. So-Fake-OOD social JPEGs: 77.0%.
- Frontier set: 69.3% (original), 53.3% (same-size JPEG) and 28.0% (JPEG capped at 960 px) at 1% FPR.

All "Tintype applicability" notes below are my inferences, not findings of the cited papers.

---

## Q1. NTIRE 2026 Robust AIGI Detection challenge and earlier challenges: distortions, winning recipes, scores

### Takeaway
The NTIRE 2026 robust track used 36 distortion types, chained 1–5 per image: blur, noise, colour/tone, crops/perspective, classic, multiple and neural compression (JPEG AI, Cheng2020), invisible watermarks and a watermark attack (WMForger). Every top team trained with that kind of chained, graded degradation augmentation, most at 384–512 px on large foundation backbones (DINOv3, SigLIP2, EVA-CLIP, MetaCLIP2), and most ensembled. The winners reached 0.97 robust AUROC; Tintype reached 0.636 on the public distorted split. The largest gap to the field is that Tintype's training never covered these distortion families, especially neural codecs and watermark operations. Model size is a smaller part of it: a single SigLIP2 model with heavy official-distortion augmentation still reached 0.86–0.88.

### Cited Findings

**[P1] NTIRE 2026 Challenge on Robust AI-Generated Image Detection in the Wild.** Gushchin, Abud, Shumitskaya, Filippov, Bychkov, Erofeev, Lavrushkin, Antsiferova, Vatolin, …, Timofte. CVPR 2026 NTIRE workshop report, arXiv:2604.11487 (Apr 2026). — [arXiv HTML](https://arxiv.org/html/2604.11487v1)
- Data: 108,750 real and 185,750 generated images from 42 generators. The training set (~277K images) has CC12M/CommonPool/RedCaps reals and 20 open generators. Test generators are mostly unseen and include proprietary models. Public and private test splits have 2.5K images each, about half of them distorted. — [P1](https://arxiv.org/html/2604.11487v1)
- Each distorted image gets 1–5 consecutive distortions at random magnitudes. The families are:
  - blur: Gaussian, lens, motion, glass
  - noise: white, impulse, shot, ISO, multiplicative
  - compression: JPEG, multiple JPEG, JPEG 2000, neural codecs (JPEG AI, Cheng2020), and multi-JPEG combined with JPEG AI or JPEG 2000
  - colour/tone: shift, saturation, brightness ±, jitter, quantization, linear contrast, RGB shift, tone curve, CLAHE
  - geometry: random crop, aspect crop, downscale, pixelation, perspective
  - watermarks: 6–7 invisible watermark algorithms, plus a watermark attack (WMForger)

  The primary metric is robust ROC AUC on distorted images, averaged over the open and hidden splits. — [P1](https://arxiv.org/html/2604.11487v1)
- Final ranking, average robust (clean) AUROC:

  | Rank | Team | Robust | Clean |
  |---:|---|---:|---:|
  | 1 | MICV | 0.9723 | 0.9974 |
  | 2 | Ant International | 0.9721 | 0.9972 |
  | 3 | TeleAI-TeleGuard | 0.9251 | 0.9786 |
  | 4 | INTSIG | 0.9130 | 0.9897 |
  | 5 | vincentlc | 0.8730 | 0.9527 |
  | 6 | UESTC | 0.8679 | 0.9729 |
  | 7 | Reagvis | 0.8603 | 0.9452 |
  | 8 | PSU | 0.8408 | 0.9227 |
  | 9 | Shallow Real | 0.8336 | 0.9953 |

  Shallow Real is the warning case: 0.995 clean but only 0.834 robust. — [P1](https://arxiv.org/html/2604.11487v1)
- **MICV (1st):**
  - Backbones: two streams of DINOv3 backbones (4 and 2), each with a projection layer and an MLP head; the two streams' probabilities are averaged.
  - Training data: GenImage, WildFake, AIGIBench, Community Forensics, So-Fake-Set, Qwen-Image, Z-Image, FLUX, plus closed-API outputs (Seedream, Kling, GPT-Image, Nano-banana-pro).
  - Augmentation: a "hierarchical, difficulty-aware" pipeline that goes from single distortions to combined multi-stage degradations.
  - Resolution: random crop and resize to 512 for training, plain resize to 512 for inference.
  - Training: focal loss (γ=2), AdamW lr 1e-5, stochastic weight averaging over the last epochs; 32 A100s for about 8 hours.
  - [P1](https://arxiv.org/html/2604.11487v1)
- **Ant International (2nd):**
  - Backbones: two fully fine-tuned DINOv3-7B experts. The team chose DINOv3-7B over EVA02-L, EVA-giant and SigLIP2-giant.
  - Data: about 1M images with offline four-level augmentation (clean, mild, moderate, heavy).
  - Expert 1: 512 px, levels 2–4, AugMix, EMA. Expert 2: 288 px, levels 1–4.
  - Test-time augmentation per expert, then weighted averaging.
  - Cost: 2.21 img/s and 78 GB VRAM on one A100.
  - [P1](https://arxiv.org/html/2604.11487v1)
- **vincentlc (5th):** a single SigLIP2-giant at 384 px with a linear head. The image is squished to 384 with no crop. Training applied the official distortion function to every image (probability 1.0): up to 3 operations at 5 severity levels. The team names this aggressive distortion augmentation as the key driver of robustness. — [P1](https://arxiv.org/html/2604.11487v1)
- **Reagvis (7th):** a six-model cascade that includes an SRM+Bayar forensic CNN, with 8-view test-time augmentation per model (flips, multi-scale and corner crops, blur, JPEG). Models run sequentially, so peak VRAM stays under 4 GB. **PSU (8th):** seven or eight encoders with LayerNorm-only tuning and flip TTA. **UESTC (6th):** CLIP-L and SigLIP experts with feature self-distillation. — [P1](https://arxiv.org/html/2604.11487v1)
- Organizers' conclusions: robustness, not clean accuracy, separated the teams; most solutions were transformer ensembles; the problem is not solved. — [P1](https://arxiv.org/html/2604.11487v1)
- **Tintype applicability (P1):**
  - The distortion list is a ready specification for a new training augmentation library. Tintype's current augmentation covers only the JPEG/resize/crop part.
  - The 110/182 false-positive concentration on JPEG AI, Cheng2020 and WMForger is a training-coverage gap: neural codecs and watermark embed/attack are absent from Tintype's pipeline.
  - A 7B ensemble is not a fit for a fast local tool, but vincentlc's single-model result shows augmentation alone gets a large share of the gain.
  - Expected effect: large AUROC gain on NTIRE-like distortions. The repo has no measurement yet.
  - Risk: tuning to NTIRE's own distortion generator (benchmark overfitting). Hold out the NTIRE public test, and keep the WhatsApp/Instagram and Arena-JPEG checks as independent tests.

**[P2] Boosting Robust AIGI Detection with LoRA-based Pairwise Training (LPT).** Ruiyang Xia, Qi Zhang et al. (TeleAI), NTIRE 2026 3rd place, arXiv:2604.12307. — [arXiv](https://arxiv.org/html/2604.12307v1)
- Method: EVA-CLIP with LoRA (rank 16) on the attention and FFN layers. Each batch holds the clean and distorted version of the same image. The loss is cross-entropy, plus KL divergence between the two predictions, plus feature MSE through a "corrector" FFN that maps distorted features toward clean ones. Training and inference run at 224×224. — [P2](https://arxiv.org/html/2604.12307v1)
- Loss ablation (AUC %, validation / public test): CE alone 97.31 / 87.91; CE+KL 97.38 / 88.06; CE+KL+MSE 97.79 / 89.26. That is +1.35 public-test AUC from the pairwise terms. — [P2](https://arxiv.org/html/2604.12307v1)
- Simulation ablation (public test): no size/distortion simulation 89.26 → +size 89.27 → +distortion 89.58. The extra distortions were speckle noise, tone curve, organic moiré and colour jitter, with severity drawn around a mean of 3 out of 5. The paper does not report robust-only AUC or false-positive analysis. — [P2](https://arxiv.org/html/2604.12307v1)
- **Tintype applicability (P2):**
  - It runs at 224 px like Tintype and adds no inference cost, so it is the most directly portable top-3 recipe.
  - Change: each training step scores (clean, degraded) pairs of the same image; add a symmetric KL on the logits and a cosine/MSE loss on the CLS feature. A corrector head is optional and used only in training.
  - Expected effect: modest but free, about 1–1.5 AUC on NTIRE-like data per P2. Its main value is that scores on degraded copies stay tied to the clean score, which should reduce calibration drift (the 155-false-positive problem).
  - Cost: about 2× forward passes per training sample.
  - Risk: little; if the degraded view gets too much weight, clean accuracy can dip (compare P4's −0.9).

**[P3] HEDGE: Heterogeneous Ensemble for Detection of AI-GEnerated Images in the Wild.** Fei Wu, Dagong Lu, Mufeng Yao, Xinlei Xu, Fengjun Guo (INTSIG), arXiv:2604.03555; 4th of 193 teams. — [arXiv](https://arxiv.org/abs/2604.03555)
- Three routes: (A) DINOv3 models built by staged data expansion and escalating augmentation; (B) a 448 px high-resolution branch; (C) a MetaCLIP2 branch. They are fused by logit-weighted averaging with dual gating for outliers. — [P3](https://arxiv.org/abs/2604.03555)
- Per the challenge report, extra data included SoFake-OOD, RRDataset (re-digitized and social-transmitted images), Chameleon and AIGIBench, plus a stage with 5 distortions × 5 levels and flip TTA. — [P1](https://arxiv.org/html/2604.11487v1)
- **Tintype applicability (P3):**
  - The staged recipe maps onto Tintype's existing phase training: data expansion first, then escalating distortion severity, then a high-resolution branch.
  - RRDataset (see P26) is a cheap public source of social-transmitted and re-captured reals and fakes. Use it for training or validation.
  - Risk: a gated ensemble adds complexity. Tintype already gates a second opinion on uncertain cases.

**Earlier challenges:**
- IEEE SP Cup 2025 (DFWild-Cup, ICASSP 2025) was a face-deepfake generalization challenge built from 8 public datasets (Celeb-DF, FF++, FaceShifter, …). — [SPS page](https://signalprocessingsociety.org/node/12307)
- One team reports winning it with an ensemble of fine-tuned DINOv2, AIMv2 and OpenCLIP ViT-L/14: AUC 96.77%, EER 9%. I found no official results page confirming the ranking. — [Towards Generalizable Deepfake Image Detection with ViTs, arXiv:2604.17376](https://deepfake-demo.aisec.fraunhofer.de/related_work/2604.17376)
- Another entry (DFCon, arXiv:2501.16704) used MaxViT, CoAtNet and EVA-02 with supervised contrastive loss and offline real-class augmentation (21,335 extra real samples). — [DFCon](https://arxiv.org/html/2501.16704v1)
- Same lesson as NTIRE: ensembles of large ViTs plus augmentation. These are face-swap datasets and less relevant to Tintype's fully-generated-image scope.

### Inferences
- What separates NTIRE winners from the rest is (a) training-time coverage of the test distortion families, applied in chains and graded by difficulty; (b) a large, modern self-supervised or vision-language backbone; (c) 384–512 px inputs; (d) ensembling or TTA. Of these, (a) is cheap for Tintype. (c) partially conflicts with Tintype's 224 px speed budget, although its native crops already preserve detail. (b) and (d) conflict with Tintype's local-speed design and are best confined to a gated second opinion.
- Tintype's NTIRE false positives are a calibration and coverage failure on real images, not just a recall failure. The shipped cutoff flags 25.8% of distorted reals, and the codec/watermark group accounts for most of them. NTIRE scores only AUROC, so the teams' papers report nothing on thresholds or calibration under distortion. Tintype needs its own fix (Q3).

### Gaps
- I did not find standalone papers for MICV or Ant International. Their exact distortion curriculum (levels, probabilities) is known only from the report's summary.
- No NTIRE team reported false-positive rate at a fixed threshold, or per-distortion AUROC. Which distortion families dominate the winners' remaining errors is unknown.
- I found no official ACM MM 2024/2025 or Kaggle AIGI-robustness challenge reports within the tool budget. The "DeepFake Game Competition" was not located.

---

## Q2. Social-network laundering: how much do shares hurt, and what training closes the gap?

### Takeaway
Real social-platform pipelines resize (to roughly 720–1200 px) and recompress (QF about 61–92). Detectors without matching augmentation collapse, either in recall or in true-negative rate. The best published in-the-wild results come from crop-based (no resize) training on content-aligned real/fake pairs with blur and JPEG augmentation (B-Free: 86–97% balanced accuracy on WildRF), not from bigger heads. Repeated re-sharing and re-digitization (print/screen re-photography) remain the hardest conditions.

### Cited Findings

**[P4] TrueFake: A Real World Case Dataset of Last Generation Fake Images also Shared on Social Networks.** Stefano Dell'Anna, Andrea Montibeller, Giulia Boato (U. Trento). arXiv:2504.20658 (2025); the repository name suggests IJCNN 2025. — [arXiv](https://arxiv.org/html/2504.20658)
- 600K images: 100K real (FFHQ, FORLAB); 120K from StyleGAN 1/2/3; 200K from SD1.5, SD2.1, SD3, SDXL and FLUX.1. A 60K subset was shared through Facebook, X and Telegram, giving 180K shared images. — [P4](https://arxiv.org/html/2504.20658)
- Measured platform processing:

  | Platform | JPEG QF | Resize | PSNR |
  |---|---|---|---|
  | Facebook | 61–92 (dynamic) | width capped at 720 | 36.2 dB |
  | X | 87 | width capped at 1200; ≤768×768 left untouched | 39.3 dB |
  | Telegram | 85 | height capped at 800 | 37.1 dB |

  Facebook is the most disruptive. — [P4](https://arxiv.org/html/2504.20658)
- Detectors were trained on non-shared data only:
  - NPR and P2G collapse on Facebook StyleGAN2 images (TPR 0.00 and 0.02).
  - CLIP-D keeps TPR 0.80 but its TNR on Facebook FFHQ reals falls to 0.79 (R50-E2P 0.74, MISLNet 0.41).
  - MISLNet's TPR rises after sharing while its TNR collapses. Sharing pushes reals toward "fake" for some detectors.
  - Detectors trained with JPEG and resize augmentation (CLIP-D, R50-ND) hold up best.
  - [P4](https://arxiv.org/html/2504.20658)
- **Tintype applicability (P4):**
  - Tintype's messenger round trip caps the long side at 960–1700 px. Facebook's 720 px width and Telegram's 800 px height fall below that range. Extend it to about 640–2048 px, with caps on width, height or long side, and include X's skip rule for small images.
  - Add double-compression chains (P26 shows 2–6 share cycles).
  - TrueFake's shared FFHQ/FORLAB reals also make a free false-positive test.
  - Expected effect: better recall on WhatsApp/Instagram-like copies at the same FPR.
  - Risk: low; the training distribution simply widens.

**[P5] A Bias-Free Training Paradigm for More General AI-generated Image Detection (B-Free).** Fabrizio Guillaro, Giada Zingarini, Ben Usman, Avneesh Sud, Davide Cozzolino, Luisa Verdoliva. CVPR 2025; arXiv:2412.17671. — [arXiv HTML](https://arxiv.org/html/2412.17671v2)
- Training data: 51,517 COCO reals. Fakes are SD2.1 "self-conditioned" regenerations (inpainting with an empty mask) plus object-replacement inpainting, six fakes per real, 309K in total. Augmentation: blur and JPEG (CNNDetect-style), and in the "++" variant also scaling, cut-out, noise and jitter. — [P5](https://arxiv.org/html/2412.17671v2)
- Model: DINOv2+registers fine-tuned end to end on **504×504 crops with no resizing**. Inference averages logits over multiple 504 crops; images smaller than 504 px are padded. — [P5](https://arxiv.org/html/2412.17671v2)
- WildRF balanced accuracy (Facebook / Reddit / Twitter): B-Free 95.6 / 86.2 / 97.3; DMID 87.8 / 74.3 / 79.1; AIDE 56.9 / 72.1 / 57.3; RINE 52.8 / 67.7 / 56.0. SynthWildX: B-Free 95.6, DMID 76.6. — [P5](https://arxiv.org/html/2412.17671v2)
- Viral images re-posted over 100 days: B-Free about 92% average accuracy; most competitors fall below 70% after one day. — [P5](https://arxiv.org/html/2412.17671v2)
- Simulated sharing on FakeInversion (resize ×0.7–1, JPEG QF 70–100):
  - B-Free: AUC 99.3 → 98.5, bAcc 86.3 → 86.4
  - DMID: bAcc 96.1 → 81.3
  - NPR: down to near chance (AUC 43.3)

  Calibration (ECE) improves from 0.185 to 0.038 as augmentation is added. — [P5](https://arxiv.org/html/2412.17671v2)
- **Tintype applicability (P5):**
  - Tintype already shares two ingredients: native-resolution crops and logit averaging.
  - The missing ingredient is content-aligned negatives. Add regenerations of Tintype's own real photos through current VAEs/regenerators (SD/FLUX inpainting with an empty mask). Fake and real then differ only in generator traces, not content, format or size.
  - Expected effect: less shortcut learning on content and format. That should help ranking after re-encoding (Arena 88.5→73.1, Frontier 69.3→28.0) and calibration.
  - Cost: generating about 50–300K reconstructions, a one-off GPU job.
  - Risk: the model may learn one VAE's fingerprint. Use several (SD1.x/2.x/XL/3, FLUX VAEs).
  - Note: B-Free's backbone is a ViT-B at 504 px; the result may not fully transfer to ViT-S at 224.

**[P6] Real-Time Deepfake Detection in the Real-World (LaDeDa, Tiny-LaDeDa, WildRF).** Bar Cavia, Eliahu Horwitz, Tal Reiss, Yedid Hoshen (HUJI). arXiv:2406.09398 (2024). — [arXiv](https://arxiv.org/html/2406.09398v1)
- A patch-level (9×9) scorer with pooling, near-perfect on simulated benchmarks but much weaker on WildRF, which collects real and fake images from Reddit, X and Facebook. The best result is 93.7% mAP on WildRF; the authors say real-world detection is unsolved. — [P6](https://arxiv.org/html/2406.09398v1)
- **Tintype applicability (P6):** WildRF is a public held-out social test set to add to Tintype's evaluation (B-Free's numbers give a target). Patch scoring like LaDeDa's is fragile to recompression: it reads tiny local traces, which supports keeping a global fit view and not relying only on crops.

**[P7] Fake or JPEG? Revealing Common Biases in Generated Image Detection Datasets.** Patrick Grommelt, Louis Weiss, Franz-Josef Pfreundt, Janis Keuper. ECCV 2024 Workshops; arXiv:2403.17608; DOI 10.1007/978-3-031-92089-9_6. — [arXiv](https://arxiv.org/pdf/2403.17608) · [mlanthology](https://mlanthology.org/eccvw/2024/grommelt2024eccvw-fake)
- GenImage-trained detectors learn JPEG-compression and image-size shortcuts. Removing these biases substantially increases robustness to JPEG compression and raises cross-generator performance by more than 11 points for ResNet-50 and Swin-T. — [P7](https://arxiv.org/pdf/2403.17608)
- **Tintype applicability (P7):**
  - Tintype's drop when both Arena classes are re-encoded (88.5→73.1) is consistent with a partial format shortcut.
  - Change: check the training manifest for per-class format, QF-distribution and size imbalance. Force fakes and reals through matched JPEG-QF and size distributions so that "has camera-JPEG history" cannot mean "real".
  - Expected effect: better ranking on uniformly re-encoded inputs, and fewer real images flagged when they are PNG or unusually compressed.
  - Cost: data pipeline only.
  - Risk: a small loss on uncompressed generator output.

**[P8] Is JPEG AI going to change image forensics?** Edoardo D. Cannas, Sara Mandelli, Nataša Popović, Ayman Alkhateeb, Alessandro Gnutti, Paolo Bestagini, Stefano Tubaro. arXiv:2412.03261 (v2, Mar 2025). — [arXiv HTML](https://arxiv.org/html/2412.03261v2)
- Pristine images compressed with JPEG AI are pushed toward "synthetic". False-positive rate on pristine images at 0.12 bpp vs uncompressed:

  | Detector | Uncompressed | JPEG AI 0.12 bpp |
  |---|---:|---:|
  | Corvi | 0.00 | 0.97 |
  | Gragnaniello | 0.03 | 0.96 |
  | NPR | 0.13 | 0.99 |
  | Mandelli | 0.16 | 0.96 |

  The effect fades as bitrate rises (e.g. Corvi 0.03 at 0.5 bpp), while MS-SSIM stays ≥0.97. — [P8](https://arxiv.org/html/2412.03261v2)
- Conventional JPEG (QF 65–95) keeps false positives flat or lowers them; it mostly raises false negatives. JPEG AI does the opposite. — [P8](https://arxiv.org/html/2412.03261v2)
- Mechanism: upsampling and interpolation in the neural decoder leave spectral peaks like those of generators. — [P8](https://arxiv.org/html/2412.03261v2)
- Mitigation: retraining with 20% JPEG AI samples cut the false-positive rate at 0.12 bpp only from 0.96 to 0.65. Balanced accuracy stayed below 0.7, and false negatives rose. The authors say much larger augmentation campaigns would be needed. — [P8](https://arxiv.org/html/2412.03261v2)
- **Tintype applicability (P8):** this is the most direct explanation for Tintype's NTIRE false positives (110/182 in the JPEG AI, Cheng2020 and WMForger group).
  - Change 1: add learned-codec round trips (JPEG AI reference software; CompressAI's Cheng2020, Ballé hyperprior and similar, at several bitrates) to **both** classes in training. Applying them to reals only would teach "neural-codec trace = real", which an attacker can use as a laundering route.
  - Change 2: since P8 shows augmentation alone only partly fixes it, also add a cheap neural-codec/degradation detector (see Q3, P14–P16). When it fires, the score is down-weighted or the verdict becomes "uncertain".
  - Expected effect: a large cut in the codec-related false positives. A complete fix is unlikely.
  - Cost: CompressAI is pip-installable; encoding is GPU-light.
  - Risk: some recall loss on fakes that look like learned-codec output. VAE-decoded generators share upsampling traces, so full invariance to codec traces could also weaken detection of real generator traces. Measure on NTIRE public test fakes.

**[P9] When Synthetic Traces Hide Real Content: Analysis of Stable Diffusion Image Laundering.** Sara Mandelli, Paolo Bestagini, Stefano Tubaro. arXiv:2407.10736 (2024; WIFS 2024 per the authors' group). — [arXiv](https://arxiv.org/pdf/2407.10736)
- Passing a real photo through Stable Diffusion's autoencoder gives a near-identical copy that detectors read as synthetic. The authors propose a two-stage pipeline: pristine vs synthetic, then fully synthetic vs laundered. Laundering also masks camera-model traces. Code: polimi-ispl/synthetic-image-detection. — [P9](https://arxiv.org/pdf/2407.10736)
- **Tintype applicability (P9):** this is the same family as P8. Any VAE or neural-codec decode looks "generated". Tintype must decide on policy: is a VAE-laundered real photo "AI" (Tintype's training labels say it is generated) or "real"? Either way, it should not confuse a standard codec (JPEG AI is a future camera and web format) with generation. A three-way output head ("real", "generated", "neural-codec/laundered") is one cheap way to keep precision.

**[P10] Deepfake-Eval-2024: A Multi-Modal In-the-Wild Benchmark of Deepfakes Circulated in 2024.** Nuria Alina Chandra et al. (TrueMedia.org). arXiv:2503.02857. — [arXiv](https://arxiv.org/html/2503.02857v1)
- 1,975 in-the-wild images that users flagged in 2024. Open image detectors lose about 45% AUC relative to earlier benchmarks. Fine-tuning on representative data helps but does not reach analyst level. — [P10](https://arxiv.org/html/2503.02857v1)
- **Tintype applicability (P10):** another external test set of real circulated media. It confirms that in-distribution social data in training is the strongest lever.

**[P11] Raising the Bar of AI-generated Image Detection with CLIP.** Davide Cozzolino, Giovanni Poggi, Riccardo Corvi, Matthias Nießner, Luisa Verdoliva. CVPR 2024 Workshops (WMF); arXiv:2312.00195. — [arXiv](https://arxiv.org/pdf/2312.00195)
- The paper notes that WebP is spreading on social networks but is rarely tested. UnivFD (Ojha) loses up to about 10 points under JPEG or WebP. Corvi's detector, trained with strong augmentation, stays robust to strong WebP even though WebP was not in its training. The paper also introduced SynthWildX (images shared on X). — [P11 via search summary](https://arxiv.org/pdf/2312.00195)
- **Tintype applicability (P11):** add WebP (and AVIF) round trips to the messenger augmentation. Strong JPEG augmentation partly generalizes to WebP, but cheap explicit coverage is safer.

### Inferences
- Tintype's messenger augmentation is the right idea but too narrow: one cycle, long side ≥960, JPEG only. Measured platforms go down to 720 px width (Facebook), use dynamic QF from 61, and real-world images are often shared 2–6 times (P26). Widening this range and chaining cycles is low-risk and cheap.
- The two best in-the-wild recipes (B-Free; NTIRE top teams) both use native-resolution crops or large inputs, augmentation in both classes, and content-aligned or very diverse data.
- Social sharing can move a detector's errors in either direction. TrueFake's MISLNet and Cannas's JPEG AI results show reals pushed toward "fake". Tintype's evaluation should always report real-image false-positive rate per processing type, not only recall at 1% FPR on a pooled set.

### Gaps
- I found no controlled study isolating WhatsApp or Instagram processing parameters (QF, caps, chroma) as TrueFake did for Facebook, X and Telegram.
- TrueFake did not test training on shared images. No paper here quantifies "train on real shared copies" against "simulate sharing".
- I could not confirm a 2024–2026 paper titled exactly "Robustness of AI-image detectors to social media sharing", or one titled "Detecting images generated by diffusers in the wild". Synthbuster laundering-specific results were not retrieved.

---

## Q3. Degradation-aware, quality-conditioned and consistency-trained detection: does any of it cut false positives on degraded reals?

### Takeaway
Three useful families exist:
1. Paired clean/degraded consistency training (DCPT, LPT, GlobalForge). It raises degraded accuracy at almost no clean cost and no inference cost.
2. Explicit degradation or quality estimation used for calibration, abstention or routing: QuAD (quality-conditioned likelihood ratio), DACOM (ICLR 2026: predicts per-sample "can the detector handle this?"), HiDA-Net (JPEG-QF regression auxiliary head).
3. Representation surgery: pruning fragile channels (DEAR).

Only the quality-conditioned calibration family directly addresses scores on degraded reals: QuAD models how real and fake logit distributions widen as quality drops. None of these papers reports false-positive rate on degraded real images, so the claim needs Tintype's own NTIRE false-positive test. I found no strong evidence that "denoise/restore-then-detect" helps. Restoration networks are themselves generative and plausibly add fake-looking traces (inference from P8/P9).

### Cited Findings

**[P12] Degradation-Consistent Paired Training for Robust AI-Generated Image Detection (DCPT).** Zongyou Yang, Yinghan Hou, Xiaokun Yang. arXiv:2604.10102 (Apr 2026). — [arXiv HTML](https://arxiv.org/html/2604.10102v1)
- Setup: a frozen DINOv2 ViT-B/14 with registers and a 197K-parameter MLP head, trained on the B-Free data. Each image yields a clean 504 crop and, with probability 0.5, a degraded view (JPEG QF 30–90, blur σ 1–3, downscale 0.25–0.75). Loss: CE on both views, plus 0.5 × feature cosine distance, plus 0.1 × symmetric KL with a stop-gradient on the clean view. — [P12](https://arxiv.org/html/2604.10102v1)
- Synthbuster accuracy (9 generators), baseline → DCPT: clean 86.2 → 85.3; average over 8 degradations 64.0 → 73.1 (+9.1); JPEG Q70/50/30 56.8 / 53.3 / 52.3 → 72.5 / 71.2 / 69.5; downscale 0.25 57.7 → 68.0. — [P12](https://arxiv.org/html/2604.10102v1)
- AUC: clean 89.1 → 87.9; degraded 78.5 → 79.4. Accuracy gains are larger than AUC gains, so a good part of the benefit is **calibration under degradation** rather than ranking. — [P12](https://arxiv.org/html/2604.10102v1)
- No inference overhead. Adding an SRM branch hurt (overfitting). No false-positive rate is reported. — [P12](https://arxiv.org/html/2604.10102v1)
- **Tintype applicability (P12):**
  - This matches Tintype's symptom: a shipped threshold that drifts on degraded inputs.
  - Change: train with paired views, KL plus feature consistency, as for P2.
  - Expected effect: fewer degraded reals crossing the cutoff, and better recall at the shipped threshold on messenger copies. AUROC gains may be small.
  - Cost: about 2× training forward passes.
  - Risk: slight clean loss (−0.9 accuracy in P12).

**[P13] GlobalForge: Towards Robust AI-Generated Image Detection.** Manni Cui, Ruiqi Liu, …, Shu Wu. arXiv:2607.14684 (Jul 2026). — [arXiv](https://arxiv.org/abs/2607.14684)
- Argues that detectors overfit local artifacts that compression and blur destroy. Adds a Local Information Bottleneck, Global Structural Reasoning and a degradation-based contrastive structural loss. Introduces RealDeg-Bench (7 operators plus compound chains). Reports +5.89% average balanced accuracy over the prior best on 8 in-the-wild groups. Per-degradation numbers were not retrieved. — [P13](https://arxiv.org/abs/2607.14684)
- **Tintype applicability (P13):** RealDeg-Bench's compound chains are a useful additional robustness test. The architectural idea is consistent with keeping Tintype's global fit view in the average. A cheap variant: weight the fit view more heavily when a degradation estimator says local detail is gone.

**[P14] Quality-Aware Calibration for AI-Generated Image Detection in the Wild (QuAD).** Fabrizio Guillaro, Vincenzo De Rosa, Davide Cozzolino, Luisa Verdoliva. arXiv:2604.15027 (Apr 2026). — [arXiv HTML](https://arxiv.org/html/2604.15027v1)
- Method:
  - A no-reference IQA model (LoDa; TReS and QCN give similar results) estimates each copy's quality q.
  - For each detector, the logit is modelled as a Gaussian whose mean and variance depend linearly on q, fitted separately for real and fake (8 coefficients, maximum likelihood).
  - The corrected score is the log-likelihood ratio, summed over near-duplicate copies. Low-quality copies count less.
  - [P14](https://arxiv.org/html/2604.15027v1)
- Finding: as degradation increases, both the real and fake logit distributions shift and spread, and they overlap more. This holds across the detectors studied. — [P14](https://arxiv.org/html/2604.15027v1)
- Results: AncesTree, six-detector average, 81.6% bAcc vs 73.2% for naive aggregation and 70.7% for a single image. ReWIND (real viral images and their web copies): 70.3% vs 63.0%. QuAD was evaluated only with near-duplicates, never on a single image. — [P14](https://arxiv.org/html/2604.15027v1)
- **Tintype applicability (P14):**
  - Single-image version: compute a degradation or quality score per image. It could come from a small NR-IQA model, a regression head on Tintype's CLS feature (see P16), or simple statistics such as JPEG table quality estimate, resolution and noise level. Then fit per-class, quality-conditioned Gaussians of Tintype's logit on held-out data (real and fake, many degradations) and report the LLR instead of the raw logit.
  - Expected effect: directly targets the 155/600 false positives, because low-quality reals' high logits get discounted. It is a monotone recalibration within a quality bin, so ranking within a bin is unchanged. Across bins, pooled AUROC and recall at fixed FPR improve.
  - Cost: tiny at inference; needs a calibration set covering degradations.
  - Risks: (a) an attacker can degrade a fake to lower its LLR. This is unavoidable and matches the information-theoretic limits in P19. (b) A quality estimator can mistake generator smoothness for degradation. Train it on both classes.
  - Bonus: for video or near-duplicate search (web copies), QuAD's multi-copy aggregation applies directly.

**[P15] Enabling Your Forensic Detector Know How Well It Performs on Distorted Samples (DACOM / Distortion-Aware Confidence).** ICLR 2026 poster (authors not retrieved). — [ICLR page](https://iclr.cc/virtual/2026/poster/10010162) · [OpenReview PDF](https://openreview.net/pdf/9a9ad3cc252735dcd9be4a445b8f7f984d73a689.pdf)
- Detection accuracy falls almost monotonically with full-reference quality as distortion grows. Full-reference IQA is used only during training, as an oracle for "detectable". At test time a light model combines the detector's intermediate features, no-reference IQA descriptors and distortion-type cues to predict per-sample confidence. It is used for selective abstention and for routing among detectors. It beats logit-based confidence and off-the-shelf IQA. One reported figure (taken from a secondary summary; verify): 64.19% bAcc, 32.23% EER with 40% of samples filtered in a cross-dataset multi-distortion test. — [ICLR page](https://iclr.cc/virtual/2026/poster/10010162); figure via [Liner review](https://liner.com/review/enabling-your-forensic-detector-know-how-well-it-performs-on)
- **Tintype applicability (P15):**
  - This fits Tintype's existing architecture closely. Tintype already routes uncertain photos to a Community Forensics second opinion and has an "uncertain" notion via early exit.
  - Change: train a small confidence head on frozen Tintype features plus cheap descriptors (estimated JPEG QF, NR-IQA, a neural-codec flag). The label is "Tintype is correct on this degraded sample", taken from augmented held-out data.
  - Use it to (a) output "inconclusive" rather than "AI" on heavily degraded inputs, protecting precision, and (b) route degraded inputs to a degradation-specialized second model.
  - Expected effect: shipped-threshold false positives on NTIRE-distorted reals should drop sharply if inconclusive verdicts are not counted as flags. Coverage drops accordingly.
  - Cost: small head, no backbone retraining.
  - Risk: users may see more "inconclusive"; this needs a UI and reporting policy.

**[P16] No Pixel Left Behind: A Detail-Preserving Architecture for Robust High-Resolution AI-Generated Image Detection (HiDA-Net).** Mu, Hu et al. (Zhejiang U.). ICLR 2026; arXiv:2508.17346. — [ICLR page](https://iclr.cc/virtual/2026/poster/10011114) · [mlanthology](https://mlanthology.org/iclr/2026/mu2026iclr-pixel/)
- Components: a Feature Aggregation Module fuses full-resolution tiles with a downsampled global view; token-wise localization; and a **JPEG Quality-Factor Estimation** head that regresses QF (MSE loss) from detail features, so the model separates compression from generation traces. Reports +13% accuracy on Chameleon and +10% on its HiRes-50K benchmark (up to 64 MP). Limitation: tile-wise inference time on large images. — [ICLR page](https://iclr.cc/virtual/2026/poster/10011114); [search summary of OpenReview PDF](https://openreview.net/pdf/9d2786143c2b2116fdc69d2de9ce0f77bce1ed60.pdf)
- **Tintype applicability (P16):**
  - Tintype's fit view plus native tiles already resembles HiDA-Net's global-plus-tiles design. The cheap transferable piece is an auxiliary head regressing the degradation parameters applied in augmentation: JPEG QF, resize factor, noise σ, blur σ and a "neural codec" flag. This is multi-task training with no inference cost.
  - The same head output can feed P14 or P15 calibration, killing two birds with one head.
  - Risk: low. The auxiliary loss weight needs tuning.

**[P17] Dissect and Prune: Enhancing Robustness in AI-Generated Image Detection (DEAR).** Jaehyun Choi, Hyun Seok Seong, Seongho Kim, Donghun Lee, Sungwon Yi, Jang-Ho Choi. arXiv:2606.10309 (Jun 2026). — [HF papers](https://huggingface.co/papers/2606.10309.md)
- Method:
  - Inpaint random regions of real images (SD1.5).
  - Score each penultimate channel of a frozen ResNet-50 detector by its activation inside the mask minus outside it.
  - Prune the channels at both extremes as "fragile".
  - Re-initialize and fine-tune only the linear head.
- Reported context: NPR's fake accuracy falls from 95.9% to 12.2% under post-processing (compression, resize, colour jitter) while its real accuracy rises from 67.2% to 94.2%. DEAR's own numbers were not retrieved; the paper claims substantially improved robustness on Chameleon, WildRF and LOKI. — [P17](https://huggingface.co/papers/2606.10309.md)
- **Tintype applicability (P17):**
  - Change: dissect and prune on Tintype's ViT CLS-feature dimensions or the pre-head MLP units. Use (clean vs degraded) pairs of the same image and drop dimensions whose activation swings most under benign degradation. Then refit the head.
  - Cost: cheap, head-only.
  - Risk: unproven on ViTs; may lose clean recall.

**[P18] Is Artificial Intelligence Generated Image Detection a Solved Problem? (AIGIBench).** Ziqiang Li, Jiazhen Yan, Ziwen He, Kai Zeng, Weiwei Jiang, Lizhi Xiong, Zhangjie Fu. NeurIPS 2025 Datasets & Benchmarks; arXiv:2505.12335. — [NeurIPS proceedings](https://proceedings.neurips.cc/paper_files/paper/2025/hash/fb693c67f61e5321746ffce8b6fdd2d0-Abstract-Datasets_and_Benchmarks_Track.html)
- Robustness tests (JPEG QF 50, Gaussian noise, down-up sampling) on 11 detectors. Most detectors drop sharply under JPEG and noise, and common augmentations give limited benefit. **Test-time preprocessing had mixed effects, mainly helping real-image accuracy rather than fake-image accuracy.** — [search summary of P18](https://neurips.cc/virtual/2025/poster/121688); [alphaXiv overview](https://alphaxiv.org/overview/2505.12335v2)
- **Tintype applicability (P18):**
  - The test-time preprocessing finding is relevant to precision. A deterministic normalization at inference (e.g. re-encode every input to one fixed JPEG quality, or downscale-then-crop) can push reals toward the real side, while costing some recall.
  - Cheap to A/B on NTIRE hard validation as a `--robust` profile: normalize, then score, with a threshold recalibrated on normalized reals.
  - Risk: recall loss on clean inputs, so offer it as a profile or apply it only when the degradation estimator fires.

### Inferences
- For Tintype's specific failure (reals flagged after strong or neural-codec degradation), the most promising combination is: (1) cover those degradations in training for both classes; (2) consistency loss; (3) a degradation estimator feeding quality-conditioned calibration or abstention. Items (1)–(2) improve ranking. Item (3) fixes the threshold problem that ranking fixes alone do not (Tintype's hard-validation-calibrated cutoff still gave 13/600 false positives at only 4.8% recall on the public distorted test).
- "Restoration in the loop" is unsupported by evidence found here. P8 and P9 imply that any learned decoder (and most SR and denoisers are learned) leaves generator-like traces. Denoise-then-detect plausibly increases false positives unless the restorer is applied to both classes in training too.

### Gaps
- No paper found that reports false-positive rate on degraded **real** images as the headline metric. All use AUC, AP or balanced accuracy.
- I found no peer-reviewed "denoise/restore-then-detect" study for fully generated image detection.
- DACOM's authors and full numbers, and DEAR's and GlobalForge's per-degradation tables, were not retrieved.
- SPECTRA-Net (arXiv:2605.08226; multi-view semantic, frequency, statistical and patch fusion, claimed resilient to JPEG and blur on WildFake) was seen only as a search snippet. — [arXiv](https://arxiv.org/html/2605.08226)

---

## Q4. Adversarial robustness and evasion: what attacks work, and what is cheap to adopt?

### Takeaway
White-box attacks trivially defeat every detector studied: Carlini & Farid drove AUC to 0.0005 by flipping one bit per pixel. Transferable black-box attacks still work, and many survive social-media recompression (Mavali et al., DIMVA 2026; RAID). Theory says robustness and reliability trade off more sharply as generators improve (Saberi et al., ICLR 2024). Cheap, useful steps for Tintype:
- Stay-Positive's non-negative last layer: degradations or "real-looking" perturbations can no longer add real evidence, only remove fake evidence.
- Evaluate on RAID's transferable adversarial set.
- Treat watermark presence or removal as a distortion to be invariant to (NTIRE included WMForger and 6–7 watermarkers).

Full adversarial training is expensive and the literature shows it does not reach clean performance.

### Cited Findings

**[P19] Evading Deepfake-Image Detectors with White- and Black-Box Attacks.** Nicholas Carlini, Hany Farid. CVPR 2020 Workshops; arXiv:2004.00622; DOI 10.1109/CVPRW50498.2020.00337. — [CVF PDF](https://openaccess.thecvf.com/content_CVPRW_2020/papers/w39/Carlini_Evading_Deepfake-Image_Detectors_With_White-_and_Black-Box_Attacks_CVPRW_2020_paper.pdf)
- Target detector at 0.95 AUC. White-box: flipping the lowest bit of each pixel gives AUC 0.0005; perturbing 1% of image area gives 0.08; one latent-space noise pattern gives 0.17. A black-box transfer attack gives 0.22. — [P19](https://openaccess.thecvf.com/content_CVPRW_2020/papers/w39/Carlini_Evading_Deepfake-Image_Detectors_With_White-_and_Black-Box_Attacks_CVPRW_2020_paper.pdf)
- **Tintype applicability (P19):**
  - Tintype ships open weights and a local binary, so it is fully white-box to attackers. No affordable defence stops a determined white-box attacker. The realistic goal is robustness to cheap, accidental and transfer perturbations.
  - Cheap hardening: randomized view selection at test time (random textured-crop jitter, random resize kernel) raises the cost of naive attacks.
  - Risk: a nondeterministic verdict, which conflicts with reproducibility. Use a seeded randomization keyed on the image hash.

**[P20] Robustness of AI-Image Detectors: Fundamental Limits and Practical Attacks.** Mehrdad Saberi, Vinu Sankar Sadasivan, Keivan Rezaei, Aounon Kumar, Atoosa Chegini, Wenxiao Wang, Soheil Feizi. ICLR 2024; arXiv:2310.00076. — [ICLR proceedings](https://proceedings.iclr.cc/paper_files/paper/2024/hash/744e09d3213ee53de88f04164a9b06ab-Abstract-Conference.html)
- Diffusion purification removes low-perturbation watermarks with minimal visible change. A spoofing attack makes real images read as watermarked. For classifier detectors, a robustness–reliability trade-off grows as the real and fake distributions converge. — [P20](https://proceedings.iclr.cc/paper_files/paper/2024/hash/744e09d3213ee53de88f04164a9b06ab-Abstract-Conference.html)
- **Tintype applicability (P20):**
  - Regeneration and purification (img2img at low strength, VAE round trip) is the cheapest laundering attack, and P9 shows it also turns reals into "fakes". Tintype should decide and document how it labels such images.
  - Add low-strength diffusion purification of **fakes** as a training augmentation, keeping the "generated" label. Tintype then learns that purified fakes stay fake, and since the purifier's own traces are generator traces, this is consistent.
  - Risk: none for precision, since purified reals are not used as real negatives.

**[P21] Adversarial Robustness of AI-Generated Image Detectors in the Real World** (v1 title: "Fake It Until You Break It"). Sina Mavali, Jonas Ricker, David Pape, Asja Fischer, Lea Schönherr (+ Yash Sharma in v1). DIMVA 2026; arXiv:2410.01574. — [arXiv](https://arxiv.org/abs/2410.01574)
- Four detectors and five attacks: performance falls greatly without architecture knowledge, and **most attacks remain effective after social-media-style degradation**. The commercial HIVE detector is vulnerable to black-box attacks. Using features from a robust pre-trained model increases robustness but does not reach benign performance (v1 described unsupervised adversarial fine-tuning of CLIP). Code: github.com/smavali/AIGI-Break. — [P21](https://arxiv.org/abs/2410.01574); [search summary](https://publications.cispa.de/articles/preprint/Fake_It_Until_You_Break_It_On_the_Adversarial_Robustness_of_AI-generated_Image_Detectors/27372414/1)
- **Tintype applicability (P21):**
  - Recompression is not a defence, so do not count on the messenger pipeline to wash out attacks.
  - A robust-backbone initialization (an adversarially fine-tuned ViT-S/DINO) could be tried as Tintype's starting checkpoint.
  - Cost: a retrain.
  - Risk: robust backbones usually lose clean accuracy.

**[P22] RAID: A Dataset for Testing the Adversarial Robustness of AI-Generated Image Detectors.** Italian and German authors (aimagelab / pralab). arXiv:2506.03988 (2025; submitted to NeurIPS 2025 D&B). — [arXiv](https://arxiv.org/abs/2506.03988)
- 72,000 adversarial examples (3 values of ε × 24K images from 4 text-to-image models), made by attacking an ensemble of 7 detectors and stored as PNG. They transfer to unseen detectors, and current detectors are "easily deceived". Data: HF aimagelab/RAID; code: GitHub pralab/RAID. — [P22](https://arxiv.org/abs/2506.03988)
- **Tintype applicability (P22):**
  - A free, off-the-shelf transfer-attack test. Run it in CI as a robustness regression.
  - Optionally fine-tune with a slice of RAID-style ensemble-transfer perturbations on fakes (labelled fake). This is cheap adversarial augmentation that does not touch the real class, so precision risk is low.

**[P23] Stay-Positive: A Case for Ignoring Real Image Features in Fake Image Detection.** Anirudh Sundara Rajan, Yong Jae Lee (UW–Madison). ICML 2025 (PMLR); arXiv:2502.07778. — [arXiv HTML](https://arxiv.org/html/2502.07778v1) · [project](https://anisundar18.github.io/Stay-Positive)
- Principle: an image is fake **iff** it contains generator artifacts. Real-image features are "at best, unnecessary and, at worst, misleading".
- Method: freeze the backbone (whose features are non-negative after ReLU), reset the last linear layer, and retrain it with BCE while clipping weights to w ≥ 0 after every step. The "real score" is then zero by construction. Constraining the whole network hurt real accuracy, so only the last layer is constrained. — [P23](https://arxiv.org/html/2502.07778v1)
- Results (AP, on Corvi's and Rajan's detectors):

  | Test set | Corvi | Corvi + Stay-Positive | Rajan | Rajan + Stay-Positive |
  |---|---:|---:|---:|---:|
  | FLUX, clean | 57.25 | 99.33 | 80.64 | 90.50 |
  | FLUX, post-processed | 74.57 | 94.23 | 87.80 | 91.85 |
  | 50%-inpainted reals | 6.13 | 97.48 | — | — |

  The authors describe it as "much more robust" to WebP and downsizing, shown only in figures. Small drops on some sets (Rajan on MJ −1.3 AP; GAN-baseline on CRN/IMLE 99.9 → 86.8 accuracy). — [P23](https://arxiv.org/html/2502.07778v1)
- Limitations stated by the authors: **upsampled real images are more likely to be called fake**; spurious *fake* features can still be learned; no false-positive rate is reported. — [P23](https://arxiv.org/html/2502.07778v1)
- **Tintype applicability (P23):**
  - Tintype's head acts on a ViT CLS token, not post-ReLU features, so features can be negative. The constraint must go on a non-negative representation. Practical route: insert a ReLU (or softplus) MLP layer before the final linear layer, train normally, then retrain only the final layer with w ≥ 0. The six-view logit average then becomes a sum of fake-evidence only.
  - Effect on the weak spots: it should help recall on messenger copies and laundered fakes, because blur and recompression can no longer add "real" evidence; they can only remove fake evidence.
  - It does **not** address the NTIRE false positives. Those come from degradations that *add* fake-like evidence (neural codecs, upsampling), and the authors themselves flag upsampled reals. Stay-Positive could make those false positives worse.
  - Cost: hours, head-only.
  - Required check: run on NTIRE distorted reals and the WhatsApp/Instagram reals before shipping. Pair with P8/P14 fixes.

**[P24] UnMarker: A Universal Attack on Defensive Image Watermarking.** Andre Kassis, Urs Hengartner (U. Waterloo). IEEE S&P 2025; arXiv:2405.08363. — [arXiv](https://arxiv.org/html/2405.08363v2) · [code](https://github.com/andrekassis/ai-watermark)
- A spectral-amplitude adversarial attack that removes invisible watermarks without detector access. Reported attack success against SynthID about 79% (secondary reporting). Diffusion purification (DiffPure-style regeneration) is used as a baseline. — [The Register](https://www.theregister.com/2025/07/24/ai_watermarks_unmarker/); [arXiv](https://arxiv.org/html/2405.08363v2)
- **Tintype applicability (P24):**
  - Tintype's provenance path (C2PA, watermarks) should not be treated as robust evidence; passive detection still has to carry the load.
  - Watermark *insertion* (6–7 algorithms) and *attack* (WMForger) were NTIRE distortions and are over-represented in Tintype's false positives. Add open watermark embedders (e.g. invisible-watermark DWT-DCT, RivaGAN, StegaStamp-style) and a spectral watermark-removal step to augmentation in **both** classes.

**[P25] The Deployment Gap in AI Media Detection: Platform-Aware and Visually Constrained Adversarial Evaluation.** Aishwarya Budhkar, Trishita Dhara, Siddhesh Sheth. CVPR 2026 AIMS workshop; arXiv:2604.09706. — [arXiv](https://arxiv.org/abs/2604.09706)
- Clean AUC is about 0.99. Platform-aware attacks confined to meme-style bands, plus resize, compression and screenshot transforms, reduce AUC "significantly". Universal perturbations exist even under band constraints, and calibration collapses: detectors become "confidently incorrect". Exact numbers were not retrieved. — [P25](https://arxiv.org/abs/2604.09706)
- **Tintype applicability (P25):** the evaluation framework is a candidate CI test. "Confidently incorrect" under attack supports the abstention and confidence head (P15) rather than raw sigmoid thresholds.

### Inferences
- With open weights, Tintype cannot be robust to white-box adversaries. Value comes from (a) robustness to unintentional and transfer perturbations, (b) never letting cheap processing add "real" evidence (Stay-Positive), and (c) treating purified or regenerated fakes as fakes in training.
- Precision-safe adversarial augmentation means perturbing **only fakes** (labelled fake). Perturbing reals toward "fake" would teach the model that perturbation noise means real.

### Gaps
- Per-attack numbers for Mavali et al. and RAID (AUC under each ε after JPEG) were not retrieved.
- I found no certified-robustness work for AIGI detectors that is practical at ViT-S scale.
- No paper found measures diffusion purification of fakes as an evasion attack against passive classifiers with numbers. Most purification work targets watermarks.

---

## Q5. Resolution, cropping, screenshots, re-photography, format conversion: which hurt most?

### Takeaway
- Re-digitization (photographing a screen or printout, scanning) is the most damaging condition in the largest controlled study: 16 of 17 detectors lost fake accuracy, some by about 90 points.
- Multi-hop social transmission is next, catastrophic for some detectors (SAFE −97 points).
- Downscaling destroys pixel-level traces. Native-resolution texture crops recover them (TextureCrop: +6.1% AUC over centre crop, +15% over resize), which supports Tintype's design.
- Neural-codec and upsampling operations create false positives rather than misses.
- WebP is partly covered by strong JPEG augmentation. No peer-reviewed data was found for AVIF or HEIC.

### Cited Findings

**[P26] Bridging the Gap Between Ideal and Real-world Evaluation: Benchmarking AI-Generated Image Detection in Challenging Scenarios (RRDataset).** Chunxiao Li, Xiaoxiao Wang, Meiling Li, Boming Miao, Peng Sun, Yunjian Zhang, Xiangyang Ji, Yao Zhu. ICCV 2025; arXiv:2509.09172. — [CVF PDF](https://openaccess.thecvf.com/content/ICCV2025/papers/Li_Bridging_the_Gap_Between_Ideal_and_Real-world_Evaluation_Benchmarking_AI-Generated_ICCV_2025_paper.pdf) · [alphaXiv](https://www.alphaxiv.org/abs/2509.09172.md)
- Two conditions:
  - Transmission: 10K real + 10K AI images shared 2–6 times through 8 platforms (Telegram, WeChat, Facebook, QQ, WhatsApp, X, Instagram, Tinder).
  - Re-digitization: 4 methods (scanning a printout, photographing a printout, photographing a screen, photographing a projection), each on 10K + 10K images.

  [P26](https://www.alphaxiv.org/abs/2509.09172.md)
- 17 detectors tested; best overall DRCT-ConvB at 89.59%.
  - Transmission: fake accuracy fell for 14 detectors (SAFE −97.41, Fusing −79.86, FreqNet −71.61). Diffusion-reconstruction methods (DIRE, DNF, DRCT) held up better.
  - Re-digitization: fake accuracy fell for 16 of 17 (DNF −90.57, DIRE −88.30); AIDE was most robust (−1.89).
  - GPT-4o: 84.09% overall, with fake accuracy −16.89 under transmission and −27.07 under re-digitization.

  [P26](https://www.alphaxiv.org/abs/2509.09172.md)
- Humans: 69.17% (everyday) and 59.52% (special scenarios). When unsure they default to "AI" (70–89%). — [P26](https://www.alphaxiv.org/abs/2509.09172.md)
- **Tintype applicability (P26):**
  - RRDataset is the best public set for Tintype's WhatsApp/Instagram weakness. It includes WhatsApp and Instagram, multi-hop sharing and recaptures. Use part for training, as INTSIG did in NTIRE, and hold out part.
  - Add synthetic re-capture augmentation (perspective warp, moiré, display-gamma or colour shift, defocus, sensor noise, then JPEG) to both classes.
  - Expected effect: better messenger recall.
  - Risk: re-captured fakes become semantically closer to real photos, which may lower recall at the strict threshold. Measure per-profile.

**[P27] TextureCrop: Enhancing Synthetic Image Detection through Texture-based Cropping.** Despina Konstantinidou, Christos Koutlis, Symeon Papadopoulos (CERTH-ITI). arXiv:2407.15500 (v4). — [arXiv](https://arxiv.org/html/2407.15500v4)
- A 224×224 sliding window with stride 224 keeps only high-texture patches and averages the logits. AUC improves by 6.1% over centre crop and 15% over resize on high-resolution images, across detectors. Plug-in; code at mever-team/texture-crop. — [P27](https://arxiv.org/html/2407.15500v4)
- **Tintype applicability (P27):**
  - This is independent support for Tintype's "four most textured native crops" design. Tintype's NTIRE all-views ablation (distorted recall 5.4→7.5%) suggests more crops help a little.
  - Under strong noise, the most textured crops may be the *noisiest* (noise inflates texture measures), so texture-based selection can pick degraded areas or fake-looking noise. Under heavy noise, select crops by a structure measure robust to noise (e.g. gradient energy after light denoising, used only for selection) rather than raw variance.
  - Cheap to test on NTIRE noise-distorted subsets.

**[P28] Improving Synthetic Image Detection Towards Generalization: An Image Transformation Perspective (SAFE).** Ouxiang Li et al. KDD 2025; arXiv:2408.06741. — [arXiv](https://arxiv.org/html/2408.06741v2)
- Argues that downsampling destroys the local pixel correlations created by generators' upsampling. Training uses random crop and inference centre crop instead of resize, plus ColorJitter, RandomRotation and patch-based random masking. +4.5% accuracy and +2.9% AP over prior methods on 26 generators. — [P28](https://arxiv.org/html/2408.06741v2)
- Contrast: SAFE lost 97.41 points of fake accuracy under RRDataset transmission. The traces it preserves are exactly the ones recompression destroys. — [P26](https://www.alphaxiv.org/abs/2509.09172.md)
- **Tintype applicability (P28):**
  - Caution: crop-only, local-correlation detectors are the most fragile to sharing. This supports Tintype keeping its resized fit view and training on degraded data, not shifting further toward pixel-level cues.
  - Colour jitter, rotation and patch masking are cheap augmentations Tintype lacks. NTIRE included colour and tone distortions.

**[P29] BIAS-ID: A Framework for Analyzing Transformation Biases in AI-Generated Image Detectors.** Jonas Ricker, Asja Fischer, Erwin Quiring (RUB). NeurIPS 2026 (listed as a poster); arXiv:2605.31153. — [alphaXiv](https://www.alphaxiv.org/abs/2605.31153)
- A framework that quantifies how much processing transformations shift detector scores. Several state-of-the-art detectors are "strongly affected" by processing biases (e.g. resolution mismatches between real and fake training data biasing toward up- or downscaling artifacts). Code: github.com/jonasricker/biasid. Specific shift magnitudes were not retrieved. — [P29](https://www.alphaxiv.org/abs/2605.31153)
- **Tintype applicability (P29):**
  - Run BIAS-ID on Tintype to measure score shifts for upscaling, downscaling, JPEG, WebP and crops on **real** images. This is the direct diagnostic for which transformations create false positives.
  - Cheap: the library exists. It complements the repo's existing finding that resampling-implementation differences flip 2% of baseline decisions.

**[P30] Is JPEG AI going to change image forensics? (cross-ref P8).** Neural codecs and upsampling produce false positives on reals: false-positive rate 0.96–0.99 at 0.12 bpp for several detectors. — [P8](https://arxiv.org/html/2412.03261v2)
- Stay-Positive likewise notes that upsampled reals are more often called fake. — [P23](https://arxiv.org/html/2502.07778v1)

### Inferences
Inferred ranking of operations by harm to Tintype-like detectors:
1. **For precision (false positives):** neural codecs (JPEG AI, Cheng2020), upsampling or super-resolution, and VAE laundering. All add generator-like upsampling traces (P8, P9, P23).
2. **For recall:** re-digitization (screen or print re-photography), multi-hop messenger chains, strong downscaling with JPEG (Tintype's own Frontier test: 69.3 → 28.0 at 960 px), then single JPEG, then WebP.
3. **Crops:** mostly harmless for Tintype (native crops), except when they leave too little texture.

Tintype's augmentation covers resize and JPEG well but none of 1) and little of 2).

### Gaps
- No peer-reviewed measurements of AVIF or HEIC conversion effects on AIGI detectors were found. This needs an in-house test (libavif, libheif round trips on both classes).
- No controlled study of screenshot (screen-capture at device scaling, with UI overlay) as distinct from camera re-photography was found, beyond P25's qualitative inclusion.
- Super-resolution-specific false-positive rates on real photos (Real-ESRGAN, phone "AI zoom") lack peer-reviewed numbers; only vendor or blog claims were found ([PhotoWorkout summary of a NewsGuard audit, unverified](https://www.photoworkout.com/?p=222434)).

---

## Q6. Ranked shortlist: the most promising ideas for Tintype from this literature

### Takeaway
The cheapest, highest-expected-value path is:
1. NTIRE-style chained degradation augmentation that includes neural codecs, watermarks, re-capture and multi-hop messenger chains, applied symmetrically to both classes.
2. Paired consistency loss.
3. A degradation/quality head driving quality-conditioned calibration and an "inconclusive" verdict.

Then content and format debiasing (B-Free-style aligned reconstructions, Fake-or-JPEG matching), and Stay-Positive head retraining for messenger recall, gated on a false-positive check.

### Cited Findings
1. **Chained, graded, symmetric degradation augmentation covering NTIRE's 36 operations, plus neural codecs, watermarks, re-capture and multi-hop sharing.**
   - Evidence:
     - Every NTIRE top team did this. vincentlc reached 0.873 average robust AUROC with a single SigLIP2 model, with official distortions applied to 100% of samples, up to 3 operations at 5 levels — [P1](https://arxiv.org/html/2604.11487v1).
     - Neural codecs are the documented cause of false positives on reals — [P8](https://arxiv.org/html/2412.03261v2).
     - Platform caps are 720–1200 px with QF 61–92 — [P4](https://arxiv.org/html/2504.20658), with 2–6 share cycles and re-capture — [P26](https://www.alphaxiv.org/abs/2509.09172.md).
   - Concrete change:
     - Extend Tintype's augmentation sampler with families from P1's list.
     - Neural codecs via CompressAI (Cheng2020, hyperprior) and the JPEG AI reference implementation at 0.06–0.5 bpp.
     - Open invisible watermarkers, plus WebP and AVIF.
     - Messenger caps at 640–2048 px on width, height or long side; 1–4 chained cycles.
     - Re-capture simulation: perspective, moiré, gamma, blur and noise.
     - A difficulty curriculum in clean/mild/moderate/heavy levels (MICV, Ant).
   - Add RRDataset and TrueFake shared images to training and validation.
   - Expected: large AUROC gains on NTIRE-like distortions and better messenger recall. The codec false-positive fix will be partial — [P8](https://arxiv.org/html/2412.03261v2).
   - Cost: training compute only; no inference cost.
   - Risk: overfitting to NTIRE's generator. Hold out the NTIRE public test, the WhatsApp/Instagram set and Arena-JPEG.
2. **Paired clean/degraded consistency training (KL + feature loss).**
   - Evidence: +1.35 public-test AUC at 224 px — [P2](https://arxiv.org/html/2604.12307v1); +9.1 degraded accuracy for −0.9 clean — [P12](https://arxiv.org/html/2604.10102v1).
   - Expected: mainly stabilizes the score, and therefore the threshold, across processing histories.
   - Cost: about 2× training forwards; zero at inference.
3. **Degradation estimator plus quality-conditioned calibration and abstention.**
   - Evidence: the logit distributions of both classes spread with degradation, and a quality-conditioned LLR gives +8 bAcc — [P14](https://arxiv.org/html/2604.15027v1). Per-sample distortion-aware confidence enables abstention and routing — [P15](https://iclr.cc/virtual/2026/poster/10010162). A QF-regression auxiliary head helps detection — [P16](https://iclr.cc/virtual/2026/poster/10011114).
   - Concrete change: an auxiliary head regressing the augmentation parameters (QF, scale, noise, blur, neural-codec flag). At inference, fit per-quality-bin real and fake logit Gaussians and output the LLR. Return "inconclusive" when the predicted confidence is low. Optionally route degraded inputs to the second opinion.
   - Expected: the most direct cut to the 155/600 shipped-cutoff false positives on NTIRE-distorted reals. It also shows what drives the `--sensitive` vs default trade-off on messenger copies.
   - Cost: small. Needs a degraded calibration set.
4. **Content- and format-aligned training data.**
   - Evidence: B-Free-style self-conditioned reconstructions of Tintype's own reals, plus Fake-or-JPEG matching of QF and size distributions across classes. B-Free reached 86–97% bAcc on WildRF and ≈92% on 100-day viral re-posts — [P5](https://arxiv.org/html/2412.17671v2). Removing JPEG and size bias gave +11 points cross-generator and better JPEG robustness — [P7](https://arxiv.org/pdf/2403.17608).
   - Expected: a smaller recall drop when both classes are re-encoded (Arena 88.5→73.1; Frontier 69.3→28.0).
   - Cost: a one-off generation job.
5. **Stay-Positive non-negative final layer (head-only retrain).**
   - Evidence: FLUX AP 57→99 clean and 75→94 post-processed on Corvi's detector, and more robust to WebP and downsizing. Upsampled reals are flagged more — [P23](https://arxiv.org/html/2502.07778v1).
   - Expected: better messenger and laundered-fake recall. It could worsen neural-codec and upsampling false positives, so ship only together with items 1 and 3, and only after an NTIRE-distorted-real false-positive check.
   - Cost: hours.

### Inferences
- Items 1–3 attack different failure modes. Item 1 adds coverage, item 2 stabilizes the score, and item 3 calibrates and abstains. They compose without any inference-time cost except item 3's tiny head. Together they are the most plausible route from 0.636 toward the 0.83+ AUROC range of NTIRE entrants without leaving the ViT-S/224 budget. That is an inference; the size of the gain is unmeasured.
- Larger backbones (DINOv3, SigLIP2-giant) and TTA ensembles were the winners' other lever. For Tintype, they fit only as a gated second opinion triggered by the degradation estimator (item 3), keeping the default path fast.
- Precision guard-rails for all items: report per-operation false-positive rate on reals (BIAS-ID-style, [P29](https://www.alphaxiv.org/abs/2605.31153)) for JPEG AI, Cheng2020, watermark insertion and attack, upscaling, WebP, AVIF and re-capture. Never add an augmentation to the real class only.

### Gaps
- None of the shortlisted methods has published false-positive-at-fixed-threshold results on degraded reals. The expected precision effects are inferences and must be measured on NTIRE hard validation (calibration) and the public test (evaluation).
- The NTIRE official distortion code was used by entrants ("official distortion function", [P1](https://arxiv.org/html/2604.11487v1)), but I did not verify its licence or public availability.
