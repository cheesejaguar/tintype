#!/usr/bin/env python3
"""Compare photo decision policies at equal false-positive counts.

A policy turns the scores of one or two detectors into one score per photo:

- a single-model profile at a re-chosen cutoff (Tintype default, sensitive,
  strict);
- the Community Forensics route of `--photo-second-opinion`: a photo whose
  calibrated first-stage logit z1 lies strictly inside a band (lo, hi) gets
  `p = sigmoid(0.5 * (weight * z1 + z2 - threshold))`, the others keep the
  first-stage score (src/detector.rs, `PhotoSecondOpinion`);
- a second model given as a score column or file (GPTTrace, a linear probe
  on a frozen encoder), blended 50:50 in logits for `abs(z1) <= m` or for
  every photo;
- logit fusion with fitted weights: plain logistic regression on the two
  logits. The weights are never fitted on the photos they are reported on:
  give `--set fit=...` and `--set test=...`, or `--cross-fit K` to fit on
  K-1 folds of whole groups (parents) and score the fold left out.

For each policy it prints the share of photos sent to the second model,
recall at each empirical false-positive rate (0.1% and 1% by default) and at
each false-positive count, the counts at the policy's own cutoff, the
number of sets on which precision and recall both reach a reference
detector, and the first-stage errors a gate leaves outside. With
`--set calibrate=...` it also freezes each cutoff on that set and prints
the cutoff (the logit of the policy's score) and the actual rates on the
test set.

Everything computed from the archives in docs/results is exploratory: those
suites were used to develop the detector, and a cutoff chosen on a cohort
and reported on it is not a transfer result. The route counts are measured
workload; end-to-end savings depend on the cohort and the hardware. Nothing
here selects what ships: see docs/runbooks/policy-comparison.md.

Standard library only. The archive loaders and the fixed-budget counting
come from scripts/replay-research.py, loaded by file path because of the
hyphen in its name; `--check-replay` shows that the two scripts print the
same numbers.

    python3 scripts/compare-policies.py --self-test
    python3 scripts/compare-policies.py --check-replay
    python3 scripts/compare-policies.py --archive confirm7
    python3 scripts/compare-policies.py --archive arena --cross-fit 5
"""

