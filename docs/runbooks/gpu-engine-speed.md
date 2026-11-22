# GPU engine speed on Apple silicon

What limits the photo engine on a Mac, what was changed, and what was
measured and set aside. Measured on an Apple M6 under macOS 27.2.

## Where the time went

There were three workloads:
- **One photo** (`tintype detect photo.jpg`): a batch of six views.
- **214 small photos** (median 0.8 MP, at most 2 MP): batches of 64.
- **44 large camera JPEGs** (over 8 MP, up to 48 MP).

`TINTYPE_STATS=1` now also counts `network_runs`.

**One photo** spent most of its 0.17 s before the first view was scored:
- The weights took 13–23 ms to read.
- Building the MPSGraph took about 48 ms.
- The first run took 45–60 ms more than later runs, which is MPSGraph
  compiling the graph and Metal creating its pipelines.

**The small photos** were GPU-bound:
- There were 8 runs of 64 slots for 292 views, so 43% of the slots were
  padding.
- Every run paid for a full batch of 64, about 45 ms.

**The large photos** are bound by decoding and resizing on the CPU: about
3.5 s of decoding and 1.7 s of preparing views, spread across the cores,
against about 0.1 s in the network. Nothing here changes that.

## What changed

**Compiled networks.**
- The GPU engine compiles the network ahead of time, through
  `MPSGraph.compile` (an `MPSGraphExecutable`), for the job's batch size.
- When that batch is larger, it also compiles batches of 16 and 32 views.
  All the sizes are copies of the network in one graph that share one
  set of weight constants.
- A run of fewer views goes through the smallest size that holds them.

**Same logits, checked per machine.**
- The GPU's kernel choice depends on the batch shape. On the M6, batches
  of nine views or more give logits bit-identical to a batch of 64. Batches
  of eight or fewer differ by up to 5×10⁻⁴.
- A smaller size is therefore kept only if a probe run, when it is
  compiled, shows bit-identical logits to the full batch on this machine.
- A view's logit was already the same in every slot of a batch and beside
  any neighbours, and that still holds.
- So every score is unchanged. On the two photo sets and 16 single photos,
  `--json` output was byte-identical to the previous release, with and
  without the cache.

**A cache of compiled networks.**
- The compiled networks are written as MPSGraph packages to
  `~/Library/Caches/tintype/gpu` (`src/gpu_cache.rs`).
- A package holds the compiled program and the weights in half precision,
  about 41 MB.
- A later run loads it in about 2 ms, with no weights file read and no graph
  built or compiled.
- The entry's name hashes everything the compiled network depends on: this
  build of tintype, the macOS build, the GPU, the weights file's path, size
  and modification time, the architecture, the precision and the batch.
- Entries are written under a temporary name and renamed into place.
  Beyond eight packages, the least recently used entries are removed. (It
  was five at first, which let two networks' many-photo entries, three
  packages each, evict each other on every run.)
- `TINTYPE_GPU_CACHE=0` turns the cache off; any other value is the
  directory to use.

**Overlap.**
- In the staged early-exit loop, a network thread runs one batch while the
  main thread hands out the previous batch's logits, prepares those photos'
  next views and gathers the next batch.
- The CPU engine keeps its previous order, because there a logit can depend
  on what it is run with.

**One input copy.**
- Each executable keeps a Metal buffer in shared memory for its input, and
  a run copies the views into it once.
- Before, the views went through an `NSData` copy and then a GPU-side copy.
- That made runs about 4% faster: a batch of 64 went from 44.8 to 43.0 ms.

## Results

Wall time of `tintype detect` in seconds, five alternating runs each, warm
file cache:

| Workload | Before | After, cached | After, empty cache |
|---|---|---|---|
| One photo | 0.17 | 0.06 | 0.17 |
| 214 small photos | 0.61 | 0.38 | 0.63–0.64 |
| 44 large photos | 0.77 | 0.60 | — |

Notes on the table:
- **Small photos, broken down:** batches sized to what is waiting removed
  most of the padding (0.61 → 0.41 s), and overlap took off another 0.03 s.
- **Large photos:** the gain is from start-up only.
- **Empty cache:** a run with an empty cache compiles and stores the
  networks. For a single photo it costs the same as before. For the small
  set it costs about 0.03 s more, once.
- **Shader cache:** the first run of a new build can take 0.2–0.3 s longer
  while macOS fills its own Metal shader cache.

## What was measured and set aside

**MLX** (0.32.4) was measured with the same network, written in MLX and
checked against a float64 reference. It is slower than this engine on the
M6:
- **Speed:** the best variant (bfloat16, `mx.compile`, fused attention) ran
  about 800–880 images/s, against about 1,300–1,400 for MPSGraph in half
  precision. At batch 32 that was 36–43 ms per batch, against 23 ms.
- **Cause:** the same chain of matrix products took 21.1 ms in MLX and
  10.3 ms in MPSGraph, about 12 against 24 TFLOPS. The products with a
  384-wide inner dimension are where MLX falls behind.
