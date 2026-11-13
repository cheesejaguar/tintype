#!/bin/sh
# Run from the repository root after cargo build --release --locked --bin tintype.
set -eu

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    printf '%s\n' 'Usage: scripts/pack.sh platform output-dir [binary]' >&2
    exit 1
fi
platform=$1
out=$2
binary=${3:-target/release/tintype}
case "$platform" in
    macos-arm64|linux-x86_64|linux-arm64) ;;
    *) printf 'Unsupported platform: %s\n' "$platform" >&2; exit 1 ;;
esac

mkdir -p "$out"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
trap 'exit 1' HUP INT TERM
install -m 755 "$binary" "$stage/tintype"
cp LICENSE-MIT LICENSE-APACHE NOTICE.md "$stage/"
COPYFILE_DISABLE=1 tar -czf "$out/tintype-$platform.tar.gz" -C "$stage" tintype LICENSE-MIT LICENSE-APACHE NOTICE.md
