//! Everything a calendar cell shows for one Gregorian date, in a single call.

use crate::canchi::{CanChi, day_can_chi, month_can_chi, year_can_chi};
use crate::error::AstroError;
use crate::lunar_calendar::{LunarDate, solar_to_lunar};
use crate::solar_terms::{SolarTerm, solar_term_at};
use crate::time::{SolarDate, local_midnight_jd};

/// Calendar data for one Gregorian date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DayInfo {
    /// The Gregorian date.
    pub solar: SolarDate,
    /// The same day in the lunar calendar.
    pub lunar: LunarDate,
    /// Julian Day Number of the date.
    pub julian_day_number: i32,
    /// Can Chi of the lunar year.
    pub year_can_chi: CanChi,
    /// Can Chi of the lunar month (a leap month shares that of the month it repeats).
    pub month_can_chi: CanChi,
    /// Can Chi of the day.
    pub day_can_chi: CanChi,
    /// Solar term in effect at the end of the day.
    pub solar_term: SolarTerm,
    /// Whether `solar_term` begins during this day.
    pub solar_term_begins: bool,
}

/// Calendar data for `date` in a zone `tz_offset` hours east of UTC.
pub fn day_info(date: SolarDate, tz_offset: f64) -> Result<DayInfo, AstroError> {
    let lunar = solar_to_lunar(date.day, date.month, date.year, tz_offset)?;
    let jdn = date.to_jdn();
    let term_at_midnight_starting = |day: i32| solar_term_at(local_midnight_jd(day, tz_offset));
    let solar_term = term_at_midnight_starting(jdn + 1);
    Ok(DayInfo {
        solar: date,
        lunar,
        julian_day_number: jdn,
        year_can_chi: year_can_chi(lunar.year),
        month_can_chi: month_can_chi(lunar.year, lunar.month),
        day_can_chi: day_can_chi(jdn),
        solar_term,
        solar_term_begins: term_at_midnight_starting(jdn) != solar_term,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{VN, date};

    #[test]
    fn tet_giap_thin() {
        let info = day_info(date(2024, 2, 10), VN).unwrap();
        assert_eq!(
            info.lunar,
            LunarDate {
                year: 2024,
                month: 1,
                day: 1,
                is_leap_month: false
            }
        );
        assert_eq!(info.julian_day_number, 2_460_351);
        assert_eq!(info.year_can_chi.to_string(), "Giáp Thìn");
        assert_eq!(info.month_can_chi.to_string(), "Bính Dần");
        assert_eq!(info.day_can_chi.to_string(), "Giáp Thìn");
        assert_eq!(
            (info.solar_term, info.solar_term_begins),
            (SolarTerm::LapXuan, false)
        );
    }

    #[test]
    fn flags_the_day_a_term_begins() {
        let eve = day_info(date(2024, 2, 3), VN).unwrap();
        assert_eq!(
            (eve.solar_term, eve.solar_term_begins),
            (SolarTerm::DaiHan, false)
        );
        let lap_xuan = day_info(date(2024, 2, 4), VN).unwrap();
        assert_eq!(
            (lap_xuan.solar_term, lap_xuan.solar_term_begins),
            (SolarTerm::LapXuan, true)
        );
        let solstice = day_info(date(2024, 12, 21), VN).unwrap();
        assert_eq!(
            (solstice.solar_term, solstice.solar_term_begins),
            (SolarTerm::DongChi, true)
        );
    }

    #[test]
    fn a_leap_month_shares_the_can_chi_of_the_month_it_repeats() {
        let regular = day_info(date(2025, 6, 25), VN).unwrap();
        let leap = day_info(date(2025, 7, 25), VN).unwrap();
        assert!(!regular.lunar.is_leap_month && leap.lunar.is_leap_month);
        assert_eq!(leap.month_can_chi, regular.month_can_chi);
        assert_eq!(leap.month_can_chi.to_string(), "Quý Mùi");
    }

    #[test]
    fn propagates_validation_errors() {
        assert!(
            day_info(
                SolarDate {
                    year: 2024,
                    month: 2,
                    day: 30
                },
                VN
            )
            .is_err()
        );
        assert!(day_info(date(2024, 2, 10), -13.0).is_err());
    }
}
