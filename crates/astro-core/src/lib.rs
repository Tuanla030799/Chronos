//! # astro-core
//!
//! Offline computational astronomy for the Vietnamese lunisolar calendar: Julian Days, the
//! Sun's position, true new moons (Sóc), the 24 solar terms (Tiết khí), lunar-calendar
//! conversion (Âm lịch), the sexagenary cycle (Can Chi), true solar time, the Four
//! Pillars (Tứ Trụ), feng shui (Phong Thủy) and Tử Vi Đẩu Số. The crate has no runtime dependencies and
//! performs no I/O.
//!
//! ## Time zones
//!
//! Every calendar function takes `tz_offset`, the zone's offset from UTC in hours. Vietnam
//! uses [`DEFAULT_TZ_OFFSET`] (UTC+7, standard meridian 105°E); pass `8.0` to reproduce
//! Chinese almanacs.
//!
//! ## Sources
//!
//! - Jean Meeus, *Astronomical Algorithms*, 2nd ed. (1998): Julian Day (ch. 7), nutation
//!   (ch. 22), solar coordinates with VSOP87 (ch. 25 and appendix III), equation of time
//!   (ch. 28), phases of the Moon (ch. 49).
//! - F. Espenak & J. Meeus, *Five Millennium Canon of Solar Eclipses* (2006): ΔT.
//! - Hồ Ngọc Đức, *Thuật toán tính âm lịch*: lunisolar month rules and Can Chi formulas.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod almanac;
pub mod astronomy;
pub mod bazi_analysis;
pub mod bazi_pillars;
pub mod canchi;
pub mod content;
pub mod day_info;
pub mod engine;
mod error;
pub mod fengshui;
pub mod lunar_calendar;
mod math;
pub mod solar_terms;
#[cfg(test)]
mod test_support;
pub mod time;
pub mod true_solar_time;
pub mod tuvi;

pub use almanac::{AgeRestrictions, AlmanacDay, age_restrictions, almanac_day};
pub use bazi_analysis::{BaziAnalysis, Gender, analyze_bazi};
pub use bazi_pillars::{
    FourPillars, FourPillarsOptions, RatHourConvention, YearMonthBoundary, four_pillars,
};
pub use canchi::{Branch, CanChi, FiveElements, NaYin, Stem, YinYang};
pub use day_info::{DayInfo, day_info};
pub use engine::{ConnectionCheck, ENGINE_VERSION, check_connection};
pub use error::AstroError;
pub use fengshui::{
    CompassDirection, EightMansionsStar, KuaCompatibility, KuaInfo, LoShuGrid, NineStar,
    NineStarBoundary, NineStars, Palace, Trigram, generate_loshu_grid, house_trigram,
    house_trigram_from_facing, kua_compatibility, kua_info, nine_stars,
};
pub use lunar_calendar::{
    LunarDate, LunarMonth, LunarYear, lunar_to_solar, lunar_year, solar_to_lunar,
};
pub use solar_terms::{SolarTerm, SolarTermEvent, get_exact_solar_term_time, solar_terms_in_year};
pub use time::{SolarDate, SolarDateTime};
pub use true_solar_time::{VIETNAM_STANDARD_MERIDIAN, to_true_solar_time};
pub use tuvi::{
    FiveElementsBureau, FourTransformations, LeapMonthRule, MenhCucRelation, StarBrightness,
    StarCategory, StarNature, TuViChart, TuViConfig, TuViPalace, TuViPalaceRole, TuViPlacedStar,
    TuViPolarity, TuViStar, YinYangHarmony, tuvi_chart, tuvi_chart_from_lunar,
};

/// Vietnam's offset from UTC in hours (UTC+7, standard meridian 105°E); the default
/// `tz_offset` of the Swift and WebAssembly bindings.
pub const DEFAULT_TZ_OFFSET: f64 = 7.0;

/// Earliest year accepted by the calendar functions.
pub const MIN_YEAR: i32 = 1600;

/// Latest year accepted by the calendar functions.
pub const MAX_YEAR: i32 = 2400;
