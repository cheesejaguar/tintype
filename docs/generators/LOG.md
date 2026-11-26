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
