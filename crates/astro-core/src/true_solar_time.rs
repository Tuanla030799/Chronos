//! True (apparent) solar time — giờ mặt trời thực.
//!
//! True solar time = clock time + longitude offset + equation of time, where the longitude
//! offset is 4 minutes per degree between the birthplace and the clock's standard meridian.
//! Vietnam's clock (UTC+7) follows the 105°E meridian, so Hà Nội (105.85°E) runs about 3.4
//! minutes ahead of the clock in mean solar time, and the equation of time adds between about
//! −14 and +16 minutes over the year.

use crate::astronomy::delta_t::jd_ut_to_jde;
use crate::astronomy::sun::equation_of_time;
use crate::error::{AstroError, check_longitude, check_tz_offset, check_year};
use crate::time::SolarDateTime;

/// Standard meridian of Vietnam's clock time (UTC+7), in degrees east.
pub const VIETNAM_STANDARD_MERIDIAN: f64 = 105.0;

/// Longitude of Hà Nội, degrees east.
pub const HA_NOI: f64 = 105.85;
/// Longitude of Thành phố Hồ Chí Minh, degrees east.
pub const HO_CHI_MINH: f64 = 106.63;
/// Longitude of Đà Nẵng, degrees east.
pub const DA_NANG: f64 = 108.20;
/// Longitude of Điện Biên Phủ, degrees east.
pub const DIEN_BIEN: f64 = 103.01;
/// Longitude of Hải Phòng, degrees east.
pub const HAI_PHONG: f64 = 106.68;
/// Longitude of Huế, degrees east.
pub const HUE: f64 = 107.59;
/// Longitude of Cần Thơ, degrees east.
pub const CAN_THO: f64 = 105.78;

/// Major Vietnamese cities with their longitudes, for birthplace pickers.
pub const MAJOR_CITIES: [(&str, f64); 7] = [
    ("Hà Nội", HA_NOI),
    ("TP. Hồ Chí Minh", HO_CHI_MINH),
    ("Đà Nẵng", DA_NANG),
    ("Điện Biên", DIEN_BIEN),
    ("Hải Phòng", HAI_PHONG),
    ("Huế", HUE),
    ("Cần Thơ", CAN_THO),
];

/// Equation of time in minutes at the instant `jd_ut` (Julian Day, UT): how far true solar time
/// runs ahead of mean solar time. About +16.4 min in early November, −14.2 min in mid-February.
pub fn equation_of_time_minutes(jd_ut: f64) -> f64 {
    equation_of_time(jd_ut_to_jde(jd_ut))
}

/// Minutes that mean solar time at `local_longitude` runs ahead of the clock of a zone whose
/// standard meridian is `standard_meridian` (both in degrees east): 4 minutes per degree.
pub fn longitude_offset_minutes(local_longitude: f64, standard_meridian: f64) -> f64 {
    (local_longitude - standard_meridian) * 4.0
}

/// Converts the wall-clock time `clock_time` of the zone whose standard meridian is
/// `standard_meridian` (105.0 for Vietnam) to true solar time at `local_longitude`, rounded to
/// the second.
///
/// Fails for an invalid date or time, a year outside the supported range, a longitude outside
/// −180…+180°, or a standard meridian that is not that of a zone within UTC−12…+14.
pub fn to_true_solar_time(
    clock_time: SolarDateTime,
    local_longitude: f64,
    standard_meridian: f64,
) -> Result<SolarDateTime, AstroError> {
    clock_time.validate()?;
    check_year(clock_time.year)?;
    check_longitude(local_longitude)?;
    let tz_offset = standard_meridian / 15.0;
    check_tz_offset(tz_offset)?;
    Ok(apparent_solar_time(
        clock_time.to_julian_day(tz_offset),
        local_longitude,
    ))
}

/// True solar time at `longitude` degrees east for the instant `jd_ut`.
pub(crate) fn apparent_solar_time(jd_ut: f64, longitude: f64) -> SolarDateTime {
    let equation_of_time_days = equation_of_time_minutes(jd_ut) / 1440.0;
    SolarDateTime::from_julian_day(jd_ut + longitude / 360.0 + equation_of_time_days, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canchi::Branch;
    use crate::test_support::{assert_close, date, date_time, utc};

    #[test]
    fn longitude_offset_is_four_minutes_per_degree() {
        assert_close(
            longitude_offset_minutes(HO_CHI_MINH, VIETNAM_STANDARD_MERIDIAN),
            6.52,
            1e-9,
        );
        assert_close(longitude_offset_minutes(DIEN_BIEN, 105.0), -7.96, 1e-9);
        assert_eq!(longitude_offset_minutes(105.0, 105.0), 0.0);
    }

    #[test]
    fn equation_of_time_through_the_year() {
        let november = equation_of_time_minutes(utc(2024, 11, 3, 12, 0));
        let february = equation_of_time_minutes(utc(2024, 2, 11, 12, 0));
        assert!((16.0..16.7).contains(&november), "{november}");
        assert!((-14.5..-13.9).contains(&february), "{february}");
    }

    #[test]
    fn on_the_standard_meridian_only_the_equation_of_time_remains() {
        let solar = to_true_solar_time(date_time(2024, 11, 3, 12, 0), 105.0, 105.0).unwrap();
        assert_eq!((solar.hour, solar.minute), (12, 16));
    }

    #[test]
    fn a_22_55_birth_in_ho_chi_minh_city_depends_on_the_date() {
        let clock = |month, day| date_time(2024, month, day, 22, 55);
        // Early November: +6.5 min for longitude and +16.4 min equation of time → 23:17:57, early Tý.
        let november =
            to_true_solar_time(clock(11, 3), HO_CHI_MINH, VIETNAM_STANDARD_MERIDIAN).unwrap();
        assert_eq!((november.hour, november.minute), (23, 17));
        assert_eq!(Branch::from_hour(november.hour), Branch::Ty);
        // Mid-February: +6.5 − 14.2 min → 22:47, still Hợi.
        let february =
            to_true_solar_time(clock(2, 11), HO_CHI_MINH, VIETNAM_STANDARD_MERIDIAN).unwrap();
        assert_eq!((february.hour, february.minute), (22, 47));
        assert_eq!(Branch::from_hour(february.hour), Branch::Hoi);
    }

    #[test]
    fn can_fall_on_the_previous_date() {
        // Điện Biên is 8 min behind the clock and mid-February adds about −14 min.
        let solar = to_true_solar_time(date_time(2024, 2, 11, 0, 5), DIEN_BIEN, 105.0).unwrap();
        assert_eq!(solar.date(), date(2024, 2, 10));
        assert_eq!(solar.hour, 23);
    }

    #[test]
    fn validates_inputs() {
        let clock = date_time(2024, 1, 1, 0, 0);
        assert!(matches!(
            to_true_solar_time(clock, 181.0, 105.0),
            Err(AstroError::InvalidLongitude { .. })
        ));
        assert!(matches!(
            to_true_solar_time(clock, HA_NOI, 300.0),
            Err(AstroError::InvalidTimeZone { .. })
        ));
        assert!(matches!(
            to_true_solar_time(
                SolarDateTime {
                    minute: 61,
                    ..clock
                },
                HA_NOI,
                105.0
            ),
            Err(AstroError::InvalidTime { .. })
        ));
    }
}
