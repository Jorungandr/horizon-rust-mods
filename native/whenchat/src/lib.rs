//! WhenChat: prepends `[HH:MM:SS]` (local time) to every incoming chat message.
//! Port of https://github.com/ErikEdits/WhenChat (MIT, (c) ErikEdits).
//!
//! The original line's component tree is preserved: the timestamp is a literal
//! component prepended as a parent's content, with the whole original message
//! kept as its only child, so colours, hovers and translation keys survive.
use mod_api::{ClientChatReceivedEvent, ClientHost, Flow, Hooks, Text};

mod local_time;

use local_time::hms;

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// NeoForge's `ClientChatReceivedEvent`: prepend the local timestamp.
    fn client_chat_received(
        &self,
        _host: &mut dyn ClientHost,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        if event.overlay {
            return Flow::Continue;
        }
        let original = std::mem::take(&mut *event.message);
        *event.message = Text::literal(format!("{} ", hms())).append(original);
        Flow::Continue
    }
}
