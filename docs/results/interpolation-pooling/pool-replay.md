Fit items: 82203 (46577 AI, 35626 real) from 1 file(s). Target false-positive rate 1.50%, averaged over the 16 real sources with at least 100 items.
745 items of the fit files are also test items (same path) and were left out of the fit. Paths are all this checks: copies and derivatives of a fit item under another path are not noticed.
Margins: refitted per pool and order on the fit items, the tightest on a grid of 0 to 12 that keep pooled precision and recall within 0.001 of all views.
Thresholds, margins and weights come from the fit items only. Recall at 1% and 0.1% FPR uses cutoffs found on the test items themselves: exploratory.

### Fitted on the fit items

| Pool | Order | Raw threshold | Slope | Margins after 1 to 5 views | Views per image | Precision / recall, all views | Precision / recall, early exit |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.692 | 0.664 | 6 / 3 / 2 / 1.5 / 1 | 1.79 | 0.9889 / 0.8783 | 0.9887 / 0.8767 |
| mean | 2,0,1,3,4,5 | 1.631 | 0.671 | 5.5 / 4 / 1.25 / 2 / 1.25 | 1.75 | 0.9885 / 0.8809 | 0.9878 / 0.8799 |
| mean | 4,1,0,3,2,5 (greedy) | 1.631 | 0.671 | 5.25 / 3.5 / 1.25 / 1.5 / 1.25 | 1.70 | 0.9885 / 0.8809 | 0.9875 / 0.8810 |
| mean | 0,2,1,3,4,5 | 1.631 | 0.671 | 6.5 / 2.5 / 1.25 / 1.5 / 1 | 1.68 | 0.9885 / 0.8809 | 0.9875 / 0.8803 |
| mean | 2,1,3,4,5,0 | 1.631 | 0.671 | 6.75 / 4.25 / 4.75 / 4.25 / 2.25 | 2.41 | 0.9885 / 0.8809 | 0.9884 / 0.8799 |
| trimmed mean | 2,0,1,3,4,5 | 1.782 | 0.618 | – / – / 3.5 / 2 / 0.75 | 3.35 | 0.9886 / 0.8691 | 0.9883 / 0.8701 |
| trimmed mean | 0,1,4,3,2,5 (greedy) | 1.782 | 0.618 | – / – / 4 / 3 / 2.5 | 3.53 | 0.9886 / 0.8691 | 0.9885 / 0.8701 |
| trimmed mean | 0,2,1,3,4,5 | 1.782 | 0.618 | – / – / 3.5 / 2 / 0.75 | 3.35 | 0.9886 / 0.8691 | 0.9883 / 0.8701 |
| trimmed mean | 2,1,3,4,5,0 | 1.782 | 0.618 | – / – / 4 / 4.25 / 2.5 | 3.63 | 0.9886 / 0.8691 | 0.9885 / 0.8681 |
| median | 2,0,1,3,4,5 | 1.828 | 0.591 | 4.25 / 2 / 2.5 / 0.5 / 0.5 | 1.45 | 0.9884 / 0.8638 | 0.9874 / 0.8636 |
| median | 4,3,1,5,0,2 (greedy) | 1.828 | 0.591 | 6 / 4.25 / 3.75 / 2 / 1.5 | 2.05 | 0.9884 / 0.8638 | 0.9885 / 0.8628 |
| median | 0,2,1,3,4,5 | 1.828 | 0.591 | 8.75 / 4.5 / 7 / 4 / 2.25 | 2.58 | 0.9884 / 0.8638 | 0.9886 / 0.8647 |
| median | 2,1,3,4,5,0 | 1.828 | 0.591 | 7.25 / 2 / 7 / 3.75 / 4.75 | 2.04 | 0.9884 / 0.8638 | 0.9879 / 0.8628 |
| top 2 | 2,0,1,3,4,5 | 4.910 | 0.634 | – / 9.25 / 7.5 / 7.5 / 5.5 | 4.37 | 0.9884 / 0.8818 | 0.9887 / 0.8808 |
| top 2 | 0,1,3,4,5,2 (greedy) | 4.910 | 0.634 | – / 9.25 / 7.5 / 6 / 5 | 4.20 | 0.9884 / 0.8818 | 0.9888 / 0.8808 |
| top 2 | 0,2,1,3,4,5 | 4.910 | 0.634 | – / 9.25 / 7.5 / 7.5 / 5.5 | 4.37 | 0.9884 / 0.8818 | 0.9887 / 0.8808 |
| top 2 | 2,1,3,4,5,0 | 4.910 | 0.634 | – / 8.5 / 7.25 / 7.25 / 6.25 | 4.39 | 0.9884 / 0.8818 | 0.9886 / 0.8808 |
| top 3 | 2,0,1,3,4,5 | 3.925 | 0.644 | – / – / 6 / 5.25 / 4 | 4.13 | 0.9882 / 0.8865 | 0.9885 / 0.8855 |
| top 3 | 0,1,4,3,5,2 (greedy) | 3.925 | 0.644 | – / – / 6 / 5.5 / 3.25 | 4.09 | 0.9882 / 0.8865 | 0.9885 / 0.8855 |
| top 3 | 0,2,1,3,4,5 | 3.925 | 0.644 | – / – / 6 / 5.25 / 4 | 4.13 | 0.9882 / 0.8865 | 0.9885 / 0.8855 |
| top 3 | 2,1,3,4,5,0 | 3.925 | 0.644 | – / – / 6.5 / 5.5 / 4.5 | 4.26 | 0.9882 / 0.8865 | 0.9883 / 0.8855 |
| max | 2,0,1,3,4,5 | 6.436 | 0.592 | – / 9.5 / 8.75 / – / 7.75 | 4.68 | 0.9878 / 0.8580 | 0.9886 / 0.8570 |
| max | 0,1,3,4,2,5 (greedy) | 6.436 | 0.592 | – / 9.25 / 9 / – / – | 4.75 | 0.9878 / 0.8580 | 0.9884 / 0.8570 |
| max | 0,2,1,3,4,5 | 6.436 | 0.592 | – / 9.5 / 8.75 / – / 7.75 | 4.68 | 0.9878 / 0.8580 | 0.9886 / 0.8570 |
| max | 2,1,3,4,5,0 | 6.436 | 0.592 | – / 9.75 / 9 / – / 8.75 | 4.75 | 0.9878 / 0.8580 | 0.9886 / 0.8570 |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,0,1,3,4,5 | 1.738 | 0.665 | 6.25 / 4.25 / 1.5 / 0.75 / 1.25 | 1.81 | 0.9887 / 0.8802 | 0.9879 / 0.8793 |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 4,0,5,3,1,2 (greedy) | 1.738 | 0.665 | 6.75 / 6.25 / 5 / 3 / 1.5 | 2.41 | 0.9887 / 0.8802 | 0.9888 / 0.8792 |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 0,2,1,3,4,5 | 1.738 | 0.665 | 6.75 / 4.25 / 1.5 / 1.5 / 1.5 | 1.88 | 0.9887 / 0.8802 | 0.9879 / 0.8793 |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,1,3,4,5,0 | 1.738 | 0.665 | 7.25 / 4.75 / 4.25 / 3.25 / 2.5 | 2.49 | 0.9887 / 0.8802 | 0.9887 / 0.8793 |

