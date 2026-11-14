#!/usr/bin/env python3
"""Reproduce the NTIRE score transfer and image subgroup diagnostics.

Two reports from the archived detector scores and the organizers' files:

- by default, the threshold transfer from hard validation to the public
  test and the image subgroups (short side, JPEG tables, the learned-codec
  group). This reads the images, so it needs the two organizer ZIP archives
  beside the label files and Pillow (`pip install Pillow` in a virtual
  environment; it is not in the standard library).
- with `--operations`, one table per split covering every NTIRE operation,
  and one per operation and severity: authentic images flagged at the
  shipped cutoff and the shift against the clean authentic images, recall,
  and AUROC. This needs only the two label files (their SHA-256 is checked
  against the pins in docs/EVIDENCE.md) and the standard library.

NTIRE applied one to five operations to each distorted image, so a row
describes the images whose chain *contains* the operation, not the
operation alone; the "without" column is the same count over the other
distorted images. Both released splits were used during development:
every number here is diagnostic.
"""

import argparse
import ast
import csv
import gzip
import hashlib
from collections import Counter
from pathlib import Path
from statistics import median
from zipfile import ZipFile


ARTIFACTS = {"jpeg_ai", "cheng2020", "wmforger"}
SPLITS = {
    "hard-validation": ("val_images_hard", "val_hard_labels.csv", "4238972a07e48a2c03908d4566032609870eaf9cc4edfeb82667967cf4f77675"),
    "public-test": ("test_images", "test_labels.csv", "336fdc42968a0395e1e9a914f1a87399e9b9d49df0d54d90cc4892d43cefc12e"),
}
SHIPPED = 0.5


def join_labels(split, organizer, results):
    """Archived scores joined to the organizer labels, without opening any image."""
    _, labels_name, pin = SPLITS[split]
    labels = organizer / labels_name
    if hashlib.sha256(labels.read_bytes()).hexdigest() != pin:
        raise ValueError(f"Unexpected hash for NTIRE {split} labels: {labels}")
    with gzip.open(results / f"ntire-2026-{split}" / "scores.tsv.gz", "rt") as stream:
        scores = {r["path"]: r for r in csv.DictReader(stream, delimiter="\t")}
    rows = []
    with labels.open(newline="") as stream:
        for label_row in csv.DictReader(stream):
            name = label_row["image_name"]
            score_row = scores.pop(name)
            label = int(label_row["label"])
            group = "distorted" if label_row["is_distorted"] == "1" else "clean"
            if score_row["label"] != str(label) or score_row["dataset"] != f"ntire-{group}":
                raise ValueError(f"Organizer and score archive labels differ for {name}")
            operations = ast.literal_eval(label_row["distortions"])
            scales = ast.literal_eval(label_row["distortion_scales"])
            if len(operations) != len(scales):
                raise ValueError(f"Operations and scales differ in length for {name}")
            rows.append({"label": label, "group": group, "score": float(score_row["tintype"]), "chain": list(zip(operations, scales))})
    if scores or len(rows) != 2500:
        raise ValueError(f"Incomplete score/label join for {split}")
    return rows


def auroc(rows):
    """Probability that a generated image outscores an authentic one, ties counting half."""
    real = sorted(r["score"] for r in rows if r["label"] == 0)
    generated = [r["score"] for r in rows if r["label"] == 1]
    if not real or not generated:
        return None
    from bisect import bisect_left, bisect_right
    wins = sum((bisect_left(real, s) + bisect_right(real, s)) / 2 for s in generated)
    return wins / (len(real) * len(generated))


def rate(rows, label, threshold):
    scores = [r["score"] for r in rows if r["label"] == label]
    hits = sum(s > threshold for s in scores)
    return hits, len(scores)


def cell(hits, total):
    return f"{hits}/{total} ({100 * hits / total:.1f}%)" if total else "–"


def severity(operation, scale):
    """The label of a severity level. NTIRE's `downscale` draws a continuous
    factor (binned here); every other operation has a handful of levels."""
    if operation == "downscale":
        return f"{min(int(scale * 10) / 10, 0.7):.1f}–{min(int(scale * 10) / 10, 0.7) + 0.1:.1f}"
    return str(scale)


