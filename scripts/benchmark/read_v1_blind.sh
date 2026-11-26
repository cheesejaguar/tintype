#!/bin/sh
# The one read of TPB v1's blind split (BLIND-PROTOCOL.md), and the open split for comparison.
# Usage: sh scripts/benchmark/read_v1_blind.sh SCORES_DIR OUT_DIR
set -eu
S=$1; O=$2; T=scripts/benchmark/tpb.py
mkdir -p "$O"
secs() { python3 -c "import json,sys; print(json.load(open('$S/timing.json'))['seconds']['$1'])"; }

# Open split: per-detector TSVs from the published batch-9 archive (scores already on the 0-1, 0.5-cutoff scale).
for c in tintype:5 commfor:6 gpttrace:7 swinv2:8; do
  n=${c%%:*}; k=${c##*:}
  gzip -dc docs/results/batch9-gpttrace/scores.tsv.gz | awk -F'\t' -v k=$k 'NR>1 {print $1"\t"$k}' > "$O/open.$n.tsv"
done
python3 -I $T score docs/benchmark/v1/open.tsv.gz --save-files --out "$O/open.json" \
  --system "Tintype v0.4.0=$O/open.tintype.tsv" --system "GPTTrace=$O/open.gpttrace.tsv" \
  --system "Community Forensics=$O/open.commfor.tsv" --system "SwinV2=$O/open.swinv2.tsv" > "$O/open.md"

python3 -I $T score docs/benchmark/v1/blind.tsv.gz --save-files --out "$O/blind.json" \
  --system "Tintype v0.4.0=$S/tintype.jsonl" --system "GPTTrace=$S/gpttrace.jsonl" \
  --system "Community Forensics=$S/commfor.tsv" --system "SwinV2=$S/swinv2.tsv" \
  --time "Tintype v0.4.0=$(secs tintype)" --time "GPTTrace=$(secs gpttrace)" \
  --time "Community Forensics=$(secs commfor)" --time "SwinV2=$(secs swinv2)" > "$O/blind.md"

# Blind against open, every detector: pooled, then on the sets present in both splits.
python3 -I $T diff "$O/blind.json" "$O/open.json" --system "Tintype v0.4.0" --system "GPTTrace" \
  --system "Community Forensics" --system "SwinV2" --out "$O/diff-pooled.json" > "$O/diff-pooled.txt"
common=$(python3 -c "
import json; a=json.load(open('$O/blind.json')); b=json.load(open('$O/open.json'))
print(','.join(sorted(set(a['files_detail']['sets']) & set(b['files_detail']['sets']))))")
python3 -I $T diff "$O/blind.json" "$O/open.json" --system "Tintype v0.4.0" --system "GPTTrace" \
  --system "Community Forensics" --system "SwinV2" --sets "$common" --out "$O/diff-common.json" > "$O/diff-common.txt"
cat "$O/blind.md" "$O/diff-pooled.txt" "$O/diff-common.txt"
