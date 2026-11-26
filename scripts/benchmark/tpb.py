#!/usr/bin/env python3
"""Tintype Photo Benchmark (TPB): rebuild a split from its sources and score detectors.

  python3 scripts/benchmark/tpb.py rebuild docs/benchmark/v1/open.tsv.gz --root ~/tpb/v1/open
  python3 scripts/benchmark/tpb.py verify  docs/benchmark/v1/open.tsv.gz --root ~/tpb/v1/open
  python3 scripts/benchmark/tpb.py score   docs/benchmark/v1/open.tsv.gz \
      --system Tintype=tintype.jsonl --system 'Community Forensics=commfor.tsv' \
      --time Tintype=812.4 --out result.json
  python3 scripts/benchmark/tpb.py diff blind-result.json open-result.json --system Tintype

A manifest is a gzipped TSV with a header:
  path  label  set  generator  source  revision  container  member  sha256  bytes
`path` is where the file sits under --root. `source`, `revision` and
`container` name a Hugging Face dataset file. `member` is a zip member name
or, for a parquet container, a 0-based row index. Every rebuilt file must
match `sha256`.

Detector outputs for `score`:
- a TSV of `path<TAB>score`, or a JSON-lines file with `path` and `score`
  (the output of `tintype batch --json` works as is);
- each score is on a 0-1 scale where 0.5 is the detector's shipped
  cutoff, and a file is flagged when its score is above 0.5;
- paths may be absolute, and are matched to the manifest by their longest
  suffix.

The metrics are fixed for every version of the benchmark (docs/benchmark/README.md).
Rebuilding parquet-sourced files needs pyarrow. Everything else is the
standard library.
"""
import argparse
import csv
import gzip
import hashlib
import io
import json
import math
import os
import random
import sys
import time
import urllib.request
import zipfile
from pathlib import Path

HF = "https://huggingface.co/datasets/{source}/resolve/{revision}/{container}"
UA = "tintype-photo-benchmark/1"
SEED, DRAWS = 20261125, 2000
FIELDS = ["path", "label", "set", "generator", "source", "revision", "container", "member", "sha256", "bytes"]


# ---------------------------------------------------------------- manifests

def read_manifest(path):
    with gzip.open(path, "rt", newline="") as f:
        rows = list(csv.DictReader(f, delimiter="\t", quoting=csv.QUOTE_NONE))
    if not rows or list(rows[0].keys()) != FIELDS:
        sys.exit(f"{path}: expected columns {FIELDS}")
    return rows


def write_manifest(path, rows):
    with gzip.GzipFile(path, "wb", mtime=0) as raw:
        with io.TextIOWrapper(raw, newline="") as f:
            w = csv.DictWriter(f, FIELDS, delimiter="\t", quoting=csv.QUOTE_NONE, lineterminator="\n")
            w.writeheader()
            w.writerows(rows)


# ---------------------------------------------------------------- remote zips

class RangeFile(io.RawIOBase):
    """Seekable read-only view of a remote file through HTTP range requests."""

    BLOCK = 1 << 20
    KEEP = 64

    def __init__(self, url, block=None, keep=None):
        self.url, self.pos, self.cache = url, 0, {}
        self.BLOCK = block or self.BLOCK
        self.KEEP = keep or self.KEEP
        req = urllib.request.Request(url, method="HEAD", headers={"User-Agent": UA})
        with urllib.request.urlopen(req, timeout=60) as r:
            self.size = int(r.headers["Content-Length"])
            self.final = r.geturl()

    def readable(self):
        return True

    def seekable(self):
        return True

    def tell(self):
        return self.pos

    def seek(self, off, whence=0):
        self.pos = off if whence == 0 else self.pos + off if whence == 1 else self.size + off
        return self.pos

    def _block(self, i):
        if i not in self.cache:
            lo, hi = i * self.BLOCK, min(self.size, (i + 1) * self.BLOCK) - 1
            for attempt in range(5):
                try:
                    req = urllib.request.Request(self.final, headers={"User-Agent": UA, "Range": f"bytes={lo}-{hi}"})
                    with urllib.request.urlopen(req, timeout=120) as r:
                        data = r.read()
                    break
                except Exception:
                    if attempt == 4:
                        raise
                    time.sleep(2 ** attempt)
                    # signed CDN links expire; resolve again
                    req = urllib.request.Request(self.url, method="HEAD", headers={"User-Agent": UA})
                    with urllib.request.urlopen(req, timeout=60) as r:
                        self.final = r.geturl()
            if len(self.cache) > self.KEEP:
                self.cache.pop(next(iter(self.cache)))
            self.cache[i] = data
        return self.cache[i]

    def readinto(self, b):
        n = min(len(b), max(0, self.size - self.pos))
        got = 0
        while got < n:
            i, off = divmod(self.pos, self.BLOCK)
            chunk = self._block(i)[off:off + n - got]
            b[got:got + len(chunk)] = chunk
            got += len(chunk)
            self.pos += len(chunk)
        return got


