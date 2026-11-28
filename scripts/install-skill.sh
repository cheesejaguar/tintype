#!/bin/sh
set -eu

fail() {
    printf 'Tintype skill: %s\n' "$*" >&2
    exit 1
}

install_skill() (
    dest="$1/skills/tintype"
    mkdir -p "$dest"
    [ ! -d "$dest/SKILL.md" ] || fail "$dest/SKILL.md is a directory"
    staged=$(mktemp "$dest/.SKILL.md.XXXXXXXX")
    trap 'rm -f "$staged"' EXIT
    trap 'exit 1' HUP INT TERM
    cat "$download" > "$staged"
    mv -f "$staged" "$dest/SKILL.md"
    printf 'Installed %s\n' "$dest/SKILL.md"
)

agent=${1-both}
[ "$#" -le 1 ] || fail 'usage: sh install-skill.sh [claude-code|codex|both]'
case "$agent" in
    claude-code|codex|both) ;;
    --help) printf '%s\n' 'Usage: sh install-skill.sh [claude-code|codex|both]'; exit 0 ;;
    *) fail 'usage: sh install-skill.sh [claude-code|codex|both]' ;;
esac
: "${HOME:?HOME must be set}"

source=https://raw.githubusercontent.com/cheesejaguar/tintype/main/skills/tintype/SKILL.md
download=$(mktemp)
trap 'rm -f "$download"' EXIT
trap 'exit 1' HUP INT TERM
curl --proto '=https' --proto-redir '=https' --connect-timeout 10 --max-time 30 --retry 2 -fsSL -o "$download" "$source"
[ -s "$download" ] || fail 'the downloaded skill is empty'

if [ "$agent" != codex ]; then
    install_skill "${CLAUDE_CONFIG_DIR:-"$HOME/.claude"}"
fi
if [ "$agent" != claude-code ]; then
    install_skill "$HOME/.agents"
fi
printf '%s\n' 'Start a new agent session to use Tintype.'
