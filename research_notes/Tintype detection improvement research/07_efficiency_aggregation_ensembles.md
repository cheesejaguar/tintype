# Inference efficiency, multi-view/patch aggregation and ensembling for AI-generated image (and video) detectors — what transfers to Tintype

Scope note. Tintype's current design (from the repo, read this session): ViT-S/16 at 224 px; up to six views per photo (1 resized "fit" view, native centre 224 crop, the 4 most-textured native 224 windows); mean of view logits; views scored one at a time, most textured first, with margin-based early exit (margins 6 / 3 / 2 / 1.5 / 1 in logit units, so 1.7–1.9 views per image on average) ([README "How it works"](../../README.md), [docs/BENCHMARK.md "Five speed changes"](../../docs/BENCHMARK.md)). Measured throughput is 530–563 photos/s on the Apple GPU (16-bit MPSGraph), 77–78 photos/s on the macOS CPU, 35 photos/s on 8-core aarch64 Linux. The repo has already tried two things in this area and dropped them: **token merging (ToMe)** cut recall from 0.848 to 0.786 at r=8 and to 0.708 at r=13 on 1,000 timing images, and **fused scaled-dot-product attention** was no faster at 197 tokens ([docs/BENCHMARK.md](../../docs/BENCHMARK.md)). The photo second opinion is a band rule in calibrated logits: photos with first-stage logit z1 in (−7, 0.5) also get the Community Forensics logit z2, and the output is sigmoid(0.5·(0.5·z1 + z2)) ([src/detector.rs](../../src/detector.rs), [models/tintype.json](../../models/tintype.json)). The repo notes the GPU now spends its time in the twelve blocks (~4 ms each for a batch of 64), so "a further large gain on photos would need fewer tokens or fewer blocks, which means retraining" ([docs/BENCHMARK.md](../../docs/BENCHMARK.md)).

Source-quality legend used below: **[fetched]** = primary abstract/page read this session; **[search]** = facts from search-result snippets of primary pages, not the full PDF; **[abstract, known]** = well-known paper whose abstract-level claims are quoted from the arXiv page linked, not re-read in full this session (numbers should be spot-checked before being quoted publicly).

---

## 1. Patch/view selection and aggregation in forensic detectors (and native-resolution inference)

### Takeaway
The literature converges on three points that bear directly on Tintype: (a) native-resolution, un-resampled pixels carry the signal (B-Free, SPAI, LaDeDa, SSP), which validates Tintype's native crops; (b) *which* patches and *how* they are pooled matters, but the evidence is split between "one simple/low-texture patch is enough" (SSP), "contrast rich vs poor texture" (PatchCraft, AIDE picks both highest- and lowest-frequency patches) and "use all patches, detectors are lazily biased to a few" (Panoptic Patch Learning, ICLR 2026) — Tintype currently samples only rich-texture windows plus the centre, so it never looks at poor-texture regions on purpose; (c) mean-logit pooling is a reasonable default, but top-k/max or learned pooling is cheap to evaluate offline on Tintype's stored per-view logits and is the lowest-cost accuracy experiment available.

### Cited Findings

