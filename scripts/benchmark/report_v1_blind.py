#!/usr/bin/env python3
"""Markdown tables for TPB v1's blind read, from read_v1_blind.sh's output folder.

    python3 report_v1_blind.py READ_DIR > tables.md
"""
import json
import sys
from pathlib import Path

d = Path(sys.argv[1])
blind, open_ = (json.loads((d / f"{s}.json").read_text()) for s in ("blind", "open"))
pooled, common = (json.loads((d / f"diff-{s}.json").read_text()) for s in ("pooled", "common"))
K = [("precision", "Precision"), ("recall", "Recall"), ("fpr", "Real flagged"), ("recall_at_1pct_fpr", "Recall at 1% FPR")]


def table(res, title, timing=False):
    print(f"\n### {title}: {res['files']:,} files\n")
    head = "| Detector | " + " | ".join(n for _, n in K) + (" | ms per image |" if timing else " |")
    print(head)
    print("|" + "---|" * (len(K) + 1 + (1 if timing else 0)))
    for name, m in res["systems"].items():
        cells = [f"{m[k]:.2f}% ({m['intervals'][k][0]:.2f}–{m['intervals'][k][1]:.2f})" for k, _ in K]
        t = f" | {m['ms_per_image']:.1f}" if timing and "ms_per_image" in m else (" | –" if timing else "")
        print(f"| {name} | " + " | ".join(cells) + t + " |")


table(open_, "Open split (the website's pool)")
table(blind, "Blind split", timing=True)

print("\n### Paired differences on the blind split (95% intervals)\n")
print("| Pair | " + " | ".join(n for _, n in K) + " |")
print("|" + "---|" * (len(K) + 1))
for pair, iv in blind["paired"].items():
    print(f"| {pair} | " + " | ".join(f"{iv[k][0]:+.2f} to {iv[k][1]:+.2f}" for k, _ in K) + " |")

for title, diff in (("all files", pooled), ("the sets in both splits", common)):
    print(f"\n### Blind minus open, {title} (unpaired, 95% intervals)\n")
    print("| Detector | " + " | ".join(n for _, n in K) + " |")
    print("|" + "---|" * (len(K) + 1))
    for name, v in diff.items():
        print(f"| {name} | " + " | ".join(
            f"{v[k]['a']:.2f} vs {v[k]['b']:.2f}: {v[k]['diff']:+.2f} ({v[k]['interval'][0]:+.2f} to {v[k]['interval'][1]:+.2f})"
            for k, _ in K) + " |")

print("\n### Tintype per set: recall / real flagged, blind (open)\n")
print("| Set | Generated | Real | Blind recall | Open recall | Blind real flagged | Open real flagged |")
print("|---|---|---|---|---|---|---|")
b, o = blind["systems"]["Tintype v0.4.0"]["per_set"], open_["systems"]["Tintype v0.4.0"]["per_set"]
fmt = lambda x: "–" if x != x else f"{x:.2f}"
for s in sorted(b):
    ob = o.get(s, {})
    print(f"| {s} | {b[s]['generated']} | {b[s]['real']} | {fmt(b[s]['recall'])} | {fmt(ob.get('recall', float('nan')))} | "
          f"{fmt(b[s]['fpr'])} | {fmt(ob.get('fpr', float('nan')))} |")
