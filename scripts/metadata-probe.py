#!/usr/bin/env python3
"""Predict the label from file metadata alone, with no pixels: a shortcut
probe for the stores a detector trains on and the sets it is scored on.

Input: feature files written by `scripts/audit-manifests.py features`
(container format, JPEG quality and tables, chroma subsampling, dimensions,
bytes per pixel, EXIF, ICC and text-chunk flags). If a classifier that sees
only these columns separates real from AI, then a detector trained on the
same files can separate them the same way, by learning the file history
instead of the generator; and a benchmark where that works rewards it.

The classifier is a logistic regression over one-hot bins of each column
(L2-regularised, classes weighted equally, fitted by Newton steps on the
distinct rows). It is evaluated three ways, and the difference between them
is the point:

  item folds    K folds over the files of a set. Says how well metadata
                separates the classes inside the set.
  group folds   K folds over groups: real sources and AI generators are
                held out whole, so a rule has to hold for a source or a
                generator it has not seen. Needs two groups per class.
  other stores  each store is scored by a model fitted on every other
                store (leave one store out). Says whether the rule
                transfers between collections.

A fold's files are scored by one model, so the AUROC of a fold scheme
counts only pairs of files from the same fold (folds weighted by their
number of pairs); scores of different models are never ranked against each
other, except in the pooled "other stores" row, which says so. AUROC is
reported with a 95% interval from a bootstrap over clusters (a real source,
an AI generator), not over files: files of one source share their metadata
and are not independent. Where a class has a single cluster
its files are resampled instead and the interval is marked `†`; it is then
too narrow. Single-column AUROCs (per-bin class odds, same folds) show
which columns carry the separation.

An AUROC near 0.5 means the columns say nothing; near 1 means the label can
be read off the file header. This measures the shortcut that is available,
not whether the detector uses it.

Standard library only; runs under `python3 -I`.

  python3 -I scripts/metadata-probe.py runs/ws4/features-train.tsv
  python3 -I scripts/metadata-probe.py runs/ws4/features-eval.tsv --by dataset --fit-on runs/ws4/features-train.tsv
  python3 -I scripts/metadata-probe.py --self-test
"""

import argparse
import csv
import math
import random
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402


def cut(value, edges, labels=None):
    """The bin of `value` among ascending `edges`: `<e0`, `e0-e1`, ..., `>=eN`."""
    for k, edge in enumerate(edges):
        if value < edge:
            return f"<{edge}" if k == 0 else f"{edges[k - 1]}-{edge}"
    return f">={edges[-1]}"


def side_bin(v):
    # The sizes generators favour get bins of their own.
    return str(v) if v in (256, 512, 768, 1024, 2048) else cut(v, [256, 384, 512, 640, 768, 1024, 1280, 1536, 2048, 3072, 4096])


def aspect_bin(w, h):
    r = w / h
    for name, target in (("1:1", 1.0), ("4:3", 4 / 3), ("3:2", 1.5), ("16:9", 16 / 9), ("3:4", 0.75), ("2:3", 2 / 3), ("9:16", 9 / 16)):
        if abs(r - target) < 0.02 * target:
            return name
    return "wide" if r > 1 else "tall"


