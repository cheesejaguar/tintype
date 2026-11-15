#!/usr/bin/env python3
"""Write a grouped copy of a four-column manifest: the same rows with the
grouping columns filled in (parent_id, scene_id, source_id, generator_family,
transform; manifest columns 5 to 9).

The input manifest is never changed. The evaluation manifests are pinned by
SHA-256 in docs/manifests/*.frozen.tsv.gz, so a grouped copy is always a NEW
file: by default `<name>.grouped.tsv` beside the input, which keeps every
relative path, and so the item paths and the path-hash validation split of
`tintype train`, exactly as they were. With --out-dir the copies go
elsewhere and their paths are rewritten relative to that directory; the
item paths then differ, and with them the path-hash split.

What fills the columns:

  generator_family  --families: rules from generator names to decoder
                    families (docs/groups/generator-families.tsv). Generated
                    items only; an unmatched name is `unknown`.
  source_id         --sources: rules from generator keys to a source
                    (docs/groups/sources.tsv), for both classes. Unmatched
                    items keep the column empty, which `tintype` reads as
                    `real:<name>` for real items and the dataset otherwise.
  parent_id, scene_id, transform (and overrides of the other two)
                    --side-table: a tab-separated file with a header row
                    naming `path` and any of the five columns; `path` is the
                    manifest's first column. --self-parent gives every item
                    without a parent its own id, `<dataset>/<path>`, so that
                    copies made later can point at it.

A value already present in the input wins over a rule; a side-table value
wins over both.

--summary prints, per family, how many generator names and items it holds
and what the assignments rest on. --write-map writes the resolved
`generator<TAB>family<TAB>basis<TAB>items` table for `tintype train
--family-map` and `tintype sampling-shares --family-map`, which lets the
four-column manifests be weighted by family without a grouped copy.

Standard library only; runs under `python3 -I`.

  python3 -I scripts/group-manifest.py data/store/*/manifest.tsv \\
      --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv \\
      --summary --write-map runs/ws4/families.tsv --dry-run
  python3 -I scripts/group-manifest.py --self-test
"""

import argparse
import sys
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402


def read_side_table(path):
    """`{manifest path: {column index: value}}` from a table with a header row."""
    table = {}
    with open(path, encoding="utf-8") as stream:
        header = stream.readline().rstrip("\n").split("\t")
        if "path" not in header:
            raise ValueError(f"{path}: the header row must name a `path` column")
        unknown = [h for h in header if h != "path" and h not in mg.GROUP_COLUMNS]
        if unknown:
            raise ValueError(f"{path}: unknown columns {unknown}; use path and any of {mg.GROUP_COLUMNS}")
        for number, raw in enumerate(stream, 2):
            line = raw.rstrip("\n")
            if not line:
                continue
            f = line.split("\t")
            if len(f) > len(header):
                raise ValueError(f"{path}:{number}: more fields than the header names")
            values = dict(zip(header, f))
            key = values.pop("path")
            if key in table:
                raise ValueError(f"{path}:{number}: {key} is listed twice")
            table[key] = {mg.GROUP_COLUMNS.index(name): value for name, value in values.items() if value and value != "-"}
    return table


def group_rows(rows, families, sources, side, self_parent):
    """Fill the grouping columns of `rows` in place. Returns the basis of each
    generated row's family and the side-table paths that matched no row."""
    bases = []
    used = set()
    for row in rows:
        if row.label == 1:
            family, basis = mg.family_of(row, families)
            if families is not None or basis == "column":
                row.set_group(3, family)
            bases.append(basis)
        else:
            bases.append(None)
        if sources is not None and not row.group(2):
            hit = sources.match(row.generator, row.dataset)
            if hit:
                row.set_group(2, hit[0])
        extra = side.get(row.path) if side else None
        if extra:
            used.add(row.path)
            for k, value in extra.items():
                row.set_group(k, value)
        if self_parent and not row.group(0):
            row.set_group(0, f"{row.dataset}/{row.path}")
    unmatched = sorted(set(side) - used) if side else []
    return bases, unmatched


