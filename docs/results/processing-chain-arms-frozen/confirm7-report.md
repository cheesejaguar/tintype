23052 items scored by all 5 systems: 12305 AI, 10747 real
Recall intervals: 95% bootstrap over items.
Precision is at 1.00% prevalence. FPR bound: one-sided 95% Clopper–Pearson, items taken as independent.

**released**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.946158 | 10 of 10747 | 0.09% | 0.16% | 63.58% (62.72%–64.43%) | 87.35% |
| 1% empirical FPR | 0.549794 | 107 of 10747 | 1.00% | 1.17% | 89.00% (88.45%–89.54%) | 47.45% |
| frozen cutoff | 0.500000 | 127 of 10747 | 1.18% | 1.37% | 90.18% (89.64%–90.71%) | 43.53% |

released by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| aigibench-blip-b3 | 400 | 400 | 100.00% | 4 of 400 | 1.00% | 1.000 |
| aigibench-dalle-3-b3 | 400 | 365 | 99.25% | 1 of 365 | 0.27% | 1.000 |
| aigibench-flux1-dev-b3 | 400 | 366 | 99.25% | 3 of 366 | 0.82% | 0.999 |
| aigibench-glide-b3 | 397 | 388 | 95.72% | 2 of 388 | 0.52% | 0.998 |
| aigibench-imagen3-b3 | 400 | 374 | 92.75% | 4 of 374 | 1.07% | 0.997 |
| aigibench-infinite-id-b3 | 400 | 381 | 100.00% | 2 of 381 | 0.52% | 1.000 |
| aigibench-instantid-b3 | 399 | 368 | 99.25% | 2 of 368 | 0.54% | 0.999 |
| aigibench-ip-adapter-b3 | 400 | 373 | 87.25% | 4 of 373 | 1.07% | 0.995 |
| aigibench-midjourney-b3 | 394 | 313 | 97.46% | 1 of 313 | 0.32% | 1.000 |
| aigibench-photomaker-b3 | 399 | 373 | 67.92% | 1 of 373 | 0.27% | 0.995 |
| aigibench-progan-b3 | 400 | 399 | 98.00% | 24 of 399 | 6.02% | 0.998 |
| aigibench-r3gan-b3 | 400 | 365 | 95.75% | 1 of 365 | 0.27% | 0.998 |
| aigibench-sd3-b3 | 400 | 383 | 100.00% | 3 of 383 | 0.78% | 1.000 |
| aigibench-sdxl-b3 | 400 | 0 | 100.00% | – | – | – |
| aigibench-stylegan-xl-b3 | 400 | 369 | 91.75% | 0 of 369 | 0.00% | 0.999 |
| aigibench-stylegan3-b3 | 400 | 382 | 86.75% | 3 of 382 | 0.79% | 0.998 |
| aigibench-styleswim-b3 | 400 | 369 | 82.50% | 2 of 369 | 0.54% | 0.997 |
| aigibench-wfir-b3 | 200 | 200 | 94.50% | 1 of 200 | 0.50% | 0.999 |
| classic-biggan-b2 | 300 | 300 | 99.67% | 6 of 300 | 2.00% | 1.000 |
| classic-crn-b2 | 300 | 300 | 98.00% | 10 of 300 | 3.33% | 0.997 |
| classic-cyclegan-b2 | 300 | 300 | 98.33% | 8 of 300 | 2.67% | 0.998 |
| classic-deepfake-b2 | 300 | 300 | 14.00% | 1 of 300 | 0.33% | 0.843 |
| classic-gaugan-b2 | 300 | 300 | 98.33% | 0 of 300 | 0.00% | 1.000 |
| classic-imle-b2 | 300 | 300 | 100.00% | 4 of 300 | 1.33% | 1.000 |
| classic-san-b2 | 219 | 200 | 73.06% | 4 of 200 | 2.00% | 0.961 |
| classic-seeingdark-b2 | 180 | 180 | 82.22% | 0 of 180 | 0.00% | 0.990 |
| classic-stargan-b2 | 300 | 300 | 98.33% | 0 of 300 | 0.00% | 1.000 |
| classic-stylegan-b2 | 300 | 292 | 92.67% | 5 of 292 | 1.71% | 0.988 |
| classic-stylegan2-b2 | 300 | 282 | 86.33% | 4 of 282 | 1.42% | 0.991 |
| sofake-ood-fresh6 | 2217 | 1525 | 84.75% | 27 of 1525 | 1.77% | 0.982 |

Worst dataset slice with at least 100 items of the class: recall 14.00% (classic-deepfake-b2); FPR 6.02% (aigibench-progan-b3).

**baseline-s1**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.937519 | 10 of 10747 | 0.09% | 0.16% | 64.56% (63.67%–65.36%) | 87.51% |
| 1% empirical FPR | 0.638380 | 107 of 10747 | 1.00% | 1.17% | 88.58% (88.00%–89.12%) | 47.33% |
| frozen cutoff | 0.500000 | 184 of 10747 | 1.71% | 1.93% | 91.99% (91.53%–92.45%) | 35.18% |

