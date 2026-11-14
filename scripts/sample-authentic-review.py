#!/usr/bin/env python3
"""Draw a review sample of authentic-labelled items from a score file: a few
from every source and score band, and the high-score tail.

Collections of "real" photos gathered from the web after 2022 can contain
AI-generated images. In training they teach the detector that a generator
is real; in calibration they push a threshold up to clear "real" files that
are not. Looking only at the highest-scoring authentic items finds the ones
the detector already suspects. Looking across every source and score band
as well finds the ones it does not, and gives a contamination rate per
source that the tail alone cannot.

This script only writes a list for a person to look at. It relabels
nothing and changes no manifest.

Input: one or more score files as `tintype score` writes them
(`path label generator dataset prob ...`). Only rows with label 0 are used.
The source of an item is its `source_id` if --manifest gives one (manifests
with grouping columns, matched to the scored paths by their trailing path
components), or else its `real:<source>` generator key.

Output (--out, tab-separated, with a header):

  reason        `stratum`, `tail`, `source-tail`, or several joined by `+`
  source, band  the stratum; bands are score ranges set by --bands
  stratum_size  authentic items in that stratum
  stratum_drawn how many of them the stratum draw took
  score         the detector's score
  path, generator, dataset
  review        empty: for the reviewer (authentic / generated / unsure)

A stratum's contamination estimate is (generated among its stratum draws) /
stratum_drawn, and a source's is the stratum_size-weighted mean; tail rows
are a targeted look and do not enter that estimate.

The draw is deterministic: within a stratum, items are ordered by a hash of
--seed and the path. Standard library only; runs under `python3 -I`.

  python3 -I scripts/sample-authentic-review.py --scores runs/ws4/real-scores.tsv \\
      --out runs/ws4/authentic-review.tsv --per-stratum 5 --tail 200 --tail-per-source 20
  python3 -I scripts/sample-authentic-review.py --self-test
"""

import argparse
import sys
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402


def read_scores(paths):
    """`(path, generator, dataset, score)` of the authentic rows."""
    rows = []
    for path in paths:
        with open(path, encoding="utf-8") as stream:
            for number, raw in enumerate(stream, 1):
                f = raw.rstrip("\n").split("\t")
                if len(f) < 5:
                    raise ValueError(f"{path}:{number}: expected `path label generator dataset prob ...`")
                if f[1] == "0":
                    rows.append((f[0], f[2], f[3], float(f[4])))
    return rows


def source_ids(manifests, depth=8):
    """`{trailing path components: source_id}` for manifest rows that carry one."""
    table = {}
    for manifest in manifests:
        for row in mg.read_manifest(manifest):
            if row.group(2):
                parts = Path(row.file).parts
                for k in range(2, min(depth, len(parts)) + 1):
                    table.setdefault("/".join(parts[-k:]), row.group(2))
    return table


def lookup_source(path, generator, table, depth=8):
    if table:
        parts = Path(path).parts
        for k in range(min(depth, len(parts)), 1, -1):
            hit = table.get("/".join(parts[-k:]))
            if hit:
                return hit
    return generator


def band_of(score, edges):
    for k, edge in enumerate(edges):
        if score < edge:
            return f"<{edge:g}" if k == 0 else f"{edges[k - 1]:g}-{edge:g}"
    return f">={edges[-1]:g}"


def draw(rows, table, edges, per_stratum, tail, tail_per_source, seed):
    """The review rows, highest score first, and the per-stratum counts."""
    items = []
    for path, generator, dataset, score in rows:
        source = lookup_source(path, generator, table)
        items.append({"path": path, "generator": generator, "dataset": dataset, "score": score, "source": source, "band": band_of(score, edges), "reasons": []})
    strata = defaultdict(list)
    for it in items:
        strata[(it["source"], it["band"])].append(it)
    counts = {}
    for key, members in strata.items():
        members.sort(key=lambda it: (mg.stable_hash(seed, it["path"]), it["path"]))
        taken = members[:per_stratum]
        for it in taken:
            it["reasons"].append("stratum")
        counts[key] = (len(members), len(taken))
    ranked = sorted(items, key=lambda it: (-it["score"], it["path"]))
    for it in ranked[:tail]:
        it["reasons"].append("tail")
    seen = Counter()
    for it in ranked:
        if seen[it["source"]] < tail_per_source:
            seen[it["source"]] += 1
            it["reasons"].append("source-tail")
    chosen = [it for it in ranked if it["reasons"]]
    for it in chosen:
        it["stratum_size"], it["stratum_drawn"] = counts[(it["source"], it["band"])]
    return chosen, counts


HEADER = ["reason", "source", "band", "stratum_size", "stratum_drawn", "score", "path", "generator", "dataset", "review"]


def write(chosen, out):
    with open(out, "w", encoding="utf-8", newline="") as stream:
        stream.write("\t".join(HEADER) + "\n")
        for it in chosen:
            stream.write("\t".join(["+".join(it["reasons"]), it["source"], it["band"], str(it["stratum_size"]), str(it["stratum_drawn"]), f"{it['score']:.6f}", it["path"], it["generator"], it["dataset"], ""]) + "\n")


def summary(chosen, counts, edges):
    bands = [band_of(e - 1e-12, edges) for e in edges] + [band_of(edges[-1], edges)]
    sources = sorted({k[0] for k in counts})
    lines = ["| Source | Authentic items | " + " | ".join(bands) + " | In the review list |", "|---|---|" + "---|" * (len(bands) + 1)]
    listed = Counter(it["source"] for it in chosen)
    for s in sources:
        cells = [f"{counts[(s, b)][1]} of {counts[(s, b)][0]}" if (s, b) in counts else "–" for b in bands]
        lines.append(f"| {s} | {sum(counts[(s, b)][0] for b in bands if (s, b) in counts)} | " + " | ".join(cells) + f" | {listed[s]} |")
    return "\n".join(lines)


