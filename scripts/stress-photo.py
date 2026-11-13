#!/usr/bin/env python3
"""Create paired JPEG stress inputs from a Tintype photo manifest."""

import argparse
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


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
        path, label, generator, dataset = line.split("\t")
        if path in seen or label not in {"0", "1"}:
            parser.error(f"duplicate path or invalid label in {source}: {line!r}")
        seen.add(path)
        rows.append((source.parent / path, label, generator, dataset))
    if not rows:
        parser.error(f"empty manifest: {source}")

    output = args.output.resolve()
    output.mkdir()
    variants = {
        "jpeg": ["-q:v", "4"],
        "jpeg-960": ["-vf", "scale='min(960,iw)':'min(960,ih)':force_original_aspect_ratio=decrease", "-q:v", "12"],
    }
    for variant in variants:
        (output / variant).mkdir()

    def encode(indexed_row):
        index, (path, label, generator, dataset) = indexed_row
        rendered = {"original": (os.path.relpath(path, output), label, generator, f"{dataset}-original")}
        for variant, options in variants.items():
            target = Path(variant) / f"{index:06d}.jpg"
            result = subprocess.run(
                ["ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin", "-threads", "1", "-i", str(path),
                 "-map_metadata", "-1", "-frames:v", "1", "-c:v", "mjpeg", "-threads", "1", "-pix_fmt", "yuvj420p",
                 *options, str(output / target)],
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True,
            )
            if result.returncode or not (output / target).is_file():
                raise RuntimeError(f"could not encode {path} as {variant}: {result.stderr.strip()}")
            rendered[variant] = (str(target), label, generator, f"{dataset}-{variant}")
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
