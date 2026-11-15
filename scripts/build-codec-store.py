#!/usr/bin/env python3
"""Plan and collect an offline store of processed training copies.

Learned codecs (JPEG AI, Cheng2020, ...) and restoration or enhancement
tools cannot run inside `tintype`: the runtime is pure Rust and stays so.
They run offline, in whatever environment they need, and this script keeps
the book-keeping on the Tintype side honest:

    plan      pick the photos to process, the same share of every generator
              and real source, in both classes, and write a work list
    collect   check what came back and write the store's manifest, with
              `parent_id` (column 5) and `transform` (column 9) set

Why the same share of every key: `tintype train` draws classes equally and,
within a class, generators by `count^alpha`. A processed copy keeps its
parent's label and generator column, so processing a fraction f of every
key puts the processing in f / (1 + f) of the draws of *both* classes. A
codec applied mostly to one class would teach "codec trace = that class".
`collect` refuses a store whose classes came back at different rates.

Why `parent_id`: `tintype split-groups` keeps items with the same
`parent_id` in one set, and `tintype report --cluster-by parent` resamples
them together. The training manifests have no such column, so `collect`
also writes `parents.tsv`: the originals of the copies that came back, with
the same `parent_id` (the original's absolute path). Splitting
`parents.tsv` and `manifest.tsv` together, with `--existing` pointing at
the split that already placed the originals, sends every copy where its
parent went. For `tintype train`, whose validation share is a hash of each
path, feed `plan` only training-side photos (`tintype hash-split` writes
that list): a processed copy of a held-out photo must not be trained on.

The external tool reads `worklist.tsv` (id, absolute source path) and
writes one decoded image per id, `<outputs>/<id>.png` (or another
extension, see --ext), at the size of its source. Nothing here downloads
or runs a codec. Standard library only.
"""

import argparse
import hashlib
import math
import os
import re
import sys
from pathlib import Path

GROUPS = 5  # parent_id, scene_id, source_id, generator_family, transform
MAGIC = {".png": b"\x89PNG\r\n\x1a\n", ".jpg": b"\xff\xd8", ".jpeg": b"\xff\xd8"}


def read_manifest(path):
    """Rows of a Tintype manifest as (absolute path, label, generator, dataset, grouping columns)."""
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line:
            continue
        fields = line.split("\t")
        if len(fields) < 4 or fields[1] not in {"0", "1"}:
            raise ValueError(f"expected path, label, generator and dataset in {path}: {line!r}")
        extra = fields[4:] + [""] * max(0, GROUPS - len(fields[4:]))
        rows.append((os.path.normpath(os.path.join(path.resolve().parent, fields[0])), fields[1], fields[2], fields[3], extra))
    return rows


def plan(args):
    rows = [row for manifest in args.manifest for row in read_manifest(manifest)]
    if len({row[0] for row in rows}) != len(rows):
        sys.exit("a path is listed twice in the manifests")
    by_key = {}
    for row in rows:
        by_key.setdefault((row[1], row[2]), []).append(row)
    chosen = []
    for key in sorted(by_key):
        # A seeded order per key, independent of manifest order.
        ranked = sorted(by_key[key], key=lambda r: hashlib.sha256(f"{args.seed}\0{r[0]}".encode()).digest())
        take = min(len(ranked), math.ceil(args.fraction * len(ranked)))
        if args.max_per_key:
            take = min(take, args.max_per_key)
        chosen.extend(ranked[:take])
    args.out.mkdir(parents=True)
    lines = []
    for index, (path, label, generator, dataset, extra) in enumerate(chosen):
        # An original is its own parent unless the manifest already names one.
        parent = extra[0] if extra[0] not in {"", "-"} else path
        lines.append("\t".join([f"{index:07d}", path, label, generator, dataset, parent, *extra[1:GROUPS]]) + "\n")
    (args.out / "worklist.tsv").write_text("".join(lines), encoding="utf-8")
    per_class = {label: sum(row[1] == label for row in chosen) for label in "01"}
    totals = {label: sum(row[1] == label for row in rows) for label in "01"}
    for label, name in (("0", "real"), ("1", "generated")):
        share = f"{100 * per_class[label] / totals[label]:.2f}%" if totals[label] else "none listed"
        print(f"{name}: {per_class[label]} of {totals[label]} photos ({share}) from {sum(k[0] == label for k in by_key)} keys")
    if args.max_per_key:
        print("--max-per-key caps large keys, so the processed share differs between keys; compare the two percentages above.")
    print(f"wrote {args.out / 'worklist.tsv'}: id, source path, label, generator, dataset, parent_id, scene_id, source_id, generator_family, earlier transform")


