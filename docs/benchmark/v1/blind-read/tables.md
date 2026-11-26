
### Open split (the website's pool): 24,961 files

| Detector | Precision | Recall | Real flagged | Recall at 1% FPR |
|---|---|---|---|---|
| Tintype v0.4.0 | 98.95% (98.79–99.13) | 93.31% (92.90–93.76) | 1.16% (0.96–1.34) | 92.29% (91.02–93.51) |
| GPTTrace | 99.11% (98.92–99.28) | 74.09% (73.33–74.82) | 0.78% (0.63–0.94) | 76.51% (74.58–78.77) |
| Community Forensics | 95.24% (94.87–95.60) | 84.12% (83.50–84.73) | 4.94% (4.54–5.34) | 70.93% (69.41–72.46) |
| SwinV2 | 83.83% (83.12–84.56) | 48.84% (47.97–49.71) | 11.05% (10.49–11.62) | 28.27% (26.98–29.56) |

### Blind split: 23,843 files

| Detector | Precision | Recall | Real flagged | Recall at 1% FPR | ms per image |
|---|---|---|---|---|---|
| Tintype v0.4.0 | 98.98% (98.81–99.14) | 93.01% (92.58–93.43) | 1.20% (1.00–1.41) | 92.49% (91.38–93.18) | 7.7 |
| GPTTrace | 99.21% (99.03–99.37) | 74.93% (74.18–75.70) | 0.75% (0.59–0.92) | 78.41% (76.48–80.42) | 82.3 |
| Community Forensics | 95.25% (94.90–95.62) | 84.03% (83.41–84.67) | 5.24% (4.81–5.64) | 69.96% (68.15–71.59) | 5.3 |
| SwinV2 | 85.92% (85.20–86.65) | 49.71% (48.83–50.60) | 10.18% (9.61–10.77) | 28.94% (27.70–30.44) | 18.9 |

### Paired differences on the blind split (95% intervals)

| Pair | Precision | Recall | Real flagged | Recall at 1% FPR |
|---|---|---|---|---|
| Tintype v0.4.0 - GPTTrace | -0.46 to +0.01 | +17.27 to +18.88 | +0.20 to +0.70 | +11.76 to +16.11 |
| Tintype v0.4.0 - Community Forensics | +3.37 to +4.12 | +8.30 to +9.60 | -4.49 to -3.62 | +20.58 to +24.24 |
| Tintype v0.4.0 - SwinV2 | +12.33 to +13.77 | +42.35 to +44.28 | -9.59 to -8.40 | +61.66 to +64.96 |
| GPTTrace - Community Forensics | +3.56 to +4.36 | -10.10 to -8.12 | -4.93 to -4.05 | +5.99 to +10.97 |
| GPTTrace - SwinV2 | +12.58 to +13.98 | +24.33 to +26.19 | -10.02 to -8.85 | +47.08 to +51.68 |
| Community Forensics - SwinV2 | +8.53 to +10.12 | +33.20 to +35.51 | -5.66 to -4.26 | +38.59 to +43.07 |

### Blind minus open, all files (unpaired, 95% intervals)

| Detector | Precision | Recall | Real flagged | Recall at 1% FPR |
|---|---|---|---|---|
| Tintype v0.4.0 | 98.98 vs 98.95: +0.03 (-0.22 to +0.27) | 93.01 vs 93.31: -0.31 (-0.90 to +0.32) | 1.20 vs 1.16: +0.04 (-0.24 to +0.32) | 92.49 vs 92.29: +0.20 (-1.56 to +1.68) |
| GPTTrace | 99.21 vs 99.11: +0.10 (-0.17 to +0.35) | 74.93 vs 74.09: +0.84 (-0.22 to +1.90) | 0.75 vs 0.78: -0.04 (-0.26 to +0.21) | 78.41 vs 76.51: +1.90 (-1.23 to +4.60) |
| Community Forensics | 95.25 vs 95.24: +0.01 (-0.49 to +0.53) | 84.03 vs 84.12: -0.09 (-0.96 to +0.82) | 5.24 vs 4.94: +0.30 (-0.29 to +0.87) | 69.96 vs 70.93: -0.97 (-3.35 to +1.33) |
| SwinV2 | 85.92 vs 83.83: +2.08 (+1.06 to +3.14) | 49.71 vs 48.84: +0.87 (-0.32 to +2.05) | 10.18 vs 11.05: -0.87 (-1.70 to -0.08) | 28.94 vs 28.27: +0.67 (-1.13 to +2.75) |

### Blind minus open, the sets in both splits (unpaired, 95% intervals)

