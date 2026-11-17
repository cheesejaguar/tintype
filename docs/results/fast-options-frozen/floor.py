"""The pre-registered accuracy floor for the fast options (docs/PREREGISTRATION.md).
Usage: python3 runs/fast/floor.py REFERENCE CANDIDATE...   (names under runs/fast)"""
import sys, os, json
here = os.path.dirname(os.path.abspath(__file__))

def load(name, f):
    rows = []
    for line in open(os.path.join(here, name, f)):
        c = line.rstrip('\n').split('\t')
        if len(c) < 5: continue
        label, source, p = int(c[1]), c[3], float(c[4])
        rows.append((label, '-' not in source, p))
    return rows

def stats(rows, thr):
    out = {}
    for part, keep in [('all', lambda r: True), ('clean', lambda r: r[1]), ('processed', lambda r: not r[1])]:
        sel = [r for r in rows if keep(r)]
        ai = [r for r in sel if r[0] == 1]; real = [r for r in sel if r[0] == 0]
        out[part] = dict(recall=sum(r[2] >= thr for r in ai) / len(ai), fpr=sum(r[2] >= thr for r in real) / len(real),
                         flagged=sum(r[2] >= thr for r in real), n_real=len(real), n_ai=len(ai))
    return out

def summary(name):
    thr = json.load(open(os.path.join(here, name, 'model.json')))['threshold']
    return stats(load(name, 'test.tsv'), thr), stats(load(name, 'test-strict.tsv'), thr)

ref = sys.argv[1]
r1, _ = summary(ref)
print(f"reference {ref}: 1% cutoff clean recall {r1['clean']['recall']:.2%}, processed {r1['processed']['recall']:.2%}, authentic flagged {r1['all']['fpr']:.2%}\n")
print("| Candidate | Profile | Authentic flagged (all test) | Clean recall | Processed recall | Clean authentic | Processed authentic | Floor |")
print("|---|---|---|---|---|---|---|---|")
for name in [ref] + sys.argv[2:]:
    p, s = summary(name)
    gates = [p['all']['fpr'] <= 0.015, p['clean']['recall'] >= r1['clean']['recall'] - 0.05, p['processed']['recall'] >= r1['processed']['recall'] - 0.08]
    verdict = 'reference' if name == ref else ('passes' if all(gates) else 'fails: ' + ', '.join(n for n, g in zip(['authentic rate', 'clean recall', 'processed recall'], gates) if not g))
    for prof, st, v in [('1% (default)', p, verdict), ('0.1% (--strict)', s, '')]:
        print(f"| {name} | {prof} | {st['all']['flagged']} of {st['all']['n_real']} ({st['all']['fpr']:.2%}) | {st['clean']['recall']:.2%} | {st['processed']['recall']:.2%} | {st['clean']['fpr']:.2%} | {st['processed']['fpr']:.2%} | {v} |")
