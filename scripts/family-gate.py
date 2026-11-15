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
the new families" means. A family's recall is over its unprocessed items
(processed copies are measured once, in their own row). Families with
fewer than --min-family unprocessed generated items are listed and left
out of the mean. The authentic sources are listed with the number of their
unprocessed items each system flags.

Inputs:

  --system name=scores.tsv@CUTOFF[,CUTOFF...]
        a score file from `tintype score` and the cutoffs to read it at:
        the probabilities frozen on the calibration set for each target
        rate (the "Cutoff" column of `tintype report --operating-points`
        run on the calibration scores). The first system is the incumbent.
  --label 1%,0.1%
        names for the cutoffs, in order.
  --raw
        the cutoffs are raw thresholds on the mean of the view logits
        (columns 6 and up of an `--all-views` score file; a calibrated
        spec's threshold is -logit_bias / logit_scale), not probabilities.
        One score file then serves every cutoff of a model.
  --groups test.tsv
        the manifest of the scored set, with grouping columns: the family
        is column 8, a processed copy has column 9 set, and the parent
        (column 5) is the unit that is resampled. `.gz` files are read as
        they are, and an archived manifest and score file whose paths were
        made relative together are matched by the path as written.
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
import math
import random
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402


def read_scores(path, raw=False):
    """`{path: score}`: the probability column, or with `raw` the mean of the
    view logits after it. A `.gz` file is read as it is."""
    import gzip
    out = {}
    with (gzip.open(path, "rt", encoding="utf-8") if str(path).endswith(".gz") else open(path, encoding="utf-8")) as stream:
        for number, line in enumerate(stream, 1):
            f = line.rstrip("\n").split("\t")
            if len(f) < (6 if raw else 5):
                raise ValueError(f"{path}:{number}: expected `path label generator dataset prob{' logits...' if raw else ' ...'}`")
            out[f[0]] = sum(float(x) for x in f[5:]) / len(f[5:]) if raw else float(f[4])
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
        for file, label, family, processed, source in items:
            flagged = scores[file] >= cutoff
            if label == 0:
                key = "real-processed" if processed else "real-unprocessed"
                total[key] += w
                hit[key] += w * flagged
                if not processed:
                    total[("source", source)] += w
                    hit[("source", source)] += w * flagged
            else:
                key = "processed" if processed else "unprocessed"
                total[key] += w
                hit[key] += w * flagged
                if not processed:
                    total[("family", family)] += w
                    hit[("family", family)] += w * flagged
    rate = lambda k: hit[k] / total[k] if total[k] else None  # noqa: E731
    families = {k[1]: (rate(k), total[k]) for k in total if isinstance(k, tuple) and k[0] == "family"}
    sources = {k[1]: (hit[k], total[k]) for k in total if isinstance(k, tuple) and k[0] == "source"}
    macro = {}
    for name, members in (("new", [f for f in families if f in new]), ("older", [f for f in families if f not in new])):
        values = [families[f][0] for f in members if families[f][1] >= min_family]
        macro[name] = sum(values) / len(values) if values else None
    return {"new-family macro recall": macro["new"], "older-family macro recall": macro["older"], "recall, processed copies": rate("processed"), "recall, unprocessed": rate("unprocessed"), "false-positive rate, unprocessed": rate("real-unprocessed"), "false-positive rate, processed copies": rate("real-processed")}, families, sources


METRICS = ["new-family macro recall", "older-family macro recall", "recall, processed copies", "recall, unprocessed", "false-positive rate, unprocessed", "false-positive rate, processed copies"]


def pct(x):
    return "–" if x is None else f"{100 * x:.2f}%"


def run(args, out=sys.stdout):
    systems = []
    for spec in args.system:
        name, rest = spec.split("=", 1)
        file, cutoffs = rest.rsplit("@", 1)
        systems.append((name, read_scores(file, args.raw), [float(c) for c in cutoffs.split(",")]))
    labels = args.label.split(",")
    if any(len(s[2]) != len(labels) for s in systems):
        raise SystemExit("every --system needs one cutoff per --label")
    new = set(args.new_family or [])
    by_parent = defaultdict(list)
    missing = 0
    for manifest in args.groups:
        for row in mg.read_manifest(manifest):
            # A score file names a file as the manifest resolves it, or, in an archive whose
            # paths were made relative, as the manifest writes it.
            key = row.file if all(row.file in s[1] for s in systems) else row.path
            if any(key not in s[1] for s in systems):
                missing += 1
                continue
            family = row.family() if row.label == 1 else None
            by_parent[row.group(0) or key].append((key, row.label, family, bool(row.group(4)), row.source()))
    units = [by_parent[k] for k in sorted(by_parent)]
    if not units:
        raise SystemExit("no row of --groups was scored by every system (the paths in the score files must be the manifest's)")
    n_items = sum(len(u) for u in units)
    print(f"{n_items} items of {len(units)} parents scored by all {len(systems)} systems" + (f"; {missing} manifest rows without a score were left out" if missing else "") + ".", file=out)
    print(f"New families: {', '.join(sorted(new)) or '(none named)'}. A family's recall is over its unprocessed items; families with fewer than {args.min_family} of them are left out of the macro means. Cutoffs are {'raw thresholds on the mean view logit' if args.raw else 'probabilities'}.\n", file=out)
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
        print("\n| Family | Kind | Unprocessed generated items | " + " | ".join(f"Recall, {s[0]}" for s in systems) + " |\n|---|---|---|" + "---|" * len(systems), file=out)
        for family in sorted(results[0][1], key=lambda f: (f not in new, f)):
            n = results[0][1][family][1]
            print(f"| {family} | {'new' if family in new else 'older'}{'' if n >= args.min_family else ', too few'} | {n:.0f} | " + " | ".join(pct(r[1][family][0]) for r in results) + " |", file=out)
        print("\n| Authentic source | Unprocessed items | " + " | ".join(f"Flagged, {s[0]}" for s in systems) + " |\n|---|---|" + "---|" * len(systems), file=out)
        for source in sorted(results[0][2]):
            n = results[0][2][source][1]
            print(f"| {source} | {n:.0f} | " + " | ".join(f"{r[2][source][0]:.0f} ({pct(r[2][source][0] / n)})" for r in results) + " |", file=out)
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
                rows.append(f"{file}\t0\treal:cam\tset\tr{p}\t\tcam{p % 2}\t\t{transform}")
                s = 0.01 + rng.random() * 0.49
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
            # The probability and two view logits whose mean is its logit.
            (tmp / f"{name}.tsv").write_text("".join(f"{f}\t{label}\tg\tset\t{s:.6f}\t{math.log(s / (1 - s)) + 1:.6f}\t{math.log(s / (1 - s)) - 1:.6f}\n" for f, label, s in data))
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
        assert "| recall, processed copies | 66.67% | 83.33% |" in first and "| false-positive rate, unprocessed | 0.00% | 0.00% |" in first, first
        assert "| ar | new | 200 | 0.00% | 50.00% |" in first and "| tiny | older, too few | 1 | 100.00% | 0.00% |" in first, first
        assert "| cam0 | 200 | 0 (0.00%) | 0 (0.00%) |" in first and "| cam1 | 200 | 0 (0.00%) | 0 (0.00%) |" in first, first
        # At the stricter cutoff nothing is flagged by either.
        assert "| new-family macro recall | 0.00% | 0.00% | +0.00" in text.split("**At the cutoffs frozen for 0.1% FPR**")[1]
        # The same cutoffs as raw thresholds on the mean view logit give the same tables.
        raw = io.StringIO()
        logit = lambda p: math.log(p / (1 - p))  # noqa: E731
        argv[3] = f"incumbent={tmp / 'old.tsv'}@{logit(0.5)},{logit(0.95)}"
        argv[5] = f"candidate={tmp / 'cand.tsv'}@{logit(0.5)},{logit(0.95)}"
        run(parser().parse_args(argv + ["--raw"]), raw)
        same = lambda t: [l for l in t.splitlines() if l.startswith("| ") and "macro" in l or l.startswith("| ar ")]  # noqa: E731
        assert same(raw.getvalue()) == same(text), raw.getvalue()
        # An archive: the same files gzipped, with the leading directory stripped from every path.
        import gzip
        for name in ("test", "old", "cand"):
            with gzip.open(tmp / f"{name}.tsv.gz", "wt") as stream:
                stream.write("".join(l[len("/set/"):] + "\n" for l in (tmp / f"{name}.tsv").read_text().splitlines()))
        archived = io.StringIO()
        argv[1], argv[3], argv[5] = str(tmp / "test.tsv.gz"), f"incumbent={tmp / 'old.tsv.gz'}@{logit(0.5)},{logit(0.95)}", f"candidate={tmp / 'cand.tsv.gz'}@{logit(0.5)},{logit(0.95)}"
        run(parser().parse_args(argv + ["--raw"]), archived)
        assert archived.getvalue() == raw.getvalue()
    print("family-gate self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--system", action="append", help="name=scores.tsv@cutoff[,cutoff...]; the first is the incumbent")
    p.add_argument("--label", default="1%,0.1%", help="names of the cutoffs, in order")
    p.add_argument("--raw", action="store_true", help="cutoffs are raw thresholds on the mean view logit of an --all-views score file")
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
