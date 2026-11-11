# Notices

Veracity's code is dual-licensed under MIT ([LICENSE-MIT](LICENSE-MIT)) or
Apache-2.0 ([LICENSE-APACHE](LICENSE-APACHE)), at your option. The trained
weights have their own licence: [models/LICENSE.md](models/LICENSE.md).

Third-party work this project builds on:

- **Community Forensics** (Park & Owens, CVPR 2025; MIT). The network starts
  from its 224 px checkpoint, and the 384 px model is the image baseline.
- **WaveRep** (GRIP-UNINA, NeurIPS 2025; licence: informational and
  non-profit use) and the **SwinV2 detector**
  `haywoodsloan/ai-image-detector-deploy` (Apache-2.0) are reference
  detectors. Their networks are re-implemented here in Rust to run their
  released weights; the weights are downloaded by `veracity data baselines`
  and are not redistributed. WaveRep's weights also serve as the optional
  second opinion for video (`detect --second-opinion`).
- **rust_h264** and **rust_h265** (MIT OR Apache-2.0) decode video where
  there is no system decoder, and **gemm** (MIT) multiplies matrices for the
  CPU engine off macOS. H.264 and HEVC are covered by patents in some
  countries; whether a licence is needed to distribute a build that decodes
  them is for whoever distributes it to check.
- Datasets are fetched from their original hosts and are not redistributed;
  see [docs/DATA.md](docs/DATA.md) for sources and licences.
