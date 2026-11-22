# NOTICE — timeformatter-horizon

An unofficial Rust port of **TimeFormatter** to the Horizon native mod API.

- Original mod: https://github.com/m3z0id/TimeFormatter
- Original author: M3z0id
- Original license: MIT, Copyright (c) 2025 M3z0id (see `LICENSE`)
- Port author: Jorungandr (unofficial port, 2026-11-17)
- Port license: MIT (same terms; original copyright retained above)

## What was ported

The core client-side behaviour only:

- Detect Discord-style tags `<t:SECONDS[:STYLE]>` in incoming chat and replace
  them with the local time (`Formatter.java` + `ChatListener.java`).
- `SECONDS` may be 1-13 digits or `now` / `yesterday` / `tomorrow`; an upper
  bound of `8640000000000` is respected.
- All seven styles are implemented: `f`, `F`, `d`, `D`, `t`, `T`, `R`
  (relative time), with `f` as the default.
- The doubled literal form `<<t:...>>` is deliberately left untouched, matching
  the original.

## Differences from the Java original

- **No graphical "store timestamp" UI** (`TimeUIScreen`, the Date/Time field
  widgets) and no Fabric API / Adventure dependency. Those are presentation
  extras; automatic chat formatting — the mod's stated purpose — is complete.
- **Styling is preserved**: the original flattens the whole message to a string
  and rebuilds it (dropping per-character colours). This port reformats each
  literal component in place, so colours and hover data around a tag survive.
- Regex is hand-written rather than using `regex`, keeping the mod dependency-
  light and offline-buildable.

## Verification

- library unit tests: 16/16 (tag parsing, all seven formats against fixed UTC,
  special values, literal escapes, multiple tags, relative time).
- `verify/` end-to-end against a stubbed Horizon API with the REAL injected
  files compiled in: 7/7 (chat replacement, literal form, overlay skip,
  child-component walk with style preservation, hover safety, list row).

Verified against Horizon main commit `85b5bcca7c79bb470b90bdb5914b413aa66fe7ae`.
