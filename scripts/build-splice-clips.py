#!/usr/bin/env python3
"""Build test clips with a short generated segment at a known time.

For the partial-edit evaluation of video (docs/runbooks/partial-edits.md).
Each authentic parent timeline yields, as separately named `transform`
slices of one store:

  reencode                  the parent alone, through the same encode
  generated-splice-<d>s     <d> seconds of a generated clip replace <d>
                            seconds of the parent at a random time
  authentic-splice-<d>s     the same cut, with <d> seconds of another
                            authentic clip: a cut and no generated content
  authentic-dissolve-<d>s   the same, cross-faded in and out: an authentic
                            effect

Every clip of a store is made by one FFmpeg run of the same filter chain and
the same encoder settings: all material (parent, inserted clip) is brought to
the parent's frame rate and to one canvas, the timeline is assembled, and
the whole of it is encoded once. The inserted part and its surroundings
therefore share one encode, and the splice is not a codec cue. Audio is
dropped.

`--parent-clips K` assembles each parent from K authentic clips of the same
source, for timelines longer than the clips on disk. The cuts between them
are authentic content with a cut in it, recorded as `parent_cuts`.

Writes under --out: `clips/<n>/<transform>.mp4`, `manifest.tsv` (nine
columns: path, label, generator, dataset, parent_id, scene_id, source_id,
generator_family, transform) and `edits.tsv` (path, kind, generated,
start_s, end_s, duration_s, donor, donor_generator, parent_cuts). The label
column means "contains generated frames": keep this manifest out of
training and calibration.

Standard library only; needs `ffmpeg` and `ffprobe` on PATH (or --ffmpeg,
--ffprobe). Nothing is downloaded.
"""

import argparse
import json
import random
import subprocess
import sys
from pathlib import Path


def read_manifest(path):
    root = Path(path).resolve().parent
    rows = []
    for line in Path(path).read_text().splitlines():
        if not line:
            continue
        f = line.split("\t")
        rows.append({"path": root / f[0], "label": f[1], "generator": f[2], "dataset": f[3], "extra": f[4:]})
    return rows


def probe(ffprobe, path):
    """(width, height, fps, duration) of the first video stream, or None."""
    try:
        out = subprocess.run(
            [ffprobe, "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height,avg_frame_rate:format=duration", "-of", "json", str(path)],
            capture_output=True, text=True, check=True,
        ).stdout
        data = json.loads(out)
        stream = data["streams"][0]
        num, den = stream["avg_frame_rate"].split("/")
        fps = float(num) / float(den) if float(den) else 0.0
        return int(stream["width"]), int(stream["height"]), fps, float(data["format"]["duration"])
    except (subprocess.CalledProcessError, KeyError, IndexError, ValueError, ZeroDivisionError):
        return None


def canvas(width, height, max_height):
    """The parent's size, scaled down to at most `max_height`, with even sides."""
    scale = min(1.0, max_height / height) if max_height > 0 else 1.0
    even = lambda v: max(2, int(round(v * scale / 2)) * 2)
    return even(width), even(height)


def window(first, count, fps):
    """A `trim` that keeps exactly frames `first .. first + count - 1` of a stream at `fps`.

    The bounds sit half a frame before the first frame kept and half a frame
    before the first one dropped, so rounding cannot move a frame across
    them and every recorded time is a whole number of frames.
    """
    return f"trim=start={(first - 0.5) / fps:.6f}:end={(first + count - 0.5) / fps:.6f},setpts=PTS-STARTPTS"


def chain(index, first, count, size, fps, label):
    """Bring input `index` to the canvas and frame rate and keep `count` frames from frame `first`."""
    w, h = size
    return (
        f"[{index}:v]fps={fps:.6g},scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h},setsar=1,format=yuv420p,"
        f"{window(first, count, fps)}[{label}]"
    )


def encode(ffmpeg, inputs, graph, out, fps, crf):
    cmd = [ffmpeg, "-v", "error", "-y"]
    for path in inputs:
        cmd += ["-i", str(path)]
    cmd += [
        "-filter_complex", ";".join(graph), "-map", "[out]", "-an", "-r", f"{fps:.6g}", "-c:v", "libx264", "-preset", "veryfast",
        "-crf", str(crf), "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-map_metadata", "-1", str(out),
    ]
    subprocess.run(cmd, check=True)


def parent_graph(parts, size, fps):
    """Filter chains that assemble the parent timeline from `parts` (inputs 0..K-1, each a path and a frame count) as [parent]."""
    graph = [chain(i, 0, count, size, fps, f"p{i}") for i, (_, count) in enumerate(parts)]
    if len(parts) == 1:
        graph.append("[p0]null[parent]")
    else:
        graph.append("".join(f"[p{i}]" for i in range(len(parts))) + f"concat=n={len(parts)}:v=1:a=0[parent]")
    return graph


