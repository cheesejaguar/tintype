#!/usr/bin/env python3
"""Compare frame sampling and window aggregation on clips with a short edit.

For the partial-edit evaluation of video (docs/runbooks/partial-edits.md).
Reads the `edits.tsv` of a store built by `build-splice-clips.py` and one or
more result files of `tintype detect --json` run over its clips:

  --system NAME=results.jsonl[@MAX_SECONDS]

A result file from `--coverage uniform|segments|shots` carries every sampled
time and frame logit, so every rule below applies to it. A plain `detect
--json` file carries the clip logit only; for it the sampled times are
rebuilt from the default sampling (MAX_SECONDS, 30 unless given, and the
early-exit frame order in --video-spec), and only the clip rule applies.

Rules (each a number per clip; a clip is flagged when it is above the
cutoff):

  clip mean      the mean over every scored frame: what `detect` decides on
  max window     the highest window mean (`segments`)
  max frame      the highest frame logit
  top-K frames   the mean of the K highest frame logits

Every (system, rule) pair is operated at its own cutoff: the one that lets
the stated share of the authentic clips through (re-encodes, authentic
splices and dissolves together, unless --cutoff-on names one kind). The
columns therefore compare at equal authentic clip FPR.

An edit that no sampled frame falls inside is counted as missed, whatever
the clip's score: it is in the denominator and never in the numerator.
"False flagged segments per hour" counts, over authentic clips, the windows
above the `max window` cutoff per hour of footage, authentic cuts and
dissolves included. A clip that could not be scored, or whose coverage was
incomplete, counts as not flagged and is listed.

Standard library only. `--self-test` checks the arithmetic on made-up rows.
"""

import argparse
import json
import math
import os
import sys
from collections import defaultdict
from pathlib import Path

RATES = (0.01, 0.001)


def read_edits(path):
    """Rows of edits.tsv keyed by the clip's real path."""
    root = Path(path).resolve().parent
    out = {}
    for line in Path(path).read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        f = line.split("\t")
        edit = None if f[3] in ("-", "") else (float(f[3]), float(f[4]))
        cuts = [] if len(f) < 9 or f[8] in ("-", "") else [float(c) for c in f[8].split(",")]
        out[os.path.realpath(root / f[0])] = {"kind": f[1], "generated": f[2] == "1", "edit": edit, "cuts": cuts, "slice": Path(f[0]).stem, "parent": Path(f[0]).parent.name}
    return out


def default_times(row, max_seconds, order):
    """Sampling targets a plain `detect` run looked at: frame k of n at k * span / n, in early-exit order."""
    n, seen, duration = row.get("frames_expected"), row.get("frames"), row.get("duration_s")
    if not n or seen is None or not duration:
        return []
    span = min(duration, max_seconds) if max_seconds > 0 else duration
    looked = order[:seen] if seen < n and order and len(order) == n else range(min(seen, n))
    return sorted(k * span / n for k in looked)


def read_results(path, max_seconds, order):
    """Per clip: clip logit, windows [(start, end, logit, times, frame logits)], sampled times, duration, cuts found."""
    out = {}
    for line in Path(path).read_text().splitlines():
        if not line.strip():
            continue
        row = json.loads(line)
        key = os.path.realpath(row["path"])
        if "error" in row or row.get("logit") is None:
            out[key] = {"usable": False}
            continue
        windows = [
            (s["start_s"], s["end_s"], s["logit"], s["sampled_s"], s["frame_logits"])
            for s in row.get("segments", [])
        ]
        times = sorted(t for w in windows for t in w[3]) if windows else default_times(row, max_seconds, order)
        out[key] = {
            "usable": bool(row.get("complete", True)),
            "logit": row["logit"],
            "windows": windows,
            "frames": [z for w in windows for z in w[4]],
            "times": times,
            "duration": row.get("duration_s") or 0.0,
            "cuts": row.get("cuts_s"),
        }
    return out


