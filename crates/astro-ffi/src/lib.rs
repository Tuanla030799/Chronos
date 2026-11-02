//! UniFFI bridge exposing astro-core to Swift (iOS) and the other UniFFI languages.
//!
//! The core types are mirrored with `#[uniffi::remote]`, so foreign code receives exactly the
//! records and enums of astro-core with no conversion layer. Every `tz_offset` argument
//! defaults to 7.0 (Vietnam, UTC+7) in the generated bindings.

use astro_core::{
    AstroError, CanChi, CelestialStem, ConnectionCheck, DayInfo, EarthlyBranch, FourPillars,
    FourPillarsOptions, LunarDate, LunarMonth, LunarYear, RatHourConvention, SolarDate,
    SolarDateTime, SolarTerm, SolarTermEvent,
};

uniffi::setup_scaffolding!();

#[uniffi::remote(Record)]
pub struct SolarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

#[uniffi::remote(Record)]
pub struct ConnectionCheck {
    pub engine_version: String,
    pub date: SolarDate,
    pub julian_day_number: i32,
}

#[uniffi::remote(Record)]
pub struct SolarDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[uniffi::remote(Record)]
pub struct LunarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub is_leap_month: bool,
}

#[uniffi::remote(Record)]
pub struct LunarMonth {
    pub month: u8,
    pub is_leap: bool,
    pub first_day: SolarDate,
    pub length: u8,
}

#[uniffi::remote(Record)]
pub struct LunarYear {
    pub year: i32,
    pub leap_month: Option<u8>,
    pub months: Vec<LunarMonth>,
}

#[uniffi::remote(Enum)]
pub enum SolarTerm {
    LapXuan,
    VuThuy,
    KinhTrap,
    XuanPhan,
    ThanhMinh,
    CocVu,
    LapHa,
    TieuMan,
    MangChung,
    HaChi,
    TieuThu,
    DaiThu,
    LapThu,
    XuThu,
    BachLo,
    ThuPhan,
    HanLo,
    SuongGiang,
    LapDong,
    TieuTuyet,
    DaiTuyet,
    DongChi,
    TieuHan,
    DaiHan,
}

#[uniffi::remote(Record)]
pub struct SolarTermEvent {
    pub term: SolarTerm,
    pub jd_ut: f64,
    pub local_time: SolarDateTime,
}

#[uniffi::remote(Enum)]
pub enum CelestialStem {
    Giap,
    At,
    Binh,
    Dinh,
    Mau,
    Ky,
    Canh,
    Tan,
    Nham,
    Quy,
}

#[uniffi::remote(Enum)]
pub enum EarthlyBranch {
    Ty,
    Suu,
    Dan,
    Mao,
    Thin,
    Ti,
    Ngo,
    Mui,
    Than,
    Dau,
    Tuat,
    Hoi,
}

#[uniffi::remote(Record)]
pub struct CanChi {
    pub stem: CelestialStem,
    pub branch: EarthlyBranch,
}

#[uniffi::remote(Enum)]
pub enum RatHourConvention {
    NewDayAtRatHour,
    SplitAtMidnight,
}

#[uniffi::remote(Record)]
pub struct FourPillarsOptions {
    #[uniffi(default = 7.0)]
    pub tz_offset: f64,
    #[uniffi(default)]
    pub longitude: Option<f64>,
    pub rat_hour: RatHourConvention,
}

#[uniffi::remote(Record)]
pub struct FourPillars {
    pub year: CanChi,
    pub month: CanChi,
    pub day: CanChi,
    pub hour: CanChi,
    pub effective_time: SolarDateTime,
}

#[uniffi::remote(Record)]
pub struct DayInfo {
    pub solar: SolarDate,
    pub lunar: LunarDate,
    pub julian_day_number: i32,
    pub year_can_chi: CanChi,
    pub month_can_chi: CanChi,
    pub day_can_chi: CanChi,
    pub solar_term: SolarTerm,
    pub solar_term_begins: bool,
}

#[uniffi::remote(Error)]
pub enum AstroError {
    InvalidSolarDate {
        year: i32,
        month: u8,
        day: u8,
    },
    InvalidTime {
        hour: u8,
        minute: u8,
        second: u8,
    },
    InvalidLunarDate {
        year: i32,
        month: u8,
        day: u8,
        is_leap_month: bool,
    },
    InvalidTimeZone {
        tz_offset: f64,
    },
    InvalidLongitude {
        longitude: f64,
    },
    YearOutOfRange {
        year: i32,
    },
}

/// Connection check: sends `date` to the Rust core and gets it back, validated, with its Julian
/// Day Number and the engine version.
#[uniffi::export]
pub fn check_connection(date: SolarDate) -> Result<ConnectionCheck, AstroError> {
    astro_core::check_connection(date)
}

/// Converts a Gregorian date to the Vietnamese lunar calendar.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn solar_to_lunar(date: SolarDate, tz_offset: f64) -> Result<LunarDate, AstroError> {
    astro_core::solar_to_lunar(date, tz_offset)
}

/// Converts a lunar date to the Gregorian calendar.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn lunar_to_solar(date: LunarDate, tz_offset: f64) -> Result<SolarDate, AstroError> {
    astro_core::lunar_to_solar(date, tz_offset)
}

