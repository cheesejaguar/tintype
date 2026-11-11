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
