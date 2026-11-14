#!/usr/bin/env python3
"""Report the real-source video audit from score files and a grouped manifest.

Score files are what `tintype score-video` writes (`path label generator
dataset prob frame-logits…`, no header), or an archived table with a header:
one score column per detector (`docs/results/confirm4-scores.tsv.gz`) or
one row per detector with its frame logits
(`docs/results/davodi-2026-video/frame-scores.tsv.gz`). The grouped manifest
(columns 5 to 9: parent_id, scene_id, source_id, generator_family,
transform) says which clips are variants of one original, which collection
each came from and how it was processed; `scripts/video_audit.py` writes
them. Without a manifest the source is the generator name of an authentic
clip and the dataset of a generated one, as in `src/split.rs::source_of`.

Printed, per detector, at the cutoff given (the shipped one, 0.5, by
default): flags per authentic collection with a one-sided 95%
Clopper-Pearson upper bound, by clip and by source video; recall per
generator; recall at fixed empirical false-positive rates where the
authentic clips allow one; frames scored before the early exit, by outcome;
paired shifts of every variant against its baseline; the sham-mark effect
against its placebos; score separation between authentic collections; and
matched-scene ranking. `--compare BASE,CANDIDATE` counts the cohorts where a
second detector or route adds detections at the first one's false-positive
count.

Every cutoff other than `--cutoff` is chosen on the clips it is reported
on: those lines are exploratory. Standard library only; imports
`scripts/video_audit.py`.
"""

import argparse
import gzip
import json
import math
import os
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import video_audit as va  # noqa: E402

BASELINE_ORDER = ("original", "trim-copy", "fixed-gop", "davodi-norm", "none")


def read_lines(path):
    path = Path(path)
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt", newline="") as stream:
        return [line.rstrip("\n") for line in stream if line.strip()]


def parse_scores(lines, name):
    """`{detector: [(path, label, generator, dataset, score, frame logits or None)]}`
    from any of the three layouts in the module docstring."""
    systems = defaultdict(list)
    header = lines[0].split("\t")
    if header[0] != "path":
        for line in lines:
            path, label, generator, dataset, score, *frames = line.split("\t")
            systems[name].append((path, int(label), generator, dataset, float(score), [float(f) for f in frames] or None))
        return dict(systems)
    for line in lines[1:]:
        row = dict(zip(header, line.split("\t")))
        base = (row["path"], int(row["label"]), row["generator"], row["dataset"])
        if "detector" in row:
            frames = [float(f) for f in row["frame_logits"].split(",")] if row.get("frame_logits") else None
            systems[row["detector"]].append((*base, float(row["score"]), frames))
        else:
            for column in header[4:]:
                systems[column].append((*base, float(row[column]), None))
    return dict(systems)


def tail(path, n):
    return "/".join(Path(path).parts[-n:])


def join(scored, manifest_rows, parents=None):
    """One record per scored clip with its grouping columns. A scored path
    matches a manifest row by full path, or else by its last three, two or
    one components where that is unambiguous in the manifest."""
    parents = parents or {}
    exact = {os.path.normpath(os.path.abspath(row["path"])): row for row in manifest_rows}
    tails = []
    for n in (3, 2, 1):
        seen = Counter(tail(row["path"], n) for row in manifest_rows)
        tails.append((n, {tail(row["path"], n): row for row in manifest_rows if seen[tail(row["path"], n)] == 1}))
    records, unmatched = [], 0
    for path, label, generator, dataset, score, frames in scored:
        row = exact.get(os.path.normpath(os.path.abspath(path)))
        for n, table in tails:
            row = row or table.get(tail(path, n))
        if row is None:
            unmatched += 1
            row = {"label": label, "generator": generator, "dataset": dataset, "rel": path, "parent_id": "", "scene_id": "", "source_id": "", "generator_family": "", "transform": ""}
        elif row["label"] != label:
            raise ValueError(f"{path} is labelled {label} in the scores and {row['label']} in the manifest")
        parent = va.parent_of(row)
        info = parents.get(parent, {})
        source = va.source_of(row)
        records.append({
            "path": path, "label": label, "score": score, "frames": frames, "generator": row["generator"], "parent": parent, "scene": row["scene_id"],
            "source": source[5:] if source.startswith("real:") else source, "family": row["generator_family"], "transform": row["transform"] or "none",
            "type": info.get("collection_type", ""), "video": info.get("source_video") or parent, "start_frame": info.get("start_frame", ""),
        })
    return records, unmatched