def bins_of(row):
    """The bin of every column for one feature row. A file that could not be
    read has `missing` in every header column."""
    w, h, size = row.get("width"), row.get("height"), row.get("bytes")
    fmt = row.get("format") or "missing"
    jpeg = fmt == "jpeg"
    out = {
        "format": fmt,
        "jpeg_quality": cut(int(row["jpeg_q"]), [50, 60, 70, 75, 80, 85, 90, 93, 96, 98]) if jpeg and row.get("jpeg_q") else "none",
        "jpeg_tables": ("ijg" if row.get("jpeg_std") == "1" else "custom") if jpeg and row.get("jpeg_q") else "none",
        "subsampling": row.get("subsampling") or "none",
        "progressive": row.get("progressive") if jpeg else "none",
        "exif": row.get("exif") or "0",
        "icc": row.get("icc") or "0",
        "jfif": row.get("jfif") if jpeg else "none",
        "xmp_or_adobe": str(int(row.get("xmp") == "1" or row.get("adobe") == "1")),
        "c2pa": row.get("c2pa") or "0",
        "png_text": row.get("png_text") if fmt == "png" else "none",
    }
    if w and h:
        w, h = int(w), int(h)
        out["short_side"] = side_bin(min(w, h))
        out["long_side"] = side_bin(max(w, h))
        out["aspect"] = aspect_bin(w, h)
        out["multiple"] = "64" if w % 64 == 0 and h % 64 == 0 else "16" if w % 16 == 0 and h % 16 == 0 else "8" if w % 8 == 0 and h % 8 == 0 else "other"
        out["megapixels"] = cut(w * h / 1e6, [0.1, 0.25, 0.5, 1.0, 1.1, 2.0, 4.0, 8.0, 16.0])
        out["exact_size"] = f"{w}x{h}"
        out["bytes_per_pixel"] = cut(int(size) / (w * h), [0.05, 0.1, 0.2, 0.4, 0.8, 1.6, 3.2]) if size else "missing"
    else:
        for name in ("short_side", "long_side", "aspect", "multiple", "megapixels", "exact_size", "bytes_per_pixel"):
            out[name] = "missing"
    out["extension"] = row.get("ext") or "none"
    return out


FEATURE_SETS = {
    "header": ["format", "jpeg_quality", "jpeg_tables", "subsampling", "progressive", "exif", "icc", "jfif", "xmp_or_adobe", "c2pa", "png_text", "short_side", "long_side", "aspect", "multiple", "megapixels", "exact_size", "bytes_per_pixel"],
    # The header columns that do not depend on the picture's size: how the file was encoded.
    "encoding": ["format", "jpeg_quality", "jpeg_tables", "subsampling", "progressive", "exif", "icc", "jfif", "xmp_or_adobe", "c2pa", "png_text", "bytes_per_pixel"],
    "extension": ["extension"],
}


class Example:
    __slots__ = ("label", "store", "cluster", "key", "bins", "h")

    def __init__(self, label, store, cluster, key, bins, h):
        self.label, self.store, self.cluster, self.key, self.bins, self.h = label, store, cluster, key, bins, h


def load(paths, by, features, seed, keep_missing=False):
    """Examples of the feature files, and how many rows were left out because
    their file is not on disk (a header probe has nothing to read there)."""
    examples, missing = [], 0
    for path in paths:
        with open(path, encoding="utf-8", newline="") as stream:
            for row in csv.DictReader(stream, delimiter="\t", quoting=csv.QUOTE_NONE):
                if row.get("error") == "FileNotFoundError" and not keep_missing:
                    missing += 1
                    continue
                label = int(row["label"])
                bins = bins_of(row)
                # A cluster is a real source or an AI generator: files of one share their metadata.
                cluster = (label, row["source"] if label == 0 else row["generator"])
                key = f"{row['store']}/{row['path']}"
                examples.append(Example(label, row[by], cluster, key, tuple(bins[f] for f in features), mg.stable_hash(seed, key)))
    return examples, missing


def sigmoid(z):
    if z >= 0:
        return 1.0 / (1.0 + math.exp(-z))
    e = math.exp(z)
    return e / (1.0 + e)


def cholesky_solve(a, b):
    """Solve `a x = b` for a symmetric positive definite `a` (lists of lists)."""
    n = len(b)
    low = [[0.0] * n for _ in range(n)]
    for i in range(n):
        row_i = low[i]
        for j in range(i + 1):
            row_j = low[j]
            s = a[i][j] - sum(row_i[k] * row_j[k] for k in range(j))
            if i == j:
                row_i[j] = math.sqrt(max(s, 1e-12))
            else:
                row_i[j] = s / row_j[j]
    y = [0.0] * n
    for i in range(n):
        y[i] = (b[i] - sum(low[i][k] * y[k] for k in range(i))) / low[i][i]
    x = [0.0] * n
    for i in reversed(range(n)):
        x[i] = (y[i] - sum(low[k][i] * x[k] for k in range(i + 1, n))) / low[i][i]
    return x


