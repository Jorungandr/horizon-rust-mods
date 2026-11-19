#!/usr/bin/env bash
# Headless verification of all nine injected mods:
#   chat trio (whenchat/timestamps/formatter) + chat highlight,
#   coords/durability/fps HUDs, item tooltip+, auto reconnect.
set -o pipefail
ROOT="/g/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports"
GAME="$ROOT/build-win/game-port"
CTL="$ROOT/build-win/horizon-src/target/fast/horizon-ctl.exe"
CLIENT="$ROOT/build-win/horizon-src/target/fast/horizon-client.exe"
RES="$GAME/.local/horizon-client/26.3"
GAMEDIR="$ROOT/build-win/game-port/gamedata"
HOME_DIR="$ROOT/build-win/game-port/gamelogs"
SOCK="$ROOT/build-win/game-port/control.sock"
LOG="$ROOT/build-win/verify-nine.log"
MODDATA="$GAMEDIR/horizon-mods/.horizon-mod-data"

if [ ! -f "$CLIENT" ]; then echo "client not built"; exit 1; fi
mkdir -p "$GAMEDIR" "$HOME_DIR"

# Fresh defaults for every mod so first-launch paths are covered.
for m in whenchat chattimestamps timeformatter chathighlight coords durability fpshud itemtooltip autoreconnect; do
    rm -rf "$MODDATA/$m"
done
cp -f "$CLIENT" "$GAME/horizon-client.exe"
: > "$LOG"
echo "ctl = $CTL" | tee -a "$LOG"

C() { "$CTL" --socket "$SOCK" --timeout 180 "$@" 2>&1 | tee -a "$LOG"; }

echo "=== launch ===" | tee -a "$LOG"
C launch --game "$GAME/horizon-client.exe" --world "Ports Proof" --wait \
    --resources "$RES" --game-dir "$GAMEDIR" --home "$HOME_DIR" \
    --render-distance 4 --test-terrain
C wait-ready

echo "=== chat: timestamps + formatter + highlight ===" | tee -a "$LOG"
C chat "server look here important"
C chat "see you at <t:1704067200:t> ok"
C messages 4

echo "=== coords HUD (top left) + FPS HUD (bottom right) ===" | tee -a "$LOG"
C wait 10
C screenshot "$ROOT/build-win/hud-coords-fps.png"

echo "=== equip gear for durability HUD ===" | tee -a "$LOG"
C command "gamemode creative"
C command "item replace entity @s armor.head with minecraft:diamond_helmet[minecraft:damage=200]"
C command "item replace entity @s armor.feet with minecraft:golden_boots[minecraft:damage=70]"
C command "item replace entity @s weapon.mainhand with minecraft:diamond_pickaxe[minecraft:damage=1400]"
C wait 10
C inventory
C screenshot "$ROOT/build-win/hud-durability-nine.png"

echo "=== item tooltip+ : open inventory and hover a damaged pickaxe ===" | tee -a "$LOG"
C open-inventory
C wait 10
C ui
C screenshot "$ROOT/build-win/tooltip-inventory.png"
C close

echo "=== Options > Mods lists all nine ===" | tee -a "$LOG"
C options mods
C wait 10
C ui
C screenshot "$ROOT/build-win/mods-page-nine.png"

echo "=== new settings pages ===" | tee -a "$LOG"
# Settings buttons are ordered by the MODS list; 5=chathighlight, 7=fpshud, 8=itemtooltip
C click settings-5
C wait 10
C ui
C screenshot "$ROOT/build-win/chathighlight-settings.png"
C click-button "Colour: Gold"
C wait 10
C ui
C click-button "Done"

C wait 10
C click settings-7
C wait 10
C ui
C screenshot "$ROOT/build-win/fpshud-settings.png"
C click-button "Position: Bottom right"
C wait 10
C ui
C click-button "Done"

C wait 10
C click settings-8
C wait 10
C ui
C screenshot "$ROOT/build-win/itemtooltip-settings.png"
C click-button "Item ID: ON"
C wait 10
C ui
C click-button "Done"

echo "=== stored files ===" | tee -a "$LOG"
for d in chathighlight fpshud itemtooltip; do
    echo "--- $d" | tee -a "$LOG"
    for f in "$MODDATA/$d"/*; do
        [ -e "$f" ] || continue
        printf '%s = ' "$(basename "$f")"; cat "$f"; echo
    done | tee -a "$LOG"
done

echo "=== quit ===" | tee -a "$LOG"
C quit
echo "=== done ===" | tee -a "$LOG"
