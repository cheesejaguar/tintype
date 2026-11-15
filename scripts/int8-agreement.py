#!/usr/bin/env python3
"""Compare `detect --json` output of `--device cpu-int8` with `--device cpu`.

Standard library only; run it as `python3 -I scripts/int8-agreement.py ...`
from the repository root (the binary finds `models/` there). Three steps:

  --make-cohort STORE --per-store N [--seed S] > cohort.tsv
        A labelled sample: from every `STORE/<name>/manifest.tsv` whose
        files are on disk, up to N/2 authentic and N/2 generated files,
        picked with a fixed seed. Lines are `path label generator dataset`.

  --run BIN --cohort cohort.tsv --out rows.jsonl [--device cpu-int8]
        [--extra=--all-views] [--env NAME=VALUE ...]
        Run `BIN detect --device D --json` over the cohort (a few hundred
        files per process) and keep the rows.

  --cohort cohort.tsv --fp32 fp32.jsonl --int8 LABEL=rows.jsonl [...]
        Compare each INT8 run with the FP32 run on the same files and print
        a markdown table:
        - the absolute logit difference (median, 99th percentile, largest)
          and the mean signed difference;
        - verdicts at the shipped threshold: recall and false-positive rate
          of each, and the flips in both directions, by label;
        - how many files stop at a different early-exit stage (`views`);
        - recall at 1% and 0.1% empirical false-positive rate, each run at
          its own cutoff: the cutoff is chosen on this cohort's authentic
          files and reported on the same cohort, so it is exploratory;
        - with `--tail K`, the K largest authentic logits of the FP32 run
          beside what the INT8 run gives the same files.

  --frozen --calibrate-fp32 A.tsv --calibrate-int8 B.tsv --test-fp32 C.tsv
        --test-int8 D.tsv --groups test.tsv
        The same comparison on score files written by `tintype score`
        (`path label generator dataset probability view-logits...`) with
        cutoffs frozen on a calibration set: each engine's cutoff at 1% and
        0.1% is the empirical one over the authentic rows of its own
        calibration file, and is then applied to its test file. Reports the
        authentic rows flagged, recall on clean rows (`transform` `-`, column
        9 of `--groups`), on processed rows and on each transform, the
        verdicts that differ, and the gate of
        docs/runbooks/cpu-profile-int8.md: at most one point of recall lost
        on each slice of at least `--min-slice` generated rows.

The logit compared is the JSON `logit`: the mean over the views looked at.
Run both with `--extra=--all-views` to compare the same six views of every
file; without it, early exit may stop the two at different views, which is
what the stage count is for.

`--self-test` checks the arithmetic on made-up rows.
"""
import argparse
import json
import os
import random
import subprocess
import sys


def read_cohort(path):
    """{path: label} from a manifest-style file."""
    labels = {}
    for line in open(path):
        f = line.rstrip("\n").split("\t")
        if len(f) >= 2 and f[1] in ("0", "1"):
            labels[f[0]] = int(f[1])
    return labels


def read_rows(path):
    """{path: row} for the rows that carry a score."""
    rows = {}
    for line in open(path):
        if line.strip():
            r = json.loads(line)
            if "logit" in r:
                rows[r["path"]] = r
    return rows


def quantile(sorted_values, q):
    if not sorted_values:
        return float("nan")
    pos = q * (len(sorted_values) - 1)
    lo = int(pos)
    hi = min(lo + 1, len(sorted_values) - 1)
    return sorted_values[lo] + (sorted_values[hi] - sorted_values[lo]) * (pos - lo)


def empirical_cutoff(real_scores, fpr):
    """The strictest cutoff that flags at most `fpr` of the authentic files
    (flagging strictly above it): `metrics::empirical_cutoff` in the Rust code."""
    ranked = sorted(real_scores, reverse=True)
    return ranked[min(int(fpr * len(ranked)), len(ranked) - 1)]


def rates(scores, labels, cutoff):
    """(false-positive rate, recall) flagging strictly above `cutoff`."""
    fp = sum(s > cutoff for s, l in zip(scores, labels) if l == 0)
    tp = sum(s > cutoff for s, l in zip(scores, labels) if l == 1)
    real, fake = labels.count(0), labels.count(1)
    return (fp / real if real else float("nan"), tp / fake if fake else float("nan"))


