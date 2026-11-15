# Tintype codebase fact sheet (for the literature-review report)

What Tintype does today, checked against the source and docs on 2026-11-13. Each claim gives `file:line` evidence. Paths are relative to the repo root (`/home/user/tintype`). **TRUE / FALSE / PARTLY** marks how an assumption in research notes 01–08 holds up. The table at the end lists every note assumption that needs correcting.

Sign conventions used below:
- **raw logit** is the network's mean view logit.
- **calibrated logit** is `scale * raw + bias`. The decision threshold is calibrated logit 0, i.e. probability 0.5 (`src/detector.rs:980-983`).

---

## 1. Training augmentations

All of them live in `augment()` (`src/data.rs:57-104`). The defaults are in `AugConfig::default()` (`src/data.rs:32-48`). The CLI overrides only `p_native` and `p_messenger` (`src/main.rs:647`).

| Op | Exact behaviour | Evidence |
|---|---|---|
| Messenger round trip | With probability `p_messenger`: cap the long side at U(960, 1700) px (only if the image is larger), resizing with bicubic or bilinear at 50/50, then JPEG at quality U(55, 90). When this branch fires, the JPEG-pre step is skipped (`if … else if`). | `src/data.rs:59-71` |
| `p_messenger` value | The struct default is 0.0, but the CLI default is **0.25**. Phase 1 ran with `--p-messenger 0`, phases 2, 3 and 6 (photo) used 0.25, and the video phases 4–5 used **0.1**. | `src/data.rs:45`; `src/main.rs:322`; `README.md:462-463, 488-489`; `docs/DATA.md:87-88` |
| JPEG before the geometric step | p = 0.25, quality U(40, 98). Skipped for images above 6 MP. | `src/data.rs:38, 40, 72-78` |
| Resampling kernel | p = 0.3 of a kernel drawn uniformly from {Nearest, Box, Bilinear, Bicubic (Catmull-Rom), Lanczos3}; otherwise bilinear. Bilinear is in the drawn set, so a non-bilinear kernel is used about 24% of the time. | `src/data.rs:79`; `src/image_io.rs:19-36` |
| View type | p_native = 0.5: a **random-position** native-resolution 224 crop, not a texture-ranked one. Otherwise a "fit" view: resize the short side to 256, then a random 224 crop. | `src/data.rs:80-94`; `src/main.rs:310` |
| Scale jitter ("random crop scale") | p = 0.3. For native crops, rescale by U(0.5, 1.25). For fit views, short side U(224, 320) instead of 256. There is no RandomResizedCrop-style area/aspect sampling. | `src/data.rs:82-86, 92` |
| JPEG after the geometric step | p = 0.25, quality U(40, 98). | `src/data.rs:39, 95-99` |
| Horizontal flip | p = 0.5. | `src/data.rs:43, 100-102` |
| Video frames | One random frame per draw. With p = 0.5, a ~0.5 s segment around it is first re-encoded to H.264 at log-uniform 0.012–0.25 bits/pixel and scale 1.0 (50%) or U(0.5, 1.0). A frame past the first of the segment is used. The image augmentations above then apply to the frame. | `src/data.rs:44, 160-161`; `src/video.rs:172-189` |
| JPEG encoder | `image` crate 0.25.10 `JpegEncoder`. Its chroma subsampling was not verified locally. | `src/image_io.rs:153-158`; `Cargo.lock:2162-2163` |

**Missing (TRUE that Tintype lacks them):** blur, noise, sharpening, WebP/AVIF/JPEG 2000, neural codecs, colour/gamma jitter, grayscale, rotation, cutout/patch masking, mixup and watermarks. `augment()` contains only the ops above (`src/data.rs:57-104`). Note 03:125 states this correctly.

**Same treatment for real and fake: TRUE for the synthetic augmentation.** `augment()` takes no label (`src/data.rs:57`), and the loader draws the label and then calls the same function (`src/data.rs:158-167`). The processing baked into the source files (platform recompression, format, size) is not equalised; see §2.

**Training views are not inference views.** Each training sample is a single random view (fit or random native crop). The six texture-ranked views are used **only at inference**. The validation set during training scores only fit + centre native (2 views) and the clip's middle frame (`src/train_cmd.rs:122`; `build_val` at `src/train_cmd.rs:45-74`). Note 03's "training is six-view inference with texture-ranked native crops" is therefore **FALSE**. Because the random native crops cover flat regions as well, the network has seen low-texture crops (note 07:18 is correct on this point).

