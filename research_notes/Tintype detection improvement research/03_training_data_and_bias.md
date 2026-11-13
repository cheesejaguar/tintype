# Training data, augmentation, dataset bias and continual learning for AI-generated image detectors (with Tintype applicability)

Scope: literature from 2019 to 2026, mostly 2023–2026, on how the training set and augmentation recipe decide whether a detector generalises. Each paper gets a "Tintype applicability" note. Tintype facts come from the repo: `README.md` ("How it works", "Limits"), `docs/DATA.md` and `docs/RESEARCH.md`. Numbers marked "brief" come from the task description (NTIRE 2026 distorted test: 2.2% recall at 1% FPR, and 155 of 600 distorted *real* images flagged; messenger copies: 27–36% recall).

Relevant facts about Tintype's current recipe:
- ViT-S/16 at 224 px, initialised from Community Forensics. Training is six-view inference with texture-ranked native crops, plus random JPEG before/after resize, a random kernel, random crop scale, and a 25% messenger round trip ([README](../../README.md); [DATA.md](../../docs/DATA.md)).
- Phase 1 includes `openfake-reddit`: **3.0 k real vs 15.8 k AI images "posted to Reddit"**. Phase 3 adds So-Fake-Set: 3.0 k social-media reals vs 4.3 k fakes ([DATA.md](../../docs/DATA.md)).
- Deduplication is byte-identical only: "916 training files that were byte-identical to an evaluation file were removed" ([DATA.md](../../docs/DATA.md)).
- Sources are drawn with weight count^0.5. Both classes are drawn equally often ([DATA.md](../../docs/DATA.md)).

---

## Q1. Dataset alignment and shortcut/bias removal (Rajan et al., B-Free, Fake-or-JPEG, DRCT, FakeInversion, semantic bias)

### Takeaway
Making real and fake training images match in everything except the generator trace (content, size, format, JPEG history) is the single best-evidenced data intervention. It gives large gains under post-processing:
- Rajan et al.: Playground recall after post-processing went from 24% to 76%.
- B-Free: +20.7 pts bAcc over the second-best method; WildRF social bAcc of 86–97.
- Grommelt et al.: more than 11 pts cross-generator gain from removing JPEG/size bias.

Tintype's mix contains a clear shortcut of this kind: far more AI images than real images carry social-platform processing. That plausibly explains both its false positives on degraded reals and its weak recall on messenger copies.

### Cited Findings