def compare(labels, fp32, int8, fprs=(0.01, 0.001)):
    """Everything the table reports for one INT8 run against the FP32 run."""
    paths = [p for p in labels if p in fp32 and p in int8]
    y = [labels[p] for p in paths]
    a = [fp32[p]["logit"] for p in paths]
    b = [int8[p]["logit"] for p in paths]
    diffs = sorted(abs(u - v) for u, v in zip(a, b))
    flag = lambda r: r["verdict"] != "likely-real"
    fa = [flag(fp32[p]) for p in paths]
    fb = [flag(int8[p]) for p in paths]
    out = {
        "files": len(paths), "authentic": y.count(0), "generated": y.count(1),
        "missing": sum(1 for p in labels if (p in fp32) != (p in int8)),
        "median_abs_diff": quantile(diffs, 0.5), "p99_abs_diff": quantile(diffs, 0.99), "max_abs_diff": diffs[-1] if diffs else float("nan"),
        "mean_signed_diff": sum(v - u for u, v in zip(a, b)) / len(paths) if paths else float("nan"),
        "generated_lost": sum(l == 1 and u and not v for l, u, v in zip(y, fa, fb)),
        "generated_gained": sum(l == 1 and v and not u for l, u, v in zip(y, fa, fb)),
        "authentic_newly_flagged": sum(l == 0 and v and not u for l, u, v in zip(y, fa, fb)),
        "authentic_no_longer_flagged": sum(l == 0 and u and not v for l, u, v in zip(y, fa, fb)),
        "stage_changes": sum(fp32[p].get("views") != int8[p].get("views") for p in paths),
        "mean_views_fp32": sum(fp32[p].get("views", 0) for p in paths) / len(paths) if paths else float("nan"),
        "mean_views_int8": sum(int8[p].get("views", 0) for p in paths) / len(paths) if paths else float("nan"),
    }
    for name, flags in (("fp32", fa), ("int8", fb)):
        out[f"shipped_recall_{name}"] = sum(f for f, l in zip(flags, y) if l == 1) / max(y.count(1), 1)
        out[f"shipped_fpr_{name}"] = sum(f for f, l in zip(flags, y) if l == 0) / max(y.count(0), 1)
    for fpr in fprs:
        for name, s in (("fp32", a), ("int8", b)):
            real = [v for v, l in zip(s, y) if l == 0]
            cutoff = empirical_cutoff(real, fpr) if real else float("nan")
            got_fpr, recall = rates(s, y, cutoff)
            out[f"recall_at_{fpr}_{name}"] = recall
            out[f"fpr_at_{fpr}_{name}"] = got_fpr
            out[f"cutoff_at_{fpr}_{name}"] = cutoff
    return out


def tail(labels, fp32, int8, k):
    """The k largest authentic FP32 logits beside the INT8 logits of the same files."""
    paths = [p for p in labels if labels[p] == 0 and p in fp32 and p in int8]
    top = sorted(paths, key=lambda p: -fp32[p]["logit"])[:k]
    return [(p, fp32[p]["logit"], int8[p]["logit"], fp32[p].get("views"), int8[p].get("views")) for p in top]


def pct(x):
    return f"{100 * x:.2f}%"


