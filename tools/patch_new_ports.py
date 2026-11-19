#!/usr/bin/env python3
"""Incrementally add the three new ports to an ALREADY-patched build tree.

The tree in build-win/horizon-src already has the first six ports injected by
inject_all_ports.py. This script adds only the chat-highlight / fps-hud /
item-tooltip wiring, so a normal `cargo build` stays incremental (refreshing
the tree through inject_all_ports.py --keep-target would remtime every file and
force a full ~40 min rebuild).
"""
import pathlib

ROOT = pathlib.Path(r"G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports")
DST = ROOT / "build-win/horizon-src"
sub = DST / "crates/horizon-client/src/game/native_hooks"

# Import the TAIL adapters and MODS map from the main injector without running
# its top-level side effects: parse them out by executing only the definitions.
# Simplest robust route: read the generated ports_*.rs if present is NOT enough
# (new ones don't exist yet), so re-run just the file-writing part.
inject_src = (ROOT / "tools/inject_all_ports.py").read_text(encoding="utf-8")

# Execute the injector's module up to (but not including) the tree-copy and
# patch section by stubbing sys.argv and monkeypatching shutil.copytree/rmtree.
import sys, types, shutil
saved_argv = sys.argv[:]
sys.argv = ["inject_all_ports.py", "--no-copy"]
real_copytree, real_rmtree = shutil.copytree, shutil.rmtree
shutil.copytree = lambda *a, **k: None
shutil.rmtree = lambda *a, **k: None
ns = {"__name__": "_inject_defs"}
try:
    # Cut off before `hooks = DST / ...` so none of the patch asserts run;
    # the file-writing loop runs and produces all nine ports_*.rs.
    cut = inject_src.index("hooks = DST / ")
    exec(compile(inject_src[:cut], "inject_all_ports.py", "exec"), ns)
finally:
    sys.argv = saved_argv
    shutil.copytree, shutil.rmtree = real_copytree, real_rmtree
print("wrote all nine ports_*.rs")

hooks = DST / "crates/horizon-client/src/game/native_hooks.rs"
text = hooks.read_text(encoding="utf-8")

assert "ports_chathighlight" not in text, "new ports already patched in this tree"

# 1. module declarations for the three new ports, after the last existing one
anchor = '#[path = "native_hooks/ports_autoreconnect.rs"]\npub(super) mod ports_autoreconnect;\n'
assert text.count(anchor) == 1
decls = (
    '#[path = "native_hooks/ports_chathighlight.rs"]\npub(super) mod ports_chathighlight;\n'
    '#[path = "native_hooks/ports_fpshud.rs"]\npub(super) mod ports_fpshud;\n'
    '#[path = "native_hooks/ports_itemtooltip.rs"]\npub(super) mod ports_itemtooltip;\n'
)
text = text.replace(anchor, anchor + decls)

# 2. chat highlight runs after the other three chat ports (still inside !overlay)
anchor = "            ports_timeformatter::apply(message);\n"
assert text.count(anchor) == 1
text = text.replace(anchor, anchor + "            ports_chathighlight::apply(&*self, message);\n")

# 3. Mods list: append three rows before the dispatch.
anchor = "        mod_api::dispatch_mods(self, mod_api::mods(), mods);"
assert text.count(anchor) == 1
rows = ""
for name, title, desc in (
    ("chathighlight", "Chat Highlight 0.1.0", "Colours configured keywords in incoming chat."),
    ("fpshud", "FPS HUD 0.1.0", "Shows frames per second, slowest frame and 1% low."),
    ("itemtooltip", "Item Tooltip+ 0.1.0", "Adds item id, durability and stack size to tooltips."),
):
    rows += (
        f'        mods.push(NativeMod {{\n'
        f'            id: "{name}".into(),\n'
        f'            title: "{title}".into(),\n'
        f'            description: "{desc}".into(),\n'
        f'            keys: Vec::new(),\n'
        f'        }});\n'
    )
text = text.replace(anchor, rows + "        " + anchor)

# 4. settings + settings_click for the three new ports
anchor = "        mod_api::dispatch_settings(self, mod_api::mods(), mod_id, buttons);"
assert text.count(anchor) == 1
calls = "".join(
    f"        ports_{n}::settings_page(&*self, mod_id, buttons);\n"
    for n in ("chathighlight", "fpshud", "itemtooltip")
)
text = text.replace(anchor, calls + "        " + anchor)

anchor = "        mod_api::dispatch_settings_click(self, mod_api::mods(), mod_id, button);"
assert text.count(anchor) == 1
calls = "".join(
    f"        ports_{n}::settings_click(&mut *self, mod_id, button);\n"
    for n in ("chathighlight", "fpshud", "itemtooltip")
)
text = text.replace(anchor, calls + "        " + anchor)

# 5. HUD: fps panel
anchor = "        mod_api::dispatch_hud(self, mod_api::mods(), batch, corners);"
assert text.count(anchor) == 1
text = text.replace(anchor, "        ports_fpshud::hud(&*self, batch, corners);\n        " + anchor)

# 6. tooltip: append our lines
anchor = "        mod_api::dispatch_tooltip(self, mod_api::mods(), stack, lines);"
assert text.count(anchor) == 1
text = text.replace(anchor, "        ports_itemtooltip::apply(self, stack, lines);\n        " + anchor)

hooks.write_text(text, encoding="utf-8")
print("patched native_hooks.rs with three new ports")
print("done")
