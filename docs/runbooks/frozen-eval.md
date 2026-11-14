# Runbook: the frozen independent evaluation split

This is step 0 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
("What to run first", item 1). It is written, and committed, **before any
model, threshold, view pool, exit margin or routing band is selected**, and
every later runbook refers back to it. It says what the split is made of,
how it is built and frozen, what is measured on it, and the rule that picks
the next model.

**Status.** The tooling exists and is tested (`split-groups`,
`verify-split`, frozen-record format 2, `report --operating-points`,
`latency` with p95 on every platform). The split itself does not exist yet:
it needs sources that Tintype has never trained on, calibrated on or
scored, which means new downloads. The candidate sources are listed below;
the choice, and the licence check for each, are recorded in
[PREREGISTRATION.md](../PREREGISTRATION.md) before the first download. No
number in this file was measured on the split.

## Why the archived suites are not enough

Confirmation 7, Arena, Frontier, the released NTIRE sets (public test and
hard validation), DaVODi and confirmation 4 have all been looked at while
the report was written. They are development and diagnostic data from here
on: a cutoff, band or model tuned on them is not a blind result, and the
score replays in the report chose their cutoffs on the cohorts they report.
They also share sources and generators with the training stores
(COCO, ImageNet, So-Fake, AIGIBench archives), which limits what they say
about transfer.

## What goes in

