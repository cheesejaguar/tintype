#!/usr/bin/env python3
"""Create paired JPEG stress inputs from a Tintype photo manifest."""

import argparse
import os
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# The helper module sits beside this script; find it under `python3 -I` too.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from benchmark_images import encode_jpeg  # noqa: E402


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("--jobs must be positive")

    source = args.manifest.resolve()
    rows = []
    seen = set()
    for line in source.read_text(encoding="utf-8").splitlines():
        fields = line.split("\t")
        if len(fields) < 4:
            parser.error(f"expected path, label, generator and dataset in {source}: {line!r}")
        # Columns 5 and up (parent_id, scene_id, source_id, generator_family,
        # transform, ...) are carried through to every variant unchanged.
        path, label, generator, dataset, *extra = fields
        if path in seen or label not in {"0", "1"}:
            parser.error(f"duplicate path or invalid label in {source}: {line!r}")
        seen.add(path)
        rows.append((source.parent / path, label, generator, dataset, extra))
    if not rows:
        parser.error(f"empty manifest: {source}")

    output = args.output.resolve()
    output.mkdir()
    variants = {
        "jpeg": (4, None),
        "jpeg-960": (12, "scale='min(960,iw)':'min(960,ih)':force_original_aspect_ratio=decrease"),
    }
    for variant in variants:
        (output / variant).mkdir()

    def encode(indexed_row):
        index, (path, label, generator, dataset, extra) = indexed_row
        rendered = {"original": (os.path.relpath(path, output), label, generator, f"{dataset}-original", *extra)}
        for variant, (quality, filters) in variants.items():
            target = Path(variant) / f"{index:06d}.jpg"
            encode_jpeg(path, output / target, quality, filters)
            rendered[variant] = (str(target), label, generator, f"{dataset}-{variant}", *extra)
        return rendered

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        converted = list(pool.map(encode, enumerate(rows)))
    for variant in ["original", *variants]:
        (output / f"{variant}.tsv").write_text(
            "".join("\t".join(row[variant]) + "\n" for row in converted),
            encoding="utf-8",
        )


if __name__ == "__main__":
    main()
