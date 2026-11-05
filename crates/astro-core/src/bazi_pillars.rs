//! The Four Pillars (Tứ Trụ / Bát Tự): the Can Chi of the year, month, day and hour of a birth.
//!
//! - With [`YearMonthBoundary::SolarTerms`] (the default) the **year** pillar changes at Lập Xuân
//!   (Sun at 315°), not at Tết, and the **month** pillar changes at each sectional term (Tiết):
//!   Lập Xuân opens the Dần month, Kinh Trập the Mão month, and so on every 30° of solar
//!   longitude. Its stem follows the year stem by the same formula as lunar months (the Ngũ Hổ
//!   Độn rule).
//! - With [`YearMonthBoundary::LunarNewYear`] they follow the lunar calendar of the clock date
//!   instead: the year changes at Tết and the month with each lunar month.
//! - The **day** pillar is the 60-day cycle of the local date.
//! - The **hour** pillar is the double-hour branch with its stem derived from the day stem
//!   (the Ngũ Thử Độn rule).
//!
//! With solar terms, year and month compare the birth *instant* with the terms, so they depend
//! neither on the time zone nor on true solar time. Day and hour use local reckoning: the clock time,
//! or true solar time when the birthplace longitude is given.

use crate::DEFAULT_TZ_OFFSET;
use crate::astronomy::get_sun_longitude_deg;
use crate::canchi::{Branch, CanChi, day_can_chi, hour_can_chi, month_can_chi, year_can_chi};
use crate::error::{AstroError, check_longitude, check_tz_offset, check_year};
use crate::lunar_calendar::solar_to_lunar;
use crate::solar_terms::solar_month_of_longitude;
use crate::time::SolarDateTime;
use crate::true_solar_time::apparent_solar_time;

/// How a birth during the Tý double-hour (23:00–00:59) is assigned to a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RatHourConvention {
    /// The day begins with the Tý hour at 23:00: a birth at 23:xx takes the next day's day
    /// pillar and that day's Tý hour pillar.
    NewDayAtRatHour,
    /// The day changes at midnight. A birth during giờ Tý Sớm, 23:00–23:59 (晚子時 in Chinese
    /// texts), keeps the Can Chi of that day but starts its hour pillar from the next day's
    /// stem; a birth during giờ Tý Muộn, 00:00–00:59, belongs wholly to the new day.
    #[default]
    SplitAtMidnight,
}

/// Where the year and month pillars change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum YearMonthBoundary {
    /// At the solar terms: Lập Xuân for the year, each Tiết for the month.
    #[default]
    SolarTerms,
    /// At Tết for the year and at each lunar month for the month; a leap month carries the Can
    /// Chi of the month it repeats.
    LunarNewYear,
}

/// Settings for [`four_pillars`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FourPillarsOptions {
    /// Offset of the birth clock time from UTC, in hours (7.0 for Vietnam).
    pub tz_offset: f64,
    /// Birthplace longitude in degrees east. When present, the day and hour pillars use true
    /// solar time there instead of the clock time.
    pub longitude: Option<f64>,
    /// Treatment of the Tý double-hour.
    pub rat_hour: RatHourConvention,
    /// Where the year and month pillars change.
    pub boundary: YearMonthBoundary,
}

impl Default for FourPillarsOptions {
    fn default() -> Self {
        Self {
            tz_offset: DEFAULT_TZ_OFFSET,
            longitude: None,
            rat_hour: RatHourConvention::default(),
            boundary: YearMonthBoundary::default(),
        }
    }
}

/// The Four Pillars of a birth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FourPillars {
    /// Year pillar (trụ năm).
    pub year: CanChi,
    /// Month pillar (trụ tháng).
    pub month: CanChi,
    /// Day pillar (trụ ngày).
    pub day: CanChi,
    /// Hour pillar (trụ giờ).
    pub hour: CanChi,
    /// Local time used for the day and hour pillars: the clock time, or true solar time when
    /// a longitude was given.
    pub effective_time: SolarDateTime,
}

