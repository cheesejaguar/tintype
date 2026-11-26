#!/usr/bin/env python3
"""Check Tintype's pure-Rust VP9/VP8 reader against FFmpeg's decoders, then
score lossless H.264 controls made from the same decoded frames.

Usage: parity.py REPO TINTYPE CLIPS_DIR WORK_DIR OUT.json

REPO is a Tintype checkout (its ignored `swdec::reference` test writes the
reader's per-frame digests), TINTYPE a release binary built from it,
CLIPS_DIR the clips to check and WORK_DIR an empty scratch directory. Needs
FFmpeg and ffprobe with libvpx and libx264. Standard library only.

1. Pixels and times. Every frame the reader returns is hashed plane by plane
   (Y, Cb, Cr, 8-bit planar) and compared with FFmpeg's output from libvpx
   and from FFmpeg's own decoder; its time is compared with the frame's
   presentation time from ffprobe. A 10-bit clip is compared after reducing
   FFmpeg's 16-bit samples to 8 bits with the reader's rounding.
2. Controls. Each 8-bit clip of even size is decoded by libvpx and re-encoded as lossless
   H.264 (`-qp 0`, no B-frames) with the source's timestamps and colour
   tags. The control is checked to decode to the same planes and times as
   the source, then both are scored by `tintype detect --json`, once as by
   default and once with `--coverage uniform`, which reports every sampled
   frame's logit. One control exists at a time and is deleted after scoring.
"""
import fractions
import hashlib
import json
import os
import pathlib
import platform
import subprocess
import sys


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, capture_output=True, **kw)


def stream(path):
    out = run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
               "stream=codec_name,width,height,pix_fmt,time_base,color_space,color_range,color_primaries,color_transfer",
               "-of", "json", str(path)]).stdout
    return json.loads(out)["streams"][0]


def frame_times(path):
    """Presentation times (exact fractions of a second) of the decoded frames."""
    info = stream(path)
    tb = fractions.Fraction(info["time_base"])
    out = run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "frame=pts",
               "-of", "json", str(path)]).stdout
    return [fractions.Fraction(f["pts"]) * tb for f in json.loads(out)["frames"]]


def plane_digests(path, decoder):
    """SHA-256 of each frame's Y, Cb and Cr planes as `decoder` outputs them, 8-bit."""
    info = stream(path)
    w, h = info["width"], info["height"]
    cw, ch = (w + 1) // 2, (h + 1) // 2
    # FFmpeg's H.264 decoder calls full-range 4:2:0 `yuvj420p`: the same layout.
    bits = {"yuv420p": 8, "yuvj420p": 8, "yuv420p10le": 10, "yuv420p12le": 12}[info["pix_fmt"]]
    deep = bits > 8
    # The decoder's own sample format, so nothing is converted on the way.
    fmt = info["pix_fmt"]
    size = (w * h + 2 * cw * ch) * (2 if deep else 1)
    cmd = ["ffmpeg", "-v", "error", "-threads", "1"] + (["-c:v", decoder] if decoder else []) + [
        "-i", str(path), "-map", "0:v:0", "-fps_mode", "passthrough", "-f", "rawvideo", "-pix_fmt", fmt, "-"]
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE)
    shift = bits - 8
    out = []
    while True:
        buf = proc.stdout.read(size)
        if not buf:
            break
        assert len(buf) == size, f"short frame from {decoder} on {path}"
        if deep:
            # Round to 8 bits as the reader does.
            buf = bytes(min(255, (s + (1 << (shift - 1))) >> shift) for s in memoryview(buf).cast("H"))
        y, u, v = buf[: w * h], buf[w * h : w * h + cw * ch], buf[w * h + cw * ch :]
        out.append(tuple(hashlib.sha256(p).hexdigest() for p in (y, u, v)))
    assert proc.wait() == 0, f"{decoder} failed on {path}"
    return out


def reader_digests(repo, clips, work):
    listing = work / "clips.txt"
    listing.write_text("".join(f"{c}\n" for c in clips))
    digests = work / "digests.tsv"
    env = dict(os.environ, TINTYPE_TEST_VIDEOS=str(listing), TINTYPE_DIGESTS=str(digests))
    run(["cargo", "test", "--release", "--lib", "swdec::reference", "--", "--ignored"], cwd=repo, env=env)
    rows = {}
    for line in digests.read_text().splitlines():
        path, index, pts, w, h, y, u, v, bt709, full = line.split("\t")
        rows.setdefault(path, []).append({"pts": float(pts), "size": (int(w), int(h)), "planes": (y, u, v), "tags": (bt709 == "true", full == "true")})
    return rows


def detect(tintype, paths, *flags, env=None):
    out = subprocess.run([tintype, "detect", "--json", *flags, *map(str, paths)], capture_output=True,
                         env=dict(os.environ, **(env or {})))
    rows = [json.loads(l) for l in out.stdout.decode().splitlines() if l.startswith("{")]
    return {r["path"]: r for r in rows}


SCORED = ("logit", "score", "verdict", "frames", "frames_expected", "complete", "analysed_until_s")


