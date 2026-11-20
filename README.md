# Horizon Rust Mods

Nine native Rust mods for [Horizon](https://playhorizon.dev) (Minecraft Java Edition rewritten in Rust): three MIT-licensed Java-mod ports and six originals.

**Recommended:** eight of them are now available as [official-format source mods](native/) that you add from the Horizon launcher and build locally. The original injected full-client package remains in the releases for reference, but it cannot be installed with the launcher's "add mod" button.

> Horizon is an unofficial fan project, not affiliated with Mojang Studios or Microsoft.

## The nine mods

| Mod | Origin / License | What it does |
|---|---|---|
| `whenchat-horizon` | [ErikEdits/WhenChat](https://github.com/ErikEdits/WhenChat) · MIT | Prefixes every chat line with a local `[HH:mm:ss]` timestamp |
| `chattimestamps-horizon` | [PlagueTR/ChatTimestamps](https://github.com/PlagueTR/ChatTimestamps) · MIT | Configurable timestamps: 12/24h, date, color, hover; options page |
| `timeformatter-horizon` | [m3z0id/TimeFormatter](https://github.com/m3z0id/TimeFormatter) · MIT | Converts Discord-style `<t:1704067200:t>` in chat to local time |
| `ports/coords.rs` | original | Coordinates HUD: X/Y/Z, facing, dimension, configurable corner |
| `ports/durability.rs` | original | Equipment durability HUD with color thresholds and item icons |
| `ports/autoreconnect.rs` | original | Auto-reconnect on unexpected disconnect, with delay and retry limit |
| `ports/chat_highlight.rs` | original | Colours configured keywords in chat (colour, keyword pack, case) |
| `ports/fps_hud.rs` | original | FPS / slowest frame / 1% low overlay, configurable corner |
| `ports/item_tooltip.rs` | original | Appends item ID, durability and stack size to tooltips |

Official-format projects: [`native/`](native/) contains `whenchat`, `chattimestamps`, `timeformatter`, `chathighlight`, `coords`, `fpshud`, `durability` and `itemtooltip`. `autoreconnect` is only in the injected package because the official `Host` API has no connect-to-server method. See [`docs/NEW-FORMAT.md`](docs/NEW-FORMAT.md) for the launcher install flow, the official registry publishing steps and the porting limitation.

In-game verification (screenshots in [`evidence/`](evidence/)):

```
[12:27:29] [12:27:29] <Player32510> see you at 08:00 ok
[12:27:29] [12:27:29] <Player32510> server look here important
```

The first gray timestamp is Chat Timestamps, the second is WhenChat, and `08:00` is Time Formatter converting `<t:1704067200:t>`. In the second line the keywords `server`, `look here`, `important` are coloured gold by Chat Highlight. The `Options > Mods` page lists all nine (scroll down for the last two), seven with working `Settings` buttons that persist to disk.

## Repository layout

```
native/               official SDK source mods (add these from the launcher) — recommended
mods/                 original Java-port projects (mixins + offline verify)
  whenchat-horizon/
  chattimestamps-horizon/
  timeformatter-horizon/
  ports/              six original mods, injection source
  native-v2/          earlier new-format experiments, superseded by native/
tools/                inject / build / verify / package scripts
docs/                 build, test and delivery notes; launcher scripts
evidence/             real-game screenshots and verification logs
```

## Two distributions

**Official native mods (`native/`, current).** Each project has `horizon-mod.toml` with `api = "0.2"`, `Cargo.toml` and `src/`. The launcher compiles them with its own Rust against the pinned SDK and loads the DLLs into the unmodified game. This is the only form that supports the launcher's Mods page. See [`native/README.md`](native/README.md) for installation and portability details.

**Injected full client (legacy).** The older release packages inject the sources into a Horizon source tree and rebuild the whole `horizon-client.exe`. The nine mods were verified in-game this way, and it is still useful as a reference implementation, but players must run its own `START_GAME.cmd` instead of the launcher. `autoreconnect` exists only here.

Earlier notes claimed no SDK release existed. That is no longer true: the official SDK is published by Horizon, and all eight `native/` projects compile through `horizon-launcher native-build`. The SDK's `max_damage` API covers durability; only connecting to a server is still missing.

## Quick start

### Install source mods with the launcher (recommended)

Download or clone this repository, then add the folders under [`native/`](native/) from the Horizon launcher's Mods page and press Play. The launcher compiles them locally; see [`native/README.md`](native/README.md).

### Legacy full-client package

The [latest release](https://github.com/Jorungandr/horizon-rust-mods/releases/latest) also ships a ready-to-run Windows package (compiled client + scripts + sources). After extracting:

1. Run `PREPARE_RESOURCES.cmd` once to import the 26.3 resources from **your own** Minecraft Java Edition (Horizon and this package contain no Minecraft assets).
2. Run `START_GAME.cmd` to launch with all nine mods.

See [`docs/START_HERE.md`](docs/START_HERE.md) for the full walkthrough and [`docs/PROJECT-DETAILS.md`](docs/PROJECT-DETAILS.md) for the technical background.

To rebuild from source on Windows, see [`docs/WINDOWS-BUILD.md`](docs/WINDOWS-BUILD.md); for the Docker/Linux full-tree compile, see [`docs/TESTING.md`](docs/TESTING.md).

## License

- The three Java-port mods are MIT; their original authors' copyright and notices are preserved in each directory's `LICENSE` and `NOTICE.md`.
- The six original mods and the tooling in this repository are MIT, copyright (c) 2026 Jorungandr — see [`LICENSE`](LICENSE) and [`NOTICE.md`](NOTICE.md).
- The release binary embeds Horizon, which is LGPL-3.0-or-later. If you redistribute the compiled package, follow Horizon's LGPL obligations; the corresponding source is in this repository.
- Minecraft is a trademark of Mojang and Microsoft. No Minecraft code, textures or sounds are bundled.
