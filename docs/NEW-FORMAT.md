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
website read its generated `index.json` (schema 1). A submission is a pull request there,
or the web form at `playhorizon.dev/mods/submit` (GitHub sign-in). Two cases matter:

### Repository-linked project (recommended here)

1. Push the mod source to a **public** GitHub repository — done:
   `Jorungandr/horizon-rust-mods`.
2. Put a `horizon-registry.toml` at the repository root in the newest release tag, with
   only these keys: `name`, `summary`, `description`, `authors`, `license`. Other keys are
   refused; any field left out is filled from the registry entry.
3. Submit the project with its `repo` (the submit service checks the repo is public and
   opens a PR adding `mods/<id>.toml`). A reviewer reads the repository once.
4. Publish versions as **GitHub releases in this repository**. Repository-linked projects
   take no version uploads through the form: their versions are the repo's releases, read
   and validated by that repo's CI with no PR. Creating the release is all that is needed.
5. Each version file is a `.hmod` (a zip of `horizon-mod.toml`, `Cargo.toml`, `src/` and
   the checked root Cargo project). Pin it by SHA-256. If the project sets a minisign
   `key`, every version must be signed with it; updates signed with another key are refused.

### File-based project (no `repo`)

Submit the `entry` plus the `.hmod` file through the form. The service inspects it
(`horizon_mod_package::index::inspect`: signature, hash, entry fields), uploads it to the
`staging` release as `<sha256>.<extension>` and opens or updates the entry PR. Owners
listed in the entry can later publish updates without a reviewer (an `owner-update` PR
verified by the registry secret).

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
