#!/usr/bin/env python3
"""Write the three ports as new-format (trait-mod) projects under native-v2/.

The format is the one `cargo-horizon-mod` scaffolds and `horizon-launcher
native-build` compiles: a `horizon-mod.toml` naming the mod and the API crate
it is built against, an `api/` crate that depends on `horizon-client`, and
`src/lib.rs` implementing `mod_api::Hooks`.

The old `.hnmod`/`horizon-mixin.toml` packages describe source injections; a
trait mod implements the hook methods directly, so these projects have no
`[[injection]]` sections at all.
"""
import pathlib, shutil

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "native-v2"
REV = "9ee13cd038559db05de6a4ced4496028ce409564"
CLIENT = "../../../build-win/horizon-src/crates/horizon-client"

MODS = {
    "whenchat": ROOT / "whenchat-horizon/mixins/whenchat/runtime.rs",
    "chattimestamps": ROOT / "chattimestamps-horizon/mixins/chattimestamps/runtime.rs",
    "timeformatter": ROOT / "timeformatter-horizon/mixins/timeformatter/runtime.rs",
    "coords": ROOT / "ports/coords.rs",
}

NAMES = {
    "whenchat": "WhenChat",
    "chattimestamps": "Plague's Chat Timestamps",
    "timeformatter": "Time Formatter",
    "coords": "Coordinates HUD",
}

MANIFEST = """\
# {name}: a native trait mod (docs/native-mods.md, "Trait mods").
# Build it with `horizon-launcher native-build <this folder> --trust-native-code`.
[mod]
id = "{id}"
version = "0.1.0"

[compatibility]
horizon_revision = "{rev}"

[api]
path = "api"
"""

API_TOML = """\
[package]
name = "mod-{id}"
version = "0.1.0"
edition = "2024"

[lib]
path = "../src/lib.rs"

[dependencies]
# The launcher points this at the Horizon source it links the mod into; the
# path below is the local checkout used to type-check the port.
horizon-client = {{ path = "{client}" }}
chrono = {{ version = "0.4.42", default-features = false, features = ["clock", "std"] }}
"""

COMMON = """\
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
"""

LIB = {}

LIB["whenchat"] = (
    "use runtime::{decorate, local_prefix, Line, DESCRIPTION, ID, NAME, VERSION};\n\n"
    + COMMON
    + """
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
"""
)

LIB["timeformatter"] = (
    "use runtime::{format_literal, DESCRIPTION, ID, NAME, VERSION};\n\n"
    + COMMON
    + """
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
"""
)

LIB["chattimestamps"] = (
    "use horizon_client::mod_api::{SettingsClickEvent, SettingsEvent};\n"
    "use runtime::{settings, timestamp_now, Segment, Settings, DESCRIPTION, FORMAT_PRESETS, ID, NAME, VERSION};\n\n"
    + COMMON
    + """
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
"""
)

LIB["coords"] = (
    "use horizon_client::extras::Corner;\n"
    "use horizon_client::mod_api::{SettingsClickEvent, SettingsEvent, HudEvent};\n"
    "use runtime::{lines, DESCRIPTION, ID, NAME, Placement, Settings, VERSION};\n\n"
    + COMMON
    + """
fn read_settings(host: &dyn Host) -> Settings {
    Settings::from_lookup(|key| host.setting(ID, key))
}

fn corner(placement: Placement) -> Corner {
    match placement {
        Placement::TopLeft => Corner::TopLeft,
        Placement::TopRight => Corner::TopRight,
        Placement::BottomLeft => Corner::BottomLeft,
        Placement::BottomRight => Corner::BottomRight,
    }
}

impl Hooks for Mod {
    fn mods(&self, _host: &mut dyn Host, event: &mut ModsEvent<'_>) {
        push_row(event);
    }

    /// The panel, drawn into the HUD batch the game hands every mod.
    fn hud(&self, host: &mut dyn Host, event: &mut HudEvent<'_>) {
        let settings = read_settings(&*host);
        if !settings.enable_mod {
            return;
        }
        let Some(camera) = host.camera() else {
            return;
        };
        let rows = lines(&settings, camera.position, camera.yaw, &camera.dimension);
        if rows.is_empty() {
            return;
        }
        const LINE: f32 = 10.0;
        const PAD: f32 = 2.0;
        let width = rows
            .iter()
            .map(|row| host.text_width(row))
            .fold(0.0f32, f32::max)
            + PAD * 2.0;
        let height = rows.len() as f32 * LINE + PAD * 2.0;
        let (x, y) = event.corners.place(corner(settings.placement), width, height);
        host.fill(event.batch, [x, y, width, height], 0x8000_0000);
        for (i, row) in rows.iter().enumerate() {
            host.text(
                event.batch,
                row,
                [x + PAD, y + PAD + i as f32 * LINE],
                0xFFFFFF,
                true,
            );
        }
    }

    fn settings(&self, host: &mut dyn Host, event: &mut SettingsEvent<'_>) {
        if event.mod_id != ID {
            return;
        }
        event.buttons.extend(read_settings(&*host).labels());
    }

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
}
"""
)

for mod_id, runtime in MODS.items():
    d = OUT / mod_id
    (d / "src").mkdir(parents=True, exist_ok=True)
    (d / "api").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(runtime, d / "src/runtime.rs")
    (d / "horizon-mod.toml").write_text(
        MANIFEST.format(name=NAMES[mod_id], id=mod_id, rev=REV), encoding="utf-8"
    )
    (d / "api/Cargo.toml").write_text(
        API_TOML.format(id=mod_id, client=CLIENT), encoding="utf-8"
    )
    (d / "src/lib.rs").write_text(LIB[mod_id], encoding="utf-8")
    print("wrote", d)
