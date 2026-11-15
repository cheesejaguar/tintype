reference incumbent-gpu: 1% cutoff clean recall 95.71%, processed 82.08%, authentic flagged 1.29%

| Candidate | Profile | Authentic flagged (all test) | Clean recall | Processed recall | Clean authentic | Processed authentic | Floor |
|---|---|---|---|---|---|---|---|
| incumbent-gpu | 1% (default) | 125 of 9726 (1.29%) | 95.71% | 82.08% | 0.75% | 2.92% | reference |
| incumbent-gpu | 0.1% (--strict) | 22 of 9726 (0.23%) | 92.71% | 72.62% | 0.08% | 0.67% |  |
| incumbent-cpu | 1% (default) | 125 of 9726 (1.29%) | 95.69% | 82.21% | 0.74% | 2.96% | passes |
| incumbent-cpu | 0.1% (--strict) | 21 of 9726 (0.22%) | 92.71% | 72.75% | 0.07% | 0.67% |  |
| int8 | 1% (default) | 127 of 9726 (1.31%) | 95.79% | 82.33% | 0.76% | 2.96% | passes |
| int8 | 0.1% (--strict) | 22 of 9726 (0.23%) | 92.88% | 72.88% | 0.10% | 0.62% |  |
| student-8 | 1% (default) | 71 of 9726 (0.73%) | 78.49% | 51.04% | 0.60% | 1.12% | fails: clean recall, processed recall |
| student-8 | 0.1% (--strict) | 5 of 9726 (0.05%) | 63.65% | 36.79% | 0.04% | 0.08% |  |
| student-7 | 1% (default) | 69 of 9726 (0.71%) | 65.03% | 39.96% | 0.35% | 1.79% | fails: clean recall, processed recall |
| student-7 | 0.1% (--strict) | 6 of 9726 (0.06%) | 47.00% | 21.92% | 0.03% | 0.17% |  |
| student-6 | 1% (default) | 91 of 9726 (0.94%) | 28.60% | 19.71% | 0.98% | 0.79% | fails: clean recall, processed recall |
| student-6 | 0.1% (--strict) | 12 of 9726 (0.12%) | 16.33% | 7.96% | 0.11% | 0.17% |  |
