# Imagen 3: first run of the new generator playbook

Registered on 2026-11-25. At registration no source photo had been fetched
or captioned, and no Imagen 3 image had been generated.

The process is [NEW_GENERATOR_PLAYBOOK.md](../../NEW_GENERATOR_PLAYBOOK.md).

## Why Imagen 3

On photo batch 9, GPTTrace found more AIGIBench Imagen 3 images than the
shipped default: 96.7% against 90.9%
([batch-9 GPTTrace comparison](../../results/batch9-gpttrace/report.md)).

The shipped weights saw few Imagen 3 images. Counts of generated rows by
generator in the `manifest.tsv` of the 18 phase-7 training stores:

| Google model | Stores | Images |
|---|---|---|
| Imagen 3 (`imagen-3`, `imagen-3.0-002`) | frontier 92, openfake 33, openfake2 11, openfake3 92 | **228** |
| Imagen 4, including Ultra | frontier 2,178, openfake 144, openfake2 75, openfake3 384 | 2,781 |
| Nano Banana / Gemini image | frontier 2,000, nano-banana 1,527, openfake 29, openfake2 9, openfake3 58 | 3,623 |

That is 228 of the 235,776 generated rows in those stores, 0.1%. The
playbook's threshold is 1,000, so Imagen 3 is added.

The count was made with:

```sh
awk -F'\t' '$2==1 && tolower($3) ~ /imagen|gemini|banana/ {c[$3]++} END {for (k in c) print k, c[k]}' \
  /Volumes/veracity-data/data/store/<store>/manifest.tsv
```

PhotoMaker, the other set where GPTTrace led (94.0% against 77.8%), has
no images in the shipped stores either. It is an open-weights SDXL
identity-adapter method and is not available through Bifrost. It is left
for a separate run.

## Route

- **Generator:** `gemini/imagen-3.0-generate-002` through the Bifrost
  gateway, `POST /v1/images/generations`, one image per request,
  `response_format: b64_json`. Each image's aspect ratio is the one of
  1:1, 3:4, 4:3, 9:16 and 16:9 closest to its source photo's.
- **Caption model:** `gemini/gemini-3.8-flash` through the same gateway.
- **Status at registration: blocked.** The gateway's virtual key returns
  `403 model_blocked` for every Imagen model. The user has to allow the
  model on the key. If Google no longer serves `imagen-3.0-generate-002`,
  falling back to `-001` needs an amendment.
- **Terms.** Google's generative AI terms govern use of outputs. Before
  scaling, record whether they allow training a detector on outputs, and
  record SynthID. The generated files are not redistributed: the recipe is
  (captions, source ids, parameters).

## Real photo pool

Wikimedia Commons, `Category:Quality_images`.
- Files added to the category from 2025-01-01 through 2025-06-30, in the
  order the API returns by timestamp.
- JPEG only, with a licence of CC0, public domain, CC BY or CC BY-SA (any
  version).
- Excluded: files whose description or categories mark them as
  illustrations, renders, scans of artwork, or computer-generated or AI
  images.

Each kept file is downloaded through Wikimedia's 2,048 px rendition, a
uniform re-encode. Its author, licence, URL and SHA-256 are recorded.

No training store or evaluation set uses Wikimedia Commons. Before use,
each file is checked by SHA-256 against every readable training and
evaluation manifest. The blind batches that may not be read (OpenSDI,
megalith-10m, Semi-Truths, fresh11, confirm16 and 17, Pico-Banana) draw
on other sources by their builder code.

Downloads go to `/Volumes/veracity-data/data/gen/wikimedia-qi/`.

## Captions

The caption instruction is in [caption_prompt.txt](caption_prompt.txt) and
is fixed. The caption model gets the 2,048 px photo, and its reply is
used unedited as the Imagen prompt. A photo is skipped and replaced by the
next one in order when either of these happens:
- the caption is refused or empty;
- Imagen refuses the prompt (safety filter).

Refusals are counted.

## Split

The unit is the source photo. A photo is **blind-assigned** when
`int(sha256("imagen-3-blind-v1:" + pageid)[:8], 16) / 2**32 < 0.2`, where
`pageid` is the Commons page id, and **train-assigned** otherwise.

Near-duplicates are grouped before the blind set is frozen. Two photos are
grouped when either holds:
- they have the same author, and capture dates within one day;
- their 64-bit dHash values are at Hamming distance 6 or less.

A group goes to the blind set only if every member is blind-assigned;
otherwise the whole group goes to the training portion. This can only
move photos out of the blind set, never into it, so the pilot (training
portion only) cannot leak into it.

## Sizes

| Portion | Source photos | Imagen 3 images |
|---|---|---|
| Pilot (training portion only) | 50 | 50 |
| Training portion, in all (pilot included) | about 2,000 | about 2,000 |
| Blind set | at least 500 | at least 500 |

The pilot is reviewed before scaling:
- cost per image;
- refusals;
- a visual check of all 50 images;
- the default's and GPTTrace's scores on these 50 **training-portion**
  images.

The blind set is generated only after the pilot review. Its manifest is
then frozen and committed before any model scores it.

## Blind set read (registered now)

Score the blind set once, as delivered and with both classes re-encoded
to JPEG q90. Report:
- the shipped v0.4.0 default and GPTTrace flag-pec-v3, at their shipped
  cutoffs;
- precision, recall, real photos flagged and recall at 1% false
  positives;
- after retraining (step 4), the selected arm.

## Retraining and gates

As in the playbook, step 4, with defaults:
- Fine-tune from the shipped weights on the full phase-7 mix plus this
  store.
- **Arms:** manifest weights 1 and 3 for the new store.
- **Imagen 3 gate:** blind recall up by at least 5 points over the shipped
  weights, with the paired-bootstrap interval above zero, and real photos
  flagged at or below 1.5%.
- **Regression gate:** no set's recall down by more than 2 points with an
  interval below zero, on:
  - the benchmark's open and blind splits;
  - CompEval;
  - the five public judges.

  Pooled precision is not lower and real flags are not above target.
- If both arms pass, the arm with higher Imagen 3 blind recall ships. Ties
  go to weight 1.

Retraining waits for the research session's current precision arms (P1,
P2) to finish, so the GPU and data share are not shared between training
runs.
