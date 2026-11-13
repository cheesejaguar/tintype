# Improving Tintype’s recall, precision, and speed

Research through November 13, 2026. Detector baseline `2e3d516`. This report combines the two literature reviews, checks their recommendations against the shipped pipeline, and adds reproducible comparisons from the score archives. The [combined bibliography](../research_notes/Tintype%20detection%20improvement%20research/99_bibliography.md) contains 376 papers, reports, benchmarks, and supporting studies.

**Start with matched processing in photo training. In parallel, validate a complementary photo model and run a real-source video audit.** For speed, first remove redundant crop and decode work. Fit and test all decision policies on independent authentic sources. The best model at a permissive false-positive rate (FPR) can lose at the stricter rate the product needs.

| Rank | Change | First decision |
|---:|---|---|
| 1 | Train photos through matched processing chains | Does the current ViT improve on unseen processing chains at fixed FPR? |
| 2 | Calibrate the deployed decision and add an inconclusive outcome | Do frozen cutoffs transfer to new authentic sources at useful coverage? |
| 3 | Test a complementary photo model and a cheaper existing route | Which candidate adds detections at the chosen FPR and latency budget? |
| 4 | Match image sources and generator families, then refresh with replay | Can new-generator recall improve without losing older families or reposted images? |
| 5 | Match video quality and content and audit authentic sources and watermarks | Which failure remains after controlling for source, codec, and visible marks? |
| 6 | Reuse identical view scores and avoid unnecessary crop and decode work | How much end-to-end latency can be removed with equivalent decisions? |
| 7 | Add a small temporal head to the frame baseline | Do adjacent-frame features beat more frame views at the same cost? |
| 8 | Profile a shallower student and portable INT8 | Which change saves wall time while preserving ranking at low FPR? |
| 9 | Evaluate partial edits with spatial or temporal supervision | Can Tintype find a local change without labeling the entire asset as generated? |
| 10 | Test weight interpolation and view pooling as bounded offline experiments | Can either recover legacy or clean recall without weakening robust performance? |

## What the archived results establish

| Cohort | Observation | Implication |
|---|---|---|
| Photo confirmation 7 | 88.5% recall at 1% empirical FPR. At the shipped threshold, 89.7% recall and 130/10,747 authentic flags. | Preserve this as a regression benchmark. Source and generator overlap limit the transfer claim. |
| Frontier paired JPEG stress | 69.3% → 53.3% → 28.0% recall at 1% FPR on original, same-size JPEG, and JPEG downscaled to 960 px. | Processing weakens the signal on otherwise identical media. |
| NTIRE public distorted | 2.2% recall at 1% FPR. The shipped cutoff catches 44.8% and flags 155/600 authentic images. | This is both a ranking failure and a threshold-transfer failure. |
| NTIRE operation replay | Public distorted files with JPEG AI, Cheng2020, or WMForger have AUROC **0.549** and 110/182 authentic flags at the shipped cutoff. Other distorted files have AUROC **0.690** and 45/418 flags. | The three operations overlap with other processing. Their group is a useful canary for a detector that confuses processing with generation. |
| NTIRE hard-validation threshold transfer | A clean validation cutoff with 0.96% FPR flagged 10.50% of public distorted reals. The distorted validation cutoff transferred at 2.17% FPR and 4.77% recall. | A lower false-alarm rate on the calibration cohort gives limited information about transfer. |
| Video confirmation 4 and content-matched DaVODi | Confirmation 4 has 1,088/1,156 generated detections and 3/900 authentic flags at the shipped cutoff. DaVODi has 72/80 and 3/20. All three DaVODi false alarms and seven of eight misses used all 16 frames. | Use the frame baseline and investigate source shift. The paired set has only 20 authentic scenes. |

[Archived scores and evaluation protocols](../docs/EVIDENCE.md) · [Benchmarks and speed ablations](../docs/BENCHMARK.md)

### Photo routing at equal false-positive counts

