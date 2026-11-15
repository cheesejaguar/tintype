# Runbook: calibrating the deployed decision, and the review outcome

This is workstream 2 of the [detection improvement research](../../reports/Tintype%20detection%20improvement%20research.md)
("Calibrate the deployed decision and add an inconclusive outcome"). It
builds on the [frozen evaluation runbook](frozen-eval.md), which fixes the
split, the operating points (recall at **0.1%** and **1%** false-positive
rate), the declared prevalence (**1%**) and how abstention is counted.

**Status.** The tooling exists and is tested, and the photo steps have
been run once on the frozen split for the released photo model
([below](#on-the-frozen-split-the-released-photo-model)): **the gate
failed**, at both operating points, on cutoff transfer. **No cutoff,
review band or exit margin has been fitted for shipping**, and none may be
until the views, the early exit, the model, the backend and any second
stage of the other workstreams are fixed. The video steps have not been
run: the frozen split has no video. The numbers under "Exploratory results
on the archives" come from archived development data and are labelled
exploratory. The released models and their specs are untouched.

## The gate

Written before the commands that test it. It is a **proposed** decision
criterion ("frozen cutoffs transfer to new authentic sources at useful
coverage"); the pre-registration entry confirms or changes each number
before the frozen `test` set is looked at.

A calibration (cutoffs, review band, exit margins) passes when, on `test`,
with everything fitted on `fit` and `calibrate` only:

1. **Cutoffs transfer.** At each frozen cutoff, with authentic parents
   counted once: no predeclared authentic stratum (each held-out source,
   each `transform`) is shown to exceed the cutoff's false-positive budget
   (`exceeds` in the transfer table: its one-sided 95% lower bound is over
   the budget), and the held-out authentic sources pooled are shown within
   **twice** the budget (upper bound at most 2 × budget). Strata the data
   cannot resolve are listed as not resolved, never counted as passes.
2. **Coverage is useful.** With the review band, at most **2%** of a
   stream at the declared prevalence goes to review, and recall with every
   reviewed AI item counted as a miss is reported beside recall without
   the band. Both operating points (0.1% and 1%) are reported.
3. **Disagreement earns its place.** The view-disagreement rule is used
   only if, at equal coverage on `calibrate` and again on `test`, it
   leaves fewer real items flagged without review and no lower recall than
   the margin alone. On the archives it does neither (below).
4. **Early exit still holds.** For every profile that shares the
   schedule, pooled precision and pooled recall under early exit are within
   0.001 of scoring every view, and the mean false-positive rate over real
   sources under early exit is within the profile's calibration target.

If 1 fails, the cutoff is reported as not transferring and nothing ships
on it. If 2 or 3 fails, the system ships without a review band, which is
today's behaviour. If 4 fails, the margins are refitted on `fit` or the
profile runs without early exit.

## Order

1. **Fix the system.** Views and their pooling, the early-exit order, the
   weights, the engine (GPU half precision or CPU), and any second stage.
   A change to any of them afterwards sends you back here.
2. **Score once**, every view, on each frozen set (Apple GPU).
3. **Fit the global cutoff of each profile** on `calibrate`, authentic
   items only, parents counted once.
4. **Refit the exit margins** on `fit` at the new thresholds, check them
   on `calibrate`, and re-express or re-choose the routing bands.
5. **Fit the review band** on `calibrate`, on the decisions of the
   refitted schedule: the band works on what will be deployed.
6. **Pre-register**: write the cutoffs, band, margins, budgets and strata
   into [PREREGISTRATION.md](../PREREGISTRATION.md).
7. **Look at `test` once**: transfer by strata, coverage, early exit. Log
   the look in [frozen-eval.md](frozen-eval.md#log).
8. **Blind batch**, as every earlier confirmation was run.

Steps 3 to 5 never read `test`. The transfer measurement and the coverage
measurement both happen in step 7, on decisions that are already frozen.

## Sizing

The bound counts **independent** authentic inputs. Derivatives of one photo
are one input: pass the manifest (`--groups`) so that parents are counted.

| Budget | Authentic parents needed with none flagged (one-sided 95%) |
|---|---|
| 0.10% | 2,995 (about 3,000) |
| 1% | 299 |
| 1.5% | 199 |

With fewer, `calibrate --rule bound` stops and says how many it needs. The
same counts apply to every stratum that is to be shown `within` its
budget: a source with 500 parents cannot show 0.1% whatever its scores.
With zero flagged among `n`, the bound is `1 − 0.05^(1/n)`.

Use clustered intervals wherever sources or parents repeat:
`--cluster-by parent --groups <manifest>` in `report`, which resamples
whole parents for recall intervals and counts parents in the transfer
table.

## Scores are not probabilities

The score is the class-balanced sigmoid of a calibrated logit: 0.5 sits at
a false-positive target, and the slope is fitted with both classes weighted
equally. It is a detector score. What a flag is worth depends on
prevalence: `precision = recall·π / (recall·π + FPR·(1 − π))`.
`report --operating-points --prevalence 0.01` prints that column. At 1%
prevalence, confirmation 7's shipped rates (89.70% recall, 130 of 10,747
authentic photos flagged) give 42.83% precision, if they transferred;
90% precision at 90% recall needs about 0.10% FPR.

## Group cutoffs

There is one false-alarm budget and one cutoff per profile. No per-group
cutoff is fitted, stored or applied anywhere in the code. A cutoff that
depends on a group would be admissible only where

- the group can be **observed at inference** (not a label that exists only
  in a benchmark),
- the group has **enough calibration data** of its own (the table above),
  and
- the group cutoff is **shown to transfer** on separate data.

The NTIRE operation labels (JPEG AI, Cheng2020, WMForger and the rest)
fail the first condition: they are available for diagnosis only. The final
JPEG quantisation tables and subsampling are identical across the NTIRE
splits ([EVIDENCE.md](../EVIDENCE.md#ntire-threshold-transfer-and-artifact-diagnostics)),
so file metadata cannot recover an operation history.

`calibrate --group-diagnostic source` (or `transform`, `dataset`,
`generator`, `family`) prints what each group's own cutoff would be and
what the global one does to it. Every line is marked `[diagnostic]`;
nothing is written.

## Commands

Run from the directory that holds `data/`. A candidate lives under
`runs/`, never under `models/`: `calibrate` and `refit-exit --write`
rewrite the spec beside `--weights`.

```bash
E=runs/<experiment>
W=$E/<name>.safetensors                      # with <name>.json beside it
# To calibrate the released model as the incumbent, work on a copy:
#   cp models/tintype.safetensors $E/incumbent.safetensors
#   cp models/tintype.json        $E/incumbent.json
```

### 2. Score every view once (Apple GPU; more than 2,000 items)

```bash
for set in fit calibrate test; do
  tintype score --weights $W --manifest data/frozen-eval/$set.tsv --out $E/$set-views.tsv --all-views
done
tintype score --weights $W --fast --manifest data/frozen-eval/calibrate.tsv --out $E/calibrate-fast.tsv
# Video candidate ($V, with its own spec):
for set in fit calibrate test; do
  tintype score-video --weights $V --manifest data/frozen-eval/$set-video.tsv --out $E/$set-frames.tsv --all-frames
done
```

Everything below is arithmetic on these files. `calibrate` is built on
macOS only but runs no network; `refit-exit` and `report` run anywhere.

### 3. The global cutoff of each profile, on `calibrate`

```bash
# Targets are the budgets the pre-registration names; these are today's.
tintype calibrate --weights $W --scores $E/calibrate-views.tsv --profile photo     --target-fpr 0.015 \
  --rule bound --confidence 0.95 --groups data/frozen-eval/calibrate.tsv --per-parent max
tintype calibrate --weights $W --scores $E/calibrate-views.tsv --profile strict    --target-fpr 0.005 \
  --rule bound --groups data/frozen-eval/calibrate.tsv
tintype calibrate --weights $W --scores $E/calibrate-views.tsv --profile sensitive --target-fpr 0.05 \
  --rule bound --groups data/frozen-eval/calibrate.tsv
tintype calibrate --weights $W --scores $E/calibrate-fast.tsv  --profile fast      --target-fpr 0.005 \
  --rule bound --groups data/frozen-eval/calibrate.tsv
tintype calibrate --weights $V --scores $E/calibrate-frames.tsv --profile video    --target-fpr 0.005 \
  --rule bound --groups data/frozen-eval/calibrate-video.tsv
tintype verify-models                         # the released files are untouched
```

`--rule bound` picks the loosest threshold whose one-sided Clopper–Pearson
upper bound on the false-positive rate, over independent authentic items,
is within the target. The default rule is still `balanced` (a quantile of
the mean per-source rate); run both and compare:

```bash
cp $E/<name>.json $E/<name>-balanced.json && cp $W $E/<name>-balanced.safetensors
tintype calibrate --weights $E/<name>-balanced.safetensors --scores $E/calibrate-views.tsv --profile photo --target-fpr 0.015
```

Every rule prints the threshold, the count flagged among independent
items and the bound that count achieves, and the per-source rates.

**One score per parent.** With `--groups`, real items that share a
`parent_id` count once. `--per-parent max` (the default) takes the highest
score among a parent's versions: the parent counts as flagged when any
version is, so the bound covers every processing in the calibration set,
at the price of a stricter cutoff. `--per-parent original` takes the
unprocessed original only: the bound then speaks for originals alone, and
transfer to processed copies has to be shown in step 7. Declare which in
the pre-registration.

### 4. Exit margins and routing bands

The exit margins are raw-logit distances from one profile's threshold.
After step 3 the thresholds have moved, so the stored schedule is checked
again, and refitted on `fit`:

```bash
# Check the stored schedule at the new thresholds (the three profiles share it).
tintype refit-exit --weights $W --profile photo,strict,sensitive --target-fpr 0.015,0.005,0.05 \
  --report $E/calibrate-views.tsv

# Fit on `fit`, report on `calibrate`, store in the copy.
tintype refit-exit --weights $W --profile photo,strict,sensitive --target-fpr 0.015,0.005,0.05 \
  --fit $E/fit-views.tsv --report $E/calibrate-views.tsv --tolerance 0.001 --write

# Clips: the schedule counts frames.
tintype refit-exit --weights $V --profile video --target-fpr 0.005 \
  --fit $E/fit-frames.tsv --report $E/calibrate-frames.tsv --write
tintype verify-models
```

It prints, per profile: the verdicts that differ from scoring every view,
pooled false-positive rate, recall and precision with and without early
exit, each real source's rate under early exit against the target, the
mean over sources (the quantity `calibrate --rule balanced` sets), recall
per generated source, and the views looked at. `--write` refuses any path
under `models/`.

The fit is a greedy descent: from "never stop early" it tightens, one
step at a time, the margin that saves the most views while pooled
precision and recall stay within the tolerance at every listed threshold.
It is repeatable and never breaks the tolerance on the fit items; it is
not a global optimum.

**Routing bands.** The photo second-opinion band is in the default
profile's calibrated logits, so step 3 silently moves it. `refit-exit`
prints the band in raw logits and how many items it routes. To keep
exactly the same photos routed and the same second-stage rule under the
new calibration:

```bash
tintype refit-exit --weights $W --profile photo --report $E/calibrate-views.tsv \
  --fit $E/fit-views.tsv --rebase-band-from $E/<spec the band was chosen under>.json --write
```

That is the neutral starting point, not a refit. Choosing a new band, or
calibrating the combined decision to a false-positive budget, needs
second-stage scores on `calibrate` and belongs to the routing workstream.
The video band is in raw logits and does not move; `refit-exit --profile
video` prints where the new threshold sits in it.

### 5. The review band, on `calibrate`

The band works on the deployed decision, so run it after step 4 has
stored the refitted schedule, and first write what the profile decides
(early exit and calibration applied) from the all-views file:

```bash
tintype refit-exit --weights $W --profile photo --report $E/calibrate-views.tsv --apply-out $E/calibrate-decided.tsv
tintype report --operating-points --system "candidate=$E/calibrate-decided.tsv@0.5" \
  --prevalence 0.01 --cluster-by parent --groups data/frozen-eval/calibrate.tsv \
  --review-margin 0.25,0.5,0.75,1,1.5,2,3
```

The rule for choosing the margin, declared here: **the widest margin whose
share of a stream reviewed at the declared prevalence is at most the
review budget (2%)**. Then test the disagreement rule against the margin
alone at the same coverage (`threshold_logit` is the profile's raw
threshold, printed by `refit-exit` and by `detect --json`):

```bash
tintype report --operating-points --system "candidate=$E/calibrate-decided.tsv@0.5" \
  --review-margin <narrower> --review-disagreement 0.5 --review-threshold-logit <threshold_logit>
```

Store the band in the candidate's spec (the default profile reads
`review`, clips read `video.review`), or pass it as flags at run time:

```bash
python3 - "$E/<name>.json" <<'PY'
import json, sys
spec = json.load(open(sys.argv[1]))
spec["review"] = {"margin": 1.0}          # the fitted margin
open(sys.argv[1], "w").write(json.dumps(spec, indent=2))
PY
tintype detect --weights $W --review-margin 1.0 --json <files>     # the same, as a flag
```

What the fields mean, and the precedence when both reasons apply (the
margin), is in [API.md](../API.md#review-fields-opt-in).

### 7. One look at `test`

```bash
tintype refit-exit --weights $W --profile photo,strict,sensitive --target-fpr 0.015,0.005,0.05 \
  --report $E/test-views.tsv --apply-out $E/test-decided.tsv        # gate 4
tintype report --operating-points --system "candidate=$E/test-decided.tsv@0.5" \
  --fpr 0.001,0.01 --prevalence 0.01 \
  --slice-by source,transform,family --cluster-by parent --groups data/frozen-eval/test.tsv --min-slice 100 \
  --transfer-budget 0.015 \
  --review-margin <the fitted margin>                               # gates 1, 2, 3
```

`--transfer-budget` prints, after each slice table, every authentic
stratum's count at the frozen cutoff with one-sided 95% Clopper–Pearson
bounds and a verdict: `within` (upper bound inside the budget), `exceeds`
(lower bound over it) or `not resolved`. Repeat with the other profiles'
decided files (`--apply-out` writes the first profile listed) and their
budgets.

To account for review decisions exactly as `detect` made them, instead of
recomputing them from the score file:

```bash
tintype detect --weights $W --review-margin 1.0 --json <files> > $E/detect.jsonl
tintype report --operating-points --system "candidate=$E/test-decided.tsv@0.5" --review-flags $E/detect.jsonl
```

## On the frozen split: the released photo model

Run on 2026-11-14 at commit `caa15c3` (the pre-registration entry, written
and committed before the look; the code is that of `ec1b8e5`), CPU only, on
an Apple M6 (12 cores, 32 GB, macOS 27.2). Entry:
[PREREGISTRATION.md](../PREREGISTRATION.md#calibration-and-the-review-outcome-on-the-frozen-split-what-was-fixed-before-the-look-at-test).
Raw output of every command and the four specs:
[`docs/results/frozen-eval-calibration/`](../results/frozen-eval-calibration/).

**Verdict: the gate failed.** Gate 1 (cutoffs transfer) fails at 1% and at
0.1%; gates 2 and 4 pass; gate 3 fails, so the disagreement rule is not
used. Nothing ships on these cutoffs, and the released cutoffs, margins
and specs are unchanged.

What was and was not run:

- **No new scoring.** The system is the released photo model, and every
  step is arithmetic on the incumbent's all-views score files
  (`{fit,calibrate,test}.all.tsv`, archived as
  [`docs/results/frozen-eval-incumbent/*.all.tsv.gz`](../results/frozen-eval-incumbent/)).
  `$F` below is the folder that holds them and `$D` the folder of the set
  manifests (`data/frozen-eval`). `calibrate` and `refit-exit` read the
  spec beside `--weights` and no weights, so each arm is a copy of
  `models/tintype.json` under `runs/frozen/ws2/`.
- **The fast profile was not scored again.** Its two views are views 1
  and 3 of the six, so its score file is those two columns.
- **Video: not run.** The frozen split has no video data. The video
  commands above stand as written.
- **Latency: not measured.** The machine was shared with a training run.
- **Order.** The exit margins were refitted before the review band was
  fitted, so that the band is fitted on the decisions that would be
  deployed; the Order section above now says so.

Caveats that go with every number here: every authentic original is a
camera JPEG of 12 megapixels or more and every generated original is a PNG
of 256 to 1,024 px, so the clean rows carry that confound (the processed
copies re-encode both classes the same way). FloreView is 36 scenes
photographed by 46 phones and CSAFE is ten units of one phone model:
intervals and bounds count parents, and scenes and devices are far fewer.
There are no real platform round trips and no learned-codec copies. `test`
is not blind: it had been looked at for the incumbent and for the
processing-chain arms, and the pre-registration entry says what I already
knew from those looks.

### Cutoffs, fitted on `calibrate`

```bash
E=runs/frozen/ws2; mkdir -p $E
for arm in bound-max bound-orig balanced; do cp models/tintype.json $E/$arm.json; done
# Arm A: the bound rule, parents counted once, the highest score of a parent's versions.
tintype calibrate --weights $E/bound-max.safetensors --scores $F/calibrate.all.tsv --profile photo     --target-fpr 0.01  --rule bound --confidence 0.95 --groups $D/calibrate.tsv --per-parent max
tintype calibrate --weights $E/bound-max.safetensors --scores $F/calibrate.all.tsv --profile strict    --target-fpr 0.001 --rule bound --groups $D/calibrate.tsv
tintype calibrate --weights $E/bound-max.safetensors --scores $F/calibrate.all.tsv --profile sensitive --target-fpr 0.05  --rule bound --groups $D/calibrate.tsv
awk -F'\t' 'BEGIN{OFS="\t"}{print $1,$2,$3,$4,$5,$6,$8}' $F/calibrate.all.tsv > $E/calibrate.fast.tsv
tintype calibrate --weights $E/bound-max.safetensors --scores $E/calibrate.fast.tsv --profile fast     --target-fpr 0.005 --rule bound --groups $D/calibrate.tsv
# Arm B: the same with --per-parent original (photo, strict, sensitive), on bound-orig.
# Arm C: the default rule, no --rule and no --groups (photo, strict, sensitive), on balanced.
tintype verify-models
```

`calibrate` holds 3,457 authentic parents; 200 of them also have twelve
processed copies (5,857 authentic rows) and there are 6,600 generated rows.

| Arm | Rule | 1% (photo profile) | 0.1% (strict profile) | 5% (sensitive) | 0.5% (fast) |
|---|---|---|---|---|---|
| **A** | bound, one score per parent, the highest of its versions | 1.636: 24 of 3,457 parents flagged, bound 0.975%, recall 0.8617 | 3.504: 0 flagged, bound 0.087%, recall 0.7980 | −0.494: 151 flagged, bound 4.984%, recall 0.9114 | 2.280: 10 flagged, bound 0.490%, recall 0.8406 |
| B | bound, the unprocessed original only | 0.450: 24 flagged, 0.975%, recall 0.8891 | 2.563: 0 flagged, 0.087%, recall 0.8333 | −1.384: 151 flagged, 4.984%, recall 0.9288 | not fitted |
| C | default (`balanced`), rows as items | 0.985: 58 of 5,857 rows flagged, bound 1.231%, recall 0.8756 | 2.965: 5 flagged, 0.179%, recall 0.8188 | −0.795: 297 flagged, 5.568%, recall 0.9171 | not fitted |

Thresholds are raw logits on the mean of the views; recall is on the
generated rows of `calibrate` with every view. Arm C's photo and strict
thresholds are the ones the incumbent's refit wrote. Counting parents and
taking the highest version makes arm A the strictest of the three; taking
originals only makes arm B the loosest.

### Exit margins, fitted on `fit`, reported on `calibrate`

```bash
tintype refit-exit --weights $E/bound-max.safetensors --profile photo,strict,sensitive --target-fpr 0.01,0.001,0.05 \
  --report $F/calibrate.all.tsv                                           # the shipped margins at the new thresholds
cp $E/bound-max.json $E/bound-max.before-refit.json
tintype refit-exit --weights $E/bound-max.safetensors --profile photo,strict,sensitive --target-fpr 0.01,0.001,0.05 \
  --fit $F/fit.all.tsv --report $F/calibrate.all.tsv --tolerance 0.001 --write
tintype verify-models
```

Fitted on the 3,759 items of `fit` for the three profiles at once:
**6.5, 4, 4, 1.75, 1** raw logits after 1 to 5 views (shipped: 6, 3, 2,
1.5, 1). On `calibrate` (12,457 rows), early exit against every view:

| Profile (threshold) | Margins | Views per image | Verdicts that differ | Authentic rows flagged: every view, early exit | Recall: every view, early exit | Precision: every view, early exit |
|---|---|---|---|---|---|---|
| photo (1.636) | shipped | 1.66 | 45 | 28, 25 of 5,857 | 86.17%, 86.14% | 0.9951, 0.9956 |
| photo (1.636) | fitted | 1.87 | 19 | 28, 26 | 86.17%, 86.15% | 0.9951, 0.9954 |
| strict (3.504) | shipped | 1.66 | 63 | 0, 0 | 79.80%, 79.85% | 1.0000, 1.0000 |
| strict (3.504) | fitted | 1.86 | 30 | 0, 0 | 79.80%, 79.74% | 1.0000, 1.0000 |
| sensitive (−0.494) | shipped | 1.86 | 77 | 220, 199 | 91.14%, 91.35% | 0.9647, 0.9680 |
| sensitive (−0.494) | fitted | 2.16 | 32 | 220, 214 | 91.14%, 91.23% | 0.9647, 0.9657 |

The shipped margins leave the 0.001 tolerance on the sensitive profile
(precision +0.0033, recall +0.0021); the fitted ones halve the verdicts
that differ at the cost of about 0.2 to 0.3 more views per image.

### Review band, fitted on `calibrate`

```bash
for p in photo strict; do
  tintype refit-exit --weights $E/bound-max.safetensors --profile $p --report $F/calibrate.all.tsv --apply-out $E/calibrate.decided-$p.tsv
  tintype report --operating-points --system "bound-max-$p=$E/calibrate.decided-$p.tsv@0.5" --prevalence 0.01 \
    --cluster-by parent --groups $D/calibrate.tsv --review-margin 0.25,0.5,0.75,1,1.25,1.5,1.75,2,2.5,3
done
```

The declared rule (the widest margin with at most 2% of a stream reviewed
at 1% prevalence) gives **0.75** at the 1% cutoff and **1.5** at the 0.1%
cutoff:

| Cutoff | Margin | Stream reviewed at 1% prevalence | Authentic rows flagged without review | Recall, reviewed AI items as misses |
|---|---|---|---|---|
| 1% | none | 0.00% | 26 of 5,857 | 86.15% |
| 1% | 0.5 | 0.78% | 10 | 84.06% |
| 1% | **0.75** | 1.44% | 4 | 82.45% |
| 1% | 1 | 2.24% | 2 | 80.65% |
| 0.1% | none | 0.00% | 0 of 5,857 | 79.74% |
| 0.1% | 1.25 | 1.13% | 0 | 66.71% |
| 0.1% | **1.5** | 1.58% | 0 | 64.02% |
| 0.1% | 1.75 | 3.06% | 0 | 58.95% |

At the 0.1% cutoff no authentic row of `calibrate` is flagged with or
without a band, so there the band gives up 15.72 points of automatic
recall and removes no false alarm on these rows. The rule looks only at
review capacity; it was applied as declared.

**Gate 3, on `calibrate`: failed.** With the photo profile's raw
threshold 1.6363:

```bash
tintype report --operating-points --system "p=$E/calibrate.decided-photo.tsv@0.5" --prevalence 0.01 \
  --review-margin 0.25 --review-disagreement 0.5 --review-threshold-logit 1.6363401638331643
```

| Band | Coverage | Authentic rows flagged without review | Recall, reviewed AI items as misses |
|---|---|---|---|
| margin 0.5 | 97.56% | 10 of 5,857 | 84.06% |
| margin 0.25, or half the views disagreeing | 97.30% | 16 | 83.85% |
| margin 0.6 | 96.73% | 8 | 83.32% |
| margin 0.5, or half the views disagreeing | 96.52% | 10 | 83.30% |

The margin alone covers more, leaves fewer authentic rows flagged and
keeps more recall. The disagreement rule was not computed on `test`.

### The look at `test`

```bash
# Gate 4, and the decisions of each profile under the fitted schedule.
tintype refit-exit --weights $E/bound-max.safetensors --profile photo,strict,sensitive --target-fpr 0.01,0.001,0.05 \
  --report $F/test.all.tsv --apply-out $E/test.A-photo.tsv
tintype refit-exit --weights $E/bound-max.before-refit.safetensors --profile photo,strict,sensitive --target-fpr 0.01,0.001,0.05 --report $F/test.all.tsv
tintype refit-exit --weights $E/bound-max.safetensors --profile strict    --report $F/test.all.tsv --apply-out $E/test.A-strict.tsv
tintype refit-exit --weights $E/bound-max.safetensors --profile sensitive --report $F/test.all.tsv --apply-out $E/test.A-sensitive.tsv
# (arms B and C the same, on bound-orig and balanced; the fast profile's file is
#  columns 6 and 8 of test.all.tsv through the fast calibration, no early exit)
# Gates 1 and 2.
tintype report --operating-points --system "A-photo=$E/test.A-photo.tsv@0.5" --fpr 0.001,0.01 --prevalence 0.01 \
  --slice-by source,transform,family --cluster-by parent --groups $D/test.tsv --min-slice 100 \
  --transfer-budget 0.01 --review-margin 0.75
tintype report --operating-points --system "A-strict=$E/test.A-strict.tsv@0.5" --fpr 0.001,0.01 --prevalence 0.01 \
  --slice-by source,transform,family --cluster-by parent --groups $D/test.tsv --min-slice 100 \
  --transfer-budget 0.001 --review-margin 1.5
```

`test` has 9,726 authentic rows of 7,326 parents (3,996 of them the
held-out phone source) and 9,600 generated rows of 7,200 parents. The
clean and degraded rows of a decided file were separated by the
`transform` column of the manifest and reported the same way.

**At the cutoffs frozen on `calibrate`** (early exit as each arm would
deploy it: fitted margins for arm A, shipped margins for B and C):

| Arm, cutoff | Authentic rows flagged, all (upper bound) | Recall, all | Clean: flagged, recall | Degraded: flagged, recall |
|---|---|---|---|---|
| **A, 1%** | 82 of 9,726, 0.84% (1.01%) | 90.98% | 35 of 7,326 (0.48%), 94.90% | 47 of 2,400 (1.96%), 79.21% |
| B, 1% | 185, 1.90% (2.15%) | 93.26% | 82 (1.12%), 96.14% | 103 (4.29%), 84.62% |
| C, 1% | 120, 1.23% (1.43%) | 92.34% | 51 (0.70%), 95.71% | 69 (2.88%), 82.25% |
| **A, 0.1%** | 12, 0.12% (0.20%) | 86.12% | 6 (0.08%), 91.72% | 6 (0.25%), 69.33% |
| B, 0.1% | 29, 0.30% (0.41%) | 88.82% | 9 (0.12%), 93.47% | 20 (0.83%), 74.88% |
| C, 0.1% | 18, 0.19% (0.27%) | 87.70% | 5 (0.07%), 92.72% | 13 (0.54%), 72.62% |
| A, 5% (sensitive) | 368, 3.78% (4.12%) | 94.73% | – | – |
| A, 0.5% (fast) | 37, 0.38% (0.50%) | 89.47% | – | – |

Arm C reproduces the incumbent's logged numbers (120 and 18 flagged). Arm
A is the only arm whose pooled rate is under its budget at 1% (0.84%; the
upper bound, 1.01%, is not), and it comes closest at 0.1% (0.12% against
0.10%), for 1.4 and 1.6 points of recall less than arm C.

**At the empirical cutoffs of `test` itself** (a ranking measure, not a
transfer result; from arm A's decided files, the photo file for 1% and the
strict file for 0.1%, since early exit follows the profile's threshold):

| Rows | Recall at 1% empirical FPR | Recall at 0.1% empirical FPR |
|---|---|---|
| all | 91.47% (90.58%–92.36%) | 84.52% (83.29%–85.77%) |
| clean | 96.08% (95.62%–96.54%) | 92.85% (92.25%–93.40%) |
| degraded | 75.04% (71.67%–78.28%) | 62.88% (59.10%–66.67%) |

**Gate 1, cutoffs transfer: failed at both operating points.** Authentic
parents counted once per stratum; one-sided 95% bounds.

| Stratum | 1% cutoff: flagged (lower to upper bound) | Against 1% | 0.1% cutoff: flagged (lower to upper bound) | Against 0.1% |
|---|---|---|---|---|
| `noise-a10` | 21 of 200, 10.50% (7.14% to 14.77%) | **exceeds** | 1 of 200 (0.03% to 2.35%) | not resolved |
| `noise-a40` | 13 of 200, 6.50% (3.89% to 10.14%) | **exceeds** | 5 of 200, 2.50% (0.99% to 5.18%) | **exceeds** |
| `csafe-iphone14pro` (held out) | 46 of 3,996, 1.15% (0.89% to 1.47%) | not resolved; within twice the budget | 10 of 3,996, 0.25% (0.14% to 0.42%) | **exceeds**; not within twice the budget |
| `floreview-xiaomi` | 4 of 292 (0.47% to 3.11%) | not resolved | 2 of 292, 0.68% (0.12% to 2.14%) | **exceeds** |
| `hdrplus` | 7 of 1,456, 0.48% (0.23% to 0.90%) | within | 0 of 1,456 (0.00% to 0.21%) | not resolved |
| unprocessed (`none`) | 35 of 7,326, 0.48% (0.35% to 0.63%) | within | 6 of 7,326, 0.08% (0.04% to 0.16%) | not resolved |
| the other 20 strata | 0 to 3 flagged each | not resolved | none flagged | not resolved |

At 1%: 2 of 26 strata exceed the budget, 2 are within it (HDR+ and the
unprocessed rows), 22 are not resolved. At 0.1%: 3 exceed, none is within,
23 are not resolved. The added-noise copies are where the cutoff does not
hold, by a wide margin at 1%; at 0.1% the held-out phone source also
exceeds, by a smaller one (lower bound 0.14% against 0.10%). Twenty-six
strata were each judged at 95%, so a stratum that exceeds narrowly on one
or two flags (FloreView Xiaomi at 0.1%) is weaker evidence than its label
suggests; the noise strata and the phone source do not depend on that.
Arms B and C fail the same gate on more strata: at 1%, arm C exceeds on
the phone source (61 of 3,996), FloreView Xiaomi, both noise copies and
`repost-720-qv9x1`; arm B on those and four more.

The sensitive and fast cutoffs of arm A fail it too: at 5%, FloreView
Doogee (14 of 42), `blur-s3.0` (27 of 200), `noise-a10` (62 of 200) and
`noise-a40` (36 of 200) exceed; at 0.5%, `noise-a10` (6 of 200) does.

**Gate 2, coverage: passed.** Arm A with its bands, on `test`:

| Cutoff, margin | Stream reviewed at 1% prevalence | Coverage of these rows | Authentic rows reviewed | Generated rows reviewed | Authentic rows flagged without review (upper bound) | Recall, reviewed generated rows as misses | Recall without the band |
|---|---|---|---|---|---|---|---|
| 1%, 0.75, all | **1.67%** | 96.44% | 159 of 9,726 (1.63%) | 529 of 9,600 (5.51%) | 24, 0.25% (0.35%) | 88.00% (86.96%–89.12%) | 90.98% |
| 1%, 0.75, clean | 0.98% | 97.81% | 70 of 7,326 (0.96%) | 248 of 7,200 (3.44%) | 9, 0.12% (0.21%) | 92.78% | 94.90% |
| 1%, 0.75, degraded | 3.79% | 92.29% | 89 of 2,400 (3.71%) | 281 of 2,400 (11.71%) | 15, 0.62% (0.96%) | 73.67% | 79.21% |
| 0.1%, 1.5, all | **1.77%** | 88.81% | 153 of 9,726 (1.57%) | 2,009 of 9,600 (20.93%) | 0, 0.00% (0.03%) | 72.95% (71.47%–74.44%) | 86.12% |
| 0.1%, 1.5, clean | 1.01% | 91.61% | 63 of 7,326 (0.86%) | 1,156 of 7,200 (16.06%) | 0 (0.04%) | 80.15% | 91.72% |
| 0.1%, 1.5, degraded | 4.07% | 80.35% | 90 of 2,400 (3.75%) | 853 of 2,400 (35.54%) | 0 (0.12%) | 51.33% | 69.33% |

Both bands stay under the 2% review budget on all of `test`. The price is
what the accounting is for: with reviewed generated items counted as
misses, recall falls by 2.98 points at 1% and by 13.17 points at 0.1%. At
0.1% all twelve false alarms on authentic rows land in review, so no
authentic row is flagged automatically; one generated row in five goes to
a person. On the degraded slice alone the review share is about 4%, twice
the budget: a stream of processed files would need a narrower band.
Coverage here is on the rows of `test`, which are half generated; the
share of a stream at 1% prevalence is the gate's quantity.

**Gate 3, disagreement: failed on `calibrate`** (above); not used.

**Gate 4, early exit: passed with the fitted margins.** On `test`, early
exit against every view:

| Profile (threshold) | Margins | Views per image | Verdicts that differ | Authentic rows flagged: every view, early exit | Recall | Precision | Mean rate over real sources, early exit (target) |
|---|---|---|---|---|---|---|---|
| photo (1.636) | fitted | 1.70 | 29 | 86, 82 of 9,726 | 90.93%, 90.98% | 0.9902, 0.9907 | 0.68% (1%) |
| strict (3.504) | fitted | 1.71 | 31 | 15, 12 | 86.12%, 86.12% | 0.9982, 0.9986 | 0.09% (0.1%) |
| sensitive (−0.494) | fitted | 1.99 | 20 | 372, 368 | 94.65%, 94.73% | 0.9607, 0.9611 | 3.49% (5%) |
| photo (1.636) | shipped | 1.53 | 61 | 86, 67 | 90.93%, 91.01% | 0.9902, 0.9924 | 0.58% |
| strict (3.504) | shipped | 1.53 | 68 | 15, 9 | 86.12%, 86.17% | 0.9982, 0.9989 | 0.07% |
| sensitive (−0.494) | shipped | 1.73 | 83 | 372, 336 | 94.65%, 94.78% | 0.9607, 0.9644 | 3.18% |

With the fitted margins every pooled rate is within 0.001 of scoring
every view, and the per-source means are within the targets. The shipped
margins at these thresholds are outside the tolerance on precision for the
photo profile (+0.0021) and the sensitive profile (+0.0037), in the
direction of fewer flags, for about 0.2 fewer views per image.

### What this says

- **A bound on the calibration population did not carry to the strata.**
  The bound rule did what it promises on `calibrate` (0.975% and 0.087%),
  and pooled over `test` it comes close (0.84% and 0.12%). The failure is
  in particular strata: authentic photos with added noise at both points,
  and the phone model that calibration never saw at 0.1%. Only 200 of the
  3,457 calibration parents have processed copies, so the noise copies
  barely move a pooled cutoff. No rule fitted on pooled authentic items
  fixes that; it is a ranking problem on those inputs (workstream 1) or a
  calibration set whose strata are each large enough to be bounded.
- **The 0.1% point cannot be resolved by stratum on this split.** Apart
  from the unprocessed rows as a whole, no stratum has the 2,995 parents
  that showing 0.1% takes, and the unprocessed rows are not shown within
  it either (6 of 7,326, upper bound 0.16%).
- **Counting parents with their worst version is the most conservative
  rule of the three and the closest to budget on `test`**; taking
  originals only overshoots most (1.90% and 0.30%).
- **The review band meets its capacity budget and removes most automatic
  false alarms** (82 to 24 at 1%, 12 to 0 at 0.1%), at a recall cost that
  is large at 0.1%. A margin band is a stricter cutoff for automatic
  flags plus a queue; whether the queue is worth 13 points of automatic
  recall is a product decision the gate does not make.
- **View disagreement did not help**, on the archives or here.
- **Exit margins need refitting when thresholds move**, and the refit on
  `fit` held on `calibrate` and on `test`.

## The NTIRE regression case (diagnostic)

Kept as a standing check that a low rate on the calibration cohort is not
a transfer result. It is archived development data. A unit test
(`calibration::tests::the_archived_ntire_cutoffs_do_not_transfer`) pins
the counts; the archive replay prints the cohort counts they rest on:

```bash
python3 scripts/replay-research.py --hard-labels val_hard_labels.csv --public-labels test_labels.csv
```

The same case through the report command:

```bash
mkdir -p runs/ws2/ntire
for split in hard-validation public-test; do
  gzcat docs/results/ntire-2026-$split/scores.tsv.gz | tail -n +2 | cut -f1-5 > runs/ws2/ntire/$split.tsv
  awk -F'\t' '$4=="ntire-clean"'     runs/ws2/ntire/$split.tsv > runs/ws2/ntire/$split-clean.tsv
  awk -F'\t' '$4=="ntire-distorted"' runs/ws2/ntire/$split.tsv > runs/ws2/ntire/$split-distorted.tsv
done
tintype report --operating-points --system "clean=runs/ws2/ntire/hard-validation-clean.tsv@0.5" --fpr 0.01
tintype report --operating-points --system "distorted=runs/ws2/ntire/hard-validation-distorted.tsv@0.5" --fpr 0.01
tintype report --operating-points --system "clean-cutoff=runs/ws2/ntire/public-test.tsv@0.941085" \
  --fpr 0.01 --slice-by dataset --transfer-budget 0.01
tintype report --operating-points --system "distorted-cutoff=runs/ws2/ntire/public-test.tsv@0.996498" \
  --fpr 0.01 --slice-by dataset --transfer-budget 0.01
```

Measured at commit `abb2532`, CPU only, Apple M6 (Mac18,5), 32 GB:

| Cutoff, fitted on hard validation at 1% empirical FPR | On its own cohort | Public clean authentic | Public distorted authentic | Public distorted recall |
|---|---|---|---|---|
| Clean, 0.941085 | 6 of 625 (0.96%) | 10 of 600 (1.67%), not resolved | 63 of 600 (10.50%), exceeds a 1% budget (lower bound 8.51%) | 21.69% |
| Distorted, 0.996498 | 6 of 625 (0.96%) | 2 of 600 (0.33%), not resolved | 13 of 600 (2.17%), exceeds a 1% budget (lower bound 1.29%) | 4.77% |

The order-statistic rule on the same 625 clean authentic images allows one
flag instead of six, and its cutoff is still shown to exceed 1% on the
public distorted images (the unit test asserts it). A bound holds for the
population the calibration items were drawn from, and for no other.

## Exploratory results on the archives

Development data: the released model's own calibration scores
(`runs/scores-v7/calib_images.tsv`, 82,948 distinct items with six view
logits) and confirmation 7's all-views files (`runs/scores-c7/*.veracity-views.tsv`,
23,052 items). Confirmation 7 has been looked at many times and shares
sources with the calibration data, so **nothing here is a transfer result,
and nothing here was selected for shipping.** All at commit `abb2532`, CPU
only, Apple M6 (Mac18,5), 32 GB; `$A` is the archive's `runs/` directory
and `runs/ws2/incumbent.*` is a copy of the released photo model and spec.

**The tools reproduce the archive.** The default rule on the archived
calibration scores writes a spec byte-identical to `models/tintype.json`,
and `refit-exit --apply-out` on the all-views files gives the 30 archived
default-profile score files row for row (130 of 10,747 authentic flagged,
89.70% recall, 1.69 views per image).

```bash
tintype calibrate --weights runs/ws2/incumbent.safetensors --scores $A/scores-v7/calib_images.tsv --profile photo --target-fpr 0.015
tintype refit-exit --weights runs/ws2/incumbent.safetensors --profile photo --report $A/scores-c7/*.veracity-views.tsv \
  --target-fpr 0.015 --apply-out runs/ws2/c7-decided.tsv
```

**Quantile rule against the bound rule** (in-sample on the calibration
scores, then applied to confirmation 7 under early exit). Items are taken
as independent because these files have no parent ids.

```bash
cp runs/ws2/incumbent.json runs/ws2/bound.json; cp runs/ws2/incumbent.safetensors runs/ws2/bound.safetensors
tintype calibrate --weights runs/ws2/bound.safetensors --scores $A/scores-v7/calib_images.tsv --profile photo --target-fpr 0.01 --rule bound
tintype refit-exit --weights runs/ws2/bound.safetensors --profile photo --report $A/scores-c7/*.veracity-views.tsv --target-fpr 0.01 --apply-out runs/ws2/c7-bound.tsv
tintype report --operating-points --system "bound-1pct=runs/ws2/c7-bound.tsv@0.5" --slice-by source --transfer-budget 0.01
```

| Rule and target | Raw threshold | Calibration: authentic flagged (upper bound) | Calibration recall | Confirmation 7: authentic flagged | Confirmation 7 recall | Confirmation 7 source strata against the target |
|---|---|---|---|---|---|---|
| balanced, 1.5% (shipped) | 1.692 | 472 of 36,110 (1.410%) | 0.8777 | 130 of 10,747 (1.21%) | 89.70% | 1 within, 4 not resolved |
| bound, 1.5% | 1.576 | 503 of 36,110 (1.499%) | 0.8825 | not run | not run | not run |
| bound, 1% | 2.312 | 329 of 36,110 (0.998%) | 0.8474 | 93 of 10,747 (0.87%) | 87.30% | 1 within, 4 not resolved |
| bound, 0.1% | 5.706 | 26 of 36,110 (0.100%) | 0.6000 | 12 of 10,747 (0.11%, upper bound 0.18%) | 66.22% | 0 within, 5 not resolved |

The bound rule pools authentic items, and the balanced rule averages
sources, so at the same nominal target they are different budgets: here
the pooled bound is the looser of the two (1.576 against 1.692). At 0.1%
no confirmation 7 source can be resolved: the largest has 6,168 images
with 4 flagged (upper bound 0.15%), and three of the five have fewer
than 2,995.

**Review bands on confirmation 7** (the shipped decision; margins in
calibrated logits; 1% prevalence).

```bash
tintype report --operating-points --system "tintype=runs/ws2/c7-decided.tsv@0.5" --prevalence 0.01 --review-margin 0.5,0.91,1,2,3
```

| Review band | Coverage | Real items reviewed | AI items reviewed | Real items flagged without review (upper bound) | Recall, reviewed AI items as misses | Share of a stream reviewed at 1% prevalence |
|---|---|---|---|---|---|---|
| none | 100.00% | 0 | 0 | 130 of 10,747, 1.21% (1.40%) | 89.70% | 0.00% |
| margin 0.5 | 98.08% | 82 (0.76%) | 360 (2.93%) | 99, 0.92% (1.09%) | 88.11% | 0.78% |
| margin 0.91 | 96.17% | 161 (1.50%) | 723 (5.88%) | 80, 0.74% (0.90%) | 86.36% | 1.54% |
| margin 1 | 95.73% | 184 (1.71%) | 800 (6.50%) | 73, 0.68% (0.82%) | 85.92% | 1.76% |
| margin 2 | 88.83% | 549 (5.11%) | 2,025 (16.46%) | 32, 0.30% (0.40%) | 78.41% | 5.22% |
| margin 3 | 77.42% | 1,368 (12.73%) | 3,838 (31.19%) | 14, 0.13% (0.20%) | 66.14% | 12.91% |

A margin band's automatic verdicts are those of a stricter cutoff on the
flagged side and a looser one on the cleared side: it does not change the
ranking. What it buys is that the items in between go to a person instead
of being decided. At margin 1, 800 AI items (6.50%) and 184 real ones go
to review, and recall counted without them falls from 89.70% to 85.92%.

**View disagreement did not beat the margin.** At the same coverage
(96.17%, 884 items reviewed), the margin alone leaves fewer real items
flagged and keeps more recall than a narrower margin plus the disagreement
rule:

```bash
tintype report --operating-points --system "tintype=runs/ws2/c7-decided.tsv@0.5" --prevalence 0.01 \
  --review-margin 0.5 --review-disagreement 0.5 --review-threshold-logit 1.692
```

| Band | Coverage | Real items flagged without review | Recall, reviewed AI items as misses |
|---|---|---|---|
| margin 0.91 | 96.17% | 80 of 10,747 (0.74%) | 86.36% |
| margin 0.5, or half the views disagreeing | 96.17% | 92 of 10,747 (0.86%) | 85.59% |
| half the views disagreeing, no margin | 97.07% | 106 of 10,747 (0.99%) | 86.23% |

One reason is early exit itself: 14,135 of the 23,052 images are decided
on one view and have nothing to disagree with. The rule stays available
behind its flag and off by default; gate 3 decides whether it is used.

**Early exit against scoring every view, on confirmation 7.** The shipped
margins were fitted to keep pooled precision and recall within 0.001 on
the development images. On confirmation 7 the default profile's recall is
0.48 points lower under early exit than with every view, which is outside
that tolerance; precision is within it.

```bash
tintype refit-exit --weights runs/ws2/incumbent.safetensors --profile photo,strict,sensitive \
  --fit $A/scores-v7/calib_images.tsv --report $A/scores-c7/*.veracity-views.tsv
```

| Profile (raw threshold) | Schedule (margins after 1 to 5 views) | Views per image | Verdicts that differ | Authentic flagged: every view, early exit | Recall: every view, early exit |
|---|---|---|---|---|---|
| default (1.692) | shipped: 6, 3, 2, 1.5, 1 | 1.69 | 170 | 127, 130 of 10,747 | 90.18%, 89.70% |
| default (1.692) | refitted: 5.75, 5.75, 1.75, 1.5, 1.25 | 1.83 | 134 | 127, 132 | 90.18%, 89.80% |
| strict (3.396) | shipped | 1.77 | 236 | 48, 48 | 82.51%, 82.36% |
| strict (3.396) | refitted | 1.91 | 204 | 48, 47 | 82.51%, 82.45% |
| sensitive (−0.776) | shipped | 1.79 | 71 | 456, 459 | 96.12%, 95.99% |
| sensitive (−0.776) | refitted | 1.97 | 48 | 456, 455 | 96.12%, 96.08% |

The refit used the archived calibration scores for all three profiles at
once (within 0.001 on those 82,948 items: default precision 0.9885
against 0.9886 and recall 0.8769 against 0.8777). It narrows the gap on
confirmation 7 and looks at more views, and the default profile's recall
is still 0.38 points under every view there. The mean false-positive rate
over confirmation 7's real sources with at least 100 images is 2.09% with
every view and 2.12% under the shipped early exit, against the 1.5%
target; the rate that is over the target is the per-source mean, with or
without early exit. The stored photo second-opinion band (−7, 0.5), which
is (−8.850, 2.445) in raw logits, routes 11,068 of the 23,052 images from
the early-exit views and 11,472 from every view. None of this changes the
shipped schedule.

## What is not built

- No final cutoff, band or margin: see Status.
- No command writes a review band into a spec; edit the copy under `runs/`
  or pass the flags.
- No asymmetric review band (different margins under and over the
  threshold). The accounting would support one through `--review-flags`.
- No refit of the second-opinion rule itself: `refit-exit` re-expresses
  the photo band and reports routing; a new band needs second-stage scores.
- `demo` does not take the review flags.

## Log

| Date | Commit | What was fitted or looked at, on which set | By |
|---|---|---|---|
| 2026-11-14 | `caa15c3` | Released photo model, arithmetic on the incumbent's all-views files. On `calibrate`: cutoffs of three arms (bound with the highest version per parent, bound with originals only, the default rule) for the photo, strict, sensitive and fast profiles; review margins 0.75 (1% cutoff) and 1.5 (0.1% cutoff); the disagreement rule (rejected). On `fit`: exit margins 6.5, 4, 4, 1.75, 1. On `test`, once per arm after the pre-registration commit: transfer by source and transform, clean and degraded slices, review accounting and early exit for arm A. Gate: failed (cutoff transfer, both operating points). CPU only, Apple M6 | workstream 2 |
