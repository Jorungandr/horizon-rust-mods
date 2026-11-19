//! Item tooltip extras: append the item's numeric id, remaining durability
//! and stack size to any item tooltip.
//!
//! The line-building is pure under a small snapshot of the item data, so it
//! can be tested without a game. The engine adapter (reading `ItemStack` and
//! `ItemInfo`, appending `Text` lines) is appended at injection time (see
//! `tools/inject_all_ports.py`).

pub const ID: &str = "itemtooltip";
pub const NAME: &str = "Item Tooltip+";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Adds the item id, durability and stack size to item tooltips.";

const KEY_ENABLE: &str = "enable_mod";
const KEY_SHOW_ID: &str = "show_id";
const KEY_SHOW_DURABILITY: &str = "show_durability";
const KEY_SHOW_STACK: &str = "show_stack";

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub show_id: bool,
    pub show_durability: bool,
    pub show_stack: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            show_id: true,
            show_durability: true,
            show_stack: true,
        }
    }
}

impl Settings {
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let defaults = Self::default();
        let on_off = |key: &str, fallback: bool| match lookup(key).as_deref() {
            Some("on") | Some("true") => true,
            Some("off") | Some("false") => false,
            _ => fallback,
        };
        Self {
            enable_mod: on_off(KEY_ENABLE, defaults.enable_mod),
            show_id: on_off(KEY_SHOW_ID, defaults.show_id),
            show_durability: on_off(KEY_SHOW_DURABILITY, defaults.show_durability),
            show_stack: on_off(KEY_SHOW_STACK, defaults.show_stack),
        }
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            (KEY_ENABLE, on_off(self.enable_mod).to_owned()),
            (KEY_SHOW_ID, on_off(self.show_id).to_owned()),
            (KEY_SHOW_DURABILITY, on_off(self.show_durability).to_owned()),
            (KEY_SHOW_STACK, on_off(self.show_stack).to_owned()),
        ]
    }

    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("Tooltip+: {}", on_off(self.enable_mod).to_uppercase()),
            format!("Item id: {}", on_off(self.show_id).to_uppercase()),
            format!("Durability: {}", on_off(self.show_durability).to_uppercase()),
            format!("Stack size: {}", on_off(self.show_stack).to_uppercase()),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enable_mod = !self.enable_mod,
            1 => self.show_id = !self.show_id,
            2 => self.show_durability = !self.show_durability,
            3 => self.show_stack = !self.show_stack,
            _ => {}
        }
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// A small snapshot of the item data the engine gives us.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemData<'a> {
    pub id: &'a str,
    pub count: u32,
    /// The item's real stack limit (64 for most blocks, 16 for ender pearls,
    /// 1 for buckets). Passed through from the engine so the stack line is
    /// never wrong.
    pub max_stack: u32,
    pub damage: u32,
    /// 0 when the item has no durability (a tool with 250 max, etc.).
    pub max_damage: u32,
}

/// One appended line and its packed RGB colour.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub text: String,
    pub color: u32,
}

const GRAY: u32 = 0x808080;
const GREEN: u32 = 0x55FF55;
const YELLOW: u32 = 0xFFFF55;
const RED: u32 = 0xFF5555;

/// Remaining durability clamped to zero, the same arithmetic the gear HUD
/// uses.
pub fn remaining(max: u32, damage: u32) -> u32 {
    max.saturating_sub(damage)
}

pub fn durability_color(remaining: u32, max: u32) -> u32 {
    if max == 0 {
        return GRAY;
    }
    // Widen before multiplying: `remaining * 100` can overflow u32 for very
    // large (hand-edited) durability values, panicking in debug or wrapping
    // in release. u64 has ample headroom for any u32 operand.
    let pct = (u64::from(remaining) * 100 / u64::from(max)) as u32;
    if pct > 50 {
        GREEN
    } else if pct > 20 {
        YELLOW
    } else {
        RED
    }
}

