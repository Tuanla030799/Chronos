//! Helpers shared by the unit tests.

use crate::time::{SolarDate, SolarDateTime};

/// Vietnam, UTC+7.
pub(crate) const VN: f64 = 7.0;

/// China, UTC+8.
pub(crate) const CN: f64 = 8.0;

#[track_caller]
pub(crate) fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected} ± {tolerance}, got {actual} (off by {})",
        actual - expected
    );
}

#[track_caller]
pub(crate) fn date(year: i32, month: u8, day: u8) -> SolarDate {
    SolarDate::new(year, month, day).expect("test dates are valid")
}

#[track_caller]
pub(crate) fn date_time(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> SolarDateTime {
    SolarDateTime::new(year, month, day, hour, minute, 0).expect("test times are valid")
}

/// Julian Day of a UTC wall-clock time.
#[track_caller]
pub(crate) fn utc(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> f64 {
    date_time(year, month, day, hour, minute).to_julian_day(0.0)
}