class Model:
    """Logistic regression over one-hot bins. Bins seen fewer than
    `min_count` times in training, and bins never seen, share an `other`
    column per feature."""

    def __init__(self, features, min_count=20, ridge=1e-3, max_train=30000):
        self.features, self.min_count, self.ridge, self.max_train = features, min_count, ridge, max_train
        self.columns, self.theta = [], []

    def encode(self, bins):
        return tuple(vocab.get(b, vocab[None]) for vocab, b in zip(self.columns, bins))

    def fit(self, examples):
        if len(examples) > self.max_train:
            # A fixed subsample: the model has a few hundred weights, and the cost is in the distinct rows.
            examples = sorted(examples, key=lambda e: e.h)[: self.max_train]
        self.columns, width = [], 1
        for k in range(len(self.features)):
            count = Counter(e.bins[k] for e in examples)
            vocab = {None: width}
            width += 1
            for b in sorted(b for b, n in count.items() if n >= self.min_count):
                vocab[b] = width
                width += 1
            self.columns.append(vocab)
        n_class = [max(1, sum(e.label == c for e in examples)) for c in (0, 1)]
        rows = defaultdict(lambda: [0.0, 0.0])
        for e in examples:
            rows[self.encode(e.bins)][e.label] += 0.5 / n_class[e.label]
        rows = [((0,) + idx, w[1], w[0] + w[1]) for idx, w in rows.items()]
        theta = [0.0] * width

        def loss(t):
            total = 0.5 * self.ridge * sum(v * v for v in t[1:])
            for idx, pos, w in rows:
                z = sum(t[i] for i in idx)
                total += w * (max(z, 0.0) + math.log1p(math.exp(-abs(z)))) - pos * z
            return total

        current = loss(theta)
        for _ in range(25):
            grad = [0.0] + [self.ridge * v for v in theta[1:]]
            hess = [[0.0] * width for _ in range(width)]
            for idx, pos, w in rows:
                p = sigmoid(sum(theta[i] for i in idx))
                g, c = w * p - pos, w * p * (1.0 - p)
                for a, i in enumerate(idx):
                    grad[i] += g
                    hi = hess[i]
                    for j in idx[: a + 1]:
                        hi[j] += c
            for i in range(width):
                hess[i][i] += self.ridge if i else 1e-9
                for j in range(i):
                    # idx is ascending, so only hess[i][j] with j <= i was filled: mirror it.
                    hess[j][i] = hess[i][j]
            step = cholesky_solve(hess, grad)
            scale = 1.0
            while True:
                trial = [t - scale * s for t, s in zip(theta, step)]
                new = loss(trial)
                if new <= current + 1e-12 or scale < 1e-4:
                    break
                scale *= 0.5
            moved = max(abs(scale * s) for s in step)
            theta, current = trial, new
            if moved < 1e-5:
                break
        self.theta = theta
        return self

    def score(self, bins):
        return self.theta[0] + sum(self.theta[i] for i in self.encode(bins))

    def largest_weights(self, n):
        named = []
        for feature, vocab in zip(self.features, self.columns):
            for b, i in vocab.items():
                named.append((self.theta[i], feature, "(other)" if b is None else b))
        named.sort(key=lambda t: -abs(t[0]))
        return named[:n]


class Odds:
    """One column: the smoothed class log-odds of each of its bins."""

    def __init__(self, k):
        self.k, self.table, self.default = k, {}, 0.0

    def fit(self, examples):
        count = defaultdict(lambda: [0, 0])
        for e in examples:
            count[e.bins[self.k]][e.label] += 1
        n = [max(1, sum(c[0] for c in count.values())), max(1, sum(c[1] for c in count.values()))]
        self.table = {b: math.log(((c[1] + 0.5) / n[1]) / ((c[0] + 0.5) / n[0])) for b, c in count.items()}
        return self

    def score(self, bins):
        return self.table.get(bins[self.k], self.default)


def auroc_from_counts(pos, neg):
    """AUROC from `{(fold, score): weight}` of each class, ties counted half.
    Only pairs within one fold are compared: scores of different folds come
    from different models and are not on one scale. The folds are weighted
    by their number of pairs."""
    wins, pairs = 0.0, 0.0
    fold, below, p_fold, n_fold = None, 0.0, 0.0, 0.0
    for key in sorted(set(pos) | set(neg)):
        if key[0] != fold:
            pairs += p_fold * n_fold
            fold, below, p_fold, n_fold = key[0], 0.0, 0.0, 0.0
        here, mine = neg.get(key, 0.0), pos.get(key, 0.0)
        wins += mine * (below + 0.5 * here)
        below += here
        p_fold += mine
        n_fold += here
    pairs += p_fold * n_fold
    return wins / pairs if pairs > 0 else None


