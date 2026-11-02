//! The 24 solar terms (Hai mươi bốn tiết khí) and the instants at which they begin.
//!
//! A solar term begins when the Sun's apparent ecliptic longitude reaches a multiple of 15°.
//! The instant is found by solving `λ☉(t) = target` numerically, so it carries the precision
//! of the VSOP87 solar position instead of a mean-motion approximation.

use crate::delta_t::{jd_ut_to_jde, jde_to_jd_ut};
use crate::error::{AstroError, check_tz_offset, check_year};
use crate::math::{angle_difference, normalize_degrees};
use crate::sun::apparent_longitude;
use crate::time::{SolarDate, SolarDateTime};

/// Mean motion of the Sun in longitude, degrees per day.
const MEAN_DAILY_MOTION: f64 = 360.0 / 365.242_189;

/// The iteration stops once its correction is below this many days (about a millisecond).
const CONVERGENCE_DAYS: f64 = 1e-8;

/// Upper bound on iterations; convergence normally takes six or seven.
const MAX_ITERATIONS: usize = 50;

/// One of the 24 solar terms, in traditional order from Lập Xuân.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SolarTerm {
    /// Lập Xuân 立春 (315°), beginning of spring; opens the Dần month and the Bazi year.
    LapXuan,
    /// Vũ Thủy 雨水 (330°).
    VuThuy,
    /// Kinh Trập 驚蟄 (345°).
    KinhTrap,
    /// Xuân Phân 春分 (0°), March equinox.
    XuanPhan,
    /// Thanh Minh 清明 (15°).
    ThanhMinh,
    /// Cốc Vũ 穀雨 (30°).
    CocVu,
    /// Lập Hạ 立夏 (45°), beginning of summer.
    LapHa,
    /// Tiểu Mãn 小滿 (60°).
    TieuMan,
    /// Mang Chủng 芒種 (75°).
    MangChung,
    /// Hạ Chí 夏至 (90°), June solstice.
    HaChi,
    /// Tiểu Thử 小暑 (105°).
    TieuThu,
    /// Đại Thử 大暑 (120°).
    DaiThu,
    /// Lập Thu 立秋 (135°), beginning of autumn.
    LapThu,
    /// Xử Thử 處暑 (150°).
    XuThu,
    /// Bạch Lộ 白露 (165°).
    BachLo,
    /// Thu Phân 秋分 (180°), September equinox.
    ThuPhan,
    /// Hàn Lộ 寒露 (195°).
    HanLo,
    /// Sương Giáng 霜降 (210°).
    SuongGiang,
    /// Lập Đông 立冬 (225°), beginning of winter.
    LapDong,
    /// Tiểu Tuyết 小雪 (240°).
    TieuTuyet,
    /// Đại Tuyết 大雪 (255°).
    DaiTuyet,
    /// Đông Chí 冬至 (270°), December solstice; always falls in lunar month 11.
    DongChi,
    /// Tiểu Hàn 小寒 (285°).
    TieuHan,
    /// Đại Hàn 大寒 (300°).
    DaiHan,
}

impl SolarTerm {
    /// All terms in traditional order, starting with Lập Xuân.
    pub const ALL: [SolarTerm; 24] = [
        SolarTerm::LapXuan,
        SolarTerm::VuThuy,
        SolarTerm::KinhTrap,
        SolarTerm::XuanPhan,
        SolarTerm::ThanhMinh,
        SolarTerm::CocVu,
        SolarTerm::LapHa,
        SolarTerm::TieuMan,
        SolarTerm::MangChung,
        SolarTerm::HaChi,
        SolarTerm::TieuThu,
        SolarTerm::DaiThu,
        SolarTerm::LapThu,
        SolarTerm::XuThu,
        SolarTerm::BachLo,
        SolarTerm::ThuPhan,
        SolarTerm::HanLo,
        SolarTerm::SuongGiang,
        SolarTerm::LapDong,
        SolarTerm::TieuTuyet,
        SolarTerm::DaiTuyet,
        SolarTerm::DongChi,
        SolarTerm::TieuHan,
        SolarTerm::DaiHan,
    ];

    /// Position in [`SolarTerm::ALL`], from 0 (Lập Xuân) to 23 (Đại Hàn).
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Apparent solar longitude, in degrees, at which this term begins.
    pub fn longitude(self) -> f64 {
        normalize_degrees(315.0 + 15.0 * self.index() as f64)
    }