def percent(k, n):
    return "n/a" if n == 0 else f"{100 * k / n:.2f}%"


def by_transform(records):
    groups = defaultdict(list)
    for r in records:
        groups[r["transform"]].append(r)
    return dict(sorted(groups.items()))


def collection_table(records, cutoff):
    """Authentic flags per transform and collection, counted by clip and by
    source video (a video is flagged when any of its clips is)."""
    groups = defaultdict(list)
    for r in records:
        if r["label"] == 0:
            groups[(r["transform"], r["source"])].append(r)
    rows, clustered = [], False
    for (transform, source), items in sorted(groups.items()):
        flags = sum(r["score"] > cutoff for r in items)
        videos = defaultdict(bool)
        for r in items:
            videos[r["video"]] |= r["score"] > cutoff
        clustered |= len(videos) < len(items)
        types = sorted({r["type"] for r in items if r["type"]})
        rows.append((transform, source, "/".join(types) or "-", flags, len(items), va.clopper_pearson_upper(flags, len(items)), sum(videos.values()), len(videos),
                     va.clopper_pearson_upper(sum(videos.values()), len(videos))))
    return rows, clustered


def low_fpr_lines(records, fprs):
    """Recall at each empirical false-positive rate, or why it is not shown."""
    out = []
    real = [r["score"] for r in records if r["label"] == 0]
    fake = [r["score"] for r in records if r["label"] == 1]
    for fpr in fprs:
        name = f"{100 * fpr:g}% FPR"
        if not real or not fake:
            out.append(f"{name}: not reported: one class only")
            continue
        cut, allowed = va.empirical_cutoff(real, fpr)
        if allowed < 1:
            need = math.ceil(1 / fpr)
            out.append(f"{name}: not reported: {len(real)} authentic clips allow no false alarm at this rate (each one is {100 / len(real):.2f} points); "
                       f"{need:,} are needed to allow one")
            continue
        fp, tp = sum(s > cut for s in real), sum(s > cut for s in fake)
        out.append(f"{name}: {tp}/{len(fake)} generated flagged ({percent(tp, len(fake))}) with {fp}/{len(real)} authentic flagged (score > {cut:.6f}, {allowed} allowed)")
    return out


def frames_table(records, cutoff):
    """`{outcome: Counter(frames scored)}` for records that have frame logits."""
    names = {(1, False): "generated, missed", (1, True): "generated, flagged", (0, True): "authentic, flagged", (0, False): "authentic, not flagged"}
    table = {name: Counter() for name in names.values()}
    for r in records:
        if r["frames"]:
            table[names[(r["label"], r["score"] > cutoff)]][len(r["frames"])] += 1
    return table


def frame_coverage(records, cutoff, raw_cutoff):
    """How the errors look frame by frame: of the missed generated clips,
    how many had any frame, and at least a quarter of their frames, above
    the raw cutoff; of the flagged authentic clips, how many had any frame
    below it."""
    misses = [r for r in records if r["frames"] and r["label"] == 1 and r["score"] <= cutoff]
    alarms = [r for r in records if r["frames"] and r["label"] == 0 and r["score"] > cutoff]
    above = lambda r: sum(f > raw_cutoff for f in r["frames"])
    return (len(misses), sum(above(r) >= 1 for r in misses), sum(4 * above(r) >= len(r["frames"]) for r in misses),
            len(alarms), sum(above(r) < len(r["frames"]) for r in alarms))


def baseline_of(transform, present, default, recorded):
    """The transform `transform` is paired against, or None for a baseline
    itself: what the preparation recorded, else the watermark convention,
    else `default`, else the least processed transform present."""
    if recorded.get(transform):
        return recorded[transform] if recorded[transform] in present else None
    if transform in recorded:
        return None
    marked = va.watermark_baseline(transform)
    if marked:
        return marked if marked in present else None
    if transform == "wm-none":
        return None
    base = default if default in present else next((b for b in BASELINE_ORDER if b in present), None)
    return base if base and base != transform else None


