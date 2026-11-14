#!/usr/bin/env python3
"""Audit what the files of a manifest are, per class: container format (by
magic bytes), JPEG quality (estimated from the quantisation tables), chroma
subsampling, dimensions, EXIF and other metadata. No pixel is decoded.

docs/DATA.md records how many images each store holds, not what kind of
files they are. If one class is mostly JPEG and the other mostly PNG, or the
classes differ in size or JPEG quality, a detector can learn the file
history instead of the generator ("Fake or JPEG?", arXiv 2403.17608). This
script measures that; scripts/metadata-probe.py then asks how well the label
can be predicted from these columns alone.

  features   read the header of every file and write one row per file
  summary    tables from one or more feature files, per class and per
             store, source or generator family

Feature columns: store, path, label, generator, dataset, source, family,
ext (the extension in the manifest), format, width, height, bytes, jpeg_q (IJG quality whose luminance table is
closest), jpeg_std (1 if that table is exactly the IJG one), subsampling,
progressive, exif, icc, jfif, xmp, adobe, c2pa, comment, png_text (text
chunks before the image data), png_keys (their keywords), error.

Video files (by extension) are skipped and counted. A file that is no
longer on disk keeps its row, with `error` set and only the manifest's
columns filled: `summary --by-extension` then still shows how the classes
divide by file extension, which is all a manifest says about a deleted set.
Standard library only; runs under `python3 -I`. Pillow is not used.

  python3 -I scripts/audit-manifests.py features data/store/*/manifest.tsv \\
      --families docs/groups/generator-families.tsv --sources docs/groups/sources.tsv \\
      --out runs/ws4/features-train.tsv
  python3 -I scripts/audit-manifests.py summary runs/ws4/features-train.tsv --by store
  python3 -I scripts/audit-manifests.py --self-test
"""

import argparse
import csv
import struct
import sys
import tempfile
import zlib
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import manifest_groups as mg  # noqa: E402

COLUMNS = ["store", "path", "label", "generator", "dataset", "source", "family", "ext", "format", "width", "height", "bytes", "jpeg_q", "jpeg_std", "subsampling", "progressive", "exif", "icc", "jfif", "xmp", "adobe", "c2pa", "comment", "png_text", "png_keys", "error"]
VIDEO_EXTS = {".mp4", ".mov", ".mkv", ".webm", ".avi", ".m4v"}

# The IJG luminance quantisation table (Annex K of the JPEG standard), in
# natural order, and the zigzag order in which a file stores a table.
IJG_LUMA = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55,
    14, 13, 16, 24, 40, 57, 69, 56, 14, 17, 22, 29, 51, 87, 80, 62,
    18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113, 92,
    49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
]
ZIGZAG = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5,
    12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
]


