// Helper injected at the HEAD of Game::native_client_chat_received.
//
// Real signature (Horizon 9ee13cd0, native_hooks.rs):
//   fn native_client_chat_received(
//       &mut self, system: bool, overlay: bool, sender: u128,
//       chat_type: Option<&str>, message: &mut Text, cancel: &mut bool)
// `Text` is the engine's horizon_client::text::Text: a flat `Vec<Span>` where
// each Span { text, style }, Style has `color: Option<u32>` and
// `hover: Option<Box<Text>>`. Colour names map via text::named_color.
//
// Mirrors MixinChatComponent.modifyMessage:
//   - disabled / overlay -> leave the message untouched
//   - hover mode        -> attach the timestamp to each span's hover event
//   - prefix mode       -> prepend timestamp spans, keeping the original spans
//                          (and their styling) as trailing siblings.
//
// horizon-mixins requires exactly one private top-level `fn` with no top-level
// `use`/attrs, so all imports and helpers are nested inside the entry point.

fn chattimestamps_client_chat_received(
    game: &mut crate::game::Game<'_>,
    overlay: bool,
    message: &mut Text,
) {
    use crate::game::chattimestamps as m;
    use horizon_client::text::named_color;

    // Build a styled Text for one segment (colour name -> packed ARGB).
    fn seg_text(text: &str, color: Option<&str>) -> Text {
        match color.and_then(named_color) {
            Some(c) => Text::colored(text, c),
            None => Text::literal(text),
        }
    }

    // Full timestamp as a styled Text, built from all segments.
    fn build_timestamp(segments: &[m::Segment]) -> Text {
        let mut out = Text::empty();
        for seg in segments {
            out.push(seg_text(&seg.text, seg.color));
        }
        out
    }

    if overlay {
        return;
    }

    let settings = m::settings(|key| game.native_setting(m::ID, key));
    if !settings.enable_mod {
        return;
    }

    let Some(segments) = m::timestamp_now(&settings.format) else {
        // Empty format means "off" in the original mod.
        return;
    };
    if segments.is_empty() {
        return;
    }

    if settings.enable_hover {
        // Attach the timestamp hover to every span of the message so it shows
        // regardless of which run the cursor is over.
        let stamp = build_timestamp(&segments);
        for span in &mut message.spans {
            span.style.hover = Some(Box::new(stamp.clone()));
        }
    } else {
        // Prepend the timestamp spans, preserving the original spans' styles.
        let original = std::mem::take(message);
        let mut wrapper = Text::empty();
        for seg in &segments {
            wrapper.push(seg_text(&seg.text, seg.color));
        }
        wrapper.push(original);
        *message = wrapper;
    }
}