def table(labels, fp32, runs, tail_k):
    results = [(name, compare(labels, fp32, rows)) for name, rows in runs]
    lines = ["| | " + " | ".join(name for name, _ in results) + " |", "|---|" + "---:|" * len(results)]

    def row(label, cell):
        lines.append(f"| {label} | " + " | ".join(cell(r) for _, r in results) + " |")

    row("Files compared (authentic / generated)", lambda r: f"{r['files']} ({r['authentic']} / {r['generated']})")
    row("Scored by one run only", lambda r: str(r["missing"]))
    row("Absolute logit difference: median", lambda r: f"{r['median_abs_diff']:.4f}")
    row("Absolute logit difference: 99th percentile", lambda r: f"{r['p99_abs_diff']:.4f}")
    row("Absolute logit difference: largest", lambda r: f"{r['max_abs_diff']:.4f}")
    row("Mean signed difference (INT8 minus FP32)", lambda r: f"{r['mean_signed_diff']:+.4f}")
    row("Shipped threshold, recall: FP32 → INT8", lambda r: f"{pct(r['shipped_recall_fp32'])} → {pct(r['shipped_recall_int8'])}")
    row("Shipped threshold, false-positive rate: FP32 → INT8", lambda r: f"{pct(r['shipped_fpr_fp32'])} → {pct(r['shipped_fpr_int8'])}")
    row("Generated: detected by FP32, missed by INT8", lambda r: str(r["generated_lost"]))
    row("Generated: missed by FP32, detected by INT8", lambda r: str(r["generated_gained"]))
    row("Authentic: newly flagged by INT8", lambda r: str(r["authentic_newly_flagged"]))
    row("Authentic: flagged by FP32 only", lambda r: str(r["authentic_no_longer_flagged"]))
    row("Files stopping at a different stage", lambda r: str(r["stage_changes"]))
    row("Views per file: FP32 → INT8", lambda r: f"{r['mean_views_fp32']:.3f} → {r['mean_views_int8']:.3f}")
    for fpr in (0.01, 0.001):
        row(f"Recall at {100 * fpr:g}% empirical FPR (own cutoff, this cohort): FP32 → INT8", lambda r: f"{pct(r[f'recall_at_{fpr}_fp32'])} → {pct(r[f'recall_at_{fpr}_int8'])} ({100 * (r[f'recall_at_{fpr}_int8'] - r[f'recall_at_{fpr}_fp32']):+.2f} points)")
        row(f"Cutoff logit at {100 * fpr:g}% FPR: FP32 → INT8", lambda r: f"{r[f'cutoff_at_{fpr}_fp32']:.3f} → {r[f'cutoff_at_{fpr}_int8']:.3f}")
    out = "\n".join(lines)
    if tail_k:
        for name, rows in runs:
            out += f"\n\nLargest authentic logits, FP32 against {name} (views looked at in brackets):\n\n| File | FP32 | {name} |\n|---|---:|---:|\n"
            out += "\n".join(f"| {os.path.join(os.path.basename(os.path.dirname(p)), os.path.basename(p))} | {x:.3f} ({va}) | {z:.3f} ({vb}) |" for p, x, z, va, vb in tail(labels, fp32, rows, tail_k))
    return out


def read_scores(path):
    """{path: (label, probability, mean view logit, views)} from a `tintype score` file."""
    rows = {}
    for line in open(path):
        f = line.rstrip("\n").split("\t")
        if len(f) >= 6 and f[1] in ("0", "1"):
            logits = [float(x) for x in f[5:] if x]
            rows[f[0]] = (int(f[1]), float(f[4]), sum(logits) / len(logits), len(logits))
    return rows


def frozen(cal_a, cal_b, test_a, test_b, transform, fprs=(0.01, 0.001), min_slice=100):
    """Each engine at cutoffs frozen on its own calibration scores: a dict per false-positive rate, and whether the gate holds."""
    paths = [p for p in test_a if p in test_b]
    slices = sorted({transform.get(p, "-") for p in paths} - {"-"})
    out, passed = {"files": len(paths), "points": {}}, True
    for fpr in fprs:
        cut = [empirical_cutoff([v[1] for v in cal.values() if v[0] == 0], fpr) for cal in (cal_a, cal_b)]
        flags = [{p: test[p][1] > c for p in paths} for test, c in ((test_a, cut[0]), (test_b, cut[1]))]
        point = {"cutoff": cut, "calibrate_authentic": [sum(v[0] == 0 for v in cal.values()) for cal in (cal_a, cal_b)], "rows": [], "flips": {}}

        def count(label, keep):
            sel = [p for p in paths if test_a[p][0] == label and keep(transform.get(p, "-"))]
            return len(sel), sum(flags[0][p] for p in sel), sum(flags[1][p] for p in sel)

        for name, label, keep in [("authentic, all", 0, lambda t: True), ("authentic, clean", 0, lambda t: t == "-"), ("authentic, processed", 0, lambda t: t != "-"),
                                  ("generated, clean", 1, lambda t: t == "-"), ("generated, processed", 1, lambda t: t != "-")] + [(f"generated, {t}", 1, lambda u, t=t: u == t) for t in slices]:
            n, a, b = count(label, keep)
            gated = label == 1 and n >= min_slice
            ok = not gated or (b - a) / n >= -0.01
            passed = passed and ok
            point["rows"].append((name, n, a, b, gated, ok))
        for p in paths:
            if flags[0][p] != flags[1][p]:
                key = ("generated" if test_a[p][0] else "authentic", "clean" if transform.get(p, "-") == "-" else "processed", "32-bit only" if flags[0][p] else "INT8 only")
                point["flips"][key] = point["flips"].get(key, 0) + 1
        # The same on the test set's own authentic rows: a ranking measure, not a transfer result.
        own = [empirical_cutoff([test[p][1] for p in paths if test[p][0] == 0], fpr) for test in (test_a, test_b)]
        point["own"] = [(name, sum(test_a[p][1] > own[0] for p in sel) / max(len(sel), 1), sum(test_b[p][1] > own[1] for p in sel) / max(len(sel), 1))
                        for name, sel in (("clean", [p for p in paths if test_a[p][0] == 1 and transform.get(p, "-") == "-"]), ("processed", [p for p in paths if test_a[p][0] == 1 and transform.get(p, "-") != "-"]))]
        out["points"][fpr] = point
    diffs = sorted(abs(test_a[p][2] - test_b[p][2]) for p in paths)
    out.update(passed=passed, median_abs_diff=quantile(diffs, 0.5), p99_abs_diff=quantile(diffs, 0.99), max_abs_diff=diffs[-1] if diffs else float("nan"),
               mean_signed_diff=sum(test_b[p][2] - test_a[p][2] for p in paths) / max(len(paths), 1), stage_changes=sum(test_a[p][3] != test_b[p][3] for p in paths),
               views=[sum(t[p][3] for p in paths) / max(len(paths), 1) for t in (test_a, test_b)])
    return out