def rules_for(results, top_k):
    """The rules every usable clip of a system has a value for."""
    usable = [r for r in results.values() if r["usable"]]
    rules = {"clip mean": lambda r: r["logit"]}
    if usable and all(r["windows"] and all(w[2] is not None for w in r["windows"]) for r in usable):
        rules["max window"] = lambda r: max(w[2] for w in r["windows"])
    if usable and all(r["frames"] for r in usable):
        rules["max frame"] = lambda r: max(r["frames"])
        rules[f"top-{top_k} frames"] = lambda r: sum(sorted(r["frames"], reverse=True)[:top_k]) / min(top_k, len(r["frames"]))
    return rules


def cutoff(values, rate):
    """The strictest cutoff that lets at most `rate` of `values` through (flag strictly above it)."""
    if not values:
        return None
    ranked = sorted(values, reverse=True)
    return ranked[min(int(math.floor(rate * len(ranked))), len(ranked) - 1)]


def sampled(times, edit):
    return any(edit[0] <= t <= edit[1] for t in times)


def fraction(k, n):
    return "–" if n == 0 else f"{k}/{n} ({100.0 * k / n:.1f}%)"


def report(edits, systems, top_k, cutoff_on, rates=RATES):
    lines = []
    kinds = sorted({e["kind"] for e in edits.values() if not e["generated"]})
    lines.append(f"Splice store: {len(edits)} clips ({sum(e['generated'] for e in edits.values())} with a generated segment; authentic kinds: {', '.join(kinds)}).")
    lines.append("Cutoffs are set per system and rule on the authentic clips" + (f" of kind `{cutoff_on}`" if cutoff_on else " of every kind") + ". An edit no sampled frame falls inside counts as missed.")
    pool = [e for e in edits.values() if not e["generated"] and (cutoff_on is None or e["kind"] == cutoff_on)]
    parents = len({e.get("parent", i) for i, e in enumerate(pool)})
    if parents < len(pool):
        lines.append(f"Those {len(pool)} authentic clips come from {parents} parent timelines, and clips of one parent share most of their frames: the authentic rate is resolved no finer than 1/{parents}, whatever the clip count allows.")
    for name, results in systems:
        rules = rules_for(results, top_k)
        rows = {k: results.get(k, {"usable": False}) for k in edits}
        unusable = sum(not r["usable"] for r in rows.values())
        authentic = [k for k, e in edits.items() if not e["generated"] and (cutoff_on is None or e["kind"] == cutoff_on)]
        hours = sum(rows[k].get("duration", 0.0) for k in edits if not edits[k]["generated"] and rows[k]["usable"]) / 3600.0
        lines.append(f"\n## {name}\n")
        lines.append(f"{len(rows) - unusable} of {len(rows)} clips scored with complete coverage ({unusable} not usable, counted as not flagged). Authentic footage: {hours:.3f} h.")
        slices = sorted({e["slice"] for e in edits.values() if e["generated"]})
        coverage = []
        for s in slices:
            keys = [k for k, e in edits.items() if e["slice"] == s]
            coverage.append(f"{s}: {fraction(sum(rows[k]['usable'] and sampled(rows[k]['times'], edits[k]['edit']) for k in keys), len(keys))}")
        lines.append("Edits with at least one sampled frame inside: " + "; ".join(coverage) + ".")
        for rate in rates:
            allowed = int(math.floor(rate * len(authentic)))
            lines.append(f"\n### {100 * rate:g}% authentic clip FPR (cutoffs may flag {allowed} of {len(authentic)} authentic clips" + ("; not resolved with this many clips: the cutoff is the highest authentic score" if allowed == 0 else "") + ")\n")
            lines.append("| slice | clips | " + " | ".join(rules) + " |")
            lines.append("|---|---:|" + "---:|" * len(rules))
            cuts = {rule: cutoff([f(rows[k]) for k in authentic if rows[k]["usable"]], rate) for rule, f in rules.items()}
            lines.append("| cutoff (logit) | | " + " | ".join("–" if c is None else f"{c:.3f}" for c in cuts.values()) + " |")
            flagged = lambda k, rule: rows[k]["usable"] and cuts[rule] is not None and rules[rule](rows[k]) > cuts[rule]
            for s in sorted({e["slice"] for e in edits.values()}, key=lambda s: (any(e["generated"] for e in edits.values() if e["slice"] == s), s)):
                keys = [k for k, e in edits.items() if e["slice"] == s]
                generated = edits[keys[0]]["generated"]
                cells = []
                for rule in rules:
                    if generated:
                        # Found: flagged and the edit was sampled. Flags on clips whose edit was never sampled are shown apart.
                        found = sum(flagged(k, rule) and sampled(rows[k]["times"], edits[k]["edit"]) for k in keys)
                        blind = sum(flagged(k, rule) and not sampled(rows[k]["times"], edits[k]["edit"]) for k in keys)
                        cells.append(fraction(found, len(keys)) + (f" +{blind} unsampled" if blind else ""))
                    else:
                        cells.append(fraction(sum(flagged(k, rule) for k in keys), len(keys)))
                lines.append(f"| {s}{' (found)' if generated else ''} | {len(keys)} | " + " | ".join(cells) + " |")
            if "max window" in rules and cuts["max window"] is not None and hours > 0:
                per_kind = []
                total = 0
                for kind in kinds:
                    keys = [k for k, e in edits.items() if e["kind"] == kind and rows[k]["usable"]]
                    n = sum(w[2] > cuts["max window"] for k in keys for w in rows[k]["windows"])
                    h = sum(rows[k]["duration"] for k in keys) / 3600.0
                    total += n
                    per_kind.append(f"{kind} {n} in {h:.3f} h" + (f" ({n / h:.1f}/h)" if h > 0 else ""))
                lines.append(f"\nFalse flagged segments per hour of authentic footage (windows above the `max window` cutoff): {total / hours:.1f}/h overall; " + "; ".join(per_kind) + ".")
                hits = defaultdict(lambda: [0, 0])
                for k, e in edits.items():
                    if e["generated"]:
                        hits[e["slice"]][1] += 1
                        hits[e["slice"]][0] += rows[k]["usable"] and any(w[2] > cuts["max window"] and w[0] < e["edit"][1] and w[1] > e["edit"][0] for w in rows[k]["windows"])
                lines.append("Edits inside a flagged window: " + "; ".join(f"{s} {fraction(a, b)}" for s, (a, b) in sorted(hits.items())) + ".")
        found_cuts = [(k, r["cuts"]) for k, r in rows.items() if r["usable"] and r.get("cuts") is not None]
        if found_cuts:
            # Hard cuts the store put in: the ends of a splice and the joins of an assembled parent.
            known = hit = extra = 0
            for k, cuts_found in found_cuts:
                e = edits[k]
                # A join of the parent that the inserted segment replaced is not in the file.
                truth = [c for c in e["cuts"] if not (e["edit"] and e["edit"][0] < c < e["edit"][1])]
                truth += list(e["edit"]) if e["edit"] and e["kind"].endswith("splice") else []
                known += len(truth)
                hit += sum(any(abs(c - t) <= 0.15 for c in cuts_found) for t in truth)
                extra += sum(not any(abs(c - t) <= 0.15 for t in truth) for c in cuts_found)
            lines.append(f"\nCut detector (`shots`): found {fraction(hit, known)} of the hard cuts the store made (within 0.15 s), and {extra} other cuts in {len(found_cuts)} clips (cuts inside the source clips, dissolves, and false ones).")
    return "\n".join(lines) + "\n"


