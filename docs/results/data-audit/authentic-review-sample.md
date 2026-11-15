```
$ python3 -I scripts/sample-authentic-review.py --scores runs/scores-v7/calib_images.tsv --out runs/ws4/authentic-review.tsv --per-stratum 5 --tail 200 --tail-per-source 20 --seed 1
```

36110 authentic-labelled items in 20 sources and 95 source-by-band strata; seed 1.
runs/ws4/authentic-review.tsv: 739 items to review (414 from the strata, 200 in the overall tail, 400 in a source's tail; an item can be several).
Nothing was relabelled.

Stratum draws per score band (drawn of present):

| Source | Authentic items | <0.01 | 0.01-0.1 | 0.1-0.5 | 0.5-0.9 | 0.9-0.99 | >=0.99 | In the review list |
|---|---|---|---|---|---|---|---|---|
| real:Bluesky | 433 | 5 of 136 | 5 of 168 | 5 of 74 | 5 of 33 | 5 of 18 | 4 of 4 | 40 |
| real:Reddit | 5861 | 5 of 3539 | 5 of 1673 | 5 of 472 | 5 of 119 | 5 of 43 | 5 of 15 | 53 |
| real:Tumblr | 735 | 5 of 368 | 5 of 218 | 5 of 82 | 5 of 50 | 5 of 17 | – | 40 |
| real:aigibench | 11798 | 5 of 9306 | 5 of 1877 | 5 of 447 | 5 of 130 | 5 of 31 | 5 of 7 | 44 |
| real:celeba-hq | 247 | 5 of 228 | 5 of 18 | – | – | 1 of 1 | – | 25 |
| real:coco | 489 | 5 of 457 | 5 of 23 | 5 of 8 | 1 of 1 | – | – | 26 |
| real:commfor | 828 | 5 of 734 | 5 of 83 | 5 of 10 | 1 of 1 | – | – | 30 |
| real:communityai | 6000 | 5 of 3011 | 5 of 1790 | 5 of 807 | 5 of 287 | 5 of 87 | 5 of 18 | 78 |
| real:defactify | 190 | 5 of 184 | 5 of 5 | 1 of 1 | – | – | – | 25 |
| real:docci | 1242 | 5 of 1040 | 5 of 166 | 5 of 29 | 4 of 4 | 3 of 3 | – | 33 |
| real:ffhq | 72 | 5 of 69 | 2 of 2 | 1 of 1 | – | – | – | 25 |
| real:imagenet | 2934 | 5 of 2541 | 5 of 315 | 5 of 58 | 5 of 12 | 5 of 8 | – | 35 |
| real:laion | 228 | 5 of 161 | 5 of 49 | 5 of 11 | 4 of 4 | 3 of 3 | – | 30 |
| real:lhq | 85 | 5 of 80 | 5 of 5 | – | – | – | – | 25 |
| real:openfake-reddit | 122 | 5 of 101 | 5 of 15 | 5 of 5 | – | 1 of 1 | – | 25 |
| real:pexels | 276 | 5 of 202 | 5 of 64 | 5 of 7 | 1 of 1 | 2 of 2 | – | 28 |
| real:raise1k | 1000 | 5 of 592 | 5 of 295 | 5 of 83 | 5 of 20 | 5 of 9 | 1 of 1 | 38 |
| real:socialrf | 3000 | 5 of 1527 | 5 of 850 | 5 of 398 | 5 of 146 | 5 of 53 | 5 of 26 | 76 |
| real:sofake-set | 86 | 5 of 65 | 5 of 18 | 1 of 1 | 1 of 1 | 1 of 1 | – | 25 |
| real:wasitai | 484 | 5 of 180 | 5 of 185 | 5 of 81 | 5 of 31 | 5 of 6 | 1 of 1 | 38 |
