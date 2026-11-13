#!/bin/sh
set -eu
cd "${1:?Usage: scripts/checksums.sh artifact-dir}"
set -- tintype-*.tar.gz
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$@" > SHA256SUMS
else
    shasum -a 256 "$@" > SHA256SUMS
fi
