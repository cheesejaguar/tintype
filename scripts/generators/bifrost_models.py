#!/usr/bin/env python3
"""List the image and video generation models the Bifrost gateway's key can reach.

Step 1 of docs/NEW_GENERATOR_PLAYBOOK.md. Reads the virtual key from
~/.bifrost_env (BIFROST_VK) and the gateway URL from BIFROST_URL
(default http://192.168.1.64:8080). With --state FILE, prints what changed
since the last run and updates FILE.

    python3 scripts/generators/bifrost_models.py --state ~/.cache/tintype-bifrost-models.json
"""
import argparse
import json
import os
import re
import urllib.request
from pathlib import Path

MEDIA = re.compile(
    r"imagen|image|banana|veo|dall-?e|gpt-image|flux|seedream|seedance|ideogram|recraft|kling|sora|"
    r"\bwan|hunyuan|qwen-image|midjourney|runway|luma|pika|hailuo|video|t2v|i2v|ti2v",
    re.I,
)


def key():
    vk = os.environ.get("BIFROST_VK")
    if not vk:
        env = Path.home() / ".bifrost_env"
        for line in env.read_text().splitlines():
            line = line.strip().removeprefix("export ").strip()
            if line.startswith("BIFROST_VK="):
                vk = line.split("=", 1)[1].strip().strip("'\"")
    if not vk:
        raise SystemExit("BIFROST_VK is not set and ~/.bifrost_env has none")
    return vk


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--url", default=os.environ.get("BIFROST_URL", "http://192.168.1.64:8080"))
    ap.add_argument("--state", type=Path, help="JSON file holding the previous list")
    ap.add_argument("--all", action="store_true", help="list every model, not only image and video ones")
    args = ap.parse_args()

    req = urllib.request.Request(f"{args.url}/v1/models", headers={"x-bf-vk": key()})
    with urllib.request.urlopen(req, timeout=30) as resp:
        models = sorted(m["id"] for m in json.load(resp).get("data", []))
    shown = models if args.all else [m for m in models if MEDIA.search(m)]
    print(f"{len(models)} models reachable; {len(shown)} shown")
    for m in shown:
        print(" ", m)
    if args.state:
        old = set(json.loads(args.state.read_text())) if args.state.exists() else set()
        new = set(models)
        for m in sorted(new - old):
            print("NEW    ", m)
        for m in sorted(old - new):
            print("REMOVED", m)
        args.state.parent.mkdir(parents=True, exist_ok=True)
        args.state.write_text(json.dumps(sorted(new), indent=1))


if __name__ == "__main__":
    main()
