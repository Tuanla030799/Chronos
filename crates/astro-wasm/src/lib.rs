//! WebAssembly bridge exposing astro-core to JavaScript and TypeScript.
//!
//! Dates go in and results come out as plain JS objects converted with serde-wasm-bindgen:
//! field names are camelCase and enums are their variant names. The TypeScript shapes are
//! declared below. Every `tz_offset` argument is optional and defaults to 7 (Vietnam, UTC+7).

use astro_core::almanac::{
    AlmanacGod, HoangOc, KimLau, Mansion, PathGod, TamTai, Truc, VietnamTaboo,
};
use astro_core::bazi_analysis::{BaziStar, LifeStage, TenGod};
use astro_core::{
    AstroError, Branch, CanChi, DEFAULT_TZ_OFFSET, FourPillarsOptions, Gender, LunarDate, NaYin,
    RatHourConvention, SolarDate, SolarDateTime, SolarTerm, Stem, YearMonthBoundary,
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
export type Stem =
  | "Giap" | "At" | "Binh" | "Dinh" | "Mau" | "Ky" | "Canh" | "Tan" | "Nham" | "Quy";
export type Branch =
  | "Ty" | "Suu" | "Dan" | "Mao" | "Thin" | "Ti" | "Ngo" | "Mui" | "Than" | "Dau" | "Tuat" | "Hoi";
export interface CanChi { stem: Stem; branch: Branch; }
export type RatHourConvention = "NewDayAtRatHour" | "SplitAtMidnight";
export interface FourPillarsInput {
  /** Defaults to "SplitAtMidnight". */
  ratHour?: RatHourConvention;
  /** Defaults to "SolarTerms". */
  boundary?: YearMonthBoundary;
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
export type NaYin = "HaiTrungKim" | "LuTrungHoa" | "DaiLamMoc" | "LoBangTho" | "KiemPhongKim" | "SonDauHoa" | "GianHaThuy" | "ThanhDauTho" | "BachLapKim" | "DuongLieuMoc" | "TuyenTrungThuy" | "OcThuongTho" | "TichLichHoa" | "TungBachMoc" | "TruongLuuThuy" | "SaTrungKim" | "SonHaHoa" | "BinhDiaMoc" | "BichThuongTho" | "KimBachKim" | "PhuDangHoa" | "ThienHaThuy" | "DaiDichTho" | "ThoaXuyenKim" | "TangDoMoc" | "DaiKheThuy" | "SaTrungTho" | "ThienThuongHoa" | "ThachLuuMoc" | "DaiHaiThuy";
export type YearMonthBoundary = "SolarTerms" | "LunarNewYear";
export type TenGod = "TyKien" | "KiepTai" | "ThucThan" | "ThuongQuan" | "ThienTai" | "ChinhTai" | "ThatSat" | "ChinhQuan" | "ThienAn" | "ChinhAn";
export type LifeStage = "TruongSinh" | "MocDuc" | "QuanDoi" | "LamQuan" | "DeVuong" | "Suy" | "Benh" | "Tu" | "Mo" | "Tuyet" | "Thai" | "Duong";
export type BaziStar = "ThienAtQuyNhan" | "VanXuong" | "LocTon" | "KinhDuong" | "DaLa" | "DichMa" | "DaoHoa" | "HoaCai" | "TuongTinh" | "KiepSat";
export type Gender = "Male" | "Female";
export type PillarPosition = "Year" | "Month" | "Day" | "Hour";
export type StarBasis = "DayStem" | "YearStem" | "DayBranch" | "YearBranch";
export type Luck = "Cat" | "Hung";
export type Truc = "Kien" | "Tru" | "Man" | "Binh" | "Dinh" | "Chap" | "Pha" | "Nguy" | "Thanh" | "Thu" | "Khai" | "Be";
export type Mansion = "GiacMocGiao" | "CangKimLong" | "DeThoLac" | "PhongNhatTho" | "TamNguyetHo" | "ViHoaHo" | "CoThuyBao" | "DauMocGiai" | "NguuKimNguu" | "NuThoBuc" | "HuNhatThu" | "NguyNguyetYen" | "ThatHoaTru" | "BichThuyDu" | "KhueMocLang" | "LauKimCau" | "ViThoTri" | "MaoNhatKe" | "TatNguyetO" | "ChuyHoaHau" | "SamThuyVien" | "TinhMocHan" | "QuyKimDuong" | "LieuThoChuong" | "TinhNhatMa" | "TruongNguyetLoc" | "DucHoaXa" | "ChanThuyDan";
export type PathGod = "ThanhLong" | "MinhDuong" | "ThienHinh" | "ChuTuoc" | "KimQuy" | "ThienDuc" | "BachHo" | "NgocDuong" | "ThienLao" | "HuyenVu" | "TuMenh" | "CauTran";
export type AlmanacGod = "ThienDuc" | "NguyetDuc" | "ThienXa" | "ThienNguyen" | "SinhKhi" | "ThienHy" | "TamHop" | "DichMa" | "NguyetPha" | "ThienCuong" | "HaKhoi" | "NguyetSat" | "KiepSat" | "TaiSat" | "NguyetHinh";
export type VietnamTaboo = "TamNuongSat" | "NguyetKy" | "NguyetTan" | "DuongCongKyNhat" | "ThuTu" | "SatChuDuong" | "SatChuAm" | "VangVong" | "ThienDiaChuyenSat";
export type TamTai = "Nhap" | "Cu" | "Xuat";
export type KimLau = "Than" | "The" | "Tu" | "Suc";
export type HoangOc = "NhatCat" | "NhiNghi" | "TamDiaSat" | "TuTanTai" | "NguThoTu" | "LucHoangOc";
export interface KhongVong {
  first: Branch;
  second: Branch;
}
export interface LuckCycle {
  index: number;
  canChi: CanChi | null;
  startYear: number;
  endYear: number;
  startAge: number;
  endAge: number;
}
export interface DaiVan {
  forward: boolean;
  startYears: number;
  startMonths: number;
  startDays: number;
  startHours: number;
  startTime: SolarDateTime;
  cycles: LuckCycle[];
}
export interface HiddenStem {
  stem: Stem;
  tenGod: TenGod;
}
export interface PillarReading {
  canChi: CanChi;
  naYin: NaYin;
  tenGod: TenGod | null;
  hiddenStems: HiddenStem[];
  lifeStage: LifeStage;
}
export interface StarPlacement {
  star: BaziStar;
  basis: StarBasis;
  pillar: PillarPosition;
}
export interface BaziAnalysis {
  pillars: FourPillars;
  year: PillarReading;
  month: PillarReading;
  day: PillarReading;
  hour: PillarReading;
  stars: StarPlacement[];
  khongVongDay: KhongVong;
  khongVongYear: KhongVong;
  isKhoiCanh: boolean;
  daiVan: DaiVan;
}
export interface AlmanacDay {
  solar: SolarDate;
  lunar: LunarDate;
  dayCanChi: CanChi;
  monthBranch: Branch;
  truc: Truc;
  mansion: Mansion;
  pathGod: PathGod;
  hoangDaoHours: Branch[];
  gods: AlmanacGod[];
  taboos: VietnamTaboo[];
}
export interface AgeRestrictions {
  lunarAge: number;
  tamTai: TamTai | null;
  kimLau: KimLau | null;
  hoangOc: HoangOc | null;
}
"#;

/// Options accepted by `fourPillars` and `analyzeBazi`; every field is optional.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FourPillarsInput {
    #[serde(default)]
    rat_hour: RatHourConvention,
    #[serde(default)]
    boundary: YearMonthBoundary,
    #[serde(default = "default_tz_offset")]
    tz_offset: f64,
    #[serde(default)]
    longitude: Option<f64>,
}

fn default_tz_offset() -> f64 {
    DEFAULT_TZ_OFFSET
}

impl From<FourPillarsInput> for FourPillarsOptions {
    fn from(input: FourPillarsInput) -> Self {
        Self {
            tz_offset: input.tz_offset,
            longitude: input.longitude,
            rat_hour: input.rat_hour,
            boundary: input.boundary,
        }
    }
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
    let options: FourPillarsOptions = from_js::<FourPillarsInput>(options)?.into();
    to_js(&astro_core::four_pillars(birth, &options).map_err(astro_error)?)
}

/// Full reading of a Bát Tự chart: pillars, Ten Gods, life stages, stars and luck cycles.
#[wasm_bindgen(js_name = analyzeBazi, unchecked_return_type = "BaziAnalysis")]
pub fn analyze_bazi(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] birth: JsValue,
    #[wasm_bindgen(unchecked_param_type = "FourPillarsInput")] options: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Gender")] gender: JsValue,
) -> Result<JsValue, JsValue> {
    let birth: SolarDateTime = from_js(birth)?;
    let options: FourPillarsOptions = from_js::<FourPillarsInput>(options)?.into();
    let gender: Gender = from_js(gender)?;
    to_js(&astro_core::analyze_bazi(birth, &options, gender).map_err(astro_error)?)
}

