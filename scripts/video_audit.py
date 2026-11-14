#!/usr/bin/env python3
"""Prepare, probe and control cohorts for the real-source video audit.

The audit (docs/runbooks/video-audit.md) asks which video failure remains
after source, codec and visible marks are controlled. This file holds the
preparation steps and the statistics the reports share:

  collection      register a held-out authentic (or a generated) collection
                  in a grouped manifest, with its licence on record
  start-frames    export the first frame of each authentic clip, to start
                  image-to-video generation from matched frames
  probe           per-clip container, codec, cadence, resolution, bitrate,
                  GOP and motion measurements
  balance         per-class balance table from a probe file
  match           pick a subset in which the classes are matched on cadence,
                  resolution, motion and bitrate
  variants        originals, a stream-copy trim, a common fixed-GOP
                  re-encode and simulated reposts of every clip
  watermark       sham-logo add/remove variants with placebo crops and fills
  metadata-probe  predict the label from metadata alone, cross-validated by
                  source: a cohort where that works has a shortcut

Every manifest written here has the nine columns of `src/ingest.rs`:
path, label, generator, dataset, parent_id, scene_id, source_id,
generator_family, transform. `parent_id` names the original clip and is the
same for every variant made from it.

Standard library only. `ffmpeg` and `ffprobe` must be on PATH for
`collection --normalize`, `start-frames`, `probe`, `variants` and
`watermark`. Nothing here downloads anything.
"""

import argparse
import csv
import hashlib
import json
import math
import os
import random
import re
import shutil
import struct
import subprocess
import sys
import zlib
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

GROUP_COLUMNS = ("parent_id", "scene_id", "source_id", "generator_family", "transform")
COLLECTION_TYPES = ("stock", "cctv", "handheld", "animation", "cgi", "screen", "generated")
VIDEO_SUFFIXES = {".mp4", ".mov", ".mkv", ".webm", ".avi", ".m4v"}

# Authentic video sources the released video weights were trained on, were
# calibrated on, or were already scored on (docs/DATA.md). A collection drawn
# from one of them is not held out, whatever its type.
TRAINING_SOURCES = {
    "pexels": "training negatives (stores `pexels`, `pexelsb`)",
    "openvid": "training negatives (GenBuster-200K-mini)",
    "activitynet": "training negatives (VidGuard-R1)",
    "kinetics": "training negatives, calibration and confirmation 4",
    "k400": "training negatives, calibration and confirmation 4",
    "ucf101": "training negatives",
    "sakugabooru": "training negatives (hand-drawn animation)",
    "humananimalcartoon": "training negatives (cartoons)",
    "sunain": "training negatives (gameplay)",
    "msrvtt": "calibration and confirmation 4",
    "msvd": "calibration and confirmation 4",
}

# The DaVODi normalisation, exactly as `scripts/prepare-davodi-video.py` runs
# it: first four seconds, 800 px wide, 24 fps, H.264 CRF 18, 4:2:0, BT.709
# tags, no audio, no metadata. `--self-test` checks the two files agree.
NORMALIZE_FILTERS = "scale=800:-2:flags=bicubic,setsar=1,fps=24,setparams=color_primaries=bt709:color_trc=bt709:colorspace=bt709"
NORMALIZE_SECONDS = 4
NORMALIZE_CRF = 18
# The fixed-GOP control adds only this to the normalisation: a closed
# two-second GOP with no scene-cut keyframes, so every clip has the same
# keyframe cadence whatever its content.
FIXED_GOP = 48
# Simulated reposts: generation k re-encodes generation k-1 (the original
# for k = 1). Each step never upscales. (maximum width, CRF).
REPOST_LADDER = ((720, 28), (540, 30), (360, 32))
POSITIONS = ("br", "bl", "tr", "tl")
FFMPEG_QUIET = ["-nostdin", "-hide_banner", "-loglevel", "error"]


# ---------------------------------------------------------------- statistics

def binomial_cdf(k, n, p):
    """P(X <= k) for X ~ Binomial(n, p), as `src/metrics.rs::binomial_cdf`."""
    if p <= 0:
        return 1.0
    if p >= 1:
        return 1.0 if k >= n else 0.0
    lp, lq = math.log(p), math.log1p(-p)
    ln_c, total = 0.0, 0.0
    for i in range(min(k, n) + 1):
        total += math.exp(ln_c + i * lp + (n - i) * lq)
        if i < n:
            ln_c += math.log(n - i) - math.log(i + 1)
    return min(total, 1.0)


def clopper_pearson_upper(k, n, confidence=0.95):
    """One-sided Clopper-Pearson upper bound on a rate after `k` events in
    `n` independent trials; the same bisection as
    `src/metrics.rs::clopper_pearson_upper`. The trials must be independent:
    count source videos, not clips cut from one."""
    if n == 0:
        return float("nan")
    if k >= n:
        return 1.0
    alpha = 1.0 - confidence
    if k == 0:
        return 1.0 - alpha ** (1.0 / n)
    lo, hi = k / n, 1.0
    for _ in range(100):
        mid = (lo + hi) / 2
        if binomial_cdf(k, n, mid) > alpha:
            lo = mid
        else:
            hi = mid
    return hi


def clopper_pearson_lower(k, n, confidence=0.95):
    if n == 0:
        return float("nan")
    if k == 0:
        return 0.0
    return 1.0 - clopper_pearson_upper(n - k, n, confidence)


def sign_test_p(b, c):
    """Exact two-sided McNemar (sign) test on `b` and `c` discordant pairs."""
    if b + c == 0:
        return 1.0
    return min(1.0, 2.0 * binomial_cdf(min(b, c), b + c, 0.5))


def auroc(positive, negative):
    """Probability that a positive scores above a negative, ties counting half."""
    if not positive or not negative:
        return float("nan")
    values = sorted([(v, 1) for v in positive] + [(v, 0) for v in negative])
    rank_sum, i = 0.0, 0
    while i < len(values):
        j = i
        while j < len(values) and values[j][0] == values[i][0]:
            j += 1
        rank_sum += (i + j + 1) / 2 * sum(values[k][1] for k in range(i, j))
        i = j
    return (rank_sum - len(positive) * (len(positive) + 1) / 2) / (len(positive) * len(negative))


def bootstrap_clustered(items, clusters, stat, rounds=1000, seed=0, level=0.95):
    """Percentile interval of `stat(items)` resampling whole clusters with
    replacement (`clusters[i]` is the cluster of `items[i]`), the scheme of
    `src/metrics.rs::bootstrap_ci_clustered`. The draws come from Python's
    generator, so the interval agrees with the Rust one only statistically."""
    members = defaultdict(list)
    for item, cluster in zip(items, clusters):
        members[cluster].append(item)
    keys = sorted(members, key=str)
    rng = random.Random(seed)
    stats = []
    for _ in range(rounds):
        sample = []
        for _ in keys:
            sample.extend(members[rng.choice(keys)])
        value = stat(sample)
        if not math.isnan(value):
            stats.append(value)
    if not stats:
        return float("nan"), float("nan")
    stats.sort()
    tail = (1 - level) / 2
    return stats[int(tail * len(stats))], stats[min(len(stats) - 1, int((1 - tail) * len(stats)))]


def empirical_cutoff(real_scores, fpr):
    """The cutoff `src/metrics.rs::empirical_cutoff` gives, and how many real
    items it may flag: flag strictly above the (allowed+1)-th highest real
    score, with allowed = floor(fpr * n)."""
    ordered = sorted(real_scores, reverse=True)
    allowed = int(math.floor(fpr * len(ordered)))
    return ordered[min(allowed, len(ordered) - 1)], allowed


def logit(p):
    p = min(1 - 1e-7, max(1e-7, float(p)))
    return math.log(p / (1 - p))


def median(values):
    ordered = sorted(values)
    if not ordered:
        return float("nan")
    mid = len(ordered) // 2
    return ordered[mid] if len(ordered) % 2 else (ordered[mid - 1] + ordered[mid]) / 2


def quantile(values, q):
    ordered = sorted(values)
    if not ordered:
        return float("nan")
    at = q * (len(ordered) - 1)
    lo = int(math.floor(at))
    hi = min(lo + 1, len(ordered) - 1)
    return ordered[lo] + (ordered[hi] - ordered[lo]) * (at - lo)


def standardized_difference(a, b):
    """Difference of means over the pooled standard deviation (0 when both
    groups are constant and equal, infinite when constant and different)."""
    if not a or not b:
        return float("nan")
    ma, mb = sum(a) / len(a), sum(b) / len(b)
    va = sum((x - ma) ** 2 for x in a) / max(1, len(a) - 1)
    vb = sum((x - mb) ** 2 for x in b) / max(1, len(b) - 1)
    pooled = math.sqrt((va + vb) / 2)
    if pooled == 0:
        return 0.0 if ma == mb else math.copysign(float("inf"), mb - ma)
    return (mb - ma) / pooled