`early_exit` for mean, order 2,0,1,3,4,5 (pooled-logit threshold 1.6312): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":5.5,"above":5.5},{"views":2,"below":4.0,"above":4.0},{"views":3,"below":1.25,"above":1.25},{"views":4,"below":2.0,"above":2.0},{"views":5,"below":1.25,"above":1.25}]}`

`early_exit` for mean, order 4,1,0,3,2,5 (pooled-logit threshold 1.6312): `{"order":[4,1,0,3,2,5],"stages":[{"views":1,"below":5.25,"above":5.25},{"views":2,"below":3.5,"above":3.5},{"views":3,"below":1.25,"above":1.25},{"views":4,"below":1.5,"above":1.5},{"views":5,"below":1.25,"above":1.25}]}`

`early_exit` for mean, order 0,2,1,3,4,5 (pooled-logit threshold 1.6312): `{"order":[0,2,1,3,4,5],"stages":[{"views":1,"below":6.5,"above":6.5},{"views":2,"below":2.5,"above":2.5},{"views":3,"below":1.25,"above":1.25},{"views":4,"below":1.5,"above":1.5},{"views":5,"below":1.0,"above":1.0}]}`

`early_exit` for mean, order 2,1,3,4,5,0 (pooled-logit threshold 1.6312): `{"order":[2,1,3,4,5,0],"stages":[{"views":1,"below":6.75,"above":6.75},{"views":2,"below":4.25,"above":4.25},{"views":3,"below":4.75,"above":4.75},{"views":4,"below":4.25,"above":4.25},{"views":5,"below":2.25,"above":2.25}]}`

`early_exit` for trimmed mean, order 2,0,1,3,4,5 (pooled-logit threshold 1.7815): `{"order":[2,0,1,3,4,5],"stages":[{"views":3,"below":3.5,"above":3.5},{"views":4,"below":2.0,"above":2.0},{"views":5,"below":0.75,"above":0.75}]}`

`early_exit` for trimmed mean, order 0,1,4,3,2,5 (pooled-logit threshold 1.7815): `{"order":[0,1,4,3,2,5],"stages":[{"views":3,"below":4.0,"above":4.0},{"views":4,"below":3.0,"above":3.0},{"views":5,"below":2.5,"above":2.5}]}`

`early_exit` for trimmed mean, order 0,2,1,3,4,5 (pooled-logit threshold 1.7815): `{"order":[0,2,1,3,4,5],"stages":[{"views":3,"below":3.5,"above":3.5},{"views":4,"below":2.0,"above":2.0},{"views":5,"below":0.75,"above":0.75}]}`

`early_exit` for trimmed mean, order 2,1,3,4,5,0 (pooled-logit threshold 1.7815): `{"order":[2,1,3,4,5,0],"stages":[{"views":3,"below":4.0,"above":4.0},{"views":4,"below":4.25,"above":4.25},{"views":5,"below":2.5,"above":2.5}]}`

`early_exit` for median, order 2,0,1,3,4,5 (pooled-logit threshold 1.8277): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":4.25,"above":4.25},{"views":2,"below":2.0,"above":2.0},{"views":3,"below":2.5,"above":2.5},{"views":4,"below":0.5,"above":0.5},{"views":5,"below":0.5,"above":0.5}]}`

`early_exit` for median, order 4,3,1,5,0,2 (pooled-logit threshold 1.8277): `{"order":[4,3,1,5,0,2],"stages":[{"views":1,"below":6.0,"above":6.0},{"views":2,"below":4.25,"above":4.25},{"views":3,"below":3.75,"above":3.75},{"views":4,"below":2.0,"above":2.0},{"views":5,"below":1.5,"above":1.5}]}`

`early_exit` for median, order 0,2,1,3,4,5 (pooled-logit threshold 1.8277): `{"order":[0,2,1,3,4,5],"stages":[{"views":1,"below":8.75,"above":8.75},{"views":2,"below":4.5,"above":4.5},{"views":3,"below":7.0,"above":7.0},{"views":4,"below":4.0,"above":4.0},{"views":5,"below":2.25,"above":2.25}]}`

`early_exit` for median, order 2,1,3,4,5,0 (pooled-logit threshold 1.8277): `{"order":[2,1,3,4,5,0],"stages":[{"views":1,"below":7.25,"above":7.25},{"views":2,"below":2.0,"above":2.0},{"views":3,"below":7.0,"above":7.0},{"views":4,"below":3.75,"above":3.75},{"views":5,"below":4.75,"above":4.75}]}`

`early_exit` for top 2, order 2,0,1,3,4,5 (pooled-logit threshold 4.9097): `{"order":[2,0,1,3,4,5],"stages":[{"views":2,"below":9.25,"above":9.25},{"views":3,"below":7.5,"above":7.5},{"views":4,"below":7.5,"above":7.5},{"views":5,"below":5.5,"above":5.5}]}`

`early_exit` for top 2, order 0,1,3,4,5,2 (pooled-logit threshold 4.9097): `{"order":[0,1,3,4,5,2],"stages":[{"views":2,"below":9.25,"above":9.25},{"views":3,"below":7.5,"above":7.5},{"views":4,"below":6.0,"above":6.0},{"views":5,"below":5.0,"above":5.0}]}`

`early_exit` for top 2, order 0,2,1,3,4,5 (pooled-logit threshold 4.9097): `{"order":[0,2,1,3,4,5],"stages":[{"views":2,"below":9.25,"above":9.25},{"views":3,"below":7.5,"above":7.5},{"views":4,"below":7.5,"above":7.5},{"views":5,"below":5.5,"above":5.5}]}`

`early_exit` for top 2, order 2,1,3,4,5,0 (pooled-logit threshold 4.9097): `{"order":[2,1,3,4,5,0],"stages":[{"views":2,"below":8.5,"above":8.5},{"views":3,"below":7.25,"above":7.25},{"views":4,"below":7.25,"above":7.25},{"views":5,"below":6.25,"above":6.25}]}`

`early_exit` for top 3, order 2,0,1,3,4,5 (pooled-logit threshold 3.9254): `{"order":[2,0,1,3,4,5],"stages":[{"views":3,"below":6.0,"above":6.0},{"views":4,"below":5.25,"above":5.25},{"views":5,"below":4.0,"above":4.0}]}`

`early_exit` for top 3, order 0,1,4,3,5,2 (pooled-logit threshold 3.9254): `{"order":[0,1,4,3,5,2],"stages":[{"views":3,"below":6.0,"above":6.0},{"views":4,"below":5.5,"above":5.5},{"views":5,"below":3.25,"above":3.25}]}`

`early_exit` for top 3, order 0,2,1,3,4,5 (pooled-logit threshold 3.9254): `{"order":[0,2,1,3,4,5],"stages":[{"views":3,"below":6.0,"above":6.0},{"views":4,"below":5.25,"above":5.25},{"views":5,"below":4.0,"above":4.0}]}`