def self_test():
    edits = {
        "a": {"kind": "reencode", "generated": False, "edit": None, "cuts": [], "slice": "reencode"},
        "b": {"kind": "authentic-splice", "generated": False, "edit": (4.0, 6.0), "cuts": [], "slice": "authentic-splice-2s"},
        "c": {"kind": "generated-splice", "generated": True, "edit": (4.0, 6.0), "cuts": [], "slice": "generated-splice-2s"},
        "d": {"kind": "generated-splice", "generated": True, "edit": (40.0, 42.0), "cuts": [], "slice": "generated-splice-2s"},
    }
    window = lambda start, end, zs, ts: (start, end, sum(zs) / len(zs), ts, zs)
    clip = lambda ws: {"usable": True, "logit": sum(z for w in ws for z in w[4]) / sum(len(w[4]) for w in ws), "windows": ws, "frames": [z for w in ws for z in w[4]], "times": sorted(t for w in ws for t in w[3]), "duration": 60.0, "cuts": None}
    results = {
        "a": clip([window(0, 30, [0.0, 0.2], [5, 20]), window(30, 60, [0.1, 0.0], [35, 50])]),
        "b": clip([window(0, 30, [0.3, 0.1], [5, 20]), window(30, 60, [0.0, 0.0], [35, 50])]),
        # The edit is sampled at 5 s and scores high in one frame only.
        "c": clip([window(0, 30, [9.0, 0.0], [5, 20]), window(30, 60, [0.0, 0.0], [35, 50])]),
        # The edit at 40-42 s lies between the frames at 35 s and 50 s; one unrelated frame scores high.
        "d": clip([window(0, 30, [9.0, 0.0], [5, 20]), window(30, 60, [0.0, 0.0], [35, 50])]),
    }
    assert cutoff([0.1, 0.5, 0.3], 0.01) == 0.5 and cutoff([0.1, 0.5, 0.3], 0.34) == 0.3 and cutoff([], 0.01) is None
    assert sampled([5, 20], (4.0, 6.0)) and not sampled([35, 50], (40.0, 42.0))
    text = report(edits, [("test", results)], 3, None, rates=(0.01,))
    # Both edited clips are flagged by every rule, but only the sampled edit counts as found.
    row = next(l for l in text.splitlines() if l.startswith("| generated-splice-2s (found)"))
    assert row.count("1/2 (50.0%) +1 unsampled") == 4, text
    assert "Edits with at least one sampled frame inside: generated-splice-2s: 1/2 (50.0%)." in text, text
    assert "parent timelines" not in text
    shared = {k: dict(e, parent="p0") for k, e in edits.items()}
    assert "Those 2 authentic clips come from 1 parent timelines" in report(shared, [("test", results)], 3, None, rates=(0.01,))
    assert "| reencode | 1 | 0/1 (0.0%) | 0/1 (0.0%) | 0/1 (0.0%) | 0/1 (0.0%) |" in text, text
    # No authentic window is above the cutoff: no false flagged segments; one edit lies in a flagged window.
    assert "0.0/h overall" in text and "Edits inside a flagged window: generated-splice-2s 1/2 (50.0%)." in text, text
    # A plain run: frame 3 of 16 over the first 30 s is at 5.625 s; early exit looked at frames 0 and 7 only.
    order = [0, 7, 3, 5, 1, 2, 4, 6, 8, 12, 10, 14, 9, 13, 11, 15]
    assert default_times({"frames": 16, "frames_expected": 16, "duration_s": 60.0}, 30.0, order)[3] == 5.625
    assert default_times({"frames": 2, "frames_expected": 16, "duration_s": 60.0}, 30.0, order) == [0.0, 13.125]
    assert default_times({"frames": 16, "frames_expected": 16, "duration_s": 60.0}, 0.0, order)[-1] == 56.25
    print("self-test passed")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--edits", help="edits.tsv of the splice store")
    ap.add_argument("--system", action="append", default=[], help="NAME=results.jsonl[@MAX_SECONDS], repeatable")
    ap.add_argument("--video-spec", default="models/tintype-video.json", help="spec whose early-exit frame order plain results were sampled in")
    ap.add_argument("--top-k", type=int, default=3)
    ap.add_argument("--cutoff-on", help="set cutoffs on authentic clips of this kind only (e.g. reencode)")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if not args.edits or not args.system:
        sys.exit("need --edits and at least one --system (or --self-test)")
    order = []
    try:
        order = json.loads(Path(args.video_spec).read_text())["video"]["early_exit"]["order"]
    except (OSError, KeyError, TypeError, ValueError):
        print(f"note: no early-exit order in {args.video_spec}; plain results are taken to have sampled their first frames", file=sys.stderr)
    edits = read_edits(args.edits)
    systems = []
    for spec in args.system:
        name, rest = spec.split("=", 1)
        path, _, seconds = rest.partition("@")
        systems.append((name, read_results(path, float(seconds) if seconds else 30.0, order)))
    sys.stdout.write(report(edits, systems, args.top_k, args.cutoff_on))


if __name__ == "__main__":
    main()
