#!/usr/bin/env python3
"""Prepare matched four-second clips for the Davodi et al. video comparison.

With `--grouped`, also write manifests with the grouping columns of
`src/ingest.rs` (parent_id, scene_id, source_id, generator_family,
transform) for the video audit (docs/runbooks/video-audit.md):
`manifest-grouped.tsv` lists the normalised clips and
`originals-grouped.tsv` the files they were made from, with the same
`parent_id`. `--collection` adds held-out authentic collections registered
with `scripts/video_audit.py collection`, normalised in the same way.
"""

import argparse
import csv
import hashlib
import io
import json
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path, PurePosixPath
from zipfile import ZipFile


ZIP_MD5 = "eb703809083d3c6d54e96eb76bd50f53"
ZIP_SHA256 = "6253192806d1dd76f3a9cf61ae8f78ac6d0fd5ece2c535d679ecec9118b5a81b"
SOURCES = {
    "pixabay": ("filename", None),
    "seedance-pro-1": ("seedance_filename", "Seedance"),
    "veo-3.1": ("veo3-1_filename", "Veo"),
    "sora-2": ("sora2_filename", "Sora"),
    "kling-2.6": ("klingv2-6_filename", "Kling"),
}
# Grouped manifests: the collection a clip came from (column 7) and the model
# line of its generator (column 8).
AUTHENTIC_SOURCE = "pixabay"
GENERATED_SOURCE = "davodi-2026"
FAMILIES = {"seedance-pro-1": "seedance", "veo-3.1": "veo", "sora-2": "sora", "kling-2.6": "kling"}


def digest(path):
    sha = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024*1024), b""):
            sha.update(block)
    return sha.hexdigest()


def validate_media(path, full=False):
    result = subprocess.check_output([
        "ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
        "format=duration:stream=width,height,codec_name,pix_fmt,avg_frame_rate,nb_frames",
        "-of", "json", str(path),
    ], text=True)
    probed = json.loads(result)
    stream, = probed["streams"]
    if float(probed["format"]["duration"]) < 4.0:
        raise ValueError(f"Source {path} is shorter than four seconds")
    if full and (stream["width"] != 800 or stream["codec_name"] != "h264"
                 or stream["pix_fmt"] != "yuv420p" or stream["avg_frame_rate"] != "24/1"
                 or stream["nb_frames"] != "96"):
        raise ValueError(f"Unexpected output format for {path}: {stream}")
    return stream, probed["format"]["duration"]


