#!/usr/bin/env python3
"""Interleaved timing of `tintype serve` (PROTOCOL.md, Speed). Copied from ../serve-beside/, unchanged but for this line.

Usage: serve_speed.py RAMDISK OUT.tsv ROUNDS B=BINARY C=BINARY

Both servers are started once, each with its own GPU cache directory, and
each scores RAMDISK/warm once over HTTP before the first round. In each round
every server scores RAMDISK/sample (2,000 photos) in requests of 16, in order;
the order of the two servers alternates from round to round. Timed: the
seconds from the first request's start to the last answer. Recorded with
each timed pass: the 1-minute load average, the processes above 20% CPU, and
how many `tintype` processes other than the two servers were running. Every
pass's results must equal B's first pass, or the script stops.
"""
import os
import statistics
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from serve_client import Server  # noqa: E402

ram, out, rounds = sys.argv[1], sys.argv[2], int(sys.argv[3])
bins = [a.split('=', 1) for a in sys.argv[4:]]


def walk(folder):
    files = []
    for r, ds, fs in os.walk(folder):
        ds.sort()
        files += sorted(os.path.join(r, f) for f in fs)
    return files


def busy():
    ps = subprocess.run(['ps', '-Ao', 'pcpu=,comm='], capture_output=True, text=True).stdout.splitlines()
    hot = [l.split(None, 1) for l in ps if l.strip() and float(l.split(None, 1)[0]) > 20]
    return ';'.join(f'{os.path.basename(c.strip())}:{p}' for p, c in hot) or '-'


def other_tintype():
    p = subprocess.run(['pgrep', '-x', 'tintype'], capture_output=True, text=True)
    return max(0, len(p.stdout.split()) - len(bins))


log = open(out + '.serve.log', 'w')
servers = {name: Server(binary, 18831 + k, os.path.join(ram, 'caches', f'sspeed-{name}'), log) for k, (name, binary) in enumerate(bins)}
try:
    warm, sample = walk(os.path.join(ram, 'warm')), walk(os.path.join(ram, 'sample'))
    for s in servers.values():
        s.score(warm)
    ref = None
    times = {n: [] for n, _ in bins}
    with open(out, 'w') as f:
        f.write('round\tserver\tseconds\tms_per_image\tload1\tbusy\tother_tintype\tresults\tsame_results\n')
        for r in range(rounds):
            k = r % len(bins)
            for name, _ in bins[k:] + bins[:k]:
                load, hot, oth = os.getloadavg()[0], busy(), other_tintype()
                t0 = time.monotonic()
                res, _ = servers[name].score(sample)
                dt = time.monotonic() - t0
                ref = res if ref is None else ref
                same = res == ref
                times[name].append(dt)
                f.write(f'{r + 1}\t{name}\t{dt:.3f}\t{1000 * dt / len(sample):.3f}\t{load:.2f}\t{hot}\t{oth}\t{len(res)}\t{same}\n')
                f.flush()
                print(f'round {r + 1} {name}: {dt:.3f} s, {1000 * dt / len(sample):.3f} ms/image, load {load:.2f}, busy {hot}, other tintype {oth}, same {same}', flush=True)
                if not same:
                    sys.exit(f'{name}: results differ from the first pass')
    base = bins[0][0]
    med = {n: statistics.median(v) for n, v in times.items()}
    for n, _ in bins:
        print(f'{n}: median {med[n]:.3f} s, {1000 * med[n] / len(sample):.3f} ms/image, {len(sample) / med[n]:.0f} images/s, ratio to {base} {med[n] / med[base]:.3f}')
finally:
    for s in servers.values():
        s.close()