/// Build the extra tooltip lines, in order: id, durability, stack size.
pub fn extra_lines(settings: &Settings, item: &ItemData) -> Vec<Line> {
    if !settings.enable_mod {
        return Vec::new();
    }
    let mut out = Vec::new();
    if settings.show_id {
        out.push(Line { text: format!("ID: {}", item.id), color: GRAY });
    }
    if settings.show_durability && item.max_damage > 0 {
        let left = remaining(item.max_damage, item.damage);
        out.push(Line {
            text: format!("Durability: {} / {}", left, item.max_damage),
            color: durability_color(left, item.max_damage),
        });
    }
    if settings.show_stack && item.count > 1 {
        out.push(Line {
            text: format!("Stack: {} / {}", item.count, item.max_stack),
            color: GRAY,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pickaxe() -> ItemData<'static> {
        ItemData {
            id: "minecraft:diamond_pickaxe",
            count: 1,
            max_stack: 1,
            damage: 50,
            max_damage: 1561,
        }
    }

    #[test]
    fn all_lines_for_a_durable_stack() {
        let s = Settings::default();
        let item = ItemData { count: 3, max_stack: 16, ..pickaxe() };
        let lines = extra_lines(&s, &item);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], Line { text: "ID: minecraft:diamond_pickaxe".into(), color: GRAY });
        assert_eq!(lines[1].text, "Durability: 1511 / 1561");
        // Uses the item's real stack limit, not a hardcoded 64.
        assert_eq!(lines[2].text, "Stack: 3 / 16");
    }

    #[test]
    fn stack_line_uses_real_limit_not_64() {
        let s = Settings::default();
        // A bucket stacks to 1, an ender pearl to 16, most blocks to 64.
        for (count, limit) in [(1, 1u32), (9, 16), (40, 64)] {
            let item = ItemData {
                id: "minecraft:test",
                count: count.max(2), // >1 so the line is emitted
                max_stack: limit,
                damage: 0,
                max_damage: 0,
            };
            let lines = extra_lines(&s, &item);
            let stack = lines.iter().find(|l| l.text.starts_with("Stack")).unwrap();
            assert_eq!(stack.text, format!("Stack: {} / {}", item.count, limit));
        }
    }

    #[test]
    fn durability_color_by_threshold() {
        assert_eq!(durability_color(100, 100), GREEN);
        assert_eq!(durability_color(51, 100), GREEN);
        assert_eq!(durability_color(50, 100), YELLOW);
        assert_eq!(durability_color(21, 100), YELLOW);
        assert_eq!(durability_color(20, 100), RED);
        assert_eq!(durability_color(0, 100), RED);
    }

    #[test]
    fn durability_color_does_not_overflow_on_huge_values() {
        // remaining * 100 used to overflow u32 here; u64 widening must not.
        assert_eq!(durability_color(u32::MAX, u32::MAX), GREEN);
        assert_eq!(durability_color(u32::MAX / 2, u32::MAX), YELLOW);
        assert_eq!(durability_color(0, u32::MAX), RED);
    }

    #[test]
    fn non_durable_item_has_no_durability_line() {
        let s = Settings::default();
        let item = ItemData { id: "minecraft:stone", count: 8, max_stack: 64, damage: 0, max_damage: 0 };
        let lines = extra_lines(&s, &item);
        assert_eq!(lines.len(), 2); // id + stack
        assert!(lines.iter().all(|l| !l.text.contains("Durability")));
    }

    #[test]
    fn single_item_has_no_stack_line() {
        let s = Settings::default();
        let lines = extra_lines(&s, &pickaxe());
        assert!(lines.iter().all(|l| !l.text.starts_with("Stack")));
    }

    #[test]
    fn toggles_drop_the_matching_lines() {
        let mut s = Settings::default();
        s.show_id = false;
        s.show_durability = false;
        let item = ItemData { count: 5, ..pickaxe() };
        let lines = extra_lines(&s, &item);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].text.starts_with("Stack"));
    }

    #[test]
    fn disabled_mod_adds_nothing() {
        let s = Settings { enable_mod: false, ..Settings::default() };
        assert!(extra_lines(&s, &pickaxe()).is_empty());
    }

    #[test]
    fn damage_beyond_max_clamps_to_zero() {
        assert_eq!(remaining(100, 150), 0);
        let s = Settings::default();
        let item = ItemData { damage: 2000, ..pickaxe() };
        let lines = extra_lines(&s, &item);
        assert!(lines.iter().any(|l| l.text == "Durability: 0 / 1561"));
    }

    #[test]
    fn settings_round_trip_and_clicks() {
        let mut s = Settings::default();
        s.show_id = false;
        s.show_stack = false;
        let stored = s.stored();
        let read = Settings::from_lookup(|k| {
            stored.iter().find(|(key, _)| *key == k).map(|(_, v)| v.clone())
        });
        assert_eq!(read, s);
        for i in 0..s.labels().len() {
            let before = s.clone();
            s.click(i);
            assert_ne!(before, s, "button {i} did nothing");
        }
    }
}
