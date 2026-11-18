#!/usr/bin/env bash
# Settings page round-trip: open it, flip "On hover only", read the labels back.
set -o pipefail
ROOT="/g/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports"
GAME="$ROOT/build-win/game-port"
CTL="$ROOT/build-win/horizon-src/target/fast/horizon-ctl.exe"
CLIENT="$ROOT/build-win/horizon-src/target/fast/horizon-client.exe"
RES="$GAME/.local/horizon-client/26.3"
GAMEDIR="$ROOT/build-win/game-port/gamedata"
HOME_DIR="$ROOT/build-win/game-port/gamelogs"
SOCK="$ROOT/build-win/game-port/control.sock"
LOG="$ROOT/build-win/verify-settings.log"
SETTINGS="$GAMEDIR/horizon-mods/.horizon-mod-data/chattimestamps"

rm -rf "$SETTINGS"
cp -f "$CLIENT" "$GAME/horizon-client.exe"
: > "$LOG"
C() { "$CTL" --socket "$SOCK" --timeout 180 "$@" 2>&1 | tee -a "$LOG"; }

echo "=== launch ===" | tee -a "$LOG"
C launch --game "$GAME/horizon-client.exe" --world "Ports Proof" --wait \
    --resources "$RES" --game-dir "$GAMEDIR" --home "$HOME_DIR" \
    --render-distance 4 --test-terrain

echo "=== options mods ===" | tee -a "$LOG"
C options mods
C wait 10
C ui
echo "=== click Settings ===" | tee -a "$LOG"
C click-button "Settings"
C wait 10
C ui
C screenshot "$ROOT/build-win/chattimestamps-settings.png"
echo "=== click 'On hover only: OFF' ===" | tee -a "$LOG"
C click-button "On hover only: OFF"
C wait 10
C ui
echo "=== stored files ===" | tee -a "$LOG"
for f in "$SETTINGS"/*; do printf '%s = ' "$(basename "$f")"; cat "$f"; echo; done | tee -a "$LOG"
echo "=== quit ===" | tee -a "$LOG"
C quit
echo "=== done ===" | tee -a "$LOG"