## 2. openfake-reddit and other per-class imbalances

- **Counts: TRUE.** `openfake-reddit` is OpenFake `supplement/reddit/2026-11` shards 0–4, with **3.0 k real vs 15.8 k AI** "posted to Reddit" (`docs/DATA.md:63`). It is in phase 1, and phase 6 retrains on "every store of phases 1–3" (`docs/DATA.md:136-137`).
- **How it enters the sampler (important nuance).** Every AI image in that store gets the single generator key `reddit-wild` (`src/plans.rs:340`). Every real gets `real:openfake-reddit` (`src/ingest.rs:167, 218`). The sampler draws the two classes 50/50, and within a class draws a key with weight `count^0.5` (`src/data.rs:106-135`; `docs/DATA.md:153-154`).
  - The AI class has about 1,200 generator keys (`README.md:293-297`). `reddit-wild` weighs about √15,800 ≈ 126 among them, i.e. roughly 1% of AI draws.
  - The real class has about 16 real-source keys (`README.md:298`). `real:openfake-reddit` weighs about √3,000 ≈ 55 among them, i.e. a few percent of real draws.
  - Estimate: the shares were not computed, because the training manifests are not in the repo. Even so, the raw 5:1 AI:real count does **not** become a 5:1 sampling ratio. Per draw, Reddit-processed reals are probably *over*-represented relative to Reddit fakes.
  - Notes 03 and 04 argue that "15.8 k Reddit-processed fakes vs 3.0 k reals" teaches "platform look ⇒ AI". That argument is **unsupported as stated**; it would need the per-key sampling shares.
- Other documented per-class source asymmetries:
  - So-Fake-Set has 3.0 k social-media reals vs 4.3 k fakes (`docs/DATA.md:95`).
  - Many AI-only stores (frontier 18.1 k, nano-banana, stylegan3, tpdne, ip-adapter) and real-only stores (openfake-test-reals, celeba-hq) (`docs/DATA.md:62-70, 81-83, 138-145`).
  - Within a store such as Defactify, reals are COCO and fakes are generator outputs (`docs/DATA.md:66`).
- **Per-class format, JPEG or size statistics for the training stores are not documented anywhere.** `docs/DATA.md` records only counts. The training manifests are not frozen or recorded (`docs/DATA.md:353-357`).
- The only explicit format confound documented is on the *evaluation* side: AIGIBench CommunityAI has every real as JPEG and every AI image as PNG (`docs/DATA.md:387-388`). EVIDENCE notes that several AIGIBench sets correlate format with class (`docs/EVIDENCE.md:14`). Arena originals have a label/format imbalance (`docs/EVIDENCE.md:126`).
- Deduplication is byte-identical only (`docs/DATA.md:147-150`). TRUE.
- Plans store image bytes as downloaded, never re-encoded (`docs/DATA.md:15-16`).

## 3. Second-opinion fusion

### Photo (`--photo-second-opinion`)

- Stored band `{lo: -7, hi: 0.5, weight: 0.5, threshold: 0}` (`models/tintype.json`, `second_opinion`).
- A photo is sent when its **calibrated** default-profile logit z1 is in (−7, 0.5) (`src/detector.rs:220-236`). In raw logits that is (−8.85, 2.45), against a raw threshold of 1.692.
- Output: `p = sigmoid(0.5·(0.5·z1 + z2 − 0))` (`src/detector.rs:234`). z2 is the Community Forensics 384 px **raw** logit, from its own released preprocessing (`src/detector.rs:243-251`; `ModelSpec::commfor(384)`).
- Photos outside the band keep the first-stage probability.
- **The combined threshold is not recalibrated to an FPR target: TRUE.**
  - The band, weight and threshold were chosen by grid search over 48 seen photo sets. The objective was the most sets with precision and recall both at or above Community Forensics, with at most half of all photos sent on, and ties broken by pooled recall (`docs/PREREGISTRATION.md:185-197`).
  - In-sample, 48.8% of photos go to stage 2: 91% of reals and 15% of AI images (`docs/PREREGISTRATION.md:196-197`).
  - On blind batch 7, real photos flagged went from 130 (1.2%) to 329 (3.1%), and recall from 0.897 to 0.935 (`docs/BENCHMARK.md:181-186`). Stage 2 added 749 flags and cleared 84 (`docs/BENCHMARK.md:233-235`).
  - In the scoring path, the first-stage logits are those of the early-exit views (`src/eval.rs:89-97`).
