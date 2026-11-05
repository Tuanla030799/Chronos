//! Vietnamese lunisolar calendar (Âm lịch).
//!
//! Month rules from Hồ Ngọc Đức's algorithm (the rules of the Chinese calendar since 1645),
//! evaluated on local civil days at `tz_offset`:
//!
//! 1. A lunar month begins on the local day of a true new moon (ngày Sóc).
//! 2. Month 11 is the month containing the December solstice (Đông Chí).
//! 3. When 13 months separate one month 11 from the next, the first of them after month 11
//!    that contains no principal term (Trung khí) is the leap month (tháng nhuận); it repeats
//!    the number of the month before it.
//! 4. The lunar year changes on day 1 of month 1 (Tết) and takes the number of the Gregorian
//!    year in which that Tết falls.
//!
//! The steps follow the original `getLunarMonth11` and `getLeapMonthOffset` on top of
//! [`get_new_moon_day`] and [`get_solar_term_index`], whose new moons and solar longitudes come
//! from Meeus' full algorithms rather than the abridged series of the original article. Day
//! boundaries therefore stay right even when a new moon falls within minutes of local midnight.

use crate::astronomy::moon_phase::lunation_on_or_before;
use crate::astronomy::{get_new_moon_day, get_solar_term_index, jd_from_date};
use crate::error::{AstroError, check_tz_offset, check_year};
use crate::time::SolarDate;
use crate::{MAX_YEAR, MIN_YEAR};

/// A date in the Vietnamese lunisolar calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LunarDate {
    /// Lunar year, numbered like the Gregorian year in which its Tết falls.
    pub year: i32,
    /// Month number, 1–12 (1 is tháng Giêng, 12 is tháng Chạp).
    pub month: u8,
    /// Day of the month, 1–30.
    pub day: u8,
    /// Whether this is the leap (nhuận) month that repeats `month`.
    pub is_leap_month: bool,
}

/// One month of a lunar year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LunarMonth {
    /// Month number, 1–12.
    pub month: u8,
    /// Whether this is the leap month repeating `month`.
    pub is_leap: bool,
    /// Gregorian date of its first day (mùng 1).
    pub first_day: SolarDate,
    /// 29 days (tháng thiếu) or 30 days (tháng đủ).
    pub length: u8,
}

/// The months of one lunar year, from Tết to the eve of the next Tết.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LunarYear {
    /// The lunar year.
    pub year: i32,
    /// Number of the month that is repeated as a leap month, if any.
    pub leap_month: Option<u8>,
    /// The 12 or 13 months in order.
    pub months: Vec<LunarMonth>,
}

/// Converts the Gregorian date `day`/`month`/`year` to the lunar calendar of a zone
/// `tz_offset` hours east of UTC (7.0 for Vietnam).
///
/// Fails for a date that does not exist, an offset outside −12…+14 h, or a date outside the
/// supported lunar years.
pub fn solar_to_lunar(
    day: u8,
    month: u8,
    year: i32,
    tz_offset: f64,
) -> Result<LunarDate, AstroError> {
    let date = SolarDate::new(year, month, day)?;
    check_tz_offset(tz_offset)?;
    // A lunar year ends early in the next Gregorian year, so dates in MAX_YEAR + 1 can still
    // belong to a supported lunar year; the result is checked below.
    if !(MIN_YEAR..=MAX_YEAR + 1).contains(&year) {
        return Err(AstroError::YearOutOfRange { year });
    }
    let jdn = date.to_jdn();
    let span = month_containing(jdn, year, tz_offset);
    check_year(span.year)?;
    Ok(LunarDate {
        year: span.year,
        month: span.month,
        day: (jdn - span.first_jdn + 1) as u8,
        is_leap_month: span.is_leap,
    })
}

