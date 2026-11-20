//! Pure logic for the FPS HUD: settings, formatting and placement helpers.

pub const ID: &str = "fpshud";

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
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub placement: Placement,
    pub detailed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            placement: Placement::BottomRight,
            detailed: true,
        }
    }
}

/// Colour by FPS: green when smooth, yellow when okay, red when low (RGB).
pub fn fps_color(fps: u32) -> u32 {
    if fps >= 60 {
        0x55FF55
    } else if fps >= 30 {
        0xFFFF55
    } else {
        0xFF5555
    }
}

/// The lines to draw. `frame_ms` is the most recent frame's wall interval.
pub fn lines(settings: &Settings, fps: u32, frame_ms: Option<u64>) -> Vec<String> {
    let mut out = vec![format!("{fps} FPS")];
    if settings.detailed {
        match frame_ms {
            Some(ms) => out.push(format!("Frame {ms} ms")),
            None => out.push(String::from("Frame -- ms")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_line_is_fps_only() {
        let s = Settings { detailed: false, ..Settings::default() };
        assert_eq!(lines(&s, 120, Some(8)), vec!["120 FPS"]);
    }

    #[test]
    fn detailed_line_shows_frame_time() {
        let s = Settings::default();
        assert_eq!(lines(&s, 120, Some(8)), vec!["120 FPS", "Frame 8 ms"]);
        assert_eq!(lines(&s, 60, None), vec!["60 FPS", "Frame -- ms"]);
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
    fn placement_parses_all() {
        for p in Placement::ALL {
            assert_eq!(Placement::parse(p.key()), Some(p));
        }
        assert_eq!(Placement::parse("nope"), None);
    }
}