import argparse
import csv
import glob
import gzip
import hashlib
import importlib.util
import io
import json
import math
import os
import re
import sys
import tempfile
from collections import Counter
from contextlib import redirect_stdout
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def _sibling(filename):
    spec = importlib.util.spec_from_file_location(filename.replace("-", "_").replace(".py", ""), Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


replay = _sibling("replay-research.py")

IDENTITY = ("path", "label", "generator", "dataset")
# Manifest columns 5 and up (src/ingest.rs, GROUP_COLUMNS), under the names `tintype report --slice-by` uses.
GROUP_COLUMNS = ("parent", "scene", "source", "family", "transform")
NAMES = {
    "tintype": "Tintype default",
    "tintype-sensitive": "Tintype sensitive",
    "tintype-strict": "Tintype strict",
    "tintype-fast": "Tintype fast",
    "tintype-second": "Shipped CF blend (stored scores)",
    "commfor": "Community Forensics",
    "gpttrace": "GPTTrace",
    "swinv2": "SwinV2 detector",
}
EVERYTHING = (-math.inf, math.inf, True)
# A fusion fitted inside a gate needs this many photos of each class there; otherwise it is fitted on every photo.
MIN_PER_CLASS = 10


def display(name):
    return NAMES.get(name, name)


# ---------------------------------------------------------------- loading

def open_text(path):
    return gzip.open(path, "rt", encoding="utf-8", newline="") if str(path).endswith(".gz") else open(path, encoding="utf-8", newline="")


def new_item(key, label, generator, dataset):
    return {"key": key, "label": int(label), "generator": generator, "dataset": dataset, "z": {}, "views": {}, "groups": {}}


def items_from_rows(rows):
    """Photos of a wide table (`path label generator dataset` and one probability column per system)."""
    items = []
    for row in rows:
        item = new_item(row["path"], row["label"], row["generator"], row["dataset"])
        for column, value in row.items():
            if column not in IDENTITY:
                item["z"][column] = replay.logit(value)
        items.append(item)
    return items


def load_table(path):
    with open_text(path) as stream:
        return items_from_rows(list(csv.DictReader(stream, delimiter="\t")))


def load_confirm7():
    return items_from_rows(replay.read("confirm7-scores.tsv.gz"))


def load_frontier():
    """The paired JPEG stress archive: the folder is the transform, the file name the parent."""
    items = items_from_rows(replay.read("frontier-jpeg-stress-scores.tsv.gz"))
    for item in items:
        transform, parent = item["key"].split("/", 1)
        item["groups"].update(parent=parent, transform=transform)
    return items


def load_arena():
    """Arena originals and paired JPEGs with GPTTrace, without the repeated authentic originals
    (the same rule as scripts/replay-research.py: a later image with the bytes of an earlier one is dropped
    from both variants)."""
    manifest = replay.read("arena-v0.1-gpu/inputs.tsv.gz")
    seen, duplicates = {}, set()
    for row in manifest:
        digest = row["original_sha256"]
        if digest in seen:
            assert row["label"] == seen[digest]
            duplicates.add(row["id"])
        seen[digest] = row["label"]
    items = []
    for item in items_from_rows(replay.read("arena-v0.1-gpttrace/scores.tsv.gz")):
        variant, parent = item["key"].split("/", 1)
        if parent not in duplicates:
            item["groups"].update(parent=parent, transform=variant)
            items.append(item)
    return items, len(duplicates)


ARCHIVES = {
    # name: (loader, cohort key, profiles, stored policies, false-positive budgets)
    "confirm7": (load_confirm7, "all", ["tintype-sensitive", "tintype-strict"], ["tintype-second"], ["107", "own:tintype-second"]),
    "frontier": (load_frontier, "transform", [], [], []),
    "arena": (lambda: load_arena()[0], "dataset", [], [], []),
}


def parse_scores_arg(text):
    """`NAME=FILE[@MODE]`: MODE is `prob` (default), `logits` or `spec=SPEC.json[#profile]`."""
    name, _, rest = text.partition("=")
    if not name or not rest:
        raise ValueError(f"use NAME=FILE[@prob|@logits|@spec=SPEC.json[#profile]], not `{text}`")
    path, mode = rest, "prob"
    head, at, tail = rest.rpartition("@")
    if at and (tail in ("prob", "logits") or tail.startswith("spec=")):
        path, mode = head, tail
    return name, path, mode


def calibration_of(spec_path, profile):
    """`(scale, bias)` of a profile in a model spec: the calibrated logit is `scale * mean view logit + bias`."""
    spec = json.loads(Path(spec_path).read_text(encoding="utf-8"))
    block = spec if profile in ("", "photo", "default") else spec.get(profile)
    if not isinstance(block, dict) or "logit_scale" not in block:
        raise ValueError(f"{spec_path} has no profile `{profile}`")
    return float(block["logit_scale"]), float(block["logit_bias"])


def read_score_file(path, mode):
    """Rows of a `tintype score` file: `path label generator dataset prob logits...` (no header).

    Returns `{path: (label, generator, dataset, logit, view logits)}`. The logit is that of the probability
    column (`prob`), the mean of the logit columns (`logits`: what a linear probe writes), or the calibrated
    logit `scale * mean + bias` of a model spec (`spec=...`: exact where the six-decimal probability
    saturates, and the number the photo second opinion compares with its band)."""
    scale, bias = 1.0, 0.0
    if mode.startswith("spec="):
        spec_path, _, profile = mode[len("spec="):].partition("#")
        scale, bias = calibration_of(spec_path, profile)
    out = {}
    with open_text(path) as stream:
        for line in stream:
            f = line.rstrip("\n").split("\t")
            if len(f) < 5:
                continue
            views = [float(x) for x in f[5:] if x != ""]
            if mode == "prob":
                z = replay.logit(f[4])
            else:
                if not views:
                    raise ValueError(f"{path}: `{f[0]}` has no logit columns for @{mode}")
                z = scale * (sum(views) / len(views)) + bias
            if f[0] in out:
                raise ValueError(f"{path}: `{f[0]}` is listed twice")
            out[f[0]] = (int(f[1]), f[2], f[3], z, views)
    return out


def load_score_files(args_scores):
    """Join per-system score files on the path column; only photos every system scored are kept."""
    systems = {}
    for text in args_scores:
        name, pattern, mode = parse_scores_arg(text)
        paths = sorted(glob.glob(pattern)) if glob.has_magic(pattern) else [pattern]
        if not paths:
            raise ValueError(f"no score file matches `{pattern}`")
        rows = systems.setdefault(name, {})
        for path in paths:
            for key, value in read_score_file(path, mode).items():
                if key in rows:
                    raise ValueError(f"`{key}` is scored twice for {name}")
                rows[key] = value
    names = list(systems)
    common = [key for key in systems[names[0]] if all(key in systems[n] for n in names)]
    items = []
    for key in common:
        label, generator, dataset, _, _ = systems[names[0]][key]
        item = new_item(key, label, generator, dataset)
        for name in names:
            other = systems[name][key]
            if other[0] != label:
                raise ValueError(f"`{key}` has label {label} for {names[0]} and {other[0]} for {name}")
            item["z"][name] = other[3]
            item["views"][name] = other[4]
        items.append(item)
    dropped = {name: len(rows) - len(common) for name, rows in systems.items() if len(rows) != len(common)}
    return items, dropped


def read_manifest(path):
    """`{path: grouping columns}` of a manifest, with paths resolved as `tintype` resolves them
    (relative to the manifest's folder), so they match the path column of its score files.

    A record written by `tintype freeze-set` (format 2) is read too: its paths are taken as written, which
    is how archived score files name them, so a split can be re-evaluated from the repository alone."""
    root = os.path.dirname(str(path))
    out = {}
    with open_text(path) as stream:
        first = stream.readline()
        record = first.startswith("# tintype frozen evaluation set")
        for line in ([] if record else [first]) + list(stream):
            f = line.rstrip("\n").split("\t")
            if line.startswith("#") and record:
                continue
            if record:
                # sha256 size label generator dataset path, then the grouping columns
                if len(f) < 6:
                    continue
                key, columns = f[5], f[6:]
            else:
                if len(f) < 4:
                    continue
                key, columns = os.path.join(root, f[0]), f[4:]
            out[key] = {name: value for name, value in zip(GROUP_COLUMNS, columns) if value not in ("", "-")}
    return out


def group_value(item, key):
    """The value of a grouping key, with the fallbacks of src/split.rs (`source_of`, `family_of`)."""
    groups = item["groups"]
    if key == "all":
        return "all"
    if key in ("dataset", "generator"):
        return item[key]
    if key == "path":
        return item["key"]
    if key == "source":
        return groups.get("source") or (item["generator"] if item["label"] == 0 else item["dataset"])
    if key == "family":
        return groups.get("family") or item["generator"]
    if key in ("parent", "scene"):
        # An item without a parent or scene stands alone.
        return groups.get(key) or "path:" + item["key"]
    if key == "transform":
        return groups.get("transform") or "-"
    if key == "processing":
        # The degraded slice of the frozen evaluation: every transform other than none.
        return "original" if groups.get("transform") in (None, "", "-", "none") else "processed"
    raise ValueError(f"unknown grouping key `{key}`")


# ---------------------------------------------------------------- metrics

def sigmoid(z):
    return 1.0 / (1.0 + math.exp(-z)) if z >= 0 else math.exp(z) / (1.0 + math.exp(z))


def allowed_at(fpr, n_real):
    """Real photos a rate lets through: `floor(fpr * n)`, the rule of `metrics::empirical_cutoff`."""
    return int(math.floor(fpr * n_real))


def cutoff_for(samples, allowed):
    """The strictest cutoff that flags at most `allowed` real photos (flagging strictly above it):
    the (allowed+1)-th highest real score; with fewer real photos than that, the lowest real score
    for a rate and nothing at all for a count (as `empirical_cutoff` and `at_false_positive_count`)."""
    return replay.cutoff_at_budget(samples, allowed)


def operate(samples, cutoff):
    """`(true positives, false positives)` flagging strictly above `cutoff`."""
    counts = replay.counts_at_cutoff(samples, cutoff)
    return counts[1, True], counts[0, True]


def rate_point(samples, fpr):
    n_real = sum(y == 0 for y, _ in samples)
    if n_real == 0:
        return None
    allowed = min(allowed_at(fpr, n_real), n_real - 1)
    cutoff = cutoff_for(samples, allowed)
    return (allowed, cutoff) + operate(samples, cutoff)


def budget_point(samples, budget):
    n_real = sum(y == 0 for y, _ in samples)
    cutoff = cutoff_for(samples, budget) if budget < n_real else -math.inf
    return (budget, cutoff) + operate(samples, cutoff)


def set_parity(datasets, samples, cutoff, reference_flags):
    """Sets where both precision and recall at `cutoff` reach the reference detector at its own cutoff
    (`joint_set_wins` of the replay script, with any reference in the place of Community Forensics)."""
    rows = [{"dataset": d, "commfor": 1.0 if flagged else 0.0} for d, flagged in zip(datasets, reference_flags)]
    return replay.joint_set_wins(rows, samples, cutoff)


def slice_counts(items, flags, key, label):
    """`{slice: [photos, flagged]}` over the photos of one class."""
    out = {}
    for it, flagged in zip(items, flags):
        if it["label"] == label:
            c = out.setdefault(group_value(it, key), [0, 0])
            c[0] += 1
            c[1] += bool(flagged)
    return out


def worst_slice(items, flags, key, label, minimum):
    """The slice of one class, among those with `minimum` photos or more, with the highest flagged share
    for authentic photos (label 0) or the lowest for generated ones (label 1): `(name, flagged, photos)`."""
    slices = [(c[1] / c[0], name, c) for name, c in slice_counts(items, flags, key, label).items() if c[0] >= minimum]
    if not slices:
        return None
    _, name, c = max(slices, key=lambda x: (x[0], x[1])) if label == 0 else min(slices, key=lambda x: (x[0], x[1]))
    return name, c[1], c[0]


def slices_reaching(items, flags, reference_flags, minimum):
    """How a policy compares with a reference detector slice by slice, where sets hold one class only:
    generator families (of `minimum` generated photos or more) on which it flags at least as many as the
    reference, and authentic sources on which it flags no more. `(families reached, families, sources
    reached, sources)`."""
    out = []
    for key, label in (("family", 1), ("source", 0)):
        ours = slice_counts(items, flags, key, label)
        theirs = slice_counts(items, reference_flags, key, label)
        names = [n for n, c in ours.items() if c[0] >= minimum]
        reached = sum((ours[n][1] >= theirs[n][1]) if label == 1 else (ours[n][1] <= theirs[n][1]) for n in names)
        out += [reached, len(names)]
    return tuple(out)


def auroc(positives, negatives):
    """P(positive > negative) + half the ties."""
    if not positives or not negatives:
        return float("nan")
    order = sorted([(v, 1) for v in positives] + [(v, 0) for v in negatives])
    wins, below, i = 0.0, 0, 0
    while i < len(order):
        j = i
        while j < len(order) and order[j][0] == order[i][0]:
            j += 1
        pos = sum(flag for _, flag in order[i:j])
        neg = (j - i) - pos
        wins += pos * (below + neg / 2.0)
        below += neg
        i = j
    return wins / (len(positives) * len(negatives))


# ---------------------------------------------------------------- logistic regression

def solve(matrix, vector):
    """Solve a small dense linear system by Gaussian elimination with partial pivoting."""
    n = len(vector)
    a = [row[:] + [vector[i]] for i, row in enumerate(matrix)]
    for col in range(n):
        pivot = max(range(col, n), key=lambda r: abs(a[r][col]))
        if abs(a[pivot][col]) < 1e-300:
            raise ValueError("singular system: a constant or duplicated feature")
        a[col], a[pivot] = a[pivot], a[col]
        for r in range(col + 1, n):
            factor = a[r][col] / a[col][col]
            for c in range(col, n + 1):
                a[r][c] -= factor * a[col][c]
    x = [0.0] * n
    for r in range(n - 1, -1, -1):
        x[r] = (a[r][n] - sum(a[r][c] * x[c] for c in range(r + 1, n))) / a[r][r]
    return x


def log_loss(weights, xs, ys):
    d = len(weights) - 1
    total = 0.0
    for x, y in zip(xs, ys):
        z = weights[d] + sum(w * v for w, v in zip(weights, x))
        total += max(z, 0.0) + math.log1p(math.exp(-abs(z))) - y * z
    return total / len(xs)


def fit_logistic(xs, ys, iterations=200, tolerance=1e-10):
    """Plain logistic regression (no class weights, no penalty) by Newton's method with step halving.

    `xs` are feature tuples, `ys` are 0/1. Returns `(weights + [bias], converged)`. On separable data the
    maximum-likelihood weights are infinite: the loop then stops at `iterations` and `converged` is False."""
    d, n = len(xs[0]), len(xs)
    weights = [0.0] * (d + 1)
    loss = log_loss(weights, xs, ys)
    for _ in range(iterations):
        gradient = [0.0] * (d + 1)
        hessian = [[0.0] * (d + 1) for _ in range(d + 1)]
        for x, y in zip(xs, ys):
            xt = tuple(x) + (1.0,)
            p = sigmoid(sum(w * v for w, v in zip(weights, xt)))
            residual, curvature = p - y, p * (1.0 - p)
            for i in range(d + 1):
                gradient[i] += residual * xt[i]
                for j in range(i, d + 1):
                    hessian[i][j] += curvature * xt[i] * xt[j]
        for i in range(d + 1):
            gradient[i] /= n
            for j in range(i, d + 1):
                hessian[i][j] /= n
                hessian[j][i] = hessian[i][j]
            # A whisper of damping keeps the solve defined when the curvature vanishes (near separation).
            hessian[i][i] += 1e-12
        step = solve(hessian, gradient)
        scale = 1.0
        while True:
            candidate = [w - scale * s for w, s in zip(weights, step)]
            candidate_loss = log_loss(candidate, xs, ys)
            if candidate_loss <= loss or scale < 1e-10:
                break
            scale /= 2.0
        moved = max(abs(scale * s) for s in step)
        weights, loss = candidate, candidate_loss
        if moved < tolerance:
            return weights, True
    return weights, False


# ---------------------------------------------------------------- policies

class Policy:
    """One way to score a photo from the logit of a first model and, inside a gate, a second one."""

    def __init__(self, name, first, second=None, gate=None, weight=1.0, threshold=0.0, fitted=False, single_model=False):
        self.name, self.first, self.second = name, first, second
        self.gate = gate  # None: never routed; else (lo, hi, inclusive) on the first-stage logit
        self.weight, self.threshold = weight, threshold
        self.fitted = fitted
        self.single_model = single_model
        self.stored = False  # a column that already holds a fused policy, e.g. a real two-stage run
        self.band = False    # a Community Forensics band computed from the two separate scores

    def routed(self, item):
        if self.gate is None:
            return False
        lo, hi, inclusive = self.gate
        z = item["z"][self.first]
        return lo <= z <= hi if inclusive else lo < z < hi

    def score(self, item, model=None):
        z1 = item["z"][self.first]
        if not self.routed(item):
            return z1 if model is None else model["out"][0] * z1 + model["out"][1]
        z2 = item["z"][self.second]
        if model is None:
            return 0.5 * (self.weight * z1 + z2 - self.threshold)
        w1, w2, bias = model["in"]
        return w1 * z1 + w2 * z2 + bias


def gate_text(gate, first):
    lo, hi, inclusive = gate
    if gate == EVERYTHING:
        return "every photo"
    if inclusive and lo == -hi:
        return f"|{first} logit| <= {hi:g}"
    return f"{lo:g} {'<=' if inclusive else '<'} {first} logit {'<=' if inclusive else '<'} {hi:g}"


def fit_fusion(policy, fit_items):
    """Fit the weights of a fitted policy. Inside the gate: logistic regression of the label on both
    logits, over the fitting photos inside the gate. Outside: logistic regression on the first logit alone
    over all fitting photos, so both sides of the gate are log-odds fitted on the same photos and one
    cutoff can run across them."""
    inside = [it for it in fit_items if policy.routed(it)]
    note = ""
    if min(sum(it["label"] == y for it in inside) for y in (0, 1)) < MIN_PER_CLASS:
        note = f" (only {len(inside)} fitting photos in the gate: fitted on all {len(fit_items)})"
        inside = fit_items
    w_in, ok_in = fit_logistic([(it["z"][policy.first], it["z"][policy.second]) for it in inside], [it["label"] for it in inside])
    w_out, ok_out = fit_logistic([(it["z"][policy.first],) for it in fit_items], [it["label"] for it in fit_items])
    if not (ok_in and ok_out):
        note += " (not converged: the classes may be separable)"
    return {"in": tuple(w_in), "out": tuple(w_out), "n_in": len(inside), "n": len(fit_items), "note": note}


def fold_of(key, folds):
    return int(hashlib.sha256(key.encode("utf-8")).hexdigest()[:12], 16) % folds


def build_policies(systems, first, profiles, stored, cf, bands, seconds, margins, with_fitted):
    policies = [Policy(display(first), first, single_model=True)]
    policies += [Policy(display(p), p, single_model=True) for p in profiles]
    for name in stored:
        policies.append(Policy(display(name), name))
        policies[-1].stored = True
    if cf:
        for label, (lo, hi, weight, threshold) in bands:
            name = f"{label} CF blend: band ({lo:g}, {hi:g}), weight {weight:g}, threshold {threshold:g}"
            policies.append(Policy(name, first, cf, (lo, hi, False), weight, threshold))
            policies[-1].band = True
    for second in seconds:
        pair = f"{display(first)} + {display(second)}"
        policies.append(Policy(f"{display(second)} alone", second, single_model=False))
        gates = [(-m, m, True) for m in margins] + [EVERYTHING]
        for gate in gates:
            where = "full fusion" if gate == EVERYTHING else f"for {gate_text(gate, first)}"
            policies.append(Policy(f"{pair}, 50:50 {where}", first, second, gate))
        if with_fitted:
            for gate in gates:
                where = "full fusion" if gate == EVERYTHING else f"for {gate_text(gate, first)}"
                policies.append(Policy(f"{pair}, fitted {where}", first, second, gate, fitted=True))
    missing = sorted({n for p in policies for n in (p.first, p.second) if n and n not in systems})
    if missing:
        raise ValueError(f"no scores for {', '.join(missing)}; have {', '.join(sorted(systems))}")
    return policies


# ---------------------------------------------------------------- reporting

def pct(a, b):
    return f"{a / b:.2%}" if b else "n/a"


def cell(tp, n, base_tp=None):
    text = f"{tp}/{n} ({pct(tp, n)})"
    if base_tp is not None and n:
        text += f" {100.0 * (tp - base_tp) / n:+.2f}"
    return text


def table(header, rows, out):
    out.write("| " + " | ".join(header) + " |\n")
    out.write("|" + "|".join("---" if i == 0 else "---:" for i in range(len(header))) + "|\n")
    for row in rows:
        out.write("| " + " | ".join(row) + " |\n")


def resolve_budgets(tokens, policies, samples_of):
    budgets = []
    for token in tokens:
        if token.startswith("own:"):
            name = token[4:]
            policy = next((p for p in policies if p.gate is None and p.first == name), None)
            if policy is None:
                raise ValueError(f"--fp-budget {token}: no single-column policy `{name}`")
            budgets.append(operate(samples_of(policy), 0.0)[1])
        else:
            budgets.append(int(token))
    return budgets


def report_cohort(title, items, policies, scores, opts, out, frozen=None, chosen_on="this cohort", whole=False):
    """Print the tables of one cohort. `scores[policy]` is aligned with `items`; `frozen[policy]` holds the
    cutoffs frozen on the calibration set."""
    labels = [it["label"] for it in items]
    n_real = labels.count(0)
    n_gen = len(items) - n_real
    out.write(f"\n## {title}: {len(items)} photos, {n_gen} generated, {n_real} authentic\n\n")
    if n_real == 0 or n_gen == 0:
        out.write("One class is missing: no operating points.\n")
        return {}

    def samples_of(policy):
        return list(zip(labels, scores[policy]))

    budgets = resolve_budgets(opts["budgets"], policies, samples_of)
    baseline = policies[0]
    reference = opts["reference"]
    datasets = [it["dataset"] for it in items]
    n_sets = len(set(datasets))
    reference_flags = None
    if reference and n_sets > 1:
        name, cutoff = reference
        reference_flags = [it["z"][name] > cutoff for it in items]

    results = {}
    for policy in policies:
        samples = samples_of(policy)
        routed = sum(policy.routed(it) for it in items)
        points = [("rate", fpr) + rate_point(samples, fpr) for fpr in opts["fprs"]]
        points += [("count", b) + budget_point(samples, b) for b in budgets]
        own = operate(samples, 0.0)
        parity = None
        if reference_flags is not None:
            parity = [set_parity(datasets, samples, p[3], reference_flags)[0] for p in points]
            parity.append(set_parity(datasets, samples, 0.0, reference_flags)[0])
        results[policy] = {"routed": routed, "points": points, "own": own, "parity": parity}

    header = ["Policy", "Routed"]
    header += [f"Recall at {100 * fpr:g}% FPR (<= {allowed_at(fpr, n_real)} FP), points vs {baseline.name}" for fpr in opts["fprs"]]
    header += [f"Recall at {b} FP ({pct(b, n_real)} FPR)" for b in budgets]
    header += ["At own cutoff: TP, FP (recall, FPR)"]
    rows = []
    notes = []
    for policy in policies:
        r = results[policy]
        row = [policy.name, f"{r['routed']}/{len(items)} ({pct(r['routed'], len(items))})"]
        for k, point in enumerate(r["points"]):
            kind, target, allowed, _, tp, fp = point
            base_tp = results[baseline]["points"][k][4] if kind == "rate" else None
            row.append(cell(tp, n_gen, base_tp))
            if fp != allowed:
                notes.append(f"{policy.name}: {fp} of {allowed} allowed FPs used at {'%g%% FPR' % (100 * target) if kind == 'rate' else '%d FP' % target} (tied scores at the cutoff)")
        tp, fp = r["own"]
        row.append(f"{tp}, {fp} ({pct(tp, n_gen)}, {pct(fp, n_real)})")
        rows.append(row)
    out.write(f"Cutoffs chosen on {chosen_on}; flagged strictly above the cutoff.\n\n")
    table(header, rows, out)
    for note in notes:
        out.write(f"- {note}\n")
    if any(p.fitted for p in policies):
        out.write("- The own cutoff of a fitted policy is fitted probability 0.5 at the class mix of the fitting photos: it is not calibrated to a false-positive target.\n")

    singles = [p for p in policies if p.single_model]
    if len(singles) > 1:
        out.write("\nStrongest single-model profile at a re-chosen cutoff")
        best = []
        for k, point in enumerate(results[singles[0]]["points"]):
            top = max(singles, key=lambda p: results[p]["points"][k][4])
            where = f"{100 * point[1]:g}% FPR" if point[0] == "rate" else f"{point[1]} FP"
            best.append(f"{where}: {top.name} ({results[top]['points'][k][4]}/{n_gen})")
        out.write(f", chosen on {chosen_on}: " + "; ".join(best) + ".\n")

    if reference_flags is not None:
        name, cutoff = reference
        out.write(f"\nSets (of {n_sets}) where precision and recall both reach {display(name)} at its own cutoff:\n\n")
        header = ["Policy"] + [f"at {100 * fpr:g}% FPR" for fpr in opts["fprs"]] + [f"at {b} FP" for b in budgets] + ["at own cutoff"]
        table(header, [[p.name] + [f"{w}/{n_sets}" for w in results[p]["parity"]] for p in policies], out)

    if frozen is not None:
        out.write(f"\nCutoffs frozen on the calibration set ({frozen['n_real']} authentic photos there), applied here unchanged:\n\n")
        header = ["Policy"] + [c for fpr in opts["fprs"] for c in (f"target {100 * fpr:g}% FPR: cutoff (logit)", "actual FP (FPR)", "recall")]
        rows = []
        for policy in policies:
            samples = samples_of(policy)
            row = [policy.name]
            for cutoff in frozen["cutoffs"][policy]:
                tp, fp = operate(samples, cutoff)
                row += [f"{cutoff:.5f}", f"{fp}/{n_real} ({pct(fp, n_real)})", cell(tp, n_gen)]
            rows.append(row)
        table(header, rows, out)

    minimum = opts["min_slice"]
    if whole and frozen is not None and opts["worst_slices"]:
        out.write(f"\nWorst slices at the frozen cutoffs (slices of {minimum} photos of the class or more): the authentic source and "
                  "the transform with the highest flagged share, and the generator family with the lowest recall.\n\n")
        header = ["Policy"] + [c for fpr in opts["fprs"] for c in (f"target {100 * fpr:g}% FPR: worst authentic source", "worst transform (authentic)", "weakest family")]
        rows = []
        for policy in policies:
            row = [policy.name]
            for cutoff in frozen["cutoffs"][policy]:
                flags = [z > cutoff for z in scores[policy]]
                for key, label in (("source", 0), ("transform", 0), ("family", 1)):
                    worst = worst_slice(items, flags, key, label, minimum)
                    row.append("n/a" if worst is None else f"{worst[0]} {worst[1]}/{worst[2]} ({pct(worst[1], worst[2])})")
            rows.append(row)
        table(header, rows, out)
    if whole and opts["slice_parity"]:
        name, cutoff = opts["slice_parity"]
        reference_flags = [it["z"][name] > cutoff for it in items]
        out.write(f"\nSlice by slice against {display(name)} at its own cutoff, each policy at its own cutoff (slices of {minimum} photos of the class "
                  "or more): generator families on which the policy flags at least as many generated photos, and authentic sources on which it flags no more.\n\n")
        rows = []
        for policy in policies:
            fam, n_fam, src, n_src = slices_reaching(items, [z > 0.0 for z in scores[policy]], reference_flags, minimum)
            rows.append([policy.name, f"{fam}/{n_fam}", f"{src}/{n_src}", f"{fam + src}/{n_fam + n_src}"])
        table(["Policy", "Families reached", "Sources reached", "Together"], rows, out)
    return results


def gate_report(items, policies, opts, out):
    """What each gate leaves outside: the first-stage errors a second model never sees."""
    pairs = []
    for policy in policies:
        if policy.second and not policy.fitted and (policy.first, policy.second) not in pairs:
            pairs.append((policy.first, policy.second))
    for first, second in pairs:
        gates = []
        for policy in policies:
            if policy.first == first and policy.second == second and policy.gate not in gates and policy.gate != EVERYTHING:
                gates.append(policy.gate)
        own = len(gates)
        gates += [g for g in ((-m, m, True) for m in opts["sweep"]) if g not in gates]
        errors = {y: [it for it in items if it["label"] == y and (it["z"][first] > 0) != bool(y)] for y in (0, 1)}
        fixed = {y: sum((it["z"][second] > 0) == bool(y) for it in errors[y]) for y in (0, 1)}
        n = {y: sum(it["label"] == y for it in items) for y in (0, 1)}
        out.write(f"\nGates on {display(first)} with {display(second)} behind them, both at their own cutoffs. "
                  f"{display(first)} misses {len(errors[1])} of {n[1]} generated photos ({display(second)} flags {fixed[1]} of them) "
                  f"and flags {len(errors[0])} of {n[0]} authentic photos ({display(second)} clears {fixed[0]}).\n\n")
        header = ["Gate", "Routed", "Misses inside", "Misses outside", f"Outside, flagged by {display(second)}", "False alarms outside", f"Outside, cleared by {display(second)}"]
        rows = []
        for k, gate in enumerate(gates):
            probe = Policy("", first, second, gate)
            routed = sum(probe.routed(it) for it in items)
            outside = {y: [it for it in errors[y] if not probe.routed(it)] for y in (0, 1)}
            caught = {y: sum((it["z"][second] > 0) == bool(y) for it in outside[y]) for y in (0, 1)}
            rows.append([gate_text(gate, first) + ("" if k < own else " (sweep)"), f"{routed}/{len(items)} ({pct(routed, len(items))})",
                         str(len(errors[1]) - len(outside[1])), str(len(outside[1])), str(caught[1]), str(len(outside[0])), str(caught[0])])
        table(header, rows, out)
        if errors[1]:
            depth = sorted(-it["z"][first] for it in errors[1])
            parts = []
            for share in (0.5, 0.9, 1.0):
                margin = depth[max(0, math.ceil(share * len(depth)) - 1)]
                routed = sum(abs(it["z"][first]) <= margin for it in items)
                parts.append(f"{share:.0%} of the misses needs |logit| <= {margin:.2f}, which routes {routed}/{len(items)} ({pct(routed, len(items))})")
            out.write("\nA symmetric margin gate that takes in " + "; ".join(parts) + ".\n")

        spread_of = opts["view_spread"]
        if spread_of and gates:
            if any(len(it["views"].get(spread_of, ())) < 2 for it in items):
                raise ValueError(f"--view-spread {spread_of}: every photo needs two or more view logits (score with --all-views)")

            def spread(it):
                views = it["views"][spread_of]
                return max(views) - min(views)

            out.write(f"\nView disagreement (largest minus smallest view logit of {spread_of}) as a routing feature, analysis only:\n\n")
            header = ["Gate", "Confident negatives outside: generated, authentic", "AUROC of view spread for generated among them",
                      "Misses inside the margin gate", "Misses among as many photos with the largest view spread", "In both"]
            rows = []
            by_spread = sorted(items, key=lambda it: (-spread(it), it["key"]))
            for gate in gates[:own] or gates[:1]:
                probe = Policy("", first, second, gate)
                below = [it for it in items if it["z"][first] < gate[0]]
                gen = [spread(it) for it in below if it["label"] == 1]
                real = [spread(it) for it in below if it["label"] == 0]
                routed = [it for it in items if probe.routed(it)]
                top = {it["key"] for it in by_spread[:len(routed)]}
                inside = {it["key"] for it in errors[1] if probe.routed(it)}
                by_view = {it["key"] for it in errors[1] if it["key"] in top}
                area = f"{auroc(gen, real):.4f}" if gen and real else "n/a"
                rows.append([gate_text(gate, first), f"{len(gen)}, {len(real)}", area, str(len(inside)), str(len(by_view)), str(len(inside & by_view))])
            table(header, rows, out)

        slice_by = opts["slice_by"]
        for gate in gates[:own] if slice_by else []:
            probe = Policy("", first, second, gate)
            counts = {}
            for it in items:
                c = counts.setdefault(group_value(it, slice_by), Counter())
                c["generated"] += it["label"]
                if it["label"] == 1 and it["z"][first] <= 0:
                    c["misses"] += 1
                    if not probe.routed(it):
                        c["outside"] += 1
                        c["caught"] += it["z"][second] > 0
            order = sorted((k for k in counts if counts[k]["outside"]), key=lambda k: (-counts[k]["outside"], k))
            if not order:
                continue
            out.write(f"\nMisses of {display(first)} outside the gate {gate_text(gate, first)}, by {slice_by} "
                      f"({min(len(order), opts['max_slices'])} of {len(order)} slices with such a miss, most first):\n\n")
            table([slice_by, "Generated", "Misses", "Misses outside the gate", f"Outside, flagged by {display(second)}"],
                  [[k, str(counts[k]["generated"]), str(counts[k]["misses"]), str(counts[k]["outside"]), str(counts[k]["caught"])] for k in order[:opts["max_slices"]]], out)


def stored_report(items, policies, scores, out):
    """A stored policy (a real two-stage run) against each band computed from the separate score files:
    how many verdicts at the own cutoff differ, and how many photos each sent on or changed."""
    stored = [p for p in policies if p.stored]
    bands = [p for p in policies if p.band]
    if not stored or not bands:
        return
    out.write("\nStored policies against the bands computed from the separate scores (verdicts at the own cutoff):\n\n")
    rows = []
    for a in stored:
        for b in bands:
            differ = sum((x > 0.0) != (y > 0.0) for x, y in zip(scores[a], scores[b]))
            moved = sum(abs(sigmoid(x) - sigmoid(it["z"][b.first])) > 5e-6 for x, it in zip(scores[a], items))
            routed = sum(b.routed(it) for it in items)
            rows.append([a.name, b.name, f"{differ}/{len(items)}", str(moved), str(routed)])
    table(["Stored policy", "Computed band", "Verdicts that differ", "Stored: probabilities that differ from the first stage", "Computed: photos inside the band"], rows, out)


def write_scores(directory, items, policies, scores):
    """One `tintype score`-style file per policy (`path label generator dataset prob`), for
    `tintype report --operating-points --system name=file`: intervals, slices and clustered bootstraps."""
    os.makedirs(directory, exist_ok=True)
    written = []
    for policy in policies:
        slug = re.sub(r"[^a-z0-9]+", "-", policy.name.lower()).strip("-")
        path = os.path.join(directory, slug + ".tsv")
        with open(path, "w", encoding="utf-8", newline="") as stream:
            for it, z in zip(items, scores[policy]):
                stream.write(f"{it['key']}\t{it['label']}\t{it['generator']}\t{it['dataset']}\t{sigmoid(z):.9g}\n")
        written.append(path)
    return written


def write_manifest(path, items):
    """`path label generator dataset parent_id scene_id source_id generator_family transform`, gaps as `-`."""
    with open(path, "w", encoding="utf-8", newline="") as stream:
        for it in items:
            columns = [it["groups"].get(name, "-") for name in GROUP_COLUMNS]
            while columns and columns[-1] == "-":
                columns.pop()
            stream.write("\t".join([it["key"], str(it["label"]), it["generator"], it["dataset"]] + columns) + "\n")


# ---------------------------------------------------------------- the comparison

def split_cohorts(items, key):
    cohorts = {}
    for it in items:
        cohorts.setdefault(group_value(it, key), []).append(it)
    return cohorts


def describe_fit(policy, model, out, where):
    w1, w2, bias = model["in"]
    scale, shift = model["out"]
    text = f"- {policy.name}, {where}: {w1:+.4f} x {policy.first} logit {w2:+.4f} x {policy.second} logit {bias:+.4f} from {model['n_in']} photos"
    if policy.gate != EVERYTHING:
        text += f" inside the gate; outside it {scale:+.4f} x {policy.first} logit {shift:+.4f} from {model['n']}"
    out.write(text + model["note"] + "\n")


def compare(items, opts, out):
    """Score every policy and print the report. `items` carry `set` (fit, calibrate, test) when a split is given."""
    systems = set(items[0]["z"])
    sets = Counter(it.get("set") for it in items)
    report = opts["report_on"]
    use_sets = report in sets
    with_fitted = bool(opts["seconds"]) and (use_sets or opts["cross_fit"] > 0)
    if use_sets and (sets.get("fit", 0) == 0 or report == "fit"):
        with_fitted = False
    policies = build_policies(systems, opts["first"], opts["profiles"], opts["stored"], opts["cf"], opts["bands"], opts["seconds"], opts["margins"], with_fitted)
    fitted = [p for p in policies if p.fitted]
    scores = {p: [p.score(it) for it in items] for p in policies if not p.fitted}

    if opts["seconds"] and not with_fitted:
        out.write("Fitted fusion is left out: it needs `--set fit=... --set test=...` or `--cross-fit K`, "
                  "so that no weight is fitted on the photos it is reported on.\n")
    if fitted and use_sets:
        fit_items = [it for it in items if it["set"] == "fit"]
        out.write(f"\nFusion weights fitted on the fit set ({len(fit_items)} photos) only:\n\n")
        for policy in fitted:
            model = fit_fusion(policy, fit_items)
            describe_fit(policy, model, out, "fit set")
            scores[policy] = [policy.score(it, model) for it in items]
    elif fitted:
        folds, by = opts["cross_fit"], opts["group_by"]
        fold = [fold_of(group_value(it, by), folds) for it in items]
        cohort = [group_value(it, opts["cohort_by"]) for it in items]
        fit_cohort = opts["fit_cohort"]
        if fit_cohort and fit_cohort not in cohort:
            raise ValueError(f"--fit-cohort {fit_cohort}: no such cohort (have {', '.join(sorted(set(cohort)))})")
        groups = len({group_value(it, by) for it in items})
        out.write(f"\nFusion weights cross-fitted: {folds} folds of whole `{by}` groups ({groups} groups), each fold scored "
                  f"with weights fitted on the other folds{' of cohort ' + fit_cohort if fit_cohort else ''}. Exploratory.\n\n")
        for policy in fitted:
            values = [None] * len(items)
            for k in range(folds):
                fit_items = [it for it, f, c in zip(items, fold, cohort) if f != k and (not fit_cohort or c == fit_cohort)]
                model = fit_fusion(policy, fit_items)
                describe_fit(policy, model, out, f"fold {k + 1}")
                for i, it in enumerate(items):
                    if fold[i] == k:
                        values[i] = policy.score(it, model)
            scores[policy] = values

    def subset(keep):
        index = [i for i, it in enumerate(items) if keep(it)]
        return [items[i] for i in index], {p: [scores[p][i] for i in index] for p in policies}

    if use_sets:
        frozen = None
        calibration = "calibrate" if sets.get("calibrate") else ("fit" if sets.get("fit") else None)
        if calibration:
            cal_items, cal_scores = subset(lambda it: it["set"] == calibration)
            cal_labels = [it["label"] for it in cal_items]
            n_real = cal_labels.count(0)
            if n_real:
                frozen = {"n_real": n_real, "cutoffs": {}}
                for policy in policies:
                    samples = list(zip(cal_labels, cal_scores[policy]))
                    frozen["cutoffs"][policy] = [rate_point(samples, fpr)[1] for fpr in opts["fprs"]]
                out.write(f"\nCutoffs are frozen on the `{calibration}` set ({len(cal_items)} photos, {n_real} authentic)"
                          + (": no calibration set was given, so the fit set stands in for it.\n" if calibration == "fit" and report != "fit" else ".\n"))
                if calibration == report:
                    out.write(f"The `{report}` set is reported here, so these cutoffs are in-sample: this is the view for choosing on `{report}`, not a transfer result.\n")
        report_items, report_scores = subset(lambda it: it["set"] == report)
        title = {"test": "Test set", "calibrate": "Calibration set", "fit": "Fit set"}[report]
        chosen_on = f"the {report} set itself (a ranking comparison, not a frozen operating point)"
        report_cohort(title, report_items, policies, report_scores, opts, out, frozen, chosen_on, whole=True)
        stored_report(report_items, policies, report_scores, out)
        if opts["gates"]:
            gate_report(report_items, policies, opts, out)
        if opts["cohort_by"] != "all":
            for name, members in split_cohorts(report_items, opts["cohort_by"]).items():
                keys = {it["key"] for it in members}
                part, part_scores = subset(lambda it: it["set"] == report and it["key"] in keys)
                report_cohort(f"{title}, {opts['cohort_by']} {name}", part, policies, part_scores, opts, out, frozen, chosen_on)
                if opts["gates"]:
                    gate_report(part, policies, opts, out)
    else:
        report_items, report_scores = items, scores
        for name, members in split_cohorts(items, opts["cohort_by"]).items():
            keys = {it["key"] for it in members}
            part, part_scores = subset(lambda it: it["key"] in keys)
            title = "All photos" if opts["cohort_by"] == "all" else name
            report_cohort(title, part, policies, part_scores, opts, out, None, "this cohort, so exploratory", whole=opts["cohort_by"] == "all")
            stored_report(part, policies, part_scores, out)
            if opts["gates"]:
                gate_report(part, policies, opts, out)
    if opts["write_scores"]:
        written = write_scores(opts["write_scores"], report_items, policies, report_scores)
        out.write(f"\nWrote {len(written)} score files to {opts['write_scores']} (`path label generator dataset prob`).\n")
    return policies, scores


def shipped_bands():
    band = json.loads((REPO / "models" / "tintype.json").read_text(encoding="utf-8"))["second_opinion"]
    shipped = (band["lo"], band["hi"], band["weight"], band["threshold"])
    return [("Shipped", shipped), ("Narrowed", (-3.0,) + shipped[1:])]


def parse_band(text):
    try:
        values = tuple(float(x) for x in text.split(","))
    except ValueError:
        values = ()
    if len(values) != 4 or not values[0] < values[1]:
        raise argparse.ArgumentTypeError("a band is lo,hi,weight,threshold with lo < hi")
    return values


def floats(text):
    return [float(x) for x in text.split(",") if x.strip()]


def default_opts():
    return {"first": "tintype", "profiles": [], "stored": [], "cf": None, "bands": shipped_bands(), "seconds": [], "margins": [2.0],
            "reference": None, "fprs": [0.001, 0.01], "budgets": [], "cohort_by": "all", "cross_fit": 0, "group_by": "parent",
            "fit_cohort": None, "sweep": [1.0, 2.0, 3.0, 4.0, 6.0, 8.0], "view_spread": None, "slice_by": None, "max_slices": 15,
            "write_scores": None, "report_on": "test", "gates": True, "worst_slices": False, "slice_parity": None, "min_slice": 100}


def archive_opts(name, items):
    _, cohort_by, profiles, stored, budgets = ARCHIVES[name]
    systems = set(items[0]["z"])
    opts = default_opts()
    opts.update(cohort_by=cohort_by, profiles=profiles, stored=stored, budgets=budgets)
    if "commfor" in systems:
        opts.update(cf="commfor", reference=("commfor", 0.0))
    if "gpttrace" in systems:
        opts["seconds"] = ["gpttrace"]
    return opts


# ---------------------------------------------------------------- checks

def replay_lines():
    """The photo lines scripts/replay-research.py prints, captured from its own functions."""
    buffer = io.StringIO()
    with redirect_stdout(buffer):
        replay.replay_cf()
        replay.replay_arena()
    return buffer.getvalue().splitlines()


def harness_lines():
    """The same lines, rebuilt from this script's policies and counting."""
    lines = []
    items = load_confirm7()
    opts = archive_opts("confirm7", items)
    policies, scores = compare(items, opts, io.StringIO())
    labels = [it["label"] for it in items]
    datasets = [it["dataset"] for it in items]
    reference_flags = [it["z"]["commfor"] > 0 for it in items]
    by_name = {p.name: p for p in policies}
    stored = by_name[display("tintype-second")]
    shipped = next(p for p in policies if p.name.startswith("Shipped CF blend: band"))
    narrow = next(p for p in policies if p.name.startswith("Narrowed CF blend"))
    mismatches = sum((a > 0) != (b > 0) for a, b in zip(scores[shipped], scores[stored]))
    lines.append(f"Confirmation 7 CF fusion reproduces shipped verdicts with {mismatches} mismatches.")
    budget_329 = operate(list(zip(labels, scores[stored])), 0.0)[1]
    for name, policy, router in (("Tintype default", by_name[display("tintype")], None), ("Tintype sensitive", by_name[display("tintype-sensitive")], None),
                                 ("Shipped CF fusion", stored, shipped), ("Narrow CF fusion", narrow, narrow)):
        routed = sum(router.routed(it) for it in items) if router else 0
        samples = list(zip(labels, scores[policy]))
        lines.append(f"  {name}: routed {routed}/{len(items)} ({routed / len(items):.2%})")
        for budget in (107, budget_329):
            _, cutoff, tp, fp = budget_point(samples, budget)
            wins, sets = set_parity(datasets, samples, cutoff, reference_flags)
            lines.append(f"    {tp}/{labels.count(1)} TPs at {fp}/{budget} allowed FPs; matched or beat CF precision and recall on {wins}/{sets} sets")
        tp, fp = operate(samples, 0.0)
        lines.append(f"    at score > 0.5: {tp} TPs, {fp} FPs")

    items = load_frontier()
    policies, scores = compare(items, archive_opts("frontier", items), io.StringIO())
    default = policies[0]
    narrow = next(p for p in policies if p.name.startswith("Narrowed CF blend"))
    for cohort in ("original", "jpeg", "jpeg-960"):
        index = [i for i, it in enumerate(items) if it["groups"]["transform"] == cohort]
        labels = [items[i]["label"] for i in index]
        found = []
        for policy in (default, narrow):
            _, _, tp, _ = rate_point([(items[i]["label"], scores[policy][i]) for i in index], 0.01)
            found.append((tp, labels.count(1)))
        routed = sum(narrow.routed(items[i]) for i in index)
        lines.append(f"  Frontier {cohort}: default {found[0]}, narrow {found[1]} (TP, generated at 1% empirical FPR); {routed / len(index):.1%} routed")

    items = load_arena()[0]
    policies, scores = compare(items, archive_opts("arena", items), io.StringIO())
    by_name = {p.name: p for p in policies}
    pair = f"{display('tintype')} + {display('gpttrace')}"
    gated = by_name[f"{pair}, 50:50 for |tintype logit| <= 2"]
    shown = (("Tintype", policies[0]), ("GPTTrace", by_name["GPTTrace alone"]), ("50:50 full fusion", by_name[f"{pair}, 50:50 full fusion"]),
             ("50:50 fusion for |Tintype logit| <= 2", gated))
    for dataset in ("arena-original", "arena-jpeg-q4"):
        index = [i for i, it in enumerate(items) if it["dataset"] == dataset]
        labels = [items[i]["label"] for i in index]
        routed = sum(gated.routed(items[i]) for i in index)
        lines.append(f"{dataset}: {len(index)} unique originals; gated {routed} ({routed / len(index):.1%})")
        for name, policy in shown:
            samples = [(items[i]["label"], scores[policy][i]) for i in index]
            _, _, tp, _ = rate_point(samples, 0.01)
            n = labels.count(1)
            lines.append(f"  {name}: {tp}/{n} ({tp / n:.2%}) at 1% empirical FPR; {operate(samples, 0.0)[1]}/{len(index) - n} FP at score > 0.5")
        for y in (0, 1):
            errors = [items[i] for i in index if items[i]["label"] == y and (items[i]["z"]["tintype"] > 0) != bool(y)]
            outside = sum(not gated.routed(it) for it in errors)
            correct = sum((it["z"]["gpttrace"] > 0) == bool(y) for it in errors)
            lines.append(f"  label {y}: {len(errors)} Tintype errors; {outside} outside band; {correct} GPTTrace correct")
    return lines


# The baselines of the research report (section "Photo routing at equal false-positive counts"): a change to an
# archive, to the shipped band or to the counting shows up here even if both scripts drift together.
PINNED = (
    "  Tintype default: routed 0/23052 (0.00%)",
    "    10887/12305 TPs at 107/107 allowed FPs; matched or beat CF precision and recall on 12/30 sets",
    "    11468/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 14/30 sets",
    "    11670/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 14/30 sets",
    "  Shipped CF fusion: routed 11068/23052 (48.01%)",
    "    10486/12305 TPs at 107/107 allowed FPs; matched or beat CF precision and recall on 13/30 sets",
    "    11504/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 26/30 sets",
    "  Narrow CF fusion: routed 2417/23052 (10.48%)",
    "    10872/12305 TPs at 107/107 allowed FPs; matched or beat CF precision and recall on 14/30 sets",
    "    11584/12305 TPs at 329/329 allowed FPs; matched or beat CF precision and recall on 20/30 sets",
    "  Frontier jpeg: default (139, 261), narrow (116, 261) (TP, generated at 1% empirical FPR); 27.0% routed",
    "  Frontier jpeg-960: default (73, 261), narrow (67, 261) (TP, generated at 1% empirical FPR); 38.1% routed",
    "arena-original: 2031 unique originals; gated 264 (13.0%)",
    "  Tintype: 901/1018 (88.51%) at 1% empirical FPR; 10/1013 FP at score > 0.5",
    "  50:50 fusion for |Tintype logit| <= 2: 930/1018 (91.36%) at 1% empirical FPR; 5/1013 FP at score > 0.5",
    "arena-jpeg-q4: 2031 unique originals; gated 382 (18.8%)",
    "  Tintype: 744/1018 (73.08%) at 1% empirical FPR; 7/1013 FP at score > 0.5",
    "  50:50 full fusion: 834/1018 (81.93%) at 1% empirical FPR; 5/1013 FP at score > 0.5",
    "  50:50 fusion for |Tintype logit| <= 2: 825/1018 (81.04%) at 1% empirical FPR; 5/1013 FP at score > 0.5",
    "  label 1: 310 Tintype errors; 140 outside band; 162 GPTTrace correct",
)


def check_replay():
    theirs, ours = replay_lines(), harness_lines()
    bad = 0
    for k in range(max(len(theirs), len(ours))):
        a = theirs[k] if k < len(theirs) else "<nothing>"
        b = ours[k] if k < len(ours) else "<nothing>"
        if a == b:
            print(f"ok       {a}")
        else:
            bad += 1
            print(f"MISMATCH replay:  {a}\n         harness: {b}")
    missing = [line for line in PINNED if line not in ours]
    for line in missing:
        print(f"MISSING  pinned baseline: {line}")
    print(f"{len(theirs)} replay lines, {bad} mismatches; {len(PINNED) - len(missing)}/{len(PINNED)} pinned report baselines present.")
    return 1 if bad or missing else 0


def self_test():
    close = lambda a, b, tol=1e-9: abs(a - b) <= tol

    # Cutoffs: four real scores, eight generated. One allowed false positive puts the cutoff at the
    # second-highest real score (0.6): the real 0.9 is flagged, and the generated scores above 0.6.
    samples = [(0, 0.9), (0, 0.6), (0, 0.2), (0, 0.1)] + [(1, s) for s in (0.95, 0.7, 0.61, 0.6, 0.5, 0.3, 0.15, 0.05)]
    assert budget_point(samples, 1) == (1, 0.6, 3, 1)
    assert budget_point(samples, 0) == (0, 0.9, 1, 0)
    assert budget_point(samples, 4) == (4, -math.inf, 8, 4), "more allowed than there are real photos flags everything"
    assert allowed_at(0.25, 4) == 1 and allowed_at(0.01, 10747) == 107 and allowed_at(0.001, 10747) == 10 and allowed_at(0.01, 1000) == 10
    assert rate_point(samples, 0.25) == (1, 0.6, 3, 1)
    assert rate_point(samples, 0.24) == (0, 0.9, 1, 0), "floor(0.24 * 4) = 0 allowed"
    assert rate_point(samples, 0.99) == (3, 0.1, 7, 3), "a rate never flags the lowest real photo"
    # Ties at the cutoff are not flagged, so fewer false positives than allowed can be used.
    tied = [(0, 0.5), (0, 0.5), (0, 0.5), (0, 0.1), (1, 0.6), (1, 0.5)]
    assert budget_point(tied, 1) == (1, 0.5, 1, 0)
    assert operate(samples, 0.5) == (4, 2)

    # Gates: the Community Forensics band is open, the margin gate is closed.
    item = lambda z1, z2, label=1, dataset="s": {"key": f"{z1}/{z2}/{label}/{dataset}", "label": label, "generator": "g", "dataset": dataset,
                                                 "z": {"a": z1, "b": z2}, "views": {}, "groups": {}}
    band = Policy("band", "a", "b", (-7.0, 0.5, False), 0.5, 0.0)
    assert [band.routed(item(z, 0)) for z in (-7.0, -6.99, 0.49, 0.5)] == [False, True, True, False]
    margin = Policy("margin", "a", "b", (-2.0, 2.0, True))
    assert [margin.routed(item(z, 0)) for z in (-2.01, -2.0, 2.0, 2.01)] == [False, True, True, False]
    # sigmoid(0.5 * (0.5 * -2 + 4 - 0)) has logit 1.5; with a threshold of 1, logit 1.0; outside, the first logit.
    assert band.score(item(-2.0, 4.0)) == 1.5
    assert Policy("t", "a", "b", (-7.0, 0.5, False), 0.5, 1.0).score(item(-2.0, 4.0)) == 1.0
    assert band.score(item(3.0, -9.0)) == 3.0
    assert margin.score(item(1.0, -4.0)) == -1.5 and Policy("full", "a", "b", EVERYTHING).score(item(9.0, -4.0)) == 2.5
    assert gate_text((-2.0, 2.0, True), "a") == "|a logit| <= 2" and gate_text((-7.0, 0.5, False), "a") == "-7 < a logit < 0.5"

    # Set parity. Set s: reference flags 2 of 3 generated and 1 of 2 real (precision 2/3, recall 2/3).
    # The policy flags 2 generated and no real photo there: both reached. Set t: the reference flags
    # its one generated photo and the policy does not: recall is short. One set of two.
    rows = [("s", 1, 2.0, True), ("s", 1, 1.0, True), ("s", 1, -1.0, False), ("s", 0, -2.0, True), ("s", 0, -3.0, False),
            ("t", 1, -1.0, True), ("t", 0, -2.0, False)]
    wins = set_parity([r[0] for r in rows], [(r[1], r[2]) for r in rows], 0.0, [r[3] for r in rows])
    assert wins == (1, 2), wins

    # Logistic regression against the closed form of a 2x2 table: with a 0/1 feature, the bias is the
    # log-odds at x = 0 and the weight the log odds ratio. x=0: 30 of 40 real -> log(10/30). x=1: 45 of 60 generated -> log(45/15).
    xs = [(0.0,)] * 40 + [(1.0,)] * 60
    ys = [0] * 30 + [1] * 10 + [0] * 15 + [1] * 45
    (w, b), converged = fit_logistic(xs, ys)
    assert converged and close(b, math.log(10 / 30), 1e-8) and close(w, math.log(45 / 15) - math.log(10 / 30), 1e-8), (w, b)
    # Two features: at the optimum the gradient vanishes, i.e. residuals are orthogonal to each feature and sum to zero.
    xs = [(-2.0, 0.5), (-1.0, -1.5), (-0.5, 1.0), (0.0, -0.5), (0.5, 0.2), (1.0, -1.0), (1.5, 2.0), (2.0, 0.1), (-1.5, 2.5), (0.7, -2.0)]
    ys = [0, 0, 1, 0, 1, 0, 1, 1, 0, 1]
    weights, converged = fit_logistic(xs, ys)
    residuals = [sigmoid(weights[0] * x[0] + weights[1] * x[1] + weights[2]) - y for x, y in zip(xs, ys)]
    assert converged and all(close(sum(r * v for r, v in zip(residuals, column)), 0.0, 1e-7) for column in ([x[0] for x in xs], [x[1] for x in xs], [1.0] * len(xs)))
    assert not fit_logistic([(-1.0,), (1.0,)], [0, 1], iterations=20)[1], "separable data must be reported, not hidden"
    assert solve([[2.0, 1.0], [1.0, 3.0]], [3.0, 5.0]) == [0.8, 1.4]

    # AUROC with a tie: positives {3, 2}, negatives {2, 1}: pairs (3>2, 3>1, 2=2, 2>1) = 3.5 of 4.
    assert auroc([3, 2], [2, 1]) == 0.875

    # Folds keep a parent's derivatives together and do not depend on the order of the items.
    assert fold_of("parent-17", 5) == fold_of("parent-17", 5) and len({fold_of(f"p{i}", 5) for i in range(200)}) == 5
    pair = [dict(item(0.0, 0.0), groups={"parent": "p9", "transform": t}) for t in ("original", "jpeg")]
    assert len({fold_of(group_value(it, "parent"), 5) for it in pair}) == 1
    lone = item(0.0, 0.0)
    assert group_value(lone, "parent") == "path:" + lone["key"] and group_value(lone, "transform") == "-"
    assert group_value(dict(lone, label=0, generator="real:coco"), "source") == "real:coco" and group_value(lone, "source") == "s"

    with tempfile.TemporaryDirectory(prefix="compare-policies-") as scratch:
        # Score files: the three ways to read a logit, and the join with a grouping manifest.
        first = os.path.join(scratch, "first.tsv")
        with open(first, "w", encoding="utf-8") as f:
            f.write("img/a.jpg\t1\tflux\tdemo\t0.880797\t2.0\t4.0\n")
            f.write("img/b.jpg\t0\treal:x\tdemo\t0.119203\t-2.0\n")
            f.write("short line\n")
        spec = os.path.join(scratch, "spec.json")
        with open(spec, "w", encoding="utf-8") as f:
            json.dump({"logit_scale": 0.5, "logit_bias": -1.0, "strict": {"logit_scale": 2.0, "logit_bias": 0.25}}, f)
        prob = read_score_file(first, "prob")
        assert close(prob["img/a.jpg"][3], 2.0, 1e-5) and close(prob["img/b.jpg"][3], -2.0, 1e-5) and len(prob) == 2
        assert read_score_file(first, "logits")["img/a.jpg"][3] == 3.0
        assert read_score_file(first, f"spec={spec}")["img/a.jpg"][3] == 0.5
        assert read_score_file(first, f"spec={spec}#strict")["img/b.jpg"][3] == -3.75
        assert parse_scores_arg("probe=runs/a@b/x.tsv@logits") == ("probe", "runs/a@b/x.tsv", "logits")
        assert parse_scores_arg("tintype=x.tsv") == ("tintype", "x.tsv", "prob")
        assert parse_scores_arg(f"t=x.tsv@spec={spec}#strict") == ("t", "x.tsv", f"spec={spec}#strict")
        second = os.path.join(scratch, "second.tsv")
        with open(second, "w", encoding="utf-8") as f:
            f.write("img/a.jpg\t1\tflux\tdemo\t0.5\t0.0\nimg/c.jpg\t1\tflux\tdemo\t0.5\t0.0\n")
        joined, dropped = load_score_files([f"a={first}@logits", f"b={second}@logits"])
        assert [it["key"] for it in joined] == ["img/a.jpg"] and dropped == {"a": 1, "b": 1} and joined[0]["views"]["a"] == [2.0, 4.0]
        manifest = os.path.join(scratch, "manifest.tsv")
        with open(manifest, "w", encoding="utf-8") as f:
            f.write("img/a.jpg\t1\tflux\tdemo\tp1\t-\tsrc\t\tjpeg-q70\n/abs/b.jpg\t0\treal:x\tdemo\n")
        groups = read_manifest(manifest)
        assert groups[os.path.join(scratch, "img/a.jpg")] == {"parent": "p1", "source": "src", "transform": "jpeg-q70"} and groups["/abs/b.jpg"] == {}
        # A manifest written from loaded photos reads back with the same groups.
        photos = [dict(new_item("/x/a.jpg", 1, "flux", "demo"), groups={"parent": "p1", "transform": "jpeg"}), new_item("/x/b.jpg", 0, "real:x", "demo")]
        write_manifest(manifest, photos)
        assert Path(manifest).read_text() == "/x/a.jpg\t1\tflux\tdemo\tp1\t-\t-\t-\tjpeg\n/x/b.jpg\t0\treal:x\tdemo\n"
        assert read_manifest(manifest) == {"/x/a.jpg": {"parent": "p1", "transform": "jpeg"}, "/x/b.jpg": {}}
        # A frozen record names its files as archived score files do: paths as written, groups after them.
        record = os.path.join(scratch, "set.frozen.tsv")
        with open(record, "w", encoding="utf-8") as f:
            f.write("# tintype frozen evaluation set, format 2\n# manifest\tdata/x.tsv\tabc\n# sha256\tsize\tlabel\tgenerator\tdataset\tpath\tparent_id\n")
            f.write("00\t12\t0\treal:x\tdemo\tdata/a.jpg\tp1\t\tsrc\t\t\n11\t34\t1\tflux\tdemo\tdata/b.png\tp2\t\t\tflux\tjpeg-qv4\n")
        assert read_manifest(record) == {"data/a.jpg": {"parent": "p1", "source": "src"}, "data/b.png": {"parent": "p2", "family": "flux", "transform": "jpeg-qv4"}}

    # A whole comparison on a toy split: weights from the fit set, cutoffs from the calibration set, counts on the test set.
    def toy(seed, split):
        out = []
        for i in range(60):
            label = i % 2
            u = ((i * 37 + seed * 11) % 23) / 23.0 - 0.5
            v = ((i * 53 + seed * 7) % 19) / 19.0 - 0.5
            z1 = (1.2 if label else -1.2) + 4.0 * u
            z2 = (1.0 if label else -1.0) + 4.0 * v
            it = item(z1, z2, label, "s1" if i % 3 else "s2")
            it["key"] = f"{split}/{i}"
            it["set"] = split
            it["groups"] = {"parent": f"{split}{i // 2}"}
            out.append(it)
        return out

    items = toy(1, "fit") + toy(2, "calibrate") + toy(3, "test")
    opts = default_opts()
    opts.update(first="a", cf="b", seconds=["b"], reference=("b", 0.0), fprs=[0.1], budgets=["3"])
    text = io.StringIO()
    policies, scores = compare(items, opts, text)
    names = [p.name for p in policies]
    assert names[0] == "a" and "a + b, fitted full fusion" in names and "a + b, 50:50 for |a logit| <= 2" in names, names
    report = text.getvalue()
    assert "Fusion weights fitted on the fit set (60 photos) only" in report and "Cutoffs are frozen on the `calibrate` set" in report
    assert "## Test set: 60 photos, 30 generated, 30 authentic" in report
    # The frozen cutoff of the first model is the fourth-highest authentic calibration score (floor(0.1 * 30) = 3 allowed).
    calibrate = sorted((it["z"]["a"] for it in items if it["set"] == "calibrate" and it["label"] == 0), reverse=True)
    test = [(it["label"], it["z"]["a"]) for it in items if it["set"] == "test"]
    tp, fp = operate(test, calibrate[3])
    assert f"| a | {calibrate[3]:.5f} | {fp}/30 ({pct(fp, 30)}) | {cell(tp, 30)} |" in report, report
    # The fitted full fusion scores the test photos with the weights of the fit set alone.
    fitted = next(p for p in policies if p.name == "a + b, fitted full fusion")
    weights, _ = fit_logistic([(it["z"]["a"], it["z"]["b"]) for it in items if it["set"] == "fit"], [it["label"] for it in items if it["set"] == "fit"])
    k = next(i for i, it in enumerate(items) if it["set"] == "test")
    assert close(scores[fitted][k], weights[0] * items[k]["z"]["a"] + weights[1] * items[k]["z"]["b"] + weights[2])
    # Reporting on the calibration set instead: its own cutoffs, said to be in-sample, and no test photo in the tables.
    text = io.StringIO()
    compare(items, dict(opts, report_on="calibrate", worst_slices=True, slice_parity=("b", 0.0), min_slice=1), text)
    chosen = text.getvalue()
    assert "## Calibration set: 60 photos" in chosen and "these cutoffs are in-sample" in chosen and "## Test set" not in chosen
    tp, fp = operate([(it["label"], it["z"]["a"]) for it in items if it["set"] == "calibrate"], calibrate[3])
    assert fp == 3 and f"| a | {calibrate[3]:.5f} | 3/30 (10.00%) | {cell(tp, 30)} |" in chosen, chosen
    assert "Worst slices at the frozen cutoffs" in chosen and "Slice by slice against b" in chosen
    # Without a split or folds nothing is fitted.
    for it in items:
        del it["set"]
    text = io.StringIO()
    policies, _ = compare(items, opts, text)
    assert not any(p.fitted for p in policies) and "Fitted fusion is left out" in text.getvalue()
    # Cross-fitting: a photo's fused score comes from weights fitted without its parent's fold.
    opts["cross_fit"] = 3
    text = io.StringIO()
    policies, scores = compare(items, opts, text)
    fitted = next(p for p in policies if p.name == "a + b, fitted full fusion")
    fold = [fold_of(group_value(it, "parent"), 3) for it in items]
    others = [it for it, f in zip(items, fold) if f != fold[0]]
    weights, _ = fit_logistic([(it["z"]["a"], it["z"]["b"]) for it in others], [it["label"] for it in others])
    assert close(scores[fitted][0], weights[0] * items[0]["z"]["a"] + weights[1] * items[0]["z"]["b"] + weights[2])
    assert "3 folds of whole `parent` groups (90 groups)" in text.getvalue()

    # Slices. Families a (3 generated) and b (2); sources s (3 authentic) and t (2). The policy flags 2 of a, 0 of b,
    # 1 of s and 0 of t; the reference flags 1 of a, 1 of b, 1 of s and 1 of t.
    def photo(label, family, source, transform, flagged, ref):
        it = item(1.0 if flagged else -1.0, 1.0 if ref else -1.0, label)
        it["groups"] = {"family": family, "source": source, "transform": transform}
        return it
    photos = ([photo(1, "a", "x", "-", f, r) for f, r in ((1, 1), (1, 0), (0, 0))] + [photo(1, "b", "x", "jpeg", f, r) for f, r in ((0, 1), (0, 0))]
              + [photo(0, "real", "s", "-", f, r) for f, r in ((1, 0), (0, 1), (0, 0))] + [photo(0, "real", "t", "jpeg", f, r) for f, r in ((0, 1), (0, 0))])
    flags = [it["z"]["a"] > 0 for it in photos]
    reference = [it["z"]["b"] > 0 for it in photos]
    assert slice_counts(photos, flags, "family", 1) == {"a": [3, 2], "b": [2, 0]} and slice_counts(photos, flags, "source", 0) == {"s": [3, 1], "t": [2, 0]}
    # Family a is reached (2 >= 1), b is not (0 < 1); both sources are (1 <= 1, 0 <= 1). With slices of 3 or more only a and s count.
    assert slices_reaching(photos, flags, reference, 1) == (1, 2, 2, 2) and slices_reaching(photos, flags, reference, 3) == (1, 1, 1, 1)
    # Worst authentic source: s with 1 of 3 flagged; weakest family: b with 0 of 2; worst transform among authentic: "-" (1 of 3).
    assert worst_slice(photos, flags, "source", 0, 1) == ("s", 1, 3) and worst_slice(photos, flags, "family", 1, 1) == ("b", 0, 2)
    assert worst_slice(photos, flags, "transform", 0, 1) == ("-", 1, 3) and worst_slice(photos, flags, "source", 0, 4) is None
    assert [group_value(it, "processing") for it in (photos[0], photos[3])] == ["original", "processed"] and group_value(item(0.0, 0.0), "processing") == "original"

    # A stored two-stage run against the band computed from the two scores: one photo inside the band (-7, 0.5) whose
    # stored score is on the other side of the cutoff, one identical, one outside the band.
    trio = [dict(item(-1.0, 4.0), z={"a": -1.0, "b": 4.0, "run": -2.0}), dict(item(0.0, 2.0), z={"a": 0.0, "b": 2.0, "run": 1.0}), dict(item(3.0, 0.0), z={"a": 3.0, "b": 0.0, "run": 3.0})]
    trio_opts = dict(default_opts(), first="a", cf="b", stored=["run"], bands=[("Shipped", (-7.0, 0.5, 0.5, 0.0))], sweep=[])
    text = io.StringIO()
    compare(trio, trio_opts, text)
    assert "| run | Shipped CF blend: band (-7, 0.5), weight 0.5, threshold 0 | 1/3 | 2 | 2 |" in text.getvalue(), text.getvalue()

    # The gate table on five photos: misses at -0.5 (inside |z| <= 2) and -3 (outside, second model flags it),
    # one false alarm at 2.5 (outside, second model clears it).
    five = [item(-0.5, -1.0, 1), item(-3.0, 2.0, 1), item(4.0, 1.0, 1), item(2.5, -1.0, 0), item(-4.0, -1.0, 0)]
    text = io.StringIO()
    gate_report(five, [Policy("m", "a", "b", (-2.0, 2.0, True))], dict(default_opts(), sweep=[]), text)
    assert "a misses 2 of 3 generated photos (b flags 1 of them) and flags 1 of 2 authentic photos (b clears 1)" in text.getvalue()
    assert "| |a logit| <= 2 | 1/5 (20.00%) | 1 | 1 | 1 | 1 | 1 |" in text.getvalue(), text.getvalue()
    print("self-test passed")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true", help="check the metric functions against hand-computed cases")
    parser.add_argument("--check-replay", action="store_true", help="show that this script reproduces scripts/replay-research.py line for line")
    parser.add_argument("--archive", choices=sorted(ARCHIVES), help="a score archive of docs/results (exploratory)")
    parser.add_argument("--table", type=Path, help="a wide table: `path label generator dataset` and a probability column per system")
    parser.add_argument("--scores", action="append", default=[], metavar="NAME=FILE[@MODE]",
                        help="a `tintype score` file of one system (repeatable; globs allowed). MODE: prob (default), logits, spec=SPEC.json[#profile]")
    parser.add_argument("--groups", action="append", default=[], type=Path, help="manifest whose columns 5 and up group the scored files (repeatable)")
    parser.add_argument("--set", action="append", default=[], metavar="NAME=MANIFEST", help="the fit, calibrate or test manifest of a split (repeatable)")
    parser.add_argument("--first", help="first-stage system (default tintype)")
    parser.add_argument("--profile", action="append", help="another single-model profile to retune (repeatable)")
    parser.add_argument("--stored", action="append", help="a column that already holds a fused policy (repeatable)")
    parser.add_argument("--cf", help="system behind the Community Forensics band (default commfor when present; `none` to leave out)")
    parser.add_argument("--cf-band", action="append", type=parse_band, metavar="LO,HI,WEIGHT,THRESHOLD", help="a band to compare (default: the shipped band and the same with lo = -3)")
    parser.add_argument("--second", action="append", help="a second model to gate by margin and to fuse (repeatable; default gpttrace when present)")
    parser.add_argument("--margin", type=floats, help="margins m of the gate abs(first logit) <= m (default 2)")
    parser.add_argument("--reference", metavar="NAME[@CUTOFF]", help="reference detector for the per-set comparison, cutoff in probability (default commfor@0.5 when present)")
    parser.add_argument("--fpr", type=floats, help="empirical false-positive rates (default 0.001,0.01)")
    parser.add_argument("--fp-budget", help="false-positive counts, comma-separated; `own:NAME` is the count of column NAME at 0.5")
    parser.add_argument("--cohort-by", choices=["all", "dataset", "generator", "source", "family", "transform", "processing"],
                        help="report each of these cohorts as well; `processing` is originals against processed copies (any transform)")
    parser.add_argument("--report-on", choices=["fit", "calibrate", "test"], default="test",
                        help="the set that is reported (default test). `calibrate` is the view for choosing a policy before the test set is looked at")
    parser.add_argument("--no-gate-tables", action="store_true", help="leave out the tables of first-stage errors per gate")
    parser.add_argument("--worst-slices", action="store_true", help="with frozen cutoffs: the worst authentic source and transform and the weakest generator family per policy")
    parser.add_argument("--slice-parity", metavar="NAME[@CUTOFF]", help="count the families and sources on which each policy reaches this detector (for splits whose sets hold one class)")
    parser.add_argument("--min-slice", type=int, default=100, help="smallest slice that counts for --worst-slices and --slice-parity (default 100)")
    parser.add_argument("--cross-fit", type=int, metavar="K", help="fit the fusion on K-1 folds of whole groups and score the fold left out (exploratory)")
    parser.add_argument("--group-by", choices=["parent", "scene", "source", "family", "dataset", "path"], help="what a fold keeps together (default parent)")
    parser.add_argument("--fit-cohort", help="with --cross-fit: fit only on this cohort of the other folds")
    parser.add_argument("--gate-sweep", type=floats, help="extra margins for the gate table (default 1,2,3,4,6,8)")
    parser.add_argument("--view-spread", metavar="NAME", help="analyse the spread of this system's view logits as a routing feature (needs --all-views score files)")
    parser.add_argument("--slice-by", choices=["dataset", "generator", "source", "family", "transform"], help="break the misses outside the first gate down by this")
    parser.add_argument("--write-scores", metavar="DIR", help="write each policy's scores as a `tintype report` score file")
    parser.add_argument("--write-manifest", metavar="FILE", help="write the loaded photos as a manifest with their grouping columns (for `tintype split-groups`) and stop")
    args = parser.parse_args()

    if args.self_test or args.check_replay:
        status = 0
        if args.self_test:
            status |= self_test()
        if args.check_replay:
            status |= check_replay()
        return status
    if sum(bool(x) for x in (args.archive, args.table, args.scores)) != 1:
        parser.error("give one of --archive, --table or --scores (or --self-test, --check-replay)")

    out = sys.stdout
    if args.archive:
        loader = ARCHIVES[args.archive][0]
        items = loader()
        opts = archive_opts(args.archive, items)
        out.write(f"# Policy comparison on the {args.archive} archive (exploratory)\n\n")
        out.write("These archived suites were used to develop the detector. Cutoffs are chosen on the cohort they are reported on: "
                  "a ranking comparison, not a transfer result and not a decision about what ships. "
                  "Routed shares are measured workload; end-to-end savings depend on the cohort and the hardware.\n")
        if args.archive == "arena":
            out.write(f"\n{load_arena()[1]} repeated authentic originals are removed from each variant.\n")
    else:
        if args.table:
            items, dropped = load_table(args.table), {}
        else:
            items, dropped = load_score_files(args.scores)
        if not items:
            parser.error("no photo is scored by every system")
        opts = default_opts()
        systems = set(items[0]["z"])
        if "commfor" in systems:
            opts.update(cf="commfor", reference=("commfor", 0.0))
        if "gpttrace" in systems:
            opts["seconds"] = ["gpttrace"]
        out.write("# Policy comparison\n\n")
        out.write(f"{len(items)} photos scored by every system ({', '.join(sorted(systems))}).")
        out.write("".join(f" {n} photos of {name} have no partner and are left out." for name, n in dropped.items()) + "\n")

    if args.write_manifest:
        write_manifest(args.write_manifest, items)
        out.write(f"\nWrote {len(items)} manifest rows to {args.write_manifest}.\n")
        return 0
    if args.groups:
        groups = {}
        for manifest in args.groups:
            groups.update(read_manifest(manifest))
        matched = 0
        for it in items:
            if it["key"] in groups:
                it["groups"].update(groups[it["key"]])
                matched += 1
        out.write(f"\nGrouping columns found for {matched} of {len(items)} photos.\n")
    if args.set:
        members = {}
        for text in args.set:
            name, _, manifest = text.partition("=")
            if name not in ("fit", "calibrate", "test") or not manifest:
                parser.error("--set takes fit=MANIFEST, calibrate=MANIFEST or test=MANIFEST")
            for key, groups in read_manifest(manifest).items():
                if members.setdefault(key, (name, groups))[0] != name:
                    parser.error(f"`{key}` is in the {members[key][0]} set and in the {name} set")
        kept = []
        for it in items:
            if it["key"] in members:
                it["set"] = members[it["key"]][0]
                it["groups"].update(members[it["key"]][1])
                kept.append(it)
        counts = Counter(it["set"] for it in kept)
        out.write(f"\nSplit: {counts.get('fit', 0)} fit, {counts.get('calibrate', 0)} calibrate, {counts.get('test', 0)} test photos; "
                  f"{len(items) - len(kept)} scored photos are in no set and are left out.\n")
        items = kept
        if not counts.get(args.report_on):
            parser.error(f"no scored photo is in the {args.report_on} set: are the manifest paths those the files were scored with?")

    if args.first:
        opts["first"] = args.first
    if args.profile is not None:
        opts["profiles"] = args.profile
    if args.stored is not None:
        opts["stored"] = args.stored
    if args.cf:
        opts["cf"] = None if args.cf == "none" else args.cf
    if args.cf_band:
        opts["bands"] = [(f"Band {k + 1}", band) for k, band in enumerate(args.cf_band)]
    if args.second is not None:
        opts["seconds"] = [s for s in args.second if s != "none"]
    if args.margin:
        opts["margins"] = args.margin
    if args.reference:
        name, _, cutoff = args.reference.partition("@")
        opts["reference"] = None if name == "none" else (name, replay.logit(cutoff or "0.5"))
    if args.fpr:
        opts["fprs"] = args.fpr
    if args.fp_budget is not None:
        opts["budgets"] = [t for t in args.fp_budget.split(",") if t]
    if args.cohort_by:
        opts["cohort_by"] = args.cohort_by
    if args.cross_fit:
        opts["cross_fit"] = args.cross_fit
    if args.group_by:
        opts["group_by"] = args.group_by
    opts["fit_cohort"] = args.fit_cohort
    if args.gate_sweep is not None:
        opts["sweep"] = args.gate_sweep
    opts["view_spread"] = args.view_spread
    opts["slice_by"] = args.slice_by
    opts["write_scores"] = args.write_scores
    opts["report_on"] = args.report_on
    opts["gates"] = not args.no_gate_tables
    opts["worst_slices"] = args.worst_slices
    opts["min_slice"] = args.min_slice
    if args.slice_parity:
        name, _, cutoff = args.slice_parity.partition("@")
        opts["slice_parity"] = (name, replay.logit(cutoff or "0.5"))
    if any(not 0 <= f < 1 for f in opts["fprs"]):
        parser.error("--fpr takes fractions, e.g. 0.001,0.01")
    if opts["cross_fit"] == 1 or opts["cross_fit"] < 0:
        parser.error("--cross-fit needs two or more folds")
    try:
        compare(items, opts, out)
    except ValueError as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    sys.exit(main())
