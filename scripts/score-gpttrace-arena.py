#!/usr/bin/env python3
"""Score the pinned GPTTrace ONNX model on the archived Arena input pairs."""

import argparse
import csv
import gzip
import importlib.util
import io
import json
import math
import platform
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from tempfile import TemporaryDirectory

# Keep the existing public Arena CLI path and reuse its extraction and encoder.
arena_spec = importlib.util.spec_from_file_location(
    "evaluate_arena", Path(__file__).with_name("evaluate-arena.py")
)
arena = importlib.util.module_from_spec(arena_spec)
sys.modules[arena_spec.name] = arena
arena_spec.loader.exec_module(arena)


MODEL_URL = "https://huggingface.co/nakas/gpttrace-img-flag-pec-v3-onnx/resolve/197d072c7ff2e41d53f5e23b57e16c9e0d9ba042"
MODEL_HASH = "9245bf6420ddfa22fa0cd34753c32e38d08fe649eafcf1e3cff84b24d5f473ce"
CONFIG_HASH = "ea7393da661319ae410ac835097d4af4adb8d783d919864e74ce98fb54c780e3"
OFFSET = 3.562
BASE_RESULTS = Path(__file__).resolve().parents[1] / "docs/results/arena-v0.1-gpu"
INPUTS = BASE_RESULTS / "inputs.tsv.gz"


def merge_scores(raw, output):
    scores = {}
    with raw.open(encoding="utf-8", newline="") as stream:
        for row in csv.DictReader(stream, delimiter="\t"):
            key = f"{row['variant']}/{row['id']}"
            logit = float(row["adjusted_logit"])
            raw_logit = float(row["raw_logit"])
            if (key in scores or not math.isfinite(logit) or not math.isfinite(raw_logit)
                    or abs(logit - (raw_logit - OFFSET)) > 1e-9):
                raise ValueError(f"invalid GPTTrace row for {key}")
            # Preserve the model card's strict logit > offset decision at score > 0.5.
            scores[key] = (1.0 + math.tanh(logit / 2.0)) / 2.0

    with gzip.open(BASE_RESULTS / "scores.tsv.gz", "rt", encoding="utf-8", newline="") as stream:
        header, *rows = csv.reader(stream, delimiter="\t", quoting=csv.QUOTE_NONE)
    expected = {row[0] for row in rows}
    if scores.keys() != expected:
        raise ValueError(f"GPTTrace coverage differs: {len(expected - scores.keys())} missing, "
                         f"{len(scores.keys() - expected)} extra")
    with output.open("wb") as binary:
        with gzip.GzipFile(filename="", fileobj=binary, mode="wb", mtime=0) as archive:
            with io.TextIOWrapper(archive, encoding="utf-8", newline="") as stream:
                writer = csv.writer(stream, delimiter="\t", lineterminator="\n", quoting=csv.QUOTE_NONE)
                writer.writerow([*header, "gpttrace"])
                writer.writerows([*row, scores[row[0]]] for row in rows)


def cases_for(archive, scratch, jobs):
    with gzip.open(INPUTS, "rt", encoding="utf-8", newline="") as stream:
        expected = {row["id"]: row for row in csv.DictReader(stream, delimiter="\t") if row["paired"] == "1"}
    items, *_ = arena.prepare_media(archive, scratch)
    items = [item for item in items if item.id in expected]
    if {item.id for item in items} != expected.keys():
        raise ValueError("Arena archive membership differs from the published pairs")
    failures = arena.normalize(items, scratch, jobs)
    if failures:
        raise ValueError(f"could not re-encode {len(failures)} images")
    for item in items:
        row = expected[item.id]
        if (item.label, item.generator, item.category, item.member) != (
            int(row["label"]), row["generator"], row["category"], row["member"]
        ):
            raise ValueError(f"Arena metadata differs for {item.id}")
        for path, digest in ((item.original, row["original_sha256"]), (item.jpeg, row["jpeg_sha256"])):
            arena.require_digest(scratch / path, digest)
    return [(item.id, variant, scratch / item.path(variant)) for variant in arena.VARIANTS for item in items]


def pixels(path, res, np, image):
    with image.open(path) as source:
        rgb = source.convert("RGB")
    width, height = rgb.size
    scale = res / min(width, height)
    fit = rgb.resize((round(width * scale), round(height * scale)), image.Resampling.BICUBIC)
    left, top = (fit.width - res) // 2, (fit.height - res) // 2
    views = (fit.crop((left, top, left + res, top + res)), rgb.resize((res, res), image.Resampling.BICUBIC))
    return np.stack([(np.asarray(view, np.float32) / 255 - 0.5) / 0.5 for view in views]).transpose(0, 3, 1, 2)


