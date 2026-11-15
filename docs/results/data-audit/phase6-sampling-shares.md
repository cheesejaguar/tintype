```
$ tintype sampling-shares $(cat runs/v7/manifests.txt) --family-map runs/ws4/families.tsv --top 25
```

train 356038 images (16 real sources, 1222 generators), val pool 11025

Sampling shares of 356038 items (122119 real, 233919 AI) from 25 manifests; generators weigh count^0.5.
A share is of the draws of its class; each class is half of all draws. "Lift" is the share divided by the share of the class's items.

| Manifest | Pool | Real items | Share of real draws | AI items | Share of AI draws |
|---|---|---|---|---|---|
| openfake | main | 11669 | 9.82% | 11571 | 5.39% |
| openfake2 | main | 5949 | 5.01% | 5664 | 2.65% |
| openfake-reddit | main | 2896 | 4.25% | 15292 | 1.14% |
| frontier | main | 0 | 0.00% | 17558 | 6.28% |
| frontier-heldout | main | 0 | 0.00% | 3222 | 1.24% |
| nano-banana | main | 0 | 0.00% | 1485 | 0.36% |
| defactify | main | 6810 | 6.52% | 33891 | 3.80% |
| commfor-small | main | 23290 | 18.97% | 34834 | 16.66% |
| openfake-test-reals | main | 6874 | 6.45% | 0 | 0.00% |
| sofake-set | main | 2916 | 4.27% | 4138 | 2.41% |
| commfor-small2 | main | 28197 | 13.27% | 43525 | 49.60% |
| tiny-genimage | main | 13580 | 7.95% | 13558 | 2.85% |
| stylegan3 | main | 0 | 0.00% | 1945 | 0.32% |
| tpdne | main | 0 | 0.00% | 2913 | 0.42% |
| ip-adapter | main | 0 | 0.00% | 5834 | 1.00% |
| celeba-hq | main | 7253 | 6.73% | 0 | 0.00% |
| vidguard | main | 3746 | 4.84% | 4842 | 0.69% |
| genbuster | main | 4650 | 5.39% | 4869 | 0.97% |
| genvideo100k | main | 0 | 0.00% | 19373 | 1.78% |
| aigvd | main | 0 | 0.00% | 5815 | 1.73% |
| k400 | main | 1936 | 2.46% | 0 | 0.00% |
| pexels | main | 79 | 0.31% | 0 | 0.00% |
| k400b | main | 1940 | 2.46% | 0 | 0.00% |
| pexelsb | main | 334 | 1.30% | 0 | 0.00% |
| aigvd2 | main | 0 | 0.00% | 3590 | 0.73% |

Generator family, real class: 16 in all

| Generator family | Items | Share of real draws | Lift |
|---|---|---|---|
| real:commfor | 28197 | 13.27% | 0.57 |
| real:imagenet | 18211 | 10.66% | 0.72 |
| real:coco | 17463 | 10.44% | 0.73 |
| real:laion | 9127 | 7.55% | 1.01 |
| real:pexels | 8491 | 7.28% | 1.05 |
| real:celeba-hq | 7253 | 6.73% | 1.13 |
| real:defactify | 6810 | 6.52% | 1.17 |
| real:openvid | 4650 | 5.39% | 1.42 |
| real:kinetics400 | 3876 | 4.92% | 1.55 |
| real:activitynet | 3746 | 4.84% | 1.58 |
| real:ffhq | 2920 | 4.27% | 1.79 |
| real:sofake-set | 2916 | 4.27% | 1.79 |
| real:lhq | 2907 | 4.26% | 1.79 |
| real:openfake-reddit | 2896 | 4.25% | 1.79 |
| real:docci | 2243 | 3.74% | 2.04 |
| real:pexels-video | 413 | 1.61% | 4.75 |

Generator family, AI class: 9 in all

| Generator family | Items | Share of AI draws | Lift |
|---|---|---|---|
| latent-diffusion | 79100 | 72.38% | 2.14 |
| commercial-unknown | 23389 | 8.12% | 0.81 |
| unknown | 49551 | 5.34% | 0.25 |
| gan | 27982 | 5.33% | 0.45 |
| video-diffusion | 35258 | 5.09% | 0.34 |
| autoregressive-vq | 9520 | 1.65% | 0.41 |
| pixel-diffusion | 6768 | 1.31% | 0.45 |
| hybrid | 1933 | 0.57% | 0.70 |
| other | 418 | 0.19% | 1.06 |

Generator, real class: 16 in all

| Generator | Items | Share of real draws | Lift |
|---|---|---|---|
| real:commfor | 28197 | 13.27% | 0.57 |
| real:imagenet | 18211 | 10.66% | 0.72 |
| real:coco | 17463 | 10.44% | 0.73 |
| real:laion | 9127 | 7.55% | 1.01 |
| real:pexels | 8491 | 7.28% | 1.05 |
| real:celeba-hq | 7253 | 6.73% | 1.13 |
| real:defactify | 6810 | 6.52% | 1.17 |
| real:openvid | 4650 | 5.39% | 1.42 |
| real:kinetics400 | 3876 | 4.92% | 1.55 |
| real:activitynet | 3746 | 4.84% | 1.58 |
| real:ffhq | 2920 | 4.27% | 1.79 |
| real:sofake-set | 2916 | 4.27% | 1.79 |
| real:lhq | 2907 | 4.26% | 1.79 |
| real:openfake-reddit | 2896 | 4.25% | 1.79 |
| real:docci | 2243 | 3.74% | 2.04 |
| real:pexels-video | 413 | 1.61% | 4.75 |

Generator, AI class: 1222 in all, the 25 largest shares shown

| Generator | Items | Share of AI draws | Lift |
|---|---|---|---|
| reddit-wild | 15292 | 1.14% | 0.17 |
| OpenSora | 10669 | 0.95% | 0.21 |
| ZeroScope | 9673 | 0.91% | 0.22 |
| defactify-gen3 | 6789 | 0.76% | 0.26 |
| defactify-gen5 | 6785 | 0.76% | 0.26 |
| defactify-gen1 | 6779 | 0.76% | 0.26 |
| defactify-gen4 | 6775 | 0.76% | 0.26 |
| defactify-gen2 | 6763 | 0.76% | 0.26 |
| LFM | 6047 | 0.72% | 0.28 |
| GigaGAN | 5797 | 0.70% | 0.28 |
| tamingTransformers | 5424 | 0.68% | 0.29 |
| CogVideoX | 4855 | 0.64% | 0.31 |
| StyleGAN2 | 4154 | 0.60% | 0.34 |
| HunyuanVideo | 3591 | 0.55% | 0.36 |
| StyleGAN3 | 3235 | 0.53% | 0.38 |
| IP-Adapter FaceID | 2917 | 0.50% | 0.40 |
| IP-Adapter FaceID SD3.5 | 2917 | 0.50% | 0.40 |
| StyleSANXL | 2908 | 0.50% | 0.40 |
| glide | 2908 | 0.50% | 0.40 |
| BigGAN | 2894 | 0.50% | 0.40 |
| StyleSwin | 2758 | 0.49% | 0.41 |
| sd-3.5 | 2085 | 0.42% | 0.47 |
| flux.1-dev | 2064 | 0.42% | 0.48 |
| sd-2.1 | 2021 | 0.42% | 0.48 |
| VQDiffusion | 2011 | 0.41% | 0.48 |
| (1197 others) | 105808 | 84.12% | |
