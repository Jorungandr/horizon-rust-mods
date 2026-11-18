//! A durability HUD: how much wear each piece of gear has left.
//!
//! The arithmetic and colour rules are pure, so they are tested without a
//! game. The engine adapter is appended at injection time (see
//! `tools/inject_all_ports.py`).

pub const ID: &str = "durability";
pub const NAME: &str = "Durability HUD";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Shows how much durability your armour and held items have left.";

/// The equipment slots the HUD reads, in draw order: armour top down, then
/// the hands.
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

    pub fn name(self) -> &'static str {
        match self {
            Self::Head => "Helmet",
            Self::Chest => "Chestplate",
            Self::Legs => "Leggings",
            Self::Feet => "Boots",
            Self::MainHand => "Main hand",
            Self::OffHand => "Off hand",
        }
    }
}

/// Which corner the panel sits in.
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

    pub fn name(self) -> &'static str {
        match self {
            Self::TopLeft => "Top left",
            Self::TopRight => "Top right",
            Self::BottomLeft => "Bottom left",
            Self::BottomRight => "Bottom right",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::TopLeft => "top_left",
            Self::TopRight => "top_right",
            Self::BottomLeft => "bottom_left",
            Self::BottomRight => "bottom_right",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == value)
    }

    fn next(self) -> Self {
        let i = Self::ALL.iter().position(|p| *p == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

const KEY_ENABLE: &str = "enable_mod";
const KEY_PLACEMENT: &str = "placement";
const KEY_ICONS: &str = "show_icons";
const KEY_PERCENT: &str = "show_percent";
const KEY_ARROWS: &str = "only_damaged";

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub placement: Placement,
    /// Draw the item's own icon beside the number.
    pub show_icons: bool,
    /// Draw the percentage; with this off the bar itself carries the level.
    pub show_percent: bool,
    /// Hide gear at full durability, as the original mod does.
    pub only_damaged: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            placement: Placement::TopRight,
            show_icons: true,
            show_percent: true,
            only_damaged: true,
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
            placement: lookup(KEY_PLACEMENT)
                .as_deref()
                .and_then(Placement::parse)
                .unwrap_or(defaults.placement),
            show_icons: on_off(KEY_ICONS, defaults.show_icons),
            show_percent: on_off(KEY_PERCENT, defaults.show_percent),
            only_damaged: on_off(KEY_ARROWS, defaults.only_damaged),
        }
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            (KEY_ENABLE, on_off(self.enable_mod).to_owned()),
            (KEY_PLACEMENT, self.placement.key().to_owned()),
            (KEY_ICONS, on_off(self.show_icons).to_owned()),
            (KEY_PERCENT, on_off(self.show_percent).to_owned()),
            (KEY_ARROWS, on_off(self.only_damaged).to_owned()),
        ]
    }

    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("Durability: {}", on_off(self.enable_mod).to_uppercase()),
            format!("Position: {}", self.placement.name()),
            format!("Icons: {}", on_off(self.show_icons).to_uppercase()),
            format!("Percent: {}", on_off(self.show_percent).to_uppercase()),
            format!("Damaged only: {}", on_off(self.only_damaged).to_uppercase()),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enable_mod = !self.enable_mod,
            1 => self.placement = self.placement.next(),
            2 => self.show_icons = !self.show_icons,
            3 => self.show_percent = !self.show_percent,
            4 => self.only_damaged = !self.only_damaged,
            _ => {}
        }
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
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
    /// True for an item that wears out; a block or a piece of food never
    /// shows a bar.
    pub fn has_durability(&self) -> bool {
        self.max > 0
    }

    /// 0 to 100.
    pub fn percent(&self) -> f32 {
        if self.max == 0 {
            return 100.0;
        }
        (self.remaining.min(self.max) as f32 / self.max as f32) * 100.0
    }

    /// `87%`, rounding down so a nearly broken tool never reads `0%` while it
    /// still has a point left.
    pub fn label(&self) -> String {
        let percent = self.percent().floor() as u32;
        if self.remaining > 0 && percent == 0 {
            "<1%".to_owned()
        } else {
            format!("{percent}%")
        }
    }

    /// A green-yellow-red colour, 0xRRGGBB, for the text.
    pub fn color(&self) -> u32 {
        match self.percent() {
            p if p >= 50.0 => 0x55FF55,
            p if p >= 25.0 => 0xFFFF55,
            p if p >= 10.0 => 0xFFAA00,
            _ => 0xFF5555,
        }
    }
}

