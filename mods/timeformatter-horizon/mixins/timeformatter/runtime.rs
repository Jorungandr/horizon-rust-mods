// Pure logic for the TimeFormatter port.
//
// Ported from m3z0id/TimeFormatter (MIT, Copyright (c) 2025 M3z0id),
// src/main/java/com/m3z0id/timeformatter/client/listeners/{Formatter,ChatListener}.java.
// This file is both the mod's Cargo library (its unit tests) and the module the
// bundle adds to the game as `crate::game::timeformatter`.
//
// The original scans each chat message for Discord-style tags
// `<t:SECONDS[:STYLE]>` and replaces them with the local time. SECONDS may be
// 1-13 digits or one of now/yesterday/tomorrow; STYLE defaults to `f`.
// A doubled `<<t:...>>` is deliberately left untouched (it is the "literal"
// form other players see without the mod).
//
// No `regex` dependency is used: the tag grammar is tiny and hand-written here
// so the crate builds offline. Formatting mirrors the original Java
// DateTimeFormatter patterns, but takes a TimeZone parameter so tests are
// deterministic; production passes `chrono::Local`.

use chrono::{Local, TimeZone};
use std::fmt::Display;

pub const ID: &str = "timeformatter";
pub const NAME: &str = "Time Formatter";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Formats Discord-style <t:...> timestamps in chat into your local time.";

/// The upper bound the original rejects tags above (`> 8640000000000`).
const MAX_TIMESTAMP: i64 = 8_640_000_000_000;

/// Styles accepted by the original, matching its `[fFdDtTrR]` character class.
fn is_valid_style(c: u8) -> bool {
    matches!(c, b'f' | b'F' | b'd' | b'D' | b't' | b'T' | b'R')
}

/// A parsed tag payload.
#[derive(Clone, Debug, PartialEq)]
struct Tag {
    /// Inclusive start byte of the whole `<t:...>` in the source string.
    start: usize,
    /// Exclusive end byte of the whole `<t:...>` in the source string.
    end: usize,
    /// The seconds token exactly as written (digits or a special word).
    raw: String,
    /// Formatting style, default `f`.
    style: u8,
}

/// Parse the payload after `<t:` at `content_start`, requiring a closing `>`
/// immediately after the optional `:STYLE`. Returns (end_before_gt, raw, style).
fn parse_payload(bytes: &[u8], content_start: usize) -> Option<(usize, String, u8)> {
    let mut i = content_start;

    // Token: 1-13 digits OR one of the three words.
    let token_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let digit_len = i - token_start;
    let raw = if digit_len > 0 {
        if digit_len > 13 {
            // Java's \d{1,13} cannot match a longer run followed by >/:.
            return None;
        }
        std::str::from_utf8(&bytes[token_start..i]).ok()?.to_owned()
    } else {
        // Try the special words; require the next byte to be ':' or '>', so
        // "nowhere" is not accepted as "now".
        let rest = std::str::from_utf8(&bytes[i..]).ok()?;
        let word = ["yesterday", "tomorrow", "now"]
            .into_iter()
            .find(|w| rest.starts_with(w))?;
        i += word.len();
        word.to_owned()
    };

    // Optional ':' + exactly one valid style character.
    let mut style = b'f';
    if i < bytes.len() && bytes[i] == b':' {
        if i + 1 >= bytes.len() || !is_valid_style(bytes[i + 1]) {
            return None;
        }
        style = bytes[i + 1];
        i += 2;
    }

    // Must be immediately followed by '>'.
    if i >= bytes.len() || bytes[i] != b'>' {
        return None;
    }
    Some((i, raw, style))
}

/// Find the first well-formed `<t:...>` starting at or after `from`.
/// A `<<t:...>` is skipped so its inner `<t:...>` is never treated as a tag.
fn find_tag(s: &str, from: usize) -> Option<Tag> {
    let bytes = s.as_bytes();
    let mut i = from;
    while i + 3 <= bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1] == b't' && bytes[i + 2] == b':' {
            // Doubled bracket: `<<t:...>>` -> skip entirely.
            if i > 0 && bytes[i - 1] == b'<' {
                i += 3;
                continue;
            }
            if let Some((end, raw, style)) = parse_payload(bytes, i + 3) {
                return Some(Tag {
                    start: i,
                    end: end + 1, // include '>'
                    raw,
                    style,
                });
            }
        }
        i += 1;
    }
    None
}

