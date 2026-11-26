# Speed per image on matched, idle hardware: protocol (registered 2026-11-26, before any timing)

## Why

The speed figures so far were not matched:
- the website's 300–364 photos/s for Tintype is on the batch-7 sample;
- GPTTrace's 0.109 s per photo is from the NTIRE run;
- the TPB v1 blind read timed all four detectors, but on files read from
  the network share while training shared the GPU.

This run times the four detectors on the same files, on the same idle
machine, from the same storage.

## Set-up

- **Machine:** Apple M6, macOS 27.2. It must be idle: no other `tintype`,
  training or scoring process, with the 1-minute load average recorded at
  the start of each round.
- **Files:** TPB v1's speed sample, the 2,000 files of
  `docs/benchmark/v1/open.tsv.gz` with the smallest SHA-256. This is a
  content-random draw across the 27 sets.
  - They are rebuilt with `tpb.py rebuild`, so their hashes are checked.
  - They sit on a RAM disk, so storage is identical for every detector
    and not the network.
  - The warm-up files are the next 100 by SHA-256.
- **Detectors, at their shipped settings:**
  1. Tintype v0.4.0 default: `tintype batch --json <dir>`, build `0947f3a6…`
     with `TINTYPE_MODEL_DIR=models`, with the photo second opinion as
     shipped.
  2. GPTTrace flag-pec-v3: research build `dc1cd035…`,
     `tintype batch --json --gpttrace <weights>`, with the pinned weights
     (SHA-256 `a0adec67…`) on the RAM disk.
  3. Community Forensics 384: `tintype score --reference commfor`, 32-bit,
     batch 64.
  4. SwinV2 detector: `tintype score --reference swinv2`, 32-bit,
     batch 64.

  Community Forensics and SwinV2 run through Tintype's validated Rust
  ports, so their times are "through this engine", not as released.

## Measure

- **Time per image:** wall-clock seconds from process start to exit, over
  the 2,000 files, divided by 2,000. Model loading and decoding are
  included.
- **Order:** each detector runs once on the 100 warm-up files, then once
  on the 2,000. That makes one round. There are three rounds, in rotated
  order: 1-2-3-4, 2-3-4-1, 3-4-1-2.
- **Reported:** each round, the median of three per detector, and the
  ratio of each detector's median to Tintype's.
- **Ranking:** "Tintype faster" if all three of its rounds beat all three
  of the other detector's. Otherwise "not separated".

## Clean-up

Unmount the RAM disk, and delete the GPTTrace weights copy that
`tpb-v1` re-downloaded to the share.

Script: [speed_run.py](speed_run.py), run once.
