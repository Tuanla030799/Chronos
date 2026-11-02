//! True (apparent) solar time — giờ mặt trời thực.
//!
//! Apparent solar time at longitude λ is Universal Time + λ/15 hours + the equation of time.
//! Vietnam's clock follows the 105°E meridian, so Hà Nội (105.85°E) is about 3.4 minutes
//! ahead of the clock in mean solar time, and the equation of time adds between −14 and +16
//! minutes over the year.

use crate::delta_t::jd_ut_to_jde;
use crate::error::{AstroError, check_longitude, check_tz_offset, check_year};
use crate::sun::equation_of_time;
use crate::time::SolarDateTime;

/// Converts a wall-clock time in a zone `tz_offset` hours east of UTC to true solar time at
/// `longitude` degrees east, rounded to the second.
pub fn true_solar_time(
    clock: SolarDateTime,
    tz_offset: f64,
    longitude: f64,
) -> Result<SolarDateTime, AstroError> {
    clock.validate()?;
    check_year(clock.year)?;
    check_tz_offset(tz_offset)?;
    check_longitude(longitude)?;
    Ok(apparent_solar_time(
        clock.to_julian_day(tz_offset),
        longitude,
    ))
}

/// Apparent solar time at `longitude` degrees east for the instant `jd_ut`.
pub(crate) fn apparent_solar_time(jd_ut: f64, longitude: f64) -> SolarDateTime {
    let equation_of_time_days = equation_of_time(jd_ut_to_jde(jd_ut)) / 1440.0;
    SolarDateTime::from_julian_day(jd_ut + longitude / 360.0 + equation_of_time_days, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{VN, assert_close, date, date_time};

    #[test]
    fn on_the_standard_meridian_only_the_equation_of_time_remains() {
        // 105°E is the meridian of UTC+7; in early November the Sun is about 16.4 min ahead.
        let solar = true_solar_time(date_time(2024, 11, 3, 12, 0), VN, 105.0).unwrap();
        assert_eq!((solar.hour, solar.minute), (12, 16));
    }

    #[test]
    fn each_degree_of_longitude_is_four_minutes() {
        let clock = date_time(2024, 6, 13, 12, 0);
        let meridian = true_solar_time(clock, VN, 105.0)
            .unwrap()
            .to_julian_day(0.0);
        let ho_chi_minh_city = true_solar_time(clock, VN, 106.7)
            .unwrap()
            .to_julian_day(0.0);
        // Both results are rounded to the second.
        assert_close(
            (ho_chi_minh_city - meridian) * 1440.0,
            1.7 * 4.0,
            1.0 / 60.0 + 1e-6,
        );
    }

    #[test]
    fn can_fall_on_the_previous_date() {
        // Điện Biên Phủ (103°E) is 8 min behind the clock and mid-February adds about −14 min.
        let solar = true_solar_time(date_time(2024, 2, 11, 0, 5), VN, 103.0).unwrap();
        assert_eq!(solar.date(), date(2024, 2, 10));
        assert_eq!(solar.hour, 23);
    }

    #[test]
    fn validates_inputs() {
        let clock = date_time(2024, 1, 1, 0, 0);
        assert!(matches!(
            true_solar_time(clock, VN, 181.0),
            Err(AstroError::InvalidLongitude { .. })
        ));
        assert!(matches!(
            true_solar_time(clock, f64::NAN, 105.0),
            Err(AstroError::InvalidTimeZone { .. })
        ));
        assert!(matches!(
            true_solar_time(
                SolarDateTime {
                    minute: 61,
                    ..clock
                },
                VN,
                105.0
            ),
            Err(AstroError::InvalidTime { .. })
        ));
    }
}
