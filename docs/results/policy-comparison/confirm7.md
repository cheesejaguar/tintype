# Policy comparison on the confirm7 archive (exploratory)

These archived suites were used to develop the detector. Cutoffs are chosen on the cohort they are reported on: a ranking comparison, not a transfer result and not a decision about what ships. Routed shares are measured workload; end-to-end savings depend on the cohort and the hardware.

## All photos: 23052 photos, 12305 generated, 10747 authentic

Cutoffs chosen on this cohort, so exploratory; flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 10 FP), points vs Tintype default | Recall at 1% FPR (<= 107 FP), points vs Tintype default | Recall at 107 FP (1.00% FPR) | Recall at 329 FP (3.06% FPR) | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|---:|---:|
| Tintype default | 0/23052 (0.00%) | 7174/12305 (58.30%) +0.00 | 10887/12305 (88.48%) +0.00 | 10887/12305 (88.48%) | 11468/12305 (93.20%) | 11038, 130 (89.70%, 1.21%) |
| Tintype sensitive | 0/23052 (0.00%) | 6572/12305 (53.41%) -4.89 | 10608/12305 (86.21%) -2.27 | 10608/12305 (86.21%) | 11670/12305 (94.84%) | 11812, 459 (95.99%, 4.27%) |
| Tintype strict | 0/23052 (0.00%) | 7759/12305 (63.06%) +4.75 | 10647/12305 (86.53%) -1.95 | 10647/12305 (86.53%) | 11238/12305 (91.33%) | 10135, 48 (82.36%, 0.45%) |
| Shipped CF blend (stored scores) | 0/23052 (0.00%) | 6754/12305 (54.89%) -3.41 | 10486/12305 (85.22%) -3.26 | 10486/12305 (85.22%) | 11504/12305 (93.49%) | 11504, 329 (93.49%, 3.06%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 11068/23052 (48.01%) | 6754/12305 (54.89%) -3.41 | 10486/12305 (85.22%) -3.26 | 10486/12305 (85.22%) | 11504/12305 (93.49%) | 11504, 329 (93.49%, 3.06%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 2417/23052 (10.48%) | 7382/12305 (59.99%) +1.69 | 10872/12305 (88.35%) -0.12 | 10872/12305 (88.35%) | 11584/12305 (94.14%) | 11413, 213 (92.75%, 1.98%) |

Strongest single-model profile at a re-chosen cutoff, chosen on this cohort, so exploratory: 0.1% FPR: Tintype strict (7759/12305); 1% FPR: Tintype default (10887/12305); 107 FP: Tintype default (10887/12305); 329 FP: Tintype sensitive (11670/12305).

Sets (of 30) where precision and recall both reach Community Forensics at its own cutoff:

| Policy | at 0.1% FPR | at 1% FPR | at 107 FP | at 329 FP | at own cutoff |
|---|---:|---:|---:|---:|---:|
| Tintype default | 4/30 | 12/30 | 12/30 | 14/30 | 12/30 |
| Tintype sensitive | 3/30 | 13/30 | 13/30 | 14/30 | 14/30 |
| Tintype strict | 5/30 | 12/30 | 12/30 | 12/30 | 12/30 |
| Shipped CF blend (stored scores) | 3/30 | 13/30 | 13/30 | 26/30 | 26/30 |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 3/30 | 13/30 | 13/30 | 26/30 | 26/30 |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 4/30 | 14/30 | 14/30 | 20/30 | 21/30 |

Stored policies against the bands computed from the separate scores (verdicts at the own cutoff):

| Stored policy | Computed band | Verdicts that differ | Stored: probabilities that differ from the first stage | Computed: photos inside the band |
|---|---:|---:|---:|---:|
| Shipped CF blend (stored scores) | Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 0/23052 | 11064 | 11068 |
| Shipped CF blend (stored scores) | Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 207/23052 | 11064 | 2417 |

Gates on Tintype default with Community Forensics behind them, both at their own cutoffs. Tintype default misses 1267 of 12305 generated photos (Community Forensics flags 598 of them) and flags 130 of 10747 authentic photos (Community Forensics clears 103).

| Gate | Routed | Misses inside | Misses outside | Outside, flagged by Community Forensics | False alarms outside | Outside, cleared by Community Forensics |
|---|---:|---:|---:|---:|---:|---:|
| -7 < tintype logit < 0.5 | 11068/23052 (48.01%) | 1266 | 1 | 0 | 99 | 78 |
| -3 < tintype logit < 0.5 | 2417/23052 (10.48%) | 938 | 329 | 120 | 99 | 78 |
| |tintype logit| <= 1 (sweep) | 984/23052 (4.27%) | 335 | 932 | 389 | 73 | 57 |
| |tintype logit| <= 2 (sweep) | 2574/23052 (11.17%) | 635 | 632 | 233 | 32 | 22 |
| |tintype logit| <= 3 (sweep) | 5206/23052 (22.58%) | 938 | 329 | 120 | 14 | 11 |
| |tintype logit| <= 4 (sweep) | 7489/23052 (32.49%) | 1069 | 198 | 80 | 10 | 8 |
| |tintype logit| <= 6 (sweep) | 17513/23052 (75.97%) | 1256 | 11 | 3 | 1 | 1 |
| |tintype logit| <= 8 (sweep) | 22908/23052 (99.38%) | 1267 | 0 | 0 | 0 | 0 |

A symmetric margin gate that takes in 50% of the misses needs |logit| <= 1.98, which routes 2541/23052 (11.02%); 90% of the misses needs |logit| <= 4.36, which routes 9307/23052 (40.37%); 100% of the misses needs |logit| <= 7.00, which routes 21529/23052 (93.39%).