def paired_shift(before, after, cutoff, rounds, seed):
    """Paired comparison of two lists of records aligned by parent."""
    flagged_before = sum(b["score"] > cutoff for b in before)
    flagged_after = sum(a["score"] > cutoff for a in after)
    gained = sum(a["score"] > cutoff >= b["score"] for b, a in zip(before, after))
    lost = sum(b["score"] > cutoff >= a["score"] for b, a in zip(before, after))
    shifts = [va.logit(a["score"]) - va.logit(b["score"]) for b, a in zip(before, after)]
    mean = lambda values: sum(values) / len(values) if values else float("nan")
    low, high = va.bootstrap_clustered(shifts, [b["video"] for b in before], mean, rounds, seed)
    return {"pairs": len(before), "before": flagged_before, "after": flagged_after, "gained": gained, "lost": lost, "p": va.sign_test_p(gained, lost), "shift": mean(shifts), "low": low, "high": high}


def paired_rows(records, cutoff, default, recorded, rounds, seed):
    groups = by_transform(records)
    index = {t: {r["parent"]: r for r in items} for t, items in groups.items()}
    rows = []
    for transform in groups:
        base = baseline_of(transform, groups, default, recorded)
        if not base:
            continue
        for label, name in ((0, "authentic"), (1, "generated")):
            parents = [p for p, r in index[transform].items() if r["label"] == label and p in index[base]]
            if parents:
                rows.append((transform, base, name, paired_shift([index[base][p] for p in parents], [index[transform][p] for p in parents], cutoff, rounds, seed)))
    return rows


def mark_versus_placebo(records, rounds, seed):
    """For each sham-mark position and class: the mean logit difference
    between the marked clip and each placebo of the same parent. Both alter
    the same region of the same frames; what is left is the mark itself."""
    groups = by_transform(records)
    index = {t: {r["parent"]: r for r in items} for t, items in groups.items()}
    rows = []
    for transform in groups:
        if not transform.startswith("wm-mark-") or transform.endswith(("-crop", "-fill")):
            continue
        position = transform[len("wm-mark-"):]
        for label, name in ((0, "authentic"), (1, "generated")):
            for placebo in (f"wm-fill-{position}", f"wm-crop-{position}"):
                if placebo not in index:
                    continue
                parents = [p for p, r in index[transform].items() if r["label"] == label and p in index[placebo]]
                if not parents:
                    continue
                diffs = [va.logit(index[transform][p]["score"]) - va.logit(index[placebo][p]["score"]) for p in parents]
                mean = lambda values: sum(values) / len(values)
                low, high = va.bootstrap_clustered(diffs, [index[transform][p]["video"] for p in parents], mean, rounds, seed)
                rows.append((position, name, placebo, len(parents), mean(diffs), low, high))
    return rows


def real_vs_real(records, minimum, rounds, seed):
    """AUROC of the scores of one authentic collection against another,
    within a transform; the interval resamples source videos."""
    rows = []
    for transform, items in by_transform(records).items():
        sources = defaultdict(list)
        for r in items:
            if r["label"] == 0:
                sources[r["source"]].append(r)
        names = sorted(s for s, clips in sources.items() if len(clips) >= minimum)
        for i, a in enumerate(names):
            for b in names[i + 1:]:
                pooled = [(r["score"], 1) for r in sources[a]] + [(r["score"], 0) for r in sources[b]]
                stat = lambda sample: va.auroc([s for s, y in sample if y], [s for s, y in sample if not y])
                low, high = va.bootstrap_clustered(pooled, [r["video"] for r in sources[a]] + [r["video"] for r in sources[b]], stat, rounds, seed)
                rows.append((transform, a, b, len(sources[a]), len(sources[b]), stat(pooled), low, high))
    return rows


def matched_scenes(records):
    """Per transform: generated clips scoring above, level with and below
    the authentic clip of their scene, and how many of those pairs start
    from a frame of that authentic clip."""
    rows = []
    for transform, items in by_transform(records).items():
        scenes = defaultdict(lambda: ([], []))
        for r in items:
            if r["scene"]:
                scenes[r["scene"]][r["label"]].append(r)
        wins = ties = losses = started = 0
        for real, fake in scenes.values():
            for f in fake:
                for a in real:
                    wins += f["score"] > a["score"]
                    ties += f["score"] == a["score"]
                    losses += f["score"] < a["score"]
                    started += bool(f["start_frame"]) and f["start_frame"] == a["parent"]
        if wins + ties + losses:
            rows.append((transform, sum(1 for real, fake in scenes.values() if real and fake), wins, ties, losses, started))
    return rows


