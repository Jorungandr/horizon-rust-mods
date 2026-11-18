#!/usr/bin/env bash
# Settings pages of the three new ports: open each, flip one button, read the
# labels and the files back.
set -o pipefail
ROOT="/g/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports"
GAME="$ROOT/build-win/game-port"
CTL="$ROOT/build-win/horizon-src/target/fast/horizon-ctl.exe"
CLIENT="$ROOT/build-win/horizon-src/target/fast/horizon-client.exe"
RES="$GAME/.local/horizon-client/26.3"
GAMEDIR="$ROOT/build-win/game-port/gamedata"
HOME_DIR="$ROOT/build-win/game-port/gamelogs"
SOCK="$ROOT/build-win/game-port/control.sock"
LOG="$ROOT/build-win/verify-settings2.log"
MODDATA="$GAMEDIR/horizon-mods/.horizon-mod-data"

for m in coords durability autoreconnect; do rm -rf "$MODDATA/$m"; done
cp -f "$CLIENT" "$GAME/horizon-client.exe"
: > "$LOG"
C() { "$CTL" --socket "$SOCK" --timeout 180 "$@" 2>&1 | tee -a "$LOG"; }

echo "=== launch ===" | tee -a "$LOG"
C launch --game "$GAME/horizon-client.exe" --world "Ports Proof" --wait \
    --resources "$RES" --game-dir "$GAMEDIR" --home "$HOME_DIR" \
    --render-distance 4 --test-terrain
C wait-ready

echo "=== coords settings ===" | tee -a "$LOG"
C options mods
C wait 10
C click settings-2
C wait 10
C ui
C screenshot "$ROOT/build-win/coords-settings.png"
C click-button "Position: Top left"
C wait 10
C ui
C click-button "Decimals: 1"
C wait 10
C ui
C screenshot "$ROOT/build-win/coords-settings-clicked.png"
C click-button "Done"

echo "=== durability settings ===" | tee -a "$LOG"
C wait 10
C click settings-3
C wait 10
C ui
C screenshot "$ROOT/build-win/durability-settings.png"
C click-button "Damaged only: ON"
C wait 10
C ui
C click-button "Done"

echo "=== auto reconnect settings ===" | tee -a "$LOG"
C wait 10
C click settings-4
C wait 10
C ui
C screenshot "$ROOT/build-win/autoreconnect-settings.png"
C click-button "Delay: 5s"
C wait 10
C ui
C click-button "Max attempts: 3"
C wait 10
C ui
C click-button "Done"

echo "=== stored files ===" | tee -a "$LOG"
for d in coords durability autoreconnect; do
    echo "--- $d" | tee -a "$LOG"
    for f in "$MODDATA/$d"/*; do
        [ -e "$f" ] || continue
        printf '%s = ' "$(basename "$f")"; cat "$f"; echo
    done | tee -a "$LOG"
done

echo "=== quit ===" | tee -a "$LOG"
C quit
echo "=== done ===" | tee -a "$LOG"
