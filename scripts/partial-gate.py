#!/usr/bin/env python3
"""Evaluate the partial-edit gate on a test set at cutoffs frozen elsewhere.

For docs/runbooks/partial-edits.md. Reads the per-item file of `tintype
partial-report --cutoffs <frozen> --out <items>` (a test set scored at
cutoffs frozen on a calibration set) and the frozen cutoff file, and checks
the gate "finds local changes without labelling whole assets as generated,
at equal authentic FPR" for one local rule against the global score, at
each false-positive rate:

  transfer   the frozen cutoff lets through no more of the untouched test
             images than its target: the two-sided 95% Clopper-Pearson
             interval of the achieved rate reaches down to the target, for
             the global and for the local rule
  local      over parents, the local rule flags more of a parent's
             generated-paste items than the global score does: one-sided
             exact sign test over parents, p < 0.025
  controls   for each control kind (resave, self-paste, authentic-paste),
             over parents, the local rule does not flag a larger share of
             the parent's control items than of the parent's own untouched
             image: one-sided exact sign test, p >= 0.05 to pass

The comparisons are over parents, because the items of one parent are not
independent. A rate is "not resolved" when the calibration set had fewer
authentic images than one false positive at that rate needs; the gate at
that rate then cannot be evaluated. The "not global" part of the gate is
not a measurement: no verdict or score changes, which the default-output
check of the runbook covers.

Standard library only. `--self-test` checks the arithmetic on made-up rows.
"""

import argparse
import math
import sys
from collections import defaultdict
from pathlib import Path

COLUMN = {"global (deployed)": "global_logit", "grid max": "grid_max", "grid top-3": "grid_top_k", "views mean": "views_mean"}
CONTROLS = ("resave", "self-paste", "authentic-paste")


def binom_cdf(k, n, p):
    """P(X <= k) for X ~ Binomial(n, p)."""
    if k < 0:
        return 0.0
    if k >= n or p <= 0.0:
        return 1.0
    if p >= 1.0:
        return 0.0
    total = 0.0
    for i in range(k + 1):
        total += math.exp(math.lgamma(n + 1) - math.lgamma(i + 1) - math.lgamma(n - i + 1) + i * math.log(p) + (n - i) * math.log1p(-p))
    return min(1.0, total)


def clopper_pearson(k, n, confidence=0.95):
    """Two-sided exact interval of a rate after k events in n trials."""
    if n == 0:
        return float("nan"), float("nan")
    alpha = (1.0 - confidence) / 2.0

    def solve(f):
        lo, hi = 0.0, 1.0
        for _ in range(60):
            mid = (lo + hi) / 2.0
            lo, hi = (mid, hi) if f(mid) else (lo, mid)
        return (lo + hi) / 2.0

    lower = 0.0 if k == 0 else solve(lambda p: 1.0 - binom_cdf(k - 1, n, p) < alpha)
    upper = 1.0 if k == n else solve(lambda p: binom_cdf(k, n, p) > alpha)
    return lower, upper


def sign_test(more, fewer):
    """One-sided exact p-value that `more` wins out of more + fewer is chance."""
    n = more + fewer
    return 1.0 if n == 0 else 1.0 - binom_cdf(more - 1, n, 0.5)


def read_items(path):
    rows = []
    header = None
    for line in Path(path).read_text().splitlines():
        if line.startswith("#"):
            header = line.lstrip("# ").split("\t")
            continue
        if line:
            rows.append(dict(zip(header, line.split("\t"))))
    return rows


def read_cutoffs(path):
    cutoffs, authentic = {}, 0
    for line in Path(path).read_text().splitlines():
        f = line.split("\t")
        if f[0] == "# authentic":
            authentic = int(f[1])
        elif not line.startswith("#") and len(f) == 3:
            cutoffs[(f[0], f[1])] = float(f[2])
    return cutoffs, authentic


def pct(k, n):
    return "–" if n == 0 else f"{k}/{n} ({100.0 * k / n:.1f}%)"


