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
| `ntire-2026-hard-validation/` | Both detectors' full scores and tables for the organizers' 2,500 released hard-validation images |
| `ntire-2026-public-test/` | Both detectors' full scores and tables for the organizers' 2,500 released public test images |
| `davodi-2026-video/` | Matched detector scores, source and normalized hashes, and run parameters for 20 of 24 released video themes |
| `confirm7_second_opinion.log` | That batch set by set against Community Forensics, the timing of the mode on a sample of it, and the check of a real two-stage run against the score-file arithmetic |
| `linux.log` | Photo and video detection built, tested and run in an aarch64 Linux virtual machine: a sample first, then both whole blind batches next to Community Forensics as released – agreement with macOS, precision, recall and timings |
| `linux_after_speed.log` | The Linux runs after the speed changes: the build with them against the build before on both blind batches, and the photo second opinion on blind batches 6 and 7 next to Community Forensics as released |
| `photo_operating_points.log` | Third blind photo batch, computed after scoring: per-set AUC against Community Forensics, Tintype's recall when it may flag no more real photos than Community Forensics does on each set, and precision and recall as one threshold is moved – the per-set recall gap is mostly the shipped threshold, and no single threshold wins on every set |
| `interpolation-pooling/` | Exploratory, on development data, nothing selected: the pooling replay of the released weights' saved view logits (`pool-replay.md`, and `pool-replay-asymmetric.md` with margins fitted separately above and below) and the ranking table of the interpolation sweep on three small development sets (`alpha-sweep.md`). Commands, commit and hardware are in [the runbook](../runbooks/interpolation-pooling.md) |
