# Dynamic FPS (Horizon native mod)

Holds the frame rate down while the Horizon window is not the one being looked at, so the game
stops rendering hundreds of frames a second into a window you cannot see.

- **In the background:** the cap you choose (1, 5, 10, 15, 20 or 30 FPS), 10 by default.
- **Minimised:** down to 1 FPS by default.
- A lower cap you set yourself is never raised, and the game is never pushed below one frame.
- Settings are under **Options > Mods > Dynamic FPS**.

It only answers Horizon's `framerate_limit` hook, the same place the game's own Battery Mode uses.

## Install

Use a launcher that supports native mods (0.2.81 or newer), add this folder on the **Mods** page
and press Play. The launcher compiles it against the Horizon build you play.

```
horizon-launcher.exe native-build <path-to-this-folder> --instance <name> --update --trust-native-code
```

## Licence

MIT. It follows the original [Dynamic FPS](https://github.com/juliand665/Dynamic-FPS) by
juliand665 (MIT) without copying its code; see `NOTICE.md`.
