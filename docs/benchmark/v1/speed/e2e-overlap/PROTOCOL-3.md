# Protocol 3: idle-GPU re-time of origin/main against C3 (registered before its timed run)

[RESULT.md](RESULT.md) and [RESULT-2.md](RESULT-2.md) were timed while
another session trained on the same GPU. This run re-times them with the GPU
free, as `../PROTOCOL.md` asks for any speed figure that is reported.

## Machine

- The Apple M6. Nothing is training or scoring on the GPU. The research
  session's jobs during the run are network-bound: image generation through
  OpenRouter, and a Commons/TPB fetch writing to the data share, which keeps
  `diskimagesiod` busy on the CPU.
- The 1-minute load was 3.10 just before registration, a little above 3.
  It is recorded before every timed run, with the processes above 20% CPU and
  any other `tintype`, `cargo` or `rustc` process, as in
  [speed_ab.py](speed_ab.py).

## What is timed

- **Files:** TPB v1's speed sample, the 2,000 open-split files with the
  smallest SHA-256, and the next 100 as warm-up. They were copied from
  `data/fresh10` to a fresh RAM disk just before this registration, and every
  SHA-256 matched the manifest.
- **Binaries:**
  - B, origin/main 456d5ab0 (SHA-256 a3e4d4ba…);
  - C3, 43525b32, the code at PR #60's head (80637fac…). The commits after
    it change only documentation.

  Both use `batch --json <folder>` with the shipped models, so the second
  opinion is on as shipped. Each binary has its own GPU cache directory on the
  RAM disk.
- **Rounds:** seven. Each binary runs once on the warm-up, then once on the
  2,000 files, timed from process start to exit. The order alternates:
  B-C3, C3-B, and so on. This is `../PROTOCOL.md`'s round, with more rounds
  for two binaries. The output goes to [speed-3.tsv](speed-3.tsv). Every timed
  run's rows must equal the first run's.
- **Parity:** C3's rows were checked against B's in
  [RESULT-2.md](RESULT-2.md), on the sample, on all 24,961 batch-9 open files
  and on the mixed folder. Here every timed run must give the same 2,000
  rows.

## Reported

- Every round's ms per image and load.
- The median ms per image and images per second for each binary, and C3's
  ratio to B.
- Under `../PROTOCOL.md`'s rule: C3 is faster only if each of its rounds
  beats each of B's; otherwise "not separated". The comparison within each
  round is reported too.
- **One timed run**, reported as it is. If a training job starts on the GPU
  during the run, the affected rounds are marked as shared.