def at_allowance(records, allowed):
    """Generated and authentic flags when at most `allowed` authentic clips
    may be flagged: strictly above the (allowed+1)-th highest authentic score."""
    real = sorted((r["score"] for r in records if r["label"] == 0), reverse=True)
    cut = real[allowed] if allowed < len(real) else -math.inf
    return sum(r["score"] > cut for r in records if r["label"] == 1), sum(s > cut for s in real), cut


def compare_rows(base, candidate, cutoff):
    """Per transform, on the clips both scored: the candidate's detections
    at the base's false-positive count at `cutoff`."""
    rows = []
    theirs = {r["path"]: r for r in candidate}
    for transform, items in by_transform([r for r in base if r["path"] in theirs]).items():
        other = [theirs[r["path"]] for r in items]
        n_real, n_fake = sum(r["label"] == 0 for r in items), sum(r["label"] == 1 for r in items)
        if not n_real or not n_fake:
            continue
        allowed = sum(r["score"] > cutoff for r in items if r["label"] == 0)
        base_tp, base_fp, _ = at_allowance(items, allowed)
        cand_tp, cand_fp, _ = at_allowance(other, allowed)
        rows.append((transform, n_real, n_fake, allowed, base_tp, base_fp, cand_tp, cand_fp))
    return rows