def operation_tables(split, rows):
    clean = [r for r in rows if r["group"] == "clean"]
    distorted = [r for r in rows if r["group"] == "distorted"]
    clean_fp, clean_real = rate(clean, 0, SHIPPED)
    clean_cut = cutoff(clean)
    print(f"\n## NTIRE {split}: every operation, at the shipped cutoff (score > {SHIPPED})\n")
    print(f"Clean images: {cell(clean_fp, clean_real)} authentic flagged, {cell(*rate(clean, 1, SHIPPED))} recall, AUROC {auroc(clean):.3f}.")
    print(f"All distorted images: {cell(*rate(distorted, 0, SHIPPED))} authentic flagged, {cell(*rate(distorted, 1, SHIPPED))} recall, AUROC {auroc(distorted):.3f}.")
    print(f"The clean cutoff column uses the score that flags 1% of this split's clean authentic images ({clean_cut:.6f}).\n")
    header = ("| {} | AI / real | Real flagged | Shift vs clean real | Real flagged without it | Recall | AUROC | Clean 1% cutoff: real flagged, recall |\n"
              "|---|---|---|---|---|---|---|---|")
    operations = sorted({op for r in distorted for op, _ in r["chain"]})

    def line(name, subset, rest):
        fp, real = rate(subset, 0, SHIPPED)
        tp, generated = rate(subset, 1, SHIPPED)
        shift = f"{100 * (fp / real - clean_fp / clean_real):+.1f} pt" if real else "–"
        area = auroc(subset)
        transfer = f"{cell(*rate(subset, 0, clean_cut))}, {cell(*rate(subset, 1, clean_cut))}"
        without = cell(*rate(rest, 0, SHIPPED)) if rest is not None else "–"
        return f"| {name} | {generated} / {real} | {cell(fp, real)} | {shift} | {without} | {cell(tp, generated)} | {'–' if area is None else f'{area:.3f}'} | {transfer} |"

    print(header.format("Operation in the chain"))
    for op in operations:
        has = [r for r in distorted if any(o == op for o, _ in r["chain"])]
        print(line(op, has, [r for r in distorted if not any(o == op for o, _ in r["chain"])]))
    group = [r for r in distorted if {o for o, _ in r["chain"]} & ARTIFACTS]
    print(line("jpeg_ai, cheng2020 or wmforger", group, [r for r in distorted if not {o for o, _ in r["chain"]} & ARTIFACTS]))
    print(f"\nBy chain length:\n\n{header.format('Operations applied')}")
    for length in sorted({len(r["chain"]) for r in distorted}):
        print(line(str(length), [r for r in distorted if len(r["chain"]) == length], None))
    print(f"\nBy severity (the organizers' scale value; small rows are noisy):\n\n{header.format('Operation, severity')}")
    for op in operations:
        levels = {}
        for r in distorted:
            for o, scale in r["chain"]:
                if o == op:
                    levels.setdefault(severity(op, scale), (scale, []))[1].append(r)
        for name, (_, subset) in sorted(levels.items(), key=lambda kv: (isinstance(kv[1][0], str), kv[1][0])):
            print(line(f"{op} {name}", subset, None))


def join(split, organizer, results):
    # Pillow is needed only here, to read sizes and JPEG tables from the archives.
    from PIL import Image
    from PIL.JpegImagePlugin import get_sampling

    archive_name, labels_name, _ = SPLITS[split]
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
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("organizer", type=Path, help="Directory with both NTIRE ZIP and label CSV pairs")
    parser.add_argument("--results", type=Path, default=Path("docs/results"))
    parser.add_argument("--operations", action="store_true",
                        help="print the per-operation and per-severity tables instead (label files only; no ZIP archives, no Pillow)")
    args = parser.parse_args()
    if args.operations:
        print("NTIRE 2026 per-operation diagnostics from archived Tintype scores. Diagnostic only: both splits were used in development.")
        print("A row covers the distorted images whose chain contains the operation; chains hold one to five operations.")
        for split in SPLITS:
            operation_tables(split, join_labels(split, args.organizer, args.results))
        return
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
