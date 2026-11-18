#!/usr/bin/env python3
"""Make an isolated game folder for the injected client, reusing prepared assets."""
import pathlib, shutil, sys

V = pathlib.Path(r"C:/Users/ASUS/AppData/Roaming/horizon-launcher/versions/nightly-2026-11-13-9f42064")
DST = pathlib.Path(r"G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/build-win/game-port")
EXE = pathlib.Path(r"G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/build-win/horizon-src/target/fast/horizon-client.exe")

if "--only-exe" not in sys.argv:
    if DST.exists():
        shutil.rmtree(DST)
    DST.mkdir(parents=True)
    for item in V.iterdir():
        if item.name == "horizon-client.exe":
            continue
        target = DST / item.name
        if item.is_dir():
            shutil.copytree(item, target)
        else:
            shutil.copy2(item, target)
    print("copied install ->", DST)

if EXE.is_file():
    shutil.copy2(EXE, DST / "horizon-client.exe")
    print("installed built client ->", DST / "horizon-client.exe")
else:
    print("built client not ready:", EXE)