baseline-s1 by dataset, at the frozen cutoff 0.5:

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

**baseline-s2**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.937399 | 10 of 10747 | 0.09% | 0.16% | 62.72% (61.90%–63.62%) | 87.19% |
| 1% empirical FPR | 0.632599 | 107 of 10747 | 1.00% | 1.17% | 88.83% (88.26%–89.36%) | 47.40% |
| frozen cutoff | 0.500000 | 194 of 10747 | 1.81% | 2.03% | 92.18% (91.72%–92.64%) | 34.03% |

baseline-s2 by dataset, at the frozen cutoff 0.5:

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

**chains-s1**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.982357 | 10 of 10747 | 0.09% | 0.16% | 57.91% (57.05%–58.78%) | 86.28% |
| 1% empirical FPR | 0.796200 | 107 of 10747 | 1.00% | 1.17% | 84.88% (84.27%–85.55%) | 46.27% |
| frozen cutoff | 0.500000 | 319 of 10747 | 2.97% | 3.25% | 93.24% (92.77%–93.66%) | 24.09% |

chains-s1 by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| aigibench-blip-b3 | 400 | 400 | 100.00% | 7 of 400 | 1.75% | 1.000 |
| aigibench-dalle-3-b3 | 400 | 365 | 100.00% | 1 of 365 | 0.27% | 0.999 |
| aigibench-flux1-dev-b3 | 400 | 366 | 99.25% | 7 of 366 | 1.91% | 0.999 |
| aigibench-glide-b3 | 397 | 388 | 96.47% | 8 of 388 | 2.06% | 0.997 |
| aigibench-imagen3-b3 | 400 | 374 | 96.00% | 6 of 374 | 1.60% | 0.997 |
| aigibench-infinite-id-b3 | 400 | 381 | 100.00% | 8 of 381 | 2.10% | 1.000 |
| aigibench-instantid-b3 | 399 | 368 | 99.75% | 5 of 368 | 1.36% | 1.000 |
| aigibench-ip-adapter-b3 | 400 | 373 | 92.50% | 9 of 373 | 2.41% | 0.993 |
| aigibench-midjourney-b3 | 394 | 313 | 99.24% | 1 of 313 | 0.32% | 1.000 |
| aigibench-photomaker-b3 | 399 | 373 | 79.95% | 4 of 373 | 1.07% | 0.994 |
| aigibench-progan-b3 | 400 | 399 | 99.25% | 42 of 399 | 10.53% | 0.996 |
| aigibench-r3gan-b3 | 400 | 365 | 94.50% | 5 of 365 | 1.37% | 0.995 |
| aigibench-sd3-b3 | 400 | 383 | 100.00% | 4 of 383 | 1.04% | 1.000 |
| aigibench-sdxl-b3 | 400 | 0 | 100.00% | – | – | – |
| aigibench-stylegan-xl-b3 | 400 | 369 | 91.75% | 4 of 369 | 1.08% | 0.996 |
| aigibench-stylegan3-b3 | 400 | 382 | 91.50% | 1 of 382 | 0.26% | 0.998 |
| aigibench-styleswim-b3 | 400 | 369 | 91.50% | 3 of 369 | 0.81% | 0.996 |
| aigibench-wfir-b3 | 200 | 200 | 95.50% | 1 of 200 | 0.50% | 0.998 |
| classic-biggan-b2 | 300 | 300 | 99.33% | 10 of 300 | 3.33% | 1.000 |
| classic-crn-b2 | 300 | 300 | 98.33% | 21 of 300 | 7.00% | 0.994 |
| classic-cyclegan-b2 | 300 | 300 | 95.00% | 12 of 300 | 4.00% | 0.993 |
| classic-deepfake-b2 | 300 | 300 | 51.00% | 18 of 300 | 6.00% | 0.865 |
| classic-gaugan-b2 | 300 | 300 | 98.00% | 5 of 300 | 1.67% | 0.997 |
| classic-imle-b2 | 300 | 300 | 100.00% | 16 of 300 | 5.33% | 1.000 |
| classic-san-b2 | 219 | 200 | 83.11% | 14 of 200 | 7.00% | 0.951 |
| classic-seeingdark-b2 | 180 | 180 | 85.56% | 0 of 180 | 0.00% | 0.986 |
| classic-stargan-b2 | 300 | 300 | 98.00% | 2 of 300 | 0.67% | 0.999 |
| classic-stylegan-b2 | 300 | 292 | 91.33% | 18 of 292 | 6.16% | 0.973 |
| classic-stylegan2-b2 | 300 | 282 | 84.33% | 14 of 282 | 4.96% | 0.973 |
| sofake-ood-fresh6 | 2217 | 1525 | 89.49% | 73 of 1525 | 4.79% | 0.976 |

