#!/usr/bin/env python3
"""Row parity for `batch` and `serve` (PROTOCOL.md), staged through a RAM disk.

Copied from ../serve-beside/ (branch perf/serve-second-beside), unchanged but for this line.

Usage:
  parity_serve.py staged OPEN_TSV_GZ SOURCE STAGE OUT.txt B=BINARY C=BINARY
  parity_serve.py folder FOLDER OUT.txt B=BINARY C=BINARY

`staged` copies the files of OPEN_TSV_GZ in manifest order from SOURCE in
chunks of PARITY_CHUNK (default 500) to STAGE/<path>, checking each copy's
SHA-256 against the manifest. `folder` scores FOLDER as it is. For each chunk
(or the folder):
  1. each binary runs `batch --json`; every field of every row of C must
     equal B's, and the standard output must be byte-identical;
  2. each binary's server (started once, kept for the whole run) scores the
     same files over HTTP, 16 to a request; every field of every result of C
     must equal B's;
  3. each server's results must agree with its own binary's `batch` rows on
     kind, profile, score, verdict, views, logit, threshold_logit and
     second_opinion, and on which files could not be scored.
Only the files this script copied are removed afterwards, then the
directories they leave empty. Finished chunks are recorded in OUT.txt.done.
"""
import gzip
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from serve_client import MODELS, Server  # noqa: E402

FIELDS = ('kind', 'profile', 'score', 'verdict', 'views', 'logit', 'threshold_logit', 'second_opinion')
EXT = ('.jpg', '.jpeg', '.png', '.webp', '.bmp', '.tif', '.tiff', '.gif', '.mp4', '.mov', '.m4v', '.webm', '.mkv')
mode = sys.argv[1]
if mode == 'staged':
    tsv, src, stage, outp = sys.argv[2:6]
    bins = dict(a.split('=', 1) for a in sys.argv[6:])
else:
    stage, outp = sys.argv[2:4]
    bins = dict(a.split('=', 1) for a in sys.argv[4:])
stage = os.path.abspath(stage)
assert stage.startswith('/Volumes/') and stage.count('/') >= 3, f'{stage}: below a mounted volume'
ram = '/'.join(stage.split('/')[:3])
N = int(os.environ.get('PARITY_CHUNK', '500'))
log = open(outp, 'a')
server_log = open(outp + '.serve.log', 'a')


def say(*a):
    s = ' '.join(map(str, a))
    print(s, flush=True)
    log.write(s + '\n')
    log.flush()


def batch(name, folder):
    cache = os.path.join(ram, 'caches', f'pserve-batch-{name}')
    os.makedirs(cache, exist_ok=True)
    p = subprocess.run([bins[name], 'batch', '--json', folder], env={**os.environ, 'TINTYPE_MODEL_DIR': MODELS, 'TINTYPE_GPU_CACHE': cache}, capture_output=True)
    if p.returncode not in (0, 2):
        say('  NOTE', name, 'batch exit', p.returncode, p.stderr[-300:])
    return p.stdout, {r['path']: r for r in (json.loads(l) for l in p.stdout.decode().splitlines() if l.startswith('{'))}


def walk(folder):
    files = []
    for r, ds, fs in os.walk(folder):
        ds.sort()
        files += sorted(os.path.join(r, f) for f in fs if f.lower().endswith(EXT))
    return files


servers = {name: Server(binary, 18731 + k, os.path.join(ram, 'caches', f'pserve-serve-{name}'), server_log) for k, (name, binary) in enumerate(sorted(bins.items()))}
donep = outp + '.done'
done = json.load(open(donep)) if os.path.exists(donep) else {'n': N, 'chunks': [], 'files': 0, 'batch_diff': 0, 'batch_bytes_diff': 0, 'serve_diff': 0, 'serve_vs_batch_diff': 0}
assert done['n'] == N


