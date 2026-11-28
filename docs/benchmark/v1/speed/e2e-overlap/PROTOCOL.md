# Protocol: keep the GPU busy through early exit and the photo second opinion (registered before the timed run)

## Why

With `TINTYPE_STATS` and stage marks added to a profiling build of
origin/main (456d5ab0), `batch --json` on the TPB v1 speed sample (2,000
photos on a RAM disk) spends its time like this. These are three runs at a
1-minute load of about 15, with another session training on the same GPU:

| Stage | Wall time | Inside network runs |
|---|---|---|
| Photo network loaded from the GPU cache | 0.033–0.040 s | |
| First stage, early exit, 2,000 photos | 3.04–3.10 s | 2.77–2.82 s |
| Second network loaded | 0.004 s | |
| Second opinion, the 516 photos inside the band | 1.73–1.80 s | 1.47–1.52 s |
| Rows written | 0.002 s | |

- **The GPU sets the pace, and it waits.** One 64-view run of the photo
  network takes 46–49 ms alone (`tintype profile --model-only`) and 50–58 ms
  in the pipeline. The Community Forensics network takes 165–185 ms per run.
  The GPU is idle for about 0.6–0.9 s of the 4.9 s:
  - between most first-stage runs, 5–25 ms each, about 0.3 s in all. The
    pipeline gave the network its next batch only after the running batch's
    logits came back;
  - at the start of the second opinion (80–94 ms), and between its runs
    (11–54 ms each), while its photos are read and resized again;
  - in the first stage's last, part-filled batches, which run with nothing
    beside them.
- **The CPU is not the limit.** The run uses 15.4 s of user time in 4.9 s,
  3.4 of the 12 cores on average. Decoding all 2,000 photos alone takes 0.81 s
  on 12 threads, and cutting out all six views of each takes 0.64 s.

Two other ideas were measured and dropped:
- Submitting GPU runs asynchronously, with two command buffers in flight,
  made a 64-view run 1–2% faster alone and no faster under CPU load.
- Running the two networks at once on two GPU queues stretched the runs to
  77 ms and 280 ms, and the GPU was busy for longer in all than with the runs
  taken in turn.

## The changes

Two commits on `perf/e2e-speed`, both in `src/detector.rs`, the second also
in `src/scan.rs`:
1. **C1, 7ef4f84c: early exit queues the next batch behind the running one**,
   and takes in what the workers finish while it waits for logits. A batch
   that is not full is sent only when nothing more is coming and nothing
   else is in flight, as before.
2. **C2, 27aba0fb: the photo second opinion runs beside early exit.** A photo
   whose first-stage calibrated logit is inside the band goes to the second
   network as soon as its own first stage is done. Workers of its own read
   it again, and the two networks take turns on the GPU, one run at a time.
   This applies in `batch`, and in `detect` with more than 10 photos, on the
   GPU engine.

Neither changes which views are scored, their pixels, which photos are
asked, or any logit. On the GPU engine a view's logit does not depend on
what it is run with: this is checked bit for bit by the ignored GPU test in
`tests/view_reuse.rs`, and `gpu_runs` keeps only the batch sizes that give
the full batch's logits bit for bit. Only when each batch runs, and what it
runs with, change.

**Disclosed before this registration:** while developing, the C2 code was
timed informally against origin/main in three interleaved rounds on the
speed sample, at a 1-minute load of 9 to 11. origin/main took 4.472, 4.602
and 4.477 s, and the C2 code 3.983, 3.951 and 3.998 s. Its 2,000 rows were
identical. Its peak resident memory was about 1.9–2.1 GB, against 1.5–1.7 GB.
These figures are not this protocol's result.

## Parity gates, before the timed run (all must hold)

The binaries are B, origin/main 456d5ab0 (SHA-256 a3e4d4ba…), and C1 and C2,
all built with the same toolchain (rustc 1.99.1), with
`TINTYPE_MODEL_DIR` set to the verified release models.

1. **The speed sample.** B, C1 and C2 each score the 2,000 files. Every
   `batch --json` row must be equal.
2. **All batch-9 open files.** [parity_staged.py](parity_staged.py) stages
   the 24,961 files of `docs/benchmark/v1/open.tsv.gz` from `data/fresh10`
   through the RAM disk in chunks of 1,500, checking each file's SHA-256
   against the manifest. B, C1 and C2 each score every chunk. Every field of
   every row must equal B's, and each chunk's standard output must be
   byte-identical.
3. **A mixed folder.** [mixed_parity.py](mixed_parity.py) builds the folder of
   `../photo-chunk/`: the first 1,000 photos of the speed sample and the 300
   clips of `../photo-chunk/mixed_videos.txt`, interleaved. Clips go through
   the early-exit pipeline of C1 too. The standard output of B, C1 and C2
   must be byte-identical.
4. **Tests:** `cargo test --release` passes, and so do the ignored GPU tests
   `view_reuse` and `second_beside`.

Any difference stops the timing.

These sets are non-blind evaluation data. Parity fits nothing.

## Speed

- **Machine:** the Apple M6. **It is not idle tonight:** another session is
  training and scoring on the same GPU, and the user is away. So the rounds
  are interleaved, and the result is a ratio under the same load, not an
  idle-machine time.
- **Files:** TPB v1's speed sample, the 2,000 open-split files with the
  smallest SHA-256, with the next 100 as a warm-up. They are copied from
  `data/fresh10` to a RAM disk, with every SHA-256 checked.
- **Binaries:** B, C1 and C2, each running `batch --json <folder>` with the
  shipped models, so the second opinion is on as shipped. Each has its own
  GPU cache directory on the RAM disk.
- **Rounds:** [speed_ab.py](speed_ab.py) runs five rounds. In each, every
  binary runs once on the warm-up, then once on the 2,000 files, timed from
  process start to exit. The order rotates: B-C1-C2, C1-C2-B, C2-B-C1, and
  so on. Each timed run records the 1-minute load, the processes above 20%
  CPU, and whether another `tintype`, `cargo` or `rustc` process was
  running. Every run's rows must be equal.
- **Reported:** every round, the median per binary, and each median's ratio
  to B's. C2 is called faster than B only if each of its five rounds beats
  each of B's; otherwise "not separated".
- **One timed run.** It is reported as it is, whatever the load, and not
  repeated selectively.
- **The final speed figure needs an idle-machine re-time** under
  `../PROTOCOL.md` before it goes on the website.