/// Day selection data of a Gregorian date: Trực, mansion, gods and Vietnamese taboos.
#[wasm_bindgen(js_name = almanacDay, unchecked_return_type = "AlmanacDay")]
pub fn almanac_day(
    #[wasm_bindgen(unchecked_param_type = "SolarDate")] date: JsValue,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let date: SolarDate = from_js(date)?;
    to_js(&astro_core::almanac_day(date, zone(tz_offset)).map_err(astro_error)?)
}

/// Tam Tai, Kim Lâu and Hoang Ốc of a person born in one lunar year, in another.
#[wasm_bindgen(js_name = ageRestrictions, unchecked_return_type = "AgeRestrictions")]
pub fn age_restrictions(birth_lunar_year: i32, target_lunar_year: i32) -> Result<JsValue, JsValue> {
    to_js(&astro_core::age_restrictions(
        birth_lunar_year,
        target_lunar_year,
    ))
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

/// The main hidden stem (Bản khí) of a branch.
#[wasm_bindgen(js_name = mainHiddenStem, unchecked_return_type = "Stem")]
pub fn main_hidden_stem(
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&astro_core::bazi_analysis::main_hidden_stem(from_js::<
        Branch,
    >(branch)?))
}

/// Vietnamese name of a Heavenly Stem, e.g. "Giáp".
#[wasm_bindgen(js_name = stemName)]
pub fn stem_name(
    #[wasm_bindgen(unchecked_param_type = "Stem")] stem: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Stem>(stem)?.name_vi().to_owned())
}

