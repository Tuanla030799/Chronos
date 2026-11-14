//! WebAssembly bridge exposing astro-core to JavaScript and TypeScript.
//!
//! Dates go in and results come out as plain JS objects converted with serde-wasm-bindgen:
//! field names are camelCase and enums are their variant names. The TypeScript shapes are
//! declared below. Every `tz_offset` argument is optional and defaults to 7 (Vietnam, UTC+7).
//! A feng shui `boundary` may be omitted for `LapXuanDay` and a `forward` for `true`. The options of
//! a Tử Vi chart may be omitted, and so may each of their fields.

use astro_core::almanac::{AgeRestrictions, AlmanacDay};
use astro_core::almanac::{
    AlmanacGod, HoangOc, KimLau, Mansion, PathGod, TaiSui, TamTai, Truc, VietnamTaboo,
};
use astro_core::bazi_analysis::BaziAnalysis;
use astro_core::bazi_analysis::{BaziStar, LifeStage, TenGod};
use astro_core::content::{self, EightUse, Gate};
use astro_core::fengshui::{
    CompassDirection, EightMansionsStar, KuaCompatibility, KuaInfo, LoShuGrid, NineStar,
    NineStarBoundary, NineStarPhase, Palace, TamNguyen, Trigram, TrigramGroup,
};
use astro_core::tuvi::{
    FiveElementsBureau, FourTransformations, LeapMonthRule, MenhCucRelation, StarBrightness,
    StarNature, TuViChart, TuViConfig, TuViPalaceRole, TuViPolarity, TuViStar, YinYangHarmony,
};
use astro_core::{
    AstroError, Branch, CanChi, DEFAULT_TZ_OFFSET, FiveElements, FourPillarsOptions, Gender,
    LunarDate, NaYin, RatHourConvention, SolarDate, SolarDateTime, SolarTerm, Stem,
    YearMonthBoundary,
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
export type TaiSui = "Truc" | "Xung" | "Hai" | "Hinh" | "Pha" | "LucHop" | "TamHop";
export interface TaiSuiReading {
  birthBranch: Branch;
  yearBranch: Branch;
  relations: TaiSui[];
}
export interface AgeRestrictions {
  lunarAge: number;
  tamTai: TamTai | null;
  kimLau: KimLau | null;
  hoangOc: HoangOc | null;
}
export type Trigram = "Kham" | "Khon" | "Chan" | "Ton" | "CanKim" | "Doai" | "CanTho" | "Ly";
export type TrigramGroup = "DongTu" | "TayTu";
export type Palace = "Kham" | "Khon" | "Chan" | "Ton" | "TrungCung" | "CanKim" | "Doai" | "CanTho" | "Ly";
export type CompassDirection = "Bac" | "DongBac" | "Dong" | "DongNam" | "Nam" | "TayNam" | "Tay" | "TayBac";
export type EightMansionsStar = "SinhKhi" | "ThienY" | "DienNien" | "PhucVi" | "TuyetMenh" | "NguQuy" | "LucSat" | "HoaHai";
export type NineStar = "NhatBach" | "NhiHac" | "TamBich" | "TuLuc" | "NguHoang" | "LucBach" | "ThatXich" | "BatBach" | "CuuTu";
export type TamNguyen = "ThuongNguyen" | "TrungNguyen" | "HaNguyen";
export type NineStarBoundary = "LunarNewYear" | "LapXuanDay" | "LapXuanExact";
export type NineStarPhase = "DuongVan" | "SinhKhi" | "ThoaiKhi" | "ThatVan";
export type EightUse = "HuongNha" | "Bep" | "BanLamViec" | "KetHon";
export interface Gate { all: string[]; any: string[][]; none: string[]; }
export interface PassageLookup { anchors: string[]; facts: string[]; }
export interface PalaceQuery { keys: string[]; lookups: PassageLookup[]; }
export type FiveElements = "Moc" | "Hoa" | "Tho" | "Kim" | "Thuy";
export interface DirectionStar {
  direction: CompassDirection;
  star: EightMansionsStar;
}
export interface KuaInfo {
  birthLunarYear: number;
  gender: Gender;
  rawNumber: number;
  number: number;
  trigram: Trigram;
  group: TrigramGroup;
}
export interface KuaCompatibility {
  first: KuaInfo;
  second: KuaInfo;
  star: EightMansionsStar;
  luck: Luck;
}
export interface FlyingStar {
  star: NineStar;
  forward: boolean;
}
export interface NineStars {
  nguyen: TamNguyen;
  van: NineStar;
  year: NineStar;
  month: NineStar;
  day: FlyingStar;
  hour: FlyingStar;
}
export interface LoShuGrid {
  forward: boolean;
  kham: NineStar;
  khon: NineStar;
  chan: NineStar;
  ton: NineStar;
  trungCung: NineStar;
  canKim: NineStar;
  doai: NineStar;
  canTho: NineStar;
  ly: NineStar;
}
export type LeapMonthRule = "SameAsCurrentMonth" | "SplitAt15th";
export interface TuViInput {
  /** Hours east of UTC; defaults to 7 (Vietnam). */
  tzOffset?: number;
  /** Defaults to "SplitAt15th". */
  leapMonthRule?: LeapMonthRule;
  /** Defaults to "SplitAtMidnight". */
  ratHour?: RatHourConvention;
  /** Takes the day and double-hour from true solar time; defaults to false. */
  useTrueSolarTime?: boolean;
  /** Birthplace longitude in degrees east, read only with useTrueSolarTime. */
  longitude?: number | null;
}
export type TuViPalaceRole = "Menh" | "PhuMau" | "PhucDuc" | "DienTrach" | "QuanLoc" | "NoBoc" | "ThienDi" | "TatAch" | "TaiBach" | "TuTuc" | "PhuThe" | "HuynhDe";
export type FiveElementsBureau = "ThuyNhi" | "MocTam" | "KimTu" | "ThoNgu" | "HoaLuc";
export type StarBrightness = "Mieu" | "Vuong" | "Dac" | "Binh" | "Ham";
export type StarCategory = "ChinhTinh" | "PhuTinh";
export type StarNature = "Cat" | "Trung" | "Hung";
export type TuViStar = "TuVi" | "ThienCo" | "ThaiDuong" | "VuKhuc" | "ThienDong" | "LiemTrinh" | "ThienPhu" | "ThaiAm" | "ThamLang" | "CuMon" | "ThienTuong" | "ThienLuong" | "ThatSat" | "PhaQuan" | "ThaiTue" | "ThieuDuong" | "TangMon" | "ThieuAm" | "QuanPhuThaiTue" | "TuPhu" | "TuePha" | "LongDuc" | "BachHo" | "PhucDuc" | "DieuKhach" | "TrucPhu" | "LocTon" | "KinhDuong" | "DaLa" | "BacSy" | "LucSy" | "ThanhLong" | "TieuHao" | "TuongQuan" | "TauThu" | "PhiLiem" | "HyThan" | "BenhPhu" | "DaiHao" | "PhucBinh" | "QuanPhuBacSy" | "TruongSinh" | "MocDuc" | "QuanDoi" | "LamQuan" | "DeVuong" | "Suy" | "Benh" | "Tu" | "Mo" | "Tuyet" | "Thai" | "Duong" | "TaPhu" | "HuuBat" | "ThienHinh" | "ThienRieu" | "ThienY" | "ThienGiai" | "DiaGiai" | "VanXuong" | "VanKhuc" | "DiaKhong" | "DiaKiep" | "ThaiPhu" | "PhongCao" | "TamThai" | "BatToa" | "AnQuang" | "ThienQuy" | "HoaTinh" | "LinhTinh" | "ThienKhoi" | "ThienViet" | "ThienQuan" | "ThienPhuc" | "LuuHa" | "ThienTru" | "QuocAn" | "DuongPhu" | "LongTri" | "PhuongCac" | "GiaiThan" | "HongLoan" | "ThienHy" | "CoThan" | "QuaTu" | "DaoHoa" | "ThienMa" | "KiepSat" | "HoaCai" | "PhaToai" | "ThienKhong" | "ThienKhoc" | "ThienHu" | "ThienDuc" | "NguyetDuc" | "ThienTai" | "ThienTho" | "ThienThuong" | "ThienSu" | "ThienLa" | "DiaVong" | "LuuNienVanTinh" | "DauQuan";
export type FourTransformations = "HoaLoc" | "HoaQuyen" | "HoaKhoa" | "HoaKy";
export type TuViPolarity = "DuongNam" | "AmNam" | "DuongNu" | "AmNu";
export type YinYangHarmony = "ThuanLy" | "NghichLy";
export type MenhCucRelation = "MenhSinhCuc" | "CucSinhMenh" | "MenhKhacCuc" | "CucKhacMenh" | "BinhHoa";
export interface TuViPlacedStar {
  star: TuViStar;
  brightness: StarBrightness | null;
  transformation: FourTransformations | null;
}
export interface TuViPalace {
  branch: Branch;
  stem: Stem;
  role: TuViPalaceRole;
  isThan: boolean;
  mainStars: TuViPlacedStar[];
  minorStars: TuViPlacedStar[];
  lifeStage: LifeStage;
  daiHanStartAge: number;
  tieuHanYearBranch: Branch;
  tieuHanFirstAge: number;
  hasTuan: boolean;
  hasTriet: boolean;
}
export interface TuViChart {
  gender: Gender;
  effectiveTime: SolarDateTime | null;
  solarDate: SolarDate;
  lunar: LunarDate;
  effectiveMonth: number;
  hourBranch: Branch;
  yearCanChi: CanChi;
  monthCanChi: CanChi;
  dayCanChi: CanChi;
  hourCanChi: CanChi;
  polarity: TuViPolarity;
  forward: boolean;
  banMenh: NaYin;
  bureau: FiveElementsBureau;
  menhBranch: Branch;
  thanBranch: Branch;
  thanRole: TuViPalaceRole;
  menhChu: TuViStar;
  thanChu: TuViStar;
  harmony: YinYangHarmony;
  menhCuc: MenhCucRelation;
  tuan: KhongVong;
  triet: KhongVong;
  palaces: TuViPalace[];
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

/// Options accepted by `tuviChart` and `tuviChartFromLunar`; every field is optional.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TuViInput {
    #[serde(default = "default_tz_offset")]
    tz_offset: f64,
    #[serde(default)]
    leap_month_rule: LeapMonthRule,
    #[serde(default)]
    rat_hour: RatHourConvention,
    #[serde(default)]
    use_true_solar_time: bool,
    #[serde(default)]
    longitude: Option<f64>,
}

impl From<TuViInput> for TuViConfig {
    fn from(input: TuViInput) -> Self {
        Self {
            tz_offset: input.tz_offset,
            leap_month_rule: input.leap_month_rule,
            rat_hour: input.rat_hour,
            use_true_solar_time: input.use_true_solar_time,
            longitude: input.longitude,
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

/// Relation to the Thái Tuế of one lunar year of a person born in another.
#[wasm_bindgen(js_name = taiSui, unchecked_return_type = "TaiSuiReading")]
pub fn tai_sui(birth_lunar_year: i32, target_lunar_year: i32) -> Result<JsValue, JsValue> {
    to_js(&astro_core::almanac::tai_sui(
        birth_lunar_year,
        target_lunar_year,
    ))
}

/// Vietnamese name of a Thái Tuế relation, e.g. "Xung Thái Tuế".
#[wasm_bindgen(js_name = taiSuiName)]
pub fn tai_sui_name(
    #[wasm_bindgen(unchecked_param_type = "TaiSui")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TaiSui>(value)?.name_vi().to_owned())
}

/// Fortune of a Thái Tuế relation: Cát for the two Hợp.
#[wasm_bindgen(js_name = taiSuiLuck, unchecked_return_type = "Luck")]
pub fn tai_sui_luck(
    #[wasm_bindgen(unchecked_param_type = "TaiSui")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<TaiSui>(value)?.luck())
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

/// Mệnh Quái of a person born in a lunar year.
#[wasm_bindgen(js_name = kuaInfo, unchecked_return_type = "KuaInfo")]
pub fn kua_info(
    birth_lunar_year: i32,
    #[wasm_bindgen(unchecked_param_type = "Gender")] gender: JsValue,
) -> Result<JsValue, JsValue> {
    let gender: Gender = from_js(gender)?;
    to_js(&astro_core::fengshui::kua_info(birth_lunar_year, gender))
}

/// Eight Mansions star between the Mệnh Quái of two people, with its fortune.
#[wasm_bindgen(js_name = kuaCompatibility, unchecked_return_type = "KuaCompatibility")]
pub fn kua_compatibility(
    #[wasm_bindgen(unchecked_param_type = "KuaInfo")] first: JsValue,
    #[wasm_bindgen(unchecked_param_type = "KuaInfo")] second: JsValue,
) -> Result<JsValue, JsValue> {
    let first: KuaInfo = from_js(first)?;
    let second: KuaInfo = from_js(second)?;
    to_js(&astro_core::fengshui::kua_compatibility(first, second))
}

/// Trạch Quái of a house from its sitting direction (Tọa).
#[wasm_bindgen(js_name = houseTrigram, unchecked_return_type = "Trigram")]
pub fn house_trigram(
    #[wasm_bindgen(unchecked_param_type = "CompassDirection")] sitting: JsValue,
) -> Result<JsValue, JsValue> {
    let sitting: CompassDirection = from_js(sitting)?;
    to_js(&astro_core::fengshui::house_trigram(sitting))
}

/// Trạch Quái of a house from the direction it faces; it sits the opposite way.
#[wasm_bindgen(js_name = houseTrigramFromFacing, unchecked_return_type = "Trigram")]
pub fn house_trigram_from_facing(
    #[wasm_bindgen(unchecked_param_type = "CompassDirection")] facing: JsValue,
) -> Result<JsValue, JsValue> {
    let facing: CompassDirection = from_js(facing)?;
    to_js(&astro_core::fengshui::house_trigram_from_facing(facing))
}

/// The opposite compass direction.
#[wasm_bindgen(js_name = oppositeDirection, unchecked_return_type = "CompassDirection")]
pub fn opposite_direction(
    #[wasm_bindgen(unchecked_param_type = "CompassDirection")] direction: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<CompassDirection>(direction)?.opposite())
}

/// Trigram of a compass direction.
#[wasm_bindgen(js_name = directionTrigram, unchecked_return_type = "Trigram")]
pub fn direction_trigram(
    #[wasm_bindgen(unchecked_param_type = "CompassDirection")] direction: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<CompassDirection>(direction)?.trigram())
}

/// Eight Mansions star between two trigrams; the order does not matter.
#[wasm_bindgen(js_name = starTowards, unchecked_return_type = "EightMansionsStar")]
pub fn star_towards(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] trigram: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Trigram")] other: JsValue,
) -> Result<JsValue, JsValue> {
    let trigram: Trigram = from_js(trigram)?;
    to_js(&trigram.star_towards(from_js(other)?))
}

/// Eight Mansions star in a direction, for a person's Mệnh Quái or a house's Trạch Quái.
#[wasm_bindgen(js_name = starAt, unchecked_return_type = "EightMansionsStar")]
pub fn star_at(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] trigram: JsValue,
    #[wasm_bindgen(unchecked_param_type = "CompassDirection")] direction: JsValue,
) -> Result<JsValue, JsValue> {
    let trigram: Trigram = from_js(trigram)?;
    to_js(&trigram.star_at(from_js(direction)?))
}

/// Eight Mansions stars of a trigram in the eight directions, clockwise from north.
#[wasm_bindgen(js_name = eightMansionsChart, unchecked_return_type = "DirectionStar[]")]
pub fn eight_mansions_chart(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] trigram: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Trigram>(trigram)?.chart())
}

// `tz_offset` repeats the attribute of `boundary`: wasm-bindgen drops the `?` of a plain `Option`
// parameter when a parameter with a type override follows it, and `nineStars(time)` would not
// type-check.

/// Era, period and stars ruling a wall-clock time.
#[wasm_bindgen(js_name = nineStars, unchecked_return_type = "NineStars")]
pub fn nine_stars(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] time: JsValue,
    #[wasm_bindgen(unchecked_optional_param_type = "number | null")] tz_offset: Option<f64>,
    #[wasm_bindgen(unchecked_optional_param_type = "NineStarBoundary | null")] boundary: JsValue,
) -> Result<JsValue, JsValue> {
    let time: SolarDateTime = from_js(time)?;
    let boundary = star_boundary(boundary)?;
    to_js(&astro_core::fengshui::nine_stars(time, zone(tz_offset), boundary).map_err(astro_error)?)
}