def build(args):
    rng = random.Random(args.seed)
    real = [r for m in args.real for r in read_manifest(m) if r["label"] == "0"]
    fake = [r for m in args.fake for r in read_manifest(m) if r["label"] == "1"]
    if not real or not fake:
        sys.exit("need authentic clips (label 0) in --real and generated clips (label 1) in --fake")
    out = Path(args.out)
    if (out / "manifest.tsv").exists():
        sys.exit(f"{out} already holds a store; choose a new directory")
    (out / "clips").mkdir(parents=True, exist_ok=True)
    rng.shuffle(real)
    durations = sorted(args.edit_seconds)
    info = {}

    def known(row):
        key = str(row["path"])
        if key not in info:
            info[key] = probe(args.ffprobe, row["path"])
        return info[key]

    def donor(pool, need, avoid, fps):
        """A clip of `pool` at least `need` seconds long that is not in `avoid`, and the frame (at the parent's rate) to start from."""
        for _ in range(40):
            row = rng.choice(pool)
            meta = known(row)
            if str(row["path"]) in avoid or meta is None or meta[3] < need + 0.2:
                continue
            return row, int(rng.uniform(0.0, meta[3] - need - 0.1) * fps)
        return None

    manifest, edits = [], ["# path\tkind\tgenerated\tstart_s\tend_s\tduration_s\tdonor\tdonor_generator\tparent_cuts"]
    made, cursor = 0, 0
    fade = args.fade_seconds
    while made < args.clips and cursor + args.parent_clips <= len(real):
        # A parent: K authentic clips of one source, each long enough to be worth cutting into.
        first = real[cursor]
        group = [first]
        scan = cursor + 1
        while len(group) < args.parent_clips and scan < len(real):
            if real[scan]["generator"] == first["generator"]:
                group.append(real.pop(scan))
            else:
                scan += 1
        cursor += 1
        metas = [known(r) for r in group]
        if len(group) < args.parent_clips or any(m is None for m in metas):
            continue
        fps = metas[0][2] if 1.0 <= metas[0][2] <= 120.0 else 25.0
        # Everything is counted in frames at the parent's rate, so that the times written down are the times in the file.
        parts = [(r["path"], int(min(m[3] - 0.05, args.max_part_seconds) * fps)) for r, m in zip(group, metas)]
        frames = sum(count for _, count in parts)
        total = frames / fps
        if any(count < fps for _, count in parts) or total < args.min_seconds:
            continue
        size = canvas(metas[0][0], metas[0][1], args.max_height)
        cuts, at = [], 0
        for _, count in parts[:-1]:
            at += count
            cuts.append(round(at / fps, 3))
        key = f"{made:05d}"
        folder = out / "clips" / key
        folder.mkdir(exist_ok=True)
        parent_id = first["extra"][0] if first["extra"] and first["extra"][0] not in ("", "-") else "/".join(first["path"].parts[-3:])
        if args.parent_clips > 1:
            parent_id += f"+{args.parent_clips - 1}"
        inputs = [p for p, _ in parts]
        base = parent_graph(parts, size, fps)
        used = {str(p) for p in inputs}
        plan = [("reencode", None, None, None, None)]
        for d in durations:
            if total < d + 2 * args.margin_seconds + 2 * fade:
                continue
            # The edit starts at frame `start` and replaces `d` frames; a dissolve takes `fade` frames at each end.
            start = round(rng.uniform(args.margin_seconds, total - d - args.margin_seconds - fade) * fps)
            generated, authentic = donor(fake, d + fade, used, fps), donor(real, d + fade, used, fps)
            if generated:
                plan.append((f"generated-splice-{d:g}s", "generated-splice", start, d, generated))
            if authentic:
                plan.append((f"authentic-splice-{d:g}s", "authentic-splice", start, d, authentic))
                if fade > 0:
                    plan.append((f"authentic-dissolve-{d:g}s", "authentic-dissolve", start, d, authentic))
        try:
            for transform, kind, start, d, source in plan:
                path = folder / f"{transform}.mp4"
                k = len(inputs)
                if kind is None:
                    encode(args.ffmpeg, inputs, base + ["[parent]null[out]"], path, fps, args.crf)
                    row = ("reencode", 0, "-", "-", "-", "-", "-")
                elif kind == "authentic-dissolve":
                    # Fade from the parent into the inserted clip from frame `start` and back out from frame `start + n`.
                    n, f = round(d * fps), max(1, round(fade * fps))
                    graph = base + [
                        "[parent]split=2[a][b]",
                        # The cross-fade wants one time base and frame rate on both sides.
                        f"[a]{window(0, start + f, fps)},settb=AVTB,fps={fps:.6g}[head]",
                        f"[b]{window(start + n, frames - start - n, fps)},settb=AVTB,fps={fps:.6g}[tail]",
                        chain(k, source[1], n + f, size, fps, "raw"),
                        f"[raw]settb=AVTB,fps={fps:.6g}[ins]",
                        f"[head][ins]xfade=transition=fade:duration={f / fps:.6f}:offset={(start - 0.25) / fps:.6f}[x]",
                        f"[x][tail]xfade=transition=fade:duration={f / fps:.6f}:offset={(start + n - 0.25) / fps:.6f}[out]",
                    ]
                    encode(args.ffmpeg, inputs + [source[0]["path"]], graph, path, fps, args.crf)
                    row = (kind, 0, f"{start / fps:.3f}", f"{(start + n + f) / fps:.3f}", f"{(n + f) / fps:.3f}", str(source[0]["path"]), source[0]["generator"])
                else:
                    n = round(d * fps)
                    graph = base + [
                        "[parent]split=2[a][b]",
                        f"[a]{window(0, start, fps)}[head]",
                        f"[b]{window(start + n, frames - start - n, fps)}[tail]",
                        chain(k, source[1], n, size, fps, "ins"),
                        "[head][ins][tail]concat=n=3:v=1:a=0[out]",
                    ]
                    encode(args.ffmpeg, inputs + [source[0]["path"]], graph, path, fps, args.crf)
                    row = (kind, int(kind == "generated-splice"), f"{start / fps:.3f}", f"{(start + n) / fps:.3f}", f"{n / fps:.3f}", str(source[0]["path"]), source[0]["generator"])
                rel = f"clips/{key}/{transform}.mp4"
                is_generated = row[1] == 1
                generator = f"partial:{row[6]}" if is_generated else first["generator"]
                family = row[6] if is_generated else ""
                scene = first["extra"][1] if len(first["extra"]) > 1 else ""
                source_id = first["extra"][2] if len(first["extra"]) > 2 else ""
                manifest.append("\t".join([rel, str(row[1]), generator, args.dataset, parent_id, scene, source_id, family, transform]))
                edits.append("\t".join([rel, row[0], str(row[1]), row[2], row[3], row[4], row[5], row[6], ",".join(f"{c:g}" for c in cuts) or "-"]))
        except subprocess.CalledProcessError as error:
            print(f"skipping parent {first['path']}: ffmpeg failed ({error})", file=sys.stderr)
            continue
        made += 1
    if not made:
        sys.exit("no parent timeline could be built (clips too short for --min-seconds, or unreadable)")
    (out / "manifest.tsv").write_text("\n".join(manifest) + "\n")
    (out / "edits.tsv").write_text("\n".join(edits) + "\n")
    print(f"{out}: {len(manifest)} clips from {made} authentic parent timelines; edit intervals in edits.tsv")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--real", nargs="+", required=True, help="video manifests; their label-0 clips are parents and authentic inserts")
    ap.add_argument("--fake", nargs="+", required=True, help="video manifests; their label-1 clips are the generated inserts")
    ap.add_argument("--out", required=True, help="a new directory for the store")
    ap.add_argument("--clips", type=int, default=40, help="authentic parent timelines to build")
    ap.add_argument("--parent-clips", type=int, default=1, help="authentic clips of one source concatenated into each parent")
    ap.add_argument("--edit-seconds", type=float, nargs="+", default=[1.0, 2.0, 4.0], help="durations of the inserted segment")
    ap.add_argument("--min-seconds", type=float, default=8.0, help="shortest parent timeline")
    ap.add_argument("--max-part-seconds", type=float, default=30.0, help="seconds used of each authentic clip")
    ap.add_argument("--margin-seconds", type=float, default=0.5, help="no edit this close to the start or end")
    ap.add_argument("--fade-seconds", type=float, default=0.4, help="cross-fade of the dissolve control (0: no dissolve control)")
    ap.add_argument("--max-height", type=int, default=480, help="canvas height cap (0: the parent's own size)")
    ap.add_argument("--crf", type=int, default=20)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--dataset", default="partial-splice")
    ap.add_argument("--ffmpeg", default="ffmpeg")
    ap.add_argument("--ffprobe", default="ffprobe")
    build(ap.parse_args())


if __name__ == "__main__":
    main()
