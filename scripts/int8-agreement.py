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
    a = ap.parse_args()
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