The existing opt-in cascade blends Tintype with Community Forensics (CF). The replay reproduced every shipped verdict on confirmation 7 from the archived scores and current band. Changing the lower logit bound from −7 to −3 cut routes from **11,068 to 2,417** of 23,052. The table uses one empirical cutoff per policy and budget. The final column counts sets where both precision and recall reach CF at its shipped cutoff, using each policy’s single cutoff at the 329-FP budget.

| Policy | Routed | Recall at 107 FP (~1% FPR) | Recall at 329 FP (~3.06% FPR) | Sets matching or beating CF at 329 FP |
|---|---:|---:|---:|---:|
| Tintype default | 0% | 88.48% | 93.20% | 14/30 |
| Tintype sensitive | 0% | 86.21% | **94.84%** | 14/30 |
| Shipped CF blend | 48.01% | 85.22% | 93.49% | **26/30** |
| Narrowed CF blend | 10.48% | 88.35% | 94.14% | 20/30 |

At the original score cutoff of 0.5, the narrowed blend catches 11,413 generated images with 213 false positives, compared with 11,504 and 329 for the shipped blend. On paired Frontier JPEGs, narrowing routes 27.0% of same-size JPEGs and 38.1% of capped JPEGs. It catches 116/261 and 67/261 at 1% empirical FPR, versus 139/261 and 73/261 for Tintype alone. The routing reduction is measured workload. End-to-end savings depend on the cohort and hardware. [Confirmation 7 scores](../docs/results/confirm7-scores.tsv.gz) · [Frontier score archive](../docs/results/frontier-jpeg-stress-scores.tsv.gz)

