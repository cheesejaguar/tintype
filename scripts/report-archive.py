#!/usr/bin/env python3
"""Check archived detector scores and regenerate tables with tintype report."""

import argparse
import csv
import gzip
import math
import subprocess
from pathlib import Path, PurePosixPath
from tempfile import TemporaryDirectory


IDENTITY = ["path", "label", "generator", "dataset"]
NAMES = {
    "tintype": "Tintype",
    "commfor": "Community Forensics",
    "swinv2": "SwinV2 detector",
    "waverep": "WaveRep",
    "tintype-second": "Tintype + second opinion",
    "tintype-strict": "Tintype (strict)",
    "tintype-fast": "Tintype (fast)",
    "tintype-all-views": "Tintype (all views)",
    "tintype-sensitive": "Tintype (sensitive)",
}


def read_archive(path):
    with gzip.open(path, "rt", encoding="utf-8", newline="") as stream:
        header, *rows = csv.reader(stream, delimiter="\t", quoting=csv.QUOTE_NONE)
    columns = header[len(IDENTITY):]
    if header[:len(IDENTITY)] != IDENTITY or not columns or len(set(columns)) != len(columns):
        raise ValueError(f"invalid archive columns in {path}")
    if set(columns) - NAMES.keys():
        raise ValueError(f"unknown detector column in {path}")

    seen = set()
    for row in rows:
        if len(row) != len(header) or not row[0] or row[0] in seen or row[1] not in {"0", "1"}:
            raise ValueError(f"invalid or duplicate archive row: {row!r}")
        seen.add(row[0])
        if any(not math.isfinite(float(score)) or not 0 <= float(score) <= 1 for score in row[len(IDENTITY):]):
            raise ValueError(f"invalid archived score for {row[0]}")
    return columns, rows


def check_frozen(path, rows):
    expected = {}
    with gzip.open(path, "rt", encoding="utf-8") as stream:
        for line in stream:
            if line.startswith("#"):
                continue
            _, _, label, generator, dataset, name = line.rstrip("\n").split("\t")
            if name in expected:
                raise ValueError(f"duplicate path in frozen set: {name}")
            expected[name] = label, generator, dataset
    observed = {name: (label, generator, dataset) for name, label, generator, dataset, *_ in rows}
    missing = expected.keys() - observed.keys()
    extra = observed.keys() - expected.keys()
    changed = sum(expected[name] != observed[name] for name in expected.keys() & observed.keys())
    if missing or extra or changed:
        raise ValueError(f"archive differs from frozen set ({len(missing)} missing, {len(extra)} extra, {changed} relabeled)")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--frozen", type=Path)
    parser.add_argument("--tintype", default="tintype", help="Tintype binary path")
    parser.add_argument("--dataset", help="only this dataset")
    parser.add_argument("--jpeg-only", action="store_true", help="only paths with a JPEG extension")
    parser.add_argument("--systems", nargs="+", choices=NAMES, help="detector columns to compare")
    parser.add_argument("--pooled", default=None, help="pooled table name")
    parser.add_argument("--at-rate-of", choices=NAMES, help="compare at this detector's observed false-positive count")
    args = parser.parse_args()

    try:
        columns, rows = read_archive(args.archive)
        if args.frozen:
            check_frozen(args.frozen, rows)
    except (OSError, ValueError) as error:
        parser.error(str(error))

    systems = args.systems or columns
    if len(systems) != len(set(systems)) or set(systems) - set(columns):
        parser.error("each selected detector must appear once in the archive")
    if args.at_rate_of and args.at_rate_of not in systems:
        parser.error("--at-rate-of must be one of the selected detectors")
    if args.dataset:
        rows = [row for row in rows if row[3] == args.dataset]
    if args.jpeg_only:
        rows = [row for row in rows if PurePosixPath(row[0]).suffix.lower() in {".jpg", ".jpeg"}]
    if not rows:
        parser.error("no rows matched")

    with TemporaryDirectory(prefix="tintype-report-") as scratch:
        command = [args.tintype, "report", "--markdown"]
        for name in systems:
            index = columns.index(name) + len(IDENTITY)
            scores = Path(scratch) / f"{name}.tsv"
            with scores.open("w", encoding="utf-8", newline="") as stream:
                csv.writer(stream, delimiter="\t", lineterminator="\n", quoting=csv.QUOTE_NONE).writerows(
                    [*row[:len(IDENTITY)], row[index]] for row in rows
                )
            command.extend(["--system", f"{NAMES[name]}={scores}"])
        if args.pooled:
            command.extend(["--pooled", args.pooled])
        if args.at_rate_of:
            command.extend(["--at-rate-of", NAMES[args.at_rate_of]])
        return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
