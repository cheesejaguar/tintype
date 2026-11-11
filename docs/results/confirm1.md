
## Images

**aigibench-socialrf** (3000 AI, 3000 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.972 (0.965–0.979) | 0.628 (0.611–0.645) | 0.763 | 1.80% | 0.938 | 0.510 |
| SwinV2 detector | 0.888 (0.878–0.899) | 0.948 (0.940–0.956) | 0.917 | 11.97% | 0.975 | 0.690 |
| Veracity | 0.981 (0.975–0.987) | 0.673 (0.656–0.690) | 0.798 | 1.30% | 0.972 | 0.649 |
| Veracity (fast) | 0.982 (0.976–0.987) | 0.667 (0.651–0.685) | 0.794 | 1.23% | 0.976 | 0.597 |
| Veracity (sensitive) | 0.904 (0.893–0.914) | 0.927 (0.917–0.937) | 0.915 | 9.87% | 0.972 | 0.649 |

**sofake-ood-fresh** (1507 AI, 953 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.989 (0.979–0.997) | 0.411 (0.386–0.435) | 0.581 | 0.73% | 0.925 | 0.475 |
| SwinV2 detector | 0.947 (0.935–0.959) | 0.812 (0.793–0.831) | 0.875 | 7.14% | 0.938 | 0.446 |
| Veracity | 0.995 (0.991–0.999) | 0.724 (0.701–0.746) | 0.838 | 0.52% | 0.987 | 0.847 |
| Veracity (fast) | 0.995 (0.989–0.999) | 0.612 (0.588–0.636) | 0.758 | 0.52% | 0.969 | 0.752 |
| Veracity (sensitive) | 0.967 (0.958–0.976) | 0.936 (0.923–0.948) | 0.951 | 5.04% | 0.987 | 0.847 |

**synthbuster-fresh** (1725 AI, 0 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | – | 0.788 (0.768–0.807) | – | – | – | – |
| SwinV2 detector | – | 0.653 (0.632–0.675) | – | – | – | – |
| Veracity | – | 0.667 (0.646–0.688) | – | – | – | – |
| Veracity (fast) | – | 0.482 (0.457–0.506) | – | – | – | – |
| Veracity (sensitive) | – | 0.910 (0.896–0.923) | – | – | – | – |

**confirm1, pooled** (6232 AI, 3953 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Community Forensics | 0.984 (0.981–0.988) | 0.620 (0.609–0.632) | 0.761 | 1.54% | 0.937 | 0.549 |
| SwinV2 detector | 0.924 (0.917–0.931) | 0.834 (0.824–0.843) | 0.876 | 10.80% | 0.932 | 0.548 |
| Veracity | 0.990 (0.987–0.993) | 0.684 (0.672–0.696) | 0.809 | 1.11% | 0.974 | 0.676 |
| Veracity (fast) | 0.989 (0.985–0.992) | 0.603 (0.590–0.615) | 0.749 | 1.06% | 0.962 | 0.583 |
| Veracity (sensitive) | 0.944 (0.938–0.949) | 0.925 (0.918–0.931) | 0.934 | 8.70% | 0.974 | 0.676 |

## Video

**aigvd-fresh** (450 AI, 200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.998 (0.993–1.000) | 0.998 (0.993–1.000) | 0.998 | 0.50% | 1.000 | 0.998 |
| Veracity | 1.000 (1.000–1.000) | 0.982 (0.969–0.993) | 0.991 | 0.00% | 0.999 | 0.991 |
| Veracity + second opinion | 1.000 (1.000–1.000) | 0.991 (0.982–0.998) | 0.996 | 0.00% | 0.999 | 0.996 |

**comgenvid-fresh** (200 AI, 200 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.994 (0.982–1.000) | 0.850 (0.797–0.898) | 0.916 | 0.50% | 0.986 | 0.900 |
| Veracity | 0.994 (0.982–1.000) | 0.895 (0.850–0.937) | 0.942 | 0.50% | 0.998 | 0.960 |
| Veracity + second opinion | 0.995 (0.983–1.000) | 0.910 (0.868–0.947) | 0.950 | 0.50% | 0.998 | 0.975 |

**genvideo-val-fresh** (456 AI, 300 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.990 (0.980–0.998) | 0.897 (0.868–0.928) | 0.941 | 1.33% | 0.968 | 0.866 |
| Veracity | 0.991 (0.981–0.998) | 0.925 (0.902–0.952) | 0.957 | 1.33% | 0.986 | 0.901 |
| Veracity + second opinion | 0.993 (0.984–1.000) | 0.945 (0.924–0.967) | 0.969 | 1.00% | 0.986 | 0.947 |

**wasitai-video-messenger** (101 AI, 100 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.857 (0.500–1.000) | 0.059 (0.019–0.109) | 0.111 | 1.00% | 0.736 | 0.228 |
| Veracity | 1.000 (1.000–1.000) | 0.158 (0.095–0.231) | 0.274 | 0.00% | 0.865 | 0.267 |
| Veracity + second opinion | 1.000 (1.000–1.000) | 0.109 (0.052–0.174) | 0.196 | 0.00% | 0.820 | 0.277 |

**confirm1, pooled** (1207 AI, 800 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| WaveRep | 0.993 (0.988–0.998) | 0.857 (0.837–0.878) | 0.920 | 0.88% | 0.952 | 0.860 |
| Veracity | 0.995 (0.991–0.999) | 0.877 (0.859–0.896) | 0.933 | 0.62% | 0.986 | 0.901 |
| Veracity + second opinion | 0.996 (0.992–0.999) | 0.886 (0.867–0.905) | 0.938 | 0.50% | 0.982 | 0.892 |