def summarise(all_rows, all_bases):
    """Per family: generator names, items, and the bases they rest on."""
    names = defaultdict(set)
    items = Counter()
    basis_names = defaultdict(set)
    basis_items = Counter()
    for row, basis in zip(all_rows, all_bases):
        if row.label != 1:
            continue
        family = row.group(3) or "unknown"
        names[family].add(row.generator)
        items[family] += 1
        basis_names[basis].add(row.generator)
        basis_items[basis] += 1
    return names, items, basis_names, basis_items


def print_summary(all_rows, all_bases, out=sys.stdout):
    names, items, basis_names, basis_items = summarise(all_rows, all_bases)
    total_items = sum(items.values())
    total_names = len(set().union(*names.values())) if names else 0
    real = sum(1 for r in all_rows if r.label == 0)
    print(f"{len(all_rows)} items: {real} real, {total_items} generated under {total_names} generator names", file=out)
    print("\n| Family | Generator names | Items | Share of generated items |\n|---|---|---|---|", file=out)
    for family in sorted(items, key=lambda f: (-items[f], f)):
        print(f"| {family} | {len(names[family])} | {items[family]} | {100.0 * items[family] / max(total_items, 1):.1f}% |", file=out)
    print("\n| Basis of the assignment | Generator names | Items |\n|---|---|---|", file=out)
    for basis in sorted(basis_items, key=lambda b: (-basis_items[b], str(b))):
        print(f"| {basis} | {len(basis_names[basis])} | {basis_items[basis]} |", file=out)


def resolved_map(all_rows, all_bases):
    """`{generator: (family, basis, items)}`; fails if one name got two families."""
    out = {}
    for row, basis in zip(all_rows, all_bases):
        if row.label != 1:
            continue
        family = row.group(3) or "unknown"
        old = out.get(row.generator)
        if old and old[0] != family:
            raise ValueError(f"generator {row.generator} is {old[0]} in one dataset and {family} in another: the exact-name map cannot hold both; use grouped copies instead")
        out[row.generator] = (family, basis, (old[2] if old else 0) + 1)
    return out


def output_path(manifest, out_dir, suffix):
    manifest = Path(manifest)
    if out_dir is None:
        return manifest.with_name(manifest.stem + suffix)
    return Path(out_dir) / (mg.manifest_name(manifest) + suffix)


def run(args):
    families = mg.Rules.load(args.families) if args.families else None
    sources = mg.Rules.load(args.sources) if args.sources else None
    side = read_side_table(args.side_table) if args.side_table else None
    if side is not None and len(args.manifest) != 1:
        raise SystemExit("--side-table is keyed by the paths of one manifest: pass one manifest with it")
    all_rows, all_bases = [], []
    for manifest in args.manifest:
        rows = mg.read_manifest(manifest)
        bases, unmatched = group_rows(rows, families, sources, side, args.self_parent)
        if unmatched:
            raise SystemExit(f"{args.side_table}: {len(unmatched)} paths are not in {manifest}, e.g. {unmatched[0]}")
        all_rows.extend(rows)
        all_bases.extend(bases)
        if args.dry_run:
            continue
        out = output_path(manifest, args.out_dir, args.suffix)
        if out.resolve() == Path(manifest).resolve():
            raise SystemExit(f"{out}: a grouped copy is never written over its input")
        if out.exists() and not args.force:
            raise SystemExit(f"{out} exists; pass --force to replace a grouped copy made earlier")
        out.parent.mkdir(parents=True, exist_ok=True)
        with out.open("w", encoding="utf-8", newline="") as stream:
            for row in rows:
                path = row.path if args.out_dir is None else mg.relative_to(row.file, out.parent)
                stream.write(row.line(path) + "\n")
        print(f"{out}: {len(rows)} rows", file=sys.stderr)
    if args.summary:
        print_summary(all_rows, all_bases)
    if args.write_map:
        table = resolved_map(all_rows, all_bases)
        Path(args.write_map).parent.mkdir(parents=True, exist_ok=True)
        with open(args.write_map, "w", encoding="utf-8", newline="") as stream:
            stream.write("# generator\tfamily\tbasis\titems\n")
            stream.write(f"# resolved by scripts/group-manifest.py from {args.families} over {len(args.manifest)} manifests\n")
            for generator in sorted(table):
                family, basis, n = table[generator]
                stream.write(f"{generator}\t{family}\t{basis}\t{n}\n")
        print(f"{args.write_map}: {len(table)} generator names", file=sys.stderr)


