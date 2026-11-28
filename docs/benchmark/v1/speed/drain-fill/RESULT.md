# Result: filling the padding of part-full runs makes `serve` about 7% faster, with every row unchanged

The protocol is [PROTOCOL.md](PROTOCOL.md), registered in dff49b39 before
either timed run. Each timed run was its only one.

Binaries:
- B, PR #60's code, C3 at 43525b32 (SHA-256 80637fac…);
- D, a2a33be6 (0ea6c99a…).

In the `serve` files, the scripts call D "C".

## Parity gates: all passed

1. **The speed sample**, 2,000 files ([parity_sample.txt](parity_sample.txt)):
   - **batch:** 0 rows differed, and the standard output was byte-identical;
   - **serve:** 0 results differed over HTTP;
   - **serve against batch:** every result agreed with its binary's `batch`
     row.
2. **All 24,961 batch-9 open files**, staged from `data/fresh10` in 50 chunks
   of 500, every SHA-256 matching the manifest
   ([parity_all.txt](parity_all.txt)):
   - **batch:** B and D each wrote 24,961 rows. **0 rows differed in any
     field**, and every chunk's standard output was byte-identical;
   - **serve:** **0 of the 24,961 results differed** over HTTP;
   - **serve against batch:** every result agreed with its binary's `batch`
     row except two, the same two for both binaries. They are `sofake-ood`
     photos of 92 and 53 megapixels: `batch` scores them, and `serve`'s
     50-megapixel decode limit refuses them, as it did before.
   - No copy was retried.
3. **The mixed folder**, 1,000 photos and 300 clips
   ([parity_mixed.txt](parity_mixed.txt)):
   - **batch:** the standard output was **byte-identical**;
   - **serve:** **0 of the 1,300 results differed**;
   - **serve against batch:** all 1,000 photos agreed. The 300 clips differ
     from `batch`'s rows, the same way for both binaries: `serve` reads only
     MP4 and MOV clips and samples them its own way.
4. **Tests:** `cargo test --release` passed 313 tests, with 14 ignored. The
   ignored GPU tests `view_reuse`, `second_beside` and `drain_fill` passed.

The timed runs came after gate 1 and before gates 2 and 3, as registered.

## Speed

The Apple M6, with the speed sample on a RAM disk. **The GPU was free.**
The research session's two GPU reads ran from 04:27:28 to 04:28:01 and from
about 04:29:20 to 04:29:42, before these runs began. Nothing of its ran on
the GPU from 04:30 on.

The CPU was not quiet:
- the research session's network-bound jobs ran throughout (OpenRouter
  generation and the blind-2 fetch), as during `../e2e-overlap/RESULT-3.md`;
- `FPCKService`, a system file-provider check, used about 90% of one core.

No other `tintype` process ran during any timed pass.

### `serve`: D is faster

04:32:31 to 04:35:10 on 2026-11-28, with the 1-minute load between 4.4 and
7.5 ([speed-serve.tsv](speed-serve.tsv)). The times are milliseconds per
image for 2,000 photos over HTTP, 16 to a request:

| Round | Order | B | D | D against B |
|---|---|---|---|---|
| 1 | B, D | 8.019 | 7.302 | −8.9% |
| 2 | D, B | 7.993 | 7.233 | −9.5% |
| 3 | B, D | 8.010 | 7.550 | −5.7% |
| 4 | D, B | 8.124 | 7.429 | −8.6% |
| 5 | B, D | 8.323 | 7.477 | −10.2% |
| **Median** | | **8.019** | **7.429** | |
| **Images per second** | | 125 | 135 | |

D's ratio to B is 0.926. **D is faster than B** by the registered rule: D's
slowest round, 7.550, beats B's fastest, 7.993. The client sent one request
at a time, so the average request's latency fell by as much.

### `batch`: not separated

04:35:10 to 04:36:01, with the load rising from 4.2 to 10.8
([speed-batch.tsv](speed-batch.tsv)):

| Round | Order | B | D | D against B |
|---|---|---|---|---|
| 1 | B, D | 2.018 | 2.097 | +3.9% |
| 2 | D, B | 2.217 | 2.180 | −1.7% |
| 3 | B, D | 2.310 | 2.258 | −2.3% |
| 4 | D, B | 2.262 | 2.230 | −1.4% |
| 5 | B, D | 2.260 | 2.204 | −2.5% |
| **Median** | | **2.260** | **2.204** | |

D's ratio is 0.975 by the medians, but the two are **not separated**: D was
faster in four rounds and slower in the first. That is about what was
expected. In `batch` only a chunk's last few runs are part full. On this
sample, D scored 3,705 views in 85 network runs, against 3,699 in 87
(`TINTYPE_STATS`, outside the timed run).

## Scope

- What each photo looks at, and every logit, are unchanged, so precision
  and recall are unchanged. D scores a few more views, in slots the GPU
  computed as padding before.
- The change is in the early-exit pipeline, so it applies to `batch`,
  `detect`, `serve` and clips with early exit, on the GPU engine. The CPU
  engine is unchanged.