def frozen_table(r):
    lines = [f"{r['files']} test rows scored by both engines. Mean view logit, INT8 against 32-bit: median absolute difference {r['median_abs_diff']:.4f}, 99th percentile {r['p99_abs_diff']:.4f}, largest {r['max_abs_diff']:.4f}, mean signed {r['mean_signed_diff']:+.4f}; "
             f"{r['stage_changes']} rows stop at a different stage; views per row {r['views'][0]:.3f} and {r['views'][1]:.3f}."]
    for fpr, point in r["points"].items():
        lines += ["", f"**{100 * fpr:g}% FPR.** Cutoffs frozen on the authentic calibration rows ({point['calibrate_authentic'][0]} and {point['calibrate_authentic'][1]}): probability {point['cutoff'][0]:.6f} (32-bit) and {point['cutoff'][1]:.6f} (INT8).", "",
                  "| Test rows | Rows | Flagged, 32-bit | Flagged, INT8 | Rate, 32-bit → INT8 | Difference (points) | Gate |", "|---|---:|---:|---:|---|---:|---|"]
        for name, n, a, b, gated, ok in point["rows"]:
            lines.append(f"| {name} | {n} | {a} | {b} | {pct(a / n) if n else '–'} → {pct(b / n) if n else '–'} | {100 * (b - a) / n if n else 0:+.2f} | {('holds' if ok else '**fails**') if gated else ''} |")
        flips = ", ".join(f"{n} {k[0]} {k[1]} flagged by {k[2]}" for k, n in sorted(point["flips"].items())) or "none"
        lines += ["", f"Verdicts that differ at these cutoffs: {sum(point['flips'].values())} ({flips}).",
                  "At each engine's own empirical cutoff on the test set's authentic rows (a ranking measure, not a transfer result): " + "; ".join(f"recall on {name} rows {pct(a)} → {pct(b)} ({100 * (b - a):+.2f} points)" for name, a, b in point["own"]) + "."]
    lines += ["", f"Gate (at most one point of recall lost on every gated slice, at both rates): **{'holds' if r['passed'] else 'fails'}**."]
    return "\n".join(lines)


def make_cohort(store, per_store, seed):
    rng = random.Random(seed)
    for name in sorted(os.listdir(store)):
        manifest = os.path.join(store, name, "manifest.tsv")
        if not os.path.isfile(manifest):
            continue
        by = {0: [], 1: []}
        for line in open(manifest):
            f = line.rstrip("\n").split("\t")
            if len(f) >= 4 and f[1] in ("0", "1"):
                by[int(f[1])].append(f)
        picked = 0
        for label in (0, 1):
            rng.shuffle(by[label])
            took = 0
            for f in by[label]:
                if took >= per_store // 2:
                    break
                path = os.path.join(store, name, f[0])
                if os.path.isfile(path):
                    print("\t".join([os.path.abspath(path), *f[1:4]]))
                    took += 1
            picked += took
        print(f"{name}: {picked} files", file=sys.stderr)


