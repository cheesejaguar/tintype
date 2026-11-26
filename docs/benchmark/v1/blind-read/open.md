24961 files
| Detector | Precision | Recall | Real flagged | Recall at 1% FPR | ms per image |
|---|---|---|---|---|---|
| Tintype v0.4.0 | 98.95 (98.79–99.13) | 93.31 (92.90–93.76) | 1.16 (0.96–1.34)% | 92.29 (91.02–93.51) | – |
| GPTTrace | 99.11 (98.92–99.28) | 74.09 (73.33–74.82) | 0.78 (0.63–0.94)% | 76.51 (74.58–78.77) | – |
| Community Forensics | 95.24 (94.87–95.60) | 84.12 (83.50–84.73) | 4.94 (4.54–5.34)% | 70.93 (69.41–72.46) | – |
| SwinV2 | 83.83 (83.12–84.56) | 48.84 (47.97–49.71) | 11.05 (10.49–11.62)% | 28.27 (26.98–29.56) | – |
Tintype v0.4.0 - GPTTrace: precision -0.40 to +0.09, recall +18.46 to +20.01, fpr +0.14 to +0.62, recall_at_1pct_fpr +13.31 to +17.93
Tintype v0.4.0 - Community Forensics: precision +3.34 to +4.10, recall +8.60 to +9.81, fpr -4.20 to -3.37, recall_at_1pct_fpr +19.61 to +23.27
Tintype v0.4.0 - SwinV2: precision +14.39 to +15.89, recall +43.58 to +45.39, fpr -10.49 to -9.32, recall_at_1pct_fpr +62.28 to +65.92
GPTTrace - Community Forensics: precision +3.45 to +4.27, recall -11.01 to -9.05, fpr -4.57 to -3.73, recall_at_1pct_fpr +3.19 to +8.27
GPTTrace - SwinV2: precision +14.56 to +16.01, recall +24.36 to +26.17, fpr -10.87 to -9.69, recall_at_1pct_fpr +46.08 to +50.81
Community Forensics - SwinV2: precision +10.58 to +12.22, recall +34.16 to +36.41, fpr -6.80 to -5.41, recall_at_1pct_fpr +40.65 to +44.74
