```
$ python3 -I scripts/group-manifest.py $(sed "s/--manifest //g" runs/v7/manifests.txt) --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv --summary --dry-run --write-map runs/ws4/families.tsv
```

367063 items: 125751 real, 241312 generated under 1222 generator names

| Family | Generator names | Items | Share of generated items |
|---|---|---|---|
| latent-diffusion | 1116 | 81518 | 33.8% |
| unknown | 13 | 51161 | 21.2% |
| video-diffusion | 16 | 36369 | 15.1% |
| gan | 22 | 28887 | 12.0% |
| commercial-unknown | 44 | 24146 | 10.0% |
| autoregressive-vq | 5 | 9807 | 4.1% |
| pixel-diffusion | 3 | 6991 | 2.9% |
| hybrid | 2 | 2000 | 0.8% |
| other | 1 | 433 | 0.2% |

| Basis of the assignment | Generator names | Items |
|---|---|---|
| model-doc | 59 | 76560 |
| none | 13 | 51161 |
| name | 68 | 46362 |
| dataset-doc | 1038 | 43083 |
| closed | 44 | 24146 |

```
$ python3 -I scripts/group-manifest.py data/store/*/manifest.tsv --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv --summary --dry-run
```

343384 items: 125308 real, 218076 generated under 1245 generator names

| Family | Generator names | Items | Share of generated items |
|---|---|---|---|
| latent-diffusion | 1129 | 86848 | 39.8% |
| unknown | 16 | 57252 | 26.3% |
| gan | 22 | 28887 | 13.2% |
| commercial-unknown | 61 | 25329 | 11.6% |
| autoregressive-vq | 5 | 9807 | 4.5% |
| pixel-diffusion | 3 | 7351 | 3.4% |
| hybrid | 2 | 2000 | 0.9% |
| other | 1 | 433 | 0.2% |
| video-diffusion | 6 | 169 | 0.1% |

| Basis of the assignment | Generator names | Items |
|---|---|---|
| none | 16 | 57252 |
| name | 78 | 49927 |
| dataset-doc | 1038 | 43083 |
| model-doc | 52 | 42485 |
| closed | 61 | 25329 |
