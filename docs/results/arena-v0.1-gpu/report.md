# AI Detector Arena v0.1

Author-run evaluation on an external benchmark. Original and normalized rows use the same paired inputs.

Paired coverage is 2038 images (1018 generated, 1020 authentic). See `run.json` and `errors.tsv` for source coverage and exclusions.

Normalization uses a same-dimension 4:2:0 JPEG re-encode at FFmpeg MJPEG q4 for both classes. It retains traces of previous compression. Recall at 1% FPR is an empirical ranking comparison on this cohort. Shipped-threshold results use a score above 0.5.

**arena-jpeg-q4** (1018 AI, 1020 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.990 (0.983–0.997) | 0.695 (0.667–0.724) | 0.817 | 0.69% | 0.958 | 0.731 |
| Community Forensics | 0.953 (0.923–0.978) | 0.239 (0.212–0.265) | 0.382 | 1.18% | 0.802 | 0.237 |

**arena-original** (1018 AI, 1020 real)

| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |
|---|---|---|---|---|---|---|
| Tintype | 0.989 (0.981–0.995) | 0.882 (0.863–0.902) | 0.933 | 0.98% | 0.980 | 0.885 |
| Community Forensics | 0.974 (0.959–0.987) | 0.472 (0.440–0.504) | 0.635 | 1.27% | 0.892 | 0.463 |