def input_metadata(archive):
    names = archive.namelist()
    member, = [name for name in names if PurePosixPath(name).name == "metadata.csv"]
    blob = archive.read(member)
    rows = list(csv.DictReader(io.StringIO(blob.decode("utf-8-sig"), newline="")))
    rows.sort(key=lambda row: int(row["video_number"]))
    if [int(row["video_number"]) for row in rows] != list(range(1, 25)):
        raise ValueError("Expected exactly 24 numbered study themes")
    return rows, blob


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("authentic", type=Path, help="Directory containing the downloaded Pixabay clips named by scene, e.g. 01.mp4")
    parser.add_argument("output", type=Path)
    parser.add_argument("--jobs", type=int, default=4)
    parser.add_argument("--source-record", type=Path, help="Recorded inputs.tsv to verify source hashes and recover original URLs")
    parser.add_argument("--grouped", action="store_true", help="Also write manifest-grouped.tsv, originals-grouped.tsv and audit.tsv for the video audit")
    parser.add_argument("--collection", type=Path, action="append", default=[],
                        help="Cohort directory written by `video_audit.py collection`: normalise its clips too (implies --grouped; repeatable)")
    args = parser.parse_args()
    grouped = args.grouped or bool(args.collection)
    if grouped:
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        import video_audit
    if digest(args.archive) != ZIP_SHA256:
        raise ValueError("Zenodo v1 ZIP checksum mismatch")
    out = args.output.resolve()
    out.mkdir()
    clips = out / "clips"
    clips.mkdir()
    sources = out / "sources"
    sources.mkdir()
    tasks = []
    recorded = {}
    if args.source_record:
        with args.source_record.open(newline="") as record:
            recorded = {row["path"]: row for row in csv.DictReader(record, delimiter="\t")}
    with ZipFile(args.archive) as archive:
        rows, metadata_blob = input_metadata(archive)
        names = set(archive.namelist())
        for row in rows:
            scene = int(row["video_number"])
            original = args.authentic / f"{scene:02d}.mp4"
            if not original.is_file():
                print(f"Skipping all five clips in scene {scene}: authentic clip unavailable")
                continue
            for source, (field, folder) in SOURCES.items():
                label = int(folder is not None)
                filename = row[field].strip()
                record = recorded.get(f"clips/{scene:02d}-{source}.mp4")
                if args.source_record and record is None:
                    raise ValueError(f"Unrecorded input in scene {scene}: {source}")
                member = f"ForZenodo/{folder}/{filename}" if folder else (record["source"] if record else row["selected_video_url"].strip())
                if folder:
                    if member not in names:
                        raise ValueError(f"Missing archive member {member}")
                    original = sources / f"{scene:02d}-{source}.mp4"
                    with archive.open(member) as stream, original.open("wb") as dest:
                        shutil.copyfileobj(stream, dest)
                tasks.append((scene, source, label, member, original, record))

    def encode(original, result):
        filters = "scale=800:-2:flags=bicubic,setsar=1,fps=24,setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709"
        subprocess.run([
            "ffmpeg", "-nostdin", "-hide_banner", "-loglevel", "error", "-threads", "1",
            "-i", str(original), "-t", "4", "-map", "0:v:0", "-an", "-vf", filters, "-map_metadata", "-1",
            "-c:v", "libx264", "-threads", "1", "-crf", "18", "-preset", "veryfast", "-pix_fmt", "yuv420p", str(result),
        ], check=True)

    def normalize(task):
        scene, source, label, origin, original, record = task
        source_sha = digest(original)
        if record and source_sha != record["source_sha256"]:
            raise ValueError(f"Source SHA-256 mismatch for {original}")
        stream, duration = validate_media(original)
        relpath = Path("clips") / f"{scene:02d}-{source}.mp4"
        result = out / relpath
        encode(original, result)
        output, _ = validate_media(result, full=True)
        normalized_sha = digest(result)
        if record and normalized_sha != record["normalized_sha256"]:
            raise ValueError(f"Normalized SHA-256 mismatch for {result}; verify the FFmpeg build")
        return (relpath.as_posix(), label, source, scene, origin, source_sha, normalized_sha,
                stream["width"], stream["height"], stream["avg_frame_rate"], duration, output["height"])

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        finished = list(pool.map(normalize, tasks))
    if not finished:
        raise ValueError("No complete matched themes")
    if recorded and {row[0] for row in finished} != recorded.keys():
        raise ValueError("Supplied clips do not cover the recorded matched scenes")
    with (out / "manifest.tsv").open("w", newline="") as f:
        writer = csv.writer(f, delimiter="\t", lineterminator="\n")
        writer.writerows((path, label, generator, "davodi-2026-normalized") for path, label, generator, *_ in finished)
    with (out / "inputs.tsv").open("w", newline="") as f:
        writer = csv.writer(f, delimiter="\t", lineterminator="\n")
        writer.writerow(["path", "label", "generator", "scene", "source", "source_sha256", "normalized_sha256", "source_width",
                         "source_height", "source_fps", "source_duration", "normalized_height"])
        writer.writerows(finished)
    (out / "run.json").write_text(json.dumps({
        "archive_sha256": ZIP_SHA256, "archive_md5": ZIP_MD5,
        "metadata_sha256": hashlib.sha256(metadata_blob).hexdigest(), "scenes": len(finished)//5,
        "ffmpeg": subprocess.check_output(["ffmpeg", "-version"], text=True).splitlines()[0],
        "duration_seconds": 4, "width": 800, "fps": 24, "codec": "h264", "crf": 18,
    }, indent=2) + "\n")
    print(f"Prepared {len(finished)} normalized clips across {len(finished)//5} complete matched scenes")
    if not grouped:
        return

    # The audit pairs every normalised clip with the file it was made from: same parent_id, different transform.
    normalized, originals, audit = [], [], []
    for path, label, generator, scene, *_ in finished:
        stem = Path(path).stem
        original = sources / f"{stem}.mp4"
        if not label:
            shutil.copyfile(args.authentic / f"{scene:02d}.mp4", original)
        row = {"label": label, "generator": generator, "dataset": "davodi-2026-normalized", "parent_id": f"davodi-{stem}", "scene_id": f"davodi-{scene:02d}",
               "source_id": GENERATED_SOURCE if label else AUTHENTIC_SOURCE, "generator_family": FAMILIES.get(generator, "")}
        normalized.append({**row, "rel": path, "transform": "davodi-norm"})
        originals.append({**row, "rel": f"sources/{stem}.mp4", "dataset": "davodi-2026", "transform": "original"})
        audit.append((row["parent_id"], row["source_id"], "generated" if label else "stock", row["parent_id"], "", f"sources/{stem}.mp4", digest(original)))

    extra, skipped = [], 0
    for cohort in args.collection:
        types = {r["parent_id"]: r for r in video_audit.read_table(cohort / "audit.tsv")}
        for row in video_audit.read_manifest(cohort / "manifest.tsv"):
            extra.append((row, types[row["parent_id"]], video_audit.safe_name(row["parent_id"])))

    def normalize_extra(item):
        # The same encode as the matched clips. A video track that starts a frame late gives 95 frames instead of 96,
        # which the matched comparison refuses and an authentic-only collection can live with; the count is on record.
        row, registered, name = item
        try:
            validate_media(row["path"])
        except ValueError:
            return None
        result = Path("clips") / f"{name}.mp4"
        encode(row["path"], out / result)
        output = json.loads(subprocess.check_output([
            "ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,codec_name,pix_fmt,avg_frame_rate,nb_frames", "-of", "json", str(out / result)], text=True))["streams"][0]
        if (output["width"], output["codec_name"], output["pix_fmt"], output["avg_frame_rate"]) != (800, "h264", "yuv420p", "24/1") or int(output["nb_frames"]) < 90:
            raise ValueError(f"Unexpected output format for {result}: {output}")
        original = sources / f"{name}{row['path'].suffix.lower()}"
        shutil.copyfile(row["path"], original)
        return row, registered, result.as_posix(), original, digest(original), output["nb_frames"]

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        added = list(pool.map(normalize_extra, extra))
    for item in added:
        if item is None:
            skipped += 1
            continue
        row, registered, result, original, sha, _ = item
        normalized.append({**row, "rel": result, "transform": "davodi-norm"})
        originals.append({**row, "rel": f"sources/{original.name}", "transform": "original"})
        audit.append((row["parent_id"], row["source_id"], registered["collection_type"], registered["source_video"], registered["start_frame"], f"sources/{original.name}", sha))
    video_audit.write_manifest(out / "manifest-grouped.tsv", normalized)
    video_audit.write_manifest(out / "originals-grouped.tsv", originals)
    video_audit.write_table(out / "audit.tsv", ["parent_id", "source_id", "collection_type", "source_video", "start_frame", "file", "sha256"], audit)
    print(f"Grouped manifests: {len(normalized)} clips ({len(finished)} DaVODi, {len(normalized) - len(finished)} from registered collections"
          + (f", {skipped} shorter than four seconds left out" if skipped else "") + ")")
    short = sum(1 for item in added if item and item[5] != "96")
    if short:
        print(f"  {short} collection clips have fewer than 96 frames after the trim (late-starting video track)")


if __name__ == "__main__":
    main()
