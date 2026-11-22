//! Which frame rate cap to ask for, kept apart from the game so it can be tested on its own.

/// Frame rates a window in the background may be held to.
pub const UNFOCUSED: [i32; 6] = [1, 5, 10, 15, 20, 30];
/// Frame rates a minimised window may be held to. Horizon already caps a
/// minimised window at 10 FPS before this hook runs, and the cap we report is
/// clamped to the game's own, so anything above 10 would be ineffective. Only
/// values at or below 10 are offered.
pub const MINIMIZED: [i32; 3] = [1, 5, 10];
/// The index of "20 FPS" in the old six-option minimised list. It was already
/// clamped to Horizon's own 10 FPS cap, which is now index 2.
pub const LEGACY_MINIMIZED_20: u32 = 3;
/// New index for the same effective cap.
pub const MINIMIZED_10_INDEX: u32 = 2;

/// Move a choice stored by 0.2.0 onto the equivalent option in 0.2.1. The old
/// minimised list had six entries and index 3 was "20 FPS"; it was reported as
/// 10 FPS because the game clamps it. Without this, the new three-entry list
/// rejects index 3 and the reader silently falls back to 1 FPS.
pub fn migrate_minimized_index(index: u32) -> u32 {
    if index == LEGACY_MINIMIZED_20 {
        MINIMIZED_10_INDEX
    } else {
        index
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caps {
    pub unfocused: i32,
    pub minimized: i32,
}

impl Default for Caps {
    fn default() -> Self {
        Self {
            unfocused: UNFOCUSED[2],
            minimized: MINIMIZED[0],
        }
    }
}

/// Pick a value from a list by index, falling back to `default` when the index is
/// out of range, as a settings file edited by hand can make it.
pub fn pick(values: &[i32], index: u32, default: i32) -> i32 {
    values.get(index as usize).copied().unwrap_or(default)
}

/// The cap to report: never above the game's own limit, and never below one frame.
/// A minimised window is the strongest case, then an unfocused one.
pub fn limit_for(original: i32, focused: bool, minimized: bool, caps: Caps) -> i32 {
    let wanted = if minimized {
        caps.minimized
    } else if !focused {
        caps.unfocused
    } else {
        return original.max(1);
    };
    original.max(1).min(wanted.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPS: Caps = Caps {
        unfocused: 10,
        minimized: 1,
    };

    #[test]
    fn a_focused_window_keeps_its_limit() {
        assert_eq!(limit_for(260, true, false, CAPS), 260);
        assert_eq!(limit_for(60, true, false, CAPS), 60);
    }

    #[test]
    fn a_background_window_is_capped() {
        assert_eq!(limit_for(260, false, false, CAPS), 10);
        assert_eq!(limit_for(260, false, true, CAPS), 1);
    }

    #[test]
    fn a_lower_limit_is_never_raised() {
        assert_eq!(limit_for(5, false, false, CAPS), 5);
        assert_eq!(limit_for(1, false, true, CAPS), 1);
    }

    #[test]
    fn nonsense_settings_still_leave_one_frame() {
        let bad = Caps {
            unfocused: 0,
            minimized: -3,
        };
        assert_eq!(limit_for(260, false, false, bad), 1);
        assert_eq!(limit_for(260, false, true, bad), 1);
    }

    #[test]
    fn a_hand_edited_index_falls_back() {
        assert_eq!(pick(&UNFOCUSED, 2, 10), 10);
        assert_eq!(pick(&UNFOCUSED, 99, 10), 10);
        assert_eq!(pick(&MINIMIZED, 0, 1), 1);
    }

    #[test]
    fn old_twenty_fps_choice_becomes_ten_fps() {
        assert_eq!(migrate_minimized_index(3), 2);
        assert_eq!(pick(&MINIMIZED, migrate_minimized_index(3), 1), 10);
        // Other stored choices are left alone.
        assert_eq!(migrate_minimized_index(0), 0);
        assert_eq!(migrate_minimized_index(2), 2);
        assert_eq!(migrate_minimized_index(99), 99);
    }
}
