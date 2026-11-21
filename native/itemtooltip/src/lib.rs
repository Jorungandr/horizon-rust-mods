//! Item Tooltip+: append item id, durability and stack size to tooltips.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{ItemData, Settings, extra_lines};
use mod_api::{
    ClientHost, Hooks, ItemTooltipEvent, Registrar, RegistrationContext, SettingSpec, Text,
};

#[derive(Default)]
struct Mod;

fn hex(rgb: u32) -> String {
    format!("#{:06X}", rgb & 0x00FF_FFFF)
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle("enable_mod", "tip.setting.enable", true));
        let _ = r.register_setting(SettingSpec::toggle("show_id", "tip.setting.id", true));
        let _ = r.register_setting(SettingSpec::toggle("show_durability", "tip.setting.durability", true));
        let _ = r.register_setting(SettingSpec::toggle("show_stack", "tip.setting.stack", true));
    }

    fn item_tooltip(&self, host: &dyn ClientHost, event: &mut ItemTooltipEvent<'_>) {
        let settings = Settings {
            enable_mod: host.setting_bool("enable_mod").unwrap_or(true),
            show_id: host.setting_bool("show_id").unwrap_or(true),
            show_durability: host.setting_bool("show_durability").unwrap_or(true),
            show_stack: host.setting_bool("show_stack").unwrap_or(true),
        };
        let id = event.stack.item().id(host);
        let data = ItemData {
            id: &id,
            count: event.stack.count(),
            max_stack: host.max_stack_size(event.stack.item()).unwrap_or(64),
            damage: event.stack.damage(),
            max_damage: host.max_damage(event.stack.item()).unwrap_or(0),
        };
        for line in extra_lines(&settings, &data) {
            let mut text = Text::literal(line.text);
            text.style.color = Some(hex(line.color));
            // The trailing u32 is RGBA, not RGB. Passing `line.color` straight
            // through gives GRAY (0x808080) an alpha of zero and hides the ID
            // and stack lines entirely; GREEN is only 1/3 opaque. The colour is
            // already on the Text, so keep the trailing value fully opaque.
            event.lines.push((text, u32::MAX));
        }
    }
}
