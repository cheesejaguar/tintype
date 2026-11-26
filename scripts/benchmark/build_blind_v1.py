#!/usr/bin/env python3
"""Build the TPB v1 blind manifest under docs/benchmark/v1/BLIND-PROTOCOL.md.

This is the record of how the blind split was chosen. It reads the
research data share for the earlier manifests it must avoid, so it runs
only where that share is mounted. Anyone can rebuild the resulting
manifest with `tpb.py rebuild`.

    python3 build_blind_v1.py --open docs/benchmark/v1/open.tsv.gz --openfake docs/benchmark/v1/openfake.tsv.gz \
        --out docs/benchmark/v1/blind.tsv.gz --report blind-build.txt

Pass 1 only lists, selects, downloads and hashes. Nothing is written but
the manifest and the report, and no detector runs.
"""
import argparse
import collections
import glob
import gzip
import hashlib
import json
import os
import re
import sys
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tpb  # noqa: E402

SHARE = "/Volumes/veracity-data"
AIGIB = ("HorizonTEL/AIGIBench", "e44ec40efe5117a5ccdaa6ff0e89ed934d03d310")
CNND = ("sywang/CNNDetection", "f287cf3c3e92ca2e5451ce200bab8b6f5e0175b2")
SOFAKE = ("saberzl/So-Fake-OOD", "b4b34d16fda2020c2923a21d431d1ab3acbdbaff")
SOFAKE_SHARDS = ["00002", "00044"]
IMAGE = re.compile(r"\.(png|jpe?g|webp|tiff?|bmp|gif)$", re.I)
FOLDER = re.compile(r"(?:^|/)data/fresh\d*/(aigibench-[^/]+|classic-[^/]+)/(.+)$")


def used_members():
    """Member paths of earlier draws, keyed by the folder name the plans used."""
    used = collections.defaultdict(set)
    # batches 7, 8 and 9 only; fresh11 is a blind set and is never opened
    sources = [m for b in ("fresh8", "fresh9", "fresh10") for m in glob.glob(f"{SHARE}/data/{b}/*/manifest.tsv")]
    for m in sources:
        folder = Path(m).parent.name
        for line in open(m):
            used[folder].add(line.split("\t", 1)[0])
    for m in glob.glob(str(Path(__file__).resolve().parents[2] / "docs/manifests/*.frozen.tsv.gz")):
        for line in gzip.open(m, "rt"):
            if line.startswith("#"):
                continue
            p = line.rstrip("\n").split("\t")[5]
            g = FOLDER.search(p)
            if g:
                used[g.group(1)].add(g.group(2))
    return used


def known_hashes(open_rows, openfake_rows):
    h = {r["sha256"] for r in open_rows} | {r["sha256"] for r in openfake_rows}
    for m in glob.glob(str(Path(__file__).resolve().parents[2] / "docs/manifests/*.frozen.tsv.gz")):
        for line in gzip.open(m, "rt"):
            if not line.startswith("#"):
                h.add(line.split("\t", 1)[0])
    return h


def store_sizes(cache):
    """size -> [paths] over every store manifest on the share (for the exact-overlap check)."""
    if cache.exists():
        return {int(k): v for k, v in json.loads(cache.read_text()).items()}
    from concurrent.futures import ThreadPoolExecutor
    paths = []
    for m in glob.glob(f"{SHARE}/data/store/*/manifest.tsv"):
        root = Path(m).parent
        for line in open(m, errors="replace"):
            rel = line.split("\t", 1)[0]
            if rel and "\0" not in rel:
                paths.append(str(root / rel))

    def size(p):
        try:
            return os.stat(p).st_size
        except (OSError, ValueError):
            return None

    sizes = collections.defaultdict(list)
    with ThreadPoolExecutor(32) as ex:
        for p, s in zip(paths, ex.map(size, paths, chunksize=256)):
            if s is not None:
                sizes[s].append(p)
    cache.write_text(json.dumps(sizes))
    return sizes


def in_training(data, sizes, memo):
    for p in sizes.get(len(data), []):
        if p not in memo:
            try:
                memo[p] = tpb.sha256_file(p)
            except OSError:
                memo[p] = None
        if memo[p] == hashlib.sha256(data).hexdigest():
            return p
    return None


