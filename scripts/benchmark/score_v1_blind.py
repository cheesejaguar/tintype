#!/usr/bin/env python3
"""Score TPB v1's blind split with the four detectors of BLIND-PROTOCOL.md (record of the run).

Set by set: rebuild the set's files from their sources into TMP (with
SHA-256 checks), run each detector over them, append the outputs, and
delete the files. The data share has room for one set at a time, not the
whole split. Wall time is summed per detector, from start-up to last
verdict of each set's run.

    python3 score_v1_blind.py docs/benchmark/v1/blind.tsv.gz OUTDIR
"""
import collections
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tpb  # noqa: E402

TMP = Path("/Volumes/veracity-data/data/tpb-v1-blind-tmp")
SCRATCH = Path("/Users/aaron/Code/veracity-scratch")
RESEARCH = Path("/Users/aaron/.codex/worktrees/afe8/veracity")
GPTTRACE = "/Volumes/veracity-data/models/gpttrace-pec-v3/model.safetensors"
DETECTORS = {
    # name: (cwd, env, argv builder taking (set dir, store manifest, out file)), output kind
    "tintype": (SCRATCH / "edit-check", {"TINTYPE_MODEL_DIR": "models"},
                lambda d, m, o: ["./target/release/tintype", "batch", "--json", str(d)], "stdout"),
    "gpttrace": (RESEARCH, {}, lambda d, m, o: ["./target/release/tintype", "batch", "--json", "--gpttrace", GPTTRACE, str(d)], "stdout"),
    "commfor": (SCRATCH / "release-040", {}, lambda d, m, o: ["./target/release/tintype", "score", "--reference", "commfor",
                "--weights", str(SCRATCH / "baselines/commfor-384.safetensors"), "--manifest", str(m), "--out", str(o)], "file"),
    "swinv2": (SCRATCH / "release-040", {}, lambda d, m, o: ["./target/release/tintype", "score", "--reference", "swinv2",
               "--weights", str(SCRATCH / "baselines/swinv2-detector.safetensors"), "--manifest", str(m), "--out", str(o)], "file"),
}


def main():
    manifest, out = sys.argv[1], Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    rows = tpb.read_manifest(manifest)
    by_set = collections.defaultdict(list)
    for r in rows:
        by_set[r["set"]].append(r)
    timing_path = out / "timing.json"
    timing = json.loads(timing_path.read_text()) if timing_path.exists() else {"done": [], "seconds": {}}
    for st, items in sorted(by_set.items()):
        if st in timing["done"]:
            continue
        root = TMP / "files"
        if root.exists():
            shutil.rmtree(root)
        sub = TMP / f"{st}.tsv.gz"
        TMP.mkdir(parents=True, exist_ok=True)
        tpb.write_manifest(sub, items)
        r = subprocess.run([sys.executable, "-I", str(Path(__file__).parent / "tpb.py"), "rebuild", str(sub), "--root", str(root)])
        if r.returncode:
            sys.exit(f"{st}: rebuild failed")
        store = TMP / f"{st}.store.tsv"
        store.write_text("".join(f"{root / x['path']}\t{x['label']}\t{x['generator']}\t{st}\n" for x in items))
        for name, (cwd, env, argv, kind) in DETECTORS.items():
            o = out / f"{name}.{st}.tsv"
            t0 = time.time()
            if kind == "stdout":
                with open(out / f"{name}.jsonl", "a") as f:
                    p = subprocess.run(argv(root / st, store, o), cwd=cwd, env={**__import__("os").environ, **env},
                                       stdout=f, stderr=subprocess.PIPE, text=True)
            else:
                p = subprocess.run(argv(root / st, store, o), cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            dt = time.time() - t0
            if p.returncode:
                # `tintype batch` exits 1 when some inputs could not be decoded; those rows carry an
                # `error` and count as not flagged (Amendment 1). Anything else stops the run.
                if "could not be scored" in p.stderr:
                    print(f"{st} {name}: {p.stderr.strip().splitlines()[-1]}", flush=True)
                else:
                    sys.exit(f"{st} {name}: exit {p.returncode}\n{p.stderr[-2000:]}")
            timing["seconds"][name] = timing["seconds"].get(name, 0) + dt
            print(f"{st}: {name} {len(items)} files in {dt:.1f} s", flush=True)
        shutil.rmtree(root)
        sub.unlink()
        timing["done"].append(st)
        timing_path.write_text(json.dumps(timing, indent=1))
    # one TSV per reference detector: path, prob (column 5 of `tintype score`)
    for name in ("commfor", "swinv2"):
        with open(out / f"{name}.tsv", "w") as f:
            for p in sorted(out.glob(f"{name}.*.tsv")):
                if p.name == f"{name}.tsv":
                    continue
                for line in open(p):
                    c = line.rstrip("\n").split("\t")
                    f.write(f"{c[0]}\t{c[4]}\n")
    print(json.dumps(timing))


if __name__ == "__main__":
    main()