    /// The term in effect while the Sun's apparent longitude is `longitude` degrees.
    pub fn from_sun_longitude(longitude: f64) -> SolarTerm {
        let steps_from_lap_xuan = (normalize_degrees(longitude - 315.0) / 15.0).floor() as usize;
        Self::ALL[steps_from_lap_xuan.min(23)]
    }

    /// Whether this is a principal term (Trung khí 中氣), at a multiple of 30°.
    ///
    /// Principal terms number the lunar months, and a lunar month without one can become a
    /// leap month. The other twelve are sectional terms (Tiết 節), which open the Bazi months.
    pub const fn is_principal(self) -> bool {
        self.index() % 2 == 1
    }

    /// Vietnamese name, e.g. "Lập Xuân".
    pub const fn name_vi(self) -> &'static str {
        match self {
            SolarTerm::LapXuan => "Lập Xuân",
            SolarTerm::VuThuy => "Vũ Thủy",
            SolarTerm::KinhTrap => "Kinh Trập",
            SolarTerm::XuanPhan => "Xuân Phân",
            SolarTerm::ThanhMinh => "Thanh Minh",
            SolarTerm::CocVu => "Cốc Vũ",
            SolarTerm::LapHa => "Lập Hạ",
            SolarTerm::TieuMan => "Tiểu Mãn",
            SolarTerm::MangChung => "Mang Chủng",
            SolarTerm::HaChi => "Hạ Chí",
            SolarTerm::TieuThu => "Tiểu Thử",
            SolarTerm::DaiThu => "Đại Thử",
            SolarTerm::LapThu => "Lập Thu",
            SolarTerm::XuThu => "Xử Thử",
            SolarTerm::BachLo => "Bạch Lộ",
            SolarTerm::ThuPhan => "Thu Phân",
            SolarTerm::HanLo => "Hàn Lộ",
            SolarTerm::SuongGiang => "Sương Giáng",
            SolarTerm::LapDong => "Lập Đông",
            SolarTerm::TieuTuyet => "Tiểu Tuyết",
            SolarTerm::DaiTuyet => "Đại Tuyết",
            SolarTerm::DongChi => "Đông Chí",
            SolarTerm::TieuHan => "Tiểu Hàn",
            SolarTerm::DaiHan => "Đại Hàn",
        }
    }

    /// Name in Hán characters, e.g. "立春".
    pub const fn name_han(self) -> &'static str {
        match self {
            SolarTerm::LapXuan => "立春",
            SolarTerm::VuThuy => "雨水",
            SolarTerm::KinhTrap => "驚蟄",
            SolarTerm::XuanPhan => "春分",
            SolarTerm::ThanhMinh => "清明",
            SolarTerm::CocVu => "穀雨",
            SolarTerm::LapHa => "立夏",
            SolarTerm::TieuMan => "小滿",
            SolarTerm::MangChung => "芒種",
            SolarTerm::HaChi => "夏至",
            SolarTerm::TieuThu => "小暑",
            SolarTerm::DaiThu => "大暑",
            SolarTerm::LapThu => "立秋",
            SolarTerm::XuThu => "處暑",
            SolarTerm::BachLo => "白露",
            SolarTerm::ThuPhan => "秋分",
            SolarTerm::HanLo => "寒露",
            SolarTerm::SuongGiang => "霜降",
            SolarTerm::LapDong => "立冬",
            SolarTerm::TieuTuyet => "小雪",
            SolarTerm::DaiTuyet => "大雪",
            SolarTerm::DongChi => "冬至",
            SolarTerm::TieuHan => "小寒",
            SolarTerm::DaiHan => "大寒",
        }
    }
}

/// The beginning of a solar term.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SolarTermEvent {
    /// The term that begins.
    pub term: SolarTerm,
    /// The instant, as a Julian Day (UT).
    pub jd_ut: f64,
    /// The same instant in local wall-clock time, rounded to the second.
    pub local_time: SolarDateTime,
}

/// Instant (Julian Day, UT) at which the Sun's apparent longitude equals `longitude`
/// degrees, solved from the guess `jd_ut_guess`; it converges to the crossing nearest the
/// guess.
pub fn sun_longitude_crossing(longitude: f64, jd_ut_guess: f64) -> f64 {
    let mut jde = jd_ut_to_jde(jd_ut_guess);
    for _ in 0..MAX_ITERATIONS {
        let correction = angle_difference(longitude, apparent_longitude(jde)) / MEAN_DAILY_MOTION;
        jde += correction;
        if correction.abs() < CONVERGENCE_DAYS {
            break;
        }
    }
    jde_to_jd_ut(jde)
}

