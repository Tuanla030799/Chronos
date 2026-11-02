//! Astronomy for the calendar: Julian Days, new moons (Sóc), the Sun's longitude and the
//! principal-term index that places leap months.
//!
//! The functions here keep the names of Hồ Ngọc Đức's lunar-calendar algorithm. Behind them
//! are Jean Meeus' full algorithms (*Astronomical Algorithms*, 2nd ed.), with no terms dropped:
//! - [`delta_t`]: ΔT = TT − UT and conversions between Julian Days (UT) and Julian Ephemeris
//!   Days (JDE, TT);
//! - [`sun`]: VSOP87 solar position with nutation and aberration, equation of time;
//! - [`moon_phase`]: true new moons from chapter 49, with all 25 periodic terms and the 14
//!   planetary arguments.

pub mod delta_t;
pub mod moon_phase;
pub mod sun;
mod vsop87_earth;

pub use delta_t::{jd_ut_to_jde, jde_to_jd_ut};

use crate::time::{SolarDate, local_midnight_jd};

/// Julian Day Number of the Gregorian date `day`/`month`/`year`.
///
/// The date is not validated: out-of-range days and months roll over (31 February 2023 gives
/// the number of 3 March 2023), as in Hồ Ngọc Đức's `jdFromDate`. Validate with
/// [`SolarDate::new`] when the input is untrusted.
pub fn jd_from_date(day: u8, month: u8, year: i32) -> i32 {
    SolarDate { year, month, day }.to_jdn()
}

/// Gregorian date `(day, month, year)` of the Julian Day Number `jd`.
pub fn jd_to_date(jd: i32) -> (u8, u8, i32) {
    let date = SolarDate::from_jdn(jd);
    (date.day, date.month, date.year)
}

/// Local date, as a Julian Day Number, of new moon number `k` in a zone `tz_offset` hours east
/// of UTC. New moons are counted from the one of 1 January 1900 (k = 0).
pub fn get_new_moon_day(k: i32, tz_offset: f64) -> i32 {
    moon_phase::new_moon_day(k, tz_offset)
}

/// Apparent geocentric ecliptic longitude of the Sun, in degrees in [0, 360), at the instant
/// `jd_ut` (Julian Day on the UT scale, fractional).
///
/// This is the Sun's true longitude corrected for nutation and aberration: the longitude that
/// defines the 24 solar terms.
pub fn get_sun_longitude_deg(jd_ut: f64) -> f64 {
    sun::apparent_longitude(jd_ut_to_jde(jd_ut))
}

/// Index 0–11 of the principal term (Trung khí) most recently passed at the local midnight that
/// begins day `day_jd`: the Sun is then between `index × 30°` and the next multiple of 30°
/// (0 = Xuân Phân at 0°, 9 = Đông Chí at 270°).
///
/// A lunar month contains a principal term exactly when this index differs between its first
/// day and the first day of the next month; the first month without one becomes the leap month.
pub fn get_solar_term_index(day_jd: i32, tz_offset: f64) -> u8 {
    (get_sun_longitude_deg(local_midnight_jd(day_jd, tz_offset)) / 30.0).floor() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{CN, VN, assert_close, utc};

    #[test]
    fn julian_day_numbers_of_reference_dates() {
        assert_eq!(jd_from_date(1, 1, 1900), 2_415_021);
        assert_eq!(jd_from_date(1, 1, 2000), 2_451_545);
        assert_eq!(jd_from_date(29, 1, 2025), 2_460_705);
        assert_eq!(jd_to_date(2_460_705), (29, 1, 2025));
        assert_eq!(jd_to_date(2_415_021), (1, 1, 1900));
    }

    #[test]
    fn julian_day_numbers_roll_over_like_the_original_algorithm() {
        assert_eq!(jd_from_date(31, 2, 2023), jd_from_date(3, 3, 2023));
        assert_eq!(jd_from_date(1, 13, 2023), jd_from_date(1, 1, 2024));
    }

    #[test]
    fn new_moons_are_counted_from_1_january_1900() {
        // New moon 0: 1900-01-01 13:51 UTC.
        assert_eq!(get_new_moon_day(0, VN), jd_from_date(1, 1, 1900));
        // New moon 1547 opens Tết Ất Tỵ: 2025-01-29 12:36 UTC.
        assert_eq!(get_new_moon_day(1547, VN), jd_from_date(29, 1, 2025));
        // New moon 1325, 2007-02-17 16:14 UTC, falls on different dates in Hà Nội and Beijing.
        assert_eq!(get_new_moon_day(1325, VN), jd_from_date(17, 2, 2007));
        assert_eq!(get_new_moon_day(1325, CN), jd_from_date(18, 2, 2007));
    }

    #[test]
    fn sun_longitude_at_equinoxes_and_solstices() {
        // US Naval Observatory instants of 2024, to the minute (the Sun moves 0.0007° a minute).
        for (instant, longitude) in [
            (utc(2024, 3, 20, 3, 6), 0.0),
            (utc(2024, 6, 20, 20, 51), 90.0),
            (utc(2024, 9, 22, 12, 44), 180.0),
            (utc(2024, 12, 21, 9, 20), 270.0),
        ] {
            let computed = get_sun_longitude_deg(instant);
            assert!((0.0..360.0).contains(&computed));
            let miss = (computed - longitude + 180.0).rem_euclid(360.0) - 180.0;
            assert_close(miss, 0.0, 0.001);
        }
    }

    #[test]
    fn solar_term_index_changes_on_the_day_a_principal_term_begins() {
        // Đông Chí 2024 fell at 16:20 on 21 December in Vietnam.
        assert_eq!(get_solar_term_index(jd_from_date(21, 12, 2024), VN), 8);
        assert_eq!(get_solar_term_index(jd_from_date(22, 12, 2024), VN), 9);
        // Xuân Phân 2025 fell at 16:01 on 20 March in Vietnam; the index wraps from 11 to 0.
        assert_eq!(get_solar_term_index(jd_from_date(20, 3, 2025), VN), 11);
        assert_eq!(get_solar_term_index(jd_from_date(21, 3, 2025), VN), 0);
    }
}
