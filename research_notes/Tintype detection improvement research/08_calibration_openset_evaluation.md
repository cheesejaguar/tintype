# Calibration, open-set / unseen-generator detection, test-time adaptation, attribution, and evaluation methodology for AI-generated image/video detection: what could improve Tintype

Scope note. Tintype is a ViT-S/16 binary detector. Its thresholds are calibrated on held-out real media at 0.5% / 1.5% / 5% FPR, averaged over real sources. Its main known weaknesses, from the repo (README "Results", "Limits"; docs/EVIDENCE.md), are these:
- The calibrated FPR does not transfer to shifted reals. On the NTIRE 2026 distorted public test, 155/600 authentic images (25.83%) were flagged at the shipped cutoff, and 3/20 authentic content-matched video clips were flagged.
- Recall at 1% FPR collapses under distortion: 2.2% on NTIRE distorted versus 22.9% on clean.
- Recall can decay on generators released after the Nov 2026 cutoff.

The repo already contains one decisive in-house data point. When the cutoff was re-fit on NTIRE *hard-validation* distorted reals and applied to the *public-test* distorted cohort, FPR fell from 25.83% to 2.17% (13/600), but recall fell to 4.77% (docs/EVIDENCE.md, "NTIRE threshold transfer"). This finding is used below as the anchor for several inferences.

Papers covered (29 primary sources, 2022–2026, most from 2024–2026) are listed under the key question they answer. Each has a full citation, key numbers and a "Tintype applicability" note.

---

## Q1. Calibration and threshold transfer under shift; conformal FPR guarantees; domain-conditional thresholds; selective prediction / abstention

### Takeaway
Under distribution shift the main failure is that the operating threshold moves, more than that the ranking breaks. A one-parameter logit-bias re-fit recovers 5–16 accuracy points across nine published detectors. Tintype's own NTIRE experiment shows the same effect for reals: in-domain recalibration took FPR from 26% to 2%.

The confidence scores used to build an "unsure" band are themselves unreliable under shift. Max-softmax and max-logit confidence are at or below random under blur and FGSM. Abstention therefore needs its own validated uncertainty signal.

No paper was found that gives per-source conformal FPR guarantees for a *passive* image detector. The closest are RAW, which does this for watermark detection, and text-detector work.

### Cited Findings