def evaluate(rows, cutoffs, authentic, local, rates):
    """Markdown lines and, per rate, the verdict."""
    lines, verdicts = [], {}
    by_parent = defaultdict(list)
    for r in rows:
        by_parent[r["parent_id"]].append(r)
    for rate in rates:
        key = f"{rate:g}"
        lines.append(f"\n### {key}% authentic FPR, `{local}` against the global score\n")
        if (key, local) not in cutoffs or (key, "global (deployed)") not in cutoffs:
            lines.append("No frozen cutoff for this rate: **could not be evaluated**.")
            verdicts[key] = "could not be evaluated"
            continue
        cut = {"global": cutoffs[(key, "global (deployed)")], "local": cutoffs[(key, local)]}
        col = {"global": COLUMN["global (deployed)"], "local": COLUMN[local]}
        flagged = lambda r, rule: r[col[rule]] not in ("-", "") and float(r[col[rule]]) > cut[rule]
        resolved = authentic * rate / 100.0 >= 1.0
        untouched = [r for r in rows if r["kind"] == "parent"]
        transfer = True
        for rule, name in (("global", "global (deployed)"), ("local", local)):
            k = sum(flagged(r, rule) for r in untouched)
            lo, hi = clopper_pearson(k, len(untouched))
            ok = lo <= rate / 100.0
            transfer &= ok
            lines.append(f"- Transfer, `{name}` (cutoff {cut[rule]:.3f}): {pct(k, len(untouched))} untouched test images flagged, 95% interval {100 * lo:.2f}% to {100 * hi:.2f}%: {'within' if ok else 'above'} the {key}% target.")
        generated = [r for r in rows if r["generated"] == "1"]
        lines.append(f"- Edited items found: global {pct(sum(flagged(r, 'global') for r in generated), len(generated))}, `{local}` {pct(sum(flagged(r, 'local') for r in generated), len(generated))}.")
        more = fewer = 0
        for items in by_parent.values():
            g = [r for r in items if r["generated"] == "1"]
            if g:
                d = sum(flagged(r, "local") for r in g) - sum(flagged(r, "global") for r in g)
                more += d > 0
                fewer += d < 0
        p_local = sign_test(more, fewer)
        local_ok = more > fewer and p_local < 0.025
        lines.append(f"- Local: `{local}` finds more of a parent's edits than the global score in {more} parents and fewer in {fewer} (one-sided sign test p = {p_local:.4g}): {'more, beyond chance' if local_ok else 'not shown'}.")
        controls_ok = True
        for kind in CONTROLS:
            items = [r for r in rows if r["kind"] == kind]
            if not items:
                continue
            more = fewer = 0
            for parent in by_parent.values():
                c = [r for r in parent if r["kind"] == kind]
                u = [r for r in parent if r["kind"] == "parent"]
                if c and u:
                    share = sum(flagged(r, "local") for r in c) / len(c)
                    own = sum(flagged(r, "local") for r in u) / len(u)
                    more += share > own
                    fewer += share < own
            p = sign_test(more, fewer)
            ok = not (more > fewer and p < 0.05)
            controls_ok &= ok
            lines.append(f"- Control `{kind}`: `{local}` flags {pct(sum(flagged(r, 'local') for r in items), len(items))} (global {pct(sum(flagged(r, 'global') for r in items), len(items))}); a larger share than of the parent's own untouched image in {more} parents, a smaller one in {fewer} (p = {p:.4g}): {'no excess shown' if ok else 'flagged more than untouched images'}.")
        if not resolved:
            verdict = "could not be evaluated"
            lines.append(f"- The cutoffs were frozen on {authentic} authentic images, fewer than one false positive at {key}% needs: the rate is not resolved.")
        elif transfer and local_ok and controls_ok:
            verdict = "passed"
        else:
            verdict = "failed"
        failed = [n for n, ok in (("transfer", transfer), ("local", local_ok), ("controls", controls_ok)) if not ok]
        lines.append(f"- **Gate at {key}%: {verdict}**" + (f" ({', '.join(failed)} not met)." if failed and verdict == "failed" else "."))
        verdicts[key] = verdict
    return lines, verdicts


