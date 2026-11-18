//! Auto Reconnect: rejoin the last server after a lost connection.
//!
//! The retry bookkeeping is pure, so it is tested without a game. The engine
//! adapter is appended at injection time and hooks three places in
//! `game/remote.rs` and `game/native_hooks.rs` (see
//! `tools/inject_all_ports.py`).

pub const ID: &str = "autoreconnect";
pub const NAME: &str = "Auto Reconnect";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Rejoins the last server after the connection is lost.";

/// Countdowns the settings button cycles through, in seconds.
const DELAYS: [u32; 5] = [3, 5, 10, 30, 60];
/// Attempt limits the settings button cycles through.
const ATTEMPTS: [u32; 4] = [1, 3, 5, 10];
/// Never rejoin sooner than this after losing the connection, even with a
/// zero delay: the disconnect screen has to draw first, and an instant retry
/// against a server that is still shutting down only fails again.
const MIN_DELAY_MS: u64 = 1_000;

const KEY_ENABLE: &str = "enable_mod";
const KEY_DELAY: &str = "delay";
const KEY_ATTEMPTS: &str = "max_attempts";

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub enable_mod: bool,
    /// Seconds to wait on the disconnect screen before rejoining.
    pub delay: u32,
    /// Attempts in a row before giving up, so a kick loop cannot spin.
    pub max_attempts: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mod: true,
            delay: 5,
            max_attempts: 3,
        }
    }
}

impl Settings {
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let defaults = Self::default();
        Self {
            enable_mod: match lookup(KEY_ENABLE).as_deref() {
                Some("on") | Some("true") => true,
                Some("off") | Some("false") => false,
                _ => defaults.enable_mod,
            },
            delay: lookup(KEY_DELAY)
                .and_then(|v| v.trim().parse::<u32>().ok())
                .filter(|d| DELAYS.contains(d))
                .unwrap_or(defaults.delay),
            max_attempts: lookup(KEY_ATTEMPTS)
                .and_then(|v| v.trim().parse::<u32>().ok())
                .filter(|a| ATTEMPTS.contains(a))
                .unwrap_or(defaults.max_attempts),
        }
    }

    pub fn stored(&self) -> Vec<(&'static str, String)> {
        vec![
            (KEY_ENABLE, on_off(self.enable_mod).to_owned()),
            (KEY_DELAY, self.delay.to_string()),
            (KEY_ATTEMPTS, self.max_attempts.to_string()),
        ]
    }

    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("Auto reconnect: {}", on_off(self.enable_mod).to_uppercase()),
            format!("Delay: {}s", self.delay),
            format!("Max attempts: {}", self.max_attempts),
        ]
    }

    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.enable_mod = !self.enable_mod,
            1 => {
                let i = DELAYS.iter().position(|d| *d == self.delay).unwrap_or(0);
                self.delay = DELAYS[(i + 1) % DELAYS.len()];
            }
            2 => {
                let i = ATTEMPTS
                    .iter()
                    .position(|a| *a == self.max_attempts)
                    .unwrap_or(0);
                self.max_attempts = ATTEMPTS[(i + 1) % ATTEMPTS.len()];
            }
            _ => {}
        }
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// The retry counter, one connection's lifetime at a time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Retry {
    /// Attempts made since the last successful join.
    pub attempts: u32,
    /// Milliseconds left on the countdown; `None` when nothing is pending.
    pub remaining_ms: Option<u64>,
}

impl Retry {
    /// The connection was lost (`lost`) or left on purpose.
    ///
    /// Returns true when a countdown started. A deliberate leave, a disabled
    /// mod, or a spent attempt budget all leave nothing pending.
    pub fn lost(&mut self, settings: &Settings, lost: bool) -> bool {
        self.remaining_ms = None;
        if !lost || !settings.enable_mod {
            return false;
        }
        if self.attempts >= settings.max_attempts {
            return false;
        }
        self.attempts += 1;
        let delay = u64::from(settings.delay) * 1_000;
        self.remaining_ms = Some(delay.max(MIN_DELAY_MS));
        true
    }

    /// A join succeeded: the budget starts over.
    pub fn joined(&mut self) {
        self.attempts = 0;
        self.remaining_ms = None;
    }

    /// Advance by `dt_ms`. True on the frame the countdown reaches zero, when
    /// the mod should rejoin; the countdown is cleared first, so a failed
    /// rejoin starts a fresh one from `lost`.
    pub fn advance(&mut self, dt_ms: u64) -> bool {
        let Some(left) = self.remaining_ms else {
            return false;
        };
        if left <= dt_ms {
            self.remaining_ms = None;
            return true;
        }
        self.remaining_ms = Some(left - dt_ms);
        false
    }

