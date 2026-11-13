# Detecting fully AI-generated video (T2V/I2V): temporal and spatio-temporal methods, and what they offer Tintype

Scope: fully generated video (Sora, Veo, Kling, Runway, Hunyuan, Wan, Seedance, ...), 2022–2026, with most weight on 2024–2026. Face-swap work is left out except where the technique transfers (TALL, UNITE). The notes were compiled on 2026-11-13. Numbers come from arXiv abstracts and full-text HTML. Where a paper's own text contradicts itself, that is flagged.

Tintype baseline used in every "Tintype applicability" note below (from the repo): 16 evenly spaced frames, a ViT-S/16 with six views per frame, clip score = mean logit, early exit after about 5 frames on average, and a video checkpoint fine-tuned on 30 generators with random H.264 re-encoding. There is no temporal modelling. WaveRep is an optional second opinion for uncertain clips. Blind batch: P 0.997 / R 0.941 on 2,056 clips. Content-matched cohort (Davodi et al., Pixabay authentic clips against Seedance Pro 1, Veo 3.1, Sora 2, Kling 2.6): 72/80 generated and 3/20 authentic flagged, pooled AUROC 0.924. WaveRep on the same cohort: 34/80 and 4/20, AUROC 0.69. — [/home/user/tintype/README.md "How it works"](../../README.md); [/home/user/tintype/docs/EVIDENCE.md "Content-matched generated video clips"](../../docs/EVIDENCE.md)

Naming correction for the coordinator: **DIVID** comes from Liu et al., "Turns Out I'm Not Real" (Columbia, arXiv 2406.09601). The Vahdati et al. CVPRW 2024 paper is a different paper, "Beyond Deepfake Images: Detecting AI-Generated Videos" (Drexel MISL, arXiv 2404.15955). Both are covered below.

---

## Q1. Temporal-artifact methods for generated video: what do they add over per-frame image detectors, especially on unseen generators?

### Takeaway
On large, current, bias-controlled benchmarks, **per-frame image detectors averaged over frames still match or beat dedicated temporal architectures**. AIGVDBench (31 generators) puts the image detectors Effort and ForgeLens above DeMamba and DeCoF, and WaveRep's frame-level DINOv2 beats a Hiera video backbone by 18 bAcc points. Many headline temporal results (D3, ReStraV, NSG-VD) come largely from frame-rate, duration, resolution and motion-level shortcuts, and they collapse to near chance once those are controlled. The one credible positive result for temporal modelling is narrow: a **patch-level temporal readout** (V-PVP, 2026) lifts frozen video backbones to 95.3 AUC on AIGVDBench, against about 90 for per-frame image detectors. Temporal cues look most useful as a *complementary* signal, not as a replacement.

### Cited Findings

**Paper-by-paper (with Tintype applicability)**

