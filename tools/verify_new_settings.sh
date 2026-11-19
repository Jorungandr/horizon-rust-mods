#!/usr/bin/env bash
# Verify the two settings pages that sit below the fold in Options > Mods.
# The Mods list is scrollable; ctl's `ui` only reports currently visible rows,
# so we scroll down first to reach FPS HUD (settings-6) and Item Tooltip+ (settings-7).
set -o pipefail
ROOT="/g/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports"
GAME="$ROOT/build-win/game-port"
CTL="$ROOT/build-win/horizon-src/target/fast/horizon-ctl.exe"
RES="$GAME/.local/horizon-client/26.3"
GAMEDIR="$GAME/gamedata"
HOME_DIR="$GAME/gamelogs"
SOCK="$GAME/control.sock"
LOG="$ROOT/build-win/verify-new-settings.log"
MODDATA="$GAMEDIR/horizon-mods/.horizon-mod-data"

for m in fpshud itemtooltip; do rm -rf "$MODDATA/$m"; done
: > "$LOG"
C() { "$CTL" --socket "$SOCK" --timeout 180 "$@" >> "$LOG" 2>&1; echo "[$?] $*" | tee -a "$LOG"; }

echo "=== launch ===" | tee -a "$LOG"
C launch --game "$GAME/horizon-client.exe" --world "Ports Proof" --wait \
    --resources "$RES" --game-dir "$GAMEDIR" --home "$HOME_DIR" \
    --render-distance 4 --test-terrain

echo "=== Mods page + scroll down ===" | tee -a "$LOG"
C options mods
C scroll-at 130 120 -5
C ui

echo "=== FPS HUD settings (settings-6) ===" | tee -a "$LOG"
C click settings-6
C ui
C screenshot "$ROOT/build-win/fpshud-settings.png"
C click-button "Position: Bottom right"
C ui
C click-button "Detailed: ON"
C ui
C click-button "Done"

echo "=== Item Tooltip+ settings (settings-7) ===" | tee -a "$LOG"
# Done resets the list scroll to the top, so scroll down again first.
C scroll-at 130 120 -6
C ui
C click settings-7
C ui
C screenshot "$ROOT/build-win/itemtooltip-settings.png"
C click-button "Item id: ON"
C ui
C click-button "Done"

echo "=== stored files ===" | tee -a "$LOG"
for d in fpshud itemtooltip; do
    echo "--- $d" | tee -a "$LOG"
    for f in "$MODDATA/$d"/*; do
        [ -e "$f" ] || continue
        printf '%s = ' "$(basename "$f")"; cat "$f"; echo
    done | tee -a "$LOG"
done

echo "=== quit ===" | tee -a "$LOG"
C quit
echo "=== done ===" | tee -a "$LOG"
