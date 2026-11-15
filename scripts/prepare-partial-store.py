#!/usr/bin/env python3
"""Pick parents, an authentic pool and donors for a pasted-region store from one set of a split.

For the partial-edit evaluation (docs/runbooks/partial-edits.md). Reads a
manifest of one set (`calibrate.tsv` or `test.tsv` of the frozen split) and
writes, under --out:

  real-<k>.tsv   chunks of authentic originals for `tintype paste-edits
                 --real`: each chunk gives one batch of parents, so a batch
                 can be built, scored and deleted before the next
  pool.tsv       further authentic originals, used untouched: more
                 authentic images to set (or check) cutoffs on
  donors.tsv     generated originals large enough to donate a region

Only originals are taken (column 9 empty or `-`): a processed copy is not a
parent. Authentic images are shuffled with --seed and must have a shorter
side of at least --min-side and a longer side of at most --max-side;
donors must be at least --donor-min pixels on both sides, because a region
is pasted at native resolution and a donor smaller than the region cannot
supply it. The chunks, the pool and the composites built from them never
share an image. Sizes are read from the JPEG and PNG headers; standard
library only.
"""

import argparse
import collections
import random
import struct
import sys
from pathlib import Path


def image_size(path):
    """(width, height) from a PNG or JPEG header, or None."""
    try:
        with open(path, "rb") as f:
            d = f.read(1 << 23)
    except OSError:
        return None
    if d[:8] == b"\x89PNG\r\n\x1a\n":
        return struct.unpack(">II", d[16:24])
    if d[:2] != b"\xff\xd8":
        return None
    i = 2
    while i + 9 < len(d):
        if d[i] != 0xFF:
            i += 1
            continue
        marker = d[i + 1]
        if marker in (0xC0, 0xC1, 0xC2):
            h, w = struct.unpack(">HH", d[i + 5:i + 9])
            return w, h
        if marker == 0xD8 or 0xD0 <= marker <= 0xD7 or marker == 0xFF:
            i += 2 if marker != 0xFF else 1
            continue
        i += 2 + struct.unpack(">H", d[i + 2:i + 4])[0]
    return None


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--set", required=True, help="manifest of one set, absolute paths, nine columns")
    ap.add_argument("--out", required=True)
    ap.add_argument("--chunks", type=int, default=4)
    ap.add_argument("--chunk-rows", type=int, default=30, help="authentic originals per chunk (a few more than the parents wanted from it)")
    ap.add_argument("--pool", type=int, default=1000)
    ap.add_argument("--min-side", type=int, default=448)
    ap.add_argument("--max-side", type=int, default=4800)
    ap.add_argument("--donor-min", type=int, default=1024)
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args()
    rows = [l.split("\t") for l in Path(args.set).read_text().splitlines() if l]
    originals = [r for r in rows if len(r) < 9 or r[8] in ("", "-")]
    real = [r for r in originals if r[1] == "0"]
    fake = [r for r in originals if r[1] == "1"]
    random.Random(args.seed).shuffle(real)
    want = args.chunks * args.chunk_rows + args.pool
    kept, skipped = [], collections.Counter()
    for r in real:
        if len(kept) >= want:
            break
        size = image_size(r[0])
        if size is None:
            skipped["unreadable header"] += 1
        elif min(size) < args.min_side:
            skipped["too small"] += 1
        elif max(size) > args.max_side:
            skipped["too large"] += 1
        else:
            kept.append(r)
    if len(kept) < want:
        sys.exit(f"only {len(kept)} of the {len(real)} authentic originals qualify; {want} wanted")
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    for k in range(args.chunks):
        chunk = kept[k * args.chunk_rows:(k + 1) * args.chunk_rows]
        (out / f"real-{k}.tsv").write_text("".join("\t".join(r) + "\n" for r in chunk))
    pool = kept[args.chunks * args.chunk_rows:want]
    (out / "pool.tsv").write_text("".join("\t".join(r) + "\n" for r in pool))
    donors, sizes = [], collections.Counter()
    for r in fake:
        size = image_size(r[0])
        sizes[(r[2], size)] += 1
        if size is not None and min(size) >= args.donor_min:
            donors.append(r)
    (out / "donors.tsv").write_text("".join("\t".join(r) + "\n" for r in donors))
    print(f"{args.set}: {len(real)} authentic and {len(fake)} generated originals")
    print(f"authentic: {args.chunks} chunks of {args.chunk_rows} and a pool of {len(pool)}; passed over while filling them: {dict(skipped) or 'none'}")
    print(f"authentic sources in the chunks: {dict(collections.Counter(r[2] for r in kept[:args.chunks * args.chunk_rows]))}; in the pool: {dict(collections.Counter(r[2] for r in pool))}")
    print(f"donors: {len(donors)} of {len(fake)} generated originals are at least {args.donor_min} px on both sides")
    for (generator, size), n in sorted(sizes.items(), key=lambda kv: str(kv[0])):
        usable = size is not None and min(size) >= args.donor_min
        print(f"  {generator}\t{size[0]}x{size[1]}\t{n}\t{'donor' if usable else 'too small to donate'}" if size else f"  {generator}\tunreadable\t{n}")


if __name__ == "__main__":
    main()