/// Year star (Niên Tinh) at a wall-clock time.
#[wasm_bindgen(js_name = yearNineStar, unchecked_return_type = "NineStar")]
pub fn year_nine_star(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] time: JsValue,
    #[wasm_bindgen(unchecked_optional_param_type = "number | null")] tz_offset: Option<f64>,
    #[wasm_bindgen(unchecked_optional_param_type = "NineStarBoundary | null")] boundary: JsValue,
) -> Result<JsValue, JsValue> {
    let time: SolarDateTime = from_js(time)?;
    let boundary = star_boundary(boundary)?;
    to_js(
        &astro_core::fengshui::year_nine_star(time, zone(tz_offset), boundary)
            .map_err(astro_error)?,
    )
}

/// Month star (Nguyệt Tinh) at a wall-clock time.
#[wasm_bindgen(js_name = monthNineStar, unchecked_return_type = "NineStar")]
pub fn month_nine_star(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] time: JsValue,
    #[wasm_bindgen(unchecked_optional_param_type = "number | null")] tz_offset: Option<f64>,
    #[wasm_bindgen(unchecked_optional_param_type = "NineStarBoundary | null")] boundary: JsValue,
) -> Result<JsValue, JsValue> {
    let time: SolarDateTime = from_js(time)?;
    let boundary = star_boundary(boundary)?;
    to_js(
        &astro_core::fengshui::month_nine_star(time, zone(tz_offset), boundary)
            .map_err(astro_error)?,
    )
}