def auroc(examples, scores, replicates=0, rng=None, folds=None):
    """`(auroc, lo, hi, item_level)`: the AUROC of scored examples and a 95%
    percentile interval from resampling clusters within each class. With
    `folds` (the fold that scored each example) only pairs of one fold are
    compared. `item_level` is true when a class had one cluster and its
    files were resampled instead."""
    per_cluster = [defaultdict(Counter), defaultdict(Counter)]
    for k, (e, s) in enumerate(zip(examples, scores)):
        per_cluster[e.label][e.cluster][(folds[k] if folds else 0, round(s, 6))] += 1
    whole = [Counter(), Counter()]
    for label in (0, 1):
        for c in per_cluster[label].values():
            whole[label].update(c)
    point = auroc_from_counts(whole[1], whole[0])
    if point is None or not replicates:
        return point, None, None, False
    item_level = False
    units = []
    for label in (0, 1):
        clusters = list(per_cluster[label].values())
        if len(clusters) == 1:
            item_level = True
            values, weights = zip(*sorted(clusters[0].items()))
            units.append(("items", values, weights, sum(weights)))
        else:
            units.append(("clusters", clusters, None, len(clusters)))
    draws = []
    for _ in range(replicates):
        sample = []
        for kind, a, b, n in units:
            if kind == "items":
                sample.append(Counter(rng.choices(a, weights=b, k=n)))
            else:
                total = Counter()
                for c in rng.choices(a, k=n):
                    total.update(c)
                sample.append(total)
        value = auroc_from_counts(sample[1], sample[0])
        if value is not None:
            draws.append(value)
    draws.sort()
    lo = draws[int(0.025 * len(draws))] if draws else None
    hi = draws[min(len(draws) - 1, int(0.975 * len(draws)))] if draws else None
    return point, lo, hi, item_level


def fold_scores(examples, fold_of, folds, make):
    """Out-of-fold scores: each fold scored by a model fitted on the others.
    `None` for an example whose training folds lacked a class."""
    scores = [None] * len(examples)
    for k in range(folds):
        train = [e for e, f in zip(examples, fold_of) if f != k]
        if not train or len({e.label for e in train}) < 2:
            continue
        model = make().fit(train)
        cache = {}
        for i, (e, f) in enumerate(zip(examples, fold_of)):
            if f == k:
                s = cache.get(e.bins)
                if s is None:
                    s = cache[e.bins] = model.score(e.bins)
                scores[i] = s
    return scores


def item_folds(examples, folds):
    return [e.h % folds for e in examples]


def group_folds(examples, folds, seed):
    """Clusters dealt round-robin to folds, each class separately, so that
    every fold holds clusters of both classes. `(fold of each example, K)`,
    or `(None, 0)` when a class has fewer than two clusters."""
    by_class = [sorted({e.cluster for e in examples if e.label == c}, key=lambda c: mg.stable_hash(seed, repr(c))) for c in (0, 1)]
    k = min(folds, len(by_class[0]), len(by_class[1]))
    if k < 2:
        return None, 0
    fold = {}
    for clusters in by_class:
        for i, c in enumerate(clusters):
            fold[c] = i % k
    return [fold[e.cluster] for e in examples], k


def kept(examples, scores, folds=None):
    """The scored examples, their scores and (with `folds`) their folds."""
    keep = [k for k, s in enumerate(scores) if s is not None]
    return [examples[k] for k in keep], [scores[k] for k in keep], ([folds[k] for k in keep] if folds else None)


def cv_auroc(examples, fold_of, folds, make, replicates=0, rng=None):
    """Out-of-fold AUROC over within-fold pairs, with its cluster interval."""
    e, s, f = kept(examples, fold_scores(examples, fold_of, folds, make), fold_of)
    return auroc(e, s, replicates, rng, f)


def fmt_auroc(result):
    point, lo, hi, item_level = result
    if point is None:
        return "–"
    return f"{point:.3f}" if lo is None else f"{point:.3f} [{lo:.3f}, {hi:.3f}]{'†' if item_level else ''}"


