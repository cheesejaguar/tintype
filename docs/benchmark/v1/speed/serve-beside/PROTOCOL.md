# Protocol: `serve` runs the photo second opinion beside early exit (registered before any timed run)

## The change

Stacked on `perf/e2e-speed` (PR #60). Commit b36cabb9, called C below,
changes how `serve` handles a request's photos on the GPU engine.

Before, it scored them with early exit, then read the photos inside the
second-opinion band again and ran the Community Forensics network on them
after the first stage was done. Now it uses `batch`'s pipeline from PR #60:
- a photo inside the band goes to the second network as soon as its own
  first stage is done;
- a worker of its own decodes the photo again while the first stage goes on;
- the two networks take turns on the GPU.

`Detector::score_staged_with_second` is the shared pipeline, and `batch`
now goes through it too. The CPU engines and turbo mode keep the old order.

Which photos are asked, every view's pixels and every logit are unchanged,
for the reasons in `../e2e-overlap/PROTOCOL.md`.

The baseline here, B, is PR #60's code, C3 at 43525b32 (SHA-256 80637fac…),
because this change is reviewed on top of it. C's binary is 9fbdf91c….

**Disclosed before this registration:** the parity script's smoke run scored
42 files of the speed sample with both. Every batch row, every served result
and their agreement were identical. The request time was 0.7 s for B and
0.6 s for C. No other timing was done.

## Parity gates (all must hold)

[parity_serve.py](parity_serve.py) runs three checks on each set of files:
- each binary's `batch --json`: every field of every row of C equals B's,
  and the standard output is byte-identical;
- each binary's `serve`, over HTTP to `/v1/batch`, 16 files to a request:
  every field of every result of C equals B's;
- each server against its own binary's `batch` rows: the same kind, profile,
  score, verdict, views, logit, threshold_logit and second_opinion, and the
  same files unscored.

The sets are:
1. **the speed sample**, 2,000 files (`folder` mode on the RAM disk);
2. **all 24,961 batch-9 open files**, staged from `data/fresh10` in chunks of
   750 with every SHA-256 checked against `open.tsv.gz`;
3. **the mixed folder** of `../photo-chunk/`: 1,000 photos and 300 clips;
4. **tests:** `cargo test --release` passes, and so do the ignored GPU tests
   `view_reuse`, `second_beside` and server's
   `the_second_opinion_beside_early_exit_answers_as_after_it`.

**Order.** The GPU is free now and will be shared later tonight. So the timed
run below comes after gate 1 and before gates 2 and 3. Its result counts only
if every gate passes, and RESULT.md says so either way.

## Speed

[serve_speed.py](serve_speed.py) times both servers on the Apple M6:
- **Set-up:** both servers are started once and kept running, each with its
  own GPU cache on the RAM disk. Each scores the 100-file warm-up first.
- **Rounds:** five. In each, every server scores the 2,000-file speed sample
  over HTTP in requests of 16, in order. The order alternates: B-C, C-B, and
  so on.
- **Timed:** the time from the first request to the last answer. This
  includes the HTTP upload of every file, which is the same for both.
- **Recorded with each pass:** the 1-minute load, the processes above 20% CPU,
  and the number of other `tintype` processes. A pass that ran while another
  `tintype` process, or a known GPU job of the research session, was running
  is marked as shared. Every pass's results must equal the first pass's.
- **Reported:** every round; the median per server and C's ratio to B. C is
  faster only if each of its rounds beats each of B's; otherwise "not
  separated". The comparison within each round is reported too.
- **One timed run**, reported as it is.
