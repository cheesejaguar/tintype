#!/usr/bin/env python3
"""Time tintype over a list of files and add up its work counters.

Standard library only; run it as `python3 -I scripts/view-reuse-bench.py ...`
from the repository root (the binaries find `models/` there).

  --each      `detect`, one process per file: file-to-verdict wall time per
              file (it includes starting the process and loading the weights)
  --together  `detect`, one process for all files: wall time and files per
              second
  --latency   `latency` on a manifest of images: one process, one image at a
              time from file to verdict, without the start-up (photos only)

Give `--bin LABEL=PATH` once per binary to compare. The binaries take turns
(on every file with `--each`, on the whole list otherwise, in a different
order each pass), so that a busy machine slows them alike; `--repeat N`
makes N passes. With `--each` and `--latency` the report gives p50, p95 and
mean over all runs, and over each file's fastest run ("best"), which is the
steadier figure on a shared machine; every binary after the first also gets
the median, over files, of its best time divided by the first binary's best
time for the same file. `--env LABEL:NAME=VALUE` sets a variable for one
binary, e.g. `--env new:TINTYPE_VIEW_REUSE=off`.

Every report also gives the counters the binary prints with TINTYPE_STATS=1
for one pass over the list (frames decoded and converted, views
materialised, scored and reused, clips reopened, and the milliseconds spent
decoding, converting, preparing views and in the network and its parts,
summed over threads and averaged over passes; with `--latency` also
`file_to_verdict_ms`, the images' own times added up, warm-up included as in
the counters), the largest resident set size of any of its processes, and
the one-minute load average before and after. See
docs/runbooks/view-reuse.md; `scripts/cpu-profile.py` turns the JSON lines
into the table of docs/runbooks/cpu-profile-int8.md.
"""
import argparse
import json
import os
import subprocess
import sys
import tempfile
import time


def quantile(sorted_values, q):
    if not sorted_values:
        return float("nan")
    pos = q * (len(sorted_values) - 1)
    lo = int(pos)
    hi = min(lo + 1, len(sorted_values) - 1)
    return sorted_values[lo] + (sorted_values[hi] - sorted_values[lo]) * (pos - lo)


