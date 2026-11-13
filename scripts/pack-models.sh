#!/bin/sh
# Run from the repository root, once per release.
set -eu
out=${1:?Usage: scripts/pack-models.sh output-dir [binary]}
binary=${2:-target/release/tintype}
"$binary" verify-models
mkdir -p "$out"
COPYFILE_DISABLE=1 tar -czf "$out/tintype-models.tar.gz" \
    models/LICENSE.md models/tintype.json models/tintype.safetensors \
    models/tintype-video.json models/tintype-video.safetensors
