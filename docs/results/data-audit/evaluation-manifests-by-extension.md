```
$ python3 -I scripts/audit-manifests.py summary runs/ws4/features-eval-manifests.tsv --by-extension --by store
```

52154 manifest rows: 23385 real, 28769 AI. Shares are of the rows of a line, by the file extension written in the manifest.

| Store | Class | Files | On disk | .png | .jpg | .webp | .jpeg | .gif |
|---|---|---|---|---|---|---|---|---|
| aigibench | real | 6000 | 0 | 0.0% | 100.0% | 0.0% | 0.0% | 0.0% |
| aigibench | AI | 6000 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-BLIP | real | 400 | 0 | 0.0% | 100.0% | 0.0% | 0.0% | 0.0% |
| aigibench-BLIP | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-DALLE-3 | real | 365 | 0 | 29.9% | 70.1% | 0.0% | 0.0% | 0.0% |
| aigibench-DALLE-3 | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-FLUX1-dev | real | 366 | 0 | 29.2% | 70.8% | 0.0% | 0.0% | 0.0% |
| aigibench-FLUX1-dev | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-GLIDE | real | 388 | 0 | 17.3% | 82.7% | 0.0% | 0.0% | 0.0% |
| aigibench-GLIDE | AI | 397 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-IP-Adapter | real | 373 | 0 | 29.2% | 70.8% | 0.0% | 0.0% | 0.0% |
| aigibench-IP-Adapter | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-Imagen3 | real | 374 | 0 | 31.3% | 68.7% | 0.0% | 0.0% | 0.0% |
| aigibench-Imagen3 | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-Infinite-ID | real | 381 | 0 | 25.5% | 74.5% | 0.0% | 0.0% | 0.0% |
| aigibench-Infinite-ID | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-InstantID | real | 368 | 0 | 27.7% | 72.3% | 0.0% | 0.0% | 0.0% |
| aigibench-InstantID | AI | 399 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-Midjourney | real | 313 | 0 | 98.7% | 1.3% | 0.0% | 0.0% | 0.0% |
| aigibench-Midjourney | AI | 394 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-PhotoMaker | real | 373 | 0 | 28.7% | 71.3% | 0.0% | 0.0% | 0.0% |
| aigibench-PhotoMaker | AI | 399 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-ProGAN | real | 399 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-ProGAN | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-R3GAN | real | 365 | 0 | 27.9% | 72.1% | 0.0% | 0.0% | 0.0% |
| aigibench-R3GAN | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-SD3 | real | 383 | 0 | 23.0% | 77.0% | 0.0% | 0.0% | 0.0% |
| aigibench-SD3 | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-SDXL | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-StyleGAN-XL | real | 369 | 0 | 28.5% | 71.5% | 0.0% | 0.0% | 0.0% |
| aigibench-StyleGAN-XL | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-StyleGAN3 | real | 382 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-StyleGAN3 | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-StyleSwim | real | 369 | 0 | 28.7% | 71.3% | 0.0% | 0.0% | 0.0% |
| aigibench-StyleSwim | AI | 400 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| aigibench-WFIR | real | 200 | 0 | 0.0% | 0.0% | 0.0% | 100.0% | 0.0% |
| aigibench-WFIR | AI | 200 | 0 | 0.0% | 0.0% | 0.0% | 100.0% | 0.0% |
| classic-biggan | real | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-biggan | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-crn | real | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-crn | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-cyclegan | real | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-cyclegan | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-deepfake | real | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-deepfake | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-gaugan | real | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-gaugan | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-imle | real | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-imle | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-san | real | 200 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-san | AI | 219 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-seeingdark | real | 180 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-seeingdark | AI | 180 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-stargan | real | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-stargan | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-stylegan | real | 292 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-stylegan | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-stylegan2 | real | 282 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| classic-stylegan2 | AI | 300 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| dev-openfake-test | real | 3563 | 0 | 0.0% | 100.0% | 0.0% | 0.0% | 0.0% |
| dev-openfake-test | AI | 2873 | 0 | 24.6% | 74.6% | 0.9% | 0.0% | 0.0% |
| fresh8/sofake-ood | real | 1525 | 0 | 7.7% | 92.0% | 0.0% | 0.0% | 0.3% |
| fresh8/sofake-ood | AI | 2217 | 0 | 51.9% | 39.6% | 8.5% | 0.0% | 0.0% |
| store/sofake-ood | real | 2075 | 0 | 7.0% | 92.7% | 0.0% | 0.0% | 0.3% |
| store/sofake-ood | AI | 2912 | 0 | 50.9% | 40.6% | 8.5% | 0.0% | 0.0% |
| synthbuster | real | 1000 | 0 | 100.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| synthbuster | AI | 4679 | 0 | 92.3% | 7.7% | 0.0% | 0.0% | 0.0% |

