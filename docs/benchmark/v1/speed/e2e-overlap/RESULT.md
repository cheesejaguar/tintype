# Result: keeping the GPU busy is about 6% faster on a shared machine, with every row unchanged

The protocol is [PROTOCOL.md](PROTOCOL.md), registered in 99ac5493 before
the timed run. This was the only timed run under it.

Binaries, all built with rustc 1.99.1:
- B, origin/main 456d5ab0 (SHA-256 a3e4d4ba…);
- C1, 7ef4f84c (055c32ac…);
- C2, 27aba0fb (2e077f09…).

## Parity gates: all passed

1. **The speed sample.** B, C1 and C2 each wrote 2,000 rows. **0 rows
   differed in any field**, and the standard output was byte-identical
   ([parity_sample.txt](parity_sample.txt)).
2. **All batch-9 open files.** All 24,961 files of `open.tsv.gz` were staged
   from `data/fresh10` in 17 chunks, and every SHA-256 matched the manifest.
   - B, C1 and C2 each wrote 24,961 rows.
   - **0 rows differed in any field**, C1 against B and C2 against B.
   - Every chunk's standard output was byte-identical.
   - No copy was retried. The log is [parity_all.txt](parity_all.txt).
3. **The mixed folder** of 1,000 photos and 300 clips. The standard output
   of C1 and C2 was **byte-identical** to B's, 1,300 rows each with no
   errors ([parity_mixed.txt](parity_mixed.txt)).
   - C2's first attempt stopped before writing a row: the RAM disk was full,
     and writing its GPU cache failed (`No space left on device`). After
     the old cache directories were cleared, all three binaries were run
     again, and that run is the result.
4. **Tests.** `cargo test --release` passed 313 tests at C1 and at C2,
   with 12 ignored at C1 and 13 at C2, which adds the GPU test
   `second_beside`. The ignored GPU tests `view_reuse` (both commits) and
   `second_beside` (C2) passed.

## Speed

The machine was the Apple M6, with the TPB v1 speed sample on a RAM disk.
**It was not idle:** another session was training and scoring on the same
GPU. The run went from 03:26:09 to 03:27:26 on 2026-11-28, and the
1-minute load rose from 3.9 to 10.4 over it. No other `tintype`, `cargo` or
`rustc` process ran during any timed run ([speed.tsv](speed.tsv)).

Milliseconds per image, from process start to exit over 2,000 files:

| Round | Order | B | C1 | C2 | Load at C2's run |
|---|---|---|---|---|---|
| 1 | B, C1, C2 | 2.211 | 2.123 | 2.033 | 3.86 |
| 2 | C1, C2, B | 2.360 | 2.192 | 2.169 | 4.89 |
| 3 | C2, B, C1 | 2.380 | 2.369 | 2.221 | 5.03 |
| 4 | B, C1, C2 | 2.374 | 2.298 | 2.270 | 8.69 |
| 5 | C1, C2, B | 2.399 | 2.285 | 2.288 | 9.97 |
| **Median** | | **2.374** | **2.285** | **2.221** | |
| **Ratio to B** | | 1.000 | **0.963** | **0.936** | |

- **Under the registered rule, C2 and B are not separated**: C2's slowest
  round, 2.288, is slower than B's fastest, 2.211. The load rose through the
  run, and B's fastest round was the first run of all, at the lowest load.
- **Round by round, C2 beat B in all five rounds**, by 8.0%, 8.1%, 6.7%,
  4.4% and 4.7%. The gap shrank as the load rose. With the GPU shared, the
  other session's work fills some of the gaps B leaves, so removing them
  gains less than it would on an idle GPU.
- C1 alone was faster than B in all five rounds too, by 0.5% to 7.1%.

**This is not an idle-machine figure.** The speed for the website needs a
re-time under `../PROTOCOL.md` on an idle machine.

## What changed in the profile

With the same stage marks as the protocol's profile, C2's GPU is idle for
about 0.01 s of a 4.0 s run, against 0.6–0.9 s for B, and the work on it is
the same 63 runs. The run is now as long as the GPU work it holds, plus
about 0.06 s before the first run and 0.03 s after the last.

Peak resident memory is about 0.4 GB higher, 1.9–2.1 GB against 1.5–1.7 GB:
both networks are loaded from the start, and the second stage keeps up to
two batches of prepared photos.

## Scope

- Which views are scored, their pixels, which photos are asked, and every
  logit are the same, so precision and recall are unchanged.
- The second stage beside the first applies to `batch`, and to `detect`
  with more than 10 photos, on the GPU engine. The CPU engines keep the old
  order.
