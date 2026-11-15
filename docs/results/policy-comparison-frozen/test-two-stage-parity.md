# Policy comparison

19326 photos scored by every system (commfor, tintype, tintype-narrow, tintype-second).

Split: 0 fit, 0 calibrate, 19326 test photos; 0 scored photos are in no set and are left out.

## Test set: 19326 photos, 9600 generated, 9726 authentic

Cutoffs chosen on the test set itself (a ranking comparison, not a frozen operating point); flagged strictly above the cutoff.

| Policy | Routed | Recall at 0.1% FPR (<= 9 FP), points vs Tintype default | Recall at 1% FPR (<= 97 FP), points vs Tintype default | At own cutoff: TP, FP (recall, FPR) |
|---|---:|---:|---:|---:|
| Tintype default | 0/19326 (0.00%) | 8073/9600 (84.09%) +0.00 | 8869/9600 (92.39%) +0.00 | 8726, 65 (90.90%, 0.67%) |
| Shipped CF blend (stored scores) | 0/19326 (0.00%) | 7650/9600 (79.69%) -4.41 | 9149/9600 (95.30%) +2.92 | 9152, 101 (95.33%, 1.04%) |
| tintype-narrow | 0/19326 (0.00%) | 7592/9600 (79.08%) -5.01 | 9072/9600 (94.50%) +2.11 | 9060, 83 (94.38%, 0.85%) |
| Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 10379/19326 (53.70%) | 7650/9600 (79.69%) -4.41 | 9149/9600 (95.30%) +2.92 | 9152, 101 (95.33%, 1.04%) |
| Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 1775/19326 (9.18%) | 7592/9600 (79.08%) -5.01 | 9072/9600 (94.50%) +2.11 | 9060, 83 (94.38%, 0.85%) |

Stored policies against the bands computed from the separate scores (verdicts at the own cutoff):

| Stored policy | Computed band | Verdicts that differ | Stored: probabilities that differ from the first stage | Computed: photos inside the band |
|---|---:|---:|---:|---:|
| Shipped CF blend (stored scores) | Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 0/19326 | 10371 | 10379 |
| Shipped CF blend (stored scores) | Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 110/19326 | 10371 | 1775 |
| tintype-narrow | Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 110/19326 | 1775 | 10379 |
| tintype-narrow | Narrowed CF blend: band (-3, 0.5), weight 0.5, threshold 0 | 0/19326 | 1775 | 1775 |
