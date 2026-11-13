# Train on distortion first: ten Tintype upgrades

The single highest-impact change for Tintype is to retrain its ViT-S/16 with a **class-symmetric, chained degradation curriculum**: noise, blur, colour and tone shifts, neural codecs, invisible watermarks, and multi-hop messenger round trips with caps down to 640 px. It should go with a **paired clean/degraded consistency loss**. Tintype's largest measured failures are distortion failures, not generator failures. On the NTIRE 2026 distorted public test it reaches **AUROC 0.636 against 0.830–0.974 for the nine entrants**, and its shipped cutoff flags **155 of 600 authentic images**. On WhatsApp/Instagram copies its recall is **27–36%**. Every top NTIRE team closed that gap with the same recipe ([NTIRE 2026](https://arxiv.org/html/2604.11487v1)). The second priority is **quality-stratified calibration with an UNSURE verdict**. Re-fitting the threshold in-domain cut NTIRE false positives from 25.83% to 2.17%, but recall fell from 44.8% to 4.77%. Ranking and calibration are therefore both broken, and each needs its own fix. Eight further changes follow, ranked by expected impact × confidence ÷ cost: recalibrating the photo second opinion at a fixed false-positive rate, content- and format-aligned training data, an offline study of view pooling and view order, a real-footage false-positive programme for video, anchoring fine-tuning to the Community Forensics prior for GANs, self-blended face training, INT8 matrix multiplies on the CPU engine, and a cheap continual refresh loop. Eight of the ten add no or negligible inference cost. Four make Tintype faster: gating the second opinion, ordering views, retiring the second network, and INT8. Every effect size below is an extrapolation. The published numbers come from ViT-B/L, DINOv2/v3 or CLIP backbones at 336–512 px, never from a ViT-S at 224 px initialised from Community Forensics, so each change must pass Tintype's own pre-registered blind protocol. This report synthesises eight research notes citing **231 distinct academic papers, reports and benchmarks** (219 with arXiv IDs, 12 without), checked against the repository on 2026-11-13.

## Tintype fails on processing history, not on new generators

The repository's own measurements show where Tintype loses. Clean, current-generator photos are its strength: precision 0.988 and recall 0.897 on blind batch 7 (`docs/BENCHMARK.md:185`), and 0.997 / 0.941 on the blind video batch (`docs/BENCHMARK.md:881`). Recall and precision collapse when an image has been processed, and on two families of generators.

| Weak spot | Measured result | Metric hit | Source in repo |
|---|---|---|---|
| NTIRE 2026 distorted public test | Recall at 1% FPR 2.2%, AUROC 0.636 (clean 0.870); 155/600 authentic flagged (25.83%) at the shipped cutoff | Precision and recall | `docs/EVIDENCE.md:193-198` |
| NTIRE false positives by distortion | 110 of the 155 sit in the 182 authentic images processed by JPEG AI, Cheng2020 or WMForger (60.4% flagged vs 10.8% for other distortions) | Precision | `docs/EVIDENCE.md:222` |
| NTIRE threshold transfer | Cutoff re-fit on hard-validation distorted reals: FPR 2.17%, recall 4.77% | Ranking | `docs/EVIDENCE.md:212-218` |
| WhatsApp/Instagram copies of 2026 AI images | 27–36% recall (default), 55–59% (`--sensitive`) | Recall | `README.md:513-516` |
| Frontier set JPEG stress | 69.3% original → 53.3% same-size JPEG → 28.0% JPEG capped at 960 px | Recall | `docs/EVIDENCE.md:36-40` |
| Arena, both classes JPEG-normalised | 88.5% → 73.1%; GPTTrace (PE-Core-L, 448 px) reaches 76.7% | Recall | `docs/EVIDENCE.md:93-96, 139-144` |
| GAN sets | 84–89% recall | Recall | `README.md:517-519` |
| FF++ face swaps | 0.143 recall (0.480 with the CF-384 second opinion; CF alone 0.573) | Recall | `docs/BENCHMARK.md:220-222` |
| Photo second opinion | Real photos flagged 1.2% → 3.1%; recall 0.897 → 0.935; GPU throughput roughly halves | Precision, speed | `docs/BENCHMARK.md:181-186` |
| Content-matched video (Davodi et al.) | 3/20 authentic Pixabay clips flagged; 72/80 generated found | Precision | `docs/EVIDENCE.md:236-243` |
| CPU throughput | 77–78 photos/s macOS, 35 photos/s 8-core aarch64 Linux (FP32 only) | Speed | `docs/BENCHMARK.md:250, 965` |

Three facts from the codebase shape the ranking. First, **the training augmentation never covers noise, blur, sharpening, WebP/AVIF/JPEG 2000, neural codecs, colour changes or watermarks**. `augment()` in `src/data.rs:57-104` has only JPEG before and after resizing, a random resampling kernel, mild scale jitter, a flip, a messenger round trip (long side 960–1700 px, JPEG quality 55–90, p = 0.25 in photo phases 2, 3 and 6, 0 in phase 1, 0.1 for video) and H.264 re-encoding for video. It takes no label, so whatever it does is already class-symmetric. Second, **calibration is a single real-quantile threshold averaged over real sources**, with no quality conditioning and no abstention; "inconclusive" only means incomplete video decoding (`src/eval.rs:276-337`, `src/scan.rs:611-614`). Third, **fine-tuning has no anchor to the initial checkpoint**. AdamW decays weights toward zero, and only layer-wise learning-rate decay of 0.8 limits drift across 36k steps from Community Forensics 224 (`src/trainer.rs:131-136`).

The notes contain several claims the codebase contradicts, and the ranking below does not rely on them. Note 03 says training already uses the six texture-ranked views, but each training sample is one random view: a fit view or a random-position native crop (`src/data.rs:80-94`). Notes 03 and 04 argue that 15.8k Reddit fakes against 3.0k Reddit reals teach "platform look ⇒ AI". The class-balanced, √count-weighted sampler makes `reddit-wild` about 1% of AI draws, and Reddit reals are probably a *larger* share of real draws, so that mechanism is unsupported until per-key shares are computed from the training manifests, which are not in the repo. Note 07 assumes per-view logits are archived, but the archived score files hold probabilities only, so a local re-score with `--all-views` is needed for per-view analyses. Note 05 says stock footage is absent from video training, but Pexels stock and OpenVid reals are training negatives; only Pixabay is absent. Note 08 calls the NTIRE failure "mainly threshold transfer, not ranking". The repo's own transfer experiment refutes that, because fixing the threshold destroyed recall.

## The ranked ten at a glance

Impact is judged against the size of the measured gap each change targets. Confidence reflects how directly the published evidence transfers to a ViT-S at 224 px trained on about 1,200 generators. Cost counts engineering in the Rust engine and the macOS-only MPSGraph trainer (`src/trainer.rs`), which has no PyTorch. That invalidates the notes' "1–2 days in PyTorch" estimates.

| Rank | Change | Weak spot → metric | Inference cost | Engineering cost | Confidence |
|---|---|---|---|---|---|
| 1 | Symmetric chained degradation curriculum plus paired consistency loss | NTIRE, messenger, JPEG stress → recall and precision | None | Medium (augment ops, offline codec pairs, pair loss) | Medium-high |
| 2 | Quality-stratified calibration with an UNSURE verdict | Degraded-real false positives → precision | Negligible | Low | Medium |
| 3 | Learned, recalibrated second-opinion fusion with a gated band | 1.2% → 3.1% FPR jump → precision, speed | Lower (fewer photos routed) | Very low | High |
| 4 | Content- and format-aligned data: audit, JPEG-matched VAE reconstructions, regional inpaints, more real domains | Messenger, Arena/Frontier re-encode, unseen LDMs → recall, precision | None | Medium-high (offline GPU generation) | Medium |
| 5 | View pooling, view order and view selection study | Recall per view, views per image → recall, speed | Neutral or lower | Low | Medium |
| 6 | Real-footage false-positive programme and early-exit hardening for video | Pixabay false positives, long clips → precision, recall | Small, uncertain band only | Low-medium | Medium |
| 7 | Anchor fine-tuning to Community Forensics and distil CF-384 into the ViT-S | GANs, face swaps, second-opinion cost → recall, speed | Lower if stage 2 is retired | Medium | Low-medium |
| 8 | Self-blended face training with a face-gated view | Face swaps (14%) → recall | +17–33% on images with faces only | Medium-high | Low-medium |
| 9 | INT8 matmuls in the CPU engine behind a verdict-flip gate | Linux/CPU throughput → speed | Lower | Medium-high | Low-medium |
| 10 | Continual refresh loop with replay, and a chronological hold-out | Post-cutoff generators → recall | None | Medium | Medium |

## Fixing the distortion gap: recommendations 1–4

### 1. Train on what NTIRE and messengers do to images, symmetrically and in pairs

**Change.** Extend `augment()` in `src/data.rs` with a chained degradation sampler that draws 1–5 operations at graded severity, modelled on the NTIRE 2026 distortion families. These are Gaussian, Poisson, speckle, impulse and ISO-like noise; Gaussian, motion and lens blur; unsharp sharpening; pixelation and downscale-upscale chains; WebP, AVIF and JPEG 2000 round trips; low-probability tone curves, gamma, saturation and colour casts; and screen or print recapture simulation (perspective, moiré, gamma, defocus, noise, then JPEG). Widen the messenger round trip to caps of 640–2048 px on width, height or long side, and chain 1–4 share cycles. Neural codecs (JPEG AI reference software, CompressAI Cheng2020 and hyperprior at 0.06–0.5 bpp) and open invisible watermark embedders have no Rust implementation. Pre-encode them offline as additional plan stores, drawn for both classes, the same way Ant International used offline augmentation levels. Organise the phase as a curriculum of clean, mild, moderate and heavy levels. In `src/trainer.rs`, add a paired loss: each step scores a clean view and a degraded view of the same image, and adds a symmetric KL term between their logits plus a cosine term between their class-token features. Because `augment()` is label-blind, symmetry is automatic for synthetic operations. The offline codec and watermark stores must also contain both classes.