def run(binary, files, device, extra, env, latency=None):
    """One process: (seconds, counters, rows, peak resident megabytes).

    `detect` over `files`, or with `latency=(weights, manifest, n)` the
    `latency` command, whose rows are (image index, milliseconds).
    """
    command = [binary, "detect", "--device", device, "--json", *extra, *files]
    if latency:
        command = [binary, "latency", "--weights", latency[0], "--manifest", latency[1], "--n", str(latency[2]), "--device", device, *extra]
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        start = time.perf_counter()
        p = subprocess.Popen(command, stdout=out, stderr=err, env=env)
        _, status, usage = os.wait4(p.pid, 0)
        seconds = time.perf_counter() - start
        p.returncode = os.waitstatus_to_exitcode(status)
        out.seek(0)
        err.seek(0)
        stdout, stderr = out.read().decode(), err.read().decode(errors="replace")
    # Exit status 2 means some file could not be scored; its row says so.
    if p.returncode not in (0, 2):
        sys.exit(f"{binary} failed ({p.returncode}): {stderr[-2000:]}")
    counters = {}
    for line in stderr.splitlines():
        if line.startswith("stats: "):
            counters = {k: int(v) for k, v in (f.split("=") for f in line.split()[1:])}
    if latency:
        rows = [(int(f[1]), float(f[2])) for f in (l.split() for l in stderr.splitlines()) if f and f[0] == "latency_ms"]
        # The counters cover the warm-up images too; so does this, to set beside them.
        counters["file_to_verdict_ms"] = round(sum(float(f[2]) for f in (l.split() for l in stderr.splitlines()) if f and f[0] in ("latency_ms", "warmup_ms")))
    else:
        rows = [json.loads(l) for l in stdout.splitlines() if l]
    # Resident set size: bytes on macOS, kilobytes on Linux.
    peak = usage.ru_maxrss / (1 << 20) if sys.platform == "darwin" else usage.ru_maxrss / 1024
    return seconds, counters, rows, peak


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bin", action="append", required=True, metavar="LABEL=PATH", help="a tintype binary to time (repeatable)")
    ap.add_argument("--list", help="text file with one media path per line (--each, --together)")
    ap.add_argument("--manifest", help="manifest of images (--latency)")
    ap.add_argument("--n", type=int, default=200, help="images to time with --latency, after its five warm-up images")
    ap.add_argument("--weights", default="models/tintype.safetensors", help="photo weights for --latency")
    ap.add_argument("--device", default="cpu")
    ap.add_argument("--extra", default="", help="more detect flags, e.g. '--all-views'")
    ap.add_argument("--env", action="append", default=[], metavar="LABEL:NAME=VALUE", help="a variable for one binary (repeatable)")
    ap.add_argument("--repeat", type=int, default=1, help="passes over the list")
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument("--each", action="store_true")
    mode.add_argument("--together", action="store_true")
    mode.add_argument("--latency", action="store_true")
    a = ap.parse_args()
    if a.latency and not a.manifest or not a.latency and not a.list:
        ap.error("--latency takes --manifest; --each and --together take --list")

    files = [l for l in open(a.manifest if a.latency else a.list).read().splitlines() if l]
    if a.latency:
        files = files[5:5 + a.n]
    extra = a.extra.split()
    bins = []
    for spec in a.bin:
        label, _, path = spec.partition("=")
        env = dict(os.environ, TINTYPE_STATS="1")
        env.pop("TINTYPE_VIEW_REUSE", None)
        for e in a.env:
            who, _, kv = e.partition(":")
            if who == label:
                name, _, value = kv.partition("=")
                env[name] = value
        bins.append({"label": label, "path": path, "env": env, "times": [[] for _ in files], "walls": [], "counters": {}, "peak": 0.0, "rows": []})
    latency = (a.weights, a.manifest, a.n) if a.latency else None

    def note(b, counters, rows, peak, first_pass):
        b["peak"] = max(b["peak"], peak)
        for k, v in counters.items():
            # Counts from the first pass only (they repeat); times from every pass.
            if first_pass or k.endswith("_ms"):
                b["counters"][k] = b["counters"].get(k, 0) + v
        if first_pass and not a.latency:
            b["rows"].extend(rows)

    load_before = os.getloadavg()
    # One untimed run each first, so nobody pays for a cold file cache.
    for b in bins:
        if not a.latency:
            run(b["path"], files[:1], a.device, extra, b["env"])
    for n in range(a.repeat):
        order = bins[n % len(bins):] + bins[:n % len(bins)]
        if a.latency:
            for b in order:
                _, c, per_image, peak = run(b["path"], [], a.device, extra, b["env"], latency)
                for k, ms in per_image:
                    b["times"][k - 5].append(ms)
                note(b, c, [], peak, n == 0)
        elif a.each:
            for k, f in enumerate(files):
                for b in order:
                    s, c, r, peak = run(b["path"], [f], a.device, extra, b["env"])
                    b["times"][k].append(s * 1e3)
                    note(b, c, r, peak, n == 0)
        else:
            for b in order:
                s, c, r, peak = run(b["path"], files, a.device, extra, b["env"])
                b["walls"].append(s)
                note(b, c, r, peak, n == 0)

    for b in bins:
        b["counters"] = {k: round(v / a.repeat) if k.endswith("_ms") else v for k, v in b["counters"].items()}
        looked = [r.get("views", r.get("frames")) for r in b["rows"] if "score" in r]
        result = {
            "label": b["label"], "bin": b["path"], "device": a.device, "extra": a.extra, "env": [e for e in a.env if e.startswith(b["label"] + ":")],
            "files": len(files), "passes": a.repeat, "scored_rows": len(looked),
            "mean_views_or_frames_looked_at": round(sum(looked) / len(looked), 3) if looked else None,
            "peak_rss_mb": round(b["peak"], 1), "counters": b["counters"],
            "load_average_before_and_after": [round(load_before[0], 2), round(os.getloadavg()[0], 2)],
        }
        if a.each or a.latency:
            # An image `latency` could not read has no times.
            timed = [(mine, first) for mine, first in zip(b["times"], bins[0]["times"]) if mine and first]
            every = sorted(t for mine, _ in timed for t in mine)
            best = sorted(min(mine) for mine, _ in timed)
            result.update(mode="each" if a.each else "latency", p50_ms=round(quantile(every, 0.5), 1), p95_ms=round(quantile(every, 0.95), 1), mean_ms=round(sum(every) / len(every), 1),
                          best_p50_ms=round(quantile(best, 0.5), 1), best_p95_ms=round(quantile(best, 0.95), 1), best_mean_ms=round(sum(best) / len(best), 1))
            head = (f"{'file-to-verdict' if a.each else 'single-image latency'}, {len(every)} runs: p50 {result['p50_ms']} p95 {result['p95_ms']} mean {result['mean_ms']} ms; "
                    f"each file's best of {a.repeat}: p50 {result['best_p50_ms']} p95 {result['best_p95_ms']} mean {result['best_mean_ms']} ms")
            if b is not bins[0]:
                ratios = sorted(min(mine) / min(first) for mine, first in timed)
                result["median_best_ratio_to_" + bins[0]["label"]] = round(quantile(ratios, 0.5), 4)
                head += f"; median per-file ratio to {bins[0]['label']} {result['median_best_ratio_to_' + bins[0]['label']]}"
        else:
            walls = sorted(b["walls"])
            result.update(mode="together", wall_s=[round(t, 3) for t in b["walls"]], best_wall_s=round(walls[0], 3), median_wall_s=round(quantile(walls, 0.5), 3),
                          best_files_per_s=round(len(files) / walls[0], 1), median_files_per_s=round(len(files) / quantile(walls, 0.5), 1))
            head = f"{len(files)} files in one process, {len(walls)} runs: best {walls[0]:.2f} s ({result['best_files_per_s']} files/s), median {quantile(walls, 0.5):.2f} s ({result['median_files_per_s']} files/s)"
        print(f"{b['label']} [{a.device} {a.extra}]: {head}; peak RSS {result['peak_rss_mb']} MB; " + " ".join(f"{k}={v}" for k, v in b["counters"].items()), file=sys.stderr)
        print(json.dumps(result))


if __name__ == "__main__":
    main()
