# Notices

This is an unofficial Rust port of Plague's Chat Timestamps for Horizon.

- Original mod: Plague's Chat Timestamps by PlagueTR — https://github.com/PlagueTR/ChatTimestamps
- Original license: MIT
- Port license: MIT (see LICENSE)

The original mod's colour-code handling and three-option configuration
(enableMod / timestampFormat / enableHover) are re-implemented here against
Horizon's native mod API. This port offers six fixed timestamp presets instead
of arbitrary SimpleDateFormat patterns; the Java pattern language is not
re-implemented. The settings page is rendered by Horizon's built-in
native-settings UI, replacing the original's Cloth Config dependency.

This project is not affiliated with or endorsed by Mojang, Microsoft, the Horizon
team, or PlagueTR.