**Evidence.** The NTIRE 2026 robust track chained 1–5 of 36 distortions, including JPEG AI, Cheng2020 and WMForger. The top two teams reached **robust AUROC 0.9723 and 0.9721 against clean 0.9974 and 0.9972**, and none of the top six used a frequency or noise-residual stream ([NTIRE 2026](https://arxiv.org/html/2604.11487v1)). The most transferable data point is a single SigLIP2-giant with a linear head that applied the official distortion function to every training image. It reached **0.873 robust AUROC**, and its authors called that augmentation "the key driver" ([NTIRE 2026](https://arxiv.org/html/2604.11487v1)). The warning case is Shallow Real: **0.995 clean but 0.834 robust** ([NTIRE 2026](https://arxiv.org/html/2604.11487v1)). For the codec false positives, Cannas et al. measured **false-positive rates of 0.96–0.99 on pristine images compressed with JPEG AI at 0.12 bpp** across four detectors. They traced it to decoder upsampling that mimics generator spectra, and found that retraining with 20% JPEG AI samples cut it only to 0.65 ([Cannas et al.](https://arxiv.org/html/2412.03261v2)). TrueFake measured real platform pipelines: Facebook caps width at 720 px with dynamic quality 61–92, X caps at 1200 px with quality 87, and Telegram caps height at 800 px with quality 85. All three fall partly outside Tintype's 960–1700 px window ([TrueFake](https://arxiv.org/html/2504.20658)). RRDataset showed that 2–6 share cycles cost SAFE **97.41 points of fake accuracy** ([RRDataset](https://www.alphaxiv.org/abs/2509.09172.md)). Konstantinidou et al. measured what a broad augmentation set buys on in-the-wild social data: **ITW-SM AUC 82.20 → 92.26 for DMID, 93.70 → 98.23 for RINE and 91.46 → 98.10 for SPAI** ([Navigating the challenges](https://arxiv.org/html/2507.10236v1)). SPAI's ablation shows that dropping blur and noise augmentation costs **6.8 AUC** ([SPAI](https://arxiv.org/html/2411.19417v2)). For the paired loss, TeleAI's LoRA-on-EVA-CLIP entry, which also runs at 224 px, gained **+1.35 public-test AUC** from its KL and feature terms (87.91 → 89.26) ([LPT](https://arxiv.org/html/2604.12307v1)). DCPT raised average degraded accuracy on Synthbuster from **64.0 to 73.1** for a 0.9-point clean cost, at no inference cost ([DCPT](https://arxiv.org/html/2604.10102v1)).

**Expected effect on Tintype (extrapolated).** This is the most plausible route from AUROC 0.636 toward the 0.83+ range of NTIRE entrants without leaving the 224 px budget. It should also lift messenger recall and narrow the Frontier and Arena JPEG gaps. The codec false positives should fall but not disappear, as Cannas et al. found. A ViT-S has far less capacity than DINOv3-7B, so the gain will be smaller than the winners'. DCPT's pattern, where accuracy at a fixed threshold rises more than AUC, means part of the benefit will show up as more stable scores at the shipped cutoff rather than as better ranking.

**Cost.** Training only; no inference cost. Rust augmentation ops are moderate work. Offline codec and watermark pairs need a one-off Python job on any machine. The paired loss roughly doubles forward passes per sample.

**Risks.** Overfitting to NTIRE's own distortion generator, so hold out some operation families and keep the WhatsApp/Instagram, Arena-JPEG and So-Fake-OOD sets as independent tests. A dip in clean recall, as with Shallow Real. Colour augmentation is contested: SPAI found chromatic augmentation cut its AUC from 91.0 to 80.5 ([SPAI](https://arxiv.org/html/2411.19417v2)), while NTIRE's test set includes colour and tone distortions. Keep colour operations at low probability inside the chain and measure them separately.

**Cheapest first experiment.** Run a shortcut audit before training anything. Take 2,000 held-out calibration reals, apply each candidate operation one at a time (including CompressAI codecs and a watermark embedder), and `tintype score` each manifest with the shipped weights. Plot real false-positive rate per operation against the 1.5% target. Any operation far above target is a learned shortcut and a priority for the curriculum. Then run a short phase (about 4k steps from the phase-6 weights) with the new operations, run `finalize`, and compare NTIRE hard-validation distorted recall at 1% FPR (baseline 5.4%) and AUROC (baseline 0.734) through `evaluate`. Clean recall on batch 7 must hold before a pre-registered confirmation batch is drawn.

### 2. Calibrate per quality stratum and let Tintype say UNSURE

**Change.** In `calibrate` (`src/eval.rs:276-337`), replace the single Balanced(100) threshold with group-conditional (Mondrian) thresholds over observable strata. Free metadata gives part of this: estimated JPEG quality from the quantisation tables, resolution bucket and container format. A learned degradation estimate gives the rest. Train it as an auxiliary head in recommendation 1's phase, regressing the applied augmentation parameters (JPEG quality, scale factor, noise and blur sigma, a neural-codec flag) from the class token. Store one bias per stratum in the model spec, and apply it in `src/detector.rs`. Add an `UNSURE` verdict in `src/scan.rs` and `src/server.rs` for strata with too little calibration data, and for inputs whose degradation estimate says the detector is unreliable. Report the maximum per-source FPR with Clopper–Pearson upper bounds alongside the average.

**Evidence.** QuAD showed that as quality drops, **both real and fake logit distributions shift and widen**, and their overlap grows. Modelling logit mean and variance per class as a function of a no-reference quality score raised six-detector average balanced accuracy on AncesTree from **73.2% (naive aggregation) to 81.6%**, and cut NLL from 0.97 to 0.43 ([QuAD](https://arxiv.org/html/2604.15027v1)). LAION-Mobile found that thresholds re-fitted on modern content flag **17–91% of real smartphone photos**. No detector beat AUC 0.624 on modern content at a deployable false-alarm rate ([LAION-Mobile](https://arxiv.org/abs/2609.11134)). Abstention needs its own signal, though. ReSIDe showed that max-softmax confidence is *worse than random* for abstention under blur (AURC 0.7276 against an error rate of about 0.495). Its intermediate-layer probes cut blur AURC to 0.2214 ([ReSIDe](https://arxiv.org/html/2605.08574)). HiDA-Net's JPEG-quality regression head is direct precedent for a degradation auxiliary head that separates compression from generation traces ([HiDA-Net](https://iclr.cc/virtual/2026/poster/10011114)). The statistical basis is split conformal prediction. Averaging FPR over sources guarantees only marginal coverage over the mixture, not per-source coverage ([Angelopoulos & Bates](https://arxiv.org/abs/2107.07511)). RAW's conformal FPR guarantee holds only for transformations covered by the calibration data ([RAW](https://proceedings.neurips.cc/paper_files/paper/2024/file/ee62ab636066cf45a27246acca9545b7-Paper-Conference.pdf)). That is exactly why NTIRE's distorted reals broke Tintype's 1.5% target.

**Expected effect on Tintype (extrapolated).** False-positive rates on degraded reals should return near target *within each covered stratum*, which directly attacks the 155/600. This is a calibration fix, not a ranking fix. The repo's transfer experiment shows what happens when only the threshold moves: recall in the hardest stratum collapses (4.77%). Recommendation 2 should therefore ship with recommendation 1, and Tintype's reports should state recall per stratum. On messenger copies, per-stratum thresholds help recall only if messenger *reals* score lower than the pooled reals, which must be measured. An important limit: **all 5,000 NTIRE files share the same JPEG quantisation tables and 4:2:0 subsampling** (`docs/EVIDENCE.md:222`). Metadata strata cannot detect NTIRE's neural-codec history, so for that case the learned degradation head is the working half of the design.

**Cost.** Metadata strata need no retraining and a few hundred lines of Rust. The auxiliary head costs one 384-dimensional dot product at inference and a loss term in the trainer. Tight bounds need roughly 2,000 reals per stratum at 0.5% FPR.

**Risks.** An attacker can degrade a fake to push it into a lenient stratum. Mapping uncovered or heavily degraded strata to UNSURE rather than "real" limits this. Strata that correlate with fakes, such as PNG, would hand fakes a lenient threshold, so thresholds must be fitted on reals only. Abstention can hide failure, so coverage must always be reported with precision.

**Cheapest first experiment.** The archived NTIRE score files hold probabilities, and `logit(p)` recovers the calibrated logit behind each verdict, so this needs no re-scoring. Fit per-distortion-group thresholds on hard-validation reals (which contain JPEG AI but not Cheng2020 or WMForger), apply them to the public test, where `docs/EVIDENCE.md:222` already groups authentic images by operation, and compare FPR and recall against the single re-fitted cutoff (2.17% / 4.77%). Repeat on the WhatsApp/Instagram copies, stratified by quantisation-table family and long side.

### 3. Recalibrate the photo second opinion at a fixed false-positive rate

**Change.** The current rule sends photos with calibrated logit in (−7, 0.5) to Community Forensics 384 and outputs `sigmoid(0.5·(0.5·z1 + z2))`, with no recalibration of the combined threshold (`src/detector.rs:220-251`). Replace the fixed combination with a logistic regression on z1, z2, view spread (maximum minus minimum view logit) and views used. Fit it on development sets disjoint from the blind batches. Then choose its threshold with the same Balanced(100) rule at 1.5% FPR. Make the band symmetric or learned so that Community Forensics can also *veto* borderline positives. The current upper bound of 0.5 means almost no photo Tintype flags can be cleared. Narrow the band to where the two models disagree usefully. Store the result through `set-photo-second-opinion` in `models/tintype.json`.

**Evidence.** The repo's numbers make the rule's behaviour predictable. In-sample, **48.8% of photos go to stage 2, including 91% of reals**, and the combined threshold was chosen by grid search on seen sets (`docs/PREREGISTRATION.md:185-197`). On batch 7 that raised real photos flagged from 130 to 329 (1.2% to 3.1%) for recall 0.897 → 0.935 (`docs/BENCHMARK.md:181-186`). The honest baseline is Tintype alone at about 3.1% FPR. At 5% (`--sensitive`), Tintype alone already reaches **0.963 / 0.960** (`README.md:315-318`), so part of the second opinion's gain may be a threshold move. The repo's own earlier estimate on seen data found a rule that flags "where the two scores add up high enough" moved 0.986 / 0.876 → 0.983 / 0.905 at nearly constant precision (`docs/BENCHMARK.md`). Cascade theory says a confident first model should stop most items: a ViT cascade matched ViT-L-384 accuracy at **2.3× lower cost** ([Wisdom of Committees](https://arxiv.org/abs/2012.01988)). Score-fusion work in audio deepfake detection uses logistic-regression fusion, and warns that the data used to fit the fuser drives its generalisation ([fusion of SSL countermeasures](https://arxiv.org/pdf/2509.12003); [multi-objective fusion](https://arxiv.org/abs/2604.01330)).

**Expected effect on Tintype (extrapolated).** FPR returns to about 1.5% by construction. Recall above 0.897 may survive, but by less than the current +3.8 points. A narrower band sends fewer photos to the second network, lifting throughput above the current 281–288 photos/s on GPU. The experiment may also show that the second opinion adds little at matched FPR. That is itself useful: drop it from the default advice, or keep it for `--sensitive` only.

**Cost.** Hours. `photo_second_opinion_from_scores` (`src/eval.rs:240`) already fuses from score files offline, and `report --at-rate-of` prints systems at another system's false-positive count. Inference cost is a dot product.

**Risks.** A combiner fitted on seen sets will overfit them. Pre-register the evaluation on a fresh batch.

**Cheapest first experiment.** Run `report --at-rate-of` on the batch-7 score files to compare Tintype alone with the second-opinion output at the same false-positive count. Then fit the logistic combiner on the 48 development photo sets, and re-score batch 7 retrospectively, with confirmation reserved for the next pre-registered batch.

### 4. Make real and fake differ only in generator traces

**Change.** This merges four overlapping shortlist items from notes 01, 02, 03, 04 and 08. First, freeze the training manifests (`freeze-set` exists) and audit each store, per class, for container format, JPEG quality and table family, chroma subsampling, resolution and EXIF presence. Resample or augment where they differ. Second, generate offline VAE reconstructions of Tintype's real training images through 4-channel (SD 1.x/SDXL) and 16-channel (SD3/FLUX) autoencoders. With probability 0.5, re-encode each reconstruction at its source real's estimated JPEG quality, then add pixel mixup between real and reconstruction, as DDA does. Mix them in at roughly 20–30% of fake draws rather than replacing generator data. Third, add regional inpaints of real photos with the background pixels restored, B-Free style, which also opens a path to local edits. Fourth, add real domains Tintype lacks: smartphone photos with EXIF, camera originals, screenshots, and social-platform reals such as the shared reals in RRDataset and TrueFake. Tintype has about 1,200 generator keys but only about 16 real sources.

**Evidence.** Aligned reconstructions raised post-processed recall from **24.27% to 76.36% on Playground and 58.31% to 91.08% on LCM**. With 1k images per class, TPR at 5% FPR was 83.4 against 46.5 for the Corvi baseline. FLUX's 16-channel VAE was caught only **9.6%** of the time when training used 4-channel VAEs only, and WebP artifacts present only in reals became a shortcut ([Aligned Datasets](https://arxiv.org/html/2410.11835v3)). DDA showed that reconstructions carry more high-frequency energy than JPEG reals, so naive reconstruction training teaches "sharp = fake". Its JPEG-matched recipe reached **90.7% average and 81.4% minimum accuracy over 11 benchmarks**, led the runner-up by 10.5 points at JPEG quality 60, and took 5.9 hours to build. Its weak spot was GANs: ForenSynths 81.4 against FatFormer's 90.1 ([DDA](https://arxiv.org/html/2505.14359v6)). B-Free's ablation raised balanced accuracy from **81.4 (self-conditioned reconstructions) to 92.2 with inpainted fakes and 96.4 with extra augmentation**. Its social-network balanced accuracy was Facebook 95.6, Reddit 86.2 and Twitter 97.3, and simulated sharing left it flat (86.3 → 86.4) ([B-Free](https://arxiv.org/html/2412.17671v2)). Removing JPEG and size bias alone gained **more than 11 points** cross-generator ([Fake or JPEG?](https://arxiv.org/abs/2403.17608)). ScaleDF found error falls as a power law in real domains (**1−AUC = 0.729·N^−0.476**), steeper than in generation methods (0.618·N^−0.375), while raw image count saturates ([ScaleDF](https://arxiv.org/html/2510.16320)). Community Forensics found gains flatten beyond about 1,000 generators ([Community Forensics](https://arxiv.org/html/2411.04125)), so Tintype is past the knee on generators but not on real domains.

**Expected effect on Tintype (extrapolated).** A smaller recall drop when both classes are re-encoded (Arena 88.5 → 73.1; Frontier 69.3 → 28.0). Better messenger recall. Better recall on new latent-diffusion generators that reuse existing VAEs. Fewer false positives on unusual real content. Little help for autoregressive or pixel-space generators, and possibly a GAN regression, which recommendation 7 counters.

**Cost.** Training only. VAE reconstructions are cheap (one encode-decode per real). B-Free-scale inpainting took 258.8 GPU-hours for 309k fakes ([DDA](https://arxiv.org/html/2505.14359v6)). Generation needs a GPU machine outside the repo's tooling, and the results enter as ordinary plan stores.

**Risks.** **This recommendation conflicts with recommendation 1.** Labelling VAE-decoded reals as fake while labelling neural-codec-compressed reals as real asks the model to separate two learned decoders that share upsampling traces. Mandelli et al. showed that passing a real photo through Stable Diffusion's autoencoder produces a copy detectors read as synthetic ([SD laundering](https://arxiv.org/pdf/2407.10736)). Tintype needs an explicit labelling policy for VAE-laundered reals before it trains on both. Mixup is also contested. DDA uses it, while B-Free measured cutmix and mixup *lowering* balanced accuracy from 81.4 to 78.6 ([B-Free](https://arxiv.org/html/2412.17671v2)), so treat it as an ablation, not a default.

**Cheapest first experiment.** Reconstruct 5,000 held-out calibration reals through an SD VAE and a FLUX VAE. Score them, and the same reals compressed with JPEG AI and Cheng2020, with the shipped weights. If Tintype already flags the reconstructions at a high rate, the recall headroom is small. The codec scores show how sharp the labelling conflict is. Then run a 4k-step phase with a reconstruction store at about 25% of fake draws, and check Arena-JPEG (73.1%, against GPTTrace's 76.7%), Frontier JPEG-960 (28.0%) and the messenger copies.

## Cheap structural wins: recommendations 5–7

### 5. Re-derive view pooling and view order from all-view scores

**Change.** Re-score the development sets, batch 7 and NTIRE hard validation with `tintype score --all-views`. Offline, compare the plain mean against top-2 mean, trimmed mean, log-mean-exp, native-only mean, and a per-view-type logistic weighting (fit, centre, rich-k). Test two orders: fit view first for images with messenger-like quantisation tables or a long side of 1600 px or less, and fit view last otherwise. Add a least-textured non-flat window (lowest gradient energy above a floor) as an optional view in `src/image_io.rs:291-353`. Test a noise-robust texture measure for selection. Add a quality-aware view cap, so that tiny or heavily recompressed images stop at fit plus centre. Re-derive early-exit margins and calibration with `apply_profile` (`src/eval.rs:365-392`), because any pooled statistic other than the mean shifts the threshold.

**Evidence.** Tintype's own ablation is the strongest data point. Scoring all six views instead of early exit raised hard-validation **clean recall at 1% FPR from 51.8% to 60.5%** (AUROC 0.938 → 0.955), but distorted recall only from 5.4% to 7.5% (`docs/EVIDENCE.md:224`). TextureCrop found texture-selected native crops beat centre crops by **+3.13 bAcc, +5.17 AP and +5.71 AUC**, and beat resizing by +9.33, +15.78 and +13.96. Mean aggregation was best, its best fixed count was 15 crops, and resizing won only on DALL·E 3 ([TextureCrop](https://arxiv.org/html/2407.15500v1)). SPAI's learned attention over all patches beat a ten-crop mean by **5.7 AUC** ([SPAI](https://arxiv.org/html/2411.19417v2)). Panoptic Patch Learning found detectors develop a few-patch bias: masking a single patch cut accuracy by **18.7 ± 4.1%** ([Panoptic Patch Learning](https://arxiv.org/html/2504.01396v2)). For a fully-generated head, average pooling beat max pooling on every LaDeDa benchmark (UFD accuracy 96.3 against 88.9) ([LaDeDa](https://arxiv.org/html/2406.09398v1)). AIDE and PatchCraft deliberately include the lowest-texture or lowest-frequency patches ([AIDE](https://arxiv.org/abs/2406.19435); [PatchCraft](https://arxiv.org/abs/2311.12397)). NTIRE winners resized distorted inputs to 512 or 288 px rather than tiling them ([NTIRE 2026](https://arxiv.org/html/2604.11487v1)), which suggests the fit view matters more once native pixels are already resampled.

**Where the notes disagree.** Note 07 argues the fit view is the most fragile view, because resampling details flip Community Forensics decisions, and should be scored last. Note 01 argues it should come *first* for recompressed images, citing the NTIRE winners. Both can be right for different processing histories, which is why the experiment splits by recompression evidence rather than picking one order.

**Expected effect on Tintype (extrapolated).** Either equal accuracy with fewer views (speed) or higher recall at equal views. The 51.8% → 60.5% gap says that early exit currently gives up clean recall that a better order or pooling might recover. Expect small gains on distorted inputs.

**Cost.** A re-scoring run and an analysis script. A small Rust change for the new window type. No retraining, because training already uses random native crops that cover flat regions.

**Risks.** Learned per-view weights overfit development generators. Flat crops in JPEG copies are dominated by 8×8 block artifacts and could raise false positives, so check per-source FPR.

**Cheapest first experiment.** The all-view re-score plus the offline pooling table: recall at the default calibrated FPR and mean views per image for each rule, on batch 7 and NTIRE hard validation.

### 6. Treat stock footage as a real-video domain and stop clearing clips early

**Change.** Score held-out stock (Pixabay first, then additional Pexels clips), drone and gimbal, stabilised phone, screen-recorded and broadcast footage at the shipped video threshold, and report FPR per collection. Add Pixabay and professionally graded footage, motion-balanced, to the video negatives and to the video calibration pool, which today holds only MSR-VTT, MSVD and Kinetics (`docs/DATA.md:401-403`). In the next video fine-tune, add WaveRep's wavelet band-replacement augmentation: swap the fake frame's LL, LH and HL Haar bands with a matched real or VAE-reconstructed frame. Harden sampling in `src/video.rs` and `src/detector.rs` in three ways: spread frames over the whole clip rather than the first 30 s (`src/yuv.rs:141`), never clear a clip as real before both halves are sampled, and score all 16 frames for clips in the uncertain band.

**Evidence.** Cakiroglu et al. found that detectors learn the identity of their real corpus. At a threshold flagging 5% of the training benchmark's reals, they flag **15% or more of another real collection**. Re-fitting the threshold on half of the target collection's reals brought FPR back to **4.1–6.8%**, at a recall cost of only 95 → 93 for a WaveRep-like head ([Cakiroglu et al.](https://arxiv.org/html/2606.31004v2)). Vahdati et al. showed that aggregating more patches across time raised AUC at H.264 CRF 40 from **0.953 (one patch) to 0.984 (80 patches)** ([Vahdati et al.](https://arxiv.org/html/2404.15955v1)), which argues against clearing a clip after about 4.8 frames. WaveRep's band replacement lifted balanced accuracy from **88.2 to 91.8**, and its frame-level DINOv2 beat a Hiera video backbone 91.8 to 73.2 ([WaveRep](https://arxiv.org/html/2506.16802v2)). Under motion rebalancing it held 0.989 → 0.989 AUC while motion-based detectors fell, and one blurred frame collapsed D3 from 0.794 to 0.456 ([Michels et al.](https://arxiv.org/html/2607.00948v2)). FakeParts shows partial fakes defeat clip-mean pooling ([FakeParts](https://arxiv.org/abs/2508.21052)). Tintype's own history agrees: adding about 2,400 cartoon, animation and gameplay clips fixed most of its earlier video false alarms (`docs/RESEARCH.md` §4).

**Expected effect on Tintype (extrapolated).** Fewer false positives on stock and graded footage, with a small recall cost on stock-like generated clips. Better recall on long or partially generated clips. The 20-clip Davodi cohort cannot resolve changes below 5 points, so a larger matched set is needed to confirm any gain.

**Cost.** Data collection plus one video fine-tune. Extra inference only on uncertain clips.

**Risks.** Generators are trained on stock footage, so stock negatives may cost recall on stock-like fakes. Any temporal feature added later needs fps and duration normalisation to avoid shortcuts.

**Cheapest first experiment.** `score-video` already writes per-frame logits. Score about 300 Pixabay clips outside the Davodi cohort plus two other real collections at the shipped threshold. On the Davodi cohort, re-score with whole-clip sampling (`--max-seconds 0`) and with all frames, then re-aggregate the per-frame logits offline to see how no early exit and whole-clip coverage move its AUROC (0.9244) and its 3/20 false positives.

### 7. Anchor fine-tuning to Community Forensics and distil its 384 px model

**Change.** In `src/trainer.rs`, add an L2-SP penalty pulling weights toward the Community Forensics 224 initialisation, or a soft Effort-style penalty on movement in the top singular directions of each linear layer. Add a distillation loss toward Community Forensics 384 logits on GAN, face and older-generator stores. These logits can be cached offline with `tintype score --reference commfor`, because CF-384 already runs in Tintype's engine. Replay GAN stores during the phase. If the distilled model matches the second opinion's recall, retire stage 2 and get back the full 530–563 photos/s. A larger foundation-model teacher (DINOv3 or PE-Core class) scoring clean originals while the student sees degraded copies is a later, costlier extension that needs an external GPU machine.

**Evidence.** Effort showed full fine-tuning collapses the feature space. Effective rank fell to 238 of 479 dimensions, against 316 for its SVD-residual method, and mean accuracy was **86.22 (full fine-tuning) vs 93.53 (LoRA) vs 95.19 (Effort)** on UniversalFakeDetect ([Effort](https://arxiv.org/html/2411.15633v4)). DGS-Net, which steers fine-tuning gradients toward a frozen teacher, reported **+6.6 average** over the state of the art across 50 generators ([DGS-Net](https://arxiv.org/abs/2511.13108)). GAPL describes a "Benefit then Conflict" pattern, where adding generators eventually degrades older ones ([GAPL](https://arxiv.org/abs/2512.12982)). Tintype's GAN regression (84–89%) and face-swap gap (0.143 against CF's 0.573) fit that pattern. Large frozen encoders carry knowledge a ViT-S lacks: a frozen DINOv3-7B linear probe scored **87.5% on So-Fake-OOD against 75.61% for Community Forensics** ([Huang et al.](https://arxiv.org/html/2511.22471)). In the repo, GPTTrace (PE-Core-L, 448 px) beats Tintype after JPEG normalisation on Arena, 76.7% vs 73.1% (`docs/EVIDENCE.md:139-144`).

**Where the evidence conflicts.** Effort's gains come from narrow training (ProGAN or SD v1.4 only). With broad data, end-to-end fine-tuning beats frozen probes by a wide margin: **95.2 vs 68.5 bAcc** in B-Free ([B-Free](https://arxiv.org/html/2412.17671v2)). Community Forensics found freezing "consistently" worse ([Community Forensics](https://arxiv.org/html/2411.04125)). Tintype is in the broad-data regime, so anchoring may buy little beyond what the low learning rate and layer decay already do. That is why confidence here is low to medium.

**Expected effect on Tintype (extrapolated).** GAN and face-swap recall should move toward Community Forensics levels, possibly at a small cost on the newest commercial generators. If stage 2 can be retired, the second-opinion mode's throughput penalty disappears.

**Cost.** Moderate trainer work. L2-SP is simple. Distillation needs cached teacher scores and a second loss input. No inference cost.

**Risks.** Regression on current commercial generators. Inheriting Community Forensics' own resampling fragility (`docs/RESEARCH.md` §1).

**Cheapest first experiment.** Confirm the drift first. Score the GAN sets and FF++ with the CF-224 initialisation checkpoint and compare against Tintype. Only CF-384 is a built-in reference, so this needs a model spec for the 224 px checkpoint. If the initialisation beats Tintype on GANs, forgetting is real. Then run a 4k-step phase with L2-SP plus cached CF-384 distillation, and compare GAN and FF++ recall and batch-7 precision and recall.

## Specialist and long-horizon work: recommendations 8–10

### 8. Teach face swaps from self-blended real faces

**Change.** Generate self-blended images (SBI) offline from faces in Tintype's real stores (CelebA-HQ and other face-bearing reals): one pristine face, colour and frequency jitter, a deformed landmark mask, blended back. Train them into a separate face-manipulation head on the shared trunk, with its own threshold calibrated on real face photos, including beauty-filtered selfies. At inference, a tiny face detector in the Rust engine adds one or two face-crop views only when a face is present. README marks face swaps out of scope (`README.md:510-512`), so this is a scope decision as much as an engineering one.

**Evidence.** SBI needs no deepfake data and improved over its baseline by **+4.90 points on DFDC and +11.78 on DFDCP** ([SBI](https://arxiv.org/abs/2204.08376)). Under Effort's re-evaluation (FF++ c23, video AUC) SBI averaged **0.821** over seven sets, against UCF's 0.785 and Effort's 0.917 ([Effort](https://arxiv.org/html/2411.15633v4)). Every DeepfakeBench detector consumes aligned face crops ([DeepfakeBench](https://github.com/SCLBD/DeepfakeBench)). Tintype's texture ranking favours hair, foliage and fabric over smooth skin, so its native crops can miss the face. Deepfake-Eval-2024 is the sobering counterweight: open-source detectors lost about **45% AUC on in-the-wild images** compared with academic benchmarks ([Deepfake-Eval-2024](https://arxiv.org/abs/2503.02857)).

**Expected effect on Tintype (extrapolated).** The most direct route from 14% face-swap recall toward the range Community Forensics reaches (0.573), with no change for images without faces. Treat this as a hypothesis, since no study tests SBI on a ViT-S at 224 px.

**Cost.** Medium to high: offline landmarks and blending, a new head, and a ported face detector. Inference is +17–33% compute on images with faces only (six views become seven or eight).

**Risks.** False positives on retouched, filtered or AR-effect selfies, and on faces in cartoons and games. Note 06 believes the FF++ test images may already be face crops. If so, the face detector adds nothing on that benchmark and only the training signal matters.

**Cheapest first experiment.** Check whether the FF++ test images are face crops. Score 2,000 SBI blends of held-out real faces with the shipped weights. Then fine-tune a short phase with SBI at about 5% of fake draws in the main head, and measure FF++ recall and FPR on real face sets.

### 9. Quantise the CPU engine's matmuls to INT8 behind a verdict-flip gate

**Change.** In `src/cpu_vit.rs`, quantise the four large matmuls per block (QKV, attention output, MLP up, MLP down) to INT8 weights with per-channel scales. Use INT8 activations with per-tensor scales calibrated on *degraded* images, accumulating in int32 through ARM SDOT/I8MM and AVX-VNNI kernels. Keep patch embedding, LayerNorm, softmax and the residual stream in f32. If post-LayerNorm outlier channels cause flips, fold RepQ-ViT's reparameterisation into the weights offline. Ship it behind a flag only if verdict flips stay within a few per 10,000 and pooled precision and recall stay within 0.001, the rule already used for early-exit margins (`docs/BENCHMARK.md:883-893`).

**Evidence.** General-vision post-training quantisation at 8 bits loses **under 0.5% ImageNet top-1** for ViT and DeiT ([PTQ4ViT](https://arxiv.org/abs/2111.12293)). RepQ-ViT handles post-LayerNorm channel outliers ([RepQ-ViT](https://arxiv.org/abs/2212.08254)). The only peer-reviewed forensic quantisation study kept full-precision shortcuts "to retain method-specific textures" and reached **91.8% accuracy at 2 bits** on cross-GAN tests with a ResNet ([Tao et al., IJCAI 2025](https://www.ijcai.org/proceedings/2025/59)). The counter-evidence is serious. Static INT8 object detectors were more noise-sensitive than FP32 ([arXiv 2508.19600](https://arxiv.org/abs/2508.19600v1)). Tintype's own token-merging test lost **6.2 and 14.0 recall points** (`docs/BENCHMARK.md:911-916`), which shows forensic signal can be fragile under approximations that are harmless on ImageNet.

**Expected effect on Tintype (extrapolated).** Roughly 1.5–2× CPU throughput, for example 35 → 55–70 photos/s on 8-core aarch64 Linux. That figure is inferred from instruction widths and is unmeasured. No effect on GPU, which already runs FP16.

**Cost.** Moderate to high engineering, but no training. The project already writes its own CPU kernels.

**Risks.** Loss of subtle traces on recompressed images. No published study covers INT8 ViT detectors of modern generators.

**Cheapest first experiment.** Simulate INT8 by quantising and dequantising weights and activations inside the existing f32 engine. Count verdict flips on 10,000 images, including messenger copies and NTIRE distorted images, before writing any kernel.

### 10. Refresh for new generators cheaply, and measure decay chronologically

**Change.** Replace full-replay retraining for new generators with a short refresh phase. Use 500–3,000 images per new generator plus a fixed, stratified replay buffer of 50–200 images per older generator family and per real source, for 1–2k steps. Always follow it with `finalize`, which recalibrates on reals. Separately, run one chronological hold-out: train with a hypothetical cutoff and test on generators released after it. That turns README's "may drop after the cutoff" into a measured decay rate per six months.

**Evidence.** Community Forensics found that a few hundred images per systematically sampled generator suffice, and that performance plateaus at about 27k images at fixed diversity ([Community Forensics](https://arxiv.org/html/2411.04125)). DECODE found that forgetting in continual detection is driven largely by decision-head drift rather than feature loss. It reached **0.39% forgetting over 19 domains** ([DECODE](https://arxiv.org/abs/2607.27882)), which supports recalibrating after every refresh. Mining fact-check articles for in-the-wild data added **+9.14% and +8%** average accuracy on two detectors ([Pantsios et al.](https://arxiv.org/abs/2605.02567)). For video, under one minute of footage per new generator restored **AUC to 0.98 or above, including Sora (0.982)** ([Vahdati et al.](https://arxiv.org/html/2404.15955v1)). AI-GenBench defines the chronological protocol ([AI-GenBench](https://arxiv.org/pdf/2504.20865)). Few-shot metric learning reports gains from 10 samples, though its two arXiv versions disagree on the size (+11.6% vs +7.4%) ([FSD](https://arxiv.org/abs/2501.08763v2)).

**Expected effect on Tintype (extrapolated).** Recall on a newly released generator recovered within days, with older-generator recall preserved by replay and FPR held by recalibration.

**Cost.** Low per refresh on the Apple GPU. One extra training run for the chronological hold-out.

**Risks.** Calibration drift if `finalize` is skipped. Over-fitting to the support generator's content. A refreshed model is a new model, so it needs a new blind batch.

**Cheapest first experiment.** Pick the two or three generators in the WasItAIGenerated frontier set that Tintype misses most. Fine-tune 1.5k steps on 1,000 images of each plus a replay buffer. Check held-out recall on those generators and regression on batch 7.

## Where the notes disagree, and which side the evidence favours

Several disagreements change how a recommendation should be run. On **threshold transfer vs ranking**, note 08 says Tintype's FPR problem is mainly threshold transfer. The repo's own transfer experiment says both are broken, since FPR became fixable only by sacrificing nearly all recall. This report sides with the repo, which is why recommendations 1 and 2 are paired. On **fit-view order**, note 07 says score it last because it is resampled and fragile, and note 01 says score it first on recompressed inputs; recommendation 5 tests both by processing type. On **colour augmentation**, SPAI measured chromatic augmentation cutting AUC from 91.0 to 80.5 ([SPAI](https://arxiv.org/html/2411.19417v2)), while NTIRE's test set applies colour and tone distortions and SAFE used ColorJitter ([SAFE](https://arxiv.org/abs/2408.06741)). On **mixup**, DDA uses real/reconstruction pixel mixup, but B-Free found mixup and cutmix harmful (81.4 → 78.6). On **which encoder family is best** as a teacher, SSAFE favours multimodal encoders (PE-Core, SigLIP) ([SSAFE](https://arxiv.org/html/2606.08634v1)), while Huang et al. and the embedding-explanations paper favour DINOv3 ([Huang et al.](https://arxiv.org/html/2511.22471); [embedding-space explanations](https://arxiv.org/html/2611.04111v1)). That matters only for recommendation 7's optional extension. On **PEFT vs full fine-tuning**, Effort's gains under narrow training conflict with B-Free's and Community Forensics' broad-data results, which lowers confidence in recommendation 7. On **Stay-Positive**, notes 03 and 08 see a cheap last-layer win for messenger recall, while note 04 points out that its authors report upsampled reals flagged more often, which could worsen the NTIRE codec false positives ([Stay-Positive](https://arxiv.org/html/2502.07778v1)). Smaller bookkeeping conflicts: the LPT paper says TeleAI placed 3rd of 514 teams, while note 04 calls INTSIG's HEDGE "4th of 193". The Breaking Semantic Artifacts gain is 10.59% in the proceedings abstract and 6.74% on its poster page ([NeurIPS proceedings](https://proceedings.neurips.cc/paper_files/paper/2024/hash/6dddcff5b115b40c998a08fbd1cea4d7-Abstract.html)).

## Ideas considered and rejected

Most rejections come down to cost against Tintype's 2 ms-per-photo budget, or to 2025–26 independent evaluations that overturned a method's headline numbers.

| Idea | Why it is not in the ten |
|---|---|
| Hand-crafted low-level streams (NPR, LGrad, SRM/DCT branches, AIDE-style fusion) | Independent 2025 tables put them at 42–65% on current generators. NPR scores 42.7% average bAcc on B-Free's suite ([B-Free](https://arxiv.org/html/2412.17671v2)) and AIDE 54.1% average over DDA's 11 benchmarks ([DDA](https://arxiv.org/html/2505.14359v6)). None of the NTIRE top six used one. |
| Test-time reconstruction or inversion (DIRE, LaRE², FIRE, AEROBLADE, manifold bias) | One to three orders of magnitude slower; DIRE scores 50.8 AUC on SPAI's 13-generator set ([SPAI](https://arxiv.org/html/2411.19417v2)). Their lasting value is as training data (recommendation 4). |
| SPAI or ZED as a second stage | About three ViT-B passes per patch for SPAI, with self-declared compression fragility. ZED's coding cost is a pixel statistic that JPEG requantisation changes directly ([ZED](https://arxiv.org/abs/2409.15875)). |
| Token merging, pruning, FlexiViT coarse patches, efficient hybrid backbones | Tintype measured −6.2 and −14.0 recall points from token merging. ForensicZip explains why saliency-driven pruning discards the backgrounds where traces live ([ForensicZip](https://arxiv.org/abs/2603.12208)). Hybrid backbones need an engine rewrite and downsample early. |
| Swapping the backbone for ViT-B, or a DINOv3/SigLIP third opinion | About 3.8× compute per view, new RoPE/register kernels, a custom DINOv3 licence, and frozen DINOv3 still loses 12–18 points at JPEG quality 80–70 ([Huang et al.](https://arxiv.org/html/2511.22471)). Better reached through distillation (recommendation 7). |
| MLLM detectors at runtime (FakeVLM, AIGI-Holmes, Skyra, BusterX++) | Billions of parameters per item. A frozen PE-Core linear probe matched AIGI-Holmes on its own benchmark (99.9 vs 99.2) ([SSAFE](https://arxiv.org/html/2606.08634v1)). Skyra needed an asymmetric reward because its MLLM drifted toward "fake" on real footage ([Skyra](https://arxiv.org/html/2512.15693v2)). |
| Stay-Positive non-negative head | Cheap, and promising for messenger recall (FLUX AP 57 → 99 on Corvi's detector), but its authors report upsampled reals flagged more often ([Stay-Positive](https://arxiv.org/html/2502.07778v1)). Deferred until recommendations 1 and 2 have cut the codec false positives, then worth a head-only trial. |
| Max pooling for the fully-generated head | Average pooling beat max on every LaDeDa benchmark ([LaDeDa](https://arxiv.org/html/2406.09398v1)). Top-k belongs only in a separately calibrated local-edit or face head. |
| Temporal video heads (optical flow, D3, ReStraV, V-PVP) | Flow and trajectory statistics rely on fps and motion shortcuts and collapse on motion-balanced data ([Michels et al.](https://arxiv.org/html/2607.00948v2)). V-PVP's 95.28 AUC on AIGVDBench ([V-PVP](https://arxiv.org/html/2607.15321v2)) is the one credible temporal result, but it used a video-pretrained backbone and is untested on an image-pretrained ViT-S. Tintype's video problem today is precision, not recall. A candidate after recommendation 6. |
| Multi-block (RINE-style) readout and depth-wise early exit | RINE's +9 accuracy points came from frozen CLIP ([RINE](https://arxiv.org/html/2402.19091v2)). For a fully fine-tuned ViT-S the gap is likely small, and depth exits fragment the GPU batch scheduler. A cheap side experiment, not a top-ten change. |
| Stage-0 distilled ViT-Ti cascade | Plausible 1.3–1.7× speed-up by cascade logic ([Wisdom of Committees](https://arxiv.org/abs/2012.01988)), but Tintype already exits after 1.7 views on average. Recommendations 3, 5 and 7 attack the same cost more cheaply. |
| Generic or per-image test-time adaptation (TENT, SAR, EATA, IAPL) | TENT-family methods fell to or below the unadapted detector, and the transductive TTC method needs about 90% pseudo-label accuracy and class-balanced batches that a mostly-real stream cannot supply ([TTC](https://arxiv.org/html/2608.00559)). |
| Unsupervised symmetric-density recalibration (AAAI-26) | It assumes reals stay stable and the logit density is bimodal. Tintype's degraded reals shift, and a mostly-real stream is not bimodal ([Yang et al.](https://arxiv.org/html/2602.01973)). The supervised variant is what recommendation 2 does. |
| User-facing "unknown generator" verdicts and attribution heads | Held-out DALL·E 3 and Midjourney 6 evade open-set rejection and get assigned to known generators ([black-box attribution](https://arxiv.org/pdf/2608.15652)). Useful internally for triage only. |
| Audio-visual cues for video with sound | The best cross-pattern F1 on MVAD is 68.7% ([MVAD](https://arxiv.org/html/2512.00336v4)). Physics checks reach about 70–72% ([PIVOT](https://arxiv.org/abs/2609.15562)). |
| Adversarial training | Open weights make Tintype white-box. Carlini and Farid drove AUC to 0.0005 with lowest-bit flips ([Carlini & Farid](https://openaccess.thecvf.com/content_CVPRW_2020/papers/w39/Carlini_Evading_Deepfake-Image_Detectors_With_White-_and_Black-Box_Attacks_CVPRW_2020_paper.pdf)). RAID belongs in CI as a regression test ([RAID](https://arxiv.org/abs/2506.03988)), not in training. |

## Conclusion

The literature reframes Tintype's problem. Its headline numbers come from a recipe the field broadly endorses: a plain end-to-end ViT, broad generator coverage and native-resolution crops. What the 2025–26 evidence adds is that robustness is a *data and calibration* property, not an architectural one. The NTIRE winners, B-Free and DDA did not add a smarter feature; they made the classes indistinguishable in everything except the generator trace, then trained through the corruptions they expected. For Tintype, the most useful immediate result may come from a diagnostic rather than a model. The per-operation shortcut audit in recommendation 1 and the stratified re-thresholding in recommendation 2 can both run this week on existing tooling and archived scores. Together they will show how much of the NTIRE failure is codec-specific and how much is general degradation.

The second implication concerns what Tintype claims. A single source-averaged FPR is a marginal guarantee over the calibration mixture, and the NTIRE run showed it can be 17× wrong on a shifted source. Per-stratum FPR with upper bounds, an UNSURE verdict, and per-collection video FPR would make the README's precision claims checkable on the traffic operators actually see. That is also the honest baseline against which every other change in this report should be judged.

## References

The recommendations rest mainly on the sources cited inline above. The full deduplicated bibliography compiled from research notes 01–08 is reproduced in the appendix. It lists **231 distinct academic papers, reports and benchmarks: 219 with an arXiv ID (all resolved on export.arxiv.org on 2026-11-13) and 12 cited only by venue, DOI or proceedings link**. Software, project pages, model cards, blogs and challenge pages are listed separately and not counted. Two arXiv IDs in the notes were corrected: SPSL is 2103.01856 (not 2103.01412), and SPAI is 2411.19417 (not 2507.10236). DEAR's first author is Dahye Kim. The "DINOv3 paper" in this report means Huang et al., arXiv 2511.22471, not Meta's backbone paper.

Repository evidence is cited by path and line, following the fact sheet `00_codebase_facts.md`: `src/data.rs`, `src/trainer.rs`, `src/eval.rs`, `src/detector.rs`, `src/image_io.rs`, `src/video.rs`, `src/yuv.rs`, `src/cpu_vit.rs`, `src/scan.rs`, `src/server.rs`, `models/tintype.json`, `models/tintype-video.json`, `README.md`, `docs/BENCHMARK.md`, `docs/EVIDENCE.md`, `docs/DATA.md`, `docs/PREREGISTRATION.md` and `docs/RESEARCH.md`.

## Appendix: full bibliography (231 distinct papers)

The tables below are reproduced from `99_bibliography.md`, grouped by topic area A–Q and sorted by year within each area. "Cited in [tag]" gives the note number and that note's verification tag (V = numbers read from the paper; A/R = recalled or abstract-level; rec = recalled, not re-verified; fetched/search/abs = note 07's levels; u = untagged default of primary-page checking; snip = search snippet only; sec = secondary source; ctx = context only; lead = unverified lead). "Check" gives the arXiv API result.

### A. Low-level, frequency and artifact signals (image)

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 1 | AutoGAN: "Detecting and Simulating Artifacts in GAN Fake Images" | Zhang, Karaman & Chang | WIFS | 2019 | [arXiv 1907.06515](https://arxiv.org/abs/1907.06515) | 01[A] | OK |
| 2 | CNNSpot: "CNN-generated images are surprisingly easy to spot… for now" | S.-Y. Wang et al. | CVPR | 2020 | [arXiv 1912.11035](https://arxiv.org/abs/1912.11035) | 01[A], 03[u] | OK |
| 3 | "Leveraging Frequency Analysis for Deep Fake Image Recognition" | Frank et al. | ICML | 2020 | [arXiv 2003.08685](https://arxiv.org/abs/2003.08685) | 01[A] | OK |
| 4 | "Watch your Up-Convolution…" | Durall, Keuper & Keuper | CVPR | 2020 | [arXiv 2003.01826](https://arxiv.org/abs/2003.01826) | 01[A] | OK |
| 5 | Patch Forensics: "What makes fake images detectable?" | Chai et al. | ECCV | 2020 | [arXiv 2008.10588](https://arxiv.org/abs/2008.10588) | 06[u] | OK |
| 6 | SPSL: "Spatial-Phase Shallow Learning…" | H. Liu et al. | CVPR | 2021 | notes: arXiv 2103.01412 | 01[A] | **CORRECTED → 2103.01856** |
| 7 | BiHPF: "Bilateral High-Pass Filters for Robust Deepfake Detection" | Jeong et al. | WACV | 2022 | [CVF](https://openaccess.thecvf.com/content/WACV2022/html/Jeong_BiHPF_Bilateral_High-Pass_Filters_for_Robust_Deepfake_Detection_WACV_2022_paper.html) | 01[A] | CONFIRMED (arXiv 2109.00911) |
| 8 | "On the detection of synthetic images generated by diffusion models" | Corvi et al. | ICASSP | 2023 | [arXiv 2211.00680](https://arxiv.org/abs/2211.00680) | 01[A] | OK |
| 9 | "Intriguing properties of synthetic images: from GANs to diffusion" | Corvi et al. | CVPRW | 2023 | [arXiv 2304.06408](https://arxiv.org/abs/2304.06408) | 01[A] | OK |
| 10 | LGrad: "Learning on Gradients…" | C. Tan et al. | CVPR | 2023 | [CVF](https://openaccess.thecvf.com/content/CVPR2023/html/Tan_Learning_on_Gradients_Generalized_Artifacts_Representation_for_GAN-Generated_Images_Detection_CVPR_2023_paper.html) | 01[A; third-party numbers V] | n/c |
| 11 | Synthbuster | Bammey | IEEE OJSP | 2023 | [DOI 10.1109/OJSP.2023.3337714](https://doi.org/10.1109/OJSP.2023.3337714) | 01[A] | CONFIRMED (web) |
| 12 | PatchCraft (earlier title "Rich and Poor Texture Contrast") | Zhong et al. | arXiv | 2023 | [arXiv 2311.12397](https://arxiv.org/abs/2311.12397) | 01[A/V], 07[fetched] | OK |
| 13 | NPR: "Rethinking the Up-Sampling Operations…" | C. Tan et al. | CVPR | 2024 | [arXiv 2312.10461](https://arxiv.org/abs/2312.10461) | 01[A/V] | OK |
| 14 | FreqNet: "Frequency-Aware Deepfake Detection…" | C. Tan et al. | AAAI | 2024 | [arXiv 2403.07240](https://arxiv.org/abs/2403.07240) | 01[A] | OK |
| 15 | "Towards the Detection of Diffusion Model Deepfakes" | Ricker, Damm, Holz & Fischer | VISAPP | 2024 | [arXiv 2210.14571](https://arxiv.org/abs/2210.14571) | 01[A] | OK |
| 16 | "Frequency Masking for Universal Deepfake Detection" | Doloriel & Cheung | ICASSP | 2024 | (no link) | 03[lead] | CONFIRMED (arXiv 2401.06506) |
| 17 | SAFE: "Improving Synthetic Image Detection Towards Generalization: An Image Transformation Perspective" | O. Li et al. | KDD | 2025 | [arXiv 2408.06741](https://arxiv.org/abs/2408.06741) | 01[V], 04[u] | OK |

### B. Reconstruction, zero-shot, training-free and real-only detectors

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 18 | DIRE | Z. Wang et al. | ICCV | 2023 | [arXiv 2303.09295](https://arxiv.org/abs/2303.09295) | 01[A/V] | OK |
| 19 | AEROBLADE | Ricker, Lukovnikov & Fischer | CVPR | 2024 | [arXiv 2401.17879](https://arxiv.org/abs/2401.17879) | 01[A] | OK |
| 20 | LaRE²: "Latent Reconstruction Error…" | Luo et al. | CVPR | 2024 | [arXiv 2403.17465](https://arxiv.org/abs/2403.17465) | 01[A] | OK |
| 21 | DRCT: "Diffusion Reconstruction Contrastive Training…" (DRCT-2M) | B. Chen et al. | ICML (PMLR v235) | 2024 | [PMLR](https://proceedings.mlr.press/v235/chen24ay.html) | 01[A/V], 03[u] | n/c |
| 22 | RIGID | He, Chen & Ho | arXiv | 2024 | [arXiv 2405.20112](https://arxiv.org/pdf/2405.20112) | 01[V] | OK |
| 23 | ZED: "Zero-Shot Detection of AI-Generated Images" | Cozzolino et al. | ECCV | 2024 | [arXiv 2409.15875](https://arxiv.org/abs/2409.15875) | 01[V], 08[u] | OK |
| 24 | SPAI: "Any-Resolution AI-Generated Image Detection by Spectral Learning" | Karageorgiou et al. | CVPR | 2025 | [arXiv 2411.19417](https://arxiv.org/abs/2411.19417) | 01[V], 02[R; ID "not confirmed"], 03[u], 07[fetched], 08[u] | **CORRECTED for 02** (OK elsewhere) |
| 25 | FIRE: "Frequency-Guided Reconstruction Error" | Chu et al. | CVPR | 2025 | [arXiv 2412.07140](https://arxiv.org/abs/2412.07140) | 01[A] | OK |
| 26 | "Manifold Induced Biases for Zero-shot and Few-shot Detection of Generated Images" | Brokman et al. | ICLR | 2025 | [OpenReview](https://openreview.net/forum?id=7gGl6HB5Zd) (01); [arXiv 2504.15470](https://arxiv.org/abs/2504.15470) (08) | 01[A], 08[u] | CONFIRMED |

### C. Foundation-model and representation-based detectors (CLIP / DINO / PEFT / fusion)

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 27 | UnivFD: "Towards Universal Fake Image Detectors…" | Ojha et al. | CVPR | 2023 | [arXiv 2302.10174](https://arxiv.org/abs/2302.10174) | 02[R/V] | OK |
| 28 | "Raising the Bar of AI-generated Image Detection with CLIP" | Cozzolino et al. | CVPRW (WMF) | 2024 | [arXiv 2312.00195](https://arxiv.org/abs/2312.00195); [CVF](https://openaccess.thecvf.com/content/CVPR2024W/WMF/papers/Cozzolino_Raising_the_Bar_of_AI-generated_Image_Detection_with_CLIP_CVPRW_2024_paper.pdf) | 01[ctx], 02[R], 03[u], 04[u] | OK |
| 29 | FatFormer: "Forgery-aware Adaptive Transformer…" | H. Liu et al. | CVPR | 2024 | [arXiv 2312.16649](https://arxiv.org/abs/2312.16649) | 01[A/V], 02[R/V] | OK |
| 30 | "CLIPping the Deception…" | Khan & Dang-Nguyen | ICMR | 2024 | [arXiv 2402.12927](https://arxiv.org/abs/2402.12927) | 02[R] | OK |
| 31 | RINE: "Leveraging Representations from Intermediate Encoder-blocks…" | Koutlis & Papadopoulos | ECCV | 2024 | [arXiv 2402.19091](https://arxiv.org/html/2402.19091v2) | 02[V] | OK |
| 32 | C2P-CLIP: "Injecting Category Common Prompt into CLIP…" | C. Tan et al. | AAAI | 2025 | [arXiv 2408.09647](https://arxiv.org/abs/2408.09647) | 02[R/V] | OK |
| 33 | D³: "Scaling Up Deepfake Detection by Learning from Discrepancy" | Y. Yang et al. | CVPR | 2025 | [arXiv 2404.04584](https://arxiv.org/abs/2404.04584) | 02[R] | OK |
| 34 | AIDE + Chameleon: "A Sanity Check for AI-generated Image Detection" | S. Yan et al. | ICLR | 2025 | [arXiv 2406.19435](https://arxiv.org/abs/2406.19435) | 01[V], 02[R/V/snip], 03[u], 07[search], 08[u] | OK |
| 35 | Effort: "Orthogonal Subspace Decomposition for Generalizable AI-Generated Image Detection" | Z. Yan et al. | ICML (per notes; arXiv lists none) | 2025 | [arXiv 2411.15633](https://arxiv.org/abs/2411.15633) | 02[V], 06[u] | OK |
| 36 | CO-SPY: "Combining Semantic and Pixel Features…" | Cheng et al. | CVPR | 2025 | [arXiv 2503.18286](https://arxiv.org/abs/2503.18286) | 02[R] | OK |
| 37 | "Rethinking Cross-Generator Image Forgery Detection through DINOv3" (02's "DINOv3 paper") | Z. Huang et al. | arXiv | 2025 | [arXiv 2511.22471](https://arxiv.org/abs/2511.22471) | 02[V; one R] | OK |
| 38 | GAPL: "Scaling Up AI-Generated Image Detection with Generator-Aware Prototypes" | Qin et al. | arXiv | 2025 | [arXiv 2512.12982](https://arxiv.org/abs/2512.12982) | 02[V] | OK |
| 39 | DGS-Net: "Distillation-Guided Gradient Surgery for CLIP Fine-Tuning…" | J. Yan et al. | ICML (Spotlight, per notes) | 2026 | [arXiv 2511.13108](https://arxiv.org/abs/2511.13108) | 02[V] | OK |
| 40 | SPECTRA-Net | Arab et al. | arXiv | 2026 | [arXiv 2605.08226](https://arxiv.org/abs/2605.08226) | 04[snip] | OK |
| 41 | SSAFE: "Simple and Strong AI-Generated Image Detection via Frozen Vision Encoders" | S. Lee et al. | arXiv | 2026 | [arXiv 2606.08634](https://arxiv.org/abs/2606.08634) | 02[V; one R] | OK |
| 42 | "Anchor-Regularized Adaptation… with DINOv3" | H. Choi et al. | arXiv | 2026 | [arXiv 2608.15196](https://arxiv.org/abs/2608.15196) | 02[snip] | OK |
| 43 | "Generalizable AI-Generated Image Detection with Embedding-Space Explanations" | N. Giakoumoglou et al. | arXiv | 2026 | [arXiv 2611.04111](https://arxiv.org/abs/2611.04111) | 02[snip] | CONFIRMED |

### D. MLLM / VLM explainable detection (image)

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 44 | "Can ChatGPT Detect DeepFakes?…" | S. Jia et al. | CVPRW | 2024 | [arXiv 2403.14077](https://arxiv.org/abs/2403.14077) | 02[R] | OK |
| 45 | FakeBench | Yixuan Li et al. | arXiv | 2024 | [arXiv 2404.13306](https://arxiv.org/abs/2404.13306) | 02[R] | OK |
| 46 | LOKI | J. Ye et al. | ICLR (spotlight) | 2025 | [arXiv 2410.09732](https://arxiv.org/abs/2410.09732) | 02[sec], 05[u] | OK |
| 47 | FakeShield | Z. Xu et al. | ICLR | 2025 | [arXiv 2410.02761](https://arxiv.org/abs/2410.02761) | 02[R], 06[u/sec] | OK |
| 48 | HEIE | F. Yang et al. | CVPR | 2025 | [arXiv 2411.17261](https://arxiv.org/abs/2411.17261) | 02[R] | OK |
| 49 | SIDA (SID-Set) | Z. Huang et al. | CVPR | 2025 | [arXiv 2412.04292](https://arxiv.org/abs/2412.04292) | 02[R], 06[u] | OK |
| 50 | FakeVLM: "Spot the Fake…" (FakeClue) | S. Wen et al. | NeurIPS | 2025 | [arXiv 2503.14905](https://arxiv.org/abs/2503.14905) | 02[u] | OK |
| 51 | LEGION | H. Kang et al. | ICCV | 2025 | [arXiv 2503.15264](https://arxiv.org/abs/2503.15264) | 02[R] | OK |
| 52 | So-Fake / So-Fake-R1 (So-Fake-Set, So-Fake-OOD) | Z. Huang et al. | arXiv | 2025 | [arXiv 2505.18660](https://arxiv.org/abs/2505.18660) | 02[u], 08[u] | OK |
| 53 | AIGI-Holmes | Z. Zhou et al. | ICCV | 2025 | [arXiv 2507.02664](https://arxiv.org/abs/2507.02664) | 02[u] | OK |
| 54 | ThinkFake | T.-M. Huang et al. | arXiv | 2025 | [arXiv 2509.19841](https://arxiv.org/abs/2509.19841) | 02[u] | OK |
| 55 | Mirage | P. Sharma et al. | arXiv | 2025 | [arXiv 2510.03840](https://arxiv.org/pdf/2510.03840) | 02[snip] | OK |
| 56 | Face-manipulation MLLM study (GPT-4, Gemini 2.5 Pro, DeepSeek) | not given | TechRxiv | 2025 | [TechRxiv](https://www.techrxiv.org/doi/pdf/10.36227/techrxiv.175459536.67892800) | 02[u] | n/c |
| 57 | "Dissecting Agentic Forensics…" | X. Li et al. | arXiv | 2026 | [arXiv 2609.24359](https://arxiv.org/abs/2609.24359) | 06[sec] | OK |

### E. Training data, dataset bias, alignment and scaling

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 58 | "Fake or JPEG? Revealing Common Biases in Generated Image Detection Datasets" | Grommelt et al. | ECCV Workshops | 2024 | [arXiv 2403.17608](https://arxiv.org/abs/2403.17608) | 01[A], 03[u], 04[u], 08[u] | OK |
| 59 | FakeInversion | Cazenavette et al. | CVPR | 2024 | [arXiv 2406.08603](https://arxiv.org/abs/2406.08603) | 03[u] | OK |
| 60 | "Breaking Semantic Artifacts for Generalized AI-generated Image Detection" | C. Zheng et al. | NeurIPS | 2024 | [NeurIPS](https://proceedings.neurips.cc/paper_files/paper/2024/hash/6dddcff5b115b40c998a08fbd1cea4d7-Abstract.html) | 03[u], 07[search] | n/c |
| 61 | Aligned Datasets / AlignedForensics ("On the Effectiveness of Dataset Alignment…") | Sundara Rajan et al. | ICLR | 2025 | [arXiv 2410.11835](https://arxiv.org/abs/2410.11835) | 01[A/V], 02[R/V], 03[u], 08[u] | OK |
| 62 | Community Forensics | Park & Owens | CVPR | 2025 | [arXiv 2411.04125](https://arxiv.org/abs/2411.04125) | 02[V], 03[u], 07[ctx] | CONFIRMED |
| 63 | B-Free: "A Bias-Free Training Paradigm…" | Guillaro et al. | CVPR | 2025 | [arXiv 2412.17671](https://arxiv.org/abs/2412.17671) | 01[V], 02[V], 03[u], 04[u], 06[u], 07[search], 08[u] | CONFIRMED |
| 64 | Stay-Positive | Sundara Rajan & Lee | ICML (PMLR v267) | 2025 | [arXiv 2502.07778](https://arxiv.org/abs/2502.07778) | 03[u; venue "not confirmed"], 04[u], 08[u] | OK |
| 65 | DDA: "Dual Data Alignment…" | R. Chen et al. | NeurIPS (Spotlight) | 2025 | [arXiv 2505.14359](https://arxiv.org/abs/2505.14359) | 01[V], 02[V] | CONFIRMED |
| 66 | "Navigating the Challenges of AI-Generated Image Detection in the Wild" (ITW-SM) | Konstantinidou et al. | arXiv (Pattern Recognition per 03) | 2025 | [arXiv 2507.10236](https://arxiv.org/abs/2507.10236) | 02[ctx; mislinked as SPAI], 03[u], 08[u] | OK |
| 67 | OpenFake | Livernoche et al. | arXiv | 2025 | [arXiv 2509.09495](https://arxiv.org/abs/2509.09495) | 03[u], 08[u] | OK |
| 68 | ScaleDF: "Scaling Laws for Deepfake Detection" | W. Wang et al. | arXiv / OpenReview | 2025 | [arXiv 2510.16320](https://arxiv.org/abs/2510.16320) | 02[u], 03[u] | OK |
| 69 | LongCat-Image Technical Report | Meituan LongCat Team | tech report | 2025 | [arXiv 2512.07584](https://arxiv.org/pdf/2512.07584) | 03[u] | OK |
| 70 | LAION-Mobile | von Stryk & Keuper | arXiv | 2026 | [arXiv 2609.11134](https://arxiv.org/abs/2609.11134) | 03[u] | OK |

### F. Continual, few-shot and test-time adaptation

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 71 | OST: "One-Shot Test-Time Training" | L. Chen et al. | NeurIPS | 2022 | [NeurIPS](https://proceedings.neurips.cc/paper_files/paper/2022/hash/9bf0810a4a1597a36d27ceea58667d92-Abstract.html) | 08[u] | CONFIRMED (web) |
| 72 | CDDB: "A Continual Deepfake Detection Benchmark" | C. Li et al. | WACV | 2023 | [arXiv 2205.05467](https://arxiv.org/abs/2205.05467) | 03[u] | OK |
| 73 | E3: "Ensemble of Expert Embedders…" | Azizpour et al. | CVPRW (WMF) | 2024 | [arXiv 2404.08814](https://arxiv.org/abs/2404.08814) | 03[u] | OK |
| 74 | FSD: "Few-Shot Learner Generalizes Across AI-Generated Image Detection" | S. Wu et al. | ICML (PMLR v267) | 2025 | [arXiv 2501.08763](https://arxiv.org/abs/2501.08763) | 03[u], 08[u] | OK |
| 75 | T2A: "Think Twice before Adaptation…" | Nguyen-Le et al. | IJCAI | 2025 | [arXiv 2505.18787](https://arxiv.org/abs/2505.18787) | 08[u] | OK |
| 76 | IAPL: "…Image-Adaptive Prompt Learning" | Yiheng Li et al. | CVPR | 2026 | [arXiv 2508.01603](https://arxiv.org/abs/2508.01603) | 08[u] | OK |
| 77 | "Generalizable and Adaptive Continual Learning Framework…" | H. Wang et al. | arXiv | 2026 | [arXiv 2601.05580](https://arxiv.org/abs/2601.05580) | 03[snip] | OK |
| 78 | "Automated In-the-Wild Data Collection for Continual AI Generated Image Detection" | Pantsios et al. | arXiv (ACM DOI given) | 2026 | [arXiv 2605.02567](https://arxiv.org/abs/2605.02567) | 03[u] | OK |
| 79 | DECODE | Z. Cai et al. | arXiv | 2026 | [arXiv 2607.27882](https://arxiv.org/abs/2607.27882) | 03[u] | OK |
| 80 | TTC: "Test-Time Curriculum for Open-Set AIGC Detection" | Y. Zhang et al. | arXiv | 2026 | [arXiv 2608.00559](https://arxiv.org/abs/2608.00559) | 08[u] | CONFIRMED |
| 81 | EvoKnow | Z. Peng et al. | arXiv | 2026 | [arXiv 2610.11381](https://arxiv.org/html/2610.11381) | 03[snip] | OK |

### G. Robustness to degradation, social-network laundering, and robustness challenges

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 82 | SIDBench | Schinas & Papadopoulos | arXiv | 2024 | [arXiv 2404.18552](https://arxiv.org/abs/2404.18552) | 03[u] | OK |
| 83 | LaDeDa / Tiny-LaDeDa / WildRF: "Real-Time Deepfake Detection in the Real-World" | Cavia et al. | arXiv | 2024 | [arXiv 2406.09398](https://arxiv.org/abs/2406.09398) | 03[u], 04[u], 06[u], 07[search] | OK |
| 84 | "When Synthetic Traces Hide Real Content: …SD Image Laundering" | Mandelli et al. | WIFS (per authors) | 2024 | [arXiv 2407.10736](https://arxiv.org/pdf/2407.10736) | 04[u] | OK |
| 85 | "Is JPEG AI going to change image forensics?" | Cannas et al. | arXiv | 2024 | [arXiv 2412.03261](https://arxiv.org/html/2412.03261v2) | 04[u] | OK |
| 86 | DFCon (IEEE SP Cup 2025 entry) | Shanto et al. | arXiv | 2025 | [arXiv 2501.16704](https://arxiv.org/abs/2501.16704) | 04[u] | OK |
| 87 | TrueFake | Dell'Anna et al. | IJCNN | 2025 | [arXiv 2504.20658](https://arxiv.org/abs/2504.20658) | 03[u], 04[u] | OK |
| 88 | AIGIBench: "Is AIGI Detection a Solved Problem?" | Ziqiang Li et al. | NeurIPS D&B | 2025 | [NeurIPS](https://proceedings.neurips.cc/paper_files/paper/2025/hash/fb693c67f61e5321746ffce8b6fdd2d0-Abstract-Datasets_and_Benchmarks_Track.html); arXiv 2505.12335 | 04[u] | OK |
| 89 | RRDataset: "Bridging the Gap Between Ideal and Real-world Evaluation…" | Chunxiao Li et al. | ICCV | 2025 | [CVF](https://openaccess.thecvf.com/content/ICCV2025/papers/Li_Bridging_the_Gap_Between_Ideal_and_Real-world_Evaluation_Benchmarking_AI-Generated_ICCV_2025_paper.pdf); arXiv 2509.09172 | 04[u] | OK |
| 90 | HiDA-Net: "No Pixel Left Behind…" | Mu et al. | ICLR | 2026 | [ICLR](https://iclr.cc/virtual/2026/poster/10011114); arXiv 2508.17346 | 01[A, snip], 04[u] | OK |
| 91 | NTIRE 2026 Challenge on Robust AIGI Detection in the Wild (report) | Gushchin et al. | CVPRW (NTIRE) | 2026 | [arXiv 2604.11487](https://arxiv.org/abs/2604.11487) | 01[V], 02[u], 03[u], 04[u] | CONFIRMED |
| 92 | HEDGE (NTIRE 2026 entry) | F. Wu et al. | arXiv | 2026 | [arXiv 2604.03555](https://arxiv.org/abs/2604.03555) | 04[u] | OK |
| 93 | DCPT: "Degradation-Consistent Paired Training…" | Z. Yang et al. | arXiv | 2026 | [arXiv 2604.10102](https://arxiv.org/html/2604.10102v1) | 04[u] | CONFIRMED |
| 94 | LPT: "…LoRA-based Pairwise Training" (NTIRE 3rd) | Xia et al. | CVPRW | 2026 | [arXiv 2604.12307](https://arxiv.org/html/2604.12307v1) | 04[u] | CONFIRMED |
| 95 | QuAD: "Quality-Aware Calibration…" | Guillaro et al. | CVPRW (APAI, per arXiv) | 2026 | [arXiv 2604.15027](https://arxiv.org/html/2604.15027v1) | 03[u], 04[u] | CONFIRMED |
| 96 | "Towards Generalizable Deepfake Image Detection with Vision Transformers" (claimed DFWild-Cup winner) | Srinanda et al. | arXiv | 2026 | [Fraunhofer related-work page](https://deepfake-demo.aisec.fraunhofer.de/related_work/2604.17376) (arXiv 2604.17376) | 04[u; ranking unconfirmed] | OK |
| 97 | DACOM: "Enabling Your Forensic Detector Know How Well It Performs on Distorted Samples" | authors not retrieved | ICLR | 2026 | [ICLR](https://iclr.cc/virtual/2026/poster/10010162); [OpenReview PDF](https://openreview.net/pdf/9a9ad3cc252735dcd9be4a445b8f7f984d73a689.pdf) | 04[u; figure sec] | CONFIRMED (web) |
| 98 | BIAS-ID | Ricker, Fischer & Quiring | NeurIPS (E&D track) | 2026 | [arXiv 2605.31153](https://arxiv.org/abs/2605.31153) | 04[u] | CONFIRMED |
| 99 | DEAR: "Dissect and Prune…" | **D. Kim**, J. Choi et al. | ICML (per arXiv) | 2026 | [HF papers](https://huggingface.co/papers/2606.10309) (arXiv 2606.10309) | 04[u; own numbers not retrieved] | CONFIRMED (first author corrected) |
| 100 | GlobalForge / RealDeg-Bench | M. Cui et al. | arXiv | 2026 | [arXiv 2607.14684](https://arxiv.org/abs/2607.14684) | 04[u] | OK |

### H. Adversarial robustness, watermarking and provenance

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 101 | "Evading Deepfake-Image Detectors with White- and Black-Box Attacks" | Carlini & Farid | CVPRW | 2020 | [CVF](https://openaccess.thecvf.com/content_CVPRW_2020/papers/w39/Carlini_Evading_Deepfake-Image_Detectors_With_White-_and_Black-Box_Attacks_CVPRW_2020_paper.pdf); arXiv 2004.00622 | 04[u] | OK |
| 102 | "Robustness of AI-Image Detectors: Fundamental Limits and Practical Attacks" | Saberi et al. | ICLR | 2024 | [ICLR](https://proceedings.iclr.cc/paper_files/paper/2024/hash/744e09d3213ee53de88f04164a9b06ab-Abstract-Conference.html); arXiv 2310.00076 | 04[u] | OK |
| 103 | RAW: "A Robust and Agile Plug-and-Play Watermark Framework…" | Xian et al. | NeurIPS | 2024 | [arXiv 2403.18774](https://arxiv.org/abs/2403.18774) | 08[u] | OK |
| 104 | "AI-generated Image Detection: Passive or Watermark?" | M. Guo et al. | arXiv | 2024 | [arXiv 2411.13553](https://arxiv.org/abs/2411.13553) | 08[u] | OK |
| 105 | UnMarker | Kassis & Hengartner | IEEE S&P | 2025 | [arXiv 2405.08363](https://arxiv.org/html/2405.08363v2) | 04[u; SynthID figure sec] | OK |
| 106 | RAID: adversarial-robustness dataset | Eddoubi et al. | arXiv (submitted to NeurIPS D&B) | 2025 | [arXiv 2506.03988](https://arxiv.org/abs/2506.03988) | 04[u] | CONFIRMED |
| 107 | SynthID-Image | Gowal et al. (Google DeepMind) | tech report | 2025 | [arXiv 2510.09263](https://arxiv.org/pdf/2510.09263) | 08[u] | OK |
| 108 | "Adversarial Robustness of AI-Generated Image Detectors in the Real World" (v1 "Fake It Until You Break It") | Mavali et al. | DIMVA | 2026 | [arXiv 2410.01574](https://arxiv.org/abs/2410.01574) | 04[u] | OK |
| 109 | "The Deployment Gap in AI Media Detection…" | Budhkar et al. | CVPRW (AIMS) | 2026 | [arXiv 2604.09706](https://arxiv.org/abs/2604.09706) | 04[u] | OK |

### I. Patch and view selection, native resolution, and aggregation

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 110 | "A Full-Image Full-Resolution End-to-End-Trainable CNN Framework for Image Forgery Detection" | Marra et al. | arXiv | 2019 | [arXiv 1909.06751](https://arxiv.org/abs/1909.06751) | 06[u] | OK |
| 111 | "Fusing Global and Local Features for Generalized AI-Synthesized Image Detection" | Ju et al. | ICIP | 2022 | [arXiv 2203.13964](https://arxiv.org/abs/2203.13964) | 07[search] | OK |
| 112 | FlexiViT | Beyer et al. | CVPR | 2023 | [arXiv 2212.08013](https://arxiv.org/abs/2212.08013) | 07[abs] | OK |
| 113 | NaViT: "Patch n' Pack" | Dehghani et al. | NeurIPS | 2023 | [arXiv 2307.06304](https://arxiv.org/abs/2307.06304) | 07[abs] | OK |
| 114 | SSP: "A Single Simple Patch is All You Need…" | J. Chen et al. | arXiv | 2024 | [arXiv 2402.01123](https://arxiv.org/abs/2402.01123) | 01[via review], 07[search] | OK |
| 115 | TextureCrop | Konstantinidou et al. | arXiv | 2024 | [arXiv 2407.15500](https://arxiv.org/abs/2407.15500) | 01[V], 04[u], 06[u] | OK |
| 116 | Panoptic Patch Learning: "All Patches Matter, More Patches Better" | Z. Yang et al. | ICLR (per 07) | 2026 | [arXiv 2504.01396](https://arxiv.org/abs/2504.01396); [ICLR](https://iclr.cc/virtual/2026/poster/10007395) | 01[V], 07[fetched+search] | OK |
| 117 | WaRPAD: "Training-free Detection of AI-generated images via Cropping Robustness" | S. Choi et al. | arXiv | 2025 | [alphaXiv 2511.14030](https://www.alphaxiv.org/abs/2511.14030) | 07[search] | OK |
| 118 | "Attention to Detail: Global-Local Attention for High-Resolution AIGI Detection" | L. Han | arXiv | 2026 | [arXiv 2601.00141](https://arxiv.org/html/2601.00141v1) | 01[A, title+snip] | OK |
| 119 | MDMF: "Micro-Defects Expose Macro-Fakes" | B. Zhang et al. | arXiv | 2026 | [arXiv 2605.09296](https://arxiv.org/html/2605.09296v1) | 06[sec] | OK |
| 120 | PatchHead | S. Qi et al. | arXiv | 2026 | [arXiv 2608.09223](https://arxiv.org/html/2608.09223v1) | 06[sec] | OK |
| 121 | "Structured Local Differential Modeling for AIGI Detection" | J. Yang et al. | arXiv | 2026 | [arXiv 2608.12811](https://arxiv.org/pdf/2608.12811) | 01[A, snip] | OK |
| 122 | "Look Closer: Patch-wise Supervision…" | Z. Zhang et al. | arXiv | 2026 | [arXiv 2609.37937](https://arxiv.org/abs/2609.37937) | 06[u] | CONFIRMED |

### J. Efficiency: backbones, early exit, token reduction, quantization, distillation, ensembles and fusion

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 123 | DeiT: "Training data-efficient image transformers…" | Touvron et al. | ICML | 2021 | [arXiv 2012.12877](https://arxiv.org/abs/2012.12877) | 02[R] | OK |
| 124 | "Adaptive Inference through Early-Exit Networks" (survey) | Laskaridis et al. | EMDL | 2021 | [arXiv 2106.05022](https://arxiv.org/abs/2106.05022) | 07[search] | OK |
| 125 | DynamicViT | Rao et al. | NeurIPS | 2021 | [arXiv 2106.02034](https://arxiv.org/abs/2106.02034) | 07[abs] | OK |
| 126 | Wisdom of Committees | X. Wang et al. | ICLR | 2022 | [arXiv 2012.01988](https://arxiv.org/abs/2012.01988) | 07[search] | OK |
| 127 | A-ViT (arXiv title "AdaViT: Adaptive Tokens…") | Yin et al. | CVPR | 2022 | [arXiv 2112.07658](https://arxiv.org/abs/2112.07658) | 07[abs] | OK |
| 128 | EViT: "Not All Patches are What You Need" | Liang et al. | ICLR | 2022 | [arXiv 2202.07800](https://arxiv.org/abs/2202.07800) | 07[abs] | OK |
| 129 | MobileViT | Mehta & Rastegari | ICLR | 2022 | [arXiv 2110.02178](https://arxiv.org/abs/2110.02178) | 07[abs] | OK |
| 130 | PTQ4ViT | Yuan et al. | ECCV | 2022 | [arXiv 2111.12293](https://arxiv.org/abs/2111.12293) | 07[abs] | OK |
| 131 | FQ-ViT | Lin et al. | IJCAI | 2022 | [arXiv 2111.13824](https://arxiv.org/abs/2111.13824) | 07[abs] | OK |
| 132 | FlashAttention | Dao et al. | NeurIPS | 2022 | [arXiv 2205.14135](https://arxiv.org/abs/2205.14135) | 07[abs] | OK |
| 133 | "Analysis of Score-Level Fusion Rules for Deepfake Detection" | not given | Applied Sciences 12(15):7365 | 2022 | [MDPI](https://www2.mdpi.com/2076-3417/12/15/7365) | 07[snip] | n/c |
| 134 | ToMe: "Token Merging: Your ViT But Faster" | Bolya et al. | ICLR | 2023 | [arXiv 2210.09461](https://arxiv.org/abs/2210.09461) | 07[abs] | OK |
| 135 | RepQ-ViT | Zhikai Li et al. | ICCV | 2023 | [arXiv 2212.08254](https://arxiv.org/abs/2212.08254) | 07[abs] | OK |
| 136 | EfficientViT (multi-scale linear attention) | H. Cai et al. | ICCV | 2023 | [arXiv 2205.14756](https://arxiv.org/abs/2205.14756) | 07[abs] | OK |
| 137 | FastViT | Vasu et al. | ICCV | 2023 | [arXiv 2303.14189](https://arxiv.org/abs/2303.14189) | 07[abs] | OK |
| 138 | MoLE: "Mixture of Low-rank Experts…" | Zihan Liu et al. | arXiv | 2024 | [arXiv 2404.04883](https://arxiv.org/abs/2404.04883) | 07[search] | OK |
| 139 | HyperDet | H. Cao et al. | arXiv | 2024 | [arXiv 2410.06044](https://arxiv.org/abs/2410.06044) | 07[search] | OK |
| 140 | DistilDIRE | Lim et al. | arXiv | 2024 | [arXiv 2406.00856](https://arxiv.org/abs/2406.00856) | 02[sec aggregator], 07[search] | OK |
| 141 | "Unlocking the Potential of Lightweight Quantized Models for Deepfake Detection" | R. Tao et al. | IJCAI | 2025 | [IJCAI](https://www.ijcai.org/proceedings/2025/59) | 07[fetched] | CONFIRMED (web) |
| 142 | ALEI: "Exploring the Collaborative Advantage of Low-level Information…" | Z. Zhou et al. | arXiv | 2025 | [themoonlight review](https://www.themoonlight.io/review/exploring-the-collaborative-advantage-of-low-level-information-on-generalizable-ai-generated-image-detection) (arXiv 2504.00463) | 07[search, sec] | OK |
| 143 | LAID: lightweight AIGI detection | Chivaran & Ni | PST | 2025 | [arXiv 2507.05162](https://arxiv.org/abs/2507.05162) | 07[search] | OK |
| 144 | "Quantization Robustness to Input Degradations for Object Detection" | Karimov et al. | arXiv | 2025 | [arXiv 2508.19600](https://arxiv.org/abs/2508.19600v1) | 07[search] | OK |
| 145 | Audio deepfake SSL countermeasure fusion | Serrano et al. | arXiv | 2025 | [arXiv 2509.12003](https://arxiv.org/abs/2509.12003) | 07[search] | OK |
| 146 | OmniAID | Yuncheng Guo et al. | arXiv | 2025 | arXiv 2511.08423 (via scour.ing link) | 07[search, abstract only] | OK |
| 147 | DINO-Detect (blur-robust distillation) | J. Shen et al. | arXiv | 2025 | [alphaXiv 2511.12511](https://www.alphaxiv.org/abs/2511.12511) | 07[search] | OK |
| 148 | ForensicZip | Lai et al. | arXiv | 2026 | [arXiv 2603.12208](https://arxiv.org/abs/2603.12208) | 07[fetched] | CONFIRMED |
| 149 | "Evolutionary Multi-Objective Fusion of Deepfake Speech Detectors" | Staněk et al. | WCCI/CEC | 2026 | [arXiv 2604.01330](https://arxiv.org/abs/2604.01330) | 07[search] | OK |
| 150 | DFD-HR: "…Hierarchical Routing Learning" | J. Sun et al. | CVPR | 2026 | [papernotes summary](https://en.papernotes.org/CVPR2026/ai_safety/dfd-hr_generalizable_deepfake_detection_via_hierarchical_routing_learning) | 07[sec] | CONFIRMED (web) |
| 151 | "Adaptive Gated Deepfake Detection for Low-Resolution and Resource-Constrained Environments" | Sen et al. | arXiv | 2026 | [arXiv 2609.05320](https://arxiv.org/abs/2609.05320) | 07[search] | OK |

### K. Face forgery (image and video)

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 152 | Face X-ray | Lingzhi Li et al. | CVPR | 2020 | [arXiv 1912.13458](https://arxiv.org/abs/1912.13458) | 06[u] | OK |
| 153 | LipForensics ("Lips Don't Lie") | Haliassos et al. | CVPR | 2021 | [arXiv 2012.07657](https://arxiv.org/abs/2012.07657) | 06[rec] | OK |
| 154 | FTCN ("Exploring Temporal Coherence…") | Y. Zheng et al. | ICCV | 2021 | [arXiv 2108.06693](https://arxiv.org/abs/2108.06693) | 06[rec] | OK |
| 155 | RealForensics | Haliassos et al. | CVPR | 2022 | [arXiv 2201.07131](https://arxiv.org/abs/2201.07131) | 06[rec] | OK |
| 156 | SBI: Self-Blended Images | Shiohara & Yamasaki | CVPR | 2022 | [arXiv 2204.08376](https://arxiv.org/abs/2204.08376) | 06[rec] | OK |
| 157 | UCF | Z. Yan et al. | ICCV | 2023 | [arXiv 2304.13949](https://arxiv.org/abs/2304.13949) | 06[u] | OK |
| 158 | AltFreezing | Z. Wang et al. | CVPR | 2023 | [arXiv 2307.08317](https://arxiv.org/abs/2307.08317) | 06[rec] | OK |
| 159 | DeepfakeBench | Z. Yan et al. | NeurIPS D&B | 2023 | [arXiv 2307.01426](https://arxiv.org/abs/2307.01426) | 06[u] | OK |
| 160 | TALL: Thumbnail Layout | Y. Xu et al. | ICCV | 2023 | [arXiv 2307.07494](https://arxiv.org/abs/2307.07494) | 05[u] | OK |
| 161 | LAA-Net | D. Nguyen et al. | CVPR | 2024 | [CVF](https://openaccess.thecvf.com/content/CVPR2024/html/Nguyen_LAA-Net_Localized_Artifact_Attention_Network_for_Quality-Agnostic_and_Generalizable_Deepfake_CVPR_2024_paper.html) (arXiv 2401.13856) | 06[u; partial] | OK |
| 162 | FreqBlender | H. Li et al. | NeurIPS | 2024 | [arXiv 2404.13872](https://arxiv.org/abs/2404.13872) | 06[u] | OK |
| 163 | DF40 | Z. Yan et al. | NeurIPS D&B | 2024 | [NeurIPS](https://proceedings.neurips.cc/paper_files/paper/2024/hash/34239f60eca7ce9bee5280aaf81362d8-Abstract.html); arXiv 2406.13495 | 06[u; split sec] | OK |
| 164 | Facial Component Guided Adaptation (video deepfakes) | Y.-H. Han et al. | CVPR | 2025 | [CVF](https://openaccess.thecvf.com/content/CVPR2025/html/Han_Towards_More_General_Video-based_Deepfake_Detection_through_Facial_Component_Guided_CVPR_2025_paper.html); arXiv 2404.05583 | 06[u] | OK |

### L. Local edits, inpainting and forgery localization

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 165 | CAT-Net | Kwon et al. | WACV (IJCV 2022 ext.) | 2021 | [arXiv 2108.12947](https://arxiv.org/abs/2108.12947) | 06[u] | OK |
| 166 | MVSS-Net | X. Chen et al. | ICCV | 2021 | [arXiv 2104.06832](https://arxiv.org/abs/2104.06832) | 06[u] | OK |
| 167 | TruFor | Guillaro et al. | CVPR | 2023 | [arXiv 2212.10957](https://arxiv.org/abs/2212.10957) | 06[u] | OK |
| 168 | HiFi-Net | X. Guo et al. | CVPR | 2023 | [arXiv 2303.17111](https://arxiv.org/abs/2303.17111) | 06[u] | OK |
| 169 | IML-ViT | X. Ma et al. | arXiv | 2023 | [arXiv 2307.14863](https://arxiv.org/abs/2307.14863) | 06[u] | OK |
| 170 | Semi-Truths | Pal, Kruk et al. | NeurIPS D&B | 2024 | [arXiv 2411.07472](https://arxiv.org/abs/2411.07472) | 06[u], 08[u] | OK |
| 171 | DeCLIP | Smeu et al. | WACV | 2025 | [CVF](https://openaccess.thecvf.com/content/WACV2025/html/Smeu_DeCLIP_Decoding_CLIP_Representations_for_Deepfake_Localization_WACV_2025_paper.html); arXiv 2409.08849 | 06[u] | OK |
| 172 | OpenSDI / OpenSDID | Y. Wang et al. | CVPR | 2025 | [CVF](https://openaccess.thecvf.com/content/CVPR2025/html/Wang_OpenSDI_Spotting_Diffusion-Generated_Images_in_the_Open_World_CVPR_2025_paper.html); arXiv 2503.19653 | 06[u] | OK |
| 173 | SAGI / SAGI-D | P. Giakoumoglou et al. | ICCV | 2025 | [CVF](https://openaccess.thecvf.com/content/ICCV2025/html/Giakoumoglou_SAGI_Semantically_Aligned_and_Uncertainty_Guided_AI_Image_Inpainting_ICCV_2025_paper.html); arXiv 2502.06593 | 06[u] | OK |
| 174 | COCO-Inpaint | H. Yan et al. | ACM MM | 2025 | [arXiv 2504.18361](https://arxiv.org/abs/2504.18361) | 06[u] | OK |
| 175 | "Zooming In on Fakes" (BR-Gen, NFA-ViT) | L. Cai et al. | AAAI | 2026 | [AAAI OJS](https://ojs.aaai.org/index.php/AAAI/article/view/37240); arXiv 2504.11922 | 06[u] | OK |

### M. Generated-video detection: methods

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 176 | DeCoF: "Detecting AI-Generated Video via Frame Consistency" | L. Ma et al. | arXiv | 2024 | [arXiv 2402.02085](https://arxiv.org/abs/2402.02085) | 05[u] | OK |
| 177 | AIGVDet: "AI-Generated Video Detection via Spatio-Temporal Anomaly Learning" | Bai et al. | PRCV | 2024 | [arXiv 2403.16638](https://arxiv.org/abs/2403.16638) | 05[u] | OK |
| 178 | "Beyond Deepfake Images: Detecting AI-Generated Videos" | Vahdati et al. | CVPRW | 2024 | [arXiv 2404.15955](https://arxiv.org/abs/2404.15955) | 05[u] | OK |
| 179 | Local-and-global temporal defects ("Exposing AI-generated Videos") | P. He et al. | arXiv | 2024 | [arXiv 2405.04133](https://arxiv.org/abs/2405.04133) | 05[u] | OK |
| 180 | DeMamba + GenVideo | H. Chen et al. | arXiv | 2024 | [arXiv 2405.19707](https://arxiv.org/abs/2405.19707) | 05[u] | OK |
| 181 | DIVID: "Turns Out I'm Not Real" | Q. Liu et al. | arXiv | 2024 | [arXiv 2406.09601](https://arxiv.org/abs/2406.09601) | 05[u] | OK |
| 182 | L3DE: "How Far are AI-generated Videos from Simulating the 3D Visual World" | C. Chang et al. | arXiv | 2024 | [arXiv 2406.19568](https://arxiv.org/abs/2406.19568) | 05[u] | OK |
| 183 | MM-Det + DVF: "On Learning Multi-Modal Forgery Representation…" | X. Song et al. | NeurIPS | 2024 | [arXiv 2410.23623](https://arxiv.org/abs/2410.23623) | 05[u] | OK |
| 184 | UNITE: "Towards a Universal Synthetic Video Detector" | Kundu et al. | CVPR | 2025 | [arXiv 2412.12278](https://arxiv.org/abs/2412.12278) | 05[u] | OK |
| 185 | REVEAL | Y.-Y. Tsai et al. | ICCV Workshop | 2025 | [arXiv 2502.14994](https://arxiv.org/abs/2502.14994) | 05[u] | OK |
| 186 | BusterX (GenBuster-200K) | H. Wen et al. | arXiv | 2025 | [arXiv 2505.12620](https://arxiv.org/abs/2505.12620) | 05[u] | OK |
| 187 | WaveRep: "Seeing What Matters…" | Corvi et al. | NeurIPS | 2025 | [arXiv 2506.16802](https://arxiv.org/abs/2506.16802) | 05[u] | OK |
| 188 | ReStraV: "…via Perceptual Straightening" | Internò et al. | NeurIPS | 2025 | [arXiv 2507.00583](https://arxiv.org/abs/2507.00583) | 05[u] | OK |
| 189 | BusterX++ | H. Wen et al. | arXiv | 2025 | [arXiv 2507.14632](https://arxiv.org/abs/2507.14632) | 05[u] | OK |
| 190 | D3: "Training-Free AI-Generated Video Detection Using Second-Order Features" | C. Zheng et al. | ICCV | 2025 | [arXiv 2508.00701](https://arxiv.org/abs/2508.00701) | 05[u] | OK |
| 191 | FakeParts | Z. Liu et al. | arXiv | 2025 | [arXiv 2508.21052](https://arxiv.org/abs/2508.21052) | 05[u] | OK |
| 192 | NSG-VD: "Physics-Driven Spatiotemporal Modeling…" | S. Zhang et al. | NeurIPS (spotlight) | 2025 | [arXiv 2510.08073](https://arxiv.org/abs/2510.08073) | 05[u] | OK |
| 193 | Skyra (ViF-Bench) | Yifei Li et al. | arXiv | 2025 | [arXiv 2512.15693](https://arxiv.org/abs/2512.15693) | 05[u] | OK |
| 194 | VidGuard-R1 | K. Park et al. | ICLR | 2026 | [arXiv 2510.02282](https://arxiv.org/abs/2510.02282) | 05[u] | OK |
| 195 | VideoVeritas (MintVid) | H. Tan et al. | arXiv | 2026 | [arXiv 2602.08828](https://arxiv.org/abs/2602.08828) | 05[u] | OK |
| 196 | STALL + ComGenVid: "Training-free Detection of Generated Videos via Spatial-Temporal Likelihoods" | Ben Hayun et al. | CVPR | 2026 | [arXiv 2603.15026](https://arxiv.org/abs/2603.15026) | 05[u] | CONFIRMED |
| 197 | Multi-modal forgery representation for AI-video detection and localization | D. Le et al. | arXiv | 2026 | [arXiv 2605.07232](https://arxiv.org/abs/2605.07232) | 05[u] | OK |
| 198 | SpecSem-Net | Z. Wei et al. | arXiv | 2026 | [arXiv 2605.17311](https://arxiv.org/abs/2605.17311) | 05[u] | OK |
| 199 | V-PVP: "Rethinking the Readout…" | M. Cui et al. | arXiv | 2026 | [arXiv 2607.15321](https://arxiv.org/abs/2607.15321) | 05[u] | CONFIRMED |
| 200 | "Detect Early, Escalate Rarely: Anytime Detection… from the Compressed Bits" (05's "Cakiroglu et al., codec paper") | Cakiroglu et al. | arXiv | 2026 | [arXiv 2607.19476](https://arxiv.org/html/2607.19476v1) | 05[u] | OK |
| 201 | PIVOT (PhysForensics-Bench) | B. Zheng et al. | arXiv | 2026 | [arXiv 2609.15562](https://arxiv.org/abs/2609.15562) | 05[u] | OK |
| 202 | TRACE | H. Cao et al. | arXiv | 2026 | [arXiv 2609.25775](https://arxiv.org/abs/2609.25775) | 05[u] | OK |

### N. Generated-video benchmarks, audits and audio-visual work

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 203 | VidProM | W. Wang & Y. Yang | NeurIPS D&B | 2024 | [arXiv 2403.06098](https://arxiv.org/abs/2403.06098) | 05[u] | OK |
| 204 | AGAV-Rater (quality assessment, not detection) | Y. Cao et al. | arXiv | 2025 | arXiv 2501.18314 | 05[u] | OK |
| 205 | MVAD: multimodal video-audio detection benchmark | M. Hu et al. | arXiv | 2025 | [arXiv 2512.00336](https://arxiv.org/html/2512.00336v4) | 05[u] | OK |
| 206 | VideoASMR-Bench | J. Wang et al. | arXiv | 2025 | [arXiv 2512.13281](https://arxiv.org/abs/2512.13281) | 05[u] | OK |
| 207 | GenVidBench | Z. Ni et al. | AAAI | 2026 | [arXiv 2501.11340](https://arxiv.org/abs/2501.11340) | 05[u] | OK |
| 208 | AIGVDBench (arXiv title "Your One-Stop Solution for AI-Generated Video Detection") | L. Ma et al. | arXiv (CVPR 2026 per Tintype survey) | 2026 | [arXiv 2601.11035](https://arxiv.org/html/2601.11035v1) | 05[u] | CONFIRMED (title differs) |
| 209 | GenVideoLens | Y. Zou et al. | arXiv | 2026 | [arXiv 2603.18625](https://arxiv.org/abs/2603.18625) | 05[u] | OK |
| 210 | BRITE (T2V evaluation; not detection) | Tilak et al. | arXiv | 2026 | arXiv 2605.00873 | 05[u] | OK |
| 211 | Artifact-Bench | Y. Tang et al. | arXiv | 2026 | [arXiv 2605.18984](https://arxiv.org/abs/2605.18984) | 05[u] | OK |
| 212 | "Where the Real Clips Came From: Corpus Identity in AI-Generated Video Detection" | Cakiroglu et al. | arXiv | 2026 | [arXiv 2606.31004](https://arxiv.org/html/2606.31004v2) | 05[u] | CONFIRMED |
| 213 | "Dataset Biases and Shortcut Learning in Motion-Based AI-Generated Video Detection" | Michels et al. | arXiv | 2026 | [arXiv 2607.00948](https://arxiv.org/html/2607.00948v2) | 05[u] | CONFIRMED |

### O. Calibration, selective prediction and conformal guarantees

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 214 | "A Gentle Introduction to Conformal Prediction…" | Angelopoulos & Bates | arXiv | 2021 | [arXiv 2107.07511](https://arxiv.org/abs/2107.07511) | 08[u] | OK |
| 215 | Conformal FPR control for machine-generated text detection (multi-level CP) | X. Zhu et al. | ACL | 2025 | [arXiv 2505.05084](https://arxiv.org/abs/2505.05084); [ACL](https://preview.aclanthology.org/setup/2025.acl-long.601) | 08[u] | OK |
| 216 | "Your AI-Generated Image Detector Can Secretly Achieve SOTA Accuracy, If Calibrated" | M. Yang et al. | AAAI | 2026 | [arXiv 2602.01973](https://arxiv.org/abs/2602.01973) | 08[u] | CONFIRMED |
| 217 | ReSIDe: "Post-hoc Selective Classification for Reliable Synthetic Image Detection" | K. Zheng & Seidman | NeurIPS (per 08; not on arXiv) | 2026 | [arXiv 2605.08574](https://arxiv.org/abs/2605.08574) | 08[u] | CONFIRMED |
| 218 | DISCERN: "Uncertainty-Aware Deepfake Detection via Multi-View Structural Learning" | M. U. Farooq et al. | arXiv | 2026 | [arXiv 2607.28769](https://arxiv.org/pdf/2607.28769) | 08[snip] | OK |

### P. Source attribution and open-set recognition

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 219 | WOUAF | C. Kim et al. | CVPR | 2024 | [arXiv 2306.04744](https://arxiv.org/abs/2306.04744) | 08[u] | OK |
| 220 | "Are CLIP features all you need for Universal Synthetic Image Origin Attribution?" | Cioni et al. | ECCV Workshop | 2024 | [arXiv 2408.09153](https://arxiv.org/abs/2408.09153) | 08[u] | OK |
| 221 | OmniDFA (current arXiv title "Few-Shot Synthetic Image Attribution…") | S. Wu et al. | arXiv (ECCV 2026 per arXiv comment) | 2025 | [arXiv 2509.25682](https://arxiv.org/abs/2509.25682) | 08[u] | CONFIRMED (title differs) |
| 222 | "Supervised Contrastive Learning for Few-Shot AIGI Detection and Attribution" | Álvarez Urueña et al. | arXiv | 2025 | [arXiv 2511.16541](https://arxiv.org/abs/2511.16541) | 08[snip] | OK |
| 223 | Face-Trace: open-set attribution of synthetic face generators | Infantino et al. | arXiv | 2026 | arXiv 2607.07545 (no link) | 08[u] | OK |
| 224 | "Scalable Black-Box Model Attribution for Images" | Livne et al. | arXiv | 2026 | [arXiv 2608.15652](https://arxiv.org/pdf/2608.15652) | 08[snip] | OK |

### Q. Image benchmarks, evaluation methodology and surveys

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 225 | ImagiNet | Boychev & Cholakov | arXiv | 2024 | [arXiv 2407.20020](https://arxiv.org/pdf/2407.20020v1.pdf) | 08[u] | OK |
| 226 | "Methods and Trends in Detecting AI-Generated Images: A Comprehensive Review" | Mahara & Rishe | arXiv | 2025 | [arXiv 2502.15176](https://arxiv.org/pdf/2502.15176) | 01[review, source for SSP] | OK |
| 227 | Deepfake-Eval-2024 | Chandra et al. | arXiv | 2025 | [arXiv 2503.02857](https://arxiv.org/abs/2503.02857) | 04[u], 05[u], 06[u], 08[u] | OK |
| 228 | AI-GenBench | Pellegrini et al. | arXiv | 2025 | [arXiv 2504.20865](https://arxiv.org/abs/2504.20865) | 08[u] | OK |
| 229 | DRAGON | Bertazzini et al. | arXiv | 2025 | [arXiv 2505.11257](https://arxiv.org/abs/2505.11257) | 08[u] | OK |
| 230 | "How well are open sourced AI-generated image detection models out-of-the-box…" ("Out-of-the-box benchmark") | S. Ren et al. | arXiv | 2026 | [arXiv 2602.07814](https://arxiv.org/abs/2602.07814) | 07[ctx], 08[u] | OK |
| 231 | "The Synthetic Media Shift…" (Community Notes in-the-wild study) | Chrysidis et al. | arXiv | 2026 | arXiv 2604.15372 | 02[u] | OK |
#### Count reconciliation

- I extracted 219 distinct arXiv IDs from the eight notes. All 219 resolve on arXiv, and each maps to exactly one row above. For SPSL, the ID in the note (2103.01412) is wrong and the row gives the corrected ID 2103.01856.
- 12 papers are cited without an arXiv ID: BiHPF (#7), LGrad (#10), Synthbuster (#11), Frequency Masking (#16), DRCT (#21), TechRxiv MLLM face study (#56), Breaking Semantic Artifacts (#60), OST (#71), DACOM (#97), MDPI score-fusion (#133), Tao et al. IJCAI 2025 (#141) and DFD-HR (#150).
- 219 + 12 = **231 distinct academic papers, reports and benchmarks**, numbered 1–231 above. BusterX and BusterX++ are counted separately. So are the two Cakiroglu et al. 2026 papers (corpus identity; codec/anytime) and the two Corvi et al. 2023 papers.

---

### Mentioned without any citation (not counted)

Named in the notes with no title, authors or link: GenD (08, face-deepfake baseline), ForAda (DFD-HR context), Over-Coherence and FVMD (05, video baselines), the "Where did I come from? Origin attribution" paper (08, flagged as not located), "The Adversarial AI-Art" benchmark (08, not verified), C2PA, Tree-Ring and Stable Signature (08, as provenance or watermark schemes), FaceForensics++, Celeb-DF and DFDC (06, as datasets), Meta's DINOv3 backbone paper (never cited directly), and the Davodi et al. content-matched cohort (Tintype repo evidence).

### Non-academic sources: software, project pages, model cards, blogs, reviews and challenge pages (not counted)

**Code repositories (GitHub):** mever-team/rine (02); UltraCapture/GAPL (02); beibuwandeluori/DRCT (03); MMLab-unitn/TrueFake-IJCNN25 (03); Coral79/CDDB (03); andrekassis/ai-watermark (04); grip-unina/WaveRep-SyntheticVideoDetection (05); ZSHsh98/NSG-VD (05); manic-cui/Rethinking-the-readout (05); JoeLeelyf/Skyra (05); LongMa-2025/AIGVDBench (05); HuMengXue0104/MVAD (05); SCLBD/DeepfakeBench (06); YZY-stack/Effort-AIGI-Detection (06); chail/patch-forensics (06); Zig-HS/FakeImageDetection (07); nuriachandra/Deepfake-Eval-2024 (08). 06 also names data repos without links: YZY-stack/DF40, iamwangyabin/OpenSDI, clpbc/BR-Gen, mever-team.github.io/SAGI and mever-team/texture-crop (04).

**Project pages:** LOKI (opendatalab.github.io/LOKI, 02); Stay-Positive (anisundar18.github.io, 04); B-Free (grip-unina.github.io/B-Free, 07, 08); ZED (grip-unina.github.io/ZED, 08); STALL (omerbenhayun.github.io/stall-video, 05); LaDeDa (vision.huji.ac.il/ladeda, 07); B-Free CVPR 2025 slides (cvpr.thecvf.com, 08).

**Model cards and datasets:** Hugging Face model card KabirAcharya/waterspy-sieve-ft1-int8 (07; unreviewed INT8 evidence); Hugging Face dataset README semi-truths/Semi-Truths (06); HF dataset aimagelab/RAID (04, named).

**Blogs, reviews, aggregators and press:** liner.com reviews of DACOM (04) and DF40 (06); themoonlight.io reviews of AIDE and ALEI (07); lacuna.tiptreesystems.com summary of DistilDIRE (02); en.papernotes.org summary of DFD-HR (07); theregister.com article on UnMarker (04); photoworkout.com summary of a NewsGuard audit (04; unverified); deepfake-demo.aisec.fraunhofer.de related-work pages (04, 06); alphaxiv.org overview pages (04, 07); scour.ing link to OmniAID (07); mlanthology.org entries (04); CISPA publications page hosting the Mavali et al. preprint (04).

**Challenge and society pages:** IEEE SP Cup 2025 / DFWild-Cup page at signalprocessingsociety.org (04).