def run(binary, cohort, out, device, extra, env_pairs, chunk):
    paths = list(read_cohort(cohort))
    env = dict(os.environ)
    for pair in env_pairs:
        name, _, value = pair.partition("=")
        env[name] = value
    with open(out, "w") as f:
        for at in range(0, len(paths), chunk):
            p = subprocess.run([binary, "detect", "--device", device, "--json", *extra, *paths[at:at + chunk]], stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
            # Exit status 2 means some file could not be scored; its row says so.
            if p.returncode not in (0, 2):
                sys.exit(f"{binary} failed ({p.returncode}): {p.stderr.decode(errors='replace')[-2000:]}")
            f.write(p.stdout.decode())
    print(f"{out}: {len(paths)} files, load average {os.getloadavg()[0]:.2f}", file=sys.stderr)


def self_test():
    assert empirical_cutoff([0.1, 0.9, 0.5, 0.3], 0.25) == 0.5 and empirical_cutoff([0.1, 0.9], 0.0) == 0.9
    assert rates([0.2, 0.8, 0.6, 0.4], [0, 0, 1, 1], 0.5) == (0.5, 0.5)
    labels = {"r1": 0, "r2": 0, "r3": 0, "r4": 0, "a1": 1, "a2": 1, "a3": 1, "a4": 1, "gone": 1}
    mk = lambda logit, views=1, thr=1.0: {"logit": logit, "verdict": "ai-generated" if logit > thr else "likely-real", "views": views}
    fp32 = {"r1": mk(-5.0), "r2": mk(-1.0), "r3": mk(0.5, 3), "r4": mk(1.5, 6), "a1": mk(4.0), "a2": mk(1.2, 6), "a3": mk(0.8, 6), "a4": mk(-2.0), "gone": mk(3.0)}
    int8 = {"r1": mk(-5.5), "r2": mk(-1.0), "r3": mk(1.25, 6), "r4": mk(0.9, 6), "a1": mk(4.0), "a2": mk(0.9, 6), "a3": mk(1.1, 6), "a4": mk(-2.0)}
    r = compare(labels, fp32, int8)
    assert (r["files"], r["authentic"], r["generated"], r["missing"]) == (8, 4, 4, 1)
    assert (r["generated_lost"], r["generated_gained"], r["authentic_newly_flagged"], r["authentic_no_longer_flagged"]) == (1, 1, 1, 1)
    assert r["stage_changes"] == 1 and abs(r["max_abs_diff"] - 0.75) < 1e-12 and abs(r["mean_signed_diff"] - (-0.5 + 0.75 - 0.6 - 0.3 + 0.3) / 8) < 1e-12
    assert r["shipped_recall_fp32"] == 0.5 and r["shipped_recall_int8"] == 0.5 and r["shipped_fpr_fp32"] == 0.25 and r["shipped_fpr_int8"] == 0.25
    # At 25% FPR one authentic file of four may be flagged: the cutoff is the second-highest authentic logit.
    r = compare(labels, fp32, int8, fprs=(0.25,))
    assert r["cutoff_at_0.25_fp32"] == 0.5 and r["recall_at_0.25_fp32"] == 0.75 and r["fpr_at_0.25_fp32"] == 0.25
    assert r["cutoff_at_0.25_int8"] == 0.9 and r["recall_at_0.25_int8"] == 0.5
    assert [t[0] for t in tail(labels, fp32, int8, 2)] == ["r4", "r3"]
    assert "| Files compared" in table(labels, fp32, [("x", int8)], 2)
    # Frozen cutoffs: 100 authentic calibration rows with probabilities 0.00 to 0.99, so 1% lets one through: cutoff 0.98.
    cal = {f"c{i}": (0, i / 100, 0.0, 1) for i in range(100)}
    cal_b = {f"c{i}": (0, i / 200, 0.0, 1) for i in range(100)}
    mk2 = lambda label, prob, logit=0.0, views=1: (label, prob, logit, views)
    test_a = {"r1": mk2(0, 0.985), "r2": mk2(0, 0.5), "g1": mk2(1, 0.99, 2.0), "g2": mk2(1, 0.97), "g3": mk2(1, 0.99), "g4": mk2(1, 0.2, 0.0, 6)}
    test_b = {"r1": mk2(0, 0.3), "r2": mk2(0, 0.5), "g1": mk2(1, 0.99, 2.5), "g2": mk2(1, 0.6), "g3": mk2(1, 0.4), "g4": mk2(1, 0.2, 0.0, 5)}
    r = frozen(cal, cal_b, test_a, test_b, {"g3": "jpeg", "g4": "jpeg"}, fprs=(0.01,), min_slice=2)
    point = r["points"][0.01]
    assert point["cutoff"] == [0.98, 0.49] and point["calibrate_authentic"] == [100, 100]
    rows = {name: (n, a, b, gated, ok) for name, n, a, b, gated, ok in point["rows"]}
    assert rows["authentic, all"] == (2, 1, 1, False, True) and rows["authentic, clean"] == (2, 1, 1, False, True)
    # Clean generated: g1 by both, g2 by INT8 only. Processed: g3 by the 32-bit engine only, a loss of 50 points.
    assert rows["generated, clean"] == (2, 1, 2, True, True) and rows["generated, processed"] == (2, 1, 0, True, False) and rows["generated, jpeg"] == (2, 1, 0, True, False)
    assert point["flips"] == {("authentic", "clean", "32-bit only"): 1, ("authentic", "clean", "INT8 only"): 1, ("generated", "clean", "INT8 only"): 1, ("generated", "processed", "32-bit only"): 1}
    assert not r["passed"] and r["stage_changes"] == 1 and abs(r["max_abs_diff"] - 0.5) < 1e-12 and "**fails**" in frozen_table(r)
    assert frozen(cal, cal, test_a, test_a, {}, fprs=(0.01,))["passed"]
    print("self-test passed")


def main():
    if sys.argv[1:] == ["--self-test"]:
        return self_test()
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--make-cohort", metavar="STORE", help="directory of stores, each with a manifest.tsv")
    ap.add_argument("--per-store", type=int, default=250)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--run", metavar="BIN", help="a tintype binary to run over the cohort")
    ap.add_argument("--cohort", help="labelled list written by --make-cohort")
    ap.add_argument("--out", help="where --run writes the rows")
    ap.add_argument("--device", default="cpu-int8")
    ap.add_argument("--extra", default="", help="more detect flags, e.g. --extra=--all-views")
    ap.add_argument("--env", action="append", default=[], metavar="NAME=VALUE", help="a variable for the binary (repeatable)")
    ap.add_argument("--chunk", type=int, default=400, help="files per process")
    ap.add_argument("--fp32", help="rows of the `--device cpu` run")
    ap.add_argument("--int8", action="append", default=[], metavar="LABEL=ROWS", help="rows of a `--device cpu-int8` run (repeatable)")
    ap.add_argument("--tail", type=int, default=0, help="also list this many of the largest authentic logits")
    ap.add_argument("--frozen", action="store_true", help="compare `tintype score` files at cutoffs frozen on calibration scores")
    for name in ("calibrate-fp32", "calibrate-int8", "test-fp32", "test-int8"):
        ap.add_argument("--" + name, help="score file for --frozen")
    ap.add_argument("--groups", help="with --frozen: the test manifest, whose ninth column is the transform")
    ap.add_argument("--min-slice", type=int, default=100, help="with --frozen: smallest slice of generated rows the gate applies to")
    a = ap.parse_args()
    if a.frozen:
        if not (a.calibrate_fp32 and a.calibrate_int8 and a.test_fp32 and a.test_int8 and a.groups):
            ap.error("--frozen takes the four score files and --groups")
        transform = {f[0]: f[8] for f in (l.rstrip("\n").split("\t") for l in open(a.groups)) if len(f) >= 9}
        return print(frozen_table(frozen(read_scores(a.calibrate_fp32), read_scores(a.calibrate_int8), read_scores(a.test_fp32), read_scores(a.test_int8), transform, min_slice=a.min_slice)))
    if a.make_cohort:
        return make_cohort(a.make_cohort, a.per_store, a.seed)
    if a.run:
        if not (a.cohort and a.out):
            ap.error("--run takes --cohort and --out")
        return run(a.run, a.cohort, a.out, a.device, a.extra.split(), a.env, a.chunk)
    if not (a.cohort and a.fp32 and a.int8):
        ap.error("give --make-cohort, --run, or --cohort with --fp32 and --int8")
    labels, fp32 = read_cohort(a.cohort), read_rows(a.fp32)
    runs = [(name, read_rows(path)) for name, _, path in (s.partition("=") for s in a.int8)]
    print(table(labels, fp32, runs, a.tail))


if __name__ == "__main__":
    main()
