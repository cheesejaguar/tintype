# Speed per image on matched, idle hardware: result (2026-11-26)

The protocol is [PROTOCOL.md](PROTOCOL.md), registered in dbe60fef before
any timing. [speed_run.py](speed_run.py) ran once; the raw output is
[speed.tsv](speed.tsv).

## Set-up

- **Machine:** Apple M6, idle. Other `tintype`, training and scoring
  processes had finished, and the 1-minute load average was 2.0 to 4.0
  from background apps.
- **Files:** TPB v1's speed sample, the 2,000 open-split files with the
  smallest SHA-256 (1,092 generated, 908 real, all 27 sets).
  - They were rebuilt from their sources, and all hashes matched.
  - They were read from a RAM disk.
- **Detectors:** all four at their shipped settings. Community Forensics
  and SwinV2 ran through Tintype's Rust ports.
- **Timing:** from process start to exit, so model loading is included.
  There were three rounds in rotated order, each run after a 100-file
  warm-up.

## Result

| Detector | Round 1 | Round 2 | Round 3 | **Median ms per image** | Images per second | × Tintype's time |
|---|---|---|---|---|---|---|
| **Tintype v0.4.0 default** | 2.57 | 2.66 | 2.54 | **2.57** | 389 | 1.0 |
| Community Forensics 384 | 2.94 | 3.04 | 3.04 | 3.04 | 329 | 1.18 |
| SwinV2 detector | 13.58 | 13.52 | 13.38 | 13.52 | 74 | 5.3 |
| GPTTrace flag-pec-v3 | 63.92 | 61.25 | 60.79 | 61.25 | 16 | 23.8 |

**Tintype is faster than each of the other three**, under the registered
rule: each of its three rounds beat each round of every other detector.

- **Against Community Forensics:** the margin is narrow, about 18%. The
  shipped default also runs the Community Forensics network as a second
  opinion on uncertain photos, yet it is still faster than Community
  Forensics alone, because its own network exits early on most photos.
- **Against GPTTrace:** Tintype is about 24 times faster per image.
  GPTTrace's precision is level with Tintype's on both TPB v1 splits.

## Caveats

- **Content of the sample.** It is drawn from TPB v1's sets, many of
  them small classic GAN images. Times per image depend on image size,
  and the website's 300–364 photos/s for Tintype is from a different
  sample (batch 7).
- **One machine.** This says nothing about other hardware.
- **Engine, not release.** Community Forensics and SwinV2 were timed
  through Tintype's engine, not their released PyTorch pipelines. Those
  were slower as released (`docs/BENCHMARK.md`).
