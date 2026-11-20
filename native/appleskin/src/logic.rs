//! Pure layout and food maths, kept apart from the game so it can be tested on its own.

use horizon_core::nbt::JavaString;
use mod_api::Tag;

/// Vanilla draws ten hunger icons, right to left from the screen's centre.
pub const ICON: f32 = 9.0;
pub const STEP: f32 = 8.0;
pub const BAR_RIGHT: f32 = 91.0;
pub const BAR_BOTTOM: f32 = 39.0;

/// Each icon holds two food points, so a point of saturation covers half an icon:
/// the whole bar is twenty halves across ten icons.
pub fn halves_total(saturation: f32) -> u32 {
    if !saturation.is_finite() {
        return 0;
    }
    saturation.clamp(0.0, 20.0).round() as u32
}

/// Halves drawn on icon `index`, 0 to 2.
pub fn halves_on(saturation: f32, index: usize) -> u32 {
    let start = index as u32 * 2;
    halves_total(saturation).saturating_sub(start).min(2)
}

/// Left edge and top edge of hunger icon `index`, zero being the rightmost.
pub fn icon_origin(width: f32, height: f32, index: usize) -> [f32; 2] {
    [
        width / 2.0 + BAR_RIGHT - ICON - STEP * index as f32,
        height - BAR_BOTTOM,
    ]
}

/// The saturation highlight drawn over an icon: `[x, y, w, h]` in GUI pixels.
/// Halves sit side by side, the left one first, one pixel above the food row.
pub fn highlight(width: f32, height: f32, index: usize, halves: u32) -> Option<[f32; 4]> {
    if halves == 0 {
        return None;
    }
    let [x, y] = icon_origin(width, height, index);
    let half_width = ICON / 2.0;
    Some([x, y - 1.0, half_width * halves as f32, 2.0])
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoodValue {
    /// Food points the item restores, `nutrition` in `minecraft:food`.
    pub nutrition: i32,
    /// The multiplier `saturation` in the same component.
    pub saturation_modifier: f32,
}

impl FoodValue {
    /// What eating it adds to the saturation level: `nutrition * modifier * 2`.
    pub fn saturation_gain(&self) -> f32 {
        self.nutrition as f32 * self.saturation_modifier * 2.0
    }
}

/// `minecraft:food` as the item report gives it: a compound of `nutrition` and `saturation`.
pub fn parse_food(tag: &Tag) -> Option<FoodValue> {
    let Tag::Compound(compound) = tag else {
        return None;
    };
    let nutrition = match compound.get(&JavaString::from("nutrition")) {
        Some(Tag::Int(value)) => *value,
        Some(Tag::Byte(value)) => i32::from(*value),
        Some(Tag::Short(value)) => i32::from(*value),
        _ => return None,
    };
    let saturation_modifier = match compound.get(&JavaString::from("saturation")) {
        Some(Tag::Float(bits)) => f32::from_bits(*bits),
        Some(Tag::Double(bits)) => f64::from_bits(*bits) as f32,
        _ => 0.0,
    };
    Some(FoodValue {
        nutrition,
        saturation_modifier,
    })
}

/// One tooltip line per fact, already in the order they should appear.
pub fn tooltip_lines(value: FoodValue) -> Vec<(String, u32)> {
    let mut lines = Vec::new();
    if value.nutrition != 0 {
        lines.push((format!("Hunger +{}", value.nutrition), 0x55FF55));
    }
    let gain = value.saturation_gain();
    if gain > 0.0 {
        lines.push((format!("Saturation +{gain:.1}"), 0xFFAA00));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_follow_the_food_scale() {
        assert_eq!(halves_total(0.0), 0);
        assert_eq!(halves_total(0.4), 0);
        assert_eq!(halves_total(0.6), 1);
        assert_eq!(halves_total(5.0), 5);
        assert_eq!(halves_total(20.0), 20);
        assert_eq!(halves_total(99.0), 20);
        assert_eq!(halves_total(f32::NAN), 0);
    }

    #[test]
    fn halves_stay_inside_their_icon() {
        assert_eq!(halves_on(1.6, 0), 2);
        assert_eq!(halves_on(1.6, 1), 0);
        assert_eq!(halves_on(3.0, 0), 2);
        assert_eq!(halves_on(3.0, 1), 1);
        assert_eq!(halves_on(20.0, 9), 2);
    }

    #[test]
    fn the_rightmost_icon_sits_against_the_bar() {
        let [x, y] = icon_origin(400.0, 300.0, 0);
        assert_eq!(x, 200.0 + 91.0 - 9.0);
        assert_eq!(y, 261.0);
        let [x1, _] = icon_origin(400.0, 300.0, 1);
        assert_eq!(x - x1, 8.0);
    }

    #[test]
    fn no_highlight_for_an_empty_half() {
        assert_eq!(highlight(400.0, 300.0, 0, 0), None);
        let [x, y, w, h] = highlight(400.0, 300.0, 0, 1).unwrap();
        assert_eq!([x, y, w, h], [282.0, 260.0, 4.5, 2.0]);
    }

    #[test]
    fn a_steak_restores_what_the_formula_says() {
        let value = FoodValue {
            nutrition: 8,
            saturation_modifier: 0.8,
        };
        assert!((value.saturation_gain() - 12.8).abs() < 0.001);
        let lines = tooltip_lines(value);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "Hunger +8");
        assert_eq!(lines[1].0, "Saturation +12.8");
    }
}
