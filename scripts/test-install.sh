#!/bin/sh
# Exercise the installer offline using the exact archives built by CI.
set -eu
case "$1" in
    /*) TINTYPE_TEST_ASSETS=$1 ;;
    *) TINTYPE_TEST_ASSETS="$PWD/$1" ;;
esac
TINTYPE_TEST_VERSION="v$(awk -F'"' '/^version =/ { print $2; exit }' Cargo.toml)"
export TINTYPE_TEST_ASSETS TINTYPE_TEST_VERSION
unset TINTYPE_MODEL_DIR TINTYPE_VERSION
installer="$PWD/scripts/install.sh"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
trap 'exit 1' HUP INT TERM
mkdir "$stage/tools" "$stage/elsewhere"
cat > "$stage/tools/curl" <<'MOCK'
#!/bin/sh
set -eu
case "$*" in
    *' https://github.com/cheesejaguar/tintype/releases/latest')
        printf '%s' "https://github.com/cheesejaguar/tintype/releases/tag/$TINTYPE_TEST_VERSION"
        exit 0
        ;;
esac
while [ "$1" != "-o" ]; do shift; done
file=${3##*/}
case "$3" in
    "https://github.com/cheesejaguar/tintype/releases/download/$TINTYPE_TEST_VERSION/$file") ;;
    *) exit 1 ;;
esac
cp "$TINTYPE_TEST_ASSETS/$file" "$2"
if [ "${TINTYPE_TEST_CORRUPT:-}" = yes ] && [ "$file" = "tintype-models.tar.gz" ]; then
    printf 'bad bytes' >> "$2"
fi
MOCK
chmod +x "$stage/tools/curl"
export PATH="$stage/tools:$PATH"
export TINTYPE_INSTALL_DIR="$stage/bin with spaces"
cd "$stage/elsewhere"
sh "$installer"
launcher="$TINTYPE_INSTALL_DIR/tintype"
"$launcher" demo --device cpu --json > result.jsonl
test "$(wc -l < result.jsonl | tr -d ' ')" = 1
awk '/"verdict":"(ai-generated|likely-real)"/ { found = 1 } END { exit !found }' result.jsonl
previous=$(readlink "$launcher")
if TINTYPE_TEST_CORRUPT=yes sh "$installer" > corrupt.log 2>&1; then
    printf '%s\n' 'Corrupt model archive was accepted' >&2
    exit 1
fi
awk '/checksum mismatch/ { found = 1 } END { exit !found }' corrupt.log
test "$(readlink "$launcher")" = "$previous"
TINTYPE_VERSION="$TINTYPE_TEST_VERSION" sh "$installer" --no-models
"$launcher" --version
test ! -d "$TINTYPE_INSTALL_DIR/$(dirname "$(readlink "$launcher")")/models"
