```
$ tintype report --operating-points --system control=confirm7.control.tsv@0.5 --system control-seed2=confirm7.control-seed2.tsv@0.5 --system family=confirm7.family.tsv@0.5 --system source=confirm7.source.tsv@0.5 --fpr 0.001,0.01
```

Confirmation batch 7, every view, on the unpacked archive. Read the two empirical rows: the "frozen cutoff" row is each model at its frozen-split 1% cutoff, which was not fitted for these files.

23052 items scored by all 4 systems: 12305 AI, 10747 real
Recall intervals: 95% bootstrap over items.
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**control**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.937519 | 10 of 10747 | 0.09% | 0.16% | 64.56% (63.67%–65.36%) | 87.51% |
| 1% empirical FPR | 0.638380 | 107 of 10747 | 1.00% | 1.17% | 88.58% (88.00%–89.12%) | 47.33% |
| frozen cutoff | 0.500000 | 184 of 10747 | 1.71% | 1.93% | 91.99% (91.53%–92.45%) | 35.18% |

control by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| aigibench-blip-b3 | 400 | 400 | 100.00% | 5 of 400 | 1.25% | 1.000 |
| aigibench-dalle-3-b3 | 400 | 365 | 99.50% | 1 of 365 | 0.27% | 1.000 |
| aigibench-flux1-dev-b3 | 400 | 366 | 99.25% | 5 of 366 | 1.37% | 0.999 |
| aigibench-glide-b3 | 397 | 388 | 96.98% | 3 of 388 | 0.77% | 0.998 |
| aigibench-imagen3-b3 | 400 | 374 | 93.75% | 5 of 374 | 1.34% | 0.997 |
| aigibench-infinite-id-b3 | 400 | 381 | 100.00% | 6 of 381 | 1.57% | 1.000 |
| aigibench-instantid-b3 | 399 | 368 | 99.25% | 2 of 368 | 0.54% | 0.999 |
| aigibench-ip-adapter-b3 | 400 | 373 | 91.00% | 4 of 373 | 1.07% | 0.995 |
| aigibench-midjourney-b3 | 394 | 313 | 98.98% | 1 of 313 | 0.32% | 1.000 |
| aigibench-photomaker-b3 | 399 | 373 | 75.69% | 1 of 373 | 0.27% | 0.996 |
| aigibench-progan-b3 | 400 | 399 | 98.50% | 31 of 399 | 7.77% | 0.998 |
| aigibench-r3gan-b3 | 400 | 365 | 96.75% | 2 of 365 | 0.55% | 0.998 |
| aigibench-sd3-b3 | 400 | 383 | 100.00% | 3 of 383 | 0.78% | 1.000 |
| aigibench-sdxl-b3 | 400 | 0 | 100.00% | – | – | – |
| aigibench-stylegan-xl-b3 | 400 | 369 | 94.25% | 0 of 369 | 0.00% | 0.999 |
| aigibench-stylegan3-b3 | 400 | 382 | 92.50% | 3 of 382 | 0.79% | 0.998 |
| aigibench-styleswim-b3 | 400 | 369 | 86.75% | 2 of 369 | 0.54% | 0.997 |
| aigibench-wfir-b3 | 200 | 200 | 96.50% | 1 of 200 | 0.50% | 0.999 |
| classic-biggan-b2 | 300 | 300 | 99.67% | 10 of 300 | 3.33% | 1.000 |
| classic-crn-b2 | 300 | 300 | 98.67% | 26 of 300 | 8.67% | 0.997 |
| classic-cyclegan-b2 | 300 | 300 | 98.67% | 8 of 300 | 2.67% | 0.998 |
| classic-deepfake-b2 | 300 | 300 | 14.33% | 1 of 300 | 0.33% | 0.855 |
| classic-gaugan-b2 | 300 | 300 | 99.00% | 1 of 300 | 0.33% | 1.000 |
| classic-imle-b2 | 300 | 300 | 100.00% | 13 of 300 | 4.33% | 1.000 |
| classic-san-b2 | 219 | 200 | 78.54% | 5 of 200 | 2.50% | 0.963 |
| classic-seeingdark-b2 | 180 | 180 | 88.33% | 0 of 180 | 0.00% | 0.991 |
| classic-stargan-b2 | 300 | 300 | 99.00% | 0 of 300 | 0.00% | 1.000 |
| classic-stylegan-b2 | 300 | 292 | 93.33% | 6 of 292 | 2.05% | 0.987 |
| classic-stylegan2-b2 | 300 | 282 | 90.33% | 6 of 282 | 2.13% | 0.991 |
| sofake-ood-fresh6 | 2217 | 1525 | 87.24% | 33 of 1525 | 2.16% | 0.982 |

