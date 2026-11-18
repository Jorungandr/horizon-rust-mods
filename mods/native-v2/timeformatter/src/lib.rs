use runtime::{format_literal, DESCRIPTION, ID, NAME, VERSION};

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

    /// Rewrite every well-formed `<t:...>` in the line, keeping each span's
    /// style. Rebuilt through `Text::styled` so the component (`source`) and
    /// the spans the game draws stay in step.
    fn client_chat_received(
        &self,
        _host: &mut dyn Host,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        if !event
            .message
            .spans
            .iter()
            .any(|s| format_literal(&s.text).is_some())
        {
            return Flow::Continue;
        }
        let mut rebuilt = Text::empty();
        for span in &event.message.spans {
            let text = format_literal(&span.text).unwrap_or_else(|| span.text.clone());
            rebuilt.push(Text::styled(text, span.style.clone()));
        }
        *event.message = rebuilt;
        Flow::Continue
    }
}
