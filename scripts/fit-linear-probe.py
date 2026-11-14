#!/usr/bin/env python3
"""Fit and apply a linear probe on features of a frozen image encoder.

The encoder (PE, DINOv3, MetaCLIP 2, ...) runs offline, outside this
repository, and writes one feature vector per photo. This script does the
rest with the standard library only, so the probe that is compared is
exactly reproducible: L2-regularised logistic regression on standardised
features, fitted by L-BFGS.

    fit    --features FIT.npy --manifest fit.tsv --out probe.json
           [--l2 1.0 | --l2-grid 0.1,1,10 --folds 5] [--encoder TEXT]
    score  --probe probe.json --features TEST.npy --manifest test.tsv --out scores.tsv
    --self-test

Inputs:

- a feature file in NumPy `.npy` format: a C-ordered 2-D array of
  little-endian float32, float16 or float64, one row per photo;
- the manifest those rows were extracted from, in the same order:
  `path label generator dataset [parent_id ...]` (tab-separated, as
  `tintype` reads it; relative paths are resolved against the manifest's
  folder, so the score file lines up with `tintype score` on that manifest).

Fit on the `fit` set of the frozen split only. `--l2-grid` chooses the
penalty by cross-validated log loss inside that set, with every `parent_id`
(column 5) kept in one fold; the calibration and test sets are only ever
scored.

`score` writes `path label generator dataset prob logit`, the layout of
`tintype score` with one logit column; `scripts/compare-policies.py` reads it
with `--scores probe=scores.tsv@logits`.

The objective is `mean log loss + l2 / (2 N) * |w|^2` on standardised
features (the bias is not penalised): `l2` is the `1 / C` of the usual
sum-of-losses form. Pure Python, so not fast: one pass over 2,000 rows of
1,024 features took 0.34 s on an Apple M-series laptop (about 8 s per
50,000 rows), and a fit takes some tens of passes.
"""

import argparse
import array
import ast
import hashlib
import importlib.util
import json
import math
import os
import struct
import sys
import tempfile
from operator import mul
from pathlib import Path

FORMAT = "tintype-linear-probe-1"
TYPES = {"<f4": ("f", 4), "<f8": ("d", 8), "<f2": ("e", 2), "|f4": ("f", 4), "=f4": ("f", 4)}


# ---------------------------------------------------------------- files

def read_npy(path):
    """Rows of a 2-D `.npy` array as `array('f')` (float32), without NumPy."""
    with open(path, "rb") as stream:
        if stream.read(6) != b"\x93NUMPY":
            raise ValueError(f"{path} is not a .npy file")
        major, _minor = stream.read(2)
        (length,) = struct.unpack("<H", stream.read(2)) if major == 1 else struct.unpack("<I", stream.read(4))
        header = ast.literal_eval(stream.read(length).decode("latin1"))
        shape, kind = header["shape"], header["descr"]
        if header.get("fortran_order") or len(shape) != 2 or kind not in TYPES:
            raise ValueError(f"{path}: need a C-ordered 2-D float32, float16 or float64 array, found {kind} {shape}")
        if sys.byteorder != "little":
            raise ValueError("this reader assumes a little-endian machine")
        code, size = TYPES[kind]
        n, d = shape
        rows = []
        for _ in range(n):
            chunk = stream.read(d * size)
            if len(chunk) != d * size:
                raise ValueError(f"{path} ends before row {len(rows) + 1} of {n}")
            if code == "f":
                row = array.array("f")
                row.frombytes(chunk)
            else:
                row = array.array("f", struct.unpack(f"<{d}{code}", chunk))
            rows.append(row)
        if stream.read(1):
            raise ValueError(f"{path} holds more data than its header says")
    return rows


def write_npy(path, rows):
    """Write rows of floats as a float32 `.npy` (version 1.0): what the extractor's `numpy.save` produces."""
    d = len(rows[0])
    header = "{'descr': '<f4', 'fortran_order': False, 'shape': (%d, %d), }" % (len(rows), d)
    header += " " * ((64 - (10 + len(header) + 1) % 64) % 64) + "\n"
    with open(path, "wb") as stream:
        stream.write(b"\x93NUMPY\x01\x00" + struct.pack("<H", len(header)) + header.encode("latin1"))
        for row in rows:
            stream.write(array.array("f", row).tobytes())


