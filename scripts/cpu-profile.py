#!/usr/bin/env python3
"""Turn the timers of a CPU run into a table of where the time goes.

Standard library only: `python3 -I scripts/cpu-profile.py results.jsonl ...`
(or the JSON lines on standard input). The input is what
`scripts/view-reuse-bench.py` prints on standard output, one JSON object per
binary, from a run with `--latency` (one image at a time, file to verdict)
or `--together` (all files in one process: throughput):

    python3 -I scripts/view-reuse-bench.py --bin accelerate=target/release/tintype \\
        --latency --manifest photos.tsv --n 300 --repeat 5 --device cpu > latency.jsonl
    python3 -I scripts/cpu-profile.py latency.jsonl

One column pair per input line: milliseconds per file and the share of the
file-to-verdict time. What the rows mean:

- With `--latency` everything happens on one thread after another, so the
  rows add up to the images' own times (`file_to_verdict_ms`).
- With `--together` the files are decoded and their views prepared on worker
  threads while the network runs, so those two rows are processor time on
  other threads, shown without a share; the shares are of the wall time of
  the whole process, and "outside the network" is the wall time the scoring
  thread was not in the network (start-up, loading the weights, waiting for
  files, output).
- The parts of the forward pass are timed on the thread that runs the pass.
  Where the views of a batch run side by side (the pure-Rust kernels) those
  add up to more than the time the scoring thread waited, and are scaled to
  it (`model_ms / forward_ms`) so that the table still adds up.

`--self-test` checks the arithmetic on made-up counters.
"""
import json
import sys

PARTS = [
    ("patch_ms", "Patch embedding (unfold, normalise, linear, positions)"),
    ("qkv_ms", "Attention: query/key/value linear"),
    ("proj_ms", "Attention: output linear"),
    ("fc1_ms", "MLP: first linear"),
    ("fc2_ms", "MLP: second linear"),
    ("attention_ms", "Attention: query-key and attention-value products"),
    ("softmax_ms", "Softmax"),
    ("gather_ms", "Attention: copying heads together and back"),
    ("gelu_ms", "GELU"),
    ("norm_ms", "Layer normalisation"),
    ("head_ms", "Head"),
    ("residual_ms", "Residual additions"),
]
LINEAR = ("qkv_ms", "proj_ms", "fc1_ms", "fc2_ms")


def breakdown(result):
    """Rows of (label, milliseconds per file or None, share of the total or None), and the total per file."""
    c = result["counters"]
    latency = result["mode"] == "latency"
    # `latency` warms up on five images that the counters include.
    files = result["files"] + 5 if latency else result["files"]
    # The counters are means over the passes, so the wall time is too.
    total = c["file_to_verdict_ms"] if latency else sum(result["wall_s"]) / len(result["wall_s"]) * 1e3
    model, forward = c["model_ms"], c.get("forward_ms", 0)
    scale = model / forward if forward else 0.0
    rows = []

    def row(label, ms, shared=True):
        rows.append((label, ms / files, ms / total if shared and total else None))

    row("Reading and decoding the file" + ("" if latency else " (worker threads)"), c["decode_ms"], latency)
    row("Ranking windows, resizing and cropping views" + ("" if latency else " (worker threads)"), c["views_ms"] + c.get("convert_ms", 0), latency)
    row("In the network", model)
    parts = 0.0
    for key, label in PARTS:
        row("- " + label, c.get(key, 0) * scale)
        parts += c.get(key, 0) * scale
    row("- Buffers, gathering class tokens", max(model - parts, 0.0) if forward else 0.0)
    row("- All four linear layers of the blocks together", sum(c.get(k, 0) for k in LINEAR) * scale)
    if latency:
        row("Everything else", max(total - c["decode_ms"] - c["views_ms"] - c.get("convert_ms", 0) - model, 0.0))
    else:
        row("Outside the network (start-up, weights, waiting for files, output)", max(total - model, 0.0))
    rows.append(("File to verdict" if latency else "Wall time per file", total / files, 1.0))
    return rows


def title(result):
    what = "latency" if result["mode"] == "latency" else "throughput"
    extra = " ".join(x for x in [result.get("extra", ""), *(e.partition(":")[2] for e in result.get("env", []))] if x)
    return f"{result['label']}, {what}, `{result['device']}`" + (f" {extra}" if extra else "")


