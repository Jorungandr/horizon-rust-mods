// Pure logic for the Plague's Chat Timestamps port.
//
// Ported from PlagueTR/ChatTimestamps (MIT), common/.../Main.java +
// ModConfig.java. This file is both the mod's Cargo library (its tests) and
// the module the bundle adds to the game as `crate::game::chattimestamps`.
//
// The original uses Java `SimpleDateFormat` with a user-editable pattern and
// `&`-prefixed colour codes (e.g. "&7[HH:mm:ss] "). Rust's `chrono` uses a
// different pattern syntax, so `compile_pattern` translates the supported
// SimpleDateFormat letters into chrono strftime tokens. Colour codes are
// handled separately: they are emitted as plain `§` control sequences in the
// resulting string (Horizon preserves legacy colour codes in literal text).

use chrono::Local;

pub const ID: &str = "chattimestamps";
pub const NAME: &str = "Plague Chat Timestamps";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Customizable chat timestamps: time/date formatting, colours and hover mode.";

/// The original mod's default format string.
pub const DEFAULT_FORMAT: &str = "&7[HH:mm:ss] ";

/// A selection of ready-made formats for the settings page (button cycles them).
/// Each entry is (stored value, human label). The stored value uses the original
/// mod's `&` colour-code syntax and SimpleDateFormat pattern letters.
pub const FORMAT_PRESETS: [(&str, &str); 6] = [
    ("&7[HH:mm:ss] ", "Gray [HH:mm:ss]"),
    ("&7[HH:mm] ", "Gray [HH:mm]"),
    ("&8[&7HH:mm:ss&8] ", "Dark gray bracket [HH:mm:ss]"),
    ("&7[yyyy-MM-dd HH:mm:ss] ", "Date + time"),
    ("&7[hh:mm:ss a] ", "12-hour [hh:mm:ss a]"),
    ("", "Off (no timestamp)"),
];

/// The three settings, mirroring the original ModConfig.
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

/// Free wrapper used by the injected helpers, which see this module as
/// `crate::game::chattimestamps` and call `m::settings(get)`.
pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    Settings::read(get)
}

impl Settings {
    /// Reads settings from stored strings (one file per key). Missing keys keep
    /// the original mod's defaults. Values are trimmed of surrounding whitespace
    /// except `format`, whose trailing space is significant.
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let enable_mod = get("enable_mod")
            .map(|v| v.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(true);
        let enable_hover = get("enable_hover")
            .map(|v| v.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let format = get("format").unwrap_or_else(|| DEFAULT_FORMAT.to_owned());
        Self {
            enable_mod,
            format,
            enable_hover,
        }
    }

    /// Human-readable labels for the three settings buttons, in click order.
    pub fn labels(&self) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let fmt_label = FORMAT_PRESETS
            .iter()
            .find(|(v, _)| *v == self.format)
            .map(|(_, l)| *l)
            .unwrap_or("Custom");
        vec![
            format!("Timestamps: {}", on(self.enable_mod)),
            format!("Format: {fmt_label}"),
            format!("On hover only: {}", on(self.enable_hover)),
        ]
    }

    /// Advances the setting addressed by button index (0..3), matching the
    /// original's three options. Button 1 cycles through FORMAT_PRESETS.
    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enable_mod = !self.enable_mod,
            1 => {
                let current = FORMAT_PRESETS
                    .iter()
                    .position(|(v, _)| *v == self.format)
                    .unwrap_or(0);
                let next = (current + 1) % FORMAT_PRESETS.len();
                self.format = FORMAT_PRESETS[next].0.to_owned();
            }
            2 => self.enable_hover = !self.enable_hover,
            _ => {}
        }
    }

    /// The values to persist (one file per key).
    pub fn stored(&self) -> [(&'static str, String); 3] {
        [
            ("enable_mod", self.enable_mod.to_string()),
            ("format", self.format.clone()),
            ("enable_hover", self.enable_hover.to_string()),
        ]
    }
}