def probe(examples, features, args, fit_on=None, out=sys.stdout):
    rng = random.Random(args.seed)
    make = lambda: Model(features, args.min_count, args.ridge, args.max_train)  # noqa: E731
    stores = sorted({e.store for e in examples})
    by_store = {s: [e for e in examples if e.store == s] for s in stores}
    two_class = [s for s in stores if len({e.label for e in by_store[s]}) == 2]

    # Leave one store out: every store scored by a model fitted on the others.
    loso = {}
    if len(stores) > 1:
        store_index = {s: i for i, s in enumerate(stores)}
        scores = fold_scores(examples, [store_index[e.store] for e in examples], len(stores), make)
        loso = dict(zip((id(e) for e in examples), scores))
    transfer = make().fit(fit_on) if fit_on else None

    print(f"| {args.by.capitalize()} | Real | AI | Real sources | AI generators | AUROC, item folds | AUROC, group folds | AUROC, fitted on the other {args.by}s |" + (" AUROC, fitted on --fit-on |" if transfer else "") + " Strongest single columns (item folds) |", file=out)
    print("|---|---|---|---|---|---|---|---|---|" + ("---|" if transfer else ""), file=out)
    for s in stores:
        rows = by_store[s]
        real, ai = sum(e.label == 0 for e in rows), sum(e.label == 1 for e in rows)
        clusters = [len({e.cluster for e in rows if e.label == c}) for c in (0, 1)]
        cells = ["–", "–", "–", "–", ""]
        if s in two_class:
            folds = item_folds(rows, args.folds)
            cells[0] = fmt_auroc(cv_auroc(rows, folds, args.folds, make, args.bootstrap, rng))
            gf, k = group_folds(rows, args.folds, args.seed)
            if gf:
                cells[1] = fmt_auroc(cv_auroc(rows, gf, k, make, args.bootstrap, rng))
            if loso:
                cells[2] = fmt_auroc(auroc(*kept(rows, [loso[id(e)] for e in rows])[:2], args.bootstrap, rng))
            if transfer:
                cells[3] = fmt_auroc(auroc(rows, [transfer.score(e.bins) for e in rows], args.bootstrap, rng))
            single = []
            for k_feature, name in enumerate(features):
                a = cv_auroc(rows, folds, args.folds, lambda: Odds(k_feature))[0]
                if a is not None:
                    single.append((a, name))
            single.sort(reverse=True)
            cells[4] = ", ".join(f"{name} {a:.3f}" for a, name in single[:3])
        print(f"| {s} | {real} | {ai} | {clusters[0]} | {clusters[1]} | {cells[0]} | {cells[1]} | {cells[2]} |" + (f" {cells[3]} |" if transfer else "") + f" {cells[4]} |", file=out)

    if len({e.label for e in examples}) < 2:
        return
    real, ai = sum(e.label == 0 for e in examples), sum(e.label == 1 for e in examples)
    clusters = [len({e.cluster for e in examples if e.label == c}) for c in (0, 1)]
    print(f"\nPooled over {len(stores)} {args.by}s: {real} real files from {clusters[0]} sources, {ai} AI files from {clusters[1]} generators.\n", file=out)
    schemes = [("item folds", item_folds(examples, args.folds), args.folds)]
    gf, k = group_folds(examples, args.folds, args.seed)
    if gf:
        schemes.append(("group folds", gf, k))
    print("| Pooled | AUROC [95% interval over clusters] |\n|---|---|", file=out)
    for name, folds, k in schemes:
        print(f"| {name} | {fmt_auroc(cv_auroc(examples, folds, k, make, args.bootstrap, rng))} |", file=out)
    if loso:
        # Single-class stores can only be compared with files of other stores, so here the
        # scores of the per-store models are pooled as they are.
        print(f"| fitted on the other {args.by}s, scores pooled | {fmt_auroc(auroc(*kept(examples, [loso[id(e)] for e in examples])[:2], args.bootstrap, rng))} |", file=out)
    if transfer:
        print(f"| fitted on --fit-on | {fmt_auroc(auroc(examples, [transfer.score(e.bins) for e in examples], args.bootstrap, rng))} |", file=out)

    print("\n| Column alone | " + " | ".join(f"AUROC, {name}" for name, _, _ in schemes) + " |\n|---|" + "---|" * len(schemes), file=out)
    table = []
    for k_feature, name in enumerate(features):
        row = [cv_auroc(examples, folds, k, lambda: Odds(k_feature))[0] for _, folds, k in schemes]
        table.append((row, name))
    table.sort(key=lambda t: -(t[0][0] or 0))
    for row, name in table:
        print(f"| {name} | " + " | ".join("–" if a is None else f"{a:.3f}" for a in row) + " |", file=out)

    full = make().fit(examples)
    print("\nLargest weights of the model fitted on all files (positive means AI):\n\n| Column | Bin | Weight |\n|---|---|---|", file=out)
    for weight, feature, b in full.largest_weights(args.weights):
        print(f"| {feature} | {b} | {weight:+.2f} |", file=out)