/// Day star (Nhật Tinh) of a Gregorian date, with its flying direction.
#[wasm_bindgen(js_name = dayNineStar, unchecked_return_type = "FlyingStar")]
pub fn day_nine_star(
    #[wasm_bindgen(unchecked_param_type = "SolarDate")] date: JsValue,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let date: SolarDate = from_js(date)?;
    to_js(&astro_core::fengshui::day_nine_star(date, zone(tz_offset)).map_err(astro_error)?)
}

/// Hour star (Thời Tinh) at a wall-clock time, with its flying direction.
#[wasm_bindgen(js_name = hourNineStar, unchecked_return_type = "FlyingStar")]
pub fn hour_nine_star(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] time: JsValue,
    tz_offset: Option<f64>,
) -> Result<JsValue, JsValue> {
    let time: SolarDateTime = from_js(time)?;
    to_js(&astro_core::fengshui::hour_nine_star(time, zone(tz_offset)).map_err(astro_error)?)
}

/// Year star (Niên Tinh) of a year number, e.g. Tam Bích for 2024.
#[wasm_bindgen(js_name = starOfYear, unchecked_return_type = "NineStar")]
pub fn star_of_year(year: i32) -> Result<JsValue, JsValue> {
    to_js(&astro_core::fengshui::star_of_year(year))
}