/// Converts a lunar date of a zone `tz_offset` hours east of UTC to the Gregorian calendar.
///
/// Fails for a month outside 1–12, a day beyond the month's length, a leap flag on a month that
/// is not the leap month of that year, an invalid offset, or a year outside the supported range.
pub fn lunar_to_solar(lunar: LunarDate, tz_offset: f64) -> Result<SolarDate, AstroError> {
    check_year(lunar.year)?;
    check_tz_offset(tz_offset)?;
    let invalid = AstroError::InvalidLunarDate {
        year: lunar.year,
        month: lunar.month,
        day: lunar.day,
        is_leap_month: lunar.is_leap_month,
    };
    if lunar.day == 0 {
        return Err(invalid);
    }
    month_span(lunar, tz_offset)
        .filter(|span| i32::from(lunar.day) <= span.length())
        .map(|span| SolarDate::from_jdn(span.first_jdn + i32::from(lunar.day) - 1))
        .ok_or(invalid)
}

/// Number of days (29 or 30) of the lunar month containing `lunar`, in a zone `tz_offset` hours
/// east of UTC. The day of `lunar` is not used.
pub fn lunar_month_length(lunar: LunarDate, tz_offset: f64) -> Result<u8, AstroError> {
    check_year(lunar.year)?;
    check_tz_offset(tz_offset)?;
    month_span(lunar, tz_offset)
        .map(|span| span.length() as u8)
        .ok_or(AstroError::InvalidLunarDate {
            year: lunar.year,
            month: lunar.month,
            day: lunar.day,
            is_leap_month: lunar.is_leap_month,
        })
}

/// The month of `lunar`, or `None` if that month does not exist.
fn month_span(lunar: LunarDate, tz_offset: f64) -> Option<MonthSpan> {
    if !(1..=12).contains(&lunar.month) {
        return None;
    }
    // Months 11 and 12 (and a leap 11 or 12) follow the solstice of their own year; months
    // 1–10 follow the solstice of the previous year.
    let solstice_year = if lunar.month >= 11 {
        lunar.year
    } else {
        lunar.year - 1
    };
    months_after_solstice(solstice_year, tz_offset)
        .into_iter()
        .find(|span| {
            span.year == lunar.year
                && span.month == lunar.month
                && span.is_leap == lunar.is_leap_month
        })
}

/// All months of lunar year `year` in a zone `tz_offset` hours east of UTC.
pub fn lunar_year(year: i32, tz_offset: f64) -> Result<LunarYear, AstroError> {
    check_year(year)?;
    check_tz_offset(tz_offset)?;
    let previous_month_11 = month_11_lunation(year - 1, tz_offset);
    let month_11 = month_11_lunation(year, tz_offset);
    let next_month_11 = month_11_lunation(year + 1, tz_offset);
    let months: Vec<LunarMonth> = month_spans(year - 1, previous_month_11, month_11, tz_offset)
        .into_iter()
        .chain(month_spans(year, month_11, next_month_11, tz_offset))
        .filter(|span| span.year == year)
        .map(MonthSpan::to_lunar_month)
        .collect();
    let leap_month = months
        .iter()
        .find(|month| month.is_leap)
        .map(|month| month.month);
    Ok(LunarYear {
        year,
        leap_month,
        months,
    })
}

/// A lunar month located on the axis of civil days.
#[derive(Debug, Clone, Copy)]
struct MonthSpan {
    year: i32,
    month: u8,
    is_leap: bool,
    first_jdn: i32,
    next_first_jdn: i32,
}

impl MonthSpan {
    fn length(&self) -> i32 {
        self.next_first_jdn - self.first_jdn
    }

    fn to_lunar_month(self) -> LunarMonth {
        LunarMonth {
            month: self.month,
            is_leap: self.is_leap,
            first_day: SolarDate::from_jdn(self.first_jdn),
            length: self.length() as u8,
        }
    }
}

