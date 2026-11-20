//! FPS HUD: frames per second in a screen corner.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{Placement, Settings, fps_color, lines};
use mod_api::{ClientHost, GuiRenderPostEvent, Hooks, Registrar, RegistrationContext, SettingSpec};

#[derive(Default)]
struct Mod;

const EDGE: f32 = 6.0;

/// Left edge of the readout: right-hand placements anchor on the screen's
/// right edge and back off by each row's measured width.
fn origin(placement: Placement, w: f32, h: f32) -> [f32; 2] {
    match placement {
        Placement::TopLeft => [EDGE, EDGE],
        Placement::TopRight => [w - EDGE, EDGE],
        Placement::BottomLeft => [EDGE, h - 36.0],
        Placement::BottomRight => [w - EDGE, h - 36.0],
    }
}

fn anchors_right(placement: Placement) -> bool {
    matches!(placement, Placement::TopRight | Placement::BottomRight)
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle("enable_mod", "fps.setting.enable", true));
        let _ = r.register_setting(SettingSpec::choice(
            "placement",
            "fps.setting.placement",
            vec![
                String::from("top_left"),
                String::from("top_right"),
                String::from("bottom_left"),
                String::from("bottom_right"),
            ],
            3,
        ));
        let _ = r.register_setting(SettingSpec::toggle("detailed", "fps.setting.detailed", true));
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        if !host.setting_bool("enable_mod").unwrap_or(true) || event.gui_hidden {
            return;
        }
        let placement = match host.setting_choice("placement").unwrap_or(3) {
            0 => Placement::TopLeft,
            1 => Placement::TopRight,
            2 => Placement::BottomLeft,
            _ => Placement::BottomRight,
        };
        let settings = Settings {
            enable_mod: true,
            placement,
            detailed: host.setting_bool("detailed").unwrap_or(true),
        };
        let stats = host.client_session().client_frame_stats();
        let Some(stats) = stats else { return; };
        let fps = stats.fps.round().max(0.0) as u32;
        let frame_ms = stats.frame_time_ns.map(|ns| ns / 1_000_000);
        let rows = lines(&settings, fps, frame_ms);
        let [w, h] = event.batch.size();
        let [x, y0] = origin(placement, w, h);
        let color = 0xFF00_0000 | fps_color(fps);
        for (i, row) in rows.iter().enumerate() {
            let left = if anchors_right(placement) {
                (x - event.batch.text_width(row)).max(EDGE)
            } else {
                x
            };
            event.batch.text(row, [left, y0 + i as f32 * 12.0], color, true);
        }
    }
}
