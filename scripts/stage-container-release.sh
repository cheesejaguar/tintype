#!/usr/bin/env bash
# Run after downloading the attested release archives.
set -euo pipefail

if [[ $# != 3 ]]; then
    printf '%s\n' 'Usage: scripts/stage-container-release.sh linux-platform artifact-dir output-dir' >&2
    exit 1
fi
platform=$1
archives=$2
output=$3
case "$platform" in
    linux-x86_64|linux-arm64) ;;
    *) printf 'Unsupported platform: %s\n' "$platform" >&2; exit 1 ;;
esac
if [[ -e $output || -L $output ]]; then
    printf 'Output already exists: %s\n' "$output" >&2
    exit 1
fi

binary=tintype-$platform.tar.gz
models=tintype-models.tar.gz
(
    cd "$archives"
    for archive in "$binary" "$models"; do
        awk -v name="$archive" '
            $2 == name && length($1) == 64 && $1 ~ /^[0-9a-f]+$/ { print; n++ }
            END { if (n != 1) exit 1 }
        ' SHA256SUMS | sha256sum --check --strict -
    done
)

mkdir "$output"
tar -xzf "$archives/$binary" -C "$output" tintype LICENSE-MIT LICENSE-APACHE NOTICE.md
tar -xzf "$archives/$models" -C "$output" \
    models/LICENSE.md models/tintype.json models/tintype.safetensors \
    models/tintype-video.json models/tintype-video.safetensors
chmod 755 "$output/tintype" "$output/models"
chmod 644 "$output/"{LICENSE-MIT,LICENSE-APACHE,NOTICE.md} "$output/models/"*