    pub fn cancel(&mut self) {
        self.remaining_ms = None;
    }

    /// Whole seconds still to wait, rounded up, for the on-screen countdown.
    pub fn seconds_left(&self) -> Option<u64> {
        self.remaining_ms.map(|ms| ms.div_ceil(1_000))
    }
}

/// The line the disconnect screen shows while a rejoin is pending.
pub fn countdown_text(retry: &Retry, settings: &Settings) -> Option<String> {
    let seconds = retry.seconds_left()?;
    Some(format!(
        "Auto reconnect: rejoining in {seconds}s (attempt {}/{})",
        retry.attempts, settings.max_attempts
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lost_connection_starts_the_countdown() {
        let settings = Settings::default();
        let mut retry = Retry::default();
        assert!(retry.lost(&settings, true));
        assert_eq!(retry.attempts, 1);
        assert_eq!(retry.seconds_left(), Some(5));
    }

    #[test]
    fn a_deliberate_leave_does_nothing() {
        let settings = Settings::default();
        let mut retry = Retry::default();
        assert!(!retry.lost(&settings, false));
        assert_eq!(retry.attempts, 0);
        assert_eq!(retry.remaining_ms, None);
    }

    #[test]
    fn a_disabled_mod_does_nothing() {
        let mut settings = Settings::default();
        settings.enable_mod = false;
        let mut retry = Retry::default();
        assert!(!retry.lost(&settings, true));
        assert_eq!(retry.remaining_ms, None);
    }

    #[test]
    fn the_attempt_budget_stops_a_kick_loop() {
        let mut settings = Settings::default();
        settings.max_attempts = 2;
        let mut retry = Retry::default();
        assert!(retry.lost(&settings, true));
        retry.advance(10_000);
        assert!(retry.lost(&settings, true));
        retry.advance(10_000);
        // The budget is spent: no countdown, and the counter does not creep.
        assert!(!retry.lost(&settings, true));
        assert_eq!(retry.attempts, 2);
        assert_eq!(retry.remaining_ms, None);
    }

    #[test]
    fn a_successful_join_restarts_the_budget() {
        let mut settings = Settings::default();
        settings.max_attempts = 1;
        let mut retry = Retry::default();
        assert!(retry.lost(&settings, true));
        assert!(!retry.lost(&settings, true), "budget of one is spent");
        retry.joined();
        assert_eq!(retry.attempts, 0);
        assert!(retry.lost(&settings, true), "a fresh budget after joining");
    }

    #[test]
    fn advance_fires_once_at_zero() {
        let settings = Settings { delay: 3, ..Settings::default() };
        let mut retry = Retry::default();
        retry.lost(&settings, true);
        assert!(!retry.advance(1_000));
        assert!(!retry.advance(1_000));
        assert_eq!(retry.seconds_left(), Some(1));
        assert!(retry.advance(1_000), "the third second fires the rejoin");
        assert!(!retry.advance(1_000), "and never fires twice");
    }

    #[test]
    fn a_zero_delay_still_waits_for_the_disconnect_screen() {
        let settings = Settings { delay: 0, ..Settings::default() };
        let mut retry = Retry::default();
        retry.lost(&settings, true);
        assert_eq!(retry.remaining_ms, Some(MIN_DELAY_MS));
        assert!(retry.advance(MIN_DELAY_MS));
    }

    #[test]
    fn a_large_step_overshoots_the_zero_without_wrapping() {
        let settings = Settings::default();
        let mut retry = Retry::default();
        retry.lost(&settings, true);
        assert!(retry.advance(u64::MAX));
        assert_eq!(retry.remaining_ms, None);
    }

    #[test]
    fn the_countdown_line_names_the_attempt() {
        let settings = Settings { delay: 5, max_attempts: 3, ..Settings::default() };
        let mut retry = Retry::default();
        assert_eq!(countdown_text(&retry, &settings), None);
        retry.lost(&settings, true);
        let text = countdown_text(&retry, &settings).unwrap();
        assert!(text.contains("5s"), "{text}");
        assert!(text.contains("1/3"), "{text}");
    }

    #[test]
    fn settings_round_trip_through_storage() {
        let mut settings = Settings::default();
        settings.delay = 30;
        settings.max_attempts = 10;
        settings.enable_mod = false;
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
        for i in 0..settings.labels().len() {
            let before = settings.clone();
            settings.click(i);
            assert_ne!(before, settings, "button {i} did nothing");
        }
    }
}
