#!/usr/bin/env python3
"""Reproduce the NTIRE score transfer and image subgroup diagnostics."""

import argparse
import ast
import csv
import gzip
from collections import Counter
from pathlib import Path
from statistics import median
from zipfile import ZipFile

from PIL import Image
from PIL.JpegImagePlugin import get_sampling


ARTIFACTS = {"jpeg_ai", "cheng2020", "wmforger"}


def join(split, organizer, results):
    archive_name, labels_name = {
        "hard-validation": ("val_images_hard", "val_hard_labels.csv"),
        "public-test": ("test_images", "test_labels.csv"),
    }[split]
    with gzip.open(results / f"ntire-2026-{split}" / "scores.tsv.gz", "rt") as stream:
        scores = {r["path"]: r for r in csv.DictReader(stream, delimiter="\t")}
    rows = []
    with (organizer / labels_name).open(newline="") as stream, ZipFile(organizer / f"{archive_name}.zip") as archive:
        for label_row in csv.DictReader(stream):
            name = label_row["image_name"]
            score_row = scores.pop(name)
            label = int(label_row["label"])
            group = "distorted" if label_row["is_distorted"] == "1" else "clean"
            if score_row["label"] != str(label) or score_row["dataset"] != f"ntire-{group}":
                raise ValueError(f"Organizer and score archive labels differ for {name}")
            with archive.open(f"{archive_name}/{name}") as source, Image.open(source) as image:
                if image.format != "JPEG":
                    raise ValueError(f"Expected JPEG image {name}")
                tables = tuple(tuple(image.quantization[k]) for k in sorted(image.quantization))
                rows.append({
                    "label": label, "group": group, "score": float(score_row["tintype"]),
                    "operations": set(ast.literal_eval(label_row["distortions"])),
                    "short": min(image.size), "quantization": tables, "sampling": get_sampling(image),
                })
    if scores or len(rows) != 2500:
        raise ValueError(f"Incomplete score/label join for {split}")
    return rows


def cutoff(rows):
    authentic = sorted((r["score"] for r in rows if r["label"] == 0), reverse=True)
    return authentic[len(authentic) // 100]


def positives(rows, label, threshold):
    scores = [r["score"] for r in rows if r["label"] == label]
    return sum(score > threshold for score in scores), len(scores)


def show(rows, threshold):
    true, generated = positives(rows, 1, threshold)
    false, authentic = positives(rows, 0, threshold)
    return f"{true}/{generated} recall ({100*true/generated:.2f}%), {false}/{authentic} FPR ({100*false/authentic:.2f}%)"


def groups(rows):
    return {"clean": [r for r in rows if r["group"] == "clean"],
            "distorted": [r for r in rows if r["group"] == "distorted"], "pooled": rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("organizer", type=Path, help="Directory with both NTIRE ZIP and label CSV pairs")
    parser.add_argument("--results", type=Path, default=Path("docs/results"))
    args = parser.parse_args()
    hard_rows = join("hard-validation", args.organizer, args.results)
    public_rows = join("public-test", args.organizer, args.results)
    hard, public = groups(hard_rows), groups(public_rows)

    print("Calibrate on hard validation; transfer unchanged to public test")
    for source, source_rows in hard.items():
        threshold = cutoff(source_rows)
        print(f"{source} > {threshold:.6f}, calibration: {show(source_rows, threshold)}")
        for target, target_rows in public.items():
            print(f"  public {target}: {show(target_rows, threshold)}")
    print(f"Unique final JPEG quantization tables: {len({r['quantization'] for r in hard_rows + public_rows})}")
    print(f"Final JPEG subsampling counts (2 = 4:2:0): {dict(Counter(r['sampling'] for r in hard_rows + public_rows))}")

    for split, cohorts in [("hard validation", hard), ("public test", public)]:
        print(split)
        for group in ("clean", "distorted"):
            rows = cohorts[group]
            for label in (0, 1):
                subset = [r for r in rows if r["label"] == label]
                print(f"  {group}, label {label}, median short side {median(r['short'] for r in subset):.0f}px, median score {median(r['score'] for r in subset):.6f}")
        distorted = cohorts["distorted"]
        for name, predicate in [
            ("short side <512px", lambda r: r["short"] < 512),
            ("short side >=512px", lambda r: r["short"] >= 512),
            ("artifact group", lambda r: bool(r["operations"] & ARTIFACTS)),
            ("outside artifact group", lambda r: not (r["operations"] & ARTIFACTS)),
        ]:
            print(f"  {name}: {show([r for r in distorted if predicate(r)], 0.5)}")
        for operation in sorted(ARTIFACTS):
            subset = [r for r in distorted if operation in r["operations"]]
            if subset:
                print(f"  {operation}: {show(subset, 0.5)}")


if __name__ == "__main__":
    main()
