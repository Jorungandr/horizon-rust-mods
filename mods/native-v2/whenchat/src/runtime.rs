// WhenChat for Horizon: timestamp incoming chat lines.
// Original mod: https://github.com/ErikEdits/WhenChat (MIT, (c) ErikEdits)
//
// This file is injected into the game as `crate::game::whenchat`, but it also
// builds on its own with `cargo test`. chrono is already a dependency of the
// game (horizon-client), so `Local::now()` is available after injection.

use chrono::Local;

pub const ID: &str = "whenchat";
pub const NAME: &str = "WhenChat";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Prepends [HH:mm:ss] to every incoming chat message.";

/// Current local time in the player's timezone, formatted `[HH:mm:ss]`.
/// The Java original uses local time; `chrono::Local` reads the OS timezone.
pub fn local_prefix() -> String {
    format!("[{}]", Local::now().format("%H:%M:%S"))
}

/// Seconds from `SystemTime::UNIX_EPOCH` to a UTC hh:mm:ss string.
/// Dependency-free pure helper: kept as a reference implementation and a
/// fallback for hosts without timezone data. Also exercised by unit tests;
/// the game itself uses `local_prefix`, hence the explicit allow.
#[allow(dead_code)]
pub fn format_utc(epoch_seconds: u64) -> String {
    let seconds_per_day = 86_400u64;
    let seconds_of_day = epoch_seconds % seconds_per_day;

    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day / 60) % 60;
    let second = seconds_of_day % 60;

    format!("{hour:02}:{minute:02}:{second:02}")
}

/// The gray prefix WhenChat puts in front of a line (UTC variant).
#[allow(dead_code)]
pub fn prefix(now_seconds: u64) -> String {
    format!("[{}]", format_utc(now_seconds))
}

/// Build the line the game will display. `prefix` is e.g. `[13:45:30]`.
pub fn decorate(line: Line<'_>, prefix: &str) -> Option<String> {
    let Line { text, system, overlay } = line;
    if overlay {
        return None;
    }
    let _ = system;
    Some(format!("{} {}", prefix, text))
}

pub struct Line<'a> {
    pub text: &'a str,
    pub system: bool,
    pub overlay: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_midnight_and_a_normal_time() {
        assert_eq!(format_utc(0), "00:00:00");
        assert_eq!(format_utc(13 * 3600 + 45 * 60 + 30), "13:45:30");
        assert_eq!(format_utc(86_399), "23:59:59");
    }

    #[test]
    fn wraps_the_time_in_brackets() {
        assert_eq!(prefix(13 * 3600 + 45 * 60 + 30), "[13:45:30]");
    }

    #[test]
    fn local_prefix_is_a_bracketed_clock_time() {
        let value = local_prefix();
        assert_eq!(value.len(), 10, "expected [HH:mm:ss], got {value}");
        assert!(value.starts_with('[') && value.ends_with(']'));
        let digits: Vec<&str> = value[1..9].split(':').collect();
        assert_eq!(digits.len(), 3);
        assert!(digits.iter().all(|part| part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit())));
        let hour: u32 = digits[0].parse().unwrap();
        let minute: u32 = digits[1].parse().unwrap();
        let second: u32 = digits[2].parse().unwrap();
        assert!(hour < 24 && minute < 60 && second < 60);
    }

    #[test]
    fn decorates_chat_but_leaves_the_action_bar_alone() {
        let line = Line { text: "hello", system: false, overlay: false };
        assert_eq!(decorate(line, "[00:00:45]"), Some("[00:00:45] hello".to_string()));

        let action_bar = Line { text: "hello", system: true, overlay: true };
        assert_eq!(decorate(action_bar, "[00:00:45]"), None);
    }
}