def total_variation(a, b):
    """Half the L1 distance between two categorical distributions."""
    if not a or not b:
        return float("nan")
    ca, cb = Counter(a), Counter(b)
    return sum(abs(ca[k] / len(a) - cb[k] / len(b)) for k in set(ca) | set(cb)) / 2


# ------------------------------------------------------------------ manifests

def read_manifest(path):
    """Rows of a manifest; grouping columns that are absent, empty or `-`
    come back as empty strings (`src/ingest.rs::Item::group`)."""
    path = Path(path)
    rows = []
    for line in path.read_text().splitlines():
        if not line:
            continue
        fields = line.split("\t")
        if len(fields) < 4:
            raise ValueError(f"bad manifest line in {path}: {line!r}")
        groups = [("" if v == "-" else v) for v in (fields[4:9] + [""] * 5)[:5]]
        row = {"rel": fields[0], "path": path.parent / fields[0], "label": int(fields[1]), "generator": fields[2], "dataset": fields[3], "extra": fields[9:]}
        row.update(zip(GROUP_COLUMNS, groups))
        rows.append(row)
    return rows


def manifest_line(rel, row):
    """A nine-column manifest row; an unset grouping column is written `-`."""
    fields = [rel, str(row["label"]), row["generator"], row["dataset"]] + [row.get(c) or "-" for c in GROUP_COLUMNS]
    if any("\t" in f or "\n" in f for f in fields):
        raise ValueError(f"tab or newline in manifest field: {fields!r}")
    return "\t".join(fields)


def write_manifest(path, rows):
    """Write rows (each with `rel`) and return how many were written."""
    Path(path).write_text("".join(manifest_line(row["rel"], row) + "\n" for row in rows))
    return len(rows)


def source_of(row):
    """The collection a row came from: `source_id`, else the generator name
    of an authentic row and the dataset of a generated one
    (`src/split.rs::source_of`)."""
    return row.get("source_id") or (row["generator"] if row["label"] == 0 else row["dataset"])


def parent_of(row):
    """`parent_id`, or the manifest path of a row that has none."""
    return row.get("parent_id") or row["rel"]


def safe_name(text):
    return re.sub(r"[^A-Za-z0-9._-]+", "_", text).strip("_") or "clip"


def training_overlap(source_id):
    """Why `source_id` is not held out, or None: its name contains a source
    the released video weights have already seen (`TRAINING_SOURCES`)."""
    key = re.sub(r"[^a-z0-9]", "", source_id.lower())
    for name, why in TRAINING_SOURCES.items():
        if name in key:
            return why
    return None


