//! Pure logic for Plague's Chat Timestamps, adapted to the official SDK.
//!
//! The original uses Java `SimpleDateFormat` patterns and `&`-prefixed colour
//! codes. This module renders those patterns from a dependency-free local
//! clock (`local_time`) and turns colour codes into named `TextStyle` colours.

use crate::local_time::{self, DateTime};

pub const ID: &str = "chattimestamps";
pub const NAME: &str = "Plague Chat Timestamps";

pub const DEFAULT_FORMAT: &str = "&7[HH:mm:ss] ";

/// (stored value, label) pairs for the in-game settings choice.
pub const FORMAT_PRESETS: [(&str, &str); 6] = [
    ("&7[HH:mm:ss] ", "Gray [HH:mm:ss]"),
    ("&7[HH:mm] ", "Gray [HH:mm]"),
    ("&8[&7HH:mm:ss&8] ", "Dark gray bracket [HH:mm:ss]"),
    ("&7[yyyy-MM-dd HH:mm:ss] ", "Date + time"),
    ("&7[hh:mm:ss a] ", "12-hour [hh:mm:ss a]"),
    ("", "Off (no timestamp)"),
];

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    pub format: String,
    pub enable_hover: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            format: DEFAULT_FORMAT.to_owned(),
            enable_hover: false,
        }
    }
}

impl Settings {
    pub fn from_preset(index: usize, enable_mod: bool, enable_hover: bool) -> Self {
        let (value, _) = FORMAT_PRESETS
            .get(index.min(FORMAT_PRESETS.len() - 1))
            .copied()
            .unwrap_or((DEFAULT_FORMAT, ""));
        Self {
            enable_mod,
            format: value.to_owned(),
            enable_hover,
        }
    }
}

fn pad(value: u32, width: usize) -> String {
    let text = value.to_string();
    if text.len() >= width {
        text
    } else {
        format!("{:0width$}", value, width = width)
    }
}

/// Map a legacy `&` colour code to a vanilla named colour.
/// `r` resets to `None`; k/o/m/n are stripped but have no named equivalent.
pub fn colour_name(c: char) -> Option<&'static str> {
    match c.to_ascii_lowercase() {
        '0' => Some("black"),
        '1' => Some("dark_blue"),
        '2' => Some("dark_green"),
        '3' => Some("dark_aqua"),
        '4' => Some("dark_red"),
        '5' => Some("dark_purple"),
        '6' => Some("gold"),
        '7' => Some("gray"),
        '8' => Some("dark_gray"),
        '9' => Some("blue"),
        'a' => Some("green"),
        'b' => Some("aqua"),
        'c' => Some("red"),
        'd' => Some("light_purple"),
        'e' => Some("yellow"),
        'f' => Some("white"),
        _ => None,
    }
}

fn is_format_code(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, 'a'..='f' | 'k' | 'o' | 'm' | 'n' | 'r')
}

/// Render the subset of Java `SimpleDateFormat` used by the original mod.
pub fn render_pattern(pattern: &str, now: &DateTime) -> String {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '\'' {
                i += 1;
            }
            for ch in &chars[start..i] {
                out.push(*ch);
            }
            if i < chars.len() {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i] == c && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            let run = i - start;
            let token = match c {
                'y' if run >= 4 => pad(now.year as u32, 4),
                'y' => pad(now.year.rem_euclid(100) as u32, 2),
                'M' if run == 3 => now.month_short().to_owned(),
                'M' if run >= 4 => now.month_long().to_owned(),
                'M' => pad(now.month, 2),
                'd' if run >= 2 => pad(now.day, 2),
                'd' => now.day.to_string(),
                'E' if run >= 4 => now.weekday_long().to_owned(),
                'E' => now.weekday_short().to_owned(),
                'a' => if now.is_pm() { "PM" } else { "AM" }.to_owned(),
                'H' if run >= 2 => pad(now.hour, 2),
                'H' => now.hour.to_string(),
                'h' if run >= 2 => pad(now.hour12(), 2),
                'h' => now.hour12().to_string(),
                'm' if run >= 2 => pad(now.minute, 2),
                'm' => now.minute.to_string(),
                's' if run >= 2 => pad(now.second, 2),
                's' => now.second.to_string(),
                'S' => pad(now.millis, 3),
                'z' if run >= 4 => "Local".to_owned(),
                'z' => now.offset_label(),
                _ => chars[start..i].iter().collect(),
            };
            out.push_str(&token);
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// One rendered piece of the timestamp: text plus its named colour.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub text: String,
    pub color: Option<&'static str>,
}

