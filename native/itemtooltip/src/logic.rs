//! Pure logic for Item Tooltip+: which extra lines to append to a tooltip.

pub const ID: &str = "itemtooltip";

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub show_id: bool,
    pub show_durability: bool,
    pub show_stack: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { enable_mod: true, show_id: true, show_durability: true, show_stack: true }
    }
}

/// A small snapshot of the item data the engine gives us.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemData<'a> {
    pub id: &'a str,
    pub count: u32,
    pub max_stack: u32,
    pub damage: u32,
    pub max_damage: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub text: String,
    pub color: u32,
}

const GRAY: u32 = 0x808080;
const GREEN: u32 = 0x55FF55;
const YELLOW: u32 = 0xFFFF55;
const RED: u32 = 0xFF5555;

pub fn remaining(max: u32, damage: u32) -> u32 {
    max.saturating_sub(damage)
}

pub fn durability_color(remaining: u32, max: u32) -> u32 {
    if max == 0 {
        return GRAY;
    }
    // Widen before multiplying: remaining * 100 can overflow u32.
    let pct = (u64::from(remaining) * 100 / u64::from(max)) as u32;
    if pct > 50 {
        GREEN
    } else if pct > 20 {
        YELLOW
    } else {
        RED
    }
}

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
        assert_eq!(lines[2].text, "Stack: 3 / 16");
    }

    #[test]
    fn stack_line_uses_real_limit_not_64() {
        let s = Settings::default();
        for (count, limit) in [(1, 1u32), (9, 16), (40, 64)] {
            let item = ItemData {
                id: "minecraft:test",
                count: count.max(2),
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
        assert_eq!(durability_color(u32::MAX, u32::MAX), GREEN);
        assert_eq!(durability_color(u32::MAX / 2, u32::MAX), YELLOW);
        assert_eq!(durability_color(0, u32::MAX), RED);
    }

    #[test]
    fn non_durable_item_has_no_durability_line() {
        let s = Settings::default();
        let item = ItemData { id: "minecraft:stone", count: 8, max_stack: 64, damage: 0, max_damage: 0 };
        let lines = extra_lines(&s, &item);
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| !l.text.contains("Durability")));
    }

    #[test]
    fn single_item_has_no_stack_line() {
        let s = Settings::default();
        assert!(extra_lines(&s, &pickaxe()).iter().all(|l| !l.text.starts_with("Stack")));
    }

    #[test]
    fn toggles_drop_the_matching_lines() {
        let mut s = Settings::default();
        s.show_id = false;
        s.show_durability = false;
        let lines = extra_lines(&s, &ItemData { count: 5, ..pickaxe() });
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
        let lines = extra_lines(&s, &ItemData { damage: 2000, ..pickaxe() });
        assert!(lines.iter().any(|l| l.text == "Durability: 0 / 1561"));
    }
}