def digest(path):
    sha = hashlib.sha256()
    with open(path, "rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            sha.update(block)
    return sha.hexdigest()


def read_table(path):
    with open(path, newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t"))


def write_table(path, header, rows):
    with open(path, "w", newline="") as stream:
        writer = csv.writer(stream, delimiter="\t", lineterminator="\n")
        writer.writerow(header)
        writer.writerows(rows)


def link_or_copy(source, dest, copy):
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() or dest.is_symlink():
        dest.unlink()
    if copy:
        shutil.copyfile(source, dest)
    else:
        os.symlink(Path(source).resolve(), dest)


# -------------------------------------------------------------- ffmpeg lines

def encode_args(crf, preset, gop=None):
    args = ["-c:v", "libx264", "-threads", "1", "-crf", str(crf), "-preset", preset]
    if gop:
        args += ["-g", str(gop), "-keyint_min", str(gop), "-sc_threshold", "0"]
    return args + ["-pix_fmt", "yuv420p"]


def normalize_command(source, dest, gop=None, seconds=NORMALIZE_SECONDS, filters=NORMALIZE_FILTERS, logo=None):
    """The DaVODi normalisation of `source`; with `gop`, the fixed-GOP
    control; with other `filters`, an alteration applied to the normalised
    frames before the one encode. `logo` is a second input for `filters`
    that end in the label `[v]`."""
    command = ["ffmpeg", *FFMPEG_QUIET, "-threads", "1", "-i", str(source)]
    if logo:
        command += ["-i", str(logo), "-t", str(seconds), "-filter_complex", filters, "-map", "[v]", "-an"]
    else:
        command += ["-t", str(seconds), "-map", "0:v:0", "-an", "-vf", filters]
    return command + ["-map_metadata", "-1", *encode_args(NORMALIZE_CRF, "veryfast", gop), str(dest)]


def repost_command(source, dest, width, crf, seconds=NORMALIZE_SECONDS):
    """One simulated repost: cap the width (never upscale), re-encode at a
    lower quality with the encoder's own GOP, keep the cadence."""
    filters = f"scale=w='min(iw,{width})':h=-2:flags=bicubic,setsar=1"
    return ["ffmpeg", *FFMPEG_QUIET, "-threads", "1", "-i", str(source), "-t", str(seconds), "-map", "0:v:0", "-an", "-vf", filters,
            "-map_metadata", "-1", *encode_args(crf, "medium"), str(dest)]


def trim_command(source, dest, seconds=NORMALIZE_SECONDS):
    """The first seconds of the original bitstream, copied without re-encoding."""
    return ["ffmpeg", *FFMPEG_QUIET, "-i", str(source), "-t", str(seconds), "-map", "0:v:0", "-an", "-c", "copy", "-map_metadata", "-1", str(dest)]


def even(value, up=False):
    value = int(math.ceil(value) if up else round(value))
    return value + (value % 2 if up else -(value % 2))


def logo_box(width, height, position, logo_fraction=0.14, margin_fraction=0.03):
    """`(x, y, w, h)` of the sham mark in a `width` x `height` frame: a 3:1
    box `logo_fraction` of the width wide, `margin_fraction` of the width
    from the two edges `position` names (`br` = bottom right)."""
    w = max(12, even(width * logo_fraction))
    h = max(4, even(w / 3))
    margin = max(4, round(width * margin_fraction))
    if position not in POSITIONS:
        raise ValueError(f"unknown position {position!r} (use {', '.join(POSITIONS)})")
    x = width - w - margin if position[1] == "r" else margin
    y = height - h - margin if position[0] == "b" else margin
    if x < 2 or y < 2 or x + w > width - 2 or y + h > height - 2:
        raise ValueError(f"a {w}x{h} mark does not fit a {width}x{height} frame")
    return x, y, w, h


def region_box(width, height, region):
    """`(x, y, w, h)` in pixels of a region given as fractions of the frame,
    kept two pixels inside it (the fill filter needs a border to read)."""
    fx, fy, fw, fh = region
    x, y = max(2, round(fx * width)), max(2, round(fy * height))
    w = max(2, min(round(fw * width), width - 2 - x))
    h = max(2, min(round(fh * height), height - 2 - y))
    return x, y, w, h


def crop_filter(width, height, box):
    """Remove the full-width strip that holds `box` and everything between
    it and the nearer of the top and bottom edges."""
    x, y, w, h = box
    pad = max(2, (min(y, height - y - h)) // 2)
    if y + h / 2 >= height / 2:
        keep = even(max(2, y - pad))
        return f"crop={width}:{keep}:0:0"
    keep = even(max(2, height - (y + h + pad)))
    return f"crop={width}:{keep}:0:{height - keep}"


def fill_filter(box, kind="delogo"):
    """Fill `box` from its surroundings (`delogo`, FFmpeg's logo remover) or
    blur it in place: what a mark removal does to the pixels."""
    x, y, w, h = box
    if kind == "delogo":
        return f"delogo=x={x - 2}:y={y - 2}:w={w + 4}:h={h + 4}"
    if kind == "blur":
        luma = max(1, min(8, min(w, h) // 2 - 1))
        chroma = max(1, min(4, min(w, h) // 4 - 1))
        return (f"split[fa][fb];[fb]crop={w}:{h}:{x}:{y},boxblur=luma_radius={luma}:luma_power=2:chroma_radius={chroma}:chroma_power=2[fc];"
                f"[fa][fc]overlay=x={x}:y={y}")
    raise ValueError(f"unknown fill {kind!r} (use delogo or blur)")


def watermark_filters(width, height, positions=("br",), regions=(), fill="delogo", logo_fraction=0.14, margin_fraction=0.03):
    """Every watermark-control variant of one normalised `width` x `height`
    clip: `(transform, baseline transform, filtergraph, needs the logo, logo
    size)`. The baseline is the variant a paired comparison is made against.

    For each position: the sham mark added (`wm-mark-P`), the mark added and
    then removed by a crop or a fill (`wm-mark-P-crop`, `wm-mark-P-fill`),
    and the same crop and fill where there is no mark (`wm-crop-P`,
    `wm-fill-P`: the placebos). For each named region (where a clip may
    carry a real mark): the crop and the fill only."""
    out = [("wm-none", "", NORMALIZE_FILTERS, False, None)]
    for position in positions:
        box = logo_box(width, height, position, logo_fraction, margin_fraction)
        x, y, w, h = box
        crop, filled = crop_filter(width, height, box), fill_filter(box, fill)
        marked = f"[0:v]{NORMALIZE_FILTERS}[base];[base][1:v]overlay=x={x}:y={y}:format=auto"
        out.append((f"wm-mark-{position}", "wm-none", marked + "[v]", True, (w, h)))
        out.append((f"wm-mark-{position}-crop", f"wm-mark-{position}", f"{marked}[m];[m]{crop}[v]", True, (w, h)))
        out.append((f"wm-mark-{position}-fill", f"wm-mark-{position}", f"{marked}[m];[m]{filled}[v]", True, (w, h)))
        out.append((f"wm-crop-{position}", "wm-none", f"{NORMALIZE_FILTERS},{crop}", False, None))
        out.append((f"wm-fill-{position}", "wm-none", f"[0:v]{NORMALIZE_FILTERS}[base];[base]{filled}[v]" if fill == "blur" else f"{NORMALIZE_FILTERS},{filled}", False, None))
    for name, region in regions:
        box = region_box(width, height, region)
        crop, filled = crop_filter(width, height, box), fill_filter(box, fill)
        out.append((f"wm-crop-{name}", "wm-none", f"{NORMALIZE_FILTERS},{crop}", False, None))
        out.append((f"wm-fill-{name}", "wm-none", f"[0:v]{NORMALIZE_FILTERS}[base];[base]{filled}[v]" if fill == "blur" else f"{NORMALIZE_FILTERS},{filled}", False, None))
    return out


def watermark_baseline(transform):
    """The transform a watermark variant is compared against, or None for a
    transform that is not one: the marked clip for a removal, the unaltered
    control for an added mark and for a placebo."""
    if not transform.startswith("wm-") or transform == "wm-none":
        return None
    if transform.startswith("wm-mark-") and transform.rsplit("-", 1)[1] in ("crop", "fill"):
        return transform.rsplit("-", 1)[0]
    return "wm-none"


# ------------------------------------------------------------- the sham mark

def sham_logo_pixels(width, height, opacity=0.6):
    """RGBA rows of a neutral synthetic mark: a ring and three bars, white,
    at `opacity`. It is drawn here so that it resembles no real logo."""
    def inside(px, py):
        r = math.hypot(px - height / 2, py - height / 2)
        if 0.26 * height <= r <= 0.40 * height:
            return True
        for centre, end in ((0.25, 0.97), (0.50, 0.80), (0.75, 0.62)):
            if abs(py - centre * height) <= 0.07 * height and 1.15 * height <= px <= end * width:
                return True
        return False

    rows = []
    for y in range(height):
        row = bytearray()
        for x in range(width):
            covered = sum(inside(x + (i + 0.5) / 3, y + (j + 0.5) / 3) for i in range(3) for j in range(3)) / 9
            row += bytes((255, 255, 255, round(255 * opacity * covered)))
        rows.append(bytes(row))
    return rows


def png_bytes(rows, width, height):
    """An 8-bit RGBA PNG of `rows` (each `4 * width` bytes)."""
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    raw = b"".join(b"\x00" + row for row in rows)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def write_sham_logo(path, width, height, opacity=0.6):
    Path(path).write_bytes(png_bytes(sham_logo_pixels(width, height, opacity), width, height))


# -------------------------------------------------------------------- probing

def parse_rate(text):
    """`30000/1001` -> 29.97; an absent or zero rate is NaN."""
    try:
        num, _, den = str(text).partition("/")
        value = float(num) / (float(den) if den else 1.0)
    except (ValueError, ZeroDivisionError):
        return float("nan")
    return value if value > 0 else float("nan")


def gop_stats(flags):
    """`(keyframes, mean GOP length, longest GOP)` from the packet flags of
    a video stream in order (`K` marks a keyframe)."""
    starts = [i for i, flag in enumerate(flags) if "K" in flag]
    if not starts:
        return 0, float("nan"), 0
    lengths = [b - a for a, b in zip(starts, starts[1:] + [len(flags)])]
    return len(starts), len(flags) / len(starts), max(lengths)


def motion_statistic(frames):
    """Mean absolute difference between consecutive frames, in 8-bit luma
    levels; NaN with fewer than two frames."""
    if len(frames) < 2:
        return float("nan")
    total = sum(sum(abs(a - b) for a, b in zip(before, after)) for before, after in zip(frames, frames[1:]))
    return total / ((len(frames) - 1) * len(frames[0]))


PROBE_COLUMNS = ["path", "label", "generator", "dataset", *GROUP_COLUMNS, "container", "codec", "profile", "pix_fmt", "width", "height", "fps", "duration",
                 "frames", "bitrate", "bits_per_pixel", "audio", "keyframes", "gop_mean", "gop_max", "b_frames", "motion"]
MOTION_SIZE = (64, 36)
MOTION_FPS = 8


def probe_clip(path, motion_seconds=30.0):
    """Measurements of one clip. Motion is the mean absolute difference
    between frames 1/8 s apart at 64x36 grey over the first
    `motion_seconds`: a fixed scale and cadence, so clips of different
    size and frame rate compare."""
    info = json.loads(subprocess.check_output([
        "ffprobe", "-v", "error", "-show_entries",
        "format=format_name,duration,bit_rate:stream=codec_type,codec_name,profile,pix_fmt,width,height,avg_frame_rate,nb_frames,has_b_frames,bit_rate",
        "-of", "json", str(path)], text=True))
    video = next(s for s in info["streams"] if s.get("codec_type") == "video")
    flags = subprocess.check_output(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "packet=flags", "-of", "csv=p=0", str(path)], text=True).split()
    keyframes, gop_mean, gop_max = gop_stats(flags)
    fps = parse_rate(video.get("avg_frame_rate"))
    duration = float(info["format"].get("duration") or "nan")
    bitrate = float(video.get("bit_rate") or info["format"].get("bit_rate") or "nan")
    width, height = int(video["width"]), int(video["height"])
    w, h = MOTION_SIZE
    raw = subprocess.check_output(["ffmpeg", *FFMPEG_QUIET, "-t", str(motion_seconds), "-i", str(path), "-map", "0:v:0", "-an",
                                   "-vf", f"fps={MOTION_FPS},scale={w}:{h}:flags=area,format=gray", "-f", "rawvideo", "-"])
    frames = [raw[i:i + w * h] for i in range(0, len(raw) - w * h + 1, w * h)]
    return {
        "container": info["format"].get("format_name", ""), "codec": video.get("codec_name", ""), "profile": video.get("profile", ""), "pix_fmt": video.get("pix_fmt", ""),
        "width": width, "height": height, "fps": f"{fps:.3f}", "duration": f"{duration:.3f}", "frames": video.get("nb_frames") or len(flags), "bitrate": f"{bitrate:.0f}",
        "bits_per_pixel": f"{bitrate / (width * height * fps):.5f}", "audio": int(any(s.get("codec_type") == "audio" for s in info["streams"])),
        "keyframes": keyframes, "gop_mean": f"{gop_mean:.2f}", "gop_max": gop_max, "b_frames": video.get("has_b_frames", ""), "motion": f"{motion_statistic(frames):.4f}",
    }


NUMERIC_FEATURES = {
    "fps": lambda r: float(r["fps"]),
    "log2_pixels": lambda r: math.log2(float(r["width"]) * float(r["height"])),
    "log2_duration": lambda r: math.log2(float(r["duration"])),
    "log2_bitrate": lambda r: math.log2(float(r["bitrate"])),
    "log2_bits_per_pixel": lambda r: math.log2(float(r["bits_per_pixel"])),
    "gop_mean": lambda r: float(r["gop_mean"]),
    "motion": lambda r: float(r["motion"]),
}
CATEGORICAL_FEATURES = {
    "container": lambda r: r["container"],
    "codec": lambda r: r["codec"],
    "profile": lambda r: r["profile"],
    "pix_fmt": lambda r: r["pix_fmt"],
    "cadence": lambda r: r["fps"],
    "resolution": lambda r: f"{r['width']}x{r['height']}",
    "audio": lambda r: str(r["audio"]),
    "b_frames": lambda r: str(r["b_frames"]),
}
# Motion is content, not metadata: it is in the balance table and the
# matching, and out of the metadata-only probe unless asked for.
# What the balance table shows for each numeric feature: the plain quantity, not its logarithm.
DISPLAY = {
    "fps": ("frames per second", lambda r: float(r["fps"])),
    "log2_pixels": ("pixels per frame", lambda r: float(r["width"]) * float(r["height"])),
    "log2_duration": ("duration, s", lambda r: float(r["duration"])),
    "log2_bitrate": ("bitrate, kbit/s", lambda r: float(r["bitrate"]) / 1000),
    "log2_bits_per_pixel": ("bits per pixel", lambda r: float(r["bits_per_pixel"])),
    "gop_mean": ("mean GOP, frames", lambda r: float(r["gop_mean"])),
    "motion": ("motion, luma levels", lambda r: float(r["motion"])),
}
METADATA_NUMERIC = ("fps", "log2_pixels", "log2_duration", "log2_bitrate", "log2_bits_per_pixel", "gop_mean")
MATCH_FEATURES = ("fps", "log2_pixels", "motion", "log2_bits_per_pixel")


def numeric(row, name):
    try:
        value = NUMERIC_FEATURES[name](row)
    except (ValueError, ZeroDivisionError, KeyError):
        return float("nan")
    return value if math.isfinite(value) else float("nan")


def balance_rows(rows, flag_smd=0.25, flag_tv=0.1):
    """Per feature: the authentic and generated summaries, their distance
    (standardized mean difference, or total variation for a category) and
    whether it exceeds the flag level. Sizes, durations and rates are
    compared on a log2 scale and shown in plain units."""
    real = [r for r in rows if str(r["label"]) == "0"]
    fake = [r for r in rows if str(r["label"]) == "1"]
    out = []
    for name in NUMERIC_FEATURES:
        a = [v for v in (numeric(r, name) for r in real) if not math.isnan(v)]
        b = [v for v in (numeric(r, name) for r in fake) if not math.isnan(v)]
        d = standardized_difference(a, b)
        label, plain = DISPLAY[name]

        def summary(group):
            values = []
            for r in group:
                try:
                    values.append(plain(r))
                except (ValueError, KeyError):
                    pass
            values = [v for v in values if math.isfinite(v)]
            return f"{median(values):.6g} [{quantile(values, .25):.6g}, {quantile(values, .75):.6g}]" if values else "n/a"
        out.append((label, summary(real), summary(fake), "SMD", d, not math.isnan(d) and abs(d) > flag_smd))
    for name, get in CATEGORICAL_FEATURES.items():
        a, b = [get(r) for r in real], [get(r) for r in fake]
        d = total_variation(a, b)
        summary = lambda v: ", ".join(f"{k} {n}" for k, n in Counter(v).most_common(3)) + (" …" if len(set(v)) > 3 else "") if v else "n/a"
        out.append((name, summary(a), summary(b), "TV", d, not math.isnan(d) and d > flag_tv))
    return len(real), len(fake), out


def print_balance(rows, by, flag_smd, flag_tv):
    groups = defaultdict(list)
    for row in rows:
        groups[row.get(by) or "-"].append(row)
    for key in sorted(groups):
        n_real, n_fake, table = balance_rows(groups[key], flag_smd, flag_tv)
        print(f"\n**{by} = {key}**: {n_real} authentic, {n_fake} generated clips\n")
        print("| Feature | Authentic | Generated | Distance | Mismatch |")
        print("| --- | --- | --- | ---: | --- |")
        for name, a, b, kind, d, flagged in table:
            print(f"| {name} | {a} | {b} | {kind} {d:.2f} | {'yes' if flagged else ''} |")
    print("\nNumbers are median [quartiles]; categories list their most common values. SMD is the difference of class means over the pooled standard deviation, "
f"TV the total-variation distance between the class distributions. A mismatch is |SMD| > {flag_smd} or TV > {flag_tv}: a convention, not a test, "
          "and a feature that barely varies gives a large SMD for a small difference, so read the values beside it.")


def match_classes(rows, features=MATCH_FEATURES, ratio=1, caliper=1.0):
    """Greedy nearest-neighbour matching of generated to authentic clips on
    z-scored `features`: each authentic clip takes up to `ratio` generated
    clips within `caliper` (Euclidean distance in standard deviations),
    closest pairs first, each generated clip used once. Clips that share a
    `scene_id` are only matched to each other. Returns the kept rows."""
    usable = [r for r in rows if all(not math.isnan(numeric(r, f)) for f in features)]
    stats = {}
    for f in features:
        values = [numeric(r, f) for r in usable]
        mean = sum(values) / max(1, len(values))
        stats[f] = (mean, math.sqrt(sum((v - mean) ** 2 for v in values) / max(1, len(values) - 1)) or 1.0)
    point = lambda r: [(numeric(r, f) - stats[f][0]) / stats[f][1] for f in features]
    real = [(r, point(r)) for r in usable if str(r["label"]) == "0"]
    fake = [(r, point(r)) for r in usable if str(r["label"]) == "1"]
    pairs = []
    for i, (a, pa) in enumerate(real):
        for j, (b, pb) in enumerate(fake):
            if (a.get("scene_id") or b.get("scene_id")) and a.get("scene_id") != b.get("scene_id"):
                continue
            distance = math.dist(pa, pb)
            if distance <= caliper:
                pairs.append((distance, i, j))
    pairs.sort()
    taken, used = Counter(), set()
    for _, i, j in pairs:
        if taken[i] < ratio and j not in used:
            taken[i] += 1
            used.add(j)
    keep = [real[i][0] for i in sorted(taken)] + [fake[j][0] for j in sorted(used)]
    return keep


# ------------------------------------------------------ metadata-only probe

def featurize(train, numeric_names, categorical_names):
    """A function from a probe row to a feature vector, fitted on `train`:
    z-scored numbers (missing -> 0) and one-hot categories seen in training."""
    stats = {}
    for name in numeric_names:
        values = [v for v in (numeric(r, name) for r in train) if not math.isnan(v)]
        mean = sum(values) / len(values) if values else 0.0
        stats[name] = (mean, math.sqrt(sum((v - mean) ** 2 for v in values) / max(1, len(values) - 1)) or 1.0)
    levels = [(name, level) for name in categorical_names for level in sorted({CATEGORICAL_FEATURES[name](r) for r in train})]
    names = list(numeric_names) + [f"{name}={level}" for name, level in levels]

    def vector(row):
        out = []
        for name in numeric_names:
            value = numeric(row, name)
            out.append(0.0 if math.isnan(value) else (value - stats[name][0]) / stats[name][1])
        out.extend(1.0 if CATEGORICAL_FEATURES[name](row) == level else 0.0 for name, level in levels)
        return out
    return vector, names


def fit_logistic(xs, ys, l2=0.01, steps=300, rate=0.5):
    """Class-balanced, L2-regularised logistic regression by full-batch
    gradient descent; returns `(weights, bias)`."""
    n_pos = sum(ys)
    n_neg = len(ys) - n_pos
    weight = [0.5 / n_pos if y else 0.5 / n_neg for y in ys]
    w, b = [0.0] * len(xs[0]), 0.0
    for _ in range(steps):
        gw, gb = [l2 * v for v in w], 0.0
        for x, y, sw in zip(xs, ys, weight):
            z = b + sum(wi * xi for wi, xi in zip(w, x))
            error = sw * (1 / (1 + math.exp(-max(-30.0, min(30.0, z)))) - y)
            gb += error
            for k, xi in enumerate(x):
                if xi:
                    gw[k] += error * xi
        w = [wi - rate * gi for wi, gi in zip(w, gw)]
        b -= rate * gb
    return w, b


def grouped_folds(groups, labels, folds, seed=0):
    """Fold index per item so that a group is never split, with the groups
    of each class spread over the folds. None when either class has fewer
    than two groups: no fold can then hold a group out and still train on
    that class."""
    by_class = defaultdict(set)
    for group, label in zip(groups, labels):
        by_class[label].add(group)
    if len(by_class) < 2 or min(len(v) for v in by_class.values()) < 2:
        return None
    k = max(2, min(folds, min(len(v) for v in by_class.values())))
    rng = random.Random(seed)
    fold_of = {}
    for label in sorted(by_class):
        ordered = sorted(by_class[label], key=str)
        rng.shuffle(ordered)
        for i, group in enumerate(ordered):
            fold_of.setdefault(group, i % k)
    return [fold_of[group] for group in groups]


def metadata_probe(rows, group_keys=("source", "scene", "parent"), folds=5, seed=0, rounds=1000, numeric_names=METADATA_NUMERIC, categorical_names=tuple(CATEGORICAL_FEATURES)):
    """Out-of-fold AUROC of a label classifier that sees only metadata, with
    a bootstrap interval over the cross-validation groups. The first of
    `group_keys` that can hold groups out is used. Returns a dict, or None
    when the rows have one class."""
    labels = [int(r["label"]) for r in rows]
    if len(set(labels)) < 2:
        return None
    keys = {"source": lambda r: source_of({**r, "label": int(r["label"])}), "scene": lambda r: r.get("scene_id") or r.get("parent_id") or r["path"], "parent": lambda r: r.get("parent_id") or r["path"]}
    chosen = assignment = groups = None
    for key in group_keys:
        groups = [keys[key](r) for r in rows]
        if any(not g for g in groups):
            continue
        assignment = grouped_folds(groups, labels, folds, seed)
        if assignment:
            chosen = key
            break
    if not chosen:
        return {"grouping": None}
    scores = [0.0] * len(rows)
    for fold in sorted(set(assignment)):
        train = [i for i, f in enumerate(assignment) if f != fold]
        if len({labels[i] for i in train}) < 2:
            continue
        vector, _ = featurize([rows[i] for i in train], numeric_names, categorical_names)
        w, b = fit_logistic([vector(rows[i]) for i in train], [labels[i] for i in train])
        for i, f in enumerate(assignment):
            if f == fold:
                scores[i] = b + sum(wi * xi for wi, xi in zip(w, vector(rows[i])))
    stat = lambda items: auroc([s for s, y in items if y == 1], [s for s, y in items if y == 0])
    items = list(zip(scores, labels))
    low, high = bootstrap_clustered(items, groups, stat, rounds, seed)
    vector, names = featurize(rows, numeric_names, categorical_names)
    w, _ = fit_logistic([vector(r) for r in rows], labels)
    top = sorted(zip(names, w), key=lambda nw: -abs(nw[1]))[:3]
    return {"grouping": chosen, "groups": len(set(groups)), "folds": len(set(assignment)), "auroc": stat(items), "low": low, "high": high, "top": top}


# ---------------------------------------------------------------- subcommands

def cmd_collection(args):
    if args.type not in COLLECTION_TYPES:
        raise SystemExit(f"unknown collection type {args.type!r} (use {', '.join(COLLECTION_TYPES)})")
    label = 1 if args.type == "generated" else 0
    overlap = training_overlap(args.source_id) if label == 0 else None
    if overlap and not args.not_held_out:
        raise SystemExit(f"{args.source_id} is not held out: {overlap} (docs/DATA.md). Pass --not-held-out to register it as an in-distribution comparison.")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    index = {}
    if args.index:
        index = {row["file"]: row for row in read_table(args.index)}
    files = sorted(p for p in args.clips.rglob("*") if p.suffix.lower() in VIDEO_SUFFIXES and p.is_file())
    if not files:
        raise SystemExit(f"no video files under {args.clips}")
    if index and {p.relative_to(args.clips).as_posix() for p in files} != index.keys():
        raise SystemExit("--index must list exactly the video files under the clips directory (column `file`, relative paths)")
    manifest_path, audit_path, registry_path = out / "manifest.tsv", out / "audit.tsv", out / "collections.tsv"
    rows = read_manifest(manifest_path) if manifest_path.exists() else []
    audit = read_table(audit_path) if audit_path.exists() else []
    registry = read_table(registry_path) if registry_path.exists() else []
    if any(r["source_id"] == args.source_id for r in registry):
        raise SystemExit(f"{args.source_id} is already registered in {registry_path}")
    seen = {parent_of(r) for r in rows}
    for path in files:
        rel = path.relative_to(args.clips)
        extra = index.get(rel.as_posix(), {})
        parent = f"{args.source_id}/{rel.with_suffix('').as_posix()}"
        if parent in seen:
            raise SystemExit(f"duplicate parent_id {parent}")
        seen.add(parent)
        dest = Path("clips") / safe_name(args.source_id) / rel
        link_or_copy(path, out / dest, args.copy)
        generator = extra.get("generator") or (args.generator if label else f"real:{args.source_id}")
        if not generator:
            raise SystemExit("a generated collection needs --generator or a `generator` column in --index")
        rows.append({"rel": dest.as_posix(), "label": label, "generator": generator, "dataset": args.dataset, "parent_id": parent, "scene_id": extra.get("scene_id", ""),
                     "source_id": args.source_id, "generator_family": extra.get("generator_family") or args.family or "", "transform": "original"})
        # Clips cut from one recording are one cluster: by default a sub-directory is one source video.
        source_video = extra.get("source_video") or (f"{args.source_id}/{rel.parent.as_posix()}" if rel.parent != Path(".") else parent)
        audit.append({"parent_id": parent, "source_id": args.source_id, "collection_type": args.type, "source_video": source_video, "start_frame": extra.get("start_frame", ""),
                      "file": str(path), "sha256": digest(path)})
    registry.append({"source_id": args.source_id, "collection_type": args.type, "label": label, "held_out": "no" if overlap else "yes", "clips": len(files),
                     "licence": args.licence, "licence_source": args.licence_source, "note": overlap or ""})
    write_manifest(manifest_path, rows)
    write_table(audit_path, ["parent_id", "source_id", "collection_type", "source_video", "start_frame", "file", "sha256"], [list(r.values()) for r in audit])
    write_table(registry_path, ["source_id", "collection_type", "label", "held_out", "clips", "licence", "licence_source", "note"], [list(r.values()) for r in registry])
    print(f"Registered {len(files)} {args.type} clips as {args.source_id} ({'NOT held out: ' + overlap if overlap else 'held out'}); {len(rows)} clips in {manifest_path}")


def cmd_start_frames(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    table = []
    for row in read_manifest(args.manifest):
        if row["label"] != 0:
            continue
        name = safe_name(parent_of(row)) + ".png"
        subprocess.run(["ffmpeg", *FFMPEG_QUIET, "-y", "-i", str(row["path"]), "-map", "0:v:0", "-frames:v", "1", str(out / name)], check=True)
        table.append((parent_of(row), row["scene_id"] or parent_of(row), name, digest(out / name)))
    write_table(out / "start-frames.tsv", ["start_frame", "scene_id", "file", "sha256"], table)
    print(f"Wrote {len(table)} first frames to {out}; give a generated clip its row's `start_frame` and `scene_id` in the --index of `collection`.")


def cmd_probe(args):
    rows = [row for manifest in args.manifest for row in read_manifest(manifest)]

    def one(row):
        measured = probe_clip(row["path"], args.motion_seconds)
        return [str(row["path"]), row["label"], row["generator"], row["dataset"], *(row[c] for c in GROUP_COLUMNS), *(measured[c] for c in PROBE_COLUMNS[9:])]
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        table = list(pool.map(one, rows))
    write_table(args.out, PROBE_COLUMNS, table)
    print(f"Probed {len(table)} clips into {args.out}")


def cmd_balance(args):
    print_balance(read_table(args.probe), args.by, args.flag_smd, args.flag_tv)


def cmd_match(args):
    probed = {row["path"]: row for row in read_table(args.probe)}
    rows = read_manifest(args.manifest)
    joined = []
    for row in rows:
        measured = probed.get(str(row["path"]))
        if measured is None:
            raise SystemExit(f"{row['path']} is not in {args.probe}")
        joined.append({**measured, "_row": row})
    groups = defaultdict(list)
    for row in joined:
        groups[row["transform"] or "-"].append(row)
    kept = []
    for key in sorted(groups):
        keep = match_classes(groups[key], args.features, args.ratio, args.caliper)
        print(f"transform {key}: kept {sum(str(r['label']) == '0' for r in keep)} authentic and {sum(str(r['label']) == '1' for r in keep)} generated of {len(groups[key])} clips")
        kept.extend(keep)
    base = args.out.resolve().parent
    out_rows = [{**r["_row"], "rel": os.path.relpath(r["_row"]["path"], base)} for r in kept]
    write_manifest(args.out, out_rows)
    print_balance(kept, "transform", args.flag_smd, args.flag_tv)


def parent_table(rows):
    """Parents in manifest order with a unique file-safe name each."""
    names, parents = {}, []
    for row in rows:
        parent = parent_of(row)
        name = safe_name(parent)
        if names.setdefault(name, parent) != parent:
            raise SystemExit(f"parents {parent!r} and {names[name]!r} map to the same file name; rename one")
        parents.append((row, parent, name))
    if len({p for _, p, _ in parents}) != len(parents):
        raise SystemExit("the input manifest lists a parent_id twice: give `variants` and `watermark` one row per original clip")
    return parents


def derived_transform(row, transform):
    """A variant of an already transformed clip keeps the history."""
    before = row["transform"]
    return transform if before in ("", "original") else f"{before}+{transform}"


def finish_variants(out, produced, settings):
    produced.sort(key=lambda r: (r["transform"], r["rel"]))
    write_manifest(out / "manifest.tsv", produced)
    write_table(out / "audit.tsv", ["path", "parent_id", "transform", "baseline", "made_from", "sha256"],
                [(r["rel"], r["parent_id"], r["transform"], r["baseline"], r["made_from"], digest(out / r["rel"])) for r in produced])
    settings["ffmpeg"] = subprocess.check_output(["ffmpeg", "-version"], text=True).splitlines()[0]
    settings["clips"] = dict(sorted(Counter(r["transform"] for r in produced).items()))
    (out / "run.json").write_text(json.dumps(settings, indent=2) + "\n")
    print(f"Wrote {len(produced)} clips to {out}: " + ", ".join(f"{n} {t}" for t, n in settings["clips"].items()))


def cmd_variants(args):
    known = ["original", "trim-copy", "davodi-norm", "fixed-gop"] + [f"repost-g{k + 1}" for k in range(len(args.ladder))]
    wanted = [v for v in args.variants.split(",") if v]
    if set(wanted) - set(known):
        raise SystemExit(f"unknown variants {sorted(set(wanted) - set(known))} (use {', '.join(known)})")
    generations = max([int(v[len("repost-g"):]) for v in wanted if v.startswith("repost-g")] or [0])
    controlled = args.repost_from == "fixed-gop"
    if controlled and generations and "fixed-gop" not in wanted:
        raise SystemExit("--repost-from fixed-gop needs fixed-gop among --variants")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    parents = parent_table(read_manifest(args.manifest))

    def one(task):
        row, parent, name = task
        made = []

        def add(transform, rel, made_from, command=None, baseline="original"):
            (out / rel).parent.mkdir(parents=True, exist_ok=True)
            if command:
                subprocess.run([*command[:1], "-y", *command[1:]], check=True)
            made.append({**row, "rel": rel.as_posix(), "parent_id": parent, "transform": derived_transform(row, transform), "baseline": "" if transform == "original" else derived_transform(row, baseline), "made_from": made_from})
        source = row["path"]
        if "original" in wanted:
            rel = Path("original") / (name + source.suffix.lower())
            link_or_copy(source, out / rel, args.copy)
            add("original", rel, str(source))
        if "trim-copy" in wanted:
            rel = Path("trim-copy") / (name + source.suffix.lower())
            add("trim-copy", rel, str(source), trim_command(source, out / rel, args.seconds))
        if "davodi-norm" in wanted:
            rel = Path("davodi-norm") / (name + ".mp4")
            add("davodi-norm", rel, str(source), normalize_command(source, out / rel, None, args.seconds))
        if "fixed-gop" in wanted:
            rel = Path("fixed-gop") / (name + ".mp4")
            add("fixed-gop", rel, str(source), normalize_command(source, out / rel, FIXED_GOP, args.seconds))
        # Reposts of the original keep its cadence and so its cadence shortcut; reposts of the fixed-GOP clip are the controlled cohort.
        previous = out / "fixed-gop" / (name + ".mp4") if controlled else source
        for k in range(generations):
            width, crf = args.ladder[k]
            transform = f"repost-g{k + 1}"
            listed = f"fixed-gop+{transform}" if controlled else transform
            # Every generation is made, so that generation k really is a re-encode of generation k-1; only the wanted ones are listed.
            rel = Path(listed) / (name + ".mp4")
            (out / rel).parent.mkdir(parents=True, exist_ok=True)
            command = repost_command(previous, out / rel, width, crf, args.seconds)
            if transform in wanted:
                add(listed, rel, str(previous), command, "fixed-gop" if controlled else "original")
            else:
                subprocess.run([*command[:1], "-y", *command[1:]], check=True)
            previous = out / rel
        return made
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        produced = [row for made in pool.map(one, parents) for row in made]
    finish_variants(out, produced, {
        "variants": wanted, "seconds": args.seconds, "repost_from": args.repost_from,
        "davodi-norm": {"filters": NORMALIZE_FILTERS, "codec": "libx264", "crf": NORMALIZE_CRF, "preset": "veryfast", "pix_fmt": "yuv420p", "gop": "encoder default (keyint 250, scene cuts on)"},
        "fixed-gop": {"filters": NORMALIZE_FILTERS, "codec": "libx264", "crf": NORMALIZE_CRF, "preset": "veryfast", "pix_fmt": "yuv420p", "gop": FIXED_GOP, "keyint_min": FIXED_GOP, "sc_threshold": 0},
        "repost": {"ladder_max_width_crf": [list(step) for step in args.ladder], "scale": "bicubic, never upscaled", "codec": "libx264", "preset": "medium", "pix_fmt": "yuv420p", "chain": "generation k re-encodes generation k-1"},
        "trim-copy": "stream copy of the first seconds, no re-encode",
        "example_commands": {"fixed-gop": " ".join(normalize_command("IN", "OUT", FIXED_GOP, args.seconds)), "repost-g1": " ".join(repost_command("IN", "OUT", *args.ladder[0], args.seconds))},
    })


def output_size(path):
    stream, = json.loads(subprocess.check_output(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "json", str(path)], text=True))["streams"]
    return int(stream["width"]), int(stream["height"])


def cmd_watermark(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    (out / "logos").mkdir(exist_ok=True)
    parents = parent_table(read_manifest(args.manifest))
    positions = [p for p in args.positions.split(",") if p]
    regions = []
    for spec in args.region:
        name, _, numbers = spec.partition("=")
        values = tuple(float(v) for v in numbers.split(","))
        if not re.fullmatch(r"[a-z0-9]+", name) or name in POSITIONS + ("none", "mark") or len(values) != 4 or not all(0 <= v <= 1 for v in values):
            raise SystemExit(f"bad --region {spec!r}: use name=x,y,w,h as fractions of the frame, name in lower-case letters and digits")
        regions.append((name, values))

    def one(task):
        row, parent, name = task
        made = []
        source = row["path"]
        base = out / "wm-none" / (name + ".mp4")
        base.parent.mkdir(parents=True, exist_ok=True)
        # The unaltered control is encoded first: its frame size places the mark, the crop and the fill.
        command = normalize_command(source, base, FIXED_GOP, args.seconds)
        subprocess.run([*command[:1], "-y", *command[1:]], check=True)
        width, height = output_size(base)
        for transform, baseline, filters, needs_logo, logo_size in watermark_filters(width, height, positions, regions, args.fill, args.logo_width, args.margin):
            rel = Path(transform) / (name + ".mp4")
            (out / rel).parent.mkdir(parents=True, exist_ok=True)
            if transform != "wm-none":
                logo = None
                if needs_logo:
                    logo = out / "logos" / f"sham-{logo_size[0]}x{logo_size[1]}.png"
                    if not logo.exists():
                        write_sham_logo(logo, *logo_size, args.opacity)
                complex_graph = filters.endswith("[v]")
                if complex_graph and not logo:
                    command = ["ffmpeg", *FFMPEG_QUIET, "-threads", "1", "-i", str(source), "-t", str(args.seconds), "-filter_complex", filters, "-map", "[v]", "-an",
                               "-map_metadata", "-1", *encode_args(NORMALIZE_CRF, "veryfast", FIXED_GOP), str(out / rel)]
                else:
                    command = normalize_command(source, out / rel, FIXED_GOP, args.seconds, filters, logo)
                subprocess.run([*command[:1], "-y", *command[1:]], check=True)
            made.append({**row, "rel": rel.as_posix(), "parent_id": parent, "transform": derived_transform(row, transform), "baseline": derived_transform(row, baseline) if baseline else "", "made_from": str(source)})
        return made
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        produced = [row for made in pool.map(one, parents) for row in made]
    finish_variants(out, produced, {
        "seconds": args.seconds, "positions": positions, "regions": {name: list(values) for name, values in regions}, "fill": args.fill,
        "logo": {"width_fraction": args.logo_width, "aspect": "3:1", "margin_fraction": args.margin, "opacity": args.opacity, "design": "synthetic ring and three bars, drawn by this script"},
        "encode": {"filters": NORMALIZE_FILTERS, "codec": "libx264", "crf": NORMALIZE_CRF, "preset": "veryfast", "pix_fmt": "yuv420p", "gop": FIXED_GOP, "keyint_min": FIXED_GOP, "sc_threshold": 0},
        "note": "every variant is one encode of the original with the alteration applied to the normalised frames; wm-none is the fixed-GOP control",
    })


def cmd_metadata_probe(args):
    rows = read_table(args.probe)
    groups = defaultdict(list)
    for row in rows:
        groups[row.get(args.by) or "-"].append(row)
    numeric_names = METADATA_NUMERIC + (("motion",) if args.with_motion else ())
    print(f"| {args.by} | Authentic | Generated | Held out by | Groups | AUROC | 95% interval | Heaviest features |")
    print("| --- | ---: | ---: | --- | ---: | ---: | --- | --- |")
    for key in sorted(groups):
        cohort = groups[key]
        n_real, n_fake = sum(r["label"] == "0" for r in cohort), sum(r["label"] == "1" for r in cohort)
        result = metadata_probe(cohort, tuple(args.group.split(",")), args.folds, args.seed, args.rounds, numeric_names)
        if result is None:
            print(f"| {key} | {n_real} | {n_fake} | one class only | | | | |")
        elif result["grouping"] is None:
            print(f"| {key} | {n_real} | {n_fake} | too few groups per class | | | | |")
        else:
            top = ", ".join(f"{name} ({weight:+.2f})" for name, weight in result["top"])
            print(f"| {key} | {n_real} | {n_fake} | {result['grouping']} | {result['groups']} | {result['auroc']:.3f} | {result['low']:.3f}–{result['high']:.3f} | {top} |")
    print("\nA logistic regression on container, codec, cadence, resolution, duration, bitrate and GOP features"
          + (" and motion" if args.with_motion else "") + " only, scored out of fold; folds never split a group, and the interval resamples whole groups. "
          "An AUROC well above 0.5 means the cohort's label can be read from its metadata: fix the cohort before trusting a detector's result on it. "
          "`Held out by` is the first of the requested groupings with at least two groups in each class (a clip without a scene is its own scene); "
          "with one source in a class, a held-out-source probe is impossible and every source difference is confounded with the label.")


# ------------------------------------------------------------------ self-test

def self_test():
    close = lambda a, b, tol: abs(a - b) <= tol
    # Reference values of src/metrics.rs (R: binom.test(k, n, alternative = "less")$conf.int[2]).
    assert close(clopper_pearson_upper(0, 3000), 0.000998, 2e-6), clopper_pearson_upper(0, 3000)
    assert clopper_pearson_upper(0, 2994) > 0.001 >= clopper_pearson_upper(0, 2995)
    assert close(clopper_pearson_upper(3, 20), 0.3437, 5e-4), clopper_pearson_upper(3, 20)
    assert close(clopper_pearson_upper(130, 10747), 0.01398, 2e-5)
    assert close(clopper_pearson_lower(3, 20), 0.0422, 1e-3)
    assert clopper_pearson_upper(5, 5) == 1.0 and clopper_pearson_lower(0, 5) == 0.0 and math.isnan(clopper_pearson_upper(0, 0))
    assert close(sign_test_p(0, 5), 0.0625, 1e-12) and sign_test_p(3, 3) == 1.0 and sign_test_p(0, 0) == 1.0
    assert auroc([3, 4], [1, 2]) == 1.0 and auroc([1, 2], [1, 2]) == 0.5 and auroc([1], [2]) == 0.0 and math.isnan(auroc([], [1]))
    assert close(auroc([0.9, 0.4, 0.4], [0.4, 0.1]), (1 + 1 + .5 + 1 + .5 + 1) / 6, 1e-12)
    assert empirical_cutoff([0.9, 0.5, 0.1], 0.5) == (0.5, 1) and empirical_cutoff([0.9, 0.5, 0.1], 0.01) == (0.9, 0)
    items = [(1.0, 1)] * 10 + [(0.0, 0)] * 10
    low, high = bootstrap_clustered(items, list(range(20)), lambda s: sum(v for v, _ in s) / len(s), 200, 1)
    assert low < 0.5 < high and bootstrap_clustered(items, [0] * 20, lambda s: sum(v for v, _ in s) / len(s), 50, 1) == (0.5, 0.5)
    assert standardized_difference([1, 1], [1, 1]) == 0 and math.isinf(standardized_difference([1, 1], [2, 2])) and close(standardized_difference([0, 2], [1, 3]), 1 / math.sqrt(2), 1e-12)
    assert total_variation(["a", "a"], ["a", "b"]) == 0.5 and median([3, 1, 2]) == 2 and quantile([0, 10], 0.25) == 2.5

    # Manifests round-trip and fall back as the Rust reader does.
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        (tmp / "m.tsv").write_text("a.mp4\t0\treal:x\tset\nb.mp4\t1\tgen\tset\tp1\t-\tsrc\t\tfixed-gop\textra\n")
        first, second = read_manifest(tmp / "m.tsv")
        assert first["parent_id"] == "" and parent_of(first) == "a.mp4" and source_of(first) == "real:x" and first["path"] == tmp / "a.mp4"
        assert (second["parent_id"], second["scene_id"], second["source_id"], second["generator_family"], second["transform"], second["extra"]) == ("p1", "", "src", "", "fixed-gop", ["extra"])
        assert manifest_line("b.mp4", second) == "b.mp4\t1\tgen\tset\tp1\t-\tsrc\t-\tfixed-gop"
        write_manifest(tmp / "n.tsv", [second])
        assert read_manifest(tmp / "n.tsv")[0]["transform"] == "fixed-gop"
        write_sham_logo(tmp / "logo.png", 12, 4)
        blob = (tmp / "logo.png").read_bytes()
        assert blob[:8] == b"\x89PNG\r\n\x1a\n" and struct.unpack(">II", blob[16:24]) == (12, 4) and blob[-8:-4] == b"IEND"
    assert source_of({"source_id": "", "generator": "g", "dataset": "d", "label": 1}) == "d"
    assert training_overlap("Pexels-2026") and training_overlap("kinetics-400") and training_overlap("MSR-VTT") and training_overlap("pixabay") is None
    assert safe_name("pixabay/01 clip") == "pixabay_01_clip" and derived_transform({"transform": "original"}, "fixed-gop") == "fixed-gop" and derived_transform({"transform": "davodi-norm"}, "repost-g1") == "davodi-norm+repost-g1"

    # The normalisation here is the one prepare-davodi-video.py runs.
    sibling = (Path(__file__).resolve().parent / "prepare-davodi-video.py").read_text()
    assert f'filters = "{NORMALIZE_FILTERS}"' in sibling and '"-crf", "18", "-preset", "veryfast", "-pix_fmt", "yuv420p"' in sibling and '"-t", "4"' in sibling
    assert normalize_command("in.mp4", "out.mp4") == [
        "ffmpeg", "-nostdin", "-hide_banner", "-loglevel", "error", "-threads", "1", "-i", "in.mp4", "-t", "4", "-map", "0:v:0", "-an", "-vf", NORMALIZE_FILTERS, "-map_metadata", "-1",
        "-c:v", "libx264", "-threads", "1", "-crf", "18", "-preset", "veryfast", "-pix_fmt", "yuv420p", "out.mp4"]
    fixed = normalize_command("in.mp4", "out.mp4", FIXED_GOP)
    at = fixed.index("-g")
    assert fixed[at:at + 6] == ["-g", "48", "-keyint_min", "48", "-sc_threshold", "0"] and fixed[:at] + fixed[at + 6:] == normalize_command("in.mp4", "out.mp4")
    assert "scale=w='min(iw,720)':h=-2:flags=bicubic,setsar=1" in repost_command("a", "b", 720, 28) and "28" in repost_command("a", "b", 720, 28)
    assert trim_command("a", "b")[-4:] == ["copy", "-map_metadata", "-1", "b"]

    # Mark geometry: inside the frame, even-sized, in the named corner.
    assert logo_box(800, 450, "br") == (664, 390, 112, 36) and logo_box(800, 450, "tl") == (24, 24, 112, 36) and even(7.2, up=True) == 8 and even(7) == 6
    assert crop_filter(800, 450, logo_box(800, 450, "br")) == "crop=800:378:0:0" and crop_filter(800, 450, logo_box(800, 450, "tl")) == "crop=800:378:0:72"
    assert fill_filter((664, 390, 112, 36)) == "delogo=x=662:y=388:w=116:h=40" and "boxblur" in fill_filter((664, 390, 112, 36), "blur")
    assert region_box(800, 450, (0.8, 0.85, 0.5, 0.5)) == (640, 382, 158, 66)
    variants = watermark_filters(800, 450, ("br",), (("veo", (0.82, 0.86, 0.15, 0.1)),))
    assert [v[0] for v in variants] == ["wm-none", "wm-mark-br", "wm-mark-br-crop", "wm-mark-br-fill", "wm-crop-br", "wm-fill-br", "wm-crop-veo", "wm-fill-veo"]
    assert all(watermark_baseline(name) == baseline for name, baseline, *_ in variants[1:]) and watermark_baseline("wm-none") is None and watermark_baseline("fixed-gop") is None
    assert [v[1] for v in variants[1:6]] == ["wm-none", "wm-mark-br", "wm-mark-br", "wm-none", "wm-none"] and [v[3] for v in variants[:6]] == [False, True, True, True, False, False]
    pixels = sham_logo_pixels(36, 12, 0.5)
    alphas = [row[4 * x + 3] for row in pixels for x in range(36)]
    assert len(pixels) == 12 and max(alphas) == 128 and min(alphas) == 0 and 0.1 < sum(a > 0 for a in alphas) / len(alphas) < 0.7

    # Probe arithmetic.
    assert close(parse_rate("30000/1001"), 29.97, 1e-3) and parse_rate("24") == 24 and math.isnan(parse_rate("0/0"))
    assert gop_stats(["K_", "__", "__", "K_", "__"]) == (2, 2.5, 3) and gop_stats(["__"])[0] == 0
    assert motion_statistic([bytes([0, 0]), bytes([2, 4]), bytes([2, 4])]) == 1.5 and math.isnan(motion_statistic([bytes([1])]))

    # Balance, matching and the metadata probe on synthetic probe rows.
    rng = random.Random(3)

    def row(label, source, scene, fps, width, bitrate, motion):
        return {"path": f"{source}-{scene}-{rng.random()}", "label": str(label), "generator": source, "dataset": "d", "parent_id": f"{source}-{scene}", "scene_id": scene, "source_id": source,
                "generator_family": "", "transform": "t", "container": "mov,mp4", "codec": "h264", "profile": "High", "pix_fmt": "yuv420p", "width": str(width), "height": "450", "fps": f"{fps:.3f}",
                "duration": "4.000", "frames": "96", "bitrate": str(bitrate), "bits_per_pixel": f"{bitrate / (width * 450 * fps):.5f}", "audio": "0", "keyframes": "2", "gop_mean": "48.00", "gop_max": "48",
                "b_frames": "2", "motion": f"{motion:.4f}"}
    shortcut = [row(0, f"real{k % 3}", f"s{k}", 30, 800, 2_000_000 + rng.randrange(1000), rng.random() * 5) for k in range(30)]
    shortcut += [row(1, f"gen{k % 3}", f"s{k}", 24, 800, 2_000_000 + rng.randrange(1000), rng.random() * 5) for k in range(30)]
    n_real, n_fake, table = balance_rows(shortcut)
    flagged = {name for name, *_, bad in table if bad}
    assert (n_real, n_fake) == (30, 30) and {"frames per second", "cadence"} <= flagged and "pixels per frame" not in flagged and "codec" not in flagged
    assert table[0][1:3] == ("30 [30, 30]", "24 [24, 24]")
    result = metadata_probe(shortcut, rounds=50)
    assert result["grouping"] == "source" and result["groups"] == 6 and result["auroc"] == 1.0 and result["low"] == 1.0
    clean = [row(k % 2, f"{'gen' if k % 2 else 'real'}{k % 3}", f"s{k // 2}", 24, 800, 2_000_000 + rng.randrange(400_000), rng.random() * 5) for k in range(120)]
    result = metadata_probe(clean, rounds=50)
    assert result["grouping"] == "source" and 0.25 < result["auroc"] < 0.75, result
    one_source = [dict(r, source_id="pixabay" if r["label"] == "0" else r["source_id"]) for r in shortcut]
    assert metadata_probe(one_source, rounds=20)["grouping"] == "scene" and metadata_probe([r for r in shortcut if r["label"] == "0"]) is None
    assert grouped_folds(["a", "a", "b"], [0, 0, 1], 5) is None
    folds = grouped_folds(["a", "a", "b", "c", "d", "e"], [0, 0, 0, 1, 1, 1], 5)
    assert folds[0] == folds[1] and len(set(folds)) == 2
    w, b = fit_logistic([[1.0], [-1.0], [1.0], [-1.0]], [1, 0, 1, 0])
    assert w[0] > 0.5 and abs(b) < 1e-9
    # Matching keeps the generated clips nearest each authentic one and drops the outliers.
    pool = [row(0, "real", "", 24, 800, 2_000_000, 2.0), row(1, "gen", "", 24, 800, 2_000_000, 2.1), row(1, "gen", "", 60, 800, 9_000_000, 9.0), row(0, "real", "", 12, 800, 500_000, 0.1)]
    kept = match_classes(pool, caliper=1.0)
    assert [r["motion"] for r in kept] == ["2.0000", "2.1000"]
    print("video_audit self-test passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true", help="check the pure functions and exit")
    sub = parser.add_subparsers(dest="command")

    p = sub.add_parser("collection", help="register a collection of clips already on disk")
    p.add_argument("output", type=Path, help="cohort directory: manifest.tsv, audit.tsv and collections.tsv are created or extended")
    p.add_argument("--source-id", required=True, help="manifest column 7, e.g. pixabay-2026 or cctv-virat")
    p.add_argument("--type", required=True, help="one of " + ", ".join(COLLECTION_TYPES))
    p.add_argument("--clips", required=True, type=Path, help="directory of video files; a sub-directory is treated as one source video")
    p.add_argument("--licence", required=True, help="the licence the clips are used under, as read at --licence-source")
    p.add_argument("--licence-source", required=True, help="URL or file where the licence was read, and the date")
    p.add_argument("--dataset", default="video-audit", help="manifest column 4")
    p.add_argument("--generator", help="generator name of a generated collection")
    p.add_argument("--family", help="generator family (column 8) of a generated collection")
    p.add_argument("--index", type=Path, help="TSV with a `file` column and any of scene_id, source_video, start_frame, generator, generator_family")
    p.add_argument("--not-held-out", action="store_true", help="register a source the weights have already seen, marked as such")
    p.add_argument("--copy", action="store_true", help="copy the clips instead of linking them")
    p.set_defaults(run=cmd_collection)

    p = sub.add_parser("start-frames", help="export first frames of authentic clips for image-to-video")
    p.add_argument("manifest", type=Path)
    p.add_argument("output", type=Path)
    p.set_defaults(run=cmd_start_frames)

    p = sub.add_parser("probe", help="measure every clip of one or more manifests")
    p.add_argument("manifest", type=Path, nargs="+")
    p.add_argument("--out", required=True, type=Path)
    p.add_argument("--motion-seconds", type=float, default=30.0, help="measure motion over the first seconds (30 is what the detector samples)")
    p.add_argument("--jobs", type=int, default=4)
    p.set_defaults(run=cmd_probe)

    for name, run, text in (("balance", cmd_balance, "per-class balance table"), ("match", cmd_match, "write a class-matched subset of a manifest")):
        p = sub.add_parser(name, help=text)
        if name == "match":
            p.add_argument("manifest", type=Path)
        p.add_argument("probe", type=Path)
        if name == "match":
            p.add_argument("out", type=Path)
            p.add_argument("--features", type=lambda s: tuple(s.split(",")), default=MATCH_FEATURES, help="numeric features to match on: " + ", ".join(NUMERIC_FEATURES))
            p.add_argument("--ratio", type=int, default=1, help="generated clips kept per authentic clip")
            p.add_argument("--caliper", type=float, default=1.0, help="largest distance of a match, in standard deviations")
        else:
            p.add_argument("--by", default="transform", help="probe column that names the cohorts")
        p.add_argument("--flag-smd", type=float, default=0.25)
        p.add_argument("--flag-tv", type=float, default=0.1)
        p.set_defaults(run=run)

    ladder = lambda s: tuple(tuple(int(v) for v in step.split(":")) for step in s.split(","))
    p = sub.add_parser("variants", help="originals, trims, a fixed-GOP re-encode and simulated reposts")
    p.add_argument("manifest", type=Path, help="one row per original clip")
    p.add_argument("output", type=Path)
    p.add_argument("--variants", default="original,fixed-gop,repost-g1,repost-g2,repost-g3", help="also: trim-copy, davodi-norm")
    p.add_argument("--repost-from", default="original", choices=("original", "fixed-gop"),
                   help="what generation 1 re-encodes: the original (keeps its cadence) or the fixed-GOP clip (transforms named fixed-gop+repost-gK)")
    p.add_argument("--ladder", type=ladder, default=REPOST_LADDER, help="repost steps as max-width:crf, e.g. 720:28,540:30,360:32")
    p.add_argument("--seconds", type=int, default=NORMALIZE_SECONDS, help="leading seconds every re-encode keeps")
    p.add_argument("--copy", action="store_true", help="copy the originals instead of linking them")
    p.add_argument("--jobs", type=int, default=4)
    p.set_defaults(run=cmd_variants)

    p = sub.add_parser("watermark", help="sham-mark, removal and placebo variants")
    p.add_argument("manifest", type=Path, help="one row per original clip")
    p.add_argument("output", type=Path)
    p.add_argument("--positions", default="br", help="corners for the sham mark: " + ", ".join(POSITIONS))
    p.add_argument("--region", action="append", default=[], help="name=x,y,w,h (fractions): also crop and fill this region, where some clips carry a real mark")
    p.add_argument("--fill", default="delogo", choices=("delogo", "blur"))
    p.add_argument("--opacity", type=float, default=0.6)
    p.add_argument("--logo-width", type=float, default=0.14, help="mark width as a fraction of the frame width")
    p.add_argument("--margin", type=float, default=0.03, help="distance from the edges as a fraction of the frame width")
    p.add_argument("--seconds", type=int, default=NORMALIZE_SECONDS)
    p.add_argument("--jobs", type=int, default=4)
    p.set_defaults(run=cmd_watermark)

    p = sub.add_parser("metadata-probe", help="predict the label from metadata alone")
    p.add_argument("probe", type=Path)
    p.add_argument("--by", default="transform", help="probe column that names the cohorts")
    p.add_argument("--group", default="source,scene,parent", help="groupings to hold out, first usable wins")
    p.add_argument("--folds", type=int, default=5)
    p.add_argument("--rounds", type=int, default=1000, help="bootstrap rounds")
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--with-motion", action="store_true", help="add the motion statistic (content, not metadata)")
    p.set_defaults(run=cmd_metadata_probe)

    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if not args.command:
        parser.error("give a subcommand or --self-test")
    args.run(args)


if __name__ == "__main__":
    main()