/// The lunar month containing day `jdn` of Gregorian year `year`.
fn month_containing(jdn: i32, year: i32, tz_offset: f64) -> MonthSpan {
    let month_11 = month_11_lunation(year, tz_offset);
    let spans = if jdn >= get_new_moon_day(month_11, tz_offset) {
        month_spans(
            year,
            month_11,
            month_11_lunation(year + 1, tz_offset),
            tz_offset,
        )
    } else {
        month_spans(
            year - 1,
            month_11_lunation(year - 1, tz_offset),
            month_11,
            tz_offset,
        )
    };
    spans
        .into_iter()
        .find(|span| (span.first_jdn..span.next_first_jdn).contains(&jdn))
        .expect("consecutive month-11 starts enclose every day of the chosen span")
}

/// The 12 or 13 months from month 11 of Gregorian year `year` up to the next month 11.
fn months_after_solstice(year: i32, tz_offset: f64) -> Vec<MonthSpan> {
    month_spans(
        year,
        month_11_lunation(year, tz_offset),
        month_11_lunation(year + 1, tz_offset),
        tz_offset,
    )
}

/// Labels the months between new moons `month_11` (month 11 of Gregorian year `year`) and
/// `next_month_11` (exclusive).
fn month_spans(year: i32, month_11: i32, next_month_11: i32, tz_offset: f64) -> Vec<MonthSpan> {
    let starts: Vec<i32> = (month_11..=next_month_11)
        .map(|k| get_new_moon_day(k, tz_offset))
        .collect();
    let count = starts.len() - 1;
    // `getLeapMonthOffset`: a month has no principal term when the principal-term index is the
    // same on its first day and on the next month's first day. With 13 months there are only
    // 11 principal terms after the solstice for months 1…12, so such a month always exists.
    let leap_index = if count == 13 {
        (1..count).find(|&i| {
            get_solar_term_index(starts[i], tz_offset)
                == get_solar_term_index(starts[i + 1], tz_offset)
        })
    } else {
        None
    };

    let mut spans = Vec::with_capacity(count);
    let (mut lunar_year, mut month) = (year, 11);
    for i in 0..count {
        let is_leap = leap_index == Some(i);
        if i > 0 && !is_leap {
            month = month % 12 + 1;
            if month == 1 {
                lunar_year += 1;
            }
        }
        spans.push(MonthSpan {
            year: lunar_year,
            month,
            is_leap,
            first_jdn: starts[i],
            next_first_jdn: starts[i + 1],
        });
    }
    spans
}

