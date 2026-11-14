# Runbook: match image sources and generator families, then refresh with replay

Workstream 4 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
(§4). It covers four things: keeping related items together, auditing what
the two classes are made of, a labelling policy and a plan for controlled
reconstructions, and explicit sampling weights with a replay pool. The
tooling is built and tested; the audits that need only a CPU have been run
and their numbers are below; the training comparisons are written out and
**have not been run**.

Everything here is tooling, diagnostics and a plan. Nothing in it selects a
model, a threshold or a training mix to ship: that takes the frozen
independent split of [frozen-eval.md](frozen-eval.md), a re-calibration,
an early-exit re-check and a new entry in
[PREREGISTRATION.md](../PREREGISTRATION.md).

## The gate

Written before any of the training commands below is run. It is a
**proposed decision criterion** from the report, not a settled one.

> **New-family macro recall improves without regressing older families or
> reposted images, at fixed false-positive rate**, at 1% FPR and at 0.1% FPR.

Read on the frozen `test` set, at cutoffs frozen on its `calibrate` set,
candidate against a control trained here with the same budget
(`runs/ws4-current`, not the shipped weights):

| Term | Definition |
|---|---|
| Fixed FPR | Each model is read at its own cutoff for 1% and for 0.1% FPR, frozen on the authentic items of `calibrate`. The actual FPR on `test` is reported beside every recall. |
| New families | The generator families named with `--new-family` when the measurement is run: for the matching, weighting and reconstruction arms, the families of the frozen split that no training store holds; for the refresh arms, the families the refresh adds, measured on items of them that were split off before training. They are written into the pre-registration entry before the first score. |
| Macro recall | The unweighted mean of per-family recall over families with at least 100 generated test items. |
| Older families | Every other family with at least 100 generated test items. |
| Reposted images | Generated items whose `transform` column is set (reposts, recompressed and resized copies), and the false-positive rate on authentic items whose `transform` is set. |
| Improves | New-family macro recall is higher at 1% FPR, with the 95% interval of the difference (bootstrap over parents) above zero, and is not lower at 0.1% FPR. |
| Without regressing | At both rates: older-family macro recall and recall on processed copies are not lower by more than one point, and the false-positive rate on processed authentic copies is not higher. |

