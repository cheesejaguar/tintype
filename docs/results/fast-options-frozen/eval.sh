#!/bin/bash
# One candidate of the "Fast options" pre-registration (docs/PREREGISTRATION.md):
#   runs/fast/eval.sh NAME WEIGHTS DEVICE [prepare|test]
# prepare: copy the weights and spec under runs/fast/NAME, score fit and calibrate
#          with every view, fit the 1% (photo) and 0.1% (strict) cutoffs on
#          calibrate, refit the early-exit margins on fit.
# test:    the one look at test: default profile and --strict, early exit as shipped.
# Run from the training tree. Nothing here writes under models/.
set -euo pipefail
cd "$(dirname "$0")/../.."
NAME=$1 SRC=$2 DEV=$3 STEP=${4:-prepare}
BIN=${TINTYPE_BIN:-bin/tintype-fast}
E=runs/fast/$NAME
F=data/frozen-eval
W=$E/model.safetensors
log() { echo "[$(date '+%F %T')] $NAME: $*"; }

if [ "$STEP" = prepare ]; then
  mkdir -p "$E"
  cp "$SRC" "$W"
  cp "${SRC%.safetensors}.json" "$E/model.json"
  for set in fit calibrate; do
    log "score $set, every view, $DEV"
    /usr/bin/time -p "$BIN" score --weights "$W" --manifest $F/$set.tsv --out "$E/$set-views.tsv" --all-views --device "$DEV" 2> "$E/$set-views.time"
  done
  log calibrate
  "$BIN" calibrate --weights "$W" --scores "$E/calibrate-views.tsv" --profile photo --target-fpr 0.01 > "$E/calibrate-photo.txt"
  "$BIN" calibrate --weights "$W" --scores "$E/calibrate-views.tsv" --profile strict --target-fpr 0.001 > "$E/calibrate-strict.txt"
  log refit-exit
  "$BIN" refit-exit --weights "$W" --profile photo,strict --target-fpr 0.01,0.001 \
    --fit "$E/fit-views.tsv" --report "$E/calibrate-views.tsv" --tolerance 0.001 --write > "$E/refit-exit.txt"
  cp "$E/model.json" "$E/model.frozen.json"
  shasum -a 256 "$W" "$E/model.json" > "$E/frozen.sha256"
  log "prepared; spec frozen"
elif [ "$STEP" = test ]; then
  shasum -a 256 -c "$E/frozen.sha256"
  log "TEST LOOK: default profile, $DEV"
  /usr/bin/time -p "$BIN" score --weights "$W" --manifest $F/test.tsv --out "$E/test.tsv" --device "$DEV" 2> "$E/test.time"
  log "TEST LOOK: strict, $DEV"
  "$BIN" score --weights "$W" --manifest $F/test.tsv --out "$E/test-strict.tsv" --device "$DEV" --strict
  echo "$(date '+%F %T') test scored once ($DEV)" >> "$E/looks.log"
  log done
fi
"$BIN" verify-models > /dev/null 2>&1 || true