`early_exit` for top 3, order 2,1,3,4,5,0 (pooled-logit threshold 3.9254): `{"order":[2,1,3,4,5,0],"stages":[{"views":3,"below":6.5,"above":6.5},{"views":4,"below":5.5,"above":5.5},{"views":5,"below":4.5,"above":4.5}]}`

`early_exit` for max, order 2,0,1,3,4,5 (pooled-logit threshold 6.4357): `{"order":[2,0,1,3,4,5],"stages":[{"views":2,"below":9.5,"above":9.5},{"views":3,"below":8.75,"above":8.75},{"views":5,"below":7.75,"above":7.75}]}`

`early_exit` for max, order 0,1,3,4,2,5 (pooled-logit threshold 6.4357): `{"order":[0,1,3,4,2,5],"stages":[{"views":2,"below":9.25,"above":9.25},{"views":3,"below":9.0,"above":9.0}]}`

`early_exit` for max, order 0,2,1,3,4,5 (pooled-logit threshold 6.4357): `{"order":[0,2,1,3,4,5],"stages":[{"views":2,"below":9.5,"above":9.5},{"views":3,"below":8.75,"above":8.75},{"views":5,"below":7.75,"above":7.75}]}`

`early_exit` for max, order 2,1,3,4,5,0 (pooled-logit threshold 6.4357): `{"order":[2,1,3,4,5,0],"stages":[{"views":2,"below":9.75,"above":9.75},{"views":3,"below":9.0,"above":9.0},{"views":5,"below":8.75,"above":8.75}]}`

`early_exit` for fitted weights 0.241/0.152/0.133/0.149/0.161/0.164, order 2,0,1,3,4,5 (pooled-logit threshold 1.7385): `{"order":[2,0,1,3,4,5],"stages":[{"views":1,"below":6.25,"above":6.25},{"views":2,"below":4.25,"above":4.25},{"views":3,"below":1.5,"above":1.5},{"views":4,"below":0.75,"above":0.75},{"views":5,"below":1.25,"above":1.25}]}`

`early_exit` for fitted weights 0.241/0.152/0.133/0.149/0.161/0.164, order 4,0,5,3,1,2 (pooled-logit threshold 1.7385): `{"order":[4,0,5,3,1,2],"stages":[{"views":1,"below":6.75,"above":6.75},{"views":2,"below":6.25,"above":6.25},{"views":3,"below":5.0,"above":5.0},{"views":4,"below":3.0,"above":3.0},{"views":5,"below":1.5,"above":1.5}]}`

`early_exit` for fitted weights 0.241/0.152/0.133/0.149/0.161/0.164, order 0,2,1,3,4,5 (pooled-logit threshold 1.7385): `{"order":[0,2,1,3,4,5],"stages":[{"views":1,"below":6.75,"above":6.75},{"views":2,"below":4.25,"above":4.25},{"views":3,"below":1.5,"above":1.5},{"views":4,"below":1.5,"above":1.5},{"views":5,"below":1.5,"above":1.5}]}`

`early_exit` for fitted weights 0.241/0.152/0.133/0.149/0.161/0.164, order 2,1,3,4,5,0 (pooled-logit threshold 1.7385): `{"order":[2,1,3,4,5,0],"stages":[{"views":1,"below":7.25,"above":7.25},{"views":2,"below":4.75,"above":4.75},{"views":3,"below":4.25,"above":4.25},{"views":4,"below":3.25,"above":3.25},{"views":5,"below":2.5,"above":2.5}]}`

### Test: confirm7 (23052 items: 12305 AI, 10747 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 89.00% | 63.58% | 127 of 10747 (1.18%) | 90.18% | +112 −53 AI, +1 −4 real |
| mean | 89.00% | 63.58% | 128 of 10747 (1.19%) | 90.33% | +123 −46 AI, +2 −4 real |
| trimmed mean | 88.16% | 65.66% | 132 of 10747 (1.23%) | 89.21% | +120 −181 AI, +10 −8 real |
| median | 87.29% | 63.96% | 133 of 10747 (1.24%) | 88.88% | +136 −237 AI, +17 −14 real |
| top 2 | 90.28% | 71.06% | 96 of 10747 (0.89%) | 89.78% | +271 −262 AI, +23 −57 real |
| top 3 | 90.17% | 69.27% | 109 of 10747 (1.01%) | 90.19% | +237 −177 AI, +19 −40 real |
| max | 89.28% | 64.89% | 79 of 10747 (0.74%) | 87.22% | +282 −588 AI, +34 −85 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 89.15% | 66.38% | 124 of 10747 (1.15%) | 90.23% | +142 −77 AI, +4 −10 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.69 | 88.48% | 58.30% | 130 of 10747 (1.21%) | 89.70% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.66 | 88.41% | 57.35% | 138 of 10747 (1.28%) | 89.94% | +74 −45 AI, +10 −2 real |
| mean | 4,1,0,3,2,5 | 1.59 | 88.87% | 63.63% | 133 of 10747 (1.24%) | 90.17% | +173 −115 AI, +14 −11 real |
| mean | 0,2,1,3,4,5 | 1.61 | 87.65% | 61.24% | 144 of 10747 (1.34%) | 90.17% | +127 −70 AI, +15 −1 real |
| mean | 2,1,3,4,5,0 | 2.22 | 88.92% | 61.63% | 128 of 10747 (1.19%) | 90.17% | +103 −45 AI, +2 −4 real |
| trimmed mean | 2,0,1,3,4,5 | 3.28 | 88.19% | 60.24% | 132 of 10747 (1.23%) | 89.17% | +116 −182 AI, +10 −8 real |
| trimmed mean | 0,1,4,3,2,5 | 3.43 | 88.34% | 62.14% | 130 of 10747 (1.21%) | 89.31% | +132 −180 AI, +9 −9 real |
| trimmed mean | 0,2,1,3,4,5 | 3.28 | 88.19% | 60.24% | 132 of 10747 (1.23%) | 89.17% | +116 −182 AI, +10 −8 real |
| trimmed mean | 2,1,3,4,5,0 | 3.51 | 88.12% | 65.27% | 133 of 10747 (1.24%) | 89.13% | +120 −190 AI, +11 −8 real |
| median | 2,0,1,3,4,5 | 1.39 | 86.07% | 54.64% | 144 of 10747 (1.34%) | 88.20% | +67 −252 AI, +25 −11 real |
| median | 4,3,1,5,0,2 | 1.89 | 87.38% | 65.18% | 131 of 10747 (1.22%) | 88.81% | +156 −266 AI, +18 −17 real |
| median | 0,2,1,3,4,5 | 2.42 | 87.79% | 64.34% | 130 of 10747 (1.21%) | 89.04% | +123 −205 AI, +15 −15 real |
| median | 2,1,3,4,5,0 | 1.90 | 87.24% | 60.08% | 133 of 10747 (1.24%) | 88.52% | +117 −263 AI, +18 −15 real |
| top 2 | 2,0,1,3,4,5 | 4.22 | 90.36% | 71.04% | 93 of 10747 (0.87%) | 89.70% | +262 −262 AI, +20 −57 real |
| top 2 | 0,1,3,4,5,2 | 4.03 | 90.52% | 71.05% | 91 of 10747 (0.85%) | 89.73% | +266 −263 AI, +18 −57 real |
| top 2 | 0,2,1,3,4,5 | 4.22 | 90.36% | 71.04% | 93 of 10747 (0.87%) | 89.70% | +262 −262 AI, +20 −57 real |
| top 2 | 2,1,3,4,5,0 | 4.24 | 90.43% | 71.04% | 93 of 10747 (0.87%) | 89.70% | +261 −262 AI, +20 −57 real |
| top 3 | 2,0,1,3,4,5 | 3.98 | 90.28% | 69.26% | 106 of 10747 (0.99%) | 90.14% | +231 −177 AI, +16 −40 real |
| top 3 | 0,1,4,3,5,2 | 3.94 | 90.29% | 68.19% | 107 of 10747 (1.00%) | 90.16% | +235 −179 AI, +17 −40 real |
| top 3 | 0,2,1,3,4,5 | 3.98 | 90.28% | 69.26% | 106 of 10747 (0.99%) | 90.14% | +231 −177 AI, +16 −40 real |
| top 3 | 2,1,3,4,5,0 | 4.10 | 90.09% | 69.27% | 108 of 10747 (1.00%) | 90.09% | +226 −178 AI, +18 −40 real |
| max | 2,0,1,3,4,5 | 4.50 | 89.33% | 64.86% | 76 of 10747 (0.71%) | 87.15% | +274 −588 AI, +31 −85 real |
| max | 0,1,3,4,2,5 | 4.54 | 89.57% | 64.88% | 75 of 10747 (0.70%) | 87.17% | +276 −588 AI, +30 −85 real |
| max | 0,2,1,3,4,5 | 4.50 | 89.33% | 64.86% | 76 of 10747 (0.71%) | 87.15% | +274 −588 AI, +31 −85 real |
| max | 2,1,3,4,5,0 | 4.60 | 89.54% | 68.00% | 73 of 10747 (0.68%) | 87.14% | +274 −589 AI, +28 −85 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,0,1,3,4,5 | 1.71 | 88.41% | 61.54% | 130 of 10747 (1.21%) | 89.91% | +107 −82 AI, +11 −11 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 4,0,5,3,1,2 | 2.26 | 89.11% | 69.30% | 123 of 10747 (1.14%) | 90.13% | +142 −90 AI, +4 −11 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 0,2,1,3,4,5 | 1.78 | 88.57% | 59.82% | 133 of 10747 (1.24%) | 90.08% | +154 −108 AI, +13 −10 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,1,3,4,5,0 | 2.29 | 89.12% | 63.38% | 124 of 10747 (1.15%) | 90.09% | +122 −75 AI, +4 −10 real |