- Note 07's formula `sigmoid(0.5·(0.5·z1 + z2))` is **TRUE**.
- Note 07's proposed baseline, "Tintype alone at ~3.1% FPR", has not been computed in the repo. The nearest profile is `--sensitive` (5% calibrated): 0.963 / 0.960 on batch 7 (`README.md:315-318`).

### Video (`--second-opinion`)

- Stored band `{lo: -1, hi: 4, threshold: 1}` (`models/tintype-video.json`, `video.second_opinion`).
- Band and rule are applied to the **raw** clip logit, i.e. the mean over frames of the per-frame mean view logit (`src/eval.rs:170-182`; `src/detector.rs:193-209`). For comparison, the video profile's raw threshold is 1.621 (`docs/PREREGISTRATION.md:51-53`).
- Output: `sigmoid(0.5·(z1 + z2 − 1))`, where z2 is WaveRep's clip logit (mean of the first 64 consecutive frames, 504 px centre crop) (`src/detector.rs:206`; `src/eval.rs:134-140`).
- The band and threshold were chosen on 8 earlier development sets, not recalibrated to an FPR target (`docs/PREREGISTRATION.md:54-61`).
- Turning on the second opinion disables video early exit (`src/eval.rs:155`).
- About 11–13.5% of clips go to stage 2 (`docs/PREREGISTRATION.md:61`; `docs/BENCHMARK.md:833`).

## 4. The six views: selection, order, aggregation, early exit, export

- **Views** (`thorough_views`, `src/detector.rs:47-51`; `models/tintype.json`):
  - view 0: fit (short side 256, centre 224);
  - view 1: native centre 224;
  - views 2–5: the four most textured native 224 windows (`native_rich` rank 0–3).
- The fast profile uses fit + rank-0 rich (`src/detector.rs:54-56`).
- **Texture measure:** the mean absolute luma gradient (|dx| + |dy| of 8-bit luma 77/150/29) on a 2×-subsampled grid, summed per window through an integral image.
  - Candidates lie on a half-crop (112 px) grid and are picked greedily, richest first, with no overlap while possible. The spacing then relaxes to half a crop, then 0 (`src/image_io.rs:291-353`).
  - Images whose short side is under 224 are upscaled and centre-cropped instead (`src/image_io.rs:394-397`).
  - Nothing selects low-texture crops. Notes 01 and 07 are **TRUE** that no "least-textured" view exists.
- **Aggregation: plain mean of raw view logits**, then `p = sigmoid(scale·mean + bias)` (`src/detector.rs:975-983`). TRUE.
- **Early exit (photos):**
  - Order `[2, 0, 1, 3, 4, 5]`: rich-0 first, then **fit second**, then centre, then rich-1..3 (`src/detector.rs:94-106`; `models/tintype.json` `early_exit`). Note 07's "fit view last" is a proposal; currently it is second.
  - After *k* views the mean is compared with the **active profile's raw threshold** `-bias/scale` (`src/detector.rs:121-125, 687-689`). Margins are 6 / 3 / 2 / 1.5 / 1 raw-logit units after 1–5 views (≈4.0 / 2.0 / 1.3 / 1.0 / 0.66 in calibrated units for the default profile).
  - The margins were fitted on 44,336 development images so that pooled precision and recall stayed within 0.001 of the previous schedule (`docs/BENCHMARK.md:883-893`).
  - The verdict uses only the views that were looked at (`src/detector.rs:146-157`).
  - Average views per image: 1.71–1.93 on development data and batch 6 (`docs/BENCHMARK.md:888-890`); 1.7 on batch 7 (`docs/BENCHMARK.md:250`).
  - The same schedule also runs for `--strict` and `--sensitive`, each relative to its own threshold. Fast and all-views turn it off (`src/eval.rs:74-77`).
  - On the GPU, the batches are filled across stages (`src/detector.rs:855-958`). For one-image latency on the GPU, all six views go in one batch and the rule is applied afterwards (`src/main.rs:567-572`).
