# Protocol: speed of `batch` with photos in 4,096-input chunks (registered before the timed run)

## Why

`batch` scores its inputs in chunks of 256 and writes a chunk's rows
when that chunk is done. Every chunk fills and drains the photo pipeline
twice: once for the first pass, where decode workers feed the network,
and once for the Community Forensics second opinion. Each time, the
network waits.

With `TINTYPE_STATS` on, origin/main (273e03f3) on the TPB v1 speed sample
spends about 4.3 s of its 5.3 s inside network runs. It needs 91 network
runs.

This change makes a chunk 4,096 inputs. A chunk also closes at 256 clips,
so a folder of videos writes its rows as often as before. Photo rows then
arrive once per 4,096 files instead of once per 256. Nothing that is
scored changes, as the parity gates below check. The change is in
`src/scan.rs` only.

**Disclosed before this registration:** while looking for the cause, the
same change with CHUNK = 4096 and no clip limit was timed informally
against origin/main. There were three interleaved rounds on the speed
sample, and the 1-minute load was 6 to 9 because other sessions were
working.
- origin/main: 2.637, 2.605 and 2.625 ms per image.
- The change: 2.211, 2.240 and 2.277 ms per image, with 63 network runs.
- The 2,000 rows were identical.

These figures are not this protocol's result. The registered run below is.

## Parity gates, before the timed run (all must hold)

1. **Photo rows against the registered build.**
   [parity_chunked.py](parity_chunked.py) compares A (0947f3a6) with C
   (this branch) on the 40,794 photos of openfake, openfake2,
   `dfdc-frames` and `dfdc-frames-real`. The photos are staged on a RAM
   disk in 300-file chunks, so C scores each staged chunk in one piece
   while A splits it at 256. Every field of every `batch --json` row must
   be equal. The same set gave 0 differing rows between A and origin/main
   in `../second-opinion-reuse/RESULT.md`.
2. **A mixed folder, against origin/main.**
   [mixed_folder.py](mixed_folder.py) builds a folder of 1,300 files:
   - the first 1,000 files of the speed sample;
   - the 300 smallest clips of vstore comgenvid, gameplay and pexelsb,
     listed in [mixed_videos.txt](mixed_videos.txt).

   They are interleaved three clips to every ten photos, so origin/main
   closes a chunk every 256 inputs and C closes one at its 256th clip.
   Both binaries' standard output must be byte-identical, which covers
   every row, its fields and its order. The comparison is with
   origin/main, B, not with A, because origin/main's VP9 and VP8 decoder
   changes video rows relative to A.
3. **Unit tests:** `cargo test --release` passes.

Any difference is reported to the main research session, and timing waits
until it agrees.

These sets are non-blind. Parity fits nothing.

## Speed, as `docs/benchmark/v1/speed/PROTOCOL.md`

- **Machine:** the Apple M6. No `cargo`, `rustc` or other `tintype`
  process may be running; [speed_run.py](speed_run.py) waits for them to
  finish. Other sessions are asked to hold their work on the Mac. The
  1-minute load average and any process above 20% CPU are recorded with
  every run.
- **Files:** TPB v1's speed sample, the 2,000 open-split files with the
  smallest SHA-256, with the next 100 as a warm-up. They are rebuilt on a
  RAM disk.
- **Detectors:**
  1. Tintype, registered build 0947f3a6.
  2. Tintype B, unmodified origin/main 273e03f3, same toolchain as C.
  3. Tintype C, this branch.
  4. BSF, AIGIBench's released Setting-II checkpoint (ce8ce061…), with its
     repository's own loader and network: MPS FP32, batch 32, 8 forked
     workers ([bsf_timer.py](bsf_timer.py)).
- Tintype runs as `batch --json <folder>` with `TINTYPE_MODEL_DIR` set to
  the shipped models, so the second opinion is on, as shipped.
- **Rounds:** each detector runs once on the warm-up, then once on the
  2,000 files, timed from process start to exit. There are three rounds
  in rotated order: 1-2-3-4, 2-3-4-1, 3-4-1-2.
- **Ranking**, for C against each other detector: "faster" if each of
  C's three rounds beats each of the other's three. Otherwise "not
  separated".
- **Wording:** if C is faster than BSF, say "fastest of the detectors
  timed on this sample and machine". Say nothing broader.
- **One timed run.** If the machine was not quiet (another `tintype`,
  `cargo` or training process ran), the run is reported as it is and not
  repeated selectively.

The 2,000-file sample fits in one chunk of C. A batch of more than 4,096
inputs has a chunk boundary every 4,096 inputs, where origin/main has one
every 256.
