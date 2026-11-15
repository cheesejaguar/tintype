```
$ python3 -I scripts/metadata-probe.py runs/ws4/features-eval-manifests.tsv --columns extension
```

Metadata-only label probe over 52154 files; columns: extension.
5 folds, seed 1, 200 bootstrap replicates over clusters, at most 30000 files per fit.

| Store | Real | AI | Real sources | AI generators | AUROC, item folds | AUROC, group folds | AUROC, fitted on the other stores | Strongest single columns (item folds) |
|---|---|---|---|---|---|---|---|---|
| aigibench | 6000 | 6000 | 1 | 1 | 1.000 [1.000, 1.000]† | – | 1.000 [1.000, 1.000]† | extension 1.000 |
| aigibench-BLIP | 400 | 400 | 1 | 1 | 1.000 [1.000, 1.000]† | – | 1.000 [1.000, 1.000]† | extension 1.000 |
| aigibench-DALLE-3 | 365 | 400 | 1 | 1 | 0.851 [0.829, 0.875]† | – | 0.851 [0.829, 0.875]† | extension 0.851 |
| aigibench-FLUX1-dev | 366 | 400 | 1 | 1 | 0.854 [0.830, 0.880]† | – | 0.854 [0.832, 0.878]† | extension 0.854 |
| aigibench-GLIDE | 388 | 397 | 1 | 1 | 0.913 [0.896, 0.934]† | – | 0.914 [0.897, 0.932]† | extension 0.913 |
| aigibench-IP-Adapter | 373 | 400 | 1 | 1 | 0.855 [0.832, 0.880]† | – | 0.854 [0.831, 0.881]† | extension 0.855 |
| aigibench-Imagen3 | 374 | 400 | 1 | 1 | 0.843 [0.820, 0.870]† | – | 0.844 [0.821, 0.866]† | extension 0.843 |
| aigibench-Infinite-ID | 381 | 400 | 1 | 1 | 0.873 [0.853, 0.895]† | – | 0.873 [0.849, 0.895]† | extension 0.873 |
| aigibench-InstantID | 368 | 399 | 1 | 1 | 0.861 [0.836, 0.882]† | – | 0.861 [0.840, 0.886]† | extension 0.861 |
| aigibench-Midjourney | 313 | 394 | 1 | 1 | 0.506 [0.502, 0.514]† | – | 0.506 [0.502, 0.513]† | extension 0.506 |
| aigibench-PhotoMaker | 373 | 399 | 1 | 1 | 0.857 [0.835, 0.882]† | – | 0.857 [0.832, 0.877]† | extension 0.857 |
| aigibench-ProGAN | 399 | 400 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| aigibench-R3GAN | 365 | 400 | 1 | 1 | 0.858 [0.837, 0.883]† | – | 0.860 [0.834, 0.879]† | extension 0.858 |
| aigibench-SD3 | 383 | 400 | 1 | 1 | 0.885 [0.857, 0.903]† | – | 0.885 [0.866, 0.903]† | extension 0.885 |
| aigibench-SDXL | 0 | 400 | 0 | 1 | – | – | – |  |
| aigibench-StyleGAN-XL | 369 | 400 | 1 | 1 | 0.858 [0.835, 0.882]† | – | 0.858 [0.836, 0.882]† | extension 0.858 |
| aigibench-StyleGAN3 | 382 | 400 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| aigibench-StyleSwim | 369 | 400 | 1 | 1 | 0.856 [0.831, 0.879]† | – | 0.856 [0.832, 0.879]† | extension 0.856 |
| aigibench-WFIR | 200 | 200 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-biggan | 300 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-crn | 300 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-cyclegan | 300 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-deepfake | 300 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-gaugan | 300 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-imle | 300 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-san | 200 | 219 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-seeingdark | 180 | 180 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-stargan | 300 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-stylegan | 292 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| classic-stylegan2 | 282 | 300 | 1 | 1 | 0.500 [0.500, 0.500]† | – | 0.500 [0.500, 0.500]† | extension 0.500 |
| dev-openfake-test | 3563 | 2873 | 2 | 15 | 0.627 [0.556, 0.808] | 0.637 [0.538, 0.896] | 0.627 [0.554, 0.766] | extension 0.627 |
| fresh8/sofake-ood | 1525 | 2217 | 3 | 15 | 0.766 [0.664, 0.896] | 0.712 [0.524, 0.934] | 0.766 [0.661, 0.899] | extension 0.767 |
| store/sofake-ood | 2075 | 2912 | 3 | 15 | 0.764 [0.642, 0.884] | 0.697 [0.534, 0.948] | 0.764 [0.636, 0.922] | extension 0.766 |
| synthbuster | 1000 | 4679 | 1 | 13 | 0.539 [0.500, 0.616]† | – | 0.462 [0.385, 0.500]† | extension 0.539 |

Pooled over 34 stores: 23385 real files from 9 sources, 28769 AI files from 71 generators.

| Pooled | AUROC [95% interval over clusters] |
|---|---|
| item folds | 0.780 [0.634, 0.911] |
| group folds | 0.758 [0.595, 0.905] |
| fitted on the other stores, scores pooled | 0.704 [0.503, 0.851] |

| Column alone | AUROC, item folds | AUROC, group folds |
|---|---|---|
| extension | 0.780 | 0.767 |

Largest weights of the model fitted on all files (positive means AI):

| Column | Bin | Weight |
|---|---|---|
| extension | jpg | -1.73 |
| extension | webp | +1.33 |
| extension | png | +0.80 |
| extension | jpeg | -0.25 |
| extension | (other) | -0.14 |
