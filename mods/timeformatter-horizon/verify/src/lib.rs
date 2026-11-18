// Compiles the REAL chat.rs / list.rs / runtime.rs against a stubbed Horizon
// API. Module layout mirrors the injection target so helpers see
// `crate::game::Game`, `crate::game::timeformatter` and `crate::game::native_hooks`.
//
// The real helper files each contain exactly one PRIVATE `fn` (a hard
// requirement of horizon-mixins). We `include!` them into the test module so
// their private entry points are callable while staying byte-for-byte the
// sources injected into the game.

pub mod game {
    /// Stub game type (the chat helper does not currently read settings).
    pub struct Game<'a> {
        _marker: std::marker::PhantomData<&'a ()>,
    }

    impl Game<'_> {
        pub fn new() -> Self {
            Game {
                _marker: std::marker::PhantomData,
            }
        }
    }

    /// Stub NativeMod row type.
    pub mod native_hooks {
        pub struct NativeMod {
            pub id: String,
            pub title: String,
            pub description: String,
            pub keys: Vec<String>,
        }
    }

    /// The real runtime logic, compiled from the actual bundle file.
    #[path = "../../../mixins/timeformatter/runtime.rs"]
    pub mod timeformatter;
}

#[cfg(test)]
mod tests {
    use mod_api::horizon_client::text::Text;

    // Expand the real, single-private-fn helper sources into this module.
    include!("../../mixins/timeformatter/chat.rs");
    include!("../../mixins/timeformatter/list.rs");

    // Real hook params bound by the toml: (overlay, message).
    fn run(overlay: bool, message: &mut Text) {
        timeformatter_client_chat_received(overlay, message);
    }

    #[test]
    fn relative_tag_is_replaced_in_chat() {
        let mut message = Text::literal("event was <t:1:R>");
        run(false, &mut message);
        let out = message.plain();
        assert!(out.starts_with("event was "), "prefix lost: {out}");
        assert!(!out.contains("<t:"), "tag was not replaced: {out}");
        assert!(out.contains("ago"), "no relative phrase: {out}");
    }

    #[test]
    fn literal_double_bracket_tag_is_left_untouched() {
        let raw = "see <<t:1704067200:F>> here";
        let mut message = Text::literal(raw);
        run(false, &mut message);
        assert_eq!(message.plain(), raw);
    }

    #[test]
    fn overlay_is_left_untouched() {
        let raw = "action <t:1:R> bar";
        let mut message = Text::literal(raw);
        run(true, &mut message);
        assert_eq!(message.plain(), raw);
    }

    #[test]
    fn walks_all_spans_and_preserves_styling() {
        // Engine Text is flat spans: a coloured second span must keep its style
        // while its <t:...> text is reformatted.
        let mut message = Text::literal("prefix ");
        message.push(Text::styled("<t:1:R>".to_string(), mod_api::horizon_client::text::Style::color(mod_api::horizon_client::text::RED)));
        run(false, &mut message);

        assert_eq!(message.spans.len(), 2);
        let c = &message.spans[1];
        assert_eq!(c.style.color, Some(mod_api::horizon_client::text::RED), "style lost");
        assert!(!c.text.contains("<t:"), "span tag not replaced: {}", c.text);
        assert!(c.text.contains("ago"), "span not formatted: {}", c.text);
    }

    #[test]
    fn no_tag_leaves_message_byte_identical() {
        let raw = "just a normal chat line";
        let mut message = Text::literal(raw);
        run(false, &mut message);
        assert_eq!(message.plain(), raw);
    }

    #[test]
    fn mod_list_row_is_registered() {
        let mut mods = Vec::new();
        timeformatter_listed(&mut mods);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].id, "timeformatter");
    }
}
