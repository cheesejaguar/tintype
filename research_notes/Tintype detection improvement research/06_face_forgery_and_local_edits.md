# Detecting partially manipulated media for Tintype: face swaps, face reenactment, local inpainting and AI edits (with localization)

Scope note. Tintype is a ViT-S/16 at 224 px. It scores six views per image (one resized fit view, the centre crop and the four most-textured native-resolution 224 crops) and averages their logits. Video is scored per frame, with the mean logit over 16 frames. It is trained only on fully generated images. On the classic-test-set "FaceForensics++ deepfakes" (300 real / 300 fake), recall is 0.143 for the default profile and 0.480 with the Community Forensics second opinion, against 0.573 for Community Forensics alone (repo `docs/BENCHMARK.md` line 222; `README.md` "Limits").

Verification status. I checked numbers against primary pages this session unless they are tagged **[recalled, not re-verified]**. Those come from my memory of the paper. They are probably right, but the report writer should check them before quoting.

---

## Q1. Generalizable face-forgery detection: what works across datasets, and which training trick gives the most cross-dataset gain per unit of cost?

### Takeaway
Two things carry most of the cross-dataset gain: synthetic "blending" fakes built from real faces only (Face X-ray, then SBI, FreqBlender, LAA-Net), and parameter-efficient adaptation of a large pretrained backbone that does not overwrite its general features (Effort, CLIP-based adapters). SBI is the cheapest trick per unit of gain: it needs no deepfake data, only real faces, landmarks and image operations. Effort-style orthogonal-subspace tuning is the cheapest way to add a new skill to a pretrained backbone without forgetting its old ones. All of this is measured on academic face-swap benchmarks. In-the-wild 2024 deepfakes cut open-source detector AUC by about half.

### Cited Findings

