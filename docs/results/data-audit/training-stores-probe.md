```
$ python3 -I scripts/metadata-probe.py runs/ws4/features-train.tsv
```

Metadata-only label probe over 284304 files; columns: format, jpeg_quality, jpeg_tables, subsampling, progressive, exif, icc, jfif, xmp_or_adobe, c2pa, png_text, short_side, long_side, aspect, multiple, megapixels, exact_size, bytes_per_pixel.
29978 rows are left out: their files are not on disk.
5 folds, seed 1, 200 bootstrap replicates over clusters, at most 30000 files per fit.

| Store | Real | AI | Real sources | AI generators | AUROC, item folds | AUROC, group folds | AUROC, fitted on the other stores | Strongest single columns (item folds) |
|---|---|---|---|---|---|---|---|---|
| celeba-hq | 7500 | 0 | 1 | 0 | – | – | – |  |
| commfor-small | 23936 | 35905 | 3 | 223 | 1.000 [0.998, 1.000] | 1.000 [0.999, 1.000] | 0.998 [0.991, 1.000] | exact_size 1.000, long_side 1.000, aspect 0.982 |
| commfor-small2 | 29025 | 44885 | 1 | 830 | 0.982 [0.955, 0.999]† | – | 0.552 [0.503, 0.591]† | long_side 0.981, exact_size 0.978, short_side 0.878 |
| defactify | 7000 | 35000 | 1 | 5 | 1.000 [1.000, 1.000]† | – | 0.983 [0.962, 1.000]† | exact_size 1.000, long_side 0.986, aspect 0.985 |
| frontier | 0 | 18145 | 0 | 35 | – | – | – |  |
| frontier-heldout | 0 | 3331 | 0 | 7 | – | – | – |  |
| ip-adapter | 0 | 6000 | 0 | 2 | – | – | – |  |
| nano-banana | 0 | 1527 | 0 | 1 | – | – | – |  |
| openfake | 12005 | 11923 | 2 | 74 | 0.999 [0.997, 1.000] | 0.965 [0.934, 0.987] | 0.943 [0.915, 0.973] | exact_size 0.994, short_side 0.983, long_side 0.968 |
| openfake-reddit | 3018 | 15782 | 1 | 1 | 0.996 [0.995, 0.996]† | – | 0.943 [0.937, 0.947]† | exact_size 0.979, jpeg_quality 0.976, jfif 0.975 |
| openfake-test-reals | 7087 | 0 | 2 | 0 | – | – | – |  |
| openfake2 | 6117 | 5853 | 2 | 73 | 0.999 [0.997, 1.000] | 0.968 [0.925, 0.995] | 0.973 [0.955, 0.990] | exact_size 0.993, short_side 0.984, long_side 0.969 |
| sofake-set | 3002 | 4263 | 1 | 30 | 0.977 [0.944, 0.998]† | – | 0.721 [0.651, 0.774]† | exact_size 0.932, short_side 0.888, aspect 0.870 |
| tpdne | 0 | 3000 | 0 | 1 | – | – | – |  |

Pooled over 14 stores: 98690 real files from 11 sources, 185614 AI files from 1201 generators.

| Pooled | AUROC [95% interval over clusters] |
|---|---|
| item folds | 0.973 [0.938, 0.993] |
| group folds | 0.726 [0.551, 0.968] |
| fitted on the other stores, scores pooled | 0.781 [0.613, 0.970] |

| Column alone | AUROC, item folds | AUROC, group folds |
|---|---|---|
| exact_size | 0.957 | 0.840 |
| long_side | 0.908 | 0.753 |
| short_side | 0.848 | 0.727 |
| aspect | 0.829 | 0.775 |
| multiple | 0.768 | 0.699 |
| megapixels | 0.716 | 0.570 |
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
| jpeg_quality | 93-96 | +1.97 |
| long_side | 640-768 | -1.73 |
| exact_size | 436x436 | +1.72 |
| aspect | 1:1 | +1.61 |
| long_side | 512 | +1.52 |
| exact_size | 512x512 | +1.43 |
| exact_size | (other) | -1.39 |
| exact_size | 270x270 | +1.34 |
| long_side | >=4096 | -1.33 |
| megapixels | >=16.0 | -1.30 |
| long_side | 1024-1280 | +1.29 |
| short_side | 768 | +1.29 |
| multiple | 64 | +1.23 |
| aspect | 3:2 | -1.09 |
| aspect | 4:3 | -1.06 |
