//! Item Tooltip+: append item id, durability and stack size to tooltips.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{ItemData, Settings, extra_lines};
use mod_api::{
    ClientHost, ComponentId, Hooks, ItemResource, ItemTooltipEvent, Registrar, RegistrationContext,
    SettingSpec, Tag, Text,
};

#[derive(Default)]
struct Mod;

fn hex(rgb: u32) -> String {
    format!("#{:06X}", rgb & 0x00FF_FFFF)
}

fn tag_u32(tag: Tag) -> Option<u32> {
    match tag {
        Tag::Byte(v) => u32::try_from(i32::from(v)).ok(),
        Tag::Short(v) => u32::try_from(i32::from(v)).ok(),
        Tag::Int(v) => u32::try_from(v).ok(),
        Tag::Long(v) => u32::try_from(v).ok(),
        Tag::Float(bits) => {
            let value = f32::from_bits(bits);
            (value.is_finite() && value >= 0.0).then(|| value as u32)
        }
        Tag::Double(bits) => {
            let value = f64::from_bits(bits);
            (value.is_finite() && value >= 0.0).then(|| value as u32)
        }
        _ => None,
    }
}

/// The stack's own component value first, then the item prototype. Reading only
/// the item's defaults shows the wrong numbers when a stack overrides them.
fn component_u32(host: &dyn ClientHost, stack: &mod_api::ItemStack, id: &str) -> Option<u32> {
    ComponentId::new(id)
        .and_then(|id| ItemResource::of(stack).value(host, &id))
        .and_then(tag_u32)
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle("enable_mod", "Enable Item Tooltip+", true));
        let _ = r.register_setting(SettingSpec::toggle("show_id", "Show item ID", true));
        let _ = r.register_setting(SettingSpec::toggle("show_durability", "Show durability", true));
        let _ = r.register_setting(SettingSpec::toggle("show_stack", "Show stack size", true));
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
            max_stack: component_u32(host, event.stack, "minecraft:max_stack_size")
                .or_else(|| host.max_stack_size(event.stack.item()))
                .unwrap_or(64),
            damage: event.stack.damage(),
            max_damage: component_u32(host, event.stack, "minecraft:max_damage")
                .or_else(|| host.max_damage(event.stack.item()))
                .unwrap_or(0),
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
