# Gap fill: stones the first eight notes left unturned

Scope note. This note was written after the draft report (`draft_report.md`) by a completeness critic. It checks which sub-areas, influential papers and improvement levers notes 01–08 missed or covered thinly, and researches each real gap against primary sources. Tintype is a ViT-S/16 at 224 px, fine-tuned from the Community Forensics 224 checkpoint on about 1,200 generators and 16 real sources. It averages logits over up to six views (one fit view, a centre native crop, four texture-ranked native crops) with early exit. Calibration is a single real-quantile threshold. The trainer has no EMA, no weight averaging and no anchor to the initial weights (`00_codebase_facts.md:210`).

Verification status. Every paper below was looked up on its arXiv abstract page or HTML full text this session (2026-11-13), unless tagged. **[abs]** means only the abstract was read, so the paper gave no figures or I did not retrieve them. **[snip]** means the facts come from a search-result snippet of the primary page. **[ctx]** means the paper is cited only as context. None of the arXiv IDs below appears in `99_bibliography.md`; I checked each one with `grep`.

**New distinct papers in this note: 27.** 26 have arXiv IDs, including the classic randomized-smoothing paper cited only as context, and 1 EUSIPCO paper is cited by DOI.

---

## Gap 1. Autoregressive and unified-multimodal image generators (GPT-image, Nano Banana, Seedream, Janus, Infinity)

### Why this is a gap
Notes 01–08 mention autoregressive (AR) generators only in passing, as "not covered". No note cites an AR-specific detector or benchmark. The repo's own per-generator table shows this is where Tintype loses recall on clean images. On So-Fake-OOD batch 3, at the shipped threshold, Tintype finds **0.58 of GPT-image-2, 0.52 of Seedream 3.0, 0.59 of Nano Banana, 0.60 of Recraft v3 and 0.32 of FLUX.1 pro**, against 0.90–0.98 for FLUX.2, HiDream and Seedream 4.5 (`docs/BENCHMARK.md:479-495`). No open AR model (LlamaGen, VAR, Infinity, Janus-Pro, Switti, BAGEL, Lumina-mGPT) is named in `docs/DATA.md`.

