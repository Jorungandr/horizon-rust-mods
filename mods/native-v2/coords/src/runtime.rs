//! A coordinates HUD: position, facing and dimension in a screen corner.
//!
//! The formatting below is pure: it takes numbers and returns the lines the
//! HUD draws, so it can be tested without a game. The engine adapter is
//! appended at injection time (see `tools/inject_all_ports.py`).

pub const ID: &str = "coords";
pub const NAME: &str = "Coordinates HUD";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Shows your position, facing and dimension in a screen corner.";

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
const KEY_FACING: &str = "show_facing";
const KEY_DIMENSION: &str = "show_dimension";
const KEY_DECIMALS: &str = "decimals";

/// How many digits after the point the coordinates keep, and the choices the
/// settings button cycles.
const DECIMALS: [u8; 3] = [0, 1, 2];

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub placement: Placement,
    /// The way you face, as a compass point and the raw yaw.
    pub show_facing: bool,
    /// The dimension's display name.
    pub show_dimension: bool,
    pub decimals: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            placement: Placement::TopLeft,
            show_facing: true,
            show_dimension: true,
            decimals: 1,
        }
    }
}

impl Settings {
    /// Read every key, falling back to the default for a value that is
    /// missing or unreadable.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let defaults = Self::default();
        let on_off = |key: &str, fallback: bool| match lookup(key).as_deref() {
            Some("on") => true,
            Some("off") => false,
            Some("true") => true,
            Some("false") => false,
            _ => fallback,
        };
        Self {
            enable_mod: on_off(KEY_ENABLE, defaults.enable_mod),
            placement: lookup(KEY_PLACEMENT)
                .as_deref()
                .and_then(Placement::parse)
                .unwrap_or(defaults.placement),
            show_facing: on_off(KEY_FACING, defaults.show_facing),
            show_dimension: on_off(KEY_DIMENSION, defaults.show_dimension),
            decimals: lookup(KEY_DECIMALS)
                .and_then(|v| v.trim().parse::<u8>().ok())
                .filter(|d| DECIMALS.contains(d))
                .unwrap_or(defaults.decimals),
        }
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            (KEY_ENABLE, on_off(self.enable_mod).to_owned()),
            (KEY_PLACEMENT, self.placement.key().to_owned()),
            (KEY_FACING, on_off(self.show_facing).to_owned()),
            (KEY_DIMENSION, on_off(self.show_dimension).to_owned()),
            (KEY_DECIMALS, self.decimals.to_string()),
        ]
    }

    /// The buttons of the settings page, in click order.
    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("Coordinates: {}", on_off(self.enable_mod).to_uppercase()),
            format!("Position: {}", self.placement.name()),
            format!("Facing: {}", on_off(self.show_facing).to_uppercase()),
            format!("Dimension: {}", on_off(self.show_dimension).to_uppercase()),
            format!("Decimals: {}", self.decimals),
        ]
    }

    /// Advance button `button`, ignoring one past the end.
    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enable_mod = !self.enable_mod,
            1 => self.placement = self.placement.next(),
            2 => self.show_facing = !self.show_facing,
            3 => self.show_dimension = !self.show_dimension,
            4 => {
                let i = DECIMALS
                    .iter()
                    .position(|d| *d == self.decimals)
                    .unwrap_or(0);
                self.decimals = DECIMALS[(i + 1) % DECIMALS.len()];
            }
            _ => {}
        }
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// Where the player looks, as Minecraft's yaw reads it: 0 faces +Z (south)
/// and the angle rises clockwise seen from above, so 90 faces -X (west).
pub fn facing(yaw: f32) -> String {
    let yaw = ((yaw % 360.0) + 360.0) % 360.0;
    let (point, axis) = match ((yaw + 45.0) / 90.0) as u32 % 4 {
        0 => ("S", "+Z"),
        1 => ("W", "-X"),
        2 => ("N", "-Z"),
        _ => ("E", "+X"),
    };
    format!("{point} ({axis}, {yaw:.0}\u{b0})")
}

