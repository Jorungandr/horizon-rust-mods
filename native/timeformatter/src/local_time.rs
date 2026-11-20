//! Local wall-clock date/time helpers without third-party crates.
//!
//! Windows reads the OS local time through `GetLocalTime`; other targets use
//! UTC from `SystemTime`. The conversion helpers are also used by the
//! TimeFormatter-style tests so formatting can be checked deterministically.

/// A civil date/time in the machine's local timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    /// 0 = Sunday, 1 = Monday, ... 6 = Saturday.
    pub weekday: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub millis: u32,
    /// Minutes east of UTC.
    pub offset_minutes: i32,
}

impl DateTime {
    /// Seconds since Unix epoch represented by these local fields.
    pub fn unix_seconds(&self) -> i64 {
        days_from_civil(self.year, self.month, self.day) * 86_400
            + self.hour as i64 * 3_600
            + self.minute as i64 * 60
            + self.second as i64
            - self.offset_minutes as i64 * 60
    }

    pub fn hour12(&self) -> u32 {
        let h = self.hour % 12;
        if h == 0 { 12 } else { h }
    }

    pub fn is_pm(&self) -> bool {
        self.hour >= 12
    }

    pub fn month_short(&self) -> &'static str {
        const NAMES: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        NAMES.get(self.month.saturating_sub(1) as usize).copied().unwrap_or("Jan")
    }

    pub fn month_long(&self) -> &'static str {
        const NAMES: [&str; 12] = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ];
        NAMES.get(self.month.saturating_sub(1) as usize).copied().unwrap_or("January")
    }

    pub fn weekday_short(&self) -> &'static str {
        const NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        NAMES.get(self.weekday as usize).copied().unwrap_or("Sun")
    }

    pub fn weekday_long(&self) -> &'static str {
        const NAMES: [&str; 7] = [
            "Sunday", "Monday", "Tuesday", "Wednesday",
            "Thursday", "Friday", "Saturday",
        ];
        NAMES.get(self.weekday as usize).copied().unwrap_or("Sunday")
    }

    pub fn offset_label(&self) -> String {
        if self.offset_minutes == 0 {
            return "UTC".to_owned();
        }
        let sign = if self.offset_minutes > 0 { '+' } else { '-' };
        let abs = self.offset_minutes.unsigned_abs();
        format!("UTC{sign}{:02}:{:02}", abs / 60, abs % 60)
    }
}

/// Current local date/time.
pub fn now() -> DateTime {
    let utc_secs = unix_now();
    let mut value = read_local_fields();
    let local_as_utc = days_from_civil(value.year, value.month, value.day) * 86_400
        + value.hour as i64 * 3_600
        + value.minute as i64 * 60
        + value.second as i64;
    value.offset_minutes = ((local_as_utc - utc_secs) / 60) as i32;
    value
}

/// Build a local `DateTime` from a Unix timestamp and an explicit UTC offset.
pub fn from_unix_with_offset(secs: i64, millis: u32, offset_minutes: i32) -> DateTime {
    let local = secs + offset_minutes as i64 * 60;
    let days = floor_div(local, 86_400);
    let rem = local - days * 86_400;
    let (year, month, day) = civil_from_days(days);
    DateTime {
        year,
        month,
        day,
        weekday: (days + 4).rem_euclid(7) as u32,
        hour: (rem / 3_600) as u32,
        minute: ((rem / 60) % 60) as u32,
        second: (rem % 60) as u32,
        millis,
        offset_minutes,
    }
}

/// Build a local `DateTime` from a Unix timestamp using this machine's offset.
pub fn from_unix(secs: i64, millis: u32) -> DateTime {
    from_unix_with_offset(secs, millis, now().offset_minutes)
}

/// Current Unix timestamp in seconds.
pub fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(windows)]
fn read_local_fields() -> DateTime {
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
    let mut raw = SysTime {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        millis: 0,
    };
    // SAFETY: `GetLocalTime` only writes its 16-byte out parameter.
    unsafe {
        GetLocalTime(&mut raw);
    }
    DateTime {
        year: raw.year as i32,
        month: raw.month as u32,
        day: raw.day as u32,
        weekday: raw.day_of_week as u32,
        hour: raw.hour as u32,
        minute: raw.minute as u32,
        second: raw.second as u32,
        millis: raw.millis as u32,
        offset_minutes: 0,
    }
}

#[cfg(not(windows))]
fn read_local_fields() -> DateTime {
    let secs = unix_now();
    let mut value = from_unix_with_offset(secs, 0, 0);
    value.offset_minutes = 0;
    value
}

fn floor_div(a: i64, b: i64) -> i64 {
    let q = a / b;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) { q - 1 } else { q }
}

/// Howard Hinnant's civil calendar algorithms.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = month as i64 + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    if m <= 2 {
        y += 1;
    }
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_round_trip() {
        let days = days_from_civil(2024, 1, 1);
        assert_eq!(civil_from_days(days), (2024, 1, 1));
        assert_eq!((days + 4).rem_euclid(7), 1); // Monday
    }

    #[test]
    fn unix_conversion_with_offset() {
        let dt = from_unix_with_offset(1_704_067_200, 0, 0);
        assert_eq!((dt.year, dt.month, dt.day), (2024, 1, 1));
        assert_eq!((dt.hour, dt.minute, dt.second), (0, 0, 0));
        assert_eq!(dt.weekday, 1);
        assert_eq!(dt.unix_seconds(), 1_704_067_200);

        let dt = from_unix_with_offset(1_704_067_200, 0, 480);
        assert_eq!((dt.year, dt.month, dt.day, dt.hour), (2024, 1, 1, 8));
        assert_eq!(dt.unix_seconds(), 1_704_067_200);
    }
}