The one-point tolerance and the 100-item floor are this runbook's choices;
change them in the pre-registration entry, not after a result.
`scripts/family-gate.py` prints every quantity in the table (commands under
[Evaluation](#evaluation)). A candidate that passes is a ship *candidate*
under the selection rule of [frozen-eval.md](frozen-eval.md#the-selection-rule),
nothing more.

## What was measured, where and when

Every number in this file and in
[docs/results/data-audit](../results/data-audit/README.md) was produced on
2026-11-13 and 2026-11-14 on an Apple M6 (12 cores, 32 GB, macOS 27.2),
CPU only, by the commands shown, with Python 3.9.6 and the scripts and
binary of commit `5cca9e7` on branch `research/ws4-source-family-matching`.
They are diagnostics of the local training stores. Two of the sixteen
phase-6 photo stores (`tiny-genimage`, `stylegan3`) and every evaluation
photo set except the Frontier 2026-10 images were no longer on disk, so
their files could not be read; what could still be said about them comes
from their manifests and is marked so.

## 1. Keep related items together

The grouping columns (manifest columns 5 to 9: `parent_id`, `scene_id`,
`source_id`, `generator_family`, `transform`) exist since step 0, and
`split-groups` / `verify-split` use them. The plans still write four-column
manifests. `scripts/group-manifest.py` writes a **grouped copy** of a
manifest: the same rows with the columns filled from

- [docs/groups/generator-families.tsv](../groups/generator-families.tsv):
  generator name → decoder family (generated items);
- [docs/groups/sources.tsv](../groups/sources.tsv): generator key → source
  id (both classes; `reddit` for both classes of `openfake-reddit`);
- an optional side table with per-row `parent_id`, `scene_id`, `transform`
  (for processed copies and reconstructions), and `--self-parent`, which
  gives every original its own id so that copies made later can point at it.

It never changes its input. The evaluation manifests are pinned by SHA-256
in `docs/manifests/*.frozen.tsv.gz`; a grouped copy is a new file,
`<name>.grouped.tsv` beside the input by default. Written there, every
relative path is unchanged, so the item paths, and with them the path-hash
validation split of `tintype train`, are exactly what they were. With
`--out-dir` the copies go elsewhere, their paths are rewritten, and the
path-hash split of those copies is a different one.

```bash
# From the directory that holds data/ and runs/. Nothing existing is rewritten.
python3 -I scripts/group-manifest.py --self-test
python3 -I scripts/group-manifest.py data/store/*/manifest.tsv \
  --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv \
  --self-parent --summary
# -> data/store/<store>/manifest.grouped.tsv beside each manifest

# The same families for the four-column manifests, without writing copies:
# an exact-name table for `tintype train --family-map`.
python3 -I scripts/group-manifest.py $(sed "s/--manifest //g" runs/v7/manifests.txt) \
  --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv \
  --summary --dry-run --write-map runs/ws4/families.tsv
```

**The starter family map.** Families: `latent-diffusion`,
`pixel-diffusion`, `gan`, `autoregressive-vq`, `hybrid`, `video-diffusion`
(open video models), `commercial-unknown` (closed services whose decoder
cannot be checked), `other` (face swaps and feed-forward networks) and
`unknown`. Every row says what it rests on: the name itself, the dataset's
documentation, the model's own documentation *as recalled when the file
was written* (not looked up again; to be confirmed), a closed service, or
nothing. A name that matches no rule is `unknown`; none is guessed.

Measured on the 25 manifests of the phase-6 recipe (second command above;
[full output](../results/data-audit/generator-families.md)): 241,312
generated items under 1,222 generator names.

| Family | Generator names | Items | Share of generated items |
|---|---|---|---|
| latent-diffusion | 1,116 | 81,518 | 33.8% |
| unknown | 13 | 51,161 | 21.2% |
| video-diffusion | 16 | 36,369 | 15.1% |
| gan | 22 | 28,887 | 12.0% |
| commercial-unknown | 44 | 24,146 | 10.0% |
| autoregressive-vq | 5 | 9,807 | 4.1% |
| pixel-diffusion | 3 | 6,991 | 2.9% |
| hybrid | 2 | 2,000 | 0.8% |
| other | 1 | 433 | 0.2% |

| What the assignment rests on | Generator names | Items |
|---|---|---|
| the model's documentation, as recalled (`model-doc`) | 59 | 76,560 |
| nothing (`none`: family `unknown`) | 13 | 51,161 |
| the name | 68 | 46,362 |
| the dataset's documentation (`dataset-doc`) | 1,038 | 43,083 |
| a closed service (`closed`: family `commercial-unknown`) | 44 | 24,146 |

Over all twenty local photo stores (`data/store/*/manifest.tsv`, which adds
four evaluation stores and leaves out the video stores): 218,076 generated
items under 1,245 names, of which 16 names and 57,252 items (26.3%) are
`unknown`. The unknown items are almost all `reddit-wild` (15,782 posts
whose generator is not recorded), the five anonymised Defactify labels
(35,000) and, among the evaluation stores, AIGIBench CommunityAI (6,000).

Two things to keep in mind about this map. The 1,037 hub repositories of
Community Forensics (`owner/name`) are `latent-diffusion` on the strength
of that dataset's description of how it collected them, not of their
names. And 59 names (76,560 items) rest on recalled model documentation:
they should be confirmed before a family is used as a hold-out or a report
slice.

## 2. Balance and audit the data

### What the files are

`scripts/audit-manifests.py` reads the header of every file, never a pixel:
container format by magic bytes, JPEG quality estimated from the
quantisation tables (the IJG quality whose luminance table is closest, and
whether the table is exactly an IJG one), chroma subsampling, dimensions,
EXIF, ICC and PNG text chunks. Standard library only (Pillow is not used).

```bash
python3 -I scripts/audit-manifests.py --self-test
python3 -I scripts/audit-manifests.py features \
  $(sed "s/--manifest //g" runs/v7/manifests.txt | tr ' ' '\n' | grep data/store) \
  --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv \
  --out runs/ws4/features-train.tsv
python3 -I scripts/audit-manifests.py summary runs/ws4/features-train.tsv --by store,family
```

Result on the sixteen phase-6 photo stores: 314,282 manifest rows, 284,304
files read (42 s), 29,978 not on disk, none of the files read had an
unreadable header. [Full tables](../results/data-audit/training-stores-audit.md);
the rows that matter most:

| Store | Class | Files | JPEG | PNG | JPEG quality p50 | Short side p50 | Megapixels p50 | Both sides multiples of 64 |
|---|---|---|---|---|---|---|---|---|
| commfor-small | real | 23,936 | 87.5% | 12.5% | 75 | 478 | 0.31 | 6.4% |
| commfor-small | AI | 35,905 | 17.3% | 82.7% | 75 | 256 | 0.07 | 100.0% |
| commfor-small2 | real | 29,025 | 29.8% | 70.2% | 75 | 1024 | 1.05 | 58.6% |
| commfor-small2 | AI | 44,885 | 0.0% | 100.0% | – | 512 | 0.26 | 100.0% |
| defactify | real | 7,000 | 100.0% | 0.0% | 75 | 428 | 0.27 | 2.1% |
| defactify | AI | 35,000 | 100.0% | 0.0% | 75 | 768 | 0.59 | 60.0% |
| openfake | real | 12,005 | 99.0% | 0.7% | 95 | 1160 | 1.92 | 7.1% |
| openfake | AI | 11,923 | 34.5% | 63.0% | 75 | 768 | 1.01 | 84.1% |
| openfake-reddit | real | 3,018 | 97.9% | 2.1% | 75 | 2976 | 11.84 | 7.1% |
| openfake-reddit | AI | 15,782 | 95.1% | 4.9% | 95 | 1080 | 1.56 | 3.4% |
| sofake-set | real | 3,002 | 66.9% | 33.1% | 91 | 512 | 0.26 | 41.9% |
| sofake-set | AI | 4,263 | 10.2% | 89.8% | 75 | 768 | 0.59 | 98.8% |

What this shows, and what it does not:

- **The classes differ in file format in most two-class stores.** Real
  items are mostly JPEG and generated items mostly PNG in `openfake`,
  `openfake2`, `commfor-small` and `sofake-set`. In `commfor-small2` it is
  the other way round for the real class (70% PNG) and every generated
  item is PNG.
- **Where the format matches, something else differs.** Both classes of
  `openfake-reddit` are JPEG, but the real items have a median estimated
  quality of 75 and 11.8 megapixels and the generated ones 95 and 1.6.
  Both classes of `defactify` are JPEG at quality 75, with different sizes.
- **Generated items are overwhelmingly sized in multiples of 64**
  (84–100% in most stores); real items mostly are not.
- EXIF is rare in every store (at most 6.1% of a class), so it separates
  little here.
- The training augmentation applies JPEG and resizing to both classes with
  the same probabilities, which blunts these differences but does not
  remove them: with the phase-6 settings (messenger 0.25, JPEG before and
  after the crop 0.25 each, `src/data.rs`) 0.75³ = 42% of samples get no
  added JPEG step, and then a PNG and a quality-75 JPEG reach the network
  as different inputs.
- `tiny-genimage` and `stylegan3` could not be read. Research note 09
  names `tiny-genimage` (GenImage: JPEG reals, PNG fakes) as the first
  store to audit; it has to be fetched again for that.

### Does metadata alone predict the label?

`scripts/metadata-probe.py` fits a logistic regression on the audit's
columns only and reports AUROC three ways: over **item folds** inside a
set; over **group folds** that hold out real sources and AI generators
whole; and with each store scored by a model **fitted on the other
stores**. Intervals are a bootstrap over clusters (a real source, an AI
generator), not files; `†` marks an interval where a class had a single
cluster and its files were resampled, which makes it too narrow. Only
files scored by the same model are ranked against each other.

```bash
python3 -I scripts/metadata-probe.py --self-test
python3 -I scripts/metadata-probe.py runs/ws4/features-train.tsv
python3 -I scripts/metadata-probe.py runs/ws4/features-train.tsv --columns encoding
```

Result on the 284,304 files read (98,690 real from 11 sources, 185,614
generated from 1,201 generators; about two minutes per run on one core).
Full output: [all header columns](../results/data-audit/training-stores-probe.md),
[encoding columns only](../results/data-audit/training-stores-probe-encoding.md)
(the header columns without the picture's size: format, JPEG quality,
tables, subsampling, progressive, EXIF, ICC, JFIF, XMP, C2PA, PNG text,
bytes per pixel).

| Store | All header columns: item folds | group folds | fitted on the other stores | Encoding columns only: item folds | fitted on the other stores |
|---|---|---|---|---|---|
| commfor-small | 1.000 [0.998, 1.000] | 1.000 [0.999, 1.000] | 0.998 [0.991, 1.000] | 0.859 [0.473, 1.000] | 0.855 [0.555, 1.000] |
| commfor-small2 | 0.982 [0.955, 0.999]† | – | 0.552 [0.503, 0.591]† | 0.756 [0.735, 0.774]† | 0.543 [0.527, 0.566]† |
| defactify | 1.000 [1.000, 1.000]† | – | 0.983 [0.962, 1.000]† | 0.616 [0.442, 0.741]† | 0.498 [0.425, 0.573]† |
| openfake | 0.999 [0.997, 1.000] | 0.965 [0.934, 0.987] | 0.943 [0.915, 0.973] | 0.903 [0.805, 0.964] | 0.586 [0.198, 0.934] |
| openfake-reddit | 0.996 [0.995, 0.996]† | – | 0.943 [0.937, 0.947]† | 0.980 [0.978, 0.981]† | 0.358 [0.342, 0.373]† |
| openfake2 | 0.999 [0.997, 1.000] | 0.968 [0.925, 0.995] | 0.973 [0.955, 0.990] | 0.903 [0.815, 0.965] | 0.632 [0.266, 0.931] |
| sofake-set | 0.977 [0.944, 0.998]† | – | 0.721 [0.651, 0.774]† | 0.818 [0.753, 0.859]† | 0.778 [0.685, 0.829]† |
| **Pooled, 14 stores** | **0.973 [0.938, 0.993]** | **0.726 [0.551, 0.968]** | **0.781 [0.613, 0.970]** | **0.735 [0.602, 0.894]** | **0.555 [0.368, 0.773]** |

(The seven single-class stores have no AUROC of their own and take part
in the pooled rows. Pooled group folds with the encoding columns only:
0.528 [0.347, 0.785]. The pooled "other stores" figures rank the scores of
the per-store models together, because a single-class store can only be
compared with files of other stores.)

| Column alone, pooled | AUROC, item folds | AUROC, group folds |
|---|---|---|
| exact size (width × height) | 0.957 | 0.840 |
| long side | 0.908 | 0.753 |
| short side | 0.848 | 0.727 |
| aspect ratio | 0.829 | 0.775 |
| sides multiples of 64 / 16 / 8 | 0.768 | 0.699 |
| megapixels | 0.716 | 0.570 |
| JPEG quality | 0.702 | 0.588 |
| bytes per pixel | 0.674 | 0.605 |
| container format | 0.659 | 0.604 |
| EXIF present | 0.504 | 0.493 |

Reading it:

- **Inside every training store that holds both classes, the label can be
  read off the file header**: AUROC 0.977 to 1.000 over item folds.
- **The picture's size carries most of it, not the format.** Exact
  dimensions alone give 0.957 pooled; the container format alone 0.659.
  The size rules also survive holding out sources and generators (0.726)
  and whole stores (0.781), with wide intervals: there are only 11 real
  sources.
- **The encoding shortcuts are real inside a store and do not agree
  between stores.** Without the size columns the probe still reaches
  0.616 to 0.980 inside a store, but a model fitted on the other stores is
  at chance pooled (0.555 [0.368, 0.773]) and *below* chance on
  `openfake-reddit` (0.358): there the real items are the low-quality
  JPEGs, the reverse of the other stores.
- **What this does and does not show.** It shows what the training files
  offer a detector, not what the detector took. The network never sees
  width and height as numbers; it sees 224-pixel crops, in which size
  shows up as resampling and as the scale of the content, and format as
  compression traces, after an augmentation that hides part of both.
  Whether the shipped weights use these cues is a different measurement:
  score the same pictures re-saved at a common size, format and quality
  (a GPU job for the per-operation audit of workstream 1). What the probe
  does establish is that nothing in the data stops a detector from
  learning them, and that an evaluation set with the same regularities
  would reward it.

### Evaluation sets

Only the Frontier 2026-10 images (`data/vstore/wasitai`) are still on disk
among the evaluation photo sets, as originals and as WhatsApp, Instagram
and re-shared copies of the same pictures.

```bash
python3 -I scripts/audit-manifests.py features data/vstore/wasitai/images.tsv data/vstore/wasitai/images_social.tsv \
  --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv --out runs/ws4/features-frontier.tsv
python3 -I scripts/audit-manifests.py summary runs/ws4/features-frontier.tsv --by dataset
python3 -I scripts/metadata-probe.py runs/ws4/features-frontier.tsv --by dataset --fit-on runs/ws4/features-train.tsv
python3 -I scripts/metadata-probe.py runs/ws4/features-frontier.tsv --by dataset --fit-on runs/ws4/features-train.tsv --columns encoding
```

[Audit](../results/data-audit/frontier-audit.md),
[probe](../results/data-audit/frontier-probe.md),
[probe, encoding columns](../results/data-audit/frontier-probe-encoding.md).
486 real and 261 generated originals (19 generators), and 482 + 261 of each
social copy. The originals: every real file is a JPEG (median 12
megapixels), 68.6% of the generated ones are PNG and 10.7% WebP (median
1.05 megapixels). The copies are all JPEG at one quality per platform
(WhatsApp 70, Instagram 80, re-shared 75).

| Set | All header columns, item folds | Encoding columns only, item folds | Model fitted on the training stores: all columns | encoding only |
|---|---|---|---|---|
| Originals | 1.000 [0.999, 1.000]† | 0.998 [0.994, 1.000]† | 0.961 [0.922, 0.991]† | 0.959 [0.924, 0.992]† |
| WhatsApp copies | 0.971 [0.940, 0.996]† | 0.730 [0.632, 0.823]† | 0.842 [0.771, 0.904]† | 0.582 [0.481, 0.679]† |
| Instagram copies | 0.936 [0.884, 0.977]† | 0.768 [0.701, 0.830]† | 0.767 [0.665, 0.861]† | 0.619 [0.531, 0.716]† |
| Re-shared copies | 0.968 [0.933, 0.995]† | 0.756 [0.671, 0.843]† | 0.595 [0.524, 0.667]† | 0.589 [0.497, 0.679]† |

- On the **originals** the header gives the label away completely, and so
  does a model that has only ever seen the training stores' headers
  (0.961): the regularities of the training files recur in this
  benchmark. EVIDENCE.md already notes the label and format imbalance of
  these originals; this puts a number on it.
- The **platform copies** remove the format and quality difference (one
  JPEG quality for both classes), and the encoding columns fall to 0.73 to
  0.77, carried by bytes per pixel. Size and shape still separate the
  classes (0.94 to 0.97): a platform resizes, but a square generated
  picture stays square.
- Every interval here is marked `†`: there is one real source, so the
  real files, not sources, were resampled, and the intervals are too
  narrow. The four sets are the same pictures, so nothing here is
  independent across them. 747 pictures is a small set.

For the sets that are gone (AIGIBench CommunityAI, the thirty sets of
confirmation batch 7, So-Fake-OOD, Synthbuster+, the OpenFake dev set) the
manifest still gives each file's extension, which is all that can be said:

```bash
python3 -I scripts/audit-manifests.py features data/store/aigibench/manifest.tsv data/store/dev-openfake-test/manifest.tsv \
  data/store/sofake-ood/manifest.tsv data/store/synthbuster/manifest.tsv data/fresh8/*/manifest.tsv \
  --out runs/ws4/features-eval-manifests.tsv
python3 -I scripts/audit-manifests.py summary runs/ws4/features-eval-manifests.tsv --by-extension --by store
python3 -I scripts/metadata-probe.py runs/ws4/features-eval-manifests.tsv --columns extension
```

[Extensions](../results/data-audit/evaluation-manifests-by-extension.md),
[probe on the extension alone](../results/data-audit/evaluation-manifests-probe-extension.md);
52,154 manifest rows, none on disk.

| Set | AUROC from the file extension alone (item folds) |
|---|---|
| AIGIBench CommunityAI (6,000 + 6,000) | 1.000: every real file is `.jpg`, every generated file `.png` |
| Batch 7, AIGIBench BLIP | 1.000 |
| Batch 7, AIGIBench DALLE-3, FLUX1-dev, GLIDE, IP-Adapter, Imagen3, Infinite-ID, InstantID, PhotoMaker, R3GAN, SD3, StyleGAN-XL, StyleSwim | 0.843 to 0.913 |
| Batch 7, AIGIBench Midjourney | 0.506 |
| Batch 7, AIGIBench ProGAN, StyleGAN3, WFIR and the eleven classic sets | 0.500 (one extension for both classes) |
| So-Fake-OOD (benchmark and batch 7 samples) | 0.764 and 0.766 |
| OpenFake dev set | 0.627 |
| Synthbuster+ | 0.539 |
| Pooled, 34 manifests | 0.780 [0.634, 0.911] |

The known caveat about CommunityAI (every real JPEG, every AI PNG) is
confirmed from its manifest, and it is not alone: in 13 of the 30 sets of
confirmation batch 7 the file extension by itself separates the classes
with AUROC 0.84 or more. An extension is less than a header, so these are
lower bounds on what the files offered; the files themselves are gone and
could not be read. These sets are archived development data; this adds a
reason to read their numbers as such.

### Authentic-label review sampling

`scripts/sample-authentic-review.py` writes a list of authentic-labelled
items for a person to look at: a few from every source and score band,
plus the highest-scoring items overall and per source. The strata give a
contamination rate per source; the tail finds what the detector already
suspects. **It relabels nothing.** The draw is fixed by `--seed`.

```bash
python3 -I scripts/sample-authentic-review.py --self-test
python3 -I scripts/sample-authentic-review.py --scores runs/scores-v7/calib_images.tsv \
  --out runs/ws4/authentic-review.tsv --per-stratum 5 --tail 200 --tail-per-source 20 --seed 1
```

Run on the phase-6 calibration score file (83,031 rows, 36,110 of them
authentic, 20 sources): 95 source-by-band strata, and a list of **739
items** (414 from the strata, 200 in the overall tail, 400 in a source's
tail; an item can be several). [Per-source table](../results/data-audit/authentic-review-sample.md).
The list was not reviewed: most of the files it names are evaluation
files that are no longer on disk. Before the next training run, score the
real items of every training store (a GPU job: commands under
[Ready to run](#ready-to-run)), draw the list from those scores, review
it, and record the contamination found per source. What to do with an item
a reviewer marks as generated is a policy point for the maintainer
([below](#policy-points-for-the-maintainer)).

## 3. The sampling shares of the current recipe

`tintype sampling-shares` prints, from the sampler's own weights, how
often each manifest, generator family and generator is drawn. It takes the
manifests and sampling flags of `train`, applies the same path-hash
validation split, and runs on every platform. `tintype train
--print-sampling` prints the same and stops.

```bash
tintype sampling-shares $(cat runs/v7/manifests.txt) --family-map runs/ws4/families.tsv --top 25
```

Its first line reproduces the phase-6 training log exactly: `train 356038
images (16 real sources, 1222 generators), val pool 11025`.
[Full output](../results/data-audit/phase6-sampling-shares.md). Each class
is half of all draws; a share below is of the draws of its class.

| Generator family | Items | Share of AI draws | Lift over its share of AI items |
|---|---|---|---|
| latent-diffusion | 79,100 | 72.38% | 2.14 |
| commercial-unknown | 23,389 | 8.12% | 0.81 |
| unknown | 49,551 | 5.34% | 0.25 |
| gan | 27,982 | 5.33% | 0.45 |
| video-diffusion | 35,258 | 5.09% | 0.34 |
| autoregressive-vq | 9,520 | 1.65% | 0.41 |
| pixel-diffusion | 6,768 | 1.31% | 0.45 |
| hybrid | 1,933 | 0.57% | 0.70 |
| other | 418 | 0.19% | 1.06 |

| Manifest | Real items | Share of real draws | AI items | Share of AI draws |
|---|---|---|---|---|
| commfor-small2 | 28,197 | 13.27% | 43,525 | 49.60% |
| commfor-small | 23,290 | 18.97% | 34,834 | 16.66% |
| frontier | 0 | – | 17,558 | 6.28% |
| openfake | 11,669 | 9.82% | 11,571 | 5.39% |
| defactify | 6,810 | 6.52% | 33,891 | 3.80% |
| openfake-reddit | 2,896 | 4.25% | 15,292 | 1.14% |

- **Two thirds of all AI draws (66.3%) are Community Forensics images**,
  and 72% are latent diffusion. `count^0.5` lifts every small generator,
  and that collection has about 1,040 of them at about 50 images each:
  what was meant to lift rare generators hands most of the class to one
  collection and one decoder family.
- **`reddit-wild` is 1.14% of AI draws and `real:openfake-reddit` 4.25% of
  real draws.** The fact sheet's estimate holds: per draw, Reddit-processed
  real photos are seen almost four times as often as Reddit-processed
  generated ones, the opposite of what the raw counts (15.8 k against
  3.0 k) suggest.
- Autoregressive, VQ and hybrid generators together are 2.2% of AI draws.

## 4. Sampler weights and replay

New flags of `tintype train` (and of `sampling-shares`). With none of them
the sampler is built by `Sampler::new` and draws the same sequence as
before for the same seed; `data::tests::the_default_sampler_draws_exactly_what_it_always_drew`
compares it, draw for draw, with a verbatim copy of the old sampler.

| Flag | Default | Effect |
|---|---|---|
| `--manifest-weight MANIFEST=W` (repeatable) | none | Multiplies the sampling mass of every item of a manifest, named by its path as given or by its store directory |
| `--family-weight FAMILY=W` (repeatable) | none | Multiplies the mass of every item of a generator family: column 8, else `--family-map`, else the generator name (`real:<source>` for real items) |
| `--weights-file FILE` | none | Tab-separated `manifest\|family  name  weight` rows; the two flags above override it for the same name |
| `--family-map FILE` | none | Tab-separated `generator  family`: names the family of generators whose manifests have no column 8. Changes no draw by itself |
| `--replay-manifest FILE` (repeatable) | none | The items of these manifests form the replay pool |
| `--replay-share S` | 0.2 | The share of draws taken from the replay pool; no effect without a replay manifest |
| `--print-sampling` (`train` only) | off | Print the shares and stop |
| `--rebalance-families GAMMA --write-weights FILE` (`sampling-shares` only) | off | Write family weights that move each AI family's share of AI draws to `share^GAMMA`, renormalised |

An item's sampling mass is `count(generator)^(alpha-1) × manifest weight ×
family weight`; a generator is drawn in proportion to the mass of its
items and an item inside it in proportion to its weight. A weight of 0
removes items; a weight that names no manifest or family is an error.
With a replay pool, one more random draw decides the pool; a class that
only one pool holds is always drawn from that pool. Replay manifests take
part in the path-hash validation split like any other manifest.

## 5. Controlled reconstructions and the labelling policy

Nothing is generated or downloaded by this workstream. This section is the
plan, the manifest tooling for it, and a **proposed** labelling policy for
the maintainer to confirm (also in [DATA.md](../DATA.md#proposed-labelling-policy)).

### Reconstructions

A reconstruction is an authentic photo passed through the final decoder of
a generator family and nothing else: encode, decode, save. Content, framing
and size match the source exactly, so the only difference between the pair
is the decoder's trace. The research notes give the case for it (aligned
reconstructions raised recall after post-processing where independent
fakes did not; training on a few decoder categories transferred to unseen
generators) and its limits (a 4-channel VAE's reconstructions barely
transfer to a 16-channel one; a format present in only one class becomes
the shortcut instead).

| Decoder family | Examples named in the notes | Licence |
|---|---|---|
| 4-channel image VAE | the Stable Diffusion 1.x / SDXL autoencoders | to check at the source before download |
| 16-channel image VAE | the SD3 and FLUX autoencoders | to check at the source before download |
| VQ and bitwise tokenizers | the LlamaGen, Janus-Pro and Infinity tokenizers | to check at the source before download |
| Learned upsamplers | a Real-ESRGAN- or EDSR-class model, ×2 and ×4 then back to size | to check at the source before download |

Steps, in order:

1. **Licence check, before any download.** For each autoencoder, tokenizer
   and upsampler: read its licence at the source, record the licence, the
   revision and where it is stated in [DATA.md](../DATA.md) and
   [NOTICE.md](../../NOTICE.md), and confirm it allows this use
   (generating training data for a detector whose weights are released). A
   component whose licence does not is dropped. None has been checked or
   downloaded.
2. **List the parents.** `scripts/list-reconstruction-parents.py` draws a
   fixed number of authentic images per source and writes them with the ids
   their reconstructions must keep. It skips every image the trainer's
   path-hash split holds out for validation, so that no validation content
   comes back into training as a reconstruction.

   ```bash
   python3 -I scripts/list-reconstruction-parents.py --self-test
   python3 -I scripts/list-reconstruction-parents.py \
     $(sed "s/--manifest //g" runs/v7/manifests.txt | tr ' ' '\n' | grep data/store) \
     --sources docs/groups/sources.tsv --per-source 500 --seed 1 --out data/recon/parents.tsv
   ```

   On the local stores this lists 5,500 parents from 11 real sources
   (3,236 real images were not eligible because validation holds them out).
3. **Reconstruct, outside this repository.** For every parent and every
   decoder: decode the parent, pass it through the decoder at its native
   size, and save the result **in the parent's format, at the parent's
   size, and for a JPEG parent at the parent's quality** (the audit's
   `jpeg_q` column gives it). A reconstruction saved as PNG next to JPEG
   parents would teach the file format, which is the shortcut the probe
   above measures. Independent recent diffusion, autoregressive and hybrid
   outputs go beside the reconstructions as ordinary generated stores; they
   are not paired with a parent.
4. **Emit a manifest the trainer reads**, one store per decoder
   (`data/store/recon-<decoder>/manifest.tsv`), each row with all nine
   columns:

   | Column | Value |
   |---|---|
   | label | per the policy below (proposed: 1) |
   | generator | `recon:<decoder>`, e.g. `recon:sd-vae-4ch` |
   | dataset | `recon-<decoder>` |
   | `parent_id`, `scene_id`, `source_id` | copied from the parent's row in `data/recon/parents.tsv` |
   | `generator_family` | `reconstruction-vae4`, `reconstruction-vae16`, `reconstruction-vq`, `reconstruction-upsampler` |
   | `transform` | `recon-<decoder>` |

5. **Check before training**: run `audit-manifests.py` and
   `metadata-probe.py` on the reconstructions together with their parents.
   The probe's AUROC over the pair should be near 0.5; if it is not, the
   reconstructions differ from their parents in something other than
   pixels, and that is fixed before they are trained on.

### Proposed labelling policy

To be confirmed by the maintainer; nothing in the code depends on it yet.
The principle: **the label says whether the picture's content was
generated; how a file was processed is recorded beside the label, in
`transform` and `generator_family`, never folded into it.** Every case
below is a named slice, so it can be reported by itself, weighted, or held
out whole.

| Case | Proposed label | `generator_family` | `transform` | Why |
|---|---|---|---|---|
| Fully generated image, any generator | 1 | the decoder family | empty, or the processing applied | What the detector is for |
| Authentic photo, camera or phone pipeline (HDR, night mode, portrait blur, denoising in the device) | 0 | empty | empty | Computational photography is what a photo is now; these are the authentic class |
| Authentic photo through a standard or learned **codec** (JPEG, WebP, AVIF, JPEG AI and other neural codecs) | 0 | empty | `codec-<name>` | A codec is a file format. JPEG AI is a camera and web format to come, and 110 of the 155 authentic NTIRE images the shipped model flagged had been through JPEG AI, Cheng2020 or WMForger (docs/EVIDENCE.md) |
| Authentic photo, platform repost (resize and recompression) | 0 | empty | `repost-<...>` | Content unchanged |
| Authentic photo, AI-upscaled or AI-denoised with the content kept (learned super-resolution, restoration) | 0 | `enhancement-upsampler` / `enhancement-restoration` | `sr-x2`, `denoise-<name>` | Content-preserving processing of a real capture. Research note 09 cites a study in which learned super-resolution of pristine frames raised detectors' false-positive rates several-fold; calling these generated would make that failure the target |
| Authentic photo round-tripped through a generator's autoencoder or tokenizer (a "laundered" photo; the controlled reconstructions above) | **1**, as its own slice | `reconstruction-<decoder>` | `recon-<decoder>` | This is the open point. Proposed 1 because that is what makes reconstructions a training signal for the decoder trace, and because the whole image has passed through a generator's decoder. The cost is stated below |
| Authentic photo with a generated region (inpainting, object removal, generative fill) | not used for the full-generation label | `partial-edit` | `inpaint-<name>` | Local edits are out of scope (README, Limits). A latent-diffusion edit passes the whole frame through a VAE, so such files look generated to this detector; they belong to workstream 9, with their own supervision |
| Face swap or reenactment | not used for the full-generation label | `other` | – | Out of scope; today `OpenForensics_fake` (433 items) is trained on as label 1 |

The tension, plainly: a VAE round trip and a learned codec are the same
kind of operation (an encoder and a decoder with the content kept), and
the policy labels one 1 and the other 0. The line proposed is the
**purpose of the component**: a codec or enhancer shipped to store or
improve a capture is processing; a generator's decoder is generation, even
when it is fed a real photo. That line is defensible for training
(reconstructions are useful only as positives) and uncomfortable for
users (a real photo laundered through a VAE is still a real scene). Two
ways to live with it, both kept open by recording the slice:

- train with reconstructions as label 1, and **report them as their own
  slice**, never inside "recall on generated images";
- or, later, give the model a third output for "authentic content,
  generator-processed" (research note 04), for which the same rows are
  the training data.

Because every case is a `transform` / family slice, changing a label later
is a manifest rewrite of one slice, and holding a slice out is one
`--hold-out-family`.

## 6. Equal-budget training mixes

**Not run.** These use the Apple GPU for about an hour each (phase 6 ran
at about 80 images per second: 8,000 steps of 32). Every arm has the same
budget, starting weights, seed and augmentation as phase 6, and every arm
is compared with the control trained here, not with the shipped weights
(the trainer's video decoder and encoder have changed since phase 6, and
two stores may be missing, so the shipped weights are not this recipe's
output any more).

Checked against the phase-6 run before writing (`runs/v7/train.log`,
`runs/v7/manifests.txt`, README "Reproduce"): 8,000 steps; learning rate
2e-5 with a 300-step warmup (the log's `lr 3.33e-6` at step 50 is
`2e-5 × 50 / 300`); the 25 manifests below; started from the phase-3
weights `runs/v4/step_007000.safetensors`. The batch size (32) and the
messenger probability (0.25) are the command-line defaults and are not
recorded in the log.

```
data/store/{openfake,openfake2,openfake-reddit,frontier,frontier-heldout,nano-banana,defactify,
            commfor-small,openfake-test-reals,sofake-set,commfor-small2,tiny-genimage,stylegan3,
            tpdne,ip-adapter,celeba-hq}/manifest.tsv
data/vstore/{vidguard,genbuster,genvideo100k,aigvd,k400,pexels,k400b,pexelsb,aigvd2}/manifest.tsv
```

`runs/v7/manifests.txt` holds them as `--manifest <path>` pairs. Before
starting: `tiny-genimage` and `stylegan3` are not on disk here. Either
fetch them again (`tintype data phase6`; a download, so the maintainer's
call) or drop both from the list **for every arm, the control included**,
and say so in the log. Use one list for all arms.

```bash
# 0. The shares each arm will train with, before any GPU time (CPU, seconds).
python3 -I scripts/group-manifest.py $(sed "s/--manifest //g" runs/v7/manifests.txt) \
  --families docs/groups/generator-families.tsv --dry-run --write-map runs/ws4/families.tsv
tintype sampling-shares $(cat runs/v7/manifests.txt) --family-map runs/ws4/families.tsv \
  --rebalance-families 0.5 --write-weights runs/ws4/family-weights.tsv

# A. Control: the current mix.
tintype train --init runs/v4/step_007000.safetensors $(cat runs/v7/manifests.txt) \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --p-messenger 0.25 --seed 1 \
  --out runs/ws4-current

# B. Family-weighted: each AI family's share moved to the square root of what it is
#    (face swaps, family `other`, left as they are).
tintype train --init runs/v4/step_007000.safetensors $(cat runs/v7/manifests.txt) \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --p-messenger 0.25 --seed 1 \
  --family-map runs/ws4/families.tsv --weights-file runs/ws4/family-weights.tsv --family-weight other=1 \
  --out runs/ws4-family

# C. Source-matched: Reddit-processed images drawn as often in the AI class as in the real class.
tintype train --init runs/v4/step_007000.safetensors $(cat runs/v7/manifests.txt) \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --p-messenger 0.25 --seed 1 \
  --family-weight reddit-wild=3.85 \
  --out runs/ws4-source

# D. Current mix plus reconstructions (after section 5 has produced the stores).
#    <W> gives the reconstruction stores a stated share of AI draws: print the shares with
#    weight 1, then W = t(1-s) / (s(1-t)) for their share s and the target t (proposed: 10%).
tintype train --init runs/v4/step_007000.safetensors $(cat runs/v7/manifests.txt) \
  --manifest data/store/recon-<decoder-1>/manifest.tsv --manifest data/store/recon-<decoder-2>/manifest.tsv \
  --manifest-weight recon-<decoder-1>=<W> --manifest-weight recon-<decoder-2>=<W> \
  --steps 8000 --batch 32 --lr 2e-5 --warmup 300 --p-messenger 0.25 --seed 1 \
  --out runs/ws4-recon
```

Append `--print-sampling` to any of these to see its shares without
training. Measured for arms B and C on the local manifests
([arm B output](../results/data-audit/family-weighted-sampling-shares.md)):

| Share of AI draws | A. current | B. family-weighted | C. source-matched |
|---|---|---|---|
| latent-diffusion | 72.38% | 39.64% | – |
| commercial-unknown | 8.12% | 13.28% | – |
| unknown (incl. `reddit-wild`) | 5.34% | 10.77% | – |
| gan | 5.33% | 10.76% | – |
| video-diffusion | 5.09% | 10.51% | – |
| autoregressive-vq | 1.65% | 5.99% | – |
| pixel-diffusion | 1.31% | 5.33% | – |
| hybrid | 0.57% | 3.53% | – |
| `reddit-wild` alone | 1.14% | 2.30% | 4.26% |
| `commfor-small` + `commfor-small2` | 66.26% | 46.88% | – |

(Arm C changes only `reddit-wild`, to 4.26% of AI draws against 4.25% of
real draws for `real:openfake-reddit`; its family shares were not
tabulated.) In arm B the 1,933 hybrid items are drawn 4.3 times as often
as their count: about 4,500 draws over 8,000 steps, a little over two
views each.

### Refresh with a replay pool

A refresh adds new generators to a trained model in a short run. Three
arms with one budget, all starting from the control `runs/ws4-current`:
new data only; new data with a replay pool; new data with every old store
(what the phases have done so far). `<new>` are the manifests of the new
generators and whatever new authentic sources come with them. None exists
locally: they are new downloads, each with a licence check first, and they
must not be sources or families of the frozen independent split. Before
training, split `<new>` by group and train only on its `fit` part; the
`test` part is where new-family recall is read:

```bash
tintype split-groups $(printf -- '--manifest %s ' <new manifests>) --out-dir data/ws4-refresh \
  --fractions 0.7,0,0.3 --seed 1 --together parent,scene

# The replay buffer: 25 images per old generator, 300 per old real source, and every
# Reddit-posted image (CPU; nothing is copied).
python3 -I scripts/build-replay-manifest.py --self-test
python3 -I scripts/build-replay-manifest.py \
  $(sed "s/--manifest //g" runs/v7/manifests.txt | tr ' ' '\n' | grep -v openfake-reddit) \
  --repost-manifest data/store/openfake-reddit/manifest.tsv \
  --families docs/groups/generator-families.tsv --per-generator 25 --per-source 300 --seed 1 \
  --out data/replay/replay.tsv
tintype freeze-set --manifest data/replay/replay.tsv --out runs/ws4/replay.frozen.tsv.gz

# R0. New data only.
tintype train --init runs/ws4-current/step_008000.safetensors --manifest data/ws4-refresh/fit.tsv \
  --steps 2000 --batch 32 --lr 1e-5 --warmup 100 --p-messenger 0.25 --seed 1 --eval-every 500 \
  --out runs/ws4-refresh-new

# R1. New data, a fifth of the draws from the replay buffer.
tintype train --init runs/ws4-current/step_008000.safetensors --manifest data/ws4-refresh/fit.tsv \
  --replay-manifest data/replay/replay.tsv --replay-share 0.2 \
  --steps 2000 --batch 32 --lr 1e-5 --warmup 100 --p-messenger 0.25 --seed 1 --eval-every 500 \
  --out runs/ws4-refresh-replay

# R2. New data and every old store, as the phases have done.
tintype train --init runs/ws4-current/step_008000.safetensors --manifest data/ws4-refresh/fit.tsv \
  $(cat runs/v7/manifests.txt) \
  --steps 2000 --batch 32 --lr 1e-5 --warmup 100 --p-messenger 0.25 --seed 1 --eval-every 500 \
  --out runs/ws4-refresh-full
```

The refresh budget (2,000 steps at 1e-5 with a 100-step warmup) and the
replay share (0.2) are this runbook's proposal, from the range the
research notes give (1–2 k steps, a buffer of 50–200 images per old
generator and source); they are not taken from an earlier run. R0 needs
both classes in `<new>`: if the refresh brings no new authentic source,
add the real-only stores (`openfake-test-reals`, `celeba-hq`) to R0, and
to R1 and R2 as well, and say so.

Built on the local manifests, the buffer above is 53,192 rows (7,518 real
from 16 sources, 45,674 generated): 29,892 from the per-generator draw,
4,500 from the per-source draw and all 18,800 rows of `openfake-reddit`.
With 100 per generator it is 80,379 rows; the roughly 1,040 hub generators
of Community Forensics are what makes it large.

## Evaluation

Only on the frozen split of [frozen-eval.md](frozen-eval.md), which does
not exist yet. Until it does, no arm can be judged; the archived suites
(confirmation 7, Arena, Frontier, NTIRE, DaVODi, confirmation 4) may be
scored as regression diagnostics and reported beside the control, never
used to choose.

```bash
# Per arm: A = runs/ws4-<arm>/step_008000.safetensors (R arms: step_002000). Apple GPU.
tintype score --weights $A --manifest data/frozen-eval/calibrate.tsv --out runs/ws4-<arm>/calibrate.tsv
tintype score --weights $A --manifest data/frozen-eval/test.tsv      --out runs/ws4-<arm>/test.tsv
tintype verify-models                       # the released files are untouched

# The cutoffs for 1% and 0.1% FPR, frozen on the calibration set: the "Cutoff" column.
tintype report --operating-points --system "<arm>=runs/ws4-<arm>/calibrate.tsv@0.5" --fpr 0.001,0.01

# The gate: new-family and older-family macro recall, processed copies, FPR, at those cutoffs.
python3 -I scripts/family-gate.py --self-test
python3 -I scripts/family-gate.py --groups data/frozen-eval/test.tsv \
  --system "current=runs/ws4-current/test.tsv@<cutoff 1%>,<cutoff 0.1%>" \
  --system "<arm>=runs/ws4-<arm>/test.tsv@<cutoff 1%>,<cutoff 0.1%>" \
  --label 1%,0.1% --new-family <family> --new-family <family>

# The standing tables of the frozen-evaluation runbook, per arm.
tintype report --operating-points \
  --system "current=runs/ws4-current/test.tsv@<cutoff 1%>" --system "<arm>=runs/ws4-<arm>/test.tsv@<cutoff 1%>" \
  --fpr 0.001,0.01 --prevalence 0.01 --slice-by family,source,transform --cluster-by parent \
  --groups data/frozen-eval/test.tsv --min-slice 100
```

For the refresh arms, add `data/ws4-refresh/test.tsv` to the scored
manifests and to `--groups`, and name the refreshed families with
`--new-family`.

### The path-hash validation split is not held-out data

`tintype train` holds out 3% of every manifest by a hash of the file path
(`is_val`). That split knows nothing of parents, scenes or sources, and
the phase-6 weights were trained on its other 97%. So:

- A **group-hash split of the training stores** (`split-groups` over
  `data/store/*`) is **not** held-out data for the phase-6 weights or for
  anything initialised from them, directly or through `runs/ws4-current`:
  most of its "test" items were training items. It must never be
  calibration or selection data for such a model. Every arm above starts
  from the phase-3 or a later checkpoint, so this applies to all of them.
- The 3% validation pool is the same for every arm (same manifests, same
  paths), which makes the training log's validation lines comparable
  between arms, and nothing more. It shares sources, parents and scenes
  with training.
- Grouped copies written beside their inputs keep that split as it is.
  Copies written with `--out-dir`, and replay or reconstruction manifests
  that reach the same files by another path, hash differently: an old
  training item can land in the validation pool and an old validation
  item in training. `list-reconstruction-parents.py` guards the second
  case for reconstructions; for a replay manifest, name the validation
  pool as contaminated and do not read anything from it.
- Calibrate and select only on the frozen `calibrate` and `test` sets.

## Ready to run

On the Apple GPU, by the maintainer, in this order. Nothing here has been
started.

1. Arms A, B and C of [section 6](#6-equal-budget-training-mixes) (about
   an hour each at phase-6 speed), after deciding the two missing stores.
2. Real-item scores of the training stores for the review sample, with
   the control or with a copy of the shipped weights
   (`cp models/tintype.safetensors models/tintype.json runs/incumbent/`):

   ```bash
   tintype score --weights runs/incumbent/tintype.safetensors \
     $(sed "s/--manifest //g" runs/v7/manifests.txt | tr ' ' '\n' | grep data/store | sed 's/^/--manifest /') \
     --out runs/ws4/train-scores.tsv
   python3 -I scripts/sample-authentic-review.py --scores runs/ws4/train-scores.tsv \
     --out runs/ws4/authentic-review-train.tsv --per-stratum 5 --tail 200 --tail-per-source 20 --seed 1
   tintype verify-models
   ```

3. Arm D, once the reconstruction stores exist (licence checks, then
   generation outside this repository).
4. The refresh arms, once new-family stores exist.
5. The evaluation, once the frozen split exists.

## Policy points for the maintainer

1. **The labelling policy** of section 5, above all the label of a VAE or
   tokenizer round trip of a real photo (proposed 1, as its own slice) and
   of an AI-upscaled or AI-denoised real photo (proposed 0, as its own
   slice).
2. **Face swaps in training.** `OpenForensics_fake` (433 items of
   So-Fake-Set) is trained on as generated; under the policy it would
   leave the full-generation label.
3. **The family map.** The 59 names that rest on recalled model
   documentation, the family of the Community Forensics hub repositories,
   whether open video models stay a family of their own, and whether
   closed services stay `commercial-unknown` where the vendor documents
   the architecture (FLUX pro and max are filed under `latent-diffusion`).
4. **What to do with an authentic-labelled item a reviewer marks as
   generated**: drop it from training and calibration (proposed), or
   relabel it. Dropping loses nothing a reviewer can be wrong about.
5. **The gate's numbers**: the one-point tolerance, the 100-item family
   floor, and which families count as new.
6. **The two missing stores**: fetch `tiny-genimage` and `stylegan3`
   again, or run every arm without them.
7. **The refresh budget and replay share** (2,000 steps, 1e-5, 0.2), which
   are proposals.

## Log

| Date | Commit | What was run | By |
|---|---|---|---|
| 2026-11-13/14 | `5cca9e7` | The audits, probes, family summary, sampling shares and review-sample draw above, on the local stores (CPU). No training, no scoring. | workstream 4 |
