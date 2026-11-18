#!/usr/bin/env bash
# Headless verification of the three injected chat mods.
set -o pipefail
ROOT="/g/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports"
GAME="$ROOT/build-win/game-port"
CTL="$ROOT/build-win/horizon-src/target/fast/horizon-ctl.exe"
CLIENT="$ROOT/build-win/horizon-src/target/fast/horizon-client.exe"
RES="$GAME/.local/horizon-client/26.3"
GAMEDIR="$ROOT/build-win/game-port/gamedata"
HOME_DIR="$ROOT/build-win/game-port/gamelogs"
SOCK="$ROOT/build-win/game-port/control.sock"
LOG="$ROOT/build-win/verify.log"
SETTINGS="$GAMEDIR/horizon-mods/.horizon-mod-data/chattimestamps"

if [ ! -f "$CLIENT" ]; then echo "client not built"; exit 1; fi
if [ ! -f "$CTL" ]; then CTL="/c/Users/ASUS/AppData/Roaming/horizon-launcher/versions/nightly-2026-11-13-9f42064/horizon-ctl.exe"; fi
mkdir -p "$GAMEDIR" "$HOME_DIR"
rm -rf "$SETTINGS"
cp -f "$CLIENT" "$GAME/horizon-client.exe"
: > "$LOG"
echo "ctl = $CTL" | tee -a "$LOG"
echo "game = $GAME/horizon-client.exe" | tee -a "$LOG"

C() { "$CTL" --socket "$SOCK" --timeout 180 "$@" 2>&1 | tee -a "$LOG"; }

echo "=== launch ===" | tee -a "$LOG"
C launch --game "$GAME/horizon-client.exe" --world "Ports Proof" --wait \
    --resources "$RES" --game-dir "$GAMEDIR" --home "$HOME_DIR" \
    --render-distance 4 --test-terrain

echo "=== chat: plain + discord timestamp ===" | tee -a "$LOG"
C chat "hello ports"
C chat "see you at <t:1704067200:t> ok"

echo "=== messages (default: prefix mode) ===" | tee -a "$LOG"
C messages 6
C screenshot "$ROOT/build-win/ports-proof.png"

echo "=== settings: hover mode on ===" | tee -a "$LOG"
mkdir -p "$SETTINGS"
printf 'true' > "$SETTINGS/enable_hover"
C chat "hover mode line"
C messages 3

echo "=== settings: mod off ===" | tee -a "$LOG"
printf 'false' > "$SETTINGS/enable_mod"
C chat "mod off line"
C messages 2

echo "=== settings page (Options > Mods) ===" | tee -a "$LOG"
printf 'true' > "$SETTINGS/enable_mod"
printf 'false' > "$SETTINGS/enable_hover"
C options mods
C ui
C screenshot "$ROOT/build-win/mods-page.png"

echo "=== quit ===" | tee -a "$LOG"
C quit
echo "=== done ===" | tee -a "$LOG"
