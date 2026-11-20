//! Pure logic for TimeFormatter, adapted to the official Horizon SDK.
//!
//! Scans chat for Discord-style `<t:SECONDS[:STYLE]>` tags and rewrites them
//! into the viewer's local time. No regex/chrono dependency is used.

use crate::local_time::{self, DateTime};

pub const ID: &str = "timeformatter";
pub const NAME: &str = "Time Formatter";

const MAX_TIMESTAMP: i64 = 8_640_000_000_000;

fn is_valid_style(c: u8) -> bool {
    matches!(c, b'f' | b'F' | b'd' | b'D' | b't' | b'T' | b'R')
}

#[derive(Clone, Debug, PartialEq)]
struct Tag {
    start: usize,
    end: usize,
    raw: String,
    style: u8,
}

fn parse_payload(bytes: &[u8], content_start: usize) -> Option<(usize, String, u8)> {
    let mut i = content_start;
    let token_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let digit_len = i - token_start;
    let raw = if digit_len > 0 {
        if digit_len > 13 {
            return None;
        }
        std::str::from_utf8(&bytes[token_start..i]).ok()?.to_owned()
    } else {
        let rest = std::str::from_utf8(&bytes[i..]).ok()?;
        let word = ["yesterday", "tomorrow", "now"]
            .into_iter()
            .find(|w| rest.starts_with(w))?;
        i += word.len();
        word.to_owned()
    };

    let mut style = b'f';
    if i < bytes.len() && bytes[i] == b':' {
        if i + 1 >= bytes.len() || !is_valid_style(bytes[i + 1]) {
            return None;
        }
        style = bytes[i + 1];
        i += 2;
    }
    if i >= bytes.len() || bytes[i] != b'>' {
        return None;
    }
    Some((i, raw, style))
}

fn find_tag(s: &str, from: usize) -> Option<Tag> {
    let bytes = s.as_bytes();
    let mut i = from;
    while i + 3 <= bytes.len() {
        if bytes[i] == b'<' && i + 1 < bytes.len() && bytes[i + 1] == b't' && bytes[i + 2] == b':' {
            if i > 0 && bytes[i - 1] == b'<' {
                i += 3;
                continue;
            }
            if let Some((end, raw, style)) = parse_payload(bytes, i + 3) {
                return Some(Tag {
                    start: i,
                    end: end + 1,
                    raw,
                    style,
                });
            }
        }
        i += 1;
    }
    None
}

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

    let mut parts = Vec::new();
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

fn format_time(secs: i64, style: u8, now: &DateTime) -> Option<String> {
    let dt = local_time::from_unix_with_offset(secs, 0, now.offset_minutes);
    let out = match style {
        b'f' => format!(
            "{} {:02}, {} at {:02}:{:02}",
            dt.month_long(), dt.day, dt.year, dt.hour, dt.minute
        ),
        b'F' => format!(
            "{}, {} {:02}, {} at {:02}:{:02}",
            dt.weekday_short(), dt.month_long(), dt.day, dt.year, dt.hour, dt.minute
        ),
        b'd' => format!("{:02}.{:02}.{}", dt.day, dt.month, dt.year),
        b'D' => format!("{} {:02}, {}", dt.month_long(), dt.day, dt.year),
        b't' => format!("{:02}:{:02}", dt.hour, dt.minute),
        b'T' => format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second),
        b'R' => format_relative(secs, now.unix_seconds()),
        _ => return None,
    };
    Some(out)
}

fn format_inner(input: &str, now_secs: i64, now: &DateTime) -> Option<String> {
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
            _ => {
                out.push_str(&input[tag.start..tag.end]);
                cursor = tag.end;
                continue;
            }
        };
        if let Some(replacement) = format_time(secs, tag.style, now) {
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
    let now = local_time::now();
    format_inner(input, now.unix_seconds(), &now)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_704_067_200; // 2024-01-01T00:00:00Z

    fn now() -> DateTime {
        local_time::from_unix_with_offset(NOW, 0, 0)
    }

    #[test]
    fn parses_tag_with_default_style() {
        let tag = find_tag("at <t:1704067200> now", 0).unwrap();
        assert_eq!(tag.raw, "1704067200");
        assert_eq!(tag.style, b'f');
        assert_eq!(&"at <t:1704067200> now"[tag.start..tag.end], "<t:1704067200>");
    }

    #[test]
    fn parses_tag_with_explicit_style() {
        assert_eq!(find_tag("<t:1704067200:T>", 0).unwrap().style, b'T');
    }

    #[test]
    fn rejects_bad_styles_and_too_long_digits() {
        assert!(find_tag("<t:1704067200:x>", 0).is_none());
        assert!(find_tag("<t:12345678901234>", 0).is_none());
        assert!(find_tag("<t:nowhere>", 0).is_none());
    }

    #[test]
    fn all_seven_styles_match_java_patterns() {
        let now = now();
        assert_eq!(
            format_time(NOW, b'f', &now).unwrap(),
            "January 01, 2024 at 00:00"
        );
        assert_eq!(
            format_time(NOW, b'F', &now).unwrap(),
            "Mon, January 01, 2024 at 00:00"
        );
        assert_eq!(format_time(NOW, b'd', &now).unwrap(), "01.01.2024");
        assert_eq!(format_time(NOW, b'D', &now).unwrap(), "January 01, 2024");
        assert_eq!(format_time(NOW, b't', &now).unwrap(), "00:00");
        assert_eq!(format_time(NOW, b'T', &now).unwrap(), "00:00:00");
        assert!(!format_time(NOW, b'R', &now).unwrap().is_empty());
    }

    #[test]
    fn special_values_resolve_to_expected_seconds() {
        assert_eq!(resolve_seconds("now", NOW), Some(NOW));
        assert_eq!(resolve_seconds("yesterday", NOW), Some(NOW - 86_400));
        assert_eq!(resolve_seconds("tomorrow", NOW), Some(NOW + 86_400));
    }

    #[test]
    fn literal_double_bracket_is_left_untouched() {
        let input = "see <<t:1704067200:F>> here";
        assert!(has_literal_escape(input));
        assert_eq!(format_inner(input, NOW, &now()), None);
    }

    #[test]
    fn replaces_single_tag_and_keeps_surrounding_text() {
        let out = format_inner("start <t:1704067200:t> end", NOW, &now()).unwrap();
        assert_eq!(out, "start 00:00 end");
    }

    #[test]
    fn replaces_multiple_tags() {
        let out = format_inner("<t:1704067200:t> and <t:1704067200:d>", NOW, &now()).unwrap();
        assert_eq!(out, "00:00 and 01.01.2024");
    }

    #[test]
    fn no_tag_returns_none() {
        assert_eq!(format_inner("plain text", NOW, &now()), None);
    }

    #[test]
    fn relative_time_past_and_future() {
        assert_eq!(format_relative(NOW, NOW), "now");
        assert_eq!(format_relative(NOW - 65, NOW), "1 minute, 5 seconds ago");
        assert_eq!(format_relative(NOW + 120, NOW), "2 minutes from now");
        assert_eq!(format_relative(NOW - 90 * 86_400, NOW), "90 days ago");
    }
}