/// Detect the literal form `<<t:...>>` anywhere in the message. The original
/// bails out for the whole message if such a tag is present.
pub fn has_literal_escape(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if &bytes[i..i + 4] == b"<<t:" {
            if let Some((end, _, _)) = parse_payload(bytes, i + 4) {
                if end + 1 < bytes.len() && bytes[end + 1] == b'>' {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Resolve the seconds token. `now_secs` is injected for deterministic tests.
fn resolve_seconds(raw: &str, now_secs: i64) -> Option<i64> {
    if let Ok(n) = raw.parse::<i64>() {
        return Some(n);
    }
    match raw {
        "now" => Some(now_secs),
        "yesterday" => Some(now_secs - 86_400),
        "tomorrow" => Some(now_secs + 86_400),
        _ => None,
    }
}

/// The original's relative-time algorithm, as a pure function of target and now.
fn format_relative(target_secs: i64, now_secs: i64) -> String {
    if target_secs == now_secs {
        return "now".to_owned();
    }
    let diff = now_secs - target_secs;
    let is_future = diff < 0;
    let total = diff.unsigned_abs();

    let total_days = total / 86_400;
    let years = (total_days as f64 / 365.25) as i64;
    let days = (total_days as f64 % 365.25) as i64;
    let hours = (total / 3_600) % 24;
    let minutes = (total / 60) % 60;
    let seconds = total % 60;

    let mut parts: Vec<String> = Vec::new();
    for (value, unit) in [
        (years, "year"),
        (days, "day"),
        (hours as i64, "hour"),
        (minutes as i64, "minute"),
        (seconds as i64, "second"),
    ] {
        if value > 0 {
            let noun = if value > 1 {
                format!("{unit}s")
            } else {
                unit.to_owned()
            };
            parts.push(format!("{value} {noun}"));
        }
    }
    let body = parts.join(", ");
    if is_future {
        format!("{body} from now")
    } else {
        format!("{body} ago")
    }
}

/// Format one timestamp with the given timezone, mirroring the Java patterns.
fn format_time<Tz: TimeZone>(secs: i64, style: u8, tz: &Tz) -> Option<String>
where
    Tz::Offset: Display,
{
    let dt = tz.timestamp_opt(secs, 0).single()?;
    let out = match style {
        // "MMMM dd, yyyy 'at' HH:mm"
        b'f' => dt.format("%B %d, %Y at %H:%M").to_string(),
        // "EEE, MMMM dd, yyyy 'at' HH:mm"
        b'F' => dt.format("%a, %B %d, %Y at %H:%M").to_string(),
        // "dd.MM.yyyy"
        b'd' => dt.format("%d.%m.%Y").to_string(),
        // "MMMM dd, yyyy"
        b'D' => dt.format("%B %d, %Y").to_string(),
        // "HH:mm"
        b't' => dt.format("%H:%M").to_string(),
        // "HH:mm:ss"
        b'T' => dt.format("%H:%M:%S").to_string(),
        // Relative time is timezone-independent.
        b'R' => format_relative(secs, Local::now().timestamp()),
        _ => return None,
    };
    Some(out)
}

/// Replace all `<t:...>` tags in `input`, using a caller-supplied timezone.
/// Returns `None` when nothing changed (no tag, or a literal `<<t:...>>` is
/// present), so injected helpers can skip rebuilding the component tree.
fn format_inner<Tz: TimeZone>(input: &str, now_secs: i64, tz: &Tz) -> Option<String>
where
    Tz::Offset: Display,
{
    if has_literal_escape(input) {
        return None;
    }
    let mut cursor = 0usize;
    let mut out = String::with_capacity(input.len());
    let mut changed = false;

    while let Some(tag) = find_tag(input, cursor) {
        out.push_str(&input[cursor..tag.start]);

        let secs = match resolve_seconds(&tag.raw, now_secs) {
            Some(v) if v <= MAX_TIMESTAMP => v,
            // Original does `continue` on too-large timestamps: keep the tag
            // as-is and carry on scanning after it.
            _ => {
                out.push_str(&input[tag.start..tag.end]);
                cursor = tag.end;
                continue;
            }
        };

        if let Some(replacement) = format_time(secs, tag.style, tz) {
            out.push_str(&replacement);
            changed = true;
        } else {
            out.push_str(&input[tag.start..tag.end]);
        }
        cursor = tag.end;
    }

    if !changed {
        return None;
    }
    out.push_str(&input[cursor..]);
    Some(out)
}

/// Production entry point: uses the machine's local timezone and current time.
pub fn format_literal(input: &str) -> Option<String> {
    format_inner(input, Local::now().timestamp(), &Local)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn utc() -> FixedOffset {
        FixedOffset::east_opt(0).unwrap()
    }
    const NOW: i64 = 1_704_067_200; // 2024-01-01T00:00:00Z

    #[test]
    fn parses_tag_with_default_style() {
        let tag = find_tag("at <t:1704067200> now", 0).unwrap();
        assert_eq!(tag.raw, "1704067200");
        assert_eq!(tag.style, b'f');
        assert_eq!(&"at <t:1704067200> now"[tag.start..tag.end], "<t:1704067200>");
    }

    #[test]
    fn parses_tag_with_explicit_style() {
        let tag = find_tag("<t:1704067200:T>", 0).unwrap();
        assert_eq!(tag.style, b'T');
    }

    #[test]
    fn rejects_bad_styles_and_too_long_digits() {
        assert!(find_tag("<t:1704067200:x>", 0).is_none());
        assert!(find_tag("<t:12345678901234>", 0).is_none()); // 14 digits
        assert!(find_tag("<t:nowhere>", 0).is_none());
    }

    #[test]
    fn all_seven_styles_match_java_patterns() {
        let tz = utc();
        assert_eq!(format_time(NOW, b'f', &tz).unwrap(), "January 01, 2024 at 00:00");
        assert_eq!(format_time(NOW, b'F', &tz).unwrap(), "Mon, January 01, 2024 at 00:00");
        assert_eq!(format_time(NOW, b'd', &tz).unwrap(), "01.01.2024");
        assert_eq!(format_time(NOW, b'D', &tz).unwrap(), "January 01, 2024");
        assert_eq!(format_time(NOW, b't', &tz).unwrap(), "00:00");
        assert_eq!(format_time(NOW, b'T', &tz).unwrap(), "00:00:00");
        assert!(!format_time(NOW, b'R', &tz).unwrap().is_empty());
    }

    #[test]
    fn special_values_resolve_to_expected_seconds() {
        assert_eq!(resolve_seconds("now", NOW), Some(NOW));
        assert_eq!(resolve_seconds("yesterday", NOW), Some(NOW - 86_400));
        assert_eq!(resolve_seconds("tomorrow", NOW), Some(NOW + 86_400));
    }

    #[test]
    fn literal_double_bracket_is_left_untouched() {
        assert!(has_literal_escape("see <<t:1704067200:F>> here"));
        assert_eq!(format_inner("see <<t:1704067200:F>> here", NOW, &utc()), None);
    }

    #[test]
    fn replaces_single_tag_and_keeps_surrounding_text() {
        let out = format_inner("start <t:1704067200:t> end", NOW, &utc()).unwrap();
        assert_eq!(out, "start 00:00 end");
    }

    #[test]
    fn replaces_multiple_tags() {
        let out = format_inner("<t:1704067200:t> and <t:1704067200:d>", NOW, &utc()).unwrap();
        assert_eq!(out, "00:00 and 01.01.2024");
    }

    #[test]
    fn no_tag_returns_none() {
        assert_eq!(format_inner("plain text", NOW, &utc()), None);
    }

    #[test]
    fn relative_time_past_and_future() {
        assert_eq!(format_relative(NOW, NOW), "now");
        assert_eq!(format_relative(NOW - 65, NOW), "1 minute, 5 seconds ago");
        assert_eq!(format_relative(NOW + 120, NOW), "2 minutes from now");
        assert_eq!(format_relative(NOW - 90 * 86_400, NOW), "90 days ago");
    }
}