Worst dataset slice with at least 100 items of the class: recall 14.33% (classic-deepfake-b2); FPR 8.67% (classic-crn-b2).

**control-seed2**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.937399 | 10 of 10747 | 0.09% | 0.16% | 62.72% (61.90%–63.62%) | 87.19% |
| 1% empirical FPR | 0.632599 | 107 of 10747 | 1.00% | 1.17% | 88.83% (88.26%–89.36%) | 47.40% |
| frozen cutoff | 0.500000 | 194 of 10747 | 1.81% | 2.03% | 92.18% (91.72%–92.64%) | 34.03% |

control-seed2 by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| aigibench-blip-b3 | 400 | 400 | 100.00% | 6 of 400 | 1.50% | 1.000 |
| aigibench-dalle-3-b3 | 400 | 365 | 99.50% | 1 of 365 | 0.27% | 0.999 |
| aigibench-flux1-dev-b3 | 400 | 366 | 99.25% | 5 of 366 | 1.37% | 0.999 |
| aigibench-glide-b3 | 397 | 388 | 96.22% | 3 of 388 | 0.77% | 0.998 |
| aigibench-imagen3-b3 | 400 | 374 | 94.25% | 5 of 374 | 1.34% | 0.997 |
| aigibench-infinite-id-b3 | 400 | 381 | 100.00% | 2 of 381 | 0.52% | 1.000 |
| aigibench-instantid-b3 | 399 | 368 | 100.00% | 2 of 368 | 0.54% | 0.999 |
| aigibench-ip-adapter-b3 | 400 | 373 | 93.25% | 4 of 373 | 1.07% | 0.996 |
| aigibench-midjourney-b3 | 394 | 313 | 98.73% | 1 of 313 | 0.32% | 1.000 |
| aigibench-photomaker-b3 | 399 | 373 | 80.45% | 1 of 373 | 0.27% | 0.997 |
| aigibench-progan-b3 | 400 | 399 | 98.50% | 29 of 399 | 7.27% | 0.997 |
| aigibench-r3gan-b3 | 400 | 365 | 96.50% | 2 of 365 | 0.55% | 0.998 |
| aigibench-sd3-b3 | 400 | 383 | 100.00% | 3 of 383 | 0.78% | 1.000 |
| aigibench-sdxl-b3 | 400 | 0 | 100.00% | – | – | – |
| aigibench-stylegan-xl-b3 | 400 | 369 | 92.75% | 3 of 369 | 0.81% | 0.998 |
| aigibench-stylegan3-b3 | 400 | 382 | 88.50% | 2 of 382 | 0.52% | 0.998 |
| aigibench-styleswim-b3 | 400 | 369 | 86.75% | 1 of 369 | 0.27% | 0.997 |
| aigibench-wfir-b3 | 200 | 200 | 95.00% | 1 of 200 | 0.50% | 0.999 |
| classic-biggan-b2 | 300 | 300 | 99.33% | 10 of 300 | 3.33% | 1.000 |
| classic-crn-b2 | 300 | 300 | 98.00% | 23 of 300 | 7.67% | 0.995 |
| classic-cyclegan-b2 | 300 | 300 | 98.33% | 9 of 300 | 3.00% | 0.998 |
| classic-deepfake-b2 | 300 | 300 | 24.00% | 2 of 300 | 0.67% | 0.843 |
| classic-gaugan-b2 | 300 | 300 | 98.67% | 2 of 300 | 0.67% | 0.999 |
| classic-imle-b2 | 300 | 300 | 100.00% | 17 of 300 | 5.67% | 1.000 |
| classic-san-b2 | 219 | 200 | 77.63% | 5 of 200 | 2.50% | 0.964 |
| classic-seeingdark-b2 | 180 | 180 | 88.33% | 0 of 180 | 0.00% | 0.992 |
| classic-stargan-b2 | 300 | 300 | 98.67% | 0 of 300 | 0.00% | 1.000 |
| classic-stylegan-b2 | 300 | 292 | 92.67% | 10 of 292 | 3.42% | 0.984 |
| classic-stylegan2-b2 | 300 | 282 | 82.67% | 6 of 282 | 2.13% | 0.980 |
| sofake-ood-fresh6 | 2217 | 1525 | 88.36% | 39 of 1525 | 2.56% | 0.982 |