def self_test():
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        store = tmp / "store" / "demo"
        store.mkdir(parents=True)
        text = (
            "a/0.jpg\t0\treal:openfake-reddit\tdemo\n"
            "a/1.png\t1\tsdxl-turbo\tdemo\n"
            "a/2.png\t1\tStyleGAN3\tdemo\n"
            "a/3.png\t1\tsomeone/my-pet-dog\tcommfor-small\n"
            "a/4.png\t1\tsomeone/vegan-organ\tdemo\n"
            "a/5.png\t1\tVQGAN\tdemo\n"
            "a/6.png\t1\treddit-wild\tdemo\n"
            "a/7.png\t1\tflux\tdemo\t\t\t\tkept-family\n"
            "a/8.jpg\t0\treal:cam\tdemo\n"
        )
        (store / "manifest.tsv").write_text(text)
        fam = tmp / "families.tsv"
        fam.write_text(
            "# comment\n"
            "reddit-wild\t*\tunknown\tnone\tno generator recorded\n"
            "*/*\tcommfor-small\tlatent-diffusion\tdataset-doc\thub repositories\n"
            "vqgan\t*\tautoregressive-vq\tname\t\n"
            "stylegan*\t*\tgan\tname\t\n"
            "sdxl*\t*\tlatent-diffusion\tname\t\n"
        )
        src = tmp / "sources.tsv"
        src.write_text("real:openfake-reddit\t*\treddit\tdataset-doc\t\nreddit-wild\t*\treddit\tdataset-doc\t\n")
        side = tmp / "side.tsv"
        side.write_text("path\tparent_id\ttransform\na/1.png\tdemo/a/8.jpg\tvae-sdxl\n")

        def parse(argv):
            return parser().parse_args(argv)

        # Beside the input: same relative paths, the input untouched.
        run(parse([str(store / "manifest.tsv"), "--families", str(fam), "--sources", str(src), "--side-table", str(side), "--self-parent", "--write-map", str(tmp / "map.tsv")]))
        assert (store / "manifest.tsv").read_text() == text, "the input manifest changed"
        rows = mg.read_manifest(store / "manifest.grouped.tsv")
        assert [r.path for r in rows] == [f"a/{i}.{'jpg' if i in (0, 8) else 'png'}" for i in range(9)]
        by = {r.path: r for r in rows}
        assert (by["a/0.jpg"].group(2), by["a/0.jpg"].group(3)) == ("reddit", None)
        assert (by["a/6.png"].group(2), by["a/6.png"].group(3)) == ("reddit", "unknown")
        assert by["a/1.png"].extra == ["demo/a/8.jpg", "", "", "latent-diffusion", "vae-sdxl"], by["a/1.png"].extra
        assert by["a/2.png"].group(3) == "gan" and by["a/5.png"].group(3) == "autoregressive-vq"
        # The hub rule applies in its dataset only; elsewhere an unmatched name is unknown, not a guess.
        assert by["a/3.png"].group(3) == "latent-diffusion" and by["a/4.png"].group(3) == "unknown"
        assert by["a/7.png"].group(3) == "kept-family"
        assert by["a/8.jpg"].group(0) == "demo/a/8.jpg" and by["a/8.jpg"].source() == "real:cam"
        assert by["a/2.png"].group(0) == "demo/a/2.png"
        table = dict(l.split("\t")[:2] for l in (tmp / "map.tsv").read_text().splitlines() if not l.startswith("#"))
        assert table == {"sdxl-turbo": "latent-diffusion", "StyleGAN3": "gan", "someone/my-pet-dog": "latent-diffusion", "someone/vegan-organ": "unknown", "VQGAN": "autoregressive-vq", "reddit-wild": "unknown", "flux": "kept-family"}, table
        # It does not overwrite an earlier copy unless told to, and never the input.
        try:
            run(parse([str(store / "manifest.tsv"), "--families", str(fam)]))
            raise AssertionError("an existing grouped copy was overwritten")
        except SystemExit:
            pass
        try:
            run(parse([str(store / "manifest.tsv"), "--suffix", ".tsv", "--force"]))
            raise AssertionError("the input was overwritten")
        except SystemExit:
            pass
        # Elsewhere: paths are rewritten so that they still point at the files.
        run(parse([str(store / "manifest.tsv"), "--families", str(fam), "--out-dir", str(tmp / "grouped")]))
        moved = mg.read_manifest(tmp / "grouped" / "demo.grouped.tsv")
        assert moved[0].path == "../store/demo/a/0.jpg" and Path(moved[0].file).resolve() == (store / "a/0.jpg").resolve()
        # A manifest with no extra columns and no rules is written back byte for byte.
        run(parse([str(store / "manifest.tsv"), "--suffix", ".copy.tsv"]))
        assert (store / "manifest.copy.tsv").read_text() == text
        # A side table that names a path the manifest lacks is an error.
        side.write_text("path\tscene_id\nmissing.png\ts1\n")
        try:
            run(parse([str(store / "manifest.tsv"), "--side-table", str(side), "--dry-run"]))
            raise AssertionError("an unknown side-table path was accepted")
        except SystemExit:
            pass
        # The same name with two families cannot go into an exact-name map.
        (store / "two.tsv").write_text("x.png\t1\ta/b\tcommfor-small\ny.png\t1\ta/b\tother\n")
        try:
            run(parse([str(store / "two.tsv"), "--families", str(fam), "--dry-run", "--write-map", str(tmp / "bad.tsv")]))
            raise AssertionError("a conflicting map was written")
        except ValueError:
            pass
    print("group-manifest self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("manifest", nargs="*", help="manifests to copy")
    p.add_argument("--families", help="rules from generator names to families (docs/groups/generator-families.tsv)")
    p.add_argument("--sources", help="rules from generator keys to source ids (docs/groups/sources.tsv)")
    p.add_argument("--side-table", help="per-row parent_id, scene_id, source_id, generator_family, transform, keyed by path")
    p.add_argument("--self-parent", action="store_true", help="give every item without a parent_id its own id, <dataset>/<path>")
    p.add_argument("--out-dir", help="write <store>.grouped.tsv here, with paths rewritten, instead of beside each input")
    p.add_argument("--suffix", default=".grouped.tsv", help="ending of the copy's name (default: .grouped.tsv)")
    p.add_argument("--force", action="store_true", help="replace a grouped copy made earlier")
    p.add_argument("--dry-run", action="store_true", help="write no manifests (the summary and the map are still written)")
    p.add_argument("--summary", action="store_true", help="print keys and items per family, and what the assignments rest on")
    p.add_argument("--write-map", help="write the resolved generator-to-family table for `tintype --family-map`")
    p.add_argument("--self-test", action="store_true")
    return p


def main():
    args = parser().parse_args()
    if args.self_test:
        return self_test()
    if not args.manifest:
        raise SystemExit("pass at least one manifest, or --self-test")
    run(args)


if __name__ == "__main__":
    main()