GPTTrace offers a different tradeoff. On Arena, after removing seven repeated authentic originals from each variant, an equal-weight logit blend for `abs(Tintype logit) ≤ 2` lifted recall at 1% empirical FPR from **88.51% to 91.36%** on originals and **73.08% to 81.04%** on paired JPEGs. It routed 13.0% and 18.8%, respectively. Full fusion had 81.93% JPEG recall. The band left 140 of 310 Tintype JPEG false negatives outside the gate at the shipped cutoff. Measure a quality or disagreement feature on independent data before deciding whether its extra complexity improves routing. Both score studies chose the comparison cutoffs on the cohorts reported. [Arena comparison and score archive](../docs/EVIDENCE.md#gpttrace-comparison-on-the-same-arena-images)

## Experiments in priority order

### 1. Train photos through matched processing chains

Audit paired originals and derivatives first. Measure false flags on authentic media alongside recall and AUROC for each operation and severity. Extend `augment()` with label-symmetric chains of resizing, JPEG, blur, noise, sharpening, and controlled repost simulations. Include real upload round trips when possible. In a separate arm, add offline paired JPEG AI, Cheng2020 or another learned codec, plus benign restoration or enhancement negatives. Preserve parent IDs through the split. Run the current recipe, pixel chains, and pixel chains plus codec examples at the same training budget. A first promotion target is **+10 recall points at 1% empirical FPR** on a fresh distorted suite, clean recall within 1 point, and fewer learned-codec alarms at a frozen cutoff.

Make clean-to-processed consistency a later ablation. TeleAI’s KL-only arm gained **0.15 percentage points of AUROC** on mixed clean and distorted inputs. DCPT’s degraded AUROC rose from 78.5% to 79.4%, while clean AUROC fell from 89.1% to 87.9%. NTIRE’s higher-scoring entrants paired heavy augmentation with different training sets and larger models, so those results guide the training recipe. [Augmentation code](../src/data.rs#L57) · [NTIRE challenge report](https://arxiv.org/html/2604.11487v1) · [JPEG AI](https://arxiv.org/html/2412.03261v2) · [TeleAI Table 4](https://arxiv.org/html/2604.12307v1#S5.SS3) · [DCPT](https://arxiv.org/html/2604.10102v1)

### 2. Calibrate the deployed decision and add an inconclusive outcome

Fit cutoffs on independent authentic media after fixing the views, exits, model/backend, and any second-stage policy. Start with a global false-alarm budget and measure transfer by predeclared source and quality strata. Compare an authentic-only order-statistic bound with the current quantile rule. Use group cutoffs only where the group is observable at inference, has sufficient calibration data, and transfers. NTIRE operation labels are available for diagnosis. Final JPEG tables are identical across the splits and cannot recover those histories. Calibrate abstention using score margin plus a tested reliability feature such as view disagreement. Count abstained generated items in overall recall and report coverage and review rate.

At 1% prevalence, confirmation 7’s shipped rates would imply about **43% precision** if they transferred. At that prevalence and 90% recall, reaching 90% precision requires about **0.10% FPR**. Even with zero flags, about **3,000 independent authentic inputs** are needed for a one-sided 95% binomial upper bound near 0.10%. Use clustered intervals where sources or parent media repeat. Treat the current class-balanced sigmoid as a detector score. Refit raw exit margins and calibrated routing boundaries when threshold or score parameters move. [Calibration implementation](../src/metrics.rs#L153) · [Early exit and routing](../src/detector.rs#L121) · [Neyman–Pearson thresholds](https://escholarship.org/uc/item/2996c8m6) · [ReSIDe](https://arxiv.org/abs/2605.08574)

### 3. Test a complementary photo model and a cheaper existing route

Compare the strongest retuned single-model profile, the existing and narrower CF route, a modern frozen image encoder with a linear probe, and full fusion. Use GPTTrace as a comparison checkpoint and PE, DINOv3 or MetaCLIP 2 probes to see how much complementarity survives at lower cost. Fit simple logit fusion before testing a learned gate. If strong negatives with weak evidence are missed, add disagreement or processing features and test them against a margin-only gate on a fresh split. First target **+5 recall points at 1% FPR** on degraded media, with bounded p95, route rate, and memory. Keep the existing CF narrowing as a candidate for opt-in speed when its per-set performance meets the intended use. [Current band](../src/detector.rs#L220) · [CO-SPY](https://openaccess.thecvf.com/content/CVPR2025/html/Cheng_CO-SPY_Combining_Semantic_and_Pixel_Features_to_Detect_Synthetic_Images_CVPR_2025_paper.html) · [Simplicity Prevails](https://arxiv.org/abs/2602.01738)

### 4. Match image sources and generator families, then refresh with replay

Use parent, prompt or scene, authentic source, transform, and generator-family IDs to keep siblings and derivatives together. Balance both classes by processing history and content. Audit metadata-only label predictability, weight families explicitly, and sample authentic labels for review across strata as well as in the high-score tail. Test same-content reconstructions from different decoder families as controlled examples alongside independent, recent diffusion, autoregressive and hybrid outputs. Distinguish full generation from content-preserving camera processing. Compare equal-budget training mixes and use an older-generator plus repost replay buffer during refreshes. Gate on new-family macro recall and older-family regression at fixed FPR. [Existing data and lineages](../docs/DATA.md) · [Fake or JPEG?](https://arxiv.org/html/2403.17608v2) · [B-Free](https://arxiv.org/abs/2412.17671) · [Community Forensics](https://arxiv.org/abs/2411.04125) · [GAPL](https://arxiv.org/abs/2512.12982) · [Continual refresh](https://arxiv.org/html/2605.02567)

### 5. Match video quality and content and audit authentic sources and watermarks

Expand DaVODi with held-out authentic collections including stock, CCTV, handheld, animation, CGI, and screen recordings. Match motion, cadence, resolution, and processing across classes. For image-to-video, use matched starting frames where possible. Score originals, a common fixed-GOP re-encode, and simulated reposts. Run a visible-watermark add/remove test with sham logos and placebo crops or removals to distinguish a shortcut from the alteration itself. Use metadata-only and real-vs-real probes. At the same three-false-positive allowance on confirmation 4, the archived WaveRep cascade caught 1,112/1,156 versus 1,099/1,156 from Tintype alone. As standalone detectors at 0.5, WaveRep caught 39 of 68 Tintype misses there and zero of eight on DaVODi. Require gains on at least two controlled cohorts before widening that route. [DaVODi protocol](../docs/EVIDENCE.md#content-matched-generated-video-clips) · [Real Data Lies](https://proceedings.mlr.press/v306/fang26s.html) · [Corpus Identity](https://arxiv.org/abs/2606.31004v2) · [RobustSora](https://arxiv.org/abs/2512.10248v2) · [RPG](https://arxiv.org/abs/2611.14941)

### 6. Reuse identical view scores and avoid unnecessary crop and decode work

Cache crop results by effective pixel transform and location and reuse the score in each original logical slot. Keep view multiplicity and early-exit order. Several native views use identical upscaled pixels below crop size, and texture-ranked windows can repeat when few positions exist. This preserves the aggregation semantics. Validate numerical parity across batch shapes and backends. Then prepare crops lazily and benchmark bounded decoder continuation for video escalation. With the current first two samples at indices 0 and 7, a sequential reader still decodes through index 7 before it can decide. Lazy preparation can save RGB conversion and crop work in that prefix, while a retained decoder can save repeated decoding for clips that continue. Measure file-to-verdict p50/p95, throughput, memory, and numbers of decoded frames and materialized views on the hardware that will run it. [Crop selection](../src/image_io.rs#L295) · [Video and crop staging](../src/detector.rs#L788) · [Compressed-domain efficiency study](https://proceedings.mlr.press/v306/zhu26ap.html)

### 7. Add a small temporal head to the frame baseline

Expose selected patch embeddings and compare a shallow head over adjacent frames with the current frame mean, an order-invariant head, and more frame views at equal decode and inference cost. Align spatial positions, use actual timestamps, and control cuts, repeated frames, motion, and cadence. Include shuffled-order and real-blur controls. Preserve the fast whole-clip path until the head improves low-FPR recall on fresh matched clips and a second reposted corpus. RPG found weak performance for several standalone temporal methods, while V-PVP reports gains from local patch motion with video-pretrained features. Tintype’s smaller image backbone needs its own comparison. [Benchmark of frame-based methods](https://arxiv.org/abs/2601.11035) · [V-PVP](https://arxiv.org/abs/2607.15321v2) · [Motion shortcut audit](https://arxiv.org/html/2607.00948v2) · [RPG](https://arxiv.org/abs/2611.14941)

### 8. Profile a shallower student and portable INT8

Profile live x86-64 and ARM64 CPU runs to locate the cost of linear layers, decoding, and preprocessing. The documented 35.2 images/s is from an ARM Linux VM. Simulate per-channel INT8 or try existing quantized kernels before writing portable kernels. Keep sensitive math in higher precision and compare score tails, verdict flips, and exit stages. Where the model dominates, also compare a trained 6–8-block student or intermediate exits at the same 224 px input. Distill only from a teacher that improves the target slices. Existing test-time token merging reduced recall by over six points even at the smaller tested setting. The initial target is **1.3× CPU throughput or 25% lower p95**, with at most one point of recall loss on each clean and degraded suite at equal FPR. [Tintype speed and token-merging results](../docs/BENCHMARK.md#five-speed-changes-before-and-after) · [PTQ4ViT](https://arxiv.org/abs/2111.12293) · [Quantization in deepfake detection](https://www.ijcai.org/proceedings/2025/59) · [Multi-exit distillation](https://openaccess.thecvf.com/content_ICCV_2019/html/Phuong_Distillation-Based_Training_for_Multi-Exit_Architectures_ICCV_2019_paper.html)

### 9. Evaluate partial edits with spatial or temporal supervision

Keep a distinct partial-edit target. For images, compare a supervised patch head against global and top-k baselines on real backgrounds with inpainted or pasted regions. Keep VAE round trips, benign retouching and neural codecs as controls. Inpainting Exchange preserved the edited pixels while replacing the background and dropped DDA accuracy from 97.6% to 58.4%. Use self-blended faces as a face-specific training ablation with identity- and parent-disjoint full-frame tests. The existing 14.3% FF++ recall was measured on cropped faces and describes a different task from full-frame generation.

For video, prototype shot-aware and early/middle/late sampling as a coverage mode. Current early exit can decide from the first half of the sampled span, and the HTTP server samples at most the first 30 seconds. The CLI can request the entire span with `--max-seconds 0`, which still samples sparsely. Test short edits at known timestamps against fixed-budget frame sampling. Compare window aggregation at equal authentic clip FPR and report false flagged segments per hour, including authentic cuts and effects. Count unsampled edits as missed. [Inpainting Exchange](https://arxiv.org/abs/2602.00192v2) · [Self-Blended Images](https://arxiv.org/abs/2204.08376) · [TASLE](https://proceedings.mlr.press/v306/feng26g.html) · [FakeParts](https://arxiv.org/abs/2508.21052v2)

### 10. Test weight interpolation and view pooling as bounded offline experiments

Sweep interpolation toward the CF-224 initializer on a common preprocessing path. The two checkpoint headers agree on all 152 tensor shapes and types after removing CF’s `vit.` prefix. Verify endpoint parity and fit each candidate on separate calibration data. Look for legacy GAN gains at fixed FPR while protecting current generators and reposts. Compare checkpoint averaging only among compatible sibling fine-tunes, with replay and model averaging measured independently.

Use saved per-view logits to compare mean pooling, view order and alternative weights. On NTIRE hard validation, all six views improved clean recall at 1% FPR from 51.8% to 60.5%, versus 5.4% to 7.5% on distortions. At the shipped threshold, extra views lost nine generated detections and added three authentic flags on distorted input. A smooth/text-aware replacement view is an experiment at the existing compute budget. Any changed pool needs a new calibration and early-exit test. [Training lineage](../docs/DATA.md#phase-1-14000-steps-from-the-community-forensics-224-px-checkpoint) · [All-view results](../docs/EVIDENCE.md#ntire-threshold-transfer-and-artifact-diagnostics) · [WiSE-FT](https://arxiv.org/abs/2109.01903) · [Model soups](https://proceedings.mlr.press/v162/wortsman22a.html) · [TextureCrop](https://arxiv.org/html/2407.15500v4)

## What to run first

1. **Freeze an independent evaluation set before model selection.** Group by parent, source and generator family. Reserve separate fitting, calibration and test sets. Use confirmation 7, Arena, Frontier and released NTIRE as known diagnostic suites. Predeclare primary operating points and realistic class prevalence.
2. **Run the processing curriculum and photo policy comparison in parallel.** Use one shared set of fixed-budget metrics. Record recall at 0.1% and 1% empirical FPR, actual FPR and recall at each frozen cutoff, worst-slice results, abstention coverage, and decode-to-verdict latency. Select the next model based on gains on independent degraded files and performance on authentic media.
3. **Start the video audit and runtime profile together.** Use the paired data to choose whether the next video experiment should tackle source shift, temporal cues, or coverage. Keep correctness-equivalent speed changes independently measurable.

The experimental targets are proposed decision criteria. The replay measures score rankings and routing counts on existing archives. The report draws on primary abstracts and selected full-text methods and tables. Publication status and evidence depth vary by record. [Combined bibliography](../research_notes/Tintype%20detection%20improvement%20research/99_bibliography.md)

Reproduce the archive results from the repository root:

```sh
python3 scripts/replay-research.py
python3 scripts/replay-research.py --hard-labels val_hard_labels.csv --public-labels test_labels.csv
```

The second command uses the pinned NTIRE label files linked in the [repository evidence](../docs/EVIDENCE.md#external-comparisons). The script validates their hashes and joins them to the archived scores.