### Test: confirm6 (16324 items: 8628 AI, 7696 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 87.67% | 61.37% | 112 of 7696 (1.46%) | 90.51% | +74 −61 AI, +2 −1 real |
| mean | 87.67% | 61.37% | 115 of 7696 (1.49%) | 90.81% | +96 −57 AI, +5 −1 real |
| trimmed mean | 87.04% | 62.81% | 108 of 7696 (1.40%) | 89.60% | +87 −152 AI, +3 −6 real |
| median | 86.23% | 60.84% | 108 of 7696 (1.40%) | 88.87% | +101 −229 AI, +10 −13 real |
| top 2 | 90.58% | 62.63% | 75 of 7696 (0.97%) | 90.48% | +213 −202 AI, +13 −49 real |
| top 3 | 90.53% | 62.00% | 81 of 7696 (1.05%) | 90.98% | +181 −127 AI, +6 −36 real |
| max | 88.73% | 64.58% | 60 of 7696 (0.78%) | 87.61% | +229 −466 AI, +20 −71 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 88.38% | 63.62% | 109 of 7696 (1.42%) | 90.96% | +121 −69 AI, +4 −6 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.71 | 88.27% | 57.59% | 111 of 7696 (1.44%) | 90.36% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.67 | 88.43% | 55.52% | 115 of 7696 (1.49%) | 90.36% | +46 −46 AI, +5 −1 real |
| mean | 4,1,0,3,2,5 | 1.62 | 88.84% | 54.80% | 116 of 7696 (1.51%) | 91.05% | +152 −92 AI, +10 −5 real |
| mean | 0,2,1,3,4,5 | 1.60 | 89.07% | 62.08% | 115 of 7696 (1.49%) | 90.81% | +106 −67 AI, +7 −3 real |
| mean | 2,1,3,4,5,0 | 2.29 | 87.67% | 64.23% | 114 of 7696 (1.48%) | 90.67% | +81 −54 AI, +4 −1 real |
| trimmed mean | 2,0,1,3,4,5 | 3.31 | 87.81% | 63.89% | 104 of 7696 (1.35%) | 89.53% | +80 −151 AI, +3 −10 real |
| trimmed mean | 0,1,4,3,2,5 | 3.46 | 87.66% | 59.27% | 106 of 7696 (1.38%) | 89.81% | +92 −139 AI, +3 −8 real |
| trimmed mean | 0,2,1,3,4,5 | 3.31 | 87.81% | 63.89% | 104 of 7696 (1.35%) | 89.53% | +80 −151 AI, +3 −10 real |
| trimmed mean | 2,1,3,4,5,0 | 3.57 | 86.98% | 56.86% | 108 of 7696 (1.40%) | 89.45% | +83 −161 AI, +3 −6 real |
| median | 2,0,1,3,4,5 | 1.41 | 85.76% | 52.63% | 111 of 7696 (1.44%) | 88.11% | +58 −252 AI, +13 −13 real |
| median | 4,3,1,5,0,2 | 1.95 | 87.12% | 54.68% | 104 of 7696 (1.35%) | 88.90% | +116 −242 AI, +9 −16 real |
| median | 0,2,1,3,4,5 | 2.44 | 87.22% | 60.77% | 105 of 7696 (1.36%) | 89.31% | +96 −186 AI, +8 −14 real |
| median | 2,1,3,4,5,0 | 1.96 | 86.27% | 60.94% | 111 of 7696 (1.44%) | 88.55% | +79 −235 AI, +12 −12 real |
| top 2 | 2,0,1,3,4,5 | 4.20 | 90.51% | 62.63% | 75 of 7696 (0.97%) | 90.41% | +207 −202 AI, +13 −49 real |
| top 2 | 0,1,3,4,5,2 | 4.04 | 90.97% | 69.27% | 70 of 7696 (0.91%) | 90.46% | +212 −203 AI, +9 −50 real |
| top 2 | 0,2,1,3,4,5 | 4.20 | 90.51% | 62.63% | 75 of 7696 (0.97%) | 90.41% | +207 −202 AI, +13 −49 real |
| top 2 | 2,1,3,4,5,0 | 4.21 | 90.45% | 62.63% | 74 of 7696 (0.96%) | 90.28% | +197 −204 AI, +12 −49 real |
| top 3 | 2,0,1,3,4,5 | 4.04 | 90.46% | 62.00% | 80 of 7696 (1.04%) | 90.91% | +175 −127 AI, +5 −36 real |
| top 3 | 0,1,4,3,5,2 | 3.99 | 90.53% | 62.07% | 79 of 7696 (1.03%) | 90.96% | +179 −127 AI, +4 −36 real |
| top 3 | 0,2,1,3,4,5 | 4.04 | 90.46% | 62.00% | 80 of 7696 (1.04%) | 90.91% | +175 −127 AI, +5 −36 real |
| top 3 | 2,1,3,4,5,0 | 4.15 | 90.41% | 62.00% | 81 of 7696 (1.05%) | 90.84% | +172 −130 AI, +6 −36 real |
| max | 2,0,1,3,4,5 | 4.45 | 88.78% | 64.57% | 58 of 7696 (0.75%) | 87.53% | +222 −466 AI, +18 −71 real |
| max | 0,1,3,4,2,5 | 4.50 | 89.01% | 64.80% | 56 of 7696 (0.73%) | 87.56% | +225 −466 AI, +18 −73 real |
| max | 0,2,1,3,4,5 | 4.45 | 88.78% | 64.57% | 58 of 7696 (0.75%) | 87.53% | +222 −466 AI, +18 −71 real |
| max | 2,1,3,4,5,0 | 4.54 | 88.94% | 64.55% | 56 of 7696 (0.73%) | 87.49% | +219 −466 AI, +16 −71 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,0,1,3,4,5 | 1.73 | 88.80% | 60.69% | 107 of 7696 (1.39%) | 90.65% | +92 −67 AI, +4 −8 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 4,0,5,3,1,2 | 2.28 | 88.40% | 61.62% | 109 of 7696 (1.42%) | 90.90% | +123 −76 AI, +4 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 0,2,1,3,4,5 | 1.78 | 89.14% | 60.69% | 110 of 7696 (1.43%) | 90.83% | +129 −88 AI, +6 −7 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,1,3,4,5,0 | 2.36 | 88.72% | 65.00% | 108 of 7696 (1.40%) | 90.77% | +101 −65 AI, +3 −6 real |