- **Early exit (video):** schedule over frames, with all six views of each frame always scored.
  - Order `[0, 7, 3, 5, 1, 2, 4, 6, 8, …]`, checks after 2 / 4 / 8 frames with margins 4 / 3 / 3 (`src/detector.rs:112-117`; `models/tintype-video.json`).
  - Decoding stops at the half-way mark unless the clip is still undecided (`src/detector.rs:774-850`). About 4.8 frames per clip on the blind batch (`docs/BENCHMARK.md:871-881`).
  - A clip can therefore be cleared as "real" from first-half frames only. Note 05's concern on this point is **TRUE**.
  - `docs/BENCHMARK.md:839` ("early exit applies to photos only") predates this change.
- **Are per-view logits stored or exportable: PARTLY.**
  - `tintype score` writes `path label generator dataset prob logits…` (`src/eval.rs:117`). With early exit on, only the views looked at are written, in looking order. With `--all-views` (or `--views`), all six are written in profile order (`src/eval.rs:59-77`).
  - `finalize` scores its calibration data with all views (`src/eval.rs:582`).
  - `apply_profile` already re-derives early exit and calibration from all-view score files, offline (`src/eval.rs:365-392`).
  - `score-video` writes per-**frame** logits (views averaged), not per-view (`src/eval.rs:170-197`).
  - **The archived public score files contain probabilities only**: `docs/results/confirm7-scores.tsv.gz`, the NTIRE `scores.tsv.gz` and the NTIRE `all-views-scores.tsv.gz`. Per-frame logits are archived only for the Davodi video run (`docs/results/davodi-2026-video/frame-scores.tsv.gz`).
  - `detect` and the API return the mean logit, threshold and view count, not per-view logits (`src/scan.rs:611-630`; `src/server.rs:441-445`).
  - So offline re-aggregation experiments need a local re-score with `--all-views`. Note 07's "Tintype stores per-view logits for its development and blind sets" is not true of what is in the repo.

## 5. Threshold calibration

- `calibrate` (`src/eval.rs:276-337`):
  - Takes the **mean logit over all six views** per item.
  - Builds the real pool, grouped by real source (the generator key `real:*`).
  - Default rule **Balanced(100)**: a bisection finds the threshold at which the *mean over real sources with ≥100 items* of the per-source FPR equals the target (`src/eval.rs:305-319`).
  - Then fits only the logistic slope, classes weighted equally, around that fixed threshold (`src/metrics.rs:177-198`). The bias is set so that the threshold maps to p = 0.5. The slope changes probabilities, not verdicts.
  - Pooled and strictest rules exist but are not used by `finalize`.
- **Targets** (`src/eval.rs:585-589`; `src/main.rs:161-171`):

  | Profile | FPR target | Raw threshold |
  |---|---|---|
  | photo default | 1.5% | 1.692 |
  | strict | 0.5% | 3.397 |
  | sensitive | 5% | −0.776 |
  | fast | 0.5% (uses `strict_fpr`) | 4.197 |

  Raw thresholds computed from `models/tintype.json`. The default moved from 0.5% to 1.5% with the weights unchanged (`docs/DATA.md:191-198`).
- **Video:** 0.5% (`--video-fpr 0.005`; the CLI default is 0.01), Balanced(100), raw threshold 1.621 (`README.md:490`; `src/main.rs:190`; `models/tintype-video.json`; `docs/PREREGISTRATION.md:50-53`).
- **Calibration data:**
  - Photos: the 3% path-hash validation split of every training store, plus the development-visible evaluation manifests (OpenFake test, So-Fake-OOD, Synthbuster, AIGIBench, WasItAI images, SocialRF, …), plus later-scored sets (`src/eval.rs:561-580`; `src/plans.rs:1098-1110`).
  - Video: real clips from MSR-VTT, MSVD and Kinetics (`docs/DATA.md:401-403`).
- Calibration is done on **all six views**, while deployment uses early exit (fewer views). The early-exit margins were constrained to leave pooled precision and recall unchanged, but the per-source FPR under early exit is not re-checked against the target in code.
- There is no quality-, format- or domain-conditioned threshold, no conformal bound and no abstention on uncertainty. The only `inconclusive` verdict is for video decode coverage that is incomplete (`src/scan.rs:611-614`; `src/server.rs:506-510`).