/// `getLunarMonth11`: number of the new moon that opens month 11 of Gregorian year `year`, the
/// month containing the December solstice.
fn month_11_lunation(year: i32, tz_offset: f64) -> i32 {
    let k = lunation_on_or_before(jd_from_date(31, 12, year), tz_offset);
    // Index 9 (270°) means the solstice had already passed when that month began, so month 11
    // is the one before it.
    if get_solar_term_index(get_new_moon_day(k, tz_offset), tz_offset) >= 9 {
        k - 1
    } else {
        k
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{CN, VN, date};

    fn lunar(year: i32, month: u8, day: u8, is_leap_month: bool) -> LunarDate {
        LunarDate {
            year,
            month,
            day,
            is_leap_month,
        }
    }

    /// `solar_to_lunar` for a date held as a [`SolarDate`].
    fn lunar_of(date: SolarDate, tz_offset: f64) -> Result<LunarDate, AstroError> {
        solar_to_lunar(date.day, date.month, date.year, tz_offset)
    }

    #[test]
    fn tet_at_ty_falls_on_29_january_2025() {
        assert_eq!(
            solar_to_lunar(29, 1, 2025, VN),
            Ok(lunar(2025, 1, 1, false))
        );
        assert_eq!(
            lunar_to_solar(lunar(2025, 1, 1, false), VN),
            Ok(date(2025, 1, 29))
        );
    }

    #[test]
    fn at_ty_2025_has_a_leap_sixth_month() {
        assert_eq!(solar_to_lunar(25, 7, 2025, VN), Ok(lunar(2025, 6, 1, true)));
        assert_eq!(
            lunar_to_solar(lunar(2025, 6, 1, true), VN),
            Ok(date(2025, 7, 25))
        );
        // The regular 6th month starts on 25 June; month 7 follows the leap month on 23 August.
        assert_eq!(
            solar_to_lunar(25, 6, 2025, VN),
            Ok(lunar(2025, 6, 1, false))
        );
        assert_eq!(
            solar_to_lunar(23, 8, 2025, VN),
            Ok(lunar(2025, 7, 1, false))
        );
        assert_eq!(lunar_year(2025, VN).unwrap().leap_month, Some(6));
    }

    #[test]
    fn vietnam_and_china_differ_when_a_new_moon_or_solstice_is_near_midnight() {
        let tet = |year, tz_offset| lunar_to_solar(lunar(year, 1, 1, false), tz_offset);
        // New moon at 16:29 UTC on 29 January 1968: 23:29 in Hà Nội, 00:29 next day in Beijing.
        assert_eq!(tet(1968, VN), Ok(date(1968, 1, 29)));
        assert_eq!(tet(1968, CN), Ok(date(1968, 1, 30)));
        // The December 1984 solstice fell just before midnight at UTC+7 but just after it at
        // UTC+8, on the new-moon day: China needed a leap 10th month in 1984, Vietnam a leap 2nd
        // month in 1985, and Vietnamese Tết came a month earlier.
        assert_eq!(tet(1985, VN), Ok(date(1985, 1, 21)));
        assert_eq!(tet(1985, CN), Ok(date(1985, 2, 20)));
        assert_eq!(lunar_year(1984, CN).unwrap().leap_month, Some(10));
        assert_eq!(lunar_year(1984, VN).unwrap().leap_month, None);
        assert_eq!(lunar_year(1985, VN).unwrap().leap_month, Some(2));
        // 1987: China's leap month repeats month 6; in Vietnam the first month without a
        // principal term comes one month later.
        assert_eq!(lunar_year(1987, CN).unwrap().leap_month, Some(6));
        assert_eq!(lunar_year(1987, VN).unwrap().leap_month, Some(7));
        // New moons at 16:14 UTC on 17 February 2007 and 16:07 UTC on 2 February 2030.
        assert_eq!(tet(2007, VN), Ok(date(2007, 2, 17)));
        assert_eq!(tet(2007, CN), Ok(date(2007, 2, 18)));
        assert_eq!(tet(2030, VN), Ok(date(2030, 2, 2)));
        assert_eq!(tet(2030, CN), Ok(date(2030, 2, 3)));
    }

    #[test]
    fn tet_1978_and_23_january_2033_are_the_same_in_vietnam_and_china() {
        // The Mậu Ngọ new moon came at 14:53 UTC on 7 February 1978 (21:53 in Hà Nội, 22:53 in
        // Beijing), so both countries celebrated Tết on 7 February.
        assert_eq!(solar_to_lunar(7, 2, 1978, VN), Ok(lunar(1978, 1, 1, false)));
        assert_eq!(solar_to_lunar(7, 2, 1978, CN), Ok(lunar(1978, 1, 1, false)));
        // No new moon or principal term near midnight either: 23 Jan 2033 is 23/12 Nhâm Tý.
        assert_eq!(
            solar_to_lunar(23, 1, 2033, VN),
            Ok(lunar(2032, 12, 23, false))
        );
        assert_eq!(
            solar_to_lunar(23, 1, 2033, CN),
            Ok(lunar(2032, 12, 23, false))
        );
    }

    /// Tết Nguyên Đán in Vietnam, 1985–2030.
    const VIETNAMESE_TET: [(i32, u8, u8); 46] = [
        (1985, 1, 21),
        (1986, 2, 9),
        (1987, 1, 29),
        (1988, 2, 17),
        (1989, 2, 6),
        (1990, 1, 27),
        (1991, 2, 15),
        (1992, 2, 4),
        (1993, 1, 23),
        (1994, 2, 10),
        (1995, 1, 31),
        (1996, 2, 19),
        (1997, 2, 7),
        (1998, 1, 28),
        (1999, 2, 16),
        (2000, 2, 5),
        (2001, 1, 24),
        (2002, 2, 12),
        (2003, 2, 1),
        (2004, 1, 22),
        (2005, 2, 9),
        (2006, 1, 29),
        (2007, 2, 17),
        (2008, 2, 7),
        (2009, 1, 26),
        (2010, 2, 14),
        (2011, 2, 3),
        (2012, 1, 23),
        (2013, 2, 10),
        (2014, 1, 31),
        (2015, 2, 19),
        (2016, 2, 8),
        (2017, 1, 28),
        (2018, 2, 16),
        (2019, 2, 5),
        (2020, 1, 25),
        (2021, 2, 12),
        (2022, 2, 1),
        (2023, 1, 22),
        (2024, 2, 10),
        (2025, 1, 29),
        (2026, 2, 17),
        (2027, 2, 6),
        (2028, 1, 26),
        (2029, 2, 13),
        (2030, 2, 2),
    ];

    #[test]
    fn tet_dates_in_vietnam() {
        for (year, month, day) in VIETNAMESE_TET {
            assert_eq!(
                solar_to_lunar(day, month, year, VN),
                Ok(lunar(year, 1, 1, false)),
                "Tết {year}"
            );
            assert_eq!(
                lunar_to_solar(lunar(year, 1, 1, false), VN),
                Ok(date(year, month, day)),
                "Tết {year}"
            );
        }
    }

    #[test]
    fn leap_months_in_vietnam() {
        for (year, leap_month) in [
            (2001, 4),
            (2004, 2),
            (2006, 7),
            (2009, 5),
            (2012, 4),
            (2014, 9),
            (2017, 6),
            (2020, 4),
            (2023, 2),
            (2025, 6),
            (2028, 5),
        ] {
            assert_eq!(
                lunar_year(year, VN).unwrap().leap_month,
                Some(leap_month),
                "{year}"
            );
        }
        for year in [
            2002, 2003, 2005, 2007, 2008, 2010, 2011, 2013, 2015, 2016, 2018, 2019, 2021, 2022,
            2024, 2026, 2027,
        ] {
            assert_eq!(lunar_year(year, VN).unwrap().leap_month, None, "{year}");
        }
    }

    #[test]
    fn leap_eleventh_month_of_2033_in_china() {
        // The "2033 problem": the official Chinese calendar has a leap 11th month.
        assert_eq!(lunar_year(2033, CN).unwrap().leap_month, Some(11));
    }

    #[test]
    fn leap_month_boundaries_of_2023() {
        // Leap 2nd month from 22 March (new moon at 00:23 UTC+7) to 19 April.
        assert_eq!(
            solar_to_lunar(21, 3, 2023, VN),
            Ok(lunar(2023, 2, 30, false))
        );
        assert_eq!(solar_to_lunar(22, 3, 2023, VN), Ok(lunar(2023, 2, 1, true)));
        assert_eq!(
            solar_to_lunar(19, 4, 2023, VN),
            Ok(lunar(2023, 2, 29, true))
        );
        assert_eq!(
            solar_to_lunar(20, 4, 2023, VN),
            Ok(lunar(2023, 3, 1, false))
        );
    }

    #[test]
    fn last_days_of_the_lunar_year() {
        // Quý Mão ended with a 30-day month ("30 Tết"); Giáp Thìn with a 29-day one.
        assert_eq!(
            solar_to_lunar(9, 2, 2024, VN),
            Ok(lunar(2023, 12, 30, false))
        );
        assert_eq!(
            solar_to_lunar(28, 1, 2025, VN),
            Ok(lunar(2024, 12, 29, false))
        );
    }

    #[test]
    fn month_lengths() {
        assert_eq!(lunar_month_length(lunar(2023, 12, 1, false), VN), Ok(30));
        assert_eq!(lunar_month_length(lunar(2024, 12, 1, false), VN), Ok(29));
        assert_eq!(lunar_month_length(lunar(2023, 2, 1, true), VN), Ok(29));
        assert!(lunar_month_length(lunar(2024, 4, 1, true), VN).is_err());
        assert!(lunar_month_length(lunar(2024, 13, 1, false), VN).is_err());
    }

    #[test]
    fn rejects_lunar_dates_that_do_not_exist() {
        assert_eq!(
            lunar_to_solar(lunar(2024, 4, 1, true), VN),
            Err(AstroError::InvalidLunarDate {
                year: 2024,
                month: 4,
                day: 1,
                is_leap_month: true
            })
        );
        assert!(lunar_to_solar(lunar(2024, 12, 30, false), VN).is_err());
        assert!(lunar_to_solar(lunar(2024, 13, 1, false), VN).is_err());
        assert!(lunar_to_solar(lunar(2024, 1, 0, false), VN).is_err());
        assert!(lunar_to_solar(lunar(2024, 1, 31, false), VN).is_err());
    }

    #[test]
    fn rejects_out_of_range_input() {
        assert_eq!(
            lunar_to_solar(lunar(1599, 1, 1, false), VN),
            Err(AstroError::YearOutOfRange { year: 1599 })
        );
        // 1 January 1600 still belongs to lunar year 1599.
        assert_eq!(
            solar_to_lunar(1, 1, 1600, VN),
            Err(AstroError::YearOutOfRange { year: 1599 })
        );
        assert_eq!(
            solar_to_lunar(29, 2, 2023, VN),
            Err(AstroError::InvalidSolarDate {
                year: 2023,
                month: 2,
                day: 29
            })
        );
        assert!(matches!(
            solar_to_lunar(1, 1, 2024, 15.0),
            Err(AstroError::InvalidTimeZone { .. })
        ));
        assert_eq!(
            lunar_year(2401, VN),
            Err(AstroError::YearOutOfRange { year: 2401 })
        );
    }

    #[test]
    fn every_month_from_1900_to_2100_is_consistent() {
        for year in 1900..=2100 {
            let info = lunar_year(year, VN).unwrap();
            let months = &info.months;
            assert_eq!(months.len() == 13, info.leap_month.is_some(), "{year}");
            assert!(matches!(months.len(), 12 | 13), "{year}");
            let total_days: u32 = months.iter().map(|month| u32::from(month.length)).sum();
            assert!(
                matches!(total_days, 353..=355 | 383..=385),
                "{year}: {total_days} days"
            );

            let mut expected_number = 1;
            for (i, month) in months.iter().enumerate() {
                assert!(matches!(month.length, 29 | 30), "{year}/{}", month.month);
                if month.is_leap {
                    assert_eq!(Some(month.month), info.leap_month);
                    assert_eq!(month.month, months[i - 1].month);
                } else {
                    assert_eq!(month.month, expected_number, "{year}");
                    expected_number += 1;
                }
                if let Some(next) = months.get(i + 1) {
                    assert_eq!(
                        month.first_day.to_jdn() + i32::from(month.length),
                        next.first_day.to_jdn()
                    );
                }
                for day in [1, month.length] {
                    let expected = lunar(year, month.month, day, month.is_leap);
                    let solar = SolarDate::from_jdn(month.first_day.to_jdn() + i32::from(day) - 1);
                    assert_eq!(lunar_of(solar, VN), Ok(expected));
                    assert_eq!(lunar_to_solar(expected, VN), Ok(solar));
                }
            }
        }
    }
}
