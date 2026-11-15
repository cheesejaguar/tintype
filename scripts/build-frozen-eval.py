#!/usr/bin/env python3
"""Fetch and prepare the sources of the frozen independent split.

The selection is the one fixed in docs/PREREGISTRATION.md ("The frozen
independent split"); docs/runbooks/frozen-eval.md gives the order of the
steps. Every step writes under the directory it is given and none touches
`models/`.

    list-hdrplus OUT.list             the 3,640 `final.jpg` of the HDR+ results, with size and MD5
    list-floreview OUT.list           FloreView's natural-scene JPEGs
    fetch LIST OUT_DIR [JOBS]         download a list (rows: url, path, size, MD5 in base64)
    fetch-ranges URL DEST SIZE [JOBS] one large file, by parallel range requests
    extract KIND RAW_DIR OUT_DIR      KIND: arforensics, csafe, genimagepp (needs `zstd`)
    manifests FROZEN_SRC_DIR          write <store>/manifest.tsv with the grouping columns
    stress-sample SET.tsv OUT.tsv N   N authentic and N generated parents of a set, for processed copies
    sets-from-record RECORD OUT.tsv   write a set's manifest from its frozen record (paths under the current directory)
    self-test

Standard library only; run with `python3 -I`. Downloaded archives are
untrusted: members are written by name under the output directory and a
member whose path would leave it is refused.
"""

import base64
import collections
import hashlib
import json
import os
import re
import subprocess
import sys
import tarfile
import threading
import time
import urllib.parse
import urllib.request
import zipfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

IMAGE = {".jpg", ".jpeg", ".png", ".webp"}
HDRPLUS = "https://storage.googleapis.com/storage/v1/b/hdrplusdata/o?"
FLOREVIEW = "https://lesc.dinfo.unifi.it/FloreView/FloreView_Dataset.txt"
AR_FAMILY = {"LlamaGen": ("LlamaGen", "llamagen"), "VAR": ("VAR", "var"), "Infinity": ("Infinity", "infinity"),
             "Janus_Pro": ("Janus-Pro", "janus-pro"), "RAR": ("RAR", "rar"), "Switti": ("Switti", "switti"),
             "Open_MAGVIT2": ("Open-MAGVIT2", "open-magvit2")}
GENIMAGEPP = {"flux_realistic": ("FLUX.1-dev", "flux"), "stable_diffusion_v_3_0_realistic": ("SD3", "sd3"),
              "flux_krea_amateur": ("FLUX.1-Krea", "flux")}