- **Fused attention:** MLX's fused attention kernel is real, about 0.17
  against 0.85 ms per block at batch 32, but attention is a small share of
  the time.
- **Conclusion:** there is no headroom here from another framework. A custom
  Metal kernel would have to beat MPSGraph's own matrix products.

**Core ML** (coremltools 9.0, model built with the MIL builder):
- **On the GPU** it matches MPSGraph: 11.2 ms at batch 16 and 1.5 ms at
  batch 1.
- **On the Neural Engine** it is only competitive at batch 1 (1.44 ms). At
  batch 16 it takes 13.7–16.5 ms.
- **Accuracy on the Neural Engine:** its probe logit was 0.03 from the
  float64 reference. MPSGraph in half precision was 0.005 away.
- **First load:** a compiled model costs 2–6 s the first time it is loaded
  on the Neural Engine. macOS caches the result, after which it loads in
  about 24 ms.
- **Conclusion:** it would need its accuracy checked on real data before
  use, and it brings no throughput gain.

Not tried:
- private Neural Engine interfaces;
- hand-written SME assembly;
- running the GPU and the Neural Engine at once.

## Second round: large photos and the hosted server

Measured on 2026-11-17 and 2026-11-18, same machine. Scores are unchanged
again: `--json` output and `tintype score` output were byte-identical to
the build before on the GPU and CPU engines.

**Texture ranking.** Choosing the textured crops reads the whole photo:
a gradient sum over every other pixel, kept as an integral image.
- For photos of about 4 MP and more it now runs on all cores: rows side by
  side, then a blocked prefix sum down the rows.
- The arithmetic is integer, so the sums are the same bit for bit; a unit
  test compares them with the one-sample-at-a-time version.
- On the 44 large JPEGs (22.7 MP on average) it went from 11.9 to 3.0 ms a
  photo.

**JPEG decoding was not sped up.** It is the largest cost of a large photo:
about 45 ms for 22.7 MP, on one thread, three quarters of it Huffman
decoding, which the format makes serial without restart markers (none of
the 44 had them).
- Apple's ImageIO decoded the same files in 39 ms against 58 ms in a
  like-for-like test, 1.5 times as fast.
- It gives slightly different pixels, though, which would move scores and
  need the accuracy checked again, so it was not adopted.
- Resizing on several threads (`fast_image_resize`'s `rayon` feature) saved
  about 1.5 ms a photo and added some 400 lines of dependencies to the
  lockfile; not adopted either.

**GPU warm-up.** A detector with one small batch size, as for one or two
photos, now makes its first run on zeros on a thread of its own while the
photo is decoded, and the first real run waits for it. What a first run
sets up (MPSGraph specialising the program, Metal creating pipelines) then
overlaps the decode instead of following it.

**`Server-Timing`.** Every scored HTTP response says where the server's
time went: upload, queue, decode, views, network and total, in
milliseconds ([API](../API.md#detect-one-file)).

**Threads on Cloud Run.** That header showed the hosted server spending
1.3–1.5 s in the network on the site's sample photo (five views), far
more than two processors should need.
- A second-generation Cloud Run instance with 2 vCPUs shows the process 3
  or 4 processors, and the thread pool made a thread for each.
- With `RAYON_NUM_THREADS=2` the same photo took 0.55–0.63 s on the server
  instead of 1.32–1.55 s, and a 22 MP JPEG 0.26–0.28 s instead of
  0.53–0.69 s.
- The server now logs its thread count at start, and both hosted services
  set the variable ([Hosting](../HOSTING.md#speed-on-cloud-run)).

**Cold starts.** The website now wakes the server when the photo picker
opens, through `GET /health`, so an instance that had stopped is starting
while the visitor chooses a photo. Cloud Run answers `/healthz` itself, so
that path never reached the server.

**Results of the second round.** Wall time of `tintype detect` in
seconds, six alternating runs, each build with its own warm cache:

| Workload | Before | After |
|---|---|---|
| One small photo | 0.06 | 0.06 |
| One 22 MP photo | 0.11–0.12 | 0.09 |
| 214 small photos | 0.39–0.42 | 0.39–0.41 |
| 44 large photos | 0.66–0.70 | 0.64–0.67 |

On the 2,096-photo sample of blind batch 7, `tintype score` ran at 563–593
photos a second on the GPU and 75–78 on the CPU, level with the build
before this round (570–592 and 73–74). The first timing of that sample,
before either round, was 530–563 and 77–78
([benchmark](../BENCHMARK.md), [raw log](../results/confirm7_speed_2026-11-18.log)).
Video was level on 100 stand-in clips; the 147 clips of the published video
timing are no longer on disk, so that figure was not measured again.

## Checking it

- `cargo test --release --test gpu_cache -- --ignored`. On both GPU
  precisions, at batches of 6 and 64, it checks that:
  - logits are the same from a fresh build and from the cache;
  - a run of 1, half a batch plus one, a batch less one, or more than a
    batch gives each view the logit it gets in a full batch;
  - `TINTYPE_GPU_CACHE=0` writes nothing.
- `cargo test --release --test view_reuse -- --ignored` still checks a
  view's logit in every slot.
