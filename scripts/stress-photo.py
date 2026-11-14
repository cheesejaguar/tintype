#!/usr/bin/env python3
"""Create paired stress inputs from a Tintype photo manifest.

Without options this writes what it always has: `original.tsv`, a
same-size JPEG re-encode (`jpeg.tsv`) and a JPEG capped at 960 px
(`jpeg-960.tsv`), the three inputs of the Frontier JPEG stress table in
docs/EVIDENCE.md, byte for byte.

With `--operations`, `--transform` or `--external` it writes a paired audit
suite instead: one manifest per operation and severity, in which

- column 5 (`parent_id`) is set on every original and every derivative, so
  `tintype split-groups` keeps a photo and all its copies in one set and
  `tintype report --cluster-by parent` resamples them together;
- column 9 (`transform`) names what was done to the file, for
  `tintype report --slice-by transform` and `scripts/report-stress.py`.

Transforms are named so that the name is the recipe:

    resize-<factor>-<kernel>     kernel: bilinear, bicubic, lanczos, area, neighbor
    jpeg-qv<N>                   FFmpeg MJPEG at -q:v N (2 best .. 31 worst), 4:2:0
    blur-s<sigma>                Gaussian blur, sigma in pixels
    noise-a<strength>            FFmpeg `noise` filter, strength 0..100, seeded per file
    sharpen-a<amount>            5x5 unsharp mask on all three channels
    repost-<cap>-qv<N>x<cycles>  cap both sides at <cap> px, JPEG at -q:v N, <cycles> times
    a+b+c                        a chain: the steps in order

`-q:v` is FFmpeg's quantiser scale, not the 1-100 quality of libjpeg; no
equivalence is claimed. `jpeg-qv4` and `repost-960-qv12x1` are the legacy
`jpeg` and `jpeg-960` variants under their recipe names (same bytes).
Operations that are not a JPEG encode are stored as PNG so that the
operation is the only change. Everything is rendered by FFmpeg, on purpose:
the audit suite must not be made by the code that makes the training
augmentation (`src/data.rs`), or it would only measure memorisation of that
code's quirks.

Real upload round trips cannot be scripted. `--upload-kit DIR` copies the
originals to `DIR/000000.jpg`, `DIR/000001.png`, ... in manifest order (with
`DIR/kit.tsv` listing them). Send those through the platform, save what
comes back under the same six-digit name (the extension may change), and
register the folder with `--external upload-<platform>=DIR`: the files are
listed in `upload-<platform>.tsv` with the parent ids of their originals,
exactly like a rendered transform. Files that did not come back are left
out and counted.

Needs FFmpeg on PATH; standard library otherwise.
"""

import argparse
import os
import re
import shutil
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# The helper module sits beside this script; find it under `python3 -I` too.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from benchmark_images import encode_jpeg, encode_png  # noqa: E402

CAP = "scale='min({cap},iw)':'min({cap},ih)':force_original_aspect_ratio=decrease"

# The severities of each operation family, mildest first.
FAMILIES = {
    "resize": ["resize-0.75-bicubic", "resize-0.5-bicubic", "resize-0.25-bicubic", "resize-0.5-bilinear",
               "resize-0.5-lanczos", "resize-0.5-area", "resize-2.0-bicubic"],
    "jpeg": ["jpeg-qv2", "jpeg-qv4", "jpeg-qv8", "jpeg-qv12", "jpeg-qv20"],
    "blur": ["blur-s0.5", "blur-s1.0", "blur-s1.5", "blur-s3.0"],
    "noise": ["noise-a5", "noise-a10", "noise-a20", "noise-a40"],
    "sharpen": ["sharpen-a0.5", "sharpen-a1.0", "sharpen-a2.0"],
    "repost": ["repost-1600-qv5x1", "repost-1280-qv7x1", "repost-960-qv7x1", "repost-960-qv12x1",
               "repost-720-qv9x1", "repost-960-qv7x2", "repost-960-qv7x4"],
}
KERNELS = {"bilinear", "bicubic", "lanczos", "area", "neighbor"}
NUMBER = r"\d+(?:\.\d+)?"