/// What is left of `max_damage` after `damage`, never below zero.
pub fn remaining(max_damage: u32, damage: u32) -> u32 {
    max_damage.saturating_sub(damage)
}

/// The rows to draw: gear that wears out, minus full-durability pieces when
/// `only_damaged` is set.
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
        Bar {
            slot,
            item: "minecraft:diamond_pickaxe".to_owned(),
            remaining,
            max,
        }
    }

    #[test]
    fn remaining_never_wraps() {
        assert_eq!(remaining(1561, 0), 1561);
        assert_eq!(remaining(1561, 100), 1461);
        assert_eq!(remaining(1561, 1561), 0);
        // A damage above the maximum (a hand-edited stack) stops at zero.
        assert_eq!(remaining(1561, 9999), 0);
    }

    #[test]
    fn percent_and_label() {
        let half = bar(Slot::Head, 50, 100);
        assert_eq!(half.percent(), 50.0);
        assert_eq!(half.label(), "50%");
        // Rounds down: 99.9% reads 99%, and a nearly spent tool reads <1%.
        assert_eq!(bar(Slot::Head, 999, 1000).label(), "99%");
        assert_eq!(bar(Slot::Head, 1, 1000).label(), "<1%");
        assert_eq!(bar(Slot::Head, 0, 1000).label(), "0%");
        // A stack with no durability reports full rather than dividing by zero.
        assert_eq!(bar(Slot::MainHand, 1, 0).percent(), 100.0);
        assert!(!bar(Slot::MainHand, 1, 0).has_durability());
    }

    #[test]
    fn colours_step_at_the_thresholds() {
        assert_eq!(bar(Slot::Head, 100, 100).color(), 0x55FF55);
        assert_eq!(bar(Slot::Head, 50, 100).color(), 0x55FF55);
        assert_eq!(bar(Slot::Head, 49, 100).color(), 0xFFFF55);
        assert_eq!(bar(Slot::Head, 25, 100).color(), 0xFFFF55);
        assert_eq!(bar(Slot::Head, 24, 100).color(), 0xFFAA00);
        assert_eq!(bar(Slot::Head, 10, 100).color(), 0xFFAA00);
        assert_eq!(bar(Slot::Head, 9, 100).color(), 0xFF5555);
        assert_eq!(bar(Slot::Head, 0, 100).color(), 0xFF5555);
    }

    #[test]
    fn only_damaged_hides_full_gear_and_items_without_durability() {
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
        let shown = visible(bars, &settings);
        assert_eq!(shown.len(), 2);
        assert_eq!(shown[0].slot, Slot::Head);
        assert_eq!(shown[1].slot, Slot::Chest);
    }

    #[test]
    fn settings_round_trip_through_storage() {
        let mut settings = Settings::default();
        settings.placement = Placement::BottomLeft;
        settings.show_icons = false;
        settings.only_damaged = false;
        let stored = settings.stored();
        let read = Settings::from_lookup(|key| {
            stored
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        });
        assert_eq!(read, settings);
    }

    #[test]
    fn clicks_cycle_every_button() {
        let mut settings = Settings::default();
        for i in 0..settings.labels().len() {
            let before = settings.clone();
            settings.click(i);
            assert_ne!(before, settings, "button {i} did nothing");
        }
    }
}
