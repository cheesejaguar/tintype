# Runbook: partial edits, with spatial or temporal supervision

Workstream 9 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
("Evaluate partial edits with spatial or temporal supervision"). Partial
edits are a **separate target with a separate output**. Nothing in this
runbook changes `verdict`, `score` or any shipped file, and no number here
selects a model, a cutoff or a pooling rule: that needs the frozen split of
[frozen-eval.md](frozen-eval.md) and a new entry in
[PREREGISTRATION.md](../PREREGISTRATION.md).

**Status.** The tooling exists and is tested: `paste-edits`,
`score-windows`, `partial-report`, `detect --coverage`,
`scripts/build-splice-clips.py`, `scripts/report-splice-clips.py`. The
pasted-region part has been run on the frozen split with cutoffs frozen on
`calibrate` ([Frozen split](#frozen-split)): **the gate failed on cutoff
transfer**, with the local and control conditions met. The numbers in
sections 3 and 9 are **exploratory diagnostics on development data**, there
to show that the pipeline runs and what it measures. The supervised patch
head, the self-blended-face ablation and every external edit set are
specified here and not built: they need downloads or a trainer change
(see [What is not done](#what-is-not-done)).

## The gate (a proposed decision criterion)

Written before the commands that test it:

> **Tintype finds local changes without labelling whole assets as
> generated, at equal authentic FPR.**

Read as three conditions, all on the frozen test set, parent- and
identity-disjoint from anything fitted:

1. **Local.** The local output (a pooled window score, or a patch head)
   finds more partially edited items than today's global score does, with
   every rule operated at the same authentic false-positive rate. Report at
   1% and at 0.1%.
2. **Not global.** The `verdict` and `score` of every item, edited or not,
   are what they are today. The local result is a new field or a new file.
3. **Not the seam, not the re-save.** At the same cutoff, the control
   slices (a re-save alone, a self-paste, an authentic paste; for video an
   authentic splice, an authentic dissolve) are flagged no more often than
   untouched authentic items, within the interval the counts allow.

A local rule that meets 1 by failing 3 has learned to find edits, not
generated content, and does not pass.

## The equal-authentic-FPR rule

Every comparison in this runbook is made the same way. For each rule (a
pooling of window scores, a head, a sampling mode) take its scores on the
authentic items, find the cutoff that lets a stated share of them through
(`metrics::empirical_cutoff`: the strictest score that lets at most that
share above it), and count what is above that cutoff in every other slice.
Two rules are then compared at the same authentic false-positive rate, not
at a shared threshold, and a rule cannot look better by flagging more of
everything. A rate is only resolved when the authentic set is large enough:
0.1% needs at least 1,000 authentic items to allow a single false positive,
and about 3,000 with none flagged to show it at 95%
(`metrics::clopper_pearson_upper`). The reports say when a rate is not
resolved.

Cutoffs set on the authentic parents of the very items being reported are
in-sample. `partial-report --disjoint-cutoff` sets them on one half of the
parents and reports the other half; the frozen split does it properly.

## The partial-edit store

A manifest with the grouping columns ([DATA.md](../DATA.md#partial-edit-stores))
and a side file `edits.tsv` beside it:

- `parent_id` (column 5) is the authentic original. Every derived item
  carries it, so `split-groups` and `verify-split` keep a parent and its
  edits in one set.
- `transform` (column 9) names the slice: `parent`, `resave`,
  `self-paste-hard-area0.05`, `authentic-paste-feather-area0.12`,
  `generated-paste-hard-area0.25`, and for external pipelines
  `inpaint-<model>-area0.12`, `background-exchange-<model>`,
  `vae-roundtrip-<vae>`, `retouch-<tool>`, `codec-<name>-q<level>`.
- The label is 1 only when the item contains generated pixels, and its
  generator is then `partial:<donor generator>`. This is not the label of
  the main target: **never pass a partial-edit manifest to `train`,
  `calibrate` or `finalize`.**
- `edits.tsv`: `path  mask  kind  generated  x  y  w  h  area  donor
  donor_generator`, the first four required. The mask is an 8-bit greyscale
  image the size of the item: 0 untouched, 255 replaced, in between the
  blend weight.

## Images

### 1. Pasted-region composites from data on disk (no downloads)

`paste-edits` pastes a region of a generated image into an authentic one at
native resolution (a resized donor would not carry its generator's traces).
Each parent gives, with the region in the same place for every kind:

| Slice | What it is | What a flag on it means |
|---|---|---|
| `parent` | the original file, byte for byte | a false positive |
| `resave` | the parent through the encoder every composite goes through, nothing pasted | the re-save is being detected |
| `self-paste-<blend>-area<a>` | a region from elsewhere in the same image | the seam is being detected: no foreign pixels at all |
| `authentic-paste-<blend>-area<a>` | a region from another authentic image | a seam, or a mismatch between two authentic sources |
| `generated-paste-<blend>-area<a>` | a region from a generated image | the target |

Blends are `hard` (the region replaces what is under it) and `feather` (a
linear ramp over a band a tenth of the region's shorter side wide, 4 to 64
pixels). Areas are shares of the image. Composites are saved in the
parent's format: JPEG at the quality estimated from its luminance table, or
PNG; a parent in any other format is skipped (`--png` saves everything
derived as PNG instead). The JPEG encoder is the `image` crate's, whose
chroma subsampling is not matched to the parent's; the `resave` slice is
there to show what that costs.

```bash
# Explicit copies of the weights: nothing below writes to models/.
mkdir -p runs/ws9
cp models/tintype.safetensors models/tintype.json runs/ws9/
cp models/tintype-video.safetensors models/tintype-video.json runs/ws9/

# REAL and FAKE are manifests the detector did not train on.
tintype paste-edits --real <REAL manifests> --fake <FAKE manifests> \
  --out runs/ws9/paste --parents 150 --extra-parents 300 --seed 1
# Default: areas 0.02,0.05,0.12,0.25; blends hard,feather; parents with a
# shorter side of 448 px or more and a longer side of 2048 px or less.
# Expect: "<N> items from 150 authentic parents"; 26 slices per parent and
# one `parent` row for each extra parent (more authentic images to set
# cutoffs on).
```

### 2. Global and top-k baselines from the model's own views, and a dense grid

`score-windows` scores, for every image, the model's own views (as
`score --all-views` does, and its first columns are that file's) and a
dense grid of native-resolution 224 px windows, edge to edge, no more than
`--stride` pixels apart, at most `--max-windows` per image. It records
where every window lies. It runs on any engine (`--device cpu` everywhere).
The grid is evaluation output: it is not a view of any profile and
`detect` never computes it.

```bash
tintype score-windows --weights runs/ws9/tintype.safetensors \
  --manifest runs/ws9/paste/manifest.tsv --out runs/ws9/paste-windows.tsv \
  --stride 112 --max-windows 64
# About 40 grid windows per image at these sizes. On the GPU engine keep a
# run to about 2,000 images (split the manifest by parent and concatenate
# the outputs); on the CPU engine any size.

tintype partial-report --scores runs/ws9/paste-windows.tsv \
  --manifest runs/ws9/paste/manifest.tsv \
  --fpr 0.01,0.001 --top-k 3 --overlap 0.1 --out runs/ws9/paste-items.tsv
# Add --disjoint-cutoff for cutoffs set on other parents than those reported,
# --cutoff-on resave to set them on the re-saved authentic images.
```

The report compares these rules, each at its own cutoff for the stated
authentic FPR:

| Rule | What is pooled |
|---|---|
| `global (deployed)` | what `detect` reports: the mean of the views early exit looked at |
| `views mean` | the mean of all six views: today's pooling with early exit off |
| `views max`, `views top-2` | the same six logits, pooled by maximum and by the mean of the two highest |
| `grid mean`, `grid max`, `grid top-3` | the dense grid, pooled by mean, maximum and the mean of the three highest |

and then the localisation readout: a window is *edited* when the mask
covers more than `--overlap` of it, and *flagged* when its logit is above
the `grid max` cutoff (so an authentic image has a flagged window at the
stated rate). It gives window-level precision and recall, the images with
at least one flagged edited window, and, with no cutoff at all, how often
the highest-scoring window of an edited image is an edited one, beside what
picking a window at random would give. `--out` writes the separate per-item
output: the global score as it is today, the local scores, and the flagged
windows.

### 3. Exploratory diagnostic: pasted regions on the validation split

**Not a result.** Development data, cutoffs in-sample unless stated, one
seed, 150 parents per slice. It shows what the pipeline measures.

- Data: the 3% path-hash validation split of the training stores
  (`data/splits/val_images.tsv`: held out of training, used for
  calibration, so development data). Parents: its authentic images from
  `openfake`, `openfake2`, `openfake-test-reals`, `sofake-set`,
  `openfake-reddit`, `defactify` (1,115 candidates; 352 are JPEG or PNG
  with a shorter side of at least 448 px and a longer side of at most
  2,048 px; median shorter side 636 px). Donors: its generated images from
  `openfake`, `openfake2`, `frontier`, `frontier-heldout`, `sofake-set`,
  `openfake-reddit`, `nano-banana` (1,894). About a quarter of the pasted
  regions come from `reddit-wild` images, which were already recompressed
  by a platform.
- Commands: as above, with `--parents 150 --extra-parents 300 --seed 1`
  (4,102 items: 150 parents × 26 slices and 202 further `parent` rows),
  scored in three GPU runs of 1,950, 1,950 and 202 images (159,672 grid
  windows, 38.9 per image; 83 s, 80 s and 9 s).
- Code: store and scores from the binary built at commit `4e86ca5`, report
  at `88c9947` (the scoring code is the same in both). Weights:
  `models/tintype.safetensors` copied to `runs/ws9/`. Hardware: Mac18,5,
  Apple M6, 32 GB, macOS 27.2, GPU engine (half precision).

Images flagged at **1% authentic FPR** (cutoffs may flag 3 of the 352
authentic images; in-sample):

| Slice | Images | Global (deployed) | Views mean | Views max | Views top-2 | Grid mean | Grid max | Grid top-3 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `parent` | 352 | 3 (0.9%) | 3 (0.9%) | 3 (0.9%) | 3 (0.9%) | 3 (0.9%) | 3 (0.9%) | 3 (0.9%) |
| `resave` | 150 | 2 (1.3%) | 1 (0.7%) | 1 (0.7%) | 1 (0.7%) | 1 (0.7%) | 2 (1.3%) | 1 (0.7%) |
| all `self-paste` | 1,200 | 8 (0.7%) | 4 (0.3%) | 8 (0.7%) | 6 (0.5%) | 5 (0.4%) | 14 (1.2%) | 11 (0.9%) |
| all `authentic-paste` | 1,200 | 4 (0.3%) | 3 (0.2%) | 8 (0.7%) | 5 (0.4%) | 2 (0.2%) | 15 (1.2%) | 7 (0.6%) |
| `generated-paste-hard-area0.02` | 150 | 1 (0.7%) | 1 (0.7%) | 1 (0.7%) | 1 (0.7%) | 2 (1.3%) | 3 (2.0%) | 2 (1.3%) |
| `generated-paste-hard-area0.05` | 150 | 3 (2.0%) | 0 (0.0%) | 7 (4.7%) | 3 (2.0%) | 1 (0.7%) | 13 (8.7%) | 7 (4.7%) |
| `generated-paste-hard-area0.12` | 150 | 5 (3.3%) | 2 (1.3%) | 14 (9.3%) | 5 (3.3%) | 1 (0.7%) | 32 (21.3%) | 22 (14.7%) |
| `generated-paste-hard-area0.25` | 150 | 24 (16.0%) | 10 (6.7%) | 42 (28.0%) | 22 (14.7%) | 11 (7.3%) | 57 (38.0%) | 58 (38.7%) |
| `generated-paste-feather-area0.02` | 150 | 1 (0.7%) | 1 (0.7%) | 1 (0.7%) | 1 (0.7%) | 2 (1.3%) | 3 (2.0%) | 2 (1.3%) |
| `generated-paste-feather-area0.05` | 150 | 3 (2.0%) | 0 (0.0%) | 7 (4.7%) | 1 (0.7%) | 1 (0.7%) | 8 (5.3%) | 3 (2.0%) |
| `generated-paste-feather-area0.12` | 150 | 5 (3.3%) | 2 (1.3%) | 11 (7.3%) | 4 (2.7%) | 1 (0.7%) | 17 (11.3%) | 14 (9.3%) |
| `generated-paste-feather-area0.25` | 150 | 17 (11.3%) | 6 (4.0%) | 31 (20.7%) | 12 (8.0%) | 7 (4.7%) | 48 (32.0%) | 48 (32.0%) |
| all `generated-paste` | 1,200 | 59 (4.9%) | 22 (1.8%) | 114 (9.5%) | 49 (4.1%) | 26 (2.2%) | 181 (15.1%) | 156 (13.0%) |

At **0.1%** the rate is not resolved (352 authentic images allow no false
positive; the cutoff is the highest authentic score, which shows only a
rate under 1/352). At that cutoff the rules flag, of the 1,200
`generated-paste` images: global 41 (3.4%), views mean 5 (0.4%), views max
64 (5.3%), views top-2 41 (3.4%), grid mean 0, grid max 89 (7.4%), grid
top-3 92 (7.7%); of the 1,200 `self-paste` images at most 2 under any rule,
and of the 1,200 `authentic-paste` images at most 2.

With cutoffs set on the other half of the parents (`--disjoint-cutoff`: 170
authentic images set the cutoffs, 182 parents and their 2,025 derivatives
are reported; one false positive allowed at 1%), the reported parents are
flagged at 1.6% to 3.8% depending on the rule, not 1%: with this few
authentic images a cutoff does not transfer exactly. At those cutoffs the
648 `generated-paste` images are flagged: global 72 (11.1%), views mean 20
(3.1%), views max 81 (12.5%), views top-2 51 (7.9%), grid mean 23 (3.5%),
grid max 117 (18.1%), grid top-3 103 (15.9%); the 648 `self-paste` images
18 (2.8%), 4, 11, 8, 7, 13 (2.0%), 13; the 648 `authentic-paste` images 7,
3, 6, 6, 2, 15 (2.3%), 12.

Localisation from the grid at the 1% `grid max` cutoff (in-sample):

| Slice | Images | Edited windows | Flagged | Flagged and edited | Precision | Window recall | Images with a flagged edited window | Top window on the edit | By chance |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `parent` | 352 | 0 | 3 | – | – | – | – | – | – |
| all `self-paste` | 1,200 | 12,215 | 19 | 6 | 31.6% | 0.0% | 3/1,184 (0.3%) | 336/1,184 (28.4%) | 30.0% |
| all `authentic-paste` | 1,200 | 12,215 | 16 | 3 | 18.8% | 0.0% | 3/1,184 (0.3%) | 468/1,184 (39.5%) | 30.0% |
| `generated-paste-hard-area0.02` | 150 | 500 | 6 | 4 | 66.7% | 0.8% | 2/147 (1.4%) | 51/147 (34.7%) | 9.8% |
| `generated-paste-hard-area0.05` | 150 | 1,055 | 17 | 15 | 88.2% | 1.4% | 11/150 (7.3%) | 93/150 (62.0%) | 21.3% |
| `generated-paste-hard-area0.12` | 150 | 1,832 | 71 | 69 | 97.2% | 3.8% | 31/150 (20.7%) | 142/150 (94.7%) | 36.2% |
| `generated-paste-hard-area0.25` | 150 | 3,026 | 236 | 236 | 100.0% | 7.8% | 57/150 (38.0%) | 147/150 (98.0%) | 57.1% |
| all `generated-paste` | 1,200 | 12,215 | 549 | 536 | 97.6% | 4.4% | 173/1,184 (14.6%) | 831/1,184 (70.2%) | 30.0% |

What this diagnostic says, and what it does not:

- **The detector reacts to the generated pixels, not to the seam or the
  re-save.** The re-save, self-paste and authentic-paste slices are flagged
  at about the authentic rate under every rule (0.2% to 1.3% against 0.9%).
  Hard and feathered seams behave alike.
- **Today's global score mostly misses these edits.** At 1% authentic FPR
  it flags 4.9% of them, and 16.0% of the largest (a quarter of the image,
  hard edge). Mean pooling over all six views does worse (1.8%): averaging
  dilutes one edited window among five clean ones.
- **Max and top-k pooling find more at the same authentic rate**, and the
  dense grid more than the model's six views: 15.1% (grid max) and 13.0%
  (grid top-3) against 9.5% (views max) and 4.9% (global). It is still a
  minority: `grid max` flags 2.0% of the 2%-area edits and 32.0% (feathered)
  to 38.0% (hard) of the 25%-area ones.
- **Where it fires, it fires in the right place.** 97.6% of flagged
  windows on edited images are edited windows, and the highest-scoring
  window is on the edit in 70.2% of edited images against 30.0% by chance
  (94.7% and 98.0% for hard regions of 12% and 25% of the image). Window
  recall is low (4.4%): most edited windows stay under an image-level
  cutoff. For authentic pastes the top window is on the pasted region
  39.5% of the time, above chance: a pasted region draws some score even
  when it is authentic, but not enough to cross the cutoff.
- These are native-resolution pastes of whole generated pixels. An
  inpainting model blends in latent space and re-decodes the surroundings;
  nothing here says how that scores. The donors include platform-recompressed
  images. One seed, in-sample cutoffs, and 0.1% is not resolved.
- The window baseline a supervised patch head has to beat, on this
  diagnostic, is `grid max` and `grid top-3`.

### 4. External edit sets: what they must emit, and the checks before any download

Nothing below has been downloaded. For each source, **before the first
download**: find the licence in the source's own repository or dataset
card, record its text or URL and the date in
[PREREGISTRATION.md](../PREREGISTRATION.md) and
[DATA.md](../DATA.md), confirm that it allows this evaluation use, and add
it to `NOTICE.md`. If a licence cannot be found, do not download.

| Source | What it adds | Licence |
|---|---|---|
| [Inpainting Exchange](https://arxiv.org/abs/2602.00192v2) | Edited pixels kept, background replaced: the case that dropped DDA from 97.6% to 58.4% | not checked: step 0 |
| SAGI-D, Semi-Truths, SID-Set, COCO-Inpaint, OpenSDID, BR-Gen | Inpainted regions from real models, with masks | not confirmed in the research notes: step 0 for each |
| [FakeParts](https://arxiv.org/abs/2508.21052v2) (FakePartsBench) | Partially generated video with pixel and frame masks | not checked: step 0 |
| An inpainting model run locally | The same parents as the pasted store, region inpainted by a real model | the model's own licence: step 0 |
| VAE round trips, benign retouching, neural codecs | Controls: authentic content through a generator's autoencoder, an editor, a learned codec, with nothing generated | each tool's licence: step 0 |

Whatever produces them, each set arrives as a store this tooling reads:

1. `manifest.tsv` with nine columns. `parent_id` is the authentic original
   (and the original itself is in the manifest, `transform` `parent`).
   `transform` is `inpaint-<model>-area<share>`,
   `background-exchange-<model>`, `vae-roundtrip-<vae>`, `retouch-<tool>`
   or `codec-<name>-q<level>`. `generator_family` is the editing model's
   family. Label 1 only for items with generated pixels.
2. `edits.tsv` with at least `path  mask  kind  generated`, and a mask per
   edited item: 255 where pixels were generated. For a background exchange
   the mask is the background; for a VAE round trip, a codec or a retouch
   there is no generated region, `generated` is 0 and the mask is `-` (the
   whole item is a control).
3. A `resave` of each parent through the same save path the edits went
   through, as its own slice.
4. The same checks as any new evaluation set: `tintype overlap` against
   every training manifest, `split-groups --together parent,scene`,
   `freeze-set`, `verify-split`.

Then the commands of section 2 apply unchanged.

### 5. The supervised patch head (specified, not built)

The comparison the report asks for is a supervised patch head against the
global and top-k baselines above, on real backgrounds with inpainted or
pasted regions.

- **Input.** The last-block patch embeddings (14 × 14 tokens of 384) of
  native-resolution 224 px windows. Today the last block computes only the
  class token (`docs/BENCHMARK.md`), so the engines have to expose the
  patch tokens of that block. Workstream 7 is adding patch-embedding
  export to both engines for its temporal head; this head should read the
  same export and not a second implementation.
- **Head.** One linear layer (384 → 1) on each patch token, giving a
  14 × 14 logit map per window, with the trunk frozen. A two-layer variant
  is the only other arm.
- **Target.** Per patch, the mask's mean blend weight over the patch's
  16 × 16 pixels ([`Mask::coverage`](../../src/partial.rs)), thresholded at
  0.5 for the loss and kept as a fraction for evaluation. Untouched parents
  and the three controls are all-zero maps: they are the negatives that
  keep the head from learning seams.
- **Loss.** Binary cross-entropy per patch, each image weighted equally,
  positives re-weighted by the inverse edited share of the batch.
- **Split.** By parent and by identity: `split-groups --together
  parent,scene` on the store (checked on the diagnostic store: 4,102 items
  into 2,436 / 832 / 834 with no parent crossing), with `scene_id` holding
  the identity for face data. Fit on `fit`, set cutoffs on `calibrate`,
  report on `test`. Donor generators in `test` are held out by family
  (`--hold-out-family`).
- **Readout and comparison.** Pool the patch map to a window score and the
  windows to an image score by maximum and by top-k, and put those two
  columns beside `grid max` and `grid top-3` in the table of section 3, at
  equal authentic FPR, with the control slices. Patch-level precision and
  recall replace the window-level ones.
- **Gate.** The one at the top, plus: no change to any existing output,
  and the head stays out of the default path until a pre-registered test.

### 6. Self-blended faces (specified, not built)

A face-specific training ablation, tested on identity- and parent-disjoint
**full frames**. The existing 14.3% on FaceForensics++ was measured on face
crops, which is a different task from finding a swapped face in a whole
frame; it is not the baseline for this.

- **Before any download:** the licence of the Self-Blended Images code
  ([paper](https://arxiv.org/abs/2204.08376)), of the landmark model it
  needs, and of every face source; FaceForensics++-style sets are released
  for research under signed agreements. Record each as in section 4.
- **Data.** Self-blends are made offline from authentic faces Tintype
  already has rights to (a pseudo source and target from one face, blended
  through a landmark mask), pasted back into their full frames, and
  written as a partial-edit store: `parent_id` the original frame,
  `transform` `sbi-<variant>`, the blend mask in `edits.tsv`,
  `generated` 0 (no generated pixels: this is a manipulation target, not
  the generated-pixel target), and `scene_id` or `source_id` carrying the
  **identity**. Evaluation frames with real swaps are a second store with
  the same columns.
- **Split.** `tintype split-groups --together parent,scene` keeps every
  frame of an identity in one set when `scene_id` is the identity (add
  `source` when `source_id` is). This needs no new code.
- **Arms.** The patch head of section 5 trained (a) on pasted and
  inpainted regions only, (b) with self-blends added. Report (b) − (a) on
  full-frame swaps at equal authentic FPR, and what (b) does to authentic
  retouched and non-face images: the notes found no measurement of that
  risk.

## Video

### 7. Coverage modes (`detect --coverage`)

Today sixteen frames are spread over the first 30 seconds, early exit can
decide from the first half of that span, and the HTTP server never looks
past 30 seconds. `--max-seconds 0` covers the whole clip, sparsely. The
opt-in modes are for seeing where in a clip the score comes from:

| Mode | Windows | Frames |
|---|---|---|
| `uniform` | one: the first `--max-seconds` (the default sampling positions) | the default ones, every one reported: the fixed-budget baseline |
| `segments` | `--segments` equal windows of the whole clip: early, middle, late | the same budget shared between the windows, at the centres of equal parts of each |
| `shots` | as `segments`, with a boundary moved onto a hard cut within a quarter of a window of it | as `segments`; the clip is read twice |

**Early exit is off in every coverage mode.** A window that was never
looked at cannot be reported, and a clip cleared from its first frames is
what the mode exists to check. The clip's `score` and `verdict` are still
the mean over all scored frames through the video calibration, `complete`
is true only when every planned frame was scored, and without the flag the
output is unchanged (checked: 63 lines of `detect`, `demo` and `batch`
output on the fixtures and the demo image, CPU and GPU, text and JSON,
byte for byte the output of the base build). Window scores use the clip calibration on a few frames each
and are not calibrated as decisions. The cut detector compares 16 × 16
luma thumbnails of consecutive frames and finds hard cuts only.
[JSON fields](../API.md#video-coverage-modes-cli).

```bash
tintype detect --json --weights runs/ws9/tintype.safetensors --coverage segments clip.mp4
tintype detect --json --weights runs/ws9/tintype.safetensors --coverage shots --segments 5 --frames 40 clip.mp4
```

### 8. Short edits at known timestamps

`scripts/build-splice-clips.py` (standard library; needs `ffmpeg` and
`ffprobe`) replaces a few seconds of an authentic clip with a generated one
at a known time. Every clip is one FFmpeg run of one filter chain and one
encode, counted in frames, so the inserted part and its surroundings share
an encode and the recorded times are the times in the file. Slices:
`reencode` (the parent alone), `generated-splice-<d>s`,
`authentic-splice-<d>s` (the same cut with another authentic clip: a cut
and no generated content) and `authentic-dissolve-<d>s` (the same insert
cross-faded: an authentic effect). `--parent-clips K` assembles a parent
from K authentic clips of one source, for timelines longer than the clips
on disk; those joins are authentic cuts too. All material is scaled to one
canvas, so the generated frames are resampled like everything else.
Authentic footage with its own edits and effects (broadcast, user uploads)
needs an external source: licence check first, as in section 4.

```bash
python3 -I scripts/build-splice-clips.py \
  --real <authentic video manifests> --fake <generated video manifests> \
  --out runs/ws9/splice --clips 24 --parent-clips 6 --edit-seconds 1 2 4 \
  --max-height 360 --seed 1

clips=$(awk -F'\t' '{print "runs/ws9/splice/"$1}' runs/ws9/splice/manifest.tsv)
w=runs/ws9/tintype.safetensors
tintype detect --json --weights $w $clips                                   > runs/ws9/splice-plain.jsonl
tintype detect --json --weights $w --coverage uniform $clips                > runs/ws9/splice-uniform30.jsonl
tintype detect --json --weights $w --coverage uniform --max-seconds 0 $clips > runs/ws9/splice-uniform-whole.jsonl
tintype detect --json --weights $w --coverage segments $clips               > runs/ws9/splice-segments.jsonl
tintype detect --json --weights $w --coverage shots $clips                  > runs/ws9/splice-shots.jsonl

python3 -I scripts/report-splice-clips.py --edits runs/ws9/splice/edits.tsv \
  --video-spec runs/ws9/tintype-video.json \
  --system "default=runs/ws9/splice-plain.jsonl@30" \
  --system "uniform, first 30 s=runs/ws9/splice-uniform30.jsonl" \
  --system "uniform, whole clip=runs/ws9/splice-uniform-whole.jsonl" \
  --system "segments=runs/ws9/splice-segments.jsonl" \
  --system "shots=runs/ws9/splice-shots.jsonl"
```

The report compares, per sampling mode, the clip mean (what `detect`
decides on), the highest window, the highest frame and the mean of the
three highest frames, each at its own cutoff for the same authentic clip
FPR, with the authentic splices and dissolves in the authentic set. It
counts **an edit no sampled frame falls inside as missed**: it stays in the
denominator and never enters the numerator, and a flag on such a clip is
shown apart. It reports **false flagged segments per hour** of authentic
footage (windows above the `max window` cutoff), by kind of authentic clip,
and how many of the cuts the store made the `shots` detector found.

### 9. Exploratory diagnostic: spliced clips from the development video set

**A pipeline check, not a result.** 24 parent timelines; the authentic
clips of one parent share most of their frames, so the authentic rate is
resolved no finer than 1 in 24, and neither 1% nor 0.1% is resolved. The
"1%" cutoffs below allow one of the 168 authentic clips above them.

- Data: `data/vstore/vdev` (the development video set: Kinetics-400 test
  clips of about 10 s, and 2 s generated clips) for parents, authentic
  inserts and the 1 s generated inserts (22 of 24 from SEINE), and the
  generated clips of `data/vstore/wasitai` (5 to 6 s; an archived
  evaluation set, so development data) for the 2 s and 4 s inserts.
  Parents are six authentic clips each (`--parent-clips 6`): 47 to 59 s
  (mean 56.5 s) at 360 px height, 2.636 h of authentic footage in all.
- Store: 240 clips (24 parents × `reencode`, and for 1, 2 and 4 s a
  generated splice, an authentic splice and an authentic dissolve), built
  with the command of section 8 at commit `15aed85`. Of the 456 hard cuts
  recorded for the re-encodes and the 1 s splices, 454 are within half a
  frame of a scene change FFmpeg's `scene` filter finds in the file.
- Scores: the five `detect` commands of section 8, binary built at
  `88c9947` (no source change since), weights copied to `runs/ws9/`, GPU
  engine, Mac18,5 (Apple M6, 32 GB, macOS 27.2). Wall time for the 240
  clips on a machine shared with other jobs: default 8 s, `uniform` 21 s,
  `uniform --max-seconds 0` 25 s, `segments` 26 s, `shots` 44 s. All 240
  clips were scored completely in every mode. Report at `fea8041`.

Edits with at least one sampled frame inside them, of 24 per duration
(sixteen frames per clip in every mode):

| Sampling | 1 s edit | 2 s edit | 4 s edit |
|---|---:|---:|---:|
| default: first 30 s, early exit | 1 | 5 | 5 |
| `uniform`: first 30 s, every frame | 6 | 15 | 13 |
| `uniform --max-seconds 0`: whole clip | 6 | 14 | 24 |
| `segments` | 5 | 13 | 24 |
| `shots` | 7 | 11 | 24 |

Edits found (the clip is above the cutoff **and** a sampled frame lies in
the edit; an unsampled edit is a miss), of 24, for 1 s / 2 s / 4 s edits,
at the cutoff that lets one of the 168 authentic clips through:

| Sampling | Clip mean | Max window | Max frame | Top-3 frames |
|---|---:|---:|---:|---:|
| default: first 30 s, early exit | 0 / 1 / 1 | – | – | – |
| `uniform`: first 30 s | 1 / 1 / 1 | (one window) | 2 / 2 / 5 | 0 / 0 / 3 |
| `uniform --max-seconds 0` | 0 / 1 / 1 | (one window) | 1 / 4 / 9 | 0 / 1 / 4 |
| `segments` | 0 / 1 / 1 | 1 / 1 / 2 | 1 / 1 / 4 | 1 / 3 / 8 |
| `shots` | 0 / 1 / 1 | 0 / 2 / 1 | 3 / 3 / 11 | 1 / 2 / 3 |

At those cutoffs every rule flags 0 or 1 of the 168 authentic clips
(re-encodes, authentic splices and dissolves together), and the windows
above the `max window` cutoff in authentic footage number 0 or 1 in
2.636 h (0.0 to 0.4 false flagged segments per hour; the one is on a
dissolve). In the whole-clip modes a few edited clips are above a cutoff
although no sampled frame lies in their edit (up to 2 of 24 per cell);
the report shows them apart and does not count them. The `shots` cut
detector found 1,378 of the 1,444 hard cuts the store made (95.4%, within
0.15 s) and 1,237 other cuts in the 240 clips; the Kinetics clips have cuts
of their own, and these were not checked one by one.

What this check says, and what it does not:

- **Most short edits are never sampled.** With sixteen frames in a clip of
  about a minute, a 1 s edit has a sampled frame inside it in 5 to 7 of 24
  clips in every mode that scores all sixteen, and a 2 s edit in 11 to 15.
  The default mode, which looks at the first 30 s and stops early, samples
  1, 5 and 5 of 24. Only the whole-clip modes sample every 4 s edit.
- **The clip mean does not find a short edit even when it is sampled**:
  0 or 1 of 24 in every mode. One or two generated frames among sixteen
  do not move a mean.
- **Frame-level maxima find more at the same authentic clip count**: up to
  11 of 24 four-second edits (`shots`, max frame) and 8 of 24
  (`segments`, top-3 frames), against 1 of 24 for the clip mean. Early,
  middle and late windows of about 19 s are too long for a 1 to 4 s edit:
  `max window` finds 0 to 2 of 24.
- With 24 parents none of the differences between modes or rules is
  established, and no false-alarm rate is resolved. The inserts are
  resampled to the parent's canvas and re-encoded, and the generated
  inserts come from two sets the detector was developed against. Real
  edited footage, longer clips and enough independent authentic parents
  are what a result needs (see below).

## Frozen split

Sections 1 and 2 on the frozen independent split
([frozen-eval.md](frozen-eval.md)): pasted-region composites from the
split's own originals, which the detector never trained on, then the
global, top-k and dense-grid rules at equal authentic FPR, with cutoffs
frozen on `calibrate` and reported on `test`. The pre-registration is in
[PREREGISTRATION.md](../PREREGISTRATION.md) ("Partial edits on the frozen
split"), committed as `84b2040` before any `test`-side composite existed.
Sections 4 to 9 (inpainting sets, the patch head, self-blended faces,
video) were not run: they need downloads or training that does not exist.

**Verdict: the gate failed, at 1% and at 0.1%, on the transfer condition.**
The frozen cutoff of the *global* score let through 1.8% and 0.6% of the
untouched `test` images, above both targets, so the comparison at frozen
cutoffs is not at equal authentic FPR. The other two conditions held:
`grid max` found more of a parent's edits than the global score (53.2%
against 21.5% of the generated pastes at the 1% cutoffs, 33.8% against
18.2% at the 0.1% cutoffs), and no control slice was flagged more than
untouched images. Nothing ships either way.

### What subset of the split is usable, and why

A region is pasted at native resolution, so a donor has to be at least as
large as the region. The split's authentic originals are camera JPEGs of 12
megapixels or more and its generated originals are PNGs of 256 to
1,024 px. A region of 1% of such a photo is about 350 px square, so:

- **Donors: the 1,024 px generators only.** In `calibrate` FLUX.1-dev,
  FLUX.1-Krea and SD3 (1,200 of 4,200 generated originals); in `test`
  those three and Infinity, a held-out family (2,700 of 7,200). Switti
  (512 px), Janus-Pro and LlamaGen (384 px), VAR, Open-MAGVIT2 and RAR
  (256 px) cannot donate a region of this size and are not measured.
- **Areas: 1%, 2% and 3% of the image** (about 350, 490 and 600 px
  square), not the 2% to 25% of the development diagnostic. A 12% region
  of these photos would be larger than any generated image in the split.
- **Parents: authentic originals with a longer side of at most 4,800 px**
  (24 of the originals passed over while shuffling were larger; a 3%
  region of them does not fit a 1,024 px donor). Processed copies are
  never parents.

The split's caveats apply: authentic and generated originals differ in
format and size, FloreView is 36 scenes on 46 phones and CSAFE ten units of
one phone model, there are no platform round trips or learned-codec copies,
and `test` had already been scored for the incumbent and two
processing-chain arms. This workstream looked at it once.

### Commands

```bash
mkdir -p runs/ws9 && cp models/tintype.safetensors models/tintype.json runs/ws9/
F=<the split>/data/frozen-eval
for side in calibrate test; do
  python3 -I scripts/prepare-partial-store.py --set $F/$side.tsv --out runs/frozen9/$side \
    --chunks 4 --chunk-rows 30 --pool 1000 --seed 1
  for k in 0 1 2 3; do            # one chunk at a time: about 2 GB of composites each
    tintype paste-edits --real runs/frozen9/$side/real-$k.tsv --fake runs/frozen9/$side/donors.tsv \
      --out runs/frozen9/$side/store-$k --parents 25 --areas 0.01,0.02,0.03 --blends hard,feather \
      --max-side 4800 --dataset partial-paste-frozen --seed 1
    tintype score-windows --weights runs/ws9/tintype.safetensors \
      --manifest runs/frozen9/$side/store-$k/manifest.tsv --out runs/frozen9/$side/windows-$k.tsv \
      --stride 224 --max-windows 128
    rm -rf runs/frozen9/$side/store-$k/img
  done
  tintype score-windows --weights runs/ws9/tintype.safetensors --manifest runs/frozen9/$side/pool.tsv \
    --out runs/frozen9/$side/windows-pool.tsv --stride 224 --max-windows 128
  cat runs/frozen9/$side/windows-{0,1,2,3,pool}.tsv > runs/frozen9/$side/windows.tsv
done
M() { printf -- '--manifest runs/frozen9/%s/store-0/manifest.tsv --manifest runs/frozen9/%s/store-1/manifest.tsv --manifest runs/frozen9/%s/store-2/manifest.tsv --manifest runs/frozen9/%s/store-3/manifest.tsv --manifest runs/frozen9/%s/pool.tsv' $1 $1 $1 $1 $1; }

# calibrate: set the cutoffs, choose the local rule
tintype partial-report --scores runs/frozen9/calibrate/windows.tsv $(M calibrate) --fpr 0.01,0.001 \
  --freeze-cutoffs runs/frozen9/calibrate-cutoffs.tsv --out runs/frozen9/calibrate-items.tsv
# test: the frozen cutoffs, then the same at test's own cutoffs (empirical), then the gate
tintype partial-report --scores runs/frozen9/test/windows.tsv $(M test) --fpr 0.01,0.001 \
  --cutoffs runs/frozen9/calibrate-cutoffs.tsv --out runs/frozen9/test-items.tsv
tintype partial-report --scores runs/frozen9/test/windows.tsv $(M test) --fpr 0.01,0.001
python3 -I scripts/partial-gate.py --items runs/frozen9/test-items.tsv \
  --cutoffs runs/frozen9/calibrate-cutoffs.tsv --local "grid max"
```

Tooling at commit `ab4808d`; the binary that built and scored both sides
was built from it (later commits change tests and documents only).
Weights: the released photo model, copied to `runs/ws9/`. Hardware: Apple
M6, 12 cores, 32 GB, macOS 27.2, GPU engine at half precision, shared with
another job, so no timing is reported. Per side: 100 parents in four
chunks of 25, 20 slices each, and 1,000 further untouched originals:
2,998 images on `calibrate` (two 3% generated pastes found no donor that
fits) and 3,000 on `test`, about 116 grid windows per image (a 12-megapixel
photo gets about 13 × 9 windows with gaps of 100 to 130 px between them:
the cap of 128 was fixed in advance as a cost bound). Composites are
deleted; the scores, cutoffs, manifests, edit geometry and reports are in
[docs/results/partial-edits-frozen](../results/partial-edits-frozen/README.md).

### Log

- 14 November 2026, `calibrate` built and scored. Cutoffs frozen on its
  1,100 untouched images (11 and 1 false positives allowed):
  [calibrate-cutoffs.tsv](../results/partial-edits-frozen/calibrate-cutoffs.tsv).
  In-sample at the 1% cutoffs, of the 598 `calibrate`-side generated
  pastes: global 50 (8.4%), `grid max` 304 (50.8%), `grid top-3` 183
  (30.6%). **The local rule of the gate is `grid max`**, recorded at
  `8edce94` before the `test` look.
- 14 November 2026, `test` built and scored, once. Results below.

### Edited images found on `test`, by area and blend, at the frozen cutoffs

Every slice has 100 images (one per parent), so a count is also a
percentage. "Untouched" is the 100 parents and the 1,000 pool originals.

At the cutoffs frozen for **1%** authentic FPR:

| Slice | n | Global (deployed) | Views mean | Views max | Views top-2 | Grid mean | Grid max | Grid top-3 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| untouched | 1,100 | 20 (1.8%) | 20 (1.8%) | 18 (1.6%) | 12 (1.1%) | 14 (1.3%) | 16 (1.5%) | 10 (0.9%) |
| `resave` | 100 | 1 | 0 | 0 | 0 | 1 | 0 | 0 |
| all `self-paste` | 600 | 8 (1.3%) | 0 | 0 | 0 | 6 (1.0%) | 0 | 0 |
| all `authentic-paste` | 600 | 4 (0.7%) | 0 | 0 | 0 | 6 (1.0%) | 0 | 0 |
| generated, hard, 1% | 100 | 14 | 6 | 17 | 14 | 1 | 36 | 6 |
| generated, hard, 2% | 100 | 21 | 15 | 31 | 24 | 1 | 62 | 31 |
| generated, hard, 3% | 100 | 32 | 24 | 44 | 34 | 1 | 70 | 62 |
| generated, feathered, 1% | 100 | 11 | 4 | 15 | 10 | 1 | 25 | 2 |
| generated, feathered, 2% | 100 | 20 | 12 | 30 | 20 | 1 | 59 | 23 |
| generated, feathered, 3% | 100 | 31 | 23 | 39 | 31 | 1 | 67 | 52 |
| all generated pastes | 600 | 129 (21.5%) | 84 (14.0%) | 176 (29.3%) | 133 (22.2%) | 6 (1.0%) | 319 (53.2%) | 176 (29.3%) |

At the cutoffs frozen for **0.1%** (one false positive allowed on
`calibrate`: the rate is resolved by a single image):

| Slice | n | Global (deployed) | Views mean | Views max | Views top-2 | Grid mean | Grid max | Grid top-3 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| untouched | 1,100 | 7 (0.6%) | 8 (0.7%) | 3 (0.3%) | 5 (0.5%) | 3 (0.3%) | 3 (0.3%) | 0 |
| `resave`, `self-paste`, `authentic-paste` | 1,300 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| generated, hard, 1% | 100 | 9 | 3 | 15 | 11 | 0 | 20 | 1 |
| generated, hard, 2% | 100 | 19 | 11 | 28 | 21 | 0 | 39 | 11 |
| generated, hard, 3% | 100 | 26 | 20 | 40 | 30 | 0 | 53 | 35 |
| generated, feathered, 1% | 100 | 10 | 2 | 12 | 8 | 0 | 8 | 0 |
| generated, feathered, 2% | 100 | 18 | 8 | 27 | 19 | 0 | 35 | 6 |
| generated, feathered, 3% | 100 | 27 | 18 | 38 | 28 | 0 | 48 | 24 |
| all generated pastes | 600 | 109 (18.2%) | 62 (10.3%) | 160 (26.7%) | 117 (19.5%) | 0 | 203 (33.8%) | 77 (12.8%) |

**The frozen cutoffs did not hold the authentic rate on `test`** for most
rules: at the 1% cutoffs the untouched images are flagged at 0.9% to 1.8%,
at the 0.1% cutoffs at 0.0% to 0.7%. The columns above are therefore not at
equal authentic FPR. The same table at `test`'s own cutoffs, where every
rule flags exactly 11 and 1 of the 1,100 untouched images, is **empirical,
not frozen**:

| All 600 generated pastes | Global (deployed) | Views mean | Views max | Views top-2 | Grid mean | Grid max | Grid top-3 |
|---|---:|---:|---:|---:|---:|---:|---:|
| empirical 1% | 114 (19.0%) | 76 (12.7%) | 170 (28.3%) | 130 (21.7%) | 6 (1.0%) | 292 (48.7%) | 190 (31.7%) |
| empirical 0.1% | 104 (17.3%) | 31 (5.2%) | 148 (24.7%) | 101 (16.8%) | 0 | 197 (32.8%) | 83 (13.8%) |

At the empirical cutoffs no `resave`, `self-paste` or `authentic-paste`
image is flagged by any rule but `grid mean` (13 of 1,300 at 1%). By area
and blend: [test-report-empirical.md](../results/partial-edits-frozen/test-report-empirical.md).

### Localisation on `test` (grid windows above the frozen `grid max` cutoff)

| Cutoff | Flagged windows on generated pastes | Of them on the edit | Window recall | Images with a flagged edited window | Flagged windows on the 1,100 untouched images |
|---|---:|---:|---:|---:|---:|
| 1% | 472 | 472 (100%) | 472 of 2,249 (21.0%) | 319 of 600 | 19 |
| 0.1% | 262 | 262 (100%) | 262 of 2,249 (11.6%) | 203 of 600 | 3 |

With no cutoff at all, the highest-scoring window of a generated paste is
on the pasted region in 545 of 600 images (90.8%), against 3.3% by chance;
for an authentic paste in 36 of 600 (6.0%) and for a self-paste in 24 of
600 (4.0%). No window on a self-paste or an authentic paste is above
either cutoff.

### The gate, as pre-registered

From [test-gate.md](../results/partial-edits-frozen/test-gate.md)
(`grid max` against the global score, tests over the 100 `test` parents):

| Condition | 1% | 0.1% |
|---|---|---|
| Transfer, global: untouched `test` images above the frozen cutoff | 20 of 1,100 (1.8%; 95% interval 1.11% to 2.79%): **above the target** | 7 of 1,100 (0.6%; 0.26% to 1.31%): **above the target** |
| Transfer, `grid max` | 16 of 1,100 (1.5%; 0.83% to 2.35%): within | 3 of 1,100 (0.3%; 0.06% to 0.79%): within |
| Local: parents where `grid max` finds more of the generated pastes than the global score, against fewer | 73 against 11 (p = 1.1e-12): met | 52 against 20 (p = 1.0e-4): met |
| Controls (`resave`, `self-paste`, `authentic-paste`) | 0 of 1,300 flagged by `grid max`: met | 0 of 1,300: met |
| **Gate** | **failed** (transfer) | **failed** (transfer) |

The failure is the global score's cutoff, not the local rule: the cutoff
frozen on FloreView and HDR+ photos lets through too many untouched `test`
photos, which add the held-out CSAFE phone. At 1% those 20 are 6 of 577
CSAFE, 8 of 230 HDR+ and 6 of 293 FloreView images. Because the global
score ran at a higher authentic rate than `grid max` at the frozen cutoffs
and still found fewer edits, the miss does not favour the local rule; but
the pre-registered gate asks for equal authentic FPR at frozen cutoffs,
and that was not shown. The gate is not passed.

### What the run says, and what it does not

- **On photos the detector never trained on, it reacts to pasted generated
  pixels and not to the paste.** No re-save, self-paste or authentic paste
  was flagged by `grid max` at either frozen cutoff (0 of 1,300), and every
  flagged window on an edited image was on the edit.
- **A dense grid with max pooling finds about half of 1% to 3% pasted
  regions at the 1% cutoff and a third at the 0.1% cutoff**; the deployed
  global score about a fifth. Area matters more than the seam: `grid max`
  finds 36, 62 and 70 of 100 hard-edged regions of 1%, 2% and 3%, and 25,
  59 and 67 feathered ones.
- **`grid top-3` and `grid mean` are worse than `grid max` here.** A 1%
  region is one or two windows of about 116, so anything that averages
  loses it.
- **The global score finds more here than on the development diagnostic**
  (21.5% against 4.9% of pastes) although the regions are a smaller share
  of the image. Why was not tested; the regions are as large in pixels,
  the donors differ, and the global score here ran at 1.8% authentic FPR.
  Not pre-registered, descriptive: at the 1% cutoffs the global score and `grid max` find 97
  and 171 of 338 Infinity pastes, 13 and 62 of 82 FLUX.1-dev, 13 and 59 of
  92 SD3, and 6 and 27 of 88 FLUX.1-Krea.
- **Cutoffs set on 1,100 authentic photos of two sources did not hold
  their rate on 1,100 photos of three**, for the global score at both
  operating points and for most pooled rules.
- It does not say anything about inpainting, about generators below
  1,024 px, about edits larger than 3% or smaller than 1% of a
  12-megapixel photo, or about processed copies: none could be built from
  what is on disk. The grid had gaps, so a full tiling could find more.
  Scoring cost was not measured (the machine was shared); a dense grid is
  about 116 forward passes per photo against the default profile's one to
  six, and is an evaluation tool, not a proposal.

## What is ready to run

Nothing here needs a GPU run that is not already described; none of it
selects anything.

| Step | Command | Needs |
|---|---|---|
| Pasted regions on the frozen split, cutoffs on `calibrate`, numbers on `test` | [done](#frozen-split): the gate failed on cutoff transfer | – |
| The same on the split's processed copies as parents, and with a full grid (`--max-windows 400`) | the commands of the frozen-split section | disk for about 12 times the composites; an idle GPU |
| Scoring cost of a dense grid: `tintype latency`, and `score-windows` timed alone | not run: the machine was shared | an idle machine |
| The same on an inpainting set | section 4, then section 2 | the licence check and the download |
| Splice clips from authentic clips at least a minute long, with enough authentic clips to resolve 1% (100 or more) | section 8 | long authentic clips: a licence check and a download |
| Coverage modes on real edited footage | section 7 | the same |

## What is not done

- **No patch head and no patch-embedding export.** The head is specified
  in section 5. Exporting patch tokens from both engines is workstream 7's
  work; a second export here would have duplicated it. At merge: one
  export of last-block patch tokens, used by the temporal head and by this
  head; `score-windows` could then write patch maps beside window logits.
- **No inpainting, background exchange, VAE round trip, retouching or
  neural codec data.** They need models or datasets that are not on disk.
  The pasted-region store is the only edit set measured.
- **No self-blended faces.** Design and data requirements only (section 6).
- **The HTTP API has no coverage mode.** It still samples the first 30
  seconds; the modes are a command-line evaluation tool.
- **No cutoff, pooling rule or sampling mode is proposed for shipping.**
  Every number above is exploratory.
