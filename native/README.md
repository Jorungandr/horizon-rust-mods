# Official-format native mods (recommended)

These ten projects use Horizon's **official native-mod SDK** (`api = "0.2"`). They are
source packages: the Horizon launcher compiles them on your machine against the exact
Horizon build you play, then loads the resulting `mod_*.dll` files. The game itself is
never rebuilt or replaced.

This is the way to install the mods from the launcher's **Mods** page — add a mod folder,
then press Play. You do not need the injected full-client package described in the root
README; that older distribution is kept only for reference.

## Included

| Folder | Mod | Verified behaviour |
|---|---|---|
| `whenchat` | WhenChat | `[HH:mm:ss]` prefix on incoming chat |
| `chattimestamps` | Chat Timestamps | Configurable timestamp prefix or hover, 6 presets |
| `timeformatter` | Time Formatter | Rewrites `<t:1704067200:t>` / `R` tags in chat |
| `chathighlight` | Chat Highlight | Colours configured keywords, including translatable chat arguments |
| `coords` | Coordinates HUD | XYZ, facing, dimension; four corner placements |
| `fpshud` | FPS HUD | FPS / frame time overlay; four corner placements |
| `durability` | Durability HUD | Damaged gear and held tools, colour thresholds, right-aligned panel |
| `itemtooltip` | Item Tooltip+ | Adds item id, durability and stack size to tooltips |
| `appleskin` | AppleSkin | Saturation in gold over the hunger bar, `Hunger +N` / `Saturation +N.N` in food tooltips |
| `dynamicfps` | Dynamic FPS | 10 FPS in the background, 1 FPS minimised; never raises a lower cap |

`autoreconnect` is **not** in this set: the official `Host` API has `disconnect` but no
method to connect to a server, so the Java port cannot be reproduced faithfully.

`whenchat` and `chattimestamps` do the same job. Install both only if you deliberately
want two prefixes; normally pick one (Chat Timestamps is configurable).

## Install from the launcher

1. Use a Horizon launcher that supports native mods (0.2.81 or newer).
2. In **Mods**, add each project folder above (`horizon-mod.toml`, `Cargo.toml`, `src/`).
3. Press **Play**. The launcher downloads/caches the pinned SDK, compiles the mods with
   its own Rust toolchain and starts the game with them loaded. A mod that registers
   settings reads them from its own data folder (`mods/.horizon-mod-data/<mod id>/`);
   this Horizon build has no in-game page for them yet, so the registered defaults
   apply. Distinct settings files can be written there to change one.
4. You must own Minecraft Java Edition and import its 26.3 resources once; Horizon and
   this repository contain no Minecraft assets.

Equivalent command line (from the launcher's folder):

```
horizon-launcher.exe native-build <path-to-this-folder> --instance <name> --update --trust-native-code
```

`native-build` also accepts a parent directory containing several mod folders, which
compiles all of them into one build. If you mirror the SDK, set `HORIZON_SDK_BASE` to the
mirror root first. `--trust-native-code` is required once because native mods run with
your user rights.

## Portability

Each manifest pins the game version and the Horizon/mixins source hashes it was built
against:

```toml
api = "0.2"
[native]
game = "26.3"
horizon = "dd67b76..."
mixins = "e59da909..."
```

The launcher matches mods to a build by the hashes of the APIs they actually use, not by
the whole binary. A mod keeps working on newer Horizon releases as long as those API
hashes stay unchanged; otherwise the launcher asks you to rebuild against the new SDK.

## Verification

All eight projects pass their Rust unit tests offline, compile together through
`horizon-launcher native-build` (`left_out: []`, no mods skipped), and were loaded in a
real game. In-game checks observed:

```
[12:39:36] [12:39:36] <Player01107> FINAL 08:00 join the server
```

The first timestamp is WhenChat, the second Chat Timestamps, `08:00` is Time Formatter
converting `<t:1704067200:t>`, and `server` is coloured gold by Chat Highlight. HUD
mods were checked on a running client with `horizon-ctl screenshot`.

## License

Same as the repository: ports of the three MIT Java mods keep their original notices;
the five original mods and tooling are MIT, copyright (c) 2026 Jorungandr.
