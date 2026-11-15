```
$ python3 -I scripts/metadata-probe.py runs/ws4/features-frontier.tsv --by dataset --fit-on runs/ws4/features-train.tsv --columns encoding
```

Metadata-only label probe over 2976 files; columns: format, jpeg_quality, jpeg_tables, subsampling, progressive, exif, icc, jfif, xmp_or_adobe, c2pa, png_text, bytes_per_pixel.
5 folds, seed 1, 200 bootstrap replicates over clusters, at most 30000 files per fit.

| Dataset | Real | AI | Real sources | AI generators | AUROC, item folds | AUROC, group folds | AUROC, fitted on the other datasets | AUROC, fitted on --fit-on | Strongest single columns (item folds) |
|---|---|---|---|---|---|---|---|---|---|
| wasitai-image | 486 | 261 | 1 | 19 | 0.998 [0.994, 1.000]† | – | 0.619 [0.567, 0.671]† | 0.959 [0.924, 0.992]† | jpeg_quality 0.969, jfif 0.948, jpeg_tables 0.940 |
| wasitai-image-instagram | 482 | 261 | 1 | 19 | 0.768 [0.701, 0.830]† | – | 0.764 [0.693, 0.839]† | 0.619 [0.531, 0.716]† | bytes_per_pixel 0.734, xmp_or_adobe 0.625, subsampling 0.500 |
| wasitai-image-reshared | 482 | 261 | 1 | 19 | 0.756 [0.671, 0.843]† | – | 0.759 [0.693, 0.830]† | 0.589 [0.497, 0.679]† | bytes_per_pixel 0.725, xmp_or_adobe 0.621, subsampling 0.500 |
| wasitai-image-whatsapp | 482 | 261 | 1 | 19 | 0.730 [0.632, 0.823]† | – | 0.730 [0.670, 0.789]† | 0.582 [0.481, 0.679]† | bytes_per_pixel 0.688, xmp_or_adobe 0.624, subsampling 0.500 |

Pooled over 4 datasets: 1932 real files from 1 sources, 1044 AI files from 19 generators.

| Pooled | AUROC [95% interval over clusters] |
|---|---|
| item folds | 0.854 [0.807, 0.898]† |
| fitted on the other datasets, scores pooled | 0.689 [0.631, 0.736]† |
| fitted on --fit-on | 0.682 [0.658, 0.714]† |

| Column alone | AUROC, item folds |
|---|---|
| bytes_per_pixel | 0.775 |
| jpeg_quality | 0.671 |
| jfif | 0.651 |
| jpeg_tables | 0.642 |
| subsampling | 0.604 |
| progressive | 0.602 |
| format | 0.599 |
| xmp_or_adobe | 0.593 |
| png_text | 0.586 |
| icc | 0.571 |
| exif | 0.500 |
| c2pa | 0.500 |

Largest weights of the model fitted on all files (positive means AI):

| Column | Bin | Weight |
|---|---|---|
| jpeg_tables | custom | -1.44 |
| bytes_per_pixel | 0.2-0.4 | -1.41 |
| icc | 0 | +1.38 |
| icc | 1 | -1.38 |
| jpeg_quality | 93-96 | +1.37 |
| jfif | 0 | -1.34 |
| bytes_per_pixel | 0.05-0.1 | +1.31 |
| jpeg_quality | 90-93 | -1.15 |
| subsampling | 420 | -0.97 |
| jpeg_quality | >=98 | -0.96 |
| bytes_per_pixel | 0.4-0.8 | -0.86 |
| jpeg_tables | ijg | +0.74 |
| jpeg_quality | none | +0.71 |
| jpeg_tables | none | +0.71 |
| subsampling | none | +0.71 |
