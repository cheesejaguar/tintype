```
$ python3 -I scripts/audit-manifests.py summary runs/ws4/features-frontier.tsv --by dataset
```

2976 files: 1932 real, 1044 AI. Format, EXIF and ICC shares are of the files read; JPEG settings of the JPEG files of a row; PNG text chunks of its PNG files; a dash means none.

### Files and JPEG settings by dataset

| Dataset | Class | Files | JPEG | PNG | WebP | Other | JPEG quality p10 / p50 / p90 | IJG tables | 4:2:0 | 4:4:4 | Progressive |
|---|---|---|---|---|---|---|---|---|---|---|---|
| wasitai-image | real | 486 | 100.0% | 0.0% | 0.0% | 0.0% | 90 / 95 / 99 | 57.4% | 87.0% | 11.1% | 2.7% |
| wasitai-image | AI | 261 | 20.7% | 68.6% | 10.7% | 0.0% | 75 / 95 / 95 | 100.0% | 68.5% | 31.5% | 0.0% |
| wasitai-image-instagram | real | 482 | 100.0% | 0.0% | 0.0% | 0.0% | 80 / 80 / 80 | 100.0% | 100.0% | 0.0% | 0.0% |
| wasitai-image-instagram | AI | 261 | 100.0% | 0.0% | 0.0% | 0.0% | 80 / 80 / 80 | 100.0% | 100.0% | 0.0% | 0.0% |
| wasitai-image-reshared | real | 482 | 100.0% | 0.0% | 0.0% | 0.0% | 75 / 75 / 75 | 100.0% | 100.0% | 0.0% | 0.0% |
| wasitai-image-reshared | AI | 261 | 100.0% | 0.0% | 0.0% | 0.0% | 75 / 75 / 75 | 100.0% | 100.0% | 0.0% | 0.0% |
| wasitai-image-whatsapp | real | 482 | 100.0% | 0.0% | 0.0% | 0.0% | 70 / 70 / 70 | 100.0% | 100.0% | 0.0% | 0.0% |
| wasitai-image-whatsapp | AI | 261 | 100.0% | 0.0% | 0.0% | 0.0% | 70 / 70 / 70 | 100.0% | 100.0% | 0.0% | 0.0% |

### Size and metadata by dataset

| Dataset | Class | Files | Short side p10 / p50 / p90 | Megapixels p50 | Bytes per pixel p50 | Both sides multiples of 64 | EXIF | ICC | PNG text chunks | Unreadable |
|---|---|---|---|---|---|---|---|---|---|---|
| wasitai-image | real | 486 | 1800 / 3000 / 3472 | 12.00 | 0.33 | 16.9% | 0.0% | 56.0% | – | 0 |
| wasitai-image | AI | 261 | 768 / 1024 / 1504 | 1.05 | 1.25 | 74.7% | 0.0% | 0.0% | 0.0% | 0 |
| wasitai-image-instagram | real | 482 | 608 / 810 / 813 | 0.87 | 0.20 | 0.2% | 0.0% | 0.0% | – | 0 |
| wasitai-image-instagram | AI | 261 | 608 / 806 / 1024 | 0.87 | 0.13 | 64.8% | 0.0% | 0.0% | – | 0 |
| wasitai-image-reshared | real | 482 | 720 / 960 / 964 | 1.23 | 0.17 | 56.4% | 0.0% | 0.0% | – | 0 |
| wasitai-image-reshared | AI | 261 | 720 / 880 / 1024 | 1.05 | 0.11 | 74.3% | 0.0% | 0.0% | – | 0 |
| wasitai-image-whatsapp | real | 482 | 900 / 1200 / 1205 | 1.92 | 0.15 | 2.3% | 0.0% | 0.0% | – | 0 |
| wasitai-image-whatsapp | AI | 261 | 768 / 900 / 1024 | 1.05 | 0.10 | 70.1% | 0.0% | 0.0% | – | 0 |