## 6. NTIRE 2026 in docs/EVIDENCE.md

- **Public test, distorted, shipped cutoff:** recall 44.8%, 155/600 authentic flagged (25.83%). Recall at 1% FPR 2.2%; AUROC 0.636 (clean 0.870) (`docs/EVIDENCE.md:193-198`).
- **Hard validation:** distorted recall at 1% FPR 5.4%, AUROC 0.734; shipped cutoff flags 103/625 (16.48%) (`docs/EVIDENCE.md:172-177`).
- **110 of the 155 false positives in the JPEG AI / Cheng2020 / WMForger group: TRUE.** 110 of 182 such authentic images (60.4%) vs 45 of 418 (10.8%) for the rest.
  - Resolution is a weaker separator: 28.2% (<512 px) vs 23.6%.
  - All 5,000 files share the same JPEG quantization tables and 4:2:0 subsampling, so metadata cannot identify prior processing.
  - Hard validation contains only JPEG AI among the three operations.
  - (`docs/EVIDENCE.md:222`)
- **Re-thresholding: TRUE, with framing caveats.**
  - The cutoffs were set on hard-validation authentic scores at 1% FPR. The distorted cutoff, 0.996498 (probability), transferred to public-distorted gives **recall 31/650 = 4.77% and FP 13/600 = 2.17%** (`docs/EVIDENCE.md:212-218`).
  - The clean cutoff on public-distorted flags 10.50% (`docs/EVIDENCE.md:220`).
  - The doc concludes "the distorted score ranking remains weak" and "the distorted results point to ranking robustness and calibration". The shipped cutoff is unchanged (`docs/EVIDENCE.md:220, 226`).
  - So note 08's "mainly a threshold-transfer problem, not a ranking problem" **contradicts the repo's own reading**: FPR is fixable, but only at a near-total recall loss.
- **All-views ablation (hard validation):**
  - Clean recall at 1% FPR 51.8% → 60.5%, AUROC 0.938 → 0.955.
  - Distorted 5.4% → 7.5%, AUROC 0.734 → 0.739.
  - At the shipped cutoff: 263 vs 272 generated flagged, 106 vs 103 authentic flagged.
  - (`docs/EVIDENCE.md:224`)
- **Entrants:** clean AUROC 0.9132–0.9978, distorted 0.8302–0.9738 (`docs/EVIDENCE.md:200`).

## 7. Content-matched video (Davodi et al.)

- **All 20 authentic clips are Pixabay renditions** (`docs/EVIDENCE.md:236`; `docs/results/davodi-2026-video/inputs.tsv`).
- **The 3 flagged authentic clips are scenes 04 (p = 0.963), 03 (0.915) and 20 (0.684); TRUE that they are Pixabay stock** (`docs/results/davodi-2026-video/scores.tsv.gz`).
  - Scene 03's source was 1920×1080 at 30 fps. Scenes 04 and 20 were 960×540 at 25 fps "_small" CDN renditions (`inputs.tsv`).
  - All 100 inputs were normalized to 800 px wide, H.264 CRF 18, 24 fps, first 4 s (`docs/EVIDENCE.md:238`).
- Results: Tintype 72/80 generated and 3/20 authentic flagged, AUROC 0.9244. WaveRep 34/80 and 4/20 (`docs/EVIDENCE.md:240-243`). With 20 reals, each false alarm is 5 points (`docs/EVIDENCE.md:238`).
- **Note 05 omission:** Tintype's video training reals already include **Pexels stock footage**.
  - The `pexels` and `pexelsb` stores (open-sora-pexels-subset shards 0–4) are counted together with Kinetics in the store tables: 2.1 k + 2.3 k clips (`docs/DATA.md:71, 85`).
  - OpenVid reals come via GenBuster (`docs/DATA.md:68`).
  - "Stock footage is absent from training" is therefore **PARTLY FALSE**. Pixabay specifically is absent.

## 8. Token merging (ToMe)