/// Vietnamese name of an Earthly Branch, e.g. "Tý".
#[wasm_bindgen(js_name = branchName)]
pub fn branch_name(
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Branch>(branch)?.name_vi().to_owned())
}

/// Vietnamese name of a solar term, e.g. "Lập Xuân".
#[wasm_bindgen(js_name = solarTermName)]
pub fn solar_term_name(
    #[wasm_bindgen(unchecked_param_type = "SolarTerm")] term: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<SolarTerm>(term)?.name_vi().to_owned())
}

/// Vietnamese name of a Nạp Âm.
#[wasm_bindgen(js_name = naYinName)]
pub fn na_yin_name(
    #[wasm_bindgen(unchecked_param_type = "NaYin")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<NaYin>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a Ten God.
#[wasm_bindgen(js_name = tenGodName)]
pub fn ten_god_name(
    #[wasm_bindgen(unchecked_param_type = "TenGod")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TenGod>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a life stage.
#[wasm_bindgen(js_name = lifeStageName)]
pub fn life_stage_name(
    #[wasm_bindgen(unchecked_param_type = "LifeStage")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<LifeStage>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a chart star.
#[wasm_bindgen(js_name = baziStarName)]
pub fn bazi_star_name(
    #[wasm_bindgen(unchecked_param_type = "BaziStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<BaziStar>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a Trực.
#[wasm_bindgen(js_name = trucName)]
pub fn truc_name(
    #[wasm_bindgen(unchecked_param_type = "Truc")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Truc>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a mansion.
#[wasm_bindgen(js_name = mansionName)]
pub fn mansion_name(
    #[wasm_bindgen(unchecked_param_type = "Mansion")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Mansion>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a path god.
#[wasm_bindgen(js_name = pathGodName)]
pub fn path_god_name(
    #[wasm_bindgen(unchecked_param_type = "PathGod")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<PathGod>(value)?.name_vi().to_owned())
}

/// Vietnamese name of an almanac god.
#[wasm_bindgen(js_name = almanacGodName)]
pub fn almanac_god_name(
    #[wasm_bindgen(unchecked_param_type = "AlmanacGod")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<AlmanacGod>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a Vietnamese taboo.
#[wasm_bindgen(js_name = vietnamTabooName)]
pub fn vietnam_taboo_name(
    #[wasm_bindgen(unchecked_param_type = "VietnamTaboo")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<VietnamTaboo>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a Tam Tai year.
#[wasm_bindgen(js_name = tamTaiName)]
pub fn tam_tai_name(
    #[wasm_bindgen(unchecked_param_type = "TamTai")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TamTai>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a Kim Lâu.
#[wasm_bindgen(js_name = kimLauName)]
pub fn kim_lau_name(
    #[wasm_bindgen(unchecked_param_type = "KimLau")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<KimLau>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a Hoang Ốc palace.
#[wasm_bindgen(js_name = hoangOcName)]
pub fn hoang_oc_name(
    #[wasm_bindgen(unchecked_param_type = "HoangOc")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<HoangOc>(value)?.name_vi().to_owned())
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