**Benchmarks and evaluation frameworks**
- **DeepfakeBench** (Yan, Zhang, Yuan, Lyu, Wu; NeurIPS 2023 Datasets & Benchmarks; arXiv 2307.01426). Unified preprocessing (face detection, alignment, cropping) with training and evaluation code. The repo now lists 36 detectors: 28 image, 8 video. Among them are Face X-ray, SBI, UCF, CLIP, Effort, FTCN, TALL and VideoMAE. Its cited frame-level AUC for FF++(c23)-trained detectors is low across domains: Xception scores 0.7365 on CDF-v2, 0.7077 on DFDC and 0.7374 on DFDCP; UCF scores 0.7527, 0.7191 and 0.7594. The repo is licensed CC BY-NC 4.0. — [DeepfakeBench GitHub](https://github.com/SCLBD/DeepfakeBench); [arXiv 2307.01426](https://arxiv.org/abs/2307.01426)
- **DF40** (Yan et al.; NeurIPS 2024 Datasets & Benchmarks; arXiv 2406.13495). 40 deepfake methods, which the authors describe as about 10× larger than FF++. A secondary summary gives the split as 10 face-swapping, 13 reenactment, 12 entire-face-synthesis and 5 face-editing methods. The paper reports 7 findings: top detectors do not reliably beat baselines, and pretrained CLIP features do well. Dataset, code and weights are on GitHub (YZY-stack/DF40). — [NeurIPS proceedings](https://proceedings.neurips.cc/paper_files/paper/2024/hash/34239f60eca7ce9bee5280aaf81362d8-Abstract.html); [arXiv 2406.13495](https://arxiv.org/abs/2406.13495); category counts from [Liner review](https://liner.com/review/df40-toward-nextgeneration-deepfake-detection) (secondary); CLIP finding from [Fraunhofer summary](https://deepfake-demo.aisec.fraunhofer.de/related_work/2406.13495) (secondary)
- **Deepfake-Eval-2024** (Chandra, Lee, Murtfeldt, Qiu, …, Etzioni; arXiv 2503.02857, 2025; no peer-reviewed venue listed). An in-the-wild set of media circulated in 2024: 45 h of video, 56.5 h of audio and 1,975 images, from 88 websites in 52 languages. Against earlier academic benchmarks, open-source detector AUC drops by about 50% for video, 48% for audio and 45% for images. Commercial and fine-tuned models do better but still fall short of human forensic analysts. — [arXiv 2503.02857](https://arxiv.org/abs/2503.02857)

**Blending-based synthetic training data (needs only real faces)**
- **Face X-ray** (Li, Bao, Zhang, Yang, Chen, Wen, Guo; CVPR 2020; arXiv 1912.13458). Blends two different real faces along a mask and trains the model to predict the blending boundary (the "X-ray"). It is trainable without any deepfake data and was the origin of the "blend real faces" idea. — [arXiv 1912.13458](https://arxiv.org/abs/1912.13458)
- **SBI, Self-Blended Images** (Shiohara & Yamasaki; CVPR 2022 Oral; arXiv 2204.08376). A pseudo source and a pseudo target are made from one pristine image (colour and frequency jitter, resize, landmark-mask deformation) and blended. The blends reproduce boundary and statistical-inconsistency artifacts. Gains over the baseline: +4.90 points on DFDC and +11.78 points on DFDCP. — [arXiv 2204.08376](https://arxiv.org/abs/2204.08376). With EfficientNet-B4 trained on FF++ raw, video-level AUC is about 93.18 on CDF-v2, 72.42 on DFDC, 86.15 on DFDCP and 84.83 on FFIW **[recalled, not re-verified]**. Under Effort's re-evaluation protocol (FF++ c23, video AUC), SBI scores 0.886 on CDF-v2, 0.717 on DFDC, 0.848 on DFDCP and 0.821 on average over 7 sets. — [Effort paper, arXiv 2411.15633v4 Table 1](https://arxiv.org/html/2411.15633v4)
- **FreqBlender** (Li, Zhou, Li, Wu, Li, Dong; NeurIPS 2024; arXiv 2404.13872). A Frequency Parsing Network separates forgery-related frequency components and blends them from fake faces into real faces. It is described as plug-and-play with other detectors, SBI included. I did not retrieve its numbers this session. — [arXiv 2404.13872](https://arxiv.org/abs/2404.13872)
- **LAA-Net** (Nguyen et al.; CVPR 2024; arXiv 2401.13856). Multi-task network with a binary classifier, a heatmap branch and a self-consistency branch on pixels likely to carry blending artifacts, plus an E-FPN. It trains on blending-based pseudo-fakes. A third-party aggregator lists 0.954 AUC on CDF-v2, which I could not confirm against the paper's table. — [CVF open access](https://openaccess.thecvf.com/content/CVPR2024/html/Nguyen_LAA-Net_Localized_Artifact_Attention_Network_for_Quality-Agnostic_and_Generalizable_Deepfake_CVPR_2024_paper.html); [Fraunhofer summary / aggregator](https://deepfake-demo.aisec.fraunhofer.de/related_work/2401.13856)

**Representation and adaptation methods**
- **UCF** (Yan, Zhang, Fan, Wu; ICCV 2023; arXiv 2304.13949). Disentangles forgery-irrelevant, method-specific and common forgery features. Video AUC under Effort's protocol: 0.837 on CDF-v2, 0.742 on DFDC, 0.785 on average. — [arXiv 2304.13949](https://arxiv.org/abs/2304.13949); numbers from [Effort Table 1](https://arxiv.org/html/2411.15633v4)
- **Effort, Orthogonal Subspace Decomposition** (Yan, Wang, Jin, Zhang, Liu, Chen, Yao, Ding, Wu, Yuan; ICML 2025 Oral; arXiv 2411.15633). SVD splits each weight matrix. The top-r principal components are frozen to keep pretrained knowledge, and only the residual components are trained. This uses 0.19 M trainable parameters on CLIP ViT-L/14. Results:
  - Trained on FF++ c23, video-level AUC: 0.956 on CDF-v2, 0.965 on DFD, 0.843 on DFDC, 0.909 on DFDCP, 0.977 on DFo, 0.848 on WDF, 0.921 on FFIW, 0.917 on average. On the same table SBI averages 0.821 and UCF 0.785.
  - Frame-level: Effort averages 0.874 against 0.796 for LoRA.
  - The same method gives 95.19% mean accuracy on UniversalFakeDetect and 91.1% on GenImage, which is relevant because one recipe covers both deepfakes and AI-generated images.
  - Code and weights are CC BY-NC 4.0.
  — [arXiv 2411.15633v4](https://arxiv.org/html/2411.15633v4); [GitHub](https://github.com/YZY-stack/Effort-AIGI-Detection)
- **Facial Component Guided Adaptation** (Han, Huang, Hua, Chen; CVPR 2025; arXiv 2404.05583). Frozen CLIP ViT-L/14 with a lightweight spatial-temporal side-network decoder. An InfoNCE "facial component guidance" loss pushes attention onto facial-part regions. The authors claim state-of-the-art cross-dataset results (FF++ → CDF-v2, DFDC and others) and good data and parameter efficiency. I did not retrieve exact numbers. — [CVF open access](https://openaccess.thecvf.com/content/CVPR2025/html/Han_Towards_More_General_Video-based_Deepfake_Detection_through_Facial_Component_Guided_CVPR_2025_paper.html); [arXiv 2404.05583](https://arxiv.org/abs/2404.05583)

**Video and temporal face-forgery methods**
- **LipForensics** (Haliassos, Vougioukas, Petridis, Pantic; CVPR 2021; arXiv 2012.07657). Mouth-region crops go through a lipreading-pretrained spatiotemporal network that looks for semantic mouth-motion irregularities. Reported CDF-v2 AUC is about 82.4 when trained on FF++ **[recalled, not re-verified]**. — [arXiv 2012.07657](https://arxiv.org/abs/2012.07657)
- **RealForensics** (Haliassos, Mira, Petridis, Pantic; CVPR 2022; arXiv 2201.07131). Audio-visual self-supervised pretraining on real talking faces, followed by detection. Reported CDF-v2 AUC is about 86.9 **[recalled, not re-verified]**. — [arXiv 2201.07131](https://arxiv.org/abs/2201.07131)
- **FTCN** (Zheng, Bao, Chen, Zeng, Wen; ICCV 2021; arXiv 2108.06693). Spatial kernel size reduced to 1, so the network must use temporal incoherence; a Transformer sits on top. Reported CDF-v2 AUC is about 86.9 **[recalled, not re-verified]**. — [arXiv 2108.06693](https://arxiv.org/abs/2108.06693)
- **AltFreezing** (Wang, Bao, Zhou, Wang, Li; CVPR 2023; arXiv 2307.08317). A 3D ConvNet whose spatial and temporal weight groups are frozen alternately during training, plus video-level blending augmentation. Reported CDF-v2 AUC is about 89.5 **[recalled, not re-verified]**. — [arXiv 2307.08317](https://arxiv.org/abs/2307.08317)

### Inferences (Tintype applicability, per paper or family)
- **SBI / Face X-ray / FreqBlender (synthetic blends)**
  - *Change:* during training, generate self-blended fakes on the faces in the real photos and frames Tintype already trains on. This needs a landmark detector offline only. Label them with a separate "face-manipulated" target, not the existing "AI-generated" target.
  - *Expected effect:* this is the most direct route from 14% recall towards the 0.5–0.9 range on FF++-style swaps. SBI's own gains are largest on DFDCP (+11.78 points). The figure is not measured for a ViT-S at 224, so treat it as a hypothesis.
  - *Speed:* zero cost at inference if the head shares the trunk.
  - *Cost:* small. No deepfake data or deepfake licences are needed, and blending is a CPU image operation.
  - *Risks:* blending fakes teach "boundary or colour inconsistency around a face". Real photos with beauty filters, retouching, stickers, AR face effects or heavy local tone mapping could then be flagged. Calibrate the face head's threshold on a held-out set of real phone and social-media photos with faces. Do not share the threshold of the generated-image head.
  - *Ordering:* SBI is the best gain per unit cost in this list; Face X-ray needs face pairs; FreqBlender needs an extra trained parsing network.
- **Effort (orthogonal-subspace residual tuning)**
  - *Change:* freeze the top singular components of each ViT-S linear layer and train only the residual subspace, plus new heads, on face-swap and local-edit data.
  - *Expected effect:* new skills are added without erasing what the current checkpoint knows, which protects the precision and recall Tintype already has on its 30 photo sets. The method is backbone-agnostic ("any ViT"), and the same recipe gave state-of-the-art results on both deepfake and AIGI benchmarks.
  - *Speed:* after training the weights can be merged back, so there is no inference cost.
  - *Cost:* a re-training run.
  - *Risks:* the published results use CLIP ViT-L/14, so the benefit at ViT-S scale is unproven. CC BY-NC code means re-implementing rather than copying (the idea is simple to re-implement).
- **CLIP-based face adapters (Han et al. CVPR 2025; DF40's CLIP finding):** they suggest that pretrained semantic features plus face-region guidance generalise. Tintype's ImageNet-21k ViT-S is weaker than CLIP-L, but running CLIP-L would cut throughput by several times. *Not recommended* for the main path. It could serve as an escalation model only on face-containing, uncertain items, the same pattern as the existing second opinion.
- **Temporal methods (LipForensics, RealForensics, FTCN, AltFreezing)**
  - All need face tracking, aligned face or mouth crop sequences and 3D or temporal networks; RealForensics also needs audio pretraining. That breaks Tintype's "one image engine for frames" design and its speed.
  - *Cheap borrowings:* (a) AltFreezing's video-level blending augmentation for training the per-frame face head; (b) replace the mean over 16 frames with a top-k frame mean for the face head, because face swaps can fail visibly on only some frames. The second point is an inference of mine, untested.
- **Deepfake-Eval-2024:** treat FF++/CDF cross-dataset AUCs as optimistic. Tintype should report any new face head on an in-the-wild set (Deepfake-Eval-2024's image split or DF40) as well as FF++, and expect large drops.

### Gaps
- I could not retrieve FreqBlender's or LAA-Net's own result tables, or AUCs for the CVPR 2025 facial-component paper. The SBI, LipForensics, RealForensics, FTCN and AltFreezing AUCs above are from memory.
- I found no study of SBI-style training on a small ViT at 224 px, or of what SBI training does to false positives on non-face or retouched real photos. This is the main precision risk and has to be measured.
- I did not establish whether Deepfake-Eval-2024's images are mostly face swaps or diffusion-generated or edited content, so its relevance to "face swap" specifically is unconfirmed.

---

## Q2. Is a face-crop branch (detect face → crop → specialist head) the right architecture for a fast detector? What is the evidence on face-region views versus the whole image?

### Takeaway
Yes, gated on the presence of a face. The whole face-forgery literature works on detected, aligned face crops, and blending artifacts sit at face scale. Tintype's views are chosen for texture, so they will often miss the face or show it too small. Nothing I found directly compares a face-crop view with a whole-image view inside one fast general detector, so the size of the gain has to be measured.

### Cited Findings
- DeepfakeBench standardises preprocessing on face detection, landmark alignment and cropping before any detector runs. Every benchmarked face detector, from Xception to Effort, consumes face crops. — [DeepfakeBench GitHub](https://github.com/SCLBD/DeepfakeBench)
- The CVPR 2025 facial-component paper improves cross-dataset generalisation by forcing attention onto specific facial parts, which is evidence that face-region focus matters even inside face crops. — [arXiv 2404.05583](https://arxiv.org/abs/2404.05583)
- LipForensics gets its generalisation from mouth-region crops alone. — [arXiv 2012.07657](https://arxiv.org/abs/2012.07657)
- Patch-based classifiers with small receptive fields generalised better than full-image classifiers to out-of-domain synthetic faces (Patch Forensics; Chai, Bau, Lim, Isola; ECCV 2020). — [arXiv 2008.10588](https://arxiv.org/abs/2008.10588)
- On FF++ face-swap images, Tintype's default profile finds 14.3% (0.143) and Community Forensics finds 57.3% (0.573). — repo `docs/BENCHMARK.md` (local file /home/user/tintype/docs/BENCHMARK.md, line 222)

### Inferences
- **Proposed architecture: a "face view" plus a "face head" on the shared trunk.**
  1. Run a tiny face detector on the already-decoded, downscaled image.
  2. For each of the top 1–2 faces above a minimum size, add one 224 crop with margin, resized to 224, as a 7th or 8th view.
  3. Score that view with a separate face-forgery head on the same ViT-S trunk. The head is trained with SBI blends on real faces plus FF++/DF40-style swaps where licences allow.
  4. Images without faces pay only the detector cost. Images with faces pay one or two extra ViT-S forwards, about +17–33% compute over six views. That is my arithmetic from 6 → 7 or 8 views.
- **Why not rely on the existing views:** the "most textured" crop rule favours hair, foliage and fabric over smooth skin, so a native-resolution crop may never cover the swapped face. The fit view shows the face too small whenever it occupies a small part of the frame.
- **The test set may already be face crops.** If Tintype's "FaceForensics++ deepfakes" test images are themselves face crops (I believe the classic CNNDetection/ForenSynths deepfake subset is cropped FF++ faces; not re-verified), then a face detector adds little on that benchmark. The training signal (SBI or swaps) is what would move recall there. Check this locally before investing in the detector.
- **Fast detector options:** I did not verify throughput figures this session. Ultra-light CNN face detectors such as YuNet in OpenCV or BlazeFace-class models are much smaller than ViT-S, so the face gate should cost a small fraction of a ViT-S forward. Benchmark this inside the Rust engine.
- **Risks:**
  - Detector misses (profile faces, small faces) cap recall.
  - Each face-crop forward is another chance of a false positive, so calibrate the face head at a fixed false-positive budget on real face photos.
  - Faces in cartoons, 3D renders and games: the repo found non-camera "real" footage was the main source of video false alarms (docs/RESEARCH.md §4). Include such faces as negatives.

### Gaps
- No paper found that ablates face-crop views against whole-image views inside a general AIGI detector, or measures throughput for such a gated design.
- Throughput numbers for candidate face detectors in a Rust/ONNX setting were not verified.

---

## Q3. Inpainting and local AI-edit detection and localization: which methods and datasets matter, and can a classifier trained with patch-level labels catch local edits?

### Takeaway
Dense localization networks (TruFor, CAT-Net, MVSS-Net, HiFi-Net, IML-ViT) and MLLM systems (SIDA, FakeShield) are too heavy for Tintype's main path. The useful ideas for a fast classifier are two:
1. **Training fakes that are real photos with only a region regenerated** and the background restored, as in B-Free's content augmentation.
2. **Weak region supervision**: crop-level labels from inpainting masks, or a per-token head on the ViT's patch tokens, as in DeCLIP's decoding of frozen features.

Large mask-annotated inpainting datasets released in 2024–2025 (Semi-Truths, SID-Set, SAGI-D, COCO-Inpaint, OpenSDID, BR-Gen) make crop-level labels practical. A key subtlety: latent-diffusion inpainting without paste-back leaves a VAE fingerprint over the whole image, while paste-back edits leave only the region.

### Cited Findings

**Classic manipulation detection and localization (IML) networks**
- **TruFor** (Guillaro, Cozzolino, Sud, Dufour, Verdoliva; CVPR 2023; arXiv 2212.10957). RGB plus the learned Noiseprint++ noise fingerprint, fused by a transformer. It outputs a localization map, a confidence map and an image-level integrity score. — [arXiv 2212.10957](https://arxiv.org/abs/2212.10957). I did not retrieve its image-level numbers this session.
- **CAT-Net** (Kwon, Nam, Yu, Lee, Lee; WACV 2021, extended in IJCV 2022; arXiv 2108.12947). Uses JPEG DCT-coefficient streams to localize splicing and compression inconsistencies. It depends on JPEG artifacts, so evidence weakens on PNG or re-encoded inputs. — [arXiv 2108.12947](https://arxiv.org/abs/2108.12947)
- **MVSS-Net** (Chen, Dong, Zhang, Zhou, Wang; ICCV 2021; arXiv 2104.06832). Multi-view (edge plus BayarConv noise) and multi-scale supervision, designed to keep false alarms on authentic images low. FakeShield lists it as the strongest prior baseline on IMD2020. — [arXiv 2104.06832](https://arxiv.org/abs/2104.06832); [FakeShield arXiv 2410.02761](https://arxiv.org/abs/2410.02761)
- **HiFi-Net** (Guo, Liu, Ren, Zhang, Zhang, Liu; CVPR 2023; arXiv 2303.17111). Hierarchical fine-grained classification of the forgery attribute (including diffusion and GAN partial manipulations) plus localization, on its HiFi-IFDL dataset. — [arXiv 2303.17111](https://arxiv.org/abs/2303.17111)
- **IML-ViT** (Ma et al.; arXiv 2307.14863, 2023). A plain ViT at high resolution (1024 px) with edge supervision for localization. It shows that high-resolution input matters for local traces, at large compute cost. — [arXiv 2307.14863](https://arxiv.org/abs/2307.14863)

**Pretrained-feature and AI-edit localization**
- **DeCLIP** (Smeu, Oneata, Oneata; WACV 2025; arXiv 2409.08849). Frozen CLIP features decoded by a "reasonably large" convolutional decoder into a localization map. It generalises better than earlier localizers on Dolos, COCO inpainting and AutoSplice, against Patch Forensics, PSCC-Net and CAT-Net. Key point: it can localize latent-diffusion inpainting where "the entire image is affected by the fingerprint of the generator". Code: bit-ml/DeCLIP. — [CVF open access](https://openaccess.thecvf.com/content/WACV2025/html/Smeu_DeCLIP_Decoding_CLIP_Representations_for_Deepfake_Localization_WACV_2025_paper.html); [arXiv 2409.08849](https://arxiv.org/abs/2409.08849)
- **B-Free** (Guillaro, Zingarini, Usman, Sud, Cozzolino, Verdoliva; arXiv 2412.17671, v2 Apr 2025; reported as CVPR 2025).
  - *Training reals:* 51,517 CC-licensed COCO reals.
  - *Training fakes:* self-conditioned SD 2.1-inpainting reconstructions with an empty mask (whole image regenerated, same content), plus content augmentation. An object is replaced by inpainting, using COCO masks for same-category swaps or rectangular boxes for different-category swaps. Each image also gets a version with the original background pixels restored. That gives six fakes per real, 309,102 fakes in total.
  - *Model and inference:* DINOv2+reg, no resizing, 504×504 training crops. At test time, multiple 504 crops with averaged logits; no max pooling.
  - *Ablation (average balanced accuracy over its test sets):* "reconstructed + inpainted" 83.9, "self-conditioned + inpainted" 92.2, "inpainted++" 96.4 (AUC 99.3).
  - *Not covered:* no separate local-edit test set and no face-swap evaluation.
  — [arXiv HTML 2412.17671v2](https://arxiv.org/html/2412.17671v2)
- **SIDA** (Huang et al.; CVPR 2025; arXiv 2412.04292). A large multimodal model with detection and segmentation tokens. It is introduced with **SID-Set**: 300K images split evenly between real, fully synthetic and tampered. SIDA-13B reaches 93.6% accuracy and 93.5% F1 on SID-Set and is reported robust to JPEG and noise. — [arXiv 2412.04292](https://arxiv.org/abs/2412.04292)
- **FakeShield** (Xu et al.; ICLR 2025; arXiv 2410.02761). An MLLM-based explainable detector and localizer: a domain-tag-guided detection and explanation module plus a multimodal localization module. It is trained on MMTD-Set (existing IFDL sets with GPT-4o descriptions) and compared with SPAN, MantraNet, HiFi-Net, PSCC-Net, CAT-Net and MVSS-Net. A secondary summary claims +0.08 accuracy and +0.05 F1 over MVSS-Net on IMD2020, which I did not verify. — [arXiv 2410.02761](https://arxiv.org/abs/2410.02761); [ICLR poster](https://iclr.cc/virtual/2025/poster/28315)
- **OpenSDI / OpenSDID** (Wang, Huang, Hong; CVPR 2025; arXiv 2503.19653; DOI 10.1109/CVPR52734.2025.00405). Open-world benchmark of global and local diffusion edits, simulated with VLM-driven pipelines, for both detection and localization. Its MaskCLIP (CLIP+MAE) model improves over prior state of the art by 14.23% relative IoU and 2.05% relative accuracy. Code and data: iamwangyabin/OpenSDI. — [CVF open access](https://openaccess.thecvf.com/content/CVPR2025/html/Wang_OpenSDI_Spotting_Diffusion-Generated_Images_in_the_Open_World_CVPR_2025_paper.html); [arXiv 2503.19653](https://arxiv.org/abs/2503.19653)
- **Zooming In on Fakes / BR-Gen / NFA-ViT** (Cai et al.; AAAI 2026 per the AAAI OJS listing; arXiv 2504.11922). BR-Gen has 150,000 locally forged images, with scene-level edits (sky, ground) as well as object-level ones. NFA-ViT uses noise fingerprints to find candidate edited regions and then "amplifies" those traces across the image through attention, so a global classifier sees them. Code: clpbc/BR-Gen. — [AAAI OJS](https://ojs.aaai.org/index.php/AAAI/article/view/37240); [arXiv 2504.11922](https://arxiv.org/abs/2504.11922)

**Inpainting datasets and benchmarks**
- **Semi-Truths** (Pal, Kruk, et al.; NeurIPS 2024 Datasets & Benchmarks; arXiv 2411.07472). Real images edited by diffusion inpainting and prompt-based editing to varying extents, with metadata on the size and semantic magnitude of each change. Six state-of-the-art detectors showed sensitivity that varies with perturbation level, data distribution and edit method. Counts differ between sources:
  - arXiv abstract: 27,600 reals, 223,400 masks, 1,472,700 edited images.
  - NeurIPS page: 1,329,155 edited images.
  - Hugging Face README: 245,300 masks, 850,200 edited images.
  — [arXiv 2411.07472](https://arxiv.org/abs/2411.07472); [NeurIPS poster](https://neurips.cc/virtual/2024/poster/97591); [HF README](https://huggingface.co/datasets/semi-truths/Semi-Truths/blob/main/README.md)
- **SAGI / SAGI-D** (Giakoumoglou et al.; ICCV 2025; arXiv 2502.06593). 95,839 inpainted images from 78,684 originals (MS-COCO, RAISE, OpenImages), each with its original, mask and prompt. Retraining forensic detectors on SAGI-D raises in-domain localization IoU by 37.4% and out-of-domain IoU by 26.1% on average. Human accuracy at spotting the inpainted images falls from 74% to 35%. — [CVF open access](https://openaccess.thecvf.com/content/ICCV2025/html/Giakoumoglou_SAGI_Semantically_Aligned_and_Uncertainty_Guided_AI_Image_Inpainting_ICCV_2025_paper.html); [arXiv 2502.06593](https://arxiv.org/abs/2502.06593)
- **COCO-Inpaint** (Yan, Hong, et al.; ACM MM 2025; arXiv 2504.18361). 258,266 inpainted COCO images from six inpainting models (including SD1.5-Inpainting, SDXL-Inpainting and SD3.5), four mask strategies and optional text guidance. Training on it improves detection and segmentation of inpainting. — [arXiv 2504.18361](https://arxiv.org/abs/2504.18361)

**Patch-based and crop-based detection (whole-image task, local evidence)**
- **TextureCrop** (Konstantinidou, Koutlis, Papadopoulos; arXiv 2407.15500, CERTH). A sliding window keeps only high-texture patches as a pre-processing step for any detector. Across detectors it gains +3.4 to +12.1 points balanced accuracy and +6.1 to +14.9 points AUC on more than 10,000 high-resolution images (Synthbuster, ForenSynths, TWIGMA, RAISE, OpenImages). Figures differ between arXiv versions. — [arXiv 2407.15500](https://arxiv.org/abs/2407.15500). This is the closest published analogue of Tintype's "most textured crops" views.
- **Look Closer: Patch-wise Supervision** (Zhang, Wu, Liu, Cao; arXiv 2609.37937, Sep 2026). Each crop inherits its image's label, and patch probabilities are averaged. ResNet-50 GenImage mean accuracy rises from 75.06 (whole image) to 95.40. The authors state this targets fully generated images and is "not a local manipulation annotation"; partial edits are out of scope. Some checkpoints were selected on target data. — [arXiv 2609.37937](https://arxiv.org/html/2609.37937v1)

### Inferences (Tintype applicability)
- **B-Free-style content augmentation (highest value)**
  - *Change:* add to training real photos where one object or region is regenerated by an inpainting model and the background pixels are restored. Also add whole-image self-conditioned reconstructions, which teach "same content, different pipeline".
  - *Why it suits Tintype:* B-Free's recipe works with no resizing and native-resolution crops, like Tintype's native views. B-Free was built on 51k CC-licensed COCO reals, which is cheap to replicate with an open inpainting model.
  - *Expected effect:* recall on inpainted and partially edited images should rise, and B-Free's ablation suggests it also improves full-generator generalisation (83.9 → 96.4 bAcc as augmentations are added).
  - *Speed:* no change.
  - *Risks:*
    - Labelling a background-restored image "fake" while most crops are real pixels creates label noise for crop-level training. Use mask-aware crop labels (next bullet).
    - Detector learns "inpainting model X" rather than "local edit": use several inpainting models (COCO-Inpaint's six; SAGI-D; Semi-Truths).
- **Crop-level labels from masks (multiple-instance learning)**
  - *Change:* during training, label a 224 native crop "edited" if its overlap with the mask exceeds a threshold, and "real" only if it lies entirely outside the mask **and** the background was restored. Otherwise mark it "unknown" and exclude it.
  - *Data:* SAGI-D, COCO-Inpaint, Semi-Truths, BR-Gen and OpenSDID all ship masks.
  - *Why the background caveat matters:* DeCLIP's point means that for pipelines without paste-back, *all* crops carry the VAE fingerprint and are legitimately "AI". The existing generated-image head may already catch those.
  - *Expected effect:* this turns Tintype's native crops into local-evidence detectors and enables max/top-k pooling (Q4).
  - *Speed:* none at inference.
  - *Cost:* dataset download and preprocessing.
- **Per-token localization head (DeCLIP's idea, scaled down)**
  - *Change:* a linear or small MLP head on the 14×14 ViT-S patch tokens of each 224 view, supervised by downsampled masks. It gives a coarse heatmap per view, which can be stitched into an image-level map, at negligible cost (one matmul over 196 tokens).
  - *Use:* the heatmap's top-k token mean is a second local-edit score, and the map is a "where" explanation for the CLI and API.
  - *Risk:* DeCLIP needed a "reasonably large" convolutional decoder on CLIP for good masks, so a linear token head on ViT-S will be coarse. Treat it as evidence, not segmentation.
- **NFA-ViT's "forgery amplification":** this addresses the same problem as max pooling, keeping a small edited region from being diluted, but inside the network. Its noise-fingerprint module adds compute, so it is lower priority than pooling changes.
- **Full localization nets and MLLMs**
  - TruFor, IML-ViT (1024 px), CAT-Net, MVSS-Net and HiFi-Net cost several times a ViT-S/224 forward, and often need full-resolution input.
  - SIDA (7B/13B) and FakeShield are LLM-scale.
  - None fits a 530 photos/s path. They could serve as an offline second stage for flagged face or local-edit items, or as teachers for distillation. TruFor's licence is, I believe, research-only (unverified).
  - CAT-Net's JPEG dependence makes it brittle on messenger re-encodes, which is already Tintype's weak spot.
- **Crop selection bias**
  - Tintype picks the four *most textured* crops, as TextureCrop does. Inpainted regions are often smoother and blurrier than camera texture, so the edited region may never be sampled (an untested hypothesis).
  - For a local-edit mode, use a regular grid of native crops, e.g. 3×3 or 4×4, or add one or two *least*-textured crops.
  - *Cost (arithmetic):* six views → 16 views is about 2.7× compute, so roughly 200 photos/s GPU instead of 530.
  - A cheaper variant runs the dense grid only when the first pass is uncertain, reusing the existing second-opinion cascade pattern.

### Gaps
- I could not verify TruFor's, CAT-Net's or IML-ViT's headline numbers this session, or their licences.
- I did not confirm whether B-Free released its inpainted training set, or the licences of SID-Set, SAGI-D, COCO-Inpaint, OpenSDID, BR-Gen or Semi-Truths.
- I found no published fast classifier trained with mask-derived crop labels and evaluated for image-level local-edit detection at a fixed low false-positive rate. The closest work averages crop scores and excludes partial edits (Look Closer).

---

## Q4. Aggregation: does max/top-k pooling over patch scores raise recall for partial manipulations, and what does it do to false positives?

### Takeaway
For fully generated images, averaging beats max pooling at a fixed threshold, because max pooling admits single-patch false alarms. Max or top-k pooling only makes sense for a head trained with crop-level local labels, and only with its own threshold calibrated to a false-positive budget. No head-to-head study of max versus mean on local-edit benchmarks with false-positive rates was found.

### Cited Findings
- **LaDeDa / Tiny-LaDeDa** (Cavia, Horwitz, Reiss, Hoshen; arXiv 2406.09398, 2024). A 9×9-receptive-field patch classifier. Pooling ablation (accuracy / AP):

  | Dataset | Max pooling | Average pooling |
  |---|---|---|
  | ForenSynths | 90.1 / 99.1 | 92.7 / 98.9 |
  | UFD | 88.9 / 97.0 | 96.3 / 98.8 |
  | WildRF | 84.9 / 92.4 | 85.7 / 93.7 |

  Average pooling wins on accuracy everywhere, and max only gets slightly higher AP on ForenSynths. Tiny-LaDeDa uses about 0.057 GFLOPs (vs 21.23) and 0.0013 M parameters, with 82.5% accuracy and 91.6 AP on WildRF. WildRF is about 5,000 Reddit, X and Facebook images. — [arXiv 2406.09398](https://arxiv.org/html/2406.09398v1)
- A full-image forgery-detection framework (arXiv 1909.06751, 2019) argues that average pooling suits evidence spread over the image, while max (or min) pooling suits localized evidence. — [arXiv 1909.06751](https://arxiv.org/pdf/1909.06751) (via search summary)
- **MDMF** ("Micro-Defects Expose Macro-Fakes", arXiv 2605.09296, 2026) says average pooling dilutes sparse local artifacts. It reports beating global pooling with mean, max and top-k aggregators alike, and concludes the operator itself is not the key factor. **PatchHead** (arXiv 2608.09223, 2026) finds that *learnable* aggregation over patch tokens, not just access to them, is needed for cross-domain generalisation. — [MDMF](https://arxiv.org/html/2605.09296v1); [PatchHead](https://arxiv.org/html/2608.09223v1) (both seen only through search summaries this session)
- Patch Forensics' code aggregates patch outputs by averaging logits before or after softmax, or by patch votes. — [GitHub chail/patch-forensics](https://www.github.com/chail/patch-forensics)
- B-Free averages logits over multiple 504 crops at test time, and Look Closer averages patch probabilities. — [B-Free](https://arxiv.org/html/2412.17671v2); [Look Closer](https://arxiv.org/html/2609.37937v1)
- A 2026 forensic-agent paper describes a local-manipulation detector producing saturated heatmaps on a real face photo, which had to be overruled as false positives. — [arXiv 2609.24359](https://arxiv.org/pdf/2609.24359) (search summary)

### Inferences
- **Keep the mean** of six views for the existing "fully generated" head. The LaDeDa ablation is direct evidence that switching that head to max would cost accuracy, mainly through false positives.
- **For a new local-edit head, use top-k (k = 1–2) over native crops**, plus the top-k of per-token scores, with a threshold set on real photos for a target false-positive rate.
  - *False-positive arithmetic (mine):* if per-crop false-positive rates were independent at p, the image-level rate under max over n crops is 1 − (1 − p)^n. For n = 5 and an image-level budget of 1%, each crop must run at p ≈ 0.2%. Crops of one image are correlated, so the real inflation is smaller, but it must be measured.
- **Prefer a learned combiner** (PatchHead/MDMF direction): a tiny MLP or logistic regression over sorted per-view logits from the generated, face and local heads (max, mean, std). It costs nothing and can be fitted on a calibration set. It also cannot do worse than the mean on the existing benchmarks if it is regularised and evaluated on held-out sets.
- **Video:** the analogue for face swaps is a top-k frame mean (e.g. the 4 highest of 16 frames) for the face head only. The current mean over 16 frames dilutes swaps that are visible only in some frames.

### Gaps
- No primary study measured max versus mean versus top-k on a *local-edit* benchmark (SAGI-D, COCO-Inpaint, Semi-Truths) with image-level false-positive rates on real photos. This is a direct experiment Tintype can run cheaply by re-aggregating existing per-view logits.
- MDMF and PatchHead claims come from search snippets, not full-text reads.

---

## Q5. Which datasets are usable for training (size, licence)?

### Takeaway
For face swaps, the cheapest and most licence-friendly path is SBI on real faces Tintype already has rights to, with DF40/FF++ for evaluation. The deepfake corpora are research-only. For local edits, several large mask-annotated sets exist (Semi-Truths, SID-Set, SAGI-D, COCO-Inpaint, OpenSDID, BR-Gen), but their licences were not confirmed this session. Generating B-Free-style inpaints from CC-licensed COCO images is the cleanest option for a project released under MIT/Apache.

### Cited Findings

| Dataset | Size / content | Licence / access | Source |
|---|---|---|---|
| DeepfakeBench (code and preprocessed index) | 36 detectors, standard face preprocessing | CC BY-NC 4.0 | [GitHub](https://github.com/SCLBD/DeepfakeBench) |
| DF40 | 40 methods (swap / reenactment / full synthesis / editing), about 10× FF++ | Released on GitHub (YZY-stack/DF40); licence not confirmed | [NeurIPS 2024](https://proceedings.neurips.cc/paper_files/paper/2024/hash/34239f60eca7ce9bee5280aaf81362d8-Abstract.html) |
| Deepfake-Eval-2024 | 1,975 images, 45 h video, 56.5 h audio, in the wild, 2024 | Access terms not confirmed | [arXiv 2503.02857](https://arxiv.org/abs/2503.02857) |
| Semi-Truths | 27.6k reals; 0.85–1.47 M edited images depending on version, with masks and edit-magnitude metadata | Hugging Face (semi-truths/Semi-Truths); licence not confirmed | [arXiv 2411.07472](https://arxiv.org/abs/2411.07472); [HF](https://huggingface.co/datasets/semi-truths/Semi-Truths/blob/main/README.md) |
| SID-Set | 300K images: 100K real, 100K full synthetic, 100K tampered with masks | Release and licence not confirmed | [arXiv 2412.04292](https://arxiv.org/abs/2412.04292) |
| SAGI-D | 95,839 inpainted images from 78,684 originals (COCO, RAISE, OpenImages); masks and prompts | mever-team.github.io/SAGI; licence not confirmed | [arXiv 2502.06593](https://arxiv.org/abs/2502.06593) |
| COCO-Inpaint | 258,266 inpainted images, 6 models, 4 mask strategies | Licence not confirmed | [arXiv 2504.18361](https://arxiv.org/abs/2504.18361) |
| OpenSDID | Global and local diffusion edits with masks | GitHub iamwangyabin/OpenSDI; licence not confirmed | [arXiv 2503.19653](https://arxiv.org/abs/2503.19653) |
| BR-Gen | 150,000 local forgeries including scene-level (sky, ground) edits | GitHub clpbc/BR-Gen; licence not confirmed | [arXiv 2504.11922](https://arxiv.org/abs/2504.11922) |
| B-Free training recipe | 51,517 CC-licensed COCO reals → 309,102 SD 2.1 inpaint and reconstruction fakes | Recipe in paper; data release not confirmed | [arXiv 2412.17671](https://arxiv.org/html/2412.17671v2) |

### Inferences
- **Training priority by licence risk:**
  1. SBI on Tintype's own real photos and frames; no external deepfake data.
  2. Self-generated B-Free-style inpaints on CC-licensed images with open-weight inpainting models.
  3. Research-licensed sets (DF40, FF++, SAGI-D, COCO-Inpaint, Semi-Truths, BR-Gen) for evaluation, or for training only if their licences allow redistribution of derived weights.
- **Evaluation:** one in-the-wild set (Deepfake-Eval-2024 images) and one mask-annotated local-edit set kept fully held out (e.g. SAGI-D's out-of-domain split), matching Tintype's blind-batch protocol.

### Gaps
- Licences for FF++, Celeb-DF, DFDC, DF40, SID-Set, SAGI-D, COCO-Inpaint, OpenSDID, BR-Gen and Semi-Truths were not verified this session. My recollection is that FF++, Celeb-DF and DFDC are research or non-commercial only; confirm before use.

---

## Q6. Ranked shortlist: the most promising ideas from this area for Tintype

### Takeaway
Add new *heads* on the shared ViT-S trunk rather than changing the existing generated-image head. Train them with cheap synthetic partial fakes (SBI faces, B-Free-style regional inpaints with mask-aware crop labels). Aggregate them with top-k and a separately calibrated threshold. Use orthogonal-subspace (Effort-style) adaptation so existing precision and recall do not regress.

### Cited Findings
Each item builds on the sources cited in Q1–Q5:
- SBI: [arXiv 2204.08376](https://arxiv.org/abs/2204.08376)
- Effort: [arXiv 2411.15633](https://arxiv.org/html/2411.15633v4)
- B-Free: [arXiv 2412.17671](https://arxiv.org/html/2412.17671v2)
- LaDeDa pooling ablation: [arXiv 2406.09398](https://arxiv.org/html/2406.09398v1)
- DeCLIP: [arXiv 2409.08849](https://arxiv.org/abs/2409.08849)
- Mask datasets: [SAGI-D](https://arxiv.org/abs/2502.06593), [COCO-Inpaint](https://arxiv.org/abs/2504.18361), [Semi-Truths](https://arxiv.org/abs/2411.07472)

### Inferences (ranked)
1. **Local-edit head with mask-aware crop labels plus B-Free-style regional inpainting fakes, aggregated by top-k over native crops at a calibrated false-positive rate.**
   - *Change:* new head; training data from 51k-scale self-generated inpaints with background restored, plus SAGI-D/COCO-Inpaint masks; top-1 or top-2 over the five native crops (plus an optional grid in uncertain cases).
   - *Expected effect:* opens recall on inpainting and AI edits, which is currently out of scope; the existing head's precision is unchanged because the heads have separate thresholds.
   - *Speed:* 0% on the default path (same views); +2–3× only for an opt-in dense grid.
   - *Main risk:* the crop-level false-positive rate inflates under max pooling, so calibrate at 1% and measure on social-media reals.
   - *Cheapest first experiment:* re-aggregate the existing six per-view logits with max or top-2 on SAGI-D/COCO-Inpaint and on a real set. This shows the headroom before any training.
2. **SBI self-blended face augmentation training a face head, plus a gated face-crop view.**
   - *Change:* SBI on real faces, offline landmarks; a tiny face detector at inference that adds one or two face-crop views only when a face is present.
   - *Expected effect:* the most direct fix for the 14% (default) / 48% (second opinion) face-swap recall; SBI was the strongest pre-2024 blending method (+11.78 points on DFDCP).
   - *Speed:* about 0% for face-free images; +17–33% for images with faces.
   - *Risks:* false positives on retouched or filtered selfies; calibrate on such reals.
   - *Licence:* SBI needs no deepfake data, so it is clean.
3. **Effort-style orthogonal-subspace residual tuning to add heads 1–2 without forgetting.**
   - *Change:* freeze the top singular directions of the ViT-S weights and tune the residual.
   - *Expected effect:* protects current blind-batch results; on CLIP-L it lifted deepfake cross-dataset AUC to 0.917 on average (SBI 0.821) and also set state of the art on AIGI benchmarks.
   - *Speed:* zero inference cost once merged.
   - *Risk:* the benefit at ViT-S scale is unproven.
4. **Per-token patch head for coarse localization (DeCLIP/PatchHead idea), plus a learned score combiner.**
   - *Change:* a linear head on the 196 patch tokens per view, supervised by downsampled masks; a small calibrated combiner over sorted per-view and per-head logits.
   - *Expected effect:* a "where" heatmap for users, and a second sparse-evidence signal.
   - *Speed:* negligible.
   - *Risk:* heatmaps on real photos can look alarming; show them only when the image-level verdict fires.
5. **Video face-swap path: per-frame face head plus top-k frame pooling, with AltFreezing-style video-level blending augmentation.**
   - *Change:* keep the per-frame engine; for the face head only, replace the mean over 16 frames with the mean of the top 4.
   - *Expected effect:* recall on clips where the swap is visible only in some frames.
   - *Speed:* the cost of face-crop views per frame.
   - *Avoid:* LipForensics, RealForensics, FTCN and AltFreezing at inference. They need face tracks, 3D networks or audio, which conflicts with the shared image engine and speed targets.
   - *Evaluate on:* DF40 and Deepfake-Eval-2024, expecting large drops compared with FF++/CDF.

### Gaps
- None of these combinations has been tested at ViT-S/224 scale in the literature I found. Every expected effect above is an informed hypothesis, to be confirmed with Tintype's preregistered blind-batch protocol.
