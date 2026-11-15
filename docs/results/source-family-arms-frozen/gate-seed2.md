```
$ python3 -I scripts/family-gate.py --groups test-groups.tsv.gz --raw --system control-seed2=test.control-seed2.tsv.gz@0.855318247091564,2.773889798196334 --system family-seed2=test.family-seed2.tsv.gz@1.31504008601601,2.717194663814126 --system control=test.control.tsv.gz@1.0773115460069014,3.0021249596178134 --system family=test.family.tsv.gz@1.045996594854998,2.8588852478318807 --label 1%,0.1% --new-family llamagen --new-family var --new-family infinity --new-family rar --new-family switti --new-family open-magvit2 --bootstrap 1000 --seed 1
```

The second seed of arm B, read against the second seed of the control (the first system). Run in this directory (with the path to the script). The cutoffs after `@` are the raw thresholds of each model for 1% and 0.1%, fitted on `calibrate`.

19326 items of 14526 parents scored by all 4 systems.
New families: infinity, llamagen, open-magvit2, rar, switti, var. A family's recall is over its unprocessed items; families with fewer than 100 of them are left out of the macro means. Cutoffs are raw thresholds on the mean view logit.

**At the cutoffs frozen for 1% FPR** (control-seed2 0.855318, family-seed2 1.31504, control 1.07731, family 1.046)

| Measure | control-seed2 | family-seed2 | control | family | family-seed2 − control-seed2 [95% over parents] | control − control-seed2 [95% over parents] | family − control-seed2 [95% over parents] |
|---|---|---|---|---|---|---|---|
| new-family macro recall | 91.00% | 92.27% | 91.54% | 93.23% | +1.27 [+0.74, +1.84] | +0.54 [+0.07, +1.03] | +2.23 [+1.69, +2.77] |
| older-family macro recall | 99.83% | 99.62% | 99.71% | 99.92% | -0.21 [-0.44, -0.04] | -0.13 [-0.35, +0.00] | +0.08 [+0.00, +0.26] |
| recall, processed copies | 81.46% | 81.83% | 82.08% | 84.54% | +0.38 [-0.49, +1.30] | +0.62 [-0.49, +1.79] | +3.08 [+1.96, +4.31] |
| recall, unprocessed | 95.38% | 95.97% | 95.61% | 96.56% | +0.60 [+0.31, +0.89] | +0.24 [-0.01, +0.50] | +1.18 [+0.90, +1.45] |
| false-positive rate, unprocessed | 1.11% | 0.51% | 0.75% | 0.66% | -0.60 [-0.78, -0.43] | -0.35 [-0.52, -0.19] | -0.45 [-0.62, -0.30] |
| false-positive rate, processed copies | 3.25% | 2.50% | 3.04% | 2.96% | -0.75 [-1.31, -0.17] | -0.21 [-0.73, +0.34] | -0.29 [-0.77, +0.20] |

| Family | Kind | Unprocessed generated items | Recall, control-seed2 | Recall, family-seed2 | Recall, control | Recall, family |
|---|---|---|---|---|---|---|
| infinity | new | 1500 | 100.00% | 100.00% | 100.00% | 100.00% |
| llamagen | new | 600 | 98.17% | 98.83% | 98.50% | 99.00% |
| open-magvit2 | new | 600 | 69.17% | 75.17% | 71.17% | 76.67% |
| rar | new | 1500 | 99.33% | 99.47% | 99.27% | 99.73% |
| switti | new | 600 | 100.00% | 100.00% | 99.83% | 100.00% |
| var | new | 600 | 79.33% | 80.17% | 80.50% | 84.00% |
| flux | older | 800 | 99.75% | 99.38% | 99.62% | 99.75% |
| janus-pro | older | 600 | 100.00% | 100.00% | 100.00% | 100.00% |
| sd3 | older | 400 | 99.75% | 99.50% | 99.50% | 100.00% |