fn push_segment(
    segments: &mut Vec<Segment>,
    java_pattern: &str,
    color: Option<&'static str>,
    now: &DateTime,
) {
    let rendered = render_pattern(java_pattern, now);
    if !rendered.is_empty() {
        segments.push(Segment { text: rendered, color });
    }
}

/// Render a format string into coloured segments at a given local time.
pub fn format_timestamp(format: &str, now: &DateTime) -> Option<Vec<Segment>> {
    if format.is_empty() {
        return None;
    }
    let chars: Vec<char> = format.chars().collect();
    let mut segments = Vec::new();
    let mut pattern = String::new();
    let mut current: Option<&'static str> = None;
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '&' && i + 1 < chars.len() && is_format_code(chars[i + 1]) {
            if !pattern.is_empty() {
                push_segment(&mut segments, &pattern, current, now);
                pattern.clear();
            }
            let code = chars[i + 1];
            if code == 'r' {
                current = None;
            } else if let Some(name) = colour_name(code) {
                current = Some(name);
            }
            i += 2;
        } else {
            pattern.push(chars[i]);
            i += 1;
        }
    }
    if !pattern.is_empty() {
        push_segment(&mut segments, &pattern, current, now);
    }
    Some(segments)
}

pub fn timestamp_now(format: &str) -> Option<Vec<Segment>> {
    format_timestamp(format, &local_time::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed() -> DateTime {
        DateTime {
            year: 2026,
            month: 11,
            day: 16,
            weekday: 1,
            hour: 23,
            minute: 7,
            second: 5,
            millis: 0,
            offset_minutes: 0,
        }
    }

    #[test]
    fn defaults_match_original() {
        let s = Settings::default();
        assert!(s.enable_mod);
        assert_eq!(s.format, "&7[HH:mm:ss] ");
        assert!(!s.enable_hover);
    }

    #[test]
    fn settings_from_preset() {
        let s = Settings::from_preset(2, false, true);
        assert!(!s.enable_mod);
        assert!(s.enable_hover);
        assert_eq!(s.format, "&8[&7HH:mm:ss&8] ");
    }

    #[test]
    fn colour_codes_map_to_vanilla_names() {
        assert_eq!(colour_name('7'), Some("gray"));
        assert_eq!(colour_name('8'), Some("dark_gray"));
        assert_eq!(colour_name('a'), Some("green"));
        assert_eq!(colour_name('R'), None);
        assert_eq!(colour_name('z'), None);
    }

    #[test]
    fn pattern_rendering_core_letters() {
        let now = fixed();
        assert_eq!(render_pattern("HH:mm:ss", &now), "23:07:05");
        assert_eq!(render_pattern("yyyy-MM-dd", &now), "2026-11-16");
        assert_eq!(render_pattern("hh:mm:ss a", &now), "11:07:05 PM");
    }

    #[test]
    fn pattern_rendering_names_and_millis() {
        let now = fixed();
        assert_eq!(render_pattern("MMM dd, yyyy", &now), "Nov 16, 2026");
        assert_eq!(render_pattern("MMM d, yyyy", &now), "Nov 16, 2026");
        assert_eq!(render_pattern("EEE HH:mm:ss.SSS", &now), "Mon 23:07:05.000");
    }

    #[test]
    fn pattern_rendering_quoted_literal() {
        assert_eq!(render_pattern("'at' HH:mm", &fixed()), "at 23:07");
    }

    #[test]
    fn formats_default_like_the_original() {
        let segs = format_timestamp("&7[HH:mm:ss] ", &fixed()).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text, "[23:07:05] ");
        assert_eq!(segs[0].color, Some("gray"));
    }

    #[test]
    fn multiple_colours_become_multiple_segments() {
        let segs = format_timestamp("&8[&7HH:mm&8] ", &fixed()).unwrap();
        let colors: Vec<_> = segs.iter().map(|s| s.color).collect();
        assert_eq!(colors, [Some("dark_gray"), Some("gray"), Some("dark_gray")]);
        let joined: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "[23:07] ");
    }

    #[test]
    fn empty_format_is_off() {
        assert_eq!(format_timestamp("", &fixed()), None);
    }
}
