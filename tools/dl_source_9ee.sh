#!/usr/bin/env bash
# Download the pinned Horizon source INSIDE the container using curl+Range.
# Verified: gh-proxy honours Range (HTTP 206) at ~246KB/s from inside the container.
set -uo pipefail
REV=9ee13cd038559db05de6a4ced4496028ce409564
URL="https://gh-proxy.com/https://codeload.github.com/horizon-rust/horizon/tar.gz/$REV"
TOTAL=58827887
CHUNK=4194304            # 4 MiB
N=$(( (TOTAL + CHUNK - 1) / CHUNK ))   # 15 chunks
DIR=/workspace/native/target/horizon-src-9ee.parts
OUT=/workspace/native/target/horizon-src-9ee.tar.gz

mkdir -p "$DIR"
echo "total=$TOTAL chunks=$N" >&2

fetch() {
  local i=$1 s=$2 e=$3 f="$DIR/$(printf '%05d' "$i")"
  if [ -f "$f" ] && [ "$(stat -c %s "$f")" -eq "$((e-s+1))" ]; then
    return 0
  fi
  for t in 1 2 3 4 5 6; do
    if curl -sS --max-time 90 -r "${s}-${e}" -o "$f" "$URL" 2>/dev/null; then
      if [ "$(stat -c %s "$f" 2>/dev/null)" -eq "$((e-s+1))" ]; then
        return 0
      fi
    fi
    sleep $((t*2))
  done
  return 1
}

fail=0
for i in $(seq 0 $((N-1))); do
  s=$((i*CHUNK)); e=$((s+CHUNK-1)); [ "$e" -ge "$TOTAL" ] && e=$((TOTAL-1))
  fetch "$i" "$s" "$e" || { echo "chunk $i FAILED" >&2; fail=1; }
  done=$(find "$DIR" -type f -size +0c 2>/dev/null | wc -l)
  echo "chunk $i/$((N-1)) done (have $done/$N)" >&2
done

[ "$fail" -ne 0 ] && { echo "SOME CHUNKS FAILED" >&2; exit 1; }

cat $(for i in $(seq 0 $((N-1))); do echo "$DIR/$(printf '%05d' "$i")"; done) > "$OUT"
SIZE=$(stat -c %s "$OUT")
echo "assembled size=$SIZE expect=$TOTAL" >&2
[ "$SIZE" -ne "$TOTAL" ] && { echo "SIZE MISMATCH" >&2; exit 1; }
gzip -t "$OUT" && echo "GZIP OK" >&2 || { echo "GZIP BAD" >&2; exit 1; }
echo "SOURCE READY: $OUT" >&2
