#!/usr/bin/env python3
"""Row parity over the batch-9 open files (PROTOCOL.md, gate 2), staged through a RAM disk.

Usage: parity_staged.py OPEN_TSV_GZ SOURCE STAGE OUT.txt NAME=BINARY [NAME=BINARY ...]

The files of OPEN_TSV_GZ (in manifest order) are copied from SOURCE in chunks of
PARITY_CHUNK (default 1500) to STAGE/<path>, keeping their relative paths, and
each copy's SHA-256 is checked against the manifest. Every binary then scores
the same staged chunk with `batch --json STAGE`, with its own GPU cache
directory under STAGE/../caches. Rows are compared with the first binary's,
field for field, by path, and the whole standard output byte for byte. Only
the files this script copied are removed afterwards, then the directories
they leave empty. Finished chunks are recorded in OUT.txt.done, so a stopped
run resumes at the next chunk. Nothing is skipped: a file that cannot be
copied after ten tries stops the run.
"""
import gzip
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time

MODELS = '/Users/aaron/Code/veracity-scratch/wt-pr55/models'
N = int(os.environ.get('PARITY_CHUNK', '1500'))
tsv, src, stage, outp = sys.argv[1:5]
bins = [a.split('=', 1) for a in sys.argv[5:]]
stage = os.path.abspath(stage)
assert stage.startswith('/Volumes/') and stage.count('/') >= 3, f'{stage}: stage below a mounted volume'
log = open(outp, 'a')


def say(*a):
    s = ' '.join(map(str, a))
    print(s, flush=True)
    log.write(s + '\n')
    log.flush()


rows = [l.rstrip('\n').split('\t') for l in gzip.open(tsv, 'rt')]
head, rows = rows[0], rows[1:]
P, H = head.index('path'), head.index('sha256')
donep = outp + '.done'
done = json.load(open(donep)) if os.path.exists(donep) else {'n': N, 'chunks': [], 'rows': 0, 'diff': 0, 'bytes_diff': 0}
assert done['n'] == N, 'the chunk size changed; start a fresh OUT.txt'
say(time.strftime('%H:%M:%S'), 'files', len(rows), 'chunk', N, 'binaries', [n for n, _ in bins], '| resuming after', len(done['chunks']), 'chunks')
for c in range(0, len(rows), N):
    if c // N in done['chunks']:
        continue
    part = rows[c:c + N]
    t0 = time.time()
    copied = []
    for r in part:
        dst = os.path.join(stage, r[P])
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        for t in range(10):
            try:
                shutil.copyfile(os.path.join(src, r[P]), dst + '.part')
                h = hashlib.sha256(open(dst + '.part', 'rb').read()).hexdigest()
                if h != r[H]:
                    raise OSError(f'sha256 {h} is not the manifest\'s {r[H]}')
                os.replace(dst + '.part', dst)
                copied.append(dst)
                break
            except OSError as e:
                say('  copy retry', t + 1, r[P], repr(e))
                if os.path.exists(dst + '.part'):
                    os.remove(dst + '.part')
                time.sleep(30)
        else:
            say('STOP: could not copy', r[P], 'after ten tries')
            sys.exit(2)
    t1 = time.time()
    out, res = {}, {}
    for name, binary in bins:
        cache = os.path.join(os.path.dirname(stage), 'caches', f'parity-{name}')
        os.makedirs(cache, exist_ok=True)
        p = subprocess.run([binary, 'batch', '--json', stage], env={**os.environ, 'TINTYPE_MODEL_DIR': MODELS, 'TINTYPE_GPU_CACHE': cache}, capture_output=True)
        out[name] = p.stdout
        res[name] = {r['path']: r for r in (json.loads(l) for l in p.stdout.decode().splitlines() if l.startswith('{'))}
        if p.returncode not in (0, 2) or len(res[name]) != len(part):
            say('  NOTE', name, 'exit', p.returncode, 'rows', len(res[name]), 'of', len(part), '| stderr tail:', p.stderr[-300:])
    ref = bins[0][0]
    cd = cb = 0
    for name, _ in bins[1:]:
        a, o = res[ref], res[name]
        bad = sorted(k for k in set(a) | set(o) if a.get(k) != o.get(k))
        cd += len(bad)
        cb += out[name] != out[ref]
        for k in bad[:5]:
            say('  DIFF', name, 'vs', ref, k, {f: (a.get(k, {}).get(f), o.get(k, {}).get(f)) for f in set(a.get(k, {})) | set(o.get(k, {})) if a.get(k, {}).get(f) != o.get(k, {}).get(f)})
    for f in copied:
        os.remove(f)
    for d, _, _ in sorted(os.walk(stage), key=lambda w: -len(w[0])):
        if d != stage and not os.listdir(d):
            os.rmdir(d)
    done['rows'] += len(res[ref])
    done['diff'] += cd
    done['bytes_diff'] += cb
    done['chunks'].append(c // N)
    with open(donep + '.tmp', 'w') as o:
        json.dump(done, o)
    os.replace(donep + '.tmp', donep)
    say(time.strftime('%H:%M:%S'), f'chunk {c // N}: {len(part)} files, rows {[len(v) for v in res.values()]}, copy {t1 - t0:.0f} s, score {time.time() - t1:.0f} s, load {os.getloadavg()[0]:.2f} | cumulative rows {done["rows"]}, differing rows {done["diff"]}, chunks whose output differs {done["bytes_diff"]}')
say('done: rows compared', done['rows'], '| rows differing in any field, over all pairs:', done['diff'], '| chunks whose standard output differs:', done['bytes_diff'])
