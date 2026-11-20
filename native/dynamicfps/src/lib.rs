//! Dynamic FPS: hold the frame rate down while the window is not the one being looked at.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{Caps, MINIMIZED, UNFOCUSED, limit_for, pick};
use mod_api::{ClientHost, FramerateLimitEvent, Hooks, Registrar, RegistrationContext, SettingSpec};

#[derive(Default)]
struct Mod;

fn labels(values: &[i32]) -> Vec<String> {
    values
        .iter()
        .map(|value| format!("{value} FPS"))
        .collect()
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle("enable_mod", "Dynamic FPS", true));
        let _ = r.register_setting(SettingSpec::choice(
            "unfocused_fps",
            "Frame rate in the background",
            labels(&UNFOCUSED),
            2,
        ));
        let _ = r.register_setting(SettingSpec::choice(
            "minimized_fps",
            "Frame rate when minimised",
            labels(&MINIMIZED),
            0,
        ));
    }

    fn framerate_limit(&self, host: &dyn ClientHost, event: &mut FramerateLimitEvent) {
        if !host.setting_bool("enable_mod").unwrap_or(true) {
            return;
        }
        let caps = Caps {
            unfocused: pick(
                &UNFOCUSED,
                host.setting_choice("unfocused_fps").unwrap_or(2),
                Caps::default().unfocused,
            ),
            minimized: pick(
                &MINIMIZED,
                host.setting_choice("minimized_fps").unwrap_or(0),
                Caps::default().minimized,
            ),
        };
        event.limit = limit_for(event.limit, event.focused, event.minimized, caps);
    }
}
