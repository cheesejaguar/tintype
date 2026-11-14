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

**Candidate sources.** These are the collections the research notes name
as held-out candidates. None has been downloaded. For each one, before
downloading: read its licence at the source, record the licence and where
it is stated in [DATA.md](../DATA.md) or [NOTICE.md](../../NOTICE.md),
confirm the collection is absent from every training manifest, and pin the
revision the way the data plans do. A candidate whose licence does not
allow this use is dropped and replaced, and the replacement is written
into the pre-registration.

| Role | Candidates (research notes 03, 04, 08, 09) | Licence |
|---|---|---|
| Camera originals | RAISE beyond the 1k subset the `benchmarks` plan uses, Dresden, VISION, Forchheim | to verify before download |
| Phone photos | LAION-Mobile, or a recent-phone set | to verify before download |
| Shared and reposted | TrueFake (three platforms), RRDataset (WhatsApp, Instagram, multi-hop, recapture), WildRF | to verify before download |
| Autoregressive and tokenizer families | ARForensics (seven open AR generators) | to verify before download |
| Recent commercial generators | the GPT Image 2 / Nano Banana 2 set of note 09, GenImage++ | to verify before download |
| Human-hard | Chameleon | to verify before download |

**Held out whole.** Every source and family in the split is held out from
fitting detectors: no item of them may enter a training manifest while the
split is in use. Inside the split, the families and sources named with
`--hold-out-family` and `--hold-out-source` go to the test set whole, so
the test set also contains families and sources that the calibration set
has not seen. The pre-registration entry lists them by name. At least one
tokenizer family and at least one authentic source are held out this way.

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

# Scores (Apple GPU). --all-views writes every view logit, which the
# offline tools need.
tintype score --weights $W --manifest data/frozen-eval/calibrate.tsv --out runs/<experiment>/calibrate.tsv --all-views
tintype score --weights $W --manifest data/frozen-eval/test.tsv      --out runs/<experiment>/test.tsv

# Cutoffs come from the calibration set's authentic items only.
tintype calibrate --weights $W --scores runs/<experiment>/calibrate.tsv --profile photo --target-fpr 0.01
tintype verify-models                          # the released files are untouched

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
