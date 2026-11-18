# Horizon Rust Mods

Six native Rust mods for [Horizon](https://playhorizon.dev) (Minecraft Java Edition rewritten in Rust): three MIT-licensed Java-mod ports and three originals. All six were built into a real `horizon-client.exe` and verified in-game.

> Horizon is an unofficial fan project, not affiliated with Mojang Studios or Microsoft.

## The six mods

| Mod | Origin / License | What it does |
|---|---|---|
| `whenchat-horizon` | [ErikEdits/WhenChat](https://github.com/ErikEdits/WhenChat) · MIT | Prefixes every chat line with a local `[HH:mm:ss]` timestamp |
| `chattimestamps-horizon` | [PlagueTR/ChatTimestamps](https://github.com/PlagueTR/ChatTimestamps) · MIT | Configurable timestamps: 12/24h, date, color, hover; options page |
| `timeformatter-horizon` | [m3z0id/TimeFormatter](https://github.com/m3z0id/TimeFormatter) · MIT | Converts Discord-style `<t:1704067200:t>` in chat to local time |
| `ports/coords.rs` | original | Coordinates HUD: X/Y/Z, facing, dimension, configurable corner |
| `ports/durability.rs` | original | Equipment durability HUD with color thresholds and item icons |
| `ports/autoreconnect.rs` | original | Auto-reconnect on unexpected disconnect, with delay and retry limit |

In-game verification (screenshots in [`evidence/`](evidence/)):

```
[12:27:29] [12:27:29] <Player32510> see you at 08:00 ok
```

The first gray timestamp is Chat Timestamps, the second is WhenChat, and `08:00` is Time Formatter converting `<t:1704067200:t>`. The `Options > Mods` page lists all six, four with working `Settings` buttons that persist to disk.

## Repository layout

```
mods/                 original Java-port projects (mixins + offline verify)
  whenchat-horizon/
  chattimestamps-horizon/
  timeformatter-horizon/
  ports/              three original mods, injection source
  native-v2/          new-format (horizon-mod.toml + Hooks trait) projects
tools/                inject / build / verify / package scripts
docs/                 build, test and delivery notes; launcher scripts
evidence/             real-game screenshots and verification logs
```

## What actually runs, and why

Horizon's official `horizon-launcher native-build` requires a mod toolchain from `nightly-2026-11-13-9f42064`; no `sdk-*` release has ever been published. So the working build here uses an equivalent path: the mod sources are injected into the real Horizon `9ee13cd0` source tree and the whole `horizon-client` is compiled with a local Rust toolchain. The result is a complete game binary with the six mods built in.

The `mods/native-v2/` projects are written in the official new format and pass `cargo check`, but are "type-checked, awaiting toolchain". The `Host` API currently lacks `item max_damage` and a connect-to-server method, so **durability** and **auto-reconnect** only work fully in the injected form; **coords** works in both.

## Quick start

The [latest release](https://github.com/Jorungandr/horizon-rust-mods/releases/latest) ships a ready-to-run Windows package (compiled client + scripts + sources). After extracting:

1. Run `PREPARE_RESOURCES.cmd` once to import the 26.3 resources from **your own** Minecraft Java Edition (Horizon and this package contain no Minecraft assets).
2. Run `START_GAME.cmd` to launch with all six mods.

See [`docs/START_HERE.md`](docs/START_HERE.md) for the full walkthrough and [`docs/PROJECT-DETAILS.md`](docs/PROJECT-DETAILS.md) for the technical background.

To rebuild from source on Windows, see [`docs/WINDOWS-BUILD.md`](docs/WINDOWS-BUILD.md); for the Docker/Linux full-tree compile, see [`docs/TESTING.md`](docs/TESTING.md).

## License

- The three Java-port mods are MIT; their original authors' copyright and notices are preserved in each directory's `LICENSE` and `NOTICE.md`.
- The three original mods and the tooling in this repository are MIT, copyright (c) 2026 Jorungandr — see [`LICENSE`](LICENSE) and [`NOTICE.md`](NOTICE.md).
- The release binary embeds Horizon, which is LGPL-3.0-or-later. If you redistribute the compiled package, follow Horizon's LGPL obligations; the corresponding source is in this repository.
- Minecraft is a trademark of Mojang and Microsoft. No Minecraft code, textures or sounds are bundled.