### Test: c7-legacy-gan (7858 items: 4000 AI, 3858 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 91.15% | 65.25% | 54 of 3858 (1.40%) | 93.23% | +29 −25 AI, +0 −0 real |
| mean | 91.15% | 65.25% | 54 of 3858 (1.40%) | 93.35% | +32 −23 AI, +0 −0 real |
| trimmed mean | 90.00% | 65.83% | 52 of 3858 (1.35%) | 91.80% | +29 −82 AI, +1 −3 real |
| median | 89.53% | 65.00% | 49 of 3858 (1.27%) | 91.45% | +38 −105 AI, +1 −6 real |
| top 2 | 95.03% | 78.00% | 21 of 3858 (0.54%) | 92.85% | +86 −97 AI, +1 −34 real |
| top 3 | 94.47% | 73.95% | 27 of 3858 (0.70%) | 93.40% | +72 −61 AI, +1 −28 real |
| max | 94.75% | 78.12% | 18 of 3858 (0.47%) | 89.65% | +91 −230 AI, +7 −43 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 92.05% | 66.27% | 49 of 3858 (1.27%) | 93.97% | +53 −19 AI, +0 −5 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.60 | 91.95% | 72.28% | 54 of 3858 (1.40%) | 93.12% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.57 | 91.85% | 73.15% | 54 of 3858 (1.40%) | 93.08% | +23 −25 AI, +0 −0 real |
| mean | 4,1,0,3,2,5 | 1.53 | 92.42% | 61.05% | 52 of 3858 (1.35%) | 93.45% | +58 −45 AI, +1 −3 real |
| mean | 0,2,1,3,4,5 | 1.47 | 93.88% | 77.20% | 53 of 3858 (1.37%) | 94.85% | +72 −3 AI, +0 −1 real |
| mean | 2,1,3,4,5,0 | 2.16 | 90.92% | 68.12% | 54 of 3858 (1.40%) | 93.00% | +21 −26 AI, +0 −0 real |
| trimmed mean | 2,0,1,3,4,5 | 3.27 | 90.45% | 62.98% | 52 of 3858 (1.35%) | 91.70% | +28 −85 AI, +1 −3 real |
| trimmed mean | 0,1,4,3,2,5 | 3.40 | 90.20% | 64.15% | 52 of 3858 (1.35%) | 91.95% | +31 −78 AI, +1 −3 real |
| trimmed mean | 0,2,1,3,4,5 | 3.27 | 90.45% | 62.98% | 52 of 3858 (1.35%) | 91.70% | +28 −85 AI, +1 −3 real |
| trimmed mean | 2,1,3,4,5,0 | 3.50 | 89.85% | 68.30% | 52 of 3858 (1.35%) | 91.60% | +27 −88 AI, +1 −3 real |
| median | 2,0,1,3,4,5 | 1.34 | 90.12% | 74.20% | 50 of 3858 (1.30%) | 91.62% | +27 −87 AI, +2 −6 real |
| median | 4,3,1,5,0,2 | 1.82 | 89.60% | 67.00% | 48 of 3858 (1.24%) | 90.97% | +38 −124 AI, +1 −7 real |
| median | 0,2,1,3,4,5 | 2.22 | 91.20% | 70.95% | 49 of 3858 (1.27%) | 92.60% | +48 −69 AI, +1 −6 real |
| median | 2,1,3,4,5,0 | 1.86 | 88.62% | 65.45% | 51 of 3858 (1.32%) | 90.38% | +21 −131 AI, +3 −6 real |
| top 2 | 2,0,1,3,4,5 | 4.06 | 95.03% | 78.00% | 20 of 3858 (0.52%) | 92.85% | +86 −97 AI, +0 −34 real |
| top 2 | 0,1,3,4,5,2 | 3.88 | 95.08% | 78.00% | 21 of 3858 (0.54%) | 92.85% | +86 −97 AI, +1 −34 real |
| top 2 | 0,2,1,3,4,5 | 4.06 | 95.03% | 78.00% | 20 of 3858 (0.52%) | 92.85% | +86 −97 AI, +0 −34 real |
| top 2 | 2,1,3,4,5,0 | 4.08 | 94.85% | 77.95% | 20 of 3858 (0.52%) | 92.73% | +81 −97 AI, +0 −34 real |
| top 3 | 2,0,1,3,4,5 | 3.88 | 94.73% | 73.95% | 26 of 3858 (0.67%) | 93.40% | +72 −61 AI, +0 −28 real |
| top 3 | 0,1,4,3,5,2 | 3.85 | 94.47% | 73.67% | 27 of 3858 (0.70%) | 93.40% | +72 −61 AI, +1 −28 real |
| top 3 | 0,2,1,3,4,5 | 3.88 | 94.73% | 73.95% | 26 of 3858 (0.67%) | 93.40% | +72 −61 AI, +0 −28 real |
| top 3 | 2,1,3,4,5,0 | 3.99 | 94.27% | 73.92% | 27 of 3858 (0.70%) | 93.25% | +66 −61 AI, +1 −28 real |
| max | 2,0,1,3,4,5 | 4.31 | 95.08% | 78.12% | 17 of 3858 (0.44%) | 89.65% | +91 −230 AI, +6 −43 real |
| max | 0,1,3,4,2,5 | 4.33 | 94.75% | 78.12% | 18 of 3858 (0.47%) | 89.65% | +91 −230 AI, +7 −43 real |
| max | 0,2,1,3,4,5 | 4.31 | 95.08% | 78.12% | 17 of 3858 (0.44%) | 89.65% | +91 −230 AI, +6 −43 real |
| max | 2,1,3,4,5,0 | 4.40 | 94.73% | 78.08% | 18 of 3858 (0.47%) | 89.58% | +88 −230 AI, +7 −43 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,0,1,3,4,5 | 1.60 | 93.25% | 73.52% | 48 of 3858 (1.24%) | 94.15% | +54 −13 AI, +0 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 4,0,5,3,1,2 | 2.09 | 91.90% | 72.85% | 48 of 3858 (1.24%) | 93.67% | +49 −27 AI, +0 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 0,2,1,3,4,5 | 1.60 | 93.95% | 76.05% | 48 of 3858 (1.24%) | 94.83% | +78 −10 AI, +0 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,1,3,4,5,0 | 2.22 | 91.85% | 66.12% | 49 of 3858 (1.27%) | 93.58% | +40 −22 AI, +0 −5 real |

