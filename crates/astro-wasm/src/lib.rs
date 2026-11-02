//! WebAssembly bridge exposing astro-core to JavaScript and TypeScript.
//!
//! Dates go in and results come out as plain JS objects converted with serde-wasm-bindgen:
//! field names are camelCase and enums are their variant names. The TypeScript shapes are
//! declared below. Every `tz_offset` argument is optional and defaults to 7 (Vietnam, UTC+7).

use astro_core::{
    AstroError, CanChi, DEFAULT_TZ_OFFSET, FourPillarsOptions, LunarDate, RatHourConvention,
    SolarDate, SolarDateTime, SolarTerm,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_TYPES: &str = r#"
export interface SolarDate { year: number; month: number; day: number; }
export interface ConnectionCheck { engineVersion: string; date: SolarDate; julianDayNumber: number; }
export interface SolarDateTime {
  year: number; month: number; day: number; hour: number; minute: number; second: number;
}
export interface LunarDate { year: number; month: number; day: number; isLeapMonth: boolean; }
export interface LunarMonth { month: number; isLeap: boolean; firstDay: SolarDate; length: number; }
export interface LunarYear { year: number; leapMonth: number | null; months: LunarMonth[]; }
export type SolarTerm =
  | "LapXuan" | "VuThuy" | "KinhTrap" | "XuanPhan" | "ThanhMinh" | "CocVu"
  | "LapHa" | "TieuMan" | "MangChung" | "HaChi" | "TieuThu" | "DaiThu"
  | "LapThu" | "XuThu" | "BachLo" | "ThuPhan" | "HanLo" | "SuongGiang"
  | "LapDong" | "TieuTuyet" | "DaiTuyet" | "DongChi" | "TieuHan" | "DaiHan";
export interface SolarTermEvent { term: SolarTerm; jdUt: number; localTime: SolarDateTime; }
export type CelestialStem =
  | "Giap" | "At" | "Binh" | "Dinh" | "Mau" | "Ky" | "Canh" | "Tan" | "Nham" | "Quy";
export type EarthlyBranch =
  | "Ty" | "Suu" | "Dan" | "Mao" | "Thin" | "Ti" | "Ngo" | "Mui" | "Than" | "Dau" | "Tuat" | "Hoi";
export interface CanChi { stem: CelestialStem; branch: EarthlyBranch; }
export type RatHourConvention = "NewDayAtRatHour" | "SplitAtMidnight";
export interface FourPillarsInput {
  ratHour: RatHourConvention;
  /** Hours east of UTC; defaults to 7 (Vietnam). */
  tzOffset?: number;
  /** Birthplace longitude in degrees east; enables true solar time for the day and hour. */
  longitude?: number | null;
}
export interface FourPillars {
  year: CanChi; month: CanChi; day: CanChi; hour: CanChi; effectiveTime: SolarDateTime;
}
export interface DayInfo {
  solar: SolarDate;
  lunar: LunarDate;
  julianDayNumber: number;
  yearCanChi: CanChi;
  monthCanChi: CanChi;
  dayCanChi: CanChi;
  solarTerm: SolarTerm;
  solarTermBegins: boolean;
}
"#;

/// Options accepted by `fourPillars`; `tzOffset` defaults to 7 and `longitude` to none.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FourPillarsInput {
    rat_hour: RatHourConvention,
    #[serde(default = "default_tz_offset")]
    tz_offset: f64,
    #[serde(default)]
    longitude: Option<f64>,
}

fn default_tz_offset() -> f64 {
    DEFAULT_TZ_OFFSET
}

/// Connection check: sends `date` to the Rust core and gets it back, validated, with its Julian
/// Day Number and the engine version.
#[wasm_bindgen(js_name = checkConnection, unchecked_return_type = "ConnectionCheck")]
pub fn check_connection(
    #[wasm_bindgen(unchecked_param_type = "SolarDate")] date: JsValue,
) -> Result<JsValue, JsValue> {
    let date: SolarDate = from_js(date)?;
    to_js(&astro_core::check_connection(date).map_err(astro_error)?)
}

/// Converts a Gregorian date to the Vietnamese lunar calendar.
#[wasm_bindgen(js_name = solarToLunar, unchecked_return_type = "LunarDate")]
pub fn solar_to_lunar(
    #[wasm_bindgen(unchecked_param_type = "SolarDate")] date: JsValue,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let date: SolarDate = from_js(date)?;
    to_js(
        &astro_core::solar_to_lunar(date.day, date.month, date.year, zone(tz_offset))
            .map_err(astro_error)?,
    )
}

