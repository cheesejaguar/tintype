#!/usr/bin/env python3
"""List the authentic images that controlled reconstructions are made from:
a fixed sample of real items per source, written as a manifest whose rows
carry the ids every reconstruction must keep.

A reconstruction is a real photo passed through a generator's final decoder
(a VAE, a VQ tokenizer, a learned upsampler) with nothing else changed, and
saved at the source's own size and JPEG quality. It shows the detector the
decoder's trace on content it also sees untouched. That only works as a
controlled pair if the reconstruction stays tied to its source: this script
writes, for each chosen parent,

  parent_id   `<dataset>/<path>`: the id every reconstruction of it carries
  scene_id    the same id: a photo and its reconstructions are one scene
  source_id   the real source (from --sources, else the `real:<name>` key)

and the reconstruction pipeline (external; nothing is generated here) must
copy those three columns into every row it emits, add the decoder as
`generator_family` and `transform` (e.g. `recon-vae16`), and keep the size
and JPEG quality of the parent.

Only items that `tintype train` samples from are listed: an item the
path-hash validation split holds out (--val-pct, default 3, as in training)
is skipped, because its reconstruction would put validation content into
training. For that the manifests must be named exactly as they are named to
`tintype train` (e.g. `data/store/openfake/manifest.tsv`, from the
directory that holds `data/`): the hash is of that path.

The draw is deterministic: within a source, items are ordered by a hash of
--seed and the path. Paths in the output are relative to its directory.
Standard library only; runs under `python3 -I`.

  python3 -I scripts/list-reconstruction-parents.py data/store/*/manifest.tsv \\
      --sources docs/groups/sources.tsv --per-source 500 --out data/recon/parents.tsv
  python3 -I scripts/list-reconstruction-parents.py --self-test
"""

import argparse
import sys
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402

IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".webp", ".tif", ".tiff", ".bmp", ".gif"}


def choose(manifests, sources, per_source, val_pct, seed):
    """`(row, source)` of the chosen parents, and how many real images were
    skipped because validation holds them out."""
    by_source = defaultdict(list)
    held_out = 0
    for manifest in manifests:
        for row in mg.read_manifest(manifest):
            if row.label != 0 or Path(row.path).suffix.lower() not in IMAGE_EXTS or row.group(4):
                continue
            if mg.is_val(row.file, val_pct):
                held_out += 1
                continue
            source = row.group(2)
            if not source and sources is not None:
                hit = sources.match(row.generator, row.dataset)
                source = hit[0] if hit else None
            by_source[source or row.source()].append(row)
    chosen = []
    for source in sorted(by_source):
        members = sorted(by_source[source], key=lambda r: (mg.stable_hash(seed, r.file), r.file))
        chosen.extend((r, source) for r in members[:per_source])
    return chosen, held_out


def run(args):
    sources = mg.Rules.load(args.sources) if args.sources else None
    chosen, held_out = choose(args.manifest, sources, args.per_source, args.val_pct, args.seed)
    out = Path(args.out)
    if out.exists() and not args.force:
        raise SystemExit(f"{out} exists; pass --force to replace it")
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="") as stream:
        for row, source in chosen:
            parent = row.group(0) or f"{row.dataset}/{row.path}"
            row.set_group(0, parent)
            row.set_group(1, row.group(1) or parent)
            row.set_group(2, source)
            stream.write(row.line(mg.relative_to(row.file, out.parent)) + "\n")
    count = Counter(source for _, source in chosen)
    print(f"{out}: {len(chosen)} parents from {len(count)} real sources (at most {args.per_source} each, seed {args.seed}); {held_out} real images held out for validation at {args.val_pct}% were not eligible.")
    print("\n| Real source | Parents |\n|---|---|")
    for source in sorted(count):
        print(f"| {source} | {count[source]} |")


def self_test():
    # The path hash is the trainer's: the same vectors are asserted of `is_val` in src/data.rs.
    assert [i for i in range(400) if mg.is_val(f"data/store/openfake/train-00000/{i:07d}.jpg", 3)] == [49, 215, 222, 326, 361]
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        store = tmp / "store" / "a"
        store.mkdir(parents=True)
        text = "".join(f"cam/{i}.jpg\t0\treal:cam\ta\n" for i in range(300))
        text += "".join(f"red/{i}.jpg\t0\treal:openfake-reddit\ta\n" for i in range(40))
        text += "".join(f"ai/{i}.png\t1\tsdxl\ta\n" for i in range(50))
        text += "clip/0.mp4\t0\treal:cam\ta\n" + "cam/copy.jpg\t0\treal:cam\ta\tp\t\t\t\tjpeg-q70\n"
        (store / "manifest.tsv").write_text(text)
        (tmp / "sources.tsv").write_text("real:openfake-reddit\t*\treddit\tdataset-doc\t\n")
        argv = [str(store / "manifest.tsv"), "--sources", str(tmp / "sources.tsv"), "--per-source", "25", "--val-pct", "10", "--out", str(tmp / "recon" / "parents.tsv")]
        run(parser().parse_args(argv))
        rows = mg.read_manifest(tmp / "recon" / "parents.tsv")
        assert Counter(r.group(2) for r in rows) == {"real:cam": 25, "reddit": 25}, Counter(r.group(2) for r in rows)
        assert all(r.label == 0 and r.group(0) == r.group(1) == f"a/{Path(r.file).parent.name}/{Path(r.file).name}" for r in rows)
        assert all(r.path.startswith("../store/a/") and not r.path.endswith(".mp4") and "copy" not in r.path for r in rows)
        # No chosen parent is one the trainer would hold out, given the manifest as it was named.
        assert not any(mg.is_val(str(store / r.group(0)[2:]), 10) for r in rows)
        eligible = sum(not mg.is_val(str(store / f"red/{i}.jpg"), 10) for i in range(40))
        assert 25 <= eligible < 40
        first = (tmp / "recon" / "parents.tsv").read_text()
        run(parser().parse_args(argv + ["--force"]))
        assert (tmp / "recon" / "parents.tsv").read_text() == first
    print("list-reconstruction-parents self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("manifest", nargs="*", help="training manifests, named as they are named to `tintype train`")
    p.add_argument("--sources", help="rules from generator keys to source ids (docs/groups/sources.tsv)")
    p.add_argument("--per-source", type=int, default=500)
    p.add_argument("--val-pct", type=int, default=3, help="the trainer's validation share; items it holds out are skipped")
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--out")
    p.add_argument("--force", action="store_true")
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
