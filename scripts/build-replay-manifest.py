#!/usr/bin/env python3
"""Build a replay manifest for a refresh: a fixed, stratified sample of the
older stores (so many images per generator and per real source) plus every
reposted or processed copy, written as one new manifest.

A refresh that trains only on new generators forgets old ones. Training on
every old store again costs a full pass each time, and needs stores that
may since have been deleted. A replay buffer is the middle: a few hundred
images per old generator and per real source, kept, and drawn at a fixed
share of the batches (`tintype train --replay-manifest replay.tsv
--replay-share 0.2`).

Selection is deterministic: within a generator or a real source, items are
ordered by a hash of --seed and the path, and the first N are taken. Rows
whose `transform` column is set (reposts, recompressed or resized copies)
and every row of a --repost-manifest are kept whole unless --per-transform
caps them: the reposted images are what a refresh most easily loses.

The output lists files where they are; nothing is copied. Paths are written
relative to the output's directory, and the grouping columns are kept, with
`generator_family` filled from --families where it is empty. To keep the
buffer when a store is deleted, keep the listed files, and record them with
`tintype freeze-set --manifest replay.tsv --out replay.frozen.tsv.gz`.

Standard library only; runs under `python3 -I`.

  python3 -I scripts/build-replay-manifest.py data/store/*/manifest.tsv \\
      --families docs/groups/generator-families.tsv \\
      --per-generator 100 --per-source 300 --out data/replay/replay.tsv
  python3 -I scripts/build-replay-manifest.py --self-test
"""

import argparse
import sys
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402


def select(rows, repost_rows, per_generator, per_source, per_transform, seed):
    """The rows of the buffer, in input order, and why each is there."""
    order = lambda r: (mg.stable_hash(seed, r.file), r.file)  # noqa: E731
    chosen = {}
    originals = defaultdict(list)
    processed = defaultdict(list)
    for r in rows:
        if r.group(4):
            processed[r.group(4)].append(r)
        else:
            originals[(r.label, r.generator)].append(r)
    for (label, _), members in originals.items():
        cap = per_generator if label == 1 else per_source
        for r in sorted(members, key=order)[:cap]:
            chosen[id(r)] = "generator" if label == 1 else "source"
    for members in processed.values():
        for r in (sorted(members, key=order)[:per_transform] if per_transform else members):
            chosen[id(r)] = "transform"
    for r in repost_rows:
        chosen[id(r)] = "repost"
    kept = [(r, chosen[id(r)]) for r in list(rows) + list(repost_rows) if id(r) in chosen]
    seen, out = set(), []
    for r, why in kept:
        if r.file not in seen:
            seen.add(r.file)
            out.append((r, why))
    return out


def run(args):
    families = mg.Rules.load(args.families) if args.families else None
    rows = [r for m in args.manifest for r in mg.read_manifest(m)]
    repost_rows = [r for m in args.repost_manifest or [] for r in mg.read_manifest(m)]
    picked = select(rows, repost_rows, args.per_generator, args.per_source, args.per_transform, args.seed)
    out = Path(args.out)
    if out.exists() and not args.force:
        raise SystemExit(f"{out} exists; pass --force to replace it")
    if any(out.resolve() == Path(m).resolve() for m in list(args.manifest) + list(args.repost_manifest or [])):
        raise SystemExit(f"{out}: the replay manifest is never written over an input")
    out.parent.mkdir(parents=True, exist_ok=True)
    by_family, by_source, why = Counter(), Counter(), Counter()
    with out.open("w", encoding="utf-8", newline="") as stream:
        for r, reason in picked:
            if r.label == 1:
                if families is not None and not r.group(3):
                    r.set_group(3, mg.family_of(r, families)[0])
                by_family[r.family() if r.group(3) else r.generator] += 1
            else:
                by_source[r.source()] += 1
            why[reason] += 1
            stream.write(r.line(mg.relative_to(r.file, out.parent)) + "\n")
    total = len(rows) + len(repost_rows)
    print(f"{out}: {len(picked)} of {total} rows ({sum(by_source.values())} real from {len(by_source)} sources, {sum(by_family.values())} AI); seed {args.seed}, at most {args.per_generator} per generator and {args.per_source} per real source.")
    print("kept as: " + ", ".join(f"{n} {k}" for k, n in sorted(why.items())))
    print("\n| " + ("Generator family" if families else "Generator") + " | AI items |\n|---|---|")
    rows_out = sorted(by_family.items(), key=lambda kv: (-kv[1], kv[0]))
    for name, n in rows_out[: args.top]:
        print(f"| {name} | {n} |")
    if len(rows_out) > args.top:
        print(f"| ({len(rows_out) - args.top} others) | {sum(n for _, n in rows_out[args.top:])} |")
    print("\n| Real source | Items |\n|---|---|")
    for name, n in sorted(by_source.items(), key=lambda kv: (-kv[1], kv[0])):
        print(f"| {name} | {n} |")


