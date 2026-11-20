# AppleSkin (Horizon native mod)

Shows the saturation that vanilla keeps hidden behind the hunger bar, and what a food item
will actually restore.

- **Saturation on the hunger bar.** Each point of saturation covers half of one hunger icon,
  drawn in gold along the top edge of the bar, right to left, the way the hunger icons fill.
- **Food values in tooltips.** A food item lists `Hunger +N` and `Saturation +N.N`, read from
  the item's own `minecraft:food` component, so modded food works too.
- Settings are under **Options > Mods > AppleSkin**: the mod on or off, the saturation overlay,
  and the tooltip lines.

It reads the game's own numbers. Nothing leaves your computer and nothing is written to the world.

## Install

Use a launcher that supports native mods (0.2.81 or newer), add this folder on the **Mods** page
and press Play. The launcher compiles it against the Horizon build you play.

```
horizon-launcher.exe native-build <path-to-this-folder> --instance <name> --update --trust-native-code
```

## Licence

MIT. It follows the original [AppleSkin](https://github.com/squeek502/AppleSkin) by squeek502
(The Unlicense) without copying its code or artwork; see `NOTICE.md`.