/// Ruling star of the 20-year period (Vận) of a year number, e.g. Cửu Tử for 2024.
#[wasm_bindgen(js_name = vanOfYear, unchecked_return_type = "NineStar")]
pub fn van_of_year(year: i32) -> Result<JsValue, JsValue> {
    to_js(&astro_core::fengshui::van_of_year(year))
}

/// Era (Nguyên) of a year number, e.g. Hạ Nguyên for 2024.
#[wasm_bindgen(js_name = nguyenOfYear, unchecked_return_type = "TamNguyen")]
pub fn nguyen_of_year(year: i32) -> Result<JsValue, JsValue> {
    to_js(&astro_core::fengshui::nguyen_of_year(year))
}

/// Lo Shu grid with a star in the center, flying forward unless `forward` is false.
#[wasm_bindgen(js_name = generateLoshuGrid, unchecked_return_type = "LoShuGrid")]
pub fn generate_loshu_grid(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] center_star: JsValue,
    forward: Option<bool>,
) -> Result<JsValue, JsValue> {
    let center_star: NineStar = from_js(center_star)?;
    to_js(&astro_core::fengshui::generate_loshu_grid(
        center_star,
        forward.unwrap_or(true),
    ))
}

/// Star in one palace of a Lo Shu grid.
#[wasm_bindgen(js_name = loShuStarAt, unchecked_return_type = "NineStar")]
pub fn lo_shu_star_at(
    #[wasm_bindgen(unchecked_param_type = "LoShuGrid")] grid: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Palace")] palace: JsValue,
) -> Result<JsValue, JsValue> {
    let grid: LoShuGrid = from_js(grid)?;
    to_js(&grid.star_at(from_js(palace)?))
}

/// Three rows of three stars of a Lo Shu grid, south on top.
#[wasm_bindgen(js_name = loShuRows, unchecked_return_type = "NineStar[][]")]
pub fn lo_shu_rows(
    #[wasm_bindgen(unchecked_param_type = "LoShuGrid")] grid: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<LoShuGrid>(grid)?.rows())
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

/// Element of a Heavenly Stem.
#[wasm_bindgen(js_name = stemElement, unchecked_return_type = "FiveElements")]
pub fn stem_element(
    #[wasm_bindgen(unchecked_param_type = "Stem")] stem: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Stem>(stem)?.element())
}

/// Element of an Earthly Branch.
#[wasm_bindgen(js_name = branchElement, unchecked_return_type = "FiveElements")]
pub fn branch_element(
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Branch>(branch)?.element())
}

/// Element of a Nạp Âm.
#[wasm_bindgen(js_name = naYinElement, unchecked_return_type = "FiveElements")]
pub fn na_yin_element(
    #[wasm_bindgen(unchecked_param_type = "NaYin")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<NaYin>(value)?.element())
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

/// Vietnamese name of a trigram, e.g. "Khảm".
#[wasm_bindgen(js_name = trigramName)]
pub fn trigram_name(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Trigram>(value)?.name_vi().to_owned())
}

/// Hán character of a trigram, e.g. "坎".
#[wasm_bindgen(js_name = trigramHanName)]
pub fn trigram_han_name(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Trigram>(value)?.name_han().to_owned())
}

/// Lạc Thư number of a trigram: 1–4 or 6–9.
#[wasm_bindgen(js_name = trigramNumber)]
pub fn trigram_number(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] value: JsValue,
) -> Result<u8, JsValue> {
    Ok(from_js::<Trigram>(value)?.number())
}

/// Element of a trigram.
#[wasm_bindgen(js_name = trigramElement, unchecked_return_type = "FiveElements")]
pub fn trigram_element(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Trigram>(value)?.element())
}

/// Group of a trigram: Đông Tứ or Tây Tứ.
#[wasm_bindgen(js_name = trigramGroup, unchecked_return_type = "TrigramGroup")]
pub fn trigram_group(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Trigram>(value)?.group())
}

/// Compass direction of a trigram.
#[wasm_bindgen(js_name = trigramDirection, unchecked_return_type = "CompassDirection")]
pub fn trigram_direction(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Trigram>(value)?.direction())
}

/// Palace of the Lo Shu grid held by a trigram.
#[wasm_bindgen(js_name = trigramPalace, unchecked_return_type = "Palace")]
pub fn trigram_palace(
    #[wasm_bindgen(unchecked_param_type = "Trigram")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Trigram>(value)?.palace())
}

