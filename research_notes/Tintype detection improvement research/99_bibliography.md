# Master bibliography: Tintype detection-improvement research notes (01–09) and verification

**Distinct academic papers, reports and benchmarks: 267.** That is 253 with an arXiv ID plus 14 cited only by venue, DOI, proceedings link or institutional record. Rows 1–231 come from notes 01–08 (219 arXiv + 12), rows 232–258 from the gap-fill note 09 (26 arXiv + 1), and rows 259–267 from the adversarial verification of the draft report (8 arXiv + 1). Updated 2026-11-13. Software, project pages, model cards, blogs, review or aggregator sites, and challenge web pages are not counted. They are listed separately at the end.

Compiled 2026-11-13 from `01_low_level_forensic_signals.md` through `08_calibration_openset_evaluation.md`. Rows 232–267 (sections R and S) were added afterwards from `09_gap_fill.md` and the verification pass. Entries cited under different names were merged into one row each. For example, B-Free is cited in 7 notes (as "B-Free", "[P5]", "#13", "1.9", and as "Guillaro et al."). Aligned Datasets is also called "AlignedForensics" and "On the effectiveness of dataset alignment". Deepfake-Eval-2024 is cited in 4 notes, and Chameleon is merged with AIDE. Entries are sorted by topic area, then by year.

## How to read this

**Cited in / tag.** Each row gives the note number and the verification tag *that note* gave the citation. Each note uses its own legend:

| Tag | Meaning | Used in |
|---|---|---|
| V | Numbers read from the paper (arXiv abstract/HTML) this session | 01, 02 |
| A | Recalled or abstract-level; figures not re-fetched | 01 |
| R | Recalled from the abstract; not re-fetched | 02 |
| rec | "[recalled, not re-verified]" | 06 |
| fetched / search / abs | 07's "[fetched]" (primary page read), "[search]" (search-snippet facts from primary pages) and "[abstract, known]" | 07 |
| u | Untagged. The note's stated default is that facts were checked against primary pages this session (06 says so explicitly; 05 says numbers come from arXiv abstracts and HTML; 03, 04 and 08 cite primary arXiv pages throughout) | 03, 04, 05, 06, 08 |
| snip | The note says it saw only a search snippet or summary | any |
| sec | Figures or details come from a secondary source (review site, aggregator, press) | any |
| ctx | Named only as context or a pointer; nothing claimed from it | any |
| lead | Listed as an unverified lead | 03 |

**Check.** I looked up all 219 arXiv IDs in the arXiv API (export.arxiv.org) on 2026-11-13 and compared the returned title and authors with the notes. "OK" means the ID exists and matches the paper the notes describe. CONFIRMED, CORRECTED and NOT FOUND mark the targeted spot checks in the next section; for the non-arXiv rows these used web search. "n/c" means not checked; only the venue link from the notes is shown.

---

## Spot-check results (targeted, prioritised by how much a recommendation leans on the paper)

All 219 arXiv IDs resolved: **no entry is NOT FOUND.** Two IDs needed correction, and several metadata details differ from the notes.

