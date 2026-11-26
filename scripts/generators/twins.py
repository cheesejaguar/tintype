#!/usr/bin/env python3
"""Real-photo twins for a new generator (docs/NEW_GENERATOR_PLAYBOOK.md, steps 2 and 3).

Subcommands, run in order, all writing under OUT (a folder on the data share):

  pool      list Wikimedia Commons Quality images in a date window, filter by
            licence and kind, assign each to the training portion or the blind
            set by the salted hash rule, write OUT/pool.jsonl
  fetch     download the 2,048 px renditions of the first N pool entries of a
            portion; record SHA-256, size and dHash in OUT/real.jsonl
  caption   caption fetched photos with a vision-language model through Bifrost
            (fixed instruction file); write OUT/captions.jsonl
  generate  one image per caption through Bifrost /v1/images/generations;
            write OUT/<slug>/*.png, OUT/meta.jsonl and OUT/manifest.tsv
  group     near-duplicate grouping (same author within a day, dHash distance <= 6);
            move every group that is not wholly blind-assigned to training

Every step is resumable: entries already done are skipped. Never run caption
or generate on the blind portion before the pilot review, and never score
blind files before the frozen manifest is committed.

Needs Pillow for dHash and image sizes (pass the interpreter that has it).
The Bifrost key comes from BIFROST_VK or ~/.bifrost_env and is never printed.
"""
import argparse
import base64
import hashlib
import io
import json
import os
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

API = "https://commons.wikimedia.org/w/api.php"
UA = "tintype-generator-twins/0.1 (research data collection; https://github.com/cheesejaguar/tintype)"
LICENCES = ("cc0", "public domain", "cc by ", "cc by-sa", "cc-by")
BAD_WORDS = ("illustration", "rendering", "drawing", "painting", "artwork", "scanned", "diagram", "map ",
             "computer-generated", "computer generated", "ai-generated", "ai generated", "cgi", "screenshot")
RATIOS = {"1:1": 1.0, "3:4": 0.75, "4:3": 4 / 3, "9:16": 9 / 16, "16:9": 16 / 9}


def bifrost_key():
    vk = os.environ.get("BIFROST_VK")
    if not vk:
        for line in (Path.home() / ".bifrost_env").read_text().splitlines():
            line = line.strip().removeprefix("export ").strip()
            if line.startswith("BIFROST_VK="):
                vk = line.split("=", 1)[1].strip().strip("'\"")
    if not vk:
        sys.exit("no Bifrost key")
    return vk


