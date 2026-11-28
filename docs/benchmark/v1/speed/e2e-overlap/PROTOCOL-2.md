# Protocol 2: the second network in 16-view runs (registered before its timed run)

Follows [PROTOCOL.md](PROTOCOL.md) and [RESULT.md](RESULT.md), with one
more change on `perf/e2e-speed`.

## The change

**C3, 43525b32: the Community Forensics second network runs 16 views at a
time on the GPU.** It is still loaded and fed for batches of 64. Each batch
is split into runs of the smallest compiled batch size of at least 16 views.
The first stage keeps 64-view runs.

Every batch size the GPU engine keeps gives the full batch's logits bit for
bit: `gpu_runs` checks it on the machine itself with probe views, and drops
any size that does not. The tail of a second-opinion pass already ran in the
16-view executable before this change. So no logit changes.

**Why, and what was measured before this registration:**
- `tintype profile --model-only` on the M6, with another session training on
  the GPU: the second network gets through 441 and 430 views per second in
  16-view runs, 423 and 419 at 32, and 401 and 405 at 64. The photo network
  gets through 1,459 and 1,443 at 16, and 1,386 and 1,375 at 64.
- In a profiling build on the speed sample, the second network's 16-view runs
  took 37.3 ms on average, against 160–180 ms for its 64-view runs.
- An environment switch in an experimental build set the run size of each
  network. In five interleaved rounds on the speed sample, at a 1-minute load
  of about 6 to 9, the medians were:
  - both networks at 64 views: 4.323 s;
  - the second network at 16: 4.181 s;
  - both at 16: 4.138 s;
  - the photo network at 32 and the second at 16: 4.172 s.

  The photo network's share of the gain was within the noise, and a 16-view
  run of it holds few tokens for a larger GPU, so C3 changes the second
  network only. Peak resident memory was 1.75–1.95 GB with 16-view runs of
  the second network, against 1.98–2.08 GB without.
- One run of the C3 binary on the speed sample took 3.950 s, with its 2,000
  rows identical to B's.

These figures are not this protocol's result.

## Parity gates, before the timed run (all must hold)

C3's binary (SHA-256 80637fac…) against B (a3e4d4ba…), as in
[PROTOCOL.md](PROTOCOL.md):
1. the speed sample: every row equal, standard output byte-identical;
2. all 24,961 batch-9 open files with [parity_staged.py](parity_staged.py),
   in chunks of 1,500: every field of every row equal, and each chunk's
   standard output byte-identical;
3. the mixed folder with [mixed_parity.py](mixed_parity.py): standard output
   byte-identical;
4. `cargo test --release` passes, and so do the ignored GPU tests
   `view_reuse` and `second_beside`.

## Speed

As in [PROTOCOL.md](PROTOCOL.md): the Apple M6, shared tonight with a
training session; the TPB v1 speed sample and warm-up on a RAM disk;
[speed_ab.py](speed_ab.py) with every binary in its own GPU cache directory.
- **Binaries:** B (origin/main), C2 (27aba0fb) and C3 (43525b32).
- **Rounds:** seven, in the rotated order B-C2-C3, C2-C3-B, C3-B-C2, and so
  on. The output goes to [speed-2.tsv](speed-2.tsv).
- **Reported:** every round; the median per binary and its ratio to B's;
  C3 against B and C3 against C2 by the registered rule (faster only if each
  of its rounds beats each of the other's, otherwise "not separated"); and,
  as a second reading, the comparison within each round.
- **One timed run**, reported as it is. The website figure still needs an
  idle-machine re-time.
