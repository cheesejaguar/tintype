#!/usr/bin/env python3
"""Per-operation, per-severity table for a scored paired stress suite.

For every transform of a suite made by `scripts/stress-photo.py` (or any
manifest with `parent_id` in column 5 and `transform` in column 9), on the
parents that have both the original and the processed copy scored:

- authentic items flagged at the frozen cutoff, original -> processed, and
  the shift in percentage points (with how many flipped each way);
- recall at the frozen cutoff, original -> processed;
- recall at 1% and at 0.1% empirical FPR, the cutoff taken on the processed
  authentic copies themselves (the ranking after processing);
- AUROC of the processed copies;
- what the cutoff that gives 1% FPR on the originals does on the processed
  copies (threshold transfer).

`tintype report --operating-points --slice-by transform --groups paired.tsv
--cluster-by parent` gives the per-transform rates at the frozen cutoff with
clustered bootstrap intervals; this script adds the paired columns that
need the original beside each copy. Cutoffs taken on the items they are
reported on are exploratory, as everywhere in this repository.

Score files are `tintype score` output (`path label generator dataset prob
...`, no header), or an archive with a header row (`--column` names the
score column), plain or gzip. With `--pair-by-name` no manifest is needed:
`original/x.jpg` is the parent of `<transform>/x.jpg` (the layout of
docs/results/frontier-jpeg-stress-scores.tsv.gz).

`--versus OTHER` compares two score columns of one archive (two
checkpoints scored on the same files): it adds the difference in recall at
1% FPR, `--column` minus `OTHER`, with a paired percentile bootstrap over
parents. Authentic and generated parents are resampled separately, both
columns see the same draws, and each column's cutoff is refitted in every
draw, so the interval carries the uncertainty of the cutoff too. It says
how much of a difference the choice of test images could explain; it says
nothing about how much two training runs of the same recipe differ.

Standard library only.
"""

import argparse
import csv
import gzip
import os
import random
import sys
from pathlib import Path


def open_text(path):
    return gzip.open(path, "rt", newline="") if str(path).endswith(".gz") else open(path, newline="")


def read_scores(path, column):
    """{normalised path: (label, score)} from one score file."""
    with open_text(path) as stream:
        rows = list(csv.reader(stream, delimiter="\t"))
    if not rows:
        raise ValueError(f"empty score file: {path}")
    index = 4
    if rows[0][0] == "path":
        if column not in rows[0]:
            raise ValueError(f"no column {column!r} in {path} (it has {', '.join(rows[0][4:])})")
        index = rows[0].index(column)
        rows = rows[1:]
    return {os.path.normpath(r[0]): (int(r[1]), float(r[index])) for r in rows if len(r) > index}


def cutoff_at(real, fpr):
    """The strictest score that lets at most `fpr` of `real` through; items
    are flagged strictly above it (as `tintype report` does)."""
    ranked = sorted(real, reverse=True)
    return ranked[min(int(fpr * len(ranked)), len(ranked) - 1)]


def auroc(real, generated):
    """Probability that a generated item outscores an authentic one, ties counting half."""
    if not real or not generated:
        return float("nan")
    ranked = sorted([(s, 0) for s in real] + [(s, 1) for s in generated])
    rank_sum, i = 0.0, 0
    while i < len(ranked):
        j = i
        while j + 1 < len(ranked) and ranked[j + 1][0] == ranked[i][0]:
            j += 1
        rank_sum += ((i + j) / 2 + 1) * sum(label for _, label in ranked[i:j + 1])
        i = j + 1
    n = len(generated)
    return (rank_sum - n * (n + 1) / 2) / (n * len(real))


def recall_at(real, generated, fpr):
    """How many of `generated` are flagged at the cutoff `fpr` gives on `real`."""
    cut = cutoff_at(real, fpr)
    return sum(s > cut for s in generated)


