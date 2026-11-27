#!/usr/bin/env python3
"""Matched speed rounds for the photo chunk change (PROTOCOL.md). Usage: speed_run.py RAMDISK OUT.tsv
RAMDISK holds sample/ and warm/ (TPB v1 speed sample and warm-up, rebuilt), ifold_b/{warm,sample} (the same files as
0_real/1_fake symlinks for BSF's loader), env/ (PyTorch venv), aigib-code/ (AIGIBench detector_codes at 70d5b75e) and
ckpt/BSF-main/checkpoint-best.pth. Tintype runs `batch --json` with the shipped models (second opinion on)."""
import os, subprocess, sys, time
from pathlib import Path

ram, out = Path(sys.argv[1]), Path(sys.argv[2])
S = Path('/Users/aaron/Code/veracity-scratch'); MODELS = str(S / 'edit-check/models')
BSF_TIMER = Path(__file__).resolve().parent / 'bsf_timer.py'
def tintype(repo):
    return (repo, {'TINTYPE_MODEL_DIR': MODELS}, lambda w: [str(repo / 'target/release/tintype'), 'batch', '--json', str(ram / w)])
DET = {1: ('Tintype 0947f3a6 (registered)', *tintype(S / 'edit-check')),
       2: ('Tintype origin/main 273e03f3 (B)', *tintype(S / 'so-base')),
       3: ('Tintype photo chunk 4096 (C)', *tintype(S / 'so-chunk')),
       4: ('BSF (AIGIBench release)', ram, {'OBJC_DISABLE_INITIALIZE_FORK_SAFETY': 'YES'},
           lambda w: [str(ram / 'env/bin/python'), '-I', str(BSF_TIMER), str(ram / 'aigib-code/detector_codes/BSF-main'),
                      str(ram / 'ckpt/BSF-main/checkpoint-best.pth'), str(ram / 'ifold_b' / w)])}
ORDERS = [(1, 2, 3, 4), (2, 3, 4, 1), (3, 4, 1, 2)]


def quiet():  # runs are sequential, so any tintype process alive here is someone else's
    while any(subprocess.run(['pgrep', '-x', n], capture_output=True).returncode == 0 for n in ('cargo', 'rustc', 'tintype')):
        print(time.strftime('%H:%M:%S'), 'cargo, rustc or another tintype running; waiting', flush=True); time.sleep(60)


def busy():  # processes over 20% CPU just before a timed run (ps's decaying average), recorded with each row
    ps = subprocess.run(['ps', '-Ao', 'pcpu=,comm='], capture_output=True, text=True).stdout.splitlines()
    hot = [l.split(None, 1) for l in ps if l.strip() and float(l.split(None, 1)[0]) > 20]
    return ';'.join(f'{os.path.basename(c.strip())}:{p}' for p, c in hot) or '-'


def run(k, which):
    name, cwd, env, argv = DET[k]
    quiet(); t0 = time.monotonic()
    p = subprocess.run(argv(which), cwd=cwd, env={**os.environ, **env}, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    dt = time.monotonic() - t0
    if p.returncode and 'could not be scored' not in p.stderr:
        sys.exit(f'{name} {which}: exit {p.returncode}\n{p.stderr[-1500:]}')
    rows = sum(1 for l in p.stdout.splitlines() if l.startswith('{'))
    return dt, rows or (p.stdout.strip().splitlines() or [''])[-1][:80]


with out.open('w') as f:
    f.write('round\tdetector\tseconds\tms_per_image\tload1\tbusy\toutput\n')
    for r, order in enumerate(ORDERS, 1):
        for k in order:
            run(k, 'warm')
            quiet(); load = os.getloadavg()[0]; hot = busy()
            dt, o = run(k, 'sample')
            f.write(f'{r}\t{DET[k][0]}\t{dt:.2f}\t{1000 * dt / 2000:.3f}\t{load:.2f}\t{hot}\t{o}\n'); f.flush()
            print(f'round {r} {DET[k][0]}: {dt:.2f} s, {1000 * dt / 2000:.3f} ms/image, load {load:.2f}, busy {hot}', flush=True)