1. **AIGVDet**: Bai, Lin, Cao, "AI-Generated Video Detection via Spatio-Temporal Anomaly Learning", PRCV 2024 / arXiv 2403.16638 (2024). [arXiv](https://arxiv.org/abs/2403.16638)
   - Two ResNet-50s, one on RGB frames and one on RAFT optical flow between adjacent frames, combined by decision-level fusion (α=0.5). Clip score = mean of per-frame fused scores. 448×448 crops; up to 95 frames and 94 flow maps per video. Introduces the GVD dataset (11,618 videos, 11 generators). — [arXiv HTML](https://arxiv.org/html/2403.16638v1)
   - Ablation, average of 4 T2V test sets: spatial only 86.5 ACC / 94.9 AUC; flow only 89.0 / 95.0; fused 91.4 / 97.1. Unseen Sora: 82.3 ACC / 92.1 AUC. — [arXiv HTML](https://arxiv.org/html/2403.16638v1)
   - Third-party results are much lower: 73.9 bAcc averaged over 15 generators in the WaveRep paper ([Corvi et al.](https://arxiv.org/html/2506.16802v2)), 50.1% ACC on GenVideo in the Skyra paper ([Skyra](https://arxiv.org/html/2512.15693v2)), and 642 s inference plus 500 s preprocessing per 1,000 clips on an RTX 4090 ([D3 paper](https://arxiv.org/html/2508.00701v2)).
   - *Tintype applicability:* An optical-flow branch would be the most direct "temporal" add-on, but RAFT is expensive (about 1 s/clip in D3's timing, against Tintype's 0.03 s/clip). Its third-party generalisation is poor, and flow is exactly the kind of motion-level signal shown to be a dataset shortcut (Q5). **Not recommended.** If tried, use it only as a cheap flow-magnitude statistic in a fused head, never as a standalone branch.

2. **DeMamba + GenVideo**: Chen et al., "DeMamba: AI-Generated Video Detection on Million-Scale GenVideo Benchmark", arXiv 2405.19707 (2024). [arXiv](https://arxiv.org/abs/2405.19707)
   - GenVideo has about 2.29M training clips (1.21M real from Kinetics-400 and Youku-mPLUG, 1.08M fake) and a 19,588-clip test set (10,000 real MSR-VTT, 8,588 fake). The introduction's counts differ from Section 3.1's. — [arXiv HTML](https://arxiv.org/html/2405.19707v3)
   - DeMamba is a plug-in Mamba module over XCLIP patch features. Many-to-many average: R 0.939 / F1 0.902 / AP 0.971, against the plain XCLIP-B-FT baseline at R 0.844 / F1 0.766 / AP 0.868. Inference cost is almost the same as the baseline (QPS 2.53 vs 2.55). — [arXiv HTML](https://arxiv.org/html/2405.19707v3)
   - Generalises worse elsewhere: AIGVDBench average AUC 80.99 on open-source and 69.43 on closed-source generators ([AIGVDBench](https://arxiv.org/html/2601.11035v1)). WaveRep paper: 84.0 bAcc ([Corvi et al.](https://arxiv.org/html/2506.16802v2)). Skyra paper: 64.3% ACC on ViF-Bench ([Skyra](https://arxiv.org/html/2512.15693v2)).
   - *Tintype applicability:* The useful lesson is that the temporal gain shows up **in-distribution** (+10 AP over its own XCLIP baseline on GenVideo) and largely disappears on newer generators. A Mamba layer would also complicate Tintype's Rust inference engine. **Low value.** GenVideo's real half (MSR-VTT at about 3 fps, per Q5) is itself a shortcut risk.

3. **DIVID ("Turns Out I'm Not Real")**: Liu, Shi, Tsai, Mao, Yang, arXiv 2406.09601 (2024). [arXiv](https://arxiv.org/abs/2406.09601)
   - Diffusion-reconstruction (DIRE-style) features per frame fed to a CNN+LSTM. 93.7% in-domain accuracy and up to +16 points on out-of-domain videos over image detectors. — [arXiv abstract](https://arxiv.org/abs/2406.09601)
   - *Tintype applicability:* Each frame needs a diffusion inversion, which costs orders of magnitude more than a ViT-S pass. Reconstruction features have also generalised poorly in later comparisons: DIRE reached 62.1 AUC on DVF ([MM-Det](https://arxiv.org/html/2410.23623v3)). **Not recommended.**

4. **"Beyond Deepfake Images: Detecting AI-Generated Videos"**: Vahdati, Nguyen, Azizpour, Stamm, CVPR Workshops 2024 / arXiv 2404.15955. [arXiv](https://arxiv.org/abs/2404.15955)
   - Image detectors applied to video: best average AUC 0.698, most at or below 0.65. Retraining them on H.264-compressed images adds only about 0.05 AUC. — [arXiv HTML](https://arxiv.org/html/2404.15955v1)
   - Detectors trained on video (MISLnet, a constrained-conv forensic CNN) reach 0.983 AUC and stay at or above 0.95 across H.264 CRF 0–40 when trained for robustness. — [arXiv HTML](https://arxiv.org/html/2404.15955v1)
   - **Aggregating many patches across time** at CRF 40 lifts AUC from 0.953 (one patch) to 0.984 (80 patches). — [arXiv HTML](https://arxiv.org/html/2404.15955v1)
   - Zero-shot on unseen generators drops from 0.990 to 0.727 (Sora 0.530). **Few-shot fine-tuning on under one minute of video per generator** recovers AUC to 0.98 or above (Sora 0.982). — [arXiv HTML](https://arxiv.org/html/2404.15955v1)
   - *Tintype applicability:* (a) This supports Tintype's design choice: video-trained low-level detectors work, and image-only training does not. (b) The 80-patch result suggests that **early exit at about 5 frames may cost accuracy on hard or heavily compressed clips**. Forcing all 16 frames, or more crops, in the uncertain band is cheap insurance that raises recall at about 3× compute only for borderline clips. (c) Few-shot refresh is a cheap, high-value process for each new generator (shortlist item 5).

5. **DeCoF**: Ma et al., "Detecting AI-Generated Video via Frame Consistency", arXiv 2402.02085 (2024). [arXiv](https://arxiv.org/abs/2402.02085)
   - Learns from inter-frame consistency of CLIP features, deliberately removing spatial artifacts. Claims generalisation to unseen commercial models. — [arXiv abstract](https://arxiv.org/abs/2402.02085)
   - Independent AIGVDBench average AUC: 82.56 on open-source and 72.90 on closed-source generators, below the image detectors Effort (87.49 / 94.05) and ForgeLens (91.82 / 85.03). — [AIGVDBench](https://arxiv.org/html/2601.11035v1)
   - *Tintype applicability:* Frame-difference features on CLIP-like embeddings could be computed from Tintype's own CLS tokens at almost no cost. Standalone they are weaker than Tintype's spatial path, so the only place for them is as an **extra input to a fused head** (Q2).

6. **TALL**: Xu et al., "TALL: Thumbnail Layout for Deepfake Video Detection", ICCV 2023 / arXiv 2307.07494 (face forgery; included for its transferable technique). [arXiv](https://arxiv.org/abs/2307.07494)
   - Tiles consecutive frames into one thumbnail grid so that an image backbone sees short-range temporal context. On DVF (fully generated video) TALL reaches only 69.5 average AUC against 84.3 for the frame-level HiFi-Net. — [MM-Det Table 1](https://arxiv.org/html/2410.23623v3)
   - *Tintype applicability:* Tiling would cut the resolution of each frame and destroy the pixel-level fingerprint that Tintype's six views rely on. **Not recommended.**

7. **MM-Det + DVF**: Song et al., "On Learning Multi-Modal Forgery Representation for Diffusion Generated Video Detection", NeurIPS 2024 / arXiv 2410.23623. [arXiv](https://arxiv.org/abs/2410.23623)
   - LLaVA (CLIP ViT-L/14-336 + 7B LLM) "multi-modal forgery representation" plus a spatio-temporal Hybrid-ViT branch on VQ-VAE reconstruction residuals with **In-and-Across Frame Attention (IAFA)**. 10 successive frames. DVF average AUC 92.0, against frame-level HiFi-Net 84.3 and the video models ViViT 79.1 and TALL 69.5. — [arXiv HTML](https://arxiv.org/html/2410.23623v3)
   - Ablation: base ViT 75.5 → +IAFA 88.5 (+13.0 AUC). MMFR alone 88.7. Full model 92.0. The paper gives no FLOPs, and its limitations section admits a "huge computational complexity". The DVF counts are inconsistent between the main text and the appendix. — [arXiv HTML](https://arxiv.org/html/2410.23623v3)
   - Third-party: 42.4 bAcc as released and 71.6 retrained in the WaveRep paper ([Corvi et al.](https://arxiv.org/html/2506.16802v2)).
   - *Tintype applicability:* The **IAFA ablation is the strongest published evidence that temporal attention across patch tokens helps a ViT**: +13 AUC in-distribution over a per-frame ViT. A light IAFA-style block (one or two attention layers over the patch tokens of short consecutive-frame bursts) could be added on top of the frozen ViT-S (Q2). The LMM branch is incompatible with Tintype's speed target.

8. **WaveRep**: Corvi, Cozzolino, Prashnani, De Mello, Nagano, Verdoliva, "Seeing What Matters: Generalizable AI-generated Video Detection with Forensic-Oriented Augmentation", NeurIPS 2025 / arXiv 2506.16802. [arXiv](https://arxiv.org/abs/2506.16802) · [GitHub](https://github.com/grip-unina/WaveRep-SyntheticVideoDetection)
   - Frame-level DINOv2 (with registers), logits averaged over 64 frames, trained on a single generator (Pyramid Flow plus VAE-reconstructed Panda70M fakes). Training-time augmentation: a 3-level Haar decomposition replaces the fake frame's baseband, horizontal and vertical bands with the real frame's, keeping only the diagonal mid/high bands, which survive H.264. — [arXiv HTML](https://arxiv.org/html/2506.16802v2)
   - Average bAcc 94.3, against DeMamba 84.0, RINE 74.1, AIGVDet 73.9 and MM-Det 42.4. **A Hiera video backbone on 16-frame clips scored 73.2 bAcc against 91.8 for the frame-level DINOv2 in the same ablation.** The augmentation lifts bAcc from 88.2 to 91.8 (AUC 97.6 → 98.7). — [arXiv HTML](https://arxiv.org/html/2506.16802v2)
   - Real training clips were filtered to exclude screen recordings, static content and minimal camera motion. That filtering matters for Tintype's real-footage false positives. — [arXiv HTML](https://arxiv.org/html/2506.16802v2)
   - *Tintype applicability:* (a) **Adopt the wavelet band-replacement augmentation in Tintype's next video fine-tuning phase.** It is training-only, costs nothing at inference, and the 2026 bias audit found WaveRep the only detector unaffected by motion rebalancing and blur (Q5). Expected effect: better recall on unseen generators and robustness to re-encoding; the precision effect is unknown. (b) WaveRep's weak result on the content-matched cohort (AUROC 0.69) suggests that its *training data* (one generator), not its augmentation, is the limitation, so combining the augmentation with Tintype's 30-generator data is the logical step.

9. **ReStraV**: Internò, Geirhos, Olhofer, Liu, Hammer, Klindt, "AI-Generated Video Detection via Perceptual Straightening", NeurIPS 2025 / arXiv 2507.00583. [arXiv](https://arxiv.org/abs/2507.00583)
   - DINOv2 ViT-S/14 block-11 tokens on 24 frames over 2 s. 21 statistics of stepwise distance and curvature of the embedding trajectory feed a 2-layer MLP. About 48 ms per video on an RTX 2080. — [arXiv HTML](https://arxiv.org/html/2507.00583v4)
   - Reported: VidProM 97.17% ACC / 98.63 AUROC; GenVidBench 93.0% avg ACC; Veo 3 zero-shot 83.2% ACC / 86.9 AUROC on 200+200 videos. The method is time-reversal invariant, and scene cuts inflate curvature. — [arXiv HTML](https://arxiv.org/html/2507.00583v4)
   - **Shortcut audit:** ReStraV duplicates frames for clips under 12 fps and repeats the last frame for clips under 2 s. 18,715 of 24,997 generated test videos were affected, against 2 of 24,999 real ones. The single "minimum distance" feature alone reaches 0.932 AUC. On GenBuster++ it falls to 0.515, and blurring one frame drops it from 0.713 to 0.474. — [Michels et al. 2026](https://arxiv.org/html/2607.00948v2)
   - Under one shared re-encode its GenVidBench AUC falls from 0.85 to 0.60. Fitted on AIGVDBench, it flags 90% of MSR-VTT real clips at a 5% target. — [Cakiroglu et al. 2026](https://arxiv.org/html/2606.31004v2)
   - *Tintype applicability:* ReStraV's backbone is almost Tintype's (ViT-S, DINO-style), so its trajectory statistics could be computed from Tintype's own tokens for nearly nothing. But it needs **24 dense consecutive frames**, which Tintype's 16-frame sparse sampling with early exit does not provide, and the 2026 audits show its signal is fragile. **Use only as one feature group in a fused head, trained on fps- and duration-normalised, motion-balanced data.**

10. **D3**: Zheng et al., "D3: Training-Free AI-Generated Video Detection Using Second-Order Features", ICCV 2025 / arXiv 2508.00701. [arXiv](https://arxiv.org/abs/2508.00701)
    - Score = standard deviation of second-order central differences of XCLIP-B/16 frame-embedding distances. 8 fps over 2 s. 56 s per 1,000 clips on an RTX 4090. Reported mAP: GenVideo 98.46, EvalCrafter 98.87, VideoPhy 99.16, VidProM 88.46 (slightly below the FID baseline). — [arXiv HTML](https://arxiv.org/html/2508.00701v2)
    - **Shortcut audit:** GenVideo's real clips are 3 fps, so D3's 8 fps sampling creates duplicate frames and spurious second-order peaks. AUC drops from 0.973 to 0.799 on unbiased real videos and to 0.484 on GenBuster++. — [Michels et al. 2026](https://arxiv.org/html/2607.00948v2)
    - The STALL authors measured AUC 0.72 on GenVideo, 0.54 on VideoFeedback and 0.73 on ComGenVid. — [STALL](https://arxiv.org/html/2603.15026v2)
    - In MVAD (audio-video), D3 has 97.6% recall but only 67.8% F1 on fake-real samples: it flags too much. — [MVAD](https://arxiv.org/html/2512.00336v4)
    - *Tintype applicability:* **Not as a detector.** As a feature it is one scalar per clip, but it encodes "real videos are jerkier", which turns into false positives on stabilised, smooth real footage (drone, gimbal, stock). That is exactly Tintype's problem class.

11. **NSG-VD (physics-driven)**: Zhang et al., "Physics-Driven Spatiotemporal Modeling for AI-Generated Video Detection", NeurIPS 2025 spotlight / arXiv 2510.08073. [arXiv](https://arxiv.org/abs/2510.08073) · [GitHub](https://github.com/ZSHsh98/NSG-VD)
    - A normalised spatiotemporal gradient (ratio of spatial score gradients to temporal density change) from a pretrained diffusion model, then MMD against a set of real videos. Claims +16.0 recall and +10.75 F1 over the state of the art. — [arXiv abstract](https://arxiv.org/abs/2510.08073)
    - Independent: 49.7% ACC on ViF-Bench and 49.1% on GenVideo ([Skyra](https://arxiv.org/html/2512.15693v2)). The bias audit found resolution and JPEG-order confounds: 0.964 → 0.685 AUC when JPEG is applied after downscaling, and temporal gaps not normalised by Δt in the official code ([Michels et al.](https://arxiv.org/html/2607.00948v2)).
    - *Tintype applicability:* It needs a diffusion model forward pass per clip. **Not recommended.**

12. **L3DE ("What Matters in Detecting AI-Generated Videos like Sora?", later retitled)**: Chang et al., "How Far are AI-generated Videos from Simulating the 3D Visual World: A Learned 3D Evaluation Approach", ICCV 2025 / arXiv 2406.19568. [arXiv](https://arxiv.org/abs/2406.19568)
    - A 3D CNN on monocular cues (motion, depth, appearance) separates real from synthetic and also works as a detector. Reports persistent 3D-consistency gaps in Kling, Sora and MiniMax. — [arXiv abstract](https://arxiv.org/abs/2406.19568)
    - *Tintype applicability:* Depth and flow estimators per clip are too expensive for the main path. The idea (geometric consistency) is a possible "explain" feature, not a detector upgrade.

13. **UNITE**: Kundu et al., "Towards a Universal Synthetic Video Detector", CVPR 2025 / arXiv 2412.12278. [arXiv](https://arxiv.org/abs/2412.12278)
    - Frozen SigLIP-So400M per-frame tokens (384 px, 729 tokens per frame) with a **4-block temporal transformer over 64 frames** (every other frame) and an attention-diversity loss. Trained on FF++ plus **GTA-V game footage (SAIL-VOS-3D) used as task-irrelevant data**. — [arXiv HTML](https://arxiv.org/html/2412.12278v2)
    - DeMamba validation split, untrained on it: P 92.76 / R 89.60. Accuracy rises with frames {1, 8, 16, 32, 64} (figure only, no numbers). Depth 4 is best; deeper overfits. The paper has no ablation of the transformer against mean pooling. — [arXiv HTML](https://arxiv.org/html/2412.12278v2)
    - *Tintype applicability:* This is the template for a "temporal transformer over frozen frame tokens" head. UNITE uses GTA-V as a synthetic proxy in training. Tintype’s use of authentic gameplay as a negative is a separate training choice. The paper does not isolate how much the temporal transformer itself contributes.

14. **V-PVP ("Rethinking the Readout")**: Cui et al., "Rethinking the Readout: Unlocking Video Backbones for AI-Generated Video Detection", arXiv 2607.15321 (2026). [arXiv](https://arxiv.org/abs/2607.15321) · [GitHub](https://github.com/manic-cui/Rethinking-the-readout)
    - Argues that global readouts (one descriptor per frame) wash out the temporal cue, and replaces the readout with two streams over the **patch velocity field** v_t = z_{t+1} − z_t (same-position patch tokens, last block, 16 frames at 224 px, 15 adjacent pairs). 492,674 trainable parameters. — [arXiv HTML](https://arxiv.org/html/2607.15321v2)
    - AIGVDBench average AUC: **V-PVP on a frozen VideoMAE 95.28**, against per-frame image detectors NPR 89.90, UnivFD 90.03, Effort 90.29 and DDA 90.73, and fully fine-tuned video models I3D 92.85, SlowFast 92.70 and VideoMAE 88.34. No ablation on frame count or stride. — [arXiv HTML](https://arxiv.org/html/2607.15321v2)
    - *Tintype applicability:* **The most transferable 2026 idea.** Tintype's ViT-S/16 at 224 px already produces 196 patch tokens per frame. A V-PVP-style head (about 0.5M parameters) over patch-token differences of short consecutive-frame bursts could be trained with the backbone frozen. Expected effect: higher recall on generators whose individual frames look clean. Cost: each sampled point needs at least two adjacent frames, so about 2× ViT passes for the frames used (a rough estimate of 0.03 → 0.05–0.06 s/clip), plus a small head. Risk: the motion shortcut (Q5), so train it motion-balanced, fps-normalised and with blur and duplicate-frame augmentation. The paper's evidence is on a *video-pretrained* backbone (VideoMAE). Whether it transfers to an image-pretrained ViT-S is untested.

15. **TRACE**: Cao et al., "TRACE: Trajectory Representation and Consistency Estimation for AI-Generated Video Detection", arXiv 2609.25775 (2026). [arXiv](https://arxiv.org/abs/2609.25775)
    - Uses a pretrained flow-matching video DiT as a "velocity-field probe" at several flow times, with cross-frame velocity differences and a real-centred objective. Claims to "substantially" outperform prior SOTA on unseen generators in AIGVDBench (no numbers in the abstract). — [arXiv abstract](https://arxiv.org/abs/2609.25775)
    - *Tintype applicability:* A video DiT forward pass per clip costs billions of parameters. **Not compatible with Tintype's speed**, but the "real-centred" training objective (pull reals together, push all generators away) is an idea that could be ported to the ViT-S head.

16. **STALL + ComGenVid**: Ben Hayun, Betser, Levi, Kassel, Gilboa, "Training-free Detection of Generated Videos via Spatial-Temporal Likelihoods", CVPR 2026 / arXiv 2603.15026. [arXiv](https://arxiv.org/abs/2603.15026) · [project](https://omerbenhayun.github.io/stall-video)
    - Training-free. DINOv3 per-frame embeddings, PCA-whitened on **about 33k real VATEX videos only**. Spatial score = maximum over frames of a Gaussian log-likelihood. Temporal score = minimum over transitions of the likelihood of L2-normalised frame-difference vectors. Each is mapped to a real-percentile and the two are averaged. 16 frames at 8 fps; about 0.49 s per video. — [arXiv HTML](https://arxiv.org/html/2603.15026v2)
    - AUC: VideoFeedback 0.83, GenVideo 0.80, ComGenVid 0.85 (Sora 0.84, Veo 3 0.86), against D3 at 0.54 / 0.72 / 0.73. Performance only drops below about 5k calibration videos. **MobileNet-v3 embeddings (0.82) work as well as DINOv3 (0.81)**, while video encoders (ViCLIP 0.59, VideoMAE 0.61) do poorly. — [arXiv HTML](https://arxiv.org/html/2603.15026v2)
    - *Tintype applicability:* Absolute AUC (about 0.8) is far below Tintype's. Its value is that the score is **calibrated only against real footage**. Fitted on Tintype's CLS embeddings with a large real corpus that includes animation, gameplay, stock and stabilised phone footage, it becomes a "how typical of real video is this?" signal that a fused head or the uncertain band can use to veto false positives. Cost is a PCA projection plus a Gaussian log-likelihood per frame: negligible. Risk: it needs consecutive frames at a fixed fps for the temporal part. The spatial part alone works on Tintype's existing sparse frames.

17. **Skyra**: Li et al., "Skyra: AI-Generated Video Detection via Grounded Artifact Reasoning", arXiv 2512.15693 (2025; camera-ready). [arXiv](https://arxiv.org/abs/2512.15693) · [GitHub](https://github.com/JoeLeelyf/Skyra)
    - Qwen2.5-VL-7B, SFT on ViF-CoT-4K (human-annotated artifacts) then RL. 16 frames at 256p. ViF-Bench: 91.0% ACC / 90.3 F1, against DeMamba 64.3, AIGVDet 69.1, BusterX++ 56.9 and Gemini-2.5-flash 53.4. On out-of-domain GenVideo only 71.8% ACC (91.0 after fine-tuning on about 2.2k GenVideo samples). Symmetric RL rewards biased it towards "fake", so the authors used an asymmetric reward. Gemini mistakes compression and motion blur for forgery. — [arXiv HTML](https://arxiv.org/html/2512.15693v2)
    - *Tintype applicability:* 7B parameters per clip; useful only as an optional "explain why" mode. The asymmetric-reward finding shows that MLLM detectors are **biased towards false positives** on real footage with natural degradations. That makes them a bad choice for fixing Tintype's false-positive problem.

18. **BusterX / BusterX++**: Wen et al., arXiv 2505.12620 (2025) and arXiv 2507.14632 (2025). [BusterX](https://arxiv.org/abs/2505.12620) · [BusterX++](https://arxiv.org/abs/2507.14632)
    - GenBuster-200K (more than 200k videos from current generators) and GenBuster-Bench with in-domain, out-of-domain and in-the-wild tracks. An MLLM with RL in which the reasoning chain is the detector. BusterX++ unifies images and video and finds that single-stage pure RL matches or beats SFT→RL. — [BusterX abstract](https://arxiv.org/abs/2505.12620); [BusterX++ abstract](https://arxiv.org/abs/2507.14632)
    - Independent: BusterX++ 56.9% ACC / 21.9 F1 on ViF-Bench. — [Skyra](https://arxiv.org/html/2512.15693v2)
    - *Tintype applicability:* Too large for the main path. **GenBuster++ is useful as an evaluation set**: it is the one benchmark the 2026 bias audit found already motion-balanced, and every motion-based detector fell to about 0.5 AUC on it while WaveRep held 0.919 ([Michels et al.](https://arxiv.org/html/2607.00948v2)).

19. **VidGuard-R1, VideoVeritas, REVEAL (MLLM detectors)**
    - VidGuard-R1 (Park et al., ICLR 2026, arXiv 2510.02282): a GRPO-trained MLLM with temporal-stability and diffusion-complexity rewards, on a 140k real/fake pair dataset; claims SOTA zero-shot. — [arXiv](https://arxiv.org/abs/2510.02282)
    - VideoVeritas (Tan et al., arXiv 2602.08828, 2026): perception-pretext RL (spatiotemporal grounding, object counting) and the MintVid set (3k videos from 9 generators plus real clips with factual errors). — [arXiv](https://arxiv.org/abs/2602.08828) On a balanced GenVidBench subsample it reaches zero-shot AUC 0.83 ([Cakiroglu et al., codec paper](https://arxiv.org/html/2607.19476v1)).
    - REVEAL (Tsai et al., ICCV 2025 workshop, arXiv 2502.14994): a tool-using VLM with +9.1 to +30.2 F1 over VLM baselines on its VidForensic set. — [arXiv](https://arxiv.org/abs/2502.14994)
    - GenVideoLens (arXiv 2603.18625, 2026): temporal perturbation experiments show that **current LVLMs make limited use of temporal information**. — [arXiv](https://arxiv.org/abs/2603.18625)
    - *Tintype applicability:* None for the core path. A VLM costs about 10^13 MACs and about 16 s per clip ([codec paper](https://arxiv.org/html/2607.19476v1)), roughly 500× Tintype's runtime.

20. **Local-and-global temporal defects**: He et al., "Exposing AI-generated Videos: A Benchmark Dataset and a Local-and-Global Temporal Defect Based Detection Method", arXiv 2405.04133 (2024). [arXiv](https://arxiv.org/abs/2405.04133)
    - Learns local motion together with global appearance variation, and includes network-degraded samples. — [arXiv abstract](https://arxiv.org/abs/2405.04133)
    - *Tintype applicability:* Same family as AIGVDet and DeCoF, and superseded by the 2026 evidence.

21. **SpecSem-Net**: Wei, Zhang, Wan, arXiv 2605.17311 (2026). [arXiv](https://arxiv.org/abs/2605.17311)
    - Fourier high-pass spectral features gated by semantic context to reduce "spectral noise" misjudgments. 87.25% ACC on a 5-commercial-generator benchmark and 95.59% on public sets. — [arXiv abstract](https://arxiv.org/abs/2605.17311)
    - *Tintype applicability:* Supports the frequency-cue direction (as WaveRep does). Gating spectral evidence with semantics is one way to stop clean, low-noise real footage (stock, CGI) from looking "too smooth" in the spectrum.

22. **FakeParts**: Liu et al., "FakeParts: a New Family of AI-Generated DeepFakes", arXiv 2508.21052 (2025). [arXiv](https://arxiv.org/abs/2508.21052)
    - FakePartsBench: 81K videos (44K partial fakes) with pixel- and frame-level masks. Partial manipulations cut human accuracy by up to 26% and also degrade detectors. — [arXiv abstract](https://arxiv.org/abs/2508.21052)
    - *Tintype applicability:* Mean-logit pooling dilutes evidence from a short generated segment. If partial or spliced clips are in scope, add a **max or top-k frame pooling** alongside the mean, and do not early-exit on "real" until both halves of the clip have been sampled. Tintype's early exit already stops at the half-way point for most clips, and that is the risk.

### Inferences
- Across independent re-evaluations (AIGVDBench, WaveRep, Skyra, the two 2026 audits), **no temporal-only method beats a strong frame-level detector trained on many generators**. Tintype's frame-based design is consistent with the field's best evidence.
- Where temporal modelling *does* add something, it comes from attention or difference features over **patch tokens** (MM-Det IAFA +13 AUC in-distribution; V-PVP 95.3 vs about 90 on AIGVDBench), not from global per-frame descriptors or hand-crafted trajectory statistics. For Tintype the realistic gain is a few points of recall on the hardest unseen generators, not a step change.
- Tintype's 72/80 recall on the content-matched cohort is limited by individual clips, not by architecture class. The 3/20 false positives are the more pressing problem, and temporal methods as a family have *worse* real-footage false-positive behaviour (Q5).

### Gaps
- No paper reports a clean head-to-head of "the same per-frame ViT, mean pooling vs. a temporal head on its tokens" on unseen *2025–26 commercial* generators (Veo 3, Sora 2, Kling 2.x).
- Exact numbers for DeCoF's own paper, NSG-VD's absolute scores, BusterX's benchmark numbers and TRACE's AIGVDBench numbers were not retrieved; only abstracts or third-party numbers are given above.
- DuB3D (dual-branch 3D) was named in Tintype's existing survey but was not located in this pass.

---

## Q2. Lightweight temporal aggregation on top of per-frame ViT features: evidence and cost

### Takeaway
The cheap options, ranked by evidence: (1) a **patch-token velocity readout** (V-PVP, about 0.5M parameters, +5 AUC over per-frame detectors on AIGVDBench); (2) a **small temporal transformer or attention block over frozen frame or patch tokens** (UNITE 4 blocks; MM-Det IAFA +13 AUC in-distribution); (3) **real-calibrated likelihood statistics of frame embeddings and their differences** (STALL, training-free, about 0.8 AUC alone); (4) **trajectory geometry** (ReStraV, D3), which is cheap but fragile. Global mean or majority-vote pooling of frame logits remains a strong baseline, and spatial patch aggregation across many frames is itself worth up to 66% relative error reduction under compression.

### Cited Findings
- V-PVP replaces only the aggregation layer with two streams over patch-velocity fields: 492,674 trainable parameters, frozen backbone, 16 frames at 224 px, 95.28 AUC on AIGVDBench against 89.90–90.73 for per-frame image detectors. — [Cui et al. 2026](https://arxiv.org/html/2607.15321v2)
- Video backbones with standard global readouts often fail to beat image-pretrained probes, which the authors attribute to excessive spatiotemporal aggregation. — [Cui et al. 2026](https://arxiv.org/abs/2607.15321)
- IAFA (attention across all patch tokens of the clip, then frame-token-to-own-patches attention) gives +13.0 AUC over a base ViT on DVF. — [MM-Det](https://arxiv.org/html/2410.23623v3)
- UNITE: a 4-block transformer over frozen SigLIP tokens of 64 frames. Deeper transformers overfit cross-domain, and more frames help. — [UNITE](https://arxiv.org/html/2412.12278v2)
- STALL: whitened Gaussian likelihoods of per-frame embeddings and of unit-normalised frame differences, fitted on real data only. MobileNet-v3 features work as well as DINOv3. About 0.49 s per video. — [STALL](https://arxiv.org/html/2603.15026v2)
- ReStraV: 21 curvature and distance statistics with an MLP, about 48 ms per video including DINOv2 ViT-S. — [ReStraV](https://arxiv.org/html/2507.00583v4)
- D3: one scalar (std of second-order differences), 56 s per 1,000 clips. — [D3](https://arxiv.org/html/2508.00701v2)
- In a streaming setting, mean pooling of per-GOP scores reached offline AUC 0.885 against 0.64 for a running max. A running max is needed only for anytime guarantees. — [Cakiroglu et al., codec paper](https://arxiv.org/html/2607.19476v1)
- Patch aggregation: 1 patch → 80 temporally distributed patches lifts AUC from 0.953 to 0.984 at H.264 CRF 40. — [Vahdati et al.](https://arxiv.org/html/2404.15955v1)
- WaveRep averages logits over 64 frames, and its frame-level design beat a Hiera video backbone (91.8 vs 73.2 bAcc). — [Corvi et al.](https://arxiv.org/html/2506.16802v2)
- AIGVDBench uses only 8 frames (the first 8 of 32 uniformly sampled from the first 128) for training and inference in its 33-detector evaluation. — [AIGVDBench](https://arxiv.org/html/2601.11035v1)

### Inferences
- **Concrete Tintype design (estimate, not measured):** keep the per-frame logit path. For each of K sample points (for example K=4–8), decode a burst of 2–3 consecutive frames, run the frozen ViT-S once per frame (the views can be cut to the full-frame view for the burst partners), and compute patch-token differences. Feed them to a V-PVP- or IAFA-style head of about 0.5M parameters that outputs a temporal logit. Add a STALL-style real-likelihood score from CLS tokens. Fuse the mean frame logit, the temporal logit and the real-likelihood score with logistic regression calibrated on held-out real collections.
- Cost estimate: about 1.5–2× the current ViT passes when the temporal head runs on every clip, or about +10–20% if it runs only on the uncertain band, mirroring the existing WaveRep cascade. The head itself is negligible next to the ViT.
- Training: freeze the ViT-S and train only the head on Tintype's 30-generator set plus real clips, with motion-balanced sampling and fps and duration normalisation (Q5). This keeps the risk to the existing frame path at zero.
- An "extra input" of cheap scalar statistics (ReStraV curvature, D3 std, flow magnitude) is unlikely to help much and likely to add false positives on smooth real footage.

### Gaps
- No published ablation of a V-PVP-style head on an **image-pretrained** ViT-S (the paper uses VideoMAE). Whether the gain transfers needs an internal experiment.
- No paper reports latency of such heads on consumer GPUs or Apple Metal.

---

## Q3. Frame sampling: dense vs sparse, how many frames, I/P-frame differences, codec-level features

### Takeaway
Frame-level detectors use sparse or long windows (WaveRep 64 consecutive frames; AIGVDBench 8 frames; Tintype 16 sparse with early exit at about 5). Temporal methods need **dense consecutive frames at a fixed rate** (ReStraV 24 frames over 2 s; D3 and STALL 8 fps over 2 s; V-PVP 16 adjacent frames), and they become unreliable when real and fake clips differ in native fps or duration. Codec-level features (motion vectors, frame types) are almost free to read, but alone they reach only AUC 0.62–0.72 on generator detection. Container metadata (frame-type counts) is a **dataset leak** reaching AUC 0.999, not a real signal.

### Cited Findings
- More frames help a temporal transformer (UNITE, tested at 1/8/16/32/64; figure only). — [UNITE](https://arxiv.org/html/2412.12278v2)
- Patch count across time improves robustness to compression (0.953 → 0.984 AUC at CRF 40 with 80 patches). — [Vahdati et al.](https://arxiv.org/html/2404.15955v1)
- ReStraV settled on 2 s and 24 frames after ablation. Clips under 12 fps or under 2 s get duplicated frames, which creates the shortcut. — [ReStraV](https://arxiv.org/html/2507.00583v4); [Michels et al.](https://arxiv.org/html/2607.00948v2)
- D3 samples at 8 fps. GenVideo's real clips are 3 fps, so upsampling introduces artificial second-order peaks. — [Michels et al.](https://arxiv.org/html/2607.00948v2)
- After an 8 fps resample, ReStraV flags 72% of AIGVDBench's own real clips, against 5% after a plain re-encode. — [Cakiroglu et al.](https://arxiv.org/html/2606.31004v2)
- **Codec features:** H.264 motion-vector fields summarised per GOP (13-d: 4 global motion and 9 spectral statistics) give leave-one-generator-out AUC 0.64 on matched GenVidBench and 0.62 on AIGVDBench, at about 10^5 MACs per GOP on CPU (about 33 ms per streaming step). Informative after one GOP (AUC 0.80 at the first prefix). A cascade deferring 15% of clips to a CLIP ViT-B/32 lifts accuracy from 0.75 to 0.78 at 7× less compute. Per-generator codec AUC spans 0.45–0.90, and stage 1 may exploit "generated video moves less". — [Cakiroglu et al., "Detect Early, Escalate Rarely"](https://arxiv.org/html/2607.19476v1)
- **Frame-type leak:** declared frame-type counts alone separate real from generated at AUC 0.999 (GenVidBench), 0.997 (AIGVDBench), 0.839 (GenVideo+K400) and 0.893 (DVF). A standard re-encode removes it only on GenVidBench. A fixed pattern (GOP 12, 2 B-frames, no scene-cut I-frames, 3-frame tail buffer) removes it everywhere, but only for clips of 2 s or longer. — [Cakiroglu et al., "Where the Real Clips Came From"](https://arxiv.org/html/2606.31004v2)
- Motion vectors alone separate *real collections from each other* at AUC 0.51–0.98. — [Cakiroglu et al.](https://arxiv.org/abs/2606.31004)
- AIGVDBench re-encodes all videos to H.264 to remove format bias between real and synthetic. — [AIGVDBench](https://arxiv.org/html/2601.11035v1)

### Inferences
- **For Tintype's current frame path**, sparse uniform sampling is well supported. The early-exit design is the one to watch: the 80-patch evidence suggests that forcing more frames (all 16, or 32) on borderline and heavily compressed clips is a cheap recall gain.
- **For any temporal head**, Tintype would need bursts of consecutive frames resampled to a canonical fps, with explicit handling of duplicate frames (drop them, as STALL does) and of scene cuts. Training data must be fps- and duration-matched.
- **I-frame vs P/B-frame:** Tintype's evenly spaced frames land on mixed frame types, and I-frames are typically decoded at higher quality. No paper found measures detector score variance by frame type. A cheap internal check: score I-frames only versus all frames on the matched cohort. Decoding only I-frames could also speed up early exit (no reference-chain decode). This is an untested hypothesis.
- **Codec features are not worth adding for accuracy** (AUC about 0.64) and not needed for speed (Tintype already runs at 0.03 s per clip). Their value is as an **audit tool**: run the VidAudit frame-type and motion-vector probes on Tintype's training set to confirm that the random H.264 re-encoding removed container and GOP shortcuts.

### Gaps
- No found study directly compares the frame-type (I vs P/B) a frame comes from against a frame detector's score for generated video.
- No ablation of "sparse 16 vs dense 16" for a frame-level detector was found.

---

## Q4. Benchmarks: which are current, and what are SOTA numbers?

### Takeaway
Current and most useful in late 2026: **AIGVDBench** (31 generators, 440k videos, 33 detectors), **GenVidBench** (6.78M videos, 11 generators, AAAI 2026, but with a container leak), **ComGenVid** (Sora, Veo 3; CVPR 2026), **GenBuster-Bench++** (motion-balanced), **ViF-Bench** and **MintVid** (small, current), and **MVAD** for audio-video. GenVideo, DVF, GVD and VidProM are older and carry documented fps, resolution and real-source shortcuts. Reported SOTA AUCs in the mid-to-high 90s often do not replicate across benchmarks. Only WaveRep's features kept their score from GenVidBench to AIGVDBench.

### Cited Findings
- **AIGVDBench** (Ma et al., arXiv 2601.11035, 2026; called CVPR 2026 in Tintype's survey): 31 generators, more than 440k videos, more than 1,500 evaluations of 33 detectors. Real clips from OpenVid-HD. Best average AUC: Effort 94.05 on closed-source generators, ForgeLens 91.82 on open-source. MLLMs are near chance on open-source (DeepSeek-VL2 51.5% ACC). Better generation quality does not imply lower detectability. — [arXiv HTML](https://arxiv.org/html/2601.11035v1) · [GitHub](https://github.com/LongMa-2025/AIGVDBench)
- AIGVDBench, newer SOTA: V-PVP 95.28 AUC ([Cui et al.](https://arxiv.org/abs/2607.15321)); TRACE claims more but gives no numbers in its abstract ([TRACE](https://arxiv.org/abs/2609.25775)).
- **GenVidBench** (Ni et al., AAAI 2026, arXiv 2501.11340): 6.78M videos, 11 generators, cross-source and cross-generator splits. — [arXiv](https://arxiv.org/abs/2501.11340). ReStraV reports 93.0% average ACC on it ([ReStraV](https://arxiv.org/html/2507.00583v4)). But frame-type counts alone reach AUC 0.999 on it ([Cakiroglu et al.](https://arxiv.org/html/2606.31004v2)).
- **GenVideo** (DeMamba, 2024): about 2.3M training clips; the test set is MSR-VTT real versus 10 generators. — [DeMamba](https://arxiv.org/html/2405.19707v3). The real clips are about 3 fps, which creates temporal shortcuts ([Michels et al.](https://arxiv.org/html/2607.00948v2)).
- **ComGenVid** (STALL, CVPR 2026): current generators including Sora and Veo 3. STALL scores 0.85 AUC, D3 0.73. — [STALL](https://arxiv.org/html/2603.15026v2)
- **DVF** (MM-Det, NeurIPS 2024): 2,750 real (InternVid, YouTube) and 3,938 fake clips; MM-Det reaches 92.0 AUC. The paper's own counts are inconsistent. — [MM-Det](https://arxiv.org/html/2410.23623v3)
- **VidProM** (Wang & Yang, NeurIPS 2024 D&B, arXiv 2403.06098): 1.67M real prompts and 6.69M videos from 4 diffusion models; CC-BY-NC 4.0. — [arXiv](https://arxiv.org/abs/2403.06098). ReStraV reports 98.63 AUROC on it ([ReStraV](https://arxiv.org/abs/2507.00583)).
- **GenBuster-200K / GenBuster-Bench(++)**: in-domain, out-of-domain and in-the-wild tracks. — [BusterX](https://arxiv.org/abs/2505.12620). Already motion-balanced; motion detectors score about 0.5 AUC on it and WaveRep 0.919 ([Michels et al.](https://arxiv.org/html/2607.00948v2)).
- **ViF-Bench** (Skyra): 3k samples from more than 10 generators; Skyra 91.0% ACC, DeMamba 64.3%. — [Skyra](https://arxiv.org/html/2512.15693v2)
- **MintVid** (VideoVeritas): 3k videos from 9 generators plus real clips with factual errors. — [VideoVeritas](https://arxiv.org/abs/2602.08828)
- **Deepfake-Eval-2024** (Chandra et al., arXiv 2503.02857): 45 h of in-the-wild video, 56.5 h of audio and 1,975 images from 88 websites in 52 languages. Open-source video detectors lose about 50% AUC compared with academic benchmarks. Mostly face and person deepfakes, so of limited relevance to T2V. — [arXiv](https://arxiv.org/abs/2503.02857)
- **LOKI** (Ye et al., ICLR 2025 spotlight, arXiv 2410.09732): 18K LMM questions across video, image, 3D, text and audio; evaluates LMMs only. — [arXiv](https://arxiv.org/abs/2410.09732)
- **Artifact-Bench** (arXiv 2605.18984, 2026): covers **photorealistic, animated and CG-style** generated video; 19 MLLMs are near or below chance in hard settings. — [arXiv](https://arxiv.org/abs/2605.18984)
- **GenVideoLens** (arXiv 2603.18625, 2026): 400 deceptive fake and 100 real clips, 15 annotated dimensions. — [arXiv](https://arxiv.org/abs/2603.18625)
- **PhysForensics-Bench** (PIVOT) and **MVAD** cover audio-video (Q6).
- Replication check: of six feature sets that clear the real-vs-real floor on GenVidBench by 0.15 or more, only WaveRep's two keep their score on AIGVDBench (head 0.996 → 0.981; backbone 0.999). ReStraV falls from 0.931 to 0.673 and FVMD from 0.880 to 0.606. WaveRep's checkpoint saw two AIGVDBench generators. — [Cakiroglu et al.](https://arxiv.org/html/2606.31004v2)

### Inferences
- For Tintype's next evaluation round, the most informative additions are AIGVDBench (closed-source subset), GenBuster-Bench++ (motion-balanced), ComGenVid and the content-matched Davodi cohort, all scored under **one shared re-encode**, with false-positive rates reported on **at least two independent real collections**, as Cakiroglu et al. recommend.
- Benchmark AUCs from GenVideo, GenVidBench (as shipped) and VidProM should not be used to choose temporal features for Tintype without fps- and container-controlled re-runs.

### Gaps
- Vript (a video captioning dataset) and MSVD-based sets were not confirmed as detection benchmarks in this pass. ComGenVid reportedly uses MSVD real clips per Tintype's survey; the STALL paper text retrieved did not confirm the real source.
- No unified SOTA leaderboard covering Veo 3.1, Sora 2, Kling 2.6 and Seedance as of November 2026 was found.

---

## Q5. Reducing false positives on real footage (animation, CGI, gameplay, stabilised or denoised phone video, screen recordings)

### Takeaway
The 2026 literature names the mechanism behind Tintype's visible failure. **Detectors learn the identity of their real corpus** (fps, codec, resolution, motion level, content style), so they flag real clips from *other* collections at 15–100% when the threshold was set on the training corpus. Calibrating the threshold on real clips from the *target* collection brings false positives back to about 5%, and frame-level frequency detectors (WaveRep) lose little recall when that is done. Motion-based temporal detectors are the most fragile here. For animation, CGI and gameplay, Tintype’s phase-5 results motivate auditing these domains as authentic negatives, alongside concurrent training changes. Benchmarks such as Artifact-Bench and MVAD include animated and CG content.

### Cited Findings
- With the threshold set to flag 5% of the training benchmark's real clips, linear readouts on ReStraV and WaveRep features, and the motion-vector and appearance probes, each flag **15% or more of another real collection**. — [Cakiroglu et al. 2026](https://arxiv.org/abs/2606.31004)
- Extremes: WaveRep's *backbone* features fitted on GenVidBench flag 81% of MSR-VTT (100% when fitted on AIGVDBench). ReStraV fitted on AIGVDBench flags 90% of MSR-VTT. WaveRep's *head* flags 24–27% of DVF's MPEG-4 clips as shipped, and at most 6% after an H.264 re-encode. — [Cakiroglu et al.](https://arxiv.org/html/2606.31004v2)
- One shared re-encode (libx264 CRF 23, GOP 12, 8 fps, 512×512) brings WaveRep's head to 9% or less everywhere. ReStraV still flags 43% of MSR-VTT. — [Cakiroglu et al.](https://arxiv.org/html/2606.31004v2)
- **Threshold set on half the target collection's real clips:** false-positive rate back to 4.1–6.8%. Recall cost: ReStraV 39 → 22%, CLIP 40 → 35%, WaveRep head 95 → 93%, WaveRep backbone 99 → 99%. The authors recommend this for any detector deployed on a new collection. — [Cakiroglu et al.](https://arxiv.org/html/2606.31004v2)
- Motion-based detectors (D3, ReStraV, Over-Coherence, NSG-VD) depend on "generated video moves less". On motion-balanced data they drop (D3 −0.097, ReStraV −0.065, Over-Coherence −0.113 AUC). Blurring a single frame of each generated video collapses them (D3 0.794 → 0.456). WaveRep is essentially unchanged (0.989 → 0.989). — [Michels, Jorissen, Michiels 2026](https://arxiv.org/html/2607.00948v2)
- The authors' trivial motion metric (std of mean absolute frame difference) rivalled the learned detectors, which indicates dataset-level signal. — [Michels et al.](https://arxiv.org/html/2607.00948v2)
- WaveRep's real training set excluded screen recordings, static content and minimal camera motion. — [Corvi et al.](https://arxiv.org/html/2506.16802v2)
- UNITE uses GTA-V game footage (SAIL-VOS-3D) as a synthetic proxy to broaden the domain, together with an attention-diversity loss. — [UNITE](https://arxiv.org/html/2412.12278v2)
- Skyra's RL reward had to be made asymmetric because symmetric penalties pushed the model towards "fake". Gemini-2.5-flash misreads compression artifacts and motion blur as forgery. — [Skyra](https://arxiv.org/html/2512.15693v2)
- Artifact-Bench's taxonomy covers photorealistic, animated and CG-style generated videos. MLLMs approach random on hard settings. — [Artifact-Bench](https://arxiv.org/abs/2605.18984)
- MVAD spans realistic and anime styles, but the paper does not analyse false positives on anime. — [MVAD](https://arxiv.org/html/2512.00336v4)
- SpecSem-Net gates spectral features with semantics to "reduce misjudgments arising from spectral noise". — [SpecSem-Net](https://arxiv.org/abs/2605.17311)
- STALL fits its whole score on about 33k real videos only, so its threshold is a percentile of real footage. — [STALL](https://arxiv.org/html/2603.15026v2)
- Tintype's own finding: cartoons, 3D animation, motion graphics and gameplay were the main source of video false alarms, and adding about 2,400 such clips to training fixed most of it. — [/home/user/tintype/docs/RESEARCH.md §4](../../docs/RESEARCH.md)

### Inferences
- **Tintype's 3/20 false positives on the Davodi cohort are probably a corpus-identity effect.** The authentic clips are Pixabay stock footage: professionally shot, often graded, stabilised, denoised and re-encoded by a stock pipeline. That kind of real footage is likely rare in Kinetics, MSR-VTT, MSVD and Panda-style training sets. The Cakiroglu results predict exactly this kind of transfer error (15% or more on a new real collection at a training-set threshold). This is an inference; it should be tested by scoring a few hundred Pixabay or Pexels clips outside the cohort.
- **Concrete actions, ranked by cost-effectiveness:**
  1. **Real-collection audit:** compute Tintype's false-positive rate at its shipped threshold separately on at least 5 real collections: stock (Pixabay/Pexels), phone (stabilised, HDR, night-mode denoised), drone/gimbal, screen recordings, animation/CGI/gameplay, and broadcast. Report each separately, as Cakiroglu et al. recommend.
  2. **Add stock and pro-graded real footage** (and stabilised, denoised phone clips) to the video fine-tuning negatives, motion-balanced against the generated clips. Expected effect: precision up on the matched cohort with little recall loss, since Tintype's frame path resembles WaveRep's head, which lost about 2 points of recall under per-collection recalibration. Cost: data collection plus one fine-tuning phase. Risk: generators are trained on stock footage, so some recall loss on generated "stock-like" clips is possible. Measure it on the matched cohort pairs.
  3. **Domain-aware thresholds:** a light content-type classifier (animation/CGI/gameplay/screen vs camera) feeding per-domain calibrated thresholds, or a STALL-style real-likelihood term in the fusion, so that "atypical for real camera footage" is not automatically read as "generated".
  4. **Shortcut guards in training:** fixed-pattern re-encoding and fps normalisation for any temporal feature, single-frame blur and duplicate-frame augmentation, and motion-balanced sampling. Run the VidAudit frame-type and motion-vector probes on Tintype's training split to verify there is no container leak.
- **Do not** add motion-only temporal statistics (D3, ReStraV, optical-flow magnitude) as standalone evidence. They push smooth, stabilised real footage towards "generated".

### Gaps
- No paper found that measures generated-video detector false-positive rates *specifically* on animation, CGI, gameplay, stabilised phone or screen-recorded footage as separate categories.
- No public study of detector behaviour on heavily denoised or computational-photography phone video (night mode, HDR merges, AI upscaling in phone pipelines) for *video*.

---

## Q6. Audio-visual cues for generated video with sound (Veo 3 era)

### Takeaway
Detection work for jointly generated audio and video is very new and weak. MVAD (Nov 2025) is the first dataset dedicated to general (non-face) AI video with audio, and existing audio-visual detectors are inconsistent on it, often near chance. PIVOT (Sep 2026) checks physical-law constraints between sound and image and reaches only about 70–72% accuracy on Seedance and Veo, though that beats Gemini 3.1 Pro (about 54–57%). Nothing here is ready to raise Tintype's precision or recall. Audio could at most become a weak extra cue for clips with sound.

### Cited Findings
- **MVAD** (Hu et al., arXiv 2512.00336, 2025): about 121k samples from 23 generators, including Veo 3 and Sora 2 with native audio and Kling, Seedance, Vidu, Wan and avatar models. Patterns: fake-fake, fake-real, real-fake. Real sources include MSR-VTT, MSVD, OpenVid-1M and InternVid-10M. Styles: realistic and anime. — [arXiv HTML](https://arxiv.org/html/2512.00336v4) · [GitHub](https://github.com/HuMengXue0104/MVAD)
- MVAD results: on real-video/fake-audio, AVFF reaches 75.5% ACC and 86.4% AP. On fake-fake, LipFD is best at 66.7 F1 while AVFF (44.9% ACC) and AVH-Align (32.5% ACC) fail. The best cross-pattern F1 is 68.7%. H.264 compression is the most harmful perturbation. No audio-only ablation was run. The paper's counts and some numbers are internally inconsistent. — [arXiv HTML](https://arxiv.org/html/2512.00336v4)
- **PIVOT** (Zheng et al., arXiv 2609.15562, 2026): estimates physical quantities from video and audio and verifies the physical laws that apply, producing inspectable evidence. On PhysForensics-Bench (9 event families, 2 generators): 70.30% ACC / 64.29 F1 on Real+Seedance and 72.16% / 65.82 on Real+Veo, against Gemini 3.1 Pro at 53.96% / 60.09 and 57.22% / 63.44. — [arXiv abstract](https://arxiv.org/abs/2609.15562)
- Multi-modal forgery representation with an LMM semantic branch, a spatio-temporal visual branch and a multi-scale partial-spoof audio branch for detection plus temporal localisation (Le et al., arXiv 2605.07232, 2026). The abstract gives no numbers. — [arXiv](https://arxiv.org/abs/2605.07232)
- **VideoASMR-Bench** (arXiv 2512.13281): 1,500 real and 2,235 synthetic ASMR clips from 9 video generators. Gemini-3-Pro fails to reliably detect the AI clips while humans do relatively well. — [arXiv](https://arxiv.org/abs/2512.13281)
- Audio-visual synchronisation was the weakest dimension for Sora 2 and Veo 3.1 in a T2V evaluation benchmark (a generation-quality study, not detection). — [BRITE, arXiv 2605.00873](https://arxiv.org/pdf/2605.00873)
- AGAV-Rater adapts an LMM for quality assessment of AI-generated audio-visual content (quality scoring, not detection). — [arXiv 2501.18314](https://arxiv.org/abs/2501.18314)

### Inferences
- For Tintype, audio is **not** a near-term lever. The audio-visual detectors that exist are face- or lip-sync-centric and do poorly on fully generated audio-video. A cheaper and more reliable route for Veo-family clips is provenance (SynthID watermark or C2PA manifests), which Tintype already partly handles for images via Content Credentials. Whether Tintype reads video C2PA was not checked here.
- A possible small experiment: an off-the-shelf audio deepfake detector score as one extra input to the fusion for clips with an audio track. Expected gain is unknown, and current results suggest it would be small and compression-sensitive.

### Gaps
- No peer-reviewed numbers were found for audio-only detection of Veo 3 or Sora 2 soundtracks.
- PIVOT's per-law coverage and runtime were not retrieved.

---

## Q7. Synthesis: ranked shortlist of the most promising ideas for Tintype

### Takeaway
Tintype's frame-level design is in line with the best 2025–26 evidence. Its visible weakness (15% false positives on real stock footage matched to generated content) is best addressed with **data and calibration** before architecture, and the one architectural addition with credible evidence is a **small patch-token temporal head on the frozen ViT-S**, trained with shortcut guards.

### Cited Findings
- Real-collection transfer false positives of 15% or more at training thresholds; per-collection recalibration returns about 5% with about 2 points of recall cost for a WaveRep-like head. — [Cakiroglu et al. 2026](https://arxiv.org/html/2606.31004v2)
- Patch-velocity readout: 95.28 AUC on AIGVDBench with about 0.5M parameters against about 90 for per-frame image detectors. — [Cui et al. 2026](https://arxiv.org/html/2607.15321v2)
- Training-free real-only likelihood (STALL): about 0.8 AUC alone, fitted on real data only, with cheap embeddings. — [STALL](https://arxiv.org/html/2603.15026v2)
- Wavelet band replacement: +3.6 bAcc / +1.1 AUC in ablation, and robust under motion rebalancing and blur where temporal detectors collapse. — [Corvi et al.](https://arxiv.org/html/2506.16802v2); [Michels et al.](https://arxiv.org/html/2607.00948v2)
- Few-shot adaptation (under 1 minute of video per new generator) recovers about 0.98 AUC. — [Vahdati et al.](https://arxiv.org/html/2404.15955v1)

### Inferences — ranked shortlist

1. **Real-footage false-positive programme: multi-collection audit, stock and graded real negatives, per-domain calibration.** *(Precision ↑↑, recall ≈ or slightly ↓, speed unchanged.)*
   - Score Tintype at its shipped threshold on at least 5 independent real collections, with Pixabay/Pexels stock first, since the 3/20 false positives come from Pixabay.
   - Add a few thousand stock, drone, gimbal and stabilised or denoised phone clips to the video negatives, motion-balanced.
   - Optionally add a content-domain flag (camera vs animation/CGI/gameplay/screen) with per-domain thresholds.
   - Evidence: Cakiroglu et al. 2026; Michels et al. 2026; Tintype's own cartoon/gameplay fix. Cost: data plus one fine-tuning phase. Risk: some recall loss on stock-like generated clips. Measure on the matched pairs and on held-out generators.

2. **Patch-token temporal head (V-PVP- or IAFA-style) on the frozen ViT-S, fused with the mean frame logit.** *(Recall ↑ on unseen generators, precision ≈ if trained with guards, speed about 1.5–2× when run on every clip or about 1.1–1.2× when run only on the uncertain band; these are estimates.)*
   - Decode bursts of 2–3 consecutive frames at 4–8 points and resample them to a canonical fps.
   - Take patch-token differences and train an attention or velocity head of about 0.5M parameters, with the backbone frozen, motion-balanced, fps- and duration-normalised, and with blur and duplicate-frame augmentation.
   - Evidence: V-PVP +5 AUC on AIGVDBench; MM-Det IAFA +13 AUC; UNITE. Risk: transfer to an image-pretrained ViT-S is unproven, and there is a motion-shortcut hazard.

3. **STALL-style real-likelihood score on Tintype's own embeddings as a calibrated "typical real footage" term in the fusion or uncertain band.** *(Precision ↑ on atypical-but-real footage, small recall effect, negligible cost.)*
   - Fit PCA whitening and a Gaussian on more than 10k real clips spanning camera, stock, animation, gameplay and screen captures.
   - Spatial term from Tintype's existing sparse frames; temporal term only if bursts are decoded (item 2).
   - Evidence: STALL (CVPR 2026); MobileNet-level embeddings suffice. Risk: on its own it is weak (about 0.8 AUC), so use it only as a fusion input.

4. **WaveRep wavelet band-replacement augmentation in the next video fine-tuning phase.** *(Recall ↑ on unseen generators and re-encoded clips, zero inference cost.)*
   - Replace the fake frame's LL, LH and HL Haar bands with the matched real frame's (or a VAE reconstruction's), forcing the model onto diagonal high-frequency cues that survive H.264.
   - Evidence: Corvi et al. NeurIPS 2025, and robustness in Michels et al. 2026. Risk: needs paired real/fake or VAE-reconstructed frames; the effect on Tintype's six-view setup is unknown.

5. **Early-exit and pooling hardening plus a few-shot generator refresh.** *(Recall ↑ on hard, compressed or partial clips at a small cost on borderline clips only.)*
   - For clips in the uncertain band, score all 16 frames (or 32), add a top-k or max pooling term alongside the mean to catch partial or spliced clips, and do not early-exit "real" before both halves are sampled.
   - Fine-tune quarterly with a few minutes of each new generator.
   - Evidence: Vahdati et al. (80 patches lift AUC 0.953 → 0.984 at CRF 40; under 1 minute per generator recovers about 0.98 AUC); FakeParts.

Explicitly **not recommended** for Tintype: optical-flow branches (AIGVDet), diffusion-inversion features (DIVID, NSG-VD, TRACE), standalone trajectory statistics (D3, ReStraV), MLLM detectors (Skyra, BusterX++, VidGuard-R1, VideoVeritas, MM-Det's LMM branch) and codec-only detectors. Each is either 30–500× slower, or shown in 2026 audits to rely on dataset shortcuts that raise false positives on real footage.

### Gaps
- None of the shortlisted gains has been measured on Tintype's ViT-S or on the Davodi content-matched cohort. Each needs an internal A/B with false-positive rates reported per real collection.
- The 20-authentic-clip cohort is too small to resolve false-positive changes below 5 percentage points. A larger matched set (more Pixabay/Pexels themes) would be needed to confirm a precision gain.