/// Computes the Four Pillars of a birth at wall-clock time `birth`.
pub fn four_pillars(
    birth: SolarDateTime,
    options: &FourPillarsOptions,
) -> Result<FourPillars, AstroError> {
    birth.validate()?;
    check_year(birth.year)?;
    check_tz_offset(options.tz_offset)?;
    if let Some(longitude) = options.longitude {
        check_longitude(longitude)?;
    }
    let jd_ut = birth.to_julian_day(options.tz_offset);
    let (pillar_year, pillar_month) = match options.boundary {
        YearMonthBoundary::SolarTerms => {
            let solar_month = solar_month_of_longitude(get_sun_longitude_deg(jd_ut));
            // The Tý and Sửu months straddle 1 January; in January and early February they
            // still belong to the pillar year that began at the previous Lập Xuân.
            let year = if solar_month >= 11 && birth.month <= 2 {
                birth.year - 1
            } else {
                birth.year
            };
            (year, solar_month)
        }
        YearMonthBoundary::LunarNewYear => {
            let lunar = solar_to_lunar(birth.day, birth.month, birth.year, options.tz_offset)?;
            (lunar.year, lunar.month)
        }
    };

    let effective_time = match options.longitude {
        Some(longitude) => apparent_solar_time(jd_ut, longitude),
        None => birth,
    };
    let date_jdn = effective_time.date().to_jdn();
    let late_rat_hour = effective_time.hour >= 23;
    let day_jdn = match options.rat_hour {
        RatHourConvention::NewDayAtRatHour if late_rat_hour => date_jdn + 1,
        _ => date_jdn,
    };
    // From 23:00 the hour pillar is the next day's Tý hour under either convention.
    let hour_day_jdn = if late_rat_hour {
        date_jdn + 1
    } else {
        date_jdn
    };
    let hour = hour_can_chi(
        day_can_chi(hour_day_jdn).stem,
        Branch::from_hour(effective_time.hour),
    );

    Ok(FourPillars {
        year: year_can_chi(pillar_year),
        month: month_can_chi(pillar_year, pillar_month),
        day: day_can_chi(day_jdn),
        hour,
        effective_time,
    })
}

#[cfg(test)]
mod tests {
    use super::RatHourConvention::{NewDayAtRatHour, SplitAtMidnight};
    use super::*;
    use crate::canchi::Branch::{Dan, Mao, Ngo, Suu, Thin, Ti, Ty};
    use crate::canchi::Stem::{self, At, Binh, Dinh, Giap, Mau, Quy};
    use crate::test_support::{VN, date, date_time};

    fn pair(stem: Stem, branch: Branch) -> CanChi {
        CanChi::new(stem, branch).expect("test pairs are valid")
    }

    fn by_clock(rat_hour: RatHourConvention) -> FourPillarsOptions {
        FourPillarsOptions {
            tz_offset: VN,
            rat_hour,
            ..FourPillarsOptions::default()
        }
    }

    fn pillars(birth: SolarDateTime) -> FourPillars {
        four_pillars(birth, &by_clock(NewDayAtRatHour)).unwrap()
    }

    #[test]
    fn morning_of_tet_giap_thin() {
        let chart = pillars(date_time(2024, 2, 10, 8, 0));
        assert_eq!(chart.year, pair(Giap, Thin));
        assert_eq!(chart.month, pair(Binh, Dan));
        assert_eq!(chart.day, pair(Giap, Thin));
        assert_eq!(chart.hour, pair(Mau, Thin));
        assert_eq!(chart.effective_time, date_time(2024, 2, 10, 8, 0));
    }

    #[test]
    fn year_and_month_change_at_lap_xuan_not_at_tet() {
        // Lập Xuân 2024 fell at about 15:27 on 4 February in Vietnam.
        let before = pillars(date_time(2024, 2, 4, 14, 30));
        assert_eq!((before.year, before.month), (pair(Quy, Mao), pair(At, Suu)));
        let after = pillars(date_time(2024, 2, 4, 16, 30));
        assert_eq!(
            (after.year, after.month),
            (pair(Giap, Thin), pair(Binh, Dan))
        );
        let mid_january = pillars(date_time(2024, 1, 15, 12, 0));
        assert_eq!(
            (mid_january.year, mid_january.month),
            (pair(Quy, Mao), pair(At, Suu))
        );
        let late_december = pillars(date_time(2023, 12, 25, 12, 0));
        assert_eq!(
            (late_december.year, late_december.month),
            (pair(Quy, Mao), pair(Giap, Ty))
        );
    }

    #[test]
    fn month_changes_at_kinh_trap() {
        // Kinh Trập 2024 fell at about 09:23 on 5 March in Vietnam.
        assert_eq!(pillars(date_time(2024, 3, 5, 8, 0)).month, pair(Binh, Dan));
        assert_eq!(pillars(date_time(2024, 3, 5, 11, 0)).month, pair(Dinh, Mao));
    }

    #[test]
    fn rat_hour_conventions() {
        // 10 Feb 2024 is a Giáp Thìn day; 11 Feb is Ất Tỵ, whose Tý hour is Bính Tý.
        let late_rat = date_time(2024, 2, 10, 23, 30);
        let new_day = four_pillars(late_rat, &by_clock(NewDayAtRatHour)).unwrap();
        assert_eq!((new_day.day, new_day.hour), (pair(At, Ti), pair(Binh, Ty)));
        let split = four_pillars(late_rat, &by_clock(SplitAtMidnight)).unwrap();
        assert_eq!((split.day, split.hour), (pair(Giap, Thin), pair(Binh, Ty)));

        for convention in [NewDayAtRatHour, SplitAtMidnight] {
            let early_rat =
                four_pillars(date_time(2024, 2, 11, 0, 30), &by_clock(convention)).unwrap();
            assert_eq!(
                (early_rat.day, early_rat.hour),
                (pair(At, Ti), pair(Binh, Ty))
            );
        }
        assert_eq!(pillars(date_time(2024, 2, 10, 0, 30)).hour, pair(Giap, Ty));
    }