/// Vietnamese name of a trigram group, "Đông Tứ" or "Tây Tứ".
#[wasm_bindgen(js_name = trigramGroupName)]
pub fn trigram_group_name(
    #[wasm_bindgen(unchecked_param_type = "TrigramGroup")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TrigramGroup>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a palace, e.g. "Trung Cung".
#[wasm_bindgen(js_name = palaceName)]
pub fn palace_name(
    #[wasm_bindgen(unchecked_param_type = "Palace")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Palace>(value)?.name_vi().to_owned())
}

/// Hán character of a palace, e.g. "中".
#[wasm_bindgen(js_name = palaceHanName)]
pub fn palace_han_name(
    #[wasm_bindgen(unchecked_param_type = "Palace")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<Palace>(value)?.name_han().to_owned())
}

/// Lạc Thư number of a palace, from 1 (Khảm) to 9 (Ly).
#[wasm_bindgen(js_name = palaceNumber)]
pub fn palace_number(
    #[wasm_bindgen(unchecked_param_type = "Palace")] value: JsValue,
) -> Result<u8, JsValue> {
    Ok(from_js::<Palace>(value)?.number())
}

/// Element of a palace.
#[wasm_bindgen(js_name = palaceElement, unchecked_return_type = "FiveElements")]
pub fn palace_element(
    #[wasm_bindgen(unchecked_param_type = "Palace")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Palace>(value)?.element())
}

/// Trigram of a palace; the center has none, so it gives null.
#[wasm_bindgen(js_name = palaceTrigram, unchecked_return_type = "Trigram | null")]
pub fn palace_trigram(
    #[wasm_bindgen(unchecked_param_type = "Palace")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Palace>(value)?.trigram())
}

/// Compass direction of a palace; the center has none, so it gives null.
#[wasm_bindgen(js_name = palaceDirection, unchecked_return_type = "CompassDirection | null")]
pub fn palace_direction(
    #[wasm_bindgen(unchecked_param_type = "Palace")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Palace>(value)?.direction())
}

/// Vietnamese name of a compass direction, e.g. "Đông Bắc".
#[wasm_bindgen(js_name = compassDirectionName)]
pub fn compass_direction_name(
    #[wasm_bindgen(unchecked_param_type = "CompassDirection")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<CompassDirection>(value)?.name_vi().to_owned())
}

/// Vietnamese name of an Eight Mansions star, e.g. "Sinh Khí".
#[wasm_bindgen(js_name = eightMansionsStarName)]
pub fn eight_mansions_star_name(
    #[wasm_bindgen(unchecked_param_type = "EightMansionsStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<EightMansionsStar>(value)?.name_vi().to_owned())
}

/// Hán characters of an Eight Mansions star, e.g. "生气".
#[wasm_bindgen(js_name = eightMansionsStarHanName)]
pub fn eight_mansions_star_han_name(
    #[wasm_bindgen(unchecked_param_type = "EightMansionsStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<EightMansionsStar>(value)?.name_han().to_owned())
}

/// Name of the star corresponding to an Eight Mansions star, e.g. "Tham Lang" for Sinh Khí.
#[wasm_bindgen(js_name = eightMansionsStarCorrespondingName)]
pub fn eight_mansions_star_corresponding_name(
    #[wasm_bindgen(unchecked_param_type = "EightMansionsStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<EightMansionsStar>(value)?
        .corresponding_star_name()
        .to_owned())
}

/// Fortune of an Eight Mansions star.
#[wasm_bindgen(js_name = eightMansionsStarLuck, unchecked_return_type = "Luck")]
pub fn eight_mansions_star_luck(
    #[wasm_bindgen(unchecked_param_type = "EightMansionsStar")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<EightMansionsStar>(value)?.luck())
}

/// Vietnamese name of a Nine Star, e.g. "Nhất Bạch Thủy".
#[wasm_bindgen(js_name = nineStarName)]
pub fn nine_star_name(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<NineStar>(value)?.name_vi().to_owned())
}

/// Hán characters of a Nine Star, e.g. "一白水".
#[wasm_bindgen(js_name = nineStarHanName)]
pub fn nine_star_han_name(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<NineStar>(value)?.name_han().to_owned())
}

/// Number of a Nine Star, from 1 to 9.
#[wasm_bindgen(js_name = nineStarNumber)]
pub fn nine_star_number(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<u8, JsValue> {
    Ok(from_js::<NineStar>(value)?.number())
}

/// Element of a Nine Star.
#[wasm_bindgen(js_name = nineStarElement, unchecked_return_type = "FiveElements")]
pub fn nine_star_element(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<NineStar>(value)?.element())
}

/// Vietnamese name of the color of a Nine Star, e.g. "Xanh biếc".
#[wasm_bindgen(js_name = nineStarColorName)]
pub fn nine_star_color_name(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<NineStar>(value)?.color_vi().to_owned())
}

/// Palace where a Nine Star belongs in the original Lạc Thư square.
#[wasm_bindgen(js_name = nineStarHomePalace, unchecked_return_type = "Palace")]
pub fn nine_star_home_palace(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<NineStar>(value)?.home_palace())
}

/// Fortune of a Nine Star, fixed rather than relative to the current era.
#[wasm_bindgen(js_name = nineStarLuck, unchecked_return_type = "Luck")]
pub fn nine_star_luck(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<NineStar>(value)?.luck())
}

/// Name of a Nine Star in Huyền Không, e.g. "Tham Lang".
#[wasm_bindgen(js_name = nineStarXuanKongName)]
pub fn nine_star_xuan_kong_name(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<NineStar>(value)?.xuan_kong_name().to_owned())
}

/// Vietnamese name of an era, e.g. "Hạ Nguyên".
#[wasm_bindgen(js_name = tamNguyenName)]
pub fn tam_nguyen_name(
    #[wasm_bindgen(unchecked_param_type = "TamNguyen")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TamNguyen>(value)?.name_vi().to_owned())
}

/// Vietnamese name of an element, e.g. "Mộc".
#[wasm_bindgen(js_name = fiveElementsName)]
pub fn five_elements_name(
    #[wasm_bindgen(unchecked_param_type = "FiveElements")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<FiveElements>(value)?.name_vi().to_owned())
}

/// Tử Vi chart of a birth at a wall-clock time.
#[wasm_bindgen(js_name = tuviChart, unchecked_return_type = "TuViChart")]
pub fn tuvi_chart(
    #[wasm_bindgen(unchecked_param_type = "SolarDateTime")] birth: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Gender")] gender: JsValue,
    #[wasm_bindgen(unchecked_optional_param_type = "TuViInput | null")] options: JsValue,
) -> Result<JsValue, JsValue> {
    let birth: SolarDateTime = from_js(birth)?;
    let gender: Gender = from_js(gender)?;
    let config = tuvi_config(options)?;
    to_js(&astro_core::tuvi_chart(birth, gender, &config).map_err(astro_error)?)
}

