```
$ python3 -I scripts/family-gate.py --groups test-groups.tsv.gz --raw --system control=test.control.tsv.gz@1.0773115460069014,3.0021249596178134 --system control-seed2=test.control-seed2.tsv.gz@0.855318247091564,2.773889798196334 --system family=test.family.tsv.gz@1.045996594854998,2.8588852478318807 --system source=test.source.tsv.gz@1.2429650142537536,2.88599175329078 --label 1%,0.1% --new-family llamagen --new-family var --new-family infinity --new-family rar --new-family switti --new-family open-magvit2 --bootstrap 1000 --seed 1
```

Run in this directory (with the path to the script). The cutoffs after `@` are each model's raw thresholds for 1% and 0.1%, fitted on `calibrate`.

19326 items of 14526 parents scored by all 4 systems.
New families: infinity, llamagen, open-magvit2, rar, switti, var. A family's recall is over its unprocessed items; families with fewer than 100 of them are left out of the macro means. Cutoffs are raw thresholds on the mean view logit.

**At the cutoffs frozen for 1% FPR** (control 1.07731, control-seed2 0.855318, family 1.046, source 1.24297)

| Measure | control | control-seed2 | family | source | control-seed2 − control [95% over parents] | family − control [95% over parents] | source − control [95% over parents] |
|---|---|---|---|---|---|---|---|
| new-family macro recall | 91.54% | 91.00% | 93.23% | 90.31% | -0.54 [-1.02, -0.07] | +1.69 [+1.15, +2.16] | -1.23 [-1.70, -0.76] |
| older-family macro recall | 99.71% | 99.83% | 99.92% | 99.42% | +0.13 [+0.00, +0.35] | +0.21 [+0.00, +0.48] | -0.29 [-0.62, +0.00] |
| recall, processed copies | 82.08% | 81.46% | 84.54% | 80.33% | -0.62 [-1.78, +0.50] | +2.46 [+1.40, +3.61] | -1.75 [-2.83, -0.56] |
| recall, unprocessed | 95.61% | 95.38% | 96.56% | 94.93% | -0.24 [-0.49, +0.01] | +0.94 [+0.67, +1.19] | -0.68 [-0.94, -0.43] |
| false-positive rate, unprocessed | 0.75% | 1.11% | 0.66% | 0.57% | +0.35 [+0.19, +0.52] | -0.10 [-0.24, +0.03] | -0.18 [-0.30, -0.07] |
| false-positive rate, processed copies | 3.04% | 3.25% | 2.96% | 2.21% | +0.21 [-0.34, +0.74] | -0.08 [-0.57, +0.45] | -0.83 [-1.34, -0.35] |

| Family | Kind | Unprocessed generated items | Recall, control | Recall, control-seed2 | Recall, family | Recall, source |
|---|---|---|---|---|---|---|
| infinity | new | 1500 | 100.00% | 100.00% | 100.00% | 99.93% |
| llamagen | new | 600 | 98.50% | 98.17% | 99.00% | 98.50% |
| open-magvit2 | new | 600 | 71.17% | 69.17% | 76.67% | 67.50% |
| rar | new | 1500 | 99.27% | 99.33% | 99.73% | 99.27% |
| switti | new | 600 | 99.83% | 100.00% | 100.00% | 99.83% |
| var | new | 600 | 80.50% | 79.33% | 84.00% | 76.83% |
| flux | older | 800 | 99.62% | 99.75% | 99.75% | 99.50% |
| janus-pro | older | 600 | 100.00% | 100.00% | 100.00% | 100.00% |
| sd3 | older | 400 | 99.50% | 99.75% | 100.00% | 98.75% |

