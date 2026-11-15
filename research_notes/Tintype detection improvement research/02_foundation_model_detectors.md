# Foundation-model / representation-based detection of AI-generated images (2021–2026), and what it means for Tintype

Scope: CLIP / DINOv2 / DINOv3 / SigLIP / PE / MAE-style backbones used frozen, with PEFT (LoRA, SVD, prompts, adapters), or fully fine-tuned; MLLM/VLM explainable detectors; distillation; scaling. Tintype baseline for reference: ViT-S/16 (22M), 224 px, initialised from Community Forensics (CommFor) and fully fine-tuned on ~1,200 generators; six views, averaged logits, early exit; P 0.988 / R 0.897 on newest blind batch; weak on messenger-recompressed images (27–36% recall), NTIRE 2026 distorted test (2.2% recall at 1% FPR), older GANs (84–89%), face swaps (14%).

Verification legend: numbers marked **[V]** were read from the paper's arXiv abstract/HTML during this session. Numbers marked **[R]** are recalled from the paper's abstract and were *not* re-fetched in this session — the report writer should treat them as needing a check before quoting. Citations are primary arXiv/CVF/OpenReview links.

Cost reference used throughout: ViT-S/16 at 224 px ≈ 4.6 GFLOPs, ViT-B/16 ≈ 17.6 GFLOPs (≈3.8× ViT-S) — [DeiT, Touvron et al. 2021](https://arxiv.org/abs/2012.12877) **[R]**; ViT-L is roughly 13× ViT-S at the same resolution and patch size (inference, not a cited figure).

---

## Q1. Frozen-feature and PEFT approaches: which beat full fine-tuning on unseen generators, and by how much?

### Takeaway
On the *classic* protocol (train on ProGAN or SD v1.4 only, test on unseen generators), frozen CLIP/DINO features with a light head or tiny PEFT (UnivFD → RINE → FatFormer → C2P-CLIP → Effort) clearly beat full fine-tuning — Effort reports 95.19 mAcc vs 86.22 for full fine-tuning (FFT) of the same CLIP ViT-L/14 on UniversalFakeDetect. But when the training set is itself broad and aligned (B-Free, Community Forensics, ScaleDF), end-to-end fine-tuning wins by a large margin (B-Free: 95.2 vs 68.5 bAcc, e2e vs linear probe). Tintype is in the second regime, so the transferable ideas are *regularised* fine-tuning and multi-layer readout, not "freeze and probe".

### Paper catalogue (with Tintype applicability)

**1. UnivFD — "Towards Universal Fake Image Detectors that Generalize Across Generative Models"** — Utkarsh Ojha, Yuheng Li, Yong Jae Lee. CVPR 2023. [arXiv 2302.10174](https://arxiv.org/abs/2302.10174)
- Frozen CLIP ViT-L/14 final-layer embedding + linear probe / nearest neighbour, trained on ProGAN only; abstract reports +15.07 mAP and +25.90% accuracy over prior SOTA on unseen diffusion/autoregressive models **[R]** — [arXiv](https://arxiv.org/abs/2302.10174).
- Later work shows its ceiling: averaged over 20 test sets UnivFD gets 80.9 Acc / 94.3 AP **[V]** — [RINE paper](https://arxiv.org/html/2402.19091v2); on B-Free's modern-generator table UnivFD averages 62.4 bAcc **[V]** — [B-Free](https://arxiv.org/html/2412.17671v2); frozen CLIP gets 55.9% on So-Fake-OOD **[V]** — [DINOv3 paper](https://arxiv.org/html/2511.22471).
- *Tintype applicability*: none directly — a frozen ViT-L/14 at 224 costs ~13× Tintype per view and is far worse on current generators than Tintype's own numbers. Historical anchor only.

**2. "Raising the Bar of AI-generated Image Detection with CLIP"** — Davide Cozzolino, Giovanni Poggi, Riccardo Corvi, Matthias Nießner, Luisa Verdoliva. CVPR Workshops (WMF) 2024. [arXiv 2312.00195](https://arxiv.org/abs/2312.00195) · [CVF](https://openaccess.thecvf.com/content/CVPR2024W/WMF/papers/Cozzolino_Raising_the_Bar_of_AI-generated_Image_Detection_with_CLIP_CVPRW_2024_paper.pdf)
- Frozen CLIP ViT-L/14 features + linear SVM, trained few-shot on a small set of paired real/fake images (fakes generated from the reals' captions); abstract claims ~+6% AUC in-distribution and ~+13% AUC out-of-distribution over SOTA, and robustness to social-network impairments **[R]** — [arXiv](https://arxiv.org/abs/2312.00195).
- *Tintype applicability*: the transferable idea is the *content-paired* data construction (real photo + fake generated from its caption), which removes semantic shortcuts — useful for building a small paired calibration/validation set for new generators. Zero inference cost. The CLIP backbone itself is not worth adopting.

**3. RINE — "Leveraging Representations from Intermediate Encoder-blocks for Synthetic Image Detection"** — Christos Koutlis, Symeon Papadopoulos. ECCV 2024. [arXiv 2402.19091](https://arxiv.org/abs/2402.19091) · [code](https://github.com/mever-team/rine)
- Frozen CLIP ViT-L/14; CLS tokens of all intermediate blocks projected and combined with a trainable per-block importance estimator. +10.6% average absolute improvement over SOTA on 20 test sets; best models train in one epoch (~8 min) **[V]** — [arXiv](https://arxiv.org/abs/2402.19091).
- Ablation (only last layer ≡ UnivFD vs full RINE): Acc 80.8 → 89.8, AP 94.2 → 98.7 averaged over 1/2/4-class training **[V]** — [RINE HTML](https://arxiv.org/html/2402.19091v2). "Many Transformer blocks obtain higher maximum importance frequency than the last one" **[V]** — same.
- Robustness (4-class, avg 20 sets): clean 91.5 Acc → compression 88.7, blur 86.5, noise 84.6, combined 82.7 **[V]** — same.
- On a modern-generator suite RINE averages only 65.0 bAcc **[V]** — [B-Free](https://arxiv.org/html/2412.17671v2).
- *Tintype applicability*: **high, cheap.** Add a multi-block readout to the ViT-S: keep the CLS (or mean patch) token after each of the 12 blocks (already computed), apply LayerNorm + a small linear projection per block, learned softmax block weights, sum → classifier. Inference cost ≈ 12 tiny matmuls (384×k), <1% of a forward pass; Rust change is local (retain intermediate CLS vectors). Expected effect: better retention of low-level cues that survive the fit-view resize; possible recall gain on GANs/older generators where Tintype lags CommFor. Risk: with full fine-tuning the last layer may already absorb this information, so gains may be small — must A/B on the blind protocol. Also enables an *early-exit by depth* (score after block 6–8 for obvious images) for speed.

**4. FatFormer — "Forgery-aware Adaptive Transformer for Generalizable Synthetic Image Detection"** — Huan Liu, Zichang Tan, Chuangchuang Tan, Yunchao Wei, Yao Zhao, Jingdong Wang. CVPR 2024. [arXiv 2312.16649](https://arxiv.org/abs/2312.16649)
- Frozen CLIP + forgery-aware adapters in image and frequency (DWT) domains + language-guided alignment; trained on 4-class ProGAN, reports 98% average accuracy on unseen GANs and 95% on unseen diffusion models **[R]** — [arXiv](https://arxiv.org/abs/2312.16649). Independent: GenImage mAcc 88.9; UniversalFakeDetect mAP 98.16 / mAcc 90.86 **[V]** — [Effort HTML tables](https://arxiv.org/html/2411.15633v4).
- *Tintype applicability*: low. Adds a text tower and wavelet branch; benefits are on the ProGAN-only protocol that Tintype does not operate in. A frequency-domain adapter branch could be considered for the GAN weak spot but adds Rust DWT code and compute.

**5. C2P-CLIP — "Injecting Category Common Prompt into CLIP to Enhance Generalization in Deepfake Detection"** — Chuangchuang Tan, Renshuai Tao, Huan Liu, Guanghua Gu, Baoyuan Wu, Yao Zhao, Yunchao Wei. AAAI 2025. [arXiv 2408.09647](https://arxiv.org/abs/2408.09647)
- LoRA-tuned CLIP image encoder whose features are pulled toward category "common prompts" during training only; no extra parameters at test time; abstract reports +12.41% accuracy over original CLIP on UniversalFakeDetect and GenImage **[R]** — [arXiv](https://arxiv.org/abs/2408.09647). On B-Free's modern suite C2P-CLIP averages 60.0 bAcc **[V]** — [B-Free](https://arxiv.org/html/2412.17671v2).
- *Tintype applicability*: low-moderate. The principle "auxiliary training-only loss, zero inference cost" is attractive, but the text-anchor trick needs a CLIP text tower and Tintype's ViT-S is not CLIP-aligned. Not recommended.

**6. "CLIPping the Deception: Adapting Vision-Language Models for Universal Deepfake Detection"** — Sohail Ahmed Khan, Duc-Tien Dang-Nguyen. ICMR 2024. [arXiv 2402.12927](https://arxiv.org/abs/2402.12927)
- Compares linear probing, full fine-tuning, prompt tuning and adapter tuning of CLIP for universal detection; reports prompt tuning as the best transfer strategy and studies few-shot regimes **[R]** — [arXiv](https://arxiv.org/abs/2402.12927).
- *Tintype applicability*: evidence point only (PEFT > FFT under the narrow-training protocol). Not directly actionable.

**7. Effort — "Orthogonal Subspace Decomposition for Generalizable AI-Generated Image Detection"** — Zhiyuan Yan, Jiangming Wang, Peng Jin, Ke-Yue Zhang, Chengchun Liu, Shen Chen, Taiping Yao, Shouhong Ding, Baoyuan Wu, Li Yuan. ICML 2025 (per the user brief; arXiv lists no venue). [arXiv 2411.15633](https://arxiv.org/abs/2411.15633)
- SVD each linear layer of a frozen VFM; freeze the top-r singular components, train only the residual (best: n−r = 1). Trainable params 0.19M on CLIP-L/14, 0.07M on CLIP-B/16 **[V]** — [Effort HTML](https://arxiv.org/html/2411.15633v4).
- Numbers **[V]** (same source): UniversalFakeDetect mAP 99.41 / mAcc 95.19 (UnivFD 90.14/81.38; FatFormer 98.16/90.86); GenImage mAcc 91.1 (DRCT 89.5, FatFormer 88.9, UnivFD 79.5); deepfake cross-dataset avg AUC 0.917. Rank sweep: FFT 86.22, best LoRA 93.53 (r=4), Effort 95.19 mAcc. Adapter comparison CDF-v2/DFD/DFDC AUC: Effort 0.901/0.923/0.798 vs LoRA 0.838/0.834/0.717.
- Smaller CLIP-B/16 performs worse than CLIP-L "by a notable margin" **[V]** — same. DINOv2 not tested.
- *Tintype applicability*: **high, zero inference cost.** Effort can be applied to Tintype's own ViT-S during its next fine-tuning phase: SVD the CommFor-initialised (or current Tintype) weights, freeze the principal subspace, train only the minor components (or a soft version: penalise movement in the top-k singular directions). After training, merge back to dense weights — the Rust engine sees an ordinary ViT-S. Expected effect: less collapse onto the 1,200 training generators → better recall on post-training generators and on GANs/face swaps (where CommFor's prior is strong and Tintype has drifted from it). Cost: training-side only (PyTorch), ~1–2 days to implement. Risk: Effort's gains were measured with narrow training data and a ViT-L backbone with rich priors; with 1,200 generators, full FT may already be near optimal (see Q2), and a 22M model has less "spare" subspace. Measure effective rank as in Effort to decide.

**8. AIDE — "A Sanity Check for AI-generated Image Detection"** — Shilin Yan, Ouxiang Li, Jiayin Cai, Yanbin Hao, Xiaolong Jiang, Yao Hu, Weidi Xie. ICLR 2025. [arXiv 2406.19435](https://arxiv.org/abs/2406.19435)
- Introduces Chameleon (hard, human-misjudged fakes); AIDE fuses DCT-selected high/low-frequency patches through SRM-style filters with frozen CLIP-ConvNeXt semantics; abstract: +3.5% and +4.6% over SOTA on AIGCDetectBenchmark and GenImage **[R]** — [arXiv](https://arxiv.org/abs/2406.19435). AIGCDetectBenchmark 92.77% **[V]** — [DINOv3 paper](https://arxiv.org/html/2511.22471). On Chameleon AIDE gets 64.12 overall (92.98 real / 25.72 fake) **[V per snippet]** — [Mirage, arXiv 2510.03840](https://arxiv.org/pdf/2510.03840). On AIGIBench 77.6% acc **[V]** — [SSAFE](https://arxiv.org/html/2606.08634v1).
- *Tintype applicability*: the "select the most/least textured patches by DCT energy" idea is already embodied in Tintype's textured-crop views. Little new to take; the dual semantic+pixel stream is costly.

**9. CO-SPY — "Combining Semantic and Pixel Features to Detect Synthetic Images by AI"** — Siyuan Cheng, Lingjuan Lyu, Zhenting Wang, Xiangyu Zhang, Vikash Sehwag. CVPR 2025. [arXiv 2503.18286](https://arxiv.org/abs/2503.18286)
- Frozen CLIP semantic branch + VAE-reconstruction artifact branch, fused; introduces Co-Spy-Bench (5 real sources, 22 generators, in-the-wild set); abstract reports large average accuracy gains over baselines (≈11–34.5% range) **[R]** — [arXiv](https://arxiv.org/abs/2503.18286).
- *Tintype applicability*: moderate as a *training-data* idea (VAE-reconstructed reals as hard fakes — same family as Aligned/B-Free/DDA below). The VAE branch at inference would mean porting an SD VAE encoder/decoder to Rust and running it per image — far too slow for 500 img/s. Not recommended at inference.

**10. D³ — "Scaling Up Deepfake Detection by Learning from Discrepancy"** — Yongqi Yang, Zhihao Qian, Ye Zhu, Olga Russakovsky, Yu Wu. CVPR 2025. [arXiv 2404.04584](https://arxiv.org/abs/2404.04584)
- Adds a discrepancy branch (original vs patch-shuffled/distorted view) on a frozen CLIP backbone to avoid overfitting as the number of training generators grows; abstract: 5.3% higher OOD accuracy than the best prior method while keeping ID performance **[R]** — [arXiv](https://arxiv.org/abs/2404.04584).
- *Tintype applicability*: moderate. A training-only consistency loss between a view and its patch-shuffled version is cheap to try in Tintype's trainer; at inference it doubles cost if kept as a two-stream model, so only adopt as a training regulariser.

**11. "Aligned Datasets Improve Detection of Latent Diffusion-Generated Images"** — Anirudh Sundara Rajan, Utkarsh Ojha, Jedidiah Schloesser, Yong Jae Lee. ICLR 2025. [arXiv 2410.11835](https://arxiv.org/abs/2410.11835)
- Fakes = real images passed through an LDM autoencoder; a ResNet-50 trained on real vs reconstructed learns the VAE fingerprint and ignores content; claims generalisation to unseen LDMs and robustness at much lower data cost **[R]** — [arXiv](https://arxiv.org/abs/2410.11835).
- Follow-up **DDA — "Dual Data Alignment Makes AI-Generated Image Detector Easier Generalizable"** — Ruoxin Chen et al., NeurIPS 2025 Spotlight. [arXiv 2505.14359](https://arxiv.org/abs/2505.14359). Shows pixel-aligned reconstructions still leak a *frequency* cue (reconstruction restores high frequencies that JPEG removed from reals), fixes it with pixel+frequency alignment; trained only on aligned MSCOCO, improves on 8 benchmarks, +7.2% on in-the-wild benchmarks **[V]** — [arXiv](https://arxiv.org/abs/2505.14359).
- *Tintype applicability*: **moderate-high, training only.** Add VAE-reconstructed real photos (SD/SDXL/FLUX VAEs) — JPEG-compressed *after* reconstruction to match real-photo frequency statistics (the DDA lesson) — as a class of fakes. Targets new latent-diffusion generators released after training (they share VAEs) and reduces reliance on content/compression shortcuts, which matters for messenger copies. Risk: detects inpainting-like/VAE-roundtripped reals as fake (a precision risk on edited real photos); DDA's frequency finding implies a naive version teaches a JPEG shortcut that would *hurt* messenger recall.

**12. B-Free — "A Bias-Free Training Paradigm for More General AI-generated Image Detection"** — Fabrizio Guillaro, Giada Zingarini, Ben Usman, Avneesh Sud, Davide Cozzolino, Luisa Verdoliva. CVPR 2025. [arXiv 2412.17671](https://arxiv.org/abs/2412.17671)
- DINOv2+registers ViT fine-tuned end-to-end on SD-self-conditioned (inpainted) versions of real images with content augmentation; no resizing, 504×504 crops, multi-crop averaging at test **[V]** — [B-Free HTML](https://arxiv.org/html/2412.17671v2).
- **Key frozen-vs-FT evidence [V]**: linear probe 68.5 bAcc / 80.8 AUC vs end-to-end 95.2 bAcc / 99.0 AUC; e2e DINOv2 without registers 91.1, SigLIP 89.9 bAcc. Average 96.3 bAcc on 27 generators incl. FLUX 92.3, SD3.5 98.9; social networks Facebook 95.6, Reddit 86.2, Twitter 97.3 bAcc; FakeInversion with simulated upload 86.4 bAcc. CLIP-based baselines on the same suite: UnivFD 62.4, RINE 65.0, C2P-CLIP 60.0, CoDE 67.3 — same source.
- *Tintype applicability*: **high as evidence and as data recipe.** (a) Confirms full fine-tuning (Tintype's choice) beats frozen probes once training data targets low-level traces. (b) Its social-network numbers (86–97 bAcc) are far above Tintype's 27–36% recall on messenger copies, suggesting content-aligned inpainting fakes + native-resolution crops at 504 px are worth copying. (c) Larger native crop (504 vs 224) is a speed trade-off: 504² tokens ≈ 5× a 224 view — could replace several 224 crops rather than be added. README notes B-Free has not been run on Tintype's blind batches; doing so is cheap and informative.

**13. Community Forensics — "Community Forensics: Using Thousands of Generators to Train Fake Image Detectors"** — Jeongsoo Park, Andrew Owens. CVPR 2025. [arXiv 2411.04125](https://arxiv.org/abs/2411.04125)
- (Tintype's initialiser.) Performance "steadily increases with the number of models" and "begins to flatten out beyond 1000 models"; data size plateaus near ~27K images at fixed diversity; "freezing the backbone consistently leads to worse results"; ViT-S and ConvNeXt-S perform similarly; 224 → 384 px raises comprehensive mAP ≈0.971 → 0.987 **[V]** — [CommFor HTML](https://arxiv.org/html/2411.04125). Model size beyond ViT-S not studied — same.
- *Tintype applicability*: Tintype is already past the ~1,000-generator knee, so more generators alone gives diminishing returns; the 384-px result says resolution, not width, is CommFor's cheapest lever (Tintype's native crops already exploit this).

**14. SPAI — "Any-Resolution AI-Generated Image Detection by Spectral Learning"** — Dimitrios Karageorgiou, Symeon Papadopoulos, Ioannis Kompatsiaris, Efstratios Gavves. CVPR 2025. (arXiv ID not confirmed in this session — look up before citing) — related pointer via [Navigating paper](https://arxiv.org/html/2507.10236v2), which evaluates SPAI.
- Masked *spectral* self-supervised learning (MAE-like) on real images only, then detects fakes as spectral out-of-distribution; works at arbitrary resolution **[R]**.
- *Tintype applicability*: low-moderate. Real-only pretraining is attractive against unseen generators, but needs a separate spectral model and FFTs per patch; cost is moderate in Rust. Consider only as a robustness ensemble member.

**15. "Rethinking Cross-Generator Image Forgery Detection through DINOv3"** — Zhenglin Huang, Jason Li, Haiquan Wen, Tianxiao Li, Xi Yang, Lu Qi, Bei Peng, Xiaowei Huang, Ming-Hsuan Yang, Guangliang Cheng. arXiv preprint, Nov 2025. [arXiv 2511.22471](https://arxiv.org/abs/2511.22471)
- **[V]** — [HTML](https://arxiv.org/html/2511.22471): frozen DINOv3 averages 73.4% on So-Fake-OOD with no training (DINOv3-L/16: 75.03%); FGTS (top-10 patch tokens) training-free 75.06%, linear probe 87.53% vs CommFor 75.61% and AIDE-PG 75.13%. GenImage 92.6% (LP); AIGCDetectBenchmark 92.45% vs AIDE 92.77%. Supervised fine-tuning of DINOv3-L/16 on 360k images stayed at 74.5–75.1% — no gain over frozen.
- Linear-probe scaling on So-Fake-OOD: DINOv3 S/16 64.6, B/16 70.3, L/16 76.7, H/16 77.8, 7B 87.5; DINOv2 S 56.4, B 57.1, L 61.2, H 61.4 **[V]**.
- Robustness (LP, So-Fake-OOD): clean 87.5 → JPEG q80 75.1, JPEG q70 69.8, noise σ5 73.5, resize 0.75 85.3 **[V]**.
- DINOv3 relies on global low-frequency structure: low-pass keeps much more accuracy than high-pass (16.2-pt gap at r=0.5); masking 50% of patches costs 0.1 pt **[V]**.
- *Tintype applicability*: **moderate, as a second-opinion candidate.** A DINOv3-ViT-B/16 linear probe (≈3.8× ViT-S FLOPs per view) is a plausible "semantic" second opinion that fails differently from Tintype's low-level-trace model, and its reliance on low-frequency structure suggests it degrades *less* under messenger resizing (resize 0.75 cost only 2.2 pts) — but JPEG still costs 12–18 pts. The DINOv3 ViT-S/16 (64.6% frozen) is not good enough frozen. Engineering: DINOv3 uses RoPE positional embeddings and register tokens → new Rust kernels; check the DINOv3 licence (a custom Meta licence, not Apache/MIT **[R]**) against Tintype's MIT/Apache distribution.

**16. SSAFE — "Simple and Strong AI-Generated Image Detection via Frozen Vision Encoders"** — Seunghyun Lee, Byoungkwon Kim, Jaehyun Nam, Kyungmin Lee, Jinwoo Shin. arXiv preprint, Jun 2026. [arXiv 2606.08634](https://arxiv.org/abs/2606.08634)
- Frozen PE-Core-G14-448 + one linear layer, trained on a *curated 10K* set (8 generators chosen by MMD clustering + farthest-point sampling from a 50K pool) **[V]** — [HTML](https://arxiv.org/html/2606.08634v1).
- **[V]** same source: AIGIBench 89.4% acc / 95.7 AP vs Effort 77.1/87.2, AIDE 77.6/82.7, SAFE 78.1/83.6 (baselines trained on 288K); RealWorldBench AUC 99.0, real TNR 98.3, avg TPR 94.4; AIGI-Holmes benchmark 99.9 acc (AIGI-Holmes MLLM 99.2, AIDE 97.0); Chameleon 83.3 acc but real accuracy only 66.9; OpenFake 99.2 TPR with 30K images vs SwinV2 trained on 4M at 98.8. Multimodal encoders (CLIP, SigLIP/2, PE-Core) separate real/fake better than self-supervised DINOv2/v3 — qualitative only. No JPEG/social-media robustness experiments; social sets weak (SocialRF real acc 41.8%, CommunityAI 66.0%).
- *Tintype applicability*: (a) **data curation** by representation-space diversity (pick generators that are far apart in a frozen-encoder embedding) is cheap and directly usable to choose which of Tintype's ~1,200 generators/new generators to weight up; (b) a giant frozen encoder (PE-Core-G ≈ 1.8B+ params at 448 px **[R]**) is impossibly slow for Tintype's engine, but is an excellent *offline teacher/auditor* (see Q4). Note the contradiction with the DINOv3 paper on which encoder family is best.

**17. "Anchor-Regularized Adaptation for Generalizable AI-Generated Image Detection with DINOv3"** — arXiv preprint, Aug 2026. [arXiv 2608.15196](https://arxiv.org/html/2608.15196)
- LoRA r=8 on DINOv3 trained on GenImage + AlignedGenImage; naive LoRA on DINOv3-B scores 62.8 vs linear probe 73.3 (i.e., naive PEFT *hurts*); the anchor regulariser beats naive LoRA at all backbone scales **[V per search snippet]** — [arXiv HTML](https://arxiv.org/html/2608.15196).
- *Tintype applicability*: supports regularising fine-tuning toward the pretrained solution ("anchor") — same family as Effort and L2-SP; zero inference cost.

**18. "Generalizable AI-Generated Image Detection with Embedding-Space Explanations"** — arXiv preprint, Nov 2026. [arXiv 2611.04111](https://arxiv.org/html/2611.04111v1)
- Under an identical readout DINOv3 transfers better than language-supervised (CLIP/SigLIP), reconstruction (MAE) and latent-prediction (I-JEPA-style) encoders; selecting LoRA checkpoints on an *external* validation set beats end-of-epoch and internal-validation selection **[V per search snippet]**.
- *Tintype applicability*: the checkpoint-selection result is free to adopt: select Tintype checkpoints on held-out *generators and transformations* (e.g., a messenger-recompressed set and an NTIRE-like distorted set) rather than on in-distribution validation loss.

**19. DGS-Net — "Distillation-Guided Gradient Surgery for CLIP Fine-Tuning in AI-Generated Image Detection"** — Jiazhen Yan, Ziqiang Li, Fan Wang, Boyu Wang, Ziwen He, Zhangjie Fu. ICML 2026 Spotlight (per arXiv comments). [arXiv 2511.13108](https://arxiv.org/abs/2511.13108)
- Fine-tunes CLIP but projects task gradients away from "harmful" directions and toward directions distilled from a frozen CLIP teacher; evaluated on 50 generative models, +6.6 average margin over SOTA (metric unspecified in abstract) **[V]** — [arXiv](https://arxiv.org/abs/2511.13108).
- *Tintype applicability*: same mechanism family as Effort, applicable to Tintype's training with the frozen CommFor checkpoint as teacher (keeps GAN/face-swap knowledge CommFor has and Tintype lost). Zero inference cost; training cost ≈ one extra forward of the frozen teacher per step.

**20. GAPL — "Scaling Up AI-Generated Image Detection with Generator-Aware Prototypes"** — Ziheng Qin, Yuheng Ji, Renshuai Tao, Yuxuan Tian, Yuyang Liu, Yipu Wang, Xiaolong Zheng. arXiv preprint, Dec 2025 (v2 Apr 2026). [arXiv 2512.12982](https://arxiv.org/abs/2512.12982) · [code](https://github.com/UltraCapture/GAPL)
- Identifies a "Benefit then Conflict" dilemma: performance stagnates then degrades as the number of training generators grows, due to real/fake distribution overlap and a fixed-encoder bottleneck; remedy = learned forgery prototypes + two-stage LoRA **[V]** — [arXiv](https://arxiv.org/abs/2512.12982).
- *Tintype applicability*: relevant warning for a model trained on 1,200 generators. A cheap probe: train with per-family auxiliary heads or prototype losses (training-only) and check whether adding the newest families hurt old-family recall (Tintype's GAN regression vs CommFor is consistent with this conflict). README lists GAPL as unrun on the blind batches.

### Cited Findings (summary across the catalogue)
- Under narrow training (ProGAN / SDv1.4), Effort: FFT 86.22 vs LoRA 93.53 vs SVD-residual 95.19 mAcc on UniversalFakeDetect — [Effort](https://arxiv.org/html/2411.15633v4).
- Under broad/aligned training, e2e fine-tuning beats linear probe 95.2 vs 68.5 bAcc — [B-Free](https://arxiv.org/html/2412.17671v2); CommFor: "freezing the backbone consistently leads to worse results" — [CommFor](https://arxiv.org/html/2411.04125).
- Contradiction: DINOv3-L fine-tuning gave no gain over frozen on So-Fake-OOD (74.5–75.1%) — [DINOv3 paper](https://arxiv.org/html/2511.22471); naive LoRA on DINOv3-B underperforms the linear probe (62.8 vs 73.3) — [ARA](https://arxiv.org/html/2608.15196).
- CLIP-family frozen/PEFT detectors trained on old generators score 60–67 bAcc on modern generators — [B-Free](https://arxiv.org/html/2412.17671v2).
- A frozen giant encoder + linear head on 10K curated images beats Effort/AIDE/SAFE trained on 288K on AIGIBench (89.4 vs 77–78 acc) — [SSAFE](https://arxiv.org/html/2606.08634v1).

### Inferences
- "PEFT beats full fine-tuning" is a statement about *narrow training data plus a big prior*. Tintype has broad data and a small backbone; the evidence (B-Free, CommFor, ScaleDF) says its full fine-tuning is the right default. What transfers is *regularisation toward the prior* (Effort/SVD, DGS-Net, anchor losses), which costs nothing at inference.
- Big frozen encoders win when training data is tiny or generators are very new (SSAFE, DINOv3-7B 87.5% on So-Fake-OOD vs CommFor 75.6%), which matches Tintype's "generators released after training" weak spot — but only at 4–300× Tintype's compute, so the realistic routes are second-opinion-on-uncertain or distillation.

### Gaps
- No paper found that compares a fully fine-tuned ViT-S on ~1,000 generators against a frozen ViT-L/DINOv3 probe *on the same broad training set and the same messenger-compressed test*; the regimes in the literature differ.
- Exact numbers for CLIPping the Deception, CO-SPY, D³, C2P-CLIP, FatFormer, SPAI were not re-fetched (marked [R]); SPAI arXiv ID needs confirmation.
- SSAFE's per-encoder accuracy table (Appendix C) was truncated in fetch; its claim that multimodal encoders beat DINO contradicts 2511.22471 and 2611.04111.

---

## Q2. Does freezing a large backbone + small head generalise better than full fine-tuning a small ViT? What about "asymmetric / low-rank feature-space collapse"?

### Takeaway
Full fine-tuning *does* collapse the feature space (Effort: FFT keeps only 238 of 479 effective dimensions and loses 64.4% of variance; LoRA loses 14.4%; SVD-residual 0.7%), and that collapse correlates with worse unseen-generator accuracy. But the collapse is worst with few training generators; with hundreds-to-thousands of generators, full fine-tuning of a small ViT reaches parity or better (CommFor, ScaleDF, B-Free). Tintype should keep full fine-tuning but add an anti-collapse regulariser.

### Cited Findings
- Asymmetry phenomenon: detectors "tend to favor overfitting to the limited and monotonous fake patterns", making the feature space "highly constrained and low-ranked" — [Effort abstract](https://arxiv.org/abs/2411.15633).
- Effective rank, general domain (baseline 479): SVD 316 (acc 95.19), LoRA 304 (93.03), FFT 238 (86.22); variance retained: SVD 99.3%, LoRA −14.4%, FFT −64.4% — [Effort HTML](https://arxiv.org/html/2411.15633v4).
- ScaleDF: model size ViT-S (21.7M) → ViT-L (303.4M) improves, saturates at ViT-H; ImageNet-21K vs CLIP vs SigLIP 2 pretraining differ by <1% AUC — [Scaling Laws for Deepfake Detection](https://arxiv.org/html/2510.16320).
- CommFor: ViT-S and ConvNeXt-S perform similarly; freezing consistently worse — [CommFor](https://arxiv.org/html/2411.04125).
- B-Free: LP 68.5 vs e2e 95.2 bAcc; authors attribute LP's weakness to last-layer features capturing "high-level semantics" — [B-Free](https://arxiv.org/html/2412.17671v2).
- DINOv3 frozen LP on So-Fake-OOD 87.5% (7B) beats CommFor's 75.6% on that set; but even DINOv3-S/16 frozen is 64.6% — [DINOv3 paper](https://arxiv.org/html/2511.22471). (Tintype's own README reports CommFor recall 0.404 on So-Fake-OOD at its threshold and Tintype 77.0% recall @1% FPR on the JPEG subset — not the same metric as the DINOv3 paper's accuracy.)
- GAPL: performance stagnates then degrades as generator count grows ("Benefit then Conflict") — [GAPL](https://arxiv.org/abs/2512.12982).
- "Naively scaling the pre-training stage… does not always lead to better detection performance": with RINE, DINOv2-L/14 (142M imgs) avg AUC 94.90 beats CLIP H/14 (2B) 89.60 and OpenCLIP L/14 (2B) 83.56 — [Navigating... in the Wild](https://arxiv.org/html/2507.10236v2).

### Inferences
- Tintype's pattern — strong on current commercial generators it trained on, regressed vs CommFor on GANs and face swaps — is what Effort's collapse picture predicts for full fine-tuning drifting away from the CommFor prior. An SVD/anchor/teacher-gradient regulariser toward the CommFor checkpoint is the most direct literature-backed fix, at zero inference cost.
- For the post-training-generator weak spot, the only convincing gains come from much bigger frozen encoders; ViT-S-sized frozen encoders (DINOv3-S 64.6%, DINOv2-S 56.4%) are worse than Tintype's fine-tuned ViT-S would be.

### Gaps
- No study measures effective rank / collapse for a ViT-S trained on >1,000 generators; whether Tintype's features are collapsed is an empirical question (compute singular-value spectrum of penultimate features on real photos vs CommFor init).

---

## Q3. Which intermediate-layer representations carry the forensic signal? Implications for a ViT-S

### Takeaway
Final-layer features of semantic pretrained models are dominated by high-level, low-frequency content; low-level forensic traces live in earlier/middle blocks. Reading out from intermediate blocks gave RINE +9 accuracy points over last-layer-only on frozen CLIP. For a fully fine-tuned ViT-S the gap should be smaller, but a multi-block readout is nearly free and also enables depth-wise early exit.

### Cited Findings
- RINE: last-layer only (≡UnivFD) 80.8 Acc / 94.2 AP vs intermediate blocks 89.8 / 98.7; many blocks get higher importance than the last one — [RINE HTML](https://arxiv.org/html/2402.19091v2). Per-block accuracies are not tabulated in the text (Figure 7 only) — same.
- B-Free attributes linear-probe failure to last-layer features capturing high-level semantics — [B-Free](https://arxiv.org/html/2412.17671v2).
- DINOv3 final layer: low-pass filtering retains far more accuracy than high-pass (16.2-pt gap); patch tokens 74.0% vs CLS 70.5% vs registers 68.3% (LP average); only final layer analysed — [DINOv3 paper](https://arxiv.org/html/2511.22471).
- NTIRE 2026 9th place ("Shallow Real") used DINOv3-L + LoRA r=32 with a multi-token head and *deep supervision* — [NTIRE 2026 report](https://arxiv.org/html/2604.11487).
- Texture-based crops beat centre crops for RINE (91.26 → 94.90 avg AUC) and DMID (78.78 → 89.46), but hurt NPR (70.11 → 67.48) — [Navigating... in the Wild](https://arxiv.org/html/2507.10236v2).

### Inferences
- For Tintype: (1) add a RINE-style learned weighted sum of per-block CLS/mean-patch tokens (blocks 3–12) as the classifier input; (2) add auxiliary deep-supervision losses on blocks 6/8/10 during training; (3) at inference, use an intermediate-block score as a first early-exit stage — an image already confidently classified after block 8 saves ~1/3 of that view's compute. Cost in Rust: retain intermediate token vectors and a few small matmuls. Risk: miscalibrated early exits; must refit exit margins on dev sets as was done for view-level early exit.
- Using mean of patch tokens rather than CLS alone is supported for DINOv3 and is free.
- The texture-crop result independently validates Tintype's "most textured windows" view selection.

### Gaps
- No found paper gives per-block forensic accuracy for a *small* ViT or for a fully fine-tuned detector; the evidence is all from frozen ViT-L-class models.

---

## Q4. Distillation of large foundation-model detectors into small students for speed

### Takeaway
There is little direct literature on distilling a large FM-based AIGI detector into a ViT-S-sized student; closest are DGS-Net (gradient-level distillation from frozen CLIP during fine-tuning), NTIRE 2026 entries using self-distillation, and DistilDIRE (diffusion-feature teacher → small student, 3.2× faster). This is an open, promising gap for Tintype.

### Cited Findings
- DGS-Net uses a frozen CLIP encoder as teacher to steer fine-tuning gradients rather than to train a smaller student; +6.6 average margin over SOTA on 50 generators — [DGS-Net](https://arxiv.org/abs/2511.13108).
- DistilDIRE distils a diffusion-reconstruction (DIRE) detector into a small student with feature-matching loss; reported comparable accuracy at 3.2× speed — [DistilDIRE summary (aggregator)](https://lacuna.tiptreesystems.com/work/distildire-a-small-fast-cheap-and-lightweight-diffusion-synthesized-deepfake/wrk_c1cf277393fbd52217fdde360cfc0ad3) (primary arXiv not fetched).
- NTIRE 2026 6th place (UESTC) used feature-level self-distillation across a CLIP ViT-L/SigLIP ensemble — [NTIRE 2026 report](https://arxiv.org/html/2604.11487).
- The "Navigating" paper ran no teacher/student distillation experiment (only DINOv2's own pretraining distillation) — [Navigating HTML](https://arxiv.org/html/2507.10236v2).
- Large teachers exist that are strong on exactly Tintype's weak cases: DINOv3-7B LP 87.5% on So-Fake-OOD — [DINOv3 paper](https://arxiv.org/html/2511.22471); NTIRE top-2 robust AUC 0.972 with DINOv3 ensembles / two fully fine-tuned DINOv3-7B experts — [NTIRE 2026](https://arxiv.org/html/2604.11487); PE-Core-G linear probe 89.4% AIGIBench — [SSAFE](https://arxiv.org/html/2606.08634v1).

### Inferences
- Recipe for Tintype (zero inference cost): train a teacher = frozen DINOv3-L/7B or PE-Core/SigLIP2-G + linear (or LoRA) head on Tintype's data with the NTIRE-style distortion pipeline; then distil its logits (and optionally intermediate features) into the ViT-S on a large *unlabelled* pool of real-world images — especially messenger-recompressed and distorted copies, where Tintype is weakest — alongside the usual labelled BCE loss. Training cost: one teacher pass per image (can be cached offline once). Risk: teacher's own JPEG sensitivity (DINOv3 LP loses 12–18 pts at JPEG q80–70) can be inherited; mitigate by teacher-scoring the *clean* original and training the student on the recompressed copy (clean-teacher / distorted-student, which is exactly the robustness objective).
- Teacher also serves offline as a label auditor and hard-negative miner.

### Gaps
- No primary paper found that reports a big-FM → ViT-S distillation for AIGI detection with numbers; effect size for Tintype is unknown.

---

## Q5. MLLM / VLM explainable detectors: accuracy vs specialists, practical role

### Takeaway
Zero-shot general MLLMs are well below specialists (GPT-4o 63.9% on LOKI judgment vs humans 76%). Fine-tuned MLLM detectors (FakeVLM, SIDA, So-Fake-R1, AIGI-Holmes, ThinkFake) report specialist-level accuracy on their own benchmarks, but a frozen-encoder linear probe matches them (PE-Linear 99.9 vs AIGI-Holmes 99.2 on AIGI-Holmes's benchmark), and they cost multi-billion-parameter inference. For Tintype their role is offline: explanations for a UI, data labelling/hard-negative mining, and localisation of edited regions — not a runtime stage.

### Cited Findings
- **LOKI** (Ye et al., ICLR 2025, [arXiv 2410.09732](https://arxiv.org/abs/2410.09732)): GPT-4o 63.9% judgment accuracy and 73.7% multiple-choice; humans 76% / 86.4%; GPT-4o 44.4% on hard questions — [LOKI project page](https://opendatalab.github.io/LOKI) / [ICLR proceedings](https://proceedings.iclr.cc/paper_files/paper/2025/hash/afd6374c7f2839cba22f537f15f4f760-Abstract-Conference.html) (hard-question figure via a secondary review, treat with care).
- **"Can ChatGPT Detect DeepFakes? A Study of Using Multimodal Large Language Models for Media Forensics"** — Shan Jia et al., CVPR Workshops 2024, [arXiv 2403.14077](https://arxiv.org/abs/2403.14077): zero-shot MLLMs reach roughly AUC ≈ 0.75 on AI-generated faces, clearly below trained detectors **[R]**.
- **FakeBench** — Yixuan Li et al., [arXiv 2404.13306](https://arxiv.org/abs/2404.13306): human-annotated benchmark for detection + reasoning + interpretation by LMMs; GPT-4V-class models lead among LMMs but are far from specialists **[R]**; no GPT-4o number found this session.
- **FakeVLM — "Spot the Fake: Large Multimodal Model-Based Synthetic Image Detection with Artifact Explanation"** — Siwei Wen et al., NeurIPS 2025, [arXiv 2503.14905](https://arxiv.org/abs/2503.14905): FakeClue dataset; "performs comparably to expert models" without extra classifiers — [NeurIPS poster](https://neurips.cc/virtual/2025/poster/115251).
- **AIGI-Holmes** — Ziyin Zhou et al., ICCV 2025, [arXiv 2507.02664](https://arxiv.org/abs/2507.02664): visual-expert pretraining → SFT → DPO; collaborative decoding of visual expert and MLLM at inference — [arXiv](https://arxiv.org/abs/2507.02664). On its benchmark AIGI-Holmes 99.2 acc vs frozen PE-Core linear probe 99.9 and AIDE 97.0 — [SSAFE](https://arxiv.org/html/2606.08634v1).
- **So-Fake / So-Fake-R1** — Zhenglin Huang et al., [arXiv 2505.18660](https://arxiv.org/abs/2505.18660): RL-trained VLM; +1.3% detection accuracy and +4.5% localisation IoU over second best; So-Fake-Set 2M images from 35 generators; So-Fake-OOD 100K images — [arXiv](https://arxiv.org/abs/2505.18660). (Same group later showed a frozen DINOv3 linear probe at 87.5% on So-Fake-OOD — [2511.22471](https://arxiv.org/html/2511.22471).)
- **SIDA** — Zhenglin Huang et al., CVPR 2025, [arXiv 2412.04292](https://arxiv.org/abs/2412.04292): SID-Set of 300K images (real / fully synthetic / tampered) with masks and explanations; LMM detection + localisation + explanation **[R]**.
- **FakeShield** — Zhipei Xu et al., ICLR 2025, [arXiv 2410.02761](https://arxiv.org/abs/2410.02761): explainable forgery detection and localisation for *edited* images (Photoshop/DeepFake/AIGC-editing), MLLM + mask decoder **[R]**.
- **LEGION** — Hengrui Kang et al., ICCV 2025, [arXiv 2503.15264](https://arxiv.org/abs/2503.15264): artifact localisation + explanation dataset (~12K images with fine-grained artifact annotations) and MLLM framework **[R]**.
- **HEIE** — CVPR 2025, [arXiv 2411.17261](https://arxiv.org/abs/2411.17261): hierarchical explainable implausibility evaluator producing heatmaps and explanations for AIGC images **[R]**.
- **ThinkFake** — [arXiv 2509.19841](https://arxiv.org/html/2509.19841v1): reasoning MLLM, self-reported 84.0% GenImage mean accuracy vs AIDE 81.7%, 75.4% on LOKI — [arXiv](https://arxiv.org/html/2509.19841v1).
- 2026 in-the-wild study on Community Notes images compared SPAI, RINE, B-Free against Gemma 3 27B, Grok 4.1 Fast, GPT-5-mini; TPR declines over time for all while FPR stays stable — [arXiv 2604.15372](https://arxiv.org/pdf/2604.15372) (overall numbers not retrieved).
- Face-manipulation study: GPT-4 87.5%, Gemini 2.5 Pro 84.0%, DeepSeek R1/Sonar ~74%, lagging specialised detectors — [TechRxiv Aug 2025](https://www.techrxiv.org/doi/pdf/10.36227/techrxiv.175459536.67892800).

### Inferences
- Runtime use would cost 3–10B-parameter LLM decoding per image (hundreds of ms–seconds on Apple GPU) vs Tintype's ~2 ms/photo; even as a second stage on the ~uncertain half it destroys the speed budget, and evidence does not show higher accuracy than a frozen-encoder probe.
- Practical offline roles: (1) generate natural-language "why" text for flagged images in a UI (optional, slow path, separate binary); (2) triage of unlabelled scraped data — have an MLLM flag visible artifacts to mine hard positives Tintype misses; (3) for face swaps/inpainting (out of scope for Tintype), localisation models like SIDA/FakeShield/LEGION are the relevant family.

### Gaps
- No apples-to-apples comparison of fine-tuned MLLM detectors vs Tintype-style specialists on messenger-recompressed images was found.

---

## Q6. Scaling: does detector accuracy scale with backbone size / pretraining data / training diversity?

### Takeaway
Training-data *diversity* (number of generators and real domains) follows power laws and matters most; backbone size helps up to roughly ViT-L then saturates when fine-tuned (ScaleDF), but keeps helping for *frozen* probes (DINOv3 S→7B: 64.6 → 87.5%). Pretraining corpus choice matters little once fine-tuned (<1% AUC), but bigger CLIP pretraining does not help frozen detection.

### Cited Findings
- ScaleDF (Wenhao Wang, Jusheng Zhang, Longqi Cai, Taihong Xiao, Yuxiao Wang, Ming-Hsuan Yang; [arXiv 2510.16320](https://arxiv.org/abs/2510.16320); [OpenReview](https://openreview.net/forum?id=HOGZJfArym)): 1−AUC = 0.729·N_real-domains^−0.476 (R² 0.982), ~300 real domains for 0.95 AUC; 1−AUC = 0.618·N_methods^−0.375 (R² 0.990), ~700 methods for 0.95; image count saturates beyond ~10⁷; model size improves to ViT-L (303M) then no gain at ViT-H; ImageNet-21K vs CLIP vs SigLIP 2 pretraining differ <1% AUC; cutting real images to 1/10 only lowers mean AUC 0.886 → 0.879 — [HTML](https://arxiv.org/html/2510.16320). (Note: ScaleDF is face-deepfake oriented: 5.8M reals, 8.8M fakes, 102 methods.)
- CommFor: gains flatten beyond ~1,000 generators; 1,000 random generators beat 10 popular ones at matched image count — [CommFor](https://arxiv.org/html/2411.04125).
- DINOv3 frozen scaling S/16 64.6 → B 70.3 → L 76.7 → H 77.8 → 7B 87.5 (So-Fake-OOD); DINOv2 flat 56.4 → 61.4 — [DINOv3 paper](https://arxiv.org/html/2511.22471).
- NTIRE 2026: Ant International reports "larger models and higher input resolutions consistently improved performance"; top two teams (DINOv3 ensembles; 2× DINOv3-7B, 14B params) reach robust AUC 0.9723/0.9721 vs clean 0.997; a SigLIP2-Giant linear-head entry trained with distortions on every image reached 0.873 — [NTIRE 2026 report](https://arxiv.org/html/2604.11487). Robustness training (distortions at p=1.0; paired clean/distorted with KL+MSE consistency — TeleAI 3rd, LoRA on EVA-CLIP, 0.925) was named a key driver — same.
- Pretraining scale ≠ detection: CLIP H/14 (2B) and OpenCLIP L/14 (2B) underperform DINOv2-L/14 (142M) as RINE backbones — [Navigating](https://arxiv.org/html/2507.10236v2).
- GAPL: stagnation/degradation with more generators for fixed encoders — [GAPL](https://arxiv.org/abs/2512.12982).

### Inferences
- Moving Tintype from ViT-S to ViT-B (~3.8× compute) is the scaling step most likely to pay; beyond ViT-L there is no evidence of gains under fine-tuning. A ViT-B would cut GPU throughput from ~550 to roughly ~150–200 photos/s (estimate from FLOP ratio, early-exit unchanged). The cheaper alternative is to get "big-model" knowledge via distillation (Q4) or a routed second opinion.
- Tintype's NTIRE failure (2.2% recall @1% FPR) is the regime where NTIRE winners show huge backbones + full distortion training reach 0.97 robust AUC; the transferable part for a small model is the *training* recipe (all 22–24 distortion types, clean/distorted consistency loss), not the 7B backbone.
- Diversity of *real* domains is a separate, steeper power law (α 0.476) than generator count (α 0.375) in ScaleDF — suggests adding more real-photo sources (including messenger-recompressed reals) may be as valuable as more generators for Tintype's precision.

### Gaps
- No study fits a joint scaling law in backbone size × generator count for general (non-face) AIGI detection; ScaleDF is face-centric.

---

## Q7. Ranked shortlist for Tintype (from this area)

### Takeaway
Best value: training-side changes that keep the 22M ViT-S at inference (anti-collapse regularisation toward CommFor, large-FM teacher distillation on distorted/messenger data, multi-block readout), plus a routed large-frozen-encoder second opinion if more recall on unseen generators is worth the cost.

### Cited Findings
1. **Regularised full fine-tuning toward the prior (Effort SVD-residual / DGS-Net teacher-gradient / anchor regularisation).** Evidence: FFT 86.22 vs SVD 95.19 mAcc and FFT effective rank 238 vs 316 — [Effort](https://arxiv.org/html/2411.15633v4); DGS-Net +6.6 on 50 generators — [DGS-Net](https://arxiv.org/abs/2511.13108); naive LoRA below LP, anchored adaptation above — [ARA](https://arxiv.org/html/2608.15196). *Tintype*: freeze/penalise top singular subspace of CommFor weights or align gradients with frozen-CommFor teacher during the next phase; merge to dense weights. Expected: recover GAN/face-swap/old-generator recall (CommFor's strengths) and improve unseen-generator recall; precision neutral. Speed: unchanged. Cost: trainer-only, days. Risk: small/no gain given Tintype's broad data (B-Free/CommFor show FFT already fine).
2. **Large-FM teacher → ViT-S distillation on unlabelled distorted/messenger data, with clean-teacher/distorted-student pairing.** Evidence: DINOv3-7B LP 87.5% So-Fake-OOD — [DINOv3](https://arxiv.org/html/2511.22471); NTIRE robust AUC 0.972 with DINOv3 — [NTIRE](https://arxiv.org/html/2604.11487); PE-Core LP 89.4% AIGIBench with 10K images — [SSAFE](https://arxiv.org/html/2606.08634v1); TeleAI paired clean/distorted KL+MSE — [NTIRE](https://arxiv.org/html/2604.11487). *Tintype*: expected recall gains on messenger copies (27–36%) and NTIRE distortions (2.2%), and on post-training generators. Speed unchanged. Cost: one-off teacher training/scoring on a GPU box (PyTorch), student training as now. Risk: teacher JPEG fragility; no published proof of the transfer.
3. **Multi-block readout + deep supervision in the ViT-S (RINE-style), optionally with depth-wise early exit.** Evidence: last-layer vs intermediate 80.8 → 89.8 Acc — [RINE](https://arxiv.org/html/2402.19091v2); deep supervision in NTIRE entry — [NTIRE](https://arxiv.org/html/2604.11487). *Tintype*: small recall gains on low-level-trace sets (GANs, uncompressed); possible speed gain via block-8 exit. Cost: small Rust change. Risk: modest gain once fully fine-tuned.
4. **Aligned / reconstruction-based hard fakes with frequency alignment (Aligned, DDA, B-Free inpainting) and representation-diversity curation (SSAFE).** Evidence: DDA +7.2% in-the-wild — [DDA](https://arxiv.org/abs/2505.14359); B-Free social networks 86–97 bAcc — [B-Free](https://arxiv.org/html/2412.17671v2); curated 10K beats 288K — [SSAFE](https://arxiv.org/html/2606.08634v1). *Tintype*: better generalisation to unseen latent-diffusion generators and social-media copies; training-only. Risk: false positives on VAE-roundtripped/edited real photos; DDA warns about frequency shortcuts.
5. **Routed "third opinion": DINOv3-B/L (or SigLIP2) frozen + linear probe on uncertain images only.** Evidence: DINOv3 scaling and low-frequency reliance (resize 0.75 costs 2.2 pts) — [DINOv3](https://arxiv.org/html/2511.22471). *Tintype*: recall on unseen generators/messenger-resized images; precision depends on fusion. Speed: ~3.8× (B) to ~13× (L) ViT-S cost per routed view. Cost: RoPE + registers in the Rust engine; licence check. Risk: JPEG fragility (−12 to −18 pts at q80–70); adds a large weight file.

### Inferences
- Items 1–4 are mutually compatible and preserve Tintype's 530–563 photos/s; item 5 trades speed for recall in the same way the existing CommFor second opinion does.
- Not recommended: MLLM detectors at runtime; CLIP ViT-L frozen probes (UnivFD/RINE/C2P/FatFormer) as replacements — they score 60–67 bAcc on modern generators per B-Free.

### Gaps
- None of these ideas has been tested on a ViT-S trained on ~1,200 generators; all effect sizes for Tintype are extrapolations and must go through the pre-registered blind protocol.
