```
$ python3 -I scripts/metadata-probe.py runs/ws4/features-train.tsv --columns encoding
```

Metadata-only label probe over 284304 files; columns: format, jpeg_quality, jpeg_tables, subsampling, progressive, exif, icc, jfif, xmp_or_adobe, c2pa, png_text, bytes_per_pixel.
29978 rows are left out: their files are not on disk.
5 folds, seed 1, 200 bootstrap replicates over clusters, at most 30000 files per fit.

| Store | Real | AI | Real sources | AI generators | AUROC, item folds | AUROC, group folds | AUROC, fitted on the other stores | Strongest single columns (item folds) |
|---|---|---|---|---|---|---|---|---|
| celeba-hq | 7500 | 0 | 1 | 0 | – | – | – |  |
| commfor-small | 23936 | 35905 | 3 | 223 | 0.859 [0.473, 1.000] | 0.702 [0.510, 1.000] | 0.855 [0.555, 1.000] | bytes_per_pixel 0.858, subsampling 0.851, progressive 0.851 |
| commfor-small2 | 29025 | 44885 | 1 | 830 | 0.756 [0.735, 0.774]† | – | 0.543 [0.527, 0.566]† | bytes_per_pixel 0.756, subsampling 0.649, progressive 0.649 |
| defactify | 7000 | 35000 | 1 | 5 | 0.616 [0.442, 0.741]† | – | 0.498 [0.425, 0.573]† | bytes_per_pixel 0.616, xmp_or_adobe 0.500, subsampling 0.500 |
| frontier | 0 | 18145 | 0 | 35 | – | – | – |  |
| frontier-heldout | 0 | 3331 | 0 | 7 | – | – | – |  |
| ip-adapter | 0 | 6000 | 0 | 2 | – | – | – |  |
| nano-banana | 0 | 1527 | 0 | 1 | – | – | – |  |
| openfake | 12005 | 11923 | 2 | 74 | 0.903 [0.805, 0.964] | 0.635 [0.441, 0.785] | 0.586 [0.198, 0.934] | jpeg_quality 0.866, bytes_per_pixel 0.847, progressive 0.835 |
| openfake-reddit | 3018 | 15782 | 1 | 1 | 0.980 [0.978, 0.981]† | – | 0.358 [0.342, 0.373]† | jpeg_quality 0.976, jfif 0.975, bytes_per_pixel 0.744 |
| openfake-test-reals | 7087 | 0 | 2 | 0 | – | – | – |  |
| openfake2 | 6117 | 5853 | 2 | 73 | 0.903 [0.815, 0.965] | 0.715 [0.444, 0.853] | 0.632 [0.266, 0.931] | jpeg_quality 0.870, bytes_per_pixel 0.849, progressive 0.839 |
| sofake-set | 3002 | 4263 | 1 | 30 | 0.818 [0.753, 0.859]† | – | 0.778 [0.685, 0.829]† | jpeg_quality 0.807, subsampling 0.800, progressive 0.784 |
| tpdne | 0 | 3000 | 0 | 1 | – | – | – |  |

Pooled over 14 stores: 98690 real files from 11 sources, 185614 AI files from 1201 generators.

| Pooled | AUROC [95% interval over clusters] |
|---|---|
| item folds | 0.735 [0.602, 0.894] |
| group folds | 0.528 [0.347, 0.785] |
| fitted on the other stores, scores pooled | 0.555 [0.368, 0.773] |

| Column alone | AUROC, item folds | AUROC, group folds |
|---|---|---|
| jpeg_quality | 0.702 | 0.588 |
| bytes_per_pixel | 0.674 | 0.605 |
| subsampling | 0.671 | 0.617 |
| jfif | 0.664 | 0.577 |
| progressive | 0.661 | 0.587 |
| jpeg_tables | 0.660 | 0.597 |
| format | 0.659 | 0.604 |
| png_text | 0.654 | 0.601 |
| icc | 0.516 | 0.516 |
| exif | 0.504 | 0.493 |
| xmp_or_adobe | 0.503 | 0.503 |
| c2pa | 0.501 | 0.501 |

Largest weights of the model fitted on all files (positive means AI):

| Column | Bin | Weight |
|---|---|---|
| jpeg_quality | 93-96 | +1.07 |
| jpeg_quality | >=98 | +0.95 |
| subsampling | 420 | +0.82 |
| bytes_per_pixel | 1.6-3.2 | +0.81 |
| icc | 1 | -0.75 |
| icc | 0 | +0.75 |
| jpeg_quality | 96-98 | -0.73 |
| jfif | 0 | -0.70 |
| jpeg_quality | 80-85 | -0.69 |
| subsampling | 444 | -0.69 |
| jpeg_quality | 85-90 | -0.68 |
| progressive | 0 | -0.60 |
| format | webp | +0.52 |
| c2pa | 0 | -0.47 |
| c2pa | 1 | +0.47 |
