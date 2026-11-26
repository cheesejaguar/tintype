# Raw results

The tables `tintype evaluate` printed for each suite, unedited, and the log
of the speed runs. [../BENCHMARK.md](../BENCHMARK.md) presents them.

The project was called Veracity while these runs were made. The name has
been replaced with Tintype throughout these files, in table rows, commands
and paths alike; nothing else in them was changed.

| File | Run |
|---|---|
| `confirm3.md` | Blind photo batch; first blind video result (phase-3 weights) |
| `confirm4.md` | Blind video batch (video checkpoint) |
| `benchmarks.md`, `confirm1.md`, `confirm2.md` | Development suites with the released weights |
| `confirm5.md` | Second blind photo batch (1.5% default, early exit) |
| `confirm3_new_default.md`, `benchmarks_new_default.md`, `confirm1_new_default.md`, `confirm2_new_default.md` | The earlier suites re-scored with the current default profile, early exit and native video decoding |
| `speed.log` | Throughput and latency runs on the idle machine, build before early exit |
| `speed_final.log` | The same for the final build |
| `speed_released.log` | The reference detectors through their authors' PyTorch pipelines, same files, idle machine |
| `confirm6.md` | Third blind photo batch (phase-6 photo weights) |
| `confirm6_phase3_weights.md` | The same batch scored with the phase-3 weights |
| `confirm6_at_reference_rates.md` | That batch with Tintype at each reference detector's false-positive rate |
| `*_phase6_weights.md` | The earlier suites re-scored with the phase-6 photo weights (seen data) |
| `speed_interleaved.log` | Phase-6 and phase-3 photo weights and Community Forensics, alternated on the same files with the machine under load |
| `speed_improvements.log` | The build before and after the five speed changes, alternated on the same files, then both builds on blind batch 6 (photos) and the blind video batch |
| `confirm7.md` | Fourth blind photo batch (photo second opinion; default profile alongside) |
| `confirm4-scores.tsv.gz`, `confirm7-scores.tsv.gz` | Per-file detector scores for the release comparison, validated against the frozen video and photo manifests |
| `frontier-jpeg-stress-scores.tsv.gz` | Paired originals and two JPEG transforms scored by Tintype and Community Forensics on the frontier photo development set |
| `arena-v0.1-gpttrace/` | Pinned GPTTrace, Tintype and Community Forensics scores on all 2,038 Arena originals and paired JPEGs |
| `policy-comparison/` | Exploratory photo policy comparisons printed by `scripts/compare-policies.py` from the confirmation 7, Frontier and Arena score archives: the CF band, GPTTrace fusion, gate misses, and a dry run of the frozen-split commands ([runbook](../runbooks/policy-comparison.md)) |
| `policy-comparison-frozen/` | The photo policy comparison on the frozen independent split: per-file scores of the released model's three profiles, Community Forensics and the two real two-stage runs on `fit`, `calibrate` and `test`, the choice made on `calibrate`, and the tables on `test` ([runbook](../runbooks/policy-comparison.md#frozen-split-the-run-of-2026-11-14)) |
| `frontier-operation-stress/` | The 745 frontier photo pairs through twelve FFmpeg transforms (resize, JPEG, blur, noise, sharpening, reposts), scored on the CPU with the released weights: per-file scores and the paired per-operation table |
| `ntire-2026-operations.md` | Every operation and severity of the two released NTIRE splits, from the archived Tintype scores and the organizers' labels (`scripts/diagnose-ntire.py --operations`) |
| `ntire-2026-hard-validation/` | Both detectors' full scores and tables for the organizers' 2,500 released hard-validation images |
| `ntire-2026-public-test/` | Both detectors' full scores and tables for the organizers' 2,500 released public test images |
| `davodi-2026-video/` | Matched detector scores, source and normalized hashes, and run parameters for 20 of 24 released video themes |
| `confirm7_second_opinion.log` | That batch set by set against Community Forensics, the timing of the mode on a sample of it, and the check of a real two-stage run against the score-file arithmetic |
| `linux.log` | Photo and video detection built, tested and run in an aarch64 Linux virtual machine: a sample first, then both whole blind batches next to Community Forensics as released – agreement with macOS, precision, recall and timings |
| `linux_after_speed.log` | The Linux runs after the speed changes: the build with them against the build before on both blind batches, and the photo second opinion on blind batches 6 and 7 next to Community Forensics as released |
| `photo_operating_points.log` | Third blind photo batch, computed after scoring: per-set AUC against Community Forensics, Tintype's recall when it may flag no more real photos than Community Forensics does on each set, and precision and recall as one threshold is moved – the per-set recall gap is mostly the shipped threshold, and no single threshold wins on every set |
| `data-audit/` | Diagnostics of the local training stores: file formats and sizes per class, a metadata-only label probe, the sampling shares of the phase-6 recipe, generator families, and a review-sample draw ([README](data-audit/README.md)) |
| `source-family-arms-frozen/` | The family-weighted and the source-matched training arm against the current mix (two seeds) on the frozen split: the gate quantities at 1% and 0.1%, per family, per transform and per authentic source, confirmation batch 7 as a diagnostic, and the score files ([README](source-family-arms-frozen/README.md)) |
| `vp9-decode/` | The pure-Rust VP9 and VP8 reader against libvpx and FFmpeg on eight synthetic clips: every plane and time of every frame, and the scores of lossless H.264 controls made from the same frames ([README](vp9-decode/README.md)). An engineering check, with no detection figures |
| `interpolation-pooling/` | Exploratory, on development data, nothing selected: the pooling replay of the released weights' saved view logits (`pool-replay.md`, and `pool-replay-asymmetric.md` with margins fitted separately above and below) and the ranking table of the interpolation sweep on three small development sets (`alpha-sweep.md`). Commands, commit and hardware are in [the runbook](../runbooks/interpolation-pooling.md). `frozen/` holds the same tools' output on the frozen independent split: the pooling candidates fitted on `fit` and read on `calibrate`, the refitted mean on `test`, the interpolation candidates' calibrate-stage table, and the one interpolant read on `test` with its all-views scores |
