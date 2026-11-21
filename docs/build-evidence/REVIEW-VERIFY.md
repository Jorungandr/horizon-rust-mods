# Real-machine verification for the registry reviews

This is the user-facing checklist for the points the reviewers asked to be
observed in-game. The launcher build log (all ten mods compiled with
`--trust-native-code`) and the exact-archive `cargo horizon-mod check` output
are in [CHECK-OUTPUT.md](CHECK-OUTPUT.md); this file covers the on-screen
observations that a static checker cannot make.

## How to launch

Use the Horizon launcher with the build that has the ten native mods installed.
The instance now plays `native-5ef7734-ef7a90e8e8c9`: the ten mods compiled from
the current sources against the pinned SDK revision `5ef7734f…` (the Nov 19
update), installed by the launcher's own native build. Open the launcher, press
**Play**. Do not add a `.hmod` by hand: native mods compile into the game build.
This build has no in-game page for native mod settings — each mod reads its own file
under `mods/.horizon-mod-data/<mod id>/` — so the observations below are of the
registered defaults. Minecraft Java 26.3 resources must be imported once.

## AppleSkin (`appleskin_hzn`)

Already observed and approved on a real client (gold saturation shown on the
hunger bar; overlay hidden while paused). Re-check after any rebuild:

1. In game (not paused), look at the hunger bar. A **gold overlay** sits over
   the hunger icons, half an icon per saturation point, filling right to left.
   It must be the original AppleSkin pixel-art shape, not a solid yellow bar.
2. Pause (**Esc**): the gold overlay **disappears**. Resume: it returns.
3. Hold a food item and hover it in the inventory. The tooltip shows
   `Hunger +N` and `Saturation +N.N` read from that stack's own food component.
4. A few screenshots from the visual iteration are kept in
   [`screenshots/`](screenshots/) (hunger-bar saturation overlay). They show the
   gold pixel-art overlay; the final build also hides it while paused and shows
   the two tooltip lines, which are best confirmed again on the current build.

## Dynamic FPS (`dynamicfps`)

The reviewer asked for observed focused / background / minimised caps. The
mod only answers the game's `framerate_limit` hook; it draws nothing, so the
three states are observed with an FPS overlay (the bundled FPS HUD mod) or the
launcher's frame-time readout:

1. **Focused**: FPS HUD shows the uncapped / normal frame rate.
2. **Background** (alt-tab to another window): the cap drops to the configured
   background value (default **10 FPS**; choices 1/5/10/15/20/30).
3. **Minimised**: the cap drops to the minimised value (default **1 FPS**;
   choices 1/5/10). Note 1 FPS is a power-saving trade-off, not a performance
   setting — the client cannot keep up with a 20 TPS world at one frame per
   second. Horizon itself clamps a minimised window to 10 FPS before this hook
   runs, so values above 10 had no effect and were removed in v0.2.1.

The three states are now captured on the current build — see the
[measured table](#measured-on-the-nov-19-build-2026-11-21) above. Static checks
cannot see frame rates; this is the part that had to come from a real client.

## Measured on the Nov 19 build (2026-11-21)

Instance `native-5ef7734-ef7a90e8e8c9`, 1920×1080 client area, survival player,
all mods on their registered defaults. Frames below were captured from the game
window itself (not the desktop), so the FPS numbers are the client's own overlay.

| Check | Observed | Screenshot |
| --- | --- | --- |
| AppleSkin, in game | gold pixel-art overlay on the right-hand hunger icons | `appleskin-playing.png`, `appleskin-hud-full-hunger.png` |
| AppleSkin, paused | overlay gone, only the pause blur remains on the icons | `appleskin-paused-hidden.png` |
| AppleSkin, food tooltip | cooked porkchop shows `恢复饥饿 +8` / `饱和度 +13.8` | `food-tooltip-values.png` |
| Dynamic FPS, focused | `115 FPS / Frame 11 ms` | `dynamicfps-foreground.png` |
| Dynamic FPS, background | `10 FPS / Frame 96 ms` with the launcher focused and the game still visible | `dynamicfps-unfocused-10fps.png` |
| Dynamic FPS, restored from minimised | first frame after restore reads `0 FPS / Frame 93 ms` while the counter climbs back, i.e. it was at the 1 FPS cap | `dynamicfps-restored-from-minimized.png` |
| Chat timestamps | two stamps per line, own messages included — expected while both chat mods are installed | `chat-timestamps-duplicated.png` |
| Time Formatter | `:t` → `06:13`, `:f` → `November 15, 2023 at 06:13`, `:R` → `3 years, 6 days, 11 hours… ago`; `<t:abc>` and an out-of-range stamp stay literal; CJK before/after the tag is kept; `<<t:…>` stays literal | `chat-timeformatter-tags.png`, `timeformatter-double-bracket-escape.png` |
| Chat Highlight | explicit colours kept (`A1` green bold, `A2` red italic, `A3` blue); the keyword is recoloured gold and the rest of the line keeps the styling it inherits from the message itself, identical to the same message without a keyword | `chat-highlight-style-preserved.png` |

Escape syntax is a doubled angle bracket (`<<t:…>` renders the tag literally); a
backslash is not special and stays as a literal character. Epoch
`1700000000` is formatted in the machine's local zone (UTC+8), hence `06:13`.

## Fixes from the user's screenshots (2026-11-21)

Two real defects showed up in the player's own screenshots:

1. **Durability HUD** printed hard-coded English (`Helmet Copper Helmet 43%`)
   even on a Chinese client. v0.2.1 resolves the item name with
   `host.translate("item.<id>")` / `block.<id>` and falls back to a short slot
   label, so the same row reads `铜头盔 43%`.
2. **Item Tooltip+** appended its lines with the RGB palette value as the
   trailing RGBA alpha. `GRAY` (`0x808080`) was therefore invisible and green
   was one-third opaque, which is why the player's screenshot of a cobblestone
   showed only its name (`item-tooltip-missing-lines.png`). v0.2.1 keeps the
   colour on the `Text` and pushes the line with full alpha.

Both are published as `v0.2.1-native` and the registry branch `add-native-mods-0.2.0`
points at the new hashes; the 0.2.0 assets are untouched. Their screenshots
will need to be retaken after the local instance is rebuilt with 0.2.1.

Two observations still need a sharper capture: the action-bar text itself
(`/title @a actionbar …`) lasts only a couple of seconds — the current
`actionbar-command-feedback.png` caught the command's chat reply, not the bar —
and `paused-fps-readout.png` is the pause menu (`10 FPS`), not a true minimised
window. Neither changes the code; both are evidence gaps only.

## What is already attached to the PRs

- Launcher native-build log for all ten mods, after the review fixes, against the
  pinned Nov 19 SDK (`CHECK-OUTPUT.md`).
- Exact SHA-256 checker output for every released archive (same file).
- Unit tests: Chat Highlight 10/10, Time Formatter 15/15, Dynamic FPS 5/5.
- Source links for every fix (source repo commits).
- AppleSkin real-machine HUD screenshots (approval screenshots from testing).