def open_remote_parquet(source, revision, container):
    """Stream a remote parquet file through range requests; nothing is written to disk."""
    import pyarrow as pa
    import pyarrow.parquet as pq
    rf = RangeFile(HF.format(source=source, revision=revision, container=container), block=16 << 20, keep=6)
    return pq.ParquetFile(pa.PythonFile(rf, mode="r"))


def open_remote_zip(source, revision, container):
    return zipfile.ZipFile(io.BufferedReader(RangeFile(HF.format(source=source, revision=revision, container=container)), 1 << 20))


class RemoteZip:
    """A remote zip's directory, plus parallel fetches of members by exact byte range."""

    def __init__(self, source, revision, container):
        self.url = HF.format(source=source, revision=revision, container=container)
        rf = RangeFile(self.url)
        self.final = rf.final
        self.info = {i.filename: i for i in zipfile.ZipFile(io.BufferedReader(rf, 1 << 20)).infolist()}

    def _range(self, lo, hi):
        for attempt in range(6):
            try:
                req = urllib.request.Request(self.final, headers={"User-Agent": UA, "Range": f"bytes={lo}-{hi}"})
                with urllib.request.urlopen(req, timeout=120) as r:
                    return r.read()
            except Exception:
                if attempt == 5:
                    raise
                time.sleep(2 ** attempt)
                with urllib.request.urlopen(urllib.request.Request(self.url, method="HEAD", headers={"User-Agent": UA}), timeout=60) as r:
                    self.final = r.geturl()

    def read(self, name):
        import struct
        import zlib
        i = self.info[name]
        head = self._range(i.header_offset, i.header_offset + 30 + len(name.encode()) + 65536 + i.compress_size)
        if head[:4] != b"PK\x03\x04":
            raise ValueError(f"{name}: no local header")
        n, m = struct.unpack("<HH", head[26:30])
        data = head[30 + n + m:30 + n + m + i.compress_size]
        if len(data) < i.compress_size:
            data += self._range(i.header_offset + 30 + n + m + len(data), i.header_offset + 30 + n + m + i.compress_size - 1)
        if i.compress_type == zipfile.ZIP_DEFLATED:
            data = zlib.decompress(data, -15)
        elif i.compress_type != zipfile.ZIP_STORED:
            raise ValueError(f"{name}: compression {i.compress_type}")
        if zlib.crc32(data) & 0xFFFFFFFF != i.CRC:
            raise ValueError(f"{name}: CRC mismatch")
        return data

    def read_many(self, names, workers=16):
        from concurrent.futures import ThreadPoolExecutor
        with ThreadPoolExecutor(workers) as ex:
            yield from zip(names, ex.map(self.read, names))


# ---------------------------------------------------------------- rebuild / verify