def evenly(names, k):
    """`k` of the sorted names, evenly spaced; all of them if there are fewer."""
    names = sorted(names)
    if len(names) <= k:
        return names
    return [names[i * len(names) // k] for i in range(k)]


def allocate(sizes, k):
    """Split `k` over groups in proportion to their sizes (largest remainder,
    ties by name), never more than a group holds."""
    total = sum(sizes.values())
    if total <= k:
        return dict(sizes)
    exact = {g: k * n / total for g, n in sizes.items()}
    share = {g: int(x) for g, x in exact.items()}
    for g in sorted(sizes, key=lambda g: (-(exact[g] - share[g]), g))[:k - sum(share.values())]:
        share[g] += 1
    return share


def floreview_scene(name):
    """`D01_L1S1C1.jpg` -> `L1S1`: the location and scene every phone photographed."""
    m = re.fullmatch(r"D\d+_(L\d+S\d+)C\d+(?:_[a-z])?\.jpe?g", name, re.I)
    if not m:
        raise ValueError(f"unexpected FloreView name {name!r}")
    return m[1]


def list_hdrplus(out):
    items, token = [], None
    while True:
        query = {"prefix": "20171106/results_20171023/", "fields": "items(name,size,md5Hash),nextPageToken", "maxResults": "1000"}
        if token:
            query["pageToken"] = token
        page = json.load(urllib.request.urlopen(HDRPLUS + urllib.parse.urlencode(query), timeout=60))
        items += [i for i in page.get("items", []) if i["name"].endswith("/final.jpg")]
        token = page.get("nextPageToken")
        if not token:
            break
    with open(out, "w") as f:
        for i in sorted(items, key=lambda i: i["name"]):
            f.write(f"https://storage.googleapis.com/hdrplusdata/{urllib.parse.quote(i['name'])}\t{i['name'].split('/')[2]}.jpg\t{i['size']}\t{i['md5Hash']}\n")
    print(len(items), "files,", sum(int(i["size"]) for i in items), "bytes")


def list_floreview(out):
    n = 0
    with open(out, "w") as f:
        for url in sorted(urllib.request.urlopen(FLOREVIEW, timeout=60).read().decode().split()):
            parts = url.split("/FloreView/Dataset/")[1].split("/")
            if parts[1] == "Nat" and parts[2] == "jpeg-h264" and parts[-1].lower().endswith((".jpg", ".jpeg")):
                f.write(f"{url}\t{parts[0]}/{parts[-1]}\n")
                n += 1
    print(n, "files")


def fetch(listing, out, jobs):
    rows = [line.rstrip("\n").split("\t") for line in open(listing) if line.strip()]

    def get(row):
        size = int(row[2]) if len(row) > 2 and row[2] else None
        md5 = row[3] if len(row) > 3 and row[3] else None
        dest = out / row[1]
        if dest.exists() and (size is None or dest.stat().st_size == size):
            return None
        dest.parent.mkdir(parents=True, exist_ok=True)
        for attempt in range(6):
            try:
                request = urllib.request.Request(row[0], headers={"User-Agent": "curl/8"})
                data = urllib.request.urlopen(request, timeout=120).read()
                if size is not None and len(data) != size:
                    raise ValueError(f"size {len(data)} != {size}")
                if md5 and base64.b64encode(hashlib.md5(data).digest()).decode() != md5:
                    raise ValueError("md5 mismatch")
                part = dest.with_name(dest.name + ".part")
                part.write_bytes(data)
                part.rename(dest)
                return None
            except Exception as error:  # noqa: BLE001
                last = error
                time.sleep(2 * (attempt + 1))
        return f"{row[1]}\t{last}"

    failed = 0
    with ThreadPoolExecutor(jobs) as pool:
        for n, result in enumerate(pool.map(get, rows), 1):
            if result:
                failed += 1
                print("FAILED", result, flush=True)
            if n % 500 == 0:
                print(f"{n}/{len(rows)}", flush=True)
    print(f"done: {len(rows)} listed, {failed} failed", flush=True)
    return failed


def fetch_ranges(url, dest, size, jobs, chunk=64 << 20):
    done_path = dest + ".done"
    done = {int(line) for line in open(done_path)} if os.path.exists(done_path) else set()
    if not os.path.exists(dest):
        with open(dest, "wb") as f:
            f.truncate(size)
    lock = threading.Lock()

    def get(i):
        start, end = i * chunk, min(size, (i + 1) * chunk) - 1
        for attempt in range(12):
            try:
                request = urllib.request.Request(url, headers={"Range": f"bytes={start}-{end}", "User-Agent": "curl/8"})
                with urllib.request.urlopen(request, timeout=120) as response, open(dest, "r+b") as f:
                    f.seek(start)
                    got = 0
                    while block := response.read(1 << 20):
                        f.write(block)
                        got += len(block)
                if got != end - start + 1:
                    raise ValueError(f"short chunk {got}")
                with lock, open(done_path, "a") as record:
                    record.write(f"{i}\n")
                return None
            except Exception as error:  # noqa: BLE001
                last = error
                time.sleep(3 * (attempt + 1))
        return f"chunk {i}: {last}"

    todo = [i for i in range((size + chunk - 1) // chunk) if i not in done]
    with ThreadPoolExecutor(jobs) as pool:
        failed = [r for r in pool.map(get, todo) if r]
    print(f"done: {len(todo)} chunks, {len(failed)} failed", *failed, sep="\n", flush=True)
    return len(failed)


def safe(out, rel):
    dest = (out / rel).resolve()
    if out.resolve() not in dest.parents:
        sys.exit(f"refusing member path {rel!r}")
    dest.parent.mkdir(parents=True, exist_ok=True)
    return dest


def extract_arforensics(raw, out, per=1500):
    zips = [zipfile.ZipFile(p) for p in sorted(raw.glob("ARForensics_updated_test_part*.zip"))]
    if len(zips) != 4:
        sys.exit("expected the four ARForensics test archives")
    where, by_generator = {}, collections.defaultdict(list)
    for index, archive in enumerate(zips):
        for info in archive.infolist():
            parts = info.filename.split("/")
            if info.is_dir() or len(parts) != 5 or parts[1] != "test" or parts[3] != "1_fake":
                continue
            where[info.filename] = index
            by_generator[parts[2]].append(info.filename)
    for generator in sorted(by_generator):
        chosen = evenly(by_generator[generator], per)
        for name in chosen:
            safe(out, f"{generator}/{name.split('/')[-1]}").write_bytes(zips[where[name]].read(name))
        print(f"arforensics {generator}: {len(chosen)} of {len(by_generator[generator])}", flush=True)


def extract_csafe(raw, out):
    archive = zipfile.ZipFile(raw / "iPhone14.zip")
    kept = collections.Counter()
    for info in archive.infolist():
        parts = info.filename.split("/")
        if info.is_dir() or "natural" not in parts or Path(info.filename).suffix.lower() not in IMAGE or parts[-1].startswith("."):
            continue
        i = parts.index("natural")
        safe(out, f"{parts[i - 1]}/{parts[i + 1]}/{parts[-1]}").write_bytes(archive.read(info))
        kept[parts[i - 1]] += 1
    print("csafe:", sum(kept.values()), "natural images;", dict(sorted(kept.items())), flush=True)


def extract_genimagepp(raw, out, per=1000):
    for name in GENIMAGEPP:
        def stream():
            process = subprocess.Popen(["zstd", "-dc", str(raw / f"{name}.tar.zst")], stdout=subprocess.PIPE)
            return process, tarfile.open(fileobj=process.stdout, mode="r|")
        process, tar = stream()
        members = [m.name for m in tar if m.isfile() and Path(m.name).suffix.lower() in IMAGE and not Path(m.name).name.startswith(".")]
        process.wait()
        chosen = set(evenly(members, per))
        process, tar = stream()
        written = 0
        for member in tar:
            if member.name in chosen:
                safe(out, f"{name}/{Path(member.name).name}").write_bytes(tar.extractfile(member).read())
                written += 1
                if written == len(chosen):
                    break
        process.kill()
        print(f"genimagepp {name}: {written} of {len(members)}", flush=True)


def manifests(root):
    def files(store):
        return sorted(p.relative_to(store).as_posix() for p in store.rglob("*") if p.is_file() and p.suffix.lower() in IMAGE)

    def write(store, rows):
        (store / "manifest.tsv").write_text("".join("\t".join(r) + "\n" for r in rows))
        print(f"{store.name}: {len(rows)} rows")

    if (store := root / "hdrplus").is_dir():
        write(store, [(f, "0", "real:hdrplus", "hdrplus", f"hdrplus/{f}", "-", "hdrplus", "-", "-") for f in files(store)])
    if (store := root / "floreview").is_dir():
        write(store, [(f, "0", "real:floreview", "floreview", f"floreview/{f}", f"floreview-{floreview_scene(f.split('/')[1])}",
                       f"floreview-{f.split('_')[1].lower()}", "-", "-") for f in files(store)])
    if (store := root / "csafe-iphone14pro").is_dir():
        write(store, [(f, "0", "real:csafe", "csafe", f"csafe/{f}", "-", "csafe-iphone14pro", "-", "-") for f in files(store)])
    if (store := root / "arforensics").is_dir():
        write(store, [(f, "1", AR_FAMILY[f.split("/")[0]][0], "arforensics", f"arforensics/{f}", "-", "-", AR_FAMILY[f.split("/")[0]][1], "-")
                      for f in files(store)])
    if (store := root / "genimagepp").is_dir():
        write(store, [(f, "1", GENIMAGEPP[f.split("/")[0]][0], "genimagepp", f"genimagepp/{f}", "-", "-", GENIMAGEPP[f.split("/")[0]][1], "-")
                      for f in files(store)])


def stress_sample(rows, n):
    """`n` authentic and `n` generated rows: each class's `n` is divided over
    its sources (authentic) or families (generated) in proportion to their
    sizes, and taken evenly spaced over each group's sorted paths."""
    chosen = []
    for label, column in (("0", 6), ("1", 7)):
        groups = collections.defaultdict(list)
        for row in rows:
            if row[1] == label:
                groups[row[column]].append(row)
        share = allocate({g: len(members) for g, members in groups.items()}, n)
        for group in sorted(groups):
            by_path = {row[0]: row for row in groups[group]}
            chosen += [by_path[p] for p in evenly(by_path, share[group])]
    return chosen


def manifest_lines(record_lines, root):
    """Manifest rows from the rows of a frozen record (format 2): the path
    made absolute under `root`, then label, generator, dataset and the five
    grouping columns, an empty one written as `-`."""
    rows = []
    for line in record_lines:
        if line.startswith("#") or not line.strip():
            continue
        f = line.rstrip("\n").split("\t")
        groups = [g or "-" for g in (f[6:11] + [""] * 5)[:5]]
        rows.append("\t".join([f"{root}/{f[5]}", f[2], f[3], f[4], *groups]))
    return rows


def self_test():
    assert evenly(["c", "a", "b"], 5) == ["a", "b", "c"]
    assert evenly([f"{i:02}" for i in range(10)], 5) == ["00", "02", "04", "06", "08"]
    assert allocate({"a": 6, "b": 3, "c": 1}, 5) == {"a": 3, "b": 2, "c": 0} or allocate({"a": 6, "b": 3, "c": 1}, 5) == {"a": 3, "b": 1, "c": 1}
    assert sum(allocate({"a": 7, "b": 7, "c": 7}, 10).values()) == 10
    assert allocate({"a": 2, "b": 1}, 9) == {"a": 2, "b": 1}
    assert floreview_scene("D01_L1S1C1.jpg") == "L1S1" and floreview_scene("D13_L12S3C2_a.jpeg") == "L12S3"
    try:
        floreview_scene("D01_I001.jpg")
        raise AssertionError("a flat-field name must be refused")
    except ValueError:
        pass
    rows = [(f"r{i:03}", "0", "real:x", "x", f"r{i}", "-", "s1" if i < 30 else "s2", "-", "-") for i in range(40)]
    rows += [(f"g{i:03}", "1", "G", "x", f"g{i}", "-", "-", "f1" if i % 2 else "f2", "-") for i in range(40)]
    picked = stress_sample(rows, 8)
    assert len(picked) == 16 and sum(r[1] == "0" for r in picked) == 8
    assert collections.Counter(r[6] for r in picked if r[1] == "0") == {"s1": 6, "s2": 2}
    assert collections.Counter(r[7] for r in picked if r[1] == "1") == {"f1": 4, "f2": 4}
    assert stress_sample(rows, 8) == picked
    record = ["# tintype frozen evaluation set, format 2", "# manifest\tdata/frozen-eval/test.tsv\tabc",
              "aa\t10\t0\treal:x\tx\tdata/frozen-src/x/a.jpg\tx/a.jpg\t\ts1\t\t",
              "bb\t11\t1\tG\ty-jpeg-qv4\tdata/frozen-src/p/000000.jpg\ty/b.png\t\t\tf1\tjpeg-qv4"]
    assert manifest_lines(record, "/r") == ["/r/data/frozen-src/x/a.jpg\t0\treal:x\tx\tx/a.jpg\t-\ts1\t-\t-",
                                            "/r/data/frozen-src/p/000000.jpg\t1\tG\ty-jpeg-qv4\ty/b.png\t-\t-\tf1\tjpeg-qv4"]
    print("build-frozen-eval self-test: ok")


def main():
    args = sys.argv[1:]
    command = args[0] if args else ""
    if command == "self-test":
        self_test()
    elif command == "list-hdrplus" and len(args) == 2:
        list_hdrplus(args[1])
    elif command == "list-floreview" and len(args) == 2:
        list_floreview(args[1])
    elif command == "fetch" and len(args) in (3, 4):
        sys.exit(1 if fetch(args[1], Path(args[2]), int(args[3]) if len(args) == 4 else 8) else 0)
    elif command == "fetch-ranges" and len(args) in (4, 5):
        sys.exit(1 if fetch_ranges(args[1], args[2], int(args[3]), int(args[4]) if len(args) == 5 else 16) else 0)
    elif command == "extract" and len(args) == 4 and args[1] in ("arforensics", "csafe", "genimagepp"):
        Path(args[3]).mkdir(parents=True, exist_ok=True)
        {"arforensics": extract_arforensics, "csafe": extract_csafe, "genimagepp": extract_genimagepp}[args[1]](Path(args[2]), Path(args[3]))
    elif command == "manifests" and len(args) == 2:
        manifests(Path(args[1]))
    elif command == "stress-sample" and len(args) == 4:
        rows = [tuple(line.split("\t")) for line in Path(args[1]).read_text().splitlines()]
        originals = [r for r in rows if len(r) < 9 or r[8] in ("", "-")]
        picked = stress_sample(originals, int(args[3]))
        Path(args[2]).write_text("".join("\t".join(r) + "\n" for r in picked))
        print(f"{len(picked)} parents -> {args[2]}")
    elif command == "sets-from-record" and len(args) == 3:
        import gzip
        with gzip.open(args[1], "rt") as record:
            lines = manifest_lines(record, os.getcwd())
        Path(args[2]).write_text("".join(line + "\n" for line in lines))
        print(f"{len(lines)} rows -> {args[2]}")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
