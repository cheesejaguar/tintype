```
$ python3 -I scripts/metadata-probe.py runs/ws4/features-frontier.tsv --by dataset --fit-on runs/ws4/features-train.tsv
```

Metadata-only label probe over 2976 files; columns: format, jpeg_quality, jpeg_tables, subsampling, progressive, exif, icc, jfif, xmp_or_adobe, c2pa, png_text, short_side, long_side, aspect, multiple, megapixels, exact_size, bytes_per_pixel.
5 folds, seed 1, 200 bootstrap replicates over clusters, at most 30000 files per fit.

| Dataset | Real | AI | Real sources | AI generators | AUROC, item folds | AUROC, group folds | AUROC, fitted on the other datasets | AUROC, fitted on --fit-on | Strongest single columns (item folds) |
|---|---|---|---|---|---|---|---|---|---|
| wasitai-image | 486 | 261 | 1 | 19 | 1.000 [0.999, 1.000]† | – | 0.965 [0.912, 0.997]† | 0.961 [0.922, 0.991]† | exact_size 0.995, short_side 0.987, megapixels 0.985 |
| wasitai-image-instagram | 482 | 261 | 1 | 19 | 0.936 [0.884, 0.977]† | – | 0.924 [0.877, 0.976]† | 0.767 [0.665, 0.861]† | exact_size 0.917, short_side 0.882, multiple 0.838 |
| wasitai-image-reshared | 482 | 261 | 1 | 19 | 0.968 [0.933, 0.995]† | – | 0.934 [0.887, 0.968]† | 0.595 [0.524, 0.667]† | exact_size 0.939, long_side 0.872, megapixels 0.866 |
| wasitai-image-whatsapp | 482 | 261 | 1 | 19 | 0.971 [0.940, 0.996]† | – | 0.967 [0.928, 0.998]† | 0.842 [0.771, 0.904]† | exact_size 0.941, short_side 0.931, megapixels 0.923 |

Pooled over 4 datasets: 1932 real files from 1 sources, 1044 AI files from 19 generators.

| Pooled | AUROC [95% interval over clusters] |
|---|---|
| item folds | 0.977 [0.956, 0.995]† |
| fitted on the other datasets, scores pooled | 0.941 [0.907, 0.976]† |
| fitted on --fit-on | 0.798 [0.739, 0.843]† |

| Column alone | AUROC, item folds |
|---|---|
| exact_size | 0.949 |
| short_side | 0.912 |
| long_side | 0.899 |
| megapixels | 0.877 |
| aspect | 0.796 |
| multiple | 0.791 |
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
| long_side | 1024 | +2.05 |
| aspect | tall | -1.67 |
| aspect | 16:9 | +1.62 |
| megapixels | 1.0-1.1 | +1.48 |
| aspect | wide | -1.34 |
| bytes_per_pixel | 0.2-0.4 | -1.23 |
| bytes_per_pixel | 0.05-0.1 | +1.21 |
| aspect | 9:16 | +1.12 |
| short_side | 768 | +0.99 |
| long_side | 1024-1280 | +0.95 |
| short_side | 2048-3072 | -0.94 |
| megapixels | 8.0-16.0 | -0.92 |
| multiple | 8 | -0.92 |
| multiple | 64 | +0.91 |
| multiple | 16 | +0.84 |
