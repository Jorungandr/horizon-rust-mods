# Real-machine verification for the registry reviews

This is the user-facing checklist for the points the reviewers asked to be
observed in-game. The launcher build log (all ten mods compiled with
`--trust-native-code`) and the exact-archive `cargo horizon-mod check` output
are in [CHECK-OUTPUT.md](CHECK-OUTPUT.md); this file covers the on-screen
observations that a static checker cannot make.

## How to launch

Use the Horizon launcher with the build that has the ten native mods installed.
Open the launcher, select that instance, press **Play**. Do not add a `.hmod`
by hand: native mods compile into the game build. Settings for each mod are under
**Options > Mods**. Minecraft Java 26.3 resources must be imported once.

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

Screenshots of the three states should be attached to the Dynamic FPS follow-up
PR if the reviewer still wants runtime confirmation. Static checks cannot see
frame rates; this is the one part that must be captured on a real client.

## What is already attached to the PRs

- Launcher native-build log for all ten mods (`CHECK-OUTPUT.md`).
- Exact SHA-256 checker output for every released archive (same file).
- Unit tests: Chat Highlight 10/10, Time Formatter 15/15, Dynamic FPS 5/5.
- Source links for every fix (source repo commits).
- AppleSkin real-machine HUD screenshots (approval screenshots from testing).
