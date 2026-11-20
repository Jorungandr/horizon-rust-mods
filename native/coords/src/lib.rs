//! Coordinates HUD: position, facing and dimension in a screen corner.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{Placement, Settings, lines};
use mod_api::{
    ClientHost, GuiRenderPostEvent, Hooks, Registrar, RegistrationContext, SettingSpec,
};

#[derive(Default)]
struct Mod;

const EDGE: f32 = 6.0;

/// Map a placement to a GUI pixel origin: right-hand placements anchor on the
/// screen's right edge, so each row is shifted left by its measured width.
fn origin(placement: Placement, w: f32, h: f32) -> [f32; 2] {
    match placement {
        Placement::TopLeft => [EDGE, EDGE],
        Placement::TopRight => [w - EDGE, EDGE],
        Placement::BottomLeft => [EDGE, h - 48.0],
        Placement::BottomRight => [w - EDGE, h - 48.0],
    }
}

fn anchors_right(placement: Placement) -> bool {
    matches!(placement, Placement::TopRight | Placement::BottomRight)
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle(
            "enable_mod",
            "coords.setting.enable",
            true,
        ));
        let _ = r.register_setting(SettingSpec::choice(
            "placement",
            "coords.setting.placement",
            vec![
                String::from("coords.corner.top_left"),
                String::from("coords.corner.top_right"),
                String::from("coords.corner.bottom_left"),
                String::from("coords.corner.bottom_right"),
            ],
            0,
        ));
        let _ = r.register_setting(SettingSpec::toggle(
            "show_facing",
            "coords.setting.facing",
            true,
        ));
        let _ = r.register_setting(SettingSpec::toggle(
            "show_dimension",
            "coords.setting.dimension",
            true,
        ));
        let _ = r.register_setting(SettingSpec::choice(
            "decimals",
            "coords.setting.decimals",
            vec![
                String::from("0"), String::from("1"), String::from("2"),
            ],
            1,
        ));
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        // Settings are stored by the game; read them back each frame.
        let on = host.setting_bool("enable_mod").unwrap_or(true);
        if !on || event.gui_hidden {
            return;
        }
        let placement = match host.setting_choice("placement").unwrap_or(0) {
            1 => Placement::TopRight,
            2 => Placement::BottomLeft,
            3 => Placement::BottomRight,
            _ => Placement::TopLeft,
        };
        let decimals = match host.setting_choice("decimals").unwrap_or(1) {
            0 => 0,
            2 => 2,
            _ => 1,
        };
        let settings = Settings {
            enable_mod: on,
            placement,
            show_facing: host.setting_bool("show_facing").unwrap_or(true),
            show_dimension: host.setting_bool("show_dimension").unwrap_or(true),
            decimals,
        };
        let Some(cam) = host.camera() else { return; };
        let rows = lines(&settings, cam.position, cam.yaw, &cam.dimension);
        if rows.is_empty() {
            return;
        }
        let [w, h] = event.batch.size();
        let [x, y0] = origin(settings.placement, w, h);
        const LINE: f32 = 12.0;
        for (i, row) in rows.iter().enumerate() {
            let left = if anchors_right(settings.placement) {
                (x - event.batch.text_width(row)).max(EDGE)
            } else {
                x
            };
            event.batch.text(row, [left, y0 + i as f32 * LINE], 0xFFFF_FFFF, true);
        }
    }
}
