// Helper injected at the HEAD of Game::native_client_chat_received.
//
// Real signature (Horizon 9ee13cd0, native_hooks.rs):
//   fn native_client_chat_received(
//       &mut self, system: bool, overlay: bool, sender: u128,
//       chat_type: Option<&str>, message: &mut Text, cancel: &mut bool)
// `Text` is the engine's horizon_client::text::Text: a flat `Vec<Span>` where
// each Span { text: String, style: Style }. Rewriting a span's `text` in place
// keeps its style / colour intact, unlike the original mod which flattened the
// component with getString() and lost all per-character styling.
//
// horizon-mixins requires exactly one private top-level `fn` with no top-level
// `use`/attrs, so all helpers are nested inside the entry point.

fn timeformatter_client_chat_received(
    overlay: bool,
    message: &mut Text,
) {
    // Action bar (overlay) is not a chat line.
    if overlay {
        return;
    }

    for span in &mut message.spans {
        if let Some(replacement) = crate::game::timeformatter::format_literal(&span.text) {
            span.text = replacement;
        }
    }
}
