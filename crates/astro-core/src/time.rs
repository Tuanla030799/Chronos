//! Gregorian (solar) dates, wall-clock times and Julian Day conversions.
//!
//! Dates use the proleptic Gregorian calendar. A Julian Day Number (JDN) labels a civil date
//! (it is the Julian Day at noon of that date); a Julian Day (JD) is a continuous instant
//! counted in days. JDs are on the UT scale unless named `jde` (Terrestrial Time).

use crate::error::AstroError;

/// Julian Day of the J2000.0 epoch, 2000-01-01 12:00 TT.
pub(crate) const J2000: f64 = 2_451_545.0;

/// Seconds in a day.
pub(crate) const SECONDS_PER_DAY: f64 = 86_400.0;

/// A date in the proleptic Gregorian calendar (Dương lịch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolarDate {
    /// Year.
    pub year: i32,
    /// Month, 1–12.
    pub month: u8,
    /// Day of the month, 1–31.
    pub day: u8,
}

impl SolarDate {
    /// Creates a date, rejecting days that do not exist.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, AstroError> {
        let date = Self { year, month, day };
        date.validate()?;
        Ok(date)
    }

    /// Checks that the fields name an existing Gregorian date.
    pub fn validate(&self) -> Result<(), AstroError> {
        match days_in_month(self.year, self.month) {
            Some(length) if (1..=length).contains(&self.day) => Ok(()),
            _ => Err(AstroError::InvalidSolarDate {
                year: self.year,
                month: self.month,
                day: self.day,
            }),
        }
    }

    /// Julian Day Number of this date (Fliegel & Van Flandern, 1968).
    pub fn to_jdn(self) -> i32 {
        let (year, month, day) = (
            i64::from(self.year),
            i64::from(self.month),
            i64::from(self.day),
        );
        let a = (14 - month).div_euclid(12);
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;
        let jdn = day + (153 * m + 2).div_euclid(5) + 365 * y + y.div_euclid(4) - y.div_euclid(100)
            + y.div_euclid(400)
            - 32_045;
        jdn as i32
    }

    /// The date whose Julian Day Number is `jdn`.
    pub fn from_jdn(jdn: i32) -> Self {
        let a = i64::from(jdn) + 32_044;
        let b = (4 * a + 3).div_euclid(146_097);
        let c = a - (146_097 * b).div_euclid(4);
        let d = (4 * c + 3).div_euclid(1461);
        let e = c - (1461 * d).div_euclid(4);
        let m = (5 * e + 2).div_euclid(153);
        Self {
            year: (100 * b + d - 4800 + m.div_euclid(10)) as i32,
            month: (m + 3 - 12 * m.div_euclid(10)) as u8,
            day: (e - (153 * m + 2).div_euclid(5) + 1) as u8,
        }
    }
}

/// A wall-clock date and time to the second; its time zone is given where it is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolarDateTime {
    /// Year.
    pub year: i32,
    /// Month, 1–12.
    pub month: u8,
    /// Day of the month, 1–31.
    pub day: u8,
    /// Hour, 0–23.
    pub hour: u8,
    /// Minute, 0–59.
    pub minute: u8,
    /// Second, 0–59.
    pub second: u8,
}

impl SolarDateTime {
    /// Creates a date-time, rejecting dates and times of day that do not exist.
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, AstroError> {
        let date_time = Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        };
        date_time.validate()?;
        Ok(date_time)
    }

    /// Checks that the fields name an existing date and time of day.
    pub fn validate(&self) -> Result<(), AstroError> {
        self.date().validate()?;
        if self.hour < 24 && self.minute < 60 && self.second < 60 {
            Ok(())
        } else {
            Err(AstroError::InvalidTime {
                hour: self.hour,
                minute: self.minute,
                second: self.second,
            })
        }
    }

    /// The calendar date.
    pub fn date(&self) -> SolarDate {
        SolarDate {
            year: self.year,
            month: self.month,
            day: self.day,
        }
    }

    /// The instant of this wall-clock time as a Julian Day (UT), in a zone `tz_offset` hours
    /// east of UTC.
    pub fn to_julian_day(&self, tz_offset: f64) -> f64 {
        let seconds =
            f64::from(self.hour) * 3600.0 + f64::from(self.minute) * 60.0 + f64::from(self.second);
        local_midnight_jd(self.date().to_jdn(), tz_offset) + seconds / SECONDS_PER_DAY
    }

    /// The wall-clock time, rounded to the nearest second, of the instant `jd_ut` in a zone
    /// `tz_offset` hours east of UTC.
    pub fn from_julian_day(jd_ut: f64, tz_offset: f64) -> Self {
        let local = jd_ut + 0.5 + tz_offset / 24.0;
        let mut jdn = local.floor();
        let mut seconds = ((local - jdn) * SECONDS_PER_DAY).round();
        if seconds >= SECONDS_PER_DAY {
            jdn += 1.0;
            seconds -= SECONDS_PER_DAY;
        }
        let date = SolarDate::from_jdn(jdn as i32);
        let seconds = seconds as u32;
        Self {
            year: date.year,
            month: date.month,
            day: date.day,
            hour: (seconds / 3600) as u8,
            minute: (seconds / 60 % 60) as u8,
            second: (seconds % 60) as u8,
        }
    }
}

/// Whether `year` is a Gregorian leap year.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in `month` (1–12) of `year`, or `None` if `month` is out of range.
pub fn days_in_month(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if is_leap_year(year) => Some(29),
        2 => Some(28),
        _ => None,
    }
}

/// Julian Day (UT) of the local midnight that begins day `jdn` in a zone `tz_offset` hours
/// east of UTC.
pub(crate) fn local_midnight_jd(jdn: i32, tz_offset: f64) -> f64 {
    f64::from(jdn) - 0.5 - tz_offset / 24.0
}

