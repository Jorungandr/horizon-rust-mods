// Compiles the REAL chat.rs + runtime.rs against a stubbed engine API.
// Module layout mirrors the injection target:
//   chat.rs is injected at the head of `Game::native_client_chat_received`,
//   so it sees `crate::game::Game` and `crate::game::whenchat`.
//
// The real helper file contains exactly one PRIVATE `fn` (a hard requirement of
// horizon-mixins). We `include!` it into the test module so its private entry
// point is callable while remaining byte-for-byte the injected source.
//
// The Text used here is the ENGINE-side Text (horizon_client::text::Text, a
// flat Vec<Span>), which is what the helper actually receives. See mod_api stub.

pub mod game {
    /// Stub of the game type the helper takes as `&mut Game`.
    pub struct Game<'a> {
        _marker: std::marker::PhantomData<&'a ()>,
    }

    impl Game<'_> {
        pub fn new() -> Self {
            Game { _marker: std::marker::PhantomData }
        }

        /// Stub of the runtime toggle the helper calls.
        pub fn native_on(&self, _id: &str) -> bool {
            true
        }
    }

    /// The real runtime logic, compiled from the actual bundle file.
    #[path = "../../../mixins/whenchat/runtime.rs"]
    pub mod whenchat;
}

#[cfg(test)]
mod tests {
    use crate::game::Game;
    use mod_api::horizon_client::text::Text;

    // Expand the real, single-private-fn helper source into this module.
    include!("../../mixins/whenchat/chat.rs");

    #[test]
    fn prefixes_a_normal_chat_line() {
        let mut game = Game::new();
        let mut message = Text::literal("hello world");
        // Real hook params: (game, system, overlay, message); system=false, overlay=false.
        whenchat_client_chat_received(&mut game, false, false, &mut message);
        let out = message.plain();
        assert!(out.ends_with(" hello world"), "unexpected: {out}");
        assert!(out.starts_with('[') && out.contains(':'), "unexpected: {out}");
    }

    #[test]
    fn leaves_action_bar_overlay_untouched() {
        let mut game = Game::new();
        let mut message = Text::literal("action bar text");
        whenchat_client_chat_received(&mut game, false, true, &mut message);
        assert_eq!(message.plain(), "action bar text");
    }

    #[test]
    fn empty_message_is_not_decorated() {
        let mut game = Game::new();
        let mut message = Text::literal("");
        whenchat_client_chat_received(&mut game, false, false, &mut message);
        assert_eq!(message.plain(), "");
    }
}
