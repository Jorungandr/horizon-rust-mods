use runtime::{decorate, local_prefix, Line, DESCRIPTION, ID, NAME, VERSION};

mod runtime;

// The shared header: a port uses the subset its hooks need.
#[allow(unused_imports)]
use horizon_client::mod_api::{
    ClientChatReceivedEvent, Flow, Hooks, Host, ModsEvent, NativeMod, Text,
};

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod)
}

struct Mod;

/// The mod's row in Options > Mods.
fn push_row(event: &mut ModsEvent<'_>) {
    event.mods.push(NativeMod {
        id: ID.to_owned(),
        title: format!("{NAME} {VERSION}"),
        description: DESCRIPTION.to_owned(),
        keys: Vec::new(),
    });
}

impl Hooks for Mod {
    fn mods(&self, _host: &mut dyn Host, event: &mut ModsEvent<'_>) {
        push_row(event);
    }

    /// Prepend `[HH:mm:ss] ` to the line, as the Java mod's mixin does.
    fn client_chat_received(
        &self,
        _host: &mut dyn Host,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        if event.overlay {
            return Flow::Continue;
        }
        let original = event.message.plain();
        if original.is_empty() {
            return Flow::Continue;
        }
        let line = Line {
            text: &original,
            system: event.system,
            overlay: event.overlay,
        };
        let Some(decorated) = decorate(line, &local_prefix()) else {
            return Flow::Continue;
        };
        *event.message = Text::literal(decorated);
        Flow::Continue
    }
}
