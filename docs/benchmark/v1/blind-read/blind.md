23843 files
| Detector | Precision | Recall | Real flagged | Recall at 1% FPR | ms per image |
|---|---|---|---|---|---|
| Tintype v0.4.0 | 98.98 (98.81–99.14) | 93.01 (92.58–93.43) | 1.20 (1.00–1.41)% | 92.49 (91.38–93.18) | 7.7 |
| GPTTrace | 99.21 (99.03–99.37) | 74.93 (74.18–75.70) | 0.75 (0.59–0.92)% | 78.41 (76.48–80.42) | 82.3 |
| Community Forensics | 95.25 (94.90–95.62) | 84.03 (83.41–84.67) | 5.24 (4.81–5.64)% | 69.96 (68.15–71.59) | 5.3 |
| SwinV2 | 85.92 (85.20–86.65) | 49.71 (48.83–50.60) | 10.18 (9.61–10.77)% | 28.94 (27.70–30.44) | 18.9 |
GPTTrace: 1 unreadable file(s) counted as not flagged: sofake-ood/test_image-00044-of-00046/0001221.jpg
Tintype v0.4.0 - GPTTrace: precision -0.46 to +0.01, recall +17.27 to +18.88, fpr +0.20 to +0.70, recall_at_1pct_fpr +11.76 to +16.11
Tintype v0.4.0 - Community Forensics: precision +3.37 to +4.12, recall +8.30 to +9.60, fpr -4.49 to -3.62, recall_at_1pct_fpr +20.58 to +24.24
Tintype v0.4.0 - SwinV2: precision +12.33 to +13.77, recall +42.35 to +44.28, fpr -9.59 to -8.40, recall_at_1pct_fpr +61.66 to +64.96
GPTTrace - Community Forensics: precision +3.56 to +4.36, recall -10.10 to -8.12, fpr -4.93 to -4.05, recall_at_1pct_fpr +5.99 to +10.97
GPTTrace - SwinV2: precision +12.58 to +13.98, recall +24.33 to +26.19, fpr -10.02 to -8.85, recall_at_1pct_fpr +47.08 to +51.68
Community Forensics - SwinV2: precision +8.53 to +10.12, recall +33.20 to +35.51, fpr -5.66 to -4.26, recall_at_1pct_fpr +38.59 to +43.07
