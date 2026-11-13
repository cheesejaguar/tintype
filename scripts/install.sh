#!/bin/sh
set -eu

repo=https://github.com/cheesejaguar/tintype

fail() {
    printf 'Tintype: %s\n' "$*" >&2
    exit 1
}

fetch() {
    curl --proto '=https' --proto-redir '=https' --connect-timeout 10 --retry 2 -fsSL "$@"
}

verify() {
    archive=$1
    expected=$(awk -v file="$archive" '$2 == file { print $1 }' "$stage/SHA256SUMS")
    [ "${#expected}" -eq 64 ] || fail "missing checksum for $archive"
    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$stage/$archive")
    elif command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "$stage/$archive")
    else
        fail 'install sha256sum or shasum first'
    fi
    [ "${actual%% *}" = "$expected" ] || fail "checksum mismatch for $archive"
}

main() {
    models=with-models
    case "${1:-}" in
        '') ;;
        --no-models) models=code-only; shift ;;
        --help)
            printf '%s\n' 'Usage: sh install.sh [--no-models]' \
                'TINTYPE_VERSION=v0.1.0 pins a release.' \
                'TINTYPE_INSTALL_DIR sets the executable directory (default ~/.local/bin).'
            exit 0 ;;
        *) fail "unknown option: $1" ;;
    esac
    [ "$#" -eq 0 ] || fail "unexpected argument: $1"

    case "$(uname -s)/$(uname -m)" in
        Darwin/arm64) platform=macos-arm64 ;;
        Linux/x86_64) platform=linux-x86_64 ;;
        Linux/aarch64|Linux/arm64) platform=linux-arm64 ;;
        *) fail 'supported targets are Apple silicon macOS and glibc Linux (x86-64 and ARM64)' ;;
    esac

    version=${TINTYPE_VERSION:-}
    if [ -z "$version" ]; then
        latest=$(fetch --max-time 30 -o /dev/null -w '%{url_effective}' "$repo/releases/latest") ||
            fail 'could not resolve the latest release'
        case "$latest" in
            "$repo/releases/tag/"*) version=${latest##*/} ;;
            *) fail "no stable release is available at $repo/releases" ;;
        esac
    fi
    case "$version" in
        v[0-9]*) ;;
        *) fail 'TINTYPE_VERSION must be a release tag such as v0.1.0' ;;
    esac
    case "$version" in
        *[!a-zA-Z0-9._-]*) fail 'invalid release tag' ;;
    esac
    source="$repo/releases/download/$version"
    dest=${TINTYPE_INSTALL_DIR:-"$HOME/.local/bin"}
    case "$dest" in
        /*) ;;
        *) fail 'TINTYPE_INSTALL_DIR must be an absolute path' ;;
    esac
    storage="$dest/.tintype"
    mkdir -p "$storage"
    stage=$(mktemp -d "$storage/.install.XXXXXXXX")
    lock="$storage/.lock"
    locked=no
    trap 'if [ "$locked" = yes ]; then rmdir "$lock" || :; fi; rm -rf "$stage"' EXIT
    trap 'exit 1' HUP INT TERM
    mkdir "$lock" 2>/dev/null || fail "another installation may be running ($lock)"
    locked=yes

    bundle="tintype-$platform.tar.gz"
    fetch -o "$stage/SHA256SUMS" "$source/SHA256SUMS"
    fetch -o "$stage/$bundle" "$source/$bundle"
    verify "$bundle"
    mkdir "$stage/payload"
    tar -xzf "$stage/$bundle" -C "$stage/payload"
    if [ "$models" = with-models ]; then
        bundle=tintype-models.tar.gz
        fetch -o "$stage/$bundle" "$source/$bundle"
        verify "$bundle"
        tar -xzf "$stage/$bundle" -C "$stage/payload"
        "$stage/payload/tintype" verify-models --root "$stage/payload" >/dev/null
    fi
    "$stage/payload/tintype" --version

    if [ -e "$dest/tintype" ] || [ -L "$dest/tintype" ]; then
        existing=$(readlink "$dest/tintype" || true)
        case "$existing" in
            .tintype/*/tintype) ;;
            *) fail "$dest/tintype already exists. Choose another TINTYPE_INSTALL_DIR." ;;
        esac
    fi
    # Put the new version in place before changing the launcher.
    key="$version-$platform-$models-${stage##*.install.}"
    target="$storage/$key"
    mv "$stage/payload" "$target"
    ln -s ".tintype/$key/tintype" "$stage/launcher"
    mv -f "$stage/launcher" "$dest/tintype"

    printf '\nInstalled %s\n' "$dest/tintype"
    case ":$PATH:" in
        *":$dest:"*) ;;
        *) printf 'Add %s to your PATH or run the executable at that path.\n' "$dest" ;;
    esac
    if [ "$models" = with-models ]; then
        printf '%s\n' 'Run tintype demo or tintype detect photo.jpg.' \
            'Bundled model weights are CC BY-NC-SA 4.0.'
    fi
}

main "$@"