| Detector | Precision | Recall | Real flagged | Recall at 1% FPR |
|---|---|---|---|---|
| Tintype v0.4.0 | 98.98 vs 98.95: +0.03 (-0.22 to +0.27) | 93.01 vs 93.31: -0.31 (-0.90 to +0.32) | 1.20 vs 1.16: +0.04 (-0.24 to +0.32) | 92.49 vs 92.29: +0.20 (-1.56 to +1.68) |
| GPTTrace | 99.21 vs 99.11: +0.10 (-0.17 to +0.35) | 74.93 vs 74.09: +0.84 (-0.22 to +1.90) | 0.75 vs 0.78: -0.04 (-0.26 to +0.21) | 78.41 vs 76.51: +1.90 (-1.23 to +4.60) |
| Community Forensics | 95.25 vs 95.24: +0.01 (-0.49 to +0.53) | 84.03 vs 84.12: -0.09 (-0.96 to +0.82) | 5.24 vs 4.94: +0.30 (-0.29 to +0.87) | 69.96 vs 70.93: -0.97 (-3.35 to +1.33) |
| SwinV2 | 85.92 vs 83.83: +2.08 (+1.06 to +3.14) | 49.71 vs 48.84: +0.87 (-0.32 to +2.05) | 10.18 vs 11.05: -0.87 (-1.70 to -0.08) | 28.94 vs 28.27: +0.67 (-1.13 to +2.75) |

### Tintype per set: recall / real flagged, blind (open)

| Set | Generated | Real | Blind recall | Open recall | Blind real flagged | Open real flagged |
|---|---|---|---|---|---|---|
| aigibench-blip | 400 | 400 | 100.00 | 100.00 | 0.75 | 0.75 |
| aigibench-dalle-3 | 440 | 372 | 98.86 | 98.64 | 0.00 | 0.49 |
| aigibench-flux1-dev | 440 | 365 | 98.18 | 99.09 | 0.27 | 0.24 |
| aigibench-glide | 390 | 360 | 98.97 | 99.50 | 1.11 | 0.78 |
| aigibench-imagen3 | 450 | 373 | 94.44 | 90.89 | 0.27 | 1.22 |
| aigibench-infinite-id | 400 | 322 | 99.75 | 100.00 | 0.00 | 0.00 |
| aigibench-instantid | 400 | 335 | 98.75 | 99.00 | 0.30 | 0.54 |
| aigibench-ip-adapter | 400 | 327 | 89.75 | 91.00 | 0.92 | 1.09 |
| aigibench-midjourney | 199 | 87 | 95.98 | 95.98 | 0.00 | 0.00 |
| aigibench-photomaker | 400 | 330 | 73.50 | 77.75 | 0.00 | 0.54 |
| aigibench-progan | 450 | 447 | 99.78 | 99.56 | 5.59 | 4.25 |
| aigibench-r3gan | 400 | 322 | 99.25 | 99.25 | 0.62 | 0.82 |
| aigibench-sd3 | 450 | 386 | 99.78 | 99.78 | 1.04 | 0.24 |
| aigibench-sdxl | 450 | 0 | 100.00 | 100.00 | – | – |
| aigibench-stylegan-xl | 400 | 323 | 96.50 | 97.25 | 0.93 | 0.54 |
| aigibench-stylegan3 | 400 | 322 | 97.00 | 95.25 | 0.00 | 0.00 |
| aigibench-styleswim | 400 | 317 | 99.25 | 97.00 | 0.95 | 0.00 |
| classic-biggan | 540 | 540 | 100.00 | 100.00 | 0.74 | 0.56 |
| classic-crn | 600 | 600 | 99.00 | 98.83 | 5.50 | 6.33 |
| classic-cyclegan | 122 | 123 | 99.18 | 98.67 | 1.63 | 1.34 |
| classic-deepfake | 450 | 600 | 25.56 | 27.33 | 0.50 | 0.83 |
| classic-gaugan | 600 | 600 | 99.50 | 99.83 | 0.50 | 0.33 |
| classic-imle | 600 | 0 | 100.00 | 100.00 | – | – |
| classic-stargan | 540 | 540 | 99.81 | 100.00 | 0.00 | 0.00 |
| classic-stylegan | 600 | 560 | 95.17 | 94.83 | 2.32 | 1.77 |
| classic-stylegan2 | 828 | 681 | 94.08 | 93.72 | 1.62 | 1.23 |
| sofake-ood | 1495 | 967 | 82.27 | 84.36 | 0.83 | 1.46 |