def read_manifest(path):
    """`(path, label, generator, dataset, parent or None)` per row, paths resolved as `tintype` resolves them."""
    root = os.path.dirname(str(path))
    rows = []
    with open(path, encoding="utf-8", newline="") as stream:
        for line in stream:
            line = line.rstrip("\n")
            if not line:
                continue
            f = line.split("\t")
            if len(f) < 4 or f[1] not in ("0", "1"):
                raise ValueError(f"bad manifest line in {path}: {line}")
            parent = f[4] if len(f) > 4 and f[4] not in ("", "-") else None
            rows.append((os.path.join(root, f[0]), int(f[1]), f[2], f[3], parent))
    return rows


def sha256_of(path):
    digest = hashlib.sha256()
    with open(path, "rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


# ---------------------------------------------------------------- fitting

def sigmoid(z):
    return 1.0 / (1.0 + math.exp(-z)) if z >= 0 else math.exp(z) / (1.0 + math.exp(z))


def moments(rows):
    """Per-feature mean and standard deviation (population), the latter floored so a constant feature is harmless."""
    n, d = len(rows), len(rows[0])
    total, squares = [0.0] * d, [0.0] * d
    for row in rows:
        total = [t + v for t, v in zip(total, row)]
        squares = [s + v * v for s, v in zip(squares, row)]
    mean = [t / n for t in total]
    std = [max(math.sqrt(max(s / n - m * m, 0.0)), 1e-6) for s, m in zip(squares, mean)]
    return mean, std


def standardise(rows, mean, std):
    inverse = [1.0 / s for s in std]
    return [array.array("d", [(v - m) * i for v, m, i in zip(row, mean, inverse)]) for row in rows]


def objective(weights, rows, labels, l2):
    """`(loss, gradient)` of `mean log loss + l2 / (2 N) * |w|^2`; the last weight is the bias."""
    n, d = len(rows), len(weights) - 1
    w, bias = weights[:d], weights[d]
    loss, grad_bias = 0.0, 0.0
    grad = [0.0] * d
    for row, y in zip(rows, labels):
        z = bias + sum(map(mul, w, row))
        loss += max(z, 0.0) + math.log1p(math.exp(-abs(z))) - y * z
        r = sigmoid(z) - y
        grad_bias += r
        grad = [g + r * v for g, v in zip(grad, row)]
    penalty = l2 / n
    loss = loss / n + 0.5 * penalty * sum(v * v for v in w)
    return loss, [g / n + penalty * v for g, v in zip(grad, w)] + [grad_bias / n]


def dot(a, b):
    return sum(map(mul, a, b))


def lbfgs(function, start, max_iterations=300, tolerance=1e-6, memory=10):
    """Minimise a smooth convex `function(x) -> (value, gradient)` by L-BFGS with a backtracking line search.

    Stops when the largest gradient component is below `tolerance`. Returns `(x, value, iterations, converged)`."""
    x = list(start)
    value, gradient = function(x)
    history = []
    for iteration in range(1, max_iterations + 1):
        if max(abs(g) for g in gradient) < tolerance:
            return x, value, iteration - 1, True
        direction = [-g for g in gradient]
        alphas = []
        for s, y, rho in reversed(history):
            alpha = rho * dot(s, direction)
            alphas.append(alpha)
            direction = [q - alpha * v for q, v in zip(direction, y)]
        if history:
            s, y, _ = history[-1]
            gamma = dot(s, y) / dot(y, y)
            direction = [gamma * q for q in direction]
        for (s, y, rho), alpha in zip(history, reversed(alphas)):
            beta = rho * dot(y, direction)
            direction = [q + (alpha - beta) * v for q, v in zip(direction, s)]
        slope = dot(gradient, direction)
        if slope >= 0:
            # Not a descent direction (numerical trouble): fall back to the gradient.
            history, direction = [], [-g for g in gradient]
            slope = dot(gradient, direction)
        step = 1.0 if history else min(1.0, 1.0 / max(math.sqrt(dot(gradient, gradient)), 1e-12))
        while True:
            candidate = [v + step * q for v, q in zip(x, direction)]
            new_value, new_gradient = function(candidate)
            if new_value <= value + 1e-4 * step * slope or step < 1e-12:
                break
            step /= 2.0
        s = [a - b for a, b in zip(candidate, x)]
        y = [a - b for a, b in zip(new_gradient, gradient)]
        curvature = dot(s, y)
        if curvature > 1e-12:
            history.append((s, y, 1.0 / curvature))
            history = history[-memory:]
        stalled = value - new_value <= 1e-12 * max(1.0, abs(value))
        x, value, gradient = candidate, new_value, new_gradient
        if stalled:
            return x, value, iteration, max(abs(g) for g in gradient) < 100 * tolerance
    return x, value, max_iterations, False


def fit(rows, labels, l2, max_iterations=300, tolerance=1e-6):
    """Standardise, fit, and return the probe as plain lists (the weights apply to standardised features)."""
    mean, std = moments(rows)
    data = standardise(rows, mean, std)
    weights, loss, iterations, converged = lbfgs(lambda w: objective(w, data, labels, l2), [0.0] * (len(mean) + 1), max_iterations, tolerance)
    return {"mean": mean, "std": std, "weights": weights[:-1], "bias": weights[-1], "l2": l2, "loss": loss, "iterations": iterations, "converged": converged}


def logit_of(probe, row):
    return probe["bias"] + sum(w * (v - m) / s for w, v, m, s in zip(probe["weights"], row, probe["mean"], probe["std"]))


def fold_of(key, folds):
    return int(hashlib.sha256(key.encode("utf-8")).hexdigest()[:12], 16) % folds


def choose_l2(rows, labels, groups, grid, folds, max_iterations, log):
    """The penalty with the lowest held-out log loss over `folds` folds of whole groups (ties: the larger penalty)."""
    fold = [fold_of(g, folds) for g in groups]
    results = []
    for l2 in grid:
        total, count = 0.0, 0
        for k in range(folds):
            train = [i for i, f in enumerate(fold) if f != k]
            held = [i for i, f in enumerate(fold) if f == k]
            if not held or len({labels[i] for i in train}) < 2:
                continue
            probe = fit([rows[i] for i in train], [labels[i] for i in train], l2, max_iterations)
            for i in held:
                z = logit_of(probe, rows[i])
                total += max(z, 0.0) + math.log1p(math.exp(-abs(z))) - labels[i] * z
            count += len(held)
        if not count:
            raise ValueError("no fold has both classes to fit on and photos to hold out: use fewer folds")
        results.append((total / count, -l2))
        log(f"l2 {l2:g}: held-out log loss {total / count:.6f} over {count} photos in {folds} folds")
    return -min(results)[1]


# ---------------------------------------------------------------- commands

def load(features, manifest):
    rows, listed = read_npy(features), read_manifest(manifest)
    if len(rows) != len(listed):
        raise ValueError(f"{features} has {len(rows)} rows and {manifest} lists {len(listed)} photos")
    return rows, listed


def command_fit(args):
    rows, listed = load(args.features, args.manifest)
    labels = [r[1] for r in listed]
    if len(set(labels)) < 2:
        raise ValueError("the fit set needs both classes")
    log = lambda text: print(text, file=sys.stderr)
    l2 = args.l2
    if args.l2_grid:
        groups = [r[4] or "path:" + r[0] for r in listed]
        l2 = choose_l2(rows, labels, groups, args.l2_grid, args.folds, args.max_iterations, log)
        log(f"chosen l2 {l2:g}")
    probe = fit(rows, labels, l2, args.max_iterations)
    probe.update(format=FORMAT, encoder=args.encoder, features=len(probe["mean"]), photos=len(rows), generated=sum(labels),
                 features_sha256=sha256_of(args.features), manifest_sha256=sha256_of(args.manifest),
                 l2_grid=args.l2_grid or None, folds=args.folds if args.l2_grid else None)
    Path(args.out).write_text(json.dumps(probe) + "\n", encoding="utf-8")
    log(f"fitted on {len(rows)} photos ({sum(labels)} generated), {probe['features']} features: loss {probe['loss']:.6f} after "
        f"{probe['iterations']} iterations{'' if probe['converged'] else ' (NOT converged: raise --max-iterations)'}; wrote {args.out}")
    return 0 if probe["converged"] else 1


def command_score(args):
    probe = json.loads(Path(args.probe).read_text(encoding="utf-8"))
    if probe.get("format") != FORMAT:
        raise ValueError(f"{args.probe} is not a {FORMAT} file")
    rows, listed = load(args.features, args.manifest)
    if len(rows[0]) != probe["features"]:
        raise ValueError(f"the probe takes {probe['features']} features and {args.features} has {len(rows[0])}")
    with open(args.out, "w", encoding="utf-8", newline="") as stream:
        for row, (path, label, generator, dataset, _) in zip(rows, listed):
            z = logit_of(probe, row)
            stream.write(f"{path}\t{label}\t{generator}\t{dataset}\t{sigmoid(z):.6f}\t{z:.5f}\n")
    print(f"scored {len(rows)} photos; wrote {args.out}", file=sys.stderr)
    return 0


def self_test():
    close = lambda a, b, tol=1e-6: abs(a - b) <= tol

    # L-BFGS on a quadratic with a known minimum: (x - 3)^2 + 10 (y + 1)^2.
    x, value, _, converged = lbfgs(lambda v: ((v[0] - 3) ** 2 + 10 * (v[1] + 1) ** 2, [2 * (v[0] - 3), 20 * (v[1] + 1)]), [0.0, 0.0], tolerance=1e-9)
    assert converged and close(x[0], 3.0) and close(x[1], -1.0) and close(value, 0.0)

    # Without a penalty, a 0/1 feature has the closed form of a 2x2 table: bias = log-odds at 0, weight = log odds ratio.
    rows = [array.array("f", [0.0])] * 40 + [array.array("f", [1.0])] * 60
    labels = [0] * 30 + [1] * 10 + [0] * 15 + [1] * 45
    probe = fit(rows, labels, 0.0, tolerance=1e-9)
    assert probe["converged"]
    at = lambda v: logit_of(probe, array.array("f", [v]))
    assert close(at(0.0), math.log(10 / 30), 1e-5) and close(at(1.0), math.log(45 / 15), 1e-5), (at(0.0), at(1.0))
    assert close(probe["mean"][0], 0.6) and close(probe["std"][0], math.sqrt(0.24))

    # With a penalty, the fitted weights are a stationary point of the stated objective, checked independently.
    rows = [array.array("f", r) for r in ([-2.0, 0.5, 1.0], [-1.0, -1.5, 0.0], [-0.5, 1.0, 2.0], [0.0, -0.5, 1.5], [0.5, 0.2, -1.0],
                                          [1.0, -1.0, 0.5], [1.5, 2.0, 0.0], [2.0, 0.1, -0.5], [-1.5, 2.5, 1.0], [0.7, -2.0, 3.0])]
    labels = [0, 0, 1, 0, 1, 0, 1, 1, 0, 1]
    probe = fit(rows, labels, 0.5, tolerance=1e-9)
    assert probe["converged"]
    standard = standardise(rows, probe["mean"], probe["std"])
    residual = [sigmoid(probe["bias"] + dot(probe["weights"], r)) - y for r, y in zip(standard, labels)]
    for j in range(3):
        assert close(sum(r * row[j] for r, row in zip(residual, standard)) / 10 + 0.5 / 10 * probe["weights"][j], 0.0, 1e-7)
    assert close(sum(residual), 0.0, 1e-7)
    # ... and without one, L-BFGS agrees with the Newton solver of the policy-comparison harness.
    spec = importlib.util.spec_from_file_location("compare_policies", Path(__file__).with_name("compare-policies.py"))
    harness = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(harness)
    plain = fit(rows, labels, 0.0, tolerance=1e-9)
    newton, ok = harness.fit_logistic([tuple(r) for r in rows], labels)
    assert ok and all(close(logit_of(plain, r), newton[3] + dot(newton[:3], r), 1e-5) for r in rows)
    # A constant feature is ignored rather than dividing by zero.
    flat = fit([array.array("f", [r[0], 7.0]) for r in rows], labels, 1.0)
    assert flat["converged"] and close(flat["weights"][1], 0.0)

    assert fold_of("p1", 4) == fold_of("p1", 4) and len({fold_of(f"p{i}", 4) for i in range(100)}) == 4

    with tempfile.TemporaryDirectory(prefix="fit-linear-probe-") as scratch:
        features, manifest = os.path.join(scratch, "fit.npy"), os.path.join(scratch, "fit.tsv")
        write_npy(features, rows)
        again = read_npy(features)
        assert [list(r) for r in again] == [list(r) for r in rows]
        with open(manifest, "w", encoding="utf-8") as f:
            for i, y in enumerate(labels):
                # Two photos share each parent; one path is absolute.
                f.write(f"{'/abs/' if i == 0 else 'img/'}{i}.jpg\t{y}\t{'flux' if y else 'real:x'}\tdemo\tp{i // 2}\n")
        listed = read_manifest(manifest)
        assert listed[0][0] == "/abs/0.jpg" and listed[1] == (os.path.join(scratch, "img/1.jpg"), 0, "real:x", "demo", "p0")
        # float64 files read to the same values, float16 files to the nearest half-precision ones.
        for kind, code, tolerance in (("<f8", "d", 0.0), ("<f2", "e", 1e-3)):
            other = os.path.join(scratch, kind[1:] + ".npy")
            header = "{'descr': '%s', 'fortran_order': False, 'shape': (10, 3), }" % kind
            header += " " * ((64 - (10 + len(header) + 1) % 64) % 64) + "\n"
            with open(other, "wb") as f:
                f.write(b"\x93NUMPY\x01\x00" + struct.pack("<H", len(header)) + header.encode("latin1"))
                for r in rows:
                    f.write(struct.pack(f"<3{code}", *r))
            assert all(abs(a - b) <= tolerance for got, want in zip(read_npy(other), rows) for a, b in zip(got, want))
            assert list(read_npy(other)[0][:2]) == [-2.0, 0.5]

        out, scores = os.path.join(scratch, "probe.json"), os.path.join(scratch, "scores.tsv")
        quiet = lambda *a, **k: None
        chosen = choose_l2(rows, labels, [r[4] for r in listed], [0.1, 10.0], 2, 300, quiet)
        assert chosen in (0.1, 10.0)
        fit_args = argparse.Namespace(features=features, manifest=manifest, out=out, l2=0.5, l2_grid=None, folds=5, max_iterations=300, encoder="self-test")
        stderr, sys.stderr = sys.stderr, open(os.devnull, "w")
        try:
            assert command_fit(fit_args) == 0
            assert command_score(argparse.Namespace(probe=out, features=features, manifest=manifest, out=scores)) == 0
        finally:
            sys.stderr.close()
            sys.stderr = stderr
        saved = json.loads(Path(out).read_text())
        assert saved["format"] == FORMAT and saved["photos"] == 10 and saved["features"] == 3 and saved["features_sha256"] == sha256_of(features)
        lines = Path(scores).read_text().splitlines()
        first = lines[1].split("\t")
        assert len(lines) == 10 and first[:4] == [os.path.join(scratch, "img/1.jpg"), "0", "real:x", "demo"]
        assert close(float(first[5]), logit_of(probe, rows[1]), 1e-4) and close(float(first[4]), sigmoid(float(first[5])), 1e-5)
        # The harness reads that file as a second model.
        assert close(harness.read_score_file(scores, "logits")[first[0]][3], float(first[5]), 1e-12)
    print("self-test passed")
    return 0


def floats(text):
    return [float(x) for x in text.split(",") if x.strip()]


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true", help="check the reader and the fit against hand-computed cases")
    commands = parser.add_subparsers(dest="command")
    fit_parser = commands.add_parser("fit", help="fit a probe on the fit set")
    fit_parser.add_argument("--features", required=True, help=".npy features, one row per manifest row")
    fit_parser.add_argument("--manifest", required=True, help="the manifest the features were extracted from")
    fit_parser.add_argument("--out", required=True, help="probe JSON to write")
    fit_parser.add_argument("--l2", type=float, default=1.0, help="penalty on the standardised weights (default 1)")
    fit_parser.add_argument("--l2-grid", type=floats, help="choose the penalty among these by cross-validated log loss inside the fit set")
    fit_parser.add_argument("--folds", type=int, default=5, help="folds for --l2-grid; a parent_id stays in one fold (default 5)")
    fit_parser.add_argument("--max-iterations", type=int, default=300)
    fit_parser.add_argument("--encoder", default="", help="free text recorded in the probe: encoder, revision, preprocessing")
    score_parser = commands.add_parser("score", help="score a feature file with a fitted probe")
    score_parser.add_argument("--probe", required=True)
    score_parser.add_argument("--features", required=True)
    score_parser.add_argument("--manifest", required=True)
    score_parser.add_argument("--out", required=True, help="score file: path label generator dataset prob logit")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.command is None:
        parser.error("give a command (fit, score) or --self-test")
    try:
        return command_fit(args) if args.command == "fit" else command_score(args)
    except ValueError as error:
        parser.error(str(error))


if __name__ == "__main__":
    sys.exit(main())
