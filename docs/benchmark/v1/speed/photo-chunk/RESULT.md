# Result: photos in 4,096-input chunks are faster, with every row unchanged

The protocol is [PROTOCOL.md](PROTOCOL.md), registered with the change in
2d3f9887 before the timed run. This was the only registered timed run.

## Parity gates: all passed

1. **Photo rows against the registered build.** A (0947f3a6) and C (this
   branch, binary fbfe8457) were compared on 40,794 photos from openfake,
   openfake2, `dfdc-frames` and `dfdc-frames-real`. They were staged in
   300-file chunks.
   - Both binaries wrote 40,794 rows.
   - **0 rows differed in any field.**
   - No copy failed or was retried. The log is
     [parity_photos.txt](parity_photos.txt).
2. **The mixed folder against origin/main.** The folder has 1,300 files:
   1,000 photos and 300 clips, interleaved. B (273e03f3) and C each
   scored all 1,300, with no errors.
   - Their standard output was **byte-identical**: every row, field and
     order.
   - B took 19.0 s and C took 16.0 s.
3. **Unit tests:** `cargo test --release` passed 313 tests, with 12
   ignored as usual.

## Speed

The machine was the Apple M6, using the TPB v1 speed sample on a RAM
disk. Each figure is milliseconds per image, timed from process start to
exit over 2,000 files, as in the main speed protocol. The run went from
14:13:18 to 14:14:40 on 2026-11-26.
- **Binaries:** A 0947f3a6, B 1545e539, C fbfe8457.
- **Machine state:** the other sessions held their work on the Mac. The
  1-minute load rose from 3.2 to 6.9 during the run as the detectors'
  own threads ran.
- **Busy processes:** none above 20% CPU before any run, except
  `sysmond` at 25.8% before BSF's first round
  ([speed.tsv](speed.tsv)).

| Detector | Round 1 | Round 2 | Round 3 |
|---|---|---|---|
| Tintype 0947f3a6 (registered, A) | 2.573 | 2.619 | 2.644 |
| Tintype origin/main 273e03f3 (B) | 2.586 | 2.610 | 2.644 |
| **Tintype photo chunk 4096 (C)** | **2.250** | **2.264** | **2.249** |
| BSF (AIGIBench release) | 2.421 | 2.440 | 2.455 |

Ranked by the registered rule, where one detector is faster than another
only if each of its rounds beats each of the other's:
- **C is faster than A.** C's slowest round, 2.264, beats A's fastest,
  2.573.
- **C is faster than B.** 2.264 beats 2.586.
- **C is faster than BSF.** 2.264 beats 2.421.

C is about 0.35 ms per image faster than origin/main, roughly 13%, and
about 0.18 ms faster than BSF, roughly 7%.

In the registered wording, C is the **fastest of the detectors timed on
this sample and machine**.

BSF's figure is its wall time over all 2,000 files, as in the main speed
protocol, although its loader scored 1,984 of them.

## Scope

- **Batches the result covers.** The 2,000-file sample fits in one chunk
  of C. A batch of more than 4,096 inputs closes a chunk every 4,096
  inputs, where origin/main closes one every 256. The fill and drain cost
  per chunk stays the same, so it is spread over 16 times as many files.
- **When rows appear.** Photo rows now arrive once per 4,096 files, about
  9 s at this rate, instead of once per 256. A folder of videos still
  writes its rows every 256 clips.
- **Precision and recall are unchanged.** Every row is the same, so this
  says nothing new about accuracy.