    #[test]
    fn true_solar_time_can_change_the_hour() {
        // In early November the Sun is ~16.4 min ahead of the clock; Hà Nội adds ~3.4 min.
        let clock = date_time(2024, 11, 3, 10, 50);
        assert_eq!(pillars(clock).hour.branch, Ti);
        let options = FourPillarsOptions {
            longitude: Some(105.85),
            ..by_clock(NewDayAtRatHour)
        };
        let chart = four_pillars(clock, &options).unwrap();
        assert_eq!(chart.hour.branch, Ngo);
        assert_eq!(chart.effective_time.hour, 11);
        assert!(
            (9..=10).contains(&chart.effective_time.minute),
            "{:?}",
            chart.effective_time
        );
    }

    #[test]
    fn true_solar_time_can_change_the_day() {
        // 00:05 at Điện Biên Phủ (103°E) in mid-February is about 23:43 the previous evening.
        let clock = date_time(2024, 2, 11, 0, 5);
        let at = |rat_hour| FourPillarsOptions {
            tz_offset: VN,
            longitude: Some(103.0),
            rat_hour,
            boundary: YearMonthBoundary::SolarTerms,
        };
        let split = four_pillars(clock, &at(SplitAtMidnight)).unwrap();
        assert_eq!(split.effective_time.date(), date(2024, 2, 10));
        assert_eq!((split.day, split.hour), (pair(Giap, Thin), pair(Binh, Ty)));
        let new_day = four_pillars(clock, &at(NewDayAtRatHour)).unwrap();
        assert_eq!((new_day.day, new_day.hour), (pair(At, Ti), pair(Binh, Ty)));
    }

    #[test]
    fn defaults_follow_the_project_decisions() {
        let options = FourPillarsOptions::default();
        assert_eq!(options.tz_offset, VN);
        assert_eq!(options.rat_hour, SplitAtMidnight);
        assert_eq!(options.boundary, YearMonthBoundary::SolarTerms);
    }

    #[test]
    fn ty_som_keeps_the_day_but_starts_the_hour_from_the_next_day() {
        let chart = four_pillars(
            date_time(2024, 2, 10, 23, 30),
            &FourPillarsOptions::default(),
        )
        .unwrap();
        assert_eq!(chart.day, pair(Giap, Thin));
        assert_eq!(chart.hour, pair(Binh, Ty));
    }

    #[test]
    fn lunar_new_year_boundary_follows_tet_and_lunar_months() {
        let lunar = FourPillarsOptions {
            boundary: YearMonthBoundary::LunarNewYear,
            ..FourPillarsOptions::default()
        };
        // 5 February 2024 is after Lập Xuân but before Tết: 26/12 of Quý Mão.
        let birth = date_time(2024, 2, 5, 10, 0);
        let by_terms = pillars(birth);
        assert_eq!(
            (by_terms.year, by_terms.month),
            (pair(Giap, Thin), pair(Binh, Dan))
        );
        let by_tet = four_pillars(birth, &lunar).unwrap();
        assert_eq!((by_tet.year, by_tet.month), (pair(Quy, Mao), pair(At, Suu)));
        assert_eq!(by_tet.day, by_terms.day);
        // From Tết the year changes; the leap 6th month of 2025 carries the Can Chi of month 6.
        let tet = four_pillars(date_time(2024, 2, 10, 8, 0), &lunar).unwrap();
        assert_eq!((tet.year, tet.month), (pair(Giap, Thin), pair(Binh, Dan)));
        let regular = four_pillars(date_time(2025, 6, 25, 8, 0), &lunar).unwrap();
        let leap = four_pillars(date_time(2025, 7, 25, 8, 0), &lunar).unwrap();
        assert_eq!(regular.month, leap.month);
    }

    #[test]
    fn validates_inputs() {
        let birth = date_time(2024, 1, 1, 0, 0);
        let bad_zone = FourPillarsOptions {
            tz_offset: 99.0,
            ..by_clock(NewDayAtRatHour)
        };
        assert!(matches!(
            four_pillars(birth, &bad_zone),
            Err(AstroError::InvalidTimeZone { .. })
        ));
        let bad_longitude = FourPillarsOptions {
            longitude: Some(200.0),
            ..by_clock(NewDayAtRatHour)
        };
        assert!(matches!(
            four_pillars(birth, &bad_longitude),
            Err(AstroError::InvalidLongitude { .. })
        ));
        let bad_time = SolarDateTime { hour: 24, ..birth };
        assert!(matches!(
            four_pillars(bad_time, &by_clock(NewDayAtRatHour)),
            Err(AstroError::InvalidTime { .. })
        ));
        let too_early = SolarDateTime {
            year: 1500,
            ..birth
        };
        assert_eq!(
            four_pillars(too_early, &by_clock(NewDayAtRatHour)),
            Err(AstroError::YearOutOfRange { year: 1500 })
        );
    }
}
