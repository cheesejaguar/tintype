#!/usr/bin/env python3
"""Gate 3 (PROTOCOL.md): the mixed photo-and-video folder through `batch` and `serve`.

Usage: mixed_serve.py RAMDISK OUT.txt B=BINARY C=BINARY

Builds RAMDISK/mixed/mixed as ../e2e-overlap/mixed_parity.py does (the first
1,000 photos of the speed sample and the 300 clips of
../photo-chunk/mixed_videos.txt, interleaved), then:
  1. each binary's `batch --json`: the standard output must be byte-identical;
  2. each binary's server scores the folder over HTTP, 16 files to a request:
     every result of C must equal B's;
  3. each server against its own binary's `batch` rows, as in
     parity_serve.py. `serve` reads only MP4 and MOV clips and samples them
     its own way, so clip results are not expected to match `batch`'s. What
     must hold is that each binary disagrees with its own `batch` on the same
     files in the same way.
"""
import json
import os
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from serve_client import MODELS, Server  # noqa: E402

ram, outp = sys.argv[1], sys.argv[2]
bins = dict(a.split('=', 1) for a in sys.argv[3:])
here = os.path.dirname(os.path.abspath(__file__))
FIELDS = ('kind', 'profile', 'score', 'verdict', 'views', 'logit', 'threshold_logit', 'second_opinion')
vids = ['/Volumes/veracity-data/data/vstore/' + l.split(' ', 1)[1].rstrip('\n') for l in open(os.path.join(here, '../photo-chunk/mixed_videos.txt'))]
photos = []
for r, ds, fs in os.walk(os.path.join(ram, 'sample')):
    ds.sort()
    photos += sorted(os.path.join(r, f) for f in fs)
photos = photos[:1000]
out = os.path.join(ram, 'mixed', 'mixed')
os.makedirs(out, exist_ok=True)
vi, pi = iter(vids), iter(photos)
names = []
for i in range(1300):
    s = next(vi) if i % 13 in (3, 7, 11) else next(pi)
    dst = os.path.join(out, f'm{i:05d}{os.path.splitext(s)[1].lower()}')
    if not os.path.exists(dst):
        shutil.copyfile(s, dst + '.part')
        os.replace(dst + '.part', dst)
    names.append(dst)
log = open(outp, 'a')


def say(*a):
    s = ' '.join(map(str, a))
    print(s, flush=True)
    log.write(s + '\n')
    log.flush()


stdout, rows, served, mismatch = {}, {}, {}, {}
for k, name in enumerate(('B', 'C')):
    cache = os.path.join(ram, 'caches', f'mixed-{name}')
    os.makedirs(cache, exist_ok=True)
    p = subprocess.run([bins[name], 'batch', '--json', out], env={**os.environ, 'TINTYPE_MODEL_DIR': MODELS, 'TINTYPE_GPU_CACHE': cache}, capture_output=True)
    stdout[name] = p.stdout
    rows[name] = {r['path']: r for r in (json.loads(l) for l in p.stdout.decode().splitlines() if l.startswith('{'))}
    server = Server(bins[name], 18931 + k, os.path.join(ram, 'caches', f'mixed-serve-{name}'), open(outp + '.serve.log', 'a'))
    try:
        res, spent = server.score(names)
    finally:
        server.close()
    served[name] = dict(zip(names, res))
    mismatch[name] = {}
    for f in names:
        b, s = rows[name].get(f), served[name][f]
        same = b is not None and (('error' in b) == ('error' in s)) and ('error' in b or all(b.get(x) == s.get(x) for x in FIELDS))
        if not same:
            mismatch[name][f] = (None if b is None else {x: b.get(x) for x in FIELDS + ('error',)}, {x: s.get(x) for x in FIELDS + ('error',)})
    photos_off = sum(1 for f in mismatch[name] if not f.endswith(('.mp4', '.webm', '.mov', '.mkv')))
    say(f'{name}: batch exit {p.returncode}, {len(rows[name])} rows, served {len(res)} results in {spent:.1f} s, served vs batch disagreements {len(mismatch[name])} (photos {photos_off})')
say('batch standard output C vs B:', 'byte-identical' if stdout['C'] == stdout['B'] else 'DIFFERS')
diff = [f for f in names if {x: v for x, v in served['B'][f].items() if x != 'name'} != {x: v for x, v in served['C'][f].items() if x != 'name'}]
say('served results C vs B:', len(diff), 'differ')
say('served vs batch disagreements the same, file for file and field for field, for B and C:', mismatch['B'] == mismatch['C'])