### Takeaway
Two results matter for Tintype. First, broad trained detectors already beat AR-specialist detectors on open AR text-to-image models (PRADA's own table: B-Free 99.8 and RINE 99.0 AUROC vs D³QE 76.5). The missing ingredient is training data from AR models and their decoders, not a new architecture. Second, the shared component across diffusion and AR generators is the *final decoder* (VAE, VQ tokenizer or super-resolution upsampler). Training on real images passed through those decoders reached 98.83% average accuracy on 22 unseen-generator sets from 100 samples per decoder category. This generalises recommendation 4's VAE reconstructions to VQ decoders and upsamplers.

### Cited findings
- **D³QE: Discrete Distribution Discrepancy-aware Quantization Error for AR-generated image detection** (Y. Zhang, B. Yu, Y. Zheng, W. Zheng, Y. Duan, L. Chen, J. Zhou, J. Lu; ICCV 2025; arXiv 2510.05891). It introduces the ARForensics benchmark, covering 7 visual AR models: LlamaGen, VAR, Infinity, Janus-Pro, RAR, Switti and Open-MAGVIT2. Training uses LlamaGen only (100k fakes vs 100k ImageNet reals), with a frozen CLIP and LlamaGen's VQ encoder. Mean accuracy / AP: **D³QE 82.11 / 92.07, UnivFD 79.56 / 90.44, NPR 72.32 / 91.43, CNNSpot 64.90 / 80.21**. D³QE is strongest on LlamaGen (97.19 Acc) and Janus-Pro (92.28). It is weakest on Infinity, a bitwise-tokenizer model (62.88 Acc / 79.39 AP), where UnivFD scores higher (71.72 / 85.77). Baselines degrade on architecturally distinct AR models (VAR, Switti). AP stays above 85% at JPEG quality 60. — [arXiv 2510.05891](https://arxiv.org/abs/2510.05891); [HTML](https://arxiv.org/html/2510.05891); dataset at huggingface.co/datasets/Yanran21/ARForensics
  - *Tintype applicability:* ARForensics is a ready-made plan store of 7 open AR generators. Use it as held-out evaluation first, then as training data. The Infinity result warns that VQ-tokenizer families differ, so holding out one tokenizer family is a better test than holding out one model.
- **PRADA: Probability-Ratio-Based Attribution and Detection of AR-generated images** (S. Damm, J. Ricker, H. Petzka, A. Fischer; CVPR 2026 Findings; arXiv 2511.20068). This is a white-box detector built on the ratio of the AR generator's conditional to unconditional token likelihoods. Mean AUROC is **93.7% on 8 class-to-image and 99.4% on 4 text-to-image AR models** (Infinity-2B, Janus-Pro-1B, LlamaGen-XL, Switti-1024). Attribution accuracy is 88.0% and 96.3%. Comparison on T2I AR models: **B-Free 99.8, PRADA 99.4, RINE 99.0, Corvi 97.0, DRCT 89.2, D³QE 76.5, RIGID 72.0**. Limits: it needs generator weights, costs about one generation pass per image, needs a prompt (BLIP-2 captions), and class-to-image performance drops under cropping and noise. — [arXiv 2511.20068](https://arxiv.org/abs/2511.20068); [HTML](https://arxiv.org/html/2511.20068v2)
  - *Tintype applicability:* reject as a runtime method, since it is white-box and costs a generation pass. Its comparison table is the useful part: a broadly trained detector (B-Free) is near-perfect on open AR T2I models. Tintype's AR weakness is therefore most likely a data gap that recommendation 4 or 10 can close.
- **Exploiting the Final Component of Generator Architectures for AI-Generated Image Detection** (Y. Liu, X. Liu, Y. Wang, M. Soumik; arXiv 2601.20461, v2 March 2026) **[abs]**. Many diffusion and AR generators share a final component that maps internal representations to pixels. The authors "contaminate" real images by passing them through that component and train real-vs-contaminated. They give a taxonomy of 21 generators by final component, fine-tune DINOv3 on only **100 samples from each of three component categories**, and report **98.83% average accuracy over 22 unseen-generator test sets**. — [arXiv 2601.20461](https://arxiv.org/abs/2601.20461)
  - *Tintype applicability:* this generalises recommendation 4. Build reconstruction stores of Tintype's reals through (a) the 4- and 16-channel VAEs already proposed, (b) open VQ/BSQ tokenizers (LlamaGen, Janus-Pro, Infinity), and (c) open learned upsamplers. The per-generator gaps on GPT-image-2, Seedream 3 and Nano Banana are the targets. The labelling conflict with neural codecs (draft §4 risks) applies here too.
- **GPT-ImgEval** (Z. Yan et al.; arXiv 2504.02782, v3). Section 5.2 tests whether forensic detectors catch GPT-4o images. Accuracy: **CNNSpot 73.81, UnivFD 75.58, CLIP-LoRA 77.81, NPR 78.25, DRCT 88.24, Effort 94.75, FakeVLM 99.60**. The authors attribute detectability to an internal super-resolution or upscaling stage, since GPT-4o returns sharpened high-resolution output even when asked to reproduce a blurry input unchanged. The prose claims NPR reaches 99%, but the table says 78.25, so the text overstates. — [arXiv HTML 2504.02782v3](https://arxiv.org/html/2504.02782v3)
  - *Tintype applicability:* if GPT-image outputs carry an upsampler trace, Tintype's native-resolution crops should see it, but messenger downscaling (cap 960 px) destroys it first. That is consistent with Tintype's GPT-image-2 recall of 0.58 clean, and it suggests training on downscaled GPT-image samples specifically. It also supports the final-component idea above (upsampler as decoder).
- **A Benchmark Dataset for MLLM-Generated Image Detection: GPT Image2 & Nano Banana2** (Z. Zhang, Y. Yu, D. Guan, C. Tian, D. Zhang, Q. Zhu; arXiv 2608.01258, Aug 2026). 726 real and 726 fake images per generator, in Texture, Structure and Hybrid splits, with JPEG and mild blur applied in preprocessing. Overall accuracy of recent generalisable detectors on **GPT Image2: LTD 82.52, GAPL 75.07, IAPL 70.37, PGC 57.21**. On **Nano Banana2: GAPL 81.14, LTD 80.71, IAPL 72.25, PGC 72.78**. The proposed SAP-DSP reaches 87.9–96.1 per split. — [arXiv 2608.01258](https://arxiv.org/abs/2608.01258); [HTML](https://arxiv.org/html/2608.01258v1)
  - *Tintype applicability:* a small, current, independent held-out set for exactly the generators where Tintype's per-generator recall is weakest. Add it to the external evaluation list (README "External image benchmarks") rather than to training.
- **Guess the Unified Model: How Much Can We Recover from Generated Images?** (J. Cekinmez, R. Mitsuhashi, A. J. Wu, Y. Yin; arXiv 2605.25254, May 2026) **[abs]**. Attribution among seven unified generation-understanding models is "highly feasible", with near-perfect accuracy from about 20k images per model. Corruptions and structural perturbations have only a modest effect. Semantic content helps but is not the dominant signal. — [arXiv 2605.25254](https://arxiv.org/abs/2605.25254)
  - *Tintype applicability:* unified models leave strong, corruption-robust model traces, so a modest number of samples per unified model should be enough as training data. That supports recommendation 10's 500–3,000 images per new generator.
- **Breaking Latent Prior Bias in Detectors (OMAT, on-manifold adversarial training)** (Y. Zhou, X. He, K. Lin, B. Fan, F. Ding, B. Li; arXiv 2506.00874). The method optimises a diffusion model's initial latent noise against the detector, keeping the conditioning fixed, and adds 6,000 such on-manifold fakes from SD v1.4 to GenImage-SD1.4 training. GenImage average accuracy rises from **79.20 → 84.07 (CLIP linear)** and to **94.63 with CLIP LoRA rank 4 (+15.43)**. On the test-only GenImage++ (FLUX.1 and SD3), OMAT CLIP-LoRA averages **96.78%** (FLUX 92.6–97.7, SD3 90.4–98.8), against 73.54% for DRCT/Conv-B. — [arXiv HTML 2506.00874v1](https://arxiv.org/html/2506.00874v1)
  - *Tintype applicability:* a training-data idea in the flow-matching era. It generates "hard fakes" from an open generator by attacking Tintype itself in latent space. It needs offline GPU generation with an open FLUX or SD3 model and the Tintype weights exported to PyTorch, which the repo does not do today. A candidate later store for recommendation 4 or 10, not a top-10 change.

### Effect on the ranking
**Strengthens recommendation 4 and widens it.** Rename it "real-image reconstructions through every final-decoder family (VAE 4/16-ch, VQ/BSQ tokenizers, learned upsamplers), plus open AR generators". Add ARForensics, the GPT Image2 / Nano Banana2 set and GenImage++ as held-out tests. **Add to recommendation 10** a per-family chronological test: hold out the AR/unified family as a whole.

---

## Gap 2. AI-upscaled, AI-edited and computationally processed real photos as false-positive sources

### Why this is a gap
Notes 01–08 cover neural codecs (Cannas et al.), smartphone reals (LAION-Mobile) and BIAS-ID's bilinear upscaling, but none covers *generative* super-resolution or AI editing of real photos, now standard in phone galleries and photo apps. Nor do they cover the finding that inpainting detectors work through a global VAE trace.

### Takeaway
Two mechanisms make an edited real photo look fully generated. A learned upsampler adds generator-like high frequencies, and latent-diffusion editing passes the *whole* image through a VAE. INP-X shows that **97% of the background spectral shift of an inpainted image comes from the autoencoder round trip alone**, and that restoring the unedited pixels drops detectors to near chance. For Tintype this cuts both ways. It will likely flag AI-edited real photos, which may be the desired outcome, but its "fully generated" claim is then partly a VAE-trace detector. Tintype also needs a written labelling policy for upscaled and edited reals before training on them.

### Cited findings
- **AI-Generated Image Detectors Exploit Global Artifacts: Evidence from Inpainting Exchange (INP-X)** (E. Nebioglu, E. Bilgiç, A. Popescu; WACV 2027; arXiv 2602.00192, v2 Nov 2026). INP-X restores the original pixels outside the inpainted mask and keeps the synthesized region. Commercial APIs fall from **>90% to about 55%**. The bias-aware DDA falls from **97.6% to 58.4%**, and other detectors collapse toward chance. A variance decomposition attributes **97% of the background spectral shift to the VAE round trip**. Human accuracy fell from 63.8% to 56.2% (153 participants, p = 0.002). The benchmark has 90k images (30k triplets over four datasets and three inpainters). Mixed-distribution and localisation-supervised training help, but detecting INP-X edits stays open. — [arXiv 2602.00192](https://arxiv.org/abs/2602.00192)
  - *Tintype applicability:* (1) It supports recommendation 4's VAE-reconstruction store as an efficient source of the trace Tintype already relies on. (2) It confirms the draft's warning that VAE-laundered reals will be flagged. (3) INP-X triplets are a free test of whether Tintype's six-view average catches a pasted generated region at all (expected: no, consistent with README's "local edits are out of scope"). Score the 90k-image set to put a number on that limit in README.
- **Detecting Localized Deepfakes: How Well Do Synthetic Image Detectors Handle Inpainting?** (S. Pandolfini, L. Pellegrini, M. Ferrara, D. Maltoni; arXiv 2512.16688, Dec 2025) **[abs]**. Detectors trained on many generators transfer partly to inpainting. They reliably catch medium and large edited areas and regeneration-style inpainting, but not small edits. — [arXiv 2512.16688](https://arxiv.org/abs/2512.16688)
  - *Tintype applicability:* a cheap prior for the INP-X test. Expect Tintype to catch large-area and regeneration edits, which pass the whole image through a VAE, and miss small pasted ones.
- **UniAIDet: A Unified and Universal Benchmark for AI-Generated Image Content Detection and Localization** (arXiv 2510.23023, Oct 2025) **[snip]**. About 80k real and generated images from 20 generators, including inpainting and editing models, with masks. Existing detection and localisation methods perform poorly. — [arXiv 2510.23023](https://arxiv.org/html/2510.23023v1)
  - *Tintype applicability:* an alternative edit-aware evaluation set if Tintype ever reports partial-edit numbers. Evaluation only.
- **Adversarial Magnification to Deceive Deepfake Detection through Super Resolution** (D. A. Coccomini, R. Caldelli, G. Amato, F. Falchi, C. Gennaro; arXiv 2407.02670, 2024). EDSR ×2 super-resolution of *pristine* FF++ frames raises false-positive rates several-fold: **ResNet50 3.2 → 10.1%, Swin 3.6 → 12.4%, XceptionNet 2.6 → 12.4%** (Deepfakes split), and up to **12.8 → 23.9%** for Swin on NeuralTextures. — [arXiv HTML 2407.02670](https://arxiv.org/html/2407.02670)
  - *Tintype applicability:* add learned super-resolution (EDSR or Real-ESRGAN class, ×2 and ×4, then downscale back) to recommendation 1's per-operation shortcut audit on held-out reals. If FPR jumps, add SR'd reals to training *only* after deciding a labelling policy ("AI-upscaled real = real" is the choice the draft's codec policy implies).
- **Apple's Synthetic Defocus Noise Pattern: Characterization and Forensic Applications** (arXiv 2505.07380, 2025) **[snip, ctx]**. Portrait-mode synthetic defocus adds a characteristic noise pattern that has caused false positives in camera-source attribution. Masking the affected regions reduces them. — [arXiv 2505.07380](https://arxiv.org/abs/2505.07380)
  - *Tintype applicability:* context for the phone-processing false-positive source that LAION-Mobile measures. If LAION-Mobile-style reals are added (recommendation 4), stratify FPR by capture mode (portrait, night, HDR) where EXIF allows.

### Effect on the ranking
**No new top-10 item.** It strengthens recommendation 1's shortcut audit: add generative SR, a VAE round trip, and INP-X-style partial VAE passes to the per-operation list. It forces an explicit labelling policy in recommendation 4 that covers upscaled and AI-edited reals as well as VAE-laundered ones and codecs.

---

## Gap 3. AI images of text, documents, receipts and screenshots

### Why this is a gap
Notes 01–08 do not discuss text-rich images, and Tintype's real sources contain few screenshots. This domain interacts badly with Tintype's design. Its native crops go to the **highest mean luma gradient** windows (`src/image_io.rs`), and in a document those are dense text, where the "Text Density Curse" below says low-level detectors break down.

### Cited findings
- **TextFake: Benchmarking AI-Generated Image Detection on Text-Rich Images** (Y. Zhang, C. Miao, M. Liao, T. Liu, X. Wang, T. Gong, Q. Chu, N. Yu; arXiv 2606.01050, May 2026) **[abs]**. 20k fabricated screenshots, documents and news pages in 28 languages. Zero-shot, **none of 14 specialised detectors or 3 frontier VLM APIs exceeds 80% accuracy**, and some drop by more than 60% relative to natural-image benchmarks. It names three failure modes: the *Text Density Curse*, where dense text overwhelms low-level detectors; *Cloaking via Rendering Fidelity*; and *Threshold Collapse*, where routine perturbations push detectors to chance. — [arXiv 2606.01050](https://arxiv.org/abs/2606.01050)
  - *Tintype applicability:* the Text Density Curse lands directly on texture-ranked crop selection, which favours text. Score TextFake with `--all-views` and compare per-view logits for text-dense crops against the fit view. If text crops are less informative, add a text-density penalty to the texture measure (part of recommendation 5's view-selection study).
- **Beyond Natural Images: Rethinking AI-Generated Image Detection in Documents (AIGDoc)** (Z. Fu, J. Yan, Y. Chen, X. Yu, Y. Li, H. Jiang, L. Gao, C. Bao; arXiv 2609.14352, Sept 2026) **[abs]**. 27k synthetic document images, 17 subsets, 13 generation methods. Mean detector AUC falls by **more than 7%** on documents compared with natural images. Document-domain training narrows the gap only partly. A search summary notes it disagrees with TextFake on whether text density drives the drop. — [arXiv 2609.14352](https://arxiv.org/abs/2609.14352)
  - *Tintype applicability:* the more moderate estimate (−7 AUC) of the domain gap. Use it with TextFake for an honest README limit line.
- **TextRich: A Multi-Domain Benchmark for Detecting AI-Generated Text-Rich Images from GPT-Image-2** (Y. Wang, S. Wang, W. Zhang, Y. Ouyang; arXiv 2606.19259) **[abs]**. 12,095 GPT-Image-2 images over six categories: posters, infographic charts, academic posters, receipts, tables and UI screenshots. The strongest of five detectors is competitive overall, but ineffective on some structured categories and "highly sensitive to JPEG compression". — [arXiv 2606.19259](https://arxiv.org/abs/2606.19259)
  - *Tintype applicability:* GPT-image-2 is already a weak generator for Tintype (0.58 recall). Receipts and UI screenshots are a high-stakes fraud use case, so this is the most operationally relevant text-rich test.

### Effect on the ranking
**No new top-10 item**, but a **new README Limits line** and an evaluation task. Measure Tintype on TextFake, AIGDoc and TextRich before claiming coverage of screenshots or documents. Add "text-dense windows" as a condition in recommendation 5's view-selection experiment. Recommendation 4's "add screenshots as a real domain" becomes more important, because real screenshots are needed to calibrate FPR on text.

---

## Gap 4. Weight-space methods: WiSE-FT interpolation, model soups, EMA and model merging

### Why this is a gap
The draft's recommendation 7 proposes L2-SP or an Effort-style penalty to stop drift from Community Forensics, at "moderate trainer work", and is rated low-medium confidence. No note considered **weight-space interpolation**, which needs no retraining. The trainer has no EMA of weights either (`00_codebase_facts.md:210`).

### Takeaway
Linear interpolation between a pretrained and a fine-tuned model (WiSE-FT) and averaging several fine-tuned runs (soups) give robustness under distribution shift at zero inference cost. In deepfake detection specifically, plain weight averaging of specialists beat joint training on cross-dataset (**0.834 vs 0.662 AUC**) and unseen generators (**0.724 vs 0.665**). For Tintype the zero-cost experiment is θ(α) = (1−α)·θ_Tintype + α·θ_CF224. Sweep α from 0 to 0.5, run `finalize` to recalibrate, and score the GAN sets, FF++ and batch 7. This gives the cheapest test of recommendation 7's forgetting hypothesis and may fix part of the GAN gap with no training at all.

### Cited findings
- **WiSE-FT: Robust fine-tuning of zero-shot models** (M. Wortsman, G. Ilharco, J. W. Kim, M. Li, S. Kornblith, R. Roelofs, R. Gontijo-Lopes, H. Hajishirzi, A. Farhadi, H. Namkoong, L. Schmidt; CVPR 2022; arXiv 2109.01903). Ensembling the weights of the zero-shot and fine-tuned models gives **+4 to 6 pp under distribution shift** over prior work on ImageNet's five derived shifts, with **+1.6 pp in distribution**. It gains 2–23 pp on six further shifts and 0.8–3.3 pp on seven transfer datasets, at **no extra compute in training or inference**. — [arXiv 2109.01903](https://arxiv.org/abs/2109.01903)
  - *Tintype applicability:* both endpoints share the ViT-S/16 architecture (Tintype starts from CF-224), so interpolation is a ten-line offline tool over the safetensors files. It turns recommendation 7's first experiment from "a trainer change plus a 4k-step phase" into "an afternoon". It can also interpolate the photo and video checkpoints, or two phases of the same run.
- **Model soups: averaging weights of multiple fine-tuned models improves accuracy without increasing inference time** (M. Wortsman, G. Ilharco, S. Y. Gadre, R. Roelofs, R. Gontijo-Lopes, A. S. Morcos, H. Namkoong, A. Farhadi, Y. Carmon, S. Kornblith, L. Schmidt; ICML 2022; arXiv 2203.05482). Uniform and greedy soups of fine-tuned models from one initialisation lie beyond the WiSE-FT interpolation curve. A ViT-G soup reached **90.94% ImageNet top-1**, then a new state of the art. — [arXiv 2203.05482](https://arxiv.org/abs/2203.05482)
  - *Tintype applicability:* the refresh loop (recommendation 10) and the degradation curriculum (recommendation 1) will produce several sibling checkpoints from the phase-6 weights. Souping them, for example "clean-specialist + degradation-specialist + GAN-replay", is a zero-inference-cost alternative to choosing one, and to running the second opinion as a separate network.
- **R²M: Real-Aware Residual Model Merging for Robust and Generalizable Deepfake Detection** (J. Park, G. Kim, C. Cho, J. Kwon; ECCV 2026; arXiv 2509.24367). CLIP-L/14 specialists on DF40 protocols, mean AUC (in-domain FF / cross-domain CDF / unseen generators): **joint training 0.979 / 0.662 / 0.665; plain weight averaging 0.982 / 0.834 / 0.724; task arithmetic 0.972 / 0.852 / 0.622; R²M 0.992 / 0.901 / 0.818**. Joint training collapsed to 0.177 AUC on Midjourney, where averaging reached 0.613. Averaging suppresses generator-specific fake cues (FS 0.968 vs specialist 0.995). R²M's premise is that *real* representations are shared across specialists while fake cues are generator-specific. — [arXiv 2509.24367](https://arxiv.org/abs/2509.24367); [HTML](https://arxiv.org/html/2509.24367)
  - *Tintype applicability:* direct in-domain evidence that weight averaging beats one jointly trained model on unseen forgeries, the failure mode GAPL calls "Benefit then Conflict". R²M itself needs a forward-backward pass per image for routing, which Tintype's budget rules out. Plain averaging or a WiSE-FT interpolation is the deployable part.

### Effect on the ranking
**Raises recommendation 7's confidence-per-cost and changes its first experiment.** Make "WiSE-FT interpolation toward CF-224, then `finalize`" the cheapest first experiment, and add weight EMA to `src/trainer.rs` as a default (a few lines, zero inference cost). Rank 7 could move up to 5 or 6 if the interpolation recovers GAN recall at matched batch-7 FPR. Recommendation 10 gains a soup option for merging refresh checkpoints.

---

## Gap 5. Visible-watermark and provenance shortcuts in generated video

### Why this is a gap
Note 05 discusses provenance watermarks as a *signal* (SynthID, C2PA), but no note checks whether Tintype's video model learned **visible watermarks or logos** as a shortcut. Its training includes Sora, Veo 3.1 (Rapidata) and Kling clips (`docs/DATA.md:85, 109, 112`).

### Cited findings
- **RobustSora: De-Watermarked Benchmark for Robust AI-Generated Video Detection** (Z. Wang, X. Liu, L. Sun; arXiv 2512.10248, v2 May 2026). 6,500 verified videos in four groups: Authentic-Clean, Generated-Watermarked, Generated-DeWatermarked and Authentic-Spoofed (a Sora watermark overlaid on real video). Ten detectors were tested. Removing or spoofing the watermark shifts accuracy by **−9.4 to +1.6 pp (mean 6.6 pp)**, significant (p < 0.01) for **7 of 10 models** on each task. For **Sora 2 the drop is −11 to −14 pp**, against −3 to −6 pp for Pika and Open-Sora 2. A placebo control bounds inpainting confounds at ≤2 pp, and watermark-aware augmentation recovers 3–4 pp. — [arXiv 2512.10248](https://arxiv.org/abs/2512.10248)
  - *Tintype applicability:* a cheap, high-value audit for recommendation 6. (1) Overlay Sora, Veo and Kling visible marks on about 300 held-out real clips and measure the FPR change at the shipped video threshold. (2) Crop or inpaint the marks off generated test clips and measure the recall change. Tintype's texture-ranked crops may select a logo corner, which makes the shortcut more likely than for a centre-crop model. If either shift exceeds a few points, add watermark-aware augmentation (random logo overlays on both classes) to the next video fine-tune.
- **RA-Bench: Can We Defend Against AI-Generated Video Attacks on Real-World Crisis Events?** (S. Liang et al., 36 authors; arXiv 2608.14391, Aug 2026) **[abs]**. 17,886 videos: 1,830 real anchors across 10 social-risk categories and 16,056 clips from 4 open and 5 closed generators. None of the seven traditional detectors, ten zero-shot multimodal models or two fine-tuned MLLMs generalises consistently, and social dissemination makes detection harder. — [arXiv 2608.14391](https://arxiv.org/abs/2608.14391)
  - *Tintype applicability:* a content-matched, crisis-event video test set that fills the gap the draft notes in recommendation 6 (the 20-clip Davodi cohort cannot resolve changes below 5 points). Evaluation only.

### Effect on the ranking
**Folds into recommendation 6** as a new cheapest-first experiment: the watermark spoof and removal audit. It matters for precision too, because a spoofed watermark on real news footage is a plausible adversarial false-positive attack.

---

## Gap 6. Label noise: AI images inside the "real" training sets

### Why this is a gap
Tintype's real stores include post-2022 social-media reals (So-Fake-Set's 3.0k social reals, Reddit, Bluesky evaluation reals) and web-scraped sets. No note asks how many "reals" are in fact AI-generated, or how that biases the real-quantile calibration.

### Cited findings
- **Will Large-scale Generative Models Corrupt Future Datasets?** (R. Hataya, H. Bao, H. Arai; ICCV 2023; arXiv 2211.08095). The authors simulate contamination by swapping about three million Stable Diffusion images into ImageNet- and COCO-scale sets. Generated images degrade downstream classification and generation. The size of the effect depends on the task and on the contamination share. — [arXiv 2211.08095](https://arxiv.org/abs/2211.08095)
  - *Tintype applicability:* AI images labelled real do two kinds of harm. In training they teach "this generator is real". In calibration they inflate the measured FPR, because the threshold rises to clear "reals" that are fakes. A cheap audit: score every real store with Tintype and CF-384, hand-inspect the top 0.5% of reals by both scores, and record a contamination estimate per source. Drop flagged items from calibration pools. This belongs in recommendation 4's audit step.

### Effect on the ranking
**Adds one step to recommendation 4** (audit), and a caveat to recommendation 2. Per-stratum calibration on contaminated real pools will mis-set thresholds most in the social-media strata, where contamination is likeliest.

---

## Gap 7. Self-supervised, real-only forensic pretraining

### Why this is a gap
Note 01 covers ZED and other zero-shot coding-cost methods, but not the CVPR 2025 real-only *forensic self-description* line, which reports the best zero-shot numbers and doubles as open-set attribution.

### Cited findings
- **Forensic Self-Descriptions Are All You Need for Zero-Shot Detection, Open-Set Source Attribution, and Clustering of AI-generated Images** (T. D. Nguyen, A. Azizpour, M. C. Stamm; CVPR 2025, pp. 3040–3050; arXiv 2503.21003). It learns predictive filters on *real images only*, then models the multi-scale residuals of each image as a compact "self-description". Zero-shot average AUC over 96 real-vs-synthetic pairs (24 generators; COCO17, IN-1k, IN-22k, MIDB reals): **0.960, vs NPR 0.926, UFD 0.798, ZED 0.723, AEROBLADE 0.674**. Worst case is 0.714 on IN-22k. Average AUC under JPEG quality 50–100 is **0.972**, vs 0.986 uncompressed. Open-set attribution: 0.964 known-source accuracy and 0.913 AU-OSCR. Cost: per-image filter fitting of up to 10,000 iterations. — [arXiv HTML 2503.21003v1](https://arxiv.org/html/2503.21003v1); [CVPR poster](https://cvpr.thecvf.com/virtual/2025/poster/33420)
  - *Tintype applicability:* the per-image fitting cost rules it out at runtime. Two parts transfer. (1) Its JPEG stability (0.986 → 0.972) beats every trained detector the notes cite, which suggests residual-based self-supervised features as an *auxiliary pretraining target* for recommendation 1's degradation phase. (2) It is a candidate internal tool for clustering Tintype's false negatives by unknown source, to choose which generators recommendation 10 should add. Neither is a top-10 change.

---

## Gap 8. Certified robustness

### Why this is a gap
The draft rejects adversarial training because open weights make Tintype white-box. It does not consider certification, the only route to a provable guarantee.

### Cited findings
- **Certifiably Robust Synthetic Image Detectors** (K. Zeng, M. Barni, B. Tondi; EUSIPCO 2025, pp. 815–819; DOI 10.23919/eusipco63237.2025.11226152) **[snip]**. Randomized smoothing applied to synthetic-image detectors on StyleGAN2 and latent-diffusion images. Pre-processing, input size and especially architecture strongly affect the certificate. Smoothed detectors stay robust to a wide range of processing beyond what the certificate covers. — [Siena repository](https://usiena-air.unisi.it/handle/11365/1315942)
- **Certifiably Robust Image Watermark** (Z. Jiang, M. Guo, Y. Hu, J. Jia, N. Z. Gong; ECCV 2024; arXiv 2407.04086) **[abs]**. The first watermark-based AI-image detector with certified robustness to removal and forgery, built by randomized smoothing (smoothed HiDDeN). Smoothing also improves empirical robustness. — [arXiv 2407.04086](https://arxiv.org/abs/2407.04086)
- **Certified Adversarial Robustness via Randomized Smoothing** (J. Cohen, E. Rosenfeld, J. Z. Kolter; ICML 2019; arXiv 1902.02918) **[ctx]**. The base technique: a Gaussian-noise-smoothed classifier has a certified ℓ2 radius. — [arXiv 1902.02918](https://arxiv.org/abs/1902.02918)
  - *Tintype applicability (all three):* certification needs on the order of 100–10,000 noisy forward passes per input. That is incompatible with Tintype's 2 ms-per-photo budget, and Gaussian noise destroys exactly the faint traces Tintype reads. **Keep it rejected.** Add one line to the draft's rejected-ideas table, so readers know certification was considered. Zeng et al.'s observation that smoothed detectors resist ordinary processing is a weak argument for Gaussian-noise augmentation, already in recommendation 1.

---

## Gap 9. Explanations for end users

### Cited findings
- **AI-Generated Images: What Humans and Machines See When They Look at the Same Image** (S. Poletti, J. Ilyes, M. Hasenbalg, D. Fischinger, M. Boyer; arXiv 2605.06143, May 2026; Springer CCIS) **[abs]**. 16 XAI methods integrated into AI-image detectors and evaluated against a 100-participant survey of where humans see generation cues. The abstract does not name a winning method. — [arXiv 2605.06143](https://arxiv.org/abs/2605.06143)
  - *Tintype applicability:* the closest primary study of heatmaps for AI-image detection users. A CHI 2026 workshop study found by search (Zenodo record 19700249; 381 participants; not academic-counted here) reports that visual explanations raised trust and accuracy but *not appropriate reliance*. Users over-relied on wrong AI advice when shown explanations. Tintype already exposes per-view scores. A heatmap UI would add persuasion without evidence of better decisions, so **do not add it to the top 10**. Reporting which views drove the verdict (fit vs native crops) is a cheaper, honest alternative.

---

## Gap 10. An influential benchmark missing from the bibliography

- **GenImage: A Million-Scale Benchmark for Detecting AI-Generated Image** (M. Zhu, H. Chen, et al.; NeurIPS 2023 Datasets & Benchmarks; arXiv 2306.08571). About 1.3M fakes paired with ImageNet reals over eight generators (Midjourney, SD 1.4/1.5, ADM, GLIDE, Wukong, VQDM, BigGAN). It defines cross-generator and degraded-image (low resolution, JPEG, blur) protocols. — [arXiv 2306.08571](https://arxiv.org/html/2306.08571v2); [NeurIPS](https://neurips.cc/virtual/2023/poster/73644)
  - *Tintype applicability:* Tintype trains on `tiny-genimage` (14k ImageNet reals vs 14k fakes from 7 GenImage generators, `docs/DATA.md:142`). GenImage's reals are JPEG ImageNet photos and its fakes are mostly PNG, the very bias "Fake or JPEG?" (already cited) measured. Recommendation 4's per-store format audit should start with this store. It is also the most-cited benchmark in the field, and OMAT's GenImage++ extends it to FLUX and SD3.

---

## Gaps checked and found adequately covered (no new papers added)
- **Watermark + passive fusion.** Covered by SynthID-Image (2510.09263), "Passive or Watermark?" (2411.13553), UnMarker and RAW in notes 04 and 08, and by Tintype's C2PA reader.
- **Audio-visual generated video.** Covered by MVAD, Video Reality Test (2512.13281), PIVOT and note 05's conclusion. Nothing found changes the rejection.
- **Smartphone computational photography as an FPR source.** Covered by LAION-Mobile (2609.11134) in note 03. Only context added above (2505.07380).
- **Any-resolution ViTs.** SPAI, TextureCrop and the global-local attention paper (2601.00141) are covered. No new primary result changes recommendation 5.
- **Synthetic-data generation at scale.** Community Forensics, ScaleDF, OpenFake, So-Fake, DDA and B-Free are covered. OMAT and the final-component paper above are the new additions.
- **Active learning / human-in-the-loop for AI-image detection.** Two searches found **no primary paper** applying active learning to AIGI detection. That is a true literature gap, not a missed paper. The closest practice is Pantsios et al.'s fact-check mining (already cited). For Tintype, uncertainty-band sampling from `tintype batch` logs into the refresh loop (recommendation 10) is an untested engineering idea.

---

## Proposed changes to the draft's top 10
1. **Recommendation 4: widen it and raise its confidence.** Add VQ/BSQ-tokenizer and learned-upsampler reconstructions of reals (final-component paper: 98.83% over 22 unseen sets from 100 samples per category), and add open AR generators (ARForensics: LlamaGen, VAR, Infinity, Janus-Pro, RAR, Switti, Open-MAGVIT2). Target Tintype's weakest clean generators (FLUX.1 pro 0.32, Seedream 3 0.52, GPT-image-2 0.58, Nano Banana 0.59, Recraft v3 0.60). Add a real-store contamination audit (Hataya et al.), and start the format audit with `tiny-genimage` (GenImage bias). Extend the labelling policy to AI-upscaled and AI-edited reals (INP-X, Coccomini et al.).
2. **Recommendation 7: new zero-training first experiment, and possibly rank up to 5–6.** Run a WiSE-FT interpolation between Tintype and CF-224 weights, then `finalize`. Add weight EMA to the trainer. Evidence: WiSE-FT +4–6 pp under shift; R²M shows weight averaging beating joint training on unseen forgeries (0.724 vs 0.665) and cross-dataset (0.834 vs 0.662).
3. **Recommendation 6: add a visible-watermark spoof and removal audit** (RobustSora: −11 to −14 pp for Sora 2; 7/10 detectors significantly affected; augmentation recovers 3–4 pp). Add RA-Bench as a larger content-matched real/fake test.
4. **Recommendation 1: extend the per-operation shortcut audit** with learned super-resolution, a whole-image VAE round trip and partial (INP-X) VAE passes.
5. **Recommendation 5: add a text-density condition.** Texture-ranked crops favour text, the TextFake "Text Density Curse". Evaluate on TextFake, AIGDoc and TextRich, and add a README Limits line for documents and screenshots until measured.
6. **Recommendation 10: add a family-level chronological hold-out** (all AR/unified models held out together), and use soups to merge refresh checkpoints.
7. **Rejected-ideas table: add** PRADA and D³QE as runtime detectors (white-box or slower, and beaten by broad training), certified robustness (randomized smoothing: 100+ passes, and noise destroys traces), forensic self-descriptions at runtime (10k-iteration per-image fit), and user-facing heatmaps (no evidence of better reliance).
