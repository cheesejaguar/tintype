#!/usr/bin/env python3
"""Small development sets and the ranking table for an interpolation sweep.

Two steps of docs/runbooks/interpolation-pooling.md, standard library only
(run with `python3 -I`):

  manifests  Cut three small manifests (at most --limit items each) out of a
             validation-split manifest written by `tintype finalize`
             (`data/splits/val_images.tsv`: the 3% of every training store
             that training never saw, by path hash): legacy GANs, current
             generators, and reposted images. Files that are not on disk are
             left out. These are development data: the released model was
             calibrated on them.

  report     From all-views score files of several checkpoints on the same
             manifest (`tintype score --all-views`), print recall at 1% and
             0.1% empirical false-positive rate and AUROC of the mean view
             logit, one row per checkpoint. The cutoffs come from the very
             items reported, and the checkpoints are not calibrated: a
             ranking diagnostic, nothing more.

The cutoff rule is that of `metrics::empirical_cutoff` in the Rust source:
flag strictly above the (floor(fpr * reals) + 1)-th highest real score.
"""
import argparse
import math
import os
import sys

# Generator names (lower case) that are GANs, as the training stores spell them.
GANS = {
    "biggan", "gigagan", "stylegan2", "stylegan3", "styleganxl", "stylesanxl", "progan", "projectedgan",
    "styleswin", "gansformer", "genimage-biggan", "r3gan", "stgan", "mmdgan",
}
SETS = {
    # name: (stores whose AI items count, an optional filter on the generator, stores whose real items count)
    "legacy-gan": (("commfor-small", "commfor-small2", "tpdne", "sofake-set", "stylegan3", "tiny-genimage"), lambda g: g.lower() in GANS, ("commfor-small", "commfor-small2", "celeba-hq")),
    "current": (("frontier", "frontier-heldout", "nano-banana", "openfake", "openfake2"), None, ("openfake", "openfake2", "openfake-test-reals")),
    "reposted": (("openfake-reddit", "sofake-set"), None, ("openfake-reddit", "sofake-set")),
}


def evenly(rows, n):
    """`n` of `rows`, evenly spaced, in order."""
    if len(rows) <= n:
        return rows
    return [rows[(i * len(rows)) // n] for i in range(n)]


def manifests(args):
    rows = []
    with open(args.val, encoding="utf-8") as f:
        for line in f:
            c = line.rstrip("\n").split("\t")
            if len(c) < 4 or "/data/store/" not in c[0]:
                continue
            # The split was written with absolute paths on the machine that trained; keep what follows data/.
            rel = c[0].split("/data/", 1)[1]
            path = os.path.abspath(os.path.join(args.data, rel))
            if os.path.isfile(path):
                rows.append((path, c[1], c[2], c[3], rel.split("/")[1]))
    os.makedirs(args.out_dir, exist_ok=True)
    for name, (ai_stores, keep, real_stores) in SETS.items():
        ai = [r for r in rows if r[1] == "1" and r[4] in ai_stores and (keep is None or keep(r[2]))]
        real = [r for r in rows if r[1] == "0" and r[4] in real_stores]
        ai = evenly(ai, args.limit // 2) if len(ai) > args.limit // 2 and len(real) >= args.limit // 2 else ai[: args.limit]
        real = evenly(real, max(args.limit - len(ai), 0))
        out = os.path.join(args.out_dir, name + ".tsv")
        with open(out, "w", encoding="utf-8") as f:
            for path, label, generator, dataset, _ in ai + real:
                f.write(f"{path}\t{label}\t{generator}\t{dataset}\n")
        print(f"{out}: {len(ai)} AI, {len(real)} real")


def auroc(real, fake):
    """Rank-sum AUROC with mid-ranks for ties."""
    scored = sorted([(s, 0) for s in real] + [(s, 1) for s in fake])
    rank_sum, i = 0.0, 0
    while i < len(scored):
        j = i
        while j + 1 < len(scored) and scored[j + 1][0] == scored[i][0]:
            j += 1
        rank_sum += ((i + j) / 2 + 1) * sum(label for _, label in scored[i : j + 1])
        i = j + 1
    return (rank_sum - len(fake) * (len(fake) + 1) / 2) / (len(real) * len(fake))


def recall_at(real, fake, fpr):
    ordered = sorted(real, reverse=True)
    cutoff = ordered[min(math.floor(fpr * len(ordered)), len(ordered) - 1)]
    return sum(s > cutoff for s in fake) / len(fake), sum(s > cutoff for s in real)


def report(args):
    print("| Checkpoint | AI | Real | AUROC | Recall at 1% FPR (real flagged) | Recall at 0.1% FPR (real flagged) |")
    print("|---|---|---|---|---|---|")
    for spec in args.scores:
        name, path = spec.split("=", 1)
        real, fake = [], []
        with open(path, encoding="utf-8") as f:
            for line in f:
                c = line.rstrip("\n").split("\t")
                logits = [float(x) for x in c[5:]]
                if not logits:
                    sys.exit(f"{path}: a row holds no view logits; score with --all-views")
                (fake if c[1] == "1" else real).append(sum(logits) / len(logits))
        if not real or not fake:
            sys.exit(f"{path}: needs items of both classes")
        (r1, f1), (r01, f01) = recall_at(real, fake, 0.01), recall_at(real, fake, 0.001)
        print(f"| {name} | {len(fake)} | {len(real)} | {auroc(real, fake):.4f} | {100 * r1:.2f}% ({f1}) | {100 * r01:.2f}% ({f01}) |")


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    m = sub.add_parser("manifests")
    m.add_argument("--val", required=True, help="validation-split manifest written by `tintype finalize`")
    m.add_argument("--data", default="data", help="local directory that holds store/")
    m.add_argument("--out-dir", required=True)
    m.add_argument("--limit", type=int, default=2000, help="most items per manifest")
    m.set_defaults(run=manifests)
    r = sub.add_parser("report")
    r.add_argument("scores", nargs="+", help="name=scores.tsv, all-views score files of one manifest")
    r.set_defaults(run=report)
    args = p.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