| Authentic source | Unprocessed items | Flagged, control | Flagged, control-seed2 | Flagged, family | Flagged, source |
|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 34 (0.85%) | 62 (1.55%) | 36 (0.90%) | 24 (0.60%) |
| floreview-apple | 249 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-doogee | 42 | 4 (9.52%) | 2 (4.76%) | 3 (7.14%) | 4 (9.52%) |
| floreview-google | 125 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-huawei | 249 | 1 (0.40%) | 2 (0.80%) | 1 (0.40%) | 0 (0.00%) |
| floreview-lenovo | 42 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-lg | 126 | 1 (0.79%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-motorola | 248 | 2 (0.81%) | 1 (0.40%) | 0 (0.00%) | 1 (0.40%) |
| floreview-oneplus | 84 | 1 (1.19%) | 0 (0.00%) | 0 (0.00%) | 1 (1.19%) |
| floreview-samsung | 377 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-sony | 40 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-xiaomi | 292 | 6 (2.05%) | 6 (2.05%) | 4 (1.37%) | 6 (2.05%) |
| hdrplus | 1456 | 6 (0.41%) | 8 (0.55%) | 4 (0.27%) | 6 (0.41%) |

**At the cutoffs frozen for 0.1% FPR** (control 3.00212, control-seed2 2.77389, family 2.85889, source 2.88599)

| Measure | control | control-seed2 | family | source | control-seed2 − control [95% over parents] | family − control [95% over parents] | source − control [95% over parents] |
|---|---|---|---|---|---|---|---|
| new-family macro recall | 87.16% | 86.35% | 88.76% | 86.44% | -0.81 [-1.38, -0.30] | +1.60 [+1.03, +2.12] | -0.72 [-1.15, -0.23] |
| older-family macro recall | 98.71% | 99.08% | 98.75% | 98.92% | +0.38 [+0.12, +0.67] | +0.04 [-0.24, +0.37] | +0.21 [-0.05, +0.49] |
| recall, processed copies | 72.33% | 71.50% | 74.46% | 71.88% | -0.83 [-1.95, +0.26] | +2.12 [+1.06, +3.14] | -0.46 [-1.35, +0.47] |
| recall, unprocessed | 92.92% | 92.62% | 93.83% | 92.62% | -0.29 [-0.58, -0.01] | +0.92 [+0.61, +1.22] | -0.29 [-0.55, -0.04] |
| false-positive rate, unprocessed | 0.12% | 0.16% | 0.12% | 0.08% | +0.04 [-0.04, +0.12] | +0.00 [-0.04, +0.04] | -0.04 [-0.09, +0.00] |
| false-positive rate, processed copies | 0.62% | 0.67% | 0.67% | 0.50% | +0.04 [-0.16, +0.26] | +0.04 [-0.17, +0.26] | -0.13 [-0.53, +0.15] |

| Family | Kind | Unprocessed generated items | Recall, control | Recall, control-seed2 | Recall, family | Recall, source |
|---|---|---|---|---|---|---|
| infinity | new | 1500 | 99.33% | 99.40% | 99.33% | 99.20% |
| llamagen | new | 600 | 96.50% | 96.17% | 98.00% | 96.67% |
| open-magvit2 | new | 600 | 59.50% | 57.50% | 64.67% | 57.67% |
| rar | new | 1500 | 98.27% | 98.20% | 99.20% | 98.27% |
| switti | new | 600 | 98.83% | 99.33% | 99.17% | 99.33% |
| var | new | 600 | 70.50% | 67.50% | 72.17% | 67.50% |
| flux | older | 800 | 97.38% | 98.25% | 97.25% | 98.25% |
| janus-pro | older | 600 | 100.00% | 100.00% | 100.00% | 100.00% |
| sd3 | older | 400 | 98.75% | 99.00% | 99.00% | 98.50% |

| Authentic source | Unprocessed items | Flagged, control | Flagged, control-seed2 | Flagged, family | Flagged, source |
|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 6 (0.15%) | 12 (0.30%) | 6 (0.15%) | 5 (0.13%) |
| floreview-apple | 249 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-doogee | 42 | 1 (2.38%) | 0 (0.00%) | 1 (2.38%) | 1 (2.38%) |
| floreview-google | 125 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-huawei | 249 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-lenovo | 42 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-lg | 126 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-motorola | 248 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-oneplus | 84 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-samsung | 377 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-sony | 40 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-xiaomi | 292 | 2 (0.68%) | 0 (0.00%) | 2 (0.68%) | 0 (0.00%) |
| hdrplus | 1456 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |

