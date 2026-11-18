// Compiles the REAL chat.rs / settings.rs / click.rs / runtime.rs against a
// stubbed Horizon API. Module layout mirrors the injection target: helpers see
// `crate::game::Game`, `crate::game::chattimestamps` and `crate::game::native_hooks`.
//
// The real helper files each contain exactly one PRIVATE `fn` (a hard
// requirement of horizon-mixins). To test them we `include!` the files directly
// into the test module, so their private functions become callable here while
// remaining byte-for-byte the same sources that get injected into the game.

// The real chat.rs refers to the engine's `horizon_client::text`; bind our stub
// crate under that name so the byte-for-byte source compiles unchanged.
extern crate mod_api as horizon_client;

pub mod game {
    /// Stub game type. Settings are stored in a simple map so click/settings
    /// helpers can be exercised end-to-end.
    pub struct Game<'a> {
        settings: std::cell::RefCell<std::collections::HashMap<String, String>>,
        _marker: std::marker::PhantomData<&'a ()>,
    }

    impl Game<'_> {
        pub fn new() -> Self {
            Game {
                settings: std::cell::RefCell::new(std::collections::HashMap::new()),
                _marker: std::marker::PhantomData,
            }
        }

        /// Stub of `native_setting`.
        pub fn native_setting(&self, _id: &str, key: &str) -> Option<String> {
            self.settings.borrow().get(key).cloned()
        }

        /// Stub of `native_set_setting`.
        pub fn native_set_setting(&self, _id: &str, key: &str, value: &str) {
            self.settings.borrow_mut().insert(key.to_owned(), value.to_owned());
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
    #[path = "../../../mixins/chattimestamps/runtime.rs"]
    pub mod chattimestamps;
}

#[cfg(test)]
mod tests {
    use crate::game::Game;
    use mod_api::horizon_client::text::Text;

    // Expand the real, single-private-fn helper sources into this module so the
    // private entry points are directly callable by the tests below.
    include!("../../mixins/chattimestamps/chat.rs");
    include!("../../mixins/chattimestamps/settings.rs");
    include!("../../mixins/chattimestamps/click.rs");
    include!("../../mixins/chattimestamps/list.rs");

    // Real hook params bound by the toml: (game, overlay, message).
    fn run(game: &mut Game<'_>, overlay: bool, message: &mut Text) {
        chattimestamps_client_chat_received(game, overlay, message);
    }

    #[test]
    fn prefix_mode_adds_styled_timestamp_and_keeps_message() {
        let mut game = Game::new();
        let mut message = Text::literal("hello world");
        run(&mut game, false, &mut message);
        let out = message.plain();
        assert!(out.ends_with("hello world"), "message lost: {out}");
        assert!(out.contains(':'), "no time in: {out}");
        // Prefix spans come first (stamped), original span is last.
        assert!(message.spans.len() >= 2, "expected prefix + original spans");
        assert!(message.spans.iter().any(|s| s.style.color.is_some()), "no coloured stamp span");
        assert_eq!(message.spans.last().unwrap().text, "hello world");
    }

    #[test]
    fn overlay_is_left_untouched() {
        let mut game = Game::new();
        let mut message = Text::literal("action bar");
        run(&mut game, true, &mut message);
        assert_eq!(message.plain(), "action bar");
        assert_eq!(message.spans.len(), 1);
        assert!(message.spans[0].style.hover.is_none());
    }

    #[test]
    fn disabled_mod_is_left_untouched() {
        let mut game = Game::new();
        game.native_set_setting("chattimestamps", "enable_mod", "false");
        let mut message = Text::literal("hi");
        run(&mut game, false, &mut message);
        assert_eq!(message.plain(), "hi");
    }

    #[test]
    fn hover_mode_attaches_hover_and_leaves_visible_text() {
        let mut game = Game::new();
        game.native_set_setting("chattimestamps", "enable_hover", "true");
        let mut message = Text::literal("visible");
        run(&mut game, false, &mut message);
        assert_eq!(message.plain(), "visible");
        // Hover is attached to each span; its text carries the timestamp.
        let hover = message.spans[0].style.hover.as_ref().expect("no hover attached");
        assert!(hover.plain().contains(':'), "no time in hover: {}", hover.plain());
    }

    #[test]
    fn settings_page_and_click_round_trip() {
        let mut game = Game::new();
        let mut buttons = Vec::new();
        chattimestamps_page(&mut game, "chattimestamps", &mut buttons);
        assert_eq!(buttons.len(), 3);
        assert_eq!(buttons[0], "Timestamps: ON");
        chattimestamps_click(&mut game, "chattimestamps", 0);
        assert_eq!(game.native_setting("chattimestamps", "enable_mod").as_deref(), Some("false"));
        buttons.clear();
        chattimestamps_page(&mut game, "chattimestamps", &mut buttons);
        assert_eq!(buttons[0], "Timestamps: OFF");
    }

    #[test]
    fn mod_list_row_is_registered() {
        let mut mods = Vec::new();
        chattimestamps_listed(&mut mods);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].id, "chattimestamps");
    }

    #[test]
    fn wrong_mod_id_is_ignored() {
        let mut game = Game::new();
        let mut buttons = Vec::new();
        chattimestamps_page(&mut game, "some_other_mod", &mut buttons);
        assert!(buttons.is_empty());
    }
}
