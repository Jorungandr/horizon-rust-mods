# Notices and third-party credits

## Ported mods

- **WhenChat** — Copyright (c) ErikEdits. MIT license. Source: https://github.com/ErikEdits/WhenChat
  Ported to Rust in `mods/whenchat-horizon/`; original copyright and license retained there.
- **Plague's Chat Timestamps** — Copyright (c) PlagueTR. MIT license. Source: https://github.com/PlagueTR/ChatTimestamps
  Ported to Rust in `mods/chattimestamps-horizon/`; original copyright and license retained there.
- **Time Formatter** — Copyright (c) 2025 M3z0id. MIT license. Source: https://github.com/m3z0id/TimeFormatter
  Ported to Rust in `mods/timeformatter-horizon/`; original copyright and license retained there.

Each port is a subset: the core chat behavior was transplanted; GUI config screens
and Fabric API dependencies were not. Details are in each project's NOTICE.md.

## Original mods

Six mods are original to this project (`mods/ports/`): Coordinates HUD, Durability
HUD, Auto Reconnect, Chat Highlight, FPS HUD and Item Tooltip+. They are MIT,
copyright (c) 2026 Jorungandr — see the top-level `LICENSE`.

## Horizon

The compiled client in the release binary is built from the Horizon project,
https://playhorizon.dev — LGPL-3.0-or-later. Source for the pinned revision used
by the injection build is referenced in `docs/WINDOWS-BUILD.md`. If you
redistribute the binary, comply with the LGPL (provide the corresponding source,
allow relinking, keep license texts). The mod sources and build scripts needed to
reproduce the build are in this repository.

## Minecraft

Minecraft is a trademark of Mojang Studios and Microsoft. This repository and
its release contain no Minecraft code, textures, sounds or other game assets.
The `PREPARE_RESOURCES.cmd` script imports them from the user's own legitimately
acquired copy of Minecraft Java Edition 26.3.
