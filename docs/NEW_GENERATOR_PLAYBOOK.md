# New generator playbook

This is the standard process for when a new image or video generator comes
out. Every session working on Tintype follows it, and every run is logged
in [generators/LOG.md](generators/LOG.md). The goal is a model that keeps
pace with new generators without losing what it already detects.

There are five steps:

1. Track and decide.
2. Generate or collect images, with matched real photos.
3. Split into a training portion and a blind set, fixed up front.
4. Retrain, then gate on the blind set and the existing benchmarks.
5. Log the result.

Each run lives in its own folder, `docs/generators/<slug>/`, for example
`docs/generators/imagen-3/`. The folder holds:
- `PROTOCOL.md`: committed before any image is generated or scored;
- the caption and generation templates;
- the frozen blind manifest;
- `RESULT.md`.

## Rules that apply to every step

1. **Register first.** Commit the plan before any image is generated,
   downloaded or scored: sources, prompt rule, split rule, sample sizes,
   gates and metrics. Record a change made after that as a dated amendment
   that says what was already seen.
2. **Blind sets stay blind.** A generator's blind set is never used for
   training, calibration, threshold or band fitting, early-exit schedules,
   checkpoint selection or prompt tuning. It is read only by a registered
   scoring run. The same holds for:
   - every other blind set listed in the repository's `AGENTS.md`;
   - the benchmark's blind split ([BENCHMARK.md](BENCHMARK.md));
   - the public held-out judges, NTIRE 2026 public test, WildAIGC V3, MIRAGE
     and AIGenImages2026: nothing is fitted on their per-file results;
   - photo batch 9, the bar set behind the website. Nobody trains on it.
3. **No overlap with evaluation data.** Prompts, source photos and
   generated files must not come from any evaluation set. Check every new
   file by SHA-256 against every training and evaluation manifest you are
   allowed to read. For a blind set you may not read, check its builder
   code instead, never its files.
4. **Data location.** Images go to the data share
   (`/Volumes/veracity-data`) or the GPU machine's data drives, not the
   Mac's internal disk.
5. **Licences and terms.** Record the licence or terms of service of every
   source and API: whether outputs may be used to train a detector and
   whether they may be redistributed. Where they may not be redistributed,
   ship the recipe (prompts, source ids, parameters), not the files.
6. **Spend.** Run a pilot first, record its cost per image and its quality,
   and scale up only after that review. Record the total spend in the log.

## 1. Track and decide

### Where to look

Check monthly, and whenever a major release is announced.

- **The Bifrost gateway's model list** (`GET /v1/models`):
  `scripts/generators/bifrost_models.py` prints the image and video models
  the gateway's key can reach, and what changed since the last check.
- **Provider release notes.** For images: Google (Imagen, Gemini image /
  Nano Banana), OpenAI, Black Forest Labs, ByteDance (Seedream), Alibaba
  (Qwen-Image, Wan), Tencent (Hunyuan), Midjourney, Ideogram and Recraft.
  For video: Google Veo, OpenAI Sora, Kling, Runway, Luma, Pika, Hailuo and
  Wan.
- **Leaderboards:** the LMArena text-to-image and video arenas, and
  Artificial Analysis.
- **New open weights** on Hugging Face (trending and new text-to-image or
  text-to-video models).
- **Datasets that add generators:** OpenFake supplements, So-Fake,
  frontier-synthetic-images and AIGIBench updates. Use their training
  splits only.

### How to decide

For each candidate, collect a probe of 100 to 200 images through the
route in step 2. Score the probe with the shipped default at its shipped
cutoff. Probe images are never placed in the blind set; once the decision
is made they join the training portion.

Add the generator, as `train` status, when any of these holds:
- recall on the probe is below 90%;
- real photos flagged on the matched reals are above the default's 1.5%
  target;
- the shipped weights have fewer than 1,000 images from it (count by
  generator in the training stores' `manifest.tsv`; see the Imagen 3 run
  for the command);
- it ranks in the top ten of a leaderboard above, or is the default model
  of a major consumer product.

Track it as `eval-only` (blind set, no training) when its terms forbid
training a detector on its outputs. Record the decision in the log even
when the generator is not added.

## 2. Generate or collect, with matched real photos

### Real photos first

Each generated image is the twin of a real photo.
1. Pick real photos from the run's **real pool**.
   - The pool is a source in no training manifest and no evaluation set,
     recorded in the protocol.
   - The default pool is Wikimedia Commons *Quality images*: camera photos
     under CC0, CC BY or CC BY-SA, with author and licence kept per file.
2. Caption each photo with a fixed vision-language model and a fixed
   instruction, through Bifrost. The prompt describes the scene as a
   photograph. It leaves out named people, brands and text, and it is not
   edited by hand afterwards.
3. Generate one image per caption with the new generator. Use the
   supported aspect ratio closest to the real photo's.

Content then matches across classes. A detector cannot win by learning
that generated images show different things.

### What to record per image