### Test: c7-current (4195 items: 2394 AI, 1801 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 98.87% | 88.35% | 12 of 1801 (0.67%) | 98.12% | +6 −1 AI, +0 −0 real |
| mean | 98.87% | 88.35% | 12 of 1801 (0.67%) | 98.20% | +8 −1 AI, +0 −0 real |
| trimmed mean | 98.83% | 89.43% | 13 of 1801 (0.72%) | 97.79% | +6 −9 AI, +1 −0 real |
| median | 98.79% | 87.89% | 12 of 1801 (0.67%) | 97.62% | +7 −14 AI, +1 −1 real |
| top 2 | 98.58% | 89.77% | 12 of 1801 (0.67%) | 97.99% | +14 −12 AI, +3 −3 real |
| top 3 | 98.83% | 89.26% | 13 of 1801 (0.72%) | 98.16% | +14 −8 AI, +2 −1 real |
| max | 98.04% | 85.59% | 10 of 1801 (0.56%) | 97.24% | +15 −31 AI, +4 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 98.83% | 85.63% | 11 of 1801 (0.61%) | 98.08% | +9 −5 AI, +0 −1 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.41 | 98.66% | 74.02% | 12 of 1801 (0.67%) | 97.91% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.38 | 98.50% | 71.14% | 13 of 1801 (0.72%) | 97.95% | +3 −2 AI, +1 −0 real |
| mean | 4,1,0,3,2,5 | 1.31 | 98.66% | 86.72% | 12 of 1801 (0.67%) | 97.91% | +9 −9 AI, +1 −1 real |
| mean | 0,2,1,3,4,5 | 1.38 | 98.54% | 79.49% | 14 of 1801 (0.78%) | 97.95% | +7 −6 AI, +2 −0 real |
| mean | 2,1,3,4,5,0 | 1.70 | 98.62% | 81.08% | 12 of 1801 (0.67%) | 98.08% | +5 −1 AI, +0 −0 real |
| trimmed mean | 2,0,1,3,4,5 | 3.11 | 98.87% | 86.93% | 12 of 1801 (0.67%) | 97.74% | +5 −9 AI, +0 −0 real |
| trimmed mean | 0,1,4,3,2,5 | 3.18 | 99.00% | 89.93% | 12 of 1801 (0.67%) | 97.83% | +7 −9 AI, +0 −0 real |
| trimmed mean | 0,2,1,3,4,5 | 3.11 | 98.87% | 86.93% | 12 of 1801 (0.67%) | 97.74% | +5 −9 AI, +0 −0 real |
| trimmed mean | 2,1,3,4,5,0 | 3.22 | 98.75% | 86.93% | 13 of 1801 (0.72%) | 97.79% | +7 −10 AI, +1 −0 real |
| median | 2,0,1,3,4,5 | 1.22 | 97.91% | 67.92% | 14 of 1801 (0.78%) | 97.20% | +3 −20 AI, +3 −1 real |
| median | 4,3,1,5,0,2 | 1.49 | 98.58% | 91.85% | 12 of 1801 (0.67%) | 97.58% | +7 −15 AI, +1 −1 real |
| median | 0,2,1,3,4,5 | 1.95 | 98.50% | 84.34% | 12 of 1801 (0.67%) | 97.54% | +6 −15 AI, +1 −1 real |
| median | 2,1,3,4,5,0 | 1.57 | 98.16% | 79.78% | 12 of 1801 (0.67%) | 97.37% | +3 −16 AI, +1 −1 real |
| top 2 | 2,0,1,3,4,5 | 4.07 | 98.50% | 89.77% | 12 of 1801 (0.67%) | 97.91% | +12 −12 AI, +3 −3 real |
| top 2 | 0,1,3,4,5,2 | 3.79 | 98.87% | 89.77% | 11 of 1801 (0.61%) | 97.99% | +14 −12 AI, +2 −3 real |
| top 2 | 0,2,1,3,4,5 | 4.07 | 98.50% | 89.77% | 12 of 1801 (0.67%) | 97.91% | +12 −12 AI, +3 −3 real |
| top 2 | 2,1,3,4,5,0 | 4.17 | 98.50% | 89.77% | 12 of 1801 (0.67%) | 97.99% | +14 −12 AI, +3 −3 real |
| top 3 | 2,0,1,3,4,5 | 3.70 | 98.96% | 89.26% | 12 of 1801 (0.67%) | 98.12% | +13 −8 AI, +1 −1 real |
| top 3 | 0,1,4,3,5,2 | 3.68 | 99.00% | 88.55% | 13 of 1801 (0.72%) | 98.16% | +14 −8 AI, +2 −1 real |
| top 3 | 0,2,1,3,4,5 | 3.70 | 98.96% | 89.26% | 12 of 1801 (0.67%) | 98.12% | +13 −8 AI, +1 −1 real |
| top 3 | 2,1,3,4,5,0 | 3.85 | 98.83% | 89.26% | 13 of 1801 (0.72%) | 98.16% | +14 −8 AI, +2 −1 real |
| max | 2,0,1,3,4,5 | 4.50 | 98.20% | 85.59% | 9 of 1801 (0.50%) | 97.20% | +14 −31 AI, +3 −6 real |
| max | 0,1,3,4,2,5 | 4.57 | 98.29% | 85.59% | 9 of 1801 (0.50%) | 97.24% | +15 −31 AI, +3 −6 real |
| max | 0,2,1,3,4,5 | 4.50 | 98.20% | 85.59% | 9 of 1801 (0.50%) | 97.20% | +14 −31 AI, +3 −6 real |
| max | 2,1,3,4,5,0 | 4.62 | 98.04% | 85.59% | 10 of 1801 (0.56%) | 97.24% | +15 −31 AI, +4 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,0,1,3,4,5 | 1.45 | 98.62% | 70.26% | 11 of 1801 (0.61%) | 97.74% | +4 −8 AI, +0 −1 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 4,0,5,3,1,2 | 1.75 | 98.66% | 88.01% | 11 of 1801 (0.61%) | 98.04% | +9 −6 AI, +0 −1 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 0,2,1,3,4,5 | 1.49 | 98.75% | 83.00% | 11 of 1801 (0.61%) | 97.95% | +9 −8 AI, +0 −1 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,1,3,4,5,0 | 1.79 | 98.66% | 83.29% | 11 of 1801 (0.61%) | 97.99% | +7 −5 AI, +0 −1 real |

