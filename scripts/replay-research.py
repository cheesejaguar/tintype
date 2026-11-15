#!/usr/bin/env python3
"""Reproduce the research report's exploratory replay of archived scores."""

import argparse
import ast
import csv
import gzip
import hashlib
import json
import math
from bisect import bisect_left, bisect_right
from collections import Counter
from pathlib import Path


REPO = Path(__file__).resolve().parents[1]
ROOT = REPO / "docs" / "results"
NTIRE_LABEL_PINS = {
    "hard-validation": "4238972a07e48a2c03908d4566032609870eaf9cc4edfeb82667967cf4f77675",
    "public-test": "336fdc42968a0395e1e9a914f1a87399e9b9d49df0d54d90cc4892d43cefc12e",
}
NTIRE_ARTIFACT_OPS = {"jpeg_ai", "cheng2020", "wmforger"}


def read(path):
    with gzip.open(ROOT / path, "rt", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def logit(p):
    p = min(1 - 1e-7, max(1e-7, float(p)))
    return math.log(p / (1 - p))


def cutoff_at_budget(samples, budget):
    real = sorted((score for y, score in samples if y == 0), reverse=True)
    return real[budget]


def counts_at_cutoff(samples, cutoff):
    return Counter((y, score > cutoff) for y, score in samples)


def at_one_percent(samples):
    n_real = sum(y == 0 for y, _ in samples)
    counts = counts_at_cutoff(samples, cutoff_at_budget(samples, n_real // 100))
    return counts[1, True], len(samples) - n_real


def cf_fusion(rows, band):
    samples, routed = [], 0
    for row in rows:
        base, expert = float(row["tintype"]), float(row["commfor"])
        base_logit = logit(base)
        if band["lo"] < base_logit < band["hi"]:
            fused = 0.5 * (band["weight"] * base_logit + logit(expert) - band["threshold"])
            routed += 1
            score = 1 / (1 + math.exp(-fused))
        else:
            score = base
        samples.append((int(row["label"]), score))
    return samples, routed


def from_column(rows, column):
    return [(int(row["label"]), float(row[column])) for row in rows]


def joint_set_wins(rows, samples, cutoff):
    by_set = {}
    for row, (y, score) in zip(rows, samples):
        results = by_set.setdefault(row["dataset"], (Counter(), Counter()))
        results[0][y, score > cutoff] += 1
        results[1][y, float(row["commfor"]) > 0.5] += 1
    wins = 0
    for candidate, cf in by_set.values():
        tp, fp = candidate[1, True], candidate[0, True]
        cf_tp, cf_fp = cf[1, True], cf[0, True]
        empty = tp + fp == 0 or cf_tp + cf_fp == 0
        if tp >= cf_tp and not empty and tp * cf_fp >= cf_tp * fp:
            wins += 1
    return wins, len(by_set)


def replay_cf():
    band = json.loads((REPO / "models" / "tintype.json").read_text())["second_opinion"]
    narrow_band = dict(band, lo=-3)
    rows = read("confirm7-scores.tsv.gz")
    shipped, shipped_routes = cf_fusion(rows, band)
    narrow, narrow_routes = cf_fusion(rows, narrow_band)
    stored = from_column(rows, "tintype-second")
    mismatches = sum((a > 0.5) != (b > 0.5) for (_, a), (_, b) in zip(shipped, stored))
    counts = counts_at_cutoff(stored, 0.5)
    print(f"Confirmation 7 CF fusion reproduces shipped verdicts with {mismatches} mismatches.")
    assert mismatches == 0
    for name, samples, routed in (
        ("Tintype default", from_column(rows, "tintype"), 0),
        ("Tintype sensitive", from_column(rows, "tintype-sensitive"), 0),
        ("Shipped CF fusion", stored, shipped_routes),
        ("Narrow CF fusion", narrow, narrow_routes),
    ):
        print(f"  {name}: routed {routed}/{len(rows)} ({routed / len(rows):.2%})")
        for budget in (107, counts[0, True]):
            cutoff = cutoff_at_budget(samples, budget)
            at = counts_at_cutoff(samples, cutoff)
            wins, datasets = joint_set_wins(rows, samples, cutoff)
            print(f"    {at[1, True]}/{at[1, True] + at[1, False]} TPs at {at[0, True]}/{budget} allowed FPs; "
                  f"matched or beat CF precision and recall on {wins}/{datasets} sets")
        at = counts_at_cutoff(samples, 0.5)
        print(f"    at score > 0.5: {at[1, True]} TPs, {at[0, True]} FPs")

    rows = read("frontier-jpeg-stress-scores.tsv.gz")
    for cohort in ("original", "jpeg", "jpeg-960"):
        subset = [r for r in rows if r["path"].split("/", 1)[0] == cohort]
        default = from_column(subset, "tintype")
        narrow, routed = cf_fusion(subset, narrow_band)
        print(f"  Frontier {cohort}: default {at_one_percent(default)}, narrow {at_one_percent(narrow)} "
              f"(TP, generated at 1% empirical FPR); {routed / len(subset):.1%} routed")


def replay_ntire(labels, split):
    if hashlib.sha256(labels.read_bytes()).hexdigest() != NTIRE_LABEL_PINS[split]:
        raise ValueError(f"Unexpected hash for NTIRE {split} labels: {labels}")
    archive = {r["path"]: r for r in read(f"ntire-2026-{split}/scores.tsv.gz")}
    groups = {"all distorted": [], "with artifact operations": [], "without artifact operations": []}
    with labels.open(newline="") as f:
        for r in csv.DictReader(f):
            a = archive.pop(r["image_name"])
            distorted = r["is_distorted"] == "1"
            if a["label"] != r["label"] or a["dataset"] != ("ntire-distorted" if distorted else "ntire-clean"):
                raise ValueError(f"Score and label disagree for {r['image_name']}")
            if distorted:
                sample = (int(a["label"]), float(a["tintype"]))
                groups["all distorted"].append(sample)
                ops = set(ast.literal_eval(r["distortions"]))
                key = "with artifact operations" if ops & NTIRE_ARTIFACT_OPS else "without artifact operations"
                groups[key].append(sample)
    if archive:
        raise ValueError(f"Unmatched archive rows in NTIRE {split}")
    print(f"NTIRE {split}:")
    for group, samples in groups.items():
        real = sorted(score for y, score in samples if y == 0)
        ai = [score for y, score in samples if y == 1]
        wins = sum((bisect_left(real, z) + bisect_right(real, z)) / 2 for z in ai)
        tp, n = at_one_percent(samples)
        at_shipped = counts_at_cutoff(samples, 0.5)
        print(f"  {group}: {len(real)} real, {len(ai)} generated, AUC {wins / (len(real) * len(ai)):.4f}, "
              f"{tp}/{n} TPs at <=1% FPR, {at_shipped[0, True]}/{len(real)} FPs at score > 0.5")


def replay_arena():
    manifest = read("arena-v0.1-gpu/inputs.tsv.gz")
    assert len({row["id"] for row in manifest}) == len(manifest)
    scores = read("arena-v0.1-gpttrace/scores.tsv.gz")
    assert len(scores) == 2 * len(manifest)
    labels = {row["id"]: row["label"] for row in manifest}
    hashes, duplicates = {}, set()
    for row in manifest:
        digest = row["original_sha256"]
        if digest in hashes:
            assert row["label"] == hashes[digest]
            duplicates.add(row["id"])
        hashes[digest] = row["label"]

    for dataset in ("arena-original", "arena-jpeg-q4"):
        rows = [r for r in scores if r["dataset"] == dataset]
        assert {r["path"].split("/", 1)[1]: r["label"] for r in rows} == labels
        assert len(rows) == len(manifest)
        examples = {"Tintype": [], "GPTTrace": [], "50:50 full fusion": [], "50:50 fusion for |Tintype logit| <= 2": []}
        errors = Counter()
        gated = 0
        for r in rows:
            if r["path"].split("/", 1)[1] in duplicates:
                continue
            y, a, b = int(r["label"]), logit(r["tintype"]), logit(r["gpttrace"])
            fuse = (a + b) / 2
            use_second = abs(a) <= 2
            gated += use_second
            for v, z in zip(examples.values(), [a, b, fuse, fuse if use_second else a]):
                v.append((y, z))
            if (a > 0) != y:
                errors[(y, "Tintype errors")] += 1
                errors[(y, "outside band")] += not use_second
                errors[(y, "GPTTrace correct")] += (b > 0) == y
        total = len(next(iter(examples.values())))
        print(f"{dataset}: {total} unique originals; gated {gated} ({gated / total:.1%})")
        for name, d in examples.items():
            tp, n = at_one_percent(d)
            fp = sum(not y and z > 0 for y, z in d)
            print(f"  {name}: {tp}/{n} ({tp / n:.2%}) at 1% empirical FPR; {fp}/{total - n} FP at score > 0.5")
        for y in (0, 1):
            print(f"  label {y}: {errors[y, 'Tintype errors']} Tintype errors; "
                  f"{errors[y, 'outside band']} outside band; {errors[y, 'GPTTrace correct']} GPTTrace correct")


def replay_video():
    for p in ("confirm4-scores.tsv.gz", "davodi-2026-video/scores.tsv.gz"):
        rows = read(p)
        negatives = [r for r in rows if float(r["tintype"]) <= 0.5]
        misses = [r for r in negatives if r["label"] == "1"]
        reals = [r for r in negatives if r["label"] == "0"]
        print(f"{p}: WaveRep flags {sum(float(r['waverep']) > 0.5 for r in misses)}/{len(misses)} "
              f"Tintype misses and {sum(float(r['waverep']) > 0.5 for r in reals)}/{len(reals)} "
              "authentic clips Tintype did not flag")
    rows = read("confirm4-scores.tsv.gz")
    for name in ("tintype", "tintype-second"):
        samples = from_column(rows, name)
        cutoff = cutoff_at_budget(samples, 3)
        at = counts_at_cutoff(samples, cutoff)
        print(f"  Confirmation 4 {name}: {at[1, True]} TPs at {at[0, True]}/3 allowed FPs")
    rows = read("davodi-2026-video/frame-scores.tsv.gz")
    missed = Counter()
    false_alarms = Counter()
    for r in rows:
        if r["detector"] != "tintype":
            continue
        frames = len(r["frame_logits"].split(","))
        positive = float(r["score"]) > 0.5
        if r["label"] == "1" and not positive:
            missed[frames] += 1
        if r["label"] == "0" and positive:
            false_alarms[frames] += 1
    print(f"  DaVODi Tintype errors by frames scored: misses {dict(sorted(missed.items()))}; "
          f"false alarms {dict(sorted(false_alarms.items()))}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hard-labels", type=Path, help="Pinned NTIRE hard-validation labels CSV")
    parser.add_argument("--public-labels", type=Path, help="Pinned NTIRE public-test labels CSV")
    args = parser.parse_args()
    replay_cf()
    replay_arena()
    replay_video()
    if args.hard_labels:
        replay_ntire(args.hard_labels, "hard-validation")
    if args.public_labels:
        replay_ntire(args.public_labels, "public-test")
