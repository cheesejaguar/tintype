Partial-edit gate: 3000 test items, cutoffs frozen on 1100 authentic images.
### 1% authentic FPR, `grid max` against the global score

- Transfer, `global (deployed)` (cutoff -1.035): 20/1100 (1.8%) untouched test images flagged, 95% interval 1.11% to 2.79%: above the 1% target.
- Transfer, `grid max` (cutoff 8.299): 16/1100 (1.5%) untouched test images flagged, 95% interval 0.83% to 2.35%: within the 1% target.
- Edited items found: global 129/600 (21.5%), `grid max` 319/600 (53.2%).
- Local: `grid max` finds more of a parent's edits than the global score in 73 parents and fewer in 11 (one-sided sign test p = 1.142e-12): more, beyond chance.
- Control `resave`: `grid max` flags 0/100 (0.0%) (global 1/100 (1.0%)); a larger share than of the parent's own untouched image in 0 parents, a smaller one in 1 (p = 1): no excess shown.
- Control `self-paste`: `grid max` flags 0/600 (0.0%) (global 8/600 (1.3%)); a larger share than of the parent's own untouched image in 0 parents, a smaller one in 1 (p = 1): no excess shown.
- Control `authentic-paste`: `grid max` flags 0/600 (0.0%) (global 4/600 (0.7%)); a larger share than of the parent's own untouched image in 0 parents, a smaller one in 1 (p = 1): no excess shown.
- **Gate at 1%: failed** (transfer not met).

### 0.1% authentic FPR, `grid max` against the global score

- Transfer, `global (deployed)` (cutoff 0.674): 7/1100 (0.6%) untouched test images flagged, 95% interval 0.26% to 1.31%: above the 0.1% target.
- Transfer, `grid max` (cutoff 9.860): 3/1100 (0.3%) untouched test images flagged, 95% interval 0.06% to 0.79%: within the 0.1% target.
- Edited items found: global 109/600 (18.2%), `grid max` 203/600 (33.8%).
- Local: `grid max` finds more of a parent's edits than the global score in 52 parents and fewer in 20 (one-sided sign test p = 0.0001039): more, beyond chance.
- Control `resave`: `grid max` flags 0/100 (0.0%) (global 0/100 (0.0%)); a larger share than of the parent's own untouched image in 0 parents, a smaller one in 0 (p = 1): no excess shown.
- Control `self-paste`: `grid max` flags 0/600 (0.0%) (global 0/600 (0.0%)); a larger share than of the parent's own untouched image in 0 parents, a smaller one in 0 (p = 1): no excess shown.
- Control `authentic-paste`: `grid max` flags 0/600 (0.0%) (global 0/600 (0.0%)); a larger share than of the parent's own untouched image in 0 parents, a smaller one in 0 (p = 1): no excess shown.
- **Gate at 0.1%: failed** (transfer not met).
