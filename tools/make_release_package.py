#!/usr/bin/env python3
"""Build the ready-to-run Horizon Ports release zip in dist/.

Unlike make_package.py (source archive), this bundles the compiled injected
client and a self-contained game layout so that, after resources are prepared
from the user's own Minecraft, START_GAME.cmd just works.

Minecraft assets are intentionally NOT bundled: Horizon imports them from the
user's own copy of the game. A placeholder is left in the resources folder.
"""
import pathlib, zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIST = ROOT / "dist"
OUT = DIST / "Horizon-Ports-Rust-6mods.zip"
RELEASE = ROOT / "release"
GAME = ROOT / "build-win" / "game-port"
BUILD = ROOT / "build-win"

TOP = "Horizon-Ports-Rust"
SKIP_DIRS = {"target", "__pycache__", ".git", "horizon-mods-staging",
             "cargo-home", "check-target", "check-target2"}

# Files that must sit beside the client for it to run.
GAME_FILES = [
    "horizon-client.exe", "horizon-ctl.exe", "horizon-prepare.exe",
    "horizon-server.exe", "dxcompiler.dll",
    "LICENSE", "PLAY.txt", "DXC-NOTICE.txt", "DXC-LICENSE-LLVM.txt",
    "DXC-LICENSE-MS.txt", ".horizon-game-architecture.json", "research-lock.json",
]
# Directories copied verbatim (config, saves, logs stay isolated from the launcher).
GAME_DIRS = ["gamedata", "gamelogs", "mods", "crates", "exporters", "tools"]
# Source projects / tooling / docs shipped under source/.
SOURCE_DIRS = [
    "whenchat-horizon", "chattimestamps-horizon", "timeformatter-horizon",
    "ports", "native-v2", "tools",
]
SOURCE_FILES = ["README.md", "WINDOWS-BUILD.md", "TESTING.md",
                "DELIVERY.md", "DOCKER-BUILD.md", "PLAY.cmd"]
# Proof of the real-game verification.
EVIDENCE = [
    "ports-proof.png", "hud-coords.png", "hud-durability.png", "mods-page.png",
    "chattimestamps-settings.png", "coords-settings.png", "durability-settings.png",
    "autoreconnect-settings.png", "verify-final-stdout.log",
    "verify-settings2-stdout.log", "verify-stdout.log", "verify-settings-stdout.log",
]

count = 0
size = 0

def add(z, src: pathlib.Path, arc: str):
    global count, size
    if not src.is_file():
        return
    z.write(src, arc)
    count += 1
    size += src.stat().st_size

def add_tree(z, base: pathlib.Path, arc_prefix: str):
    for p in sorted(base.rglob("*")):
        if any(part in SKIP_DIRS for part in p.parts):
            continue
        if p.is_file():
            add(z, p, arc_prefix + "/" + p.relative_to(base).as_posix())

DIST.mkdir(exist_ok=True)
with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
    # 0. entry documents and scripts at archive root
    for name in ["START_HERE.md", "START_GAME.cmd", "PREPARE_RESOURCES.cmd"]:
        add(z, RELEASE / name, f"{TOP}/{name}")

    # 1. compiled game
    for name in GAME_FILES:
        add(z, GAME / name, f"{TOP}/game/{name}")
    for name in GAME_DIRS:
        d = GAME / name
        if d.is_dir():
            add_tree(z, d, f"{TOP}/game/{name}")

    # 2. resources placeholder (assets come from the user's own Minecraft)
    res = GAME / ".local" / "horizon-client" / "26.3"
    placeholder = ("This folder is where PREPARE_RESOURCES.cmd imports the 26.3\n"
                   "resources from YOUR OWN Minecraft Java Edition. Horizon and\n"
                   "this package contain no Minecraft assets. Run the script first.\n")
    z.writestr(f"{TOP}/game/.local/horizon-client/26.3/README.txt", placeholder)
    count += 1
    for empty in ["gamelogs", "mods"]:
        z.writestr(f"{TOP}/game/{empty}/.gitkeep", "")
        count += 1

    # 3. sources, tools and docs
    for name in SOURCE_DIRS:
        base = ROOT / name
        if base.is_dir():
            add_tree(z, base, f"{TOP}/source/{name}")
    for name in SOURCE_FILES:
        add(z, ROOT / name, f"{TOP}/source/{name}")

    # 4. verification evidence
    for name in EVIDENCE:
        add(z, BUILD / name, f"{TOP}/evidence/{name}")

print(f"{count} files, {size/1024/1024:.0f} MiB raw -> {OUT} ({OUT.stat().st_size/1024/1024:.0f} MiB zipped)")