**1. Yang, Goenawan, Wang, Qin, Xu, Yang, Fang, Sun, Lim, Zhu. "Your AI-Generated Image Detector Can Secretly Achieve SOTA Accuracy, If Calibrated." AAAI 2026. arXiv:2602.01973 (Feb 2026).**
- Method. Bayesian decision theory shows the default threshold of 0 is not Bayes-optimal under label-prior shift and fake-class-conditional shift. The fix is a single learned scalar bias α added to the logit; the backbone stays frozen.
  - Supervised variant: class-wise Gaussian KDEs on a small labelled target set. The default is 100 images (about 1% of test), and results are stable down to about 10.
  - Unsupervised variant: picks the α where the unlabelled logit density is symmetric (bimodality assumption). — [arXiv HTML](https://arxiv.org/html/2602.01973)
- Numbers (mean accuracy, before → supervised / unsupervised). — [arXiv HTML](https://arxiv.org/html/2602.01973)
  - AIGCDetectBenchmark: RINE 86.77 → 94.56 / 93.79; UnivFD 78.43 → 86.35 / 86.20; Effort 79.35 → 89.48 / 85.08.
  - GenImage: RINE 81.78 → 97.94 / 97.59.
  - AIDE under JPEG QF90: 60.22 → 75.61.
  - A few cases get slightly worse, e.g. LGrad under blur (−0.50).
- The authors assume the *real* distribution stays roughly stable and the shift is in the fakes. — [arXiv HTML](https://arxiv.org/html/2602.01973)
- **Tintype applicability.**
  - The assumption that reals stay stable is exactly what fails for Tintype: NTIRE distorted reals shift *up*. Do not adopt the unsupervised symmetric-density variant blindly. In a ~99%-real deployment stream the logit density is not bimodal, so the symmetry point would be badly placed.
  - The supervised variant, though, is the same thing Tintype's `calibrate` already does, and it shows that a few hundred labelled target-domain samples suffice.
  - Concrete change: expose `tintype calibrate --reference-real <folder>` for operators to re-fit the logit cutoff on 300–1,000 known-real items from their own pipeline, and optionally a small labelled fake set to choose a bias rather than only an FPR quantile.
  - Expected effect: restores the nominal FPR in-domain. NTIRE in-house evidence is 25.83% → 2.17%, so precision improves, at the cost of recall in hard domains (44.8% → 4.77%).
  - Cost: negligible compute; operator effort to collect reals.
  - Risk: operators calibrate on contaminated "real" folders. Mitigate with a contamination check (flag items above the shipped cutoff for review before fitting).

**2. Zheng & Seidman (Univ. Waterloo / Reality Defender). "Post-hoc Selective Classification for Reliable Synthetic Image Detection" (ReSIDe). NeurIPS 2026 poster. arXiv:2605.08574 (May 2026).**
- Setup: ResNet50 and Swin-T trained on GenImage (ADM + BigGAN), tested under blur, low resolution, JPEG, FGSM, unseen classes and unseen generators (Wukong, Midjourney, SD1.4/1.5). — [arXiv HTML](https://arxiv.org/html/2605.08574)
- Logit-based confidence functions (MSP, max-logit, etc.) can be *worse than random* for abstention under shift. Under blur, ResNet50 MSP has AURC 0.7276 against an error rate of about 0.495; a random confidence score has AURC equal to the error rate. — [arXiv HTML](https://arxiv.org/html/2605.08574)
- ReSIDe combines unsupervised linear probes on intermediate layers, with weights tuned by optimising an AURC upper bound on about 1,000 labelled validation images per shift. The validation shifts and generators (GLIDE, VQDM) differ from the test shifts. — [arXiv HTML](https://arxiv.org/html/2605.08574)
  - Blur AURC: 0.7271 → 0.2214, a 69.55% reduction.
  - Mixture of all shifts: 0.3563 → 0.2004.
  - No gain on JPEG.
  - About 5% extra latency and tens of parameters.
- **Tintype applicability.**
  - Tintype exposes only a single score per profile and has no "unsure" verdict except for decoding failures (`INCONCLUSIVE`). This paper warns that an "unsure" band built as |logit − threshold| < δ may be close to useless exactly on shifted inputs like NTIRE distorted images.
  - Concrete change: add an `UNSURE` verdict driven by a combined uncertainty score:
    - intermediate-layer probe agreement (ReSIDe-style), plus
    - disagreement among Tintype's up to 6 views (crops/scales), which it already computes.
  - Tune it on held-out *shifted* sets that are disjoint from the test shifts, e.g. tune on NTIRE hard-validation distorted and evaluate on the public test.
  - Report risk–coverage curves (AURC, and precision/recall at 90% coverage) in BENCHMARK.md.
  - Expected effect: higher precision on the items that do get a verdict. The cost is a fraction routed to `--photo-second-opinion` or a human.
  - Cost: low (a few linear probes).
  - Risk: abstention can hide failure. Always report coverage alongside precision, and pre-register the abstention rule.

**3. "Uncertainty-Aware Deepfake Detection via Multi-View Structural Learning" (DISCERN). arXiv:2607.28769 (Jul 2026).**
- Reports lower calibration error and a better abstention signal than softmax confidence. At a 10% abstention budget on Celeb-DF v2, AUC rises from 95.72 to 97.94, against +0.69 for GenD's softmax confidence. — [search summary of arXiv:2607.28769](https://arxiv.org/pdf/2607.28769) (full text not read; face-deepfake domain)
- **Tintype applicability.** This supports using multi-view consistency, which Tintype already has (6 views), as the abstention signal instead of a calibrated probability. It is evidence that "unsure" bands can buy measurable reliability for about 10% coverage loss. Not verified on full-frame AI images.

**4. Xian, Wang, …, Ding. "RAW: A Robust and Agile Plug-and-Play Watermark Framework for AI-Generated Images with Provable Guarantees." NeurIPS 2024. arXiv:2403.18774.**
- Uses conformal prediction to give distribution-free FPR guarantees for watermark detection. The guarantee holds only while test transformations stay within the predefined range covered by the calibration data. — [NeurIPS PDF](https://proceedings.neurips.cc/paper_files/paper/2024/file/ee62ab636066cf45a27246acca9545b7-Paper-Conference.pdf); [arXiv](https://arxiv.org/html/2403.18774v1)
- **Tintype applicability.** Tintype's real-quantile threshold is already a split-conformal quantile. RAW's lesson is the *conditional* part: the FPR guarantee holds only for the transformation family represented in calibration. That is exactly why NTIRE distorted reals broke Tintype's 1.5%.
  - Concrete change, using Mondrian / group-conditional split conformal:
    - Stratify the real calibration pool by observable covariates: estimated JPEG quality from quantisation tables, resolution bucket, a noise-level estimate, and source type (camera / screenshot / rendered / video frame).
    - Set one threshold per stratum with the finite-sample rank (⌈(n+1)(1−α)⌉).
    - Publish per-stratum FPR with Clopper–Pearson upper bounds.
  - Expected effect: per-stratum FPR guarantees, and a visible "no calibration data for this stratum" state that can map to `UNSURE`.
  - Cost: needs about 1/α × 10 reals per stratum for tight bounds, i.e. about 2,000 per stratum at 0.5% FPR.
  - Risk: strata that correlate with fakes (e.g. PNG, high resolution) give fakes a "free" lenient threshold. Thresholds must be per stratum on reals only, and recall must be reported per stratum.

**5. Angelopoulos & Bates. "A Gentle Introduction to Conformal Prediction and Distribution-Free Uncertainty Quantification." arXiv:2107.07511 (2021).**
- Standard reference for split conformal, class- and group-conditional (Mondrian) coverage, and conformal risk control. Its guarantees assume exchangeability between calibration and test data. — [arXiv](https://arxiv.org/abs/2107.07511)
- **Tintype applicability.** This is the formal basis for item 4. It is also the formal reason that the shipped "averaged over real sources" FPR is *not* a guarantee for any single source. Averaging over sources gives marginal coverage over the mixture, not per-source coverage. Change PREREGISTRATION/BENCHMARK reporting to state "max over sources" FPR alongside the average.

**6. Zhu et al. Conformal FPR control for machine-generated *text* detectors (ACL 2025; arXiv:2505.05084).**
- Plain conformal prediction bounds FPR but lets some machine text evade detection. Their multi-level conformal (MCP) variant keeps the FPR bound while recovering power. — [arXiv](https://arxiv.org/abs/2505.05084v1); [ACL Anthology](https://preview.aclanthology.org/setup/2025.acl-long.601)
- **Tintype applicability.** This is direct precedent that per-group conformal thresholds trade recall for an FPR guarantee, the same trade seen on NTIRE. It suggests separate (multi-level) calibration per difficulty stratum instead of one global quantile. Text-domain only.

### Inferences
- Tintype's FPR problem is mainly a **threshold-transfer** problem, not a ranking problem. Evidence:
  - On NTIRE distorted, in-domain recalibration restored FPR to 2.17%.
  - AUROC was 0.636, so the ranking is poor too, but FPR was fixable.
  - The AAAI-26 paper finds the same in nine detectors.
- The cheapest high-value change is to stop presenting one global FPR as transferable. Instead, Tintype should:
  - publish per-stratum (JPEG quality × resolution × source type) FPRs with upper confidence bounds,
  - ship a supported recalibration path for operators, and
  - return `UNSURE` when an input falls in a stratum with no or sparse calibration data.
- Abstention must be validated with risk–coverage curves on *shifted* data. The logit margin is not a safe uncertainty signal (ReSIDe).

### Gaps
- No paper found that applies conformal or Mondrian FPR guarantees to a *passive* AI-image detector with per-source reporting. This looks like an open niche Tintype could fill.
- No paper found that measures FPR transfer specifically under JPEG-quality-conditional or resolution-conditional thresholds for AI-image detectors. The evidence comes by analogy from RAW and conformal theory.
- DISCERN's numbers come from a search summary; the full text was not read.

---

## Q2. Open-set and unseen-generator detection: real-only / one-class modelling, zero-shot, few-shot, novelty detection

### Takeaway
Two families exist:
- **Real-only / zero-shot** detectors (ZED, SPAI, manifold-bias). They model real images or use a generative model's own likelihood, and need no fakes from new generators.
- **Few-shot** detectors (FSD), which adapt to a new generator from about 10 samples.

Both report gains on unseen generators. For Tintype the main risk is the opposite of its strength: real-only detectors treat any out-of-distribution *real* (heavy distortion, rendered content) as anomalous. That is precisely Tintype's existing FPR failure. They are better used as an `UNSURE` / second-opinion trigger than as a flag.

### Cited Findings

**7. Cozzolino, Poggi, Nießner, Verdoliva. "Zero-Shot Detection of AI-Generated Images" (ZED). ECCV 2024 (oral). arXiv:2409.15875; DOI 10.1007/978-3-031-72649-1_4.**
- A lossless image coder (SReC-based) trained only on real photos predicts each pixel's distribution. Synthetic images show a larger gap between actual and expected coding cost. One feature, no fake training data. — [arXiv](https://arxiv.org/abs/2409.15875); [project](https://grip-unina.github.io/ZED)
- Reports more than 3% average accuracy improvement over the prior state of the art across a range of generators. — [arXiv](https://arxiv.org/abs/2409.15875)
- **Tintype applicability.**
  - Change: run ZED, or a light coding-cost proxy, as a novelty score on items Tintype calls real with low margin. A high surprise score yields `UNSURE: possible unknown generator`.
  - Expected effect: some recall recovery on post-cutoff generators, without retraining.
  - Cost: an extra network; lossless-coder inference is not cheap at full resolution.
  - Risk: coding cost is very sensitive to JPEG and resizing, so on NTIRE-like distorted reals it may fire constantly. Calibrate it per stratum (Q1) and never use it alone to flag.

**8. Karageorgiou, Papadopoulos, Kompatsiaris, Gavves. "Any-Resolution AI-Generated Image Detection by Spectral Learning" (SPAI). CVPR 2025, pp. 18706–18717. arXiv:2411.19417.**
- Learns the spectral distribution of real images through masked frequency reconstruction, self-supervised on reals. Generated images are flagged as out-of-distribution through "spectral reconstruction similarity". "Spectral context attention" handles any resolution. — [CVF](https://openaccess.thecvf.com/content/CVPR2025/html/Karageorgiou_Any-Resolution_AI-Generated_Image_Detection_by_Spectral_Learning_CVPR_2025_paper.html); [arXiv](https://arxiv.org/abs/2411.19417v2)
- Reports +5.5% absolute AUC over the previous state of the art across 13 recent generators, and robustness to common online perturbations. — [arXiv](https://arxiv.org/abs/2411.19417v2)
- **Tintype applicability.**
  - The any-resolution design matches Tintype's own "native-resolution crop" view.
  - Change: add a real-only spectral pretext objective (masked frequency reconstruction on reals) as an auxiliary pre-training stage for the ViT-S. Alternatively, use SPAI's similarity score as an extra input to an ensemble.
  - Expected effect: better recall on unseen generators.
  - Cost: a pre-training stage.
  - Risk: spectral cues are destroyed by the strong distortions in NTIRE, so no help there.

**9. Brokman, Giloni, Hofman, Vainshtein, Kojima, Gilboa. "Manifold Induced Biases for Zero-shot and Few-shot Detection of Generated Images." ICLR 2025. arXiv:2504.15470.**
- Uses score-function analysis of a pre-trained diffusion model to estimate the curvature and gradient of its implicit manifold as zero-shot criteria, with a mixture-of-experts few-shot extension. Evaluated on 20 generators; outperforms prior zero-shot and few-shot methods. — [arXiv](https://arxiv.org/abs/2504.15470); [ICLR](https://iclr.cc/virtual/2025/poster/30810)
- **Tintype applicability.** Needs a diffusion model at inference, which is incompatible with Tintype's lightweight, CPU-capable design. Useful only as an offline audit tool, e.g. to triage post-cutoff samples for retraining. Low priority.

**10. Wu, Liu, Li, Wang. "Few-Shot Learner Generalizes Across AI-Generated Image Detection" (FSD). ICML 2025 (PMLR v267). arXiv:2501.08763.**
- A prototypical network: support images per class define prototypes, and test images go to the nearest one. The v1 abstract reports +11.6% average accuracy on GenImage with 10 extra samples; the v2 abstract reports +7.4%. — [arXiv](https://arxiv.org/abs/2501.08763v2); [PMLR](https://proceedings.mlr.press/v267/wu25r.html)
- **Tintype applicability.** This is the most practical response to "performance may drop after the Nov 2026 cutoff".
  - Change: keep a prototype bank, or a cheap last-layer refit, in Tintype's embedding space. When a new generator appears, 10–100 of its images become a new "fake" prototype. A `tintype adapt --fake-examples <dir>` command would write a delta file without full retraining.
  - Expected effect: fast recall recovery on a named new generator.
  - Cost: small.
  - Risks:
    - Over-fitting to the content of 10 samples.
    - Thresholds must be recalibrated on reals afterwards, or FPR drifts.
    - Pre-register the evaluation, since gains on the support generator are not gains on the next one.

**11. Sundara Rajan & Lee. "Stay-Positive: A Case for Ignoring Real Image Features in Fake Image Detection." ICML 2025 (PMLR v267, pp. 58008–58028). arXiv:2502.07778.**
- Constrains the final layer so features associated with real images cannot push a decision toward "real". An image is real when fake traces are *absent*. Only the last layer is retrained; whole-network constrained training hurt accuracy on reals. Reports better generalisation, robustness to post-processing, and detection of partially inpainted reals. — [arXiv](https://arxiv.org/abs/2502.07778v1); [PMLR](https://proceedings.mlr.press/v267/sundara-rajan25a.html)
- **Tintype applicability.**
  - Change: retrain Tintype's linear head with non-negative weights on "fake-indicative" features, a cheap last-layer refit.
  - Expected effect:
    - Fakes cannot be "rescued" by real-looking cues (e.g. camera noise pasted on), which may raise recall on messenger-compressed fakes and partially inpainted images (currently out of scope: 14% on face swaps).
    - FPR effect is uncertain. If distortions (noise, watermark overlays) were learned as fake features, Stay-Positive will not fix NTIRE FPR and may worsen it.
  - Cost: hours.
  - Risk: must be re-validated on the 30 blind photo sets and NTIRE before shipping.

**12. Sundara Rajan, Ojha, Schloesser, Lee. "Aligned Datasets Improve Detection of Latent Diffusion-Generated Images." ICLR 2025. arXiv:2410.11835.**
- Builds fakes by passing real images through an LDM autoencoder (no denoising), giving fakes that match reals in size, aspect ratio and content. The detector learns decoder artifacts. — [arXiv](https://arxiv.org/abs/2410.11835); [ICLR PDF](https://proceedings.iclr.cc/paper_files/paper/2025/file/9ead108421b202494d01b5060d12aa34-Paper-Conference.pdf)
- Generalises to later SD versions, Playground, Kandinsky, PixArt-α, LCM and Midjourney, and is less prone to exploiting resizing artifacts. Stated limit: very different architectures, such as pixel-space diffusion, confuse it. — [arXiv](https://arxiv.org/abs/2410.11835)
- **Tintype applicability.**
  - Change: add VAE reconstructions of Tintype's *real* training images (SD/FLUX/SDXL VAEs) as extra fakes. Apply the same distortion augmentations to each real/reconstruction pair.
  - Expected effect: removes content and format shortcuts. That should cut false positives on reals with unusual content and help with new latent-diffusion generators.
  - Cost: one VAE pass over the real set.
  - Risk: little benefit on autoregressive or pixel-space generators (GPT-image-class models).

**13. Guillaro et al. (Univ. Naples Federico II and Google DeepMind). "A Bias-Free Training Paradigm for More General AI-generated Image Detection" (B-Free). CVPR 2025, pp. 18685–18694. arXiv:2412.17671.**
- Self-conditioned reconstruction makes content-matched fakes from reals, plus inpainted variants, so the detector learns generator traces rather than dataset bias. — [CVF PDF](https://openaccess.thecvf.com/content/CVPR2025/papers/Guillaro_A_Bias-Free_Training_Paradigm_for_More_General_AI-generated_Image_Detection_CVPR_2025_paper.pdf); [project](https://grip-unina.github.io/B-Free)
- Evaluated on 27 generators, including FLUX and SD 3.5. On viral images tracked over time, B-Free held about 92% average accuracy days after first posting, while most competitors fell below 70% after about a day. — [project page / slides](https://cvpr.thecvf.com/media/cvpr-2025/Slides/34882.pdf)
- **Tintype applicability.** The viral-image test is the closest analogue to Tintype's WhatsApp/Instagram weakness (27–36% recall). Same change as #12, plus inpainted variants of reals. It also addresses the inpainting limit in README "Limits". Cost: generation compute. Risk: inpainted-as-fake labels could raise FPR on legitimately edited photos; measure on edited-real sets.

**14. "Are CLIP features all you need for Universal Synthetic Image Origin Attribution?" ECCV 2024 workshop. arXiv:2408.09153.** Plus later open-set attribution work:
- **OmniDFA** (arXiv:2509.25682, 2025): open-set detection plus few-shot attribution, with a dataset covering 4,545 generative models.
- **Scalable Black-Box Model Attribution** (arXiv:2608.15652, 2026).
- **Face-trace** (arXiv:2607.07545, 2026): energy-based rejection plus clustering of rejected samples; 71.25% balanced accuracy for rejection.

Findings:
- The CLIP paper trains a classifier that assigns a known model or rejects as unknown-synthetic. — [arXiv 2408.09153](https://arxiv.org/pdf/2408.09153)
- OmniDFA argues plain rejection "lumps every unknown into one bin" and proposes few-shot identification. — [arXiv 2509.25682](https://arxiv.org/html/2509.25682v1)
- When held out, DALL·E 3 and Midjourney 6 largely *evade* rejection and are confidently assigned to a known generator. — [arXiv 2608.15652](https://arxiv.org/pdf/2608.15652) (from search snippet)
- **Tintype applicability.** "Unknown generator" novelty flags are unreliable when the new generator resembles a known one, which is the typical case for successor models. Do not promise an "unknown generator" verdict. Novelty scores are useful internally to prioritise samples for retraining.

### Inferences
- For a post-cutoff generator, the best evidence-backed path is few-shot adaptation (FSD-style) plus real-only recalibration. Zero-shot real-only detectors are attractive but share Tintype's weakness: distorted or atypical reals look anomalous.
- Training-data alignment (Aligned Datasets, B-Free) is the open-set idea with the clearest route to *both* better recall on unseen generators *and* lower FPR. It removes non-generator cues that otherwise fire on unusual reals.
- Real-only and novelty scores should feed an `UNSURE` route, not a `FLAGGED` verdict.

### Gaps
- No found paper reports FPR at 1% or below for real-only detectors on heavily distorted reals (NTIRE-like). Their impact on Tintype's FPR problem is untested.
- SPAI's exact training regime was not verified from the full paper (whether any fakes are used in the final classifier stage).
- The FSD v1 and v2 abstracts give different headline gains (11.6% vs 7.4%).
- The user-suggested titles "Detecting AI-generated images via real-only training" and "Learning on real images only" were not located as exact titles; ZED, SPAI and Aligned Datasets are the closest verified works.

---

## Q3. Test-time adaptation / test-time training for detectors

### Takeaway
Transductive TTA on large *offline* batches can raise balanced accuracy a lot on unseen generators: TTC goes 62.4 → 91.1 on Chameleon. Generic entropy-based TTA (TENT, SAR, EATA) falls *below* the unadapted detector.

TTC needs pseudo-label accuracy of about 90% or more, class-balanced batches, and roughly GPU-hours per 10k images. It was not tested on real-heavy or streaming data, which is Tintype's deployment regime. Treat it as an optional batch-audit mode, not a default.

### Cited Findings

**15. Zhang, Gu, Hao, Zhang, Mao, Hu, Miao, Yu, Zhang, Wang, Sun. "Test-Time Curriculum for Open-Set AIGC Detection" (TTC). arXiv:2608.00559 (Aug 2026), preprint.**
- Method: curriculum self-training on unlabelled test data. It starts from reliable pseudo-labels with class-balanced selection, refines pseudo-labels across scales, and uses noisy-or fusion. It introduces AIGCGuard (3,100 real / 124,000 generated, 40 T2I models). — [arXiv abs](https://arxiv.org/abs/2608.00559); [HTML](https://arxiv.org/html/2608.00559)
- Balanced accuracy (base → DDA → TTC). — [HTML](https://arxiv.org/html/2608.00559)

  | Benchmark | Base | DDA | TTC |
  |---|---|---|---|
  | AIGCDetect | 81.6 | 87.8 | 93.0 |
  | GenImage | 80.1 | 91.7 | 92.5 |
  | Chameleon | 62.4 | 82.4 | 91.1 |
  | AIGCGuard | 60.6 | 77.2 | 78.4 |

- On AIGCDetect, TENT, SAR and EATA fall to or below the base detector; CoTTA reaches 83.8 and T3A 84.4. — [HTML](https://arxiv.org/html/2608.00559)
- Stated limits — [HTML](https://arxiv.org/html/2608.00559):
  - Offline and transductive only; streaming adaptation is untested.
  - Not tested on all-real or highly imbalanced streams; a round is skipped if a class is empty.
  - Pseudo-label accuracy below about 90% degrades results sharply (79.3% balanced accuracy at 80% label accuracy).
  - Negative transfer on some subsets (GauGAN, SD1.4/1.5, Wukong).
  - The noisy-or score is not calibrated.
  - Cost: 0.79 H800 GPU-hours per 10k images.
- **Tintype applicability.**
  - Change: an optional `tintype batch --adapt` for a large, homogeneous corpus, such as a known-mixed investigative dataset, with TTC's safety fallback. Re-run calibration on reals afterwards; adapted scores must not be reported under the shipped FPR.
  - Expected effect: higher recall on corpora dominated by one unseen generator.
  - Cost: hours on Apple GPU per 10k images.
  - Risks:
    - In moderation streams with about 1% AI, class balancing is starved and pseudo-labels are dominated by reals.
    - Adaptation on distorted reals could learn distortions as "real" and then miss distorted fakes, or the reverse.
    - Pre-registered blind batches would no longer be comparable.

**16. "Towards Generalizable AI-Generated Image Detection via Image-Adaptive Prompt Learning" (IAPL). CVPR 2026. arXiv:2508.01603.**
- Single-image test-time adaptation: adaptive tokens are tuned per test image by minimising entropy across generated views. The authors note the added inference latency. — [arXiv](https://arxiv.org/html/2508.01603v3); [CVPR 2026](https://cvpr.thecvf.com/virtual/2026/poster/38405)
- **Tintype applicability.** Per-image TTA needs a backward pass per image, which is incompatible with Tintype's throughput goals (MPSGraph and CPU engines). Entropy-minimising per-image adaptation also pushes ambiguous reals toward confident decisions, which risks FPR. Low priority. If tried, use only in `--all-views` forensic mode.

**17. Nguyen-Le, Tran, Nguyen, Le-Khac. "Think Twice before Adaptation: Improving Adaptability of DeepFake Detection via Online Test-Time Adaptation" (T2A). IJCAI 2025. arXiv:2505.18787; DOI 10.24963/ijcai.2025/854.**
- Online TTA without source data or labels. An uncertainty-aware negative-learning objective replaces entropy minimisation, with uncertain-sample prioritisation and gradient masking. Motivation: unknown post-processing applied by adversaries at inference. Reports state of the art among TTA methods (face deepfakes). — [arXiv](https://arxiv.org/abs/2505.18787v1); [IJCAI](https://www.ijcai.org/proceedings/2025/0854)
- **Tintype applicability.** Negative learning ("this is not class X") is safer than entropy minimisation for a ~99%-real stream. Possible future use for online adaptation to a fixed post-processing pipeline, e.g. one platform's recompression. Not validated on full-image AI detection. Low to medium priority.

**18. Chen et al. "OST: Improving Generalization of DeepFake Detection via One-Shot Test-Time Training." NeurIPS 2022.**
- Meta-learning makes a single test-time gradient step on a test-sample-specific auxiliary task effective. — [NeurIPS](https://proceedings.neurips.cc/paper_files/paper/2022/hash/9bf0810a4a1597a36d27ceea58667d92-Abstract.html)
- **Tintype applicability.** Historical baseline only. Requires meta-training and a per-image backward pass.

### Inferences
- The safest TTA-like gain for Tintype is the *degenerate* form: recalibrate the threshold or bias on target reals (Q1, AAAI-26). It gets part of the benefit without changing weights.
- Full TTA conflicts with pre-registration. Any adapted model is a new model and needs a new blind batch.
- Generic TTA (TENT-family) is reported to *hurt* AI-image detectors, so do not add LayerNorm-statistic adaptation by default.

### Gaps
- No paper found that tests TTA on imbalanced (≤5% fake) streams or reports FPR at ≤1% after adaptation. These are Tintype's two key deployment conditions.
- No found TTA study for AI-*video* detectors.
- Numeric gains for T2A and IAPL were not extracted.

---

## Q4. Source / model attribution and fingerprinting: does a multi-class attribution auxiliary loss help binary detection?

### Takeaway
Attribution and fingerprinting work is active, but no controlled ablation was found that isolates "binary vs binary + generator-ID auxiliary loss" for generalisation to unseen generators. The claims that attribution helps detection (OmniDFA, supervised contrastive few-shot work) are indirect.

Proactive fingerprinting (WOUAF) needs generator cooperation and is a provenance tool, not a passive detector.

### Cited Findings

**19. "Supervised Contrastive Learning for Few-Shot AI-Generated Image Detection and Attribution." arXiv:2511.16541 (2025).**
- Supervised contrastive loss over generator labels. Reports 91.3% average detection accuracy, 5.2 points above prior approaches, and open-set attribution gains of +14.70% AUC and +4.27% OSCR. — [arXiv](https://arxiv.org/pdf/2511.16541) (from search snippet)
- **Tintype applicability.**
  - Change: add a generator-ID supervised-contrastive auxiliary head during fine-tuning, using generator names already in Tintype's manifests.
  - Expected effect: possibly better-separated fake clusters and a natural embedding space for FSD-style prototypes (#10). The effect on binary recall at 1% FPR is unproven.
  - Cost: low (labels exist).
  - Risk: the head could encourage memorising generator-specific features at the cost of generality. Ablate on the blind batches.

**20. OmniDFA. "A Unified Framework for Open Set Synthesis Image Detection and Few-Shot Attribution." arXiv:2509.25682 (2025).**
- Argues that attribution's fine-grained discrimination is more robust than binary detection, which over-fits specific forgery traits. Reports state-of-the-art generalisation for detection and introduces a dataset over 4,545 generative models. — [arXiv](https://arxiv.org/html/2509.25682v1)
- **Tintype applicability.** Same as #19. The 4,545-model dataset could serve as extra training diversity, in line with Community Forensics' finding that the number of generators matters most (docs/RESEARCH.md).

**21. Kim, Min, Patel, Cheng, Yang. "WOUAF: Weight Modulation for User Attribution and Fingerprinting in Text-to-Image Diffusion Models." CVPR 2024, pp. 8974–8983. arXiv:2306.04744.**
- Modulates SD decoder weights with a user fingerprint and recovers the ID with a ResNet-50 decoder. Near-perfect attribution accuracy, and on average 11% better than baselines under post-processing. — [CVF](https://openaccess.thecvf.com/content/CVPR2024/html/Kim_WOUAF_Weight_Modulation_for_User_Attribution_and_Fingerprinting_in_Text-to-Image_CVPR_2024_paper.html); [arXiv](https://arxiv.org/html/2306.04744v3)
- **Tintype applicability.** Applies only to cooperating generator operators, so it is not usable by a passive detector. Out of scope except as context for provenance (Q6).

**22. DRAGON (Bertazzini, Baracchi, Shullani, Echizen, Piva). "DRAGON: A Large-Scale Dataset of Realistic Images Generated by Diffusion Models." arXiv:2505.11257 (May 2025).**
- 25 diffusion models, with a dedicated test set for detection *and* attribution. Reported size differs across sources (270k vs 1.5M images). — [arXiv](https://arxiv.org/pdf/2505.11257)
- **Tintype applicability.** Provides generator-labelled data for an attribution-auxiliary ablation and a per-generator recall table. Cost: download only.

### Inferences
- Treat a generator-ID auxiliary head as a cheap experiment, not an expected win. Its clearest benefit is an *embedding* suited to few-shot updates and to per-generator error analysis.
- Open-set attribution is unreliable for successor generators (#14), so it should not drive user-facing verdicts.

### Gaps
- No controlled binary vs multi-class ablation found for unseen-generator generalisation at low FPR. The suggested "Where did I come from? Origin attribution" (2023) was not verified in this session.
- No found study of whether attribution auxiliaries change FPR on real images.

---

## Q5. Evaluation pitfalls, benchmark design, and in-the-wild gaps; precision at prevalence

### Takeaway
Lab-to-wild gaps are large and consistent. Deepfake-Eval-2024 measured AUC drops of about 45% (image) and 50% (video) for open-source detectors on in-the-wild 2024 content, with no open model above AUC 0.58. Dataset format biases (JPEG vs PNG, image size) inflate cross-generator results by more than 11 points (Fake or JPEG).

Tintype's protocol of pre-registered blind batches plus external sets is already above typical practice. It lacks:
- a **chronological hold-out** (AI-GenBench style) that directly measures decay on post-cutoff generators,
- **human-fooling / in-the-wild** sets (Chameleon, Deepfake-Eval-2024, ITW-SM), and
- **per-stratum and prevalence-adjusted** reporting.

### Cited Findings

**23. Chandra et al. (TrueMedia.org / Univ. of Washington). "Deepfake-Eval-2024: A Multi-Modal In-the-Wild Benchmark of Deepfakes Circulated in 2024." arXiv:2503.02857 (2025).**
- About 45 h of video, 56.5 h of audio and 1,975 images from 88 websites in 52 languages, all circulated in 2024. — [arXiv](https://arxiv.org/abs/2503.02857)
- Open-source detectors lose about 50% (video), 48% (audio) and 45% (image) AUC versus academic benchmarks. The best open model reached AUC 0.58. Commercial and fine-tuned models do better but still fall short of human forensic analysts. — [arXiv](https://arxiv.org/abs/2503.02857)
- **Tintype applicability.**
  - Add the image and video subsets as an external, non-tuned benchmark: [github.com/nuriachandra/Deepfake-Eval-2024](https://github.com/nuriachandra/Deepfake-Eval-2024).
  - It contains manipulations (face swaps, edits) that are out of Tintype's scope. Report the fully-generated subset separately, if labels allow, so the result does not unfairly dominate.
  - Expected effect: a more honest estimate of wild precision and recall. Cost: small. Risk: label granularity may not allow scope filtering.

**24. Konstantinidou et al. "Navigating the Challenges of AI-Generated Image Detection in the Wild: What Truly Matters?" arXiv:2507.10236 (Jul 2025).**
- Introduces ITW-SM: 10,000 real and generated images from major social platforms. Detectors excellent on controlled benchmarks struggle on it. — [arXiv](https://arxiv.org/abs/2507.10236v1)
- Backbone, training-data composition, pre-processing and augmentation choices matter more than scale. Optimising them gave an average +26.87% AUC across several detectors in real-world conditions. Multi-megapixel wild images versus small benchmark inputs is a key issue. — [arXiv](https://arxiv.org/abs/2507.10236v1)
- **Tintype applicability.**
  - Add ITW-SM to the external suite.
  - Their pre-processing finding supports Tintype's multi-view design, and its observation that the result hinges on resampling details (docs/RESEARCH.md §1). Audit Tintype's resize path on ITW-SM.
  - Cost: small.

**25. Grommelt, Weiss, Pfreundt, Keuper. "Fake or JPEG? Revealing Common Biases in Generated Image Detection Datasets." ECCV 2024 Workshops. arXiv:2403.17608.**
- GenImage and similar datasets store reals as JPEG and fakes as PNG, and have size biases. Removing the biases improves cross-generator performance by more than 11 points (ResNet50, Swin-T) and changes JPEG robustness substantially. — [arXiv](https://arxiv.org/abs/2403.17608v1)
- **Tintype applicability.** This is the most likely mechanism behind the NTIRE distorted-real FPR spike. If any training source has distortion types such as noise, watermark overlays or heavy downscaling *more often on fakes*, the model learns "distorted ⇒ fake".
  - Change 1: a *shortcut audit*. Take Tintype's held-out reals, apply each NTIRE distortion one at a time, and plot FPR per distortion. Any distortion that inflates real FPR far above 1.5% is a learned shortcut.
  - Change 2: enforce *paired augmentation*, meaning the same distortion distribution for real and fake in every training batch.
  - Expected effect: directly targets the 26% FPR. Cost: low for the audit.
  - Risk: none for the audit. Retraining needs a new blind batch.

**26. Pellegrini et al. "AI-GenBench: A New Ongoing Benchmark for AI-Generated Image Detection." arXiv:2504.20865 (2025).**
- 36 generators ordered chronologically in sliding windows. Detectors are trained incrementally on earlier generators and tested on later ones (e.g. GAN → diffusion). Protocol limits on extra data; leaderboard per version. Performance drops at the Stable Diffusion transition. Reported size differs across sources (about 144k train / 36k test in the paper's table). — [arXiv](https://arxiv.org/pdf/2504.20865)
- **Tintype applicability.**
  - Adopt the *protocol*, not just the data. Retrain or fine-tune with a hypothetical cutoff (e.g. generators released before May 2026), then test on generators released after it. This quantifies "performance may drop after the Nov 2026 cutoff" with a measured decay per 6 months, instead of a disclaimer.
  - Cost: one or two extra training runs.
  - Expected effect: trustworthiness; gives operators a refresh interval.

**27. Yan et al. "A Sanity Check for AI-generated Image Detection" (AIDE + Chameleon). ICLR 2025. arXiv:2406.19435.**
- Chameleon keeps only AI images that human annotators misclassified as real, mostly above 720p and up to 4K. Nine off-the-shelf detectors misclassify most of its AI images as real. AIDE beats prior work by +3.5% (AIGCDetectBenchmark) and +4.6% (GenImage) and is "competitive" on Chameleon. — [arXiv](https://arxiv.org/abs/2406.19435v3); [ICLR](https://proceedings.iclr.cc/paper_files/paper/2025/hash/b0303773962ea1b5394c3a83cc7dd066-Abstract-Conference.html)
- TTC (#15) lifts a base detector on Chameleon from 62.4 to 91.1 balanced accuracy, showing how far off-the-shelf models sit from its ceiling. — [TTC HTML](https://arxiv.org/html/2608.00559)
- **Tintype applicability.** Already listed as "worth using" in RESEARCH.md but not run, since it needs an email request. Run it. It measures recall on exactly the images that matter, those that fool humans. Cost: an access request.

**28. Pal, Kruk et al. "Semi-Truths: A Large-Scale Dataset of AI-Augmented Images for Evaluating Robustness of AI-Generated Image Detectors." NeurIPS 2024 Datasets & Benchmarks. arXiv:2411.07472.**
- 27,600 reals, 223,400 masks, and about 1.33M (NeurIPS) to 1.47M (arXiv) AI-augmented images from 5 diffusion inpainting and editing models. Edit magnitude is tagged by region size and semantic change. Detectors respond very differently by edit type, magnitude and data distribution. — [NeurIPS](https://papers.neurips.cc/paper_files/paper/2024/hash/d5cdf7e56422f2a229c497dd89c3b995-Abstract-Datasets_and_Benchmarks_Track.html); [arXiv](https://arxiv.org/abs/2411.07472v1)
- **Tintype applicability.** Local edits are declared out of scope. Semi-Truths lets Tintype *measure and document* the boundary: recall versus edit area. It also measures FPR on lightly edited reals, a likely false-alarm source in practice. Cost: download.

**29. OpenFake. "OpenFake: An Open Dataset and Platform Toward Large-Scale / Real-World Deepfake Detection." arXiv:2509.09495 (Sep 2025; v2 retitled).**
- About 3M politically filtered LAION reals and about 963k synthetic images (v2: nearly 4M total). OpenFake Arena is a crowdsourced adversarial loop where users try to fool a live detector, and passing images join the dataset. Detectors trained on it generalise better, especially with compression-robust augmentation. — [arXiv](https://arxiv.org/abs/2509.09495)
- **Tintype applicability.** Tintype already trains on OpenFake. Its Arena submissions are a continuously refreshed adversarial test stream. Keep a held-out, date-stamped Arena slice for the chronological evaluation (#26). Risk: train/test leakage if Arena images flow into training. Use date cutoffs.

**Other benchmark facts relevant to choosing sets:**
- **So-Fake** (arXiv:2505.18660, 2025). So-Fake-Set has more than 2M images from 35 generators. So-Fake-OOD has 100K images: Reddit reals against commercial generators excluded from training. — [arXiv](https://arxiv.org/html/2505.18660v4). Already in Tintype's suite.
- **ImagiNet** (2024): 200K images across 4 content categories, with tracks for detection and generator ID. Baselines report 86–95% accuracy including social-network compression. The AI-GenBench comparison table lists it as 20K/20K over 8 generators instead, so the counts conflict. **WildFake**: listed with 23 generators. — [AI-GenBench paper table](https://arxiv.org/pdf/2504.20865v1); [ImagiNet arXiv](https://arxiv.org/pdf/2407.20020v1.pdf)
- **Out-of-the-box benchmark** (arXiv:2602.07814, Feb 2026, already cited in RESEARCH.md): rankings are unstable across 12 datasets (Spearman ρ 0.01–0.87). — [arXiv](https://arxiv.org/abs/2602.07814)

### Inferences
- **Precision at prevalence.** README already gives one worked example: about 43% precision at 1% prevalence. Make it systematic. For each benchmark and profile, report precision at 0.1%, 1% and 10% prevalence, computed from measured TPR and FPR with bootstrap intervals that propagate the FPR interval.
  - At 0.5% FPR the FPR's upper confidence bound dominates precision at low prevalence. The calibration-set size therefore decides how trustworthy the precision claim is: about 10/α reals per source for a usable bound.
- **Report max-over-sources FPR, not only the average.** The shipped calibration averages FPR over real sources, which hides sources like NTIRE distorted where FPR is 17× nominal.
- **Benchmarks to add, in order of value per cost:**
  1. A chronological hold-out (AI-GenBench protocol on Tintype's own data).
  2. Chameleon.
  3. Deepfake-Eval-2024 images and video.
  4. ITW-SM.
  5. Semi-Truths, for boundary documentation.
  6. DRAGON, for a per-generator and attribution table.
- **Benchmark-side pitfalls to audit in Tintype's own sets**, following Fake or JPEG:
  - format and size confounds between real and fake halves of each blind batch,
  - whether NTIRE distortions are applied identically to both classes (the organisers did), and
  - whether Tintype's training mix did the same.

### Gaps
- The DRAGON, ImagiNet and AI-GenBench sizes conflict across sources.
- Per-detector Chameleon numbers were not extracted.
- No found benchmark reports precision at realistic prevalence as a standard metric. This is a reporting gap in the field, not only in Tintype.
- "The Adversarial AI-Art" benchmark and "Community Forensics eval" were not verified in this session.

---

## Q6. Watermark and provenance complements (C2PA, SynthID, Tree-Ring, Stable Signature): do they improve precision when combined with passive detection?

### Takeaway
Where a watermark exists, watermark detection beats passive detection on effectiveness, robustness and speed: about 100× faster in Guo et al. A *positive* watermark or credential is a near-zero-FPR signal that can only raise precision. Its *absence* carries almost no information, so combining should be an OR on positives, never "no watermark ⇒ real".

### Cited Findings
- **Guo et al. "AI-generated Image Detection: Passive or Watermark?" arXiv:2411.13553 (rev. Jan 2025).**
  - Benchmarks passive versus watermark detectors. Watermark detectors win in every scenario tested, and the best is about two orders of magnitude faster than the best passive detector. The authors recommend watermark detectors where both apply. — [arXiv](https://arxiv.org/abs/2411.13553)
  - Covers academic watermark schemes, not SynthID or C2PA specifically. — [arXiv HTML](https://arxiv.org/html/2411.13553v2)
- **Google DeepMind. "SynthID-Image: Image watermarking at internet scale." arXiv:2510.09263 (Oct 2025).**
  - Positions watermarking alongside C2PA and reverse image search as complementary provenance tools. — [arXiv](https://arxiv.org/pdf/2510.09263)
- **RAW** (#4) gives conformal FPR guarantees for watermark detection within a transformation range. — [NeurIPS PDF](https://proceedings.neurips.cc/paper_files/paper/2024/file/ee62ab636066cf45a27246acca9545b7-Paper-Conference.pdf)
- **Tintype applicability.**
  - Tintype already inspects embedded Content Credentials (README "Inspect an embedded Content Credential").
  - Change:
    - Make a valid C2PA "AI-generated" assertion, or an open watermark detection (e.g. open-source Stable Signature / Tree-Ring decoders where available), a separate verdict field that can only upgrade to "AI".
    - A valid camera-signed C2PA manifest with an unbroken chain could feed an *exemption* from passive flagging, i.e. a precision gain on reals. This should be opt-in, since credentials can be stripped or forged at the capture-device level.
  - Expected effect: precision rises on the subset with credentials. That subset is small in the wild today.
  - Cost: low.
  - Risk: treating a camera manifest as proof of authenticity invites adversarial re-capture. Keep the exemption opt-in and report it.

### Inferences
- Combining helps precision mostly through the *real-side exemption* (camera-signed C2PA). Watermark positives mainly add recall on cooperating generators. Neither helps with uncredentialed media, which is most of the wild today.

### Gaps
- No found study measures the combined precision of passive detection with C2PA or SynthID on realistic traffic, or under messenger stripping.
- No public SynthID detector is available to integrate.
- Saberi et al. (ICLR 2024) on evading watermark detectors was referenced in Guo et al. but not read here.

---

## Q7. Ranked shortlist: the most promising ideas for Tintype from this area

### Takeaway
The strongest ideas fix the trustworthiness of the *numbers* (threshold transfer, per-stratum FPR, abstention, chronological decay) and remove training shortcuts that cause false alarms on distorted reals. Most are cheap compared with architecture changes.

### Cited Findings (ranked)

1. **Stratified, finite-sample real-only calibration plus a supported operator recalibration path.**
   - Basis: Mondrian conformal [Angelopoulos & Bates](https://arxiv.org/abs/2107.07511); [RAW](https://proceedings.neurips.cc/paper_files/paper/2024/file/ee62ab636066cf45a27246acca9545b7-Paper-Conference.pdf); [AAAI-26 scalar calibration](https://arxiv.org/html/2602.01973); Tintype's own NTIRE transfer result, where FPR went 25.83% → 2.17% after in-domain re-fit (docs/EVIDENCE.md).
   - What to build:
     - Strata by JPEG quality, resolution and source type.
     - Per-stratum thresholds with Clopper–Pearson-bounded FPR, and "max over sources" FPR in reports.
     - `UNSURE` for uncovered strata.
     - `calibrate --reference-real` for operators.
   - Effect: FPR claims become true per domain, so precision becomes predictable.
   - Cost: low.
   - Risk: recall falls in hard strata. Report it.

2. **Shortcut audit and paired-augmentation / aligned-fake training to cut false alarms on distorted and atypical reals.**
   - Basis: [Fake or JPEG](https://arxiv.org/abs/2403.17608v1); [Aligned Datasets](https://arxiv.org/abs/2410.11835); [B-Free](https://grip-unina.github.io/B-Free); optionally the [Stay-Positive](https://arxiv.org/abs/2502.07778v1) last-layer constraint.
   - What to do:
     - Measure real FPR per individual distortion; this is the cheap diagnostic.
     - Then retrain with identical distortion distributions for both classes, plus VAE-reconstructed reals as fakes.
   - Effect: targets the 26% NTIRE FPR and messenger recall.
   - Cost: audit hours; retrain days.
   - Risk: needs a new pre-registered blind batch.

3. **Validated abstention (`UNSURE`) using multi-view disagreement and intermediate-layer confidence, reported as risk–coverage.**
   - Basis: [ReSIDe](https://arxiv.org/html/2605.08574), whose logit confidence is at or below random under blur; [DISCERN](https://arxiv.org/pdf/2607.28769).
   - Effect: higher precision on decided items, with uncertain items routed to `--photo-second-opinion` or a human. The repo already found that routing uncertain clips to a second model works for video (RESEARCH.md §4).
   - Cost: low.
   - Risk: coverage must always be reported.

4. **Chronological hold-out evaluation and a few-shot update path for post-cutoff generators.**
   - Basis: [AI-GenBench](https://arxiv.org/pdf/2504.20865) protocol; [FSD](https://arxiv.org/abs/2501.08763v2) prototypes; optional generator-ID contrastive head ([arXiv:2511.16541](https://arxiv.org/pdf/2511.16541)).
   - Effect: turns "may drop after the cutoff" into a measured decay rate, and gives a cheap hot-fix when a new generator lands.
   - Cost: one or two training runs, plus a small adapt command.
   - Risk: support-set over-fitting. Recalibrate on reals after each update.

5. **Add in-the-wild / human-fooling benchmarks and precision-at-prevalence reporting.**
   - Basis: [Deepfake-Eval-2024](https://arxiv.org/abs/2503.02857); [ITW-SM](https://arxiv.org/abs/2507.10236v1); [Chameleon](https://arxiv.org/abs/2406.19435v3); [Semi-Truths](https://arxiv.org/abs/2411.07472v1) for scope boundaries.
   - Effect: numbers closer to deployment; precision at 0.1% / 1% / 10% prevalence with propagated intervals.
   - Cost: low.

Not recommended as defaults:
- Generic TTA (TENT/SAR/EATA fell below base in [TTC](https://arxiv.org/html/2608.00559)).
- Per-image TTA ([IAPL](https://arxiv.org/html/2508.01603v3); latency).
- Diffusion-based zero-shot detectors ([Manifold biases](https://arxiv.org/abs/2504.15470); cost).
- User-facing "unknown generator" verdicts (open-set rejection fails on successor models, [arXiv:2608.15652](https://arxiv.org/pdf/2608.15652)).
- [TTC](https://arxiv.org/abs/2608.00559) is worth an *opt-in* offline batch mode only.

### Inferences
- Items 1 and 3 can ship without retraining and would directly change what the README can honestly claim. Item 2 is the only one that attacks the root cause of the NTIRE false alarms. Item 4 addresses the post-cutoff risk in the "Limits" section.

### Gaps
- None of the shortlisted methods has been evaluated at ≤1% FPR on heavily distorted reals in the literature. Tintype's own NTIRE and content-matched-video runs would be the first such test, and should be pre-registered before trying them.