/// The three coordinates, rounded to `decimals`.
pub fn format_position(position: [f64; 3], decimals: u8) -> String {
    let decimals = decimals as usize;
    format!(
        "X {:.d$}  Y {:.d$}  Z {:.d$}",
        position[0],
        position[1],
        position[2],
        d = decimals
    )
}

/// `minecraft:the_nether` reads `The Nether`; a name with no namespace keeps
/// its own words.
pub fn dimension_name(id: &str) -> String {
    let path = id.rsplit_once(':').map_or(id, |(_, path)| path);
    let mut out = String::with_capacity(path.len());
    let mut upper = true;
    for c in path.chars() {
        if c == '_' {
            out.push(' ');
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// The lines to draw, top to bottom, with the corners' box around them.
pub fn lines(settings: &Settings, position: [f64; 3], yaw: f32, dimension: &str) -> Vec<String> {
    let mut out = vec![format_position(position, settings.decimals)];
    if settings.show_facing {
        out.push(facing(yaw));
    }
    if settings.show_dimension {
        out.push(dimension_name(dimension));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_follows_minecrafts_yaw() {
        assert!(facing(0.0).starts_with("S (+Z"));
        assert!(facing(90.0).starts_with("W (-X"));
        assert!(facing(180.0).starts_with("N (-Z"));
        assert!(facing(270.0).starts_with("E (+X"));
        // 44 degrees is still south, 46 already west.
        assert!(facing(44.0).starts_with("S "));
        assert!(facing(46.0).starts_with("W "));
        // The raw angle is in the label, rounded.
        assert!(facing(123.4).contains("123"), "{}", facing(123.4));
    }

    #[test]
    fn negative_and_large_yaw_wrap() {
        assert!(facing(-90.0).starts_with("E "), "{}", facing(-90.0));
        assert!(facing(450.0).starts_with("W "), "{}", facing(450.0));
        assert!(facing(-1.0).starts_with("S "), "{}", facing(-1.0));
    }

    #[test]
    fn position_rounds_to_the_chosen_decimals() {
        assert_eq!(
            format_position([1.23456, 64.0, -7.89123], 0),
            "X 1  Y 64  Z -8"
        );
        assert_eq!(
            format_position([1.23456, 64.0, -7.89123], 1),
            "X 1.2  Y 64.0  Z -7.9"
        );
        assert_eq!(
            format_position([1.23456, 64.0, -7.89123], 2),
            "X 1.23  Y 64.00  Z -7.89"
        );
    }

    #[test]
    fn dimension_names_are_title_cased() {
        assert_eq!(dimension_name("minecraft:overworld"), "Overworld");
        assert_eq!(dimension_name("minecraft:the_nether"), "The Nether");
        assert_eq!(dimension_name("minecraft:the_end"), "The End");
        assert_eq!(dimension_name("mymod:deep_dark_cavern"), "Deep Dark Cavern");
        assert_eq!(dimension_name("overworld"), "Overworld");
    }

    #[test]
    fn lines_respect_the_toggles() {
        let mut settings = Settings::default();
        let all = lines(&settings, [0.0, 0.0, 0.0], 0.0, "minecraft:overworld");
        assert_eq!(all.len(), 3);
        settings.show_facing = false;
        settings.show_dimension = false;
        assert_eq!(lines(&settings, [0.0, 0.0, 0.0], 0.0, "x:y").len(), 1);
    }

    #[test]
    fn settings_round_trip_through_storage() {
        let mut settings = Settings::default();
        settings.placement = Placement::BottomRight;
        settings.decimals = 2;
        settings.show_facing = false;
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
        let labels = settings.labels();
        for (i, label) in labels.iter().enumerate() {
            let before = settings.clone();
            settings.click(i);
            assert_ne!(before, settings, "button {i} ({label}) did nothing");
        }
    }

    #[test]
    fn placement_cycles_and_parses() {
        let mut p = Placement::TopLeft;
        for _ in 0..Placement::ALL.len() {
            assert_eq!(Placement::parse(p.key()), Some(p));
            p = p.next();
        }
        assert_eq!(p, Placement::TopLeft);
        assert_eq!(Placement::parse("nowhere"), None);
    }
}
