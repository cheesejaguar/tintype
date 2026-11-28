#!/usr/bin/env python3
"""Interleaved timing rounds of `tintype batch --json` (PROTOCOL.md, Speed).

Usage: speed_ab.py RAMDISK OUT.tsv ROUNDS NAME=BINARY [NAME=BINARY ...]

RAMDISK holds sample/ (the TPB v1 speed sample, 2,000 files) and warm/ (the
next 100). A round runs every binary once, in an order rotated each round:
one run on warm/, then one timed run on sample/, from process start to exit.
Every binary has its own GPU cache directory, RAMDISK/caches/speed-NAME, so
its warm-up leaves the timed run a cache hit and no binary evicts another's
entries. Recorded with each timed run: the 1-minute load average, the
processes above 20% CPU (ps's decaying average) and whether any other
`tintype`, `cargo` or `rustc` process was running. Each timed run's rows must
equal the first timed run's, or the script stops. Prints the median per
binary and its ratio to the first binary's.
"""
import os
import statistics
import subprocess
import sys
import time

ram, out, rounds = sys.argv[1], sys.argv[2], int(sys.argv[3])
bins = [a.split('=', 1) for a in sys.argv[4:]]
MODELS = '/Users/aaron/Code/veracity-scratch/wt-pr55/models'


def busy():
    ps = subprocess.run(['ps', '-Ao', 'pcpu=,comm='], capture_output=True, text=True).stdout.splitlines()
    hot = [l.split(None, 1) for l in ps if l.strip() and float(l.split(None, 1)[0]) > 20]
    return ';'.join(f'{os.path.basename(c.strip())}:{p}' for p, c in hot) or '-'


def others():
    return ','.join(n for n in ('tintype', 'cargo', 'rustc') if subprocess.run(['pgrep', '-x', n], capture_output=True).returncode == 0) or '-'


def run(name, binary, which):
    cache = os.path.join(ram, 'caches', f'speed-{name}')
    os.makedirs(cache, exist_ok=True)
    env = {**os.environ, 'TINTYPE_MODEL_DIR': MODELS, 'TINTYPE_GPU_CACHE': cache}
    t0 = time.monotonic()
    p = subprocess.run([binary, 'batch', '--json', os.path.join(ram, which)], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    dt = time.monotonic() - t0
    if p.returncode != 0:
        sys.exit(f'{name} {which}: exit {p.returncode}\n{p.stderr[-1500:]}')
    return dt, [l for l in p.stdout.decode().splitlines() if l.startswith('{')]


ref = None
times = {n: [] for n, _ in bins}
with open(out, 'w') as f:
    f.write('round\tbinary\tseconds\tms_per_image\tload1\tbusy\tother_processes\trows\tsame_rows\n')
    for r in range(rounds):
        k = r % len(bins)
        for name, binary in bins[k:] + bins[:k]:
            run(name, binary, 'warm')
            load, hot, oth = os.getloadavg()[0], busy(), others()
            dt, rows = run(name, binary, 'sample')
            ref = rows if ref is None else ref
            same = rows == ref
            times[name].append(dt)
            f.write(f'{r + 1}\t{name}\t{dt:.3f}\t{1000 * dt / 2000:.3f}\t{load:.2f}\t{hot}\t{oth}\t{len(rows)}\t{same}\n')
            f.flush()
            print(f'round {r + 1} {name}: {dt:.3f} s, {1000 * dt / 2000:.3f} ms/image, load {load:.2f}, busy {hot}, others {oth}, rows {len(rows)}, same {same}', flush=True)
            if not same:
                sys.exit(f'{name}: rows differ from the first timed run')
base = bins[0][0]
med = {n: statistics.median(v) for n, v in times.items()}
for n, _ in bins:
    print(f'{n}: median {med[n]:.3f} s, {1000 * med[n] / 2000:.3f} ms/image, {2000 / med[n]:.0f} images/s, ratio to {base} {med[n] / med[base]:.3f}')
