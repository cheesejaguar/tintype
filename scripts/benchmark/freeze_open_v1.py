#!/usr/bin/env python3
"""Freeze TPB v1's open and OpenFake manifests from batch 9 (record of how they were made).

    python3 freeze_open_v1.py <commfor.confirm9.tsv.gz> <hashes.tsv> docs/benchmark/v1

<commfor.confirm9.tsv.gz> is the research archive listing batch 9's 31,951
files with labels, generators and sets. <hashes.tsv> holds `path, sha256,
bytes` for each file, read from the batch-9 folder on the data share.
"""
import gzip
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tpb  # noqa: E402

SRC = {
    "aigibench": ("HorizonTEL/AIGIBench", "e44ec40efe5117a5ccdaa6ff0e89ed934d03d310"),
    "classic": ("sywang/CNNDetection", "f287cf3c3e92ca2e5451ce200bab8b6f5e0175b2"),
    "sofake": ("saberzl/So-Fake-OOD", "b4b34d16fda2020c2923a21d431d1ab3acbdbaff"),
    "openfake": ("ComplexDataLab/OpenFake", "00b2f0ae3f5f7684c4b00f794f4c500076a9355d"),
}


def canonical(dataset):
    return re.sub(r"-(b\d+|fresh\d+)$", "", dataset)


def main():
    arch, hashes, outdir = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    h = {}
    for line in open(hashes):
        p, s, n = line.rstrip("\n").split("\t")
        h[p] = (s, n)
    open_rows, of_rows = [], []
    for line in gzip.open(arch, "rt"):
        p, label, gen, dataset = line.rstrip("\n").split("\t")[:4]
        rel = p.split("data/fresh10/", 1)[1]
        folder, rest = rel.split("/", 1)
        if folder.startswith("aigibench-"):
            (source, rev), container, member = SRC["aigibench"], f"test/{folder[len('aigibench-'):]}.zip", rest
        elif folder.startswith("classic-"):
            (source, rev), container, member = SRC["classic"], "CNN_synth_testset.zip", rest
        elif folder == "sofake-ood":
            shard, name = rest.split("/")
            (source, rev), container, member = SRC["sofake"], f"data/{shard}.parquet", str(int(name.split(".")[0]))
        elif folder == "openfake-test":
            shard, name = rest.split("/")
            (source, rev), container, member = SRC["openfake"], f"core/{shard}.parquet", str(int(name.split(".")[0]))
        else:
            sys.exit(f"unknown folder {folder}")
        sha, size = h[p]
        row = {"path": rel, "label": label, "set": canonical(dataset), "generator": gen, "source": source,
               "revision": rev, "container": container, "member": member, "sha256": sha, "bytes": size}
        (of_rows if folder == "openfake-test" else open_rows).append(row)
    open_rows.sort(key=lambda r: r["path"])
    of_rows.sort(key=lambda r: r["path"])
    tpb.write_manifest(outdir / "open.tsv.gz", open_rows)
    tpb.write_manifest(outdir / "openfake.tsv.gz", of_rows)
    print(f"open: {len(open_rows)} files; openfake: {len(of_rows)} files")


if __name__ == "__main__":
    main()
