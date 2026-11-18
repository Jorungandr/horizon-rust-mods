#!/usr/bin/env bash
# Run native.py check for the three ported mods inside a Linux container.
# The host proxy is a loopback address, so we use --network host so the
# container can reach it. Override PROXY_HOST if host networking is unavailable.
set -uo pipefail

STAGING="G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/horizon-mods-staging"
IMAGE="rust:1.90-slim"
PROXY="http://127.0.0.1:60145"

MODE="${1:-check}"   # check | probe

if [ "$MODE" = "probe" ]; then
  CMD='
    set -e
    echo "=== base tools ==="
    which cargo rustc || true
    cargo --version || true
    echo "=== connectivity (via proxy) ==="
    curl -sS -o /dev/null -w "github: %{http_code}\n" --max-time 25 https://github.com || echo "github FAILED"
    curl -sS -o /dev/null -w "codeload: %{http_code}\n" --max-time 25 https://codeload.github.com/horizon-rust/horizon || echo "codeload FAILED"
    echo "PROBE DONE"
  '
else
  CMD='
    set -e
    export DEBIAN_FRONTEND=noninteractive
    echo "=== installing build deps ==="
    apt-get update -qq
    apt-get install -y -qq python3 python3-pip rsync curl ca-certificates git >/dev/null 2>&1
    echo "python: $(python3 --version 2>&1)"
    echo "rsync: $(rsync --version 2>&1 | head -1)"
    echo "=== native.py check all ==="
    cd /workspace
    python3 native/tools/native.py check all
    echo "NATIVE CHECK DONE"
  '
fi

docker run --rm \
  --network host \
  -v "$STAGING:/workspace" \
  -w /workspace \
  -e HTTP_PROXY="$PROXY" -e HTTPS_PROXY="$PROXY" \
  -e http_proxy="$PROXY" -e https_proxy="$PROXY" \
  -e NO_PROXY="localhost,127.0.0.1" \
  -e no_proxy="localhost,127.0.0.1" \
  "$IMAGE" bash -lc "$CMD"