def run(args, scratch):
    import numpy as np
    import onnxruntime as ort
    import PIL
    from PIL import Image

    archive = args.archive or scratch / "benchmark-v0.1.zip"
    if not args.archive:
        arena.download(arena.ARENA_URL, archive)
    arena.require_digest(archive, arena.ARENA_SHA256)
    model = args.model or scratch / "model"
    if not args.model:
        model.mkdir()
        (model / "onnx").mkdir()
        arena.download(f"{MODEL_URL}/config.json", model / "config.json")
        arena.download(f"{MODEL_URL}/onnx/model_fp32.onnx", model / "onnx/model_fp32.onnx")
    config = model / "config.json"
    weights = model / "onnx/model_fp32.onnx"
    arena.require_digest(config, CONFIG_HASH)
    arena.require_digest(weights, MODEL_HASH)
    cfg = json.loads(config.read_text(encoding="utf-8"))

    cases = cases_for(archive, scratch, args.jobs)
    providers = ["CPUExecutionProvider"]
    if args.coreml:
        if "CoreMLExecutionProvider" not in ort.get_available_providers():
            raise ValueError("this ONNX Runtime build has no Core ML provider")
        providers.insert(0, ("CoreMLExecutionProvider", {
            "ModelFormat": "MLProgram",
            "RequireStaticInputShapes": "1",
            "ModelCacheDirectory": str(args.output.resolve() / "coreml-cache"),
        }))
    options = ort.SessionOptions()
    options.intra_op_num_threads = args.threads
    options.inter_op_num_threads = 1
    session = ort.InferenceSession(str(weights), sess_options=options, providers=providers)
    if args.coreml and session.get_providers()[0] != "CoreMLExecutionProvider":
        raise ValueError("ONNX Runtime did not start Core ML")
    partial = args.output / "gpttrace.partial.tsv"
    with partial.open("w", encoding="utf-8", newline="") as stream, ThreadPoolExecutor(max_workers=args.jobs) as pool:
        writer = csv.writer(stream, delimiter="\t", lineterminator="\n")
        writer.writerow(("id", "variant", "raw_logit", "adjusted_logit"))
        for offset in range(0, len(cases), 32):
            chunk = cases[offset:offset + 32]
            tensors = pool.map(lambda case: pixels(case[2], cfg["res"], np, Image), chunk)
            for (item_id, variant, _), tensor in zip(chunk, tensors):
                logit = float(session.run(None, {"pixel_values": tensor})[0].mean())
                if not math.isfinite(logit):
                    raise ValueError(f"nonfinite GPTTrace logit for {item_id} ({variant})")
                writer.writerow((item_id, variant, logit, logit - cfg["offset"]))
            stream.flush()
            print(f"Scored {min(offset + len(chunk), len(cases))}/{len(cases)}", file=sys.stderr)
    partial.rename(args.output / "gpttrace.tsv")
    run_record = {
        "model": MODEL_URL,
        "weights_sha256": MODEL_HASH,
        "config_sha256": CONFIG_HASH,
        "arena_sha256": arena.ARENA_SHA256,
        "archived_input_pairs_sha256": arena.file_digest(INPUTS),
        "platform": platform.platform(),
        "python": platform.python_version(),
        "onnxruntime": ort.__version__,
        "numpy": np.__version__,
        "pillow": PIL.__version__,
        "providers": session.get_providers(),
        "threads": args.threads,
        "offset": cfg["offset"],
        "scored": len(cases),
    }
    (args.output / "run.json").write_text(json.dumps(run_record, indent=2) + "\n", encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--raw-scores", type=Path, help="merge previously computed full GPTTrace logits")
    parser.add_argument("--archive", type=Path, help="local checksum-pinned Arena ZIP")
    parser.add_argument("--model", type=Path, help="local snapshot with config.json and onnx/model_fp32.onnx")
    parser.add_argument("--coreml", action="store_true", help="enable Core ML on supported macOS builds")
    parser.add_argument("--jobs", type=int, default=3)
    parser.add_argument("--threads", type=int, default=8, help="ONNX Runtime intra-op CPU threads")
    args = parser.parse_args()
    if args.jobs < 1 or args.threads < 1:
        parser.error("--jobs and --threads must be positive")
    args.output.mkdir(parents=True, exist_ok=True)
    raw = args.raw_scores
    if raw is None:
        with TemporaryDirectory(prefix="tintype-arena-gpttrace-") as directory:
            run(args, Path(directory))
        raw = args.output / "gpttrace.tsv"
    merge_scores(raw, args.output / "scores.tsv.gz")


if __name__ == "__main__":
    main()