def paired_difference(real, generated, fpr, draws, seed):
    """Recall at `fpr` of the first score minus the second, in points, with a
    95% percentile interval. `real` and `generated` are lists of
    `(first, second)` score pairs of the same items."""
    def difference(r, g):
        return 100 * (recall_at([a for a, _ in r], [a for a, _ in g], fpr) - recall_at([b for _, b in r], [b for _, b in g], fpr)) / len(g)
    rng = random.Random(seed)
    resampled = sorted(difference(rng.choices(real, k=len(real)), rng.choices(generated, k=len(generated))) for _ in range(draws))
    return difference(real, generated), resampled[int(0.025 * draws)], resampled[min(int(0.975 * draws), draws - 1)]


def share(count, total):
    return f"{count}/{total} ({100 * count / total:.1f}%)" if total else "–"


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--scores", type=Path, action="append", required=True, help="score file (repeatable)")
    parser.add_argument("--manifest", type=Path, action="append", default=[], help="suite manifest with parent_id and transform (repeatable)")
    parser.add_argument("--root", type=Path, help="directory the manifests' paths are relative to, if not the manifest's own")
    parser.add_argument("--pair-by-name", action="store_true", help="pair `original/<name>` with `<transform>/<name>` instead of reading manifests")
    parser.add_argument("--column", default="tintype", help="score column of a file with a header row [tintype]")
    parser.add_argument("--frozen", type=float, default=0.5, help="the frozen cutoff; items are flagged strictly above it [0.5]")
    parser.add_argument("--versus", metavar="COLUMN", help="a second score column: add recall at 1%% FPR of --column minus this one, with a paired bootstrap interval")
    parser.add_argument("--draws", type=int, default=2000, help="bootstrap draws for --versus [2000]")
    parser.add_argument("--seed", type=int, default=20261114, help="bootstrap seed for --versus")
    parser.add_argument("--self-test", action="store_true", help="check the bootstrap on made-up scores and exit")
    if "--self-test" in sys.argv[1:]:
        self_test()
        return
    args = parser.parse_args()
    if bool(args.manifest) == args.pair_by_name:
        parser.error("give either --manifest (one or more) or --pair-by-name")
    if args.versus and args.draws < 40:
        parser.error("--draws must be at least 40 for a 95% interval")

    scores, other = {}, {}
    for path in args.scores:
        scores.update(read_scores(path, args.column))
        if args.versus:
            other.update(read_scores(path, args.versus))
    if args.versus and other.keys() != scores.keys():
        parser.error(f"--versus {args.versus}: not the same rows as --column {args.column}")

    # transform -> {parent: (label, score)}; "" is the original.
    suite, unscored = {}, 0
    if args.pair_by_name:
        for path, (label, score) in scores.items():
            transform, _, name = path.partition(os.sep)
            suite.setdefault("" if transform == "original" else transform, {})[name] = (label, score, other.get(path, (label, None))[1])
    else:
        for manifest in args.manifest:
            root = args.root if args.root else manifest.parent
            for line in manifest.read_text(encoding="utf-8").splitlines():
                fields = line.split("\t")
                if len(fields) < 5 or fields[4] in {"", "-"}:
                    parser.error(f"{manifest}: a row has no parent_id (column 5): {line!r}")
                transform = fields[8] if len(fields) > 8 and fields[8] != "-" else ""
                scored = scores.get(os.path.normpath(os.path.join(root, fields[0])))
                if scored is None:
                    unscored += 1
                    continue
                if scored[0] != int(fields[1]):
                    parser.error(f"{manifest}: label of {fields[0]} differs from the score file")
                suite.setdefault(transform, {})[fields[4]] = (*scored, other.get(os.path.normpath(os.path.join(root, fields[0])), (None, None))[1])
    originals = suite.pop("", {})
    if not originals or not suite:
        parser.error("found no scored originals, or no scored processed copies")
    if unscored:
        print(f"{unscored} manifest rows have no score and are left out.", file=sys.stderr)

    frozen = args.frozen
    print(f"Frozen cutoff {frozen}. Empirical cutoffs are taken on the rows they are reported on (exploratory).")
    print("Each row uses the parents scored both as originals and under that transform.\n")
    print("| Transform | Pairs: AI / real | Real flagged, original → processed | FPR shift (newly flagged, cleared) "
          "| Recall, original → processed | Recall at 1% FPR | Recall at 0.1% FPR | AUROC "
          "| Originals' 1% cutoff on processed: FPR, recall |"
          + (f" Recall at 1% FPR, {args.column} − {args.versus} (95% paired bootstrap) |" if args.versus else ""))
    print("|---|---|---|---|---|---|---|---|---|" + ("---|" if args.versus else ""))
    for transform, copies in [("original", originals), *suite.items()]:
        parents = [p for p in copies if p in originals]
        if any(originals[p][0] != copies[p][0] for p in parents):
            parser.error(f"{transform}: a copy and its parent have different labels")
        pairs = {label: [(originals[p][1], copies[p][1]) for p in parents if copies[p][0] == label] for label in (0, 1)}
        real, generated = [c for _, c in pairs[0]], [c for _, c in pairs[1]]
        flagged = lambda rows, k: sum(row[k] > frozen for row in rows)  # noqa: E731
        cells = [transform, f"{len(generated)} / {len(real)}"]
        if real:
            before, after = flagged(pairs[0], 0), flagged(pairs[0], 1)
            new = sum(o <= frozen < c for o, c in pairs[0])
            cleared = sum(c <= frozen < o for o, c in pairs[0])
            cells += [f"{share(before, len(real))} → {share(after, len(real))}", f"{100 * (after - before) / len(real):+.2f} pt (+{new}, −{cleared})"]
        else:
            cells += ["–", "–"]
        cells.append(f"{share(flagged(pairs[1], 0), len(generated))} → {share(flagged(pairs[1], 1), len(generated))}" if generated else "–")
        for fpr in (0.01, 0.001):
            if real and generated:
                cut = cutoff_at(real, fpr)
                cells.append(f"{share(sum(s > cut for s in generated), len(generated))}, {sum(s > cut for s in real)} FP")
            else:
                cells.append("–")
        cells.append(f"{auroc(real, generated):.3f}" if real and generated else "–")
        if real:
            cut = cutoff_at([o for o, _ in pairs[0]], 0.01)
            cells.append(f"{share(sum(s > cut for s in real), len(real))}, {share(sum(s > cut for s in generated), len(generated))}")
        else:
            cells.append("–")
        if args.versus:
            both = {label: [(copies[p][1], copies[p][2]) for p in parents if copies[p][0] == label] for label in (0, 1)}
            if both[0] and both[1]:
                point, low, high = paired_difference(both[0], both[1], 0.01, args.draws, args.seed)
                cells.append(f"{point:+.1f} pt ({low:+.1f} to {high:+.1f})")
            else:
                cells.append("–")
        print("| " + " | ".join(cells) + " |")


