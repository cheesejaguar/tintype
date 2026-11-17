#!/bin/bash
# Speed gate of the fast-options pre-registration: 300 test rows drawn with
# seed 8 (150 authentic, 150 generated), systems taking turns, best of three.
# Run on an idle machine.
set -uo pipefail
cd "$(dirname "$0")/../.."
S=runs/fast/speed; mkdir -p $S
python3 - <<'PY'
import random
rows=[l for l in open('data/frozen-eval/test.tsv')]
rng=random.Random(8)
real=[l for l in rows if l.split('\t')[1]=='0']; ai=[l for l in rows if l.split('\t')[1]=='1']
pick=rng.sample(real,150)+rng.sample(ai,150); rng.shuffle(pick)
open('runs/fast/speed/sample.tsv','w').writelines(pick)
PY
run() { # label bin name device
  local label=$1 bin=$2 name=$3 dev=$4 w=runs/fast/$3/model.safetensors
  echo "== $label latency"; "$bin" latency --weights $w --manifest $S/sample.tsv --n 295 --device $dev 2>&1 | grep single-image
  local t0=$(python3 -c 'import time;print(time.time())')
  "$bin" score --weights $w --manifest $S/sample.tsv --out $S/out.tsv --device $dev > /dev/null 2>&1
  echo "== $label throughput: 300 rows in $(python3 -c "import time;print(round(time.time()-$t0,2))") s"
}
for pass in 1 2 3; do
  echo "#### pass $pass"
  run "accelerate cpu" bin/tintype-speed incumbent-cpu cpu
  run "accelerate cpu-int8" bin/tintype-speed int8 cpu-int8
  run "portable cpu" bin/tintype-fast-portable incumbent-cpu cpu
  run "portable cpu-int8" bin/tintype-fast-portable int8 cpu-int8
  run "gpu incumbent" bin/tintype-speed incumbent-gpu gpu
  run "gpu student-8" bin/tintype-speed student-8 gpu
done
echo done
