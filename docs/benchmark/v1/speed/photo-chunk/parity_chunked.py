#!/usr/bin/env python3
"""parity.py, staged: the share stalls at times, so each chunk of files is first copied to a RAM disk, then every binary
scores the same local copy, and the chunk is dropped. Rows are compared by path relative to the chunk, field for field.
Usage: parity_chunked.py RAMDIR OUT.txt NAME=REPO [NAME=REPO ...] -- FOLDER [...]
The file list is walked once and kept in RAMDIR/parity_files.txt; finished chunks are kept in RAMDIR/parity_done.json, so a
stopped run resumes at the next chunk. A file that cannot be copied after ten tries stops the run (nothing is skipped).
Chunk size: env PARITY_CHUNK (default 400), sized so a chunk fits the RAM disk with room to spare."""
import json, os, shutil, subprocess, sys, time

MODELS = '/Users/aaron/Code/veracity-scratch/edit-check/models'
EXT = ('.jpg', '.jpeg', '.png', '.webp', '.bmp', '.tif', '.tiff', '.gif', '.avif', '.heic')
N = int(os.environ.get('PARITY_CHUNK', '400'))
args = sys.argv[1:]; ram, outp = args.pop(0), args.pop(0); k = args.index('--'); bins, srcs = args[:k], args[k + 1:]
log = open(outp, 'a')
def say(*a):
    s = ' '.join(map(str, a)); print(s, flush=True); log.write(s + '\n'); log.flush()

listp, donep = os.path.join(ram, 'parity_files.txt'), os.path.join(ram, 'parity_done.json')
if os.path.exists(listp):
    files = [l.rstrip('\n') for l in open(listp) if l.strip()]
else:
    files = []
    for s in srcs:
        for r, ds, fs in os.walk(s):
            ds.sort()
            files += sorted(os.path.join(r, f) for f in fs if f.lower().endswith(EXT))
    with open(listp + '.tmp', 'w') as o: o.write('\n'.join(files) + '\n')
    os.replace(listp + '.tmp', listp)
done = json.load(open(donep)) if os.path.exists(donep) else {'n': N, 'chunks': [], 'rows': 0, 'diff': 0}
assert done['n'] == N, f'chunk size changed from {done["n"]} to {N}; start a fresh RAMDIR/parity_done.json'
say(time.strftime('%H:%M:%S'), 'files', len(files), 'chunk', N, 'binaries', [b.split('=')[0] for b in bins],
    '| resuming after', len(done['chunks']), 'chunks' if done['chunks'] else '')
for c in range(0, len(files), N):
    if c // N in done['chunks']: continue
    part = files[c:c + N]; d = os.path.join(ram, 'pchunk')
    if os.path.exists(d): shutil.rmtree(d)
    os.makedirs(d)
    t0 = time.time()
    for i, f in enumerate(part):
        dst = os.path.join(d, f'{i:04d}{os.path.splitext(f)[1].lower()}')
        for t in range(10):
            try: shutil.copyfile(f, dst + '.part'); os.replace(dst + '.part', dst); break
            except OSError as e:
                say('  copy retry', t + 1, f, repr(e))
                if os.path.exists(dst + '.part'): os.remove(dst + '.part')
                time.sleep(30)
        else:
            say('STOP: could not copy', f, 'after ten tries'); sys.exit(2)
    t1 = time.time(); res = {}
    for b in bins:
        name, repo = b.split('=', 1)
        p = subprocess.run([os.path.join(repo, 'target/release/tintype'), 'batch', '--json', d], cwd=repo,
                           env={**os.environ, 'TINTYPE_MODEL_DIR': MODELS}, capture_output=True, text=True)
        res[name] = {os.path.basename(r['path']): r for r in (json.loads(l) for l in p.stdout.splitlines() if l.startswith('{'))}
        if p.returncode != 0 or len(res[name]) != len(part):
            say('  NOTE', name, 'exit', p.returncode, 'rows', len(res[name]), 'of', len(part), '| stderr tail:', p.stderr[-300:])
    ref = bins[0].split('=')[0]; cd = 0
    for b in bins[1:]:
        name = b.split('=')[0]; a, o = res[ref], res[name]
        bad = sorted(k2 for k2 in set(a) | set(o) if a.get(k2) != o.get(k2))
        cd += len(bad)
        for k2 in bad[:5]:
            say('  DIFF', name, 'vs', ref, part[int(k2[:4])], {f: (a.get(k2, {}).get(f), o.get(k2, {}).get(f)) for f in set(a.get(k2, {})) | set(o.get(k2, {})) if a.get(k2, {}).get(f) != o.get(k2, {}).get(f)})
    done['rows'] += len(res[ref]); done['diff'] += cd; done['chunks'].append(c // N)
    with open(donep + '.tmp', 'w') as o: json.dump(done, o)
    os.replace(donep + '.tmp', donep)
    say(time.strftime('%H:%M:%S'), f'chunk {c // N}: {len(part)} files, rows {[len(v) for v in res.values()]}, copy {t1 - t0:.0f} s, score {time.time() - t1:.0f} s, load {os.getloadavg()[0]:.2f} | cumulative rows {done["rows"]}, differing rows {done["diff"]}')
    shutil.rmtree(d)
say('done: rows compared', done['rows'], '| rows differing in any field, over all pairs:', done['diff'])
