#!/usr/bin/env python3
"""The mixed photo-and-video gate (PROTOCOL.md, gate 3).

Usage: mixed_parity.py RAMDISK OUT.txt NAME=BINARY [NAME=BINARY ...]

Builds RAMDISK/mixed/mixed as in ../photo-chunk/mixed_folder.py: the first
1,000 files of the speed sample (RAMDISK/sample, in walk order) and the 300
clips of ../photo-chunk/mixed_videos.txt, interleaved three clips to every ten
photos. Every binary scores it with `batch --json`, and their standard
outputs must be byte-identical.
"""
import os
import shutil
import subprocess
import sys
import time

ram, outp = sys.argv[1], sys.argv[2]
bins = [a.split('=', 1) for a in sys.argv[3:]]
here = os.path.dirname(os.path.abspath(__file__))
MODELS = '/Users/aaron/Code/veracity-scratch/wt-pr55/models'
vids = ['/Volumes/veracity-data/data/vstore/' + l.split(' ', 1)[1].rstrip('\n') for l in open(os.path.join(here, '../photo-chunk/mixed_videos.txt'))]
photos = []
for r, ds, fs in os.walk(os.path.join(ram, 'sample')):
    ds.sort()
    photos += sorted(os.path.join(r, f) for f in fs)
photos = photos[:1000]
out = os.path.join(ram, 'mixed', 'mixed')
os.makedirs(out, exist_ok=True)
vi, pi = iter(vids), iter(photos)
for i in range(1300):
    s = next(vi) if i % 13 in (3, 7, 11) else next(pi)
    dst = os.path.join(out, f'm{i:05d}{os.path.splitext(s)[1].lower()}')
    if not os.path.exists(dst):
        shutil.copyfile(s, dst + '.part')
        os.replace(dst + '.part', dst)
log = open(outp, 'a')
res = {}
for name, binary in bins:
    cache = os.path.join(ram, 'caches', f'mixed-{name}')
    os.makedirs(cache, exist_ok=True)
    t0 = time.monotonic()
    p = subprocess.run([binary, 'batch', '--json', out], env={**os.environ, 'TINTYPE_MODEL_DIR': MODELS, 'TINTYPE_GPU_CACHE': cache}, capture_output=True)
    res[name] = p.stdout
    rows = [l for l in p.stdout.decode().splitlines() if l.startswith('{')]
    errors = sum('"error"' in l for l in rows)
    line = f'{name}: exit {p.returncode}, {len(rows)} rows, {errors} errors, {time.monotonic() - t0:.1f} s'
    print(line, flush=True)
    log.write(line + '\n')
ref = bins[0][0]
for name, _ in bins[1:]:
    line = f'{name} vs {ref}: standard output {"byte-identical" if res[name] == res[ref] else "DIFFERS"}'
    print(line, flush=True)
    log.write(line + '\n')
