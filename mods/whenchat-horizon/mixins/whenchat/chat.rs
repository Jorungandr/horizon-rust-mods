// Helper injected at the HEAD of Game::native_client_chat_received.
//
// Real signature (Horizon 9ee13cd0, native_hooks.rs):
//   fn native_client_chat_received(
//       &mut self, system: bool, overlay: bool, sender: u128,
//       chat_type: Option<&str>, message: &mut Text, cancel: &mut bool)
// `Text` here is the engine's horizon_client::text::Text (has .plain(),
// Text::literal, From<String>), NOT mod_api::text::Text.
//
// horizon-mixins requires exactly one private top-level `fn` with no top-level
// `use`/attrs, so all imports and helpers are nested inside the entry point.

fn whenchat_client_chat_received(
    game: &mut crate::game::Game<'_>,
    system: bool,
    overlay: bool,
    message: &mut Text,
) {
    use crate::game::whenchat as m;

    if !game.native_on(m::ID) || overlay {
        return;
    }

    let original = message.plain();
    if original.is_empty() {
        return;
    }

    let Some(decorated) = m::decorate(
        m::Line {
            text: &original,
            system,
            overlay,
        },
        &m::local_prefix(),
    ) else {
        return;
    };

    // Replace the line with a fresh literal. Drops per-character styling, which
    // is fine for WhenChat's plain prefix (chattimestamps preserves style).
    *message = Text::literal(decorated);
}
