//! Pure logic for the durability HUD: colours, percentages and slot layout.

pub const ID: &str = "durability";

/// Equipment slots the HUD reads, in draw order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,
    OffHand,
}

impl Slot {
    pub const ALL: [Self; 6] = [
        Self::Head,
        Self::Chest,
        Self::Legs,
        Self::Feet,
        Self::MainHand,
        Self::OffHand,
    ];
    /// Short Chinese slot label for the compact HUD row.
    pub fn name(self) -> &'static str {
        match self {
            Self::Head => "头盔",
            Self::Chest => "胸甲",
            Self::Legs => "护腿",
            Self::Feet => "靴子",
            Self::MainHand => "主手",
            Self::OffHand => "副手",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Placement {
    pub const ALL: [Self; 4] = [
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::TopLeft => "top_left",
            Self::TopRight => "top_right",
            Self::BottomLeft => "bottom_left",
            Self::BottomRight => "bottom_right",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub placement: Placement,
    pub show_percent: bool,
    pub only_damaged: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            placement: Placement::TopRight,
            show_percent: true,
            only_damaged: true,
        }
    }
}

/// One row of the panel.
#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub slot: Slot,
    pub item: String,
    pub remaining: u32,
    pub max: u32,
}

impl Bar {
    pub fn has_durability(&self) -> bool {
        self.max > 0
    }
    pub fn percent(&self) -> f32 {
        if self.max == 0 {
            return 100.0;
        }
        (self.remaining.min(self.max) as f32 / self.max as f32) * 100.0
    }
    /// 0 to 100 rounded down, with `<1%` for a nearly broken tool.
    pub fn label(&self) -> String {
        let percent = self.percent().floor() as u32;
        if self.remaining > 0 && percent == 0 {
            String::from("<1%")
        } else {
            format!("{percent}%")
        }
    }
    /// Green-yellow-red colour, 0xRRGGBB.
    pub fn color(&self) -> u32 {
        match self.percent() {
            p if p >= 50.0 => 0x55FF55,
            p if p >= 25.0 => 0xFFFF55,
            p if p >= 10.0 => 0xFFAA00,
            _ => 0xFF5555,
        }
    }
    /// The single text row for this bar. `item` is the already-localised item
    /// name (e.g. `铜头盔`), which itself says helmet/chestplate, so no extra
    /// slot prefix is needed — keeping the row short.
    pub fn row(&self, show_percent: bool) -> String {
        if show_percent {
            format!("{} {}", self.item, self.label())
        } else {
            self.item.clone()
        }
    }
}

/// `minecraft:diamond_pickaxe` reads as `Diamond Pickaxe`; mod ids keep their
/// own namespace out of the HUD, as vanilla's item names do.
pub fn item_label(id: &str) -> String {
    let path = id.split_once(':').map_or(id, |(_, path)| path);
    let words: Vec<String> = path
        .split(['_', '/', '.'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect();
    if words.is_empty() {
        return id.to_owned();
    }
    words.join(" ")
}

pub fn remaining(max_damage: u32, damage: u32) -> u32 {
    max_damage.saturating_sub(damage)
}

pub fn visible(bars: Vec<Bar>, settings: &Settings) -> Vec<Bar> {
    bars.into_iter()
        .filter(Bar::has_durability)
        .filter(|bar| !settings.only_damaged || bar.remaining < bar.max)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(slot: Slot, remaining: u32, max: u32) -> Bar {
        Bar { slot, item: String::from("minecraft:diamond_pickaxe"), remaining, max }
    }

    #[test]
    fn remaining_never_wraps() {
        assert_eq!(remaining(1561, 0), 1561);
        assert_eq!(remaining(1561, 100), 1461);
        assert_eq!(remaining(1561, 9999), 0);
    }

    #[test]
    fn percent_and_label() {
        assert_eq!(bar(Slot::Head, 50, 100).label(), "50%");
        assert_eq!(bar(Slot::Head, 999, 1000).label(), "99%");
        assert_eq!(bar(Slot::Head, 1, 1000).label(), "<1%");
        assert_eq!(bar(Slot::Head, 0, 1000).label(), "0%");
        assert!(!bar(Slot::MainHand, 1, 0).has_durability());
    }

    #[test]
    fn colours_step_at_the_thresholds() {
        assert_eq!(bar(Slot::Head, 100, 100).color(), 0x55FF55);
        assert_eq!(bar(Slot::Head, 49, 100).color(), 0xFFFF55);
        assert_eq!(bar(Slot::Head, 24, 100).color(), 0xFFAA00);
        assert_eq!(bar(Slot::Head, 9, 100).color(), 0xFF5555);
    }

    #[test]
    fn only_damaged_hides_full_gear() {
        let bars = vec![
            bar(Slot::Head, 100, 100),
            bar(Slot::Chest, 40, 100),
            bar(Slot::MainHand, 1, 0),
        ];
        let mut settings = Settings::default();
        settings.only_damaged = true;
        let shown = visible(bars.clone(), &settings);
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].slot, Slot::Chest);
        settings.only_damaged = false;
        assert_eq!(visible(bars, &settings).len(), 2);
    }

    #[test]
    fn item_labels_drop_the_namespace_and_read_as_words() {
        assert_eq!(item_label("minecraft:diamond_pickaxe"), "Diamond Pickaxe");
        assert_eq!(item_label("minecraft:iron_chestplate"), "Iron Chestplate");
        assert_eq!(item_label("mymod:oil_drum"), "Oil Drum");
        assert_eq!(item_label("stone"), "Stone");
    }

    #[test]
    fn rows_include_or_skip_percent() {
        let b = bar(Slot::Chest, 40, 100);
        assert!(b.row(true).contains("40%"));
        assert!(!b.row(false).contains("40%"));
    }
}
