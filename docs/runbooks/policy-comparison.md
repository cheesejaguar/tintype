# Runbook: photo policy comparison

Workstream 3 of the research report ("Test a complementary photo model and
a cheaper existing route"). It compares photo decision policies at equal
false-positive counts: a retuned single-model profile, the shipped and a
narrowed Community Forensics (CF) band, a second model behind a margin
gate, and logit fusion.

Status on 2026-11-14: the tooling exists and reproduces the report's
archive numbers. Everything measured so far is on archived suites and is
exploratory. Nothing here changes what ships, and no candidate has been
selected. The frozen-split commands in the second half have not been run.

## The gate, written before the commands

These are the report's *proposed* decision criteria, not results.

1. **Recall.** A candidate policy must gain **at least 5 recall points at
   1% empirical FPR on degraded media** over the Tintype default profile,
   on the test set of the frozen independent split, with the fusion
   weights fitted on its `fit` set and every cutoff frozen on its
   `calibrate` set. Degraded media are the test photos whose `transform`
   column (manifest column 9) is not the original.
2. **The stricter point is always reported too:** recall at 0.1% FPR, and
   the actual FPR and recall on the test set at each frozen cutoff. A
   candidate that gains at 1% and loses at 0.1% is reported as both.
3. **Bounded cost.** p95 single-image latency, the share of photos sent to
   the second model, and peak memory each stay under a bound that is
   written into the `docs/PREREGISTRATION.md` entry *before* the test set
   is scored. The report does not name the bounds. Measure the default
   profile and the shipped CF route first (step 6 below) and write the
   bounds relative to them: p95 ≤ ___ ms on the named hardware, route rate
   ≤ ___ %, peak resident memory ≤ ___ MiB.
4. **The narrowed CF band is only ever an opt-in speed candidate.** It is
   considered only if its per-set comparison against Community Forensics
   meets the intended use of `--photo-second-opinion` (sets on which both
   precision and recall reach CF at its shipped cutoff). It ships only
   after selection on development data, the frozen-split result, and a new
   pre-registration entry. The shipped band is not changed by anything in
   this runbook.
5. Fit simple logit fusion before any learned gate. Disagreement and
   processing features are analysed (how many first-stage misses does a
   margin-only gate leave outside?) and are not shipped as a gate.

Archived suites (confirmation 7, Arena, Frontier, NTIRE) were used to
develop the detector. A cutoff chosen on a cohort and reported on it is
exploratory, not a transfer result. Routed shares are measured workload;
end-to-end savings depend on the cohort and the hardware.

## Tools

| Entry point | What it does |
|---|---|
| `python3 scripts/compare-policies.py` | The comparison: route rate, recall at 0.1% and 1% empirical FPR and at false-positive counts, counts at each policy's own cutoff, per-set parity against a reference, frozen-cutoff transfer from a calibration set, and the first-stage errors each gate leaves outside. Standard library only. |
| `python3 scripts/compare-policies.py --self-test` | Checks its metric functions against hand-computed cases. |
| `python3 scripts/compare-policies.py --check-replay` | Shows, line for line, that it reproduces `scripts/replay-research.py`. |
| `python3 scripts/fit-linear-probe.py fit\|score` | Fits a linear probe on pre-extracted features of a frozen encoder and writes score files the harness reads. Standard library only; `--self-test`. |
| `tintype latency --photo-second-opinion <CF weights>` | Single-image latency of the two-stage route, with the share of images sent on. |
| `tintype report --operating-points` | Intervals, slices and clustered bootstraps for the score files the harness writes with `--write-scores`. |

Inputs the harness takes:

- `--archive confirm7|frontier|arena`: the score archives in `docs/results`.
- `--table FILE`: any wide table (`path label generator dataset` and one
  probability column per system).
- `--scores NAME=FILE[@MODE]` (repeatable, globs allowed): score files as
  `tintype score` writes them, `path label generator dataset prob
  logits...`, joined on the path. `@prob` (default) takes the logit of the
  probability column; `@logits` takes the mean of the logit columns (what
  the probe scorer writes); `@spec=SPEC.json[#profile]` computes the
  calibrated logit `scale * mean view logit + bias` from a model spec, which
  is exact where the six-decimal probability saturates and is the number
  the photo second opinion compares with its band.
- `--groups MANIFEST`: grouping columns (5 and up: `parent_id scene_id
  source_id generator_family transform`) for cross-fitting and slicing.
- `--set fit=MANIFEST --set calibrate=MANIFEST --set test=MANIFEST`: the
  frozen split. Fusion weights come from `fit`, cutoffs from `calibrate`,
  and only `test` is reported. Paths must be the ones the files were scored
  with; `tintype split-groups` writes absolute paths, so score those
  manifests directly.

Fusion weights are never fitted on the photos they are reported on. Without
`--set` or `--cross-fit K` the fitted policies are left out.

## Checks (run 2026-11-14, commit 9e81024, Mac18,5 Apple M6, Python 3.9.6)

```console
$ python3 scripts/compare-policies.py --self-test
self-test passed
$ python3 scripts/fit-linear-probe.py --self-test
self-test passed
$ python3 scripts/compare-policies.py --check-replay
ok       Confirmation 7 CF fusion reproduces shipped verdicts with 0 mismatches.
ok         Tintype default: routed 0/23052 (0.00%)
ok           10887/12305 TPs at 107/107 allowed FPs; matched or beat CF precision and recall on 12/30 sets
ok           11468/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 14/30 sets
ok           at score > 0.5: 11038 TPs, 130 FPs
ok         Tintype sensitive: routed 0/23052 (0.00%)
ok           10608/12305 TPs at 107/107 allowed FPs; matched or beat CF precision and recall on 13/30 sets
ok           11670/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 14/30 sets
ok           at score > 0.5: 11812 TPs, 459 FPs
ok         Shipped CF fusion: routed 11068/23052 (48.01%)
ok           10486/12305 TPs at 107/107 allowed FPs; matched or beat CF precision and recall on 13/30 sets
ok           11504/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 26/30 sets
ok           at score > 0.5: 11504 TPs, 329 FPs
ok         Narrow CF fusion: routed 2417/23052 (10.48%)
ok           10872/12305 TPs at 107/107 allowed FPs; matched or beat CF precision and recall on 14/30 sets
ok           11584/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 20/30 sets
ok           at score > 0.5: 11413 TPs, 213 FPs
ok         Frontier original: default (181, 261), narrow (177, 261) (TP, generated at 1% empirical FPR); 23.4% routed
ok         Frontier jpeg: default (139, 261), narrow (116, 261) (TP, generated at 1% empirical FPR); 27.0% routed
ok         Frontier jpeg-960: default (73, 261), narrow (67, 261) (TP, generated at 1% empirical FPR); 38.1% routed
ok       arena-original: 2031 unique originals; gated 264 (13.0%)
ok         Tintype: 901/1018 (88.51%) at 1% empirical FPR; 10/1013 FP at score > 0.5
ok         GPTTrace: 835/1018 (82.02%) at 1% empirical FPR; 13/1013 FP at score > 0.5
ok         50:50 full fusion: 935/1018 (91.85%) at 1% empirical FPR; 4/1013 FP at score > 0.5
ok         50:50 fusion for |Tintype logit| <= 2: 930/1018 (91.36%) at 1% empirical FPR; 5/1013 FP at score > 0.5
ok         label 0: 10 Tintype errors; 1 outside band; 7 GPTTrace correct
ok         label 1: 120 Tintype errors; 68 outside band; 62 GPTTrace correct
ok       arena-jpeg-q4: 2031 unique originals; gated 382 (18.8%)
ok         Tintype: 744/1018 (73.08%) at 1% empirical FPR; 7/1013 FP at score > 0.5
ok         GPTTrace: 781/1018 (76.72%) at 1% empirical FPR; 13/1013 FP at score > 0.5
ok         50:50 full fusion: 834/1018 (81.93%) at 1% empirical FPR; 5/1013 FP at score > 0.5
ok         50:50 fusion for |Tintype logit| <= 2: 825/1018 (81.04%) at 1% empirical FPR; 5/1013 FP at score > 0.5
ok         label 0: 7 Tintype errors; 1 outside band; 4 GPTTrace correct
ok         label 1: 310 Tintype errors; 140 outside band; 162 GPTTrace correct
34 replay lines, 0 mismatches; 20/20 pinned report baselines present.
```

Each `ok` line is printed by `scripts/replay-research.py` (captured from its
own functions) and rebuilt independently from the harness's policies and
counting. Twenty of them are also pinned in the harness, so a change to an
archive or to the shipped band shows up even if both scripts move together.
The harness also reproduces confirmation 7 from the 150 per-system score
files of that batch (`--scores`, local files not in the repository): the
tables were identical to those from the archive.

## Archive runs (exploratory)

All run on 2026-11-14 at commit 9e81024 on a Mac18,5 (Apple M6, 32 GiB),
Python 3.9.6, CPU only. Full, unedited outputs are in
[`docs/results/policy-comparison/`](../results/policy-comparison/). Each
cutoff below is chosen on the cohort it is reported on.

### Confirmation 7: single profiles and the CF band

```sh
python3 scripts/compare-policies.py --archive confirm7 > docs/results/policy-comparison/confirm7.md
```

23,052 photos, 12,305 generated, 10,747 authentic. "+/-" is recall points
against Tintype default.

| Policy | Routed | Recall at 0.1% FPR (≤ 10 FP) | Recall at 1% FPR (≤ 107 FP) | Recall at 329 FP (3.06% FPR) | At own cutoff: TP, FP | Sets ≥ CF at 329 FP |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 0% | 7174 (58.30%) | 10887 (88.48%) | 11468 (93.20%) | 11038, 130 | 14/30 |
| Tintype sensitive | 0% | 6572 (53.41%) −4.89 | 10608 (86.21%) −2.27 | 11670 (94.84%) | 11812, 459 | 14/30 |
| Tintype strict | 0% | 7759 (63.06%) +4.75 | 10647 (86.53%) −1.95 | 11238 (91.33%) | 10135, 48 | 12/30 |
| Shipped CF blend, band (−7, 0.5) | 11068 (48.01%) | 6754 (54.89%) −3.41 | 10486 (85.22%) −3.26 | 11504 (93.49%) | 11504, 329 | 26/30 |
| Narrowed CF blend, band (−3, 0.5) | 2417 (10.48%) | 7382 (59.99%) +1.69 | 10872 (88.35%) −0.12 | 11584 (94.14%) | 11413, 213 | 20/30 |

- The strongest single profile depends on the operating point: strict at
  0.1% FPR, default at 1%, sensitive at 329 FP.
- The band recomputed from the `tintype` and `commfor` columns gives the
  same counts as the stored `tintype-second` column at every point.
- Narrowing cuts routed photos from 11,068 to 2,417 and keeps recall at 1%
  FPR within 0.12 points of Tintype alone, but it reaches CF on 20 of 30
  sets where the shipped band reaches it on 26 (21 against 26 at each
  policy's own cutoff). That per-set loss is why it is only a speed
  candidate.
- The narrowed band leaves 329 of Tintype's 1,267 misses outside the gate
  (CF flags 120 of them); the shipped band leaves 1.

### Frontier paired JPEG stress

```sh
python3 scripts/compare-policies.py --archive frontier > docs/results/policy-comparison/frontier.md
```

745 photos per cohort, 261 generated, 484 authentic (0.1% FPR allows no
false positive, 1% allows 4).

| Cohort | Policy | Routed | Recall at 0.1% FPR | Recall at 1% FPR |
|---|---|---:|---:|---:|
| original | Tintype default | 0% | 137/261 (52.49%) | 181/261 (69.35%) |
| original | Shipped CF blend | 540 (72.48%) | 125/261 (47.89%) | 177/261 (67.82%) |
| original | Narrowed CF blend | 174 (23.36%) | 125/261 (47.89%) | 177/261 (67.82%) |
| jpeg | Tintype default | 0% | 99/261 (37.93%) | 139/261 (53.26%) |
| jpeg | Shipped CF blend | 633 (84.97%) | 47/261 (18.01%) | 116/261 (44.44%) |
| jpeg | Narrowed CF blend | 201 (26.98%) | 47/261 (18.01%) | 116/261 (44.44%) |
| jpeg-960 | Tintype default | 0% | 9/261 (3.45%) | 73/261 (27.97%) |
| jpeg-960 | Shipped CF blend | 648 (86.98%) | 10/261 (3.83%) | 67/261 (25.67%) |
| jpeg-960 | Narrowed CF blend | 284 (38.12%) | 10/261 (3.83%) | 67/261 (25.67%) |

On these degraded photos neither band helps at a fixed false-positive
count: both lose 8.81 points at 1% FPR on same-size JPEGs. CF flags 7 of
Tintype's 145 misses on same-size JPEGs and 10 of 160 on capped ones.

### Arena with GPTTrace: 50:50 and fitted fusion

```sh
python3 scripts/compare-policies.py --archive arena --cross-fit 5 > docs/results/policy-comparison/arena.md
python3 scripts/compare-policies.py --archive arena --cross-fit 5 --fit-cohort arena-original > docs/results/policy-comparison/arena-fit-on-originals.md
python3 scripts/compare-policies.py --archive arena --cross-fit 5 --fit-cohort arena-jpeg-q4 > docs/results/policy-comparison/arena-fit-on-jpeg.md
```

2,031 unique originals per variant (seven repeated authentic originals
removed from each), 1,018 generated, 1,013 authentic. The fitted rows are
cross-fitted: five folds of whole parents (an original and its JPEG stay
together), each fold scored with weights fitted on the other four, on both
variants.

| Variant | Policy | Routed | Recall at 0.1% FPR (≤ 1 FP) | Recall at 1% FPR (≤ 10 FP) |
|---|---|---:|---:|---:|
| original | Tintype default | 0% | 750 (73.67%) | 901 (88.51%) |
| original | GPTTrace alone | 0% | 563 (55.30%) −18.37 | 835 (82.02%) −6.48 |
| original | 50:50 for \|Tintype logit\| ≤ 2 | 264 (13.00%) | 781 (76.72%) +3.05 | 930 (91.36%) +2.85 |
| original | 50:50 full fusion | 100% | 828 (81.34%) +7.66 | 935 (91.85%) +3.34 |
| original | Fitted for \|Tintype logit\| ≤ 2 | 264 (13.00%) | 675 (66.31%) −7.37 | 930 (91.36%) +2.85 |
| original | Fitted full fusion | 100% | 828 (81.34%) +7.66 | 946 (92.93%) +4.42 |
| original | Shipped CF blend | 1110 (54.65%) | 753 (73.97%) +0.29 | 890 (87.43%) −1.08 |
| original | Narrowed CF blend | 286 (14.08%) | 753 (73.97%) +0.29 | 891 (87.52%) −0.98 |
| JPEG q4 | Tintype default | 0% | 553 (54.32%) | 744 (73.08%) |
| JPEG q4 | GPTTrace alone | 0% | 586 (57.56%) +3.24 | 781 (76.72%) +3.63 |
| JPEG q4 | 50:50 for \|Tintype logit\| ≤ 2 | 382 (18.81%) | 585 (57.47%) +3.14 | 825 (81.04%) +7.96 |
| JPEG q4 | 50:50 full fusion | 100% | 563 (55.30%) +0.98 | 834 (81.93%) +8.84 |
| JPEG q4 | Fitted for \|Tintype logit\| ≤ 2 | 382 (18.81%) | 340 (33.40%) −20.92 | 804 (78.98%) +5.89 |
| JPEG q4 | Fitted full fusion | 100% | 541 (53.14%) −1.18 | 832 (81.73%) +8.64 |
| JPEG q4 | Shipped CF blend | 1277 (62.88%) | 395 (38.80%) −15.52 | 694 (68.17%) −4.91 |
| JPEG q4 | Narrowed CF blend | 431 (21.22%) | 395 (38.80%) −15.52 | 695 (68.27%) −4.81 |

Fitted on one variant only (other folds), reported on both:

| Fitted on | Reported on | Policy | Recall at 0.1% FPR | Recall at 1% FPR |
|---|---|---|---:|---:|
| originals | originals | Fitted for \|logit\| ≤ 2 | 694 (68.17%) −5.50 | 922 (90.57%) +2.06 |
| originals | originals | Fitted full fusion | 836 (82.12%) +8.45 | 944 (92.73%) +4.22 |
| originals | JPEG q4 | Fitted for \|logit\| ≤ 2 | 281 (27.60%) −26.72 | 767 (75.34%) +2.26 |
| originals | JPEG q4 | Fitted full fusion | 544 (53.44%) −0.88 | 819 (80.45%) +7.37 |
| JPEG q4 | originals | Fitted for \|logit\| ≤ 2 | 658 (64.64%) −9.04 | 931 (91.45%) +2.95 |
| JPEG q4 | originals | Fitted full fusion | 825 (81.04%) +7.37 | 946 (92.93%) +4.42 |
| JPEG q4 | JPEG q4 | Fitted for \|logit\| ≤ 2 | 373 (36.64%) −17.68 | 813 (79.86%) +6.78 |
| JPEG q4 | JPEG q4 | Fitted full fusion | 528 (51.87%) −2.46 | 827 (81.24%) +8.15 |

What this does and does not show:

- On the paired JPEGs the 50:50 blend with GPTTrace gains 7.96 points at 1%
  FPR while routing 18.81%, and 8.84 with full fusion. Those are above the
  proposed 5-point target, on an archive, with cutoffs chosen on the
  cohort: a reason to run the frozen-split test, not a result.
- Fitted weights did not beat the 50:50 blend on the JPEGs at 1% FPR
  (78.98% gated, 81.73% full, against 81.04% and 81.93%), and the fitted
  *gated* policy is much worse at 0.1% FPR (33.40% against 57.47%). The
  fitted gate puts in-gate and out-of-gate photos on one log-odds scale
  from about 500 in-gate fitting photos per fold; at one allowed false
  positive that is fragile. Report 0.1% beside 1% every time.
- At 0.1% FPR on the JPEGs no policy gains more than 3.24 points.
- The CF bands lose recall on Arena at both rates.

### Confirmation 7: fitted fusion with CF, cross-fitted by set

```sh
python3 scripts/compare-policies.py --archive confirm7 --second commfor --cross-fit 5 --group-by dataset > docs/results/policy-comparison/confirm7-fitted-cf.md
```

Five folds of whole sets (30 sets); each fold is scored with weights fitted
on the other folds.

| Policy | Routed | Recall at 0.1% FPR | Recall at 1% FPR | Sets ≥ CF at 329 FP |
|---|---:|---:|---:|---:|
| Tintype default | 0% | 7174 (58.30%) | 10887 (88.48%) | 14/30 |
| Community Forensics alone | 0% | 5535 (44.98%) −13.32 | 7766 (63.11%) −25.36 | 9/30 |
| 50:50 for \|Tintype logit\| ≤ 2 | 2574 (11.17%) | 7413 (60.24%) +1.94 | 11014 (89.51%) +1.03 | 14/30 |
| 50:50 full fusion | 100% | 8110 (65.91%) +7.61 | 10272 (83.48%) −5.00 | 28/30 |
| Fitted for \|Tintype logit\| ≤ 2 | 2574 (11.17%) | 7471 (60.72%) +2.41 | 11029 (89.63%) +1.15 | 15/30 |
| Fitted full fusion | 100% | 8219 (66.79%) +8.49 | 11003 (89.42%) +0.94 | 20/30 |

### Gates and the misses they leave outside

Every run above prints a gate table. From `arena.md`, Tintype at its
shipped cutoff with GPTTrace behind a margin gate:

| Variant | Gate | Routed | Misses inside | Misses outside | Outside, flagged by GPTTrace |
|---|---|---:|---:|---:|---:|
| original | \|logit\| ≤ 2 | 264 (13.00%) | 52 | 68 | 29 |
| original | \|logit\| ≤ 4 | 794 (39.09%) | 97 | 23 | 8 |
| JPEG q4 | \|logit\| ≤ 1 | 164 (8.07%) | 86 | 224 | 106 |
| JPEG q4 | \|logit\| ≤ 2 | 382 (18.81%) | 170 | 140 | 51 |
| JPEG q4 | \|logit\| ≤ 3 | 704 (34.66%) | 231 | 79 | 21 |
| JPEG q4 | \|logit\| ≤ 4 | 951 (46.82%) | 253 | 57 | 14 |

On the JPEGs, 140 of Tintype's 310 misses lie outside `|logit| ≤ 2`, and
GPTTrace would flag 51 of those. A symmetric margin gate that takes in 90%
of the misses needs `|logit| ≤ 4.48` and routes 57.16% of the photos. A
margin alone therefore cannot reach the strong negatives cheaply.

View disagreement as a routing feature, on confirmation 7 scored with all
six views (`*.veracity-views.tsv`, local score files of that batch which
are not in the repository; the calibrated six-view logit comes from the
shipped spec):

```sh
S=/Users/aaron/Code/veracity-scratch/runs/scores-c7
python3 scripts/compare-policies.py --scores "tintype-views=$S/*.veracity-views.tsv@spec=models/tintype.json" \
  --scores "commfor=$S/*.commfor.tsv" --first tintype-views --second commfor --margin 2,3 \
  --view-spread tintype-views --gate-sweep "" --slice-by dataset > docs/results/policy-comparison/confirm7-view-spread.md
```

| Gate | Confident negatives outside: generated, authentic | AUROC of view spread for generated among them | Misses inside the margin gate | Misses among as many photos with the largest view spread |
|---|---:|---:|---:|---:|
| −3 < logit < 0.5 | 136, 9327 | 0.6282 | 1072 | 385 |
| \|logit\| ≤ 2 | 357, 10109 | 0.6354 | 851 | 450 |
| \|logit\| ≤ 3 | 136, 9327 | 0.6282 | 1072 | 625 |

Among confident negatives, the spread between the largest and smallest
view logit ranks the generated photos only weakly above the authentic ones
(AUROC 0.63), and at an equal routing budget a spread-only gate takes in
fewer misses than the margin gate (450 against 851 at `|logit| ≤ 2`). On
this cohort view spread is not a replacement for the margin; whether it
adds to it has to be measured on the frozen split. By set, 141 of the 357
misses outside `|logit| ≤ 2` are the face-swap set and 113 are So-Fake-OOD.

### Dry run of the frozen-split commands on Arena (mechanics only)

This exercises `--write-manifest`, `tintype split-groups`, `verify-split`
and `--set` end to end. Arena is not an independent split, and 406
authentic calibration photos cannot support a 0.1% target (about 3,000 are
needed), so the numbers only show that the plumbing works.

```sh
D=$(mktemp -d)
python3 scripts/compare-policies.py --archive arena --write-manifest $D/arena.tsv
tintype split-groups --manifest $D/arena.tsv --out-dir $D/split --fractions 0.4,0.2,0.4 --seed 1 --together parent
tintype verify-split --set fit=$D/split/fit.tsv --set calibrate=$D/split/calibrate.tsv --set test=$D/split/test.tsv --together parent
gzip -dc docs/results/arena-v0.1-gpttrace/scores.tsv.gz | awk -v r="$D/" 'NR==1{print;next}{print r $0}' > $D/table.tsv
python3 scripts/compare-policies.py --table $D/table.tsv --set fit=$D/split/fit.tsv --set calibrate=$D/split/calibrate.tsv \
  --set test=$D/split/test.tsv --cohort-by transform --gate-sweep "" > docs/results/policy-comparison/arena-split-dry-run.md
```

`split-groups` reported 1,624 fit, 814 calibrate and 1,624 test items and
`verify-split` reported "no group crosses sets (parent)". With the cutoff
frozen on 406 authentic calibration photos, the 1% target gave Tintype
default 19/810 false positives (2.35%) on the test photos and the 0.1%
target gave 9/810 (1.11%): a cutoff from a small calibration set does not
hold its rate, which is the reason for the frozen split's size.

### The latency tool on the two-stage route (smoke test, machine under load)

300 photos (150 from `frontier-heldout`, 150 from `openfake-test-reals`,
interleaved), CPU engine, 200 timed after 5 warm-up images, on the Mac18,5
while other jobs kept the load average between 22 and 36. These numbers
show that the tool works and what it prints. They are not a benchmark.

```sh
tintype latency --weights models/tintype.safetensors --manifest sample.tsv --n 200 --device cpu
tintype latency --weights models/tintype.safetensors --manifest sample.tsv --n 200 --device cpu --photo-second-opinion baselines/commfor-384.safetensors
tintype latency --weights runs/ws3-narrow/tintype.safetensors --manifest sample.tsv --n 200 --device cpu --photo-second-opinion baselines/commfor-384.safetensors
```

| Run | Median | p95 | Routed | Routed images: median, p95 | Peak resident memory |
|---|---:|---:|---:|---:|---:|
| Default profile | 34.0 ms | 74.1 ms | – | – | 351 MiB |
| Shipped band (−7, 0.5) | 57.1 ms | 105.3 ms | 93/200 (46.5%) | 75.5 ms, 119.3 ms | 518 MiB |
| Narrowed band (−3, 0.5), copied spec | 24.2 ms | 88.6 ms | 15/200 (7.5%) | 110.5 ms, 161.0 ms | 515 MiB |

`tintype detect --photo-second-opinion` on the same 200 files sent the same
93 and 15 photos to the second stage. Memory is `/usr/bin/time -l`
"maximum resident set size".

## Frozen-split commands (ready to run; none of these has been run)

Prerequisites:

- The frozen independent split of `docs/runbooks/frozen-eval.md`: `fit.tsv`,
  `calibrate.tsv` and `test.tsv` with grouping columns, verified with
  `tintype verify-split` and frozen with `tintype freeze-set`. Below,
  `SPLIT` is its folder.
- A new `docs/PREREGISTRATION.md` entry, written before the test set is
  scored, naming the candidates, the primary operating points (0.1% and 1%
  FPR), the degraded slice, and the cost bounds of the gate.
- A build of this branch. In earlier builds `tintype detect` passed
  `--photo-second-opinion` to the video stage (fixed here); `score` was
  never affected.

```sh
SPLIT=data/splits/frozen            # as written by the frozen-eval runbook
CF=baselines/commfor-384.safetensors
OUT=runs/ws3/scores
mkdir -p $OUT runs/ws3-shipped runs/ws3-narrow
```

### 1. Work on copies of the shipped spec

The shipped files are never written to. Both variants are copies.

```sh
cp models/tintype.safetensors models/tintype.json runs/ws3-shipped/
cp models/tintype.safetensors models/tintype.json runs/ws3-narrow/
tintype set-photo-second-opinion --weights runs/ws3-narrow/tintype.safetensors --band -3,0.5,0.5,0
tintype verify-models               # the shipped files must still verify
```

`--band` takes `lo,hi,weight,threshold` in calibrated first-stage logits
(`--band=-3,0.5,0.5,0` also works). On 2026-11-14 this step was run here:
the copied spec differed from the shipped one only in `"lo": -3.0`, and
`verify-models` printed `ok` for all four shipped files.

### 2. Score the three sets (GPU; more than 2,000 items each)

```sh
for S in fit calibrate test; do
  tintype score --weights runs/ws3-shipped/tintype.safetensors --manifest $SPLIT/$S.tsv --out $OUT/$S.tintype.tsv
  tintype score --weights runs/ws3-shipped/tintype.safetensors --manifest $SPLIT/$S.tsv --out $OUT/$S.tintype-sensitive.tsv --sensitive
  tintype score --weights runs/ws3-shipped/tintype.safetensors --manifest $SPLIT/$S.tsv --out $OUT/$S.tintype-strict.tsv --strict
  tintype score --weights runs/ws3-shipped/tintype.safetensors --manifest $SPLIT/$S.tsv --out $OUT/$S.tintype-views.tsv --all-views
  tintype score --weights $CF --reference commfor --manifest $SPLIT/$S.tsv --out $OUT/$S.commfor.tsv
done
# The two-stage route run for real, to check the harness's arithmetic against it:
tintype score --weights runs/ws3-shipped/tintype.safetensors --manifest $SPLIT/test.tsv --out $OUT/test.tintype-second.tsv --photo-second-opinion $CF
tintype score --weights runs/ws3-narrow/tintype.safetensors  --manifest $SPLIT/test.tsv --out $OUT/test.tintype-narrow.tsv --photo-second-opinion $CF
```

Expected: one score file per system and set, and on stderr the number of
unreadable files (which must be the same for every system) and, for the
last two, how many photos were given a second opinion.

### 3. The second model: licence first, then scores

**Licence check, before any download.** Read the licence text itself (not
a summary), confirm it allows the intended use (research comparison now;
redistribution of a probe or of scores later is a separate question), and
record the URL, the revision and the date in `docs/DATA.md` and
`NOTICE.md`. Where each licence is stated, as read through page summaries
on 2026-11-13 (to be re-read at the source):

| Checkpoint | Where the licence is stated | What it said |
|---|---|---|
| GPTTrace `nakas/gpttrace-img-flag-pec-v3-onnx`, revision `197d072c7ff2e41d53f5e23b57e16c9e0d9ba042` | Model card metadata and README on Hugging Face: <https://huggingface.co/nakas/gpttrace-img-flag-pec-v3-onnx> | `cc-by-sa-4.0` ("some training data is CC BY-SA"); the card says its PE-Core backbone is Apache-2.0 |
| Perception Encoder (PE-Core) | `LICENSE.PE` and the model-licence badge in <https://github.com/facebookresearch/perception_models>; model card <https://huggingface.co/facebook/PE-Core-L14-336> | Apache-2.0 (the PLM models in the same repository are under a different, research licence) |
| DINOv3 | `LICENSE.md` in <https://github.com/facebookresearch/dinov3> | "DINOv3 License": a custom Meta licence; weights are gated behind an access form; redistribution only under the same agreement, with trade-control and use restrictions. Whether a probe on its features is a derivative work is not stated: needs a decision before any release. |
| MetaCLIP 2 | `LICENSE` in <https://github.com/facebookresearch/MetaCLIP>; model card <https://huggingface.co/facebook/metaclip-2-worldwide-huge-quickgelu> | "The majority of Meta CLIP is licensed under CC-BY-NC"; the model card says `cc-by-nc-4.0` (non-commercial) |

Stop here until the licence decision is recorded. Nothing was downloaded
in this workstream.

**GPTTrace as the comparison checkpoint.** `scripts/score-gpttrace-arena.py`
holds the pinned model and config hashes and the model card's
preprocessing (`pixels()`: centre crop and resized full frame at the
config's resolution, mean of the two view logits, minus the card's offset
3.562), but it only scores the Arena archive. Scoring the frozen split
needs a manifest-driven wrapper around the same function that writes
`path label generator dataset prob logit` with the offset-adjusted logit.
It was not written here: ONNX Runtime and the checkpoint are not on this
machine, so it could not have been run. It needs `onnxruntime`, `numpy`
and `Pillow` in a virtual environment (not the system Python), and the
provider-parity check the Arena run recorded.

**A linear probe on a frozen encoder (PE, DINOv3 or MetaCLIP 2).** The
encoder runs offline in its own Python environment. The extractor is not
part of this repository; it must:

1. verify the checkpoint's SHA-256 against a pinned value and record the
   encoder name, revision, input resolution, resize and crop rule, which
   token or pooling is taken, and the package versions;
2. read each of `fit.tsv`, `calibrate.tsv` and `test.tsv` in order and write
   one `.npy` file per set: a C-ordered float32 array of shape
   `(photos, features)`, row `i` for manifest row `i`; unreadable files
   must fail the run (drop them from the manifests first, for every
   system);
3. use no label and no statistic of the calibration or test set.

Then, with the standard library only:

```sh
python3 scripts/fit-linear-probe.py fit --features runs/ws3/features/fit.pe.npy --manifest $SPLIT/fit.tsv \
  --l2-grid 0.1,1,10,100 --folds 5 --encoder "PE-Core-L14-336 <revision>, 336 px, <pooling>" --out runs/ws3/probe-pe.json
for S in fit calibrate test; do
  python3 scripts/fit-linear-probe.py score --probe runs/ws3/probe-pe.json --features runs/ws3/features/$S.pe.npy \
    --manifest $SPLIT/$S.tsv --out $OUT/$S.probe-pe.tsv
done
```

The probe is L2-regularised logistic regression on standardised features,
fitted by L-BFGS on the `fit` set only; `--l2-grid` picks the penalty by
cross-validated log loss inside the fit set with each `parent_id` in one
fold. The probe file records the SHA-256 of the features and the manifest.
The fitter is pure Python: one pass over 2,000 rows of 1,024 features took
0.34 s here, about 8 s per 50,000 rows, and a fit takes some tens of
passes (times the folds and the grid).

Score-file format the harness consumes, one line per photo, no header,
tab-separated: `path label generator dataset prob logit`, with the path
exactly as in the manifest. Read it with `@logits`.

### 4. Compare

```sh
python3 scripts/compare-policies.py \
  --scores "tintype=$OUT/*.tintype.tsv@spec=runs/ws3-shipped/tintype.json" \
  --scores "tintype-sensitive=$OUT/*.tintype-sensitive.tsv@spec=runs/ws3-shipped/tintype.json#sensitive" \
  --scores "tintype-strict=$OUT/*.tintype-strict.tsv@spec=runs/ws3-shipped/tintype.json#strict" \
  --scores "commfor=$OUT/*.commfor.tsv@logits" \
  --scores "gpttrace=$OUT/*.gpttrace.tsv@logits" \
  --scores "probe-pe=$OUT/*.probe-pe.tsv@logits" \
  --set fit=$SPLIT/fit.tsv --set calibrate=$SPLIT/calibrate.tsv --set test=$SPLIT/test.tsv \
  --profile tintype-sensitive --profile tintype-strict \
  --second gpttrace --second probe-pe --margin 1,2,3 \
  --cohort-by transform --slice-by transform \
  --write-scores runs/ws3/policies > runs/ws3/policy-comparison.md
```

Leave out the `--scores` and `--second` of a model that has no scores yet.
What to read in the output:

- "Fusion weights fitted on the fit set only": the fitted weights.
- "Cutoffs frozen on the calibration set, applied here unchanged": the
  frozen cutoff of each policy (a logit of its score) with the actual FPR
  and recall on the test set, overall and per transform. This table, on the
  degraded transforms, is what the gate is read from. The first table of
  each block re-chooses cutoffs on the test set and is a ranking comparison
  only.
- The gate tables: first-stage misses outside each gate and how many the
  second model would flag.

Check the real two-stage runs against the arithmetic (as confirmation 7's
log did): add `--scores "tintype-second=$OUT/test.tintype-second.tsv"
--scores "tintype-narrow=$OUT/test.tintype-narrow.tsv" --stored
tintype-second --stored tintype-narrow` to a run restricted to the test
set (`--set test=...` only) and compare each stored row with the
recomputed "Shipped CF blend" and "Narrowed CF blend" rows: the counts
should agree.

View disagreement, analysis only (the six-view files as the first stage):

```sh
python3 scripts/compare-policies.py \
  --scores "tintype-views=$OUT/*.tintype-views.tsv@spec=runs/ws3-shipped/tintype.json" \
  --scores "gpttrace=$OUT/*.gpttrace.tsv@logits" --first tintype-views --second gpttrace \
  --set fit=$SPLIT/fit.tsv --set calibrate=$SPLIT/calibrate.tsv --set test=$SPLIT/test.tsv \
  --margin 1,2,3 --view-spread tintype-views --slice-by transform
```

### 5. Intervals

The harness prints counts. For intervals, clustered by parent, and for
slices, hand its score files to the report of step 0:

```sh
tintype report --operating-points --fpr 0.001,0.01 --groups $SPLIT/test.tsv --cluster-by parent --slice-by transform \
  --system "tintype=runs/ws3/policies/tintype-default.tsv" \
  --system "gated=runs/ws3/policies/<policy slug>.tsv"
```

The files hold `sigmoid(policy score)`, which `tintype report` reads as
32-bit probabilities, so extremely confident scores can tie there; the
harness's counts are the reference for the ranking. This was tried on the
Arena archive: the per-variant counts at 0.5 agreed with the harness.

### 6. Latency, route rate and memory

On the hardware the bound is written for, idle, CPU engine, on a fixed
sample of the *calibration* set (not the test set):

```sh
head -500 $SPLIT/calibrate.tsv > runs/ws3/latency-sample.tsv   # absolute paths: any folder works
/usr/bin/time -l tintype latency --weights runs/ws3-shipped/tintype.safetensors --manifest runs/ws3/latency-sample.tsv --n 400 --device cpu
/usr/bin/time -l tintype latency --weights runs/ws3-shipped/tintype.safetensors --manifest runs/ws3/latency-sample.tsv --n 400 --device cpu --photo-second-opinion $CF
/usr/bin/time -l tintype latency --weights runs/ws3-narrow/tintype.safetensors  --manifest runs/ws3/latency-sample.tsv --n 400 --device cpu --photo-second-opinion $CF
/usr/bin/time -l tintype latency --weights $CF --reference commfor --manifest runs/ws3/latency-sample.tsv --n 400 --device cpu
```

Record, per run: median and p95 (first line), the share of images inside
the band and their own median and p95 (second line), and "maximum resident
set size" (`/usr/bin/time -v` on Linux). Run each twice and record the load
average. The route rate that counts for the gate is the one the harness
prints on the test set; the latency sample only prices it. A second model
that runs outside `tintype` (GPTTrace, an encoder probe) has to be timed
in its own runtime, per image, on the same files and the same machine, and
its p95 added for the routed share.

### 7. Before anything ships

A narrowed band, a new second model or a fusion rule becomes a release
candidate only with all of: selection on development data (not the test
set), the frozen-split table above meeting the gate, re-calibration and an
early-exit re-check where the first stage changes, and a new written
`docs/PREREGISTRATION.md` entry. Until then `models/tintype.json` keeps
its band `(-7, 0.5, 0.5, 0)`.

## A new second-stage network gets its own flag

Nothing is implemented for a network that does not exist in the engine. If
an encoder probe passes the gate, the intended shape is:

- a new opt-in flag, `--photo-encoder-opinion <weights>`, on `detect`,
  `score` and `latency`, conflicting with `--fast`, `--sensitive`,
  `--strict` and with `--photo-second-opinion` (which stays the Community
  Forensics route);
- a new optional spec field for its gate and fusion weights, written by
  its own `set-...` command on a copied spec, with
  `#[serde(default, skip_serializing_if = "Option::is_none")]` so the
  shipped specs load and round-trip unchanged;
- no new value of `verdict`; any extra output is a new, additive JSON
  field documented in `docs/API.md`;
- the runtime stays pure Rust, so the encoder needs its own kernels
  (DINOv3: RoPE and register tokens) before this flag can exist.

## Not done here, and why

- No scoring on a frozen split: it does not exist yet.
- No GPTTrace or encoder scores beyond the Arena archive: no checkpoint
  was downloaded, and the licences are recorded above as a step to
  confirm.
- No manifest-driven GPTTrace scorer and no feature extractor: neither
  could be run on this machine.
- No learned gate.