**1.1 PatchCraft / Smash&Reconstruct** [fetched]
- Zhong, Xu, Li, Qian, Zhang. "PatchCraft: Exploring Texture Patch for Efficient AI-generated Image Detection." arXiv 2311.12397 (v1 Nov 2023, v3 Mar 2024). Smash&Reconstruction preprocessing destroys global semantics and emphasises texture patches; the detector contrasts inter-pixel correlation between rich-texture and poor-texture regions; benchmark of 17 generators; abstract claims a "significant margin" over baselines without a single headline number — [arXiv 2311.12397](https://arxiv.org/abs/2311.12397)
- **Tintype applicability.** The transferable idea is the *rich-vs-poor contrast*, not the reconstruction. Concrete change: replace one of the four "most textured" windows with the *least* textured non-flat window (e.g. lowest-variance window above a minimum-variance floor so pure sky/clipped highlights are excluded). Speed: zero cost (same number of views; texture ranking is already computed). Precision/recall: unknown — must be tested; poor-texture windows are where diffusion decoders' upsampling/VAE periodicities are least masked by content, but also where JPEG blocking dominates. Engineering cost: trivial (view-selection code in Rust; no engine change), but requires retraining-free evaluation only if the network has seen such crops in training (training used random crops, so it has). Risk: low-texture crops of real photos may be dominated by sensor noise/JPEG and raise FPR; check per-source FPR.

**1.2 SSP — "A Single Simple Patch is All You Need"** [search]
- Chen, Yao, Niu. arXiv 2402.01123 (Feb 2024). Picks the single *simplest* (lowest texture diversity) 32×32 patch, extracts SRM noise, ResNet-50 classifier; patch size 32 best (too large or small degrades); using several patches was "comparable", with only a slight gain at four patches; reported 14.6% relative improvement over a recent method on GenImage; ESSP adds learned deblur/decompress task embeddings because the plain method fails under blur/JPEG — [arXiv 2402.01123](https://arxiv.org/abs/2402.01123)
- **Tintype applicability.** Supports the idea in 1.1 (low-texture regions are informative) but also warns that low-texture/noise-residual signals are the most fragile to JPEG and blur, which Tintype's target traffic (social-media re-shares) is full of. Do not adopt SRM or 32 px patches (incompatible with the ViT and far below the 224 receptive field). Use only as motivation for a "least-textured window" A/B test, scored separately on re-shared vs original sets.

**1.3 AIDE — "A Sanity Check for AI-generated Image Detection"** [search]
- Yan et al., ICLR 2025 (arXiv 2406.19435). Mixture of experts: a CLIP (ConvNeXt) semantic branch on the whole image plus a low-level branch fed the DCT-ranked highest- and lowest-frequency patches (SRM + ResNet-50); reported +3.5% and +4.6% over SOTA on AIGCDetectBenchmark and GenImage; introduced the Chameleon test set, where AIDE "still performs modestly" — [ICLR proceedings](https://proceedings.iclr.cc/paper_files/paper/2025/hash/b0303773962ea1b5394c3a83cc7dd066-Abstract-Conference.html), [arXiv 2406.19435](https://www.arxiv.org/pdf/2406.19435). Exact patch counts (k highest + k lowest, N=32) come from third-party summaries — [themoonlight review](https://www.themoonlight.io/en/review/a-sanity-check-for-ai-generated-image-detection)
- Independent context already in the repo: AIDE variants swing 20–60 points depending on training set in the Feb 2026 open-detector comparison (arXiv 2602.07814, as summarised in [docs/RESEARCH.md](../../docs/RESEARCH.md)).
- **Tintype applicability.** Same lesson as PatchCraft: sample *both ends* of the texture/frequency ranking. Cheaper proxy for Tintype: rank windows by a high-frequency DCT energy score instead of (or alongside) spatial variance — the ranking is computed on 8-bit crops in the decode workers, so cost is a few µs per window. The two-backbone fusion itself is not worth copying (a CLIP ConvNeXt branch would more than double cost).

**1.4 Breaking Semantic Artifacts (patch shuffle)** [search]
- Zheng, Lin, Zhao, Wang, Guo, Liu, Shen. "Breaking Semantic Artifacts for Generalized AI-generated Image Detection." NeurIPS 2024 (not ECCV), DOI 10.52202/079017-1903. Detectors overfit to scene semantics; patch-shuffle the image and train a small-receptive-field patch classifier; 31 test sets (7 GANs, 18 diffusion variants, 6 other); +2.08% absolute cross-scene accuracy and a 10.59% absolute open-world gain in the paper abstract (the NeurIPS poster page says 6.74% — the two sources disagree) — [NeurIPS proceedings](https://proceedings.neurips.cc/paper_files/paper/2024/hash/6dddcff5b115b40c998a08fbd1cea4d7-Abstract.html), [code](https://github.com/Zig-HS/FakeImageDetection)
- **Tintype applicability.** A *training* idea, not an inference one: patch-shuffle augmentation (shuffle 16×16 or 32×32 blocks inside the 224 crop) would push the ViT toward local artifacts and away from content, which may also make views more independent and mean-pooling more effective. Speed: none at inference. Cost: an augmentation in the Rust trainer (MPSGraph training already exists). Risk: may cut the fit view's value (global semantic cues such as malformed hands are deliberately destroyed); apply only to native-crop training samples.

**1.5 Fusing Global and Local Features (learned patch selection + attention fusion)** [search]
- Ju, Jia, Ke, Xue, Nagano, Lyu. arXiv 2203.13964 (2022; ICIP 2022). Global branch on the whole image + Patch Selection Module that *predicts coordinates* of informative patches (attention-based), cropped and fed to a local branch; multi-head attention fusion of global and patch embeddings; outperforms baselines in mAP on a 19-generator dataset — [arXiv 2203.13964](https://arxiv.org/abs/2203.13964)
- **Tintype applicability.** This is the "learned pooling" alternative to mean-logit. A tiny attention-pooling head over the six views' **class-token embeddings** (384-d each) is cheap at inference (one 6×384 attention; microseconds) but it breaks per-view early exit unless the head is trained to accept 1…6 views in the most-textured-first order (train with random view-count truncation). Engineering: small MPSGraph/CPU addition; requires a training pass with frozen backbone. Risk: learned pooling over few views may overfit the development generators.

**1.6 LaDeDa / Tiny-LaDeDa — "Real-Time Deepfake Detection in the Real-World"** [search]
- Cavia, Horwitz, Reiss, Hoshen. arXiv 2406.09398 (June 2024). LaDeDa scores every 9×9 patch and pools patch scores into an image score; Tiny-LaDeDa (4 conv layers) distilled from LaDeDa's patch logits has 375× fewer FLOPs and 10,000× fewer parameters at "a small accuracy drop"; WildRF (Reddit/X/Facebook) dataset; 93.7% mAP on WildRF; models near-perfect on standard benchmarks perform poorly on social-media data — [arXiv 2406.09398](https://arxiv.org/abs/2406.09398), [project page](https://vision.huji.ac.il/ladeda)
- **Tintype applicability.** Two ideas. (i) A *patch-score distillation* student as a **first-stage gate** (see §2/§5): a ~10–100 k-parameter CNN running over the full native image on CPU in well under a millisecond could pick the windows to look at (replacing variance-based texture ranking with "where does a cheap forensic model see the most evidence") or decide clear reals outright. (ii) Evidence that pooled local scores work on social-media imagery. Risk: Tiny-LaDeDa's accuracy on current commercial generators is unverified; using it to *choose* windows (not to decide) is the low-risk use.

**1.7 Panoptic Patch Learning — "All Patches Matter, More Patches Better"** [fetched + search]
- Yang, Chen, Yan, Zhang, Fu, Wu, Shu, Yao, Ding, Qin, Li. arXiv 2504.01396 (Apr 2025, v3 Mar 2026); listed as an ICLR 2026 poster in the ICLR virtual site/proceedings ([ICLR 2026](https://iclr.cc/virtual/2026/poster/10007395)). Naively trained detectors show a "Few-Patch Bias" (they decide from minority patches, a "Lazy Learner" effect); fix = Random Patch Replacement (inject synthetic patches into real images) + patch-wise contrastive learning — [arXiv 2504.01396](https://arxiv.org/abs/2504.01396). The arXiv abstract gives no headline numbers.
- **Tintype applicability.** Directly relevant to aggregation: if Tintype's ViT also has few-patch bias, then the view with the strongest evidence carries the decision and **mean-logit dilutes it** — a case for top-k/max-style pooling *or* for training that spreads evidence (patch replacement augmentation). Cheap diagnostic: from stored per-view logits on development sets, compute how often max-view and mean-view disagree and which is right. Training change (random real↔fake patch replacement within crops) is moderate cost in the trainer; zero inference cost.

**1.8 SPAI — Any-resolution detection by spectral learning** [fetched]
- Karageorgiou, Papadopoulos, Kompatsiaris, Gavves. "Any-Resolution AI-Generated Image Detection by Spectral Learning." CVPR 2025, pp. 18706–18717, arXiv 2411.19417. Masked spectral learning models the spectral distribution of *real* images; fakes are out-of-distribution via a spectral reconstruction similarity; **spectral context attention** aggregates patch-level evidence at native resolution with no resize/crop; +5.5% absolute AUC over prior SOTA across 13 recent generators; claimed robustness to online perturbations. No inference cost is given in the abstract — [CVF open access](https://openaccess.thecvf.com/content/CVPR2025/html/Karageorgiou_Any-Resolution_AI-Generated_Image_Detection_by_Spectral_Learning_CVPR_2025_paper.html), [arXiv 2411.19417](https://arxiv.org/abs/2411.19417)
- **Tintype applicability.** SPAI is the principled version of "look at many native windows and pool them with attention". Its whole-image processing is the opposite of Tintype's 1.7-view budget: at 12 MP, a full tiling is ~240 windows of 224². Transferable piece: an attention-pooling head over a *variable* number of native windows (1.5); not the spectral backbone. Cost to Tintype speed would be large if applied to all windows; acceptable only for the uncertain band (as an alternative "second opinion" made of *more Tintype windows* rather than a second network — see §4).

**1.9 B-Free — bias-free training, no resizing** [search]
- Guillaro, Zingarini, Cozzolino, Verdoliva. "A Bias-Free Training Paradigm for More General AI-generated Image Detection." CVPR 2025, arXiv 2412.17671. Resizing creates spurious cues (e.g. reals resized, fakes not) and washes out traces; B-Free avoids resizing, trains on 504×504 crops with a DINOv2 ViT on self-conditioned SD 2.1 fakes; project page reports balanced accuracy above 80% across Synthbuster generators under JPEG, resizing and blurring — [CVF PDF](https://openaccess.thecvf.com/content/CVPR2025/papers/Guillaro_A_Bias-Free_Training_Paradigm_for_More_General_AI-generated_Image_Detection_CVPR_2025_paper.pdf), [project page](https://grip-unina.github.io/B-Free)
- **Tintype applicability.** Corroborates the repo's own measurement that Community Forensics' decisions flip with resampler rounding ([docs/RESEARCH.md](../../docs/RESEARCH.md)). Implication: the **fit view is the most fragile view** and the one most likely to be noise; consider scoring it *last* (or only when native crops disagree), which also helps speed if native crops usually decide. Zero engineering cost to test from stored per-view logits.

**1.10 WaRPAD — training-free detection via cropping robustness** [search]
- arXiv 2511.14030 (Nov 2025). Rescales the image to a multiple of the backbone input, splits into patches, computes per-patch scores from self-supervised features and aggregates — [alphaxiv 2511.14030](https://www.alphaxiv.org/abs/2511.14030)
- **Tintype applicability.** Weak direct relevance (training-free, SSL-feature based); another data point that tile-and-aggregate is the standard way to handle arbitrary resolution.

**1.11 NaViT and FlexiViT (variable token counts)** [abstract, known]
- Dehghani et al. "Patch n' Pack: NaViT, a Vision Transformer for any Aspect Ratio and Resolution." NeurIPS 2023, arXiv 2307.06304 — packs variable-resolution images into one sequence with masking; supports token dropping in training — [arXiv 2307.06304](https://arxiv.org/abs/2307.06304)
- Beyer et al. "FlexiViT: One Model for All Patch Sizes." CVPR 2023, arXiv 2212.08013 — trains with randomised patch size so one set of weights runs at many patch sizes, trading compute for accuracy at deploy time — [arXiv 2212.08013](https://arxiv.org/abs/2212.08013)
- **Tintype applicability.** FlexiViT is the cleanest route to a *cheaper first look*: run the fit view at patch 32 (49 tokens instead of 196, ~4× fewer FLOPs in attention-free parts, more in attention) and only fall through to patch-16 native crops if undecided. But coarser patches average away exactly the pixel-period artifacts that matter (Tintype already saw ToMe destroy them), so the coarse view should be a *gate for obvious reals/fakes*, never the only look. NaViT-style packing would let Tintype score a non-square native region (e.g. 224×448) as one sequence — helpful for batching mixed view counts, but Tintype already fills batches with heterogeneous views, so the speed gain is small. Both require retraining (patch-embedding resize for FlexiViT is possible via the paper's PI-resize but forensic accuracy after resize is unverified).

**1.12 Community Forensics (data diversity > architecture; ViT-S works)** [context]
- Park & Owens, CVPR 2025, arXiv 2411.04125 — plain ViT-S trained on 4,803 generators beats larger specialised models (as summarised in [docs/RESEARCH.md](../../docs/RESEARCH.md); [arXiv 2411.04125](https://arxiv.org/abs/2411.04125)).
- **Tintype applicability.** Context for §5: the backbone is not the bottleneck for accuracy; data is. Shrinking the backbone is therefore plausible if training data stays wide.

### Inferences
- Tintype's view sampler covers "rich texture + centre + global"; the literature (PatchCraft, AIDE, SSP) consistently finds poor-texture/low-frequency patches informative and complementary. A "least-textured non-flat window" view is the most-supported zero-cost view-selection change to test.
- Mean-logit is not the literature's consensus; patch-level detectors use average (LaDeDa), max/top-k (implicit in few-patch-bias findings), or attention (Ju et al., SPAI). Because Tintype stores per-view logits for its development and blind sets, max, top-2-mean, trimmed mean, log-mean-exp (soft-max with temperature) and "native-only mean" can all be evaluated offline with no engine change. Any rule other than the mean interacts with early exit: margins must be re-fitted for the new pooled statistic.
- The fit view is resampled and therefore the least trustworthy per B-Free and the repo's own resampler-sensitivity finding; ordering it last could both save time and remove noise.

### Gaps
- No paper found that directly compares mean vs max vs attention pooling over views *for a ViT forensic detector at matched compute*. Panoptic Patch Learning's numbers could not be read from the abstract.
- No forensic paper reports the effect of choosing patches by texture vs randomly vs by a learned saliency at a fixed patch budget of 1–6 views.
- SPAI's inference cost and patch counts were not available from the abstract page.

---

## 2. Learned early exit / adaptive computation, and does token pruning destroy forensic signal?

### Takeaway
Cascades and confidence-based early exit are well supported as "faster at equal accuracy" in general vision (Wisdom of Committees: 2.3× over ViT-L-384 at equal accuracy) and Tintype already exploits view-level early exit. The remaining adaptive-compute gains are (a) a cheaper *first model* in a cascade, and (b) depth-wise exits within a view. Token pruning/merging is the wrong axis for forensic detectors: Tintype's own ToMe test lost 6–14 recall points, and forensic-specific work (ForensicZip, 2026) argues saliency-based token selection discards the low-saliency background where traces live, while Panoptic Patch Learning argues evidence is spread across all patches.

### Cited Findings

**2.1 Wisdom of Committees (cascades of off-the-shelf models)** [search]
- Wang, Kondratyuk, Christiansen, Kitani, Movshovitz-Attias, Eban. "Wisdom of Committees: An Overlooked Approach To Faster and More Accurate Models." ICLR 2022, arXiv 2012.01988. Cascades (run small model first, stop if confident) outperform single models in all compute regimes; EfficientNet cascade 5.4× faster than B7 and ViT cascade 2.3× faster than ViT-L-384 at equal accuracy; up to 5.5× lower latency and 3.5× higher throughput on TPUv3; two B5s match B7 at ~50% less FLOPs — [arXiv 2012.01988](https://arxiv.org/abs/2012.01988), [ICLR poster](https://www.iclr.cc/virtual/2022/poster/6397)
- **Tintype applicability.** Tintype is already a cascade *over views*; the paper's lesson is that a cascade *over models* compounds. Concrete change: a ViT-Ti/16 (≈5.7 M params, ~¼ of ViT-S FLOPs) or a smaller-input ViT-S (e.g. 160 px fit view, ~half the tokens) distilled from Tintype's own scores as stage 0, deciding the very confident images (most reals in moderation traffic) and handing the rest to the current pipeline. Expected speed: if stage 0 settles ~50–60% of images with one cheap view, average cost per image can fall by ~30–45% (inference from the 1.7-view baseline; must be measured). Precision/recall: designed to be equal by fitting stage-0 margins on development sets with the same "within 0.001 pooled P/R" rule already used for view margins. Engineering: the ViT builder in mps_vit.rs/cpu_vit.rs is generic over depth/width presumably, so a Ti/16 variant is mostly a config + distillation training run. Risk: the cheap model's confident mistakes are unrecoverable; keep its margins wide.

**2.2 Early-exit survey** [search]
- Laskaridis, Kouris, Lane. "Adaptive Inference through Early-Exit Networks: Design, Challenges and Directions." EMDL 2021, arXiv 2106.05022 — reference for exit-head placement, confidence criteria and training schemes — [arXiv 2106.05022](https://arxiv.org/abs/2106.05022)
- **Tintype applicability.** Depth-wise exits: attach a LayerNorm+linear head to the class token after block 6 and block 9; exit a *view* early if |logit| clears a margin. Because Tintype reads only the class token in the last block already (speed change 5), an intermediate head costs one 384-d dot product. Expected speed: at most ~25–40% on views that exit at block 9/6 (inference). Risk: forensic low-level cues are thought to be strongest in early/mid layers for some detectors, so mid-depth heads may be surprisingly good — or the opposite; needs a frozen-backbone head-training experiment (cheap) to find out. Batching caveat on GPU: per-sample depth exits fragment batches; Tintype's "fill the batch with whatever views are waiting" scheduler would need a per-depth queue.

**2.3 Forensic-specific adaptive inference (2026)** [search, secondary]
- arXiv 2609.05320 (2026), "Adaptive Gated Deepfake Detection for Low-Resolution and Resource-Constrained Environments": routes samples by image-quality features (blur, compression, contrast) to different network depths, spending less on easy inputs — [arXiv 2609.05320](https://arxiv.org/pdf/2609.05320)
- DFD-HR, "Generalizable Deepfake Detection via Hierarchical Routing Learning" (CVPR 2026): PEFT on a vision foundation model with an early-layer step deciding continue-or-exit; known only from a secondary summary — [papernotes summary](https://en.papernotes.org/CVPR2026/ai_safety/dfd-hr_generalizable_deepfake_detection_via_hierarchical_routing_learning/)
- **Tintype applicability.** Quality-aware routing maps onto Tintype's view schedule: images that are tiny/heavily recompressed carry little native-pixel evidence, so for them the native crops are near-duplicates of the fit view; a rule "if long side < 512 or JPEG quality estimate < Q, score fit + centre only" would save views where extra views cannot add information. Zero training; must be validated per set. Both papers are face-deepfake oriented and only abstract/secondary-level evidence was found.

**2.4 Token pruning / merging in general ViTs** [abstract, known]
- Rao et al. "DynamicViT: Efficient Vision Transformers with Dynamic Token Sparsification." NeurIPS 2021, arXiv 2106.02034 — learned prediction module prunes ~66% of tokens hierarchically; abstract reports 31–37% FLOPs reduction and >40% throughput gain within 0.5% top-1 — [arXiv 2106.02034](https://arxiv.org/abs/2106.02034)
- Liang et al. "Not All Patches are What You Need: Expediting Vision Transformers via Token Reorganizations" (EViT). ICLR 2022, arXiv 2202.07800 — keeps tokens most attended by CLS, fuses the rest; abstract: DeiT-S inference +50% speed with −0.3% accuracy — [arXiv 2202.07800](https://arxiv.org/abs/2202.07800)
- Yin et al. "A-ViT: Adaptive Tokens for Efficient Vision Transformer." CVPR 2022, arXiv 2112.07658 — per-token halting (ACT-style) with no extra parameters — [arXiv 2112.07658](https://arxiv.org/abs/2112.07658)
- Bolya et al. "Token Merging: Your ViT But Faster" (ToMe). ICLR 2023, arXiv 2210.09461 — training-free bipartite merging; abstract: 2× throughput on ViT-L@512 / ViT-H@518 with 0.2–0.3% accuracy drop — [arXiv 2210.09461](https://arxiv.org/abs/2210.09461)
- **Tintype's measured counter-evidence:** ToMe on Tintype: recall 0.848 → 0.786 (r=8) and → 0.708 (r=13) on 1,000 images; dropping instead of merging no better; merging only in the last 4 blocks costs ~6 verdicts/1,000 but saves only ~15% — [docs/BENCHMARK.md](../../docs/BENCHMARK.md)

**2.5 ForensicZip — token reduction designed for forensic VLMs** [fetched]
- Lai, Yu, Wang, Shen, Xu, Cao. "ForensicZip: More Tokens are Better but Not Necessary in Forensic Vision-Language Models." arXiv 2603.12208 (Mar 2026). Training-free token pruning with optimal transport and high-frequency priors; argues existing pruning is "largely semantic-driven" and discards background regions where manipulation traces reside; at 10% token retention reports 2.97× speedup and >90% FLOPs reduction with claimed SOTA detection — [arXiv 2603.12208](https://arxiv.org/abs/2603.12208). It targets MLLM detectors with hundreds of visual tokens, not a 197-token ViT-S.
- **Tintype applicability.** Explains Tintype's ToMe failure: CLS-attention/similarity-based merging groups exactly the low-saliency, self-similar background tokens whose fine pixel statistics are the forensic signal. The only token-reduction variant consistent with this evidence is selection by a *high-frequency prior* — which is what Tintype already does at the macro level when it picks textured windows. Recommendation: do not pursue intra-view token pruning further; treat window selection as the "token pruning" layer.

### Inferences
- In Tintype the cheapest remaining adaptive-compute lever is the **order** and **content** of views (fit view last; quality-aware view caps), then a **stage-0 cheap model**, then **depth-wise exits** (needs head training and a scheduler change).
- General-ViT token reduction results (0.2–0.5% ImageNet loss) do not transfer to forensics; Tintype's own ToMe result plus ForensicZip's argument are the best available evidence, and they agree.

### Gaps
- No paper found that trains depth-wise early-exit heads on an AI-generated-image ViT and reports where (which block) forensic evidence becomes linearly decodable.
- No forensic study of A-ViT/DynamicViT with *training* (Tintype's ToMe test was training-free; learned pruning with fine-tuning might lose less, but nothing published shows it).

---

## 3. Quantization (INT8/FP8, PTQ for ViTs) and knowledge distillation for detectors

### Takeaway
General ViT PTQ at 8 bits is near-lossless on ImageNet (PTQ4ViT <0.5% drop; FQ-ViT fully-integer including LayerNorm/Softmax), and the only peer-reviewed forensic quantization study (IJCAI 2025) reports 2–3-bit CNN detectors matching or beating full precision on cross-GAN tests — but there is **no published evidence on INT8 ViT detectors for subtle diffusion artifacts under recompression**, and adjacent object-detection evidence finds static INT8 models more noise-sensitive. For Tintype the concrete win is INT8 matmuls in the hand-written CPU kernels (AVX-512 VNNI / AVX-VNNI, ARM SDOT/I8MM), with first/last layers, LayerNorm and Softmax in higher precision, gated by a verdict-flip test. On Apple GPU, INT8 brings little compute speed-up via MPSGraph (inference; weight-only quantization mainly saves memory), so it is a CPU/Linux play.

### Cited Findings

**3.1 PTQ4ViT** [abstract, known]
- Yuan et al. "PTQ4ViT: Post-Training Quantization for Vision Transformers with Twin Uniform Quantization." ECCV 2022, arXiv 2111.12293 — twin-uniform quantizers for post-softmax and post-GELU activations plus Hessian-guided scaling; abstract: <0.5% ImageNet top-1 drop at 8 bits for ViT/DeiT/Swin — [arXiv 2111.12293](https://arxiv.org/abs/2111.12293)
- **Tintype applicability.** Gives the recipe for which tensors need non-uniform treatment (post-softmax, post-GELU). Tintype's CPU engine could quantise the four big matmuls per block (QKV, attn-out, MLP up, MLP down — the bulk of FLOPs) to INT8 weights × INT8 activations with per-channel weight scales and per-tensor activation scales calibrated on a few hundred development images, keeping patch-embedding, attention softmax and LayerNorm in f32. Expected speed: INT8 dot-product instructions give 2–4× peak MAC throughput over f32 on AVX-512 VNNI and ARMv8.2 SDOT/I8MM (inference from ISA widths; real end-to-end gain lower, maybe 1.5–2.5× on CPU). Engineering: new kernels (quantise-activations-on-the-fly + int32 accumulate + dequant) — moderate, but the project already writes NEON/AVX2/AVX-512 kernels. Risk: forensic signal is a tiny perturbation on content; activation outliers (see RepQ-ViT) may swamp it; require "no more than N verdict flips per 10k and pooled P/R within 0.001".

**3.2 FQ-ViT** [abstract, known]
- Lin et al. "FQ-ViT: Post-Training Quantization for Fully Quantized Vision Transformer." IJCAI 2022, arXiv 2111.13824 — Power-of-Two Factor for LayerNorm inputs and Log-Int-Softmax enable fully integer inference incl. LayerNorm/Softmax with ~1% accuracy loss at 8 bits — [arXiv 2111.13824](https://arxiv.org/abs/2111.13824)
- **Tintype applicability.** Only needed if LayerNorm/Softmax become a bottleneck after matmuls go INT8; at 197 tokens they are a small share. Low priority.

**3.3 RepQ-ViT** [abstract, known]
- Li et al. "RepQ-ViT: Scale Reparameterization for Post-Training Quantization of Vision Transformers." ICCV 2023, arXiv 2212.08254 — handles severe inter-channel variation of post-LayerNorm activations (per-channel quant then reparameterised to layer-wise) and log2 post-softmax quantization; aimed at 4-bit — [arXiv 2212.08254](https://arxiv.org/abs/2212.08254)
- **Tintype applicability.** The post-LayerNorm outlier-channel trick can be folded into weights offline (scale LN affine and the next linear's input columns), so the Rust engine's INT8 matmul can stay per-tensor for activations. Use this if naive INT8 flips verdicts.

**3.4 Forensic quantization: "Unlocking the Potential of Lightweight Quantized Models for Deepfake Detection"** [fetched]
- Tao, Qin, Ding, Tan, Wang, Wang. IJCAI 2025, pp. 520–528, DOI 10.24963/ijcai.2025/59. ResNet-23 on NPR inputs; Connected Quantized Block keeps full-precision shortcuts "to retain method-specific textures"; 2-bit: 91.8% Acc / 95.1% AP cross-GAN on ForenSynths, 3-bit 91.4% / 93.9%; 2.7 → 0.25 GFLOPs and 62 → 5.0 MB (10.8× / 12.4×); claims ~+2.4% Acc over prior SOTA on UniversalFakeDetect diffusion sets — [IJCAI 2025](https://www.ijcai.org/proceedings/2025/59)
- **Tintype applicability.** Positive evidence that forensic CNNs tolerate low precision *if* a high-precision path preserves fine textures — supports "INT8 matmuls with f32 residual stream" for Tintype (the residual stream stays f32/f16 in any sane INT8 design). Caveats: old GAN benchmarks, CNN not ViT, NPR input chosen *because* it tolerates quantization error better than raw RGB — Tintype uses raw RGB.

**3.5 Community INT8 evidence (unreviewed)** [search]
- A Hugging Face model card for an AI-image detector reports identical confusion matrix FP16→INT8 ONNX on 700 images and balanced accuracy 92.32% → 92.68% on a JPEG/downscale slice — [HF card](https://huggingface.co/KabirAcharya/waterspy-sieve-ft1-int8). Unreviewed; small sample.
- Object-detection robustness study: static INT8 models "consistently showed increased sensitivity (larger relative drops) to noise" than FP32/FP16; degradation-aware calibration helped little — [arXiv 2508.19600](https://arxiv.org/abs/2508.19600v1)
- **Tintype applicability.** Mixed signal; justifies calibrating INT8 activation scales on *degraded* images (messenger round-trips) and testing specifically on re-shared sets.

**3.6 Distillation for detectors** [search]
- DistilDIRE (arXiv 2406.00856, 2024): small student trained with classification + KD loss from DIRE, positioned as fast/cheap for real-world use — [arXiv 2406.00856](https://arxiv.org/abs/2406.00856)
- Tiny-LaDeDa: logit distillation on patch-level scores, 375× fewer FLOPs, small accuracy drop — [arXiv 2406.09398](https://arxiv.org/abs/2406.09398)
- DINO-Detect (arXiv 2511.12511, Nov 2025): teacher–student distillation for *robustness* (student on blurred inputs matches DINOv3 teacher features on sharp ones) — [alphaxiv 2511.12511](https://www.alphaxiv.org/abs/2511.12511)
- LAID (Chivaran & Ni, arXiv 2507.05162, PST 2025): off-the-shelf lightweight networks on a GenImage subset reach "competitive accuracy, even under adversarial conditions" at much lower memory/compute (model list and numbers not in the abstract) — [arXiv 2507.05162](https://arxiv.org/abs/2507.05162)
- **Tintype applicability.** Two uses. (a) Stage-0 student (§2.1) distilled from Tintype's six-view mean logit (so a single cheap view learns to mimic the multi-view ensemble — "ensemble distillation"), which directly targets fewer views at equal accuracy. (b) Distil Community Forensics' 384-px knowledge *into* Tintype's ViT-S (train on CF logits as soft targets for the images where CF is right and Tintype wrong), eliminating the second network entirely — keeps 530 photos/s while recovering part of the second-opinion recall. DINO-Detect's sharp→degraded feature matching is a training trick for robustness to re-shares, also free at inference.

### Inferences
- INT8 is a CPU/Linux speed lever (35 photos/s on aarch64 Linux is where it matters most); expected 1.5–2× end-to-end, conditional on a verdict-flip gate. On Apple GPU the engine is already 16-bit and MPSGraph's integer paths are unlikely to beat f16 matmul throughput (not verified).
- Distillation of the two-model system into one ViT-S is the highest-value distillation target: it attacks both speed (no second network) and the FPR problem (one calibrated score).

### Gaps
- No study of INT8/FP8 ViT AI-image detectors on modern diffusion/commercial generators under JPEG/resizing was found. This must be measured in-house.
- No published ensemble-to-single-view distillation for forensic detectors was found.

---

## 4. Ensembling and fusion: how to combine two detectors so recall rises without the FPR jump (1.2% → 3.1%)

### Takeaway
Tintype's photo second opinion is structurally an **OR-like rescue rule with a fixed, hand-chosen combination and no re-calibration of the combined threshold**: the band (−7, 0.5) is almost entirely on the "real" side of the threshold, so the second network mostly gets the chance to *add* positives, and every real photo in that band gets a fresh chance to be flagged. The fusion literature (classical Neyman–Pearson fusion; logistic-regression stacking in deepfake audio; multi-objective fusion 2026) says the fair comparison is **recall at a fixed FPR**, and the fix is to (1) learn the combiner (logistic regression on z1, z2 and simple view statistics), (2) re-set the combined threshold on held-out reals so overall FPR equals the default's 1.5% target, and (3) make the band symmetric/learned so the second model can also *veto* borderline positives. Mixture-of-experts detectors (MoLE, HyperDet, ALEI, OmniAID, AIDE) improve generalisation but at 1×+ backbone cost; for Tintype, the cheaper analogue is a learned gate deciding *whether* to call the second model.

### Cited Findings

**4.1 Tintype's own numbers (context)**
- Batch 7: default 0.988 / 0.897 at FPR 1.2%; with second opinion 0.972 / 0.935 at FPR 3.1% (329 real photos flagged); 48% of photos go to stage 2; throughput falls to 281–288 photos/s GPU — [docs/BENCHMARK.md](../../docs/BENCHMARK.md)
- The `--sensitive` operating point of Tintype *alone* (calibrated FPR 5%) gives 0.963 / 0.960 on the newest blind batch — [README operating points](../../README.md). So part of the second-opinion recall gain may simply be a threshold move.
- Rule: band lo=−7, hi=0.5, output = sigmoid(0.5·(0.5·z1 + z2 − 0)) — [models/tintype.json](../../models/tintype.json), [src/detector.rs](../../src/detector.rs)

**4.2 Score-level fusion practice** [search]
- "Analysis of Score-Level Fusion Rules for Deepfake Detection," Applied Sciences 12(15):7365 (2022) — compares fusion rules for generalisation of deepfake detectors — [MDPI](https://www2.mdpi.com/2076-3417/12/15/7365) (results not visible in snippet)
- Audio deepfake countermeasures (arXiv 2509.12003, 2025): logistic-regression fusion of SSL-based CMs vs sum of calibrated LLRs; the choice of attacks used to train the fuser strongly affects its generalisation — [arXiv 2509.12003](https://arxiv.org/pdf/2509.12003)
- "Evolutionary Multi-Objective Fusion of Deepfake Speech Detectors" (arXiv 2604.01330, WCCI/CEC 2026): logistic regression and averaging as baselines; best variant 2.37% EER on ASVspoof 5, beating both — [arXiv 2604.01330](https://arxiv.org/abs/2604.01330)
- **Tintype applicability.** Concrete change: fit a 3–5-parameter logistic regression on development photos where both scores exist: features z1 (Tintype calibrated logit), z2 (CF logit), and optionally view-disagreement (max − min view logit) and n_views_used. Then choose the decision threshold on the combined score so that the *pooled real-source-averaged* FPR equals the default's calibrated target (1.5%), exactly as Tintype's existing calibration does per operating point. Report recall at that FPR against Tintype-alone at the same FPR. The audio-fusion paper's warning applies: fit the combiner on generators/sources disjoint from the blind sets, or it overfits. Speed: identical to today (combiner is a dot product). Engineering: a few lines in detector.rs plus a fitting script. Risk: may show the second opinion adds little at matched FPR — which is itself a useful result (then drop it, or keep it only for `--sensitive`).

**4.3 Cascades and gating** [search]
- Wisdom of Committees (§2.1): cascade exit rule is a confidence threshold on the first model; the paper shows cascades match ensembles' accuracy at a fraction of cost — [arXiv 2012.01988](https://arxiv.org/abs/2012.01988)
- **Tintype applicability.** The current band sends 48% of photos to CF — far more than a cost-efficient cascade would. A gate fitted to maximise "expected recall gain per real-FPR cost" would send fewer photos (faster) and specifically those where the models are known to disagree usefully (e.g. images whose generator family is GAN-like by a cheap spectral cue, where CF is stronger per the repo). Make the band symmetric around 0 (e.g. (−3, +3)) so CF can also pull false positives back down; the current hi=0.5 means almost no photo Tintype flags can be rescued by CF.

**4.4 Mixture-of-experts / multi-expert detectors** [search]
- MoLE — "Mixture of Low-rank Experts for Transferable AI-Generated Image Detection," arXiv 2404.04883 (2024): shared + separate LoRA experts in deeper CLIP-ViT MLPs; better cross-generator generalisation and perturbation robustness — [arXiv 2404.04883](https://arxiv.org/abs/2404.04883v1)
- HyperDet, arXiv 2410.06044 (2024): hypernetwork-generated LoRA experts merged into an ensemble — [arXiv 2410.06044](https://arxiv.org/html/2410.06044v1)
- ALEI (Adaptive Low-level Experts Injection), arXiv 2504.00463 (2025): one LoRA expert per low-level cue (e.g. noise), fused by cross-attention (details via third-party review) — [themoonlight review](https://www.themoonlight.io/review/exploring-the-collaborative-advantage-of-low-level-information-on-generalizable-ai-generated-image-detection)
- OmniAID, arXiv 2511.08423 (Nov 2025): decoupled MoE separating content-specific flaws from universal generation artifacts (abstract only) — [scour.ing link to arXiv 2511.08423](https://scour.ing/p/https://arxiv.org/abs/2511.08423)
- AIDE (§1.3): expert fusion of semantic and low-level branches — [ICLR 2025](https://proceedings.iclr.cc/paper_files/paper/2025/hash/b0303773962ea1b5394c3a83cc7dd066-Abstract-Conference.html)
- **Tintype applicability.** LoRA experts that are *merged* into the base weights at inference (HyperDet-style merging) cost nothing at inference; unmerged MoE routing costs extra passes. Concrete low-cost variant: train 2–3 LoRA adapters on the ViT-S (e.g. GAN-family, diffusion-family, commercial-2025-26) and merge them by weight averaging into one checkpoint ("model soup") — inference cost unchanged. This is an alternative to running CF as a second network: CF's advantage on older GANs could be absorbed by a GAN-expert LoRA. Risk: merged experts may regress the current strengths; Tintype's training pipeline (MPSGraph autodiff) would need LoRA support (moderate).

### Inferences
- The FPR rise from 1.2% to 3.1% is predictable from the rule: an asymmetric rescue band plus a combination threshold fixed at 0 without re-calibration on reals. With a learned combiner and a combined threshold re-calibrated to 1.5% FPR, the system will by construction stay at the default FPR; whether recall still rises (above 0.897) is the empirical question. Prior repo estimate (seen data, optimistic) already suggested a rule that "flags those where the two scores add up high enough" moves 0.986/0.876 → 0.983/0.905 at nearly constant precision ([docs/BENCHMARK.md](../../docs/BENCHMARK.md)), which suggests a calibrated version can deliver a smaller but FPR-neutral recall gain.
- The honest baseline for the current second opinion is Tintype alone thresholded at FPR ≈ 3.1%; the README's `--sensitive` point (5% FPR → 0.960 recall) implies Tintype alone at 3.1% may already be close to 0.935. This should be checked before investing in better fusion.
- Distilling CF into Tintype (§3.6) or merging a GAN-expert LoRA (§4.4) are the two routes to the second opinion's recall without its 2× slowdown.

### Gaps
- No AI-generated *image* detection paper found that reports recall at a fixed FPR before/after fusing two detectors with a re-calibrated threshold; the fusion evidence found is from audio deepfake work (EER/minDCF).
- MoE detector papers report accuracy/AUC on academic benchmarks, not FPR at fixed operating points or inference cost.

---

## 5. Hardware-aware efficiency: fused attention, batching with early exit, CPU inference, smaller backbones as forensic detectors

### Takeaway
At 197 tokens, attention is a minor cost and FlashAttention-style fusion helps little (consistent with Tintype's measured "no faster"); the time is in the four dense matmuls per block. The remaining hardware levers are INT8 on CPU (§3), larger effective batch on GPU across images and stages (already done), and fewer FLOPs per view via a smaller/efficient backbone. Efficient backbones (EfficientViT, FastViT, MobileViT) are general-vision results; forensic evidence for lightweight backbones exists (LAID 2025, Tiny-LaDeDa, IJCAI 2025 quantized ResNet-23) but only on older/academic benchmarks. Given Community Forensics' finding that data diversity beats architecture, a distilled ViT-Ti/16 or a "ViT-S with fewer blocks" is the most plausible small backbone, used as a cascade stage rather than a replacement.

### Cited Findings

**5.1 FlashAttention** [abstract, known]
- Dao, Fu, Ermon, Rudra, Ré. "FlashAttention: Fast and Memory-Efficient Exact Attention with IO-Awareness." NeurIPS 2022, arXiv 2205.14135 — IO-aware tiled exact attention; gains grow with sequence length — [arXiv 2205.14135](https://arxiv.org/abs/2205.14135)
- **Tintype applicability.** Already effectively tested (MPSGraph `scaledDotProductAttention` no faster at 197 tokens — [docs/BENCHMARK.md](../../docs/BENCHMARK.md)). Only relevant if Tintype moves to larger native regions (e.g. 448² = 784 tokens per SPAI/NaViT-style views), where attention's quadratic term starts to matter.

**5.2 Efficient ViT backbones** [abstract, known]
- Cai, Li, Hu, Gan, Han. "EfficientViT: Lightweight Multi-Scale Attention for High-Resolution Dense Prediction." ICCV 2023, arXiv 2205.14756 — ReLU linear attention; hardware-friendly (no softmax) — [arXiv 2205.14756](https://arxiv.org/abs/2205.14756)
- Vasu, Gabriel, Zhu, Tuzel, Ranjan. "FastViT: A Fast Hybrid Vision Transformer using Structural Reparameterization." ICCV 2023, arXiv 2303.14189 — RepMixer token mixer, reparameterised at inference; designed for Apple mobile hardware — [arXiv 2303.14189](https://arxiv.org/abs/2303.14189)
- Mehta & Rastegari. "MobileViT: Light-weight, General-purpose, and Mobile-friendly Vision Transformer." ICLR 2022, arXiv 2110.02178 — [arXiv 2110.02178](https://arxiv.org/abs/2110.02178)
- **Tintype applicability.** All three downsample early (strided convs / stage pyramids), which is risky for pixel-level forensic traces — the same concern as FlexiViT patch-32 and ToMe. A hand-written Rust engine for a hybrid conv/attention net is a large rewrite (new ops: depthwise convs, reparameterisation, BN folding) on both MPSGraph and CPU. Not recommended over "same ViT, fewer blocks/narrower" which reuses the existing engine.

**5.3 Lightweight forensic detectors** [fetched/search]
- LAID (arXiv 2507.05162, PST 2025): lightweight off-the-shelf nets competitive on GenImage subset in spatial/spectral/fusion domains — [arXiv 2507.05162](https://arxiv.org/abs/2507.05162)
- Tiny-LaDeDa: 4 conv layers, 375× fewer FLOPs than LaDeDa — [arXiv 2406.09398](https://arxiv.org/abs/2406.09398)
- IJCAI 2025 quantized ResNet-23: 0.25 GFLOPs at 2 bits — [IJCAI 2025](https://www.ijcai.org/proceedings/2025/59)
- **Tintype applicability.** These support a tiny *auxiliary* model (window selector or stage-0 gate) more than a backbone swap. A Tiny-LaDeDa-style 4-layer conv net on the full-resolution luma plane is trivial to implement in the CPU engine (a few 3×3 convs) and could replace variance-based window ranking with learned evidence ranking — improving which windows the ViT sees first, which in turn lowers views-per-image under early exit.

**5.4 Batching with early exit (engineering, Tintype-specific)**
- Tintype already refills fixed-size GPU batches with the next views of any waiting image regardless of stage (1.61× on GPU) and runs one view per thread on Linux CPU (35.2 vs 17.4 photos/s for intra-matmul threading) — [docs/BENCHMARK.md](../../docs/BENCHMARK.md)
- **Tintype applicability.** Any new adaptive scheme (stage-0 model, depth exits) must fit this scheduler: stage-0 is easy (separate graph, separate queue); depth exits are harder on GPU (MPSGraph graph is compiled end-to-end; splitting into 2–3 sub-graphs with a host round-trip per batch adds latency).

### Inferences
- The biggest untapped hardware win is INT8 matmul on CPU (Linux/x86 and aarch64), plausibly 1.5–2×, contingent on a verdict-flip gate.
- On Apple GPU, per-view FLOPs are the limit; only fewer views (better selection/order, stage-0 gate) or a cheaper network for some views will move 530 photos/s materially.

### Gaps
- No ONNX Runtime/OpenVINO study of INT8 ViT-S *forensic* detectors on CPU was found; general OpenVINO/ONNX INT8 ViT speedups were not re-verified this session.
- No published forensic evaluation of EfficientViT/FastViT/MobileViT backbones on current commercial generators was found.

---

## 6. Ranked shortlist for Tintype (most promising first)

### Takeaway
The best ideas are those that (a) use data Tintype already stores (per-view logits, both-model scores) and (b) need no engine rewrite. Ranked by expected value per engineering hour:

### Cited Findings
1. **Re-fit the photo second opinion as calibrated, learned fusion at fixed FPR** (logistic stacking on z1, z2 [+ view disagreement]; symmetric/learned band; combined threshold re-calibrated on held-out reals to the default's 1.5% FPR; compare against Tintype alone at the same FPR). Grounding: fusion/stacking practice ([arXiv 2509.12003](https://arxiv.org/pdf/2509.12003), [arXiv 2604.01330](https://arxiv.org/abs/2604.01330)); cascade logic ([arXiv 2012.01988](https://arxiv.org/abs/2012.01988)); Tintype rule and numbers ([src/detector.rs](../../src/detector.rs), [docs/BENCHMARK.md](../../docs/BENCHMARK.md)). Effect: FPR held at default by construction; recall gain likely smaller than +0.038 but real if it survives; a narrower band also cuts the share sent to CF (faster than 281 photos/s). Cost: hours. Risk: may reveal the second opinion is mostly a threshold shift.
2. **Offline view-pooling and view-order study** on stored per-view logits: mean vs max vs top-2 mean vs trimmed mean vs log-mean-exp; native-only vs with fit view; fit view scored last; add a "least-textured non-flat window" and an "highest-DCT-energy window" variant. Grounding: PatchCraft ([arXiv 2311.12397](https://arxiv.org/abs/2311.12397)), SSP ([arXiv 2402.01123](https://arxiv.org/abs/2402.01123)), AIDE ([ICLR 2025](https://proceedings.iclr.cc/paper_files/paper/2025/hash/b0303773962ea1b5394c3a83cc7dd066-Abstract-Conference.html)), Panoptic Patch Learning few-patch bias ([arXiv 2504.01396](https://arxiv.org/abs/2504.01396)), B-Free on resizing ([CVPR 2025](https://openaccess.thecvf.com/content/CVPR2025/papers/Guillaro_A_Bias-Free_Training_Paradigm_for_More_General_AI-generated_Image_Detection_CVPR_2025_paper.pdf)). Effect: either equal accuracy with fewer views (speed) or higher recall at equal views. Cost: a script plus re-fitting early-exit margins; zero engine work for pooling, small Rust change for new window choices.
3. **Stage-0 cascade with a distilled cheap model** (ViT-Ti/16, or ViT-S with 6 blocks, or a 160-px fit view) trained to mimic Tintype's all-view mean logit, with exit margins fitted under the existing "pooled P/R within 0.001" rule. Grounding: Wisdom of Committees ([arXiv 2012.01988](https://arxiv.org/abs/2012.01988)); Tiny-LaDeDa distillation ([arXiv 2406.09398](https://arxiv.org/abs/2406.09398)). Effect: plausibly 1.3–1.7× throughput at equal P/R (inference; must be measured). Cost: moderate (training run + second graph + scheduler queue).
4. **INT8 matmuls in the CPU engine** (per-channel weight scales, calibrated activation scales on degraded images, f32 residual/LayerNorm/softmax, RepQ-style LN folding if needed), shipped only if verdict flips ≤ a few per 10k. Grounding: PTQ4ViT ([arXiv 2111.12293](https://arxiv.org/abs/2111.12293)), RepQ-ViT ([arXiv 2212.08254](https://arxiv.org/abs/2212.08254)), IJCAI 2025 forensic quantization ([IJCAI](https://www.ijcai.org/proceedings/2025/59)); caution from [arXiv 2508.19600](https://arxiv.org/abs/2508.19600v1). Effect: ~1.5–2× on Linux/x86 CPU (35 → ~55–70 photos/s on aarch64 Linux, inference). Cost: new kernels (moderate–high). Risk: forensic-signal loss under recompression is unmeasured anywhere.
5. **Absorb the second network into Tintype**: distil CF's 384-px logits (where CF is right and Tintype wrong) into the ViT-S, or train a GAN-family LoRA and merge it into the weights. Grounding: DistilDIRE/Tiny-LaDeDa distillation ([arXiv 2406.00856](https://arxiv.org/abs/2406.00856)); merged LoRA experts ([HyperDet](https://arxiv.org/html/2410.06044v1), [MoLE](https://arxiv.org/abs/2404.04883v1)). Effect: second-opinion-like recall at default speed (530 photos/s) and a single calibrated score. Cost: a training phase. Risk: regression on current commercial generators.

### Inferences
- Explicitly *not* recommended: intra-view token pruning/merging (Tintype's measured −6 to −14 recall points; ForensicZip's argument), coarse-patch FlexiViT as a sole view, hybrid efficient backbones (EfficientViT/FastViT/MobileViT) requiring an engine rewrite, and fused attention at 197 tokens.
- Items 1 and 2 can be done this week from existing score files; items 3–5 need training and should be preregistered in the same way the repo already preregisters blind batches.

### Gaps
- All speed estimates for items 3 and 4 are inferences from FLOP/ISA ratios, not measurements; none of the forensic papers reports wall-clock inference on comparable hardware.
- Whether Tintype's ViT shows few-patch bias (which decides between max-like pooling and evidence-spreading training) is unknown until item 2 is run.
