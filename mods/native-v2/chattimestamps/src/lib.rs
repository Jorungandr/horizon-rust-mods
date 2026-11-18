use horizon_client::mod_api::{SettingsClickEvent, SettingsEvent};
use runtime::{settings, timestamp_now, Segment, Settings, DESCRIPTION, FORMAT_PRESETS, ID, NAME, VERSION};

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

/// The stored settings, with `format` repaired: `Host::setting` trims every
/// value, but the format's trailing space separates the timestamp from the
/// message.
fn read_settings(host: &dyn Host) -> Settings {
    let mut settings = settings(|key| host.setting(ID, key));
    if let Some((value, _)) = FORMAT_PRESETS
        .iter()
        .find(|(value, _)| value.trim() == settings.format.trim())
    {
        settings.format = (*value).to_owned();
    }
    settings
}

fn seg_text(text: &str, color: Option<&str>) -> Text {
    match color.and_then(horizon_client::text::named_color) {
        Some(c) => Text::colored(text, c),
        None => Text::literal(text),
    }
}

fn build_timestamp(segments: &[Segment]) -> Text {
    let mut out = Text::empty();
    for seg in segments {
        out.push(seg_text(&seg.text, seg.color));
    }
    out
}

impl Hooks for Mod {
    fn mods(&self, _host: &mut dyn Host, event: &mut ModsEvent<'_>) {
        push_row(event);
    }

    /// The buttons of this mod's settings page.
    fn settings(&self, host: &mut dyn Host, event: &mut SettingsEvent<'_>) {
        if event.mod_id != ID {
            return;
        }
        event.buttons.extend(read_settings(&*host).labels());
    }

    /// Button `event.button` was clicked: advance it and keep the choice.
    fn settings_click(&self, host: &mut dyn Host, event: &mut SettingsClickEvent<'_>) {
        if event.mod_id != ID {
            return;
        }
        let mut settings = read_settings(&*host);
        settings.click(event.button);
        for (key, value) in settings.stored() {
            host.set_setting(ID, key, &value);
        }
    }

    /// Add the configured timestamp: a hover tooltip on every span, or
    /// leading spans that keep the original spans (and their styling).
    fn client_chat_received(
        &self,
        host: &mut dyn Host,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        let settings = read_settings(&*host);
        if !settings.enable_mod {
            return Flow::Continue;
        }
        let Some(segments) = timestamp_now(&settings.format) else {
            return Flow::Continue;
        };
        if segments.is_empty() {
            return Flow::Continue;
        }

        if settings.enable_hover {
            let stamp = build_timestamp(&segments);
            let mut rebuilt = Text::empty();
            for span in &event.message.spans {
                let mut style = span.style.clone();
                style.hover = Some(Box::new(stamp.clone()));
                rebuilt.push(Text::styled(span.text.clone(), style));
            }
            *event.message = rebuilt;
        } else {
            let original = std::mem::take(event.message);
            let mut wrapper = Text::empty();
            for seg in &segments {
                wrapper.push(seg_text(&seg.text, seg.color));
            }
            wrapper.push(original);
            *event.message = wrapper;
        }
        Flow::Continue
    }
}
