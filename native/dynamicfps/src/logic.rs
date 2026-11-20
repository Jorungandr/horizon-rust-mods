//! Which frame rate cap to ask for, kept apart from the game so it can be tested on its own.

/// Frame rates a window in the background may be held to.
pub const UNFOCUSED: [i32; 6] = [1, 5, 10, 15, 20, 30];
/// Frame rates a minimised window may be held to.
pub const MINIMIZED: [i32; 4] = [1, 5, 10, 20];

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
}