def http_json(url, data=None, headers=None, timeout=120):
    h = {"User-Agent": UA, **(headers or {})}
    body = None
    if data is not None:
        body = json.dumps(data).encode()
        h["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=body, headers=h)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, json.load(r)
    except urllib.error.HTTPError as e:
        try:
            return e.code, json.load(e)
        except Exception:
            return e.code, {"error": str(e)}


def read_jsonl(path):
    if not path.exists():
        return []
    return [json.loads(l) for l in path.read_text().splitlines() if l.strip()]


def append_jsonl(path, row):
    with path.open("a") as f:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")


def blind_assigned(salt, pageid, fraction):
    return int(hashlib.sha256(f"{salt}:{pageid}".encode()).hexdigest()[:8], 16) / 2**32 < fraction


def plain(html):
    out, skip = [], False
    for ch in html or "":
        if ch == "<":
            skip = True
        elif ch == ">":
            skip = False
        elif not skip:
            out.append(ch)
    return "".join(out).strip()


def cmd_pool(a):
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    pool_path = out / "pool.jsonl"
    seen = {r["pageid"] for r in read_jsonl(pool_path)}
    params = {
        "action": "query", "format": "json", "formatversion": "2",
        "generator": "categorymembers", "gcmtitle": a.category, "gcmtype": "file",
        "gcmsort": "timestamp", "gcmdir": "ascending", "gcmstart": a.start + "T00:00:00Z",
        "gcmend": a.end + "T23:59:59Z", "gcmlimit": "50",
        "prop": "imageinfo|categories", "cllimit": "max",
        "iiprop": "url|size|mime|sha1|extmetadata|timestamp", "iiurlwidth": str(a.width),
    }
    kept = len(seen)
    cont = {}
    while kept < a.want:
        status, d = http_json(API + "?" + urllib.parse.urlencode({**params, **cont}))
        if status != 200:
            sys.exit(f"Commons API {status}: {d}")
        pages = sorted(d.get("query", {}).get("pages", []), key=lambda p: p.get("index", 0))
        for p in pages:
            if p["pageid"] in seen or not p.get("imageinfo"):
                continue
            ii = p["imageinfo"][0]
            md = ii.get("extmetadata", {})
            lic = plain(md.get("LicenseShortName", {}).get("value", "")).lower()
            cats = " ".join(c["title"].lower() for c in p.get("categories", []))
            desc = plain(md.get("ImageDescription", {}).get("value", "")).lower()
            reason = None
            if ii.get("mime") != "image/jpeg":
                reason = "not jpeg"
            elif not any(x in lic for x in LICENCES) or "-nc" in lic or "-nd" in lic:
                reason = f"licence {lic!r}"
            elif any(w in cats or w in desc for w in BAD_WORDS):
                reason = "not a photograph"
            row = {
                "pageid": p["pageid"], "title": p["title"], "url": ii.get("thumburl") or ii["url"],
                "original_url": ii["url"], "width": ii.get("width"), "height": ii.get("height"),
                "commons_sha1": ii.get("sha1"), "licence": plain(md.get("LicenseShortName", {}).get("value", "")),
                "licence_url": md.get("LicenseUrl", {}).get("value", ""),
                "artist": plain(md.get("Artist", {}).get("value", "")),
                "date": plain(md.get("DateTimeOriginal", {}).get("value", "")),
                "split": "blind" if blind_assigned(a.salt, p["pageid"], a.fraction) else "train",
                "excluded": reason,
            }
            append_jsonl(pool_path, row)
            seen.add(p["pageid"])
            if not reason:
                kept += 1
                if kept >= a.want:
                    break
        if "continue" not in d:
            break
        cont = d["continue"]
        time.sleep(0.5)
    rows = read_jsonl(pool_path)
    ok = [r for r in rows if not r["excluded"]]
    print(f"pool: {len(rows)} listed, {len(ok)} kept "
          f"({sum(r['split'] == 'blind' for r in ok)} blind-assigned, {sum(r['split'] == 'train' for r in ok)} train)")


def dhash(img):
    g = img.convert("L").resize((9, 8))
    px = list(g.getdata())
    bits = 0
    for y in range(8):
        for x in range(8):
            bits = (bits << 1) | (px[y * 9 + x] > px[y * 9 + x + 1])
    return f"{bits:016x}"


def cmd_fetch(a):
    from PIL import Image
    out = Path(a.out)
    dest = out / "real"
    dest.mkdir(parents=True, exist_ok=True)
    real_path = out / "real.jsonl"
    done = {r["pageid"] for r in read_jsonl(real_path)}
    todo = [r for r in read_jsonl(out / "pool.jsonl") if not r["excluded"] and r["split"] == a.split]
    n = 0
    for r in todo:
        if n >= a.limit:
            break
        if r["pageid"] in done:
            n += 1
            continue
        req = urllib.request.Request(r["url"], headers={"User-Agent": UA})
        with urllib.request.urlopen(req, timeout=120) as resp:
            data = resp.read()
        img = Image.open(io.BytesIO(data))
        name = f"{r['pageid']}.jpg"
        (dest / name).write_bytes(data)
        append_jsonl(real_path, {"pageid": r["pageid"], "split": r["split"], "path": f"real/{name}",
                                 "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data),
                                 "width": img.width, "height": img.height, "dhash": dhash(img)})
        n += 1
        time.sleep(0.3)
    print(f"fetch: {n} {a.split} photos present")


def chat_caption(url, vk, model, instruction, jpeg):
    b64 = base64.b64encode(jpeg).decode()
    body = {"model": model, "max_tokens": 2000, "messages": [{"role": "user", "content": [
        {"type": "text", "text": instruction},
        {"type": "image_url", "image_url": {"url": f"data:image/jpeg;base64,{b64}"}}]}]}
    return http_json(f"{url}/v1/chat/completions", body, {"x-bf-vk": vk}, timeout=180)


def cmd_caption(a):
    out = Path(a.out)
    vk = bifrost_key()
    instruction = Path(a.instruction).read_text().strip()
    cap_path = out / "captions.jsonl"
    done = {r["pageid"] for r in read_jsonl(cap_path)}
    reals = [r for r in read_jsonl(out / "real.jsonl") if r["split"] == a.split]
    n = 0
    for r in reals[: a.limit]:
        if r["pageid"] in done:
            continue
        t0 = time.time()
        status, d = chat_caption(a.url, vk, a.model, instruction, (out / r["path"]).read_bytes())
        text, usage = None, d.get("usage")
        if status == 200 and d.get("choices"):
            text = (d["choices"][0]["message"].get("content") or "").strip() or None
        append_jsonl(cap_path, {"pageid": r["pageid"], "split": r["split"], "model": a.model,
                                "status": status, "caption": text, "usage": usage,
                                "error": None if text else json.dumps(d)[:400], "seconds": round(time.time() - t0, 2)})
        n += 1
    print(f"caption: {n} new captions")


def nearest_ratio(w, h):
    r = w / h
    return min(RATIOS, key=lambda k: abs(RATIOS[k] - r))


def cmd_generate(a):
    out = Path(a.out)
    vk = bifrost_key()
    slug_dir = out / a.slug
    slug_dir.mkdir(parents=True, exist_ok=True)
    meta_path = out / "meta.jsonl"
    done = {r["pageid"] for r in read_jsonl(meta_path) if r.get("path")}
    reals = {r["pageid"]: r for r in read_jsonl(out / "real.jsonl")}
    caps = [c for c in read_jsonl(out / "captions.jsonl") if c["split"] == a.split and c["caption"]]
    n = 0
    for c in caps[: a.limit]:
        if c["pageid"] in done:
            continue
        r = reals[c["pageid"]]
        ratio = nearest_ratio(r["width"], r["height"])
        body = {"model": a.model, "prompt": c["caption"], "n": 1, "response_format": "b64_json",
                "aspect_ratio": ratio}
        t0 = time.time()
        status, d = http_json(f"{a.url}/v1/images/generations", body, {"x-bf-vk": vk}, timeout=300)
        row = {"pageid": c["pageid"], "split": c["split"], "model": a.model, "aspect_ratio": ratio,
               "status": status, "usage": d.get("usage"), "seconds": round(time.time() - t0, 2),
               "date": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        item = (d.get("data") or [None])[0] if status == 200 else None
        if item and item.get("b64_json"):
            data = base64.b64decode(item["b64_json"])
            ext = "png" if data[:8] == b"\x89PNG\r\n\x1a\n" else "jpg" if data[:2] == b"\xff\xd8" else "bin"
            name = f"{c['pageid']}.{ext}"
            (slug_dir / name).write_bytes(data)
            row.update(path=f"{a.slug}/{name}", sha256=hashlib.sha256(data).hexdigest(), bytes=len(data),
                       revised_prompt=item.get("revised_prompt"))
        else:
            row["error"] = json.dumps(d)[:400]
        append_jsonl(meta_path, row)
        n += 1
        if status == 403:
            sys.exit(f"stopped: {row['error']}")
    rows = read_jsonl(meta_path)
    with (out / "manifest.tsv").open("w") as f:
        for r in rows:
            if r.get("path"):
                f.write(f"{r['path']}\t1\t{a.generator}\t{a.dataset}\n")
                f.write(f"{reals[r['pageid']]['path']}\t0\treal:wikimedia-qi\t{a.dataset}\n")
    print(f"generate: {n} requests; {sum(1 for r in rows if r.get('path'))} images in all")


def cmd_group(a):
    out = Path(a.out)
    reals = read_jsonl(out / "real.jsonl")
    pool = {r["pageid"]: r for r in read_jsonl(out / "pool.jsonl")}
    parent = {r["pageid"]: r["pageid"] for r in reals}

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for i, x in enumerate(reals):
        for y in reals[i + 1:]:
            close = bin(int(x["dhash"], 16) ^ int(y["dhash"], 16)).count("1") <= 6
            px, py = pool[x["pageid"]], pool[y["pageid"]]
            same = px["artist"] and px["artist"] == py["artist"] and px["date"][:10] and px["date"][:10] == py["date"][:10]
            if close or same:
                parent[find(x["pageid"])] = find(y["pageid"])
    groups = {}
    for r in reals:
        groups.setdefault(find(r["pageid"]), []).append(r["pageid"])
    moved = 0
    final = {}
    for members in groups.values():
        split = "blind" if all(pool[m]["split"] == "blind" for m in members) else "train"
        for m in members:
            final[m] = split
            moved += pool[m]["split"] == "blind" and split == "train"
    (out / "groups.json").write_text(json.dumps({str(k): v for k, v in final.items()}))
    print(f"group: {len(groups)} groups over {len(reals)} photos; {moved} blind-assigned photos moved to training")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True)
    ap.add_argument("--url", default=os.environ.get("BIFROST_URL", "http://192.168.1.64:8080"))
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("pool")
    p.add_argument("--category", default="Category:Quality_images")
    p.add_argument("--start", required=True)
    p.add_argument("--end", required=True)
    p.add_argument("--want", type=int, required=True)
    p.add_argument("--salt", required=True)
    p.add_argument("--fraction", type=float, default=0.2)
    p.add_argument("--width", type=int, default=2048)
    p = sub.add_parser("fetch")
    p.add_argument("--split", choices=["train", "blind"], required=True)
    p.add_argument("--limit", type=int, required=True)
    p = sub.add_parser("caption")
    p.add_argument("--split", choices=["train", "blind"], required=True)
    p.add_argument("--limit", type=int, required=True)
    p.add_argument("--model", required=True)
    p.add_argument("--instruction", required=True)
    p = sub.add_parser("generate")
    p.add_argument("--split", choices=["train", "blind"], required=True)
    p.add_argument("--limit", type=int, required=True)
    p.add_argument("--model", required=True)
    p.add_argument("--slug", required=True)
    p.add_argument("--generator", required=True)
    p.add_argument("--dataset", required=True)
    sub.add_parser("group")
    a = ap.parse_args()
    {"pool": cmd_pool, "fetch": cmd_fetch, "caption": cmd_caption, "generate": cmd_generate, "group": cmd_group}[a.cmd](a)


if __name__ == "__main__":
    main()
