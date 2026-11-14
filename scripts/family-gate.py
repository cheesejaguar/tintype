#!/usr/bin/env python3
"""The numbers the source-and-family gate is read from: macro recall over
new generator families, macro recall over older families, recall on
reposted or processed copies and the false-positive rate, for a candidate
and an incumbent at the same fixed cutoffs.

The proposed gate (docs/runbooks/source-family-matching.md): new-family
macro recall improves without regressing older families or reposted images
at fixed false-positive rate, at 1% and at 0.1%.

Macro recall is the unweighted mean of per-family recall: a family with 200
test images counts as much as one with 20,000, which is what "does it catch
the new families" means. Families with fewer than --min-family generated
items are listed and left out of the mean.

Inputs:

  --system name=scores.tsv@CUTOFF[,CUTOFF...]
        a score file from `tintype score` and the cutoffs to read it at:
        the probabilities frozen on the calibration set for each target
        rate (the "Cutoff" column of `tintype report --operating-points`
        run on the calibration scores). The first system is the incumbent.
  --label 1%,0.1%
        names for the cutoffs, in order.
  --groups test.tsv
        the manifest of the scored set, with grouping columns: the family
        is column 8, a processed copy has column 9 set, and the parent
        (column 5) is the unit that is resampled.
  --new-family NAME (repeatable)
        the families that count as new; every other family is older.

Differences (candidate minus incumbent) come with a 95% interval from a
bootstrap over parents: a photo and its processed copies are one draw.

Standard library only; runs under `python3 -I`.

  python3 -I scripts/family-gate.py --groups data/frozen-eval/test.tsv \\
      --system incumbent=runs/incumbent/test.tsv@0.93,0.995 \\
      --system candidate=runs/ws4-family/test.tsv@0.91,0.993 \\
      --label 1%,0.1% --new-family autoregressive-vq --new-family hybrid
  python3 -I scripts/family-gate.py --self-test
"""

import argparse
import random
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402


def read_scores(path):
    out = {}
    with open(path, encoding="utf-8") as stream:
        for number, raw in enumerate(stream, 1):
            f = raw.rstrip("\n").split("\t")
            if len(f) < 5:
                raise ValueError(f"{path}:{number}: expected `path label generator dataset prob ...`")
            out[f[0]] = float(f[4])
    return out


def measure(units, scores, cutoff, new, min_family, weights=None):
    """Macro recall over new and over older families, recall on processed
    and on unprocessed generated items, and the false-positive rate, with
    each unit (a parent's items) counted `weights[unit]` times."""
    hit, total = defaultdict(float), defaultdict(float)
    for k, items in enumerate(units):
        w = 1.0 if weights is None else weights[k]
        if not w:
            continue
        for file, label, family, processed in items:
            flagged = scores[file] >= cutoff
            if label == 0:
                total["real"] += w
                hit["real"] += w * flagged
                if processed:
                    total["real-processed"] += w
                    hit["real-processed"] += w * flagged
            else:
                total[("family", family)] += w
                hit[("family", family)] += w * flagged
                key = "processed" if processed else "unprocessed"
                total[key] += w
                hit[key] += w * flagged
    rate = lambda k: hit[k] / total[k] if total[k] else None  # noqa: E731
    families = {k[1]: (rate(k), total[k]) for k in total if isinstance(k, tuple)}
    macro = {}
    for name, members in (("new", [f for f in families if f in new]), ("older", [f for f in families if f not in new])):
        values = [families[f][0] for f in members if families[f][1] >= min_family]
        macro[name] = sum(values) / len(values) if values else None
    return {"new-family macro recall": macro["new"], "older-family macro recall": macro["older"], "recall, processed copies": rate("processed"), "recall, unprocessed": rate("unprocessed"), "false-positive rate": rate("real"), "false-positive rate, processed copies": rate("real-processed")}, families


METRICS = ["new-family macro recall", "older-family macro recall", "recall, processed copies", "recall, unprocessed", "false-positive rate", "false-positive rate, processed copies"]