def self_test():
    # The solver and the AUROC.
    x = cholesky_solve([[4.0, 2.0], [2.0, 3.0]], [10.0, 8.0])
    assert abs(x[0] - 1.75) < 1e-9 and abs(x[1] - 1.5) < 1e-9, x
    assert auroc_from_counts({(0, 1.0): 2, (0, 0.5): 1}, {(0, 0.0): 2, (0, 0.5): 1}) == (2 * 3 + 1 * 2.5) / 9
    assert auroc_from_counts({(0, 1.0): 1}, {}) is None
    # Two folds whose scores sit on different scales: each separates its own files perfectly,
    # and pooled as one list they would not.
    assert auroc_from_counts({(0, 0.2): 1, (1, 5.0): 1}, {(0, 0.1): 1, (1, 4.0): 1}) == 1.0
    assert cut(75, [50, 60, 70, 75, 80]) == "75-80" and cut(10, [50, 60]) == "<50" and cut(99, [50, 60]) == ">=60"
    assert aspect_bin(1024, 1024) == "1:1" and aspect_bin(4032, 3024) == "4:3" and aspect_bin(1000, 300) == "wide"

    rng = random.Random(4)
    features = ["format", "exif", "noise"]

    def make_examples(n, sources, generators, p_png_ai, p_png_real, exif_sources=()):
        out = []
        for i in range(n):
            label = i % 2
            if label:
                cluster = (1, f"gen{rng.randrange(generators)}")
                fmt = "png" if rng.random() < p_png_ai else "jpeg"
                exif = "0"
            else:
                source = rng.randrange(sources)
                cluster = (0, f"src{source}")
                fmt = "png" if rng.random() < p_png_real else "jpeg"
                exif = "1" if source in exif_sources else "0"
            key = f"s/{i}"
            out.append(Example(label, "s", cluster, key, (fmt, exif, str(rng.randrange(4))), mg.stable_hash(1, key)))
        return out

    make = lambda: Model(features, min_count=5)  # noqa: E731
    # Format follows the class in nine files of ten: about 0.9 AUROC, from the format column.
    confounded = make_examples(4000, 6, 40, 0.9, 0.1)
    result = cv_auroc(confounded, item_folds(confounded, 5), 5, make, 100, random.Random(1))
    assert 0.86 < result[0] < 0.94 and result[1] < result[0] < result[2] and not result[3], result
    single = [cv_auroc(confounded, item_folds(confounded, 5), 5, lambda: Odds(k))[0] for k in range(3)]
    assert single[0] > 0.86 and abs(single[2] - 0.5) < 0.05, single
    top = make().fit(confounded).largest_weights(2)
    assert {t[1] for t in top} == {"format"} and all((t[0] > 0) == (t[2] == "png") for t in top), top
    # Metadata that does not follow the class: chance.
    matched = make_examples(4000, 6, 40, 0.5, 0.5)
    result = cv_auroc(matched, item_folds(matched, 5), 5, make)
    assert abs(result[0] - 0.5) < 0.05, result
    # EXIF in half the real sources and never in AI files: item folds see it, and so do group
    # folds, because the sources that lack it look like AI either way; the cluster interval is
    # wider than a file interval would be because a source is all or nothing.
    exif = make_examples(4000, 6, 40, 0.5, 0.5, exif_sources=(0, 1, 2))
    item = cv_auroc(exif, item_folds(exif, 5), 5, make, 200, random.Random(2))
    assert 0.70 < item[0] < 0.80, item
    gf, k = group_folds(exif, 5, 1)
    assert k == 5 and len({(f, e.cluster) for f, e in zip(gf, exif)}) == 46
    assert all(len({e.label for f, e in zip(gf, exif) if f == fold}) == 2 for fold in range(5))
    grouped = cv_auroc(exif, gf, k, make, 200, random.Random(2))
    assert 0.6 < grouped[0] < 0.85 and grouped[2] - grouped[1] > 0.05, grouped
    # One cluster per class: no group folds, and the interval is marked as file-level.
    lone = make_examples(600, 1, 1, 0.9, 0.1)
    assert group_folds(lone, 5, 1) == (None, 0)
    assert cv_auroc(lone, item_folds(lone, 5), 5, make, 50, random.Random(3))[3]
    # A column that says nothing gives exactly 0.5, however the folds' constants differ.
    flat = [Example(i % 2, "s", (i % 2, f"c{i % 7}"), f"s/{i}", ("same",), mg.stable_hash(1, f"s/{i}")) for i in range(700)]
    assert cv_auroc(flat, item_folds(flat, 5), 5, lambda: Odds(0))[0] == 0.5
    # A bin unseen in training falls in the feature's `other` column instead of failing.
    model = make().fit(confounded)
    assert math.isfinite(model.score(("webp", "0", "9")))
    # Bins of a feature row, and a row for a file that is not on disk.
    row = {"format": "jpeg", "width": "1024", "height": "768", "bytes": "200000", "jpeg_q": "92", "jpeg_std": "1", "subsampling": "420", "progressive": "0", "exif": "1", "icc": "0", "jfif": "1", "ext": "jpg"}
    b = bins_of(row)
    assert (b["jpeg_quality"], b["jpeg_tables"], b["short_side"], b["long_side"], b["aspect"], b["multiple"], b["exact_size"], b["bytes_per_pixel"], b["png_text"]) == ("90-93", "ijg", "768", "1024", "4:3", "64", "1024x768", "0.2-0.4", "none"), b
    gone = bins_of({"format": "", "ext": "png", "error": "FileNotFoundError"})
    assert gone["format"] == "missing" and gone["short_side"] == "missing" and gone["extension"] == "png"
    print("metadata-probe self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("features", nargs="*", help="feature files from `audit-manifests.py features`")
    p.add_argument("--by", default="store", help="the column that names a set: store (default) or dataset")
    p.add_argument("--fit-on", nargs="+", help="also score every set with a model fitted on these feature files")
    p.add_argument("--columns", default="header", choices=sorted(FEATURE_SETS), help="header: everything read from the file; encoding: the header without the size columns; extension: only the manifest's file extension (for sets no longer on disk)")
    p.add_argument("--folds", type=int, default=5)
    p.add_argument("--bootstrap", type=int, default=200, help="bootstrap replicates for the intervals (0 = none)")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--min-count", type=int, default=20, help="a bin seen less often in training joins its column's `other` bin")
    p.add_argument("--ridge", type=float, default=1e-3)
    p.add_argument("--max-train", type=int, default=30000, help="fit each model on at most this many files, chosen by a hash of the path")
    p.add_argument("--weights", type=int, default=15, help="how many of the largest weights to list")
    p.add_argument("--self-test", action="store_true")
    return p


def main():
    args = parser().parse_args()
    if args.self_test:
        return self_test()
    if not args.features:
        raise SystemExit("pass at least one feature file, or --self-test")
    features = FEATURE_SETS[args.columns]
    by_extension = args.columns == "extension"
    examples, missing = load(args.features, args.by, features, args.seed, keep_missing=by_extension)
    fit_on = load(args.fit_on, args.by, features, args.seed, keep_missing=by_extension)[0] if args.fit_on else None
    print(f"Metadata-only label probe over {len(examples)} files; columns: {', '.join(features)}.")
    if missing:
        print(f"{missing} rows are left out: their files are not on disk.")
    print(f"{args.folds} folds, seed {args.seed}, {args.bootstrap} bootstrap replicates over clusters, at most {args.max_train} files per fit.\n")
    probe(examples, features, args, fit_on)


if __name__ == "__main__":
    main()
