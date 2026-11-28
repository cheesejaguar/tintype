# Result 3: with the GPU free, C3 is 13% faster than origin/main, with every row unchanged

The protocol is [PROTOCOL-3.md](PROTOCOL-3.md), registered in 36374602
before the timed run. This was the only timed run under it.

## Set-up

- **Machine:** the Apple M6, with nothing training or scoring on the GPU.
- **Background load:** the research session's jobs were network-bound:
  - OpenRouter generation and captioning (`or_gen.py`, `or_caption.py`);
  - a Commons/TPB fetch writing to the data share;
  - `build_blind2.py`, at about 6% CPU.

  `diskimagesiod`, the data share's disk image, was busy before several runs.
- **Load:** the 1-minute load was 3.10 when the protocol was registered, a
  little above 3, and 2.69 at the first timed run. It rose to between 6 and 8
  from round 2 on, as the detectors' own threads and the share's I/O ran,
  much as `../photo-chunk/RESULT.md` saw on its idle run (3.2 to 6.9).
- **Other processes:** no other `tintype`, `cargo` or `rustc` process ran
  during any timed run ([speed-3.tsv](speed-3.tsv)).
- **Run:** 04:05:59 to 04:07:07 on 2026-11-28.
- **Files:** TPB v1's speed sample and warm-up, copied from `data/fresh10` to
  a fresh RAM disk, with every SHA-256 matching the manifest.
- **Binaries:**
  - B, origin/main 456d5ab0 (SHA-256 a3e4d4ba…);
  - C3, 43525b32, the code at PR #60's head (80637fac…).

Every one of the 14 timed runs wrote the same 2,000 rows. C3's parity with B
on the sample, on all 24,961 batch-9 open files and on the mixed folder is in
[RESULT-2.md](RESULT-2.md).

## Speed

Milliseconds per image, from process start to exit over 2,000 files:

| Round | Order | B | C3 | C3 against B | Load (B, C3) |
|---|---|---|---|---|---|
| 1 | B, C3 | 2.280 | 1.922 | −15.7% | 2.69, 2.64 |
| 2 | C3, B | 2.240 | 1.972 | −12.0% | 7.03, 3.55 |
| 3 | B, C3 | 2.270 | 1.975 | −13.0% | 6.78, 6.40 |
| 4 | C3, B | 2.238 | 1.942 | −13.2% | 7.07, 6.21 |
| 5 | B, C3 | 2.260 | 1.993 | −11.8% | 7.63, 8.30 |
| 6 | C3, B | 2.348 | 2.065 | −12.1% | 7.84, 8.35 |
| 7 | B, C3 | 2.387 | 2.119 | −11.2% | 7.38, 6.95 |
| **Median** | | **2.270** | **1.975** | | |
| **Images per second** | | **441** | **506** | | |
| **Ratio to B** | | 1.000 | **0.870** | | |

- **By `../PROTOCOL.md`'s rule, C3 is faster than B.** C3's slowest round,
  2.119, beats B's fastest, 2.238.
- **Round by round, C3 was 11.2% to 15.7% faster.** By the medians it takes
  13.0% less time per image: 506 images per second against 441.
- **With the GPU shared, the gain was smaller:** 8.4% in
  [RESULT-2.md](RESULT-2.md). There the training session's work filled some
  of the GPU gaps that origin/main leaves.

This is the idle-GPU figure the earlier results asked for. The CPU was not
quiet (see above), and this is one machine and one sample.