def collect(args):
    if not re.fullmatch(r"(codec|enhance)-[A-Za-z0-9._+-]+", args.transform):
        sys.exit("--transform must look like codec-<name>-<setting> or enhance-<tool>-<setting>")
    work = [line.split("\t") for line in args.worklist.read_text(encoding="utf-8").splitlines() if line]
    outputs = args.outputs.resolve()
    args.store.mkdir(parents=True, exist_ok=True)
    manifest = args.store / "manifest.tsv"
    if manifest.exists():
        sys.exit(f"{manifest} exists; one store per transform")
    ext = args.ext if args.ext.startswith(".") else f".{args.ext}"
    lines, parents, done, planned = [], [], {"0": 0, "1": 0}, {"0": 0, "1": 0}
    for index, original, label, generator, dataset, parent, scene, source_id, family, earlier in (w + [""] * (10 - len(w)) for w in work):
        planned[label] += 1
        produced = outputs / f"{index}{ext}"
        if not produced.is_file():
            continue
        with produced.open("rb") as stream:
            head = stream.read(8)
        if ext in MAGIC and not head.startswith(MAGIC[ext]):
            sys.exit(f"{produced} is not a {ext} file")
        done[label] += 1
        transform = f"{earlier}+{args.transform}" if earlier not in {"", "-"} else args.transform
        lines.append("\t".join([os.path.relpath(produced, args.store.resolve()), label, generator, dataset, parent, scene, source_id, family, transform]) + "\n")
        parents.append("\t".join([original, label, generator, dataset, parent, scene, source_id, family, earlier]).rstrip("\t") + "\n")
    rates = {label: done[label] / planned[label] for label in "01" if planned[label]}
    for label, name in (("0", "real"), ("1", "generated")):
        if planned[label]:
            print(f"{name}: {done[label]} of {planned[label]} planned copies came back ({100 * rates[label]:.2f}%)")
    if not lines:
        sys.exit(f"no output of {outputs} is named like a work-list id (0000000{ext}, ...)")
    if len(rates) == 2 and abs(rates["0"] - rates["1"]) > args.tolerance:
        sys.exit(f"the classes came back at different rates (more than {100 * args.tolerance:.0f} points apart): "
                 "a store like that ties the processing to the label. Finish the missing class, or re-plan.")
    if len(rates) < 2:
        print("warning: the work list holds one class only; do not train on this store without its counterpart", file=sys.stderr)
    manifest.write_text("".join(lines), encoding="utf-8")
    (args.store / "parents.tsv").write_text("".join(parents), encoding="utf-8")
    print(f"wrote {manifest}: {len(lines)} copies, transform {args.transform}")
    print(f"wrote {args.store / 'parents.tsv'}: their originals with the same parent_id, for split-groups and verify-split (not for training: the training manifests already list them)")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    p = commands.add_parser("plan", help="choose the photos to process and write worklist.tsv")
    p.add_argument("--manifest", type=Path, action="append", required=True, help="training-side manifest (repeatable)")
    p.add_argument("--fraction", type=float, required=True, help="share of every generator and real source to process, e.g. 0.25")
    p.add_argument("--max-per-key", type=int, default=0, help="cap per generator or real source (0 = none); breaks the equal share, see above")
    p.add_argument("--seed", default="1")
    p.add_argument("--out", type=Path, required=True, help="new directory for worklist.tsv")
    p.set_defaults(run=plan)
    c = commands.add_parser("collect", help="check the processed files and write the store manifest")
    c.add_argument("--worklist", type=Path, required=True)
    c.add_argument("--outputs", type=Path, required=True, help="directory with <id>.<ext> files from the external tool")
    c.add_argument("--transform", required=True, help="what was done, e.g. codec-jpegai-q2, codec-cheng2020-q3, enhance-swinir-x2")
    c.add_argument("--store", type=Path, required=True, help="directory for the store's manifest.tsv")
    c.add_argument("--ext", default=".png")
    c.add_argument("--tolerance", type=float, default=0.02, help="largest allowed gap between the classes' completion rates [0.02]")
    c.set_defaults(run=collect)
    args = parser.parse_args()
    if args.command == "plan" and not 0 < args.fraction <= 1:
        parser.error("--fraction must be within (0, 1]")
    args.run(args)


if __name__ == "__main__":
    main()