def run(args):
    edges = sorted(float(x) for x in args.bands.split(","))
    rows = read_scores(args.scores)
    if not rows:
        raise SystemExit("the score files hold no authentic (label 0) rows")
    table = source_ids(args.manifest) if args.manifest else {}
    chosen, counts = draw(rows, table, edges, args.per_stratum, args.tail, args.tail_per_source, args.seed)
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    write(chosen, args.out)
    reasons = Counter(r for it in chosen for r in it["reasons"])
    print(f"{len(rows)} authentic-labelled items in {len({k[0] for k in counts})} sources and {len(counts)} source-by-band strata; seed {args.seed}.")
    print(f"{args.out}: {len(chosen)} items to review ({reasons['stratum']} from the strata, {reasons['tail']} in the overall tail, {reasons['source-tail']} in a source's tail; an item can be several).")
    print("Nothing was relabelled.\n\nStratum draws per score band (drawn of present):\n")
    print(summary(chosen, counts, edges))


def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        lines = []
        for i in range(600):
            # Three sources; scores rise with i inside a source. Some AI rows that must be ignored.
            source = ["real:cam", "real:reddit", "real:web"][i % 3]
            lines.append(f"/data/store/s/{source[5:]}/{i:04d}.jpg\t0\t{source}\ts\t{i / 600:.6f}\t0.0")
            lines.append(f"/data/store/s/ai/{i:04d}.png\t1\tgen\ts\t0.990000\t5.0")
        scores = tmp / "scores.tsv"
        scores.write_text("\n".join(lines) + "\n")
        argv = ["--scores", str(scores), "--out", str(tmp / "review.tsv"), "--per-stratum", "3", "--tail", "10", "--tail-per-source", "5", "--bands", "0.1,0.5,0.9"]
        run(parser().parse_args(argv))
        first = (tmp / "review.tsv").read_text()
        rows = [dict(zip(HEADER, l.split("\t"))) for l in first.splitlines()[1:]]
        assert all(r["generator"].startswith("real:") and r["review"] == "" for r in rows)
        # Every one of the 12 strata gives 3 items.
        strata = Counter((r["source"], r["band"]) for r in rows if "stratum" in r["reason"])
        assert len(strata) == 12 and set(strata.values()) == {3}, strata
        assert {r["band"] for r in rows} == {"<0.1", "0.1-0.5", "0.5-0.9", ">=0.9"}
        low = next(r for r in rows if r["band"] == "<0.1" and r["source"] == "real:cam")
        assert (low["stratum_size"], low["stratum_drawn"]) == ("20", "3"), low
        # The ten highest scores overall, and the five highest of each source, are all there.
        top = sorted((float(l.split("\t")[4]), l.split("\t")[0]) for l in lines if l.split("\t")[1] == "0")[-10:]
        tail = {r["path"] for r in rows if "tail" in r["reason"].split("+")}
        assert tail == {p for _, p in top}, tail
        assert Counter(r["source"] for r in rows if "source-tail" in r["reason"]) == {"real:cam": 5, "real:reddit": 5, "real:web": 5}
        assert [float(r["score"]) for r in rows] == sorted((float(r["score"]) for r in rows), reverse=True)
        assert len(rows) == len({r["path"] for r in rows})
        # Same seed, same list; another seed, another stratum draw with the same tail.
        run(parser().parse_args(argv))
        assert (tmp / "review.tsv").read_text() == first
        run(parser().parse_args(argv + ["--seed", "2"]))
        second = (tmp / "review.tsv").read_text()
        assert second != first and {l.split("\t")[6] for l in second.splitlines()[1:] if "tail" in l.split("\t")[0].split("+")} == tail
        # A manifest with a source_id column regroups the items it lists.
        store = tmp / "store" / "s"
        store.mkdir(parents=True)
        (store / "manifest.tsv").write_text("".join(f"cam/{i:04d}.jpg\t0\treal:cam\ts\t\t\tphone-{i % 2}\n" for i in range(0, 600, 3)))
        run(parser().parse_args(argv + ["--manifest", str(store / "manifest.tsv")]))
        sources = {l.split("\t")[1] for l in (tmp / "review.tsv").read_text().splitlines()[1:]}
        assert sources == {"phone-0", "phone-1", "real:reddit", "real:web"}, sources
    print("sample-authentic-review self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--scores", nargs="+", help="score files from `tintype score`")
    p.add_argument("--manifest", nargs="*", help="manifests with a source_id column, to name sources")
    p.add_argument("--out", help="the review list to write")
    p.add_argument("--bands", default="0.01,0.1,0.5,0.9,0.99", help="score edges between bands")
    p.add_argument("--per-stratum", type=int, default=5, help="items drawn from each source-by-band stratum")
    p.add_argument("--tail", type=int, default=200, help="the highest-scoring authentic items overall")
    p.add_argument("--tail-per-source", type=int, default=20, help="the highest-scoring items of each source")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--self-test", action="store_true")
    return p


def main():
    args = parser().parse_args()
    if args.self_test:
        return self_test()
    if not args.scores or not args.out:
        raise SystemExit("pass --scores and --out, or --self-test")
    run(args)


if __name__ == "__main__":
    main()