def self_test():
    real = [(i / 200, i / 200) for i in range(200)]
    # The first score separates every generated item; the second puts half of them under the authentic ones.
    generated = [(2.0 + i, 2.0 + i if i % 2 else -1.0) for i in range(100)]
    assert recall_at([a for a, _ in real], [a for a, _ in generated], 0.01) == 100
    assert recall_at([b for _, b in real], [b for _, b in generated], 0.01) == 50
    point, low, high = paired_difference(real, generated, 0.01, 400, 1)
    assert point == 50.0 and 35.0 < low <= point <= high < 65.0, (point, low, high)
    assert paired_difference(real, generated, 0.01, 400, 1) == (point, low, high), "the same seed must give the same interval"
    same = [(a, a) for a, _ in generated]
    assert paired_difference(real, same, 0.01, 400, 1) == (0.0, 0.0, 0.0)
    # One authentic item above everything: at 1% of 200 two may pass, so nothing is lost; at 0.1% none may.
    spiked = real[:-1] + [(1000.0, 1000.0)]
    assert recall_at([a for a, _ in spiked], [a for a, _ in generated], 0.01) == 100
    assert recall_at([a for a, _ in spiked], [a for a, _ in generated], 0.001) == 0
    print("report-stress self-test: ok")


if __name__ == "__main__":
    main()
