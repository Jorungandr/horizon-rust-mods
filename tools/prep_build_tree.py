#!/usr/bin/env python3
"""Prepare a Windows build tree: copy Horizon 9ee13cd0 source, then inject the
TimeFormatter port directly (a private module + a HEAD call in the chat hook)."""
import shutil, sys, pathlib

SRC = pathlib.Path(r"G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/horizon-mods-staging/native/target/horizon/9ee13cd038559db05de6a4ced4496028ce409564")
DST = pathlib.Path(r"G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/build-win/horizon-src")
RUNTIME = pathlib.Path(r"G:/WorkBuddyWork/2026-11-16-16-13-43/horizon-ports/timeformatter-horizon/mixins/timeformatter/runtime.rs")

if "--copied" not in sys.argv:
    if DST.exists():
        shutil.rmtree(DST)
    print("copying tree ...")
    shutil.copytree(SRC, DST, ignore=shutil.ignore_patterns("target"))
    print("copied", DST)

hooks = DST / "crates/horizon-client/src/game/native_hooks.rs"
sub_dir = DST / "crates/horizon-client/src/game/native_hooks"
sub_dir.mkdir(parents=True, exist_ok=True)

# 1. the port's pure logic as a private module, plus the span rewriter
runtime = RUNTIME.read_text(encoding="utf-8")
apply_fn = '''

/// Rewrite every well-formed `<t:...>` inside each span of `message`, in
/// place, so each span keeps its own style. Mirrors the mod's chat helper.
pub fn apply(message: &mut ::horizon_client::text::Text) {
    for span in &mut message.spans {
        if let Some(replacement) = format_literal(&span.text) {
            span.text = replacement;
        }
    }
}
'''
(sub_dir / "ports_timeformatter.rs").write_text(runtime + apply_fn, encoding="utf-8")

# 2. declare the module and call it at the head of the chat hook
text = hooks.read_text(encoding="utf-8")

if "ports_timeformatter" in text:
    print("already patched")
    sys.exit(0)

decl = '#[path = "native_hooks/ports_timeformatter.rs"]\nmod ports_timeformatter;\n\n'
anchor_decl = "use super::native_data_maps::NativeDataMapType;"
assert text.count(anchor_decl) == 1, "decl anchor not unique"
text = text.replace(anchor_decl, decl + anchor_decl)

anchor_call = "mod_api::dispatch_client_chat_received("
assert text.count(anchor_call) == 1, "call anchor not unique"
call = '''if !overlay {
            ports_timeformatter::apply(message);
        }
        '''
text = text.replace(anchor_call, call + anchor_call)

hooks.write_text(text, encoding="utf-8")
print("patched", hooks)
