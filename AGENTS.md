# Working on Tintype

Instructions for every agent session in this repository.

## Processes to follow

- **A new image or video generator comes out:** follow
  [docs/NEW_GENERATOR_PLAYBOOK.md](docs/NEW_GENERATOR_PLAYBOOK.md), and
  log the run in [docs/generators/LOG.md](docs/generators/LOG.md).
- **Any measurement that will be reported:** commit its protocol before
  the data is downloaded or scored, and score once. Past protocols are in
  [docs/PREREGISTRATION.md](docs/PREREGISTRATION.md).

## Data that must stay blind

Do not open, list, score, train on or calibrate with these until a
registered read says so:
- OpenSDI, megalith-10m, Semi-Truths and Pico-Banana;
- `fresh11`, `data/dev8/confirm16*` and `data/dev8/confirm17*`, and their
  score files;
- every `docs/generators/*/blind.frozen.tsv.gz` and the files it lists;
- the benchmark's blind split.

These are evaluation data, never training data:
- photo batch 9 (`data/fresh10`), the bar set behind the website's table;
- the five public held-out judges: NTIRE 2026 public test, WildAIGC V3,
  MIRAGE IID and OOD-R, and AIGenImages2026 test. Do not fit anything to
  their per-file results.

## Where data goes

Downloaded and generated media go to the data share
(`/Volumes/veracity-data`) or the GPU machine's data drives, not the Mac's
internal disk. Record the licence of every source.