### Test: wasitai-original (745 items: 261 AI, 484 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 72.41% | 45.21% | 12 of 484 (2.48%) | 78.54% | +3 −2 AI, +4 −1 real |
| mean | 72.41% | 45.21% | 12 of 484 (2.48%) | 78.93% | +4 −2 AI, +4 −1 real |
| trimmed mean | 69.35% | 47.51% | 13 of 484 (2.69%) | 77.39% | +4 −6 AI, +7 −3 real |
| median | 61.69% | 49.04% | 17 of 484 (3.51%) | 76.25% | +6 −11 AI, +11 −3 real |
| top 2 | 60.15% | 39.08% | 13 of 484 (2.69%) | 77.39% | +8 −10 AI, +7 −3 real |
| top 3 | 60.54% | 43.68% | 19 of 484 (3.93%) | 77.01% | +7 −10 AI, +12 −2 real |
| max | 44.83% | 21.84% | 16 of 484 (3.31%) | 69.73% | +7 −29 AI, +13 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 73.95% | 43.68% | 9 of 484 (1.86%) | 77.78% | +2 −3 AI, +2 −2 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 2.12 | 69.35% | 50.57% | 9 of 484 (1.86%) | 78.16% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 2.09 | 68.97% | 57.85% | 8 of 484 (1.65%) | 78.54% | +2 −1 AI, +0 −1 real |
| mean | 4,1,0,3,2,5 | 1.99 | 64.37% | 29.89% | 10 of 484 (2.07%) | 78.16% | +6 −6 AI, +2 −1 real |
| mean | 0,2,1,3,4,5 | 1.98 | 64.37% | 20.69% | 11 of 484 (2.27%) | 79.69% | +5 −1 AI, +2 −0 real |
| mean | 2,1,3,4,5,0 | 3.07 | 64.75% | 51.34% | 12 of 484 (2.48%) | 78.54% | +3 −2 AI, +4 −1 real |
| trimmed mean | 2,0,1,3,4,5 | 3.50 | 68.58% | 51.72% | 13 of 484 (2.69%) | 77.01% | +4 −7 AI, +6 −2 real |
| trimmed mean | 0,1,4,3,2,5 | 3.74 | 69.73% | 43.68% | 12 of 484 (2.48%) | 77.78% | +5 −6 AI, +6 −3 real |
| trimmed mean | 0,2,1,3,4,5 | 3.50 | 68.58% | 51.72% | 13 of 484 (2.69%) | 77.01% | +4 −7 AI, +6 −2 real |
| trimmed mean | 2,1,3,4,5,0 | 3.91 | 63.98% | 36.78% | 14 of 484 (2.89%) | 77.39% | +4 −6 AI, +7 −2 real |
| median | 2,0,1,3,4,5 | 1.64 | 47.89% | 39.08% | 11 of 484 (2.27%) | 77.39% | +8 −10 AI, +5 −3 real |
| median | 4,3,1,5,0,2 | 2.44 | 59.77% | 29.89% | 18 of 484 (3.72%) | 76.63% | +7 −11 AI, +12 −3 real |
| median | 0,2,1,3,4,5 | 2.98 | 62.84% | 46.36% | 12 of 484 (2.48%) | 76.63% | +7 −11 AI, +5 −2 real |
| median | 2,1,3,4,5,0 | 2.43 | 62.45% | 45.98% | 18 of 484 (3.72%) | 76.25% | +6 −11 AI, +10 −1 real |
| top 2 | 2,0,1,3,4,5 | 4.38 | 68.20% | 39.08% | 12 of 484 (2.48%) | 77.39% | +8 −10 AI, +6 −3 real |
| top 2 | 0,1,3,4,5,2 | 4.35 | 68.20% | 39.08% | 11 of 484 (2.27%) | 77.39% | +8 −10 AI, +5 −3 real |
| top 2 | 0,2,1,3,4,5 | 4.38 | 68.20% | 39.08% | 12 of 484 (2.48%) | 77.39% | +8 −10 AI, +6 −3 real |
| top 2 | 2,1,3,4,5,0 | 4.48 | 59.77% | 39.08% | 13 of 484 (2.69%) | 77.01% | +7 −10 AI, +7 −3 real |
| top 3 | 2,0,1,3,4,5 | 4.22 | 63.22% | 45.98% | 17 of 484 (3.51%) | 77.01% | +7 −10 AI, +10 −2 real |
| top 3 | 0,1,4,3,5,2 | 4.17 | 63.22% | 44.83% | 17 of 484 (3.51%) | 77.01% | +7 −10 AI, +10 −2 real |
| top 3 | 0,2,1,3,4,5 | 4.22 | 63.22% | 45.98% | 17 of 484 (3.51%) | 77.01% | +7 −10 AI, +10 −2 real |
| top 3 | 2,1,3,4,5,0 | 4.43 | 60.54% | 43.68% | 19 of 484 (3.93%) | 77.01% | +7 −10 AI, +12 −2 real |
| max | 2,0,1,3,4,5 | 4.70 | 48.28% | 25.29% | 14 of 484 (2.89%) | 69.73% | +7 −29 AI, +11 −6 real |
| max | 0,1,3,4,2,5 | 4.88 | 44.83% | 21.84% | 13 of 484 (2.69%) | 69.73% | +7 −29 AI, +10 −6 real |
| max | 0,2,1,3,4,5 | 4.70 | 48.28% | 25.29% | 14 of 484 (2.89%) | 69.73% | +7 −29 AI, +11 −6 real |
| max | 2,1,3,4,5,0 | 4.96 | 44.83% | 21.84% | 15 of 484 (3.10%) | 69.73% | +7 −29 AI, +12 −6 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,0,1,3,4,5 | 2.14 | 74.71% | 53.26% | 8 of 484 (1.65%) | 78.93% | +5 −3 AI, +1 −2 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 4,0,5,3,1,2 | 2.90 | 73.18% | 40.61% | 9 of 484 (1.86%) | 77.01% | +2 −5 AI, +2 −2 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 0,2,1,3,4,5 | 2.25 | 70.88% | 20.31% | 9 of 484 (1.86%) | 78.54% | +4 −3 AI, +2 −2 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,1,3,4,5,0 | 3.08 | 73.56% | 45.59% | 9 of 484 (1.86%) | 77.39% | +1 −3 AI, +2 −2 real |

