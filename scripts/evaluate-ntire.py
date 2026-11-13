#!/usr/bin/env python3
"""Compare Tintype and Community Forensics on NTIRE 2026 public images."""

import argparse
import csv
import gzip
import hashlib
import math
import platform
import re
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path
from tempfile import TemporaryDirectory
from zipfile import BadZipFile, ZipFile


PINS = {
    "val_images_hard.zip": "32bd2669c9d8e532ac45227f818b725f79317aeab279ca807efc91ba73c6c844",
    "val_hard_labels.csv": "4238972a07e48a2c03908d4566032609870eaf9cc4edfeb82667967cf4f77675",
    "test_images.zip": "a7fe4c56e0b1cd138c2de980bda7fbb640d629711022344d53cc9a1466dfe9f5",
    "test_labels.csv": "336fdc42968a0395e1e9a914f1a87399e9b9d49df0d54d90cc4892d43cefc12e",
    "tintype.safetensors": "9dcb21dc9800591ffd758a3c7ca6aa65fa5a9f09a223fecc952f34294ae9aeba",
    "tintype.json": "bfe1366bcfec28d15fb9116cb96c721de52de91d6fa4d98b6da4ec7fe6e62709",
    "commfor-384.safetensors": "b89f36275f3bf5e2b040eee36597a8f19db051bff9a473a9cf7b2466284fb387",
}

SPLITS = {
    "hard-validation": ("Hard Validation", "val_images_hard", "val_hard_labels.csv", 625, 625),
    "public-test": ("Public Test", "test_images", "test_labels.csv", 600, 650),
}


def verify(path, pin):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    if digest.hexdigest() != PINS[pin]:
        raise ValueError(f"SHA-256 mismatch for {path}: {digest.hexdigest()}")


def prepare(archive_path, labels_path, scratch, source_directory, real_count, ai_count):
    media = scratch / "media"
    media.mkdir()
    expected = {}
    with labels_path.open(encoding="utf-8-sig", newline="") as source:
        for row in csv.DictReader(source):
            name, label, distorted = row["image_name"], row["label"], row["is_distorted"]
            if (not re.fullmatch(r"[0-9a-f]{20}\.jpg", name) or name in expected
                    or label not in {"0", "1"} or distorted not in {"0", "1"}):
                raise ValueError(f"invalid or duplicate NTIRE image row: {row!r}")
            cohort = "ntire-distorted" if distorted == "1" else "ntire-clean"
            expected[name] = (label, "unknown", cohort)
    counts = Counter((label, cohort) for label, _, cohort in expected.values())
    wanted = {(label, cohort): count for label, count in [("0", real_count), ("1", ai_count)]
              for cohort in ["ntire-clean", "ntire-distorted"]}
    if counts != wanted:
        raise ValueError(f"unexpected NTIRE split counts: {counts}")

    with ZipFile(archive_path) as archive:
        members = [entry.filename for entry in archive.infolist() if not entry.is_dir()]
        if len(members) != len(expected) or set(members) != {f"{source_directory}/{name}" for name in expected}:
            raise ValueError("ZIP and label file disagree on NTIRE input membership")
        for name in expected:
            with archive.open(f"{source_directory}/{name}") as source, (media / name).open("wb") as target:
                signature = source.read(3)
                if signature != b"\xff\xd8\xff":
                    raise ValueError(f"expected JPEG bytes for {name}")
                target.write(signature)
                shutil.copyfileobj(source, target)

    manifest = scratch / "manifest.tsv"
    manifest.write_text("".join("\t".join((f"media/{name}", *fields)) + "\n"
                                for name, fields in expected.items()), encoding="utf-8")
    return expected, manifest


def load_scores(path, expected):
    scores = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        image, label, generator, cohort, score, *_ = line.split("\t")
        name = Path(image).name
        value = float(score)
        if (name in scores or expected.get(name) != (label, generator, cohort)
                or not math.isfinite(value) or not 0 <= value <= 1):
            raise ValueError(f"invalid or mismatched score in {path}: {line!r}")
        scores[name] = score
    if scores.keys() != expected.keys():
        raise ValueError(f"{path.name} is missing {len(expected.keys() - scores.keys())} scores")
    return scores


def report(binary, output, paths, expected, scores, split):
    archive = output / "scores.tsv.gz"
    rows = ["path\tlabel\tgenerator\tdataset\ttintype\tcommfor"]
    rows.extend("\t".join((name, *expected[name], scores[0][name], scores[1][name])) for name in sorted(expected))
    archive.write_bytes(gzip.compress(("\n".join(rows) + "\n").encode(), mtime=0))

    command = [binary, "report", "--markdown", "--pooled", "ntire-all",
               "--system", f"Tintype={paths[0]}", "--system", f"Community Forensics={paths[1]}"]
    tables = subprocess.check_output(command, text=True)
    matched = subprocess.check_output([*command, "--at-rate-of", "Tintype"], text=True)
    (output / "report.md").write_text(
        f"# NTIRE 2026 {split}\n\n"
        "Same 2,500 released JPEG inputs for both detectors. Each score table uses the default threshold of 0.5. "
        "Recall at 1% FPR ranks scores with a threshold selected from that cohort's authentic images.\n"
        + tables + "\n## At Tintype's shipped false-positive count\n\n" + matched,
        encoding="utf-8",
    )
    print(tables + matched, end="")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("labels", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--commfor", required=True, type=Path)
    parser.add_argument("--weights", type=Path, default=Path("models/tintype.safetensors"))
    parser.add_argument("--tintype", default="tintype")
    parser.add_argument("--split", choices=SPLITS, default="hard-validation")
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("the Community Forensics reference command requires macOS")
    split, source_directory, labels_name, real_count, ai_count = SPLITS[args.split]
    try:
        for path, pin in [(args.archive, source_directory + ".zip"), (args.labels, labels_name),
                          (args.weights, "tintype.safetensors"), (args.weights.with_suffix(".json"), "tintype.json"),
                          (args.commfor, "commfor-384.safetensors")]:
            verify(path, pin)
        args.output.mkdir()
        with TemporaryDirectory(prefix="tintype-ntire-") as temp:
            scratch = Path(temp)
            expected, manifest = prepare(args.archive, args.labels, scratch, source_directory, real_count, ai_count)
            outputs = [scratch / "tintype.tsv", scratch / "commfor.tsv"]
            for weights, out, extra in [(args.weights, outputs[0], []), (args.commfor, outputs[1], ["--reference", "commfor"])]:
                subprocess.run([args.tintype, "score", "--device", "cpu", "--manifest", str(manifest),
                                "--out", str(out), "--weights", str(weights), *extra], check=True)
            report(args.tintype, args.output, outputs, expected, [load_scores(out, expected) for out in outputs], split)
    except (OSError, ValueError, BadZipFile, KeyError, subprocess.CalledProcessError) as error:
        print(f"NTIRE evaluation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