def pick(members, n):
    m = len(members)
    return [members[int((i + 0.5) * m / n)] for i in range(n)]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--open", required=True)
    ap.add_argument("--openfake", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--report", required=True)
    ap.add_argument("--size-cache", default=f"{SHARE}/data/tpb-store-sizes.json")
    a = ap.parse_args()

    open_rows, openfake_rows = tpb.read_manifest(a.open), tpb.read_manifest(a.openfake)
    want = collections.Counter((r["set"], r["label"]) for r in open_rows)
    gen_of = {}
    for r in open_rows:
        gen_of[(r["set"], r["label"])] = r["generator"]
    folder_of = {}  # canonical set -> (plans folder, container, member prefix)
    for r in open_rows:
        if r["source"] in (AIGIB[0], CNND[0]):
            folder = r["path"].split("/", 1)[0]
            folder_of[r["set"]] = (folder, r["container"], r["member"].split("/", 1)[0] + "/")
    used = used_members()
    hashes = known_hashes(open_rows, openfake_rows)
    sizes = store_sizes(Path(a.size_cache))
    memo, report, out = {}, [], []

    archives = collections.defaultdict(list)
    for st, (folder, container, prefix) in folder_of.items():
        archives[container].append((st, folder, prefix))
    # (crc, size) of every used member, across all archives
    zips, used_crc = {}, set()
    for container, sets in archives.items():
        src = AIGIB if container.startswith("test/") else CNND
        z = tpb.RemoteZip(src[0], src[1], container)
        zips[container] = (src, z, z.info)
        for st, folder, prefix in sets:
            for mem in used.get(folder, ()):
                i = zips[container][2].get(mem)
                if i:
                    used_crc.add((i.CRC, i.file_size))
    # checkpoint: finished (set, class) selections survive a crash
    ckpt = Path(str(a.out) + ".partial.jsonl")
    done = {}
    if ckpt.exists():
        for line in ckpt.read_text().splitlines():
            d = json.loads(line)
            done[(d["set"], d["label"])] = d
    for container, sets in archives.items():
        src, z, info = zips[container]
        for st, folder, prefix in sets:
            u = used.get(folder, set())
            for label, cls in (("0", "/0_real/"), ("1", "/1_fake/")):
                n = want.get((st, label), 0)
                if not n:
                    continue
                if (st, label) in done:
                    d = done[(st, label)]
                    out.extend(d["rows"])
                    hashes.update(r["sha256"] for r in d["rows"])
                    report.append(d["line"])
                    if d.get("dropped_set"):
                        out = [r for r in out if r["set"] != st]
                        break
                    continue
                cand = sorted(m for m, i in info.items()
                              if m.startswith(prefix) and cls in m and IMAGE.search(m)
                              and m not in u and (i.CRC, i.file_size) not in used_crc)
                line = f"{st} {cls.strip('/')}: {len(cand)} unused, open split has {n}"
                if len(cand) < 100:
                    report.append(line + " -> set dropped (fewer than 100 unused)")
                    with ckpt.open("a") as f:
                        f.write(json.dumps({"set": st, "label": label, "rows": [], "line": report[-1], "dropped_set": True}) + "\n")
                    out = [r for r in out if r["set"] != st]
                    break
                chosen = pick(cand, min(n, len(cand)))
                drop_known = drop_train = 0
                start = len(out)
                for mem, data in z.read_many(chosen, workers=32):
                    h = hashlib.sha256(data).hexdigest()
                    if h in hashes:
                        drop_known += 1
                        continue
                    if in_training(data, sizes, memo):
                        drop_train += 1
                        continue
                    hashes.add(h)
                    out.append({"path": f"{st}/{mem}", "label": label, "set": st, "generator": gen_of[(st, label)],
                                "source": src[0], "revision": src[1], "container": container, "member": mem,
                                "sha256": h, "bytes": str(len(data))})
                report.append(line + f" -> {len(chosen)} chosen, {drop_known} dropped as earlier-evaluation duplicates, "
                              f"{drop_train} as training duplicates")
                with ckpt.open("a") as f:
                    f.write(json.dumps({"set": st, "label": label, "rows": out[start:], "line": report[-1]}) + "\n")
                print(report[-1], flush=True)
    # So-Fake-OOD: two unread shards, every kept row
    for x in SOFAKE_SHARDS:
        if ("sofake-ood", x) in done:
            d = done[("sofake-ood", x)]
            out.extend(d["rows"])
            hashes.update(r["sha256"] for r in d["rows"])
            report.append(d["line"])
            continue
        container = f"data/test_image-{x}-of-00046.parquet"
        start = len(out)
        # streamed through range requests: nothing is written to the share
        pf, i, kept, drop_known, drop_train, skipped = tpb.open_remote_parquet(SOFAKE[0], SOFAKE[1], container), 0, 0, 0, 0, 0
        for batch in pf.iter_batches(columns=["image", "label", "generator", "source_platform"], batch_size=64):
            for img, lab, gen, plat in zip(*(batch.column(c).to_pylist() for c in range(4))):
                lab = (lab or "").upper()
                data = (img or {}).get("bytes") if isinstance(img, dict) else img
                if lab not in ("FULL_SYNTHETIC", "REAL") or not data:
                    skipped += 1
                    i += 1
                    continue
                h = hashlib.sha256(data).hexdigest()
                if h in hashes:
                    drop_known += 1
                elif in_training(data, sizes, memo):
                    drop_train += 1
                else:
                    hashes.add(h)
                    ext = "png" if data[:8] == b"\x89PNG\r\n\x1a\n" else "jpg" if data[:2] == b"\xff\xd8" else \
                        "webp" if data[8:12] == b"WEBP" else "bin"
                    label = "1" if lab == "FULL_SYNTHETIC" else "0"
                    out.append({"path": f"sofake-ood/test_image-{x}-of-00046/{i:07d}.{ext}", "label": label,
                                "set": "sofake-ood", "generator": (gen or "unknown") if label == "1" else f"real:{plat or 'sofake-ood'}",
                                "source": SOFAKE[0], "revision": SOFAKE[1], "container": container, "member": str(i),
                                "sha256": h, "bytes": str(len(data))})
                    kept += 1
                i += 1
        report.append(f"sofake-ood shard {x}: {i} rows, {kept} kept, {skipped} not REAL/FULL_SYNTHETIC, "
                      f"{drop_known} earlier-evaluation duplicates, {drop_train} training duplicates")
        with ckpt.open("a") as f:
            f.write(json.dumps({"set": "sofake-ood", "label": x, "rows": out[start:], "line": report[-1]}) + "\n")
        print(report[-1], flush=True)
    tpb.write_manifest(a.out, out)
    c = collections.Counter((r["set"], r["label"]) for r in out)
    report.append(f"blind manifest: {len(out)} files, {sum(v for (s, l), v in c.items() if l == '1')} generated, "
                  f"{sum(v for (s, l), v in c.items() if l == '0')} real, {len({s for s, _ in c})} sets")
    Path(a.report).write_text("\n".join(report) + "\n")
    print(report[-1])


if __name__ == "__main__":
    main()