/// Tử Vi chart of a birth on a lunar date at a double-hour.
#[wasm_bindgen(js_name = tuviChartFromLunar, unchecked_return_type = "TuViChart")]
pub fn tuvi_chart_from_lunar(
    #[wasm_bindgen(unchecked_param_type = "LunarDate")] lunar: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Branch")] hour: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Gender")] gender: JsValue,
    #[wasm_bindgen(unchecked_optional_param_type = "TuViInput | null")] options: JsValue,
) -> Result<JsValue, JsValue> {
    let lunar: LunarDate = from_js(lunar)?;
    let hour: Branch = from_js(hour)?;
    let gender: Gender = from_js(gender)?;
    let config = tuvi_config(options)?;
    to_js(&astro_core::tuvi_chart_from_lunar(lunar, hour, gender, &config).map_err(astro_error)?)
}

/// Palace of the Đại Hạn at tuổi mụ `age`, or `null`.
#[wasm_bindgen(js_name = tuviDaiHanPalace, unchecked_return_type = "TuViPalace | null")]
pub fn tuvi_dai_han_palace(
    #[wasm_bindgen(unchecked_param_type = "TuViChart")] chart: JsValue,
    age: u8,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<TuViChart>(chart)?.dai_han_palace(age))
}

/// Palace of the Tiểu Hạn at tuổi mụ `age`, or `null`.
#[wasm_bindgen(js_name = tuviTieuHanPalace, unchecked_return_type = "TuViPalace | null")]
pub fn tuvi_tieu_han_palace(
    #[wasm_bindgen(unchecked_param_type = "TuViChart")] chart: JsValue,
    age: u8,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<TuViChart>(chart)?.tieu_han_palace(age))
}

/// Every star of a Tử Vi chart, in the order of the enum.
#[wasm_bindgen(js_name = tuviAllStars, unchecked_return_type = "TuViStar[]")]
pub fn tuvi_all_stars() -> Result<JsValue, JsValue> {
    to_js(&TuViStar::ALL.to_vec())
}

/// Whether a star is one of the 14 Chính Tinh.
#[wasm_bindgen(js_name = tuviStarCategory, unchecked_return_type = "StarCategory")]
pub fn tuvi_star_category(
    #[wasm_bindgen(unchecked_param_type = "TuViStar")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<TuViStar>(value)?.category())
}

/// Element of a Chính Tinh; `null` for every other star.
#[wasm_bindgen(js_name = tuviStarElement, unchecked_return_type = "FiveElements | null")]
pub fn tuvi_star_element(
    #[wasm_bindgen(unchecked_param_type = "TuViStar")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<TuViStar>(value)?.element())
}

/// The three branches of a branch's Tam Hợp group, the branch itself included.
#[wasm_bindgen(js_name = branchTrine, unchecked_return_type = "Branch[]")]
pub fn branch_trine(
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Branch>(branch)?.trine())
}

/// The branch opposite a branch (Lục Xung, xung chiếu).
#[wasm_bindgen(js_name = branchClash, unchecked_return_type = "Branch")]
pub fn branch_clash(
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<Branch>(branch)?.clash())
}

/// Inherent nature of a star, from section 4.9 of the spec.
#[wasm_bindgen(js_name = tuviStarNature, unchecked_return_type = "StarNature")]
pub fn tuvi_star_nature(
    #[wasm_bindgen(unchecked_param_type = "TuViStar")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<TuViStar>(value)?.nature())
}

/// Brightness of a star in a palace; only the Chính Tinh have one.
#[wasm_bindgen(js_name = tuviStarBrightness, unchecked_return_type = "StarBrightness | null")]
pub fn tuvi_star_brightness(
    #[wasm_bindgen(unchecked_param_type = "TuViStar")] value: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<JsValue, JsValue> {
    let branch: Branch = from_js(branch)?;
    to_js(&from_js::<TuViStar>(value)?.brightness_at(branch))
}

/// The star that takes a transformation in a year with stem `yearStem`.
#[wasm_bindgen(js_name = fourTransformationStar, unchecked_return_type = "TuViStar")]
pub fn four_transformation_star(
    #[wasm_bindgen(unchecked_param_type = "FourTransformations")] value: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Stem")] year_stem: JsValue,
) -> Result<JsValue, JsValue> {
    let year_stem: Stem = from_js(year_stem)?;
    to_js(&from_js::<FourTransformations>(value)?.star_for(year_stem))
}

/// Inherent nature of a transformation.
#[wasm_bindgen(js_name = fourTransformationNature, unchecked_return_type = "StarNature")]
pub fn four_transformation_nature(
    #[wasm_bindgen(unchecked_param_type = "FourTransformations")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<FourTransformations>(value)?.nature())
}

/// Number of a bureau, from 2 to 6.
#[wasm_bindgen(js_name = fiveElementsBureauNumber)]
pub fn five_elements_bureau_number(
    #[wasm_bindgen(unchecked_param_type = "FiveElementsBureau")] value: JsValue,
) -> Result<u8, JsValue> {
    Ok(from_js::<FiveElementsBureau>(value)?.number())
}

/// Element of a bureau.
#[wasm_bindgen(js_name = fiveElementsBureauElement, unchecked_return_type = "FiveElements")]
pub fn five_elements_bureau_element(
    #[wasm_bindgen(unchecked_param_type = "FiveElementsBureau")] value: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<FiveElementsBureau>(value)?.element())
}