- source photo id, URL, licence and SHA-256;
- the caption, and the model and instruction that wrote it;
- the generator's exact model id as the API reports it;
- API parameters, seed if given, date, cost and response format;
- the output file's SHA-256.

The record goes in a `meta.jsonl` beside the store's `manifest.tsv`. The
manifest uses the training-store format: `path`, `label`, `generator`,
`dataset`.

### Confounds to control

- **Format.** Keep files exactly as delivered. Report the blind set
  twice: as delivered, and with both classes re-encoded to JPEG q90 at the
  same size. CompEval showed real PNG and WebP photos are flagged several
  times as often as real JPEGs.
- **Watermarks and metadata.** Record whether outputs carry SynthID, C2PA
  or other marks. Tintype sees pixels only, but a pixel watermark such as
  SynthID can become a shortcut. That is one more reason the blind set must
  include re-encoded copies.
- **Size.** Record each class's resolution distribution, and resize real
  photos to a comparable long side if the generator's outputs are
  fixed-size.

### Routes

1. **Bifrost.** Use the gateway on the local network, with the virtual key
   in `~/.bifrost_env` (`x-bf-vk` header): `/v1/images/generations` for
   images. Never print or commit the key. A `403 model_blocked` means the
   key does not allow that model; ask the user to enable it rather than
   working around the gateway.
2. **Other provider APIs**, only with a key the user provides for that
   purpose.
3. **Public datasets** that publish the generator's images with a licence
   that allows training. Use their training splits only, and never files
   that also appear in an evaluation set.

### Pilot, then scale

The pilot is 50 images, generated only for source photos assigned to the
training portion. Look at them, and check the cost per image and the
failure and refusal rate. Then scale to the registered sample size.

## 3. Split up front

The split is fixed before anything is generated.

- **Unit.** The unit is the source real photo. A photo and its generated
  twins always land in the same portion. Near-duplicate photos (same
  photographer and scene, or perceptual-hash distance ≤ 6) are grouped
  first, and a group lands in one portion.
- **Rule.** A unit goes to the blind set when
  `int(sha256("<slug>-blind-v1:" + source_id)[:8], 16) / 2**32 < 0.2`, and
  to the training portion otherwise. The salt, the fraction and the source
  ids are in the protocol.
- **Size.** At least 500 generated and 500 real files in the blind set.
  At a recall of about 90% that gives a 95% interval of about ±2.6 points.
- **Freezing.** Before any model scores the blind set, its manifest
  (path, label, source id, SHA-256) is committed as
  `docs/generators/<slug>/blind.frozen.tsv.gz`. The slug is then added to
  the blind-set list in `AGENTS.md`.
- **Benchmark.** The blind set goes into the next benchmark version. It
  never edits an existing version (see [BENCHMARK.md](BENCHMARK.md)).

## 4. Retrain, then gate

### Retrain

- Fine-tune from the shipped weights on the **full** current training mix
  plus the new training portion. Training on the new data alone forgets
  older generators.
- Set the new store's sampling share with `--manifest-weight`, and use
  `tintype train ... --print-sampling` to confirm it before training.
- Register the arms (for example two manifest weights) and the selection
  rule before training.
- Calibrate with the standard calibration set, never with any blind set.
- Video generators use the video checkpoint and the same procedure.

### Gates

The gates are registered before any arm is scored, and each arm is scored
once.

1. **The new generator's blind set.**
   - Recall at the shipped cutoff improves on the shipped weights by at
     least the margin in the protocol (default: 5 points, with a paired
     bootstrap interval above zero).
   - Real photos flagged stay at or below the default's 1.5% target.
2. **Regression on older generators.** Score:
   - the benchmark's open and blind splits;
   - the latest blind photo batch;
   - CompEval;
   - the five public judges.

   No set's recall may fall by more than 2 points with an interval below
   zero. Pooled precision may not fall with an interval below zero, and
   pooled real-photo flags may not rise above the target.
3. **Speed.** Time per image on the benchmark's speed sample stays within
   5% of the shipped build.

An arm that fails a gate is not shipped. Its result is still logged. When
an arm ships, the website and README figures change only through a
registered read.

Use a paired bootstrap with 2,000 draws, resampling real and generated
files separately, and report 95% percentile intervals.

## 5. Log

Every run adds or updates a row in [generators/LOG.md](generators/LOG.md):

| Field | What |
|---|---|
| Generator | Name and exact model id |
| Released | Provider's release date |
| Added | Date of the decision |
| Status | `watching`, `probe`, `eval-only`, `collecting`, `trained`, `shipped`, `not shipped` or `blocked` |
| Route | Bifrost model id, other API or dataset |
| In shipped training | Images in the shipped weights' stores before the run |
| Samples | Training portion and blind set, generated and real |
| Cost | Total spend and cost per image |
| Before | Shipped weights on the blind set: recall, real flagged, recall at 1% false positives |
| After | The new weights, same measures |
| Regression | Worst per-set recall change on the gate sets, and whether every gate passed |
| Links | Protocol, result and pull request |

The log is the history of how the model has kept up with the field. Do not
rewrite rows: add a dated line under the row when a status changes.