Worst dataset slice with at least 100 items of the class: recall 24.00% (classic-deepfake-b2); FPR 7.67% (classic-crn-b2).

**family**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.939963 | 10 of 10747 | 0.09% | 0.16% | 67.73% (66.93%–68.60%) | 88.03% |
| 1% empirical FPR | 0.673051 | 107 of 10747 | 1.00% | 1.17% | 88.48% (87.92%–89.01%) | 47.30% |
| frozen cutoff | 0.500000 | 215 of 10747 | 2.00% | 2.24% | 92.37% (91.92%–92.87%) | 31.80% |

family by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| aigibench-blip-b3 | 400 | 400 | 100.00% | 5 of 400 | 1.25% | 1.000 |
| aigibench-dalle-3-b3 | 400 | 365 | 99.50% | 1 of 365 | 0.27% | 1.000 |
| aigibench-flux1-dev-b3 | 400 | 366 | 99.25% | 3 of 366 | 0.82% | 0.999 |
| aigibench-glide-b3 | 397 | 388 | 98.49% | 4 of 388 | 1.03% | 0.999 |
| aigibench-imagen3-b3 | 400 | 374 | 96.00% | 4 of 374 | 1.07% | 0.998 |
| aigibench-infinite-id-b3 | 400 | 381 | 100.00% | 5 of 381 | 1.31% | 1.000 |
| aigibench-instantid-b3 | 399 | 368 | 99.25% | 2 of 368 | 0.54% | 0.999 |
| aigibench-ip-adapter-b3 | 400 | 373 | 89.50% | 6 of 373 | 1.61% | 0.995 |
| aigibench-midjourney-b3 | 394 | 313 | 98.73% | 1 of 313 | 0.32% | 1.000 |
| aigibench-photomaker-b3 | 399 | 373 | 62.16% | 1 of 373 | 0.27% | 0.991 |
| aigibench-progan-b3 | 400 | 399 | 99.25% | 33 of 399 | 8.27% | 0.998 |
| aigibench-r3gan-b3 | 400 | 365 | 95.75% | 3 of 365 | 0.82% | 0.997 |
| aigibench-sd3-b3 | 400 | 383 | 100.00% | 3 of 383 | 0.78% | 1.000 |
| aigibench-sdxl-b3 | 400 | 0 | 100.00% | – | – | – |
| aigibench-stylegan-xl-b3 | 400 | 369 | 93.75% | 3 of 369 | 0.81% | 0.998 |
| aigibench-stylegan3-b3 | 400 | 382 | 94.25% | 3 of 382 | 0.79% | 0.999 |
| aigibench-styleswim-b3 | 400 | 369 | 90.25% | 1 of 369 | 0.27% | 0.998 |
| aigibench-wfir-b3 | 200 | 200 | 99.00% | 1 of 200 | 0.50% | 1.000 |
| classic-biggan-b2 | 300 | 300 | 99.67% | 11 of 300 | 3.67% | 1.000 |
| classic-crn-b2 | 300 | 300 | 99.67% | 28 of 300 | 9.33% | 0.997 |
| classic-cyclegan-b2 | 300 | 300 | 98.00% | 10 of 300 | 3.33% | 0.997 |
| classic-deepfake-b2 | 300 | 300 | 17.67% | 1 of 300 | 0.33% | 0.843 |
| classic-gaugan-b2 | 300 | 300 | 99.00% | 3 of 300 | 1.00% | 1.000 |
| classic-imle-b2 | 300 | 300 | 100.00% | 20 of 300 | 6.67% | 1.000 |
| classic-san-b2 | 219 | 200 | 83.11% | 5 of 200 | 2.50% | 0.967 |
| classic-seeingdark-b2 | 180 | 180 | 89.44% | 2 of 180 | 1.11% | 0.989 |
| classic-stargan-b2 | 300 | 300 | 98.33% | 0 of 300 | 0.00% | 1.000 |
| classic-stylegan-b2 | 300 | 292 | 94.33% | 11 of 292 | 3.77% | 0.987 |
| classic-stylegan2-b2 | 300 | 282 | 90.33% | 9 of 282 | 3.19% | 0.989 |
| sofake-ood-fresh6 | 2217 | 1525 | 89.31% | 36 of 1525 | 2.36% | 0.985 |

