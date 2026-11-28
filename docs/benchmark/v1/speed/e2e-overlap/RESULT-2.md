# Result 2: with the second network in 16-view runs, about 8% faster than origin/main on the shared machine, every row unchanged

The protocol is [PROTOCOL-2.md](PROTOCOL-2.md), registered in bcc3f9fa
before the timed run. This was the only timed run under it.

Binaries, all built with rustc 1.99.1: B, origin/main 456d5ab0 (SHA-256
a3e4d4ba…); C2, 27aba0fb (2e077f09…); C3, 43525b32 (80637fac…).

## Parity gates for C3: all passed

1. **The speed sample:** C3 wrote 2,000 rows, **0 differing** from B's in
   any field, and the standard output was byte-identical
   ([parity_sample-2.txt](parity_sample-2.txt)).
2. **All 24,961 batch-9 open files**, staged in 17 chunks with every SHA-256
   matching the manifest: B and C3 each wrote 24,961 rows, **0 differing in
   any field**, and every chunk's standard output was byte-identical. No
   copy was retried ([parity_all-2.txt](parity_all-2.txt)).
3. **The mixed folder** of 1,000 photos and 300 clips: C3's standard output
   was **byte-identical** to B's, 1,300 rows with no errors
   ([parity_mixed-2.txt](parity_mixed-2.txt)).
4. **Tests:** `cargo test --release` passed 313 tests, with 13 ignored; the
   ignored GPU tests `view_reuse` and `second_beside` passed.

## Speed

The Apple M6, with the TPB v1 speed sample on a RAM disk. **It was not
idle:** another session was training and scoring on the same GPU, and
`diskimagesiod` (the data share's disk image) was busy before several runs.
The run went from 03:50:25 to 03:52:09 on 2026-11-28, with the 1-minute load
between 4.3 and 7.3. No other `tintype`, `cargo` or `rustc` process ran
during any timed run ([speed-2.tsv](speed-2.tsv)).

Milliseconds per image, from process start to exit over 2,000 files:

| Round | Order | B | C2 | C3 | C3 against B | Load (B, C2, C3) |
|---|---|---|---|---|---|---|
| 1 | B, C2, C3 | 2.300 | 2.044 | 2.027 | −11.9% | 7.27, 6.85, 6.62 |
| 2 | C2, C3, B | 2.366 | 2.126 | 2.111 | −10.8% | 5.44, 6.25, 5.83 |
| 3 | C3, B, C2 | 2.389 | 2.214 | 2.194 | −8.2% | 4.76, 4.46, 5.09 |
| 4 | B, C2, C3 | 2.495 | 2.226 | 2.184 | −12.5% | 4.42, 4.55, 4.26 |
| 5 | C2, C3, B | 2.370 | 2.197 | 2.171 | −8.4% | 6.12, 5.52, 5.08 |
| 6 | C3, B, C2 | 2.398 | 2.203 | 2.219 | −7.5% | 6.59, 6.23, 6.91 |
| 7 | B, C2, C3 | 2.357 | 2.172 | 2.110 | −10.5% | 5.89, 5.66, 5.28 |
| **Median** | | **2.370** | **2.197** | **2.171** | | |
| **Images per second** | | 422 | 455 | 461 | | |
| **Ratio to B** | | 1.000 | **0.927** | **0.916** | | |

Ranked by the registered rule, where one binary is faster than another only
if each of its rounds beats each of the other's:
- **C3 is faster than B.** C3's slowest round, 2.219, beats B's fastest,
  2.300.
- **C2 is faster than B** in this run too: 2.226 beats 2.300.
- **C3 and C2 are not separated.** Round by round C3 was faster in six of
  the seven rounds, by 0.7% to 2.9%, and slower in one, by 0.7%. Its median
  is 1.2% lower.

C3 takes about 8.4% less time per image than origin/main here, and C2 about
7.3% less. In the first registered run, at a load that rose to 10.4, C2 was
6.4% faster by its median ([RESULT.md](RESULT.md)). The gain is smaller when
the other session uses more of the GPU, which fills some of the gaps that
origin/main leaves.

**This is not an idle-machine figure.** The speed for the website needs a
re-time under `../PROTOCOL.md` on an idle machine.

## Scope

- Which views are scored, their pixels, which photos are asked, and every
  logit are the same, so precision and recall are unchanged.
- C3 changes only how the second network's batches are run on the GPU. It
  applies wherever that network runs on the GPU engine: `batch`, `detect`,
  `serve`, `demo` and `evaluate`.
- Peak resident memory of C3 on the speed sample was 1.95 and 2.00 GB in
  two runs, against 1.62 and 1.68 GB for origin/main, measured with
  `/usr/bin/time -l` after the timed run. User CPU time was the same, about
  15.4 s.
