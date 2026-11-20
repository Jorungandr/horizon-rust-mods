//! AppleSkin: the saturation behind the hunger bar, and what a food item restores.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{halves_on, highlight, parse_food, tooltip_lines};
use mod_api::{
    ClientHost, GuiRenderPostEvent, Hooks, ItemTooltipEvent, Registrar, RegistrationContext,
    SettingSpec, Text,
};

#[derive(Default)]
struct Mod;

/// The gold AppleSkin uses for saturation.
const SATURATION: u32 = 0xFFD9_A21B;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle("enable_mod", "AppleSkin", true));
        let _ = r.register_setting(SettingSpec::toggle(
            "show_saturation",
            "Saturation on the hunger bar",
            true,
        ));
        let _ = r.register_setting(SettingSpec::toggle(
            "show_tooltip",
            "Food values in item tooltips",
            true,
        ));
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        if event.gui_hidden
            || !host.setting_bool("enable_mod").unwrap_or(true)
            || !host.setting_bool("show_saturation").unwrap_or(true)
        {
            return;
        }
        // The hunger bar is drawn for a survival player only.
        let Some(player) = host.local_player() else {
            return;
        };
        if player.is_creative(host).unwrap_or(false) || player.is_spectator(host).unwrap_or(false) {
            return;
        }
        let Some(food) = player.food(host) else {
            return;
        };
        let [width, height] = event.batch.size();
        for index in 0..10 {
            if let Some(rect) = highlight(width, height, index, halves_on(food.saturation, index)) {
                event.batch.fill(rect, SATURATION);
            }
        }
    }

    fn item_tooltip(&self, host: &dyn ClientHost, event: &mut ItemTooltipEvent<'_>) {
        if !host.setting_bool("enable_mod").unwrap_or(true)
            || !host.setting_bool("show_tooltip").unwrap_or(true)
        {
            return;
        }
        let Some(tag) = host.item_default_component(event.stack.item(), "minecraft:food") else {
            return;
        };
        let Some(value) = parse_food(&tag) else {
            return;
        };
        for (line, rgb) in tooltip_lines(value) {
            let mut text = Text::literal(line);
            text.style.color = Some(format!("#{:06X}", rgb & 0x00FF_FFFF));
            event.lines.push((text, rgb));
        }
    }
}