/// Converts a lunar date to the Gregorian calendar.
#[wasm_bindgen(js_name = lunarToSolar, unchecked_return_type = "SolarDate")]
pub fn lunar_to_solar(
    #[wasm_bindgen(unchecked_param_type = "LunarDate")] date: JsValue,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let date: LunarDate = from_js(date)?;
    to_js(&astro_core::lunar_to_solar(date, zone(tz_offset)).map_err(astro_error)?)
}

/// All months of a lunar year, including its leap month.
#[wasm_bindgen(js_name = lunarYear, unchecked_return_type = "LunarYear")]
pub fn lunar_year(year: i32, tz_offset: Option<f64>) -> Result<JsValue, JsValue> {
    to_js(&astro_core::lunar_year(year, zone(tz_offset)).map_err(astro_error)?)
}

/// The 24 solar terms beginning in a Gregorian year, in chronological order.
#[wasm_bindgen(js_name = solarTermsInYear, unchecked_return_type = "SolarTermEvent[]")]
pub fn solar_terms_in_year(year: i32, tz_offset: Option<f64>) -> Result<JsValue, JsValue> {
    to_js(&astro_core::solar_terms_in_year(year, zone(tz_offset)).map_err(astro_error)?)
}

/// Local wall-clock time, to the second, at which `term` begins in a Gregorian year.
#[wasm_bindgen(js_name = getExactSolarTermTime, unchecked_return_type = "SolarDateTime")]
pub fn get_exact_solar_term_time(
    year: i32,
    #[wasm_bindgen(unchecked_param_type = "SolarTerm")] term: JsValue,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let term: SolarTerm = from_js(term)?;
    to_js(&astro_core::get_exact_solar_term_time(year, term, zone(tz_offset)).map_err(astro_error)?)
}

/// Lunar date, Can Chi and solar term of a Gregorian date, for calendar views.
#[wasm_bindgen(js_name = dayInfo, unchecked_return_type = "DayInfo")]
pub fn day_info(
    #[wasm_bindgen(unchecked_param_type = "SolarDate")] date: JsValue,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let date: SolarDate = from_js(date)?;
    to_js(&astro_core::day_info(date, zone(tz_offset)).map_err(astro_error)?)
}

/// The Four Pillars of a birth at a wall-clock time.
#[wasm_bindgen(js_name = fourPillars, unchecked_return_type = "FourPillars")]
pub fn four_pillars(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] birth: JsValue,
    #[wasm_bindgen(unchecked_param_type = "FourPillarsInput")] options: JsValue,
) -> Result<JsValue, JsValue> {
    let birth: SolarDateTime = from_js(birth)?;
    let input: FourPillarsInput = from_js(options)?;
    let options = FourPillarsOptions {
        tz_offset: input.tz_offset,
        longitude: input.longitude,
        rat_hour: input.rat_hour,
    };
    to_js(&astro_core::four_pillars(birth, &options).map_err(astro_error)?)
}

/// True solar time at `longitude` degrees east for a wall-clock time.
#[wasm_bindgen(js_name = trueSolarTime, unchecked_return_type = "SolarDateTime")]
pub fn true_solar_time(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] clock: JsValue,
    longitude: f64,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let clock: SolarDateTime = from_js(clock)?;
    to_js(
        &astro_core::to_true_solar_time(clock, longitude, zone(tz_offset) * 15.0)
            .map_err(astro_error)?,
    )
}

/// Vietnamese name of a Can Chi pair, e.g. "Giáp Thìn".
#[wasm_bindgen(js_name = canChiName)]
pub fn can_chi_name(
    #[wasm_bindgen(unchecked_param_type = "CanChi")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<CanChi>(value)?.name_vi())
}

/// Hán characters of a Can Chi pair, e.g. "甲辰".
#[wasm_bindgen(js_name = canChiHanName)]
pub fn can_chi_han_name(
    #[wasm_bindgen(unchecked_param_type = "CanChi")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<CanChi>(value)?.name_han())
}

/// Vietnamese name of a solar term, e.g. "Lập Xuân".
#[wasm_bindgen(js_name = solarTermName)]
pub fn solar_term_name(
    #[wasm_bindgen(unchecked_param_type = "SolarTerm")] term: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<SolarTerm>(term)?.name_vi().to_owned())
}

fn zone(tz_offset: Option<f64>) -> f64 {
    tz_offset.unwrap_or(DEFAULT_TZ_OFFSET)
}

/// Malformed input surfaces as the `Error` that serde-wasm-bindgen already built.
fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(JsValue::from)
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(JsValue::from)
}

fn astro_error(error: AstroError) -> JsValue {
    JsError::new(&error.to_string()).into()
}