def self_test():
    assert abs(binom_cdf(1, 4, 0.5) - 5 / 16) < 1e-12 and binom_cdf(4, 4, 0.3) == 1.0
    lo, hi = clopper_pearson(0, 100)
    assert lo == 0.0 and abs(hi - (1 - 0.025 ** 0.01)) < 1e-6
    lo, hi = clopper_pearson(5, 100)
    assert abs(lo - 0.0164) < 5e-4 and abs(hi - 0.1128) < 5e-4, (lo, hi)
    assert sign_test(0, 0) == 1.0 and abs(sign_test(5, 0) - 1 / 32) < 1e-12 and abs(sign_test(3, 1) - 5 / 16) < 1e-12
    row = lambda parent, kind, generated, g, l: {"parent_id": parent, "kind": kind, "generated": str(generated), "global_logit": str(g), "grid_max": str(l)}
    rows = []
    for i in range(200):
        rows.append(row(f"p{i}", "parent", 0, 5.0 if i == 0 else 0.0, 5.0 if i == 1 else 0.0))
    for i in range(40):
        # The local rule finds the edit in 30 parents, the global score in 2 of those.
        rows.append(row(f"p{i}", "generated-paste", 1, 5.0 if i < 2 else 0.0, 5.0 if i < 30 else 0.0))
        rows.append(row(f"p{i}", "self-paste", 0, 0.0, 0.0))
    cutoffs = {("1", "global (deployed)"): 1.0, ("1", "grid max"): 1.0, ("0.1", "global (deployed)"): 1.0, ("0.1", "grid max"): 1.0}
    lines, verdicts = evaluate(rows, cutoffs, 1100, "grid max", (1.0, 0.1))
    text = "\n".join(lines)
    assert verdicts == {"1": "passed", "0.1": "passed"}, text
    assert "global 2/40 (5.0%), `grid max` 30/40 (75.0%)" in text and "in 28 parents and fewer in 0" in text, text
    # Too few calibration images for 0.1%: not resolved.
    assert evaluate(rows, cutoffs, 500, "grid max", (0.1,))[1] == {"0.1": "could not be evaluated"}
    # A control the local rule flags in six parents whose untouched image it does not flag fails the gate.
    for i in range(6):
        rows.append(row(f"p{i + 50}", "resave", 0, 0.0, 5.0))
    lines, verdicts = evaluate(rows, cutoffs, 1100, "grid max", (1.0,))
    assert verdicts == {"1": "failed"} and "controls not met" in "\n".join(lines), "\n".join(lines)
    # A cutoff that lets through far more than its target fails on transfer.
    hot = rows[:200] + [row(f"q{i}", "parent", 0, 0.0, 5.0) for i in range(20)] + rows[200:280]
    assert evaluate(hot, cutoffs, 1100, "grid max", (1.0,))[1] == {"1": "failed"}
    print("self-test passed")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--items", help="per-item file of `partial-report --cutoffs ... --out`")
    ap.add_argument("--cutoffs", help="the frozen cutoff file")
    ap.add_argument("--local", default="grid max", choices=[k for k in COLUMN if k != "global (deployed)"])
    ap.add_argument("--fpr", type=float, nargs="+", default=[1.0, 0.1], help="authentic false-positive rates, in percent")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if not args.items or not args.cutoffs:
        sys.exit("need --items and --cutoffs (or --self-test)")
    cutoffs, authentic = read_cutoffs(args.cutoffs)
    rows = read_items(args.items)
    lines, _ = evaluate(rows, cutoffs, authentic, args.local, args.fpr)
    print(f"Partial-edit gate: {len(rows)} test items, cutoffs frozen on {authentic} authentic images." + "\n".join(lines))


if __name__ == "__main__":
    main()