Worst dataset slice with at least 100 items of the class: recall 17.67% (classic-deepfake-b2); FPR 9.33% (classic-crn-b2).

**source**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.925968 | 10 of 10747 | 0.09% | 0.16% | 63.60% (62.77%–64.51%) | 87.35% |
| 1% empirical FPR | 0.646854 | 107 of 10747 | 1.00% | 1.17% | 87.87% (87.27%–88.43%) | 47.13% |
| frozen cutoff | 0.500000 | 207 of 10747 | 1.93% | 2.16% | 92.02% (91.53%–92.50%) | 32.55% |

source by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| aigibench-blip-b3 | 400 | 400 | 100.00% | 7 of 400 | 1.75% | 1.000 |
| aigibench-dalle-3-b3 | 400 | 365 | 99.50% | 1 of 365 | 0.27% | 1.000 |
| aigibench-flux1-dev-b3 | 400 | 366 | 99.25% | 5 of 366 | 1.37% | 0.999 |
| aigibench-glide-b3 | 397 | 388 | 95.72% | 5 of 388 | 1.29% | 0.998 |
| aigibench-imagen3-b3 | 400 | 374 | 93.50% | 5 of 374 | 1.34% | 0.997 |
| aigibench-infinite-id-b3 | 400 | 381 | 100.00% | 6 of 381 | 1.57% | 1.000 |
| aigibench-instantid-b3 | 399 | 368 | 99.75% | 2 of 368 | 0.54% | 0.999 |
| aigibench-ip-adapter-b3 | 400 | 373 | 90.00% | 5 of 373 | 1.34% | 0.994 |
| aigibench-midjourney-b3 | 394 | 313 | 98.48% | 1 of 313 | 0.32% | 1.000 |
| aigibench-photomaker-b3 | 399 | 373 | 77.44% | 2 of 373 | 0.54% | 0.995 |
| aigibench-progan-b3 | 400 | 399 | 98.75% | 28 of 399 | 7.02% | 0.998 |
| aigibench-r3gan-b3 | 400 | 365 | 95.00% | 2 of 365 | 0.55% | 0.997 |
| aigibench-sd3-b3 | 400 | 383 | 100.00% | 3 of 383 | 0.78% | 1.000 |
| aigibench-sdxl-b3 | 400 | 0 | 100.00% | – | – | – |
| aigibench-stylegan-xl-b3 | 400 | 369 | 94.50% | 1 of 369 | 0.27% | 0.999 |
| aigibench-stylegan3-b3 | 400 | 382 | 91.25% | 3 of 382 | 0.79% | 0.999 |
| aigibench-styleswim-b3 | 400 | 369 | 91.25% | 1 of 369 | 0.27% | 0.998 |
| aigibench-wfir-b3 | 200 | 200 | 97.00% | 1 of 200 | 0.50% | 0.999 |
| classic-biggan-b2 | 300 | 300 | 99.67% | 6 of 300 | 2.00% | 1.000 |
| classic-crn-b2 | 300 | 300 | 98.67% | 33 of 300 | 11.00% | 0.995 |
| classic-cyclegan-b2 | 300 | 300 | 97.67% | 7 of 300 | 2.33% | 0.998 |
| classic-deepfake-b2 | 300 | 300 | 23.33% | 7 of 300 | 2.33% | 0.795 |
| classic-gaugan-b2 | 300 | 300 | 98.67% | 1 of 300 | 0.33% | 0.999 |
| classic-imle-b2 | 300 | 300 | 100.00% | 18 of 300 | 6.00% | 1.000 |
| classic-san-b2 | 219 | 200 | 75.80% | 6 of 200 | 3.00% | 0.967 |
| classic-seeingdark-b2 | 180 | 180 | 88.89% | 0 of 180 | 0.00% | 0.994 |
| classic-stargan-b2 | 300 | 300 | 98.67% | 0 of 300 | 0.00% | 1.000 |
| classic-stylegan-b2 | 300 | 292 | 92.67% | 11 of 292 | 3.77% | 0.986 |
| classic-stylegan2-b2 | 300 | 282 | 85.00% | 6 of 282 | 2.13% | 0.984 |
| sofake-ood-fresh6 | 2217 | 1525 | 87.19% | 34 of 1525 | 2.23% | 0.981 |

Worst dataset slice with at least 100 items of the class: recall 23.33% (classic-deepfake-b2); FPR 11.00% (classic-crn-b2).