/// All months of a lunar year, including its leap month.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn lunar_year(year: i32, tz_offset: f64) -> Result<LunarYear, AstroError> {
    astro_core::lunar_year(year, tz_offset)
}

/// The 24 solar terms beginning in a Gregorian year, in chronological order.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn solar_terms_in_year(year: i32, tz_offset: f64) -> Result<Vec<SolarTermEvent>, AstroError> {
    astro_core::solar_terms_in_year(year, tz_offset)
}

/// Lunar date, Can Chi and solar term of a Gregorian date, for calendar views.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn day_info(date: SolarDate, tz_offset: f64) -> Result<DayInfo, AstroError> {
    astro_core::day_info(date, tz_offset)
}

/// The Four Pillars of a birth at a wall-clock time.
#[uniffi::export]
pub fn four_pillars(
    birth: SolarDateTime,
    options: FourPillarsOptions,
) -> Result<FourPillars, AstroError> {
    astro_core::four_pillars(birth, &options)
}

/// True solar time at `longitude` degrees east for a wall-clock time.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn true_solar_time(
    clock: SolarDateTime,
    longitude: f64,
    tz_offset: f64,
) -> Result<SolarDateTime, AstroError> {
    astro_core::true_solar_time(clock, tz_offset, longitude)
}

/// Vietnamese name of a Can Chi pair, e.g. "Giáp Thìn".
#[uniffi::export]
pub fn can_chi_name(value: CanChi) -> String {
    value.name_vi()
}

/// Hán characters of a Can Chi pair, e.g. "甲辰".
#[uniffi::export]
pub fn can_chi_han_name(value: CanChi) -> String {
    value.name_han()
}

/// Vietnamese name of a Heavenly Stem, e.g. "Giáp".
#[uniffi::export]
pub fn celestial_stem_name(stem: CelestialStem) -> String {
    stem.name_vi().to_owned()
}

/// Vietnamese name of an Earthly Branch, e.g. "Tý".
#[uniffi::export]
pub fn earthly_branch_name(branch: EarthlyBranch) -> String {
    branch.name_vi().to_owned()
}

/// Vietnamese name of a solar term, e.g. "Lập Xuân".
#[uniffi::export]
pub fn solar_term_name(term: SolarTerm) -> String {
    term.name_vi().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VN: f64 = 7.0;

    #[test]
    fn default_time_zone_literals_match_the_core_constant() {
        // The `default(tz_offset = 7.0)` attributes above must track this constant.
        assert_eq!(astro_core::DEFAULT_TZ_OFFSET, 7.0);
    }

    #[test]
    fn connection_check_round_trips_a_date() {
        let tet = SolarDate {
            year: 2024,
            month: 2,
            day: 10,
        };
        let check = check_connection(tet).unwrap();
        assert_eq!(check.date, tet);
        assert_eq!(check.julian_day_number, 2_460_351);
        assert_eq!(check.engine_version, astro_core::ENGINE_VERSION);
        assert!(check_connection(SolarDate { day: 30, ..tet }).is_err());
    }

    #[test]
    fn calendar_functions_delegate_to_the_core() {
        let tet = SolarDate {
            year: 2024,
            month: 2,
            day: 10,
        };
        let lunar = solar_to_lunar(tet, VN).unwrap();
        assert_eq!(
            lunar,
            LunarDate {
                year: 2024,
                month: 1,
                day: 1,
                is_leap_month: false
            }
        );
        assert_eq!(lunar_to_solar(lunar, VN), Ok(tet));
        assert_eq!(lunar_year(2025, VN).unwrap().leap_month, Some(6));
        assert_eq!(solar_terms_in_year(2024, VN).unwrap().len(), 24);

        let info = day_info(tet, VN).unwrap();
        assert_eq!(can_chi_name(info.day_can_chi), "Giáp Thìn");
        assert_eq!(can_chi_han_name(info.year_can_chi), "甲辰");
        assert_eq!(celestial_stem_name(info.month_can_chi.stem), "Bính");
        assert_eq!(earthly_branch_name(info.month_can_chi.branch), "Dần");
        assert_eq!(solar_term_name(info.solar_term), "Lập Xuân");
    }

    #[test]
    fn four_pillars_and_true_solar_time_delegate_to_the_core() {
        let birth = SolarDateTime {
            year: 2024,
            month: 2,
            day: 10,
            hour: 8,
            minute: 0,
            second: 0,
        };
        let options = FourPillarsOptions {
            tz_offset: VN,
            longitude: None,
            rat_hour: RatHourConvention::NewDayAtRatHour,
        };
        assert_eq!(
            can_chi_name(four_pillars(birth, options).unwrap().hour),
            "Mậu Thìn"
        );
        // Mid-February the Sun lags the clock by about 14 minutes.
        let solar = true_solar_time(birth, 105.0, VN).unwrap();
        assert_eq!((solar.hour, solar.minute), (7, 45));
    }

    #[test]
    fn errors_carry_the_rejected_input() {
        assert_eq!(
            solar_to_lunar(
                SolarDate {
                    year: 2023,
                    month: 2,
                    day: 29
                },
                VN
            ),
            Err(AstroError::InvalidSolarDate {
                year: 2023,
                month: 2,
                day: 29
            })
        );
        assert_eq!(
            lunar_year(1500, VN),
            Err(AstroError::YearOutOfRange { year: 1500 })
        );
    }
}
