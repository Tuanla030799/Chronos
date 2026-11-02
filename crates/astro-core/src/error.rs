//! Error type and the input checks shared by the public API.

use core::fmt;

use crate::{MAX_YEAR, MIN_YEAR};

/// Why an astro-core function rejected its input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AstroError {
    /// The Gregorian date does not exist, e.g. 2023-02-29.
    InvalidSolarDate {
        /// Year.
        year: i32,
        /// Month, expected 1–12.
        month: u8,
        /// Day of the month.
        day: u8,
    },
    /// The time of day is outside 00:00:00–23:59:59.
    InvalidTime {
        /// Hour, expected 0–23.
        hour: u8,
        /// Minute, expected 0–59.
        minute: u8,
        /// Second, expected 0–59.
        second: u8,
    },
    /// The lunar date does not exist: month outside 1–12, day beyond the month's length, or
    /// a leap flag on a month that is not the leap month of that year.
    InvalidLunarDate {
        /// Lunar year.
        year: i32,
        /// Month number, expected 1–12.
        month: u8,
        /// Day of the month.
        day: u8,
        /// Whether the leap (nhuận) month was requested.
        is_leap_month: bool,
    },
    /// The timezone offset is not a finite number of hours within −12…+14.
    InvalidTimeZone {
        /// The rejected offset, in hours.
        tz_offset: f64,
    },
    /// The longitude is not a finite number of degrees within −180…+180.
    InvalidLongitude {
        /// The rejected longitude, in degrees east.
        longitude: f64,
    },
    /// The year lies outside [`MIN_YEAR`]…[`MAX_YEAR`].
    YearOutOfRange {
        /// The rejected year.
        year: i32,
    },
}

impl fmt::Display for AstroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidSolarDate { year, month, day } => {
                write!(
                    f,
                    "{year:04}-{month:02}-{day:02} is not a valid Gregorian date"
                )
            }
            Self::InvalidTime {
                hour,
                minute,
                second,
            } => {
                write!(
                    f,
                    "{hour:02}:{minute:02}:{second:02} is not a valid time of day"
                )
            }
            Self::InvalidLunarDate {
                year,
                month,
                day,
                is_leap_month,
            } => {
                let kind = if is_leap_month { "leap month" } else { "month" };
                write!(f, "lunar year {year} has no day {day} in {kind} {month}")
            }
            Self::InvalidTimeZone { tz_offset } => {
                write!(f, "timezone offset {tz_offset} h is outside -12..=14 h")
            }
            Self::InvalidLongitude { longitude } => {
                write!(f, "longitude {longitude}° is outside -180..=180°")
            }
            Self::YearOutOfRange { year } => {
                write!(
                    f,
                    "year {year} is outside the supported range {MIN_YEAR}..={MAX_YEAR}"
                )
            }
        }
    }
}

impl std::error::Error for AstroError {}

pub(crate) fn check_year(year: i32) -> Result<(), AstroError> {
    if (MIN_YEAR..=MAX_YEAR).contains(&year) {
        Ok(())
    } else {
        Err(AstroError::YearOutOfRange { year })
    }
}

pub(crate) fn check_tz_offset(tz_offset: f64) -> Result<(), AstroError> {
    if (-12.0..=14.0).contains(&tz_offset) {
        Ok(())
    } else {
        Err(AstroError::InvalidTimeZone { tz_offset })
    }
}

pub(crate) fn check_longitude(longitude: f64) -> Result<(), AstroError> {
    if (-180.0..=180.0).contains(&longitude) {
        Ok(())
    } else {
        Err(AstroError::InvalidLongitude { longitude })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_values_inside_the_ranges() {
        assert!(check_year(MIN_YEAR).is_ok() && check_year(MAX_YEAR).is_ok());
        assert!(check_tz_offset(-12.0).is_ok() && check_tz_offset(14.0).is_ok());
        assert!(check_tz_offset(5.75).is_ok());
        assert!(check_longitude(-180.0).is_ok() && check_longitude(105.85).is_ok());
    }

    #[test]
    fn rejects_values_outside_the_ranges_and_non_finite_numbers() {
        assert_eq!(
            check_year(MIN_YEAR - 1),
            Err(AstroError::YearOutOfRange { year: MIN_YEAR - 1 })
        );
        assert_eq!(
            check_year(MAX_YEAR + 1),
            Err(AstroError::YearOutOfRange { year: MAX_YEAR + 1 })
        );
        assert_eq!(
            check_tz_offset(14.5),
            Err(AstroError::InvalidTimeZone { tz_offset: 14.5 })
        );
        assert!(check_tz_offset(f64::NAN).is_err());
        assert!(check_tz_offset(f64::INFINITY).is_err());
        assert!(check_longitude(180.5).is_err());
        assert!(check_longitude(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn messages_name_the_offending_input() {
        assert_eq!(
            AstroError::InvalidSolarDate {
                year: 2023,
                month: 2,
                day: 29
            }
            .to_string(),
            "2023-02-29 is not a valid Gregorian date"
        );
        assert_eq!(
            AstroError::InvalidLunarDate {
                year: 2024,
                month: 4,
                day: 1,
                is_leap_month: true
            }
            .to_string(),
            "lunar year 2024 has no day 1 in leap month 4"
        );
        assert_eq!(
            AstroError::YearOutOfRange { year: 3000 }.to_string(),
            "year 3000 is outside the supported range 1600..=2400"
        );
    }
}
