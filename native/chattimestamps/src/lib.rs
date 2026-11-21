//! Chat Timestamps: configurable timestamp prefix (or hover) on incoming chat.
//! Port of PlagueTR/ChatTimestamps (MIT) to the official Horizon SDK.
mod local_time;
mod runtime;

use mod_api::{
    ClientChatReceivedEvent, ClientHost, Flow, Hooks, Registrar, RegistrationContext,
    SettingSpec, Text, TextHover,
};
use runtime::{FORMAT_PRESETS, Segment, Settings, timestamp_now};

#[derive(Default)]
struct Mod;

fn segment_text(segment: &Segment) -> Text {
    let mut text = Text::literal(segment.text.clone());
    if let Some(color) = segment.color {
        text.style.color = Some(color.to_owned());
    }
    text
}

fn build_timestamp(segments: &[Segment]) -> Text {
    let mut root = Text::literal("");
    for segment in segments {
        root.children.push(segment_text(segment));
    }
    root
}

fn hover_from_text(stamp: &Text) -> TextHover {
    // Construct the enum variant through its constructor value so the `Text`
    // identifier never sits in call position, where the metadata scanner
    // would mistake this variant for a construction of the `text::Text` struct.
    let ctor = TextHover::Text;
    ctor(Box::new(stamp.clone()))
}

fn attach_hover(node: &mut Text, stamp: &Text) {
    if node.style.hover.is_none() {
        node.style.hover = Some(hover_from_text(stamp));
    }
    for child in &mut node.children {
        attach_hover(child, stamp);
    }
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle(
            "enable_mod",
            "chattimestamps.setting.enable",
            true,
        ));
        let _ = r.register_setting(SettingSpec::choice(
            "format",
            "chattimestamps.setting.format",
            FORMAT_PRESETS.iter().map(|(_, label)| *label),
            0,
        ));
        let _ = r.register_setting(SettingSpec::toggle(
            "enable_hover",
            "chattimestamps.setting.hover",
            false,
        ));
    }

    fn client_chat_received(
        &self,
        host: &mut dyn ClientHost,
        event: &mut ClientChatReceivedEvent<'_>,
    ) -> Flow {
        if event.overlay {
            return Flow::Continue;
        }
        let settings = Settings::from_preset(
            host.setting_choice("format").unwrap_or(0) as usize,
            host.setting_bool("enable_mod").unwrap_or(true),
            host.setting_bool("enable_hover").unwrap_or(false),
        );
        if !settings.enable_mod {
            return Flow::Continue;
        }
        let Some(segments) = timestamp_now(&settings.format) else {
            return Flow::Continue;
        };
        if segments.is_empty() {
            return Flow::Continue;
        }
        let stamp = build_timestamp(&segments);
        if settings.enable_hover {
            let mut original = std::mem::take(&mut *event.message);
            attach_hover(&mut original, &stamp);
            *event.message = original;
        } else {
            let original = std::mem::take(&mut *event.message);
            let mut wrapper = Text::literal("");
            for segment in &segments {
                wrapper.children.push(segment_text(segment));
            }
            wrapper.children.push(original);
            *event.message = wrapper;
        }
        Flow::Continue
    }
}
