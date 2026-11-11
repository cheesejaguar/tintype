# Raw results

The tables `veracity evaluate` printed for each suite, unedited, and the log
of the speed runs. [../BENCHMARK.md](../BENCHMARK.md) presents them.

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
| `confirm6_at_reference_rates.md` | That batch with Veracity at each reference detector's false-positive rate |
| `*_phase6_weights.md` | The earlier suites re-scored with the phase-6 photo weights (seen data) |
| `speed_interleaved.log` | Phase-6 and phase-3 photo weights and Community Forensics, alternated on the same files with the machine under load |
| `linux.log` | Photo detection built, tested and run in an aarch64 Linux virtual machine: agreement with macOS and timings |
