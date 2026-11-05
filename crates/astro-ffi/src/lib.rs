//! UniFFI bridge exposing astro-core to Swift (iOS) and the other UniFFI languages.
//!
//! The core types are mirrored with `#[uniffi::remote]`, so foreign code receives exactly the
//! records and enums of astro-core with no conversion layer. Every `tz_offset` argument
//! defaults to 7.0 (Vietnam, UTC+7) in the generated bindings.

use astro_core::almanac::{
    AgeRestrictions, AlmanacDay, AlmanacGod, HoangOc, KimLau, Luck, Mansion, PathGod, TamTai, Truc,
    VietnamTaboo,
};
use astro_core::bazi_analysis::{
    BaziAnalysis, BaziStar, DaiVan, Gender, HiddenStem, KhongVong, LifeStage, LuckCycle,
    PillarPosition, PillarReading, StarBasis, StarPlacement, TenGod,
};
use astro_core::{
    AstroError, Branch, CanChi, ConnectionCheck, DayInfo, FourPillars, FourPillarsOptions,
    LunarDate, LunarMonth, LunarYear, NaYin, RatHourConvention, SolarDate, SolarDateTime,
    SolarTerm, SolarTermEvent, Stem, YearMonthBoundary,
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
}