def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        store = tmp / "store" / "old"
        store.mkdir(parents=True)
        text = "".join(f"big/{i}.png\t1\tstylegan2\told\n" for i in range(500))
        text += "".join(f"small/{i}.png\t1\tsdxl\told\n" for i in range(7))
        text += "".join(f"cam/{i}.jpg\t0\treal:cam\told\n" for i in range(300))
        text += "".join(f"re/{i}.jpg\t1\tsdxl\told\tp{i}\t\t\t\trepost-960-q70\n" for i in range(40))
        (store / "manifest.tsv").write_text(text)
        social = tmp / "store" / "social"
        social.mkdir()
        (social / "manifest.tsv").write_text("".join(f"{i}.jpg\t{i % 2}\t{'reddit-wild' if i % 2 else 'real:reddit'}\tsocial\n" for i in range(30)))
        (tmp / "families.tsv").write_text("stylegan*\t*\tgan\tname\t\nsdxl*\t*\tlatent-diffusion\tname\t\n")
        argv = [str(store / "manifest.tsv"), "--repost-manifest", str(social / "manifest.tsv"), "--families", str(tmp / "families.tsv"), "--per-generator", "50", "--per-source", "20", "--out", str(tmp / "replay" / "replay.tsv")]
        run(parser().parse_args(argv))
        first = (tmp / "replay" / "replay.tsv").read_text()
        rows = mg.read_manifest(tmp / "replay" / "replay.tsv")
        count = Counter((r.generator, bool(r.group(4))) for r in rows)
        # Capped originals, a small generator whole, every processed copy and every repost row.
        assert count == {("stylegan2", False): 50, ("sdxl", False): 7, ("real:cam", False): 20, ("sdxl", True): 40, ("reddit-wild", False): 15, ("real:reddit", False): 15}, count
        assert all((tmp / "store").resolve() in Path(r.file).resolve().parents for r in rows)
        assert rows[0].path.startswith("../store/old/")
        assert {r.group(3) for r in rows if r.generator == "stylegan2"} == {"gan"} and {r.group(3) for r in rows if r.generator == "reddit-wild"} == {"unknown"}
        kept = next(r for r in rows if r.group(4))
        assert kept.group(0).startswith("p") and kept.group(4) == "repost-960-q70" and kept.group(3) == "latent-diffusion"
        # The same seed gives the same buffer; another seed another sample of the capped groups.
        try:
            run(parser().parse_args(argv))
            raise AssertionError("an existing replay manifest was overwritten")
        except SystemExit:
            pass
        run(parser().parse_args(argv + ["--force"]))
        assert (tmp / "replay" / "replay.tsv").read_text() == first
        run(parser().parse_args(argv + ["--force", "--seed", "2", "--per-transform", "10"]))
        other = mg.read_manifest(tmp / "replay" / "replay.tsv")
        assert {r.path for r in other if r.generator == "stylegan2"} != {r.path for r in rows if r.generator == "stylegan2"}
        assert sum(bool(r.group(4)) for r in other) == 10
    print("build-replay-manifest self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("manifest", nargs="*", help="manifests of the older stores to sample")
    p.add_argument("--repost-manifest", nargs="*", help="manifests kept whole: reposted and in-the-wild images")
    p.add_argument("--families", help="rules from generator names to families, to fill column 8")
    p.add_argument("--per-generator", type=int, default=100, help="unprocessed AI items kept per generator name")
    p.add_argument("--per-source", type=int, default=300, help="unprocessed real items kept per real source")
    p.add_argument("--per-transform", type=int, default=0, help="processed copies kept per transform (0 = all)")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--out", help="the replay manifest to write")
    p.add_argument("--force", action="store_true", help="replace a replay manifest made earlier")
    p.add_argument("--top", type=int, default=20, help="rows of the family table")
    p.add_argument("--self-test", action="store_true")
    return p


def main():
    args = parser().parse_args()
    if args.self_test:
        return self_test()
    if not args.manifest or not args.out:
        raise SystemExit("pass manifests and --out, or --self-test")
    run(args)


if __name__ == "__main__":
    main()
