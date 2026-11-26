# Tintype Photo Benchmark v1, blind split: result (2026-11-26)

## The read

- **Protocol:** [BLIND-PROTOCOL.md](BLIND-PROTOCOL.md), registered in
  788e31ec before any blind file was listed. The manifest was frozen in
  e1dded70 before any detector ran.
- **Amendment 1** (fa57a54a) was made after scoring stopped on the last
  set and before any score was read. It does two things:
  - three GIFs that had been named `.bin` get `.gif` paths;
  - a file a detector cannot decode counts as not flagged for it.
- **The read** ran once:
  [read_v1_blind.sh](../../../scripts/benchmark/read_v1_blind.sh).
  - Raw output: [blind-read/read.txt](blind-read/read.txt).
  - Tables: [blind-read/tables.md](blind-read/tables.md).
  - Per-file scores: [blind-scores.tsv.gz](blind-scores.tsv.gz).

## The blind split

**23,843 files: 13,244 generated and 10,599 real.** They cover the same 27
sets as the open split. No file was used by any earlier evaluation, and
none is byte-identical to a training file, an open-split file or an
OpenFake-split file. [blind-build.txt](blind-build.txt) gives the counts
per set and class:
- No set was dropped. CycleGAN had only 123 and 122 unused files, so it
  is smaller than in the open split.
- **606 selected files were dropped as byte-identical to training-store
  files.** They are mostly AIGIBench real photos, up to 65 in one set.
  The open split was cleaned the same way when batch 9 was built.
- 88 were dropped as duplicates of earlier evaluation files. Every one
  was rebuilt from its source with a matching SHA-256 before scoring.

## Results

Each detector is at its shipped cutoff. 95% intervals are in brackets.

| Detector | Precision | Recall | Real flagged | Recall at 1% FPR | ms per image, as measured |
|---|---|---|---|---|---|
| **Tintype v0.4.0 default** | 98.98% (98.81–99.14) | **93.01%** (92.58–93.43) | 1.20% (1.00–1.41) | **92.49%** (91.38–93.18) | 7.7 |
| GPTTrace flag-pec-v3 | **99.21%** (99.03–99.37) | 74.93% (74.18–75.70) | **0.75%** (0.59–0.92) | 78.41% (76.48–80.42) | 82.3 |
| Community Forensics | 95.25% (94.90–95.62) | 84.03% (83.41–84.67) | 5.24% (4.81–5.64) | 69.96% (68.15–71.59) | 5.3 |
| SwinV2 detector | 85.92% (85.20–86.65) | 49.71% (48.83–50.60) | 10.18% (9.61–10.77) | 28.94% (27.70–30.44) | 18.9 |

- GPTTrace could not decode one file, an 8000 × 12000 JPEG, so under the
  amendment it counts as not flagged.
- Times are wall-clock per image, with images read from the network
  share while other jobs used the GPU. They are not speed claims.

**Paired, Tintype − GPTTrace:**
- precision −0.46 to +0.01: not separated;
- recall +17.27 to +18.88;
- real photos flagged +0.20 to +0.70;
- recall at 1% false positives +11.76 to +16.11.

Tintype is ahead of Community Forensics and SwinV2 on every measure. The
full table of pairs is in [blind-read/tables.md](blind-read/tables.md).

## 1. Blind against the website (the open split)

| Tintype v0.4.0 | Open (website) | Blind | Blind − open (95% interval) |
|---|---|---|---|
| Precision | 98.95% | 98.98% | +0.03 (−0.22 to +0.27) |
| Recall | 93.31% | 93.01% | −0.31 (−0.90 to +0.32) |
| Real photos flagged | 1.16% | 1.20% | +0.04 (−0.24 to +0.32) |
| Recall at 1% false positives | 92.29% | 92.49% | +0.20 (−1.56 to +1.68) |

**The blind numbers differ from the website's by at most 0.31 points.**
Every interval includes zero. All 27 sets are in both splits, so the
same-sets comparison is identical.

The other detectors are also unchanged within noise. The exception is
SwinV2, which is 2.1 points more precise on the blind split (+1.06 to
+3.14).

By set, Tintype's recall moves both ways, within what samples of a few
hundred files allow:
- AIGIBench Imagen 3: 94.4% blind against 90.9% open;
- PhotoMaker: 73.5% against 77.8%;
- So-Fake-OOD: 82.3% against 84.4%.

The two weakest sets are the same on both splits: classic deepfakes
(25.6% blind) and PhotoMaker.

## 2. Do the site's claims need changing?

**No.** Under the registered rule:

| Site figure | Blind 95% interval | Holds? |
|---|---|---|
| 99.0% precision | 98.81–99.14 | yes |
| 93.3% recall | 92.58–93.43 | yes |
| 1.2% false positives | 1.00–1.41 | yes |
| 92.3% recall at 1% false positives | 91.38–93.18 | yes |
| Tintype first on recall | +8.30 or more over every other detector | yes |
| Tintype first on recall at 1% false positives | +11.76 or more over every other detector | yes |
| Tintype and GPTTrace not separated on precision | −0.46 to +0.01 | yes |

**A correction to the protocol.** It described the site's ranking on
recall as Tintype / GPTTrace / Community Forensics / SwinV2. That is the
table's row order, not its recall order. On both splits the recall order
is Tintype, then Community Forensics, then GPTTrace, then SwinV2. The
site states no ranking beyond the numbers in its table, so nothing on the
site depends on this.

As the protocol says, the site gains a sentence citing the blind split.
The note that GPTTrace's recall is 19 points lower describes the open
split. On the blind split it is 18.1 points lower.

## Exploratory: the sets that motivated the new-generator work

This comparison was run after the read. It is not a registered question.

On the open split, GPTTrace beat the default on AIGIBench Imagen 3 and
PhotoMaker. On the blind split, with a paired bootstrap of recall
(Tintype − GPTTrace, 2,000 draws):

| Set | Generated | Tintype | GPTTrace | Difference (95% interval) |
|---|---|---|---|---|
| Imagen 3 | 450 | 94.4% | 94.7% | −3.1 to +2.7 |
| PhotoMaker | 400 | 73.5% | 95.5% | −26.5 to −17.5 |
| Midjourney | 199 | 96.0% | 99.0% | −5.5 to −1.0 |
| DALL-E 3 | 440 | 98.9% | 99.5% | −1.8 to +0.2 |
| So-Fake-OOD | 1,495 | 82.3% | 79.7% | +0.4 to +5.0 |

- **Imagen 3:** the gap does not appear on the blind split. The
  default's Imagen 3 recall was 90.9% on the open split and 94.4% on the
  blind one, which is consistent with sampling noise on a few hundred
  files.
- **PhotoMaker:** the gap is large on both splits. It is the clearer
  target for the [new generator playbook](../../NEW_GENERATOR_PLAYBOOK.md).

## Caveats

- **Same generators, unseen files.** The split comes from the same
  archives as the open split, so it says nothing about new generators.
  That is what v2 is for.
- **AIGIBench was scored in full the same day.** The AIGIBench
  full-protocol comparison (ea6e4411) scored every AIGIBench test file,
  these included, with the same v0.4.0 default. Nothing was trained,
  calibrated or tuned on those scores, and this split's protocol was
  registered before that comparison was read.
- **GPTTrace's training data cannot be checked against these files.**
- **Resampling is by file.** The intervals ignore clusters of related
  images.