Worst dataset slice with at least 100 items of the class: recall 51.00% (classic-deepfake-b2); FPR 10.53% (aigibench-progan-b3).

**chains-s2**

| Operating point | Cutoff | Real items flagged | FPR | FPR upper bound | Recall | Precision at prevalence |
|---|---|---|---|---|---|---|
| 0.1% empirical FPR | 0.986655 | 10 of 10747 | 0.09% | 0.16% | 57.82% (56.98%–58.67%) | 86.26% |
| 1% empirical FPR | 0.818800 | 107 of 10747 | 1.00% | 1.17% | 84.40% (83.78%–85.04%) | 46.13% |
| frozen cutoff | 0.500000 | 338 of 10747 | 3.15% | 3.44% | 93.22% (92.79%–93.65%) | 23.04% |

chains-s2 by dataset, at the frozen cutoff 0.5:

| dataset | AI | Real | Recall | Real items flagged | FPR | AUROC |
|---|---|---|---|---|---|---|
| aigibench-blip-b3 | 400 | 400 | 100.00% | 8 of 400 | 2.00% | 1.000 |
| aigibench-dalle-3-b3 | 400 | 365 | 100.00% | 1 of 365 | 0.27% | 0.999 |
| aigibench-flux1-dev-b3 | 400 | 366 | 99.25% | 5 of 366 | 1.37% | 0.999 |
| aigibench-glide-b3 | 397 | 388 | 95.21% | 7 of 388 | 1.80% | 0.997 |
| aigibench-imagen3-b3 | 400 | 374 | 95.50% | 6 of 374 | 1.60% | 0.997 |
| aigibench-infinite-id-b3 | 400 | 381 | 100.00% | 8 of 381 | 2.10% | 1.000 |
| aigibench-instantid-b3 | 399 | 368 | 99.75% | 6 of 368 | 1.63% | 0.999 |
| aigibench-ip-adapter-b3 | 400 | 373 | 93.25% | 8 of 373 | 2.14% | 0.993 |
| aigibench-midjourney-b3 | 394 | 313 | 99.75% | 1 of 313 | 0.32% | 1.000 |
| aigibench-photomaker-b3 | 399 | 373 | 79.95% | 5 of 373 | 1.34% | 0.993 |
| aigibench-progan-b3 | 400 | 399 | 99.50% | 45 of 399 | 11.28% | 0.995 |
| aigibench-r3gan-b3 | 400 | 365 | 95.50% | 4 of 365 | 1.10% | 0.995 |
| aigibench-sd3-b3 | 400 | 383 | 100.00% | 6 of 383 | 1.57% | 1.000 |
| aigibench-sdxl-b3 | 400 | 0 | 100.00% | – | – | – |
| aigibench-stylegan-xl-b3 | 400 | 369 | 91.00% | 4 of 369 | 1.08% | 0.997 |
| aigibench-stylegan3-b3 | 400 | 382 | 91.50% | 2 of 382 | 0.52% | 0.998 |
| aigibench-styleswim-b3 | 400 | 369 | 93.25% | 2 of 369 | 0.54% | 0.997 |
| aigibench-wfir-b3 | 200 | 200 | 98.00% | 1 of 200 | 0.50% | 0.997 |
| classic-biggan-b2 | 300 | 300 | 99.67% | 14 of 300 | 4.67% | 0.999 |
| classic-crn-b2 | 300 | 300 | 99.00% | 31 of 300 | 10.33% | 0.994 |
| classic-cyclegan-b2 | 300 | 300 | 96.33% | 13 of 300 | 4.33% | 0.994 |
| classic-deepfake-b2 | 300 | 300 | 47.33% | 14 of 300 | 4.67% | 0.861 |
| classic-gaugan-b2 | 300 | 300 | 98.67% | 8 of 300 | 2.67% | 0.998 |
| classic-imle-b2 | 300 | 300 | 100.00% | 25 of 300 | 8.33% | 1.000 |
| classic-san-b2 | 219 | 200 | 79.45% | 10 of 200 | 5.00% | 0.952 |
| classic-seeingdark-b2 | 180 | 180 | 83.89% | 0 of 180 | 0.00% | 0.983 |
| classic-stargan-b2 | 300 | 300 | 98.00% | 1 of 300 | 0.33% | 0.999 |
| classic-stylegan-b2 | 300 | 292 | 92.00% | 23 of 292 | 7.88% | 0.975 |
| classic-stylegan2-b2 | 300 | 282 | 84.00% | 14 of 282 | 4.96% | 0.971 |
| sofake-ood-fresh6 | 2217 | 1525 | 89.40% | 66 of 1525 | 4.33% | 0.978 |

Worst dataset slice with at least 100 items of the class: recall 47.33% (classic-deepfake-b2); FPR 11.28% (aigibench-progan-b3).
