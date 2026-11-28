# Result: running `serve`'s second opinion beside early exit does not make it faster

The protocol is [PROTOCOL.md](PROTOCOL.md), registered in f02b5ffc. The
timed run below was its only one.

## Gate 1 passed

[parity_sample.txt](parity_sample.txt) covers the speed sample, 2,000 files,
with B (C3, 43525b32) and C (b36cabb9):
- **batch:** 0 rows differed, and the standard output was byte-identical;
- **serve:** 0 of the 2,000 results differed over HTTP;
- **serve against batch:** every result agreed with its binary's `batch` row.

Gates 2 to 4 were not run, because the change is not kept (see below).

## Speed: not separated

The Apple M6, with the GPU free and no other `tintype` process running. The
run went from 04:18:03 to 04:20:50 on 2026-11-28, and the 1-minute load was
between 3.0 and 4.5 ([speed.tsv](speed.tsv)).

Milliseconds per image for 2,000 photos over HTTP, 16 to a request:

| Round | Order | B | C |
|---|---|---|---|
| 1 | B, C | 8.003 | 7.961 |
| 2 | C, B | 8.041 | 7.964 |
| 3 | B, C | 8.129 | 8.168 |
| 4 | C, B | 8.201 | 8.219 |
| 5 | B, C | 8.263 | 8.395 |
| **Median** | | **8.129** | **8.168** |

C's ratio to B is 1.005, and the two are **not separated**: C was faster in
two rounds and slower in three.

## Why

A request holds at most 16 photos. `Server-Timing` on six requests of the
sample showed where the time goes:
- On the small classic sets, the network took 58–68 ms of a 71–75 ms request.
- The first stage's views go through one stage after another. Each stage is
  a 16-view run of a few views, padded, waiting for the stage before it.
- Reading the few photos inside the band again, which is what C overlaps,
  is a small part of a request.

So C is not kept: PR #60 stays as it was, and this branch is not opened as
a PR. Filling the padded slots of those part-empty runs with the same
photos' later views is the change that addresses where the time goes.