/// Julian Day Number of the local date containing the instant `jd_ut`.
pub(crate) fn local_jdn(jd_ut: f64, tz_offset: f64) -> i32 {
    (jd_ut + 0.5 + tz_offset / 24.0).floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_close, date, date_time};

    /// Successor computed independently of the JDN arithmetic.
    fn next_day(day: SolarDate) -> SolarDate {
        if Some(day.day) != days_in_month(day.year, day.month) {
            SolarDate {
                day: day.day + 1,
                ..day
            }
        } else if day.month < 12 {
            SolarDate {
                month: day.month + 1,
                day: 1,
                ..day
            }
        } else {
            SolarDate {
                year: day.year + 1,
                month: 1,
                day: 1,
            }
        }
    }

    #[test]
    fn jdn_of_reference_dates() {
        assert_eq!(date(2000, 1, 1).to_jdn(), 2_451_545);
        assert_eq!(date(1970, 1, 1).to_jdn(), 2_440_588);
        assert_eq!(date(1900, 1, 1).to_jdn(), 2_415_021);
        assert_eq!(date(1600, 1, 1).to_jdn(), 2_305_448);
        assert_eq!(date(1582, 10, 15).to_jdn(), 2_299_161);
        assert_eq!(date(2024, 2, 10).to_jdn(), 2_460_351);
    }

    #[test]
    fn jdn_round_trips_every_day_from_1600_to_2400() {
        let mut expected = date(1600, 1, 1);
        for jdn in expected.to_jdn()..=date(2400, 12, 31).to_jdn() {
            assert_eq!(SolarDate::from_jdn(jdn), expected);
            assert_eq!(expected.to_jdn(), jdn);
            expected = next_day(expected);
        }
    }

    #[test]
    fn leap_years_follow_gregorian_rules() {
        for (year, leap) in [
            (1600, true),
            (1700, false),
            (1900, false),
            (2000, true),
            (2023, false),
            (2024, true),
            (2100, false),
            (2400, true),
        ] {
            assert_eq!(is_leap_year(year), leap, "{year}");
        }
        assert_eq!(days_in_month(2024, 2), Some(29));
        assert_eq!(days_in_month(2023, 2), Some(28));
        assert_eq!(days_in_month(2023, 4), Some(30));
        assert_eq!(days_in_month(2023, 13), None);
    }

    #[test]
    fn rejects_dates_and_times_that_do_not_exist() {
        assert_eq!(
            SolarDate::new(2023, 2, 29),
            Err(AstroError::InvalidSolarDate {
                year: 2023,
                month: 2,
                day: 29
            })
        );
        for (month, day) in [(0, 1), (13, 1), (1, 0), (1, 32), (4, 31)] {
            assert!(SolarDate::new(2023, month, day).is_err(), "{month}/{day}");
        }
        assert!(SolarDate::new(2024, 2, 29).is_ok());
        assert!(SolarDateTime::new(2024, 1, 1, 23, 59, 59).is_ok());
        for (hour, minute, second) in [(24, 0, 0), (12, 60, 0), (12, 0, 60)] {
            assert_eq!(
                SolarDateTime::new(2024, 1, 1, hour, minute, second),
                Err(AstroError::InvalidTime {
                    hour,
                    minute,
                    second
                })
            );
        }
        assert!(matches!(
            SolarDateTime::new(2023, 2, 29, 0, 0, 0),
            Err(AstroError::InvalidSolarDate { .. })
        ));
    }

    #[test]
    fn julian_day_of_wall_clock_times() {
        assert_close(
            date_time(2000, 1, 1, 12, 0).to_julian_day(0.0),
            2_451_545.0,
            1e-9,
        );
        // 19:00 in Vietnam is 12:00 UTC.
        assert_close(
            date_time(2000, 1, 1, 19, 0).to_julian_day(7.0),
            2_451_545.0,
            1e-9,
        );
        // Meeus example 7.a: 1957 October 4.81 UT.
        let sputnik = SolarDateTime::new(1957, 10, 4, 19, 26, 24).unwrap();
        assert_close(sputnik.to_julian_day(0.0), 2_436_116.31, 1e-9);
    }

    #[test]
    fn wall_clock_times_round_trip_through_julian_days() {
        let time = SolarDateTime::new(2024, 2, 4, 15, 27, 9).unwrap();
        for tz in [-12.0, -3.5, 0.0, 5.75, 7.0, 8.0, 14.0] {
            assert_eq!(
                SolarDateTime::from_julian_day(time.to_julian_day(tz), tz),
                time,
                "tz {tz}"
            );
        }
    }

    #[test]
    fn rounding_carries_into_the_next_day() {
        let last_second = SolarDateTime::new(2023, 12, 31, 23, 59, 59).unwrap();
        let jd = last_second.to_julian_day(7.0) + 0.6 / SECONDS_PER_DAY;
        assert_eq!(
            SolarDateTime::from_julian_day(jd, 7.0),
            SolarDateTime::new(2024, 1, 1, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn local_dates_depend_on_the_time_zone() {
        // 23:30 UTC on 9 February 2024 is already 10 February in Vietnam.
        let jd = date_time(2024, 2, 9, 23, 30).to_julian_day(0.0);
        assert_eq!(local_jdn(jd, 7.0), date(2024, 2, 10).to_jdn());
        assert_eq!(local_jdn(jd, 0.0), date(2024, 2, 9).to_jdn());
        assert_close(
            local_midnight_jd(2_451_545, 7.0),
            2_451_544.5 - 7.0 / 24.0,
            1e-12,
        );
    }
}
