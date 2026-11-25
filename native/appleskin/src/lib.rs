//! AppleSkin: the saturation behind the hunger bar, and what a food item restores.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{SATURATION, cell_color, icon_origin, parse_food, saturation_icons, tooltip_lines};
use mod_api::{
    ClientHost, ComponentId, GuiLayerPostEvent, Hooks, HudMode, ItemResource, ItemTooltipEvent,
    Registrar, RegistrationContext, SettingSpec, Text,
};

#[derive(Default)]
struct Mod;

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

    fn gui_layer_post(&self, host: &dyn ClientHost, event: &mut GuiLayerPostEvent<'_>) {
        // Draw with the hunger bar so screen backgrounds affect both the same way.
        if event.layer != "minecraft:player_health"
            || host.hud_mode() != HudMode::AllVisible
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
        let [width, height] = event.draw.size();

        // Draw our original golden-brown saturation icons pixel by pixel, in
        // the palette chosen for this port. Each icon is a 9x9 ASCII grid;
        // '.' cells are transparent and skipped.
        for (index, frame) in saturation_icons(food.saturation) {
            let [ox, oy] = icon_origin(width, height, index);
            for (row, line) in SATURATION[frame].iter().enumerate() {
                for (col, ch) in line.bytes().enumerate() {
                    if let Some(argb) = cell_color(ch) {
                        event.draw.fill([ox + col as f32, oy + row as f32, 1.0, 1.0], argb);
                    }
                }
            }
        }
    }

    fn item_tooltip(&self, host: &dyn ClientHost, event: &mut ItemTooltipEvent<'_>) {
        if !host.setting_bool("enable_mod").unwrap_or(true)
            || !host.setting_bool("show_tooltip").unwrap_or(true)
        {
            return;
        }
        // Read the *effective* component of the actual stack, not the item's
        // prototype default: a stack can override or remove `minecraft:food`,
        // and that is the value the player is actually looking at.
        let Some(food_id) = ComponentId::new("minecraft:food") else {
            return;
        };
        let resource = ItemResource::of(event.stack);
        let Some(tag) = resource.value(host, &food_id) else {
            return;
        };
        let Some(value) = parse_food(&tag) else {
            return;
        };
        for (line, rgb) in tooltip_lines(value) {
            let mut text = Text::literal(line);
            text.style.color = Some(format!("#{:06X}", rgb & 0x00FF_FFFF));
            // Trailing u32 is RGBA: a bare 0xRRGGBB has alpha 0 and hides the
            // line. u32::MAX keeps the colour set on the Text above.
            event.lines.push((text, u32::MAX));
        }
    }
}