| Authentic source | Unprocessed items | Flagged, control-seed2 | Flagged, family-seed2 | Flagged, control | Flagged, family |
|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 62 (1.55%) | 27 (0.68%) | 34 (0.85%) | 36 (0.90%) |
| floreview-apple | 249 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-doogee | 42 | 2 (4.76%) | 1 (2.38%) | 4 (9.52%) | 3 (7.14%) |
| floreview-google | 125 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-huawei | 249 | 2 (0.80%) | 0 (0.00%) | 1 (0.40%) | 1 (0.40%) |
| floreview-lenovo | 42 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-lg | 126 | 0 (0.00%) | 0 (0.00%) | 1 (0.79%) | 0 (0.00%) |
| floreview-motorola | 248 | 1 (0.40%) | 0 (0.00%) | 2 (0.81%) | 0 (0.00%) |
| floreview-oneplus | 84 | 0 (0.00%) | 0 (0.00%) | 1 (1.19%) | 0 (0.00%) |
| floreview-samsung | 377 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-sony | 40 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-xiaomi | 292 | 6 (2.05%) | 4 (1.37%) | 6 (2.05%) | 4 (1.37%) |
| hdrplus | 1456 | 8 (0.55%) | 5 (0.34%) | 6 (0.41%) | 4 (0.27%) |

**At the cutoffs frozen for 0.1% FPR** (control-seed2 2.77389, family-seed2 2.71719, control 3.00212, family 2.85889)

| Measure | control-seed2 | family-seed2 | control | family | family-seed2 − control-seed2 [95% over parents] | control − control-seed2 [95% over parents] | family − control-seed2 [95% over parents] |
|---|---|---|---|---|---|---|---|
| new-family macro recall | 86.35% | 89.01% | 87.16% | 88.76% | +2.66 [+2.12, +3.22] | +0.81 [+0.30, +1.39] | +2.41 [+1.81, +3.03] |
| older-family macro recall | 99.08% | 98.79% | 98.71% | 98.75% | -0.29 [-0.63, +0.04] | -0.38 [-0.66, -0.12] | -0.33 [-0.61, -0.08] |
| recall, processed copies | 71.50% | 75.08% | 72.33% | 74.46% | +3.58 [+2.38, +4.92] | +0.83 [-0.24, +1.95] | +2.96 [+1.81, +4.07] |
| recall, unprocessed | 92.62% | 93.97% | 92.92% | 93.83% | +1.35 [+1.03, +1.67] | +0.29 [+0.03, +0.58] | +1.21 [+0.89, +1.56] |
| false-positive rate, unprocessed | 0.16% | 0.14% | 0.12% | 0.12% | -0.03 [-0.12, +0.07] | -0.04 [-0.12, +0.04] | -0.04 [-0.12, +0.04] |
| false-positive rate, processed copies | 0.67% | 0.71% | 0.62% | 0.67% | +0.04 [-0.14, +0.23] | -0.04 [-0.26, +0.16] | +0.00 [-0.20, +0.21] |

| Family | Kind | Unprocessed generated items | Recall, control-seed2 | Recall, family-seed2 | Recall, control | Recall, family |
|---|---|---|---|---|---|---|
| infinity | new | 1500 | 99.40% | 99.80% | 99.33% | 99.33% |
| llamagen | new | 600 | 96.17% | 97.17% | 96.50% | 98.00% |
| open-magvit2 | new | 600 | 57.50% | 66.33% | 59.50% | 64.67% |
| rar | new | 1500 | 98.20% | 98.73% | 98.27% | 99.20% |
| switti | new | 600 | 99.33% | 99.33% | 98.83% | 99.17% |
| var | new | 600 | 67.50% | 72.67% | 70.50% | 72.17% |
| flux | older | 800 | 98.25% | 97.38% | 97.38% | 97.25% |
| janus-pro | older | 600 | 100.00% | 100.00% | 100.00% | 100.00% |
| sd3 | older | 400 | 99.00% | 99.00% | 98.75% | 99.00% |

| Authentic source | Unprocessed items | Flagged, control-seed2 | Flagged, family-seed2 | Flagged, control | Flagged, family |
|---|---|---|---|---|---|
| csafe-iphone14pro | 3996 | 12 (0.30%) | 7 (0.18%) | 6 (0.15%) | 6 (0.15%) |
| floreview-apple | 249 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-doogee | 42 | 0 (0.00%) | 0 (0.00%) | 1 (2.38%) | 1 (2.38%) |
| floreview-google | 125 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-huawei | 249 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-lenovo | 42 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-lg | 126 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-motorola | 248 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-oneplus | 84 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-samsung | 377 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-sony | 40 | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) | 0 (0.00%) |
| floreview-xiaomi | 292 | 0 (0.00%) | 2 (0.68%) | 2 (0.68%) | 2 (0.68%) |
| hdrplus | 1456 | 0 (0.00%) | 1 (0.07%) | 0 (0.00%) | 0 (0.00%) |