**TRUE.** On 1,000 timing images, recall went 0.848 → 0.786 at 8 merged tokens per block (−6.2 points) and → 0.708 at 13 (−14.0). Dropping tokens instead of averaging them was no better. Merging only in the last 4 blocks cost 6 verdicts per 1,000 and saved about 15%. ToMe was applied without retraining (`docs/BENCHMARK.md:911-916`). Fused SDPA was no faster at 197 tokens (`docs/BENCHMARK.md:917-919`).

## 9. Quantization and precision

- **CPU engine: FP32 only, no INT8: TRUE.** Every linear layer is one `sgemm` (Accelerate on macOS, the pure-Rust `gemm` crate elsewhere) (`src/cpu_vit.rs:3-8, 19-48`; `docs/BENCHMARK.md:814`).
- **GPU (Apple, MPSGraph): FP16 by default.** `Device::Gpu` builds `Precision::F16`, and `gpu-f32` builds F32 (`src/detector.rs:463-469, 573, 584`). The pixels are converted to f16 planar by lookup table on the decode threads (`src/detector.rs:493-535`).
- The "8-bit input cast on the GPU" that was tried was an input-format experiment, not weight quantization. It was several times slower (`docs/BENCHMARK.md:917-919`).
- Training runs in FP32 (`src/trainer.rs:83`).
- No ONNX export or other runtime; the engine is custom.
- CPU speeds:
  - macOS CPU: 77–78 photos/s on batch 7 (`docs/BENCHMARK.md:250`).
  - Linux aarch64 VM (8 cores): 35.2–35.5 photos/s (`docs/BENCHMARK.md:965, 991`).

## 10. Video frame sampling and temporal modelling

