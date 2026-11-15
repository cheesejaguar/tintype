#!/usr/bin/env python3
"""Gate table for two arms on a set with processed copies.

    compare-arms.py GROUPS.tsv NAME=scores.tsv[@cutoff] NAME=scores.tsv[@cutoff]

GROUPS.tsv lists the set: path first, `parent_id` and `transform` either in
manifest columns 5 and 9, or (a cut-down archive: path and the five
grouping columns) in columns 2 and 6. Score files are `tintype score
--all-views` output (`path label generator dataset prob view-logit...`),
plain or gzip. Without `@cutoff` an item is flagged when its probability is
above 0.5, the cutoff frozen in the spec the file was scored with; with
`@cutoff` when the mean of its view logits is above that raw-logit cutoff
(the threshold `tintype calibrate` prints), so one score file serves every
cutoff fitted for those weights.

Prints, for the processed copies, the clean rows and each transform:
authentic items flagged and recall at the frozen cutoff, and recall at the
1% empirical cutoff of the rows themselves (a ranking measure). Then the
difference in processed recall, first arm minus second, with a 95%
percentile bootstrap over generated parents. Standard library only.
"""

import gzip
import random
import sys


def open_text(path):
    return gzip.open(path, "rt") if str(path).endswith(".gz") else open(path)


def read_groups(path):
    """{path: (transform or "none", parent_id)}."""
    groups = {}
    with open_text(path) as stream:
        for line in stream:
            f = line.rstrip("\n").split("\t")
            parent, transform = (f[4], f[8]) if len(f) >= 9 else (f[1], f[5])
            groups[f[0]] = (transform if transform not in ("", "-") else "none", parent)
    return groups


def read_scores(spec):
    """(name, {path: (label, score)}, cutoff) from `NAME=file[@cutoff]`."""
    name, _, rest = spec.partition("=")
    path, _, cutoff = rest.partition("@")
    scores = {}
    with open_text(path) as stream:
        for line in stream:
            f = line.rstrip("\n").split("\t")
            views = [float(x) for x in f[5:]]
            scores[f[0]] = (int(f[1]), sum(views) / len(views) if cutoff else float(f[4]))
    return name, scores, float(cutoff) if cutoff else 0.5


def cutoff_at(real, fpr):
    ranked = sorted(real, reverse=True)
    return ranked[min(int(fpr * len(ranked)), len(ranked) - 1)]


def counts(scores, paths, cutoff):
    """(authentic flagged, authentic, generated found, generated, found at the rows' own 1% cutoff)."""
    real = [scores[p][1] for p in paths if scores[p][0] == 0]
    generated = [scores[p][1] for p in paths if scores[p][0] == 1]
    own = cutoff_at(real, 0.01) if real else float("inf")
    return sum(s > cutoff for s in real), len(real), sum(s > cutoff for s in generated), len(generated), sum(s > own for s in generated)


def paired_difference(found, draws=2000, seed=20261114):
    """`found`: {parent: [(first arm found, second arm found), ...]}. Points, with a 95% interval."""
    parents = sorted(found)

    def difference(sample):
        rows = [pair for parent in sample for pair in found[parent]]
        return 100 * (sum(a for a, _ in rows) - sum(b for _, b in rows)) / len(rows)

    rng = random.Random(seed)
    resampled = sorted(difference(rng.choices(parents, k=len(parents))) for _ in range(draws))
    return difference(parents), resampled[int(0.025 * draws)], resampled[min(int(0.975 * draws), draws - 1)]


def self_test():
    assert cutoff_at([0.1, 0.9, 0.5], 0.01) == 0.9 and cutoff_at(list(range(200)), 0.01) == 197
    scores = {"a": (0, 0.2), "b": (0, 0.8), "c": (1, 0.9), "d": (1, 0.4)}
    assert counts(scores, list(scores), 0.5) == (1, 2, 1, 2, 1)
    found = {"p": [(True, False), (True, True)], "q": [(False, False)]}
    point, low, high = paired_difference(found, draws=200, seed=1)
    assert abs(point - 100 / 3) < 1e-9 and low <= point <= high
    assert paired_difference({"p": [(True, True)]}, draws=50) == (0.0, 0.0, 0.0)
    print("compare-arms self-test: ok")


def main():
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    groups = read_groups(sys.argv[1])
    arms = [read_scores(spec) for spec in sys.argv[2:]]
    for name, scores, _ in arms:
        if scores.keys() != groups.keys():
            sys.exit(f"{name}: the score file and the set list different files")
    transforms = sorted({t for t, _ in groups.values()} - {"none"})
    slices = [("processed, every transform", lambda t: t != "none"), ("clean", lambda t: t == "none")]
    slices += [(t, (lambda wanted: lambda t: t == wanted)(t)) for t in transforms]
    head = " | ".join(f"{name}: authentic flagged | {name}: recall | {name}: recall at the rows' own 1% cutoff" for name, _, _ in arms)
    print(f"| Rows | {head} |")
    print("|---|" + "---|" * (3 * len(arms)))
    for label, keep in slices:
        paths = [p for p in groups if keep(groups[p][0])]
        cells = []
        for _, scores, cutoff in arms:
            fp, real, tp, generated, own = counts(scores, paths, cutoff)
            cells += [f"{fp} of {real:,}", f"{tp} of {generated:,} ({100 * tp / generated:.1f}%)", f"{100 * own / generated:.1f}%"]
        print(f"| {label} | " + " | ".join(cells) + " |")
    (first, a, cut_a), (second, b, cut_b) = arms
    found = {}
    for p, (transform, parent) in groups.items():
        if transform != "none" and a[p][0] == 1:
            found.setdefault(parent, []).append((a[p][1] > cut_a, b[p][1] > cut_b))
    point, low, high = paired_difference(found)
    print(f"\nProcessed recall at the frozen cutoffs, {first} − {second}: {point:+.2f} points "
          f"(95% bootstrap over {len(found)} generated parents: {low:+.2f} to {high:+.2f}).")


if __name__ == "__main__":
    main()