def step_passes(step, index):
    """One named step as FFmpeg passes: a list of (filter or None, JPEG -q:v or None)."""
    if m := re.fullmatch(rf"resize-({NUMBER})-([a-z]+)", step):
        factor, kernel = float(m[1]), m[2]
        if kernel in KERNELS and 0 < factor <= 8:
            side = "'max(1,trunc({d}*{f}+0.5))'"
            return [(f"format=gbrp,scale={side.format(d='iw', f=factor)}:{side.format(d='ih', f=factor)}:flags={kernel}", None)]
    elif m := re.fullmatch(r"jpeg-qv(\d+)", step):
        if 2 <= int(m[1]) <= 31:
            return [(None, int(m[1]))]
    elif m := re.fullmatch(rf"blur-s({NUMBER})", step):
        if float(m[1]) > 0:
            return [(f"format=gbrp,gblur=sigma={float(m[1])}", None)]
    elif m := re.fullmatch(rf"noise-a({NUMBER})", step):
        if 0 < float(m[1]) <= 100:
            # A seed per file: one seed for all would stamp the same noise field on every image.
            return [(f"format=gbrp,noise=alls={float(m[1])}:all_seed={1000 + index}", None)]
    elif m := re.fullmatch(rf"sharpen-a({NUMBER})", step):
        if 0 < float(m[1]) <= 5:
            return [(f"format=gbrp,unsharp=5:5:{float(m[1])}:5:5:{float(m[1])}", None)]
    elif m := re.fullmatch(r"repost-(\d+)-qv(\d+)x(\d+)", step):
        cap, quality, cycles = int(m[1]), int(m[2]), int(m[3])
        if cap >= 16 and 2 <= quality <= 31 and 1 <= cycles <= 20:
            return [(CAP.format(cap=cap), quality)] * cycles
    raise ValueError(f"not a transform this script can render: {step!r} (see --help for the grammar)")


def transform_passes(name, index):
    """A transform (steps joined by `+`) as the fewest FFmpeg runs: filters
    are merged into the encode that follows them."""
    runs, pending = [], []
    for step in name.split("+"):
        for filters, quality in step_passes(step, index):
            if filters:
                pending.append(filters)
            if quality is not None:
                runs.append((",".join(pending) or None, quality))
                pending = []
    if pending:
        runs.append((",".join(pending), None))
    return runs


