//! Pure logic for the coordinates HUD: settings, formatting and layout helpers.
//! No Horizon API calls live here, so it stays trivially unit-testable.

pub const ID: &str = "coords";

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
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|p| *p == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

/// Settings mirrored from the stored string values.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub placement: Placement,
    pub show_facing: bool,
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

const DECIMALS: [u8; 3] = [0, 1, 2];

impl Settings {
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let defaults = Self::default();
        let on_off = |key: &str, fallback: bool| match lookup(key).as_deref() {
            Some("on") | Some("true") => true,
            Some("off") | Some("false") => false,
            _ => fallback,
        };
        Self {
            enable_mod: on_off("enable_mod", defaults.enable_mod),
            placement: lookup("placement")
                .as_deref()
                .and_then(Placement::parse)
                .unwrap_or(defaults.placement),
            show_facing: on_off("show_facing", defaults.show_facing),
            show_dimension: on_off("show_dimension", defaults.show_dimension),
            decimals: lookup("decimals")
                .and_then(|v| v.trim().parse::<u8>().ok())
                .filter(|d| DECIMALS.contains(d))
                .unwrap_or(defaults.decimals),
        }
    }
}

/// Where the player looks, as Minecraft's yaw reads it.
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

pub fn format_position(position: [f64; 3], decimals: u8) -> String {
    let decimals = decimals as usize;
    format!(
        "X {:.d$}  Y {:.d$}  Z {:.d$}",
        position[0], position[1], position[2], d = decimals
    )
}

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
    fn facing_cardinals() {
        assert!(facing(0.0).starts_with("S "));
        assert!(facing(90.0).starts_with("W "));
        assert!(facing(180.0).starts_with("N "));
        assert!(facing(270.0).starts_with("E "));
        assert!(facing(-90.0).starts_with("E ")); // -90° == 270°
    }

    #[test]
    fn position_precision() {
        assert_eq!(format_position([1.0, 2.0, 3.0], 0), "X 1  Y 2  Z 3");
        // Rust uses round-half-to-even for float formatting.
        assert_eq!(format_position([1.25, 2.0, 3.0], 1), "X 1.2  Y 2.0  Z 3.0");
    }

    #[test]
    fn dimension_titles() {
        assert_eq!(dimension_name("minecraft:the_nether"), "The Nether");
        assert_eq!(dimension_name("overworld"), "Overworld");
    }

    #[test]
    fn lines_respect_toggles() {
        let mut s = Settings::default();
        assert_eq!(lines(&s, [0.0; 3], 0.0, "minecraft:overworld").len(), 3);
        s.show_facing = false;
        s.show_dimension = false;
        assert_eq!(lines(&s, [0.0; 3], 0.0, "x").len(), 1);
    }
}