**1. Rajan, Ojha, Schloesser, Lee, "Aligned Datasets Improve Detection of Latent Diffusion-Generated Images"** (arXiv title v1: "On the effectiveness of dataset alignment for fake image detection"). arXiv:2410.11835, v3 Feb 2025; the brief lists it as ICLR 2025, and the arXiv page itself names no venue. — [arXiv](https://arxiv.org/abs/2410.11835)
- Method: each real image is passed through the LDM autoencoder with no denoising, and the reconstruction serves as the "fake". Size, aspect ratio and content therefore match, and only decoder artifacts differ. — [arXiv HTML](https://arxiv.org/html/2410.11835v3)
- Clean accuracy vs the Corvi baseline: Playground 82.13 → 94.85 (Ours) / 99.48 (Ours-Sync); Midjourney 96.90 → 98.50 / 99.37. Real accuracy stays at ≈99.8–99.9. — [arXiv HTML](https://arxiv.org/html/2410.11835v3)
- After random JPEG, resize, blur and colour jitter (Corvi → Ours-Sync):

  | Generator | Corvi | Ours-Sync |
  |---|---|---|
  | SD | 77.78 | 88.06 |
  | Midjourney | 50.45 | 72.36 |
  | Playground | 24.27 | 76.36 |
  | PixArt-α | 64.41 | 86.10 |
  | LCM | 58.31 | 91.08 |

  — [arXiv HTML](https://arxiv.org/html/2410.11835v3)
- Data efficiency (TPR at 5% FPR): with 1 k images per class, 83.4 for Ours vs 46.5 for Corvi; with 10 k, 98.1 vs 87.6. Generating the training data costs more than 10× less than Corvi's diffusion-generated set, since it needs one AE forward pass and no UNet. — [arXiv HTML](https://arxiv.org/html/2410.11835v3)
- Failure modes:
  - FLUX.1-dev, whose VAE has a 16-channel latent, was detected only 9.6% (Ours) / 25.9% (Ours-Sync) of the time.
  - WebP artifacts present in the real training images but absent from the reconstructions became a shortcut.
  - GANs were not evaluated.

  — [arXiv HTML](https://arxiv.org/html/2410.11835v3)

**2. Guillaro, Zingarini, Usman, Sud, Cozzolino, Verdoliva, "A Bias-Free Training Paradigm for More General AI-generated Image Detection" (B-Free)**, arXiv:2412.17671 (CVPR 2025 per the brief; the arXiv page lists no venue). — [arXiv](https://arxiv.org/abs/2412.17671)
- Data: 51,517 COCO reals and 309,102 fakes made by self-conditioned SD 2.1 inpainting with an empty mask. Content augmentation inpaints an object from the same or a different category; "inpainted++" adds scaling, cut-out, noise and jitter. Training uses no resizing (504 px crops) plus CNNDetect-style blur+JPEG augmentation. Social-network simulation resizes by 0.7–1 and applies JPEG at quality 70–100. — [arXiv HTML](https://arxiv.org/html/2412.17671v2)
- Ablation (average AUC / bAcc):

  | Training data | AUC / bAcc |
  |---|---|
  | Text-paired fakes | 93.5 / 61.9 |
  | Self-conditioned | 94.7 / 81.4 |
  | Self-conditioned + cutmix/mixup | 95.3 / **78.6** (mixup hurts) |
  | Self-conditioned + inpainted | 98.0 / 92.2 |
  | + inpainted++ | 99.3 / 96.4 |

  NLL falls from 1.91 to 0.10, so the scores are much better calibrated. — [arXiv HTML](https://arxiv.org/html/2412.17671v2)
- End-to-end fine-tuning beats a linear probe by a wide margin (DINOv2+reg: 95.2 vs 68.5 bAcc). Training other detectors on the B-Free data raises CLIP/ViT from 45–48 to 73.9 bAcc and RINE from 65–76 to 83.2. — [arXiv HTML](https://arxiv.org/html/2412.17671v2)
- Headline: 99.0 AUC / 96.3 bAcc averaged over Synthbuster, new generators and WildRF, which is +20.7 pts bAcc over the second-best method. WildRF bAcc: Facebook 95.6, Reddit 86.2, Twitter 97.3. Under JPEG, resize and blur, bAcc stays above 80%. — [arXiv HTML](https://arxiv.org/html/2412.17671v2)

**3. Grommelt, Weiss, Pfreundt, Keuper, "Fake or JPEG? Revealing Common Biases in Generated Image Detection Datasets"**, arXiv:2403.17608, 2024.
- In GenImage, the reals are JPEG and the fakes PNG, and their size distributions differ. Removing these biases gives more than 11 pts cross-generator gain for ResNet50 and Swin-T, and "substantially increases robustness to JPEG compression". — [arXiv](https://arxiv.org/abs/2403.17608)

**4. Chen, Zeng, Yang, Yang, "DRCT: Diffusion Reconstruction Contrastive Training towards Universal Detection of Diffusion Generated Images"**, ICML 2024 (Spotlight), PMLR v235.
- Hard fakes are made by diffusion-reconstructing reals and training contrastively against them. DRCT-2M has 16 diffusion models, 1.92 M fakes and 80 k COCO reals. — [PMLR](https://proceedings.mlr.press/v235/chen24ay.html); [code](https://github.com/beibuwandeluori/DRCT)
- I could not retrieve its per-generator numbers (see Gaps).

**5. Cazenavette, Sud, Leung, Usman, "FakeInversion: Learning to Detect Images from Unseen Text-to-Image Models by Inverting Stable Diffusion"**, CVPR 2024, arXiv:2406.08603.
- Features come from inverting SD. The model is trained only on SD fakes yet generalises to DALL·E 3.
- It proposes an evaluation protocol that uses **reverse image search to pick real images matching each fake's content and style**. Its scores "align well with in-the-wild performance". — [arXiv](https://arxiv.org/abs/2406.08603)

**6. Zheng et al., "Breaking Semantic Artifacts for Generalized AI-generated Image Detection"**, NeurIPS 2024.
- Patch shuffling plus a patch-level classifier reduces reliance on semantics. Across 31 test sets: +2.08% cross-scene accuracy, and +10.59% open-world accuracy in the proceedings abstract, though the poster page says 6.74%. — [NeurIPS proceedings](https://proceedings.neurips.cc/paper_files/paper/2024/hash/6dddcff5b115b40c998a08fbd1cea4d7-Abstract.html); [poster page](https://nips.cc/virtual/2024/poster/95403)

**7. Rajan & Lee, "Stay-Positive: A Case for Ignoring Real Image Features in Fake Image Detection"**, arXiv:2502.07778 (v2 May 2025; ICML 2025 per my recollection, not confirmed on the arXiv page).
- The detector is constrained to rely only on generator artifacts, ignoring features it associates with real images. The authors report less reliance on spurious correlations, better robustness to post-processing, and better detection of inpainted real images. The abstract gives no numbers. — [arXiv](https://arxiv.org/abs/2502.07778)

**8. Karageorgiou, Papadopoulos, Kompatsiaris, Gavves, "Any-Resolution AI-Generated Image Detection by Spectral Learning" (SPAI)**, CVPR 2025, arXiv:2411.19417.
- Masked spectral learning on real images only, so fakes are treated as out-of-distribution. It processes any resolution via spectral context attention, and reports +5.5% absolute AUC over the prior SOTA across 13 recent generators. — [arXiv](https://arxiv.org/abs/2411.19417)

**9. NTIRE 2026 challenge organisers (Gushchin et al.) bias-controlled data construction**, arXiv:2604.11487.
- Fakes are generated from VLM captions of the real images.
- The organisers also "align its distributions of resolutions, aspect ratios, JPEG compression quality factors, and other statistics to those of the real subset".
- Reals are CLIP-deduplicated and VLM-filtered from about 12 M CC12M / CommonPool / RedCaps images.
- In the robust track, **both real and generated images undergo the same degradation pipeline**. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)

### Inferences (Tintype applicability)

- **Tintype has a Fake-or-JPEG-style shortcut, and it probably drives the NTIRE false positives.**
  - `openfake-reddit` contributes 15.8 k Reddit-processed AI images against 3.0 k Reddit reals. Most other reals (COCO, ImageNet, LAION, FFHQ, CelebA-HQ) are web or dataset JPEGs, not platform recompressions.
  - The network can therefore learn "platform/degradation look ⇒ AI". This fits the brief: 155 of 600 distorted NTIRE reals were flagged, while recall on degraded fakes collapses. The model has probably learned an inconsistent mixture of degradation cues.
  - Fixes: (a) rebalance so that every processing condition (social-platform, recompressed, resized) has matched real and fake mass, by adding real Reddit, Bluesky or messenger photos or by down-weighting `openfake-reddit` fakes; (b) apply each augmentation with identical probability and parameter distribution to both classes. The second is probably already true for the synthetic augmentation, but not for the "native" processing baked into the source files.
  - Cost: low. Risk: a small loss of recall on Reddit-sourced fakes, which is the point.
- **Rajan-style AE reconstructions of Tintype's own reals** (cheap: one VAE encode/decode per image; a laptop GPU can do about 100 k).
  - Run SD1.x/SDXL (4-channel), SD3/FLUX (16-channel) and other open VAEs (e.g. Qwen-Image, Hunyuan) over the 16 real sources, at native resolution and after the same random JPEG and resize.
  - Each reconstruction is a perfectly content-, size- and format-matched fake. This directly targets recall under compression: the Playground post-processing recall went from 24% to 76%.
  - It needs the 16-channel VAEs, given the FLUX failure. WebP and other real-only formats must also be stripped or matched.
  - Expected: higher recall on messenger copies and on new latent-diffusion generators that reuse an existing VAE. Little help for GANs or autoregressive/pixel generators.
  - Risk: over-focus on the VAE decoder trace, so mix rather than replace.
- **B-Free self-conditioned inpainting plus content augmentation.** This is costlier, needing about 6 diffusion passes per real, but it also gives Tintype some coverage of *local edits*, which are currently out of scope, and better calibration.
  - The ablation suggests most of the gain comes from inpainted local regions (bAcc 81 → 96).
  - The 0.7–1 resize and JPEG 70–100 social simulation is milder than Tintype's messenger round trip. The B-Free result shows that training without resizing helps; Tintype's native-crop views already avoid resizing at inference.
- **Do not add mixup/cutmix.** B-Free measured mixup/cutmix lowering bAcc from 81.4 to 78.6.
- **Patch shuffling** (Zheng et al.) is a cheap augmentation for the native-crop views, e.g. shuffling 32 px or 56 px tiles within the 224 crop at p≈0.2.
  - It may help semantics-independent generalisation. It should not be applied to the fit view, which needs global layout for older GAN and face cues. Gain uncertain (2–10 pts open-world in the paper on weaker baselines).
- **Stay-Positive** matches Tintype's false-positive failure mode closely. When degradation erases camera or "real" cues, a detector whose evidence includes "real-ness" features drifts towards fake.
  - Constraining the last layer, or the final logit's dependence, to fake-evidence features is a cheap training change.
  - Inference: this should reduce false positives on degraded reals at some recall cost. It is unmeasured for ViT-S.
- **FakeInversion's reverse-image-search pairing** is useful mostly for building *evaluation* and hard-negative real sets matched to the content of each fake.

### Gaps
- The DRCT, FakeInversion and Stay-Positive per-benchmark numbers were not retrieved (the abstracts give none). DRCT's measured gain under JPEG is unknown to me.
- None of the alignment papers evaluates on a messenger/WhatsApp round trip comparable to Tintype's 27–36% recall case. Expected gains there are an inference.
- Alignment results are reported mainly for LDM-family generators. Effect on GANs, autoregressive models (GPT-image) and face swaps is unmeasured.

---

## Q2. Augmentation recipes that matter (blur/JPEG, social-network simulation, distortion pipelines, hard negatives)

### Takeaway
Augmentation has measured gains of +4.5 to +10 AUC on in-the-wild social data. In NTIRE 2026, aggressive, class-symmetric, multi-distortion augmentation with distortion probability up to 1.0 separated the top teams from teams with 0.99 clean AUC but 0.91–0.93 robust AUC.

Tintype's augmentation covers only JPEG, resize, kernel and crop. It lacks noise, blur, sharpening, WebP, colour casts, moiré, neural compression and watermarking, which is what the NTIRE test applies. False positives on degraded reals are best addressed with real hard negatives and quality-aware thresholds.

### Cited Findings

**10. Wang, Wang, Zhang, Owens, Efros, "CNN-generated images are surprisingly easy to spot... for now" (CNNSpot)**, CVPR 2020, arXiv:1912.11035.
- Trained on ProGAN only, with random Gaussian blur and JPEG augmentation, it generalised to other CNN generators. Augmentation improved both generalisation and robustness, and it became the standard baseline recipe. — [arXiv](https://arxiv.org/abs/1912.11035)
- Specific numbers were not re-verified here.

**11. Konstantinidou et al., "Navigating the Challenges of AI-Generated Image Detection in the Wild: What Truly Matters?"**, arXiv:2507.10236, 2025 (later in Pattern Recognition per search listing). Introduces ITW-SM: 10 k real and generated images from social platforms. — [arXiv](https://arxiv.org/abs/2507.10236v1)
- Augmentation (JPEG and WebP, crop, rotation, flip, Gaussian noise, blur, sharpening), ITW-SM AUC:

  | Detector | Without | With |
  |---|---|---|
  | DMID | 82.20 | 92.26 |
  | RINE | 93.70 | 98.23 |
  | NPR | 68.92 | 76.03 |
  | SPAI | 91.46 | 98.10 |

  RINE dropped on Chameleon (92.2 → 87.3) when the augmentations did not match that set's distortions. — [arXiv HTML](https://arxiv.org/html/2507.10236v1)
- Training data: LDM-only training was worst for every model. Real-world data (TWIGMA, with OpenImages reals) drove the gains; RINE went from 83.5 to 97.7 on ITW-SM. — [arXiv HTML](https://arxiv.org/html/2507.10236v1)
- Texture-ranked crops beat centre crops for DMID (+8.5) and RINE (+3.2) but hurt NPR (−4.0).
- Bigger pretraining (CLIP-H/14, OpenCLIP-L LAION-2B) did *not* beat DINOv2-L or OpenAI CLIP-L. Headline: +26.87% average AUC from the combined design choices. — [arXiv HTML](https://arxiv.org/html/2507.10236v1)

**12. NTIRE 2026 Challenge on Robust AI-Generated Image Detection in the Wild**, Gushchin et al., CVPR 2026 workshops, arXiv:2604.11487.
- Dataset: 108,750 reals and 185,750 fakes from 42 generators (2022–2026); the training split has 20 open generators. The robust track applies 1–5 chained distortions out of 36 types, ranging from blur and noise to invisible watermarking, neural compression and watermark-removal attacks. Distortion sets change between splits. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)
- Winning team (MICV): average robust ROC-AUC 0.9723 and best clean AUC 0.9974.
  - Recipe: an ensemble of six DINOv3 backbones fine-tuned end to end at 512 px, trained on multi-source data (GenImage, WildFake, AIGIBench, Community Forensics, So-Fake-Set, self-generated Qwen-Image / Z-Image / FLUX including I2I/editing/inpainting, and API images from Seedream, Kling, GPT-Image, Nano-banana-pro).
  - Augmentation is a "hierarchical, difficulty-aware" pipeline from single degradations to multi-stage chains, with focal loss and SWA. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)
- Ant International: about 1 M images. Four offline augmentation levels: clean; 1–3 distortions; 3–6 distortions; 6 fixed heavy. Experts are trained on different level mixes, at 512 and 288 px. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)
- TeleAI: LoRA on EVA-CLIP with **pairwise clean+distorted training**: CE + 0.5·KL(clean‖distorted) + 0.25·MSE on corrected features. They added speckle noise, colour cast and moiré after inspecting the validation set, and added So-Fake and Chameleon to training. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)
- Another team: "distortion prob=1.0, so every training image is distorted, with up to three random post-processing operations… at multiple severity levels… a key driver of our Robust ROC AUC improvements". They also used a "squish" resize of the whole image to 384 instead of random resized crop. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)
- Mid-ranked teams had clean AUC around 0.98–0.99 but robust AUC around 0.91–0.93, so robustness was the differentiator. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)

**13. Dell'Anna et al., "TrueFake: A Real World Case Dataset of Last Generation Fake Images also Shared on Social Networks"**, IJCNN 2025, arXiv:2504.20658.
- 600 k images: 100 k real and 500 k fakes from StyleGAN1–3, SD 1.5/2.1/3/SDXL and FLUX.1. 180 k of them were shared through three social networks.
- Detectors trained on unshared images (R50-ND, MISLNet, NPR, CLIP-D, P2G) are evaluated on shared copies. The paper reports per-platform JPEG quality, PSNR and SSIM. — [arXiv](https://arxiv.org/abs/2504.20658); [repo](https://github.com/MMLab-unitn/TrueFake-IJCNN25)

**14. Cavia, Horwitz, Reiss, Hoshen, "Real-Time Deepfake Detection in the Real-World" (LaDeDa, WildRF)**, arXiv:2406.09398, 2024.
- WildRF is curated from social networks. A 9×9-patch detector reaches 93.7% mAP on WildRF. The authors argue current training protocols "prevent methods from generalizing to real-world deepfakes extracted from social media". — [arXiv](https://arxiv.org/abs/2406.09398)

**15. Schinas & Papadopoulos, "SIDBench: A Python Framework for Reliably Assessing Synthetic Image Detection Methods"**, arXiv:2404.18552, 2024.
- A unified benchmark harness covering many detectors and transformations. It is useful for evaluating Tintype under standardised degradations. — [arXiv](https://arxiv.org/abs/2404.18552)
- Not re-fetched; no numbers cited.

**16. Guillaro, De Rosa, Cozzolino, Verdoliva, "Quality-Aware Calibration for AI-Generated Image Detection in the Wild" (QuAD)**, arXiv:2604.15027, 2026.
- Logit distributions for real and fake overlap more as quality drops. QuAD models the logit mean and variance as a function of no-reference IQA quality (LoDa), separately per class, and aggregates near-duplicates.
- AncesTree, average over 6 detectors: a single image gives 70.7% bAcc; naive aggregation 73.2%; QuAD 81.6% (NLL from 0.97 to 0.43). ReWIND: 67.3% → 70.3–71.4% bAcc. — [arXiv HTML](https://arxiv.org/html/2604.15027v1)

**17. von Stryk & Keuper, "LAION-Mobile: Evaluating Deepfake Detectors On One Million Smartphone Photos"**, arXiv:2609.11134, Sep 2026.
- About 1 M smartphone photos with EXIF, from re-LAION-5B. Twelve detectors were tested.
- With thresholds fitted on legacy GAN data, several detectors flag under 11% of real phone photos. When the criterion is refit on modern content, they flag **17–91%** of real phone photos.
- No detector beats AUC 0.624 on modern AI content while keeping a deployable false-alarm rate. False alarms are attributed to processing pipelines (neural ISP, multi-frame fusion, denoising). The corpus mostly covers 2018–2020 phones. — [arXiv](https://arxiv.org/abs/2609.11134)

**18. Doloriel & Cheung, "Frequency Masking for Universal Deepfake Detection"**, ICASSP 2024.
- Proposes frequency-domain masking as an augmentation. — I did not fetch or verify this one, so it is listed only as a lead (see Gaps).

### Inferences (Tintype applicability)

- **The NTIRE false positives (155/600 distorted reals) are likely a train/test augmentation mismatch combined with the class-asymmetric processing in Q1.**
  - NTIRE distorts both classes with noise, blur, colour, moiré, neural compression and watermarking. Tintype's training never shows *reals* with those distortions.
  - An artifact detector treats unfamiliar high-frequency corruption (noise, moiré, neural-codec texture) as generator evidence.
  - Fix: add a class-symmetric distortion bank. It should include Gaussian/Poisson/speckle noise, Gaussian and motion blur, unsharp sharpening, WebP and AVIF/HEIF, colour cast/jitter, gamma, moiré/screen recapture, a neural codec (e.g. a CompressAI model), invisible-watermark embedding and downscale-upscale chains. Apply it at p≈0.5–1.0 with 1–3 chained ops, in the style of Ant International's levels.
  - Expected: a large precision gain on degraded reals and much higher NTIRE robust AUC. A possible small drop in clean recall can be mitigated with TeleAI-style pairwise clean+distorted consistency, which they introduced precisely "to improve robustness without degrading clean-sample performance".
  - Cost: moderate (CPU augmentation, one more training phase). Risk: overfitting to NTIRE's specific operators. NTIRE changes operators per split, so hold some out.
- **Raise the messenger round-trip probability and diversify it.** Use real platform pipelines from TrueFake (three platforms) and the WasItAIGenerated WhatsApp/Instagram copies as *training* augmentation templates, i.e. their quantisation tables, chroma subsampling and resize targets. Raise the probability from 25% to about 50%, while keeping equal mass for real images.
  - Konstantinidou et al. and NTIRE both show augmentation gains of 5–10 AUC points on social data.
  - Messenger recall (27–36%) is a training-coverage problem *and* a threshold problem; see QuAD below.
- **Quality-conditioned thresholds (QuAD idea, adapted to single images).** Tintype can read JPEG quantisation tables and image size at decode time for free.
  - Fit per-quality-bin logit distributions for reals and fakes on the calibration set. Then either (a) use quality-dependent thresholds that keep 1.5% FPR *within each bin*, or (b) shrink logits of low-quality inputs towards the prior.
  - Effect: this directly limits false positives on heavily recompressed reals, and lets the threshold drop for messenger copies where reals score low, which would recover recall.
  - Cost: low, with no retraining. Risk: the calibration set needs enough degraded reals per bin.
- **Hard-negative reals from computational photography.** LAION-Mobile shows phone-ISP photos are the main false-positive regime for modern detectors.
  - Add a few thousand EXIF-verified smartphone photos (LAION-Mobile, and recent-phone sets if obtainable), RAISE/Forchheim camera photos, and *edited* reals: Lightroom denoise, Topaz/ESRGAN upscales, beauty filters.
  - Labelling AI-upscaled or denoised reals as "real" is a policy decision, and could lower recall on genuinely generated content that passed through upscalers. I suggest keeping classical ISP and denoise as real, and holding out AI-upscaled images as a separate evaluation slice first.
  - Expected: better precision on phone photos. Cost: low to moderate.
- **Texture crops are already supported by evidence** (Konstantinidou et al., +3 to +8.5 AUC). Keep them.
- **Do not chase backbone size alone.** Konstantinidou et al. found larger pretraining not reliably better. NTIRE winners used much larger models (DINOv3-H, 7B) at 512 px, but Tintype's latency budget precludes that. Data and augmentation changes are the levers Tintype can afford.

### Gaps
- The full NTIRE results table (per-team clean vs robust AUC on each split) was only partially extracted from the PDF. Exact numbers beyond MICV's 0.9723 / 0.9974 are not given here.
- I found no controlled study isolating beautify filters, AI upscalers or smartphone night mode as hard negatives with measured false-positive reduction. LAION-Mobile measures the problem, not the fix.
- TrueFake's per-platform detection numbers and its "most effective training strategies" were not retrieved.
- Frequency-domain augmentation (frequency masking, WaveRep wavelet rescaling for video) was not verified with image-detector numbers in this pass.

---

## Q3. Data scaling and diversity (Community Forensics, ScaleDF, OpenFake, Chameleon/AIDE)

### Takeaway
Generator diversity beats image count, with diminishing returns after about 1,000 generators at a fixed budget (Community Forensics). ScaleDF finds error falls as a power law in **both** the number of generation methods *and* the number of real-image domains, with no saturation, while raw image count saturates beyond about 10 M.

Tintype has about 1,200 generators but only 16 real sources, so *real-domain diversity* is the cheaper axis left to scale.

### Cited Findings

**19. Park & Owens, "Community Forensics: Using Thousands of Generators to Train Fake Image Detectors"**, CVPR 2025, arXiv:2411.04125.
- Data: 2.7 M generated images from 4,803 generators; 4,763 are systematically sampled latent-diffusion models with about 403 images each. — [arXiv HTML](https://arxiv.org/html/2411.04125)
- At a fixed 100 k images, performance rises with the number of generators but flattens beyond about 1,000. A 1,000-random-model classifier beat a 10-popular-model classifier at every image count, and performance plateaus at about 27 k images. The accuracy gap was larger than the mAP gap, which suggests diversity helps calibration. — [arXiv HTML](https://arxiv.org/html/2411.04125)
- A manual set of 774 k images over diverse architectures matched the 1.9 M LDM-only systematic set. Combining both was best. — [arXiv HTML](https://arxiv.org/html/2411.04125)
- Real-source alignment (LAION only vs a mix of ImageNet, COCO, LHQ, Forchheim, VISION and IMD2020) made "marginal" differences at scale. Frozen backbones were consistently worse than end-to-end fine-tuning. The best model (384 px) scores 0.987 mAP and 89.2% accuracy on 21 unseen models. **No deduplication step is described.** — [arXiv HTML](https://arxiv.org/html/2411.04125)

**20. Wang, Cai, Xiao, Wang, Yang, "Scaling Laws for Deepfake Detection" (ScaleDF)**, arXiv:2510.16320, Oct 2025.
- More than 5.8 M real images from **51 real sources** and more than 8.8 M fakes from 102 deepfake methods.
- Detection error follows a power law in the number of real domains and in the number of methods, "with no signs of saturation". Simply adding images beyond about 10 M without more diversity saturates.
- A ViT binary classifier trained on ScaleDF gave the best cross-benchmark generalisation. — [arXiv](https://arxiv.org/abs/2510.16320v1)
- Snippet-level; full tables not read.

**21. Livernoche et al., "OpenFake: An Open Dataset and Platform Toward Real-World Deepfake Detection"**, arXiv:2509.09495, 2025.
- About 3 M captioned reals and about 1 M fakes from proprietary and open models, plus a crowdsourced adversarial platform that keeps adding hard examples.
- Models trained on it generalise to unseen generators and to an in-the-wild social test set, beating models trained on older datasets. The abstract gives no numbers. — [arXiv](https://arxiv.org/abs/2509.09495)

**22. Yan et al., "A Sanity Check for AI-generated Image Detection" (Chameleon, AIDE)**, arXiv:2406.19435 (ICLR 2025 per the brief; the arXiv page lists no venue).
- On Chameleon (human-indistinguishable AI images), nine off-the-shelf detectors mostly classify AI images as real.
- AIDE combines CLIP semantics with the highest- and lowest-frequency patches: +3.5% on AIGCDetectBenchmark and +4.6% on GenImage. — [arXiv](https://arxiv.org/abs/2406.19435)

**23. Konstantinidou et al. (as in 11).** Training on real-world social data (TWIGMA) beat LDM-only training for every detector, and the combined 1.2 M set was best for most. "Naively scaling… more training data does not always lead to better detection." — [arXiv HTML](https://arxiv.org/html/2507.10236v1)

### Inferences (Tintype applicability)

- **Scale real-domain diversity.** Tintype has about 1,200 generators, past Community Forensics' plateau of about 1,000, but 16 real sources against ScaleDF's 51.
  - Candidate real domains to add:
    - smartphone photos (LAION-Mobile);
    - raw-camera photos (RAISE, Dresden, VISION, Forchheim);
    - screenshots and recaptures;
    - documents, memes and text-heavy images;
    - night, underwater and astro photography;
    - medical and scientific imagery;
    - scanned film;
    - art photography;
    - degraded archival photos;
    - social-platform reals from Reddit, Bluesky and WhatsApp, which matters most given Q1.
  - Expected: precision gains, i.e. fewer false positives on unusual real content. ScaleDF also suggests recall gains through better-defined decision boundaries.
  - Cost: moderate in licensing and curation. Risk: noisy or AI-contaminated reals (see Q5).
- **Generator diversity: target families, not counts.** Community Forensics showed architecture diversity (774 k manual images ≈ 1.9 M LDM-only images) matters. Tintype's weak spots, older GANs (84–89%) and face swaps (14%), are architecture families that are under-represented by *family*.
  - Add more GAN variants: ProGAN, StyleGAN-XL, BigGAN-deep, GigaGAN, CycleGAN/pix2pix and StarGAN.
  - Add face swap and reenactment: DF40 or FaceForensics++ frames, SimSwap, InSwapper, if face swaps come into scope.
  - Add autoregressive and pixel-space models.
  - Weight sampling by family rather than by store size. Currently sources are sampled at count^0.5.
- **Image count per generator can stay small.** Community Forensics plateaus at about 27 k images and found about 400 images per generator sufficient for systematic LDMs. A new generator can be added with a few hundred to a few thousand images. This favours frequent small updates (Q4).
- **Chameleon and similar human-hard sets** are good held-out evaluation sets. TeleAI in NTIRE trained on Chameleon, which forfeits its use for evaluation.

### Gaps
- No paper directly compares "add N more generators" vs "add N more real domains" at the same cost for a small ViT. ScaleDF's per-axis exponents were not retrieved.
- ScaleDF concerns deepfakes broadly. Whether its 102 methods include face swaps (relevant to Tintype's 14% face-swap recall) was not verified.
- OpenFake's numeric generalisation results were not retrieved.

---

## Q4. Continual, incremental and few-shot adaptation to new generators

### Takeaway
Methods from 2025–2026 report near-zero forgetting when adding generators:
- DECODE: 99.36% average accuracy, 0.39% forgetting over 19 domains.
- EvoKnow: 4.32% forgetting, with no replay.
- K-FAC regularisation with a 500-sample replay buffer and model blending.
- Few-shot metric learning: +11.6% with 10 samples.

The cheapest robust recipe for Tintype is periodic short fine-tunes on new-generator data plus a small replay buffer, or per-generator LoRA adapters merged into the backbone, followed by head recalibration and threshold re-fitting. In-the-wild fact-check mining keeps the data current.

### Cited Findings

**24. Li et al., "A Continual Deepfake Detection Benchmark: Dataset, Methods, and Essentials" (CDDB)**, WACV 2023, arXiv:2205.05467.
- Easy, hard and long task sequences over known and unknown generators, with multiclass incremental-learning methods adapted to the task. The authors found CDDB "clearly more challenging than existing benchmarks". — [arXiv](https://arxiv.org/abs/2205.05467); [code](https://github.com/Coral79/CDDB)

**25. Azizpour, Nguyen, Shrestha, Xu, Kim, Stamm, "E3: Ensemble of Expert Embedders for Adapting Synthetic Image Detectors to New Generators Using Limited Data"**, CVPR Workshops (WMF) 2024, arXiv:2404.08814.
- One expert embedder per generator, trained by transfer learning, plus an Expert Knowledge Fusion Network. It outperforms existing continual-learning methods, including detector-specific ones, with limited data. — [arXiv](https://arxiv.org/abs/2404.08814)

**26. Wu, Liu, Li, Wang, "Few-Shot Learner Generalizes Across AI-Generated Image Detection" (FSD)**, ICML 2025, arXiv:2501.08763.
- Learns a metric space so that unseen generators can be detected from a few samples: +11.6% average accuracy on GenImage with 10 extra samples. — [arXiv](https://arxiv.org/abs/2501.08763)

**27. Cai et al., "DECODE: Tackling Representation and Decision Degradation in Continual AI-Generated Image Detection"**, arXiv:2607.27882, Jul 2026.
- Identifies "dual degradation": features stay discriminative while the classification head's decision boundary drifts.
- Subspace Diversity Regularization guards against representation drift. Closed-Form Decision Alignment recalibrates the shared head after each adapter merge.
- Results: 99.36% average accuracy over 19 domains, 0.39% forgetting, 95.36% on 11 unseen generators. — [arXiv](https://arxiv.org/abs/2607.27882)

**28. EvoKnow: Continual Knowledge Evolution for AI-Generated Image Detection**, arXiv:2610.11381, 2026.
- Replay-free protocol: 96.32% mean stage-wise accuracy and 4.32% average forgetting. — [arXiv HTML](https://arxiv.org/html/2610.11381) (search-snippet level)

**29. "Generalizable and Adaptive Continual Learning Framework for AI-generated Image Detection"**, arXiv:2601.05580, 2026.
- Combines a K-FAC curvature regulariser, a fixed 500-sample replay set and blending of old and new model weights. — [alphaXiv](https://www.alphaxiv.org/abs/2601.05580) (search-snippet level)

**30. Pantsios, Karageorgiou, Koutlis, Karantaidis, Papadopoulou, Papadopoulos, "Automated In-the-Wild Data Collection for Continual AI Generated Image Detection"**, arXiv:2605.02567 (ACM DOI 10.1145/3810988.3812662), 2026.
- A weakly supervised pipeline mines in-the-wild AI and real images from fact-check articles. "Adding even a small amount of generator-driven data" enables adaptation to new generators, and combining it with in-the-wild data in a continual framework mitigates forgetting. Reported gains: +9.14% and +8% average accuracy on two detectors. — [arXiv](https://arxiv.org/abs/2605.02567)

**31. Cozzolino et al., "Raising the Bar of AI-generated Image Detection with CLIP"**, CVPR Workshops 2024.
- Few-shot training with a light head on CLIP features generalises from few examples. — [CVF](https://openaccess.thecvf.com/content/CVPR2024W/WMF/papers/Cozzolino_Raising_the_Bar_of_AI-generated_Image_Detection_with_CLIP_CVPRW_2024_paper.pdf)
- Numbers not re-verified here.

### Inferences (Tintype applicability)

- **Tintype's current update scheme is "naive fine-tune with near-full replay".** Each phase continues from the previous weights on all earlier stores ([DATA.md](../../docs/DATA.md)). This avoids forgetting but costs a full pass over about 300 k images per update.
- **A cheaper scheme for post-cutoff generators** would be monthly or quarterly:
  1. Collect 500–3,000 images per new generator. Community Forensics suggests a few hundred is enough per generator.
  2. Fine-tune for about 1–2 k steps on new data plus a fixed, stratified replay buffer: e.g. 50–200 images per old generator family and per real source, about 20–40 k total.
  3. Optionally use LoRA adapters merged afterwards, as in DECODE and NTIRE TeleAI.
  4. Recalibrate the head or bias, then re-fit the operating-point thresholds on held-out reals. DECODE shows that head drift, not feature loss, causes much of the forgetting, and Tintype already re-calibrates thresholds per release.

  Expected: recall on new generators recovered within days of release, with old-generator recall preserved.
  Cost: low, since Tintype trains on an Apple GPU. Risk: calibration drift on reals, hence the mandatory re-calibration.
- **Fact-check mining** (Pantsios et al.) is a scalable source of *in-the-wild* positives that already carry social-platform processing, and of matched real negatives. That suits Tintype's messenger weakness and is policy-compatible because the items are public.
- **E3 or few-shot heads** add inference cost (multiple embedders) or an extra metric head. For a single-ViT-S product, merging adapters is preferable to keeping an ensemble.

### Gaps
- No continual-learning paper reports results under messenger compression. Their accuracy figures are mostly on clean benchmark domains and are not comparable across papers.
- DECODE's and EvoKnow's baselines and backbones were not retrieved. Whether their gains hold for a small ViT-S at 224 px is unknown.
- CDDB's ranking of continual methods (which method forgot least) was not retrieved.

---

## Q5. Label noise, contamination and deduplication

### Takeaway
There is little detection-specific literature on AI images contaminating "real" training sets. The practice in recent large datasets and pipelines is CLIP or embedding-based near-duplicate removal, plus VLM filtering, plus an AIGC detector pass over the reals. Tintype's byte-identical deduplication misses re-encoded and resized near-duplicates, and its LAION- and Reddit-derived reals may contain post-2022 AI images.

### Cited Findings
- NTIRE 2026 organisers filtered about 12 M web reals with "resolution thresholding and CLIP-deduplication to complex VLM-based image categorization and scoring", removing 90%. They used only unique images in the validation and test splits to avoid paired-image advantages. — [arXiv PDF](https://arxiv.org/pdf/2604.11487)
- The LongCat-Image technical report deduplicates with MD5 for exact copies and SigLIP similarity for near-duplicates. It built an internal AIGC detector to purge synthetic images from its real corpus, noting that a small fraction of AIGC "can disrupt optimization". This is a generator-training context, but the same contamination issue applies to detector reals. — [arXiv](https://arxiv.org/pdf/2512.07584)
- Community Forensics describes no deduplication step; it keeps original formats and uses PNG where possible. — [arXiv HTML](https://arxiv.org/html/2411.04125)
- Rajan et al. found a format-contamination shortcut: WebP artifacts present only in reals degraded fake scores under compression. — [arXiv HTML](https://arxiv.org/html/2410.11835v3)
- Tintype removed 916 byte-identical train/eval overlaps; the check was byte-level only. — [DATA.md](../../docs/DATA.md)

### Inferences (Tintype applicability)
- **Near-duplicate deduplication.** Use a perceptual hash (pHash/dHash) plus CLIP/DINO embedding cosine above about 0.95 across train vs eval, in both classes.
  - COCO and ImageNet appear in Tiny-GenImage, Defactify, Community Forensics, OpenFake and in evaluation sets, often re-encoded or resized. Byte equality misses those copies.
  - Expected: more honest evaluation numbers, and removal of memorisation-driven "real" decisions.
  - Cost: low (embeddings for about 390 k images).
- **Screen the reals for AI contamination.**
  - Score every real training image with an ensemble (Tintype plus the Community Forensics 384 model plus one CLIP-based detector). Manually review or drop the top approximately 0.5% by score.
  - Prefer sources with capture dates before mid-2022, or EXIF/C2PA-verified provenance, for social and LAION reals.
  - Expected: less label noise in the real class, sharper boundaries, possibly higher recall.
  - Risk: removing genuinely hard reals, which would re-introduce false positives. Review rather than auto-drop.
- **Format audit per class.** Following Grommelt et al. and Rajan et al., audit the distributions of container format (JPEG, PNG, WebP), JPEG quality and quantisation-table family, chroma subsampling, resolution and EXIF presence *per class and per source*. Then resample or augment until they match.
  - The script is cheap and catches shortcuts such as "PNG ⇒ AI" or "Reddit-recompressed ⇒ AI".

### Gaps
- I found no peer-reviewed measurement of the AI-image contamination rate in LAION, Reddit or OpenFake real splits, or of its effect on detector false-positive and false-negative rates.
- No study quantifies how much near-duplicate leakage inflates AI-image-detection benchmark scores.

---

## Ranked shortlist for Tintype (3–5 most promising ideas from this area)

### Takeaway
Ranked by expected impact on the stated weak spots per unit cost:
1. Fix class-asymmetric processing, and add a class-symmetric NTIRE-style distortion bank with pairwise consistency.
2. Quality-conditioned calibration.
3. Aligned VAE reconstructions of Tintype's reals.
4. More real domains and hard-negative reals.
5. A cheap continual update loop with a replay buffer and head recalibration.

### Cited Findings
1. **Class-symmetric degradation, plus rebalancing the Reddit/social-processing shortcut.** Targets NTIRE false positives (155/600 reals) and recall, as well as messenger recall.
   - Evidence:
     - NTIRE organisers distort both classes and align JPEG and size statistics — [arXiv PDF](https://arxiv.org/pdf/2604.11487).
     - Top NTIRE teams used aggressive multi-distortion augmentation (p up to 1.0) and pairwise clean/distorted training — [arXiv PDF](https://arxiv.org/pdf/2604.11487).
     - Removing JPEG/size bias gave more than 11 pts — [arXiv](https://arxiv.org/abs/2403.17608).
     - Augmentation gave +4.5 to +10 AUC in the wild — [arXiv HTML](https://arxiv.org/html/2507.10236v1).
     - Tintype's `openfake-reddit` store is 3.0 k real vs 15.8 k AI — [DATA.md](../../docs/DATA.md).
2. **Quality-conditioned thresholds or score calibration.** Targets false positives on degraded reals and messenger recall, with no retraining.
   - Evidence: QuAD's quality-dependent logit modelling took bAcc from 73.2% (naive) to 81.6% — [arXiv HTML](https://arxiv.org/html/2604.15027v1).
   - Evidence: thresholds fitted on one content regime give 17–91% FPR on phone photos — [arXiv](https://arxiv.org/abs/2609.11134).
3. **Aligned, content-matched fakes from Tintype's own reals:** multi-VAE reconstructions, with 4- and 16-channel VAEs, and optionally B-Free self-conditioned inpainting.
   - Targets recall under compression and new LDM generators, and gives some local-edit coverage.
   - Evidence: Playground recall after post-processing went from 24% to 76% — [arXiv HTML](https://arxiv.org/html/2410.11835v3).
   - Evidence: B-Free bAcc went from 81.4 to 96.4, with WildRF bAcc of 86–97 — [arXiv HTML](https://arxiv.org/html/2412.17671v2).
4. **More real domains and computational-photography hard negatives** (smartphone/LAION-Mobile, raw camera, social reals, screenshots).
   - Evidence: power-law error reduction with the number of real domains — [arXiv](https://arxiv.org/abs/2510.16320v1); phone-ISP false-positive regime — [arXiv](https://arxiv.org/abs/2609.11134).
5. **Continual update loop:** a few hundred to a few thousand images per new generator, a stratified replay buffer, optional LoRA plus merge, then head and threshold recalibration; in-the-wild data mined from fact-checks.
   - Evidence: diversity plateau and small per-generator counts suffice — [arXiv HTML](https://arxiv.org/html/2411.04125).
   - Evidence: head drift drives forgetting, and DECODE reaches 0.39% forgetting — [arXiv](https://arxiv.org/abs/2607.27882).
   - Evidence: a 500-sample replay plus K-FAC framework — [alphaXiv](https://www.alphaxiv.org/abs/2601.05580); fact-check mining gave +8 to +9 pts — [arXiv](https://arxiv.org/abs/2605.02567).

### Inferences
- Ideas 1 and 2 are cheap and address the most damaging failures. The NTIRE false positives hurt precision in exactly the deployment setting that matters, and messenger recall is the 27–36% weak spot. Do them first, together:
  - Train with the symmetric distortion bank.
  - Calibrate per JPEG-quality bin.
  - Re-evaluate on NTIRE distorted, the WasItAIGenerated messenger copies and So-Fake-OOD, using a fresh confirmation batch per the repo's pre-registration practice.
- Idea 3 is the best-evidenced *recall* lever for current latent-diffusion generators under compression, at modest compute. Mix the reconstructions in at about 20–30% of the fake class rather than replacing generator data, because the alignment papers do not cover GANs or autoregressive models.
- Ideas to avoid or deprioritise:
  - mixup/cutmix (measured harmful in B-Free);
  - naive scaling of image count (saturates per Community Forensics, ScaleDF and Konstantinidou et al.);
  - expert ensembles (E3) that multiply inference cost.
- Older GANs and face swaps are better served by *family* diversity than by any of the above. Add GAN variants and face-swap frames weighted per family, or keep using the photo second opinion.

### Gaps
- None of the expected gains has been measured on a ViT-S/16 at 224 px with six-view inference. The magnitudes are extrapolated from larger backbones (DINOv2/v3, CLIP-L) and different benchmarks.
- The risk that aggressive distortion augmentation lowers clean recall on current generators is mitigated by pairwise training in NTIRE reports, but this is not quantified in the parts of the report I read.
