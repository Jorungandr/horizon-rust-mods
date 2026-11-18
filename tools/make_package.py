#!/usr/bin/env python3
"""Package the sources, tools, docs and proof screenshots into dist/."""
import pathlib, zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIST = ROOT / "dist"
OUT = DIST / "horizon-ports-windows-verified.zip"

INCLUDE_DIRS = [
    "whenchat-horizon",
    "chattimestamps-horizon",
    "timeformatter-horizon",
    "ports",
    "native-v2",
    "tools",
]
INCLUDE_FILES = [
    "README.md",
    "WINDOWS-BUILD.md",
    "TESTING.md",
    "DELIVERY.md",
    "DOCKER-BUILD.md",
    "PLAY.cmd",
]
PROOF = [
    "build-win/ports-proof.png",
    "build-win/hud-coords.png",
    "build-win/hud-durability.png",
    "build-win/mods-page.png",
    "build-win/chattimestamps-settings.png",
    "build-win/coords-settings.png",
    "build-win/durability-settings.png",
    "build-win/autoreconnect-settings.png",
    "build-win/verify-stdout.log",
    "build-win/verify-settings-stdout.log",
    "build-win/verify-final-stdout.log",
    "build-win/verify-settings2-stdout.log",
]
SKIP = {".git", "target", "__pycache__", "horizon-mods-staging"}

DIST.mkdir(exist_ok=True)
count = 0
with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for name in INCLUDE_DIRS:
        base = ROOT / name
        if not base.is_dir():
            continue
        for p in base.rglob("*"):
            if any(part in SKIP for part in p.parts):
                continue
            if p.is_file():
                z.write(p, p.relative_to(ROOT).as_posix())
                count += 1
    for name in INCLUDE_FILES + PROOF:
        p = ROOT / name
        if p.is_file():
            z.write(p, p.as_posix())
            count += 1
print(f"{count} files -> {OUT} ({OUT.stat().st_size / 1024:.0f} KiB)")
