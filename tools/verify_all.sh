#!/usr/bin/env bash
# Headless verification of all six injected mods: the three chat ports, the two
# HUD overlays and Auto Reconnect's settings.
set -o pipefail
ROOT="/g/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports"
GAME="$ROOT/build-win/game-port"
CTL="$ROOT/build-win/horizon-src/target/fast/horizon-ctl.exe"
CLIENT="$ROOT/build-win/horizon-src/target/fast/horizon-client.exe"
RES="$GAME/.local/horizon-client/26.3"
GAMEDIR="$ROOT/build-win/game-port/gamedata"
HOME_DIR="$ROOT/build-win/game-port/gamelogs"
SOCK="$ROOT/build-win/game-port/control.sock"
LOG="$ROOT/build-win/verify-all.log"
MODDATA="$GAMEDIR/horizon-mods/.horizon-mod-data"

if [ ! -f "$CLIENT" ]; then echo "client not built"; exit 1; fi
if [ ! -f "$CTL" ]; then CTL="/c/Users/ASUS/AppData/Roaming/horizon-launcher/versions/nightly-2026-11-13-9f42064/horizon-ctl.exe"; fi
mkdir -p "$GAMEDIR" "$HOME_DIR"

# Start from the mods' defaults, so the run also covers the first-launch path.
for m in chattimestamps coords durability autoreconnect; do rm -rf "$MODDATA/$m"; done
cp -f "$CLIENT" "$GAME/horizon-client.exe"
: > "$LOG"
echo "ctl = $CTL" | tee -a "$LOG"

C() { "$CTL" --socket "$SOCK" --timeout 180 "$@" 2>&1 | tee -a "$LOG"; }

echo "=== launch ===" | tee -a "$LOG"
C launch --game "$GAME/horizon-client.exe" --world "Ports Proof" --wait \
    --resources "$RES" --game-dir "$GAMEDIR" --home "$HOME_DIR" \
    --render-distance 4 --test-terrain
C wait-ready

echo "=== the three chat ports still work ===" | tee -a "$LOG"
C chat "hello ports"
C chat "see you at <t:1704067200:t> ok"
C messages 4

echo "=== coords HUD (default: top left) ===" | tee -a "$LOG"
C wait 10
C screenshot "$ROOT/build-win/hud-coords.png"

echo "=== equip worn gear for the durability HUD ===" | tee -a "$LOG"
C command "gamemode creative"
C command "item replace entity @s armor.head with minecraft:diamond_helmet[minecraft:damage=200]"
C command "item replace entity @s armor.chest with minecraft:netherite_chestplate[minecraft:damage=100]"
C command "item replace entity @s armor.legs with minecraft:iron_leggings[minecraft:damage=60]"
C command "item replace entity @s armor.feet with minecraft:golden_boots[minecraft:damage=70]"
C command "item replace entity @s weapon.mainhand with minecraft:diamond_pickaxe[minecraft:damage=1400]"
C command "item replace entity @s weapon.offhand with minecraft:shield"
C wait 10
C inventory
C screenshot "$ROOT/build-win/hud-durability.png"

echo "=== Options > Mods lists all six ===" | tee -a "$LOG"
C options mods
C ui
C screenshot "$ROOT/build-win/mods-page.png"

echo "=== quit ===" | tee -a "$LOG"
C quit
echo "=== done ===" | tee -a "$LOG"
