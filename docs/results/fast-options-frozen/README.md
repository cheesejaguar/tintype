# Fast options on the frozen split: INT8 kernels and distilled students

Raw output of the pre-registered entry
[Fast options on the frozen split](../../PREREGISTRATION.md#fast-options-on-the-frozen-split-int8-kernels-and-distilled-students-what-was-fixed-before-the-look),
run on 2026-11-15. The write-up is in
[the INT8 runbook](../../runbooks/cpu-profile-int8.md#step-6-integer-kernels-and-students-on-the-frozen-split).

**Outcome.** The INT8 kernels pass the accuracy floor with nothing lost
that the split can see. On the speed gate they pass with the pure-Rust
kernels that Linux and Windows run (1.93x throughput, 28% lower p95) and
fail on macOS with Accelerate (1.23x, 6% lower p95). `--device cpu-int8`
ships as an option with the released model and spec. No student ships:
every one fails the recall clauses by a wide margin and is no faster on
the GPU.

## Accuracy at each candidate's own cutoffs, fitted on `calibrate`, on `test`

19,326 rows: 9,600 generated and 9,726 authentic. Clean rows are the
14,526 originals; processed rows are the 4,800 copies. Early exit runs as
it will ship, with the margins refitted on `fit`. The floor is ≤1.5%
authentic flagged, clean recall no more than 5 points below the
incumbent, and processed recall no more than 8 below. The incumbent is
the released model with its cutoffs refitted the same way, on the GPU.

| Candidate | Profile | Authentic flagged | Clean recall | Processed recall | Floor |
|---|---|---|---|---|---|
| Incumbent, GPU | 1% | 125 of 9,726 (1.29%) | 95.71% | 82.08% | reference |
| Incumbent, `cpu` | 1% | 125 (1.29%) | 95.69% | 82.21% | (passes) |
| **INT8 kernels, `cpu-int8`** | 1% | 127 (1.31%) | 95.79% | 82.33% | **passes** |
| Student, 8 blocks | 1% | 71 (0.73%) | 78.49% | 51.04% | fails (−17.2, −31.0) |
| Student, 7 blocks | 1% | 69 (0.71%) | 65.03% | 39.96% | fails |
| Student, 6 blocks | 1% | 91 (0.94%) | 28.60% | 19.71% | fails |
| Incumbent, GPU | 0.1% | 22 (0.23%) | 92.71% | 72.62% | not gated |
| Incumbent, `cpu` | 0.1% | 21 (0.22%) | 92.71% | 72.75% | not gated |
| INT8 kernels | 0.1% | 22 (0.23%) | 92.88% | 72.88% | not gated |
| Student, 8 blocks | 0.1% | 5 (0.05%) | 63.65% | 36.79% | not gated |

Between `cpu` and `cpu-int8`, verdicts differ on 56 of 19,326 rows at 1%:

- INT8 alone flags 9 authentic and 25 generated rows.
- `cpu` alone flags 7 authentic and 15 generated rows.

At 0.1%, 78 differ: INT8 alone flags 3 authentic and 44 generated rows; `cpu` alone flags 2 authentic and 29 generated.

The full table is in `floor.md`, computed by `floor.py`.

The fitted cutoffs of INT8 and `cpu` are within 0.02 raw logits of each
other: 0.991 against 0.991 at 1%, and 2.993 against 2.973 at 0.1%. INT8
therefore ships with the **released** spec, whose cutoffs are stricter
than these research points.

That shipped configuration was checked on `fit` and `calibrate` only, so
`test` was not looked at twice. Those two sets hold 16,216 items, 7,516 of
them authentic, scored with the released cutoffs and stored early-exit
schedule:

| Profile | INT8 | `cpu` |
|---|---|---|
| Default | 36 authentic flagged, recall 87.38% | 32 flagged, recall 87.17% |
| Strict | 5 flagged, recall 82.34% | 4 flagged, recall 81.98% |

Early exit stays within 0.1 point of scoring every view for both engines.
At the default profile, 64 of the 16,216 verdicts differ (0.39%).

## Speed, file to verdict

**This Mac (Apple M6).** 300 `test` rows drawn with seed 8, 150 of each
class (`speed.sh`, `speed-m6.log`). Each system has its own early exit and
the default profile. The systems took turns, best of three, on an idle
machine. Latency is over 295 files after 5 warm-up files; throughput
times one `score` process over all 300.

| Build | Engine | p95 latency | Median | Throughput | Gate (≥1.3x or ≥25% lower p95) |
|---|---|---|---|---|---|
| Accelerate (macOS default) | `cpu` | 102.8 ms | 39.7 ms | 64.5 files/s | – |
| Accelerate | `cpu-int8` | 96.8 ms | 34.9 ms | 79.6 files/s | **fails**: 1.23x, −5.8% |
| `--features portable-cpu` (the Linux/Windows code) | `cpu` | 147.8 ms | 64.1 ms | 42.2 files/s | – |
| `portable-cpu` | `cpu-int8` | 106.7 ms | 42.8 ms | 81.3 files/s | **passes**: 1.93x, −27.8% |
| GPU | incumbent | 80.3 ms | 13.1 ms | 172 files/s | – |
| GPU | student, 8 blocks | 79.4 ms | 11.9 ms | 189 files/s | fails: 1.09x, −1.1% |

**Hosted runners.** These are reported as measured, not as the
pre-registered gate: the cohort is two demo PNGs repeated 100 times each,
not `test` rows. They come from the temporary workflow
`.github/workflows/int8-bench.yml`, run 2026-11-15, best of three.

| Runner | Kernel | `cpu` p95 | `cpu-int8` p95 | 200 photos, `cpu` | 200 photos, `cpu-int8` |
|---|---|---|---|---|---|
| Linux ARM64 | NEON `SDOT` | 449.9 ms | 286.1 ms (−36%) | 42.0 s | 21.4 s (1.96x) |
| Linux x86-64, AMD EPYC 9V45 | AVX-VNNI | 438.8 ms | 339.1 ms (−23%) | 53.1 s | 34.6 s (1.54x) |
| Windows x86-64, AMD family 25 | AVX-512 VNNI (256-bit) | 371.0 ms | 303.2 ms (−18%) | 42.0 s | 28.3 s (1.48x) |
| Linux x86-64, AVX2 forced | AVX2 `vpmaddwd` | 438.8 ms | 508.2 ms (**+16%**) | – | – |
| Windows x86-64, AVX2 forced | AVX2 `vpmaddwd` | 371.0 ms | 426.7 ms (**+15%**) | – | – |

The AVX2 kernel is exact but slower than `cpu` file to verdict. On a
processor with AVX2 and no VNNI, `cpu-int8` therefore prints a note
saying so.

## Files

| File | What |
|---|---|
| `eval.sh` | Prepare (score `fit` and `calibrate` with every view, fit the 1% and 0.1% cutoffs, refit the exit margins) and the one test look, per candidate |
| `floor.py`, `floor.md` | The accuracy floor, computed once from the test scores |
| `speed.sh`, `speed-m6.log` | The speed gate on this Mac |
| `spec-<candidate>.json` | Each candidate's spec after preparation, frozen before its look (SHA-256 checked by `eval.sh test`) |
| `calibrate-<candidate>.txt`, `refit-exit-<candidate>.txt` | Output of `calibrate` and `refit-exit` |
| `test-<candidate>.tsv.gz`, `test-strict-<candidate>.tsv.gz` | `score` on `test`, default profile and `--strict` (INT8, `cpu` incumbent, best student) |
| `fit-views-<engine>.tsv.gz`, `calibrate-views-<engine>.tsv.gz` | Every view's logit on `fit` and `calibrate`, for `cpu` and `cpu-int8`; `refit-exit --apply-out` rebuilds the decided files from these and a spec |

Paths are relative to the training tree. The students' weights are not
archived; they were trained with `train --init <released weights>
--init-blocks <list> --distill <released weights> --distill-weight 0.5
--distill-temperature 2 --steps 8000 --batch 32 --lr 2e-5 --warmup 300
--seed 1` on the 25 phase-6 manifests. Their validation AUCs at step 8000
were 0.964, 0.935 and 0.840 for 8, 7 and 6 blocks.
