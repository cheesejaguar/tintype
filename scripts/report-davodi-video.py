#!/usr/bin/env python3
"""Archive matched score rows and report per-scene video ranking."""
import argparse
import csv
import gzip
import math
from collections import defaultdict
from pathlib import Path

DATASET = "davodi-2026-normalized"
GENERATORS = ("kling-2.6", "seedance-pro-1", "sora-2", "veo-3.1")


def read_scores(path, expected, system):
    rows = {}
    for line in path.read_text().splitlines():
        filename, label, generator, dataset, score, *frames = line.split("\t")
        name = Path(filename).name
        row = expected[name]
        if (
            name in rows or row["label"] != label or row["generator"] != generator
            or dataset != DATASET or not 0 <= float(score) <= 1
            or not frames or not all(math.isfinite(float(frame)) for frame in frames)
            or (system == "WaveRep" and len(frames) != 64)
        ):
            raise ValueError(f"invalid {system} score row: {line!r}")
        rows[name] = (score, frames)
    if rows.keys() != expected.keys():
        raise ValueError(f"{system} did not score every prepared clip")
    return rows


def write_archive(path, rows):
    content = "\n".join("\t".join(row) for row in rows) + "\n"
    path.write_bytes(gzip.compress(content.encode(), mtime=0))


def report(system, table, groups, expected):
    values = {name: float(score) for name, (score, _) in table.items()}
    authentic = {scene: members["pixabay"] for scene, members in groups.items()}
    generated = sorted(set(values) - set(authentic.values()))
    real_flags = sum(values[name] > .5 for name in authentic.values())
    fake_flags = sum(values[name] > .5 for name in generated)
    auc = sum(
        (values[fake] > values[real]) + .5 * (values[fake] == values[real])
        for fake in generated for real in authentic.values()
    ) / (len(authentic) * len(generated))
    margins = defaultdict(list)
    for name in generated:
        source = expected[name]["generator"]
        real = authentic[expected[name]["scene"]]
        margins[source].append(values[name] - values[real])
    matched = [margin for source in margins.values() for margin in source]
    wins = sum(margin > 0 for margin in matched)
    ties = sum(margin == 0 for margin in matched)
    losses = len(matched) - wins - ties
    print(
        f"{system}: {fake_flags}/{len(generated)} AI flagged, "
        f"{real_flags}/{len(authentic)} real flagged, AUROC {auc:.4f}, "
        f"matched ranking {wins} wins/{ties} ties/{losses} losses"
    )
    for source, diffs in sorted(margins.items()):
        flags = sum(values[name] > .5 for name in generated if expected[name]["generator"] == source)
        print(
            f"  {source}: {flags}/{len(diffs)} flags, "
            f"matched {sum(margin > 0 for margin in diffs)} wins/{sum(margin == 0 for margin in diffs)} ties"
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input_metadata", type=Path)
    parser.add_argument("tintype", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--waverep", type=Path)
    parser.add_argument("--frame-rows", type=Path)
    args = parser.parse_args()
    with args.input_metadata.open(newline="") as stream:
        metadata = list(csv.DictReader(stream, delimiter="\t"))
    expected = {Path(row["path"]).name: row for row in metadata}
    if len(expected) != len(metadata) or not expected:
        raise ValueError("input metadata must contain unique basenames")
    groups = defaultdict(dict)
    for name, row in expected.items():
        source = row["generator"]
        members = groups[row["scene"]]
        if source in members or row["label"] != ("0" if source == "pixabay" else "1"):
            raise ValueError(f"invalid input metadata for {name}")
        members[source] = name
    if any(set(members) != {"pixabay", *GENERATORS} for members in groups.values()):
        raise ValueError("each scene must contain one authentic and four distinct generated clips")

    scores = {"Tintype": read_scores(args.tintype, expected, "Tintype")}
    if args.waverep:
        scores["WaveRep"] = read_scores(args.waverep, expected, "WaveRep")
    print(f"Matched coverage {len(groups)}/24 scenes, {len(expected)} clips")
    for system, table in scores.items():
        report(system, table, groups, expected)

    rows = [("path", "label", "generator", "dataset", *(name.lower() for name in scores))]
    raw = [("path", "label", "generator", "dataset", "detector", "score", "frame_logits")]
    for name in sorted(expected):
        prefix = (name, expected[name]["label"], expected[name]["generator"], DATASET)
        rows.append((*prefix, *(table[name][0] for table in scores.values())))
        for system, table in scores.items():
            score, frames = table[name]
            raw.append((*prefix, system.lower(), score, ",".join(frames)))
    write_archive(args.output, rows)
    if args.frame_rows:
        write_archive(args.frame_rows, raw)


if __name__ == "__main__":
    main()