def check(folder, label):
    out, rows = {}, {}
    for name in ('B', 'C'):
        out[name], rows[name] = batch(name, folder)
    paths = walk(folder)
    served = {}
    spent = {}
    for name in ('B', 'C'):
        res, spent[name] = servers[name].score(paths)
        served[name] = dict(zip(paths, res))
    bd = sorted(k for k in set(rows['B']) | set(rows['C']) if rows['B'].get(k) != rows['C'].get(k))
    sd = sorted(k for k in paths if {f: v for f, v in served['B'][k].items() if f != 'name'} != {f: v for f, v in served['C'][k].items() if f != 'name'})
    vb = 0
    for name in ('B', 'C'):
        for k in paths:
            b, s = rows[name].get(k), served[name][k]
            same = b is not None and (('error' in b) == ('error' in s)) and ('error' in b or all(b.get(f) == s.get(f) for f in FIELDS))
            if not same:
                vb += 1
                if vb <= 5:
                    say('  SERVE-VS-BATCH', name, k, {f: (None if b is None else b.get(f), s.get(f)) for f in FIELDS + ('error',)})
    for k in bd[:5]:
        say('  BATCH-DIFF', k, rows['B'].get(k), rows['C'].get(k))
    for k in sd[:5]:
        say('  SERVE-DIFF', k, served['B'][k], served['C'][k])
    bb = int(out['B'] != out['C'])
    say(time.strftime('%H:%M:%S'), f'{label}: {len(paths)} files, batch rows {[len(rows[n]) for n in ("B", "C")]}, batch rows differing {len(bd)}, batch output differs {bb}, served results differing {len(sd)}, served vs batch disagreements {vb}, request seconds B {spent["B"]:.1f} C {spent["C"]:.1f}, load {os.getloadavg()[0]:.2f}')
    return len(paths), len(bd), bb, len(sd), vb


try:
    if mode == 'folder':
        n, bd, bb, sd, vb = check(stage, 'folder')
        say(f'done: files {n} | batch rows differing {bd} | batch output differs {bb} | served results differing {sd} | served vs batch disagreements {vb}')
        sys.exit()
    rows = [l.rstrip('\n').split('\t') for l in gzip.open(tsv, 'rt')]
    head, rows = rows[0], rows[1:]
    P, H = head.index('path'), head.index('sha256')
    say(time.strftime('%H:%M:%S'), 'files', len(rows), 'chunk', N, '| resuming after', len(done['chunks']), 'chunks')
    for c in range(0, len(rows), N):
        if c // N in done['chunks']:
            continue
        copied = []
        for r in rows[c:c + N]:
            dst = os.path.join(stage, r[P])
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            for t in range(10):
                try:
                    shutil.copyfile(os.path.join(src, r[P]), dst + '.part')
                    h = hashlib.sha256(open(dst + '.part', 'rb').read()).hexdigest()
                    if h != r[H]:
                        raise OSError(f'sha256 {h} is not the manifest\'s')
                    os.replace(dst + '.part', dst)
                    copied.append(dst)
                    break
                except OSError as e:
                    say('  copy retry', t + 1, r[P], repr(e))
                    if os.path.exists(dst + '.part'):
                        os.remove(dst + '.part')
                    time.sleep(30)
            else:
                say('STOP: could not copy', r[P])
                sys.exit(2)
        n, bd, bb, sd, vb = check(stage, f'chunk {c // N}')
        for f in copied:
            os.remove(f)
        for d, _, _ in sorted(os.walk(stage), key=lambda w: -len(w[0])):
            if d != stage and not os.listdir(d):
                os.rmdir(d)
        done['files'] += n
        done['batch_diff'] += bd
        done['batch_bytes_diff'] += bb
        done['serve_diff'] += sd
        done['serve_vs_batch_diff'] += vb
        done['chunks'].append(c // N)
        with open(donep + '.tmp', 'w') as o:
            json.dump(done, o)
        os.replace(donep + '.tmp', donep)
    say(f'done: files {done["files"]} | batch rows differing {done["batch_diff"]} | chunks whose batch output differs {done["batch_bytes_diff"]} | served results differing {done["serve_diff"]} | served vs batch disagreements {done["serve_vs_batch_diff"]}')
finally:
    for s in servers.values():
        s.close()
