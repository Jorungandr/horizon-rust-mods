# New format: launcher-installable native mods

This document explains the official-format distribution under [`native/`](../native/):
how players install it, how an author publishes to the official registry, and what
cannot be ported. It supersedes the older injected full-client approach described in
[`DELIVERY.md`](DELIVERY.md), which cannot be installed from the launcher.

## Why this format

The legacy packages inject these mods into a Horizon source tree and rebuild the whole
`horizon-client.exe`. A player must download a 70 MB+ package and run its own
`START_GAME.cmd` — the launcher's **add mod** button does not see it.

The `native/` projects are ordinary **source mods** (`horizon-mod.toml` with
`api = "0.2"`, `Cargo.toml`, `src/`). The launcher compiles them on the player's own
machine with its bundled Rust against the pinned SDK and loads the resulting `mod_*.dll`
into the unmodified game. The game binary is never rebuilt or replaced.

Compatibility comes from API hashing: a mod is matched to a build by the hashes of the
APIs it actually uses, not by the whole binary. It keeps working on newer Horizon
builds as long as those hashes are unchanged. See [`native/README.md`](../native/README.md#portability).

## For players: install with the launcher

There are two ways to get these mods into the launcher.

### A. From the official registry (one click)

Once a project is listed in the official registry, a player opens **Mods** in the
launcher, finds it by name, installs it and presses **Play**. The launcher downloads the
`.hmod`, verifies its SHA-256 (and minisign signature, if the author set a key), compiles
the native source locally and loads it. Ratings and icons appear on the project page at
`playhorizon.dev/mods`.

This is the only route that is literally "click add" and requires no manual folder
handling. At the time of writing these eight mods are distributed via this repository
and releases; see section "Publish to the registry" to have them listed.

### B. Add a local folder

1. Use Horizon launcher **0.2.81 or newer**.
2. Download `Horizon-Native-Mods-0.2.0-source.zip` from the
   [latest release](https://github.com/Jorungandr/horizon-rust-mods/releases/latest)
   and extract it.
3. In **Mods**, add each project folder under `native/` (the ones containing
   `horizon-mod.toml`, `Cargo.toml` and `src/`), then press **Play**.
4. The launcher caches the pinned SDK, compiles with its bundled Rust, and starts the
   game with the mods loaded. Settings live under `Options > Mods` on mods that register
   them.
5. You must own Minecraft Java Edition and import its **26.3** resources once; Horizon
   and this repository contain no Minecraft assets.

Equivalent command line:

```
horizon-launcher.exe native-build <path-to-mod-folder> --instance <name> --update --trust-native-code
```

`native-build` also accepts a parent directory holding several mod folders and compiles
all of them into one build. Set `HORIZON_SDK_BASE` to a mirror root if you mirror the SDK.
`--trust-native-code` is required once because native mods run with your user rights.

`whenchat` and `chattimestamps` do the same job (a chat timestamp). Install just one.

## Publish to the official registry (for authors)

The registry is the `horizon-rust/horizon-mods` GitHub repository; the launcher and the
website read its generated `index.json` (schema 1). The current submission path is a
**pull request** that adds `mods/<id>.toml` (the web form is legacy and cannot take
current packages).

### Submit one project by pull request

1. Attach the `.hmod` to a **GitHub release of your own repository** (here:
   `Jorungandr/horizon-rust-mods`, tag `v0.2.0-native`), under the exact file name the
   entry uses, e.g. `whenchat-0.2.0.hmod`.
2. Open a PR against `horizon-rust/horizon-mods` that adds `mods/<id>.toml` with a
   `[[versions]]` record: `version`, `file`, `sha256` (`sha256sum <file>`), `game`,
   `published`, and the icon path `icons/<id>.png`. Link each release file in the PR
   description.
3. CI `stage` downloads each linked file, checks its name and SHA-256 against the new
   record, and runs the checker; it takes **at most 3 new files per run** (more must be
   staged by a maintainer), so submit in batches of three.
4. `check` verifies structure, hashes and the built index, then a reviewer reads the
   entry and, for a native mod, the source. Merging publishes it.

Each `.hmod` is a flat zip of `horizon-mod.toml`, `Cargo.toml`, `Cargo.lock`, `src/`
(plus `LICENSE`/`NOTICE`, which the packer keeps). Build it with the official tool so the
layout and `uses.json` are exactly what the registry expects:

```
cargo horizon-mod build                                  # -> target/horizon-mod/<id>.hmod
cargo horizon-mod check target/horizon-mod/<id>.hmod      # package check
cargo horizon-mod registry check <base> <head> --files <dir>   # the registry's own check
cargo horizon-mod registry review <base> <head> --author <login>
```

### Ports: what the policy requires

For a port of another mod the policy asks that the original's licence allows it, that the
original's **copyright and licence notices are kept**, and that the original is credited
and linked. Here each port package carries `LICENSE` with the upstream copyright line
first (e.g. `Copyright (c) 2026 ErikEdits`) and the port credited below, a `NOTICE.md`
naming the upstream author/year/repo, and the entry's `authors` lists the upstream author.

### Page requirements

- Description: 50–10,000 characters after trimming.
- One to three categories from the native-mod category list; up to ten lowercase tags.
- A square PNG icon, 64–1024 px, at most 256 KiB, stored in the registry.
- Optionally up to eight PNG/JPEG screenshots stored in the registry.
- `license` (SPDX) and `source` (`https://`) are required.

Before submitting, check each package:

```
cargo horizon-mod check my_mod.hmod
```

An entry's `id` must match its manifest's `id`; `kind` for a mod with `[native]` is
`native-mod`. Native binaries in the index are optional registry metadata and are **not**
used for source mods — the launcher always builds them locally.

## What cannot be ported: auto-reconnect

The official `Host` API exposes `disconnect` but no method to connect to a server, so the
Java mod **Auto Reconnect** cannot be reproduced faithfully as a native mod. It exists
only in the legacy injected full-client package (v1.x releases); those packages cannot be
installed with the launcher's add-mod button. The other eight mods are fully ported and
verified in-game.

## Verification status

All eight `native/` projects pass their Rust unit tests offline, compile together through
`horizon-launcher native-build` (`left_out: []`, none skipped), and were loaded in a real
game. In-game checks covered the chat timestamp/format/highlight pipeline (including
`Translatable` chat arguments), the coords/fps/durability HUDs and item tooltips.

## Registry submission status (0.2.0)

Six of the mods are submitted to `horizon-rust/horizon-mods`, in two pull requests of
three files each (the CI limit), with the `.hmod` files attached to release
[`v0.2.0-native`](https://github.com/Jorungandr/horizon-rust-mods/releases/tag/v0.2.0-native):

- [#1459](https://github.com/horizon-rust/horizon-mods/pull/1459) — `chathighlight`,
  `durability`, `itemtooltip` (original mods)
- [#1460](https://github.com/horizon-rust/horizon-mods/pull/1460) — `whenchat`,
  `chattimestamps`, `timeformatter` (MIT ports, upstream copyright kept)

Both passed CI `fetch`/`stage` and the local `cargo horizon-mod registry check` and
`registry review` (7 packages, all `native-mod`, `left_out: []`). The `check` workflow on
each PR waits for a maintainer to approve a first-time contributor's run.

`fpshud` is intentionally **not** submitted: the official FPS Counter is now a WebAssembly
mod, installable with no build, so ours would duplicate it. `coords` is held back too: the
official Coordinates HUD is being redone (issue #1214) and an entry there would compete
with that work, though our implementation shares no code with it.