def sha256_file(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def cmd_rebuild(a):
    rows = read_manifest(a.manifest)
    root = Path(a.root).expanduser()
    groups = {}
    for r in rows:
        groups.setdefault((r["source"], r["revision"], r["container"]), []).append(r)
    bad = 0
    for (source, revision, container), items in groups.items():
        todo = [r for r in items if not (root / r["path"]).exists() or sha256_file(root / r["path"]) != r["sha256"]]
        print(f"{source}/{container}: {len(items)} files, {len(todo)} to fetch", flush=True)
        if not todo:
            continue
        if container.endswith(".zip"):
            z = RemoteZip(source, revision, container)
            by_member = {r["member"]: r for r in todo}
            for mem, data in z.read_many(list(by_member)):
                bad += put(root, by_member[mem], data)
        elif container.endswith(".parquet"):
            want = {int(r["member"]): r for r in todo}
            pf, i = open_remote_parquet(source, revision, container), 0
            for batch in pf.iter_batches(columns=[a.image_column], batch_size=64):
                for cell in batch.column(0).to_pylist():
                    if i in want:
                        data = cell["bytes"] if isinstance(cell, dict) else cell
                        bad += put(root, want[i], data)
                    i += 1
        else:
            sys.exit(f"unknown container type: {container}")
    print(f"rebuild: {len(rows)} files, {bad} hash mismatches")
    sys.exit(1 if bad else 0)


def put(root, r, data):
    p = root / r["path"]
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(data)
    if hashlib.sha256(data).hexdigest() != r["sha256"]:
        print(f"MISMATCH {r['path']}", file=sys.stderr)
        return 1
    return 0


def cmd_verify(a):
    rows = read_manifest(a.manifest)
    root = Path(a.root).expanduser()
    missing = bad = 0
    for r in rows:
        p = root / r["path"]
        if not p.exists():
            missing += 1
        elif sha256_file(p) != r["sha256"]:
            bad += 1
    print(f"verify: {len(rows)} files, {missing} missing, {bad} wrong")
    sys.exit(1 if missing or bad else 0)


# ---------------------------------------------------------------- scoring

def load_scores(path, keys):
    """Map detector output to manifest paths by longest suffix.

    A row the detector marks as an error (JSON with `error` and no `score`)
    is an unreadable file: it counts as not flagged (score 0) and is listed.
    """
    by_suffix = {k: k for k in keys}
    out, unreadable = {}, []
    with open(path) as f:
        first = f.readline()
        f.seek(0)
        if first.lstrip().startswith("{"):
            rows = ((d.get("path", ""), d.get("score"), "error" in d) for d in (json.loads(l) for l in f if l.strip()))
        else:
            rows = ((*l.rstrip("\n").split("\t")[:2], False) for l in f if l.strip())
        for p, s, err in rows:
            if s is None and not err:
                continue
            parts = p.replace("\\", "/").split("/")
            for i in range(len(parts)):
                k = "/".join(parts[i:])
                if k in by_suffix:
                    if s is None:
                        out[k] = 0.0
                        unreadable.append(k)
                    else:
                        out[k] = float(s)
                    break
    return out, unreadable


def metrics(labels, scores):
    pos = sum(labels)
    neg = len(labels) - pos
    tp = sum(1 for y, s in zip(labels, scores) if y and s > 0.5)
    fp = sum(1 for y, s in zip(labels, scores) if not y and s > 0.5)
    real = sorted((s for y, s in zip(labels, scores) if not y), reverse=True)
    t = real[int(0.01 * len(real))] if real else float("inf")
    r1 = sum(1 for y, s in zip(labels, scores) if y and s > t)
    return {
        "precision": 100 * tp / (tp + fp) if tp + fp else float("nan"),
        "recall": 100 * tp / pos if pos else float("nan"),
        "fpr": 100 * fp / neg if neg else float("nan"),
        "recall_at_1pct_fpr": 100 * r1 / pos if pos and real else float("nan"),
        "generated": pos, "real": neg,
    }


def bootstrap(labels, fn, draws=DRAWS, seed=SEED):
    rng = random.Random(seed)
    pos = [i for i, y in enumerate(labels) if y]
    neg = [i for i, y in enumerate(labels) if not y]
    out = []
    for _ in range(draws):
        out.append(fn([rng.choice(pos) for _ in pos] + [rng.choice(neg) for _ in neg]))
    return out


def interval(values):
    v = sorted(x for x in values if not math.isnan(x))
    return (v[int(0.025 * len(v))], v[int(0.975 * len(v)) - 1]) if v else (float("nan"), float("nan"))


KEYS = ["precision", "recall", "fpr", "recall_at_1pct_fpr"]


def cmd_score(a):
    rows = read_manifest(a.manifest)
    keys = [r["path"] for r in rows]
    labels = [int(r["label"]) for r in rows]
    sets = [r["set"] for r in rows]
    systems, result = {}, {"manifest": str(a.manifest), "files": len(rows), "systems": {}}
    times = dict(x.split("=", 1) for x in a.time or [])
    unreadable = {}
    for spec in a.system:
        name, path = spec.split("=", 1)
        sc, unreadable[name] = load_scores(path, keys)
        missing = [k for k in keys if k not in sc]
        if missing:
            sys.exit(f"{name}: {len(missing)} of {len(keys)} manifest files have no score, e.g. {missing[:3]}")
        systems[name] = [sc[k] for k in keys]
    for name, s in systems.items():
        m = metrics(labels, s)
        m["unreadable"] = unreadable[name]
        boots = bootstrap(labels, lambda idx: metrics([labels[i] for i in idx], [s[i] for i in idx]))
        m["intervals"] = {k: interval([b[k] for b in boots]) for k in KEYS}
        if name in times:
            m["seconds"] = float(times[name])
            m["ms_per_image"] = 1000 * float(times[name]) / len(rows)
        per = {}
        for st in sorted(set(sets)):
            idx = [i for i, x in enumerate(sets) if x == st]
            per[st] = metrics([labels[i] for i in idx], [s[i] for i in idx])
        m["per_set"] = per
        result["systems"][name] = m
    names = list(systems)
    result["paired"] = {}
    for i, x in enumerate(names):
        for y in names[i + 1:]:
            sx, sy = systems[x], systems[y]
            boots = bootstrap(labels, lambda idx: (metrics([labels[j] for j in idx], [sx[j] for j in idx]),
                                                    metrics([labels[j] for j in idx], [sy[j] for j in idx])))
            result["paired"][f"{x} - {y}"] = {k: interval([bx[k] - by[k] for bx, by in boots]) for k in KEYS}
    if a.save_files:
        result["files_detail"] = {"labels": labels, "sets": sets, "scores": systems}
    if a.out:
        Path(a.out).write_text(json.dumps(result, indent=1))
    print_result(result)


def print_result(res):
    print(f"{res['files']} files")
    print("| Detector | Precision | Recall | Real flagged | Recall at 1% FPR | ms per image |")
    print("|---|---|---|---|---|---|")
    for name, m in res["systems"].items():
        ci = m["intervals"]
        cell = lambda k: f"{m[k]:.2f} ({ci[k][0]:.2f}–{ci[k][1]:.2f})"
        ms = f"{m['ms_per_image']:.1f}" if "ms_per_image" in m else "–"
        print(f"| {name} | {cell('precision')} | {cell('recall')} | {cell('fpr')}% | {cell('recall_at_1pct_fpr')} | {ms} |")
    for name, m in res["systems"].items():
        if m.get("unreadable"):
            print(f"{name}: {len(m['unreadable'])} unreadable file(s) counted as not flagged: {', '.join(m['unreadable'])}")
    for pair, iv in res.get("paired", {}).items():
        print(f"{pair}: " + ", ".join(f"{k} {lo:+.2f} to {hi:+.2f}" for k, (lo, hi) in iv.items()))


def cmd_diff(a):
    """Unpaired bootstrap of one detector between two splits (each scored with --save-files)."""
    A, B = (json.loads(Path(p).read_text()) for p in (a.a, a.b))
    out = {}
    for name in a.system:
        la, sa = A["files_detail"]["labels"], A["files_detail"]["scores"][name]
        lb, sb = B["files_detail"]["labels"], B["files_detail"]["scores"][name]
        if a.sets:
            keep = set(a.sets.split(","))
            ia = [i for i, s in enumerate(A["files_detail"]["sets"]) if s in keep]
            ib = [i for i, s in enumerate(B["files_detail"]["sets"]) if s in keep]
            la, sa, lb, sb = [la[i] for i in ia], [sa[i] for i in ia], [lb[i] for i in ib], [sb[i] for i in ib]
        ma, mb = metrics(la, sa), metrics(lb, sb)
        ba = bootstrap(la, lambda idx: metrics([la[i] for i in idx], [sa[i] for i in idx]), seed=SEED)
        bb = bootstrap(lb, lambda idx: metrics([lb[i] for i in idx], [sb[i] for i in idx]), seed=SEED + 1)
        out[name] = {k: {"a": ma[k], "b": mb[k], "diff": ma[k] - mb[k],
                         "interval": interval([x[k] - y[k] for x, y in zip(ba, bb)])} for k in KEYS}
        for k, v in out[name].items():
            lo, hi = v["interval"]
            print(f"{name} {k}: {v['a']:.2f} vs {v['b']:.2f}, difference {v['diff']:+.2f} (95% {lo:+.2f} to {hi:+.2f})")
    if a.out:
        Path(a.out).write_text(json.dumps(out, indent=1))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("rebuild", help="download every file of a manifest and check its SHA-256")
    p.add_argument("manifest")
    p.add_argument("--root", required=True)
    p.add_argument("--image-column", default="image")
    p = sub.add_parser("verify", help="check every file of a manifest under --root")
    p.add_argument("manifest")
    p.add_argument("--root", required=True)
    p = sub.add_parser("score", help="the fixed metrics for one or more detectors")
    p.add_argument("manifest")
    p.add_argument("--system", action="append", required=True, metavar="NAME=FILE")
    p.add_argument("--time", action="append", metavar="NAME=SECONDS", help="wall time from start-up to last verdict")
    p.add_argument("--out")
    p.add_argument("--save-files", action="store_true", help="keep per-file scores in --out (needed by diff)")
    p = sub.add_parser("diff", help="one detector, split A against split B")
    p.add_argument("a")
    p.add_argument("b")
    p.add_argument("--system", action="append", required=True)
    p.add_argument("--sets", help="comma-separated sets to restrict both splits to")
    p.add_argument("--out")
    a = ap.parse_args()
    {"rebuild": cmd_rebuild, "verify": cmd_verify, "score": cmd_score, "diff": cmd_diff}[a.cmd](a)


if __name__ == "__main__":
    main()
