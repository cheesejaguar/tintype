```
$ tintype sampling-shares $(cat runs/v7/manifests.txt) --family-map runs/ws4/families.tsv --rebalance-families 0.5 --write-weights runs/ws4/family-weights.tsv > /dev/null
$ cat runs/ws4/family-weights.tsv
```

```
# Family weights that move each AI family's share of AI draws to share^0.5, renormalised.
# Written by `tintype sampling-shares --rebalance-families 0.5` from 25 manifests, alpha 0.5, validation 3%.
family	autoregressive-vq	3.559478
family	commercial-unknown	1.605134
family	gan	1.981018
family	hybrid	6.036148
family	latent-diffusion	0.537744
family	other	10.526434
family	pixel-diffusion	3.997178
family	unknown	1.979201
family	video-diffusion	2.028023
```

```
$ tintype sampling-shares $(cat runs/v7/manifests.txt) --family-map runs/ws4/families.tsv --weights-file runs/ws4/family-weights.tsv --family-weight other=1 --top 25
```

train 356038 images (16 real sources, 1222 generators), val pool 11025

Sampling shares of 356038 items (122119 real, 233919 AI) from 25 manifests; generators weigh count^0.5, times the manifest and family weights given.
A share is of the draws of its class; each class is half of all draws. "Lift" is the share divided by the share of the class's items.

| Manifest | Pool | Real items | Share of real draws | AI items | Share of AI draws |
|---|---|---|---|---|---|
| openfake | main | 11669 | 9.82% | 11571 | 4.51% |
| openfake2 | main | 5949 | 5.01% | 5664 | 2.25% |
| openfake-reddit | main | 2896 | 4.25% | 15292 | 2.30% |
| frontier | main | 0 | 0.00% | 17558 | 10.83% |
| frontier-heldout | main | 0 | 0.00% | 3222 | 1.91% |
| nano-banana | main | 0 | 0.00% | 1485 | 0.58% |
| defactify | main | 6810 | 6.52% | 33891 | 7.67% |
| commfor-small | main | 23290 | 18.97% | 34834 | 16.89% |
| openfake-test-reals | main | 6874 | 6.45% | 0 | 0.00% |
| sofake-set | main | 2916 | 4.27% | 4138 | 2.62% |
| commfor-small2 | main | 28197 | 13.27% | 43525 | 29.99% |
| tiny-genimage | main | 13580 | 7.95% | 13558 | 6.71% |
| stylegan3 | main | 0 | 0.00% | 1945 | 0.64% |
| tpdne | main | 0 | 0.00% | 2913 | 0.84% |
| ip-adapter | main | 0 | 0.00% | 5834 | 0.55% |
| celeba-hq | main | 7253 | 6.73% | 0 | 0.00% |
| vidguard | main | 3746 | 4.84% | 4842 | 1.43% |
| genbuster | main | 4650 | 5.39% | 4869 | 2.00% |
| genvideo100k | main | 0 | 0.00% | 19373 | 3.67% |
| aigvd | main | 0 | 0.00% | 5815 | 3.19% |
| k400 | main | 1936 | 2.46% | 0 | 0.00% |
| pexels | main | 79 | 0.31% | 0 | 0.00% |
| k400b | main | 1940 | 2.46% | 0 | 0.00% |
| pexelsb | main | 334 | 1.30% | 0 | 0.00% |
| aigvd2 | main | 0 | 0.00% | 3590 | 1.41% |

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
| latent-diffusion | 79100 | 39.64% | 1.17 |
| commercial-unknown | 23389 | 13.28% | 1.33 |
| unknown | 49551 | 10.77% | 0.51 |
| gan | 27982 | 10.76% | 0.90 |
| video-diffusion | 35258 | 10.51% | 0.70 |
| autoregressive-vq | 9520 | 5.99% | 1.47 |
| pixel-diffusion | 6768 | 5.33% | 1.84 |
| hybrid | 1933 | 3.53% | 4.27 |
| other | 418 | 0.19% | 1.08 |

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
| tamingTransformers | 5424 | 2.47% | 1.06 |
| reddit-wild | 15292 | 2.30% | 0.35 |
| glide | 2908 | 2.03% | 1.63 |
| OpenSora | 10669 | 1.97% | 0.43 |
| ZeroScope | 9673 | 1.88% | 0.45 |
| GLM-Image | 967 | 1.77% | 4.27 |
| HunyuanImage-3.0 | 966 | 1.77% | 4.27 |
| genimage-GLIDE | 1932 | 1.65% | 2.00 |
| genimage-ADM | 1928 | 1.65% | 2.00 |
| defactify-gen3 | 6789 | 1.53% | 0.53 |
| defactify-gen5 | 6785 | 1.53% | 0.53 |
| defactify-gen1 | 6779 | 1.53% | 0.53 |
| defactify-gen4 | 6775 | 1.53% | 0.53 |
| defactify-gen2 | 6763 | 1.53% | 0.53 |
| VQDiffusion | 2011 | 1.50% | 1.75 |
| genimage-VQDM | 1951 | 1.48% | 1.77 |
| GigaGAN | 5797 | 1.42% | 0.57 |
| CogVideoX | 4855 | 1.33% | 0.64 |
| StyleGAN2 | 4154 | 1.20% | 0.68 |
| HunyuanVideo | 3591 | 1.14% | 0.74 |
| StyleGAN3 | 3235 | 1.06% | 0.77 |
| StyleSANXL | 2908 | 1.01% | 0.81 |
| BigGAN | 2894 | 1.00% | 0.81 |
| StyleSwin | 2758 | 0.98% | 0.83 |
| EasyAnimate | 1896 | 0.83% | 1.03 |
| (1197 others) | 114219 | 61.90% | |