/// Vietnamese name of a star, e.g. "Thiên Phủ".
#[wasm_bindgen(js_name = tuviStarName)]
pub fn tuvi_star_name(
    #[wasm_bindgen(unchecked_param_type = "TuViStar")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TuViStar>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a brightness, e.g. "Miếu".
#[wasm_bindgen(js_name = starBrightnessName)]
pub fn star_brightness_name(
    #[wasm_bindgen(unchecked_param_type = "StarBrightness")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<StarBrightness>(value)?.name_vi().to_owned())
}

/// Mark of a brightness on a chart: M, V, Đ, B or H.
#[wasm_bindgen(js_name = starBrightnessMark)]
pub fn star_brightness_mark(
    #[wasm_bindgen(unchecked_param_type = "StarBrightness")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<StarBrightness>(value)?.mark().to_owned())
}

/// Vietnamese name of a star's nature, e.g. "Cát".
#[wasm_bindgen(js_name = starNatureName)]
pub fn star_nature_name(
    #[wasm_bindgen(unchecked_param_type = "StarNature")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<StarNature>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a transformation, e.g. "Hóa Lộc".
#[wasm_bindgen(js_name = fourTransformationName)]
pub fn four_transformation_name(
    #[wasm_bindgen(unchecked_param_type = "FourTransformations")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<FourTransformations>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a functional palace, e.g. "Phụ Mẫu".
#[wasm_bindgen(js_name = tuviPalaceRoleName)]
pub fn tuvi_palace_role_name(
    #[wasm_bindgen(unchecked_param_type = "TuViPalaceRole")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TuViPalaceRole>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a bureau, e.g. "Mộc Tam Cục".
#[wasm_bindgen(js_name = fiveElementsBureauName)]
pub fn five_elements_bureau_name(
    #[wasm_bindgen(unchecked_param_type = "FiveElementsBureau")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<FiveElementsBureau>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a polarity, e.g. "Dương Nam".
#[wasm_bindgen(js_name = tuviPolarityName)]
pub fn tuvi_polarity_name(
    #[wasm_bindgen(unchecked_param_type = "TuViPolarity")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<TuViPolarity>(value)?.name_vi().to_owned())
}

/// Vietnamese name of a harmony, e.g. "Thuận lý".
#[wasm_bindgen(js_name = yinYangHarmonyName)]
pub fn yin_yang_harmony_name(
    #[wasm_bindgen(unchecked_param_type = "YinYangHarmony")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<YinYangHarmony>(value)?.name_vi().to_owned())
}

/// Vietnamese name of the relation between Bản Mệnh and bureau, e.g. "Bản Mệnh sinh Cục".
#[wasm_bindgen(js_name = menhCucRelationName)]
pub fn menh_cuc_relation_name(
    #[wasm_bindgen(unchecked_param_type = "MenhCucRelation")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<MenhCucRelation>(value)?.name_vi().to_owned())
}

fn zone(tz_offset: Option<f64>) -> f64 {
    tz_offset.unwrap_or(DEFAULT_TZ_OFFSET)
}

/// `undefined` and `null` select the default; an object may leave out any field.
fn tuvi_config(options: JsValue) -> Result<TuViConfig, JsValue> {
    Ok(from_js::<Option<TuViInput>>(options)?.map_or_else(TuViConfig::default, TuViConfig::from))
}

/// `undefined` and `null` select the default; anything else must name a boundary.
/// Phase of a Nine Star in the period ruled by `period`.
#[wasm_bindgen(js_name = nineStarPhase, unchecked_return_type = "NineStarPhase")]
pub fn nine_star_phase(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] star: JsValue,
    #[wasm_bindgen(unchecked_param_type = "NineStar")] period: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<NineStar>(star)?.phase_in(from_js(period)?))
}

/// Vietnamese name of a phase, e.g. "Thoái khí".
#[wasm_bindgen(js_name = nineStarPhaseName)]
pub fn nine_star_phase_name(
    #[wasm_bindgen(unchecked_param_type = "NineStarPhase")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<NineStarPhase>(value)?.name_vi().to_owned())
}

/// Whether the auspicious text of a Nine Star applies in this phase.
#[wasm_bindgen(js_name = nineStarPhaseIsAuspicious)]
pub fn nine_star_phase_is_auspicious(
    #[wasm_bindgen(unchecked_param_type = "NineStarPhase")] value: JsValue,
) -> Result<bool, JsValue> {
    Ok(from_js::<NineStarPhase>(value)?.is_auspicious())
}

/// Brightness of a transformation in a palace; only Hóa Kỵ has one.
#[wasm_bindgen(
    js_name = fourTransformationBrightness,
    unchecked_return_type = "StarBrightness | null"
)]
pub fn four_transformation_brightness(
    #[wasm_bindgen(unchecked_param_type = "FourTransformations")] value: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<JsValue, JsValue> {
    to_js(&from_js::<FourTransformations>(value)?.brightness_at(from_js(branch)?))
}

/// Interpretation keys of a day.
#[wasm_bindgen(js_name = almanacDayKeys)]
pub fn almanac_day_keys(
    #[wasm_bindgen(unchecked_param_type = "AlmanacDay")] day: JsValue,
) -> Result<Vec<String>, JsValue> {
    Ok(content::almanac_day_keys(&from_js::<AlmanacDay>(day)?))
}

/// Interpretation keys of the age restrictions in force.
#[wasm_bindgen(js_name = ageRestrictionKeys)]
pub fn age_restriction_keys(
    #[wasm_bindgen(unchecked_param_type = "AgeRestrictions")] restrictions: JsValue,
) -> Result<Vec<String>, JsValue> {
    Ok(content::age_restriction_keys(&from_js::<AgeRestrictions>(
        restrictions,
    )?))
}

/// Interpretation keys of a Bát Tự reading.
#[wasm_bindgen(js_name = baziKeys)]
pub fn bazi_keys(
    #[wasm_bindgen(unchecked_param_type = "BaziAnalysis")] analysis: JsValue,
) -> Result<Vec<String>, JsValue> {
    Ok(content::bazi_keys(&from_js::<BaziAnalysis>(analysis)?))
}

/// Interpretation key of the Day Master text of a Heavenly Stem.
#[wasm_bindgen(js_name = baziDayMasterKey)]
pub fn bazi_day_master_key(
    #[wasm_bindgen(unchecked_param_type = "Stem")] stem: JsValue,
) -> Result<String, JsValue> {
    Ok(content::bazi_day_master_key(from_js(stem)?))
}

/// Interpretation key of a Nạp Âm.
#[wasm_bindgen(js_name = baziNaYinKey)]
pub fn bazi_na_yin_key(
    #[wasm_bindgen(unchecked_param_type = "NaYin")] value: JsValue,
) -> Result<String, JsValue> {
    Ok(content::bazi_na_yin_key(from_js(value)?))
}

/// Interpretation key of a Ten God.
#[wasm_bindgen(js_name = baziTenGodKey)]
pub fn bazi_ten_god_key(
    #[wasm_bindgen(unchecked_param_type = "TenGod")] god: JsValue,
) -> Result<String, JsValue> {
    Ok(content::bazi_ten_god_key(from_js(god)?))
}

/// Interpretation key of a life stage.
#[wasm_bindgen(js_name = baziLifeStageKey)]
pub fn bazi_life_stage_key(
    #[wasm_bindgen(unchecked_param_type = "LifeStage")] stage: JsValue,
) -> Result<String, JsValue> {
    Ok(content::bazi_life_stage_key(from_js(stage)?))
}

/// Interpretation key of a Thần Sát of a chart.
#[wasm_bindgen(js_name = baziShenShaKey)]
pub fn bazi_shen_sha_key(
    #[wasm_bindgen(unchecked_param_type = "BaziStar")] star: JsValue,
) -> Result<String, JsValue> {
    Ok(content::bazi_shen_sha_key(from_js(star)?))
}

/// Interpretation key of Không Vong.
#[wasm_bindgen(js_name = baziKhongVongKey)]
pub fn bazi_khong_vong_key() -> String {
    content::bazi_khong_vong_key()
}

/// Interpretation key of Khôi Canh.
#[wasm_bindgen(js_name = baziKhoiCanhKey)]
pub fn bazi_khoi_canh_key() -> String {
    content::bazi_khoi_canh_key()
}

/// Interpretation key of a person's trigram.
#[wasm_bindgen(js_name = kuaKey)]
pub fn kua_key(
    #[wasm_bindgen(unchecked_param_type = "KuaInfo")] kua: JsValue,
) -> Result<String, JsValue> {
    Ok(content::kua_key(&from_js::<KuaInfo>(kua)?))
}

/// Interpretation key of a star of the Eight Mansions.
#[wasm_bindgen(js_name = eightStarKey)]
pub fn eight_star_key(
    #[wasm_bindgen(unchecked_param_type = "EightMansionsStar")] star: JsValue,
) -> Result<String, JsValue> {
    Ok(content::eight_star_key(from_js(star)?))
}

/// Vietnamese name of a use of the Eight Mansions, e.g. "Hướng nhà".
#[wasm_bindgen(js_name = eightUseName)]
pub fn eight_use_name(
    #[wasm_bindgen(unchecked_param_type = "EightUse")] usage: JsValue,
) -> Result<String, JsValue> {
    Ok(from_js::<EightUse>(usage)?.name_vi().to_owned())
}

/// Interpretation key of a star of the Eight Mansions in one use.
#[wasm_bindgen(js_name = eightStarUseKey)]
pub fn eight_star_use_key(
    #[wasm_bindgen(unchecked_param_type = "EightMansionsStar")] star: JsValue,
    #[wasm_bindgen(unchecked_param_type = "EightUse")] usage: JsValue,
) -> Result<String, JsValue> {
    let usage: EightUse = from_js(usage)?;
    Ok(content::eight_star_use_key(from_js(star)?, usage))
}

/// Interpretation keys of the compatibility of two people.
#[wasm_bindgen(js_name = kuaCompatibilityKeys)]
pub fn kua_compatibility_keys(
    #[wasm_bindgen(unchecked_param_type = "KuaCompatibility")] compatibility: JsValue,
) -> Result<Vec<String>, JsValue> {
    Ok(content::kua_compatibility_keys(
        &from_js::<KuaCompatibility>(compatibility)?,
    ))
}

/// Interpretation key of a Nine Star.
#[wasm_bindgen(js_name = nineStarKey)]
pub fn nine_star_key(
    #[wasm_bindgen(unchecked_param_type = "NineStar")] star: JsValue,
) -> Result<String, JsValue> {
    Ok(content::nine_star_key(from_js(star)?))
}

/// Interpretation key of the overview of a palace role.
#[wasm_bindgen(js_name = tuviPalaceKey)]
pub fn tuvi_palace_key(
    #[wasm_bindgen(unchecked_param_type = "TuViPalaceRole")] role: JsValue,
) -> Result<String, JsValue> {
    Ok(content::tuvi_palace_key(from_js(role)?))
}

/// Interpretation key of the general text of a Tử Vi star.
#[wasm_bindgen(js_name = tuviStarKey)]
pub fn tuvi_star_key(
    #[wasm_bindgen(unchecked_param_type = "TuViStar")] star: JsValue,
) -> Result<String, JsValue> {
    Ok(content::tuvi_star_key(from_js(star)?))
}

/// Interpretation key of a star at one brightness.
#[wasm_bindgen(js_name = tuviStarBrightnessKey)]
pub fn tuvi_star_brightness_key(
    #[wasm_bindgen(unchecked_param_type = "TuViStar")] star: JsValue,
    #[wasm_bindgen(unchecked_param_type = "StarBrightness")] level: JsValue,
) -> Result<String, JsValue> {
    let level: StarBrightness = from_js(level)?;
    Ok(content::tuvi_star_brightness_key(from_js(star)?, level))
}

/// Interpretation key of one of the Tứ Hóa.
#[wasm_bindgen(js_name = tuviTransformationKey)]
pub fn tuvi_transformation_key(
    #[wasm_bindgen(unchecked_param_type = "FourTransformations")] transformation: JsValue,
) -> Result<String, JsValue> {
    Ok(content::tuvi_transformation_key(from_js(transformation)?))
}

/// What the interpretation of the palace at `branch` needs.
#[wasm_bindgen(js_name = tuviPalaceQuery, unchecked_return_type = "PalaceQuery")]
pub fn tuvi_palace_query(
    #[wasm_bindgen(unchecked_param_type = "TuViChart")] chart: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Branch")] branch: JsValue,
) -> Result<JsValue, JsValue> {
    let chart: TuViChart = from_js(chart)?;
    to_js(&content::tuvi_palace_query(&chart, from_js(branch)?))
}

/// Whether a passage applies to `facts`; `gates` is the `gates` field of the content file.
#[wasm_bindgen(js_name = gateMatches)]
pub fn gate_matches(
    #[wasm_bindgen(unchecked_param_type = "Gate[]")] gates: JsValue,
    #[wasm_bindgen(unchecked_param_type = "string[]")] facts: JsValue,
) -> Result<bool, JsValue> {
    let gates: Vec<Gate> = from_js(gates)?;
    let facts: Vec<String> = from_js(facts)?;
    Ok(content::gate_matches(&gates, &facts))
}

fn star_boundary(boundary: JsValue) -> Result<NineStarBoundary, JsValue> {
    Ok(from_js::<Option<NineStarBoundary>>(boundary)?.unwrap_or_default())
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
