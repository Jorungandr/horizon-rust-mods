//! Local wall-clock time as `HH:MM:SS`, without third-party dependencies.
//!
//! Windows reads the OS local time through `GetLocalTime` (kernel32, linked by
//! the MinGW toolchain); other targets fall back to UTC from `SystemTime`.

/// The formatted local clock, `HH:MM:SS`.
pub fn hms() -> String {
    #[cfg(windows)]
    {
        local_windows()
    }
    #[cfg(not(windows))]
    {
        utc_std()
    }
}

#[cfg(windows)]
fn local_windows() -> String {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SysTime {
        year: u16,
        month: u16,
        day_of_week: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        millis: u16,
    }
    unsafe extern "system" {
        fn GetLocalTime(out: *mut SysTime);
    }
    let mut time = SysTime {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        millis: 0,
    };
    // SAFETY: `GetLocalTime` writes only its 16-byte out parameter.
    unsafe {
        GetLocalTime(&mut time);
    }
    format!("[{:02}:{:02}:{:02}]", time.hour, time.minute, time.second)
}

#[cfg(not(windows))]
fn utc_std() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let day_seconds = seconds % 86_400;
    format!(
        "[{:02}:{:02}:{:02}]",
        day_seconds / 3_600,
        (day_seconds / 60) % 60,
        day_seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::hms;

    #[test]
    fn formatted_as_bracketed_clock_time() {
        let value = hms();
        assert_eq!(value.len(), 10, "expected [HH:MM:SS], got {value}");
        assert!(value.starts_with('[') && value.ends_with(']'));
        let digits: Vec<&str> = value[1..value.len() - 1].split(':').collect();
        assert_eq!(digits.len(), 3);
        assert!(digits
            .iter()
            .all(|part| part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit())));
        let hour: u32 = digits[0].parse().unwrap();
        let minute: u32 = digits[1].parse().unwrap();
        let second: u32 = digits[2].parse().unwrap();
        assert!(hour < 24 && minute < 60 && second < 60);
    }
}