Three parts, each with both classes, each from sources and generator
families that appear in **no** training manifest
([DATA.md](../DATA.md#training)), no calibration set and no archived suite.

| Part | Class | What | Why |
|---|---|---|---|
| Authentic, camera and phone originals | real | At least three collections with known provenance, none of them COCO, ImageNet, LAION, Pexels, FFHQ, LHQ, CelebA-HQ, DOCCI, RAISE-1k or a So-Fake / OpenFake / AIGIBench real pool | False-alarm transfer to new sources is the first thing the report asks of any cutoff |
| Authentic, as shared | real | Authentic photos after real platform round trips (upload and download), and authentic photos through benign processing: restoration, enhancement, learned codecs | The NTIRE failure: 110 of 182 authentic images through JPEG AI, Cheng2020 or WMForger are flagged |
| Generated, held-out families | AI | Whole generator families that no training store has, across diffusion, autoregressive and hybrid decoders, with prompts or scenes recorded | New-family recall |
| Generated and authentic, degraded pairs | both | Every item above, also through the processing chains of [processing-chains.md](processing-chains.md) (resize, JPEG, blur, noise, sharpening, repost simulation), each derivative carrying its parent's `parent_id` | "Independent degraded files" is what selects the next model |

**Candidate sources.** None has been downloaded. The table records what
a read of each collection's own pages found on 2026-11-13 (licence, access,
size, and which source pools it draws on). It is a survey, not a licence
review: before downloading a collection, read its licence at the source
again, record the licence and where it is stated in
[DATA.md](../DATA.md) or [NOTICE.md](../../NOTICE.md), confirm it is absent
from every training manifest, and pin the revision the way the data plans
do. A collection whose terms do not allow this use is dropped, and the
replacement is written into the pre-registration.

| Role | Collection | Access and terms as read | Size | Fit for this split |
|---|---|---|---|---|
| Authentic, phone, native and as shared | [Forchheim Image Database](https://faui1-files.cs.fau.de/public/mmsec/datasets/fodb/) | Open directory. Free for research with citation; no modification, commercial use or redistribution without an agreement | 23,106 JPEGs, about 21 GB: 143 scenes by 27 phone cameras, each camera-native and through Facebook, Instagram, Telegram, Twitter and WhatsApp | **Proposed.** Real platform round trips with parents known. Ask the authors whether "may not be modified" covers applying our processing chains to copies. Exclude its `inspection/` folder |
| Authentic, phone, native | [VISION](https://lesc.dinfo.unifi.it/VISION/) | Open directory. CC BY-SA 4.0 | Over 35,000 images and videos from 35 devices; image count and size not confirmed | **Proposed** |
| Authentic, camera originals | [RAISE](http://loki.disi.unitn.it/RAISE/), all 8,156 | Form on download. Non-commercial research and education, with citation | About 350 GB as RAW and TIFF | Possible. The 1,000 images of RAISE-1k (`benchmarks` plan) must be excluded by id, and RAW needs one stated development and encoding step |
| Authentic, camera originals | Dresden Image Database | Host unreachable on 2026-11-13 | not confirmed | Not usable until it can be read |
| Authentic, phone | [LAION-Mobile](https://huggingface.co/datasets/laionmobile/laion-mobile) | Open, metadata only (CC BY 4.0); images are fetched from their own hosts | 935 k URLs | **Rejected**: drawn from LAION, which the OpenFake training reals also are |
| Generated, autoregressive and tokenizer families | [ARForensics](https://huggingface.co/datasets/Yanran21/ARForensics), generated halves only | Open. MIT | 304 k images, 59 GB; test split has 6 k per generator for LlamaGen, VAR, Infinity, Janus-Pro, RAR, Switti, Open-MAGVIT2 | **Proposed.** Its real images are ImageNet and are discarded. Pin a commit: the test set was revised once |
| Generated, recent open-weight diffusion | [GenImage++](https://huggingface.co/datasets/Lunahera/genimagepp) | Open. CC BY-NC 4.0 | About 139 k images, 207 GB: FLUX.1-dev, SD3 Medium, SDXL, SD 1.5 at 1,024 px | **Proposed, sampled.** An independent source of families the model has seen, not a held-out family. Generated only |
| Generated and authentic, shared and recaptured | [RRDataset](https://zenodo.org/records/14963880) | Open. CC BY 4.0 on Zenodo; its news photographs are third-party | 22 GB: 10 k real and 10 k AI, each original, transmitted over 2 to 6 platform cycles, and re-digitised | Possible, filtered: its COCO reals, StyleGAN and ProGAN images and its Chameleon portion must be removed, and whether it labels them per image is not confirmed |
| Generated and authentic, shared | [TrueFake](https://github.com/MMLab-unitn/TrueFake-IJCNN25) | Repository CC BY 4.0; data on a drive link that was not opened | 600 k images | Mostly unusable: FFHQ reals and FFHQ-trained StyleGAN faces are training pools here. Only its FORLAB reals and diffusion images are outside them |
| Social, as posted | [WildRF](https://github.com/barcavia/RealTime-DeepfakeDetection-in-the-RealWorld) | Research software licence, terms not read; no dataset licence stated | 5,300 images | Not proposed: labels rest on hashtags, and its relation to SocialRF (a development set here) is not confirmed |
| Human-hard | [Chameleon](https://github.com/shilinyan99/AIDE) | By request to the authors, under an agreement; academic use only | 14,863 real, 11,170 AI | Possible if granted. Only the generated images were filtered for realism; part of it is inside RRDataset, so the two are not independent |
| Recent commercial generators | several public GPT Image 2 and Nano Banana 2 sets | mixed | – | **Not held out.** The local training and evaluation manifests already hold `gpt-image-2`, `nano-banana-2.0`, `nano-banana-pro` and FLUX.2 images, so these families cannot serve as unseen ones |
| Learned codecs | none public was found | CompressAI: BSD 3-Clause Clear. JPEG AI reference software: BSD. Neither grants patent rights, and no licence is stated for CompressAI's pretrained weights | – | Made here, offline, from the authentic sources above ([processing-chains.md](processing-chains.md)) |

Which generator families are new was checked against the generator column
of every local manifest (`data/store/*/manifest.tsv` and
`data/fresh*/*/manifest.tsv`, 2026-11-13): none lists LlamaGen, VAR,
Infinity, RAR, Switti or Open-MAGVIT2; `janus-7b` appears 95 times. The
public NTIRE 2026 distortion code and its licence could not be found, so
the processing chains are written here rather than borrowed.

**Proposed composition**, to be confirmed in the pre-registration entry:
authentic items from the Forchheim database (native files, and their five
platform versions as the shared part) and VISION; generated items from
the ARForensics test split and a sample of GenImage++; learned-codec and
processing-chain copies of all of them made offline. Encoding is the
confound to control: the authentic sets are camera JPEGs, while the
generated sets' formats are not confirmed, so the report carries a
same-encoding arm (every item re-encoded one way) beside the as-released
one. This composition does not reach 3,000 independent authentic scenes
(Forchheim has 143 scenes across 27 cameras), so the 0.1% operating point
needs RAISE or a further authentic source to be resolved.

**Held out whole.** Every source and family in the split is held out from
fitting detectors: no item of them may enter a training manifest while the
split is in use. Inside the split, the families and sources named with
`--hold-out-family` and `--hold-out-source` go to the test set whole, so
the test set also contains families and sources that the calibration set
has not seen. The pre-registration entry lists them by name. At least one
tokenizer family and at least one authentic source are held out this way.
For the Forchheim images the scene is the `scene_id` and the camera is the
`source_id`, so a scene never crosses sets and a camera can be held out.

**Size.** The stricter operating point decides it. A one-sided 95%
Clopper–Pearson bound of 0.10% on the false-positive rate needs about
3,000 independent authentic items with none flagged (`0 of 2,995` is the
first count that reaches it). So the calibration set and the test set each
need at least 3,000 authentic **parents**: derivatives of one photo are
one piece of evidence, not several. If the sources found do not reach
that, the 0.1% point is still reported, with its bound, and marked as not
resolved.

## Grouping columns

Every manifest of the split carries the grouping columns after the usual
four (`path label generator dataset`):

| Column | Name | Value |
|---|---|---|
| 5 | `parent_id` | The item a file derives from. An original carries its own id; every processed copy carries the original's. |
| 6 | `scene_id` | The prompt or scene shared by sibling items (several generators given one prompt; a real photo and its reconstructions). |
| 7 | `source_id` | The authentic collection, camera or platform. |
| 8 | `generator_family` | The decoder family (`real` items leave it empty). |
| 9 | `transform` | The processing applied: empty for an original, else e.g. `jpeg-q70`, `repost-960-q70x2`. |

An empty column or `-` means "none". Four-column manifests still load, and
are written back unchanged.

## Building and freezing it

Run from the directory that holds `data/`. `<stores>` are the manifests of
the new collections and their processed copies.

```bash
# 1. Nothing in the split may be a file Tintype trained on or scored before.
tintype overlap $(printf -- '--train %s ' data/store/*/manifest.tsv data/fresh*/*/manifest.tsv data/vstore/wasitai/images*.tsv) \
  $(printf -- '--test %s ' <stores>)
# Expect: "0 of N test files are byte-identical to one of M training files".

# 2. Assign groups. Parents and scenes never cross sets; the named families
#    and sources go to the test set whole.
tintype split-groups $(printf -- '--manifest %s ' <stores>) \
  --out-dir data/frozen-eval --fractions 0.3,0.35,0.35 --seed 1 \
  --together parent,scene \
  --hold-out-family <family> --hold-out-source <source>

# 3. Record each set with its bytes, and check the records.
for set in fit calibrate test; do
  tintype freeze-set --manifest data/frozen-eval/$set.tsv \
    --out docs/manifests/frozen-eval-$set.frozen.tsv.gz
done
tintype verify-set docs/manifests/frozen-eval-{fit,calibrate,test}.frozen.tsv.gz
tintype verify-split \
  --set fit=docs/manifests/frozen-eval-fit.frozen.tsv.gz \
  --set calibrate=docs/manifests/frozen-eval-calibrate.frozen.tsv.gz \
  --set test=docs/manifests/frozen-eval-test.frozen.tsv.gz \
  --together parent,scene --held-out-family <family> --held-out-source <source>
# Expect: "no group crosses sets". It fails on a shared path, parent or
# scene, on a held-out family or source outside test, and on two files
# with identical bytes in different sets.
```

`fit` is for fitting things that sit on top of a detector: fusion weights,
a linear probe, a pooling rule, exit margins. `calibrate` is for cutoffs
and abstention bands. `test` is for the numbers. The shares are 30 / 35 /
35 so that calibration and test have the authentic parents the 0.1% point
needs.

Processed copies made after the split exists go through the same command
with `--existing data/frozen-eval`: a group that contains an item already
placed goes to that item's set, so a derivative can never land on the
other side from its parent. Freeze the new sets under a new name; the
earlier records stay as they are.

**No manifest already pinned by `docs/manifests/*.frozen.tsv.gz` is
rewritten.** Grouped copies of older sets are written as new manifests.

### Models trained on the old split

The phase-6 weights, and anything initialised from them, were trained on
the path-hash partition (`is_val`, 3% held out by a hash of the path),
which knows nothing of parents or sources. A group-hash split of those
same stores is therefore **not** held-out data for such a model: do not
calibrate or select on it. Calibrate and select only on the `calibrate`
and `test` sets above, after step 1 has shown that their items appear in
no earlier training manifest. `finalize` still chooses its calibration
data by `is_val`; for a candidate model use `score` and `calibrate` on the
frozen `calibrate` set instead (below).

## What is measured

Fixed before any candidate is scored. Every table is printed by
`tintype report --operating-points`.

| Quantity | Definition |
|---|---|
| **Primary operating points** | Recall on `test` at **0.1%** and at **1%** false-positive rate. Reported two ways, and labelled: (a) at the cutoffs frozen on `calibrate` (the result); (b) at the empirical cutoffs on `test` itself (a ranking measure, not a transfer result). |
| Frozen cutoffs | For every cutoff a candidate ships with (default, strict, sensitive, fast, and any routing rule): the **actual** false-positive rate and recall on `test`, the count of authentic items flagged, and the one-sided 95% Clopper–Pearson upper bound on the rate. |
| **Prevalence** | Declared here: **1%** of incoming items are AI-generated. Precision is reported at that prevalence, with 0.1% and 10% beside it as sensitivity. (At 1%, 90% precision at 90% recall needs about 0.10% FPR.) |
| **Worst slice** | The lowest recall over generator-family slices, and the highest false-positive rate over authentic-source slices and over transform slices, counting slices with at least 100 items of the class. |
| Intervals | 95% bootstrap over **parents** (`--cluster-by parent`), and over sources where a source has few parents. Item-level intervals are not used where derivatives repeat a parent. |
| **Abstention** | For a candidate with a review band: coverage (the share of items given a verdict), review rate (the share sent to review, per class), and recall with every abstained generated item counted as a **miss**. |
| **Latency** | File-to-verdict p50 and p95, one file at a time, on the hardware that will run it: `tintype latency`, on the Apple GPU and on the CPU engine, and on a Linux host for the CPU engine. Throughput, peak memory and second-stage route rate beside it. |
| Regression suites | Confirmation 7, Arena, Frontier pairs, NTIRE public and hard validation, as diagnostics: each candidate's rates on them are reported beside the incumbent's, and never used to choose. |

```bash
# A candidate lives under runs/, never under models/.
W=runs/<experiment>/<name>.safetensors      # with <name>.json beside it

# 1. Score the calibration set (Apple GPU). --all-views writes every view
#    logit, which calibration and the offline tools need.
tintype score --weights $W --manifest data/frozen-eval/calibrate.tsv --out runs/<experiment>/calibrate.tsv --all-views

# 2. Fit the cutoff from the calibration set's authentic items only. This
#    rewrites the spec beside $W.
tintype calibrate --weights $W --scores runs/<experiment>/calibrate.tsv --profile photo --target-fpr 0.01
tintype verify-models                          # the released files are untouched

# 3. Only now score the test set: the probabilities in a score file come
#    from the spec as it stood when the file was written, so a test file
#    scored before step 2 would not carry the frozen cutoff at 0.5.
tintype score --weights $W --manifest data/frozen-eval/test.tsv --out runs/<experiment>/test.tsv

# The tables. A frozen cutoff is the probability after `@`.
tintype report --operating-points \
  --system "candidate=runs/<experiment>/test.tsv@0.5" \
  --system "incumbent=runs/incumbent/test.tsv@0.5" \
  --fpr 0.001,0.01 --prevalence 0.01 \
  --slice-by family,source,transform --cluster-by parent \
  --groups data/frozen-eval/test.tsv --min-slice 100

# Latency, file to verdict, on each engine that will run it.
tintype latency --weights $W --manifest data/frozen-eval/test.tsv --n 500 --device gpu
tintype latency --weights $W --manifest data/frozen-eval/test.tsv --n 500 --device cpu
```

`calibrate` rewrites the spec beside `--weights`; that is why `$W` is a
copy under `runs/`. To evaluate the released model as the incumbent, copy
`models/tintype.safetensors` and `models/tintype.json` to
`runs/incumbent/` first and point every command at the copy.

## The selection rule

Written before the first candidate is scored. It applies to models, and
with the obvious reading to cutoffs, pools, margins and routing rules.

A candidate replaces the incumbent as the next model only if, on `test`,
at cutoffs frozen on `calibrate`:

1. **It gains on independent degraded files.** Recall on the degraded
   slices (every `transform` other than none) is higher at the 1%
   operating point by at least the gate its own runbook states, and is not
   lower at the 0.1% operating point.
2. **Authentic media hold.** The false-positive rate at each frozen cutoff
   stays within the cutoff's budget (its Clopper–Pearson upper bound does
   not exceed the incumbent's), and the worst authentic-source and
   worst-transform false-positive rates are not higher than the
   incumbent's.
3. **Clean recall holds**: within one point of the incumbent's on
   unprocessed generated items at both operating points.
4. **The cost is bounded**: p95 latency, route rate and memory within the
   budget its runbook states.

If two candidates pass, the one with the higher degraded recall at 0.1%
wins. If none passes, the incumbent stays and the candidate is reported as
having failed its gate. A model that is best at 1% and worse at 0.1% does
not pass.

Passing this rule makes a candidate a **ship candidate**, nothing more. It
then needs its calibration redone and its early exit re-checked with
everything else fixed, a new entry in
[PREREGISTRATION.md](../PREREGISTRATION.md), and a fresh blind batch. The
frozen `test` set is looked at once per candidate and the looks are
logged below; it is not a blind batch after the first look.

The gates in the report are proposed decision criteria. A runbook that
changes one says so in its pre-registration entry before it runs.

## Log

| Date | Commit | What was scored on which set | By |
|---|---|---|---|
| – | – | Nothing yet: the split has not been built. | – |
