#!/bin/sh
set -eu
cd "${1:?Usage: scripts/checksums.sh artifact-dir}"
set --
for file in tintype-*.tar.gz tintype-*.zip; do
    if [ -e "$file" ]; then set -- "$@" "$file"; fi
done
[ "$#" -gt 0 ] || { printf '%s\n' 'No release archives found' >&2; exit 1; }
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$@" > SHA256SUMS
else
    shasum -a 256 "$@" > SHA256SUMS
fi
