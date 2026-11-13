#!/usr/bin/env python3
"""Evaluate Tintype on the pinned AI Detector Arena v0.1 release."""

import argparse
import csv
import gzip
import hashlib
import io
import json
import math
import platform
import shutil
import subprocess
import sys
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from tempfile import TemporaryDirectory
from zipfile import BadZipFile, ZipFile

from benchmark_images import encode_jpeg


ARENA_URL = "https://aidetectarena-benchmark.nyc3.cdn.digitaloceanspaces.com/datasets-archive/benchmark-v0.1.zip"
ARENA_SHA256 = "1e62844bba714c2e190792844d4169fa3119fac3c2c143dad1eb8062e20b67e4"
METADATA_URL = "https://raw.githubusercontent.com/AI-Detect-Arena/benchmark-dataset/3014b54eec9d44c7ef60b4c7e0cda6aeb4735d5b/dataset_metadata/images_metadata.csv"
METADATA_BLOB = "394b7636d4a627d52d507f29240041a3849c850d"
COMMFOR_URL = "https://huggingface.co/OwensLab/commfor-model-384/resolve/6076002bf0d9dd37537f965ee2f06f826c333b61/model.safetensors"
COMMFOR_SHA256 = "b89f36275f3bf5e2b040eee36597a8f19db051bff9a473a9cf7b2466284fb387"
TINTYPE_SHA256 = "9dcb21dc9800591ffd758a3c7ca6aa65fa5a9f09a223fecc952f34294ae9aeba"
TINTYPE_SPEC_SHA256 = "bfe1366bcfec28d15fb9116cb96c721de52de91d6fa4d98b6da4ec7fe6e62709"
VARIANTS = ("original", "jpeg-q4")
CATEGORIES = {"animal", "art", "food", "landscape", "portrait", "product"}
NAMES = {"tintype": "Tintype", "commfor": "Community Forensics"}


@dataclass(frozen=True)
class Item:
    id: str
    label: int
    generator: str
    category: str
    member: str
    original: str
    jpeg: str
    sha256: str

    def path(self, variant):
        return self.original if variant == "original" else self.jpeg


