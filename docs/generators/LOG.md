# Generator log

One row per generator handled by the
[new generator playbook](../NEW_GENERATOR_PLAYBOOK.md). Rows are never
rewritten: when a status changes, add a dated line under the table entry's
notes.

| Generator | Released | Added | Status | Route | In shipped training | Samples (training / blind, generated + real) | Cost | Before (blind: recall / real flagged / recall at 1% FPR) | After | Regression | Links |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Imagen 3 (`imagen-3.0-generate-002`) | 2024 | 2026-11-25 | blocked | Bifrost `gemini/imagen-3.0-generate-002`; captions by `gemini/gemini-3.8-flash` | 228 of 235,776 generated rows | planned 2,000 + 2,000 / 500 + 500 | pilot pending | pending | pending | pending | [protocol](imagen-3/PROTOCOL.md) |

## Notes

### Imagen 3

- 2026-11-25:
  - Added after the batch-9 GPTTrace comparison: AIGIBench Imagen 3
    recall was 90.9% for the default against 96.7% for GPTTrace.
  - Status `blocked`: the Bifrost virtual key returns `403 model_blocked`
    for Imagen models.
  - The real photo pool and captions can proceed. Generation waits for the
    key.
- 2026-11-25, later:
  - **Pool:** 3,000 Wikimedia Commons Quality images kept, 633
    blind-assigned and 2,367 train-assigned before grouping. The rest of
    the 3,059 listed were excluded: 55 not photographs, 4 by licence.
  - **Pilot captions:** 50 train-assigned photos captioned, giving 45
    usable captions (median 79 words). The other 5 ran out of tokens and
    are replaced by the next photos in order. The caption model is
    `NVIDIA Build/nvidia/nemotron-3.5-super-vl-120b-a12b`
    (amendments 1 and 2).
  - **Gateway:** the Google provider has no credits left (402), and the
    Anthropic provider key is invalid (401).
  - **Generation is blocked on two things:** the key must allow Imagen
    models, and the Google project needs credits.
