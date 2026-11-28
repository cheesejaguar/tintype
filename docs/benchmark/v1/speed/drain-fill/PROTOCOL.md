# Protocol: fill the padding of part-full early-exit runs (registered before any timed run)

## The change

Stacked on `perf/e2e-speed` (PR #60). Commit a2a33be6, called D below,
changes the early-exit pipeline on the GPU engine.

A run that is not full is padded up to a compiled batch size, and the
padding slots are computed for nothing. D gives those slots views that the
photos in the run would look at next:
- the nearest stage first;
- only views already cut out, with no logit yet, and not the first view
  with those pixels that the current stage scores.

A photo that gets to such a stage finds the view scored, and needs no run of
its own for it.

This is aimed at `serve`. A request holds at most 16 photos, and early exit
takes it stage by stage, each stage a padded 16-view run waiting on the
stage before. `../serve-beside/RESULT.md` found that most of a request's
time goes there. `batch` ends each chunk the same way, so it gains a little.

**Why no output can change:**
- A view's logit on the GPU engine does not depend on what it is run with.
- Every stage reads the same views in the same order, with the same checks.
- A view scored early that its photo never reaches is dropped.
- The CPU engine pads nothing, so it fills nothing.

The new ignored GPU test `drain_fill` checks this: early exit gives the
logits of scoring every view and stopping by the same rule, bit for bit, at
batch sizes 16 and 64.

The binaries:
- B, PR #60's code, C3 at 43525b32 (SHA-256 80637fac…);
- D, a2a33be6 (0ea6c99a…).

**Disclosed before this registration:**
- A first build of D lacked the last of the three conditions above.
  - On the speed sample, its 2,000 `batch` rows equal B's. It scored 3,705
    views in 85 network runs, against 3,699 in 87.
  - One pass of the 2,000 photos through `serve` took 14.5 s of requests,
    against 15.8 s for B. Its served results equalled B's.
  - The GPU test `view_reuse` then failed: a view whose pixels the current
    stage already scores was taken twice. That build was fixed, and only the
    fixed one (a2a33be6) is gated and timed.
- No other timing was done.

## Parity gates (all must hold)

[parity_serve.py](parity_serve.py), which is `../serve-beside/`'s with
chunks of 500, compares D with B:
- `batch` rows field for field, and the standard output byte for byte;
- `serve` results over HTTP, field for field;
- each server against its own binary's `batch` rows.

The sets are:
1. **the speed sample**, 2,000 files;
2. **all 24,961 batch-9 open files**, staged from `data/fresh10` in chunks of
   500, with every SHA-256 checked against `open.tsv.gz`;
3. **the mixed folder** of `../photo-chunk/`: 1,000 photos and 300 clips.
   The clips go through the early-exit pipeline too;
4. **tests:** `cargo test --release` passes (313 tests), and so do the
   ignored GPU tests `view_reuse`, `second_beside` and `drain_fill`.

**Order.** The GPU is free now, and other GPU work starts later tonight. So
the timed runs come after gate 1 and before gates 2 and 3. Their result
counts only if every gate passes.

## Speed

Both runs use the Apple M6 and the speed sample on the RAM disk. Each
records, with every timed pass, the 1-minute load, the processes above 20%
CPU and other `tintype` processes. A pass during the research session's
GPU read, or with another `tintype` process running, is marked as shared.

1. **`serve`:** [serve_speed.py](serve_speed.py), five rounds, as in
   `../serve-beside/PROTOCOL.md`. Both servers start once and score the
   warm-up first. In each round, every server scores the 2,000 photos over
   HTTP in requests of 16. The order alternates. Timed: first request to
   last answer.
2. **`batch`:** `../e2e-overlap/speed_ab.py`, five rounds. Each binary runs
   the warm-up, then the timed 2,000, alternating the order.

**Reported for each:** every round; the medians, and D's ratio to B. D is
faster only if each of its rounds beats each of B's; otherwise "not
separated". The comparison within each round is reported too.

**One timed run** of each, reported as it is.