def file_digest(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require_digest(path, expected):
    actual = file_digest(path)
    if actual != expected:
        raise ValueError(f"SHA-256 mismatch for {path}: expected {expected}, got {actual}")


def download(url, destination):
    print(f"Downloading {url}", file=sys.stderr)
    subprocess.run(
        ["curl", "--fail", "--location", "--silent", "--show-error", "--retry", "3",
         "--connect-timeout", "20", "--max-time", "1200", "--retry-max-time", "1200",
         "--output", str(destination), url],
        check=True,
    )


def read_metadata(archive, scratch):
    embedded = [name for name in archive.namelist() if PurePosixPath(name).name == "images_metadata.csv"]
    if len(embedded) > 1:
        raise ValueError(f"ambiguous Arena metadata: {embedded}")
    if embedded:
        data = archive.read(embedded[0])
        source = f"archive:{embedded[0]}"
    else:
        fallback = scratch / "arena-metadata.csv"
        download(METADATA_URL, fallback)
        data = fallback.read_bytes()
        blob = hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()
        if blob != METADATA_BLOB:
            raise ValueError(f"Arena fallback metadata Git blob mismatch: {blob}")
        source = METADATA_URL
    reader = csv.DictReader(io.StringIO(data.decode("utf-8-sig"), newline=""))
    required = {"id", "filename", "is_ai", "generator", "category"}
    if not required.issubset(reader.fieldnames or []):
        raise ValueError(f"Arena metadata columns missing in {source}")
    return list(reader), source, hashlib.sha256(data).hexdigest()


def media_members(archive):
    paths = defaultdict(list)
    for info in archive.infolist():
        if info.is_dir():
            continue
        path = PurePosixPath(info.filename)
        if path.is_absolute() or ".." in path.parts or "\\" in info.filename:
            raise ValueError(f"unsafe ZIP member path: {info.filename!r}")
        if path.suffix.lower() in {".jpg", ".jpeg", ".png"}:
            for index in range(len(path.parts)):
                paths[PurePosixPath(*path.parts[index:]).as_posix()].append(info)
    return paths


def image_format(header):
    if header.startswith(b"\xff\xd8\xff"):
        return "jpg"
    if header.startswith(b"\x89PNG\r\n\x1a\n"):
        return "png"
    return "unknown"


def prepare_media(archive_path, scratch, expected_counts=True):
    (scratch / "original").mkdir()
    (scratch / "jpeg-q4").mkdir()
    items = []
    missing = []
    formats = Counter()
    seen_ids = set()
    seen_members = set()
    with ZipFile(archive_path) as archive:
        metadata, source, digest = read_metadata(archive, scratch)
        members = media_members(archive)
        for row in sorted(metadata, key=lambda row: row["id"] or ""):
            image_id, category = row["id"], row["category"]
            value = (row["is_ai"] or "").lower()
            filename = row["filename"] or ""
            generator = row["generator"] or ""
            if (not image_id or image_id in seen_ids or category not in CATEGORIES
                    or value not in {"true", "false"} or (value == "true" and not generator)
                    or any(c in field for c in "\t\r\n" for field in (image_id, generator, filename))):
                raise ValueError(f"invalid or duplicate Arena metadata row: {row!r}")
            seen_ids.add(image_id)
            label = int(value == "true")
            class_name = "ai" if label else "real"
            path = PurePosixPath(filename)
            if path.is_absolute() or ".." in path.parts or "\\" in filename or not path.name:
                raise ValueError(f"invalid image filename for {image_id}: {filename!r}")
            candidates = members.get(path.as_posix())
            if not candidates and path.parts[0] == "images":
                candidates = members.get(PurePosixPath(*path.parts[1:]).as_posix())
            elif len(path.parts) == 1:
                candidates = members.get(f"{class_name}/{category}/{path.name}", candidates)
            if not candidates:
                missing.append((image_id, label, filename))
                continue
            if len(candidates) != 1 or candidates[0].filename in seen_members:
                raise ValueError(f"ambiguous or duplicate ZIP image for {image_id}: {filename}")
            member = candidates[0]
            seen_members.add(member.filename)
            if any(c in member.filename for c in "\t\r\n"):
                raise ValueError(f"invalid ZIP image filename: {member.filename!r}")
            with archive.open(member) as stream:
                header = stream.read(8)
                kind = image_format(header)
                formats[(class_name, kind)] += 1
                # Some Arena files carry a PNG extension but actually contain JPEG bytes.
                extension = path.suffix.lstrip(".") if kind == "unknown" else kind
                index = len(items)
                original = f"original/{index:06d}.{extension}"
                jpeg = f"jpeg-q4/{index:06d}.jpg"
                with (scratch / original).open("wb") as destination:
                    destination.write(header)
                    shutil.copyfileobj(stream, destination)
            items.append(Item(image_id, label, generator or "real:arena", category,
                              member.filename, original, jpeg, file_digest(scratch / original)))

    counts = Counter(item.label for item in items)
    # The current upstream CSV lists 12 extra real portraits over the announced release.
    # Accept those only when the checksum-pinned release archive includes their bytes.
    if expected_counts and (counts[1] != 1018 or counts[0] not in {1020, 1032}):
        raise ValueError(f"Arena ZIP membership differs from the release: {counts[1]} AI, {counts[0]} real, {len(missing)} metadata paths missing")
    if expected_counts and missing and (len(missing) != 12 or any(label != 0 or not image_id.startswith("real_portrait_") for image_id, label, _ in missing)):
        raise ValueError(f"unexpected missing Arena metadata paths: {missing[:12]}")
    return items, missing, formats, source, digest


def write_tsv(path, rows):
    with path.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.writer(stream, delimiter="\t", lineterminator="\n", quoting=csv.QUOTE_NONE)
        writer.writerows(rows)


def normalize(items, scratch, jobs):
    def one(item):
        try:
            encode_jpeg(scratch / item.original, scratch / item.jpeg, quality=4, autorotate=False)
        except ValueError as error:
            return item.jpeg, str(error)
        return None

    with ThreadPoolExecutor(max_workers=jobs) as pool:
        return dict(error for error in pool.map(one, items) if error is not None)


def record(result, errors, path, score, error):
    if path in result or path in errors:
        raise ValueError(f"duplicate result for {path}")
    if error:
        errors[path] = error
    elif type(score) not in {float, int} or not math.isfinite(score) or not 0 <= score <= 1:
        raise ValueError(f"invalid detector score for {path}: {score}")
    else:
        result[path] = score


def tintype_scores(binary, weights, device, paths, scratch):
    scores, errors = {}, {}
    # Bound argv below macOS and Linux limits while loading the model for groups of files.
    for offset in range(0, len(paths), 256):
        command = [binary, "detect", "--json", "--weights", str(weights), "--device", device,
                   "--", *paths[offset:offset + 256]]
        result = subprocess.run(command, cwd=scratch, stdout=subprocess.PIPE, text=True)
        if result.returncode not in {0, 2}:
            result.check_returncode()
        for line in result.stdout.splitlines():
            row = json.loads(line)
            record(scores, errors, row["path"], row.get("score"), row.get("error"))
    require_coverage(paths, scores, errors)
    return scores, errors


def require_coverage(paths, scores, errors):
    expected = set(paths)
    observed = scores.keys() | errors.keys()
    if expected != observed:
        raise ValueError(f"detector returned {len(expected - observed)} fewer and {len(observed - expected)} unexpected paths")


def commfor_scores(binary, weights, device, expected, scratch):
    manifest = scratch / "commfor-input.tsv"
    output = scratch / "commfor-output.tsv"
    write_tsv(manifest, ((path, item.label, item.generator, variant) for path, (item, variant) in expected.items()))
    # Use full precision, matching the published baseline configuration.
    commfor_device = "gpu-f32" if device == "gpu" else device
    if commfor_device == "auto":
        commfor_device = "cpu"
    subprocess.run([binary, "score", "--reference", "commfor", "--device", commfor_device,
                    "--weights", str(weights), "--manifest", manifest.name, "--out", output.name],
                   cwd=scratch, check=True)
    scores, errors = {}, {}
    for line in output.read_text(encoding="utf-8").splitlines():
        path, label, generator, variant, score, *_ = line.split("\t")
        item, expected_variant = expected[path]
        if (int(label), generator, variant) != (item.label, item.generator, expected_variant):
            raise ValueError(f"Community Forensics output identity changed for {path}")
        record(scores, errors, path, float(score), None)
    errors.update((path, "could not decode") for path in expected.keys() - scores.keys())
    require_coverage(expected, scores, errors)
    return scores, errors


def emit_report(output, scratch, binary, paired, scores):
    header = ["path", "label", "generator", "dataset", *scores.keys()]
    rows = []
    for variant in VARIANTS:
        for item in paired:
            path = item.path(variant)
            rows.append((f"{variant}/{item.id}", item.label, item.generator,
                         f"arena-{variant}", *(table[path] for table in scores.values())))
    with (output / "scores.tsv.gz").open("wb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as archive:
            with io.TextIOWrapper(archive, encoding="utf-8", newline="") as stream:
                csv.writer(stream, delimiter="\t", lineterminator="\n", quoting=csv.QUOTE_NONE).writerows([header, *rows])

    command = [binary, "report", "--markdown"]
    for index, name in enumerate(scores, start=4):
        table = scratch / f"report-{name}.tsv"
        write_tsv(table, ((*row[:4], row[index]) for row in rows))
        command.extend(["--system", f"{NAMES[name]}={table}"])
    report = subprocess.check_output(command, text=True).strip()
    counts = Counter(item.label for item in paired)
    text = ("# AI Detector Arena v0.1\n\n"
            "Author-run evaluation on an external benchmark. Original and normalized rows use the same paired inputs.\n\n"
            f"Paired coverage is {len(paired)} images ({counts[1]} generated, {counts[0]} authentic). "
            "See `run.json` and `errors.tsv` for source coverage and exclusions.\n\n"
            "Normalization uses a same-dimension 4:2:0 JPEG re-encode at FFmpeg MJPEG q4 for both classes. "
            "It retains traces of previous compression. Recall at 1% FPR is an empirical ranking comparison "
            "on this cohort. Shipped-threshold results use a score above 0.5.\n\n"
            + report + "\n")
    (output / "report.md").write_text(text, encoding="utf-8")
    print(text, end="")


def evaluate(args):
    output = args.output.resolve()
    binary = shutil.which(args.tintype)
    if binary is None:
        raise ValueError(f"Tintype binary not found: {args.tintype}")
    binary = str(Path(binary).resolve())
    weights = args.weights.resolve()
    require_digest(weights, TINTYPE_SHA256)
    require_digest(weights.with_suffix(".json"), TINTYPE_SPEC_SHA256)
    if (args.with_commfor or args.commfor) and platform.system() != "Darwin":
        raise ValueError("the Community Forensics reference command requires macOS")
    ffmpeg_version = subprocess.check_output(["ffmpeg", "-version"], text=True).splitlines()[0]
    tintype_version = subprocess.check_output([binary, "--version"], text=True).strip()
    output.mkdir(parents=True)

    with TemporaryDirectory(prefix="arena-work-", dir=output) as temporary:
        scratch = Path(temporary)
        archive = args.archive.resolve() if args.archive else scratch / "benchmark-v0.1.zip"
        if not args.archive:
            download(ARENA_URL, archive)
        require_digest(archive, ARENA_SHA256)

        commfor = args.commfor.resolve() if args.commfor else None
        if args.with_commfor:
            commfor = scratch / "commfor-384.safetensors"
            download(COMMFOR_URL, commfor)
        if commfor:
            require_digest(commfor, COMMFOR_SHA256)

        items, missing, formats, metadata_source, metadata_sha256 = prepare_media(archive, scratch)
        print(f"Prepared {len(items)} Arena originals", file=sys.stderr)
        encode_errors = normalize(items, scratch, args.jobs)
        expected = {item.path(variant): (item, variant) for variant in VARIANTS for item in items
                    if item.path(variant) not in encode_errors}
        scores, errors = {}, {}
        scores["tintype"], errors["tintype"] = tintype_scores(binary, weights, args.device, list(expected), scratch)
        if commfor:
            scores["commfor"], errors["commfor"] = commfor_scores(binary, commfor, args.device, expected, scratch)

        paired = [item for item in items if all(item.path(variant) in table
                  for variant in VARIANTS for table in scores.values())]
        paired_ids = {item.id for item in paired}
        write_tsv(output / "errors.tsv", [
            ("id", "variant", "stage", "reason"),
            *((image_id, "original", "metadata", f"missing archive member {filename}") for image_id, _, filename in missing),
            *((item.id, "jpeg-q4", "encode", encode_errors[item.jpeg].replace(str(scratch) + "/", "").replace("\n", " ").replace("\t", " "))
              for item in items if item.jpeg in encode_errors),
            *((expected[path][0].id, expected[path][1], name, message.replace("\n", " ").replace("\t", " "))
              for name, failures in errors.items() for path, message in failures.items()),
        ])
        write_tsv(output / "inputs.tsv", [
            ("id", "label", "generator", "category", "member", "original_sha256", "jpeg_sha256", "paired"),
            *((item.id, item.label, item.generator, item.category, item.member, item.sha256,
               file_digest(scratch / item.jpeg) if item.jpeg not in encode_errors else "", int(item.id in paired_ids)) for item in items),
        ])
        run = {
            "archive_url": ARENA_URL, "archive_sha256": ARENA_SHA256,
            "metadata_source": metadata_source, "metadata_sha256": metadata_sha256,
            "announced_counts": {"generated": 1018, "authentic": 1020},
            "archive_counts": {"generated": sum(item.label for item in items), "authentic": sum(not item.label for item in items)},
            "original_formats": {f"{label}:{kind}": count for (label, kind), count in formats.items()},
            "metadata_paths_missing": len(missing), "paired_images": len(paired),
            "normalization": {"encoder": "mjpeg", "quality": 4, "pixel_format": "yuvj420p", "resize": False, "autorotate": False},
            "jpeg_encode_failures": len(encode_errors),
            "detector_failures": {name: len(failures) for name, failures in errors.items()},
            "device": args.device, "platform": platform.platform(), "ffmpeg": ffmpeg_version,
            "tintype": tintype_version, "tintype_sha256": TINTYPE_SHA256, "tintype_spec_sha256": TINTYPE_SPEC_SHA256,
            "community_forensics_sha256": COMMFOR_SHA256 if commfor else None,
        }
        (output / "run.json").write_text(json.dumps(run, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        if {item.label for item in paired} != {0, 1}:
            raise ValueError("both classes must have paired scores to report metrics")
        emit_report(output, scratch, binary, paired, scores)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="new directory for the report, scores and input identities")
    parser.add_argument("--tintype", default="tintype", help="Tintype binary path")
    parser.add_argument("--weights", type=Path, default=Path("models/tintype.safetensors"))
    parser.add_argument("--archive", type=Path, help="use a local copy of the pinned ZIP")
    baseline = parser.add_mutually_exclusive_group()
    baseline.add_argument("--commfor", type=Path, help="use a local copy of the pinned Community Forensics weights (macOS)")
    baseline.add_argument("--with-commfor", action="store_true", help="fetch the pinned Community Forensics weights (macOS)")
    parser.add_argument("--jobs", type=int, default=4, help="parallel FFmpeg jobs (default: 4)")
    parser.add_argument("--device", choices=["auto", "cpu", "gpu", "gpu-f32"], default="cpu")
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("--jobs must be positive")
    try:
        evaluate(args)
    except (OSError, ValueError, BadZipFile, KeyError, subprocess.CalledProcessError) as error:
        print(f"Arena evaluation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