def table(results):
    out = []
    columns = [breakdown(r) for r in results]
    out.append("| Part | " + " | ".join(f"{title(r)}: ms per file | share" for r in results) + " |")
    out.append("|---|" + "---:|---:|" * len(results))
    for k in range(len(columns[0])):
        cells = []
        for col in columns:
            _, ms, share = col[k]
            cells.append(f"{ms:.2f} | " + ("" if share is None else f"{100 * share:.1f}%"))
        out.append(f"| {columns[0][k][0] if len({col[k][0] for col in columns}) == 1 else ' / '.join(dict.fromkeys(col[k][0] for col in columns))} | " + " | ".join(cells) + " |")
    notes = []
    for r in results:
        load = r.get("load_average_before_and_after")
        speed = f"p50 {r['best_p50_ms']} / p95 {r['best_p95_ms']} / mean {r['best_mean_ms']} ms (each image's best of {r['passes']})" if r["mode"] == "latency" else f"{r['median_files_per_s']} files/s (median of {r['passes']}), best {r['best_files_per_s']}"
        looked = r.get("mean_views_or_frames_looked_at")
        notes.append(f"- {title(r)}: {r['files']} files, {speed}" + (f", {looked} views per file" if looked else f", {r['counters'].get('views_scored', 0)} views scored") + f", peak memory {r['peak_rss_mb']} MB" + (f", load average {load[0]} before and {load[1]} after" if load else ""))
    return "\n".join(out + [""] + notes)


def self_test():
    counters = {"decode_ms": 100, "convert_ms": 0, "views_ms": 50, "model_ms": 800, "forward_ms": 1600, "patch_ms": 100, "norm_ms": 100, "qkv_ms": 400, "attention_ms": 100, "softmax_ms": 100, "gather_ms": 40, "residual_ms": 20, "proj_ms": 100, "fc1_ms": 300,
                "gelu_ms": 100, "fc2_ms": 200, "head_ms": 0, "file_to_verdict_ms": 1000, "views_scored": 20}
    latency = {"label": "a", "mode": "latency", "device": "cpu", "files": 5, "passes": 1, "counters": counters, "best_p50_ms": 1, "best_p95_ms": 2, "best_mean_ms": 1.5, "peak_rss_mb": 1.0}
    rows = dict((label, (ms, share)) for label, ms, share in breakdown(latency))
    # Ten images with the warm-up; the forward ran on two threads at once, so its parts are halved.
    assert rows["Reading and decoding the file"] == (10.0, 0.1)
    assert rows["In the network"] == (80.0, 0.8)
    assert rows["- Attention: query/key/value linear"] == (20.0, 0.2)
    assert rows["- All four linear layers of the blocks together"] == (50.0, 0.5)
    assert rows["- Buffers, gathering class tokens"] == (2.0, 0.02), rows
    assert rows["- Residual additions"] == (1.0, 0.01)
    assert rows["Everything else"] == (5.0, 0.05)
    assert rows["File to verdict"] == (100.0, 1.0)
    parts = sum(ms for label, (ms, _) in rows.items() if label.startswith("- ") and "All four" not in label)
    assert abs(parts - rows["In the network"][0]) < 1e-9
    together = dict(latency, mode="together", wall_s=[1.5, 2.5], median_files_per_s=2.5, best_files_per_s=2.6)
    rows = dict((label, (ms, share)) for label, ms, share in breakdown(together))
    assert rows["Reading and decoding the file (worker threads)"] == (20.0, None)
    assert rows["In the network"] == (160.0, 0.4)
    assert rows["Outside the network (start-up, weights, waiting for files, output)"] == (240.0, 0.6)
    assert rows["Wall time per file"] == (400.0, 1.0)
    assert "| Part |" in table([latency, together])
    print("self-test passed")


def main():
    args = sys.argv[1:]
    if args == ["--self-test"]:
        return self_test()
    if any(a.startswith("-") for a in args):
        sys.exit(__doc__)
    lines = [l for f in args for l in open(f)] if args else sys.stdin.readlines()
    results = [json.loads(l) for l in lines if l.strip()]
    if not results:
        sys.exit("no results to tabulate")
    print(table(results))


if __name__ == "__main__":
    main()