- **16 frames** per clip in the video profile (`models/tintype-video.json` `frames: 16`). If a spec has no video profile, 8 frames are used (`src/detector.rs:962-971`).
- **Sampling is uniform over the first 30 s only, not the whole clip: PARTLY** (relative to note 05's "evenly spaced").
  - Frame *k* targets `start + k·span/16` with `span = min(duration, 30 s)` (`src/yuv.rs:141`; `src/video.rs:73, 80-87`). `--max-seconds 0` uses the whole clip (`src/scan.rs:97-99`).
  - Example: a 312 s clip is judged on its first 30 s.
- Per frame: six views, averaged to a frame logit. Clip logit = **mean of frame logits** (`src/eval.rs:170-172`; `src/video.rs:152-157`).
- **No temporal modelling of any kind: TRUE.** There are no consecutive-frame features, flow or temporal head. Training sees one random frame per draw (`src/video.rs:172-189`).
- The video checkpoint is separate: phases 4 + 5, 7 k + 4 k steps, continued from the phase-3 weights (`docs/DATA.md:101-133`; `README.md:488-489`).

## 11. Initialisation, phases, steps, regularisation

- **Initialisation: Community Forensics 224 px checkpoint: TRUE** (`docs/DATA.md:58`; `README.md:293-295, 462`, `--init baselines/commfor-224.safetensors --init-prefix vit.`).
- `docs/TRAINING.md` describes a *different* path for **separately trained** checkpoints: a DINO ViT-S/16 initialiser through `init-training` with a zero head (`docs/TRAINING.md:9-31`; `src/training_init.rs:12, 52-60`). This is not how the released weights were made.
- **Architecture:** ViT-S/16, 384 dim, 12 blocks, 6 heads, 224 px, single-logit head (`src/vit.rs:22-23`).
- **Phases (photo lineage) — 36 k steps cumulative:**

  | Phase | Steps | LR | Warmup | Started from | Evidence |
  |---|---|---|---|---|---|
  | 1 | 14 k | 5e-5 (default) | 500 | CF-224 | `README.md:462-463`; `src/main.rs:292-299` |
  | 2 | 7 k | 2.5e-5 | 300 | phase 1 | `README.md:464` |
  | 3 | 7 k | 2e-5 | 300 | phase 2 | `README.md:465` |
  | 6 (released photo weights) | 8 k | 2e-5 | 300 | phase 3 | `README.md:476`; `docs/DATA.md:134-137` |

  Phase 6 trained on the photo stores *and* the phase-1 video stores (`README.md:476`).
- **Video lineage:** phases 4 and 5 (7 k at 2e-5, then 4 k at 1.5e-5), continued from phase 3 — 39 k steps cumulative (`README.md:488-489`).
- **Optimiser:** batch 32. Linear warmup, then cosine decay (`src/train_cmd.rs:158-163`). AdamW with β (0.9, 0.999). **Decoupled weight decay 0.05 toward zero** (none on bias, norm, cls or pos). Layer-wise LR decay 0.8. Global gradient clip 1.0. Label smoothing 0. BCE on one logit (`src/trainer.rs:43-46, 94-140`; `src/main.rs:313-319`).
- **Regularisation toward the initial weights: none.** There is no L2-SP, EMA, distillation from CF, frozen layers, LoRA or SVD constraint. Weight decay pulls toward 0, not toward the init (`src/trainer.rs:134-136`). The only "anchoring" is a small LR plus layer decay. Note 02's proposals (Effort/anchor regularisation) would be new.
- **Checkpoint selection:** the last step of each phase. Validation reports AUC on fit, native and their fused mean (`src/train_cmd.rs:76-103`). The video model had no checkpoint selection (`docs/PREREGISTRATION.md:43-44`).
- **Sampler:** both classes 50/50; generators within a class weighted count^0.5. Family-level weighting is not implemented (`src/data.rs:106-135`).

## 12. Other facts a recommendation might get wrong

- **Already present (do not recommend as new):**
  - Optional cascades to CF-384 (photo) and WaveRep (video).
  - Four photo operating points.
  - View-level and frame-level early exit.
  - Texture-ranked native crops.
  - Messenger augmentation.
  - H.264 re-encode augmentation for video.
  - Real animation, cartoon and gameplay negatives for video (`docs/DATA.md:118-127`).
  - GAN, face and identity-adapter data in phase 6: StyleGAN2/3, BigGAN, GigaGAN, CelebA-HQ, IP-Adapter FaceID, Tiny-GenImage (`docs/DATA.md:134-146`).
  - C2PA Content Credentials reading behind the `provenance` feature. It also sniffs MP4/MOV, so video C2PA is at least parsed by MIME type (`Cargo.toml:53`; `src/provenance.rs:186-203`).
  - Pre-registered blind batches (`docs/PREREGISTRATION.md`).
  - Offline profile re-application from score files (`src/eval.rs:365-392`).
- **Not present:**
  - abstention/UNSURE on low confidence;
  - quality-conditioned calibration;
  - learned fusion;
  - per-view weights;
  - max/top-k pooling;
  - intermediate-layer readout (the last block computes only the class token: `docs/BENCHMARK.md:904-907`);
  - an auxiliary head;
  - local-edit or face-swap localisation (out of scope: `README.md:510-512`);
  - INT8;
  - a temporal head;
  - a WebP/blur/noise/neural-codec augmentation.
- **The GPU engine, training, evaluation and the video second opinion are macOS-only.** Linux/Windows run CPU `detect` (`README.md:521-522`; `src/main.rs:653-656`). Any training-side proposal must land in the MPSGraph trainer (`src/trainer.rs`) or in a new external pipeline. The notes' "PyTorch, 1–2 days" estimates assume tooling that does not exist in the repo.
- **ViT config is parametric** (`VitConfig::small`, and a 768-dim reg-token config) (`src/vit.rs:22-31`). A ViT-Ti stage-0 model would need a new config, but no engine rewrite.
- **Weak-spot numbers quoted by the notes: TRUE.**
  - Messenger copies: 27–36% default, 55–59% sensitive (`README.md:513-516`).
  - GAN sets: 84–89% (`README.md:517-519`).
  - Face-swap set: 14% (`README.md:510-512`; `docs/BENCHMARK.md:220`, FF++ 0.143).
  - Frontier JPEG stress: 69.3 / 53.3 / 28.0% (`docs/EVIDENCE.md:36-40`).
  - Arena: 88.5 → 73.1% (`docs/EVIDENCE.md:93-96`).
  - So-Fake-OOD JPEG subset: 77.0% (`docs/EVIDENCE.md:22`).
  - Batch 7: P 0.988 / R 0.897 (`docs/BENCHMARK.md:185`).
  - Video blind: 0.997 / 0.941 (`docs/BENCHMARK.md:881`).
- **GPTTrace (PE-Core-L, 448 px) beats Tintype after JPEG normalization on Arena:** 76.7% vs 73.1% at 1% FPR (`docs/EVIDENCE.md:139-144`). This is relevant to note 02's foundation-model case.
- **Stale comment:** the `VideoSpec::standard` doc comment says "centre and four quadrants", but the code uses the four texture-ranked crops (`src/detector.rs:274-278`).
- **Image decoding** goes through `image::load_from_memory(...).into_rgb8()`. No EXIF orientation is applied in `image_io::decode` (`src/image_io.rs:46-55`).

---

## Note assumptions that need correcting

| Note | Assumption | Verdict | Evidence |
|---|---|---|---|
| 03 (header) | "Training is six-view inference with texture-ranked native crops" | **FALSE**: training uses one random view per sample (random-position native crop or fit). Texture ranking is inference-only. | `src/data.rs:80-94` |
| 03, 04 | 15.8 k vs 3.0 k Reddit counts teach "platform look ⇒ AI" | **Counts TRUE; inference unsupported.** The class-balanced count^0.5 sampler makes `reddit-wild` about 1% of AI draws, while `real:openfake-reddit` is likely a larger share of real draws. | `src/data.rs:106-135`; `src/plans.rs:340` |
| 08 | FPR failure is "mainly threshold transfer, not ranking" | **FALSE per repo**: re-thresholding drops recall to 4.77%. The repo itself calls the distorted ranking weak. | `docs/EVIDENCE.md:216-226` |
| 07 | "Tintype stores per-view logits for its dev and blind sets" | **PARTLY**: `score --all-views` can write them, but the archived files hold probabilities only, and the default scoring writes only the early-exit views. | `src/eval.rs:59-117`; `docs/results/*` |
| 05 | Stock footage is absent from video training | **PARTLY FALSE**: Pexels stock clips (and OpenVid) are real negatives. Pixabay is absent. | `docs/DATA.md:68, 71, 85` |
| 05 | "16 evenly spaced frames" | **PARTLY**: evenly spaced over the first 30 s only. | `src/yuv.rs:141`; `src/video.rs:73` |
| 01, 04 | "random-scale crops" | **PARTLY**: p = 0.3 jitter (native ×0.5–1.25; fit short side 224–320), not RandomResizedCrop. | `src/data.rs:82-92` |
| 04, others | "25% messenger round trip" | **PARTLY**: 25% for photo phases 2, 3 and 6; 0% in phase 1; 10% for video. | `README.md:462-489` |
| 07 | Second opinion fusion formula and uncalibrated threshold | **TRUE** | `src/detector.rs:234`; `docs/PREREGISTRATION.md:185-197` |
| 07 | ToMe lost 6–14 recall points | **TRUE** (−6.2 / −14.0) | `docs/BENCHMARK.md:911-916` |
| 07 | No INT8; GPU 16-bit | **TRUE** | `src/cpu_vit.rs:3-8`; `src/detector.rs:573` |
| 05 | No temporal modelling; clip = mean logit; about 5 frames per clip | **TRUE** (4.8 frames) | `src/video.rs:152-157`; `docs/BENCHMARK.md:871-881` |
| 05, 08 | 3/20 flagged authentic clips are Pixabay | **TRUE** (scenes 3, 4, 20) | `docs/results/davodi-2026-video/scores.tsv.gz` |
| 08 | FPR 25.83% → 2.17%, recall 44.8% → 4.77% | **TRUE** (cutoff transferred from hard-validation distorted at 1%) | `docs/EVIDENCE.md:198, 212-218` |
| 04 | 110/155 false positives in the JPEG AI / Cheng2020 / WMForger group | **TRUE** | `docs/EVIDENCE.md:222` |
| 02, 01 | Initialised from Community Forensics | **TRUE** (CF-224) | `docs/DATA.md:58`; `README.md:462` |
| 02 | Fully fine-tuned with no anchoring | **TRUE**: weight decay goes toward 0, with layer-LR decay 0.8 | `src/trainer.rs:131-136` |
| 03 | Augmentation lacks noise, blur, WebP, colour, neural codecs | **TRUE** | `src/data.rs:57-104` |
| 08 | Thresholds at 0.5 / 1.5 / 5% FPR averaged over sources | **TRUE** (Balanced rule, sources with ≥100 items; fast uses 0.5%; video 0.5%) | `src/eval.rs:305-319, 585-589` |