/// Translate a SimpleDateFormat pattern into a chrono strftime pattern.
///
/// Supported (the letters the original mod advertises):
///   y/yyyy year, M/MM month number, MMM short month, MMMM long month,
///   d/dd day, E/EEE weekday name, a AM/PM,
///   H/HH 24h, h/hh 12h, m/mm minute, s/ss second, S/SS/SSS millisecond,
///   z/zzzz time zone name.
/// Text in single quotes is emitted literally (Java quoting). A `&X` colour
/// code is not a pattern letter; callers strip it in `split_colours` first,
/// but this function also passes unknown characters through unchanged.
pub fn to_chrono_pattern(pattern: &str) -> String {
    let mut out = String::new();
    let bytes = pattern.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        // Java single-quoted literal run.
        if c == '\'' {
            i += 1;
            let mut quoted = String::new();
            while i < bytes.len() && bytes[i] as char != '\'' {
                quoted.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() {
                i += 1; // closing quote
            }
            out.push_str(&escape_literal(&quoted));
            continue;
        }
        // Count consecutive identical letters (Java/chrono both use runs).
        let start = i;
        while i < bytes.len() && bytes[i] == bytes[start] && (bytes[i] as char).is_ascii_alphabetic() {
            i += 1;
        }
        let run = i - start;
        if run == 0 {
            // Non-letter: pass through, escaping chrono's '%'.
            if c == '%' {
                out.push_str("%%");
            } else {
                out.push(c);
            }
            i += 1;
            continue;
        }
        let token = match c {
            'y' if run >= 4 => "%Y",
            'y' => "%y",
            'M' if run == 3 => "%b",
            'M' if run >= 4 => "%B",
            'M' if run == 2 => "%m",
            'M' => "%m",
            'd' if run == 2 => "%d",
            'd' => "%-d",
            'E' if run >= 4 => "%A",
            'E' => "%a",
            'a' => "%p",
            'H' if run == 2 => "%H",
            'H' => "%-H",
            'h' if run == 2 => "%I",
            'h' => "%-I",
            'm' if run == 2 => "%M",
            'm' => "%-M",
            's' if run == 2 => "%S",
            's' => "%-S",
            'S' if run >= 3 => "%.3f",
            'S' if run == 2 => "%3f",
            'S' => "%3f",
            'z' if run >= 4 => "%Z",
            'z' => "%:z",
            _ => {
                // Unknown letter: emit it literally (escaped).
                out.push_str(&escape_literal(&pattern[start..i]));
                continue;
            }
        };
        out.push_str(token);
    }
    out
}

/// Escape characters that chrono treats specially in a literal run.
fn escape_literal(s: &str) -> String {
    s.replace('%', "%%")
}

/// One rendered piece of the timestamp: text plus the named colour that should
/// be applied to it. Horizon's `TextStyle.color` takes a named colour (e.g.
/// `"gray"`) or `#RRGGBB`; legacy `§` codes are NOT interpreted in literal text,
/// so we must express colour through the style instead.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub text: String,
    pub color: Option<&'static str>,
}

/// Map a legacy `&` code to a vanilla named colour. `r` resets to `None`;
/// formatting codes k/o/m/n are stripped but have no named-colour equivalent in
/// this port (the original only meaningfully used colours).
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

