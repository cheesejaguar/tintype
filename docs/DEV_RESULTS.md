# Development-set results

Per-set tables for the three suites that were visible during development,
with the released weights. The thresholds were fitted with these sets' real
items in the calibration data, so the thresholded numbers are optimistic;
[BENCHMARK.md](BENCHMARK.md) has the blind batches. Video rows use the video
checkpoint.

These tables were made when the default photo threshold was the 0.5% target:
the rows labelled "Veracity" are what is now `--strict`, with all six views.
The same suites under the current default (1.5% target, early exit) are in
`results/benchmarks_new_default.md`, `results/confirm1_new_default.md` and
`results/confirm2_new_default.md`.

## Benchmark sets (plan `benchmarks`)

### Photos

**AIGIBench CommunityAI** (6000 AI, 6000 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.967 (0.961–0.973) | 0.554 (0.542–0.567) | 0.705 | 1.90% | 0.934 | 0.477 |
| SwinV2 detector | 0.920 (0.913–0.927) | 0.881 (0.873–0.889) | 0.900 | 7.65% | 0.966 | 0.686 |
| Veracity | 0.990 (0.986–0.993) | 0.505 (0.493–0.518) | 0.669 | 0.52% | 0.976 | 0.623 |
| Veracity (fast) | 0.990 (0.987–0.993) | 0.534 (0.521–0.548) | 0.694 | 0.55% | 0.972 | 0.602 |
| Veracity (sensitive) | 0.918 (0.911–0.925) | 0.929 (0.922–0.936) | 0.924 | 8.27% | 0.976 | 0.623 |

**So-Fake-OOD** (2912 AI, 2075 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.979 (0.970–0.987) | 0.401 (0.384–0.420) | 0.569 | 1.20% | 0.899 | 0.353 |
| SwinV2 detector | 0.944 (0.935–0.952) | 0.815 (0.801–0.829) | 0.875 | 6.75% | 0.940 | 0.620 |
| Veracity | 0.994 (0.990–0.997) | 0.723 (0.708–0.739) | 0.837 | 0.63% | 0.979 | 0.779 |
| Veracity (fast) | 0.995 (0.991–0.998) | 0.600 (0.581–0.617) | 0.748 | 0.43% | 0.961 | 0.692 |
| Veracity (sensitive) | 0.960 (0.953–0.967) | 0.923 (0.913–0.932) | 0.941 | 5.45% | 0.979 | 0.779 |

**Synthbuster+** (4679 AI, 1000 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.979 (0.974–0.983) | 0.773 (0.761–0.784) | 0.864 | 7.90% | 0.899 | 0.658 |
| SwinV2 detector | 0.959 (0.953–0.966) | 0.663 (0.650–0.677) | 0.784 | 13.30% | 0.840 | 0.310 |
| Veracity | 0.998 (0.996–0.999) | 0.827 (0.815–0.837) | 0.904 | 0.90% | 0.989 | 0.847 |
| Veracity (fast) | 0.998 (0.996–0.999) | 0.674 (0.660–0.687) | 0.804 | 0.70% | 0.972 | 0.733 |
| Veracity (sensitive) | 0.986 (0.983–0.989) | 0.961 (0.955–0.966) | 0.973 | 6.40% | 0.989 | 0.847 |

**Frontier 2026-10 images** (261 AI, 484 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.815 (0.748–0.883) | 0.387 (0.324–0.446) | 0.525 | 4.75% | 0.765 | 0.287 |
| SwinV2 detector | 0.951 (0.916–0.979) | 0.674 (0.617–0.729) | 0.789 | 1.86% | 0.926 | 0.506 |
| Veracity | 0.983 (0.960–1.000) | 0.655 (0.597–0.710) | 0.786 | 0.62% | 0.975 | 0.713 |
| Veracity (fast) | 0.993 (0.977–1.000) | 0.536 (0.481–0.597) | 0.697 | 0.21% | 0.970 | 0.732 |
| Veracity (sensitive) | 0.872 (0.832–0.911) | 0.912 (0.879–0.945) | 0.891 | 7.23% | 0.975 | 0.713 |

**Frontier 2026-10 images, Instagram copies** (261 AI, 482 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.911 (0.815–0.979) | 0.157 (0.117–0.203) | 0.268 | 0.83% | 0.671 | 0.161 |
| SwinV2 detector | 0.951 (0.909–0.984) | 0.444 (0.382–0.500) | 0.606 | 1.24% | 0.894 | 0.368 |
| Veracity | 1.000 (1.000–1.000) | 0.169 (0.120–0.215) | 0.289 | 0.00% | 0.925 | 0.402 |
| Veracity (fast) | 1.000 (1.000–1.000) | 0.188 (0.141–0.242) | 0.316 | 0.00% | 0.923 | 0.456 |
| Veracity (sensitive) | 0.909 (0.859–0.950) | 0.571 (0.515–0.631) | 0.701 | 3.11% | 0.925 | 0.402 |

**Frontier 2026-10 images, re-shared copies** (261 AI, 482 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.912 (0.805–1.000) | 0.119 (0.082–0.160) | 0.210 | 0.62% | 0.676 | 0.130 |
| SwinV2 detector | 0.937 (0.887–0.980) | 0.398 (0.340–0.456) | 0.559 | 1.45% | 0.894 | 0.364 |
| Veracity | 0.971 (0.905–1.000) | 0.126 (0.084–0.169) | 0.224 | 0.21% | 0.920 | 0.364 |
| Veracity (fast) | 1.000 (1.000–1.000) | 0.138 (0.096–0.184) | 0.242 | 0.00% | 0.925 | 0.437 |
| Veracity (sensitive) | 0.906 (0.858–0.950) | 0.552 (0.493–0.610) | 0.686 | 3.11% | 0.920 | 0.364 |

**Frontier 2026-10 images, WhatsApp copies** (261 AI, 482 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.929 (0.818–1.000) | 0.100 (0.065–0.139) | 0.180 | 0.41% | 0.680 | 0.146 |
| SwinV2 detector | 0.961 (0.919–0.990) | 0.375 (0.318–0.432) | 0.540 | 0.83% | 0.904 | 0.406 |
| Veracity | 0.968 (0.889–1.000) | 0.115 (0.076–0.158) | 0.205 | 0.21% | 0.927 | 0.368 |
| Veracity (fast) | 1.000 (1.000–1.000) | 0.126 (0.087–0.173) | 0.224 | 0.00% | 0.925 | 0.529 |
| Veracity (sensitive) | 0.939 (0.899–0.973) | 0.586 (0.525–0.641) | 0.722 | 2.07% | 0.927 | 0.368 |

**Pooled** (14635 AI, 11005 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.971 (0.967–0.974) | 0.568 (0.560–0.576) | 0.717 | 2.27% | 0.904 | 0.488 |
| SwinV2 detector | 0.937 (0.932–0.941) | 0.769 (0.763–0.776) | 0.845 | 6.89% | 0.927 | 0.530 |
| Veracity | 0.994 (0.992–0.995) | 0.634 (0.627–0.642) | 0.774 | 0.53% | 0.976 | 0.718 |
| Veracity (fast) | 0.994 (0.992–0.996) | 0.571 (0.563–0.579) | 0.726 | 0.45% | 0.966 | 0.651 |
| Veracity (sensitive) | 0.947 (0.944–0.951) | 0.918 (0.914–0.923) | 0.933 | 6.80% | 0.976 | 0.718 |

### Video

**ComGenVid (Sora, Veo 3)** (300 AI, 150 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.992 (0.980–1.000) | 0.837 (0.793–0.880) | 0.908 | 1.33% | 0.989 | 0.813 |
| Veracity | 0.993 (0.982–1.000) | 0.917 (0.883–0.946) | 0.953 | 1.33% | 0.994 | 0.910 |
| Veracity + second opinion | 0.996 (0.989–1.000) | 0.923 (0.890–0.951) | 0.958 | 0.67% | 0.996 | 0.983 |

**GenVideo-Val** (485 AI, 300 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.989 (0.979–0.998) | 0.895 (0.868–0.924) | 0.939 | 1.67% | 0.968 | 0.895 |
| Veracity | 0.993 (0.985–1.000) | 0.913 (0.888–0.938) | 0.952 | 1.00% | 0.990 | 0.926 |
| Veracity + second opinion | 0.996 (0.989–1.000) | 0.930 (0.906–0.952) | 0.962 | 0.67% | 0.990 | 0.936 |

**Frontier 2026-10 video** (101 AI, 100 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.930 (0.874–0.978) | 0.921 (0.863–0.971) | 0.925 | 7.00% | 0.984 | 0.881 |
| Veracity | 1.000 (1.000–1.000) | 0.762 (0.676–0.841) | 0.865 | 0.00% | 0.962 | 0.851 |
| Veracity + second opinion | 0.978 (0.944–1.000) | 0.881 (0.819–0.942) | 0.927 | 2.00% | 0.967 | 0.881 |

**Pooled** (886 AI, 550 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.982 (0.973–0.991) | 0.878 (0.855–0.901) | 0.927 | 2.55% | 0.976 | 0.835 |
| Veracity | 0.994 (0.988–0.999) | 0.897 (0.876–0.918) | 0.943 | 0.91% | 0.989 | 0.897 |
| Veracity + second opinion | 0.994 (0.988–0.999) | 0.922 (0.904–0.940) | 0.957 | 0.91% | 0.990 | 0.929 |

## Confirmation batch 1 (plan `confirm1`)

### Photos

**AIGIBench SocialRF** (3000 AI, 3000 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.972 (0.965–0.979) | 0.628 (0.611–0.645) | 0.763 | 1.80% | 0.938 | 0.510 |
| SwinV2 detector | 0.888 (0.878–0.899) | 0.948 (0.940–0.956) | 0.917 | 11.97% | 0.975 | 0.690 |
| Veracity | 0.981 (0.975–0.987) | 0.673 (0.656–0.690) | 0.798 | 1.30% | 0.972 | 0.649 |
| Veracity (fast) | 0.982 (0.976–0.987) | 0.667 (0.651–0.685) | 0.794 | 1.23% | 0.976 | 0.597 |
| Veracity (sensitive) | 0.904 (0.893–0.914) | 0.927 (0.917–0.937) | 0.915 | 9.87% | 0.972 | 0.649 |

**So-Fake-OOD, batch 1** (1507 AI, 953 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.989 (0.979–0.997) | 0.411 (0.386–0.435) | 0.581 | 0.73% | 0.925 | 0.475 |
| SwinV2 detector | 0.947 (0.935–0.959) | 0.812 (0.793–0.831) | 0.875 | 7.14% | 0.938 | 0.446 |
| Veracity | 0.995 (0.991–0.999) | 0.724 (0.701–0.746) | 0.838 | 0.52% | 0.987 | 0.847 |
| Veracity (fast) | 0.995 (0.989–0.999) | 0.612 (0.588–0.636) | 0.758 | 0.52% | 0.969 | 0.752 |
| Veracity (sensitive) | 0.967 (0.958–0.976) | 0.936 (0.923–0.948) | 0.951 | 5.04% | 0.987 | 0.847 |

**Synthbuster+, new images (AI only)** (1725 AI, 0 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | – | 0.788 (0.768–0.807) | – | – | – | – |
| SwinV2 detector | – | 0.653 (0.632–0.675) | – | – | – | – |
| Veracity | – | 0.667 (0.646–0.688) | – | – | – | – |
| Veracity (fast) | – | 0.482 (0.457–0.506) | – | – | – | – |
| Veracity (sensitive) | – | 0.910 (0.896–0.923) | – | – | – | – |

**Pooled** (6232 AI, 3953 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.984 (0.981–0.988) | 0.620 (0.609–0.632) | 0.761 | 1.54% | 0.937 | 0.549 |
| SwinV2 detector | 0.924 (0.917–0.931) | 0.834 (0.824–0.843) | 0.876 | 10.80% | 0.932 | 0.548 |
| Veracity | 0.990 (0.987–0.993) | 0.684 (0.672–0.696) | 0.809 | 1.11% | 0.974 | 0.676 |
| Veracity (fast) | 0.989 (0.985–0.992) | 0.603 (0.590–0.615) | 0.749 | 1.06% | 0.962 | 0.583 |
| Veracity (sensitive) | 0.944 (0.938–0.949) | 0.925 (0.918–0.931) | 0.934 | 8.70% | 0.974 | 0.676 |

### Video

**AIGVDBench, batch 1 (Gen-2, Sora, Jimeng)** (450 AI, 200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.998 (0.993–1.000) | 0.998 (0.993–1.000) | 0.998 | 0.50% | 1.000 | 0.998 |
| Veracity | 1.000 (1.000–1.000) | 0.982 (0.969–0.993) | 0.991 | 0.00% | 0.999 | 0.991 |
| Veracity + second opinion | 1.000 (1.000–1.000) | 0.991 (0.982–0.998) | 0.996 | 0.00% | 0.999 | 0.996 |

**ComGenVid, batch 1 (Sora, Veo 3)** (200 AI, 200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.994 (0.982–1.000) | 0.850 (0.797–0.898) | 0.916 | 0.50% | 0.986 | 0.900 |
| Veracity | 0.994 (0.982–1.000) | 0.895 (0.850–0.937) | 0.942 | 0.50% | 0.998 | 0.960 |
| Veracity + second opinion | 0.995 (0.983–1.000) | 0.910 (0.868–0.947) | 0.950 | 0.50% | 0.998 | 0.975 |

**GenVideo-Val, batch 1** (456 AI, 300 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.990 (0.980–0.998) | 0.897 (0.868–0.928) | 0.941 | 1.33% | 0.968 | 0.866 |
| Veracity | 0.991 (0.981–0.998) | 0.925 (0.902–0.952) | 0.957 | 1.33% | 0.986 | 0.901 |
| Veracity + second opinion | 0.993 (0.984–1.000) | 0.945 (0.924–0.967) | 0.969 | 1.00% | 0.986 | 0.947 |

**Frontier 2026-10 video, messenger copies** (101 AI, 100 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.857 (0.500–1.000) | 0.059 (0.019–0.109) | 0.111 | 1.00% | 0.736 | 0.228 |
| Veracity | 1.000 (1.000–1.000) | 0.158 (0.095–0.231) | 0.274 | 0.00% | 0.865 | 0.267 |
| Veracity + second opinion | 1.000 (1.000–1.000) | 0.109 (0.052–0.174) | 0.196 | 0.00% | 0.820 | 0.277 |

**Pooled** (1207 AI, 800 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.993 (0.988–0.998) | 0.857 (0.837–0.878) | 0.920 | 0.88% | 0.952 | 0.860 |
| Veracity | 0.995 (0.991–0.999) | 0.877 (0.859–0.896) | 0.933 | 0.62% | 0.986 | 0.901 |
| Veracity + second opinion | 0.996 (0.992–0.999) | 0.886 (0.867–0.905) | 0.938 | 0.50% | 0.982 | 0.892 |

## Confirmation batch 2 (plan `confirm2`)

### Photos

**AIGIBench FLUX.1-dev** (1500 AI, 1500 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.952 (0.938–0.964) | 0.762 (0.741–0.783) | 0.846 | 3.87% | 0.970 | 0.519 |
| SwinV2 detector | 0.834 (0.816–0.850) | 0.986 (0.980–0.992) | 0.903 | 19.67% | 0.985 | 0.841 |
| Veracity | 0.996 (0.992–0.999) | 0.987 (0.981–0.992) | 0.991 | 0.40% | 0.999 | 0.995 |
| Veracity (fast) | 0.995 (0.991–0.999) | 0.942 (0.930–0.954) | 0.968 | 0.47% | 0.997 | 0.975 |
| Veracity (sensitive) | 0.918 (0.904–0.930) | 0.997 (0.995–1.000) | 0.956 | 8.93% | 0.999 | 0.995 |

**AIGIBench Midjourney** (1500 AI, 1500 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.997 (0.993–1.000) | 0.640 (0.616–0.664) | 0.780 | 0.20% | 0.996 | 0.881 |
| SwinV2 detector | 0.724 (0.705–0.743) | 0.969 (0.960–0.978) | 0.829 | 36.87% | 0.954 | 0.664 |
| Veracity | 1.000 (1.000–1.000) | 0.927 (0.913–0.940) | 0.962 | 0.00% | 0.999 | 0.987 |
| Veracity (fast) | 0.999 (0.996–1.000) | 0.719 (0.696–0.740) | 0.836 | 0.07% | 0.996 | 0.951 |
| Veracity (sensitive) | 0.943 (0.931–0.953) | 0.996 (0.993–0.999) | 0.969 | 6.07% | 0.999 | 0.987 |

**So-Fake-OOD, batch 2** (1531 AI, 980 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.982 (0.971–0.991) | 0.437 (0.413–0.460) | 0.605 | 1.22% | 0.911 | 0.413 |
| SwinV2 detector | 0.967 (0.957–0.976) | 0.822 (0.803–0.840) | 0.888 | 4.39% | 0.946 | 0.660 |
| Veracity | 0.996 (0.992–0.999) | 0.734 (0.713–0.757) | 0.845 | 0.41% | 0.984 | 0.863 |
| Veracity (fast) | 0.998 (0.995–1.000) | 0.623 (0.600–0.648) | 0.767 | 0.20% | 0.966 | 0.788 |
| Veracity (sensitive) | 0.969 (0.960–0.977) | 0.926 (0.913–0.937) | 0.947 | 4.69% | 0.984 | 0.863 |

**Pooled** (4531 AI, 3980 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.974 (0.968–0.980) | 0.612 (0.598–0.625) | 0.752 | 1.83% | 0.965 | 0.454 |
| SwinV2 detector | 0.825 (0.814–0.835) | 0.925 (0.917–0.932) | 0.872 | 22.39% | 0.943 | 0.644 |
| Veracity | 0.998 (0.996–0.999) | 0.882 (0.873–0.892) | 0.936 | 0.25% | 0.992 | 0.947 |
| Veracity (fast) | 0.997 (0.995–0.999) | 0.760 (0.747–0.773) | 0.863 | 0.25% | 0.985 | 0.892 |
| Veracity (sensitive) | 0.942 (0.935–0.948) | 0.973 (0.968–0.977) | 0.957 | 6.81% | 0.992 | 0.947 |
