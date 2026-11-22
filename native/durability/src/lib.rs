//! Durability HUD: remaining wear of equipped armour and held items.
//! Native mod for the official Horizon SDK (api 0.2).
mod logic;

use logic::{Bar, Placement, Settings, Slot, remaining, visible};
use mod_api::{
    ClientHost, ComponentId, EquipmentSlot, GuiRenderPostEvent, Hooks, ItemResource, Registrar,
    RegistrationContext, SettingSpec, Tag,
};

#[derive(Default)]
struct Mod;

const EDGE: f32 = 6.0;

/// Left edge of the panel for a placement, before the widest row is known:
/// right-hand placements anchor on the right edge and back off per row.
fn origin(placement: Placement, w: f32, h: f32) -> [f32; 2] {
    match placement {
        Placement::TopLeft => [EDGE, EDGE],
        Placement::TopRight => [w - EDGE, EDGE],
        Placement::BottomLeft => [EDGE, h - 90.0],
        Placement::BottomRight => [w - EDGE, h - 90.0],
    }
}

fn anchors_right(placement: Placement) -> bool {
    matches!(placement, Placement::TopRight | Placement::BottomRight)
}

fn api_slot(slot: Slot) -> EquipmentSlot {
    match slot {
        Slot::Head => EquipmentSlot::Head,
        Slot::Chest => EquipmentSlot::Chest,
        Slot::Legs => EquipmentSlot::Legs,
        Slot::Feet => EquipmentSlot::Feet,
        Slot::MainHand => EquipmentSlot::MainHand,
        Slot::OffHand => EquipmentSlot::OffHand,
    }
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

/// The stack's effective max damage: its own `minecraft:max_damage` component
/// first, then the item's default. Reading only the item prototype shows the
/// wrong value for a stack with a component patch (e.g. unbreakable or a
/// data-pack override).
fn effective_max_damage(host: &dyn ClientHost, stack: &mod_api::ItemStack) -> u32 {
    ComponentId::new("minecraft:max_damage")
        .and_then(|id| ItemResource::of(stack).value(host, &id))
        .and_then(tag_u32)
        .or_else(|| host.max_damage(stack.item()))
        .unwrap_or(0)
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let _ = r.register_setting(SettingSpec::toggle("enable_mod", "Enable Durability HUD", true));
        let _ = r.register_setting(SettingSpec::choice(
            "placement",
            "Panel position",
            vec![
                String::from("Top left"),
                String::from("Top right"),
                String::from("Bottom left"),
                String::from("Bottom right"),
            ],
            1,
        ));
        let _ = r.register_setting(SettingSpec::toggle("show_percent", "Show percentage", true));
        let _ = r.register_setting(SettingSpec::toggle("only_damaged", "Only damaged items", true));
    }

    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        if !host.setting_bool("enable_mod").unwrap_or(true) || event.gui_hidden {
            return;
        }
        let placement = match host.setting_choice("placement").unwrap_or(1) {
            0 => Placement::TopLeft,
            2 => Placement::BottomLeft,
            3 => Placement::BottomRight,
            _ => Placement::TopRight,
        };
        let settings = Settings {
            enable_mod: true,
            placement,
            show_percent: host.setting_bool("show_percent").unwrap_or(true),
            only_damaged: host.setting_bool("only_damaged").unwrap_or(true),
        };
        // The local player's living entity, resolved from the current camera.
        let Some(entity) = host.camera_entity() else { return; };
        let Some(living) = host.living(entity) else { return; };
        let mut bars = Vec::new();
        for slot in Slot::ALL {
            let Some(stack) = living.equipment(host, api_slot(slot)) else { continue; };
            let max = effective_max_damage(host, &stack);
            let id = stack.item().id(host);
            // Prefer the game's localised item name (Iron Helmet, Stone Axe,
            // ...), which is short and already names the piece. Fall back to
            // the English slot label when no translation exists.
            let name = host
                .translate(&format!("item.{}", id.replace(':', ".")))
                .or_else(|| host.translate(&format!("block.{}", id.replace(':', "."))))
                .unwrap_or_else(|| slot.name().to_owned());
            bars.push(Bar {
                slot,
                item: name,
                remaining: remaining(max, stack.damage()),
                max,
            });
        }
        let bars = visible(bars, &settings);
        if bars.is_empty() {
            return;
        }
        let [w, h] = event.batch.size();
        let [x, y0] = origin(placement, w, h);
        for (i, bar) in bars.iter().enumerate() {
            let row = bar.row(settings.show_percent);
            let left = if anchors_right(placement) {
                (x - event.batch.text_width(&row)).max(EDGE)
            } else {
                x
            };
            event.batch.text(&row, [left, y0 + i as f32 * 12.0], 0xFF00_0000 | bar.color(), true);
        }
    }
}
