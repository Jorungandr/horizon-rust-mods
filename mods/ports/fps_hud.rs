//! An FPS HUD: frames per second (and optionally the slowest frame and the
//! 1% low) in a screen corner.
//!
//! The engine already measures the readings (`native_frame_rate`), so this
//! port only formats and lays them out. The formatting is pure and tested
//! without a game. The engine adapter is appended at injection time (see
//! `tools/inject_all_ports.py`).

pub const ID: &str = "fpshud";
pub const NAME: &str = "FPS HUD";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Shows frames per second, with optional slowest frame and 1% low.";

const KEY_ENABLE: &str = "enable_mod";
const KEY_PLACEMENT: &str = "placement";
const KEY_DETAILED: &str = "detailed";

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

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub placement: Placement,
    /// Show the two extra readings under the main FPS number.
    pub detailed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            // Bottom right: coords uses top left and durability top right.
            placement: Placement::BottomRight,
            detailed: true,
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
            detailed: on_off(KEY_DETAILED, defaults.detailed),
        }
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            (KEY_ENABLE, on_off(self.enable_mod).to_owned()),
            (KEY_PLACEMENT, self.placement.key().to_owned()),
            (KEY_DETAILED, on_off(self.detailed).to_owned()),
        ]
    }

    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("FPS HUD: {}", on_off(self.enable_mod).to_uppercase()),
            format!("Position: {}", self.placement.name()),
            format!("Detailed: {}", on_off(self.detailed).to_uppercase()),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enable_mod = !self.enable_mod,
            1 => self.placement = self.placement.next(),
            2 => self.detailed = !self.detailed,
            _ => {}
        }
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// Colour by FPS: green when smooth, yellow when okay, red when low.
pub fn fps_color(fps: u32) -> u32 {
    if fps >= 60 {
        0x55FF55
    } else if fps >= 30 {
        0xFFFF55
    } else {
        0xFF5555
    }
}

/// The lines to draw. `readings` is `(fps, slowest, 1% low)` from the engine.
pub fn lines(settings: &Settings, readings: (u32, u32, u32)) -> Vec<String> {
    let (fps, slowest, low) = readings;
    let mut out = vec![format!("{fps} FPS")];
    if settings.detailed {
        out.push(format!("Min {slowest}  ·  1% low {low}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_line_is_fps_only() {
        let s = Settings { detailed: false, ..Settings::default() };
        assert_eq!(lines(&s, (120, 40, 55)), vec!["120 FPS"]);
    }

    #[test]
    fn detailed_line_shows_both_readings() {
        let s = Settings::default();
        assert_eq!(
            lines(&s, (120, 40, 55)),
            vec!["120 FPS", "Min 40  ·  1% low 55"]
        );
    }

    #[test]
    fn color_thresholds() {
        assert_eq!(fps_color(60), 0x55FF55);
        assert_eq!(fps_color(144), 0x55FF55);
        assert_eq!(fps_color(30), 0xFFFF55);
        assert_eq!(fps_color(59), 0xFFFF55);
        assert_eq!(fps_color(29), 0xFF5555);
        assert_eq!(fps_color(0), 0xFF5555);
    }

    #[test]
    fn settings_round_trip() {
        let mut s = Settings::default();
        s.placement = Placement::BottomLeft;
        s.detailed = false;
        let stored = s.stored();
        let read = Settings::from_lookup(|k| {
            stored.iter().find(|(key, _)| *key == k).map(|(_, v)| v.clone())
        });
        assert_eq!(read, s);
    }

    #[test]
    fn clicks_cycle_every_button() {
        let mut s = Settings::default();
        let n = s.labels().len();
        for i in 0..n {
            let before = s.clone();
            s.click(i);
            assert_ne!(before, s, "button {i} did nothing");
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
        assert_eq!(Placement::parse("nope"), None);
    }
}
