# Partial edits on the frozen split: archived scores

What [the partial-edit runbook](../../runbooks/partial-edits.md#frozen-split)
reports for the frozen split, as produced on 14 November 2026. Paths are
relative: `runs/frozen9/...` is where the composites were built (they are
deleted; `*-edits.tsv.gz` keeps each region's position, size, blend and
donor, from which `tintype paste-edits` rebuilds them with `--seed 1`), and
`data/frozen-src/...` is the split's source tree.

| File | What it is |
|---|---|
| `calibrate-cutoffs.tsv` | Cutoffs of every pooling rule at 1% and 0.1% authentic FPR, frozen on the 1,100 untouched `calibrate` images |
| `calibrate-windows.tsv.gz`, `test-windows.tsv.gz` | `tintype score-windows` output: the six view logits and every grid window's position and logit |
| `calibrate-manifest.tsv.gz`, `test-manifest.tsv.gz` | The four store manifests of each side and its pool of untouched originals |
| `calibrate-edits.tsv.gz`, `test-edits.tsv.gz` | The stores' `edits.tsv` rows |
| `calibrate-items.tsv.gz`, `test-items.tsv.gz` | `partial-report --out`: per image, the global score and the local scores (`test` at the frozen cutoffs) |
| `calibrate-report.md` | `partial-report` on `calibrate` (in-sample: where the cutoffs were set) |
| `test-report-frozen.md` | `partial-report --cutoffs calibrate-cutoffs.tsv` on `test` |
| `test-report-empirical.md` | `partial-report` on `test` at `test`'s own cutoffs: empirical, not frozen |
| `test-gate.md` | `scripts/partial-gate.py` on `test-items.tsv` |
