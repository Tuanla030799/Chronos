//! UniFFI bridge exposing astro-core to Swift (iOS) and the other UniFFI languages.
//!
//! The core types are mirrored with `#[uniffi::remote]`, so foreign code receives exactly the
//! records and enums of astro-core with no conversion layer. Every `tz_offset` argument
//! defaults to 7.0 (Vietnam, UTC+7) in the generated bindings. UniFFI has no literal default
//! for an enum, so every `NineStarBoundary` argument is optional instead: omitting it means
//! `NineStarBoundary::default()`. Likewise, omitting a `TuViConfig` means `TuViConfig::default()`.

use astro_core::almanac::{
    AgeRestrictions, AlmanacDay, AlmanacGod, HoangOc, KimLau, Luck, Mansion, PathGod, TamTai, Truc,
    VietnamTaboo,
};
use astro_core::bazi_analysis::{
    BaziAnalysis, BaziStar, DaiVan, Gender, HiddenStem, KhongVong, LifeStage, LuckCycle,
    PillarPosition, PillarReading, StarBasis, StarPlacement, TenGod,
};
use astro_core::fengshui::{
    CompassDirection, DirectionStar, EightMansionsStar, FlyingStar, KuaCompatibility, KuaInfo,
    LoShuGrid, NineStar, NineStarBoundary, NineStars, Palace, TamNguyen, Trigram, TrigramGroup,
};
use astro_core::tuvi::{
    FiveElementsBureau, FourTransformations, LeapMonthRule, MenhCucRelation, StarBrightness,
    StarCategory, StarNature, TuViChart, TuViConfig, TuViPalace, TuViPalaceRole, TuViPlacedStar,
    TuViPolarity, TuViStar, YinYangHarmony,
};
use astro_core::{
    AstroError, Branch, CanChi, ConnectionCheck, DayInfo, FiveElements, FourPillars,
    FourPillarsOptions, LunarDate, LunarMonth, LunarYear, NaYin, RatHourConvention, SolarDate,
    SolarDateTime, SolarTerm, SolarTermEvent, Stem, YearMonthBoundary,
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
pub enum Stem {
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
pub enum Branch {
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
    pub stem: Stem,
    pub branch: Branch,
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
    pub boundary: YearMonthBoundary,
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

#[uniffi::remote(Enum)]
pub enum NaYin {
    HaiTrungKim,
    LuTrungHoa,
    DaiLamMoc,
    LoBangTho,
    KiemPhongKim,
    SonDauHoa,
    GianHaThuy,
    ThanhDauTho,
    BachLapKim,
    DuongLieuMoc,
    TuyenTrungThuy,
    OcThuongTho,
    TichLichHoa,
    TungBachMoc,
    TruongLuuThuy,
    SaTrungKim,
    SonHaHoa,
    BinhDiaMoc,
    BichThuongTho,
    KimBachKim,
    PhuDangHoa,
    ThienHaThuy,
    DaiDichTho,
    ThoaXuyenKim,
    TangDoMoc,
    DaiKheThuy,
    SaTrungTho,
    ThienThuongHoa,
    ThachLuuMoc,
    DaiHaiThuy,
}

#[uniffi::remote(Enum)]
pub enum YearMonthBoundary {
    SolarTerms,
    LunarNewYear,
}

#[uniffi::remote(Enum)]
pub enum TenGod {
    TyKien,
    KiepTai,
    ThucThan,
    ThuongQuan,
    ThienTai,
    ChinhTai,
    ThatSat,
    ChinhQuan,
    ThienAn,
    ChinhAn,
}

#[uniffi::remote(Enum)]
pub enum LifeStage {
    TruongSinh,
    MocDuc,
    QuanDoi,
    LamQuan,
    DeVuong,
    Suy,
    Benh,
    Tu,
    Mo,
    Tuyet,
    Thai,
    Duong,
}

#[uniffi::remote(Enum)]
pub enum BaziStar {
    ThienAtQuyNhan,
    VanXuong,
    LocTon,
    KinhDuong,
    DaLa,
    DichMa,
    DaoHoa,
    HoaCai,
    TuongTinh,
    KiepSat,
}

#[uniffi::remote(Enum)]
pub enum Gender {
    Male,
    Female,
}

#[uniffi::remote(Enum)]
pub enum PillarPosition {
    Year,
    Month,
    Day,
    Hour,
}

#[uniffi::remote(Enum)]
pub enum StarBasis {
    DayStem,
    YearStem,
    DayBranch,
    YearBranch,
}

#[uniffi::remote(Enum)]
pub enum Luck {
    Cat,
    Hung,
}

#[uniffi::remote(Enum)]
pub enum Truc {
    Kien,
    Tru,
    Man,
    Binh,
    Dinh,
    Chap,
    Pha,
    Nguy,
    Thanh,
    Thu,
    Khai,
    Be,
}

#[uniffi::remote(Enum)]
pub enum Mansion {
    GiacMocGiao,
    CangKimLong,
    DeThoLac,
    PhongNhatTho,
    TamNguyetHo,
    ViHoaHo,
    CoThuyBao,
    DauMocGiai,
    NguuKimNguu,
    NuThoBuc,
    HuNhatThu,
    NguyNguyetYen,
    ThatHoaTru,
    BichThuyDu,
    KhueMocLang,
    LauKimCau,
    ViThoTri,
    MaoNhatKe,
    TatNguyetO,
    ChuyHoaHau,
    SamThuyVien,
    TinhMocHan,
    QuyKimDuong,
    LieuThoChuong,
    TinhNhatMa,
    TruongNguyetLoc,
    DucHoaXa,
    ChanThuyDan,
}

#[uniffi::remote(Enum)]
pub enum PathGod {
    ThanhLong,
    MinhDuong,
    ThienHinh,
    ChuTuoc,
    KimQuy,
    ThienDuc,
    BachHo,
    NgocDuong,
    ThienLao,
    HuyenVu,
    TuMenh,
    CauTran,
}

#[uniffi::remote(Enum)]
pub enum AlmanacGod {
    ThienDuc,
    NguyetDuc,
    ThienXa,
    ThienNguyen,
    SinhKhi,
    ThienHy,
    TamHop,
    DichMa,
    NguyetPha,
    ThienCuong,
    HaKhoi,
    NguyetSat,
    KiepSat,
    TaiSat,
    NguyetHinh,
}

#[uniffi::remote(Enum)]
pub enum VietnamTaboo {
    TamNuongSat,
    NguyetKy,
    NguyetTan,
    DuongCongKyNhat,
    ThuTu,
    SatChuDuong,
    SatChuAm,
    VangVong,
    ThienDiaChuyenSat,
}

#[uniffi::remote(Enum)]
pub enum TamTai {
    Nhap,
    Cu,
    Xuat,
}

#[uniffi::remote(Enum)]
pub enum KimLau {
    Than,
    The,
    Tu,
    Suc,
}

#[uniffi::remote(Enum)]
pub enum HoangOc {
    NhatCat,
    NhiNghi,
    TamDiaSat,
    TuTanTai,
    NguThoTu,
    LucHoangOc,
}

#[uniffi::remote(Record)]
pub struct KhongVong {
    pub first: Branch,
    pub second: Branch,
}

#[uniffi::remote(Record)]
pub struct LuckCycle {
    pub index: u8,
    pub can_chi: Option<CanChi>,
    pub start_year: i32,
    pub end_year: i32,
    pub start_age: i32,
    pub end_age: i32,
}

#[uniffi::remote(Record)]
pub struct DaiVan {
    pub forward: bool,
    pub start_years: u32,
    pub start_months: u32,
    pub start_days: u32,
    pub start_hours: u32,
    pub start_time: SolarDateTime,
    pub cycles: Vec<LuckCycle>,
}

#[uniffi::remote(Record)]
pub struct HiddenStem {
    pub stem: Stem,
    pub ten_god: TenGod,
}

#[uniffi::remote(Record)]
pub struct PillarReading {
    pub can_chi: CanChi,
    pub na_yin: NaYin,
    pub ten_god: Option<TenGod>,
    pub hidden_stems: Vec<HiddenStem>,
    pub life_stage: LifeStage,
}

#[uniffi::remote(Record)]
pub struct StarPlacement {
    pub star: BaziStar,
    pub basis: StarBasis,
    pub pillar: PillarPosition,
}

#[uniffi::remote(Record)]
pub struct BaziAnalysis {
    pub pillars: FourPillars,
    pub year: PillarReading,
    pub month: PillarReading,
    pub day: PillarReading,
    pub hour: PillarReading,
    pub stars: Vec<StarPlacement>,
    pub khong_vong_day: KhongVong,
    pub khong_vong_year: KhongVong,
    pub is_khoi_canh: bool,
    pub dai_van: DaiVan,
}

#[uniffi::remote(Record)]
pub struct AlmanacDay {
    pub solar: SolarDate,
    pub lunar: LunarDate,
    pub day_can_chi: CanChi,
    pub month_branch: Branch,
    pub truc: Truc,
    pub mansion: Mansion,
    pub path_god: PathGod,
    pub hoang_dao_hours: Vec<Branch>,
    pub gods: Vec<AlmanacGod>,
    pub taboos: Vec<VietnamTaboo>,
}

#[uniffi::remote(Record)]
pub struct AgeRestrictions {
    pub lunar_age: i32,
    pub tam_tai: Option<TamTai>,
    pub kim_lau: Option<KimLau>,
    pub hoang_oc: Option<HoangOc>,
}

#[uniffi::remote(Enum)]
pub enum FiveElements {
    Moc,
    Hoa,
    Tho,
    Kim,
    Thuy,
}

#[uniffi::remote(Enum)]
pub enum Trigram {
    Kham,
    Khon,
    Chan,
    Ton,
    CanKim,
    Doai,
    CanTho,
    Ly,
}

#[uniffi::remote(Enum)]
pub enum TrigramGroup {
    DongTu,
    TayTu,
}

#[uniffi::remote(Enum)]
pub enum Palace {
    Kham,
    Khon,
    Chan,
    Ton,
    TrungCung,
    CanKim,
    Doai,
    CanTho,
    Ly,
}

#[uniffi::remote(Enum)]
pub enum CompassDirection {
    Bac,
    DongBac,
    Dong,
    DongNam,
    Nam,
    TayNam,
    Tay,
    TayBac,
}

#[uniffi::remote(Enum)]
pub enum EightMansionsStar {
    SinhKhi,
    ThienY,
    DienNien,
    PhucVi,
    TuyetMenh,
    NguQuy,
    LucSat,
    HoaHai,
}

#[uniffi::remote(Enum)]
pub enum NineStar {
    NhatBach,
    NhiHac,
    TamBich,
    TuLuc,
    NguHoang,
    LucBach,
    ThatXich,
    BatBach,
    CuuTu,
}

#[uniffi::remote(Enum)]
pub enum TamNguyen {
    ThuongNguyen,
    TrungNguyen,
    HaNguyen,
}

#[uniffi::remote(Enum)]
pub enum NineStarBoundary {
    LunarNewYear,
    LapXuanDay,
    LapXuanExact,
}

#[uniffi::remote(Record)]
pub struct DirectionStar {
    pub direction: CompassDirection,
    pub star: EightMansionsStar,
}

#[uniffi::remote(Record)]
pub struct KuaInfo {
    pub birth_lunar_year: i32,
    pub gender: Gender,
    pub raw_number: u8,
    pub number: u8,
    pub trigram: Trigram,
    pub group: TrigramGroup,
}

#[uniffi::remote(Record)]
pub struct KuaCompatibility {
    pub first: KuaInfo,
    pub second: KuaInfo,
    pub star: EightMansionsStar,
    pub luck: Luck,
}

#[uniffi::remote(Record)]
pub struct FlyingStar {
    pub star: NineStar,
    pub forward: bool,
}

#[uniffi::remote(Record)]
pub struct NineStars {
    pub nguyen: TamNguyen,
    pub van: NineStar,
    pub year: NineStar,
    pub month: NineStar,
    pub day: FlyingStar,
    pub hour: FlyingStar,
}

#[uniffi::remote(Record)]
pub struct LoShuGrid {
    pub forward: bool,
    pub kham: NineStar,
    pub khon: NineStar,
    pub chan: NineStar,
    pub ton: NineStar,
    pub trung_cung: NineStar,
    pub can_kim: NineStar,
    pub doai: NineStar,
    pub can_tho: NineStar,
    pub ly: NineStar,
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
    astro_core::solar_to_lunar(date.day, date.month, date.year, tz_offset)
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

/// Local wall-clock time, to the second, at which `term` begins in a Gregorian year.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn get_exact_solar_term_time(
    year: i32,
    term: SolarTerm,
    tz_offset: f64,
) -> Result<SolarDateTime, AstroError> {
    astro_core::get_exact_solar_term_time(year, term, tz_offset)
}

/// Lunar date, Can Chi and solar term of a Gregorian date, for calendar views.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn day_info(date: SolarDate, tz_offset: f64) -> Result<DayInfo, AstroError> {
    astro_core::day_info(date, tz_offset)
}

#[uniffi::remote(Enum)]
pub enum LeapMonthRule {
    SameAsCurrentMonth,
    SplitAt15th,
}

#[uniffi::remote(Record)]
pub struct TuViConfig {
    pub tz_offset: f64,
    pub leap_month_rule: LeapMonthRule,
    pub rat_hour: RatHourConvention,
    pub use_true_solar_time: bool,
    pub longitude: Option<f64>,
}

#[uniffi::remote(Enum)]
pub enum TuViPalaceRole {
    Menh,
    PhuMau,
    PhucDuc,
    DienTrach,
    QuanLoc,
    NoBoc,
    ThienDi,
    TatAch,
    TaiBach,
    TuTuc,
    PhuThe,
    HuynhDe,
}

#[uniffi::remote(Enum)]
pub enum FiveElementsBureau {
    ThuyNhi,
    MocTam,
    KimTu,
    ThoNgu,
    HoaLuc,
}

#[uniffi::remote(Enum)]
pub enum StarBrightness {
    Mieu,
    Vuong,
    Dac,
    Binh,
    Ham,
}

#[uniffi::remote(Enum)]
pub enum StarCategory {
    ChinhTinh,
    PhuTinh,
}

#[uniffi::remote(Enum)]
pub enum StarNature {
    Cat,
    Trung,
    Hung,
}

#[uniffi::remote(Enum)]
pub enum TuViStar {
    TuVi,
    ThienCo,
    ThaiDuong,
    VuKhuc,
    ThienDong,
    LiemTrinh,
    ThienPhu,
    ThaiAm,
    ThamLang,
    CuMon,
    ThienTuong,
    ThienLuong,
    ThatSat,
    PhaQuan,
    ThaiTue,
    ThieuDuong,
    TangMon,
    ThieuAm,
    QuanPhuThaiTue,
    TuPhu,
    TuePha,
    LongDuc,
    BachHo,
    PhucDuc,
    DieuKhach,
    TrucPhu,
    LocTon,
    KinhDuong,
    DaLa,
    BacSy,
    LucSy,
    ThanhLong,
    TieuHao,
    TuongQuan,
    TauThu,
    PhiLiem,
    HyThan,
    BenhPhu,
    DaiHao,
    PhucBinh,
    QuanPhuBacSy,
    TruongSinh,
    MocDuc,
    QuanDoi,
    LamQuan,
    DeVuong,
    Suy,
    Benh,
    Tu,
    Mo,
    Tuyet,
    Thai,
    Duong,
    TaPhu,
    HuuBat,
    ThienHinh,
    ThienRieu,
    ThienY,
    ThienGiai,
    DiaGiai,
    VanXuong,
    VanKhuc,
    DiaKhong,
    DiaKiep,
    ThaiPhu,
    PhongCao,
    TamThai,
    BatToa,
    AnQuang,
    ThienQuy,
    HoaTinh,
    LinhTinh,
    ThienKhoi,
    ThienViet,
    ThienQuan,
    ThienPhuc,
    LuuHa,
    ThienTru,
    QuocAn,
    DuongPhu,
    LongTri,
    PhuongCac,
    GiaiThan,
    HongLoan,
    ThienHy,
    CoThan,
    QuaTu,
    DaoHoa,
    ThienMa,
    KiepSat,
    HoaCai,
    PhaToai,
    ThienKhong,
    ThienKhoc,
    ThienHu,
    ThienDuc,
    NguyetDuc,
    ThienTai,
    ThienTho,
    ThienThuong,
    ThienSu,
    ThienLa,
    DiaVong,
    LuuNienVanTinh,
    DauQuan,
}

#[uniffi::remote(Enum)]
pub enum FourTransformations {
    HoaLoc,
    HoaQuyen,
    HoaKhoa,
    HoaKy,
}

#[uniffi::remote(Enum)]
pub enum TuViPolarity {
    DuongNam,
    AmNam,
    DuongNu,
    AmNu,
}

#[uniffi::remote(Enum)]
pub enum YinYangHarmony {
    ThuanLy,
    NghichLy,
}

#[uniffi::remote(Enum)]
pub enum MenhCucRelation {
    MenhSinhCuc,
    CucSinhMenh,
    MenhKhacCuc,
    CucKhacMenh,
    BinhHoa,
}

#[uniffi::remote(Record)]
pub struct TuViPlacedStar {
    pub star: TuViStar,
    pub brightness: Option<StarBrightness>,
    pub transformation: Option<FourTransformations>,
}

#[uniffi::remote(Record)]
pub struct TuViPalace {
    pub branch: Branch,
    pub stem: Stem,
    pub role: TuViPalaceRole,
    pub is_than: bool,
    pub main_stars: Vec<TuViPlacedStar>,
    pub minor_stars: Vec<TuViPlacedStar>,
    pub life_stage: LifeStage,
    pub dai_han_start_age: u8,
    pub tieu_han_year_branch: Branch,
    pub tieu_han_first_age: u8,
    pub has_tuan: bool,
    pub has_triet: bool,
}

#[uniffi::remote(Record)]
pub struct TuViChart {
    pub gender: Gender,
    pub effective_time: Option<SolarDateTime>,
    pub solar_date: SolarDate,
    pub lunar: LunarDate,
    pub effective_month: u8,
    pub hour_branch: Branch,
    pub year_can_chi: CanChi,
    pub month_can_chi: CanChi,
    pub day_can_chi: CanChi,
    pub hour_can_chi: CanChi,
    pub polarity: TuViPolarity,
    pub forward: bool,
    pub ban_menh: NaYin,
    pub bureau: FiveElementsBureau,
    pub menh_branch: Branch,
    pub than_branch: Branch,
    pub than_role: TuViPalaceRole,
    pub menh_chu: TuViStar,
    pub than_chu: TuViStar,
    pub harmony: YinYangHarmony,
    pub menh_cuc: MenhCucRelation,
    pub tuan: KhongVong,
    pub triet: KhongVong,
    pub palaces: Vec<TuViPalace>,
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
    astro_core::to_true_solar_time(clock, longitude, tz_offset * 15.0)
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
pub fn stem_name(stem: Stem) -> String {
    stem.name_vi().to_owned()
}

/// Vietnamese name of an Earthly Branch, e.g. "Tý".
#[uniffi::export]
pub fn branch_name(branch: Branch) -> String {
    branch.name_vi().to_owned()
}

/// Vietnamese name of a solar term, e.g. "Lập Xuân".
#[uniffi::export]
pub fn solar_term_name(term: SolarTerm) -> String {
    term.name_vi().to_owned()
}

/// The main hidden stem (Bản khí) of a branch.
#[uniffi::export]
pub fn main_hidden_stem(branch: Branch) -> Stem {
    astro_core::bazi_analysis::main_hidden_stem(branch)
}

/// Project defaults for the Four Pillars: UTC+7, clock time, `SplitAtMidnight`, `SolarTerms`.
#[uniffi::export]
pub fn default_four_pillars_options() -> FourPillarsOptions {
    FourPillarsOptions::default()
}

/// Full reading of a Bát Tự chart: pillars, Ten Gods, life stages, stars and luck cycles.
#[uniffi::export]
pub fn analyze_bazi(
    birth: SolarDateTime,
    options: FourPillarsOptions,
    gender: Gender,
) -> Result<BaziAnalysis, AstroError> {
    astro_core::analyze_bazi(birth, &options, gender)
}

/// Day selection data of a Gregorian date: Trực, mansion, gods and Vietnamese taboos.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn almanac_day(date: SolarDate, tz_offset: f64) -> Result<AlmanacDay, AstroError> {
    astro_core::almanac_day(date, tz_offset)
}

/// Tam Tai, Kim Lâu and Hoang Ốc of a person born in one lunar year, in another.
#[uniffi::export]
pub fn age_restrictions(birth_lunar_year: i32, target_lunar_year: i32) -> AgeRestrictions {
    astro_core::age_restrictions(birth_lunar_year, target_lunar_year)
}

/// Fortune of a mansion.
#[uniffi::export]
pub fn mansion_luck(value: Mansion) -> Luck {
    value.luck()
}

/// Fortune of an almanac god.
#[uniffi::export]
pub fn almanac_god_luck(value: AlmanacGod) -> Luck {
    value.luck()
}

/// Fortune of a Hoang Ốc palace.
#[uniffi::export]
pub fn hoang_oc_luck(value: HoangOc) -> Luck {
    value.luck()
}

/// Whether a path god is a yellow path god (Hoàng Đạo).
#[uniffi::export]
pub fn path_god_is_hoang_dao(value: PathGod) -> bool {
    value.is_hoang_dao()
}

/// Vietnamese name of a Nạp Âm.
#[uniffi::export]
pub fn na_yin_name(value: NaYin) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a Ten God.
#[uniffi::export]
pub fn ten_god_name(value: TenGod) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a life stage.
#[uniffi::export]
pub fn life_stage_name(value: LifeStage) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a chart star.
#[uniffi::export]
pub fn bazi_star_name(value: BaziStar) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a Trực.
#[uniffi::export]
pub fn truc_name(value: Truc) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a mansion.
#[uniffi::export]
pub fn mansion_name(value: Mansion) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a path god.
#[uniffi::export]
pub fn path_god_name(value: PathGod) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of an almanac god.
#[uniffi::export]
pub fn almanac_god_name(value: AlmanacGod) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a Vietnamese taboo.
#[uniffi::export]
pub fn vietnam_taboo_name(value: VietnamTaboo) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a Tam Tai year.
#[uniffi::export]
pub fn tam_tai_name(value: TamTai) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a Kim Lâu.
#[uniffi::export]
pub fn kim_lau_name(value: KimLau) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a Hoang Ốc palace.
#[uniffi::export]
pub fn hoang_oc_name(value: HoangOc) -> String {
    value.name_vi().to_owned()
}

/// Mệnh Quái of a person born in a lunar year.
#[uniffi::export]
pub fn kua_info(birth_lunar_year: i32, gender: Gender) -> KuaInfo {
    astro_core::fengshui::kua_info(birth_lunar_year, gender)
}

/// Compatibility of two people by the Eight Mansions; their order does not matter.
#[uniffi::export]
pub fn kua_compatibility(first: KuaInfo, second: KuaInfo) -> KuaCompatibility {
    astro_core::fengshui::kua_compatibility(first, second)
}

/// Trạch Quái of a house from its sitting direction (Tọa): a house sitting north is Khảm.
#[uniffi::export]
pub fn house_trigram(sitting: CompassDirection) -> Trigram {
    astro_core::fengshui::house_trigram(sitting)
}

/// Trạch Quái of a house from its facing direction; it sits the opposite way.
#[uniffi::export]
pub fn house_trigram_from_facing(facing: CompassDirection) -> Trigram {
    astro_core::fengshui::house_trigram_from_facing(facing)
}

/// The opposite direction: a house facing south sits north.
#[uniffi::export]
pub fn opposite_direction(direction: CompassDirection) -> CompassDirection {
    direction.opposite()
}

/// Trigram of a compass direction.
#[uniffi::export]
pub fn direction_trigram(direction: CompassDirection) -> Trigram {
    direction.trigram()
}

/// Star of the Eight Mansions between two trigrams; their order does not matter.
#[uniffi::export]
pub fn star_towards(trigram: Trigram, other: Trigram) -> EightMansionsStar {
    trigram.star_towards(other)
}

/// Star of the Eight Mansions in a direction, for a Mệnh Quái or a Trạch Quái.
#[uniffi::export]
pub fn star_at(trigram: Trigram, direction: CompassDirection) -> EightMansionsStar {
    trigram.star_at(direction)
}

/// Stars of the eight directions of a trigram, clockwise from north.
#[uniffi::export]
pub fn eight_mansions_chart(trigram: Trigram) -> Vec<DirectionStar> {
    trigram.chart().to_vec()
}

/// Era, period and stars ruling a wall-clock time; `boundary` defaults to `LapXuanDay`.
#[uniffi::export(default(tz_offset = 7.0, boundary))]
pub fn nine_stars(
    time: SolarDateTime,
    tz_offset: f64,
    boundary: Option<NineStarBoundary>,
) -> Result<NineStars, AstroError> {
    astro_core::fengshui::nine_stars(time, tz_offset, boundary.unwrap_or_default())
}

/// Year star (Niên Tinh) at a wall-clock time; `boundary` defaults to `LapXuanDay`.
#[uniffi::export(default(tz_offset = 7.0, boundary))]
pub fn year_nine_star(
    time: SolarDateTime,
    tz_offset: f64,
    boundary: Option<NineStarBoundary>,
) -> Result<NineStar, AstroError> {
    astro_core::fengshui::year_nine_star(time, tz_offset, boundary.unwrap_or_default())
}

/// Month star (Nguyệt Tinh) at a wall-clock time; `boundary` defaults to `LapXuanDay`.
#[uniffi::export(default(tz_offset = 7.0, boundary))]
pub fn month_nine_star(
    time: SolarDateTime,
    tz_offset: f64,
    boundary: Option<NineStarBoundary>,
) -> Result<NineStar, AstroError> {
    astro_core::fengshui::month_nine_star(time, tz_offset, boundary.unwrap_or_default())
}

/// Day star (Nhật Tinh) of a Gregorian date, with the direction it flies in.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn day_nine_star(date: SolarDate, tz_offset: f64) -> Result<FlyingStar, AstroError> {
    astro_core::fengshui::day_nine_star(date, tz_offset)
}

/// Hour star (Thời Tinh) at a wall-clock time, with the direction it flies in.
#[uniffi::export(default(tz_offset = 7.0))]
pub fn hour_nine_star(time: SolarDateTime, tz_offset: f64) -> Result<FlyingStar, AstroError> {
    astro_core::fengshui::hour_nine_star(time, tz_offset)
}

/// Year star (Niên Tinh) of a year: 2024 is Tam Bích.
#[uniffi::export]
pub fn star_of_year(year: i32) -> NineStar {
    astro_core::fengshui::star_of_year(year)
}

/// Ruling star of the 20-year period (Vận) of a year: 2024–2043 is Cửu Tử.
#[uniffi::export]
pub fn van_of_year(year: i32) -> NineStar {
    astro_core::fengshui::van_of_year(year)
}

/// Era (Nguyên) of a year: 1984–2043 is Hạ Nguyên.
#[uniffi::export]
pub fn nguyen_of_year(year: i32) -> TamNguyen {
    astro_core::fengshui::nguyen_of_year(year)
}

/// Lo Shu grid with `center_star` in the center, flying forward (thuận) by default.
#[uniffi::export(default(forward = true))]
pub fn generate_loshu_grid(center_star: NineStar, forward: bool) -> LoShuGrid {
    astro_core::fengshui::generate_loshu_grid(center_star, forward)
}

/// Star in one palace of a Lo Shu grid.
#[uniffi::export]
pub fn lo_shu_star_at(grid: LoShuGrid, palace: Palace) -> NineStar {
    grid.star_at(palace)
}

/// The 3×3 layout of a Lo Shu grid, south on top: three rows of three stars.
#[uniffi::export]
pub fn lo_shu_rows(grid: LoShuGrid) -> Vec<Vec<NineStar>> {
    grid.rows().into_iter().map(Vec::from).collect()
}

/// Vietnamese name of a trigram, e.g. "Khảm".
#[uniffi::export]
pub fn trigram_name(value: Trigram) -> String {
    value.name_vi().to_owned()
}

/// Hán character of a trigram, e.g. "坎".
#[uniffi::export]
pub fn trigram_han_name(value: Trigram) -> String {
    value.name_han().to_owned()
}

/// Lạc Thư number of a trigram: 1–4 or 6–9.
#[uniffi::export]
pub fn trigram_number(value: Trigram) -> u8 {
    value.number()
}

/// Element of a trigram.
#[uniffi::export]
pub fn trigram_element(value: Trigram) -> FiveElements {
    value.element()
}

/// Group of a trigram: Đông Tứ or Tây Tứ.
#[uniffi::export]
pub fn trigram_group(value: Trigram) -> TrigramGroup {
    value.group()
}

/// Compass direction of a trigram.
#[uniffi::export]
pub fn trigram_direction(value: Trigram) -> CompassDirection {
    value.direction()
}

/// Palace of the Lo Shu grid held by a trigram.
#[uniffi::export]
pub fn trigram_palace(value: Trigram) -> Palace {
    value.palace()
}

/// Vietnamese name of a trigram group, "Đông Tứ" or "Tây Tứ".
#[uniffi::export]
pub fn trigram_group_name(value: TrigramGroup) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a palace, e.g. "Trung Cung".
#[uniffi::export]
pub fn palace_name(value: Palace) -> String {
    value.name_vi().to_owned()
}

/// Hán character of a palace, e.g. "中".
#[uniffi::export]
pub fn palace_han_name(value: Palace) -> String {
    value.name_han().to_owned()
}

/// Lạc Thư number of a palace, from 1 (Khảm) to 9 (Ly).
#[uniffi::export]
pub fn palace_number(value: Palace) -> u8 {
    value.number()
}

/// Element of a palace.
#[uniffi::export]
pub fn palace_element(value: Palace) -> FiveElements {
    value.element()
}

/// Trigram of a palace; the center has none.
#[uniffi::export]
pub fn palace_trigram(value: Palace) -> Option<Trigram> {
    value.trigram()
}

/// Compass direction of a palace; the center has none.
#[uniffi::export]
pub fn palace_direction(value: Palace) -> Option<CompassDirection> {
    value.direction()
}

/// Vietnamese name of a compass direction, e.g. "Đông Bắc".
#[uniffi::export]
pub fn compass_direction_name(value: CompassDirection) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of an Eight Mansions star, e.g. "Sinh Khí".
#[uniffi::export]
pub fn eight_mansions_star_name(value: EightMansionsStar) -> String {
    value.name_vi().to_owned()
}

/// Hán characters of an Eight Mansions star, e.g. "生气".
#[uniffi::export]
pub fn eight_mansions_star_han_name(value: EightMansionsStar) -> String {
    value.name_han().to_owned()
}

/// Name of the star corresponding to an Eight Mansions star, e.g. "Tham Lang" for Sinh Khí.
#[uniffi::export]
pub fn eight_mansions_star_corresponding_name(value: EightMansionsStar) -> String {
    value.corresponding_star_name().to_owned()
}

/// Fortune of an Eight Mansions star.
#[uniffi::export]
pub fn eight_mansions_star_luck(value: EightMansionsStar) -> Luck {
    value.luck()
}

/// Vietnamese name of a Nine Star, e.g. "Nhất Bạch Thủy".
#[uniffi::export]
pub fn nine_star_name(value: NineStar) -> String {
    value.name_vi().to_owned()
}

/// Hán characters of a Nine Star, e.g. "一白水".
#[uniffi::export]
pub fn nine_star_han_name(value: NineStar) -> String {
    value.name_han().to_owned()
}

/// Number of a Nine Star, from 1 to 9.
#[uniffi::export]
pub fn nine_star_number(value: NineStar) -> u8 {
    value.number()
}

/// Element of a Nine Star.
#[uniffi::export]
pub fn nine_star_element(value: NineStar) -> FiveElements {
    value.element()
}

/// Vietnamese name of the color of a Nine Star, e.g. "Xanh biếc".
#[uniffi::export]
pub fn nine_star_color_name(value: NineStar) -> String {
    value.color_vi().to_owned()
}

/// Palace where a Nine Star belongs in the original Lạc Thư square.
#[uniffi::export]
pub fn nine_star_home_palace(value: NineStar) -> Palace {
    value.home_palace()
}

/// Fortune of a Nine Star, fixed rather than relative to the current era.
#[uniffi::export]
pub fn nine_star_luck(value: NineStar) -> Luck {
    value.luck()
}

/// Name of a Nine Star in Huyền Không, e.g. "Tham Lang".
#[uniffi::export]
pub fn nine_star_xuan_kong_name(value: NineStar) -> String {
    value.xuan_kong_name().to_owned()
}

/// Vietnamese name of an era, e.g. "Hạ Nguyên".
#[uniffi::export]
pub fn tam_nguyen_name(value: TamNguyen) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of one of the Five Elements, e.g. "Mộc".
#[uniffi::export]
pub fn five_elements_name(value: FiveElements) -> String {
    value.name_vi().to_owned()
}

/// The settings of a Tử Vi chart when none are given.
#[uniffi::export]
pub fn default_tuvi_config() -> TuViConfig {
    TuViConfig::default()
}

/// Tử Vi chart of a birth at a wall-clock time.
#[uniffi::export(default(config))]
pub fn tuvi_chart(
    birth: SolarDateTime,
    gender: Gender,
    config: Option<TuViConfig>,
) -> Result<TuViChart, AstroError> {
    astro_core::tuvi_chart(birth, gender, &config.unwrap_or_default())
}

/// Tử Vi chart of a birth on a lunar date at a double-hour.
#[uniffi::export(default(config))]
pub fn tuvi_chart_from_lunar(
    lunar: LunarDate,
    hour: Branch,
    gender: Gender,
    config: Option<TuViConfig>,
) -> Result<TuViChart, AstroError> {
    astro_core::tuvi_chart_from_lunar(lunar, hour, gender, &config.unwrap_or_default())
}

/// Palace of the Đại Hạn at tuổi mụ `age`, if any.
#[uniffi::export]
pub fn tuvi_dai_han_palace(chart: TuViChart, age: u8) -> Option<TuViPalace> {
    chart.dai_han_palace(age).cloned()
}

/// Palace of the Tiểu Hạn at tuổi mụ `age`, if any.
#[uniffi::export]
pub fn tuvi_tieu_han_palace(chart: TuViChart, age: u8) -> Option<TuViPalace> {
    chart.tieu_han_palace(age).cloned()
}

/// Every star of a Tử Vi chart, in the order of the enum.
#[uniffi::export]
pub fn tuvi_all_stars() -> Vec<TuViStar> {
    TuViStar::ALL.to_vec()
}

/// Vietnamese name of a star, e.g. "Thiên Phủ".
#[uniffi::export]
pub fn tuvi_star_name(value: TuViStar) -> String {
    value.name_vi().to_owned()
}

/// Whether a star is one of the 14 Chính Tinh.
#[uniffi::export]
pub fn tuvi_star_category(value: TuViStar) -> StarCategory {
    value.category()
}

/// Inherent nature of a star. This is the project's reading of the book, not a table of the book.
#[uniffi::export]
pub fn tuvi_star_nature(value: TuViStar) -> StarNature {
    value.nature()
}

/// Brightness of a star in a palace; only the Chính Tinh have one.
#[uniffi::export]
pub fn tuvi_star_brightness(value: TuViStar, branch: Branch) -> Option<StarBrightness> {
    value.brightness_at(branch)
}

/// Vietnamese name of a brightness, e.g. "Miếu".
#[uniffi::export]
pub fn star_brightness_name(value: StarBrightness) -> String {
    value.name_vi().to_owned()
}

/// Mark of a brightness on a chart: M, V, Đ, B or H.
#[uniffi::export]
pub fn star_brightness_mark(value: StarBrightness) -> String {
    value.mark().to_owned()
}

/// Vietnamese name of a star's nature, e.g. "Cát".
#[uniffi::export]
pub fn star_nature_name(value: StarNature) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a transformation, e.g. "Hóa Lộc".
#[uniffi::export]
pub fn four_transformation_name(value: FourTransformations) -> String {
    value.name_vi().to_owned()
}

/// The star that takes a transformation in a year with stem `year_stem`.
#[uniffi::export]
pub fn four_transformation_star(value: FourTransformations, year_stem: Stem) -> TuViStar {
    value.star_for(year_stem)
}

/// Inherent nature of a transformation.
#[uniffi::export]
pub fn four_transformation_nature(value: FourTransformations) -> StarNature {
    value.nature()
}

/// Vietnamese name of a functional palace, e.g. "Phụ Mẫu".
#[uniffi::export]
pub fn tuvi_palace_role_name(value: TuViPalaceRole) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a bureau, e.g. "Mộc Tam Cục".
#[uniffi::export]
pub fn five_elements_bureau_name(value: FiveElementsBureau) -> String {
    value.name_vi().to_owned()
}

/// Number of a bureau, from 2 to 6.
#[uniffi::export]
pub fn five_elements_bureau_number(value: FiveElementsBureau) -> u8 {
    value.number()
}

/// Element of a bureau.
#[uniffi::export]
pub fn five_elements_bureau_element(value: FiveElementsBureau) -> FiveElements {
    value.element()
}

/// Vietnamese name of a polarity, e.g. "Dương Nam".
#[uniffi::export]
pub fn tuvi_polarity_name(value: TuViPolarity) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of a harmony, e.g. "Thuận lý".
#[uniffi::export]
pub fn yin_yang_harmony_name(value: YinYangHarmony) -> String {
    value.name_vi().to_owned()
}

/// Vietnamese name of the relation between Bản Mệnh and bureau, e.g. "Bản Mệnh sinh Cục".
#[uniffi::export]
pub fn menh_cuc_relation_name(value: MenhCucRelation) -> String {
    value.name_vi().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VN: f64 = 7.0;

    /// Spec vector 2: giờ Tý Sớm, past Hạ Chí but before the Giáp Tý day where the day star turns.
    const VECTOR_2: SolarDateTime = SolarDateTime {
        year: 2000,
        month: 6,
        day: 25,
        hour: 23,
        minute: 30,
        second: 0,
    };

    fn star_numbers(rows: Vec<Vec<NineStar>>) -> Vec<Vec<u8>> {
        rows.into_iter()
            .map(|row| row.into_iter().map(nine_star_number).collect())
            .collect()
    }

    #[test]
    fn default_literals_match_the_documented_defaults() {
        // Rust callers pass every argument, so only the source shows what Swift callers get.
        assert_eq!(astro_core::DEFAULT_TZ_OFFSET, 7.0);
        let exports = include_str!("lib.rs")
            .lines()
            .filter(|line| line.starts_with("#[uniffi::export(default("));
        for export in exports {
            assert!(
                export.contains("tz_offset = 7.0")
                    || export.contains("forward = true")
                    || export.contains("default(config)"),
                "{export}"
            );
        }
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
        assert_eq!(stem_name(info.month_can_chi.stem), "Bính");
        assert_eq!(branch_name(info.month_can_chi.branch), "Dần");
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
            boundary: YearMonthBoundary::SolarTerms,
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
    fn analysis_and_almanac_delegate_to_the_core() {
        let birth = SolarDateTime {
            year: 2024,
            month: 2,
            day: 10,
            hour: 8,
            minute: 0,
            second: 0,
        };
        let analysis = analyze_bazi(birth, default_four_pillars_options(), Gender::Male).unwrap();
        assert_eq!(na_yin_name(analysis.day.na_yin), "Phú Đăng Hỏa");
        assert_eq!(stem_name(main_hidden_stem(Branch::Dan)), "Giáp");
        assert_eq!(ten_god_name(analysis.month.ten_god.unwrap()), "Thực Thần");
        assert!(analysis.dai_van.forward);

        let day = almanac_day(
            SolarDate {
                year: 2024,
                month: 2,
                day: 16,
            },
            VN,
        )
        .unwrap();
        assert_eq!(truc_name(day.truc), "Thành");
        assert_eq!(mansion_name(day.mansion), "Ngưu Kim Ngưu");
        assert!(path_god_is_hoang_dao(day.path_god));
        assert_eq!(vietnam_taboo_name(day.taboos[0]), "Tam Nương Sát");

        let restrictions = age_restrictions(1990, 2026);
        assert_eq!(kim_lau_name(restrictions.kim_lau.unwrap()), "Kim Lâu Thân");
        assert_eq!(hoang_oc_name(restrictions.hoang_oc.unwrap()), "Tứ Tấn Tài");
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

    #[test]
    fn kua_functions_delegate_to_the_core() {
        let man = kua_info(1990, Gender::Male);
        assert_eq!(man.birth_lunar_year, 1990);
        assert_eq!(man.gender, Gender::Male);
        assert_eq!((man.raw_number, man.number), (1, 1));
        assert_eq!(man.trigram, Trigram::Kham);
        assert_eq!(man.group, TrigramGroup::DongTu);

        // The formula lands on the center, which has no trigram: a woman takes Cấn.
        let woman = kua_info(1990, Gender::Female);
        assert_eq!((woman.raw_number, woman.number), (5, 8));
        assert_eq!(woman.trigram, Trigram::CanTho);
        assert_eq!(woman.group, TrigramGroup::TayTu);

        assert_eq!(
            kua_compatibility(man, woman),
            KuaCompatibility {
                first: man,
                second: woman,
                star: EightMansionsStar::NguQuy,
                luck: Luck::Hung,
            }
        );
    }

    #[test]
    fn house_trigram_reads_the_sitting_direction() {
        assert_eq!(house_trigram(CompassDirection::Bac), Trigram::Kham);
        assert_eq!(
            house_trigram_from_facing(CompassDirection::Nam),
            Trigram::Kham
        );
        assert_eq!(
            opposite_direction(CompassDirection::DongBac),
            CompassDirection::TayNam
        );
        assert_eq!(direction_trigram(CompassDirection::TayBac), Trigram::CanKim);
    }

    #[test]
    fn eight_mansions_lookups_delegate_to_the_core() {
        assert_eq!(
            star_towards(Trigram::Kham, Trigram::Ton),
            EightMansionsStar::SinhKhi
        );
        assert_eq!(
            star_towards(Trigram::Kham, Trigram::Ly),
            EightMansionsStar::DienNien
        );
        assert_eq!(
            star_at(Trigram::Kham, CompassDirection::DongNam),
            EightMansionsStar::SinhKhi
        );
        assert_eq!(
            star_at(Trigram::Kham, CompassDirection::Nam),
            EightMansionsStar::DienNien
        );

        let chart = eight_mansions_chart(Trigram::Kham);
        assert_eq!(chart.len(), 8);
        assert_eq!(
            chart[0],
            DirectionStar {
                direction: CompassDirection::Bac,
                star: EightMansionsStar::PhucVi,
            }
        );
        assert_eq!(
            chart[3],
            DirectionStar {
                direction: CompassDirection::DongNam,
                star: EightMansionsStar::SinhKhi,
            }
        );
    }

    #[test]
    fn nine_stars_read_spec_vector_2_with_the_default_boundary() {
        let expected = NineStars {
            nguyen: TamNguyen::HaNguyen,
            van: NineStar::ThatXich,
            year: NineStar::CuuTu,
            month: NineStar::NhatBach,
            day: FlyingStar {
                star: NineStar::CuuTu,
                forward: true,
            },
            hour: FlyingStar {
                star: NineStar::TamBich,
                forward: false,
            },
        };
        assert_eq!(nine_stars(VECTOR_2, VN, None), Ok(expected));
        assert_eq!(year_nine_star(VECTOR_2, VN, None), Ok(expected.year));
        assert_eq!(month_nine_star(VECTOR_2, VN, None), Ok(expected.month));
        assert_eq!(hour_nine_star(VECTOR_2, VN), Ok(expected.hour));
        assert_eq!(
            day_nine_star(
                SolarDate {
                    year: 2000,
                    month: 6,
                    day: 25
                },
                VN
            ),
            Ok(expected.day)
        );
    }

    #[test]
    fn an_omitted_boundary_means_the_core_default_and_a_given_one_is_passed_on() {
        // The doc comments of the boundary functions above name this variant.
        assert_eq!(NineStarBoundary::default(), NineStarBoundary::LapXuanDay);

        // After Tết but before the day of Lập Xuân: lunar year 2022, still 2021 by Lập Xuân.
        let after_tet = SolarDateTime {
            year: 2022,
            month: 2,
            day: 2,
            hour: 8,
            minute: 0,
            second: 0,
        };
        let tet = Some(NineStarBoundary::LunarNewYear);
        assert_eq!(year_nine_star(after_tet, VN, None), Ok(NineStar::LucBach));
        assert_eq!(year_nine_star(after_tet, VN, tet), Ok(NineStar::NguHoang));

        // The morning of Lập Xuân, before its instant: 2024 by the day, still 2023 by the instant.
        let lap_xuan_morning = SolarDateTime {
            year: 2024,
            month: 2,
            day: 4,
            hour: 8,
            minute: 0,
            second: 0,
        };
        let exact = Some(NineStarBoundary::LapXuanExact);
        assert_eq!(
            year_nine_star(lap_xuan_morning, VN, None),
            Ok(NineStar::TamBich)
        );
        assert_eq!(
            year_nine_star(lap_xuan_morning, VN, exact),
            Ok(NineStar::TuLuc)
        );
        assert_eq!(
            month_nine_star(lap_xuan_morning, VN, None),
            Ok(NineStar::NguHoang)
        );
        assert_eq!(
            month_nine_star(lap_xuan_morning, VN, exact),
            Ok(NineStar::LucBach)
        );
        assert_eq!(
            nine_stars(lap_xuan_morning, VN, None).unwrap().year,
            NineStar::TamBich
        );
        assert_eq!(
            nine_stars(lap_xuan_morning, VN, exact).unwrap().year,
            NineStar::TuLuc
        );
    }

    #[test]
    fn year_functions_delegate_to_the_core() {
        assert_eq!(star_of_year(2024), NineStar::TamBich);
        assert_eq!(van_of_year(2024), NineStar::CuuTu);
        assert_eq!(nguyen_of_year(2024), TamNguyen::HaNguyen);
    }

    #[test]
    fn lo_shu_functions_read_the_grid_south_on_top() {
        let grid = generate_loshu_grid(NineStar::CuuTu, true);
        assert!(grid.forward);
        assert_eq!(grid.trung_cung, NineStar::CuuTu);
        assert_eq!(
            star_numbers(lo_shu_rows(grid)),
            [[8, 4, 6], [7, 9, 2], [3, 5, 1]]
        );
        assert_eq!(lo_shu_star_at(grid, Palace::Ly), NineStar::TuLuc);
        assert_eq!(lo_shu_star_at(grid, Palace::Kham), NineStar::NguHoang);
        assert_eq!(lo_shu_star_at(grid, Palace::TrungCung), NineStar::CuuTu);

        // Section 4.7 of the spec: Tam Bích flying backward.
        let backward = generate_loshu_grid(NineStar::TamBich, false);
        assert!(!backward.forward);
        assert_eq!(
            star_numbers(lo_shu_rows(backward)),
            [[4, 8, 6], [5, 3, 1], [9, 7, 2]]
        );
    }

    #[test]
    fn only_the_center_palace_has_no_trigram_or_direction() {
        assert_eq!(palace_trigram(Palace::TrungCung), None);
        assert_eq!(palace_direction(Palace::TrungCung), None);
        assert_eq!(palace_trigram(Palace::Ly), Some(Trigram::Ly));
        assert_eq!(palace_direction(Palace::Ly), Some(CompassDirection::Nam));
    }

    #[test]
    fn trigram_and_palace_attributes_delegate_to_the_core() {
        assert_eq!(trigram_name(Trigram::CanKim), "Càn");
        assert_eq!(trigram_name(Trigram::CanTho), "Cấn");
        assert_eq!(trigram_han_name(Trigram::CanKim), "乾");
        assert_eq!(trigram_number(Trigram::CanKim), 6);
        assert_eq!(trigram_element(Trigram::Ly), FiveElements::Hoa);
        assert_eq!(trigram_group(Trigram::Doai), TrigramGroup::TayTu);
        assert_eq!(
            trigram_direction(Trigram::CanTho),
            CompassDirection::DongBac
        );
        assert_eq!(trigram_palace(Trigram::Ton), Palace::Ton);
        assert_eq!(trigram_group_name(TrigramGroup::DongTu), "Đông Tứ");

        assert_eq!(palace_name(Palace::TrungCung), "Trung Cung");
        assert_eq!(palace_han_name(Palace::TrungCung), "中");
        assert_eq!(palace_number(Palace::TrungCung), 5);
        assert_eq!(palace_element(Palace::TrungCung), FiveElements::Tho);
        assert_eq!(palace_element(Palace::Ly), FiveElements::Hoa);
        assert_eq!(palace_element(Palace::Kham), FiveElements::Thuy);
        assert_eq!(
            compass_direction_name(CompassDirection::DongBac),
            "Đông Bắc"
        );
    }

    #[test]
    fn star_attributes_delegate_to_the_core() {
        let sinh_khi = EightMansionsStar::SinhKhi;
        assert_eq!(eight_mansions_star_name(sinh_khi), "Sinh Khí");
        assert_eq!(eight_mansions_star_han_name(sinh_khi), "生气");
        assert_eq!(
            eight_mansions_star_corresponding_name(sinh_khi),
            "Tham Lang"
        );
        assert_eq!(eight_mansions_star_luck(sinh_khi), Luck::Cat);
        assert_eq!(
            eight_mansions_star_luck(EightMansionsStar::HoaHai),
            Luck::Hung
        );

        let cuu_tu = NineStar::CuuTu;
        assert_eq!(nine_star_name(cuu_tu), "Cửu Tử Hỏa");
        assert_eq!(nine_star_han_name(cuu_tu), "九紫火");
        assert_eq!(nine_star_number(cuu_tu), 9);
        assert_eq!(nine_star_element(cuu_tu), FiveElements::Hoa);
        assert_eq!(nine_star_color_name(cuu_tu), "Tím");
        assert_eq!(nine_star_home_palace(cuu_tu), Palace::Ly);
        assert_eq!(nine_star_luck(cuu_tu), Luck::Cat);
        assert_eq!(nine_star_luck(NineStar::NhiHac), Luck::Hung);
        assert_eq!(nine_star_xuan_kong_name(cuu_tu), "Hữu Bật");

        assert_eq!(tam_nguyen_name(TamNguyen::HaNguyen), "Hạ Nguyên");
        assert_eq!(five_elements_name(FiveElements::Thuy), "Thủy");
    }

    #[test]
    fn nine_star_functions_pass_on_the_time_zone() {
        // Lập Xuân 2009 began on 3 February in Vietnam and on 4 February in China.
        let lap_xuan_2009 = SolarDateTime {
            year: 2009,
            month: 2,
            day: 3,
            hour: 12,
            minute: 0,
            second: 0,
        };
        for (tz_offset, year, month) in [
            (VN, NineStar::CuuTu, NineStar::NguHoang),
            (8.0, NineStar::NhatBach, NineStar::LucBach),
        ] {
            assert_eq!(year_nine_star(lap_xuan_2009, tz_offset, None), Ok(year));
            assert_eq!(month_nine_star(lap_xuan_2009, tz_offset, None), Ok(month));
            let stars = nine_stars(lap_xuan_2009, tz_offset, None).unwrap();
            assert_eq!((stars.year, stars.month), (year, month));
        }

        // Đông Chí 2017 began on 21 December in Vietnam and on 22 December in China.
        let dong_chi_2017 = SolarDateTime {
            year: 2017,
            month: 12,
            day: 21,
            hour: 12,
            minute: 0,
            second: 0,
        };
        assert!(hour_nine_star(dong_chi_2017, VN).unwrap().forward);
        assert!(!hour_nine_star(dong_chi_2017, 8.0).unwrap().forward);

        // The turning point near Hạ Chí 2077 is 22 May in Vietnam and 21 July in China.
        let june_2077 = SolarDate {
            year: 2077,
            month: 6,
            day: 1,
        };
        assert!(!day_nine_star(june_2077, VN).unwrap().forward);
        assert!(day_nine_star(june_2077, 8.0).unwrap().forward);
    }

    #[test]
    fn nine_star_functions_return_the_error_of_the_core() {
        assert_eq!(
            nine_stars(
                SolarDateTime {
                    month: 13,
                    ..VECTOR_2
                },
                VN,
                None
            ),
            Err(AstroError::InvalidSolarDate {
                year: 2000,
                month: 13,
                day: 25
            })
        );
        assert_eq!(
            hour_nine_star(
                SolarDateTime {
                    hour: 24,
                    ..VECTOR_2
                },
                VN
            ),
            Err(AstroError::InvalidTime {
                hour: 24,
                minute: 30,
                second: 0
            })
        );
        assert_eq!(
            year_nine_star(VECTOR_2, 15.0, None),
            Err(AstroError::InvalidTimeZone { tz_offset: 15.0 })
        );
        assert_eq!(
            month_nine_star(
                SolarDateTime {
                    year: 1500,
                    ..VECTOR_2
                },
                VN,
                None
            ),
            Err(AstroError::YearOutOfRange { year: 1500 })
        );
        assert_eq!(
            day_nine_star(
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
    }

    /// Spec vector 3: Dương Nam in a leap month.
    const TUVI_VECTOR_3: SolarDateTime = SolarDateTime {
        year: 2020,
        month: 6,
        day: 10,
        hour: 10,
        minute: 30,
        second: 0,
    };

    #[test]
    fn tuvi_chart_delegates_to_the_core_with_the_default_config() {
        let chart = tuvi_chart(TUVI_VECTOR_3, Gender::Male, None).unwrap();
        assert_eq!(chart.menh_branch, Branch::Suu);
        assert_eq!(chart.effective_month, 5);
        assert_eq!(five_elements_bureau_number(chart.bureau), 6);
        assert_eq!(chart.palaces.len(), 12);
        assert_eq!(
            Some(chart.clone()),
            tuvi_chart(TUVI_VECTOR_3, Gender::Male, Some(default_tuvi_config())).ok()
        );

        let same_month = TuViConfig {
            leap_month_rule: LeapMonthRule::SameAsCurrentMonth,
            ..default_tuvi_config()
        };
        let other = tuvi_chart(TUVI_VECTOR_3, Gender::Male, Some(same_month)).unwrap();
        assert_eq!(other.menh_branch, Branch::Ty);
    }

    #[test]
    fn tuvi_chart_from_lunar_casts_the_charts_of_the_book() {
        let lunar = LunarDate {
            year: 1891,
            month: 6,
            day: 6,
            is_leap_month: false,
        };
        let chart = tuvi_chart_from_lunar(lunar, Branch::Mao, Gender::Male, None).unwrap();
        assert_eq!(chart.menh_branch, Branch::Thin);
        assert_eq!(five_elements_bureau_name(chart.bureau), "Thủy Nhị Cục");
        assert_eq!(tuvi_polarity_name(chart.polarity), "Âm Nam");
        assert_eq!(yin_yang_harmony_name(chart.harmony), "Nghịch lý");
        assert_eq!(menh_cuc_relation_name(chart.menh_cuc), "Cục sinh Bản Mệnh");
        let palace = tuvi_dai_han_palace(chart.clone(), 12).unwrap();
        assert_eq!(tuvi_palace_role_name(palace.role), "Huynh Đệ");
        assert_eq!(
            tuvi_tieu_han_palace(chart.clone(), 1).unwrap().branch,
            Branch::Suu
        );
        assert_eq!(tuvi_dai_han_palace(chart.clone(), 1), None);
        assert_eq!(tuvi_tieu_han_palace(chart, 0), None);

        let invalid = LunarDate { month: 13, ..lunar };
        assert!(tuvi_chart_from_lunar(invalid, Branch::Mao, Gender::Male, None).is_err());
    }

    #[test]
    fn tuvi_star_helpers_delegate_to_the_core() {
        assert_eq!(tuvi_all_stars().len(), 105);
        assert_eq!(tuvi_star_name(TuViStar::QuanPhuBacSy), "Quan Phủ");
        assert_eq!(tuvi_star_category(TuViStar::TuVi), StarCategory::ChinhTinh);
        assert_eq!(tuvi_star_category(TuViStar::LocTon), StarCategory::PhuTinh);
        assert_eq!(tuvi_star_nature(TuViStar::KinhDuong), StarNature::Hung);
        let brightness = tuvi_star_brightness(TuViStar::TuVi, Branch::Ngo).unwrap();
        assert_eq!(star_brightness_name(brightness), "Miếu");
        assert_eq!(star_brightness_mark(brightness), "M");
        assert_eq!(tuvi_star_brightness(TuViStar::LocTon, Branch::Ngo), None);
        assert_eq!(star_nature_name(StarNature::Cat), "Cát");
        assert_eq!(
            four_transformation_name(FourTransformations::HoaKy),
            "Hóa Kỵ"
        );
        assert_eq!(
            four_transformation_star(FourTransformations::HoaKhoa, Stem::Nham),
            TuViStar::TaPhu
        );
        assert_eq!(
            four_transformation_nature(FourTransformations::HoaKy),
            StarNature::Hung
        );
        assert_eq!(
            five_elements_bureau_element(FiveElementsBureau::KimTu),
            FiveElements::Kim
        );
    }
}
