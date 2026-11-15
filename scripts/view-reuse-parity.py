#!/usr/bin/env python3
"""Compare the `detect --json` output of two tintype binaries, byte for byte.

Standard library only; run it as `python3 -I scripts/view-reuse-parity.py ...`
from the repository root (the binaries find `models/` there).

  --make-edge-cases DIR   write small synthetic PNGs whose views coincide
                          (below the crop size, exactly the crop size, few
                          window positions) and print their paths
  --base BIN --new BIN --list FILES.txt
                          run both over the files for every device and mode
                          asked for and compare the rows

Two schemes, because the batch a view is scored in depends on what else is
being scored:

  --each      one process per file (the batches are the same every time)
  --together  one process for all files; the base binary is run twice as
              well, to show how much it differs from itself

A row is "identical" when the two JSON lines are the same bytes. For rows
that differ the report gives the largest logit and score difference and
counts changed verdicts and changed numbers of views or frames looked at.
`--require-identical` makes any differing row in an `--each` comparison an
error. See docs/runbooks/view-reuse.md.
"""
import argparse
import concurrent.futures
import json
import os
import struct
import subprocess
import sys
import zlib

MODES = {"default": [], "all-views": ["--all-views"], "fast": ["--fast"], "strict": ["--strict"], "sensitive": ["--sensitive"]}

# (width, height): below the 224-pixel crop; exactly at it; few window
# positions; a 256-pixel short side (where the whole-image view and the
# centre crop coincide); and two ordinary sizes as controls.
EDGE_SIZES = [
    (64, 64), (120, 90), (200, 300), (223, 223), (100, 400), (223, 500),
    (224, 224), (224, 225), (225, 225), (224, 300), (230, 230), (448, 224), (335, 224), (336, 336),
    (256, 256), (320, 256), (256, 400), (300, 300), (448, 448),
    (640, 480), (900, 700),
]


def png(width, height, seed):
    """A textured RGB picture from integer arithmetic: gradients, grain whose strength varies across the picture, a disc."""
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for x in range(width):
            h = (x * 0x9E3779B1 ^ y * 0x85EBCA77 ^ seed * 0xC2B2AE3D) & 0xFFFFFFFF
            h ^= h >> 15
            h = (h * 0x2C1B3C6D) & 0xFFFFFFFF
            h ^= h >> 12
            strength = 4 + (x * 28 // max(width - 1, 1) if seed % 2 else y * 28 // max(height - 1, 1))
            grain = (h % (2 * strength + 1)) - strength
            disc = (x - width // 3) ** 2 + (y - height // 2) ** 2 < (min(width, height) // 4) ** 2
            base = (x * 255 // max(width - 1, 1), y * 255 // max(height - 1, 1), ((x + y) * 255 // max(width + height - 2, 1) + 40 * seed) % 256)
            for c in range(3):
                v = base[c] + grain
                if disc:
                    v = 255 - v // 2
                rows.append(min(255, max(0, v)))

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(bytes(rows), 6)) + chunk(b"IEND", b"")


def make_edge_cases(directory):
    os.makedirs(directory, exist_ok=True)
    for (w, h) in EDGE_SIZES:
        for seed in (1, 2):
            path = os.path.join(directory, f"edge-{w}x{h}-{seed}.png")
            with open(path, "wb") as f:
                f.write(png(w, h, seed))
            print(os.path.abspath(path))


def detect(binary, files, device, flags, env):
    p = subprocess.run([binary, "detect", "--device", device, "--json", *flags, *files], stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
    if p.returncode not in (0, 2):
        sys.exit(f"{binary} failed ({p.returncode}) on {files[:1]}: {p.stderr.decode(errors='replace')[-2000:]}")
    return p.stdout.decode().splitlines()


def lines(binary, files, device, flags, env, each, jobs):
    if not each:
        return detect(binary, files, device, flags, env)
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        out = list(pool.map(lambda f: detect(binary, [f], device, flags, env), files))
    return [l for one in out for l in one]


def compare(a, b):
    if len(a) != len(b):
        sys.exit(f"row counts differ: {len(a)} and {len(b)}")
    d = {"rows": len(a), "identical": 0, "differing": 0, "max_abs_logit_diff": 0.0, "max_abs_score_diff": 0.0, "verdict_changes": 0, "looked_at_changes": 0, "scored_on_one_side_only": 0}
    for x, y in zip(a, b):
        if x == y:
            d["identical"] += 1
            continue
        d["differing"] += 1
        x, y = json.loads(x), json.loads(y)
        if x.get("path") != y.get("path"):
            sys.exit(f"rows are for different files: {x.get('path')} and {y.get('path')}")
        if ("score" in x) != ("score" in y):
            d["scored_on_one_side_only"] += 1
            continue
        if "score" not in x:
            continue
        d["verdict_changes"] += x["verdict"] != y["verdict"]
        d["looked_at_changes"] += (x.get("views"), x.get("frames")) != (y.get("views"), y.get("frames"))
        d["max_abs_logit_diff"] = max(d["max_abs_logit_diff"], abs(x["logit"] - y["logit"]))
        d["max_abs_score_diff"] = max(d["max_abs_score_diff"], abs(x["score"] - y["score"]))
    return d


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--make-edge-cases", metavar="DIR")
    ap.add_argument("--base", help="the reference binary")
    ap.add_argument("--new", help="the binary under test")
    ap.add_argument("--list", help="text file with one media path per line")
    ap.add_argument("--devices", default="cpu", help="comma-separated: cpu, gpu, gpu-f32")
    ap.add_argument("--modes", default="default,all-views,fast,strict,sensitive", help="comma-separated; use default,all-views for clips")
    ap.add_argument("--new-env", action="append", default=[], help="NAME=VALUE for the binary under test only (repeatable)")
    ap.add_argument("--jobs", type=int, default=2, help="processes at a time with --each")
    ap.add_argument("--require-identical", action="store_true")
    scheme = ap.add_mutually_exclusive_group()
    scheme.add_argument("--each", action="store_true")
    scheme.add_argument("--together", action="store_true")
    a = ap.parse_args()
    if a.make_edge_cases:
        return make_edge_cases(a.make_edge_cases)
    if not (a.base and a.new and a.list and (a.each or a.together)):
        ap.error("give --base, --new, --list and one of --each or --together (or --make-edge-cases DIR)")

    files = [l for l in open(a.list).read().splitlines() if l]
    env = {k: v for k, v in os.environ.items() if k not in ("TINTYPE_STATS", "TINTYPE_VIEW_REUSE")}
    new_env = dict(env)
    for kv in a.new_env:
        name, _, value = kv.partition("=")
        new_env[name] = value
    failed = False
    for device in a.devices.split(","):
        for mode in a.modes.split(","):
            flags = MODES[mode]
            base = lines(a.base, files, device, flags, env, a.each, a.jobs)
            new = lines(a.new, files, device, flags, new_env, a.each, a.jobs)
            report = {"device": device, "mode": mode, "scheme": "each" if a.each else "together", "new_env": a.new_env, "new_vs_base": compare(base, new)}
            if a.together:
                report["base_vs_base"] = compare(base, lines(a.base, files, device, flags, env, False, a.jobs))
            print(json.dumps(report), flush=True)
            failed |= a.each and a.require_identical and report["new_vs_base"]["differing"] > 0
    if failed:
        sys.exit("some rows differ")


if __name__ == "__main__":
    main()
