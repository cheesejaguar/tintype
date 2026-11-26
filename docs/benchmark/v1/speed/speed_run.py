#!/usr/bin/env python3
"""Matched idle-hardware speed run (PROTOCOL.md). Usage: speed_run.py RAMDISK OUT.tsv

RAMDISK holds `sample/` (2,000 files), `warm/` (100 files), their store
manifests `sample.tsv` and `warm.tsv`, and `gpttrace.safetensors`.
"""
import os
import subprocess
import sys
import time
from pathlib import Path

ram, out = Path(sys.argv[1]), Path(sys.argv[2])
SCRATCH = Path("/Users/aaron/Code/veracity-scratch")
RESEARCH = Path("/Users/aaron/.codex/worktrees/afe8/veracity")
DET = {
    1: ("Tintype v0.4.0 default", SCRATCH / "edit-check", {"TINTYPE_MODEL_DIR": "models"},
        lambda d, m: ["./target/release/tintype", "batch", "--json", str(d)]),
    2: ("GPTTrace flag-pec-v3", RESEARCH, {},
        lambda d, m: ["./target/release/tintype", "batch", "--json", "--gpttrace", str(ram / "gpttrace.safetensors"), str(d)]),
    3: ("Community Forensics 384", SCRATCH / "release-040", {},
        lambda d, m: ["./target/release/tintype", "score", "--reference", "commfor", "--weights",
                      str(SCRATCH / "baselines/commfor-384.safetensors"), "--manifest", str(m), "--out", str(ram / "o.tsv")]),
    4: ("SwinV2 detector", SCRATCH / "release-040", {},
        lambda d, m: ["./target/release/tintype", "score", "--reference", "swinv2", "--weights",
                      str(SCRATCH / "baselines/swinv2-detector.safetensors"), "--manifest", str(m), "--out", str(ram / "o.tsv")]),
}
ORDERS = [(1, 2, 3, 4), (2, 3, 4, 1), (3, 4, 1, 2)]


def run(k, which):
    name, cwd, env, argv = DET[k]
    d, m = ram / which, ram / f"{which}.tsv"
    t0 = time.monotonic()
    p = subprocess.run(argv(d, m), cwd=cwd, env={**os.environ, **env}, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    dt = time.monotonic() - t0
    if p.returncode and "could not be scored" not in p.stderr:
        sys.exit(f"{name} {which}: exit {p.returncode}\n{p.stderr[-1500:]}")
    return dt


with out.open("w") as f:
    f.write("round\tdetector\tseconds\tms_per_image\tload1\n")
    for r, order in enumerate(ORDERS, 1):
        for k in order:
            run(k, "warm")
            load = os.getloadavg()[0]
            dt = run(k, "sample")
            f.write(f"{r}\t{DET[k][0]}\t{dt:.2f}\t{1000 * dt / 2000:.2f}\t{load:.2f}\n")
            f.flush()
            print(f"round {r} {DET[k][0]}: {dt:.1f} s, {1000 * dt / 2000:.2f} ms/image, load {load:.2f}", flush=True)
