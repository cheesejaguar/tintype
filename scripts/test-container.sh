#!/usr/bin/env bash
set -euo pipefail

bundled=${1:?Usage: scripts/test-container.sh bundled-image [code-only-image staged-model-directory]}
if [[ $# != 1 && $# != 3 ]]; then
    printf '%s\n' 'Provide the code-only image and model directory together.' >&2
    exit 1
fi

temp=$(mktemp -d)
container=
cleanup() {
    if [[ -n $container ]]; then
        docker logs "$container" || true
        docker rm -f "$container" >/dev/null || true
    fi
    rm -rf "$temp"
}
trap cleanup EXIT

check_demo() {
    docker run --rm "$@" demo --device cpu --json |
        jq -e '.fixture_label == "ai-generated" and (.score | type == "number")'
}

check_demo "$bundled"
docker run --rm "$bundled" verify-models --root /
if [[ $# == 3 ]]; then
    code_only=$2
    models=$(cd "$3" && pwd)
    docker run --rm --entrypoint /bin/sh "$code_only" -c 'test ! -e /models'
    check_demo --mount "type=bind,source=$models,target=/models,readonly" "$code_only"
fi

token=$(openssl rand -hex 24)
container=$(docker run --detach --init --env "TINTYPE_API_TOKEN=$token" -p 127.0.0.1::8080 "$bundled")
address=$(docker port "$container" 8080/tcp)
curl --fail --silent --show-error --retry 20 --retry-all-errors --retry-delay 1 \
    --max-time 2 --output "$temp/health.json" "http://$address/healthz"
jq -e '.status == "ok"' "$temp/health.json"
curl --fail --silent --show-error --max-time 100 -H "Authorization: Bearer $token" \
    -F 'file=@site/assets/demo.png' -F 'file=@tests/fixtures/video.mp4' \
    "http://$address/v1/batch" |
    jq -e '.results | length == 2 and .[0].kind == "image" and .[1].kind == "video" and all(.[]; .score | type == "number")'
docker stop --timeout 15 "$container" >/dev/null
test "$(docker inspect --format '{{.State.ExitCode}}' "$container")" = 0