/// Instant (Julian Day, UT) at which `term` begins in Gregorian year `year`.
pub fn solar_term_jd_ut(term: SolarTerm, year: i32) -> Result<f64, AstroError> {
    check_year(year)?;
    Ok(term_instant(term, year))
}

/// All 24 solar terms beginning in Gregorian year `year`, in chronological order (Tiểu Hàn
/// first, Đông Chí last), with local times in a zone `tz_offset` hours east of UTC.
pub fn solar_terms_in_year(year: i32, tz_offset: f64) -> Result<Vec<SolarTermEvent>, AstroError> {
    check_year(year)?;
    check_tz_offset(tz_offset)?;
    let mut events: Vec<SolarTermEvent> = SolarTerm::ALL
        .into_iter()
        .map(|term| {
            let jd_ut = term_instant(term, year);
            SolarTermEvent {
                term,
                jd_ut,
                local_time: SolarDateTime::from_julian_day(jd_ut, tz_offset),
            }
        })
        .collect();
    events.sort_by(|a, b| a.jd_ut.total_cmp(&b.jd_ut));
    Ok(events)
}

/// The solar term in effect at the instant `jd_ut`: the last one that began at or before it.
pub fn solar_term_at(jd_ut: f64) -> SolarTerm {
    SolarTerm::from_sun_longitude(apparent_longitude(jd_ut_to_jde(jd_ut)))
}

