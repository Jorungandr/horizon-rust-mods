# Dynamic FPS (Horizon native mod)

Holds the frame rate down while the Horizon window is not the one being looked at, so the game
stops rendering hundreds of frames a second into a window you cannot see.

- **In the background:** the cap you choose (1, 5, 10, 15, 20 or 30 FPS), 10 by default.
- **Minimised:** 1, 5 or 10 FPS, 1 by default. Horizon itself already caps a minimised window at
  10 FPS before this mod's hook runs, and the reported cap is clamped to the game's own, so higher
  values would have no effect and are not offered.
- A lower cap you set yourself is never raised, and the game is never pushed below one frame.
- **1 FPS is a power-saving trade-off, not a performance setting.** At one frame per second the
  client cannot keep up with a 20 TPS world: ticks, chat and the network still run, but the picture
  updates about once a second and the client may fall behind the server until the window is
  focused again. Use 10 FPS if you need the client to stay in step.
- Settings are under **Options > Mods > Dynamic FPS**.

It only answers Horizon's `framerate_limit` hook, the same place the game's own Battery Mode uses.
The observed caps are: focused = your normal frame rate, background = the chosen background cap,
minimised = the chosen minimised cap (no higher than Horizon's own 10 FPS limit).

## Install

Use a launcher that supports native mods (0.2.81 or newer), add this folder on the **Mods** page
and press Play. The launcher compiles it against the Horizon build you play.

```
horizon-launcher.exe native-build <path-to-this-folder> --instance <name> --update --trust-native-code
```

## Licence

MIT. It follows the original [Dynamic FPS](https://github.com/juliand665/Dynamic-FPS) by
juliand665 (MIT) without copying its code; see `NOTICE.md`.