def ijg_table(quality):
    """The luminance table libjpeg writes at a quality setting, in zigzag order."""
    scale = 5000 // quality if quality < 50 else 200 - 2 * quality
    natural = [min(255, max(1, (v * scale + 50) // 100)) for v in IJG_LUMA]
    return tuple(natural[k] for k in ZIGZAG)


IJG_TABLES = {q: ijg_table(q) for q in range(1, 101)}
_quality_cache = {}


def estimate_quality(table):
    """`(quality, exact)`: the IJG quality whose luminance table is closest to
    `table` (64 values in zigzag order) in summed absolute difference, and
    whether it is that table exactly. Encoders with their own tables (cameras,
    Photoshop, MozJPEG) get the nearest quality and exact = False."""
    table = tuple(table)
    hit = _quality_cache.get(table)
    if hit is None:
        best = min(range(1, 101), key=lambda q: (sum(abs(a - b) for a, b in zip(IJG_TABLES[q], table)), -q))
        hit = (best, IJG_TABLES[best] == table)
        _quality_cache[table] = hit
    return hit


def sniff(head):
    """Container format from the first bytes of a file."""
    if head[:3] == b"\xff\xd8\xff":
        return "jpeg"
    if head[:8] == b"\x89PNG\r\n\x1a\n":
        return "png"
    if head[:4] == b"RIFF" and head[8:12] == b"WEBP":
        return "webp"
    if head[:4] in (b"GIF8",):
        return "gif"
    if head[:2] == b"BM":
        return "bmp"
    if head[:4] in (b"II*\x00", b"MM\x00*"):
        return "tiff"
    if head[4:8] == b"ftyp":
        brand = head[8:12]
        if brand in (b"avif", b"avis"):
            return "avif"
        if brand in (b"heic", b"heix", b"hevc", b"mif1", b"msf1"):
            return "heif"
        return "mp4"
    if head[:2] == b"\xff\x0a" or head[:12] == b"\x00\x00\x00\x0cJXL \r\n\x87\n":
        return "jxl"
    return "other"


def parse_jpeg(data):
    """Header facts of a JPEG, read up to the start of scan. Returns the dict
    and whether the start of scan was reached (if not, `data` was too short)."""
    out = {"exif": 0, "icc": 0, "jfif": 0, "xmp": 0, "adobe": 0, "c2pa": 0, "comment": 0, "progressive": 0}
    tables = {}
    pos, n = 2, len(data)
    while pos + 4 <= n:
        if data[pos] != 0xFF:
            pos += 1
            continue
        marker = data[pos + 1]
        if marker == 0xFF:
            pos += 1
            continue
        if marker in (0x01, 0xD8) or 0xD0 <= marker <= 0xD7:
            pos += 2
            continue
        if marker in (0xDA, 0xD9):
            break
        length = struct.unpack(">H", data[pos + 2:pos + 4])[0]
        body = data[pos + 4:pos + 2 + length]
        if pos + 2 + length > n and marker in (0xDB, 0xC0, 0xC1, 0xC2, 0xC3):
            return out, False
        if marker == 0xDB:
            k = 0
            while k < len(body):
                precision, ident = body[k] >> 4, body[k] & 15
                size = 128 if precision else 64
                raw = body[k + 1:k + 1 + size]
                if len(raw) < size:
                    break
                tables[ident] = struct.unpack(">64H", raw) if precision else tuple(raw)
                k += 1 + size
        elif marker in (0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF) and len(body) >= 6:
            out["height"], out["width"] = struct.unpack(">HH", body[1:5])
            out["progressive"] = int(marker in (0xC2, 0xC6, 0xCA, 0xCE))
            comps = [(body[6 + 3 * c], body[7 + 3 * c] >> 4, body[7 + 3 * c] & 15, body[8 + 3 * c]) for c in range(body[5]) if 8 + 3 * c < len(body)]
            out["components"] = comps
        elif marker == 0xE0 and body[:5] == b"JFIF\x00":
            out["jfif"] = 1
        elif marker == 0xE1 and body[:6] == b"Exif\x00\x00":
            out["exif"] = 1
        elif marker == 0xE1 and body[:28] == b"http://ns.adobe.com/xap/1.0/":
            out["xmp"] = 1
        elif marker == 0xE2 and body[:12] == b"ICC_PROFILE\x00":
            out["icc"] = 1
        elif marker == 0xEE and body[:5] == b"Adobe":
            out["adobe"] = 1
        elif marker == 0xEB and body[:2] == b"JP":
            out["c2pa"] = 1
        elif marker == 0xFE:
            out["comment"] = 1
        pos += 2 + length
    else:
        return out, False
    comps = out.pop("components", [])
    if len(comps) == 1:
        out["subsampling"] = "gray"
    elif len(comps) == 3:
        (_, h0, v0, _), (_, h1, v1, _), (_, h2, v2, _) = comps
        if (h1, v1, h2, v2) != (1, 1, 1, 1):
            out["subsampling"] = "other"
        else:
            out["subsampling"] = {(2, 2): "420", (2, 1): "422", (1, 1): "444", (1, 2): "440", (4, 1): "411"}.get((h0, v0), "other")
    elif comps:
        out["subsampling"] = "other"
    luma_id = comps[0][3] if comps else 0
    table = tables.get(luma_id) or tables.get(0)
    if table:
        out["jpeg_q"], exact = estimate_quality(table)
        out["jpeg_std"] = int(exact)
    return out, "width" in out


def parse_png(data):
    """Header facts of a PNG, from the chunks before the image data."""
    out = {"exif": 0, "icc": 0, "c2pa": 0, "png_text": 0}
    keys = []
    pos, n = 8, len(data)
    while pos + 8 <= n:
        length, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + length]
        if kind == b"IHDR" and len(body) >= 8:
            out["width"], out["height"] = struct.unpack(">II", body[:8])
        elif kind in (b"tEXt", b"iTXt", b"zTXt"):
            out["png_text"] = 1
            keys.append(body.split(b"\x00", 1)[0][:40].decode("latin-1"))
        elif kind == b"eXIf":
            out["exif"] = 1
        elif kind == b"iCCP":
            out["icc"] = 1
        elif kind == b"caBX":
            out["c2pa"] = 1
        elif kind in (b"IDAT", b"IEND"):
            out["png_keys"] = ",".join(sorted(set(k.replace("\t", " ").replace("\n", " ").replace(",", ";") for k in keys)))
            return out, True
        pos += 12 + length
    return out, False


def parse_webp(data):
    out = {"exif": 0, "icc": 0, "xmp": 0}
    pos, n = 12, len(data)
    while pos + 8 <= n:
        kind, length = data[pos:pos + 4], struct.unpack("<I", data[pos + 4:pos + 8])[0]
        body = data[pos + 8:pos + 8 + min(length, 32)]
        if kind == b"VP8X" and len(body) >= 10:
            flags = body[0]
            out["icc"], out["exif"], out["xmp"] = int(bool(flags & 0x20)), int(bool(flags & 0x08)), int(bool(flags & 0x04))
            out["width"] = 1 + int.from_bytes(body[4:7], "little")
            out["height"] = 1 + int.from_bytes(body[7:10], "little")
        elif kind == b"VP8 " and len(body) >= 10 and body[3:6] == b"\x9d\x01\x2a":
            out.setdefault("width", struct.unpack("<H", body[6:8])[0] & 0x3FFF)
            out.setdefault("height", struct.unpack("<H", body[8:10])[0] & 0x3FFF)
            return out, True
        elif kind == b"VP8L" and len(body) >= 5 and body[0] == 0x2F:
            bits = int.from_bytes(body[1:5], "little")
            out.setdefault("width", 1 + (bits & 0x3FFF))
            out.setdefault("height", 1 + ((bits >> 14) & 0x3FFF))
            return out, True
        elif kind in (b"ANIM", b"ANMF"):
            return out, "width" in out
        pos += 8 + length + (length & 1)
    return out, "width" in out


def inspect(path, head_bytes=131072):
    """Feature dict of one file. Reads the head, and the whole file only when
    the header runs past it (large EXIF or ICC blocks)."""
    out = {}
    try:
        size = Path(path).stat().st_size
        with open(path, "rb") as stream:
            data = stream.read(head_bytes)
            out["bytes"] = size
            fmt = sniff(data)
            out["format"] = fmt
            parser = {"jpeg": parse_jpeg, "png": parse_png, "webp": parse_webp}.get(fmt)
            if parser:
                facts, complete = parser(data)
                if not complete and size > len(data):
                    data += stream.read()
                    facts, complete = parser(data)
                out.update(facts)
                if not complete:
                    out["error"] = "header incomplete"
            elif fmt == "gif" and len(data) >= 10:
                out["width"], out["height"] = struct.unpack("<HH", data[6:10])
            elif fmt == "bmp" and len(data) >= 26:
                out["width"], out["height"] = struct.unpack("<ii", data[18:26])
                out["height"] = abs(out["height"])
    except OSError as error:
        out["error"] = type(error).__name__
    return out


def cmd_features(args):
    families = mg.Rules.load(args.families) if args.families else None
    sources = mg.Rules.load(args.sources) if args.sources else None
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    written = skipped = errors = 0
    with out.open("w", encoding="utf-8", newline="") as stream, ThreadPoolExecutor(max_workers=args.threads) as pool:
        stream.write("\t".join(COLUMNS) + "\n")
        names = [mg.manifest_name(m) for m in args.manifest]
        for manifest, store in zip(args.manifest, names):
            if names.count(store) > 1:
                # Two manifests with one short name (the same set in two batches): keep the directory above too.
                store = "/".join(Path(manifest).parts[-3:-1]) if Path(manifest).name == "manifest.tsv" else str(manifest)
            rows = [r for r in mg.read_manifest(manifest)]
            images = [r for r in rows if Path(r.path).suffix.lower() not in VIDEO_EXTS]
            skipped += len(rows) - len(images)
            for row, facts in zip(images, pool.map(inspect, [r.file for r in images], chunksize=256)):
                source = row.group(2)
                if not source and sources is not None:
                    hit = sources.match(row.generator, row.dataset)
                    source = hit[0] if hit else None
                source = source or row.source()
                family = mg.family_of(row, families)[0] if row.label == 1 else ""
                record = {"store": store, "path": row.path, "label": row.label, "generator": row.generator, "dataset": row.dataset, "source": source, "family": family, "ext": Path(row.path).suffix.lower().lstrip(".")}
                record.update(facts)
                errors += "error" in facts
                stream.write("\t".join(str(record.get(c, "")) for c in COLUMNS) + "\n")
                written += 1
            print(f"{store}: {len(images)} files", file=sys.stderr)
    print(f"{out}: {written} files, {errors} with an unreadable or incomplete header, {skipped} video files skipped", file=sys.stderr)


def read_features(paths):
    rows = []
    for path in paths:
        with open(path, encoding="utf-8", newline="") as stream:
            for row in csv.DictReader(stream, delimiter="\t", quoting=csv.QUOTE_NONE):
                row["label"] = int(row["label"])
                for key in ("width", "height", "bytes", "jpeg_q"):
                    row[key] = int(row[key]) if row.get(key) else None
                rows.append(row)
    return rows


def quantile(sorted_values, q):
    """Nearest-rank quantile of sorted values."""
    return sorted_values[min(len(sorted_values) - 1, int(q * len(sorted_values)))]


def pct(part, whole):
    return "–" if not whole else f"{100.0 * part / whole:.1f}%"


def three(values):
    values = sorted(values)
    return "–" if not values else f"{quantile(values, 0.1)} / {quantile(values, 0.5)} / {quantile(values, 0.9)}"


def summary_tables(rows, by, min_items=1):
    """Two Markdown tables: files and JPEG settings, then size and metadata,
    one row per (group, class)."""
    groups = defaultdict(list)
    for r in rows:
        key = r[by] if by != "family" or r["label"] == 1 else r["source"]
        groups[(key, r["label"])].append(r)
    order = sorted(groups, key=lambda k: (k[0], k[1]))
    first = [f"| {by.capitalize()} | Class | Files | JPEG | PNG | WebP | Other | JPEG quality p10 / p50 / p90 | IJG tables | 4:2:0 | 4:4:4 | Progressive |", "|---|---|---|---|---|---|---|---|---|---|---|---|"]
    second = [f"| {by.capitalize()} | Class | Files | Short side p10 / p50 / p90 | Megapixels p50 | Bytes per pixel p50 | Both sides multiples of 64 | EXIF | ICC | PNG text chunks | Unreadable |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    for key in order:
        g = groups[key]
        if len(g) < min_items:
            continue
        n = len(g)
        read = sum(r["error"] != "FileNotFoundError" for r in g)
        fmt = Counter(r["format"] for r in g)
        jpeg = [r for r in g if r["format"] == "jpeg"]
        png = [r for r in g if r["format"] == "png"]
        sized = [r for r in g if r["width"] and r["height"]]
        other = read - fmt["jpeg"] - fmt["png"] - fmt["webp"]
        name = f"{key[0]} | {'AI' if key[1] else 'real'}"
        first.append(
            f"| {name} | {n} | {pct(fmt['jpeg'], read)} | {pct(fmt['png'], read)} | {pct(fmt['webp'], read)} | {pct(other, read)} | {three([r['jpeg_q'] for r in jpeg if r['jpeg_q']])} | "
            f"{pct(sum(r['jpeg_std'] == '1' for r in jpeg), len(jpeg))} | {pct(sum(r['subsampling'] == '420' for r in jpeg), len(jpeg))} | {pct(sum(r['subsampling'] == '444' for r in jpeg), len(jpeg))} | {pct(sum(r['progressive'] == '1' for r in jpeg), len(jpeg))} |"
        )
        mp = sorted(r["width"] * r["height"] / 1e6 for r in sized)
        bpp = sorted(r["bytes"] / (r["width"] * r["height"]) for r in sized if r["bytes"])
        second.append(
            f"| {name} | {n} | {three([min(r['width'], r['height']) for r in sized])} | {('%.2f' % quantile(mp, 0.5)) if mp else '–'} | {('%.2f' % quantile(bpp, 0.5)) if bpp else '–'} | "
            f"{pct(sum(r['width'] % 64 == 0 and r['height'] % 64 == 0 for r in sized), len(sized))} | {pct(sum(r['exif'] == '1' for r in g), read)} | {pct(sum(r['icc'] == '1' for r in g), read)} | {pct(sum(r['png_text'] == '1' for r in png), len(png))} | {sum(bool(r['error']) for r in g)} |"
        )
    return "\n".join(first), "\n".join(second)


def extension_table(rows, by):
    """Per (group, class): how many files are on disk and the share of each
    file extension in the manifest."""
    groups = defaultdict(list)
    for r in rows:
        groups[(r[by], r["label"])].append(r)
    exts = [e for e, _ in Counter(r["ext"] for r in rows).most_common()]
    lines = [f"| {by.capitalize()} | Class | Files | On disk | " + " | ".join("." + e for e in exts) + " |", "|---|---|---|---|" + "---|" * len(exts)]
    for key in sorted(groups):
        g = groups[key]
        count = Counter(r["ext"] for r in g)
        on_disk = sum(r["error"] != "FileNotFoundError" for r in g)
        lines.append(f"| {key[0]} | {'AI' if key[1] else 'real'} | {len(g)} | {on_disk} | " + " | ".join(pct(count[e], len(g)) for e in exts) + " |")
    return "\n".join(lines)


def cmd_summary(args):
    rows = read_features(args.features)
    real = sum(r["label"] == 0 for r in rows)
    if args.by_extension:
        print(f"{len(rows)} manifest rows: {real} real, {len(rows) - real} AI. Shares are of the rows of a line, by the file extension written in the manifest.\n")
        for by in args.by.split(","):
            print(extension_table(rows, by) + "\n")
        return
    missing = sum(r["error"] == "FileNotFoundError" for r in rows)
    if missing:
        print(f"{missing} of {len(rows)} files are not on disk; they count as files and as unreadable, and every share is of the files that were read.\n")
    print(f"{len(rows)} files: {real} real, {len(rows) - real} AI. Format, EXIF and ICC shares are of the files read; JPEG settings of the JPEG files of a row; PNG text chunks of its PNG files; a dash means none.\n")
    for by in args.by.split(","):
        first, second = summary_tables(rows, by, args.min_items)
        heading = {"store": "store", "source": "source", "family": "generator family (real rows: source)", "dataset": "dataset", "generator": "generator"}[by]
        print(f"### Files and JPEG settings by {heading}\n\n{first}\n\n### Size and metadata by {heading}\n\n{second}\n")


def make_png(width, height, text=None):
    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
    raw = b"".join(b"\x00" + bytes(3 * width) for _ in range(height))
    parts = [b"\x89PNG\r\n\x1a\n", chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))]
    if text:
        parts.append(chunk(b"tEXt", text[0].encode() + b"\x00" + text[1].encode()))
    return b"".join(parts + [chunk(b"IDAT", zlib.compress(raw)), chunk(b"IEND", b"")])


def make_jpeg_header(width, height, quality, sampling=(2, 2), exif=False, progressive=False, pad=0):
    """The segments of a JPEG up to the start of scan (enough for the parser)."""
    def segment(marker, body):
        return b"\xff" + bytes([marker]) + struct.pack(">H", len(body) + 2) + body
    out = b"\xff\xd8" + segment(0xE0, b"JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00")
    if exif:
        out += segment(0xE1, b"Exif\x00\x00" + bytes(pad))
    out += segment(0xDB, b"\x00" + bytes(IJG_TABLES[quality]))
    comps = bytes([1, (sampling[0] << 4) | sampling[1], 0, 2, 0x11, 1, 3, 0x11, 1])
    out += segment(0xC2 if progressive else 0xC0, b"\x08" + struct.pack(">HH", height, width) + b"\x03" + comps)
    return out + segment(0xDA, b"\x03\x01\x00\x02\x11\x03\x11\x00\x3f\x00") + b"\x00" * 16 + b"\xff\xd9"


def self_test():
    # Quality estimation: exact for IJG tables, nearest for others.
    for q in (10, 50, 75, 90, 95, 100):
        assert estimate_quality(IJG_TABLES[q]) == (q, True), q
    nudged = list(IJG_TABLES[80])
    nudged[5] += 1
    assert estimate_quality(nudged) == (80, False)
    assert IJG_TABLES[50][:4] == (16, 11, 12, 14) and IJG_TABLES[100] == (1,) * 64
    assert sniff(b"RIFF\x00\x00\x00\x00WEBPVP8 ") == "webp" and sniff(b"\x00\x00\x00\x18ftypisom") == "mp4" and sniff(b"GIF89a") == "gif" and sniff(b"hello") == "other"

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        store = tmp / "demo"
        store.mkdir()
        (store / "a.jpg").write_bytes(make_jpeg_header(640, 480, 75))
        (store / "b.jpg").write_bytes(make_jpeg_header(4032, 3024, 92, sampling=(1, 1), exif=True, progressive=True))
        # An EXIF block larger than the first read: the parser has to read on.
        (store / "big.jpg").write_bytes(make_jpeg_header(800, 600, 60, sampling=(2, 1), exif=True, pad=60000))
        (store / "c.png").write_bytes(make_png(1024, 1024, ("parameters", "a photo of a cat")))
        (store / "d.png").write_bytes(make_png(100, 60))
        vp8x = b"VP8X" + struct.pack("<I", 10) + bytes([0x08, 0, 0, 0]) + (799).to_bytes(3, "little") + (599).to_bytes(3, "little")
        (store / "e.webp").write_bytes(b"RIFF" + struct.pack("<I", 4 + len(vp8x)) + b"WEBP" + vp8x)
        (store / "f.jpg").write_bytes(b"\xff\xd8\xff\xe0\x00")
        (store / "g.mp4").write_bytes(b"\x00\x00\x00\x18ftypisom")
        (store / "manifest.tsv").write_text(
            "a.jpg\t0\treal:cam\tdemo\nb.jpg\t0\treal:cam\tdemo\nbig.jpg\t0\treal:phone\tdemo\nc.png\t1\tsdxl\tdemo\nd.png\t1\tsdxl\tdemo\n"
            "e.webp\t1\tother-gen\tdemo\nf.jpg\t0\treal:cam\tdemo\ng.mp4\t1\tvideo-gen\tdemo\nmissing.jpg\t0\treal:cam\tdemo\n"
        )
        (tmp / "families.tsv").write_text("sdxl*\t*\tlatent-diffusion\tname\t\n")
        assert inspect(store / "big.jpg", head_bytes=4096)["jpeg_q"] == 60
        args = parser().parse_args(["features", str(store / "manifest.tsv"), "--families", str(tmp / "families.tsv"), "--out", str(tmp / "features.tsv"), "--threads", "2"])
        cmd_features(args)
        rows = {r["path"]: r for r in read_features([tmp / "features.tsv"])}
        assert len(rows) == 8 and "g.mp4" not in rows
        a, b, big, c, d, e = (rows[k] for k in ("a.jpg", "b.jpg", "big.jpg", "c.png", "d.png", "e.webp"))
        assert (a["format"], a["width"], a["height"], a["jpeg_q"], a["jpeg_std"], a["subsampling"], a["progressive"], a["exif"], a["jfif"]) == ("jpeg", 640, 480, 75, "1", "420", "0", "0", "1"), a
        assert (b["width"], b["height"], b["jpeg_q"], b["subsampling"], b["progressive"], b["exif"]) == (4032, 3024, 92, "444", "1", "1"), b
        assert (big["width"], big["jpeg_q"], big["subsampling"], big["exif"], big["source"]) == (800, 60, "422", "1", "real:phone"), big
        assert (c["format"], c["width"], c["png_text"], c["png_keys"], c["family"], c["source"]) == ("png", 1024, "1", "parameters", "latent-diffusion", "demo"), c
        assert (d["png_text"], d["family"]) == ("0", "latent-diffusion") and (e["format"], e["width"], e["height"], e["exif"], e["family"]) == ("webp", 800, 600, "1", "unknown")
        assert rows["f.jpg"]["error"] == "header incomplete" and rows["missing.jpg"]["error"] == "FileNotFoundError"
        first, second = summary_tables(list(rows.values()), "store")
        assert "| demo | AI | 3 | 0.0% | 66.7% | 33.3% | 0.0% | – |" in first, first
# Five real rows: four JPEG (one of them cut short, so without tables) and one missing file.
        assert "| demo | real | 5 | 100.0% | 0.0% | 0.0% | 0.0% | 60 / 75 / 92 | 75.0% | 25.0% | 25.0% | 25.0% |" in first, first
        table = extension_table(list(rows.values()), "store")
        assert "| Store | Class | Files | On disk | .jpg | .png | .webp |" in table and "| demo | real | 5 | 4 | 100.0% | 0.0% | 0.0% |" in table and "| demo | AI | 3 | 3 | 0.0% | 66.7% | 33.3% |" in table, table
        assert "| demo | AI | 3 | 60 / 600 / 1024 |" in second and "| 50.0% | 0 |" in second, second
    print("audit-manifests self-test: ok")


def parser():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--self-test", action="store_true")
    sub = p.add_subparsers(dest="command")
    f = sub.add_parser("features", help="read file headers and write one row per file")
    f.add_argument("manifest", nargs="+")
    f.add_argument("--out", required=True)
    f.add_argument("--families", help="rules from generator names to families (docs/groups/generator-families.tsv)")
    f.add_argument("--sources", help="rules from generator keys to source ids (docs/groups/sources.tsv)")
    f.add_argument("--threads", type=int, default=4)
    s = sub.add_parser("summary", help="tables from feature files")
    s.add_argument("features", nargs="+")
    s.add_argument("--by", default="store", help="comma-separated: store, source, family, dataset, generator")
    s.add_argument("--min-items", type=int, default=1, help="leave out rows with fewer files")
    s.add_argument("--by-extension", action="store_true", help="only the manifest's file extensions per class (works for sets that are no longer on disk)")
    return p


def main():
    args = parser().parse_args()
    if args.self_test:
        return self_test()
    if args.command == "features":
        return cmd_features(args)
    if args.command == "summary":
        return cmd_summary(args)
    parser().print_help()


if __name__ == "__main__":
    main()