/// Unchecked [`solar_term_jd_ut`].
pub(crate) fn term_instant(term: SolarTerm, year: i32) -> f64 {
    // Start from the mean-motion position relative to the March equinox; the terms from
    // Tiểu Hàn (285°) to Kinh Trập (345°) come before it in the year.
    let mut degrees_after_equinox = term.longitude();
    if degrees_after_equinox >= 285.0 {
        degrees_after_equinox -= 360.0;
    }
    let equinox = f64::from(
        SolarDate {
            year,
            month: 3,
            day: 20,
        }
        .to_jdn(),
    );
    sun_longitude_crossing(
        term.longitude(),
        equinox + degrees_after_equinox / MEAN_DAILY_MOTION,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{VN, assert_close, utc};

    /// Published instants are rounded to the minute.
    const ONE_AND_A_HALF_MINUTES: f64 = 1.5 / 1440.0;

    #[test]
    fn longitudes_and_kinds() {
        assert_eq!(SolarTerm::LapXuan.longitude(), 315.0);
        assert_eq!(SolarTerm::XuanPhan.longitude(), 0.0);
        assert_eq!(SolarTerm::HaChi.longitude(), 90.0);
        assert_eq!(SolarTerm::ThuPhan.longitude(), 180.0);
        assert_eq!(SolarTerm::DongChi.longitude(), 270.0);
        assert_eq!(SolarTerm::DaiHan.longitude(), 300.0);
        for (index, term) in SolarTerm::ALL.into_iter().enumerate() {
            assert_eq!(term.index(), index);
            assert_eq!(
                term.is_principal(),
                term.longitude() % 30.0 == 0.0,
                "{term:?}"
            );
        }
    }

    #[test]
    fn term_in_effect_for_a_longitude() {
        assert_eq!(SolarTerm::from_sun_longitude(0.0), SolarTerm::XuanPhan);
        assert_eq!(SolarTerm::from_sun_longitude(14.999), SolarTerm::XuanPhan);
        assert_eq!(SolarTerm::from_sun_longitude(15.0), SolarTerm::ThanhMinh);
        assert_eq!(SolarTerm::from_sun_longitude(314.999), SolarTerm::DaiHan);
        assert_eq!(SolarTerm::from_sun_longitude(315.0), SolarTerm::LapXuan);
        assert_eq!(SolarTerm::from_sun_longitude(359.999), SolarTerm::KinhTrap);
        assert_eq!(SolarTerm::from_sun_longitude(-0.5), SolarTerm::KinhTrap);
    }

    #[test]
    fn names() {
        assert_eq!(SolarTerm::LapXuan.name_vi(), "Lập Xuân");
        assert_eq!(SolarTerm::DongChi.name_vi(), "Đông Chí");
        assert_eq!(SolarTerm::KinhTrap.name_han(), "驚蟄");
    }

    #[test]
    fn equinoxes_and_solstices_match_published_times() {
        // US Naval Observatory, UTC.
        for (year, term, published) in [
            (2000, SolarTerm::XuanPhan, utc(2000, 3, 20, 7, 35)),
            (2000, SolarTerm::HaChi, utc(2000, 6, 21, 1, 48)),
            (2000, SolarTerm::ThuPhan, utc(2000, 9, 22, 17, 27)),
            (2000, SolarTerm::DongChi, utc(2000, 12, 21, 13, 37)),
            (2024, SolarTerm::XuanPhan, utc(2024, 3, 20, 3, 6)),
            (2024, SolarTerm::HaChi, utc(2024, 6, 20, 20, 51)),
            (2024, SolarTerm::ThuPhan, utc(2024, 9, 22, 12, 44)),
            (2024, SolarTerm::DongChi, utc(2024, 12, 21, 9, 20)),
            (2025, SolarTerm::XuanPhan, utc(2025, 3, 20, 9, 1)),
            (2025, SolarTerm::HaChi, utc(2025, 6, 21, 2, 42)),
            (2025, SolarTerm::ThuPhan, utc(2025, 9, 22, 18, 19)),
            (2025, SolarTerm::DongChi, utc(2025, 12, 21, 15, 3)),
        ] {
            assert_close(
                solar_term_jd_ut(term, year).unwrap(),
                published,
                ONE_AND_A_HALF_MINUTES,
            );
        }
    }

    #[test]
    fn lap_xuan_matches_published_times() {
        // Chinese almanac times (UTC+8) converted to UTC.
        for (year, published) in [
            (2023, utc(2023, 2, 4, 2, 42)),
            (2024, utc(2024, 2, 4, 8, 27)),
            (2025, utc(2025, 2, 3, 14, 10)),
        ] {
            assert_close(
                solar_term_jd_ut(SolarTerm::LapXuan, year).unwrap(),
                published,
                ONE_AND_A_HALF_MINUTES,
            );
        }
    }

    #[test]
    fn the_sun_is_at_the_term_longitude_at_the_computed_instant() {
        for term in SolarTerm::ALL {
            let jde = jd_ut_to_jde(solar_term_jd_ut(term, 2024).unwrap());
            let miss = angle_difference(apparent_longitude(jde), term.longitude());
            assert!(miss.abs() < 1e-6, "{term:?} misses by {miss}°");
        }
    }

    #[test]
    fn a_year_has_twenty_four_terms_in_order() {
        let events = solar_terms_in_year(2024, VN).unwrap();
        assert_eq!(events.len(), 24);
        assert_eq!(events[0].term, SolarTerm::TieuHan);
        assert_eq!(events[2].term, SolarTerm::LapXuan);
        assert_eq!(events[23].term, SolarTerm::DongChi);
        assert!(events.windows(2).all(|pair| pair[0].jd_ut < pair[1].jd_ut));
        assert!(events.iter().all(|event| event.local_time.year == 2024));
        // Đông Chí 2024: 09:20 UTC, i.e. 16:20 on 21 December in Vietnam.
        let solstice = events[23].local_time;
        assert_eq!((solstice.month, solstice.day, solstice.hour), (12, 21, 16));
        assert!(solstice.minute.abs_diff(20) <= 1, "{solstice:?}");
    }

    #[test]
    fn term_in_effect_at_an_instant() {
        assert_eq!(solar_term_at(utc(2024, 2, 10, 0, 0)), SolarTerm::LapXuan);
        assert_eq!(solar_term_at(utc(2024, 12, 25, 0, 0)), SolarTerm::DongChi);
        assert_eq!(solar_term_at(utc(2025, 1, 1, 0, 0)), SolarTerm::DongChi);
    }

    #[test]
    fn validates_inputs() {
        assert_eq!(
            solar_terms_in_year(2024, 20.0).unwrap_err(),
            AstroError::InvalidTimeZone { tz_offset: 20.0 }
        );
        assert_eq!(
            solar_term_jd_ut(SolarTerm::LapXuan, 2401),
            Err(AstroError::YearOutOfRange { year: 2401 })
        );
    }
}