/// Render a format string into coloured segments at a given local time.
///
/// Mirrors Main.SetFormat + getFormattedTimestamp: each `&X` code sets the
/// colour for the pattern text that follows it; that text is formatted as a
/// SimpleDateFormat pattern (translated to chrono), then emitted as a `Segment`
/// carrying the named colour. Returns `None` for an empty format (off).
pub fn format_timestamp(
    format: &str,
    now: &chrono::DateTime<Local>,
) -> Option<Vec<Segment>> {
    if format.is_empty() {
        return None;
    }
    let chars: Vec<char> = format.chars().collect();
    let mut segments: Vec<Segment> = Vec::new();
    let mut pattern = String::new();
    let mut current: Option<&'static str> = None;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' && i + 1 < chars.len() && is_format_code(chars[i + 1]) {
            // Flush any accumulated pattern before switching colour.
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
            // k/o/m/n are cosmetic-only here: strip the code, keep colour.
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

fn push_segment(
    segments: &mut Vec<Segment>,
    java_pattern: &str,
    color: Option<&'static str>,
    now: &chrono::DateTime<Local>,
) {
    let chrono_pattern = to_chrono_pattern(java_pattern);
    let rendered = now.format(&chrono_pattern).to_string();
    if !rendered.is_empty() {
        segments.push(Segment { text: rendered, color });
    }
}

/// Convenience: format using the current local time.
pub fn timestamp_now(format: &str) -> Option<Vec<Segment>> {
    format_timestamp(format, &Local::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};

    fn fixed() -> chrono::DateTime<Local> {
        // 2026-11-16 23:07:05 local, with a fixed offset of +0 for determinism
        // in the pattern tests (we only assert on the formatted fields).
        Local.with_ymd_and_hms(2026, 11, 16, 23, 7, 5).unwrap()
    }

    #[test]
    fn defaults_match_original() {
        let s = Settings::default();
        assert!(s.enable_mod);
        assert_eq!(s.format, "&7[HH:mm:ss] ");
        assert!(!s.enable_hover);
    }

    #[test]
    fn reads_missing_keys_as_defaults() {
        let s = Settings::read(|_| None);
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn reads_stored_values() {
        let s = Settings::read(|k| match k {
            "enable_mod" => Some("false".into()),
            "enable_hover" => Some("TRUE".into()),
            "format" => Some("&7[HH:mm] ".into()),
            _ => None,
        });
        assert!(!s.enable_mod);
        assert!(s.enable_hover);
        assert_eq!(s.format, "&7[HH:mm] ");
    }

    #[test]
    fn button_cycles_settings() {
        let mut s = Settings::default();
        s.click(0);
        assert!(!s.enable_mod);
        s.click(2);
        assert!(s.enable_hover);
        // default format is preset 0 -> clicking moves to preset 1
        s.click(1);
        assert_eq!(s.format, "&7[HH:mm] ");
        s.click(1);
        assert_eq!(s.format, "&8[&7HH:mm:ss&8] ");
    }

    #[test]
    fn colour_codes_map_to_vanilla_names() {
        assert_eq!(colour_name('7'), Some("gray"));
        assert_eq!(colour_name('8'), Some("dark_gray"));
        assert_eq!(colour_name('a'), Some("green"));
        assert_eq!(colour_name('R'), None); // reset handled separately
        assert_eq!(colour_name('z'), None); // not a code
    }

    #[test]
    fn pattern_translation_core_letters() {
        assert_eq!(to_chrono_pattern("HH:mm:ss"), "%H:%M:%S");
        assert_eq!(to_chrono_pattern("yyyy-MM-dd"), "%Y-%m-%d");
        assert_eq!(to_chrono_pattern("hh:mm:ss a"), "%I:%M:%S %p");
    }

    #[test]
    fn pattern_translation_names_and_millis() {
        // Java `dd` is zero-padded -> chrono `%d`; single `d` -> `%-d`.
        assert_eq!(to_chrono_pattern("MMM dd, yyyy"), "%b %d, %Y");
        assert_eq!(to_chrono_pattern("MMM d, yyyy"), "%b %-d, %Y");
        assert_eq!(to_chrono_pattern("EEE HH:mm:ss.SSS"), "%a %H:%M:%S.%.3f");
    }

    #[test]
    fn pattern_translation_quoted_literal() {
        assert_eq!(to_chrono_pattern("'at' HH:mm"), "at %H:%M");
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
        // "&8[" -> dark_gray, "&7HH:mm" -> gray, "&8] " -> dark_gray
        let colors: Vec<_> = segs.iter().map(|s| s.color).collect();
        assert_eq!(colors, [Some("dark_gray"), Some("gray"), Some("dark_gray")]);
        let joined: String = segs.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, "[23:07] ");
    }

    #[test]
    fn empty_format_is_off() {
        assert_eq!(format_timestamp("", &fixed()), None);
    }

    #[test]
    fn hover_label_reflects_setting() {
        let mut s = Settings::default();
        assert_eq!(s.labels()[2], "On hover only: OFF");
        s.click(2);
        assert_eq!(s.labels()[2], "On hover only: ON");
    }
}
