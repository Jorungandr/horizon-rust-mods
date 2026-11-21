# Exact-SDK Build Evidence — v0.2.0-native (rebuild)

Checker: `cargo-horizon-mod` built from horizon-rust/horizon @ `5ef7734fdd854add5efd6817178be3c988d8d050` (registry .github/horizon-rev).

```text
$ cargo horizon-mod check whenchat-0.2.0.hmod
whenchat 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  7296 bytes, SHA-256 54a09cf8395878b61ca586d1a7d47e33f59d4f2a7aa0a334c24d6fa19f51cd82, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check chattimestamps-0.2.0.hmod
chattimestamps 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  12618 bytes, SHA-256 145be47730cd2106f93f1da39a2a1ebaa6794a08acd956c248bda0458e66869f, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check timeformatter-0.2.0.hmod
timeformatter 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  13202 bytes, SHA-256 3fba38d11f037d0bb4f88824c2c5e7e3e0f16802983656faa3198567f3eeb2d5, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check chathighlight-0.2.0.hmod
chathighlight 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  10398 bytes, SHA-256 b15a5bca68e925bcfa3d6314b27269ee41e343b6f5e9bbb6ab4a96a44efcc68d, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check durability-0.2.0.hmod
durability 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  10601 bytes, SHA-256 4be1c3a6961d9ecb6b5503b09e8ca66e545c477105f4493d40ccde9647b28042, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check itemtooltip-0.2.0.hmod
itemtooltip 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  8559 bytes, SHA-256 ab7e22e400d62d2ba7ff698336a58e5b0117ac139c80f0c32ad87f8f3d332402, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check appleskin-0.2.0.hmod
appleskin 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  10844 bytes, SHA-256 462ae6cfd3da03ba1a746f7714520810ffdadbdb412d9146f045a7b7b03ca215, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check dynamicfps-0.2.0.hmod
dynamicfps 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  7995 bytes, SHA-256 ff368963a5d743c147051c64cbb2182f50f73aa69a3e398f12eb61c740e94ffc, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

```

## Rebuild after registry review (three packages)

AppleSkin, Dynamic FPS and Chat Timestamps were rebuilt after review. Each
package below was produced with `cargo-horizon-mod build` from the exact pinned
SDK (`5ef7734fdd854add5efd6817178be3c988d8d050`) and re-checked:

```text
$ cargo horizon-mod check appleskin_hzn-0.2.0.hmod
appleskin_hzn 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  12351 bytes, SHA-256 e3d367acdd4de948dad3385b781d7c284d8a6989edd42a32be1dab22643b3c62, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check dynamicfps-0.2.0.hmod
dynamicfps 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  8432 bytes, SHA-256 14f2bd0c210fd9903435d86341a213352034e401a6a20d86768a799599d24806, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check chattimestamps-0.2.0.hmod
chattimestamps 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  12682 bytes, SHA-256 8941f9b1c78d500ab1580740e8635607c27b77727d1f16147ec88c676467945f, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.
```

## Rebuild after the second review (client-stall fixes)

The review also asked for two quadratic client-stall paths to be removed:
Chat Highlight cloned the full inherited `TextStyle` (including hover/click/item
data) onto every keyword fragment, and Time Formatter decoded the whole rest of
the line as UTF-8 for every `<t:` candidate. Both were fixed in source, unit
tested (Chat Highlight 10/10, Time Formatter 15/15), rebuilt with the same pinned
SDK and re-checked. Neither change adds or removes SDK API uses, so `uses.json`
is byte-identical to the previously reviewed package for both mods.

```text
$ cargo horizon-mod check chathighlight-0.2.0.hmod
chathighlight 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  10890 bytes, SHA-256 6c12c88ccbd293307cf17c4318f09a7d7315dff3ef82d7bbae9ad002b57690f7, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.

$ cargo horizon-mod check timeformatter-0.2.0.hmod
timeformatter 0.2.0 (native-mod, format native-1/5ef7734fdd854add5efd6817178be3c988d8d050/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  13581 bytes, SHA-256 ee97ffc6d8ae8f3445f37ae2ac5017244ba8623b7cdd5f250c6138de28a71501, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.
```

### Launcher build

All ten mods were also compiled by the Horizon launcher itself (native build,
`--trust-native-code`), which compiles every native source mod against the
prebuilt game rather than only packing it. This log was recorded for the
ten-mod bundle before the client-stall fixes below; the three packages rebuilt
afterwards are compiled against the same pinned SDK in
[Pinned-SDK compile of the rebuilt packages](#pinned-sdk-compile-of-the-rebuilt-packages).

```text
Checking 10 mods
Using the mod tools for the Nov 18 update kept from an earlier build
Prebuilt game: only the mods compile (optimized)
Compiling the mods against the prebuilt game
Compiling appleskin (1 of 10)
Compiling chathighlight (2 of 10)
Compiling chattimestamps (3 of 10)
Compiling coords (4 of 10)
Compiling durability (5 of 10)
Compiling dynamicfps (6 of 10)
Compiling fpshud (7 of 10)
Compiling itemtooltip (8 of 10)
Compiling timeformatter (9 of 10)
Compiling whenchat (10 of 10)
Installing the build
Your mods are built (took 51 s).
```

## Dynamic FPS 0.2.1 (post-merge follow-up)

Dynamic FPS was merged as 0.2.0 (`ff368963…`) before the review's 20 FPS point was fixed.
Because a published file never changes, the fix is a new version. 0.2.1 removes the
ineffective 20 FPS minimised option (Horizon already clamps a minimised window to 10 FPS
before the hook) and documents the 1 FPS power-saving trade-off. Unit tests 5/5 pass.

```text
$ cargo horizon-mod check dynamicfps-0.2.1.hmod
dynamicfps 0.2.1 (native-mod, format native-1/5ef7734.../e59da90..., for 26.3)
  8432 bytes, SHA-256 31d4e99f35b1644740d2caf75a7864b917d4a52863b7d908a3c9351a82ce4e7b, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.
```

The 0.2.0 release asset was restored to its published `ff368963…` bytes; 0.2.1 is a
separate asset. Launcher build of all ten mods is recorded above.

## Pinned-SDK compile of the rebuilt packages

The three packages rebuilt after the reviews (`chathighlight-0.2.0.hmod`,
`timeformatter-0.2.0.hmod`, `dynamicfps-0.2.1.hmod`) come from the same source
directories the launcher build compiles, and the archives reproduce byte for byte
from them. Each was recompiled against the SDK revision the launcher pins
(`5ef7734f…`) and its unit tests were run, with the sources touched first so the
compile is not a cache hit:

```text
$ cargo build --offline                  # native/chathighlight
   Compiling mod-chathighlight v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.50s
$ cargo test --offline
running 10 tests
test result: ok. 10 passed; 0 failed

$ cargo build --offline                  # native/timeformatter
   Compiling mod-timeformatter v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.84s
$ cargo test --offline
running 15 tests
test result: ok. 15 passed; 0 failed

$ cargo build --offline                  # native/dynamicfps
   Compiling mod-dynamicfps v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.22s
$ cargo test --offline
running 5 tests
test result: ok. 5 passed; 0 failed
```

The regression tests added with the fixes are part of those runs: Chat Highlight
keeps the inherited style only on the parent, Time Formatter matches word-form tags
without decoding the rest of the line, and Dynamic FPS offers 1/5/10 FPS when
minimised.

The launcher build log above predates these fixes. Re-running it on the current
sources needs the launcher closed, because it holds the instance lock while open;
that log will be added here once captured.