def main():
    repo, tintype, clips_dir, work, out_path = map(pathlib.Path, sys.argv[1:6])
    work.mkdir(parents=True, exist_ok=True)
    clips = sorted(p for p in clips_dir.iterdir() if p.suffix in (".mp4", ".webm", ".mkv"))
    ours = reader_digests(repo, clips, work)
    report = {
        "ffmpeg": run(["ffmpeg", "-hide_banner", "-version"]).stdout.decode().splitlines()[0],
        "platform": f"{platform.system()} {platform.release()} {platform.machine()}",
        "clips": [],
    }
    for clip in clips:
        info = stream(clip)
        vp9 = info["codec_name"] == "vp9"
        frames = ours[str(clip)]
        times = frame_times(clip)
        entry = {
            "clip": clip.name, "codec": info["codec_name"], "pix_fmt": info["pix_fmt"],
            "size": [info["width"], info["height"]], "bytes": clip.stat().st_size,
            "reader_frames": len(frames), "reader_tags_bt709_full_range": list(frames[0]["tags"]),
            "stream_colour": {k: info.get(k) for k in ("color_space", "color_range", "color_primaries")},
            # The reader's times are f64 seconds; ffprobe's are exact fractions of the stream time base.
            "max_time_difference_s": max(abs(float(t) - f["pts"]) for t, f in zip(times, frames)) if len(times) == len(frames) else None,
            "ffprobe_frames": len(times),
            "decoders": {},
        }
        for decoder in (["libvpx-vp9", "vp9"] if vp9 else ["libvpx", "vp8"]):
            ref = plane_digests(clip, decoder)
            same = sum(a == f["planes"] for a, f in zip(ref, frames))
            first_diff = next((i for i, (a, f) in enumerate(zip(ref, frames)) if a != f["planes"]), None)
            entry["decoders"][decoder] = {"frames": len(ref), "identical": same, "first_difference": first_diff}
        entry["pixels_and_times_match"] = all(d["frames"] == len(frames) == d["identical"] for d in entry["decoders"].values()) and entry["max_time_difference_s"] is not None and entry["max_time_difference_s"] < 1e-9
        if info["pix_fmt"] != "yuv420p":
            entry["control"] = "not made: the reader reduces deeper samples to 8 bits itself"
        elif info["width"] % 2 or info["height"] % 2:
            entry["control"] = "not made: x264 encodes 4:2:0 only at even dimensions"
        else:
            entry["control"] = control(tintype, clip, info, work, "libvpx-vp9" if vp9 else "libvpx")
        report["clips"].append(entry)
        print(json.dumps({k: entry[k] for k in ("clip", "reader_frames", "max_time_difference_s", "pixels_and_times_match")}), flush=True)
    # The pure-Rust path on macOS (TINTYPE_DECODER=rust) must score the clips exactly as the default path does.
    default = detect(tintype, clips)
    rust = detect(tintype, clips, env={"TINTYPE_DECODER": "rust"})
    report["rust_path_matches_default"] = {c.name: all(default[str(c)].get(k) == rust[str(c)].get(k) for k in SCORED) for c in clips}
    report["scores"] = {c.name: {k: default[str(c)].get(k) for k in SCORED + ("duration_s",)} for c in clips}
    out_path.write_text(json.dumps(report, indent=1) + "\n")


def control(tintype, clip, info, work, libvpx):
    """Lossless H.264 from libvpx's frames; checked, scored beside the clip, deleted."""
    ctrl = work / (clip.stem + ".h264-qp0.mp4")
    tags = []
    for flag, key in (("-colorspace", "color_space"), ("-color_range", "color_range"), ("-color_primaries", "color_primaries"), ("-color_trc", "color_transfer")):
        if info.get(key) not in (None, "unknown"):
            tags += [flag, info[key]]
    run(["ffmpeg", "-v", "error", "-y", "-threads", "1", "-c:v", libvpx, "-i", str(clip), "-map", "0:v:0", "-fps_mode", "passthrough",
         "-enc_time_base", "demux", "-c:v", "libx264", "-qp", "0", "-bf", "0", "-preset", "veryfast", "-pix_fmt", "yuv420p", *tags,
         "-map_metadata", "-1", str(ctrl)])
    try:
        same_pixels = plane_digests(ctrl, None) == plane_digests(clip, libvpx)
        same_times = frame_times(ctrl) == frame_times(clip)
        result = {"bytes": ctrl.stat().st_size, "decodes_to_source_planes": same_pixels, "same_times": same_times}
        for name, flags in (("default", ()), ("every_frame", ("--coverage", "uniform"))):
            rows = detect(tintype, [clip, ctrl], *flags)
            a, b = rows[str(clip)], rows[str(ctrl)]
            per_frame = lambda r, key: [s[key] for s in r.get("segments", [])]
            result[name] = {
                "identical": all(a.get(k) == b.get(k) for k in SCORED) and per_frame(a, "frame_logits") == per_frame(b, "frame_logits"),
                "fields_that_differ": [k for k in SCORED if a.get(k) != b.get(k)],
                "frame_logits_identical": per_frame(a, "frame_logits") == per_frame(b, "frame_logits"),
                "sampled_times_identical": per_frame(a, "sampled_s") == per_frame(b, "sampled_s"),
                "durations_s": [a.get("duration_s"), b.get("duration_s")],
                "vp9_logit": a.get("logit"), "control_logit": b.get("logit"),
                "verdicts": [a.get("verdict"), b.get("verdict")],
            }
        return result
    finally:
        ctrl.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