def pct(x):
    return "–" if x is None else f"{100 * x:.2f}%"


def run(args, out=sys.stdout):
    systems = []
    for spec in args.system:
        name, rest = spec.split("=", 1)
        file, cutoffs = rest.rsplit("@", 1)
        systems.append((name, read_scores(file), [float(c) for c in cutoffs.split(",")]))
    labels = args.label.split(",")
    if any(len(s[2]) != len(labels) for s in systems):
        raise SystemExit("every --system needs one cutoff per --label")
    new = set(args.new_family or [])
    by_parent = defaultdict(list)
    missing = 0
    for manifest in args.groups:
        for row in mg.read_manifest(manifest):
            if any(row.file not in s[1] for s in systems):
                missing += 1
                continue
            family = row.family() if row.label == 1 else None
            by_parent[row.group(0) or row.file].append((row.file, row.label, family, bool(row.group(4))))
    units = [by_parent[k] for k in sorted(by_parent)]
    if not units:
        raise SystemExit("no row of --groups was scored by every system (the paths in the score files must be the manifest's)")
    n_items = sum(len(u) for u in units)
    print(f"{n_items} items of {len(units)} parents scored by all {len(systems)} systems" + (f"; {missing} manifest rows without a score were left out" if missing else "") + ".", file=out)
    print(f"New families: {', '.join(sorted(new)) or '(none named)'}. Families with fewer than {args.min_family} generated items are left out of the macro means.\n", file=out)
    rng = random.Random(args.seed)
    draws = []
    for _ in range(args.bootstrap):
        w = [0] * len(units)
        for k in rng.choices(range(len(units)), k=len(units)):
            w[k] += 1
        draws.append(w)
    for c, label in enumerate(labels):
        results = [measure(units, s[1], s[2][c], new, args.min_family) for s in systems]
        print(f"**At the cutoffs frozen for {label} FPR** ({', '.join(f'{s[0]} {s[2][c]:g}' for s in systems)})\n", file=out)
        print("| Measure | " + " | ".join(s[0] for s in systems) + " | " + " | ".join(f"{s[0]} − {systems[0][0]} [95% over parents]" for s in systems[1:]) + " |", file=out)
        print("|---|" + "---|" * (2 * len(systems) - 1), file=out)
        boot = [[measure(units, s[1], s[2][c], new, args.min_family, w)[0] for w in draws] for s in systems] if args.bootstrap else None
        for m in METRICS:
            cells = [pct(r[0][m]) for r in results]
            for k in range(1, len(systems)):
                if results[k][0][m] is None or results[0][0][m] is None:
                    cells.append("–")
                    continue
                diff = 100 * (results[k][0][m] - results[0][0][m])
                if boot:
                    d = sorted(100 * (a[m] - b[m]) for a, b in zip(boot[k], boot[0]) if a[m] is not None and b[m] is not None)
                    cells.append(f"{diff:+.2f} [{d[int(0.025 * len(d))]:+.2f}, {d[min(len(d) - 1, int(0.975 * len(d)))]:+.2f}]" if d else f"{diff:+.2f}")
                else:
                    cells.append(f"{diff:+.2f}")
            print(f"| {m} | " + " | ".join(cells) + " |", file=out)
        print("\n| Family | Kind | Generated items | " + " | ".join(f"Recall, {s[0]}" for s in systems) + " |\n|---|---|---|" + "---|" * len(systems), file=out)
        for family in sorted(results[0][1], key=lambda f: (f not in new, f)):
            n = results[0][1][family][1]
            print(f"| {family} | {'new' if family in new else 'older'}{'' if n >= args.min_family else ', too few'} | {n:.0f} | " + " | ".join(pct(r[1][family][0]) for r in results) + " |", file=out)
        print(file=out)