| # | Entry (note) | ID / claim in notes | Result | Detail |
|---|---|---|---|---|
| S1 | SPSL, "Spatial-Phase Shallow Learning" (01) | arXiv 2103.01412 | **CORRECTED → 2103.01856** | 2103.01412 is "Some Finite Sample Properties of the Sign Test" (Yong Cai). SPSL (Honggu Liu et al.) is 2103.01856. |
| S2 | SPAI (02 #14) | "arXiv ID not confirmed", linked to 2507.10236 | **CORRECTED → 2411.19417** | 2507.10236 is Konstantinidou et al., "Navigating the Challenges…", which only evaluates SPAI. 01, 03, 07 and 08 already give the correct 2411.19417. |
| S3 | Yang et al., "…Secretly Achieve SOTA Accuracy, If Calibrated" (08) | 2602.01973, AAAI 2026 | **CONFIRMED** | arXiv comment says "AAAI 2026". |
| S4 | ReSIDe, Zheng & Seidman (08) | 2605.08574 | **CONFIRMED** | arXiv lists no venue. The "NeurIPS 2026 poster" claim was not verified. |
| S5 | Cakiroglu et al., corpus identity (05) | 2606.31004 | **CONFIRMED** | "Where the Real Clips Came From: Corpus Identity in AI-Generated Video Detection". |
| S6 | Michels et al., motion shortcuts (05) | 2607.00948 | **CONFIRMED** | "Dataset Biases and Shortcut Learning in Motion-Based AI-Generated Video Detection". |
| S7 | V-PVP, "Rethinking the Readout" (05) | 2607.15321 | **CONFIRMED** | Cui, Qin et al. |
| S8 | TTC, Test-Time Curriculum (08) | 2608.00559 | **CONFIRMED** | Zhang, Gu et al. |
| S9 | ForensicZip (07) | 2603.12208 | **CONFIRMED** | Lai, Yu et al. |
| S10 | NTIRE 2026 challenge report (01–04) | 2604.11487 | **CONFIRMED** | arXiv comment says "CVPR 2026 NTIRE Workshop". |
| S11 | Manifold Induced Biases (01: OpenReview only; 08: 2504.15470) | 2504.15470 | **CONFIRMED** | Brokman et al.; arXiv comment says "Accepted to ICLR 2025". |
| S12 | B-Free (7 notes) | 2412.17671 | **CONFIRMED** | Guillaro et al. |
| S13 | Community Forensics (02, 03, 07) | 2411.04125 | **CONFIRMED** | journal_ref: CVPR 2025, pp. 8245–8257. |
| S14 | DCPT (04) | 2604.10102 | **CONFIRMED** | Yang, Hou, Yang. 6-page preprint, no venue. |
| S15 | LPT, NTIRE 3rd place (04) | 2604.12307 | **CONFIRMED** | arXiv comment says "3th place (3/514)… CVPRW-26". 04 calls HEDGE "4th of 193 teams", so the team counts disagree. |
| S16 | QuAD (03, 04) | 2604.15027 | **CONFIRMED** | arXiv comment says "APAI Workshop at CVPR 2026" (the notes give no venue). |
| S17 | DEAR, "Dissect and Prune" (04) | 2606.10309, "Jaehyun Choi, Hyun Seok Seong, …" | **CONFIRMED (author fix)** | arXiv lists **Dahye Kim** as first author, then Jaehyun Choi and Hyun Seok Seong. arXiv comment says "Accepted to ICML 2026". |
| S18 | OmniDFA (08) | 2509.25682 | **CONFIRMED (title differs)** | v1 is titled "OmniDFA: A Unified Framework…". The current version is "Few-Shot Synthetic Image Attribution: Identifying Unseen Generators with Limited Samples" (Shiyu Wu et al.), and its arXiv comment says "ECCV 2026". |
| S19 | AIGVDBench (05) | 2601.11035 | **CONFIRMED (title differs)** | The arXiv title is "Your One-Stop Solution for AI-Generated Video Detection" (Long Ma et al.). |
| S20 | BIAS-ID (04) | 2605.31153 | **CONFIRMED** | arXiv comment says "NeurIPS 2026, Track on Evaluations and Datasets". |
| S21 | Look Closer (06) | 2609.37937 | **CONFIRMED** | Zhang, Wu, Liu et al. |
| S22 | Embedding-Space Explanations (02 #18) | 2611.04111, no authors given | **CONFIRMED** | Nikos Giakoumoglou et al., 2026-11-03. |
| S23 | STALL / ComGenVid (05) | 2603.15026 | **CONFIRMED** | arXiv comment says "Accepted to CVPR 2026". |
| S24 | DDA (01, 02) | 2505.14359 | **CONFIRMED** | arXiv comment says "NeurIPS 2025 Spotlight". |
| S25 | DACOM, ICLR 2026 poster (04) | OpenReview / ICLR page only | **CONFIRMED (web)** | The ICLR 2026 poster page and OpenReview PDF exist. Authors are still not identified. The 04 figure (64.19% bAcc) comes from a secondary review site. |
| S26 | DFD-HR, CVPR 2026 (07) | papernotes.org only | **CONFIRMED (web)** | CVPR 2026 poster by Jiamu Sun, Zhiyuan Yan, Ke-Yue Zhang, Taiping Yao, Shouhong Ding. The CVF supplement exists. |
| S27 | Tao et al., quantized deepfake detection (07) | IJCAI 2025, pp. 520–528 | **CONFIRMED (web)** | DOI 10.24963/ijcai.2025/59. |
| S28 | OST (08) | NeurIPS 2022 | **CONFIRMED (web)** | Liang Chen et al. |
| S29 | Frequency Masking, Doloriel & Cheung (03, lead) | ICASSP 2024, no link | **CONFIRMED** | arXiv 2401.06506. |
| S30 | BiHPF (01) | CVF link only | **CONFIRMED** | arXiv 2109.00911 also exists. |
| S31 | Synthbuster (01) | DOI 10.1109/OJSP.2023.3337714 | **CONFIRMED (web)** | IEEE OJSP, Nov 2023. The DOI page itself was not opened. |
| S32 | RAID (04) | 2506.03988, "Italian and German authors" | **CONFIRMED** | First author Hicham Eddoubi, with Jonas Ricker et al. |

Naming caveat: in 02, "DINOv3 paper" always means **Huang et al., "Rethinking Cross-Generator Image Forgery Detection through DINOv3" (2511.22471)**, not Meta's DINOv3 backbone paper. Meta's paper is never cited.

---

## A. Low-level, frequency and artifact signals (image)

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

## B. Reconstruction, zero-shot, training-free and real-only detectors

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

## C. Foundation-model and representation-based detectors (CLIP / DINO / PEFT / fusion)

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

## D. MLLM / VLM explainable detection (image)

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

## E. Training data, dataset bias, alignment and scaling

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

## F. Continual, few-shot and test-time adaptation

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

## G. Robustness to degradation, social-network laundering, and robustness challenges

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

## H. Adversarial robustness, watermarking and provenance

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

## I. Patch and view selection, native resolution, and aggregation

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

## J. Efficiency: backbones, early exit, token reduction, quantization, distillation, ensembles and fusion

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

## K. Face forgery (image and video)

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

## L. Local edits, inpainting and forgery localization

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

## M. Generated-video detection: methods

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

## N. Generated-video benchmarks, audits and audio-visual work

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

## O. Calibration, selective prediction and conformal guarantees

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 214 | "A Gentle Introduction to Conformal Prediction…" | Angelopoulos & Bates | arXiv | 2021 | [arXiv 2107.07511](https://arxiv.org/abs/2107.07511) | 08[u] | OK |
| 215 | Conformal FPR control for machine-generated text detection (multi-level CP) | X. Zhu et al. | ACL | 2025 | [arXiv 2505.05084](https://arxiv.org/abs/2505.05084); [ACL](https://preview.aclanthology.org/setup/2025.acl-long.601) | 08[u] | OK |
| 216 | "Your AI-Generated Image Detector Can Secretly Achieve SOTA Accuracy, If Calibrated" | M. Yang et al. | AAAI | 2026 | [arXiv 2602.01973](https://arxiv.org/abs/2602.01973) | 08[u] | CONFIRMED |
| 217 | ReSIDe: "Post-hoc Selective Classification for Reliable Synthetic Image Detection" | K. Zheng & Seidman | NeurIPS (per 08; not on arXiv) | 2026 | [arXiv 2605.08574](https://arxiv.org/abs/2605.08574) | 08[u] | CONFIRMED |
| 218 | DISCERN: "Uncertainty-Aware Deepfake Detection via Multi-View Structural Learning" | M. U. Farooq et al. | arXiv | 2026 | [arXiv 2607.28769](https://arxiv.org/pdf/2607.28769) | 08[snip] | OK |

## P. Source attribution and open-set recognition

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 219 | WOUAF | C. Kim et al. | CVPR | 2024 | [arXiv 2306.04744](https://arxiv.org/abs/2306.04744) | 08[u] | OK |
| 220 | "Are CLIP features all you need for Universal Synthetic Image Origin Attribution?" | Cioni et al. | ECCV Workshop | 2024 | [arXiv 2408.09153](https://arxiv.org/abs/2408.09153) | 08[u] | OK |
| 221 | OmniDFA (current arXiv title "Few-Shot Synthetic Image Attribution…") | S. Wu et al. | arXiv (ECCV 2026 per arXiv comment) | 2025 | [arXiv 2509.25682](https://arxiv.org/abs/2509.25682) | 08[u] | CONFIRMED (title differs) |
| 222 | "Supervised Contrastive Learning for Few-Shot AIGI Detection and Attribution" | Álvarez Urueña et al. | arXiv | 2025 | [arXiv 2511.16541](https://arxiv.org/abs/2511.16541) | 08[snip] | OK |
| 223 | Face-Trace: open-set attribution of synthetic face generators | Infantino et al. | arXiv | 2026 | arXiv 2607.07545 (no link) | 08[u] | OK |
| 224 | "Scalable Black-Box Model Attribution for Images" | Livne et al. | arXiv | 2026 | [arXiv 2608.15652](https://arxiv.org/pdf/2608.15652) | 08[snip] | OK |

## Q. Image benchmarks, evaluation methodology and surveys

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 225 | ImagiNet | Boychev & Cholakov | arXiv | 2024 | [arXiv 2407.20020](https://arxiv.org/pdf/2407.20020v1.pdf) | 08[u] | OK |
| 226 | "Methods and Trends in Detecting AI-Generated Images: A Comprehensive Review" | Mahara & Rishe | arXiv | 2025 | [arXiv 2502.15176](https://arxiv.org/pdf/2502.15176) | 01[review, source for SSP] | OK |
| 227 | Deepfake-Eval-2024 | Chandra et al. | arXiv | 2025 | [arXiv 2503.02857](https://arxiv.org/abs/2503.02857) | 04[u], 05[u], 06[u], 08[u] | OK |
| 228 | AI-GenBench | Pellegrini et al. | arXiv | 2025 | [arXiv 2504.20865](https://arxiv.org/abs/2504.20865) | 08[u] | OK |
| 229 | DRAGON | Bertazzini et al. | arXiv | 2025 | [arXiv 2505.11257](https://arxiv.org/abs/2505.11257) | 08[u] | OK |
| 230 | "How well are open sourced AI-generated image detection models out-of-the-box…" ("Out-of-the-box benchmark") | S. Ren et al. | arXiv | 2026 | [arXiv 2602.07814](https://arxiv.org/abs/2602.07814) | 07[ctx], 08[u] | OK |
| 231 | "The Synthetic Media Shift…" (Community Notes in-the-wild study) | Chrysidis et al. | arXiv | 2026 | arXiv 2604.15372 | 02[u] | OK |
## R. Added by the completeness critic (note 09, gap fill)

Tags follow note 09: u = full text or HTML read on 2026-11-13; abs = abstract only, no full-text figures; snip = search-result snippet of the primary page; ctx = context only. "Check" for these rows means the critic confirmed the ID is absent from rows 1–231 with `grep` and looked it up on its arXiv abstract page.

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 232 | D³QE: "Discrete Distribution Discrepancy-aware Quantization Error for AR-generated image detection" (ARForensics benchmark) | Y. Zhang et al. | ICCV | 2025 | [arXiv 2510.05891](https://arxiv.org/abs/2510.05891) | 09[u] | OK |
| 233 | PRADA: "Probability-Ratio-Based Attribution and Detection of AR-generated images" | S. Damm, J. Ricker, H. Petzka & A. Fischer | CVPR Findings | 2026 | [arXiv 2511.20068](https://arxiv.org/abs/2511.20068) | 09[u] | OK |
| 234 | "Exploiting the Final Component of Generator Architectures for AI-Generated Image Detection" | Y. Liu et al. | arXiv | 2026 | [arXiv 2601.20461](https://arxiv.org/abs/2601.20461) | 09[abs] | OK |
| 235 | GPT-ImgEval | Z. Yan et al. | arXiv | 2025 | [arXiv 2504.02782](https://arxiv.org/html/2504.02782v3) | 09[u] | OK |
| 236 | "A Benchmark Dataset for MLLM-Generated Image Detection: GPT Image2 & Nano Banana2" | Z. Zhang et al. | arXiv | 2026 | [arXiv 2608.01258](https://arxiv.org/abs/2608.01258) | 09[u] | OK |
| 237 | "Guess the Unified Model: How Much Can We Recover from Generated Images?" | J. Cekinmez et al. | arXiv | 2026 | [arXiv 2605.25254](https://arxiv.org/abs/2605.25254) | 09[abs] | OK |
| 238 | OMAT / GenImage++: "Breaking Latent Prior Bias in Detectors" | Y. Zhou et al. | arXiv | 2025 | [arXiv 2506.00874](https://arxiv.org/html/2506.00874v1) | 09[u] | OK |
| 239 | INP-X: "AI-Generated Image Detectors Exploit Global Artifacts: Evidence from Inpainting Exchange" | E. Nebioglu, E. Bilgiç & A. Popescu | WACV | 2027 | [arXiv 2602.00192](https://arxiv.org/abs/2602.00192) | 09[u], verifier R4 | OK |
| 240 | "Detecting Localized Deepfakes: How Well Do Synthetic Image Detectors Handle Inpainting?" | S. Pandolfini et al. | arXiv | 2025 | [arXiv 2512.16688](https://arxiv.org/abs/2512.16688) | 09[abs] | OK |
| 241 | UniAIDet | authors not retrieved | arXiv | 2025 | [arXiv 2510.23023](https://arxiv.org/html/2510.23023v1) | 09[snip] | OK |
| 242 | "Adversarial Magnification to Deceive Deepfake Detection through Super Resolution" | D. A. Coccomini et al. | arXiv | 2024 | [arXiv 2407.02670](https://arxiv.org/html/2407.02670) | 09[u] | OK |
| 243 | "Apple's Synthetic Defocus Noise Pattern: Characterization and Forensic Applications" | authors not retrieved | arXiv | 2025 | [arXiv 2505.07380](https://arxiv.org/abs/2505.07380) | 09[snip, ctx] | OK |
| 244 | TextFake: "Benchmarking AI-Generated Image Detection on Text-Rich Images" | Y. Zhang et al. | arXiv | 2026 | [arXiv 2606.01050](https://arxiv.org/abs/2606.01050) | 09[abs] | OK |
| 245 | AIGDoc: "Beyond Natural Images: Rethinking AI-Generated Image Detection in Documents" | Z. Fu et al. | arXiv | 2026 | [arXiv 2609.14352](https://arxiv.org/abs/2609.14352) | 09[abs] | OK |
| 246 | TextRich: "A Multi-Domain Benchmark for Detecting AI-Generated Text-Rich Images from GPT-Image-2" | Y. Wang et al. | arXiv | 2026 | [arXiv 2606.19259](https://arxiv.org/abs/2606.19259) | 09[abs] | OK |
| 247 | WiSE-FT: "Robust fine-tuning of zero-shot models" | M. Wortsman et al. | CVPR | 2022 | [arXiv 2109.01903](https://arxiv.org/abs/2109.01903) | 09[u] | OK |
| 248 | "Model soups: averaging weights of multiple fine-tuned models…" | M. Wortsman et al. | ICML | 2022 | [arXiv 2203.05482](https://arxiv.org/abs/2203.05482) | 09[u] | OK |
| 249 | R²M: "Real-Aware Residual Model Merging for Robust and Generalizable Deepfake Detection" | J. Park et al. | ECCV | 2026 | [arXiv 2509.24367](https://arxiv.org/abs/2509.24367) | 09[u] | OK |
| 250 | RobustSora: "De-Watermarked Benchmark for Robust AI-Generated Video Detection" | Z. Wang, X. Liu & L. Sun | arXiv | 2025 | [arXiv 2512.10248](https://arxiv.org/abs/2512.10248) | 09[u] | OK |
| 251 | RA-Bench: "Can We Defend Against AI-Generated Video Attacks on Real-World Crisis Events?" | S. Liang et al. | arXiv | 2026 | [arXiv 2608.14391](https://arxiv.org/abs/2608.14391) | 09[abs] | OK |
| 252 | "Will Large-scale Generative Models Corrupt Future Datasets?" | R. Hataya, H. Bao & H. Arai | ICCV | 2023 | [arXiv 2211.08095](https://arxiv.org/abs/2211.08095) | 09[u] | OK |
| 253 | "Forensic Self-Descriptions Are All You Need for Zero-Shot Detection, Open-Set Source Attribution, and Clustering of AI-generated Images" | T. D. Nguyen, A. Azizpour & M. C. Stamm | CVPR | 2025 | [arXiv 2503.21003](https://arxiv.org/html/2503.21003v1) | 09[u] | OK |
| 254 | "Certifiably Robust Synthetic Image Detectors" | K. Zeng, M. Barni & B. Tondi | EUSIPCO | 2025 | DOI 10.23919/eusipco63237.2025.11226152; [Siena repository](https://usiena-air.unisi.it/handle/11365/1315942) | 09[snip] | n/c (no arXiv ID) |
| 255 | "Certifiably Robust Image Watermark" | Z. Jiang et al. | ECCV | 2024 | [arXiv 2407.04086](https://arxiv.org/abs/2407.04086) | 09[abs] | OK |
| 256 | "Certified Adversarial Robustness via Randomized Smoothing" | J. Cohen, E. Rosenfeld & J. Z. Kolter | ICML | 2019 | [arXiv 1902.02918](https://arxiv.org/abs/1902.02918) | 09[ctx] | OK |
| 257 | "AI-Generated Images: What Humans and Machines See When They Look at the Same Image" | S. Poletti et al. | arXiv (Springer CCIS) | 2026 | [arXiv 2605.06143](https://arxiv.org/abs/2605.06143) | 09[abs] | OK |
| 258 | GenImage: "A Million-Scale Benchmark for Detecting AI-Generated Image" | M. Zhu et al. | NeurIPS D&B | 2023 | [arXiv 2306.08571](https://arxiv.org/html/2306.08571v2) | 09[u] | OK |

## S. Added during adversarial verification of the draft

These were introduced by the evidence-lens verifiers as corrections or counter-evidence. "ver" = cited by a verifier; the verifier's reading depth is given in brackets. The arXiv IDs of rows 259–266 were looked up in the arXiv API (export.arxiv.org) on 2026-11-13 and the returned titles and authors match.

| # | Paper | First author et al. | Venue | Year | Link (as given) | Cited in [tag] | Check |
|---|---|---|---|---|---|---|---|
| 259 | "Conformal prediction beyond exchangeability" | R. F. Barber, E. J. Candès, A. Ramdas & R. J. Tibshirani | Annals of Statistics | 2023 | [arXiv 2202.13415](https://arxiv.org/abs/2202.13415) | ver R2 [ctx; replaces the RAW citation] | OK |
| 260 | "Conformal Prediction Under Covariate Shift" | R. J. Tibshirani, R. F. Barber, E. J. Candès & A. Ramdas | NeurIPS | 2019 | [arXiv 1904.06019](https://arxiv.org/abs/1904.06019) | ver R2 [ctx] | OK |
| 261 | "The limits of distribution-free conditional predictive inference" | R. F. Barber et al. | Information and Inference | 2021 | [arXiv 1903.04684](https://arxiv.org/abs/1903.04684) | ver R2 [ctx] | OK |
| 262 | CoReD: "Generalizing Fake Media Detection with Continual Representation using Distillation" | M. Kim, S. Tariq & S. S. Woo | ACM MM | 2021 | [arXiv 2107.02408](https://arxiv.org/abs/2107.02408) | ver R7 [abs] | OK |
| 263 | LiteUpdate: "A Lightweight Framework for Updating AI-Generated Image Detectors" | J. Lu et al. | arXiv | 2025 | [arXiv 2511.07192](https://arxiv.org/html/2511.07192v1) | ver R10 [HTML] | OK |
| 264 | "Defensive Quantization: When Efficiency Meets Robustness" | J. Lin, C. Gan & S. Han | ICLR | 2019 | [arXiv 1904.08444](https://arxiv.org/abs/1904.08444) | ver R9 [abs] | OK |
| 265 | "Understanding vision transformer robustness through the lens of out-of-distribution detection" | J. Kuang & A. Wong | arXiv | 2026 | [arXiv 2602.01459](https://arxiv.org/abs/2602.01459) | ver R9 [ctx; 4-bit only] | OK |
| 266 | "Generalizable Detection of AI Generated Images with Large Models and Fuzzy Decision Tree" | F. Wu et al. | arXiv | 2026 | [arXiv 2603.28508](https://arxiv.org/abs/2603.28508) | ver R3 [ctx; numbers not checked] | OK |
| 267 | CAID: "Benchmark dataset and framework for continual AI-generated image detection" | authors not retrieved | (HIT scholar record) | 2025–26 | [HIT scholar page](https://scholar.hit.edu.cn/en/publications/benchmark-dataset-and-framework-for-continual-ai-generated-image-/) | ver R7 [snip; numbers not verified] | n/c (no arXiv ID) |

### Count reconciliation

- Rows 1–231 come from notes 01–08. I extracted 219 distinct arXiv IDs from those notes; all 219 resolve on arXiv and each maps to exactly one row. For SPSL, the ID in the note (2103.01412) is wrong and the row gives the corrected ID 2103.01856. 12 of those papers are cited without an arXiv ID: BiHPF (#7), LGrad (#10), Synthbuster (#11), Frequency Masking (#16), DRCT (#21), TechRxiv MLLM face study (#56), Breaking Semantic Artifacts (#60), OST (#71), DACOM (#97), MDPI score-fusion (#133), Tao et al. IJCAI 2025 (#141) and DFD-HR (#150).
- Rows 232–258 come from the gap-fill note (09): 26 with arXiv IDs plus Zeng et al. (EUSIPCO, DOI only, #254). Each ID was grepped against rows 1–231 before it was added; none duplicates an earlier row.
- Rows 259–267 were introduced by the adversarial verifiers: 8 with arXiv IDs, title-checked on export.arxiv.org on 2026-11-13, plus the CAID record (#267, no arXiv ID). None duplicates an earlier row. DACOM (#97), DF40 and HEDGE, which the verifiers also cited, were already listed and are not double-counted.
- Total: 219 + 26 + 8 = **253 with an arXiv ID**, and 12 + 1 + 1 = **14 without**, for **267 distinct academic papers, reports and benchmarks**. BusterX and BusterX++ are counted separately, as are the two Cakiroglu et al. 2026 papers, the two Corvi et al. 2023 papers, and the two Wortsman et al. 2022 papers (WiSE-FT, model soups).

---

## Mentioned without any citation (not counted)

Named in the notes with no title, authors or link: GenD (08, face-deepfake baseline), ForAda (DFD-HR context), Over-Coherence and FVMD (05, video baselines), the "Where did I come from? Origin attribution" paper (08, flagged as not located), "The Adversarial AI-Art" benchmark (08, not verified), C2PA, Tree-Ring and Stable Signature (08, as provenance or watermark schemes), FaceForensics++, Celeb-DF and DFDC (06, as datasets), Meta's DINOv3 backbone paper (never cited directly), and the Davodi et al. content-matched cohort (Tintype repo evidence). Added after verification: "Three Forensic Cues for JPEG AI Images" (2025; seen only in a search listing, not verified), DFIL and SUR-LID (continual deepfake methods cited only through DECODE's table), and a CHI 2026 workshop study of explanation reliance (Zenodo record 19700249, named in note 09).

## Non-academic sources: software, project pages, model cards, blogs, reviews and challenge pages (not counted)

**Code repositories (GitHub):** mever-team/rine (02); UltraCapture/GAPL (02); beibuwandeluori/DRCT (03); MMLab-unitn/TrueFake-IJCNN25 (03); Coral79/CDDB (03); andrekassis/ai-watermark (04); grip-unina/WaveRep-SyntheticVideoDetection (05); ZSHsh98/NSG-VD (05); manic-cui/Rethinking-the-readout (05); JoeLeelyf/Skyra (05); LongMa-2025/AIGVDBench (05); HuMengXue0104/MVAD (05); SCLBD/DeepfakeBench (06); YZY-stack/Effort-AIGI-Detection (06); chail/patch-forensics (06); Zig-HS/FakeImageDetection (07); nuriachandra/Deepfake-Eval-2024 (08). 06 also names data repos without links: YZY-stack/DF40, iamwangyabin/OpenSDI, clpbc/BR-Gen, mever-team.github.io/SAGI and mever-team/texture-crop (04).

**Project pages:** LOKI (opendatalab.github.io/LOKI, 02); Stay-Positive (anisundar18.github.io, 04); B-Free (grip-unina.github.io/B-Free, 07, 08); ZED (grip-unina.github.io/ZED, 08); STALL (omerbenhayun.github.io/stall-video, 05); LaDeDa (vision.huji.ac.il/ladeda, 07); B-Free CVPR 2025 slides (cvpr.thecvf.com, 08).

**Model cards and datasets:** Hugging Face model card KabirAcharya/waterspy-sieve-ft1-int8 (07; unreviewed INT8 evidence); Hugging Face dataset README semi-truths/Semi-Truths (06); HF dataset aimagelab/RAID (04, named).

**Blogs, reviews, aggregators and press:** liner.com reviews of DACOM (04) and DF40 (06); themoonlight.io reviews of AIDE and ALEI (07); lacuna.tiptreesystems.com summary of DistilDIRE (02); en.papernotes.org summary of DFD-HR (07); theregister.com article on UnMarker (04); photoworkout.com summary of a NewsGuard audit (04; unverified); deepfake-demo.aisec.fraunhofer.de related-work pages (04, 06); alphaxiv.org overview pages (04, 07); scour.ing link to OmniAID (07); mlanthology.org entries (04); CISPA publications page hosting the Mavali et al. preprint (04).

**Challenge and society pages:** IEEE SP Cup 2025 / DFWild-Cup page at signalprocessingsociety.org (04).