def render(source, target_stem, name, index):
    """Run the passes of `name` on `source`; returns the final file (`.jpg` or `.png`)."""
    runs = transform_passes(name, index)
    current, scratch = source, []
    for k, (filters, quality) in enumerate(runs):
        last = k == len(runs) - 1
        suffix = ".jpg" if quality is not None else ".png"
        target = target_stem.with_name(target_stem.name + (suffix if last else f".pass{k}{suffix}"))
        if quality is not None:
            encode_jpeg(current, target, quality, filters)
        else:
            encode_png(current, target, filters)
        if not last:
            scratch.append(target)
        current = target
    for path in scratch:
        path.unlink()
    return current


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--jobs", type=int, default=4)
    parser.add_argument("--operations", default="",
                        help=f"comma-separated operation families to render at every severity: {', '.join(FAMILIES)}, or `all`")
    parser.add_argument("--transform", action="append", default=[], metavar="NAME",
                        help="one more transform by name, e.g. blur-s2.0+resize-0.5-bicubic+jpeg-qv6 (repeatable)")
    parser.add_argument("--external", action="append", default=[], metavar="upload-NAME=DIR",
                        help="register files that went through a real upload round trip (see above; repeatable)")
    parser.add_argument("--upload-kit", type=Path, metavar="DIR",
                        help="copy the originals to DIR under six-digit names, ready to upload")
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("--jobs must be positive")
    families = [f for f in args.operations.split(",") if f]
    if "all" in families:
        families = list(FAMILIES)
    if unknown := [f for f in families if f not in FAMILIES]:
        parser.error(f"unknown operation family {unknown[0]!r} (use {', '.join(FAMILIES)} or all)")
    transforms = list(dict.fromkeys([t for f in families for t in FAMILIES[f]] + args.transform))
    externals = {}
    for spec in args.external:
        name, _, folder = spec.partition("=")
        if not re.fullmatch(r"upload-[A-Za-z0-9._-]+", name) or not Path(folder).is_dir():
            parser.error(f"--external takes upload-NAME=DIR with an existing DIR, not {spec!r}")
        externals[name] = Path(folder).resolve()
    for name in transforms:
        try:
            transform_passes(name, 0)
        except ValueError as error:
            parser.error(str(error))
    # The paired audit suite; without any of these options, the three legacy JPEG inputs.
    grouped = bool(args.operations or args.transform or args.external or args.upload_kit)

    source = args.manifest.resolve()
    rows = []
    seen = set()
    for line in source.read_text(encoding="utf-8").splitlines():
        fields = line.split("\t")
        if len(fields) < 4:
            parser.error(f"expected path, label, generator and dataset in {source}: {line!r}")
        # Columns 5 and up (parent_id, scene_id, source_id, generator_family,
        # transform, ...) are carried through to every variant unchanged.
        path, label, generator, dataset, *extra = fields
        if path in seen or label not in {"0", "1"}:
            parser.error(f"duplicate path or invalid label in {source}: {line!r}")
        seen.add(path)
        if grouped:
            # An original is its own parent unless the manifest already names one.
            extra += [""] * (5 - len(extra))
            if extra[0] in {"", "-"}:
                extra[0] = f"{dataset}:{path}"
        rows.append((source.parent / path, label, generator, dataset, extra))
    if not rows:
        parser.error(f"empty manifest: {source}")
    if grouped and len({row[4][0] for row in rows}) != len(rows):
        parser.error(f"two rows of {source} share a parent_id: give this script the originals only")

    output = args.output.resolve()
    output.mkdir()
    if grouped:
        variants = {name: None for name in transforms}
    else:
        variants = {
            "jpeg": (4, None),
            "jpeg-960": (12, CAP.format(cap=960)),
        }
    for variant in variants:
        (output / variant).mkdir()

    def processed(extra, name):
        """The grouping columns of a derivative: the parent's, plus what was done."""
        earlier = extra[4] if extra[4] not in {"", "-"} else ""
        return [*extra[:4], f"{earlier}+{name}" if earlier else name, *extra[5:]]

    def trimmed(row):
        """Drop trailing empty columns, as `tintype` writes manifests."""
        row = list(row)
        while len(row) > 4 and row[-1] == "":
            row.pop()
        return row

    def encode(indexed_row):
        index, (path, label, generator, dataset, extra) = indexed_row
        rendered = {"original": (os.path.relpath(path, output), label, generator, f"{dataset}-original", *extra)}
        for variant, legacy in variants.items():
            if legacy:
                quality, filters = legacy
                target = Path(variant) / f"{index:06d}.jpg"
                encode_jpeg(path, output / target, quality, filters)
                rendered[variant] = (str(target), label, generator, f"{dataset}-{variant}", *extra)
            else:
                target = render(path, output / variant / f"{index:06d}", variant, index)
                rendered[variant] = (str(target.relative_to(output)), label, generator, f"{dataset}-{variant}", *processed(extra, variant))
        for name, folder in externals.items():
            found = sorted(p for p in folder.glob(f"{index:06d}.*") if p.is_file())
            if len(found) > 1:
                raise ValueError(f"more than one file for {index:06d} in {folder}")
            if found:
                rendered[name] = (os.path.relpath(found[0], output), label, generator, f"{dataset}-{name}", *processed(extra, name))
        return rendered

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        converted = list(pool.map(encode, enumerate(rows)))
    names = ["original", *variants, *externals]
    for variant in names:
        present = [row[variant] for row in converted if variant in row]
        if not present:
            parser.error(f"no file of {externals[variant]} is named like an original (000000.*, 000001.*, ...)")
        if len(present) < len(converted):
            print(f"{variant}: {len(converted) - len(present)} of {len(converted)} originals have no file and are left out", file=sys.stderr)
        (output / f"{variant}.tsv").write_text(
            "".join("\t".join(trimmed(row) if grouped else row) + "\n" for row in present),
            encoding="utf-8",
        )
    if grouped:
        # Everything in one manifest, originals first: one `tintype score` run, one `--groups` file.
        (output / "paired.tsv").write_text(
            "".join((output / f"{variant}.tsv").read_text(encoding="utf-8") for variant in names), encoding="utf-8")
    if args.upload_kit:
        kit = args.upload_kit.resolve()
        kit.mkdir(parents=True)
        lines = []
        for index, (path, _label, _generator, _dataset, extra) in enumerate(rows):
            name = f"{index:06d}{path.suffix.lower()}"
            shutil.copyfile(path, kit / name)
            lines.append(f"{name}\t{extra[0]}\n")
        (kit / "kit.tsv").write_text("".join(lines), encoding="utf-8")


if __name__ == "__main__":
    main()