def report(name, records, args, recorded, raw_cutoff):
    cutoff = args.cutoff
    real, fake = [r for r in records if r["label"] == 0], [r for r in records if r["label"] == 1]
    real_flags, fake_flags = sum(r["score"] > cutoff for r in real), sum(r["score"] > cutoff for r in fake)
    print(f"\n## {name}\n")
    print(f"{len(records)} clips: {fake_flags}/{len(fake)} generated flagged, {real_flags}/{len(real)} authentic flagged at score > {cutoff:g}"
          + (f" (95% upper bound on the false-positive rate {percent(va.clopper_pearson_upper(real_flags, len(real)), 1)}, counting clips as independent)" if real else ""))

    rows, clustered = collection_table(records, cutoff)
    if rows:
        print("\n### Authentic collections\n")
        print("| Transform | Collection | Type | Flagged | FPR | 95% lower bound | 95% upper bound | Source videos flagged | 95% upper bound (videos) |")
        print("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
        for transform, source, kind, flags, n, upper, video_flags, videos, video_upper in rows:
            print(f"| {transform} | {source} | {kind} | {flags}/{n} | {percent(flags, n)} | {percent(va.clopper_pearson_lower(flags, n), 1)} | {percent(upper, 1)} | {video_flags}/{videos} | {percent(video_upper, 1)} |")
        print("\nOne-sided Clopper–Pearson bounds, each at 95%. " + ("Some clips share a source video: their flags are not independent, so read the bound by video, where a video counts as flagged when any of its clips is."
              if clustered else "No two clips of a row share a source video on record, so the two bounds agree."))

    groups = defaultdict(list)
    for r in fake:
        groups[(r["transform"], r["generator"])].append(r)
    if groups:
        print("\n### Generators\n")
        print("| Transform | Generator | Flagged | Recall |")
        print("| --- | --- | ---: | ---: |")
        for (transform, generator), items in sorted(groups.items()):
            flags = sum(r["score"] > cutoff for r in items)
            print(f"| {transform} | {generator} | {flags}/{len(items)} | {percent(flags, len(items))} |")

    print("\n### Recall at fixed empirical false-positive rates\n")
    print("The cutoff of each line is chosen on the clips of that line: exploratory.\n")
    said = defaultdict(list)
    for transform, items in by_transform(records).items():
        for line in low_fpr_lines(items, args.fpr):
            said[line].append(transform)
    for line, transforms in said.items():
        print(f"- {', '.join(transforms)}: {line}")

    table = frames_table(records, cutoff)
    if any(table.values()):
        counts = sorted({n for counter in table.values() for n in counter})
        print("\n### Frames scored before the early exit\n")
        print("| Outcome | Clips | " + " | ".join(f"{n} frames" for n in counts) + " | Mean |")
        print("| --- | ---: | " + " | ".join("---:" for _ in counts) + " | ---: |")
        for outcome, counter in table.items():
            total = sum(counter.values())
            mean = f"{sum(n * k for n, k in counter.items()) / total:.1f}" if total else "n/a"
            print(f"| {outcome} | {total} | " + " | ".join(str(counter[n]) for n in counts) + f" | {mean} |")
        if raw_cutoff is not None:
            misses, any_above, quarter_above, alarms, any_below = frame_coverage(records, cutoff, raw_cutoff)
            print(f"\nFrame logits against the raw cutoff {raw_cutoff:.3f}: of {misses} missed generated clips, {any_above} had at least one frame above it and {quarter_above} at least a quarter of their frames; "
                  f"of {alarms} flagged authentic clips, {any_below} had at least one frame below it.")

    rows = paired_rows(records, cutoff, args.baseline, recorded, args.rounds, args.seed)
    if rows:
        print("\n### Paired shifts against the baseline variant\n")
        print("| Transform | Against | Class | Pairs | Flagged before | Flagged after | Newly flagged | No longer flagged | Sign test p | Mean logit shift (95% interval) |")
        print("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |")
        for transform, base, kind, s in rows:
            print(f"| {transform} | {base} | {kind} | {s['pairs']} | {s['before']} ({percent(s['before'], s['pairs'])}) | {s['after']} ({percent(s['after'], s['pairs'])}) | {s['gained']} | {s['lost']} | {'<0.001' if s['p'] < 0.001 else format(s['p'], '.3f')} | "
                  f"{s['shift']:+.2f} ({s['low']:+.2f} to {s['high']:+.2f}) |")
        print("\nPairs are a variant and its baseline made from the same original (`parent_id`). For authentic clips `flagged` is the false-positive count, for generated clips the detections. "
              "The logit is that of the clip score; the interval resamples source videos.")

    rows = mark_versus_placebo(records, args.rounds, args.seed)
    if rows:
        print("\n### Sham mark against its placebos\n")
        print("| Position | Class | Placebo | Pairs | Mean logit, marked minus placebo (95% interval) |")
        print("| --- | --- | --- | ---: | --- |")
        for position, kind, placebo, pairs, mean, low, high in rows:
            print(f"| {position} | {kind} | {placebo} | {pairs} | {mean:+.2f} ({low:+.2f} to {high:+.2f}) |")
        print("\nThe mark and the placebo alter the same region of the same frames with the same encode. An interval that excludes zero says the mark itself moves the score: "
              "a learned shortcut if it is large against the paired shifts above. An interval around zero with a shift in both says the alteration moves it, not the mark.")

    rows = real_vs_real(records, args.min_collection, args.rounds, args.seed)
    if rows:
        print("\n### Score separation between authentic collections\n")
        print("| Transform | Collection A | Collection B | Clips A | Clips B | AUROC, A above B (95% interval) |")
        print("| --- | --- | --- | ---: | ---: | --- |")
        for transform, a, b, n_a, n_b, value, low, high in rows:
            print(f"| {transform} | {a} | {b} | {n_a} | {n_b} | {value:.3f} ({low:.3f}–{high:.3f}) |")
        print("\nBoth collections are authentic: 0.5 means the detector does not tell them apart. An interval far from 0.5 means it separates sources, not generation.")

    rows = matched_scenes(records)
    if rows:
        print("\n### Matched scenes\n")
        print("| Transform | Scenes with both classes | Generated above authentic | Ties | Below | Pairs with a matched starting frame |")
        print("| --- | ---: | ---: | ---: | ---: | ---: |")
        for transform, scenes, wins, ties, losses, started in rows:
            print(f"| {transform} | {scenes} | {wins} | {ties} | {losses} | {started} |")


def self_test():
    raw = ["/x/original/a.mp4\t0\treal:s1\td\t0.200000\t-1.0\t-2.0", "/x/original/b.mp4\t1\tgen\td\t0.900000\t3.0\t4.0\t5.0\t6.0", "/x/fixed-gop/a.mp4\t0\treal:s1\td\t0.700000\t1.0\t2.0",
           "/x/fixed-gop/b.mp4\t1\tgen\td\t0.400000\t1.0\t-3.0"]
    systems = parse_scores(raw, "tintype")
    assert list(systems) == ["tintype"] and systems["tintype"][1][4:] == (0.9, [3.0, 4.0, 5.0, 6.0])
    table = parse_scores(["path\tlabel\tgenerator\tdataset\ttintype\twaverep", "a.mp4\t0\tpixabay\td\t0.2\t0.6"], "ignored")
    assert table == {"tintype": [("a.mp4", 0, "pixabay", "d", 0.2, None)], "waverep": [("a.mp4", 0, "pixabay", "d", 0.6, None)]}
    rows = parse_scores(["path\tlabel\tgenerator\tdataset\tdetector\tscore\tframe_logits", "a.mp4\t1\tveo\td\ttintype\t0.9\t1.5,2.5"], "ignored")
    assert rows == {"tintype": [("a.mp4", 1, "veo", "d", 0.9, [1.5, 2.5])]}

    def row(rel, label, parent, transform, source):
        return {"rel": rel, "path": Path("/x") / rel, "label": label, "generator": "gen" if label else "real:s1", "dataset": "d", "parent_id": parent, "scene_id": "scene", "source_id": source,
                "generator_family": "", "transform": transform, "extra": []}
    manifest = [row("original/a.mp4", 0, "pa", "original", "s1"), row("original/b.mp4", 1, "pb", "original", "lab"), row("fixed-gop/a.mp4", 0, "pa", "fixed-gop", "s1"), row("fixed-gop/b.mp4", 1, "pb", "fixed-gop", "lab")]
    records, unmatched = join(systems["tintype"], manifest, {"pa": {"collection_type": "stock", "source_video": "v1", "start_frame": ""}, "pb": {"collection_type": "generated", "source_video": "", "start_frame": "pa"}})
    assert unmatched == 0 and [(r["parent"], r["transform"], r["source"], r["video"], r["type"]) for r in records[:2]] == [("pa", "original", "s1", "v1", "stock"), ("pb", "original", "lab", "pb", "generated")]
    # Paths that differ in their roots still join on their unambiguous tails; unknown clips fall back to the score columns.
    moved, missing = join([("other/root/fixed-gop/a.mp4", 0, "real:s1", "d", 0.7, None), ("elsewhere/c.mp4", 0, "real:s9", "d", 0.1, None)], manifest)
    assert missing == 1 and moved[0]["transform"] == "fixed-gop" and (moved[1]["source"], moved[1]["transform"], moved[1]["parent"]) == ("s9", "none", "elsewhere/c.mp4")
    try:
        join([("/x/original/a.mp4", 1, "g", "d", 0.5, None)], manifest)
        raise AssertionError("label conflict accepted")
    except ValueError:
        pass

    table, clustered = collection_table(records, 0.5)
    assert [(t, s, k, f, n, vf, v) for t, s, k, f, n, _, vf, v, _ in table] == [("fixed-gop", "s1", "stock", 1, 1, 1, 1), ("original", "s1", "stock", 0, 1, 0, 1)] and not clustered
    shared = [dict(records[0], score=s, parent=f"p{i}", video="one" if i < 3 else f"v{i}") for i, s in enumerate((0.9, 0.8, 0.1, 0.2, 0.3))]
    (_, _, _, flags, n, upper, video_flags, videos, video_upper), = collection_table(shared, 0.5)[0]
    assert (flags, n, video_flags, videos) == (2, 5, 1, 3) and collection_table(shared, 0.5)[1] and abs(upper - va.clopper_pearson_upper(2, 5)) < 1e-12 and video_upper == va.clopper_pearson_upper(1, 3)

    frames = frames_table(records, 0.5)
    assert frames["generated, missed"] == Counter({2: 1}) and frames["generated, flagged"] == Counter({4: 1}) and frames["authentic, flagged"] == Counter({2: 1})
    assert frame_coverage(records, 0.5, 0.0) == (1, 1, 1, 1, 0) and frame_coverage(records, 0.5, 1.5) == (1, 0, 0, 1, 1)

    present = {"original", "fixed-gop", "repost-g1", "wm-none", "wm-mark-br", "wm-mark-br-fill", "wm-fill-br"}
    assert baseline_of("fixed-gop", present, "original", {}) == "original" and baseline_of("original", present, "original", {}) is None
    assert baseline_of("wm-mark-br", present, "original", {}) == "wm-none" and baseline_of("wm-mark-br-fill", present, "original", {}) == "wm-mark-br" and baseline_of("wm-none", present, "original", {}) is None
    assert baseline_of("repost-g1", {"fixed-gop", "repost-g1"}, "original", {}) == "fixed-gop" and baseline_of("a+b", {"a", "a+b"}, "original", {"a+b": "a", "a": ""}) == "a"
    assert baseline_of("a", {"a", "a+b", "original"}, "original", {"a+b": "a", "a": ""}) is None
    (t1, b1, k1, s1), (t2, b2, k2, s2) = paired_rows(records, 0.5, "original", {}, 50, 0)
    assert (t1, b1, k1, s1["pairs"], s1["before"], s1["after"], s1["gained"], s1["lost"]) == ("fixed-gop", "original", "authentic", 1, 0, 1, 1, 0)
    assert (k2, s2["before"], s2["after"], s2["gained"], s2["lost"]) == ("generated", 1, 0, 0, 1) and abs(s2["shift"] - (va.logit(0.4) - va.logit(0.9))) < 1e-9

    marks = [dict(records[0], transform=t, score=s) for t, s in (("wm-none", 0.2), ("wm-mark-br", 0.6), ("wm-fill-br", 0.3), ("wm-crop-br", 0.6))]
    (p1, c1, placebo1, n1, mean1, _, _), (_, _, placebo2, _, mean2, _, _) = mark_versus_placebo(marks, 20, 0)
    assert (p1, c1, placebo1, n1, placebo2) == ("br", "authentic", "wm-fill-br", 1, "wm-crop-br") and abs(mean1 - (va.logit(0.6) - va.logit(0.3))) < 1e-9 and mean2 == 0

    two = [dict(records[0], source="a", score=s, parent=f"a{i}", video=f"a{i}") for i, s in enumerate((0.8, 0.9, 0.7))] + [dict(records[0], source="b", score=s, parent=f"b{i}", video=f"b{i}") for i, s in enumerate((0.1, 0.2, 0.3))]
    (_, a, b, n_a, n_b, value, low, high), = real_vs_real(two, 3, 50, 0)
    assert (a, b, n_a, n_b, value, low, high) == ("a", "b", 3, 3, 1.0, 1.0, 1.0) and real_vs_real(two, 4, 50, 0) == []
    assert matched_scenes(records) == [("fixed-gop", 1, 0, 0, 1, 1), ("original", 1, 1, 0, 0, 1)]

    lines = low_fpr_lines([dict(records[0], score=i / 200) for i in range(100)] + [dict(records[1], score=s) for s in (0.1, 0.497, 0.9)], [0.001, 0.01])
    assert "not reported: 100 authentic clips allow no false alarm at this rate (each one is 1.00 points); 1,000 are needed" in lines[0] and "2/3 generated flagged" in lines[1] and "1/100 authentic flagged" in lines[1]
    cohort = [dict(records[0], score=s, path=f"r{i}") for i, s in enumerate((0.9, 0.6, 0.2, 0.1))] + [dict(records[1], score=s, path=f"f{i}") for i, s in enumerate((0.95, 0.7, 0.3))]
    second = [dict(r, score={"r0": 0.9, "r1": 0.3, "r2": 0.2, "r3": 0.1, "f0": 0.95, "f1": 0.7, "f2": 0.25}[r["path"]]) for r in cohort]
    assert at_allowance(cohort, 2) == (3, 2, 0.2) and at_allowance(cohort, 9)[:2] == (3, 4)
    assert compare_rows(cohort, second, 0.5) == [("original", 4, 3, 2, 3, 2, 3, 2)] and compare_rows(cohort, second, 0.8) == [("original", 4, 3, 1, 2, 1, 2, 1)]
    print("report-video-audit self-test passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--scores", action="append", default=[], metavar="[NAME=]FILE", help="score file (repeatable); NAME names the detector of a headerless score-video file, and files that share a name are joined")
    parser.add_argument("--manifest", type=Path, action="append", default=[], help="grouped manifest of the scored clips (repeatable)")
    parser.add_argument("--sidecar", type=Path, action="append", default=[], help="audit.tsv written by the preparation: collection types and source videos, or variant baselines (repeatable)")
    parser.add_argument("--spec", type=Path, help="model spec JSON of the scored video weights, for the raw frame cutoff")
    parser.add_argument("--cutoff", type=float, default=0.5, help="flag scores above this (default: the shipped cutoff)")
    parser.add_argument("--baseline", default="original", help="transform the variants are paired against")
    parser.add_argument("--fpr", type=lambda s: [float(v) for v in s.split(",")], default=[0.001, 0.01], help="empirical false-positive rates to operate at")
    parser.add_argument("--min-collection", type=int, default=10, help="fewest clips of an authentic collection in the real-against-real table")
    parser.add_argument("--compare", metavar="BASE,CANDIDATE", help="count the cohorts where CANDIDATE adds detections at BASE's false-positive count")
    parser.add_argument("--system", action="append", default=[], help="report only these detectors (repeatable)")
    parser.add_argument("--rounds", type=int, default=1000, help="bootstrap rounds")
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--self-test", action="store_true", help="check the pure functions and exit")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if not args.scores:
        parser.error("give at least one --scores file")

    manifest_rows = [row for manifest in args.manifest for row in va.read_manifest(manifest)]
    parents, recorded = {}, {}
    for sidecar in args.sidecar:
        for row in va.read_table(sidecar):
            if "collection_type" in row:
                parents[row["parent_id"]] = row
            if "baseline" in row:
                recorded[row["transform"]] = row["baseline"]
    raw_cutoff = None
    if args.spec:
        video = json.loads(args.spec.read_text())["video"]
        raw_cutoff = (va.logit(args.cutoff) - video["logit_bias"]) / video["logit_scale"]

    scored = defaultdict(list)
    for spec in args.scores:
        name, _, path = spec.rpartition("=")
        # Several files under one name are one detector's scores in parts, e.g. one file per variant.
        for system, rows in parse_scores(read_lines(path), name or Path(path).name.split(".")[0]).items():
            scored[system].extend(rows)
    systems = {}
    for system, rows in scored.items():
        repeated = [path for path, n in Counter(row[0] for row in rows).items() if n > 1]
        if repeated:
            raise SystemExit(f"{system} scores {repeated[0]} more than once; name the detectors apart with NAME=FILE")
        systems[system], unmatched = join(rows, manifest_rows, parents)
        if unmatched and manifest_rows:
            print(f"note: {unmatched} of {len(rows)} clips scored by {system} are not in the manifests; they are grouped by their score-file columns", file=sys.stderr)
    wanted = args.system or list(systems)
    if set(wanted) - set(systems) or (args.compare and set(args.compare.split(",")) - set(systems)):
        raise SystemExit(f"unknown detector; the score files hold {', '.join(systems)}")

    print("# Video audit report")
    print(f"\nScores: {', '.join(args.scores)}. Manifests: {', '.join(str(m) for m in args.manifest) or 'none (grouped by score-file columns)'}.")
    for name in wanted:
        report(name, systems[name], args, recorded, raw_cutoff)

    if args.compare:
        base, candidate = args.compare.split(",")
        rows = compare_rows(systems[base], systems[candidate], args.cutoff)
        print(f"\n## {candidate} against {base} at equal false-positive counts\n")
        print(f"| Transform | Authentic | Generated | Allowed false positives | {base} detections | {candidate} detections | Gain |")
        print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
        for transform, n_real, n_fake, allowed, base_tp, base_fp, cand_tp, cand_fp in rows:
            print(f"| {transform} | {n_real} | {n_fake} | {allowed} | {base_tp} (at {base_fp} FP) | {cand_tp} (at {cand_fp} FP) | {cand_tp - base_tp:+d} |")
        gains = sum(cand_tp > base_tp for *_, base_tp, _, cand_tp, _ in rows)
        print(f"\nThe allowance of each row is {base}'s false-positive count at score > {args.cutoff:g} on that row's clips; both cutoffs are then chosen on those clips (exploratory). "
              f"{candidate} adds detections on {gains} of {len(rows)} cohorts listed. The proposed gate asks for gains on at least two controlled cohorts; "
              "whether a row is a controlled cohort (held-out authentic sources, matched classes, a common encode) is for the runbook to say, not this table.")


if __name__ == "__main__":
    main()