def self_test():
    import io
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        rows, old, cand = [], [], []
        rng = random.Random(5)
        for p in range(400):
            for transform in ("", "repost"):
                file = f"/set/real/{p}{transform}.jpg"
                rows.append(f"{file}\t0\treal:cam\tset\tr{p}\t\t\t\t{transform}")
                s = rng.random() * 0.5
                old.append((file, 0, s))
                cand.append((file, 0, s))
        for p in range(600):
            family = ["gan", "latent-diffusion", "ar"][p % 3]
            for transform in ("", "repost"):
                file = f"/set/ai/{p}{transform}.png"
                rows.append(f"{file}\t1\tgen{p % 3}\tset\ta{p}\t\t\t{family}\t{transform}")
                # The incumbent misses the new family; the candidate finds it and loses nothing elsewhere.
                base = 0.9 if family != "ar" else 0.2
                old.append((file, 1, base))
                cand.append((file, 1, 0.9 if family == "ar" and p % 2 == 0 else base))
        rows.append("/set/ai/rare.png\t1\tgenx\tset\tax\t\t\ttiny\t")
        old.append(("/set/ai/rare.png", 1, 0.9))
        cand.append(("/set/ai/rare.png", 1, 0.1))
        rows.append("/set/ai/unscored.png\t1\tgenx\tset\tay\t\t\ttiny\t")
        (tmp / "test.tsv").write_text("\n".join(rows) + "\n")
        for name, data in (("old", old), ("cand", cand)):
            (tmp / f"{name}.tsv").write_text("".join(f"{f}\t{label}\tg\tset\t{s:.6f}\n" for f, label, s in data))
        out = io.StringIO()
        argv = ["--groups", str(tmp / "test.tsv"), "--system", f"incumbent={tmp / 'old.tsv'}@0.5,0.95", "--system", f"candidate={tmp / 'cand.tsv'}@0.5,0.95", "--label", "1%,0.1%", "--new-family", "ar", "--min-family", "50", "--bootstrap", "200"]
        run(parser().parse_args(argv), out)
        text = out.getvalue()
        assert "2001 items of 1001 parents scored by all 2 systems; 1 manifest rows without a score were left out." in text, text
        first = text.split("**At the cutoffs frozen for 0.1% FPR**")[0]
        # New family: 0% to 50%; older families (gan, latent-diffusion; `tiny` has too few) stay at 100%.
        line = next(l for l in first.splitlines() if l.startswith("| new-family macro recall"))
        assert "| 0.00% | 50.00% | +50.00 [" in line, line
        lo, hi = (float(x) for x in line.split("[")[1].split("]")[0].split(","))
        assert 40 < lo < 50 < hi < 60, line
        assert "| older-family macro recall | 100.00% | 100.00% | +0.00 [+0.00, +0.00] |" in first, first
        assert "| recall, processed copies | 66.67% | 83.33% |" in first and "| false-positive rate | 0.00% | 0.00% |" in first, first
        assert "| ar | new | 400 | 0.00% | 50.00% |" in first and "| tiny | older, too few | 1 | 100.00% | 0.00% |" in first, first
        # At the stricter cutoff nothing is flagged by either.
        assert "| new-family macro recall | 0.00% | 0.00% | +0.00" in text.split("**At the cutoffs frozen for 0.1% FPR**")[1]
    print("family-gate self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--system", action="append", help="name=scores.tsv@cutoff[,cutoff...]; the first is the incumbent")
    p.add_argument("--label", default="1%,0.1%", help="names of the cutoffs, in order")
    p.add_argument("--groups", nargs="+", help="manifests of the scored set, with grouping columns")
    p.add_argument("--new-family", action="append", help="a family that counts as new (repeatable)")
    p.add_argument("--min-family", type=int, default=100, help="smallest family that enters a macro mean")
    p.add_argument("--bootstrap", type=int, default=1000)
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--self-test", action="store_true")
    return p


def main():
    args = parser().parse_args()
    if args.self_test:
        return self_test()
    if not args.system or not args.groups:
        raise SystemExit("pass --system (twice or more) and --groups, or --self-test")
    run(args)


if __name__ == "__main__":
    main()
