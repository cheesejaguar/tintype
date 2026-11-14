# Data audit of the local training stores (diagnostic)

What the training files are, whether their headers alone give the label
away, how often the current recipe draws each generator family, and a
review sample of authentic-labelled items. These are **diagnostics**: they
describe the data on one machine and select nothing. The reading of them,
and the experiments they lead to, are in
[the source and family runbook](../../runbooks/source-family-matching.md).

| | |
|---|---|
| Date | 2026-11-13 and 2026-11-14 |
| Hardware | Apple M6, 12 cores, 32 GB, macOS 27.2; CPU only, no GPU job |
| Software | Python 3.9.6 (standard library only); `tintype` built with `cargo build --release --locked` |
| Commit | `5cca9e7` on `research/ws4-source-family-matching` (scripts and binary) |
| Run from | the directory that holds `data/` and `runs/` (here: a worktree whose `data/store`, `data/vstore` and `data/fresh8` link to the local dataset tree; `runs/v7/manifests.txt` and `runs/scores-v7/calib_images.tsv` are the phase-6 files of that tree) |

Every file below starts with the command that wrote it. The feature files
the commands read (`runs/ws4/features-*.tsv`) are written by
`scripts/audit-manifests.py features`, as the runbook shows; they are not
committed (one row per file, 284,304 rows for the training stores).

| File | What |
|---|---|
| [generator-families.md](generator-families.md) | Generator names and items per decoder family under the starter map, for the 25 phase-6 manifests and for all local photo stores, and what the assignments rest on |
| [phase6-sampling-shares.md](phase6-sampling-shares.md) | `tintype sampling-shares` on the phase-6 manifests: the share of every manifest, family and (largest) generator |
| [family-weighted-sampling-shares.md](family-weighted-sampling-shares.md) | The weights `--rebalance-families 0.5` suggests, and the shares they give (arm B of the runbook) |
| [training-stores-audit.md](training-stores-audit.md) | Format, JPEG settings, size and metadata per class, by store and by generator family |
| [training-stores-probe.md](training-stores-probe.md) | Metadata-only label probe, all header columns |
| [training-stores-probe-encoding.md](training-stores-probe-encoding.md) | The same without the size columns |
| [frontier-audit.md](frontier-audit.md), [frontier-probe.md](frontier-probe.md), [frontier-probe-encoding.md](frontier-probe-encoding.md) | The Frontier 2026-10 images (originals and platform copies): the one evaluation photo set still on disk |
| [evaluation-manifests-by-extension.md](evaluation-manifests-by-extension.md), [evaluation-manifests-probe-extension.md](evaluation-manifests-probe-extension.md) | The evaluation sets whose files are gone, by the file extension in their manifests |
| [authentic-review-sample.md](authentic-review-sample.md) | The review-sample draw on the phase-6 calibration scores: how many items per source and score band. The list itself is not committed and was not reviewed |

## What was and was not read

- Training photo stores of phase 6: 314,282 manifest rows in 16 stores.
  284,304 files in 14 stores were read. `tiny-genimage` (27,979) and
  `stylegan3` (1,999) are no longer on disk. No file that was read had an
  unreadable header.
- Video stores are not audited (the audit reads image headers).
- Evaluation photo sets: only the Frontier 2026-10 images (2,976 files)
  are on disk. AIGIBench CommunityAI, the thirty sets of confirmation
  batch 7 (`data/fresh8`), So-Fake-OOD, Synthbuster+ and the OpenFake dev
  set exist as manifests only, so for them the audit has file extensions
  and nothing else.

## Headline numbers

All from the files above.

- **Families.** 241,312 generated items under 1,222 generator names in the
  phase-6 manifests; 13 names and 51,161 items (21.2%) are `unknown`
  (Reddit posts, anonymised Defactify generators); 59 names and 76,560
  items rest on recalled model documentation.
- **Sampling shares of the phase-6 recipe.** Latent diffusion 72.38% of AI
  draws, closed commercial services 8.12%, unknown 5.34%, GANs 5.33%, open
  video models 5.09%, VQ and autoregressive 1.65%, pixel diffusion 1.31%,
  hybrid 0.57%. The two Community Forensics stores are 66.26% of AI draws.
  `reddit-wild` is 1.14% of AI draws; `real:openfake-reddit` is 4.25% of
  real draws.
- **Files.** In `openfake`, `openfake2`, `commfor-small` and `sofake-set`
  the real class is 67–99% JPEG and the generated class 63–90% PNG.
  `openfake-reddit` is JPEG in both classes, at a median estimated quality
  of 75 (real) against 95 (generated) and 11.84 against 1.56 megapixels.
- **Metadata-only probe, training stores.** AUROC 0.977–1.000 inside each
  two-class store. Pooled: 0.973 [0.938, 0.993] over item folds, 0.726
  [0.551, 0.968] with sources and generators held out, 0.781 [0.613,
  0.970] fitted on the other stores. Without the size columns: 0.735
  [0.602, 0.894], 0.528 [0.347, 0.785] and 0.555 [0.368, 0.773].
- **Metadata-only probe, Frontier originals.** 1.000 [0.999, 1.000]†
  inside the set; 0.961 [0.922, 0.991]† from a model fitted on the
  training stores' headers.
- **Evaluation manifests, extension only.** CommunityAI 1.000; 13 of the
  30 sets of confirmation batch 7 at 0.843 or more.
- **Review sample.** 36,110 authentic-labelled items in the phase-6
  calibration scores, 20 sources; 739 items listed for review; none
  reviewed, nothing relabelled.

## Limits

- An interval marked `†` comes from resampling files, not sources (the
  class had one source or one generator); it is too narrow.
- The probe measures what the files offer a detector, not what the
  released weights use. The network sees 224-pixel crops after an
  augmentation that applies JPEG and resizing to both classes.
- JPEG quality is an estimate: the IJG quality whose luminance table is
  closest. Files with an encoder's own tables get the nearest quality and
  count as not IJG.
- The family map is a starter proposal; a share "by family" is only as
  right as the map.
- Evaluation sets here are archived development data. Nothing in this
  directory is a result on held-out data.