### Test: wasitai-reposted (2229 items: 783 AI, 1446 real)

All 6 views of every image (the order does not matter). Flips are against the first row of the next table.

| Pool | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|
| shipped (mean, spec as is), all views | 41.38% | 20.95% | 4 of 1446 (0.28%) | 33.33% | +3 −14 AI, +0 −2 real |
| mean | 41.38% | 20.95% | 4 of 1446 (0.28%) | 34.10% | +8 −13 AI, +0 −2 real |
| trimmed mean | 40.36% | 20.43% | 6 of 1446 (0.41%) | 31.55% | +8 −33 AI, +2 −2 real |
| median | 38.95% | 17.88% | 6 of 1446 (0.41%) | 30.78% | +9 −40 AI, +2 −2 real |
| top 2 | 42.66% | 25.67% | 3 of 1446 (0.21%) | 32.57% | +35 −52 AI, +0 −3 real |
| top 3 | 44.06% | 21.07% | 5 of 1446 (0.35%) | 33.59% | +26 −35 AI, +1 −2 real |
| max | 31.55% | 11.62% | 10 of 1446 (0.69%) | 30.01% | +43 −80 AI, +8 −4 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 42.78% | 21.20% | 3 of 1446 (0.21%) | 35.38% | +16 −11 AI, +0 −3 real |

With early exit (what the detector would run). Flips are against the first row.

| Pool | Order | Views per image | Recall at 1% FPR | Recall at 0.1% FPR | Real flagged at the fitted cutoff | Recall at the fitted cutoff | Verdicts changed |
|---|---|---|---|---|---|---|---|
| shipped (mean, spec as is) | 2,0,1,3,4,5 | 2.02 | 39.85% | 24.27% | 6 of 1446 (0.41%) | 34.74% | +0 −0 AI, +0 −0 real |
| mean | 2,0,1,3,4,5 | 1.98 | 40.23% | 17.37% | 7 of 1446 (0.48%) | 36.14% | +15 −4 AI, +1 −0 real |
| mean | 4,1,0,3,2,5 | 1.91 | 41.38% | 25.29% | 5 of 1446 (0.35%) | 35.38% | +26 −21 AI, +1 −2 real |
| mean | 0,2,1,3,4,5 | 1.98 | 40.74% | 21.97% | 6 of 1446 (0.41%) | 37.68% | +24 −1 AI, +0 −0 real |
| mean | 2,1,3,4,5,0 | 2.57 | 41.12% | 13.92% | 4 of 1446 (0.28%) | 34.10% | +9 −14 AI, +0 −2 real |
| trimmed mean | 2,0,1,3,4,5 | 3.44 | 43.30% | 21.20% | 6 of 1446 (0.41%) | 32.31% | +9 −28 AI, +2 −2 real |
| trimmed mean | 0,1,4,3,2,5 | 3.68 | 40.23% | 20.43% | 6 of 1446 (0.41%) | 31.42% | +8 −34 AI, +2 −2 real |
| trimmed mean | 0,2,1,3,4,5 | 3.44 | 43.30% | 21.20% | 6 of 1446 (0.41%) | 32.31% | +9 −28 AI, +2 −2 real |
| trimmed mean | 2,1,3,4,5,0 | 3.70 | 40.36% | 20.43% | 6 of 1446 (0.41%) | 31.55% | +8 −33 AI, +2 −2 real |
| median | 2,0,1,3,4,5 | 1.56 | 42.40% | 11.49% | 9 of 1446 (0.62%) | 36.02% | +32 −22 AI, +4 −1 real |
| median | 4,3,1,5,0,2 | 2.20 | 37.42% | 14.43% | 6 of 1446 (0.41%) | 30.78% | +11 −42 AI, +2 −2 real |
| median | 0,2,1,3,4,5 | 2.91 | 38.83% | 19.41% | 6 of 1446 (0.41%) | 30.78% | +9 −40 AI, +2 −2 real |
| median | 2,1,3,4,5,0 | 2.14 | 36.02% | 13.54% | 8 of 1446 (0.55%) | 29.37% | +8 −50 AI, +3 −1 real |
| top 2 | 2,0,1,3,4,5 | 3.77 | 43.17% | 25.67% | 3 of 1446 (0.21%) | 32.57% | +35 −52 AI, +0 −3 real |
| top 2 | 0,1,3,4,5,2 | 3.69 | 43.17% | 25.67% | 3 of 1446 (0.21%) | 32.57% | +35 −52 AI, +0 −3 real |
| top 2 | 0,2,1,3,4,5 | 3.77 | 43.17% | 25.67% | 3 of 1446 (0.21%) | 32.57% | +35 −52 AI, +0 −3 real |
| top 2 | 2,1,3,4,5,0 | 3.54 | 42.91% | 25.67% | 3 of 1446 (0.21%) | 32.44% | +34 −52 AI, +0 −3 real |
| top 3 | 2,0,1,3,4,5 | 3.91 | 43.81% | 21.07% | 5 of 1446 (0.35%) | 33.59% | +26 −35 AI, +1 −2 real |
| top 3 | 0,1,4,3,5,2 | 3.86 | 43.81% | 21.07% | 5 of 1446 (0.35%) | 33.59% | +26 −35 AI, +1 −2 real |
| top 3 | 0,2,1,3,4,5 | 3.91 | 43.81% | 21.07% | 5 of 1446 (0.35%) | 33.59% | +26 −35 AI, +1 −2 real |
| top 3 | 2,1,3,4,5,0 | 3.89 | 44.70% | 20.95% | 5 of 1446 (0.35%) | 33.33% | +24 −35 AI, +1 −2 real |
| max | 2,0,1,3,4,5 | 4.04 | 32.31% | 11.62% | 9 of 1446 (0.62%) | 30.01% | +43 −80 AI, +7 −4 real |
| max | 0,1,3,4,2,5 | 4.18 | 34.87% | 11.62% | 8 of 1446 (0.55%) | 30.01% | +43 −80 AI, +6 −4 real |
| max | 0,2,1,3,4,5 | 4.04 | 32.31% | 11.62% | 9 of 1446 (0.62%) | 30.01% | +43 −80 AI, +7 −4 real |
| max | 2,1,3,4,5,0 | 4.01 | 32.06% | 11.49% | 9 of 1446 (0.62%) | 29.76% | +41 −80 AI, +7 −4 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,0,1,3,4,5 | 2.03 | 43.04% | 23.24% | 5 of 1446 (0.35%) | 38.44% | +30 −1 AI, +0 −1 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 4,0,5,3,1,2 | 2.67 | 42.78% | 22.73% | 3 of 1446 (0.21%) | 35.50% | +17 −11 AI, +0 −3 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 0,2,1,3,4,5 | 2.23 | 44.06% | 23.50% | 5 of 1446 (0.35%) | 38.44% | +31 −2 AI, +0 −1 real |
| fitted weights 0.241/0.152/0.133/0.149/0.161/0.164 | 2,1,3,4,5,0 | 2.62 | 42.02% | 21.33% | 3 of 1446 (0.21%) | 35.38% | +16 −11 AI, +0 −3 real |
