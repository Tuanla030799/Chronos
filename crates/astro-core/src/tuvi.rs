//! Tử Vi Đẩu Số: the chart of the Vietnamese southern school, after *Tử Vi Đẩu Số Tân Biên* by Vân
//! Đằng Thái Thứ Lang.
//!
//! The rules and tables come from `docs/TUVI_SPEC.md`. [`tuvi_chart`] casts a chart from a
//! wall-clock birth time and [`tuvi_chart_from_lunar`] from a lunar date and a double-hour.
//!
//! - Everything follows the lunar calendar: the year changes at Tết, not at Lập Xuân.
//! - A leap month counts as [`LeapMonthRule`] says. *Tân Biên* is silent on leap months.
//! - A birth during giờ Tý Sớm follows [`RatHourConvention`], as in [`four_pillars`].
//! - Ages are tuổi mụ, the way all four sample charts of the book reckon them.
//!
//! The copy of the book used is a retyped edition with misprints. Section 0.4 of the spec lists
//! them and what was done about each. Two parts of [`TuViChart`] do not come from the book, and
//! the project owner approved both: [`menh_chu`] and [`than_chu`], and [`TuViStar::nature`].

use crate::DEFAULT_TZ_OFFSET;
use crate::bazi_analysis::{Gender, KhongVong, LifeStage, khong_vong};
use crate::bazi_pillars::{FourPillarsOptions, RatHourConvention, YearMonthBoundary, four_pillars};
use crate::canchi::Branch::{self, Dan, Dau, Hoi, Mao, Mui, Ngo, Suu, Than, Thin, Ti, Tuat, Ty};
use crate::canchi::{
    CanChi, FiveElements, NaYin, Stem, YinYang, day_can_chi, hour_can_chi, month_can_chi,
    year_can_chi,
};
use crate::error::AstroError;
use crate::lunar_calendar::{LunarDate, lunar_to_solar, solar_to_lunar};
use crate::time::{SolarDate, SolarDateTime};

/// How a leap month counts when the month of birth is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LeapMonthRule {
    /// As the month it repeats.
    SameAsCurrentMonth,
    /// Days 1–15 count as the month it repeats, days 16 onwards as the next month.
    #[default]
    SplitAt15th,
}

/// Settings for [`tuvi_chart`] and [`tuvi_chart_from_lunar`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TuViConfig {
    /// Offset of the birth clock time from UTC, in hours (7.0 for Vietnam).
    pub tz_offset: f64,
    /// Treatment of a leap month.
    pub leap_month_rule: LeapMonthRule,
    /// Treatment of the Tý double-hour.
    pub rat_hour: RatHourConvention,
    /// Whether the day and double-hour come from true solar time instead of the clock time.
    pub use_true_solar_time: bool,
    /// Birthplace longitude in degrees east, read only when `use_true_solar_time` is set. Without
    /// it the zone's standard meridian `tz_offset × 15` is used, which leaves only the equation
    /// of time.
    pub longitude: Option<f64>,
}

impl Default for TuViConfig {
    fn default() -> Self {
        Self {
            tz_offset: DEFAULT_TZ_OFFSET,
            leap_month_rule: LeapMonthRule::default(),
            rat_hour: RatHourConvention::default(),
            use_true_solar_time: false,
            longitude: None,
        }
    }
}

/// The twelve functional palaces, in order clockwise from Mệnh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TuViPalaceRole {
    /// Mệnh
    Menh,
    /// Phụ Mẫu
    PhuMau,
    /// Phúc Đức
    PhucDuc,
    /// Điền Trạch
    DienTrach,
    /// Quan Lộc
    QuanLoc,
    /// Nô Bộc
    NoBoc,
    /// Thiên Di
    ThienDi,
    /// Tật Ách
    TatAch,
    /// Tài Bạch
    TaiBach,
    /// Tử Tức
    TuTuc,
    /// Phu Thê
    PhuThe,
    /// Huynh Đệ
    HuynhDe,
}

impl TuViPalaceRole {
    /// All roles in order, from Mệnh.
    pub const ALL: [TuViPalaceRole; 12] = [
        TuViPalaceRole::Menh,
        TuViPalaceRole::PhuMau,
        TuViPalaceRole::PhucDuc,
        TuViPalaceRole::DienTrach,
        TuViPalaceRole::QuanLoc,
        TuViPalaceRole::NoBoc,
        TuViPalaceRole::ThienDi,
        TuViPalaceRole::TatAch,
        TuViPalaceRole::TaiBach,
        TuViPalaceRole::TuTuc,
        TuViPalaceRole::PhuThe,
        TuViPalaceRole::HuynhDe,
    ];

    /// Vietnamese name, e.g. "Phụ Mẫu".
    pub const fn name_vi(self) -> &'static str {
        match self {
            TuViPalaceRole::Menh => "Mệnh",
            TuViPalaceRole::PhuMau => "Phụ Mẫu",
            TuViPalaceRole::PhucDuc => "Phúc Đức",
            TuViPalaceRole::DienTrach => "Điền Trạch",
            TuViPalaceRole::QuanLoc => "Quan Lộc",
            TuViPalaceRole::NoBoc => "Nô Bộc",
            TuViPalaceRole::ThienDi => "Thiên Di",
            TuViPalaceRole::TatAch => "Tật Ách",
            TuViPalaceRole::TaiBach => "Tài Bạch",
            TuViPalaceRole::TuTuc => "Tử Tức",
            TuViPalaceRole::PhuThe => "Phu Thê",
            TuViPalaceRole::HuynhDe => "Huynh Đệ",
        }
    }
}

/// The five bureaus (Ngũ Hành Cục).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FiveElementsBureau {
    /// Thủy Nhị Cục, 2
    ThuyNhi,
    /// Mộc Tam Cục, 3
    MocTam,
    /// Kim Tứ Cục, 4
    KimTu,
    /// Thổ Ngũ Cục, 5
    ThoNgu,
    /// Hỏa Lục Cục, 6
    HoaLuc,
}

impl FiveElementsBureau {
    /// All bureaus by number, from Thủy Nhị Cục.
    pub const ALL: [FiveElementsBureau; 5] = [
        FiveElementsBureau::ThuyNhi,
        FiveElementsBureau::MocTam,
        FiveElementsBureau::KimTu,
        FiveElementsBureau::ThoNgu,
        FiveElementsBureau::HoaLuc,
    ];

    /// The bureau of an element.
    pub const fn from_element(element: FiveElements) -> Self {
        match element {
            FiveElements::Thuy => FiveElementsBureau::ThuyNhi,
            FiveElements::Moc => FiveElementsBureau::MocTam,
            FiveElements::Kim => FiveElementsBureau::KimTu,
            FiveElements::Tho => FiveElementsBureau::ThoNgu,
            FiveElements::Hoa => FiveElementsBureau::HoaLuc,
        }
    }

    /// Bureau number, from 2 to 6.
    pub const fn number(self) -> u8 {
        self as u8 + 2
    }

    /// Element of the bureau.
    pub const fn element(self) -> FiveElements {
        match self {
            FiveElementsBureau::ThuyNhi => FiveElements::Thuy,
            FiveElementsBureau::MocTam => FiveElements::Moc,
            FiveElementsBureau::KimTu => FiveElements::Kim,
            FiveElementsBureau::ThoNgu => FiveElements::Tho,
            FiveElementsBureau::HoaLuc => FiveElements::Hoa,
        }
    }

    /// Palace where the Trường Sinh ring starts.
    pub const fn truong_sinh_start(self) -> Branch {
        match self {
            FiveElementsBureau::ThuyNhi | FiveElementsBureau::ThoNgu => Than,
            FiveElementsBureau::MocTam => Hoi,
            FiveElementsBureau::KimTu => Ti,
            FiveElementsBureau::HoaLuc => Dan,
        }
    }

    /// Vietnamese name, e.g. "Mộc Tam Cục".
    pub const fn name_vi(self) -> &'static str {
        match self {
            FiveElementsBureau::ThuyNhi => "Thủy Nhị Cục",
            FiveElementsBureau::MocTam => "Mộc Tam Cục",
            FiveElementsBureau::KimTu => "Kim Tứ Cục",
            FiveElementsBureau::ThoNgu => "Thổ Ngũ Cục",
            FiveElementsBureau::HoaLuc => "Hỏa Lục Cục",
        }
    }
}

/// Brightness of a star in a palace: see [`TuViStar::brightness_at`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StarBrightness {
    /// Miếu địa
    Mieu,
    /// Vượng địa
    Vuong,
    /// Đắc địa
    Dac,
    /// Bình hòa
    Binh,
    /// Hãm địa
    Ham,
}

impl StarBrightness {
    /// All levels, from the brightest.
    pub const ALL: [StarBrightness; 5] = [
        StarBrightness::Mieu,
        StarBrightness::Vuong,
        StarBrightness::Dac,
        StarBrightness::Binh,
        StarBrightness::Ham,
    ];

    /// Vietnamese name, e.g. "Miếu".
    pub const fn name_vi(self) -> &'static str {
        match self {
            StarBrightness::Mieu => "Miếu",
            StarBrightness::Vuong => "Vượng",
            StarBrightness::Dac => "Đắc",
            StarBrightness::Binh => "Bình",
            StarBrightness::Ham => "Hãm",
        }
    }

    /// One-letter mark used on the charts of the book: M, V, Đ, B or H.
    pub const fn mark(self) -> &'static str {
        match self {
            StarBrightness::Mieu => "M",
            StarBrightness::Vuong => "V",
            StarBrightness::Dac => "Đ",
            StarBrightness::Binh => "B",
            StarBrightness::Ham => "H",
        }
    }
}

/// Whether a star is one of the 14 Chính Tinh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StarCategory {
    /// Chính Tinh, the stars of the Tử Vi and Thiên Phủ groups
    ChinhTinh,
    /// Phụ Tinh, every other star
    PhuTinh,
}

/// Inherent nature of a star, before brightness or the stars it meets.
///
/// *Tân Biên* has no table of this. Section 4.9 of the spec derives it from the labels and the
/// descriptions in the book's chapter on star traits, and the project owner approved the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StarNature {
    /// Cát: favourable
    Cat,
    /// Trung: mixed, or as good or bad as the stars it meets
    Trung,
    /// Hung: harmful
    Hung,
}

impl StarNature {
    /// Vietnamese name, e.g. "Cát".
    pub const fn name_vi(self) -> &'static str {
        match self {
            StarNature::Cat => "Cát",
            StarNature::Trung => "Trung",
            StarNature::Hung => "Hung",
        }
    }
}

/// A star of the chart. The Four Transformations are not stars here: see
/// [`FourTransformations`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TuViStar {
    /// Tử Vi
    TuVi,
    /// Thiên Cơ
    ThienCo,
    /// Thái Dương
    ThaiDuong,
    /// Vũ Khúc
    VuKhuc,
    /// Thiên Đồng
    ThienDong,
    /// Liêm Trinh
    LiemTrinh,
    /// Thiên Phủ
    ThienPhu,
    /// Thái Âm
    ThaiAm,
    /// Tham Lang
    ThamLang,
    /// Cự Môn
    CuMon,
    /// Thiên Tướng
    ThienTuong,
    /// Thiên Lương
    ThienLuong,
    /// Thất Sát
    ThatSat,
    /// Phá Quân
    PhaQuan,
    /// Thái Tuế
    ThaiTue,
    /// Thiếu Dương
    ThieuDuong,
    /// Tang Môn
    TangMon,
    /// Thiếu Âm
    ThieuAm,
    /// Quan Phù, of the Thái Tuế ring
    QuanPhuThaiTue,
    /// Tử Phù
    TuPhu,
    /// Tuế Phá
    TuePha,
    /// Long Đức
    LongDuc,
    /// Bạch Hổ
    BachHo,
    /// Phúc Đức
    PhucDuc,
    /// Điếu Khách
    DieuKhach,
    /// Trực Phù
    TrucPhu,
    /// Lộc Tồn
    LocTon,
    /// Kình Dương
    KinhDuong,
    /// Đà La
    DaLa,
    /// Bác Sỹ
    BacSy,
    /// Lực Sỹ
    LucSy,
    /// Thanh Long
    ThanhLong,
    /// Tiểu Hao
    TieuHao,
    /// Tướng Quân
    TuongQuan,
    /// Tấu Thư
    TauThu,
    /// Phi Liêm
    PhiLiem,
    /// Hỷ Thần
    HyThan,
    /// Bệnh Phù
    BenhPhu,
    /// Đại Hao
    DaiHao,
    /// Phục Binh
    PhucBinh,
    /// Quan Phủ, of the Bác Sỹ ring
    QuanPhuBacSy,
    /// Trường Sinh
    TruongSinh,
    /// Mộc Dục
    MocDuc,
    /// Quan Đới
    QuanDoi,
    /// Lâm Quan
    LamQuan,
    /// Đế Vượng
    DeVuong,
    /// Suy
    Suy,
    /// Bệnh
    Benh,
    /// Tử
    Tu,
    /// Mộ
    Mo,
    /// Tuyệt
    Tuyet,
    /// Thai
    Thai,
    /// Dưỡng
    Duong,
    /// Tả Phụ
    TaPhu,
    /// Hữu Bật
    HuuBat,
    /// Thiên Hình
    ThienHinh,
    /// Thiên Riêu
    ThienRieu,
    /// Thiên Y
    ThienY,
    /// Thiên Giải
    ThienGiai,
    /// Địa Giải
    DiaGiai,
    /// Văn Xương
    VanXuong,
    /// Văn Khúc
    VanKhuc,
    /// Địa Không
    DiaKhong,
    /// Địa Kiếp
    DiaKiep,
    /// Thai Phụ
    ThaiPhu,
    /// Phong Cáo
    PhongCao,
    /// Tam Thai
    TamThai,
    /// Bát Tọa
    BatToa,
    /// Ân Quang
    AnQuang,
    /// Thiên Quý
    ThienQuy,
    /// Hỏa Tinh
    HoaTinh,
    /// Linh Tinh
    LinhTinh,
    /// Thiên Khôi
    ThienKhoi,
    /// Thiên Việt
    ThienViet,
    /// Thiên Quan
    ThienQuan,
    /// Thiên Phúc
    ThienPhuc,
    /// Lưu Hà
    LuuHa,
    /// Thiên Trù
    ThienTru,
    /// Quốc Ấn
    QuocAn,
    /// Đường Phù
    DuongPhu,
    /// Long Trì
    LongTri,
    /// Phượng Các
    PhuongCac,
    /// Giải Thần
    GiaiThan,
    /// Hồng Loan
    HongLoan,
    /// Thiên Hỷ
    ThienHy,
    /// Cô Thần
    CoThan,
    /// Quả Tú
    QuaTu,
    /// Đào Hoa
    DaoHoa,
    /// Thiên Mã
    ThienMa,
    /// Kiếp Sát
    KiepSat,
    /// Hoa Cái
    HoaCai,
    /// Phá Toái
    PhaToai,
    /// Thiên Không
    ThienKhong,
    /// Thiên Khốc
    ThienKhoc,
    /// Thiên Hư
    ThienHu,
    /// Thiên Đức
    ThienDuc,
    /// Nguyệt Đức
    NguyetDuc,
    /// Thiên Tài
    ThienTai,
    /// Thiên Thọ
    ThienTho,
    /// Thiên Thương
    ThienThuong,
    /// Thiên Sứ
    ThienSu,
    /// Thiên La
    ThienLa,
    /// Địa Võng
    DiaVong,
    /// Lưu Niên Văn Tinh
    LuuNienVanTinh,
    /// Đẩu Quân
    DauQuan,
}

impl TuViStar {
    /// Number of stars.
    pub const COUNT: usize = 105;

    /// All stars, in the order of the enum.
    pub const ALL: [TuViStar; TuViStar::COUNT] = [
        TuViStar::TuVi,
        TuViStar::ThienCo,
        TuViStar::ThaiDuong,
        TuViStar::VuKhuc,
        TuViStar::ThienDong,
        TuViStar::LiemTrinh,
        TuViStar::ThienPhu,
        TuViStar::ThaiAm,
        TuViStar::ThamLang,
        TuViStar::CuMon,
        TuViStar::ThienTuong,
        TuViStar::ThienLuong,
        TuViStar::ThatSat,
        TuViStar::PhaQuan,
        TuViStar::ThaiTue,
        TuViStar::ThieuDuong,
        TuViStar::TangMon,
        TuViStar::ThieuAm,
        TuViStar::QuanPhuThaiTue,
        TuViStar::TuPhu,
        TuViStar::TuePha,
        TuViStar::LongDuc,
        TuViStar::BachHo,
        TuViStar::PhucDuc,
        TuViStar::DieuKhach,
        TuViStar::TrucPhu,
        TuViStar::LocTon,
        TuViStar::KinhDuong,
        TuViStar::DaLa,
        TuViStar::BacSy,
        TuViStar::LucSy,
        TuViStar::ThanhLong,
        TuViStar::TieuHao,
        TuViStar::TuongQuan,
        TuViStar::TauThu,
        TuViStar::PhiLiem,
        TuViStar::HyThan,
        TuViStar::BenhPhu,
        TuViStar::DaiHao,
        TuViStar::PhucBinh,
        TuViStar::QuanPhuBacSy,
        TuViStar::TruongSinh,
        TuViStar::MocDuc,
        TuViStar::QuanDoi,
        TuViStar::LamQuan,
        TuViStar::DeVuong,
        TuViStar::Suy,
        TuViStar::Benh,
        TuViStar::Tu,
        TuViStar::Mo,
        TuViStar::Tuyet,
        TuViStar::Thai,
        TuViStar::Duong,
        TuViStar::TaPhu,
        TuViStar::HuuBat,
        TuViStar::ThienHinh,
        TuViStar::ThienRieu,
        TuViStar::ThienY,
        TuViStar::ThienGiai,
        TuViStar::DiaGiai,
        TuViStar::VanXuong,
        TuViStar::VanKhuc,
        TuViStar::DiaKhong,
        TuViStar::DiaKiep,
        TuViStar::ThaiPhu,
        TuViStar::PhongCao,
        TuViStar::TamThai,
        TuViStar::BatToa,
        TuViStar::AnQuang,
        TuViStar::ThienQuy,
        TuViStar::HoaTinh,
        TuViStar::LinhTinh,
        TuViStar::ThienKhoi,
        TuViStar::ThienViet,
        TuViStar::ThienQuan,
        TuViStar::ThienPhuc,
        TuViStar::LuuHa,
        TuViStar::ThienTru,
        TuViStar::QuocAn,
        TuViStar::DuongPhu,
        TuViStar::LongTri,
        TuViStar::PhuongCac,
        TuViStar::GiaiThan,
        TuViStar::HongLoan,
        TuViStar::ThienHy,
        TuViStar::CoThan,
        TuViStar::QuaTu,
        TuViStar::DaoHoa,
        TuViStar::ThienMa,
        TuViStar::KiepSat,
        TuViStar::HoaCai,
        TuViStar::PhaToai,
        TuViStar::ThienKhong,
        TuViStar::ThienKhoc,
        TuViStar::ThienHu,
        TuViStar::ThienDuc,
        TuViStar::NguyetDuc,
        TuViStar::ThienTai,
        TuViStar::ThienTho,
        TuViStar::ThienThuong,
        TuViStar::ThienSu,
        TuViStar::ThienLa,
        TuViStar::DiaVong,
        TuViStar::LuuNienVanTinh,
        TuViStar::DauQuan,
    ];

    /// Vietnamese name, e.g. "Thiên Phủ".
    pub const fn name_vi(self) -> &'static str {
        match self {
            TuViStar::TuVi => "Tử Vi",
            TuViStar::ThienCo => "Thiên Cơ",
            TuViStar::ThaiDuong => "Thái Dương",
            TuViStar::VuKhuc => "Vũ Khúc",
            TuViStar::ThienDong => "Thiên Đồng",
            TuViStar::LiemTrinh => "Liêm Trinh",
            TuViStar::ThienPhu => "Thiên Phủ",
            TuViStar::ThaiAm => "Thái Âm",
            TuViStar::ThamLang => "Tham Lang",
            TuViStar::CuMon => "Cự Môn",
            TuViStar::ThienTuong => "Thiên Tướng",
            TuViStar::ThienLuong => "Thiên Lương",
            TuViStar::ThatSat => "Thất Sát",
            TuViStar::PhaQuan => "Phá Quân",
            TuViStar::ThaiTue => "Thái Tuế",
            TuViStar::ThieuDuong => "Thiếu Dương",
            TuViStar::TangMon => "Tang Môn",
            TuViStar::ThieuAm => "Thiếu Âm",
            TuViStar::QuanPhuThaiTue => "Quan Phù",
            TuViStar::TuPhu => "Tử Phù",
            TuViStar::TuePha => "Tuế Phá",
            TuViStar::LongDuc => "Long Đức",
            TuViStar::BachHo => "Bạch Hổ",
            TuViStar::PhucDuc => "Phúc Đức",
            TuViStar::DieuKhach => "Điếu Khách",
            TuViStar::TrucPhu => "Trực Phù",
            TuViStar::LocTon => "Lộc Tồn",
            TuViStar::KinhDuong => "Kình Dương",
            TuViStar::DaLa => "Đà La",
            TuViStar::BacSy => "Bác Sỹ",
            TuViStar::LucSy => "Lực Sỹ",
            TuViStar::ThanhLong => "Thanh Long",
            TuViStar::TieuHao => "Tiểu Hao",
            TuViStar::TuongQuan => "Tướng Quân",
            TuViStar::TauThu => "Tấu Thư",
            TuViStar::PhiLiem => "Phi Liêm",
            TuViStar::HyThan => "Hỷ Thần",
            TuViStar::BenhPhu => "Bệnh Phù",
            TuViStar::DaiHao => "Đại Hao",
            TuViStar::PhucBinh => "Phục Binh",
            TuViStar::QuanPhuBacSy => "Quan Phủ",
            TuViStar::TruongSinh => "Trường Sinh",
            TuViStar::MocDuc => "Mộc Dục",
            TuViStar::QuanDoi => "Quan Đới",
            TuViStar::LamQuan => "Lâm Quan",
            TuViStar::DeVuong => "Đế Vượng",
            TuViStar::Suy => "Suy",
            TuViStar::Benh => "Bệnh",
            TuViStar::Tu => "Tử",
            TuViStar::Mo => "Mộ",
            TuViStar::Tuyet => "Tuyệt",
            TuViStar::Thai => "Thai",
            TuViStar::Duong => "Dưỡng",
            TuViStar::TaPhu => "Tả Phụ",
            TuViStar::HuuBat => "Hữu Bật",
            TuViStar::ThienHinh => "Thiên Hình",
            TuViStar::ThienRieu => "Thiên Riêu",
            TuViStar::ThienY => "Thiên Y",
            TuViStar::ThienGiai => "Thiên Giải",
            TuViStar::DiaGiai => "Địa Giải",
            TuViStar::VanXuong => "Văn Xương",
            TuViStar::VanKhuc => "Văn Khúc",
            TuViStar::DiaKhong => "Địa Không",
            TuViStar::DiaKiep => "Địa Kiếp",
            TuViStar::ThaiPhu => "Thai Phụ",
            TuViStar::PhongCao => "Phong Cáo",
            TuViStar::TamThai => "Tam Thai",
            TuViStar::BatToa => "Bát Tọa",
            TuViStar::AnQuang => "Ân Quang",
            TuViStar::ThienQuy => "Thiên Quý",
            TuViStar::HoaTinh => "Hỏa Tinh",
            TuViStar::LinhTinh => "Linh Tinh",
            TuViStar::ThienKhoi => "Thiên Khôi",
            TuViStar::ThienViet => "Thiên Việt",
            TuViStar::ThienQuan => "Thiên Quan",
            TuViStar::ThienPhuc => "Thiên Phúc",
            TuViStar::LuuHa => "Lưu Hà",
            TuViStar::ThienTru => "Thiên Trù",
            TuViStar::QuocAn => "Quốc Ấn",
            TuViStar::DuongPhu => "Đường Phù",
            TuViStar::LongTri => "Long Trì",
            TuViStar::PhuongCac => "Phượng Các",
            TuViStar::GiaiThan => "Giải Thần",
            TuViStar::HongLoan => "Hồng Loan",
            TuViStar::ThienHy => "Thiên Hỷ",
            TuViStar::CoThan => "Cô Thần",
            TuViStar::QuaTu => "Quả Tú",
            TuViStar::DaoHoa => "Đào Hoa",
            TuViStar::ThienMa => "Thiên Mã",
            TuViStar::KiepSat => "Kiếp Sát",
            TuViStar::HoaCai => "Hoa Cái",
            TuViStar::PhaToai => "Phá Toái",
            TuViStar::ThienKhong => "Thiên Không",
            TuViStar::ThienKhoc => "Thiên Khốc",
            TuViStar::ThienHu => "Thiên Hư",
            TuViStar::ThienDuc => "Thiên Đức",
            TuViStar::NguyetDuc => "Nguyệt Đức",
            TuViStar::ThienTai => "Thiên Tài",
            TuViStar::ThienTho => "Thiên Thọ",
            TuViStar::ThienThuong => "Thiên Thương",
            TuViStar::ThienSu => "Thiên Sứ",
            TuViStar::ThienLa => "Thiên La",
            TuViStar::DiaVong => "Địa Võng",
            TuViStar::LuuNienVanTinh => "Lưu Niên Văn Tinh",
            TuViStar::DauQuan => "Đẩu Quân",
        }
    }

    /// Whether this is one of the 14 Chính Tinh.
    pub const fn category(self) -> StarCategory {
        if (self as usize) < 14 {
            StarCategory::ChinhTinh
        } else {
            StarCategory::PhuTinh
        }
    }

    /// Ngũ hành of a Chính Tinh, ignoring Âm/Dương, from the heading of each star (pp. 33–38);
    /// `None` for every other star.
    pub const fn element(self) -> Option<FiveElements> {
        if (self as usize) < 14 {
            Some(ELEMENT[self as usize])
        } else {
            None
        }
    }

    /// Brightness in the palace at `branch`: every palace for the Chính Tinh, the places listed by
    /// [`minor_brightness`] for 17 other stars, `None` otherwise.
    pub const fn brightness_at(self, branch: Branch) -> Option<StarBrightness> {
        if (self as usize) < 14 {
            return Some(BRIGHTNESS[self as usize][branch.index() as usize]);
        }
        match minor_brightness(self) {
            Some((dac, others)) => brightness_in(dac, others, branch),
            None => None,
        }
    }

    /// Inherent nature, from section 4.9 of the spec: see [`StarNature`]. Thất Sát is the one
    /// exception to the book's labels. They call it Quyền tinh, Dũng tinh, but it counts as `Hung`
    /// with Phá Quân and Tham Lang, the Sát, Phá, Tham group of the tradition.
    pub const fn nature(self) -> StarNature {
        match self {
            TuViStar::TuVi
            | TuViStar::ThienCo
            | TuViStar::ThaiDuong
            | TuViStar::VuKhuc
            | TuViStar::ThienDong
            | TuViStar::ThienPhu
            | TuViStar::ThaiAm
            | TuViStar::ThienTuong
            | TuViStar::ThieuDuong
            | TuViStar::ThieuAm
            | TuViStar::LongDuc
            | TuViStar::PhucDuc
            | TuViStar::LocTon
            | TuViStar::BacSy
            | TuViStar::ThanhLong
            | TuViStar::TauThu
            | TuViStar::HyThan
            | TuViStar::TruongSinh
            | TuViStar::DeVuong
            | TuViStar::Duong
            | TuViStar::TaPhu
            | TuViStar::HuuBat
            | TuViStar::ThienY
            | TuViStar::ThienGiai
            | TuViStar::DiaGiai
            | TuViStar::VanXuong
            | TuViStar::VanKhuc
            | TuViStar::TamThai
            | TuViStar::BatToa
            | TuViStar::AnQuang
            | TuViStar::ThienQuy
            | TuViStar::ThienKhoi
            | TuViStar::ThienViet
            | TuViStar::ThienQuan
            | TuViStar::ThienPhuc
            | TuViStar::ThienTru
            | TuViStar::QuocAn
            | TuViStar::DuongPhu
            | TuViStar::LongTri
            | TuViStar::PhuongCac
            | TuViStar::GiaiThan
            | TuViStar::ThienHy
            | TuViStar::ThienMa
            | TuViStar::HoaCai
            | TuViStar::ThienDuc
            | TuViStar::NguyetDuc
            | TuViStar::ThienTho
            | TuViStar::LuuNienVanTinh => StarNature::Cat,
            TuViStar::LiemTrinh
            | TuViStar::CuMon
            | TuViStar::ThienLuong
            | TuViStar::ThaiTue
            | TuViStar::QuanPhuThaiTue
            | TuViStar::LucSy
            | TuViStar::TuongQuan
            | TuViStar::PhiLiem
            | TuViStar::PhucBinh
            | TuViStar::QuanDoi
            | TuViStar::LamQuan
            | TuViStar::Tu
            | TuViStar::Mo
            | TuViStar::Tuyet
            | TuViStar::Thai
            | TuViStar::ThienHinh
            | TuViStar::ThienRieu
            | TuViStar::ThaiPhu
            | TuViStar::PhongCao
            | TuViStar::HongLoan
            | TuViStar::CoThan
            | TuViStar::QuaTu
            | TuViStar::DaoHoa
            | TuViStar::ThienTai
            | TuViStar::DauQuan => StarNature::Trung,
            TuViStar::ThamLang
            | TuViStar::ThatSat
            | TuViStar::PhaQuan
            | TuViStar::TangMon
            | TuViStar::TuPhu
            | TuViStar::TuePha
            | TuViStar::BachHo
            | TuViStar::DieuKhach
            | TuViStar::TrucPhu
            | TuViStar::KinhDuong
            | TuViStar::DaLa
            | TuViStar::TieuHao
            | TuViStar::BenhPhu
            | TuViStar::DaiHao
            | TuViStar::QuanPhuBacSy
            | TuViStar::MocDuc
            | TuViStar::Suy
            | TuViStar::Benh
            | TuViStar::DiaKhong
            | TuViStar::DiaKiep
            | TuViStar::HoaTinh
            | TuViStar::LinhTinh
            | TuViStar::LuuHa
            | TuViStar::KiepSat
            | TuViStar::PhaToai
            | TuViStar::ThienKhong
            | TuViStar::ThienKhoc
            | TuViStar::ThienHu
            | TuViStar::ThienThuong
            | TuViStar::ThienSu
            | TuViStar::ThienLa
            | TuViStar::DiaVong => StarNature::Hung,
        }
    }
}

/// `Dac` at the branches of `dac`, `others` elsewhere.
const fn brightness_in(
    dac: &[Branch],
    others: Option<StarBrightness>,
    branch: Branch,
) -> Option<StarBrightness> {
    let mut index = 0;
    while index < dac.len() {
        if dac[index] as u8 == branch as u8 {
            return Some(StarBrightness::Dac);
        }
        index += 1;
    }
    others
}

/// The palaces where a minor star is Đắc địa, from pp. 39–43, and its brightness elsewhere: `Ham`
/// for the stars the book calls Hãm địa, `None` for Đại Hao, Tiểu Hao, Thiên Mã and Thiên Hình,
/// which it never does. The list of the book names the Đắc địa only.
const fn minor_brightness(star: TuViStar) -> Option<(&'static [Branch], Option<StarBrightness>)> {
    const HAM: Option<StarBrightness> = Some(StarBrightness::Ham);
    match star {
        TuViStar::KinhDuong | TuViStar::DaLa => Some((&[Thin, Tuat, Suu, Mui], HAM)),
        TuViStar::HoaTinh | TuViStar::LinhTinh => Some((&[Dan, Mao, Thin, Ti, Ngo], HAM)),
        TuViStar::VanXuong | TuViStar::VanKhuc => Some((&[Thin, Tuat, Suu, Mui, Ti, Hoi], HAM)),
        TuViStar::DiaKhong | TuViStar::DiaKiep => Some((&[Dan, Than, Ti, Hoi], HAM)),
        TuViStar::TangMon | TuViStar::BachHo => Some((&[Dan, Than, Mao, Dau], HAM)),
        TuViStar::ThienKhoc | TuViStar::ThienHu => Some((&[Ty, Ngo, Mao, Dau, Suu, Mui], HAM)),
        TuViStar::ThienRieu => Some((&[Dan, Mao, Dau, Tuat], HAM)),
        TuViStar::DaiHao | TuViStar::TieuHao | TuViStar::ThienHinh => {
            Some((&[Dan, Than, Mao, Dau], None))
        }
        TuViStar::ThienMa => Some((&[Ti, Dan], None)),
        _ => None,
    }
}

/// Rows of [`TuViStar::brightness_at`], in the order of the Chính Tinh in [`TuViStar`]; columns
/// from Tý to Hợi. From the lists on pp. 33–38 of the book.
const BRIGHTNESS: [[StarBrightness; 12]; 14] = {
    use StarBrightness::{Binh as B, Dac as D, Ham as H, Mieu as M, Vuong as V};
    [
        [B, D, M, B, V, M, M, D, M, B, V, B],
        [D, D, H, M, M, V, D, D, V, M, M, H],
        [H, D, V, V, V, M, M, D, H, H, H, H],
        [V, M, V, D, M, H, V, M, V, D, M, H],
        [V, H, M, D, H, D, H, H, M, H, H, D],
        [V, D, V, H, M, H, V, D, V, H, M, H],
        [M, B, M, B, V, D, M, D, M, B, V, D],
        [V, D, H, H, H, H, H, D, V, M, M, M],
        [H, M, D, H, V, H, H, M, D, H, V, H],
        [V, H, V, M, H, H, V, H, D, M, H, D],
        [V, D, M, H, V, D, V, D, M, H, V, D],
        [V, D, V, V, M, H, M, D, V, H, M, H],
        [M, D, M, H, H, V, M, D, M, H, H, V],
        [M, V, H, H, D, H, M, V, H, H, D, H],
    ]
};

/// Ngũ hành of the Chính Tinh, in the order of [`TuViStar`], read from the heading of each star
/// (pp. 33–38 of the book): Tử Vi Dương Thổ, Thiên Cơ Âm Mộc, Thái Dương Dương Hỏa, Vũ Khúc Âm
/// Kim, Thiên Đồng Dương Thủy, Liêm Trinh Âm Hỏa, Thiên Phủ Âm Thổ, Thái Âm Âm Thủy, Tham Lang Âm
/// Thủy, Cự Môn Âm Thủy, Thiên Tướng Dương Thủy, Thiên Lương Âm Mộc, Thất Sát Dương Kim, Phá Quân
/// Âm Thủy.
const ELEMENT: [FiveElements; 14] = {
    use FiveElements::{Hoa, Kim, Moc, Tho, Thuy};
    [Tho, Moc, Hoa, Kim, Thuy, Hoa, Tho, Thuy, Thuy, Thuy, Thuy, Moc, Kim, Thuy]
};

/// The Four Transformations (Tứ Hóa). Each one attaches to a star set by the year stem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FourTransformations {
    /// Hóa Lộc
    HoaLoc,
    /// Hóa Quyền
    HoaQuyen,
    /// Hóa Khoa
    HoaKhoa,
    /// Hóa Kỵ
    HoaKy,
}

impl FourTransformations {
    /// All transformations, in the order of the book's table.
    pub const ALL: [FourTransformations; 4] = [
        FourTransformations::HoaLoc,
        FourTransformations::HoaQuyen,
        FourTransformations::HoaKhoa,
        FourTransformations::HoaKy,
    ];

    /// The star that takes this transformation in a year with stem `year_stem`.
    pub const fn star_for(self, year_stem: Stem) -> TuViStar {
        TU_HOA[year_stem.index() as usize][self as usize]
    }

    /// Brightness in the palace at `branch`: only Hóa Kỵ has one, from p. 41.
    pub const fn brightness_at(self, branch: Branch) -> Option<StarBrightness> {
        match self {
            FourTransformations::HoaKy => {
                brightness_in(&[Thin, Tuat, Suu, Mui], Some(StarBrightness::Ham), branch)
            }
            _ => None,
        }
    }

    /// Inherent nature, by rule 4 of section 4.9 of the spec.
    pub const fn nature(self) -> StarNature {
        match self {
            FourTransformations::HoaLoc | FourTransformations::HoaKhoa => StarNature::Cat,
            FourTransformations::HoaQuyen => StarNature::Trung,
            FourTransformations::HoaKy => StarNature::Hung,
        }
    }

    /// Vietnamese name, e.g. "Hóa Lộc".
    pub const fn name_vi(self) -> &'static str {
        match self {
            FourTransformations::HoaLoc => "Hóa Lộc",
            FourTransformations::HoaQuyen => "Hóa Quyền",
            FourTransformations::HoaKhoa => "Hóa Khoa",
            FourTransformations::HoaKy => "Hóa Kỵ",
        }
    }
}

/// Stars of Hóa Lộc, Hóa Quyền, Hóa Khoa and Hóa Kỵ by year stem, from p. 14 of the book.
///
/// - Tân: Hóa Quyền is Thái Dương. The table on p. 14 misprints Thiên Lương, but both Tân charts
///   of the book put Hóa Quyền with Thái Dương.
/// - Nhâm: Hóa Khoa is Tả Phụ, as on p. 14. Some variant texts misread Tả Phụ as Thiên Phủ.
/// - Canh: Thái Dương, Vũ Khúc, Thái Âm, Thiên Đồng.
const TU_HOA: [[TuViStar; 4]; 10] = [
    [
        TuViStar::LiemTrinh,
        TuViStar::PhaQuan,
        TuViStar::VuKhuc,
        TuViStar::ThaiDuong,
    ],
    [
        TuViStar::ThienCo,
        TuViStar::ThienLuong,
        TuViStar::TuVi,
        TuViStar::ThaiAm,
    ],
    [
        TuViStar::ThienDong,
        TuViStar::ThienCo,
        TuViStar::VanXuong,
        TuViStar::LiemTrinh,
    ],
    [
        TuViStar::ThaiAm,
        TuViStar::ThienDong,
        TuViStar::ThienCo,
        TuViStar::CuMon,
    ],
    [
        TuViStar::ThamLang,
        TuViStar::ThaiAm,
        TuViStar::HuuBat,
        TuViStar::ThienCo,
    ],
    [
        TuViStar::VuKhuc,
        TuViStar::ThamLang,
        TuViStar::ThienLuong,
        TuViStar::VanKhuc,
    ],
    [
        TuViStar::ThaiDuong,
        TuViStar::VuKhuc,
        TuViStar::ThaiAm,
        TuViStar::ThienDong,
    ],
    [
        TuViStar::CuMon,
        TuViStar::ThaiDuong,
        TuViStar::VanKhuc,
        TuViStar::VanXuong,
    ],
    [
        TuViStar::ThienLuong,
        TuViStar::TuVi,
        TuViStar::TaPhu,
        TuViStar::VuKhuc,
    ],
    [
        TuViStar::PhaQuan,
        TuViStar::CuMon,
        TuViStar::ThaiAm,
        TuViStar::ThamLang,
    ],
];

/// Lộc Tồn by year stem, from Giáp to Quý.
const LOC_TON: [Branch; 10] = [Dan, Mao, Ti, Ngo, Ti, Ngo, Than, Dau, Hoi, Ty];
/// Thiên Khôi by year stem.
const THIEN_KHOI: [Branch; 10] = [Suu, Ty, Hoi, Hoi, Suu, Ty, Ngo, Ngo, Mao, Mao];
/// Thiên Việt by year stem.
const THIEN_VIET: [Branch; 10] = [Mui, Than, Dau, Dau, Mui, Than, Dan, Dan, Ti, Ti];
/// Thiên Quan by year stem.
const THIEN_QUAN: [Branch; 10] = [Mui, Thin, Ti, Dan, Mao, Dau, Hoi, Dau, Tuat, Ngo];
/// Thiên Phúc by year stem.
const THIEN_PHUC: [Branch; 10] = [Dau, Than, Ty, Hoi, Mao, Dan, Ngo, Ti, Ngo, Ti];
/// Lưu Hà by year stem, from p. 16 of the book. Tân is Mão: the two Tân charts of the book
/// misprint it at Thìn, which belongs to Đinh.
const LUU_HA: [Branch; 10] = [Dau, Tuat, Mui, Thin, Ti, Ngo, Than, Mao, Hoi, Dan];
/// Thiên Trù by year stem.
const THIEN_TRU: [Branch; 10] = [Ti, Ngo, Ty, Ti, Ngo, Than, Dan, Ngo, Dau, Tuat];
/// Lưu Niên Văn Tinh by year stem.
const LUU_NIEN_VAN_TINH: [Branch; 10] = [Ti, Ngo, Than, Dau, Than, Dau, Hoi, Ty, Dan, Mao];
/// The two palaces of Triệt for stems Giáp/Kỷ, Ất/Canh, Bính/Tân, Đinh/Nhâm and Mậu/Quý.
const TRIET: [(Branch, Branch); 5] = [(Than, Dau), (Ngo, Mui), (Thin, Ti), (Dan, Mao), (Ty, Suu)];

// By trine of the year branch, in the row order of `Branch::trine_group`: Thân-Tý-Thìn,
// Dần-Ngọ-Tuất, Tỵ-Dậu-Sửu, Hợi-Mão-Mùi.

/// Palaces where Hỏa Tinh and Linh Tinh start. Hợi-Mão-Mùi starts Hỏa Tinh at Dậu: p. 11 of the
/// book misprints Dần, and working back from its two such charts gives Dậu.
const HOA_LINH_START: [(Branch, Branch); 4] = [(Dan, Tuat), (Suu, Mao), (Mao, Tuat), (Dau, Tuat)];
/// Đào Hoa.
const DAO_HOA: [Branch; 4] = [Dau, Mao, Ngo, Ty];
/// Thiên Mã.
const THIEN_MA: [Branch; 4] = [Dan, Than, Hoi, Ti];
/// Kiếp Sát.
const KIEP_SAT: [Branch; 4] = [Ti, Hoi, Dan, Than];
/// Hoa Cái.
const HOA_CAI: [Branch; 4] = [Thin, Tuat, Suu, Mui];
/// Palace of the Tiểu Hạn at age 1.
const TIEU_HAN_START: [Branch; 4] = [Tuat, Thin, Mui, Suu];

/// The Thái Tuế ring, from the palace of the year branch.
const THAI_TUE_RING: [TuViStar; 12] = [
    TuViStar::ThaiTue,
    TuViStar::ThieuDuong,
    TuViStar::TangMon,
    TuViStar::ThieuAm,
    TuViStar::QuanPhuThaiTue,
    TuViStar::TuPhu,
    TuViStar::TuePha,
    TuViStar::LongDuc,
    TuViStar::BachHo,
    TuViStar::PhucDuc,
    TuViStar::DieuKhach,
    TuViStar::TrucPhu,
];

/// The Bác Sỹ ring, from the palace of Lộc Tồn.
const BAC_SY_RING: [TuViStar; 12] = [
    TuViStar::BacSy,
    TuViStar::LucSy,
    TuViStar::ThanhLong,
    TuViStar::TieuHao,
    TuViStar::TuongQuan,
    TuViStar::TauThu,
    TuViStar::PhiLiem,
    TuViStar::HyThan,
    TuViStar::BenhPhu,
    TuViStar::DaiHao,
    TuViStar::PhucBinh,
    TuViStar::QuanPhuBacSy,
];

/// The Trường Sinh ring, in the order of [`LifeStage::ALL`].
const TRUONG_SINH_RING: [TuViStar; 12] = [
    TuViStar::TruongSinh,
    TuViStar::MocDuc,
    TuViStar::QuanDoi,
    TuViStar::LamQuan,
    TuViStar::DeVuong,
    TuViStar::Suy,
    TuViStar::Benh,
    TuViStar::Tu,
    TuViStar::Mo,
    TuViStar::Tuyet,
    TuViStar::Thai,
    TuViStar::Duong,
];

impl From<LifeStage> for TuViStar {
    fn from(stage: LifeStage) -> Self {
        TRUONG_SINH_RING[stage as usize]
    }
}

/// Sex and polarity of the birth year (Âm Dương Nam Nữ).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TuViPolarity {
    /// Dương Nam
    DuongNam,
    /// Âm Nam
    AmNam,
    /// Dương Nữ
    DuongNu,
    /// Âm Nữ
    AmNu,
}

impl TuViPolarity {
    /// Polarity of a person of `gender` born in a year with stem `year_stem`.
    pub const fn of(year_stem: Stem, gender: Gender) -> Self {
        match (year_stem.yin_yang(), gender) {
            (YinYang::Duong, Gender::Male) => TuViPolarity::DuongNam,
            (YinYang::Am, Gender::Male) => TuViPolarity::AmNam,
            (YinYang::Duong, Gender::Female) => TuViPolarity::DuongNu,
            (YinYang::Am, Gender::Female) => TuViPolarity::AmNu,
        }
    }

    /// Whether the rings and the Đại Hạn run clockwise: true for Dương Nam and Âm Nữ.
    pub const fn is_forward(self) -> bool {
        matches!(self, TuViPolarity::DuongNam | TuViPolarity::AmNu)
    }

    /// Vietnamese name, e.g. "Dương Nam".
    pub const fn name_vi(self) -> &'static str {
        match self {
            TuViPolarity::DuongNam => "Dương Nam",
            TuViPolarity::AmNam => "Âm Nam",
            TuViPolarity::DuongNu => "Dương Nữ",
            TuViPolarity::AmNu => "Âm Nữ",
        }
    }
}

/// Day (giờ Mão đến Thân) or night (giờ Dậu đến Dần) birth, used by passages that say "sinh ban
/// ngày" or "sinh ban đêm" without a sharper boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BirthTimeOfDay {
    /// Mão, Thìn, Tỵ, Ngọ, Mùi, Thân
    Day,
    /// Dậu, Tuất, Hợi, Tý, Sửu, Dần
    Night,
}

impl BirthTimeOfDay {
    /// Day for an `hour` from Mão to Thân, night otherwise.
    pub const fn of(hour: Branch) -> Self {
        match hour {
            Mao | Thin | Ti | Ngo | Mui | Than => BirthTimeOfDay::Day,
            _ => BirthTimeOfDay::Night,
        }
    }

    /// Vietnamese name, e.g. "Ban ngày".
    pub const fn name_vi(self) -> &'static str {
        match self {
            BirthTimeOfDay::Day => "Ban ngày",
            BirthTimeOfDay::Night => "Ban đêm",
        }
    }
}

/// Whether the birth year and the Mệnh palace share their polarity (pp. 31 and 52 of the book).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum YinYangHarmony {
    /// Thuận lý: same polarity
    ThuanLy,
    /// Nghịch lý: opposite polarity
    NghichLy,
}

impl YinYangHarmony {
    /// Harmony of a year with stem `year_stem` and a Mệnh palace at `menh`.
    pub const fn of(year_stem: Stem, menh: Branch) -> Self {
        if year_stem.index() % 2 == menh.index() % 2 {
            YinYangHarmony::ThuanLy
        } else {
            YinYangHarmony::NghichLy
        }
    }

    /// Vietnamese name, e.g. "Thuận lý".
    pub const fn name_vi(self) -> &'static str {
        match self {
            YinYangHarmony::ThuanLy => "Thuận lý",
            YinYangHarmony::NghichLy => "Nghịch lý",
        }
    }
}

/// Relation between the Bản Mệnh and the bureau (p. 31 of the book).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MenhCucRelation {
    /// Bản Mệnh sinh Cục, the best case in the book
    MenhSinhCuc,
    /// Cục sinh Bản Mệnh, good but less so
    CucSinhMenh,
    /// Bản Mệnh khắc Cục, very bad in the book
    MenhKhacCuc,
    /// Cục khắc Bản Mệnh
    CucKhacMenh,
    /// Same element
    BinhHoa,
}

impl MenhCucRelation {
    /// Relation of a Bản Mệnh of element `menh` to a bureau of element `cuc`.
    pub const fn of(menh: FiveElements, cuc: FiveElements) -> Self {
        if menh as u8 == cuc as u8 {
            MenhCucRelation::BinhHoa
        } else if generated_by(menh) as u8 == cuc as u8 {
            MenhCucRelation::MenhSinhCuc
        } else if generated_by(cuc) as u8 == menh as u8 {
            MenhCucRelation::CucSinhMenh
        } else if controlled_by(menh) as u8 == cuc as u8 {
            MenhCucRelation::MenhKhacCuc
        } else {
            MenhCucRelation::CucKhacMenh
        }
    }

    /// Vietnamese name, e.g. "Bản Mệnh sinh Cục".
    pub const fn name_vi(self) -> &'static str {
        match self {
            MenhCucRelation::MenhSinhCuc => "Bản Mệnh sinh Cục",
            MenhCucRelation::CucSinhMenh => "Cục sinh Bản Mệnh",
            MenhCucRelation::MenhKhacCuc => "Bản Mệnh khắc Cục",
            MenhCucRelation::CucKhacMenh => "Cục khắc Bản Mệnh",
            MenhCucRelation::BinhHoa => "Bình hòa",
        }
    }
}

/// Sinh, khắc relation from a first Ngũ Hành to a second, generic version of [`MenhCucRelation`].
/// Used to compare the Bản Mệnh, the Ngũ hành of a cung and the Ngũ hành of a Chính Tinh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ElementRelation {
    /// First sinh second
    Sinh,
    /// Second sinh first
    DuocSinh,
    /// First khắc second
    Khac,
    /// Second khắc first
    BiKhac,
    /// Same element
    BinhHoa,
}

impl ElementRelation {
    /// Relation from a first Ngũ Hành to a second.
    pub const fn of(first: FiveElements, second: FiveElements) -> Self {
        if first as u8 == second as u8 {
            ElementRelation::BinhHoa
        } else if generated_by(first) as u8 == second as u8 {
            ElementRelation::Sinh
        } else if generated_by(second) as u8 == first as u8 {
            ElementRelation::DuocSinh
        } else if controlled_by(first) as u8 == second as u8 {
            ElementRelation::Khac
        } else {
            ElementRelation::BiKhac
        }
    }

    /// Vietnamese name, e.g. "Sinh".
    pub const fn name_vi(self) -> &'static str {
        match self {
            ElementRelation::Sinh => "Sinh",
            ElementRelation::DuocSinh => "Được sinh",
            ElementRelation::Khac => "Khắc",
            ElementRelation::BiKhac => "Bị khắc",
            ElementRelation::BinhHoa => "Bình hòa",
        }
    }
}

/// The element that `element` generates.
const fn generated_by(element: FiveElements) -> FiveElements {
    match element {
        FiveElements::Kim => FiveElements::Thuy,
        FiveElements::Thuy => FiveElements::Moc,
        FiveElements::Moc => FiveElements::Hoa,
        FiveElements::Hoa => FiveElements::Tho,
        FiveElements::Tho => FiveElements::Kim,
    }
}

/// The element that `element` controls.
const fn controlled_by(element: FiveElements) -> FiveElements {
    match element {
        FiveElements::Kim => FiveElements::Moc,
        FiveElements::Moc => FiveElements::Tho,
        FiveElements::Tho => FiveElements::Thuy,
        FiveElements::Thuy => FiveElements::Hoa,
        FiveElements::Hoa => FiveElements::Kim,
    }
}

/// A star as it sits in a palace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TuViPlacedStar {
    /// The star.
    pub star: TuViStar,
    /// Its brightness in this palace, see [`TuViStar::brightness_at`].
    pub brightness: Option<StarBrightness>,
    /// The transformation it carries, if any.
    pub transformation: Option<FourTransformations>,
}

/// One palace of the chart.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TuViPalace {
    /// Earthly Branch of the palace.
    pub branch: Branch,
    /// Heavenly Stem of the palace (Ngũ Hổ Độn).
    pub stem: Stem,
    /// Function of the palace.
    pub role: TuViPalaceRole,
    /// Whether the Thân palace falls here.
    pub is_than: bool,
    /// Chính Tinh of the palace.
    pub main_stars: Vec<TuViPlacedStar>,
    /// All other stars of the palace, including the star of the Trường Sinh ring.
    pub minor_stars: Vec<TuViPlacedStar>,
    /// The Trường Sinh ring star of the palace, as a stage.
    pub life_stage: LifeStage,
    /// First age of the ten-year Đại Hạn of the palace; it lasts to `dai_han_start_age + 9`.
    pub dai_han_start_age: u8,
    /// Earthly Branch of the years whose Tiểu Hạn falls here.
    pub tieu_han_year_branch: Branch,
    /// First age whose Tiểu Hạn falls here, from 1 to 12; it comes back every 12 years.
    pub tieu_han_first_age: u8,
    /// Whether Tuần falls on this palace.
    pub has_tuan: bool,
    /// Whether Triệt falls on this palace.
    pub has_triet: bool,
}

/// A complete chart: the centre of the board (Thiên Bàn) and its twelve palaces (Địa Bàn).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TuViChart {
    /// Sex of the person.
    pub gender: Gender,
    /// Local time used, after true solar time; `None` for a chart cast from a lunar date.
    pub effective_time: Option<SolarDateTime>,
    /// Gregorian date of `lunar`.
    pub solar_date: SolarDate,
    /// Lunar date used, after the giờ Tý Sớm convention.
    pub lunar: LunarDate,
    /// Month of birth used for the stars, after [`LeapMonthRule`].
    pub effective_month: u8,
    /// Double-hour of birth.
    pub hour_branch: Branch,
    /// Can Chi of the lunar year.
    pub year_can_chi: CanChi,
    /// Can Chi of the lunar month.
    pub month_can_chi: CanChi,
    /// Can Chi of the day.
    pub day_can_chi: CanChi,
    /// Can Chi of the double-hour.
    pub hour_can_chi: CanChi,
    /// Sex and polarity of the birth year.
    pub polarity: TuViPolarity,
    /// Whether the rings and the Đại Hạn run clockwise.
    pub forward: bool,
    /// Bản Mệnh, the Nạp Âm of the birth year.
    pub ban_menh: NaYin,
    /// The bureau.
    pub bureau: FiveElementsBureau,
    /// Palace of Mệnh.
    pub menh_branch: Branch,
    /// Palace of Thân.
    pub than_branch: Branch,
    /// Function of the palace where Thân falls.
    pub than_role: TuViPalaceRole,
    /// Mệnh Chủ, see [`menh_chu`].
    pub menh_chu: TuViStar,
    /// Thân Chủ, see [`than_chu`].
    pub than_chu: TuViStar,
    /// Thuận lý or nghịch lý between the birth year and the Mệnh palace.
    pub harmony: YinYangHarmony,
    /// Relation between the Bản Mệnh and the bureau.
    pub menh_cuc: MenhCucRelation,
    /// The two palaces of Tuần.
    pub tuan: KhongVong,
    /// The two palaces of Triệt.
    pub triet: KhongVong,
    /// The twelve palaces, from Tý to Hợi.
    pub palaces: Vec<TuViPalace>,
}

impl TuViChart {
    /// The palace at `branch`.
    pub fn palace(&self, branch: Branch) -> &TuViPalace {
        &self.palaces[branch.index() as usize]
    }

    /// The palace with function `role`.
    pub fn palace_of_role(&self, role: TuViPalaceRole) -> &TuViPalace {
        self.palace(shift(self.menh_branch, role as i64))
    }

    /// The palace holding `star`.
    pub fn palace_of_star(&self, star: TuViStar) -> &TuViPalace {
        let holds = |palace: &&TuViPalace| {
            let mut stars = palace.main_stars.iter().chain(&palace.minor_stars);
            stars.any(|placed| placed.star == star)
        };
        self.palaces
            .iter()
            .find(holds)
            .expect("every star sits in one palace")
    }

    /// The palace of the Đại Hạn at tuổi mụ `age`. `None` before the first Đại Hạn and after the
    /// twelfth.
    pub fn dai_han_palace(&self, age: u8) -> Option<&TuViPalace> {
        self.palaces
            .iter()
            .find(|p| (p.dai_han_start_age..p.dai_han_start_age.saturating_add(10)).contains(&age))
    }

    /// The palace of the Tiểu Hạn at tuổi mụ `age`. `None` for age 0.
    pub fn tieu_han_palace(&self, age: u8) -> Option<&TuViPalace> {
        let first = (age.checked_sub(1)? % 12) + 1;
        self.palaces
            .iter()
            .find(|palace| palace.tieu_han_first_age == first)
    }
}

/// Moves `steps` palaces clockwise, or counterclockwise when negative.
const fn shift(branch: Branch, steps: i64) -> Branch {
    Branch::from_index(branch.index() as i64 + steps)
}

/// Palace of Mệnh for lunar month `month` and double-hour `hour`.
pub const fn menh_branch(month: u8, hour: Branch) -> Branch {
    shift(Dan, month as i64 - 1 - hour.index() as i64)
}

/// Palace of Thân for lunar month `month` and double-hour `hour`.
pub const fn than_branch(month: u8, hour: Branch) -> Branch {
    shift(Dan, month as i64 - 1 + hour.index() as i64)
}

/// Heavenly Stem of the palace at `branch` in a year with stem `year_stem` (Ngũ Hổ Độn). Stems
/// run on from Dần to Sửu, so Tý and Sửu take the tenth and eleventh after that of Dần.
pub const fn palace_stem(year_stem: Stem, branch: Branch) -> Stem {
    let dan_stem = (year_stem.index() as i64 % 5) * 2 + 2;
    Stem::from_index(dan_stem + (branch.index() as i64 - 2).rem_euclid(12))
}

/// The bureau of a year with stem `year_stem` and a Mệnh palace at `menh`: the Nạp Âm element of
/// the Can Chi of that palace.
pub const fn bureau_of(year_stem: Stem, menh: Branch) -> FiveElementsBureau {
    let pillar = CanChi {
        stem: palace_stem(year_stem, menh),
        branch: menh,
    };
    FiveElementsBureau::from_element(pillar.na_yin().element())
}

/// Palace of Tử Vi for lunar day `day` (1–30) in `bureau`.
pub const fn tu_vi_branch(day: u8, bureau: FiveElementsBureau) -> Branch {
    let day = day as i64;
    let number = bureau.number() as i64;
    let remainder = day % number;
    if remainder == 0 {
        return shift(Dan, day / number - 1);
    }
    let borrowed = number - remainder;
    let base = shift(Dan, (day + borrowed) / number - 1);
    if borrowed % 2 == 1 {
        shift(base, -borrowed)
    } else {
        shift(base, borrowed)
    }
}

/// The 14 Chính Tinh and their palaces when Tử Vi is at `tu_vi`, in the order of [`TuViStar`].
pub const fn chinh_tinh_branches(tu_vi: Branch) -> [(TuViStar, Branch); 14] {
    use TuViStar as S;
    let thien_phu = Branch::from_index(4 - tu_vi.index() as i64);
    [
        (S::TuVi, tu_vi),
        (S::ThienCo, shift(tu_vi, -1)),
        (S::ThaiDuong, shift(tu_vi, -3)),
        (S::VuKhuc, shift(tu_vi, -4)),
        (S::ThienDong, shift(tu_vi, -5)),
        (S::LiemTrinh, shift(tu_vi, -8)),
        (S::ThienPhu, thien_phu),
        (S::ThaiAm, shift(thien_phu, 1)),
        (S::ThamLang, shift(thien_phu, 2)),
        (S::CuMon, shift(thien_phu, 3)),
        (S::ThienTuong, shift(thien_phu, 4)),
        (S::ThienLuong, shift(thien_phu, 5)),
        (S::ThatSat, shift(thien_phu, 6)),
        (S::PhaQuan, shift(thien_phu, 10)),
    ]
}

/// Month of birth used for the stars: only a leap month can differ from `lunar.month`, and only
/// the month changes, never the day or the year.
pub const fn effective_month(lunar: LunarDate, rule: LeapMonthRule) -> u8 {
    match rule {
        LeapMonthRule::SplitAt15th if lunar.is_leap_month && lunar.day >= 16 => {
            lunar.month % 12 + 1
        }
        _ => lunar.month,
    }
}

/// Mệnh Chủ of a Mệnh palace at `menh`.
///
/// *Tân Biên* never mentions it. The table is the one the project owner approved, after *Tử Vi
/// Đẩu Số Toàn Thư*, keyed on the Earthly Branch of the Mệnh palace.
pub const fn menh_chu(menh: Branch) -> TuViStar {
    match menh {
        Ty => TuViStar::ThamLang,
        Suu | Hoi => TuViStar::CuMon,
        Dan | Tuat => TuViStar::LocTon,
        Mao | Dau => TuViStar::VanKhuc,
        Thin | Than => TuViStar::LiemTrinh,
        Ti | Mui => TuViStar::VuKhuc,
        Ngo => TuViStar::PhaQuan,
    }
}

/// Thân Chủ of a birth year with branch `year_branch`.
///
/// *Tân Biên* never mentions it. The table is the one the project owner approved, after *Tử Vi
/// Đẩu Số Toàn Thư*, keyed on the Earthly Branch of the birth year.
pub const fn than_chu(year_branch: Branch) -> TuViStar {
    match year_branch {
        Ty | Ngo => TuViStar::HoaTinh,
        Suu | Mui => TuViStar::ThienTuong,
        Dan | Than => TuViStar::ThienLuong,
        Mao | Dau => TuViStar::ThienDong,
        Thin | Tuat => TuViStar::VanXuong,
        Ti | Hoi => TuViStar::ThienCo,
    }
}

/// The two palaces of Triệt in a year with stem `year_stem`.
pub const fn triet(year_stem: Stem) -> KhongVong {
    let (first, second) = TRIET[year_stem.index() as usize % 5];
    KhongVong { first, second }
}

/// Where each star of [`TuViStar`] sits, by its position in the enum.
fn place_stars(
    year: CanChi,
    month: u8,
    day: u8,
    hour: Branch,
    forward: bool,
) -> [Branch; TuViStar::COUNT] {
    use TuViStar as S;

    let stem = year.stem.index() as usize;
    let (y, h, m, d) = (
        year.branch.index() as i64,
        hour.index() as i64,
        month as i64,
        day as i64,
    );
    let step = if forward { 1 } else { -1 };
    let menh = menh_branch(month, hour);
    let than = than_branch(month, hour);
    let bureau = bureau_of(year.stem, menh);
    let trine = year.branch.trine_group();

    let mut at = [Ty; TuViStar::COUNT];
    let mut put = |star: S, branch: Branch| at[star as usize] = branch;

    for (star, branch) in chinh_tinh_branches(tu_vi_branch(day, bureau)) {
        put(star, branch);
    }

    let loc_ton = LOC_TON[stem];
    let truong_sinh = bureau.truong_sinh_start();
    for i in 0..12 {
        let offset = i as i64;
        put(THAI_TUE_RING[i], shift(year.branch, offset));
        put(BAC_SY_RING[i], shift(loc_ton, step * offset));
        put(TRUONG_SINH_RING[i], shift(truong_sinh, step * offset));
    }
    put(S::LocTon, loc_ton);
    put(S::KinhDuong, shift(loc_ton, 1));
    put(S::DaLa, shift(loc_ton, -1));

    let ta_phu = shift(Thin, m - 1);
    let huu_bat = shift(Tuat, 1 - m);
    put(S::TaPhu, ta_phu);
    put(S::HuuBat, huu_bat);
    put(S::ThienHinh, shift(Dau, m - 1));
    put(S::ThienRieu, shift(Suu, m - 1));
    put(S::ThienY, shift(Suu, m - 1));
    put(S::ThienGiai, shift(Than, m - 1));
    put(S::DiaGiai, shift(Mui, m - 1));

    let van_xuong = shift(Tuat, -h);
    let van_khuc = shift(Thin, h);
    put(S::VanXuong, van_xuong);
    put(S::VanKhuc, van_khuc);
    put(S::DiaKhong, shift(Hoi, -h));
    put(S::DiaKiep, shift(Hoi, h));
    put(S::ThaiPhu, shift(Ngo, h));
    put(S::PhongCao, shift(Dan, h));

    put(S::TamThai, shift(ta_phu, d - 1));
    put(S::BatToa, shift(huu_bat, 1 - d));
    put(S::AnQuang, shift(van_xuong, d - 2));
    // "Lùi lại một cung" goes back against the count, which here runs counterclockwise.
    put(S::ThienQuy, shift(van_khuc, 2 - d));

    let (hoa_tinh, linh_tinh) = HOA_LINH_START[trine];
    put(S::HoaTinh, shift(hoa_tinh, step * h));
    put(S::LinhTinh, shift(linh_tinh, -step * h));

    put(S::ThienKhoi, THIEN_KHOI[stem]);
    put(S::ThienViet, THIEN_VIET[stem]);
    put(S::ThienQuan, THIEN_QUAN[stem]);
    put(S::ThienPhuc, THIEN_PHUC[stem]);
    put(S::LuuHa, LUU_HA[stem]);
    put(S::ThienTru, THIEN_TRU[stem]);
    put(S::QuocAn, shift(loc_ton, 8));
    put(S::DuongPhu, shift(loc_ton, -7));

    put(S::LongTri, shift(Thin, y));
    put(S::PhuongCac, shift(Tuat, -y));
    put(S::GiaiThan, shift(Tuat, -y));
    put(S::HongLoan, shift(Mao, -y));
    put(S::ThienHy, shift(Dau, -y));
    let (co_than, qua_tu) = match year.branch {
        Hoi | Ty | Suu => (Dan, Tuat),
        Dan | Mao | Thin => (Ti, Suu),
        Ti | Ngo | Mui => (Than, Thin),
        Than | Dau | Tuat => (Hoi, Mui),
    };
    put(S::CoThan, co_than);
    put(S::QuaTu, qua_tu);
    put(S::DaoHoa, DAO_HOA[trine]);
    put(S::ThienMa, THIEN_MA[trine]);
    put(S::KiepSat, KIEP_SAT[trine]);
    put(S::HoaCai, HOA_CAI[trine]);
    put(
        S::PhaToai,
        match year.branch {
            Ty | Ngo | Mao | Dau => Ti,
            Dan | Than | Ti | Hoi => Dau,
            Thin | Tuat | Suu | Mui => Suu,
        },
    );
    put(S::ThienKhong, shift(year.branch, 1));
    put(S::ThienKhoc, shift(Ngo, -y));
    put(S::ThienHu, shift(Ngo, y));

    put(S::ThienDuc, shift(Dau, y));
    put(S::NguyetDuc, shift(Ti, y));
    put(S::ThienTai, shift(menh, y));
    put(S::ThienTho, shift(than, y));
    put(S::ThienThuong, shift(menh, 5));
    put(S::ThienSu, shift(menh, 7));
    put(S::ThienLa, Thin);
    put(S::DiaVong, Tuat);
    put(S::LuuNienVanTinh, LUU_NIEN_VAN_TINH[stem]);
    put(S::DauQuan, shift(year.branch, h + 1 - m));

    at
}

/// The pillars and times that differ between the two ways of casting a chart.
struct ChartInput {
    effective_time: Option<SolarDateTime>,
    solar_date: SolarDate,
    lunar: LunarDate,
    hour: Branch,
    day_can_chi: CanChi,
    hour_can_chi: CanChi,
}

fn build_chart(input: ChartInput, gender: Gender, rule: LeapMonthRule) -> TuViChart {
    let lunar = input.lunar;
    let year = year_can_chi(lunar.year);
    let month = effective_month(lunar, rule);
    let polarity = TuViPolarity::of(year.stem, gender);
    let forward = polarity.is_forward();
    let step = if forward { 1 } else { -1 };

    let menh = menh_branch(month, input.hour);
    let than = than_branch(month, input.hour);
    let bureau = bureau_of(year.stem, menh);
    let number = bureau.number();
    let tuan = khong_vong(year);
    let triet = triet(year.stem);
    let at = place_stars(year, month, lunar.day, input.hour, forward);
    let transformation_of = |star: TuViStar| {
        FourTransformations::ALL
            .into_iter()
            .find(|hoa| hoa.star_for(year.stem) == star)
    };
    let tieu_han_start = TIEU_HAN_START[year.branch.trine_group()];
    let tieu_han_step = match gender {
        Gender::Male => 1,
        Gender::Female => -1,
    };

    let palaces = Branch::ALL
        .into_iter()
        .map(|branch| {
            let (main_stars, minor_stars) = TuViStar::ALL
                .into_iter()
                .filter(|star| at[*star as usize] == branch)
                .map(|star| TuViPlacedStar {
                    star,
                    brightness: star.brightness_at(branch),
                    transformation: transformation_of(star),
                })
                .partition(|placed: &TuViPlacedStar| {
                    placed.star.category() == StarCategory::ChinhTinh
                });
            let from_menh = (branch.index() as i64 - menh.index() as i64).rem_euclid(12);
            let dai_han_index = (step * from_menh).rem_euclid(12) as u8;
            let tieu_han_index = (tieu_han_step
                * (branch.index() as i64 - tieu_han_start.index() as i64))
                .rem_euclid(12);
            let stage_index = (step
                * (branch.index() as i64 - bureau.truong_sinh_start().index() as i64))
                .rem_euclid(12);
            TuViPalace {
                branch,
                stem: palace_stem(year.stem, branch),
                role: TuViPalaceRole::ALL[from_menh as usize],
                is_than: branch == than,
                main_stars,
                minor_stars,
                life_stage: LifeStage::ALL[stage_index as usize],
                dai_han_start_age: number + 10 * dai_han_index,
                tieu_han_year_branch: shift(year.branch, tieu_han_index),
                tieu_han_first_age: tieu_han_index as u8 + 1,
                has_tuan: branch == tuan.first || branch == tuan.second,
                has_triet: branch == triet.first || branch == triet.second,
            }
        })
        .collect();

    let ban_menh = year.na_yin();
    let from_menh = (than.index() as i64 - menh.index() as i64).rem_euclid(12);
    TuViChart {
        gender,
        effective_time: input.effective_time,
        solar_date: input.solar_date,
        lunar,
        effective_month: month,
        hour_branch: input.hour,
        year_can_chi: year,
        month_can_chi: month_can_chi(lunar.year, lunar.month),
        day_can_chi: input.day_can_chi,
        hour_can_chi: input.hour_can_chi,
        polarity,
        forward,
        ban_menh,
        bureau,
        menh_branch: menh,
        than_branch: than,
        than_role: TuViPalaceRole::ALL[from_menh as usize],
        menh_chu: menh_chu(menh),
        than_chu: than_chu(year.branch),
        harmony: YinYangHarmony::of(year.stem, menh),
        menh_cuc: MenhCucRelation::of(ban_menh.element(), bureau.element()),
        tuan,
        triet,
        palaces,
    }
}

/// Casts the chart of a birth at wall-clock time `birth`.
///
/// Fails for an invalid date or time, an offset or longitude out of range, or a year outside the
/// supported range. With `use_true_solar_time` and no `longitude`, a zone beyond UTC+12 has no
/// valid standard meridian and fails too.
pub fn tuvi_chart(
    birth: SolarDateTime,
    gender: Gender,
    config: &TuViConfig,
) -> Result<TuViChart, AstroError> {
    let longitude = config
        .use_true_solar_time
        .then(|| config.longitude.unwrap_or(config.tz_offset * 15.0));
    let pillars = four_pillars(
        birth,
        &FourPillarsOptions {
            tz_offset: config.tz_offset,
            longitude,
            rat_hour: config.rat_hour,
            boundary: YearMonthBoundary::LunarNewYear,
        },
    )?;
    let time = pillars.effective_time;
    let next_day = config.rat_hour == RatHourConvention::NewDayAtRatHour && time.hour >= 23;
    let solar_date = if next_day {
        SolarDate::from_jdn(time.date().to_jdn() + 1)
    } else {
        time.date()
    };
    let lunar = solar_to_lunar(
        solar_date.day,
        solar_date.month,
        solar_date.year,
        config.tz_offset,
    )?;
    Ok(build_chart(
        ChartInput {
            effective_time: Some(time),
            solar_date,
            lunar,
            hour: Branch::from_hour(time.hour),
            day_can_chi: pillars.day,
            hour_can_chi: pillars.hour,
        },
        gender,
        config.leap_month_rule,
    ))
}

/// Casts the chart of a birth on lunar date `lunar` at double-hour `hour`.
///
/// `lunar` is taken as given, so `rat_hour` and true solar time play no part. The hour pillar is
/// that of the same day.
///
/// Fails for a lunar date that does not exist, an invalid offset, or a year outside the supported
/// range.
pub fn tuvi_chart_from_lunar(
    lunar: LunarDate,
    hour: Branch,
    gender: Gender,
    config: &TuViConfig,
) -> Result<TuViChart, AstroError> {
    let solar_date = lunar_to_solar(lunar, config.tz_offset)?;
    let day = day_can_chi(solar_date.to_jdn());
    Ok(build_chart(
        ChartInput {
            effective_time: None,
            solar_date,
            lunar,
            hour,
            day_can_chi: day,
            hour_can_chi: hour_can_chi(day.stem, hour),
        },
        gender,
        config.leap_month_rule,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canchi::Stem::{At, Binh, Canh, Dinh, Giap, Ky, Mau, Nham, Quy, Tan};
    use crate::lunar_calendar::lunar_year;
    use crate::test_support::{VN, date_time};
    use std::collections::BTreeMap;

    fn lunar(year: i32, month: u8, day: u8) -> LunarDate {
        LunarDate {
            year,
            month,
            day,
            is_leap_month: false,
        }
    }

    fn from_lunar(date: LunarDate, hour: Branch, gender: Gender) -> TuViChart {
        tuvi_chart_from_lunar(date, hour, gender, &TuViConfig::default()).unwrap()
    }

    /// Any chart of year `stem` `branch`: `year_can_chi` of a year in the 1984–2043 cycle.
    fn chart_of_year(stem: Stem, branch: Branch) -> TuViChart {
        let year = (1984..2044)
            .find(|&y| {
                let can_chi = year_can_chi(y);
                can_chi.stem == stem && can_chi.branch == branch
            })
            .expect("stem and branch share their polarity");
        from_lunar(lunar(year, 1, 1), Ty, Gender::Male)
    }

    fn branch_of(chart: &TuViChart, star: TuViStar) -> Branch {
        chart.palace_of_star(star).branch
    }

    fn branch_named(name: &str) -> Branch {
        *Branch::ALL.iter().find(|b| b.name_vi() == name).unwrap()
    }

    /// Stars of a palace other than the Chính Tinh, by name, with the Tứ Hóa as separate names.
    fn other_names(palace: &TuViPalace) -> Vec<String> {
        let stars = palace.main_stars.iter().chain(&palace.minor_stars);
        let hoa = stars.filter_map(|placed| placed.transformation);
        let mut names: Vec<String> = palace
            .minor_stars
            .iter()
            .map(|placed| placed.star.name_vi().to_string())
            .chain(hoa.map(|hoa| hoa.name_vi().to_string()))
            .collect();
        names.sort();
        names
    }

    /// Chính Tinh of a palace as the spec writes them, e.g. "Tử Vi (V)".
    fn main_names(palace: &TuViPalace) -> Vec<String> {
        palace
            .main_stars
            .iter()
            .map(|placed| {
                let mark = placed.brightness.expect("Chính Tinh have a brightness");
                format!("{} ({})", placed.star.name_vi(), mark.mark())
            })
            .collect()
    }

    fn sorted(names: &[&str]) -> Vec<String> {
        let mut names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        names.sort();
        names
    }

    // ---- Tables of the book ----

    /// P. 7, Lập Cục: bureau by pair of year stems and group of Mệnh palaces.
    const LAP_CUC: [(Stem, Stem, [u8; 5]); 5] = [
        (Giap, Ky, [2, 6, 3, 5, 4]),
        (At, Canh, [6, 5, 4, 3, 2]),
        (Binh, Tan, [5, 3, 2, 4, 6]),
        (Dinh, Nham, [3, 4, 6, 2, 5]),
        (Mau, Quy, [4, 2, 5, 6, 3]),
    ];
    const LAP_CUC_PALACES: [&[Branch]; 5] = [
        &[Ty, Suu],
        &[Dan, Mao, Tuat, Hoi],
        &[Thin, Ti],
        &[Ngo, Mui],
        &[Than, Dau],
    ];

    #[test]
    fn bureau_matches_the_lap_cuc_table_on_all_120_cells() {
        let mut cells = 0;
        for (first, second, row) in LAP_CUC {
            for (palaces, number) in LAP_CUC_PALACES.iter().zip(row) {
                for &palace in *palaces {
                    for stem in [first, second] {
                        assert_eq!(bureau_of(stem, palace).number(), number);
                        cells += 1;
                    }
                }
            }
        }
        assert_eq!(cells, 120);
    }

    /// A palace and the lunar days that put Tử Vi there.
    type PalaceDays = (Branch, &'static [u8]);

    /// Pp. 7–8: days of each palace by bureau. In Kim Tứ Cục the book prints 21 at Mùi; the day
    /// is 24, since 21 also stands at Thìn and 24 stands nowhere.
    #[rustfmt::skip]
    const TU_VI_TABLE: [(FiveElementsBureau, [PalaceDays; 12]); 5] = [
        (FiveElementsBureau::ThuyNhi, [
            (Ty, &[22, 23]),
            (Suu, &[1, 24, 25]),
            (Dan, &[2, 3, 26, 27]),
            (Mao, &[4, 5, 28, 29]),
            (Thin, &[6, 7, 30]),
            (Ti, &[8, 9]),
            (Ngo, &[10, 11]),
            (Mui, &[12, 13]),
            (Than, &[14, 15]),
            (Dau, &[16, 17]),
            (Tuat, &[18, 19]),
            (Hoi, &[20, 21]),
        ]),
        (FiveElementsBureau::MocTam, [
            (Ty, &[25]),
            (Suu, &[2, 28]),
            (Dan, &[3, 5]),
            (Mao, &[6, 8]),
            (Thin, &[1, 9, 11]),
            (Ti, &[4, 12, 14]),
            (Ngo, &[7, 15, 17]),
            (Mui, &[10, 18, 20]),
            (Than, &[13, 21, 23]),
            (Dau, &[16, 24, 26]),
            (Tuat, &[19, 27, 29]),
            (Hoi, &[22, 30]),
        ]),
        (FiveElementsBureau::KimTu, [
            (Ty, &[5]),
            (Suu, &[3, 9]),
            (Dan, &[4, 7, 13]),
            (Mao, &[8, 11, 17]),
            (Thin, &[2, 12, 15, 21]),
            (Ti, &[6, 16, 19, 25]),
            (Ngo, &[10, 20, 23, 29]),
            (Mui, &[14, 24, 27]),
            (Than, &[18, 28]),
            (Dau, &[22]),
            (Tuat, &[26]),
            (Hoi, &[1, 30]),
        ]),
        (FiveElementsBureau::ThoNgu, [
            (Ty, &[7]),
            (Suu, &[4, 12]),
            (Dan, &[5, 9, 17]),
            (Mao, &[10, 14, 22]),
            (Thin, &[3, 15, 19, 27]),
            (Ti, &[8, 20, 24]),
            (Ngo, &[1, 13, 25, 29]),
            (Mui, &[6, 18, 30]),
            (Than, &[11, 23]),
            (Dau, &[16, 28]),
            (Tuat, &[21]),
            (Hoi, &[2, 26]),
        ]),
        (FiveElementsBureau::HoaLuc, [
            (Ty, &[9, 19]),
            (Suu, &[5, 15, 25]),
            (Dan, &[6, 11, 21]),
            (Mao, &[12, 17, 27]),
            (Thin, &[4, 18, 23]),
            (Ti, &[10, 24, 29]),
            (Ngo, &[2, 16, 30]),
            (Mui, &[8, 22]),
            (Than, &[14, 28]),
            (Dau, &[1, 20]),
            (Tuat, &[7, 26]),
            (Hoi, &[3, 13]),
        ]),
    ];

    #[test]
    fn tu_vi_matches_the_book_on_all_150_days() {
        let mut days = 0;
        for (bureau, row) in TU_VI_TABLE {
            for (palace, list) in row {
                for &day in list {
                    assert_eq!(tu_vi_branch(day, bureau), palace, "{bureau:?}, day {day}");
                    days += 1;
                }
            }
        }
        assert_eq!(days, 150);
    }

    #[test]
    fn tuan_and_triet_match_the_book_in_all_60_years() {
        // Pp. 16–17: Tuần by the Giáp year opening the decade, Triệt by stem.
        let tuan = [
            (Tuat, Hoi),
            (Than, Dau),
            (Ngo, Mui),
            (Thin, Ti),
            (Dan, Mao),
            (Ty, Suu),
        ];
        let triet_by_stem = [
            (Than, Dau),
            (Ngo, Mui),
            (Thin, Ti),
            (Dan, Mao),
            (Ty, Suu),
            (Than, Dau),
            (Ngo, Mui),
            (Thin, Ti),
            (Dan, Mao),
            (Ty, Suu),
        ];
        for index in 0..60 {
            let pillar = CanChi::from_cycle_index(index);
            let chart = chart_of_year(pillar.stem, pillar.branch);
            let (first, second) = tuan[index as usize / 10];
            assert_eq!((chart.tuan.first, chart.tuan.second), (first, second));
            let (first, second) = triet_by_stem[pillar.stem.index() as usize];
            assert_eq!((chart.triet.first, chart.triet.second), (first, second));
            for palace in &chart.palaces {
                let branch = palace.branch;
                assert_eq!(
                    palace.has_tuan,
                    branch == chart.tuan.first || branch == chart.tuan.second
                );
                assert_eq!(palace.has_triet, branch == first || branch == second);
            }
        }
    }

    #[test]
    fn tu_hoa_matches_the_approved_table() {
        use FourTransformations::{HoaKhoa, HoaKy, HoaLoc, HoaQuyen};
        let table = [
            (Giap, ["Liêm Trinh", "Phá Quân", "Vũ Khúc", "Thái Dương"]),
            (At, ["Thiên Cơ", "Thiên Lương", "Tử Vi", "Thái Âm"]),
            (Binh, ["Thiên Đồng", "Thiên Cơ", "Văn Xương", "Liêm Trinh"]),
            (Dinh, ["Thái Âm", "Thiên Đồng", "Thiên Cơ", "Cự Môn"]),
            (Mau, ["Tham Lang", "Thái Âm", "Hữu Bật", "Thiên Cơ"]),
            (Ky, ["Vũ Khúc", "Tham Lang", "Thiên Lương", "Văn Khúc"]),
            (Canh, ["Thái Dương", "Vũ Khúc", "Thái Âm", "Thiên Đồng"]),
            // Tân: p. 14 misprints Thiên Lương for Hóa Quyền.
            (Tan, ["Cự Môn", "Thái Dương", "Văn Khúc", "Văn Xương"]),
            // Nhâm: Hóa Khoa on Tả Phụ, not Thiên Phủ (T2).
            (Nham, ["Thiên Lương", "Tử Vi", "Tả Phụ", "Vũ Khúc"]),
            (Quy, ["Phá Quân", "Cự Môn", "Thái Âm", "Tham Lang"]),
        ];
        for (stem, row) in table {
            let got = [HoaLoc, HoaQuyen, HoaKhoa, HoaKy].map(|hoa| hoa.star_for(stem).name_vi());
            assert_eq!(got, row, "{stem:?}");
        }
    }

    #[test]
    fn each_transformation_lands_on_the_palace_of_its_star_in_every_year() {
        for index in 0..60 {
            let pillar = CanChi::from_cycle_index(index);
            let chart = chart_of_year(pillar.stem, pillar.branch);
            let mut seen = 0;
            for palace in &chart.palaces {
                for placed in palace.main_stars.iter().chain(&palace.minor_stars) {
                    if let Some(hoa) = placed.transformation {
                        assert_eq!(hoa.star_for(pillar.stem), placed.star);
                        seen += 1;
                    }
                }
            }
            assert_eq!(seen, 4);
        }
    }

    /// Pp. 33–38, as the book lists them: Miếu, Vượng, Đắc, Bình, Hãm.
    #[rustfmt::skip]
    const BRIGHTNESS_LISTS: [(TuViStar, [&[Branch]; 5]); 14] = [
        (TuViStar::TuVi, [&[Ti, Ngo, Dan, Than], &[Thin, Tuat], &[Suu, Mui], &[Hoi, Ty, Mao, Dau], &[]]),
        (TuViStar::ThienCo, [&[Thin, Tuat, Mao, Dau], &[Ti, Than], &[Ty, Ngo, Suu, Mui], &[], &[Dan, Hoi]]),
        (TuViStar::ThaiDuong, [&[Ti, Ngo], &[Dan, Mao, Thin], &[Suu, Mui], &[], &[Than, Dau, Tuat, Hoi, Ty]]),
        (TuViStar::VuKhuc, [&[Thin, Tuat, Suu, Mui], &[Dan, Than, Ty, Ngo], &[Mao, Dau], &[], &[Ti, Hoi]]),
        (TuViStar::ThienDong, [&[Dan, Than], &[Ty], &[Mao, Ti, Hoi], &[], &[Ngo, Dau, Thin, Tuat, Suu, Mui]]),
        (TuViStar::LiemTrinh, [&[Thin, Tuat], &[Ty, Ngo, Dan, Than], &[Suu, Mui], &[], &[Ti, Hoi, Mao, Dau]]),
        (TuViStar::ThienPhu, [&[Dan, Than, Ty, Ngo], &[Thin, Tuat], &[Ti, Hoi, Mui], &[Mao, Dau, Suu], &[]]),
        (TuViStar::ThaiAm, [&[Dau, Tuat, Hoi], &[Than, Ty], &[Suu, Mui], &[], &[Dan, Mao, Thin, Ti, Ngo]]),
        (TuViStar::ThamLang, [&[Suu, Mui], &[Thin, Tuat], &[Dan, Than], &[], &[Ti, Hoi, Ty, Ngo, Mao, Dau]]),
        (TuViStar::CuMon, [&[Mao, Dau], &[Ty, Ngo, Dan], &[Than, Hoi], &[], &[Thin, Tuat, Suu, Mui, Ti]]),
        (TuViStar::ThienTuong, [&[Dan, Than], &[Thin, Tuat, Ty, Ngo], &[Suu, Mui, Ti, Hoi], &[], &[Mao, Dau]]),
        (TuViStar::ThienLuong, [&[Ngo, Thin, Tuat], &[Ty, Mao, Dan, Than], &[Suu, Mui], &[], &[Dau, Ti, Hoi]]),
        (TuViStar::ThatSat, [&[Dan, Than, Ty, Ngo], &[Ti, Hoi], &[Suu, Mui], &[], &[Mao, Dau, Thin, Tuat]]),
        (TuViStar::PhaQuan, [&[Ty, Ngo], &[Suu, Mui], &[Thin, Tuat], &[], &[Mao, Dau, Dan, Than, Ti, Hoi]]),
    ];

    #[test]
    fn brightness_matches_the_lists_of_the_book() {
        use StarBrightness::{Binh as B, Dac, Ham, Mieu, Vuong};
        for (star, lists) in BRIGHTNESS_LISTS {
            let mut seen = 0;
            for (level, list) in [Mieu, Vuong, Dac, B, Ham].into_iter().zip(lists) {
                for &branch in list {
                    assert_eq!(
                        star.brightness_at(branch),
                        Some(level),
                        "{star:?} {branch:?}"
                    );
                    seen += 1;
                }
            }
            assert_eq!(seen, 12, "{star:?}");
        }
    }

    #[test]
    fn minor_stars_follow_pages_39_to_43() {
        use StarBrightness::Dac;
        use TuViStar as S;
        let dac_of = |star: TuViStar| -> Vec<Branch> {
            Branch::ALL
                .into_iter()
                .filter(|&branch| star.brightness_at(branch) == Some(Dac))
                .collect()
        };
        for (stars, dac) in [
            (&[S::KinhDuong, S::DaLa][..], &[Suu, Thin, Mui, Tuat][..]),
            (&[S::HoaTinh, S::LinhTinh], &[Dan, Mao, Thin, Ti, Ngo]),
            (&[S::VanXuong, S::VanKhuc], &[Suu, Thin, Ti, Mui, Tuat, Hoi]),
            (&[S::DiaKhong, S::DiaKiep], &[Dan, Ti, Than, Hoi]),
            (
                &[S::DaiHao, S::TieuHao, S::TangMon, S::BachHo, S::ThienHinh],
                &[Dan, Mao, Than, Dau],
            ),
            (&[S::ThienKhoc, S::ThienHu], &[Ty, Suu, Mao, Ngo, Mui, Dau]),
            (&[S::ThienMa], &[Dan, Ti]),
            (&[S::ThienRieu], &[Dan, Mao, Dau, Tuat]),
        ] {
            for &star in stars {
                assert_eq!(dac_of(star), dac, "{star:?}");
            }
        }
    }

    #[test]
    fn only_stars_the_book_calls_ham_dia_are_ham_outside_their_list() {
        use TuViStar as S;
        assert_eq!(S::KinhDuong.brightness_at(Ty), Some(StarBrightness::Ham));
        assert_eq!(S::VanKhuc.brightness_at(Dan), Some(StarBrightness::Ham));
        assert_eq!(S::ThienKhoc.brightness_at(Thin), Some(StarBrightness::Ham));
        for star in [S::DaiHao, S::TieuHao, S::ThienMa, S::ThienHinh] {
            assert_eq!(star.brightness_at(Hoi), None, "{star:?}");
        }
    }

    #[test]
    fn hoa_ky_alone_has_a_brightness() {
        use FourTransformations::{HoaKhoa, HoaKy, HoaLoc, HoaQuyen};
        assert_eq!(HoaKy.brightness_at(Suu), Some(StarBrightness::Dac));
        assert_eq!(HoaKy.brightness_at(Ty), Some(StarBrightness::Ham));
        for hoa in [HoaLoc, HoaQuyen, HoaKhoa] {
            assert!(
                Branch::ALL
                    .iter()
                    .all(|&branch| hoa.brightness_at(branch).is_none())
            );
        }
    }

    #[test]
    fn chart_stars_carry_the_brightness_of_their_palace() {
        let chart = from_lunar(lunar(2024, 3, 12), Branch::Ngo, Gender::Male);
        for palace in &chart.palaces {
            for placed in palace.main_stars.iter().chain(&palace.minor_stars) {
                assert_eq!(placed.brightness, placed.star.brightness_at(palace.branch));
            }
        }
    }

    #[test]
    fn only_the_listed_stars_have_a_brightness() {
        let main: Vec<TuViStar> = TuViStar::ALL
            .into_iter()
            .filter(|star| star.category() == StarCategory::ChinhTinh)
            .collect();
        assert_eq!(main.len(), 14);
        assert_eq!(main, BRIGHTNESS_LISTS.map(|(star, _)| star));
        let mut listed = 0;
        for star in TuViStar::ALL.into_iter().skip(14) {
            assert_eq!(star.category(), StarCategory::PhuTinh);
            if minor_brightness(star).is_some() {
                listed += 1;
            } else {
                assert!(Branch::ALL.iter().all(|&b| star.brightness_at(b).is_none()));
            }
        }
        assert_eq!(listed, 17);
    }

    #[test]
    fn natures_follow_section_4_9_of_the_spec() {
        let count = |nature| {
            let stars = TuViStar::ALL.into_iter();
            stars.filter(|star| star.nature() == nature).count()
        };
        assert_eq!(
            count(StarNature::Cat) + count(StarNature::Trung) + count(StarNature::Hung),
            105
        );
        assert_eq!(
            (
                count(StarNature::Cat),
                count(StarNature::Trung),
                count(StarNature::Hung)
            ),
            (48, 25, 32)
        );
        // Labels in the book: Sát tinh, Bại tinh, Hung tinh; and Thất Sát, by the owner's decision.
        for star in [
            TuViStar::KinhDuong,
            TuViStar::DaLa,
            TuViStar::HoaTinh,
            TuViStar::LinhTinh,
            TuViStar::DiaKhong,
            TuViStar::DiaKiep,
            TuViStar::DaiHao,
            TuViStar::TieuHao,
            TuViStar::TangMon,
            TuViStar::BachHo,
            TuViStar::ThienKhoc,
            TuViStar::ThienHu,
            TuViStar::ThamLang,
            TuViStar::ThatSat,
            TuViStar::PhaQuan,
        ] {
            assert_eq!(star.nature(), StarNature::Hung, "{star:?}");
        }
        // Tù tinh and Ám tinh.
        for star in [TuViStar::LiemTrinh, TuViStar::CuMon, TuViStar::ThienLuong] {
            assert_eq!(star.nature(), StarNature::Trung, "{star:?}");
        }
        assert_eq!(TuViStar::TuVi.nature(), StarNature::Cat);
        assert_eq!(FourTransformations::HoaLoc.nature(), StarNature::Cat);
        assert_eq!(FourTransformations::HoaKhoa.nature(), StarNature::Cat);
        assert_eq!(FourTransformations::HoaQuyen.nature(), StarNature::Trung);
        assert_eq!(FourTransformations::HoaKy.nature(), StarNature::Hung);
    }

    #[test]
    fn brightness_levels_are_complete_and_in_order() {
        for (index, level) in StarBrightness::ALL.into_iter().enumerate() {
            assert_eq!(level as usize, index);
        }
        let mut names: Vec<&str> = StarBrightness::ALL.iter().map(|b| b.name_vi()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), StarBrightness::ALL.len());
    }

    #[test]
    fn chinh_tinh_branches_list_the_fourteen_stars_in_enum_order() {
        for tu_vi in Branch::ALL {
            let stars = chinh_tinh_branches(tu_vi);
            for (index, (star, _)) in stars.into_iter().enumerate() {
                assert_eq!(star, TuViStar::ALL[index]);
            }
            assert_eq!(stars[0], (TuViStar::TuVi, tu_vi));
        }
    }

    #[test]
    fn chinh_tinh_branches_agree_with_a_full_chart() {
        for day in 1..=29 {
            let chart = from_lunar(lunar(2024, 3, day), Branch::Ngo, Gender::Male);
            let tu_vi = chart.palace_of_star(TuViStar::TuVi).branch;
            for (star, branch) in chinh_tinh_branches(tu_vi) {
                assert_eq!(
                    chart.palace_of_star(star).branch,
                    branch,
                    "{star:?}, day {day}"
                );
            }
        }
    }

    #[test]
    fn tu_vi_and_thien_phu_share_a_palace_only_at_dan_and_than() {
        let together: Vec<Branch> = Branch::ALL
            .into_iter()
            .filter(|&tu_vi| chinh_tinh_branches(tu_vi)[6].1 == tu_vi)
            .collect();
        assert_eq!(together, [Branch::Dan, Branch::Than]);
    }

    #[test]
    fn star_enum_is_complete_and_in_order() {
        for (index, star) in TuViStar::ALL.into_iter().enumerate() {
            assert_eq!(star as usize, index);
        }
        let mut names: Vec<&str> = TuViStar::ALL.iter().map(|star| star.name_vi()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), TuViStar::COUNT);
    }

    #[test]
    fn examples_printed_in_the_book() {
        let at = |stem, branch, star| branch_of(&chart_of_year(stem, branch), star);
        assert_eq!(at(Ky, Hoi, TuViStar::LocTon), Ngo); // p. 9
        assert_eq!(at(Quy, Suu, TuViStar::KinhDuong), Suu); // p. 10
        assert_eq!(at(Quy, Suu, TuViStar::DaLa), Hoi);
        assert_eq!(at(At, Mui, TuViStar::ThienKhoi), Ty); // p. 11
        assert_eq!(at(At, Mui, TuViStar::ThienViet), Than);
        assert_eq!(at(Binh, Ty, TuViStar::ThienQuan), Ti); // p. 14
        assert_eq!(at(Binh, Ty, TuViStar::ThienPhuc), Ty);
        assert_eq!(at(At, Hoi, TuViStar::CoThan), Dan);
        assert_eq!(at(At, Hoi, TuViStar::QuaTu), Tuat);
        assert_eq!(at(At, Dau, TuViStar::DaoHoa), Ngo); // p. 15
        assert_eq!(at(Giap, Ty, TuViStar::ThienMa), Dan);
        assert_eq!(at(At, Mui, TuViStar::KiepSat), Than);
        assert_eq!(at(Giap, Tuat, TuViStar::PhaToai), Suu);
        assert_eq!(at(Giap, Ngo, TuViStar::HoaCai), Tuat);
        assert_eq!(at(Dinh, Ti, TuViStar::LuuHa), Thin); // p. 16
        assert_eq!(at(Ky, Hoi, TuViStar::ThienTru), Than);
        assert_eq!(at(Binh, Ngo, TuViStar::LuuNienVanTinh), Than);
        assert_eq!(at(At, Mui, TuViStar::ThienKhong), Than);
        assert_eq!(at(Giap, Tuat, TuViStar::HongLoan), Ti); // p. 12
        assert_eq!(at(Giap, Tuat, TuViStar::ThienHy), Hoi);
        assert_eq!(FiveElementsBureau::MocTam.truong_sinh_start(), Hoi); // p. 10
        let tuan = chart_of_year(Binh, Dan).tuan; // p. 17
        assert_eq!((tuan.first, tuan.second), (Tuat, Hoi));
    }

    #[test]
    fn age_examples_printed_in_the_book() {
        // P. 22: a boy born in a Tý year starts at Tuất and runs clockwise.
        let boy = from_lunar(lunar(1984, 1, 1), Ty, Gender::Male);
        assert_eq!(boy.tieu_han_palace(1).unwrap().branch, Tuat);
        assert_eq!(boy.tieu_han_palace(2).unwrap().branch, Hoi);
        assert_eq!(boy.palace(Tuat).tieu_han_year_branch, Ty);
        assert_eq!(boy.palace(Hoi).tieu_han_year_branch, Suu);
        assert_eq!(boy.palace(Ty).tieu_han_year_branch, Dan);
        // P. 23: a girl born in a Ngọ year starts at Thìn and runs counterclockwise.
        let girl = from_lunar(lunar(1990, 1, 1), Ty, Gender::Female);
        assert_eq!(girl.tieu_han_palace(1).unwrap().branch, Thin);
        assert_eq!(girl.palace(Thin).tieu_han_year_branch, Ngo);
        assert_eq!(girl.palace(Mao).tieu_han_year_branch, Mui);
        assert_eq!(girl.palace(Dan).tieu_han_year_branch, Than);
    }

    #[test]
    fn properties_stated_in_the_book_hold_on_every_input() {
        use TuViPalaceRole::{Menh, PhuThe, PhucDuc, QuanLoc, TaiBach, ThienDi};
        let allowed = [Menh, PhucDuc, QuanLoc, ThienDi, TaiBach, PhuThe];
        for month in 1..=12 {
            for hour in Branch::ALL {
                let from_menh = (than_branch(month, hour).index() as i64
                    - menh_branch(month, hour).index() as i64)
                    .rem_euclid(12);
                assert!(allowed.contains(&TuViPalaceRole::ALL[from_menh as usize]));
            }
        }
        for index in 0..60 {
            let pillar = CanChi::from_cycle_index(index);
            for (month, day, hour) in [(1, 1, Ty), (5, 16, Ngo), (12, 30, Hoi), (7, 7, Mao)] {
                let at = place_stars(pillar, month, day, hour, index % 2 == 0);
                let get = |star: TuViStar| at[star as usize];
                assert_eq!(get(TuViStar::ThienHu), get(TuViStar::TuePha));
                assert_eq!(get(TuViStar::ThaiPhu), shift(get(TuViStar::VanKhuc), 2));
                assert_eq!(get(TuViStar::PhongCao), shift(get(TuViStar::VanKhuc), -2));
                assert_eq!(get(TuViStar::ThienY), get(TuViStar::ThienRieu));
                assert_eq!(get(TuViStar::GiaiThan), get(TuViStar::PhuongCac));
                assert_eq!(get(TuViStar::BacSy), get(TuViStar::LocTon));
                assert_eq!(get(TuViStar::ThienHy), shift(get(TuViStar::HongLoan), 6));
                assert_eq!(get(TuViStar::ThienKhong), get(TuViStar::ThieuDuong));
                let sum = get(TuViStar::AnQuang).index() + get(TuViStar::ThienQuy).index();
                assert_eq!(sum % 12, 2);
            }
        }
    }

    #[test]
    fn palace_stems_follow_ngu_ho_don() {
        assert_eq!(palace_stem(Giap, Dan), Binh);
        assert_eq!(palace_stem(Giap, Hoi), At);
        assert_eq!(palace_stem(Giap, Ty), Binh);
        assert_eq!(palace_stem(Giap, Suu), Dinh);
        assert_eq!(palace_stem(Mau, Dan), Giap);
        assert_eq!(palace_stem(Quy, Dan), Giap);
        for stem in Stem::ALL {
            for branch in Branch::ALL {
                assert!(CanChi::new(palace_stem(stem, branch), branch).is_some());
            }
        }
        for year in 2020..2030 {
            assert_eq!(
                palace_stem(year_can_chi(year).stem, Dan),
                month_can_chi(year, 1).stem
            );
        }
    }

    /// The Vietnamese names of the palaces of `stars` on `chart`.
    fn palaces_of<const N: usize>(chart: &TuViChart, stars: [TuViStar; N]) -> [&'static str; N] {
        stars.map(|star| branch_of(chart, star).name_vi())
    }

    #[rustfmt::skip]
    #[test]
    fn stars_by_month_match_the_table_of_section_4_1() {
        let stars = [TuViStar::TaPhu, TuViStar::HuuBat, TuViStar::ThienHinh, TuViStar::ThienRieu, TuViStar::ThienGiai, TuViStar::DiaGiai];
        let rows = [
            ["Thìn", "Tuất", "Dậu", "Sửu", "Thân", "Mùi"],
            ["Tỵ", "Dậu", "Tuất", "Dần", "Dậu", "Thân"],
            ["Ngọ", "Thân", "Hợi", "Mão", "Tuất", "Dậu"],
            ["Mùi", "Mùi", "Tý", "Thìn", "Hợi", "Tuất"],
            ["Thân", "Ngọ", "Sửu", "Tỵ", "Tý", "Hợi"],
            ["Dậu", "Tỵ", "Dần", "Ngọ", "Sửu", "Tý"],
            ["Tuất", "Thìn", "Mão", "Mùi", "Dần", "Sửu"],
            ["Hợi", "Mão", "Thìn", "Thân", "Mão", "Dần"],
            ["Tý", "Dần", "Tỵ", "Dậu", "Thìn", "Mão"],
            ["Sửu", "Sửu", "Ngọ", "Tuất", "Tỵ", "Thìn"],
            ["Dần", "Tý", "Mùi", "Hợi", "Ngọ", "Tỵ"],
            ["Mão", "Hợi", "Thân", "Tý", "Mùi", "Ngọ"],
        ];
        for (index, row) in rows.into_iter().enumerate() {
            let chart = from_lunar(lunar(2024, index as u8 + 1, 1), Ty, Gender::Male);
            assert_eq!(palaces_of(&chart, stars), row, "month {}", index + 1);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn stars_by_hour_match_the_table_of_section_4_2() {
        let stars = [TuViStar::VanXuong, TuViStar::VanKhuc, TuViStar::DiaKhong, TuViStar::DiaKiep, TuViStar::ThaiPhu, TuViStar::PhongCao];
        let rows = [
            ["Tuất", "Thìn", "Hợi", "Hợi", "Ngọ", "Dần"],
            ["Dậu", "Tỵ", "Tuất", "Tý", "Mùi", "Mão"],
            ["Thân", "Ngọ", "Dậu", "Sửu", "Thân", "Thìn"],
            ["Mùi", "Mùi", "Thân", "Dần", "Dậu", "Tỵ"],
            ["Ngọ", "Thân", "Mùi", "Mão", "Tuất", "Ngọ"],
            ["Tỵ", "Dậu", "Ngọ", "Thìn", "Hợi", "Mùi"],
            ["Thìn", "Tuất", "Tỵ", "Tỵ", "Tý", "Thân"],
            ["Mão", "Hợi", "Thìn", "Ngọ", "Sửu", "Dậu"],
            ["Dần", "Tý", "Mão", "Mùi", "Dần", "Tuất"],
            ["Sửu", "Sửu", "Dần", "Thân", "Mão", "Hợi"],
            ["Tý", "Dần", "Sửu", "Dậu", "Thìn", "Tý"],
            ["Hợi", "Mão", "Tý", "Tuất", "Tỵ", "Sửu"],
        ];
        for (hour, row) in Branch::ALL.into_iter().zip(rows) {
            let chart = from_lunar(lunar(2024, 1, 1), hour, Gender::Male);
            assert_eq!(palaces_of(&chart, stars), row, "{hour:?}");
        }
    }

    #[rustfmt::skip]
    #[test]
    fn stars_by_year_stem_match_the_tables_of_sections_3_2_and_4_5() {
        let loc_ton = [TuViStar::LocTon, TuViStar::KinhDuong, TuViStar::DaLa];
        let by_stem = [TuViStar::ThienKhoi, TuViStar::ThienViet, TuViStar::ThienQuan, TuViStar::ThienPhuc, TuViStar::LuuHa, TuViStar::ThienTru, TuViStar::QuocAn, TuViStar::DuongPhu];
        let loc_ton_rows = [
            ["Dần", "Mão", "Sửu"],
            ["Mão", "Thìn", "Dần"],
            ["Tỵ", "Ngọ", "Thìn"],
            ["Ngọ", "Mùi", "Tỵ"],
            ["Tỵ", "Ngọ", "Thìn"],
            ["Ngọ", "Mùi", "Tỵ"],
            ["Thân", "Dậu", "Mùi"],
            ["Dậu", "Tuất", "Thân"],
            ["Hợi", "Tý", "Tuất"],
            ["Tý", "Sửu", "Hợi"],
        ];
        let by_stem_rows = [
            ["Sửu", "Mùi", "Mùi", "Dậu", "Dậu", "Tỵ", "Tuất", "Mùi"],
            ["Tý", "Thân", "Thìn", "Thân", "Tuất", "Ngọ", "Hợi", "Thân"],
            ["Hợi", "Dậu", "Tỵ", "Tý", "Mùi", "Tý", "Sửu", "Tuất"],
            ["Hợi", "Dậu", "Dần", "Hợi", "Thìn", "Tỵ", "Dần", "Hợi"],
            ["Sửu", "Mùi", "Mão", "Mão", "Tỵ", "Ngọ", "Sửu", "Tuất"],
            ["Tý", "Thân", "Dậu", "Dần", "Ngọ", "Thân", "Dần", "Hợi"],
            ["Ngọ", "Dần", "Hợi", "Ngọ", "Thân", "Dần", "Thìn", "Sửu"],
            ["Ngọ", "Dần", "Dậu", "Tỵ", "Mão", "Ngọ", "Tỵ", "Dần"],
            ["Mão", "Tỵ", "Tuất", "Ngọ", "Hợi", "Dậu", "Mùi", "Thìn"],
            ["Mão", "Tỵ", "Ngọ", "Tỵ", "Dần", "Tuất", "Thân", "Tỵ"],
        ];
        let van_tinh = ["Tỵ", "Ngọ", "Thân", "Dậu", "Thân", "Dậu", "Hợi", "Tý", "Dần", "Mão"];
        for stem in Stem::ALL {
            let index = stem.index() as usize;
            let chart = chart_of_year(stem, Branch::ALL[index % 2]);
            assert_eq!(palaces_of(&chart, loc_ton), loc_ton_rows[index], "{stem:?}");
            assert_eq!(palaces_of(&chart, by_stem), by_stem_rows[index], "{stem:?}");
            assert_eq!(branch_of(&chart, TuViStar::LuuNienVanTinh).name_vi(), van_tinh[index]);
            assert_eq!(branch_of(&chart, TuViStar::BacSy), branch_of(&chart, TuViStar::LocTon));
        }
    }

    #[rustfmt::skip]
    #[test]
    fn stars_by_year_branch_match_the_table_of_section_4_6() {
        let stars = [TuViStar::LongTri, TuViStar::PhuongCac, TuViStar::HongLoan, TuViStar::ThienHy, TuViStar::CoThan, TuViStar::QuaTu, TuViStar::DaoHoa, TuViStar::ThienMa, TuViStar::KiepSat, TuViStar::HoaCai, TuViStar::PhaToai, TuViStar::ThienKhong, TuViStar::ThienKhoc, TuViStar::ThienHu];
        let rows = [
            ["Thìn", "Tuất", "Mão", "Dậu", "Dần", "Tuất", "Dậu", "Dần", "Tỵ", "Thìn", "Tỵ", "Sửu", "Ngọ", "Ngọ"],
            ["Tỵ", "Dậu", "Dần", "Thân", "Dần", "Tuất", "Ngọ", "Hợi", "Dần", "Sửu", "Sửu", "Dần", "Tỵ", "Mùi"],
            ["Ngọ", "Thân", "Sửu", "Mùi", "Tỵ", "Sửu", "Mão", "Thân", "Hợi", "Tuất", "Dậu", "Mão", "Thìn", "Thân"],
            ["Mùi", "Mùi", "Tý", "Ngọ", "Tỵ", "Sửu", "Tý", "Tỵ", "Thân", "Mùi", "Tỵ", "Thìn", "Mão", "Dậu"],
            ["Thân", "Ngọ", "Hợi", "Tỵ", "Tỵ", "Sửu", "Dậu", "Dần", "Tỵ", "Thìn", "Sửu", "Tỵ", "Dần", "Tuất"],
            ["Dậu", "Tỵ", "Tuất", "Thìn", "Thân", "Thìn", "Ngọ", "Hợi", "Dần", "Sửu", "Dậu", "Ngọ", "Sửu", "Hợi"],
            ["Tuất", "Thìn", "Dậu", "Mão", "Thân", "Thìn", "Mão", "Thân", "Hợi", "Tuất", "Tỵ", "Mùi", "Tý", "Tý"],
            ["Hợi", "Mão", "Thân", "Dần", "Thân", "Thìn", "Tý", "Tỵ", "Thân", "Mùi", "Sửu", "Thân", "Hợi", "Sửu"],
            ["Tý", "Dần", "Mùi", "Sửu", "Hợi", "Mùi", "Dậu", "Dần", "Tỵ", "Thìn", "Dậu", "Dậu", "Tuất", "Dần"],
            ["Sửu", "Sửu", "Ngọ", "Tý", "Hợi", "Mùi", "Ngọ", "Hợi", "Dần", "Sửu", "Tỵ", "Tuất", "Dậu", "Mão"],
            ["Dần", "Tý", "Tỵ", "Hợi", "Hợi", "Mùi", "Mão", "Thân", "Hợi", "Tuất", "Sửu", "Hợi", "Thân", "Thìn"],
            ["Mão", "Hợi", "Thìn", "Tuất", "Dần", "Tuất", "Tý", "Tỵ", "Thân", "Mùi", "Dậu", "Tý", "Mùi", "Tỵ"],
        ];
        for branch in Branch::ALL {
            let index = branch.index() as usize;
            let chart = chart_of_year(Stem::ALL[index % 2], branch);
            assert_eq!(palaces_of(&chart, stars), rows[index], "{branch:?}");
            assert_eq!(branch_of(&chart, TuViStar::GiaiThan), branch_of(&chart, TuViStar::PhuongCac));
        }
    }

    #[test]
    fn thai_tue_and_truong_sinh_rings_keep_their_order() {
        let chart = from_lunar(lunar(1913, 9, 23), Dan, Gender::Male);
        for (index, star) in THAI_TUE_RING.into_iter().enumerate() {
            assert_eq!(branch_of(&chart, star), shift(Suu, index as i64));
        }
        // Âm Nam: both other rings run counterclockwise, from Tý and from Hợi.
        for (index, star) in BAC_SY_RING.into_iter().enumerate() {
            assert_eq!(branch_of(&chart, star), shift(Ty, -(index as i64)));
        }
        for (index, stage) in LifeStage::ALL.into_iter().enumerate() {
            let branch = shift(Hoi, -(index as i64));
            assert_eq!(branch_of(&chart, stage.into()), branch);
            assert_eq!(chart.palace(branch).life_stage, stage);
        }
    }

    #[test]
    fn hour_pillar_of_a_lunar_chart_is_that_of_the_same_day() {
        let chart = from_lunar(lunar(2024, 1, 1), Ngo, Gender::Male);
        assert_eq!(chart.day_can_chi, day_can_chi(chart.solar_date.to_jdn()));
        assert_eq!(chart.day_can_chi.stem, Stem::Giap);
        assert_eq!(chart.hour_can_chi.stem, Stem::Canh);
        let chart = from_lunar(lunar(2024, 1, 2), Ngo, Gender::Male);
        assert_eq!(chart.day_can_chi.stem, Stem::At);
        assert_eq!(chart.hour_can_chi.stem, Stem::Nham);
    }

    #[test]
    fn missing_longitude_means_the_standard_meridian_of_the_zone() {
        let birth = date_time(2024, 11, 3, 22, 55);
        let china = TuViConfig {
            tz_offset: 8.0,
            use_true_solar_time: true,
            ..TuViConfig::default()
        };
        let at_meridian = TuViConfig {
            longitude: Some(120.0),
            ..china
        };
        assert_eq!(
            tuvi_chart(birth, Gender::Male, &china),
            tuvi_chart(birth, Gender::Male, &at_meridian)
        );
    }

    #[test]
    fn than_flag_marks_the_than_palace_only() {
        let chart = from_lunar(lunar(1891, 6, 6), Mao, Gender::Male);
        let marked: Vec<Branch> = chart
            .palaces
            .iter()
            .filter(|palace| palace.is_than)
            .map(|palace| palace.branch)
            .collect();
        assert_eq!(marked, [Tuat]);
    }

    // ---- Spec vectors, section 6.2 ----

    /// Branch, Can Chi of the palace, role, Đại Hạn, Chính Tinh and other stars, as in the spec.
    type VectorRow = (
        &'static str,
        &'static str,
        &'static str,
        u8,
        &'static [&'static str],
        &'static [&'static str],
    );

    #[rustfmt::skip]
    const V1: [VectorRow; 12] = [
        ("Tý", "Canh Tý", "Tài Bạch", 42, &["Vũ Khúc (V)", "Thiên Phủ (M)"], &["Phúc Đức", "Đại Hao", "Mộ", "Địa Giải", "Bát Tọa", "Hồng Loan", "Đào Hoa", "Thiên Đức", "Lưu Niên Văn Tinh"]),
        ("Sửu", "Tân Sửu", "Tử Tức", 32, &["Thái Dương (Đ)", "Thái Âm (Đ)"], &["Điếu Khách", "Bệnh Phù", "Tử", "Thiên Giải", "Linh Tinh", "Quả Tú", "Hóa Quyền", "Thiên Thọ", "Đẩu Quân"]),
        ("Dần", "Canh Dần", "Phu Thê", 22, &["Tham Lang (Đ)"], &["Trực Phù", "Hỷ Thần", "Bệnh", "Thiên Hình", "Địa Kiếp", "Tam Thai", "Thiên Việt", "Đường Phù"]),
        ("Mão", "Tân Mão", "Huynh Đệ", 12, &["Thiên Cơ (M)", "Cự Môn (M)"], &["Thái Tuế", "Phi Liêm", "Suy", "Thiên Quý", "Lưu Hà", "Thiên Khốc", "Hóa Lộc"]),
        ("Thìn", "Nhâm Thìn", "Mệnh", 2, &["Tử Vi (V)", "Thiên Tướng (V)"], &["Thiếu Dương", "Tấu Thư", "Đế Vượng", "Thiên Không", "Thiên La"]),
        ("Tỵ", "Quý Tỵ", "Phụ Mẫu", 112, &["Thiên Lương (H)"], &["Tang Môn", "Tướng Quân", "Lâm Quan", "Hữu Bật", "Phong Cáo", "Thiên Phúc", "Quốc Ấn", "Cô Thần", "Thiên Mã", "Phá Toái"]),
        ("Ngọ", "Giáp Ngọ", "Phúc Đức", 102, &["Thất Sát (M)"], &["Thiếu Âm", "Tiểu Hao", "Quan Đới", "Thiên Riêu", "Thiên Y", "Hỏa Tinh", "Thiên Khôi", "Thiên Trù", "Thiên Hỷ"]),
        ("Mùi", "Ất Mùi", "Điền Trạch", 92, &[], &["Quan Phù", "Thanh Long", "Mộc Dục", "Văn Xương", "Văn Khúc", "Long Trì", "Phượng Các", "Giải Thần", "Hoa Cái", "Hóa Khoa", "Hóa Kỵ", "Thiên Tài"]),
        ("Thân", "Bính Thân", "Quan Lộc", 82, &["Liêm Trinh (V)"], &["Tử Phù", "Đà La", "Lực Sỹ", "Trường Sinh", "Địa Không", "Kiếp Sát", "Nguyệt Đức"]),
        ("Dậu", "Đinh Dậu", "Nô Bộc", 72, &[], &["Tuế Phá", "Lộc Tồn", "Bác Sỹ", "Dưỡng", "Tả Phụ", "Thai Phụ", "Thiên Quan", "Thiên Hư", "Thiên Thương"]),
        ("Tuất", "Mậu Tuất", "Thiên Di", 62, &["Phá Quân (Đ)"], &["Long Đức", "Kình Dương", "Quan Phủ", "Thai", "Địa Võng"]),
        ("Hợi", "Kỷ Hợi", "Tật Ách", 52, &["Thiên Đồng (Đ)"], &["Bạch Hổ", "Phục Binh", "Tuyệt", "Ân Quang", "Thiên Sứ"]),
    ];

    #[rustfmt::skip]
    const V2: [VectorRow; 12] = [
        ("Tý", "Bính Tý", "Tài Bạch", 43, &["Cự Môn (V)"], &["Bạch Hổ", "Thanh Long", "Dưỡng", "Tả Phụ", "Thai Phụ", "Ân Quang"]),
        ("Sửu", "Đinh Sửu", "Tử Tức", 33, &["Thiên Tướng (Đ)"], &["Phúc Đức", "Đà La", "Lực Sỹ", "Thai", "Thiên Khôi", "Quả Tú", "Phá Toái", "Thiên Đức"]),
        ("Dần", "Bính Dần", "Phu Thê", 23, &["Thiên Đồng (M)", "Thiên Lương (V)"], &["Điếu Khách", "Lộc Tồn", "Bác Sỹ", "Tuyệt", "Hữu Bật", "Thiên Quý", "Thiên Mã", "Thiên Khốc", "Đẩu Quân"]),
        ("Mão", "Đinh Mão", "Huynh Đệ", 13, &["Vũ Khúc (Đ)", "Thất Sát (H)"], &["Trực Phù", "Kình Dương", "Quan Phủ", "Mộ", "Địa Giải", "Hóa Khoa"]),
        ("Thìn", "Mậu Thìn", "Mệnh", 3, &["Thái Dương (V)"], &["Thái Tuế", "Phục Binh", "Tử", "Thiên Giải", "Văn Xương", "Linh Tinh", "Hoa Cái", "Hóa Kỵ", "Thiên La"]),
        ("Tỵ", "Kỷ Tỵ", "Phụ Mẫu", 113, &[], &["Thiếu Dương", "Đại Hao", "Bệnh", "Thiên Hình", "Địa Không", "Địa Kiếp", "Bát Tọa", "Thiên Trù", "Thiên Hỷ", "Cô Thần", "Kiếp Sát", "Thiên Không", "Lưu Niên Văn Tinh"]),
        ("Ngọ", "Canh Ngọ", "Phúc Đức", 103, &["Thiên Cơ (Đ)"], &["Tang Môn", "Bệnh Phù", "Suy", "Phượng Các", "Giải Thần"]),
        ("Mùi", "Tân Mùi", "Điền Trạch", 93, &["Tử Vi (Đ)", "Phá Quân (V)"], &["Thiếu Âm", "Hỷ Thần", "Đế Vượng", "Thiên Việt", "Thiên Quan", "Đường Phù", "Hóa Quyền"]),
        ("Thân", "Nhâm Thân", "Quan Lộc", 83, &[], &["Quan Phù", "Phi Liêm", "Lâm Quan", "Phong Cáo", "Hỏa Tinh", "Long Trì", "Thiên Tài", "Thiên Thọ"]),
        ("Dậu", "Quý Dậu", "Nô Bộc", 73, &["Thiên Phủ (B)"], &["Tử Phù", "Tấu Thư", "Quan Đới", "Thiên Riêu", "Thiên Y", "Tam Thai", "Thiên Phúc", "Lưu Hà", "Đào Hoa", "Nguyệt Đức", "Thiên Thương"]),
        ("Tuất", "Giáp Tuất", "Thiên Di", 63, &["Thái Âm (M)"], &["Tuế Phá", "Tướng Quân", "Mộc Dục", "Văn Khúc", "Quốc Ấn", "Thiên Hư", "Địa Võng"]),
        ("Hợi", "Ất Hợi", "Tật Ách", 53, &["Liêm Trinh (H)", "Tham Lang (H)"], &["Long Đức", "Tiểu Hao", "Trường Sinh", "Hồng Loan", "Hóa Lộc", "Thiên Sứ"]),
    ];

    #[rustfmt::skip]
    const V3: [VectorRow; 12] = [
        ("Tý", "Mậu Tý", "Huynh Đệ", 116, &["Tử Vi (B)"], &["Thái Tuế", "Tướng Quân", "Thai", "Thiên Giải", "Bát Tọa"]),
        ("Sửu", "Kỷ Sửu", "Mệnh", 6, &[], &["Thiếu Dương", "Tấu Thư", "Dưỡng", "Thiên Hình", "Đường Phù", "Thiên Không", "Thiên Tài", "Đẩu Quân"]),
        ("Dần", "Mậu Dần", "Phụ Mẫu", 16, &["Phá Quân (H)"], &["Tang Môn", "Phi Liêm", "Trường Sinh", "Tam Thai", "Thiên Việt", "Thiên Trù", "Cô Thần", "Thiên Mã"]),
        ("Mão", "Kỷ Mão", "Phúc Đức", 26, &[], &["Thiếu Âm", "Hỷ Thần", "Mộc Dục", "Hồng Loan"]),
        ("Thìn", "Canh Thìn", "Điền Trạch", 36, &["Liêm Trinh (M)", "Thiên Phủ (V)"], &["Quan Phù", "Bệnh Phù", "Quan Đới", "Địa Kiếp", "Thiên Quý", "Quốc Ấn", "Long Trì", "Hoa Cái", "Thiên La"]),
        ("Tỵ", "Tân Tỵ", "Quan Lộc", 46, &["Thái Âm (H)"], &["Tử Phù", "Đại Hao", "Lâm Quan", "Thiên Riêu", "Thiên Y", "Văn Xương", "Linh Tinh", "Kiếp Sát", "Phá Toái", "Hóa Khoa", "Nguyệt Đức"]),
        ("Ngọ", "Nhâm Ngọ", "Nô Bộc", 56, &["Tham Lang (H)"], &["Tuế Phá", "Phục Binh", "Đế Vượng", "Hữu Bật", "Địa Không", "Thiên Khôi", "Thiên Phúc", "Thiên Khốc", "Thiên Hư", "Thiên Thương"]),
        ("Mùi", "Quý Mùi", "Thiên Di", 66, &["Thiên Đồng (H)", "Cự Môn (H)"], &["Long Đức", "Đà La", "Quan Phủ", "Suy", "Phong Cáo", "Hỏa Tinh", "Hóa Kỵ"]),
        ("Thân", "Giáp Thân", "Tật Ách", 76, &["Vũ Khúc (V)", "Thiên Tướng (M)"], &["Bạch Hổ", "Lộc Tồn", "Bác Sỹ", "Bệnh", "Tả Phụ", "Lưu Hà", "Hóa Quyền", "Thiên Sứ"]),
        ("Dậu", "Ất Dậu", "Tài Bạch", 86, &["Thái Dương (H)", "Thiên Lương (H)"], &["Phúc Đức", "Kình Dương", "Lực Sỹ", "Tử", "Văn Khúc", "Thiên Hỷ", "Đào Hoa", "Hóa Lộc", "Thiên Đức"]),
        ("Tuất", "Bính Tuất", "Tử Tức", 96, &["Thất Sát (H)"], &["Điếu Khách", "Thanh Long", "Mộ", "Ân Quang", "Phượng Các", "Giải Thần", "Quả Tú", "Địa Võng"]),
        ("Hợi", "Đinh Hợi", "Phu Thê", 106, &["Thiên Cơ (H)"], &["Trực Phù", "Tiểu Hao", "Tuyệt", "Địa Giải", "Thai Phụ", "Thiên Quan", "Thiên Thọ", "Lưu Niên Văn Tinh"]),
    ];

    #[rustfmt::skip]
    const V4: [VectorRow; 12] = [
        ("Tý", "Giáp Tý", "Quan Lộc", 83, &["Liêm Trinh (V)", "Thiên Tướng (V)"], &["Trực Phù", "Lộc Tồn", "Bác Sỹ", "Dưỡng", "Tả Phụ", "Linh Tinh"]),
        ("Sửu", "Ất Sửu", "Nô Bộc", 73, &["Thiên Lương (Đ)"], &["Thái Tuế", "Kình Dương", "Quan Phủ", "Thai", "Địa Kiếp", "Hỏa Tinh", "Hoa Cái", "Phá Toái", "Thiên Thọ", "Thiên Thương"]),
        ("Dần", "Giáp Dần", "Thiên Di", 63, &["Thất Sát (M)"], &["Thiếu Dương", "Phục Binh", "Tuyệt", "Hữu Bật", "Lưu Hà", "Hồng Loan", "Cô Thần", "Kiếp Sát", "Thiên Không"]),
        ("Mão", "Ất Mão", "Tật Ách", 53, &["Thiên Đồng (Đ)"], &["Tang Môn", "Đại Hao", "Mộ", "Địa Giải", "Thiên Khôi", "Thiên Sứ", "Lưu Niên Văn Tinh"]),
        ("Thìn", "Bính Thìn", "Tài Bạch", 43, &["Vũ Khúc (M)"], &["Thiếu Âm", "Bệnh Phù", "Tử", "Thiên Giải", "Phong Cáo", "Bát Tọa", "Thiên La"]),
        ("Tỵ", "Đinh Tỵ", "Tử Tức", 33, &["Thái Dương (M)"], &["Quan Phù", "Hỷ Thần", "Bệnh", "Thiên Hình", "Ân Quang", "Thiên Việt", "Thiên Phúc", "Đường Phù", "Long Trì", "Thiên Khốc"]),
        ("Ngọ", "Mậu Ngọ", "Phu Thê", 23, &["Phá Quân (M)"], &["Tử Phù", "Phi Liêm", "Suy", "Văn Khúc", "Thiên Quan", "Đào Hoa", "Hóa Lộc", "Nguyệt Đức"]),
        ("Mùi", "Kỷ Mùi", "Huynh Đệ", 13, &["Thiên Cơ (Đ)"], &["Tuế Phá", "Tấu Thư", "Đế Vượng", "Thiên Hư", "Đẩu Quân"]),
        ("Thân", "Canh Thân", "Mệnh", 3, &["Tử Vi (M)", "Thiên Phủ (M)"], &["Long Đức", "Tướng Quân", "Lâm Quan", "Văn Xương", "Thai Phụ", "Quốc Ấn", "Thiên Hỷ"]),
        ("Dậu", "Tân Dậu", "Phụ Mẫu", 113, &["Thái Âm (M)"], &["Bạch Hổ", "Tiểu Hao", "Quan Đới", "Thiên Riêu", "Thiên Y", "Địa Không", "Thiên Quý", "Phượng Các", "Giải Thần", "Hóa Khoa", "Thiên Tài"]),
        ("Tuất", "Nhâm Tuất", "Phúc Đức", 103, &["Tham Lang (V)"], &["Phúc Đức", "Thanh Long", "Mộc Dục", "Tam Thai", "Thiên Trù", "Quả Tú", "Hóa Kỵ", "Thiên Đức", "Địa Võng"]),
        ("Hợi", "Quý Hợi", "Điền Trạch", 93, &["Cự Môn (Đ)"], &["Điếu Khách", "Đà La", "Lực Sỹ", "Trường Sinh", "Thiên Mã", "Hóa Quyền"]),
    ];

    #[rustfmt::skip]
    const V5: [VectorRow; 12] = [
        ("Tý", "Canh Tý", "Phúc Đức", 103, &[], &["Tử Phù", "Đại Hao", "Dưỡng", "Hữu Bật", "Ân Quang", "Linh Tinh", "Đào Hoa", "Nguyệt Đức", "Lưu Niên Văn Tinh"]),
        ("Sửu", "Tân Sửu", "Điền Trạch", 93, &[], &["Tuế Phá", "Bệnh Phù", "Thai", "Địa Kiếp", "Phá Toái", "Thiên Hư"]),
        ("Dần", "Canh Dần", "Quan Lộc", 83, &[], &["Long Đức", "Hỷ Thần", "Tuyệt", "Tả Phụ", "Thiên Quý", "Thiên Việt", "Đường Phù", "Thiên Hỷ"]),
        ("Mão", "Tân Mão", "Nô Bộc", 73, &["Liêm Trinh (H)", "Phá Quân (H)"], &["Bạch Hổ", "Phi Liêm", "Mộ", "Lưu Hà", "Phượng Các", "Giải Thần", "Thiên Thương"]),
        ("Thìn", "Nhâm Thìn", "Thiên Di", 63, &[], &["Phúc Đức", "Tấu Thư", "Tử", "Phong Cáo", "Quả Tú", "Thiên Đức", "Thiên La"]),
        ("Tỵ", "Quý Tỵ", "Tật Ách", 53, &["Thiên Phủ (Đ)"], &["Điếu Khách", "Tướng Quân", "Bệnh", "Địa Giải", "Thiên Phúc", "Quốc Ấn", "Thiên Mã", "Thiên Tài", "Thiên Sứ"]),
        ("Ngọ", "Giáp Ngọ", "Tài Bạch", 43, &["Thiên Đồng (H)", "Thái Âm (H)"], &["Trực Phù", "Tiểu Hao", "Suy", "Thiên Giải", "Văn Khúc", "Thiên Khôi", "Thiên Trù", "Hóa Khoa"]),
        ("Mùi", "Ất Mùi", "Tử Tức", 33, &["Vũ Khúc (M)", "Tham Lang (M)"], &["Thái Tuế", "Thanh Long", "Đế Vượng", "Thiên Hình", "Tam Thai", "Bát Tọa", "Hỏa Tinh", "Hoa Cái"]),
        ("Thân", "Bính Thân", "Phu Thê", 23, &["Thái Dương (H)", "Cự Môn (Đ)"], &["Thiếu Dương", "Đà La", "Lực Sỹ", "Lâm Quan", "Văn Xương", "Thai Phụ", "Hồng Loan", "Cô Thần", "Kiếp Sát", "Thiên Không", "Hóa Lộc", "Hóa Quyền", "Hóa Kỵ"]),
        ("Dậu", "Đinh Dậu", "Huynh Đệ", 13, &["Thiên Tướng (H)"], &["Tang Môn", "Lộc Tồn", "Bác Sỹ", "Quan Đới", "Địa Không", "Thiên Quan", "Thiên Thọ"]),
        ("Tuất", "Mậu Tuất", "Mệnh", 3, &["Thiên Cơ (M)", "Thiên Lương (M)"], &["Thiếu Âm", "Kình Dương", "Quan Phủ", "Mộc Dục", "Địa Võng"]),
        ("Hợi", "Kỷ Hợi", "Phụ Mẫu", 113, &["Tử Vi (B)", "Thất Sát (V)"], &["Quan Phù", "Phục Binh", "Trường Sinh", "Thiên Riêu", "Thiên Y", "Long Trì", "Thiên Khốc", "Đẩu Quân"]),
    ];

    fn check_vector(chart: &TuViChart, rows: &[VectorRow; 12]) {
        for (branch, can_chi, role, dai_han, main, others) in rows {
            let palace = chart.palace(branch_named(branch));
            assert_eq!(
                format!("{} {}", palace.stem.name_vi(), palace.branch.name_vi()),
                *can_chi
            );
            assert_eq!(palace.role.name_vi(), *role, "{branch}");
            assert_eq!(palace.dai_han_start_age, *dai_han, "{branch}");
            assert_eq!(main_names(palace), *main, "{branch}");
            assert_eq!(other_names(palace), sorted(others), "{branch}");
        }
    }

    #[test]
    fn vector_1_second_chart_of_the_book() {
        let chart = from_lunar(lunar(1891, 6, 6), Mao, Gender::Male);
        check_vector(&chart, &V1);
        assert_eq!(chart.polarity, TuViPolarity::AmNam);
        assert!(!chart.forward);
        assert_eq!(chart.menh_branch, Thin);
        assert_eq!(chart.than_branch, Tuat);
        assert_eq!(chart.than_role, TuViPalaceRole::ThienDi);
        assert_eq!(chart.bureau, FiveElementsBureau::ThuyNhi);
        assert_eq!(chart.ban_menh.name_vi(), "Tùng Bách Mộc");
        assert_eq!(
            chart.solar_date,
            SolarDate {
                year: 1891,
                month: 7,
                day: 11
            }
        );
        assert_eq!((chart.tuan.first, chart.tuan.second), (Ngo, Mui));
        assert_eq!((chart.triet.first, chart.triet.second), (Thin, Ti));
        assert_eq!(chart.tieu_han_palace(1).unwrap().branch, Suu);
    }

    #[test]
    fn vector_2_third_chart_of_the_book() {
        let chart = from_lunar(lunar(1904, 9, 10), Ngo, Gender::Female);
        check_vector(&chart, &V2);
        assert_eq!(chart.polarity, TuViPolarity::DuongNu);
        assert_eq!(chart.than_role, TuViPalaceRole::Menh);
        assert!(chart.palace(Thin).is_than);
        assert_eq!(chart.bureau, FiveElementsBureau::MocTam);
        assert_eq!(chart.ban_menh.name_vi(), "Phú Đăng Hỏa");
        assert_eq!(chart.tieu_han_palace(47).unwrap().branch, Ty);
        assert_eq!(chart.dai_han_palace(47).unwrap().dai_han_start_age, 43);
    }

    #[test]
    fn vector_3_duong_nam_in_a_leap_month() {
        let birth = date_time(2020, 6, 10, 10, 30);
        let chart = tuvi_chart(birth, Gender::Male, &TuViConfig::default()).unwrap();
        check_vector(&chart, &V3);
        assert_eq!(
            chart.lunar,
            LunarDate {
                year: 2020,
                month: 4,
                day: 19,
                is_leap_month: true
            }
        );
        assert_eq!(chart.effective_month, 5);
        assert_eq!(chart.hour_branch, Ti);
        assert_eq!(chart.polarity, TuViPolarity::DuongNam);
        assert!(chart.forward);
        assert_eq!(chart.menh_branch, Suu);
        assert_eq!(chart.than_branch, Hoi);
        assert_eq!(chart.bureau, FiveElementsBureau::HoaLuc);
        assert_eq!(chart.ban_menh.name_vi(), "Bích Thượng Thổ");
        // P. 20: Hỏa Lục Cục writes 6 at Mệnh, 16 at Phụ Mẫu, 26 at Phúc Đức.
        for (role, age) in [
            (TuViPalaceRole::Menh, 6),
            (TuViPalaceRole::PhuMau, 16),
            (TuViPalaceRole::PhucDuc, 26),
        ] {
            assert_eq!(chart.palace_of_role(role).dai_han_start_age, age);
        }

        let same_month = TuViConfig {
            leap_month_rule: LeapMonthRule::SameAsCurrentMonth,
            ..TuViConfig::default()
        };
        let chart = tuvi_chart(birth, Gender::Male, &same_month).unwrap();
        assert_eq!(chart.effective_month, 4);
        assert_eq!(chart.menh_branch, Ty);
        assert_eq!(chart.than_branch, Tuat);
        assert_eq!(chart.bureau, FiveElementsBureau::HoaLuc);
        assert_eq!(branch_of(&chart, TuViStar::TuVi), Ty);
        assert_eq!(branch_of(&chart, TuViStar::ThienPhu), Thin);
    }

    #[test]
    fn vector_4_first_chart_of_the_book() {
        let chart = from_lunar(lunar(1913, 9, 23), Dan, Gender::Male);
        check_vector(&chart, &V4);
        assert_eq!(
            chart.solar_date,
            SolarDate {
                year: 1913,
                month: 10,
                day: 22
            }
        );
        assert_eq!(chart.ban_menh.name_vi(), "Tang Đố Mộc");
        assert_eq!(chart.than_role, TuViPalaceRole::QuanLoc);
    }

    #[test]
    fn vector_5_fourth_chart_of_the_book() {
        let chart = from_lunar(lunar(1931, 11, 30), Dan, Gender::Male);
        check_vector(&chart, &V5);
        assert_eq!(
            chart.solar_date,
            SolarDate {
                year: 1932,
                month: 1,
                day: 7
            }
        );
        assert_eq!(chart.ban_menh.name_vi(), "Lộ Bàng Thổ");
        // "Cung nhập hạn có Tuần án ngữ, Triệt xung chiếu" at 23 tuổi.
        let han = chart.tieu_han_palace(23).unwrap();
        assert_eq!(han.branch, Hoi);
        assert!(han.has_tuan);
        assert!(chart.palace(han.branch.clash()).has_triet);
    }

    // ---- The four charts as printed, section 7.1 of the spec ----

    /// Role, Đại Hạn when printed, Chính Tinh with their marks, and other stars, read off the page.
    type PrintedPalace = (
        TuViPalaceRole,
        Option<u8>,
        &'static [&'static str],
        &'static [&'static str],
    );

    struct PrintedChart {
        lunar: LunarDate,
        hour: Branch,
        gender: Gender,
        number: u8,
        ban_menh: &'static str,
        than: TuViPalaceRole,
        tuan: (TuViPalaceRole, TuViPalaceRole),
        triet: (TuViPalaceRole, TuViPalaceRole),
        palaces: [PrintedPalace; 12],
    }

    #[rustfmt::skip]
    const BOOK_1: PrintedChart = PrintedChart {
        lunar: LunarDate { year: 1913, month: 9, day: 23, is_leap_month: false },
        hour: Dan,
        gender: Gender::Male,
        number: 3,
        ban_menh: "Tang Đố Mộc",
        than: TuViPalaceRole::QuanLoc,
        tuan: (TuViPalaceRole::ThienDi, TuViPalaceRole::TatAch),
        triet: (TuViPalaceRole::NoBoc, TuViPalaceRole::QuanLoc),
        palaces: [
            (TuViPalaceRole::Menh, Some(3), &["Tử Vi (M)", "Thiên Phủ (M)"], &["Tướng Quân", "Long Đức", "Văn Xương", "Quốc Ấn", "Thiên Hỷ", "Thai Phụ", "Lâm Quan"]),
            (TuViPalaceRole::PhuMau, None, &["Thái Âm (M)"], &["Tiểu Hao", "Bạch Hổ", "Thiên Riêu", "Địa Không", "Phượng Các", "Thiên Quý", "Thiên Y", "Giải Thần", "Thiên Tài", "Hóa Khoa", "Quan Đới"]),
            (TuViPalaceRole::PhucDuc, None, &["Tham Lang (V)"], &["Địa Võng", "Quả Tú", "Hóa Kỵ", "Thanh Long", "Phúc Đức", "Tam Thai", "Thiên Đức", "Thiên Trù", "Mộc Dục"]),
            (TuViPalaceRole::DienTrach, Some(93), &["Cự Môn (Đ)"], &["Điếu Khách", "Đà La", "Lực Sỹ", "Hóa Quyền", "Thiên Mã", "Tràng Sinh"]),
            (TuViPalaceRole::QuanLoc, Some(83), &["Liêm Trinh (V)", "Thiên Tướng (V)"], &["Trực Phù", "Linh Tinh", "Lộc Tồn", "Bác Sỹ", "Tả Phù", "Dưỡng"]),
            (TuViPalaceRole::NoBoc, Some(73), &["Thiên Lương (Đ)"], &["Quan Phù", "Thái Tuế", "Thiên Thương", "Phá Toái", "Kình Dương", "Địa Kiếp", "Hỏa Tinh", "Thiên Thọ", "Hoa Cái", "Thai"]),
            (TuViPalaceRole::ThienDi, Some(63), &["Thất Sát (M)"], &["Phục Binh", "Cô Thần", "Lưu Hà", "Kiếp Sát", "Thiên Không", "Thiếu Dương", "Hữu Bật", "Hồng Loan", "Tuyệt"]),
            (TuViPalaceRole::TatAch, Some(53), &["Thiên Đồng (Đ)"], &["Đại Hao", "Tang Môn", "Thiên Sứ", "Thiên Khôi", "Địa Giải", "Lưu Niên", "Mộ"]),
            (TuViPalaceRole::TaiBach, Some(43), &["Vũ Khúc (M)"], &["Bệnh Phù", "Thiên La", "Thiếu Âm", "Bát Tọa", "Thiên Giải", "Phong Cáo", "Tử"]),
            (TuViPalaceRole::TuTuc, Some(33), &["Thái Dương (M)"], &["Quan Phù", "Thiên Khốc", "Thiên Hình", "Hỷ Thần", "Long Trì", "Ân Quang", "Thiên Việt", "Đường Phù", "Thiên Phúc", "Bệnh"]),
            (TuViPalaceRole::PhuThe, Some(23), &["Phá Quân (M)"], &["Phi Liêm", "Tử Phù", "Văn Khúc", "Nguyệt Đức", "Đào Hoa", "Thiên Quan", "Hóa Lộc", "Suy"]),
            (TuViPalaceRole::HuynhDe, Some(13), &["Thiên Cơ (Đ)"], &["Tuế Phá", "Thiên Hư", "Tấu Thư", "Đẩu Quân", "Đế Vượng"]),
        ],
    };

    #[rustfmt::skip]
    const BOOK_2: PrintedChart = PrintedChart {
        lunar: LunarDate { year: 1891, month: 6, day: 6, is_leap_month: false },
        hour: Mao,
        gender: Gender::Male,
        number: 2,
        ban_menh: "Tùng Bách Mộc",
        than: TuViPalaceRole::ThienDi,
        tuan: (TuViPalaceRole::PhucDuc, TuViPalaceRole::DienTrach),
        triet: (TuViPalaceRole::PhuMau, TuViPalaceRole::Menh),
        palaces: [
            (TuViPalaceRole::Menh, Some(2), &["Tử Vi (V)", "Thiên Tướng (V)"], &["Thiên La", "Lưu Hà", "Thiên Không", "Tấu Thư", "Thiếu Dương", "Đế Vượng"]),
            (TuViPalaceRole::PhuMau, None, &["Thiên Lương (H)"], &["Tang Môn", "Cô Thần", "Phá Toái", "Tướng Quân", "Hữu Bật", "Quốc Ấn", "Phong Cáo", "Thiên Mã", "Thiên Phúc", "Lâm Quan"]),
            (TuViPalaceRole::PhucDuc, None, &["Thất Sát (M)"], &["Tiểu Hao", "Thiên Riêu", "Hỏa Tinh", "Thiếu Âm", "Thiên Khôi", "Thiên Y", "Thiên Hỷ", "Thiên Trù", "Quan Đới"]),
            (TuViPalaceRole::DienTrach, None, &[], &["Quan Phù", "Hóa Kỵ", "Thanh Long", "Long Trì", "Phượng Các", "Văn Xương", "Văn Khúc", "Giải Thần", "Thiên Tài", "Hoa Cái", "Hóa Khoa", "Mộc Dục"]),
            (TuViPalaceRole::QuanLoc, None, &["Liêm Trinh (V)"], &["Tử Phù", "Kiếp Sát", "Đà La", "Địa Không", "Lực Sỹ", "Nguyệt Đức", "Tràng Sinh"]),
            (TuViPalaceRole::NoBoc, None, &[], &["Tuế Phá", "Thiên Hư", "Thiên Thương", "Lộc Tồn", "Bác Sỹ", "Tả Phù", "Thai Phụ", "Thiên Quan", "Dưỡng"]),
            (TuViPalaceRole::ThienDi, Some(62), &["Phá Quân (Đ)"], &["Quan Phủ", "Địa Võng", "Kình Dương", "Long Đức", "Thai"]),
            (TuViPalaceRole::TatAch, Some(52), &["Thiên Đồng (Đ)"], &["Phục Binh", "Bạch Hổ", "Thiên Sứ", "Ân Quang", "Tuyệt"]),
            (TuViPalaceRole::TaiBach, Some(42), &["Vũ Khúc (M)", "Thiên Phủ (M)"], &["Đại Hao", "Phúc Đức", "Bát Tọa", "Thiên Đức", "Đào Hoa", "Hồng Loan", "Địa Giải", "Lưu Niên", "Mộ"]),
            (TuViPalaceRole::TuTuc, Some(32), &["Thái Dương (Đ)", "Thái Âm (Đ)"], &["Bệnh Phù", "Quả Tú", "Điếu Khách", "Linh Tinh", "Thiên Giải", "Thiên Thọ", "Đẩu Quân", "Hóa Quyền", "Tử"]),
            (TuViPalaceRole::PhuThe, Some(22), &["Tham Lang (Đ)"], &["Trực Phù", "Thiên Hình", "Địa Kiếp", "Hỷ Thần", "Tam Thai", "Thiên Việt", "Đường Phù", "Bệnh"]),
            (TuViPalaceRole::HuynhDe, Some(12), &["Cự Môn (M)", "Thiên Cơ (M)"], &["Phi Liêm", "Thái Tuế", "Thiên Khốc", "Thiên Quý", "Hóa Lộc", "Suy"]),
        ],
    };

    #[rustfmt::skip]
    const BOOK_3: PrintedChart = PrintedChart {
        lunar: LunarDate { year: 1904, month: 9, day: 10, is_leap_month: false },
        hour: Ngo,
        gender: Gender::Female,
        number: 3,
        ban_menh: "Phú Đăng Hỏa",
        than: TuViPalaceRole::Menh,
        tuan: (TuViPalaceRole::HuynhDe, TuViPalaceRole::PhuThe),
        triet: (TuViPalaceRole::QuanLoc, TuViPalaceRole::NoBoc),
        palaces: [
            (TuViPalaceRole::Menh, Some(3), &["Thái Dương (V)"], &["Phục Binh", "Thái Tuế", "Thiên La", "Linh Tinh", "Hóa Kỵ", "Văn Xương", "Thiên Giải", "Hoa Cái", "Tử"]),
            (TuViPalaceRole::PhuMau, None, &[], &["Đại Hao", "Thiên Hình", "Cô Thần", "Kiếp Sát", "Thiên Không", "Địa Kiếp", "Địa Không", "Thiếu Dương", "Bát Tọa", "Thiên Hỷ", "Lưu Niên", "Thiên Trù", "Bệnh"]),
            (TuViPalaceRole::PhucDuc, None, &["Thiên Cơ (Đ)"], &["Bệnh Phù", "Tang Môn", "Phượng Các", "Giải Thần", "Suy"]),
            (TuViPalaceRole::DienTrach, None, &["Tử Vi (Đ)", "Phá Quân (Đ)"], &["Hỷ Thần", "Thiếu Âm", "Thiên Việt", "Đường Phù", "Thiên Quan", "Hóa Quyền", "Đế Vượng"]),
            (TuViPalaceRole::QuanLoc, None, &[], &["Phi Liêm", "Quan Phù", "Hỏa Tinh", "Long Trì", "Phong Cáo", "Thiên Tài", "Thiên Thọ", "Lâm Quan"]),
            (TuViPalaceRole::NoBoc, None, &["Thiên Phủ (B)"], &["Tử Phù", "Thiên Riêu", "Thiên Thương", "Lưu Hà", "Tấu Thư", "Tam Thai", "Nguyệt Đức", "Thiên Y", "Đào Hoa", "Thiên Phúc", "Quan Đới"]),
            (TuViPalaceRole::ThienDi, Some(63), &["Thái Âm (M)"], &["Tuế Phá", "Thiên Hư", "Địa Võng", "Tướng Quân", "Văn Khúc", "Quốc Ấn", "Mộc Dục"]),
            (TuViPalaceRole::TatAch, Some(53), &["Liêm Trinh (H)", "Tham Lang (H)"], &["Tiểu Hao", "Thiên Sứ", "Long Đức", "Hồng Loan", "Hóa Lộc", "Tràng Sinh"]),
            (TuViPalaceRole::TaiBach, Some(43), &["Cự Môn (V)"], &["Bạch Hổ", "Thanh Long", "Tả Phù", "Ân Quang", "Thai Phụ", "Dưỡng"]),
            (TuViPalaceRole::TuTuc, Some(33), &["Thiên Tướng (Đ)"], &["Quả Tú", "Phá Toái", "Đà La", "Lực Sỹ", "Phúc Đức", "Thiên Khôi", "Thiên Đức", "Thai"]),
            (TuViPalaceRole::PhuThe, Some(23), &["Thiên Đồng (M)", "Thiên Lương (V)"], &["Điếu Khách", "Thiên Khốc", "Lộc Tồn", "Bác Sỹ", "Hữu Bật", "Thiên Quý", "Thiên Mã", "Đẩu Quân", "Tuyệt"]),
            (TuViPalaceRole::HuynhDe, Some(13), &["Vũ Khúc (Đ)", "Thất Sát (H)"], &["Quan Phủ", "Trực Phù", "Kình Dương", "Địa Giải", "Hóa Khoa", "Mộ"]),
        ],
    };

    #[rustfmt::skip]
    const BOOK_4: PrintedChart = PrintedChart {
        lunar: LunarDate { year: 1931, month: 11, day: 30, is_leap_month: false },
        hour: Dan,
        gender: Gender::Male,
        number: 3,
        ban_menh: "Lộ Bàng Thổ",
        than: TuViPalaceRole::QuanLoc,
        tuan: (TuViPalaceRole::Menh, TuViPalaceRole::PhuMau),
        triet: (TuViPalaceRole::TatAch, TuViPalaceRole::ThienDi),
        palaces: [
            (TuViPalaceRole::Menh, Some(3), &["Thiên Cơ (M)", "Thiên Lương (M)"], &["Quan Phủ", "Địa Võng", "Kình Dương", "Thiếu Âm", "Mộc Dục"]),
            (TuViPalaceRole::PhuMau, None, &["Tử Vi (B)", "Thất Sát (V)"], &["Phục Binh", "Quan Phù", "Thiên Khốc", "Thiên Riêu", "Long Trì", "Thiên Y", "Đẩu Quân", "Thiên Mã", "Tràng Sinh"]),
            (TuViPalaceRole::PhucDuc, None, &[], &["Đại Hao", "Tử Phù", "Linh Tinh", "Hữu Bật", "Ân Quang", "Nguyệt Đức", "Đào Hoa", "Lưu Niên", "Dưỡng"]),
            (TuViPalaceRole::DienTrach, None, &[], &["Bệnh Phù", "Tuế Phá", "Thiên Hư", "Phá Toái", "Địa Kiếp", "Thai"]),
            (TuViPalaceRole::QuanLoc, None, &[], &["Hỷ Thần", "Long Đức", "Tả Phù", "Thiên Quý", "Thiên Việt", "Đường Phù", "Thiên Hỷ", "Tuyệt"]),
            (TuViPalaceRole::NoBoc, None, &["Liêm Trinh (H)", "Phá Quân (H)"], &["Phi Liêm", "Bạch Hổ", "Thiên Thương", "Phượng Các", "Giải Thần", "Mộ"]),
            (TuViPalaceRole::ThienDi, Some(63), &[], &["Thiên La", "Quả Tú", "Lưu Hà", "Tấu Thư", "Phúc Đức", "Thiên Đức", "Phong Cáo", "Tử"]),
            (TuViPalaceRole::TatAch, Some(53), &["Thiên Phủ (Đ)"], &["Điếu Khách", "Thiên Sứ", "Thiếu Dương", "Tướng Quân", "Quốc Ấn", "Địa Giải", "Thiên Tài", "Thiên Mã", "Thiên Phúc", "Bệnh"]),
            (TuViPalaceRole::TaiBach, Some(43), &["Thiên Đồng (H)", "Thái Âm (H)"], &["Tiểu Hao", "Trực Phù", "Văn Khúc", "Thiên Khôi", "Thiên Giải", "Thiên Trù", "Hóa Khoa", "Suy"]),
            (TuViPalaceRole::TuTuc, Some(33), &["Vũ Khúc (M)", "Tham Lang (M)"], &["Thái Tuế", "Thiên Hình", "Hỏa Tinh", "Thanh Long", "Tam Thai", "Bát Tọa", "Hoa Cái", "Đế Vượng"]),
            (TuViPalaceRole::PhuThe, Some(23), &["Thái Dương (H)", "Cự Môn (Đ)"], &["Cô Thần", "Kiếp Sát", "Hóa Kỵ", "Thiên Không", "Đà La", "Lực Sỹ", "Thiếu Dương", "Văn Xương", "Hồng Loan", "Thai Phụ", "Hóa Quyền", "Hóa Lộc", "Lâm Quan"]),
            (TuViPalaceRole::HuynhDe, Some(13), &["Thiên Tướng (H)"], &["Tang Môn", "Địa Không", "Lộc Tồn", "Bác Sỹ", "Thiên Thọ", "Thiên Quan", "Quan Đới"]),
        ],
    };

    /// A misprint the code rightly does not reproduce: role, what the page prints (or `None`) and
    /// what belongs there (or `None`).
    type Misprint = (TuViPalaceRole, Option<&'static str>, Option<&'static str>);

    fn check_printed(chart: &PrintedChart, misprints: &[Misprint], marks: &[(&str, &str)]) {
        let cast = from_lunar(chart.lunar, chart.hour, chart.gender);
        assert_eq!(cast.bureau.number(), chart.number);
        assert_eq!(cast.ban_menh.name_vi(), chart.ban_menh);
        assert_eq!(cast.than_role, chart.than);
        for role in [chart.tuan.0, chart.tuan.1] {
            assert!(cast.palace_of_role(role).has_tuan, "Tuần at {role:?}");
        }
        for role in [chart.triet.0, chart.triet.1] {
            assert!(cast.palace_of_role(role).has_triet, "Triệt at {role:?}");
        }

        let mut placements = 0;
        for (role, dai_han, main, others) in &chart.palaces {
            let palace = cast.palace_of_role(*role);
            if let Some(age) = dai_han {
                assert_eq!(palace.dai_han_start_age, *age, "{role:?}");
            }
            // The page's marks, with the two the book's own lists overrule.
            let printed: Vec<&str> = main
                .iter()
                .map(|star| {
                    let fixed = marks.iter().find(|(from, _)| from == star);
                    fixed.map_or(*star, |(_, to)| to)
                })
                .collect();
            let mut cast_main = main_names(palace);
            cast_main.sort();
            assert_eq!(cast_main, sorted(&printed), "{role:?}");

            let mut printed: BTreeMap<&str, u32> = BTreeMap::new();
            for name in *others {
                *printed.entry(canonical(name)).or_default() += 1;
            }
            for (at, wrong, right) in misprints {
                if at == role {
                    if let Some(wrong) = wrong {
                        *printed.get_mut(wrong).expect("the misprint is on the page") -= 1;
                    }
                    if let Some(right) = right {
                        *printed.entry(right).or_default() += 1;
                    }
                }
            }
            let mut names: Vec<String> = printed
                .into_iter()
                .filter(|(_, count)| *count > 0)
                .map(|(name, _)| name.to_string())
                .collect();
            names.sort();
            assert_eq!(other_names(palace), names, "{role:?}");
            placements += main.len() + names.len();
        }
        assert_eq!(placements, TuViStar::COUNT + 4);
    }

    /// The spec's name of a star as the charts of the book spell it.
    fn canonical(name: &str) -> &str {
        match name {
            "Tả Phù" => "Tả Phụ",
            "Tràng Sinh" => "Trường Sinh",
            "Lưu Niên" => "Lưu Niên Văn Tinh",
            other => other,
        }
    }

    #[test]
    fn first_chart_of_the_book_star_by_star() {
        // Only the spelling of Quan Phủ, the last star of the Bác Sỹ ring.
        let misprints = [(TuViPalaceRole::NoBoc, Some("Quan Phù"), Some("Quan Phủ"))];
        check_printed(&BOOK_1, &misprints, &[]);
    }

    #[test]
    fn second_chart_of_the_book_star_by_star() {
        // Lưu Hà of Can Tân belongs at Mão, Huynh Đệ here (T3).
        let misprints = [
            (TuViPalaceRole::Menh, Some("Lưu Hà"), None),
            (TuViPalaceRole::HuynhDe, None, Some("Lưu Hà")),
        ];
        check_printed(&BOOK_2, &misprints, &[("Vũ Khúc (M)", "Vũ Khúc (V)")]);
    }

    #[test]
    fn third_chart_of_the_book_star_by_star() {
        check_printed(&BOOK_3, &[], &[("Phá Quân (Đ)", "Phá Quân (V)")]);
    }

    #[test]
    fn fourth_chart_of_the_book_star_by_star() {
        let misprints = [
            // Lưu Hà of Can Tân belongs at Mão, Nô Bộc here (T3).
            (TuViPalaceRole::ThienDi, Some("Lưu Hà"), None),
            (TuViPalaceRole::NoBoc, None, Some("Lưu Hà")),
            // Printed twice: both stars also stand in their right palace.
            (TuViPalaceRole::PhuMau, Some("Thiên Mã"), None),
            (TuViPalaceRole::TatAch, Some("Thiếu Dương"), None),
        ];
        check_printed(&BOOK_4, &misprints, &[]);
    }

    #[test]
    fn hoa_tinh_and_hoa_quyen_keep_the_readings_of_the_charts() {
        // Working back from the Tân Mão and Tân Mùi charts: Hỏa Tinh starts at Dậu.
        let chart = from_lunar(lunar(1891, 6, 6), Mao, Gender::Male);
        assert_eq!(branch_of(&chart, TuViStar::HoaTinh), Ngo);
        let quyen = chart.palace(Suu).main_stars[0];
        assert_eq!(quyen.star, TuViStar::ThaiDuong);
        assert_eq!(quyen.transformation, Some(FourTransformations::HoaQuyen));
        let chart = from_lunar(lunar(1931, 11, 30), Dan, Gender::Male);
        assert_eq!(branch_of(&chart, TuViStar::HoaTinh), Mui);
    }

    // ---- Chart assembly ----

    #[test]
    fn every_star_sits_in_exactly_one_palace_on_a_sweep() {
        for index in 0..60 {
            let pillar = CanChi::from_cycle_index(index);
            let year = (1984..2044).find(|&y| year_can_chi(y) == pillar).unwrap();
            for (month, day, hour, gender) in [
                (1, 1, Ty, Gender::Male),
                (6, 15, Ngo, Gender::Female),
                (11, 29, Hoi, Gender::Male),
                (12, 2, Dan, Gender::Female),
            ] {
                let chart = from_lunar(lunar(year, month, day), hour, gender);
                let mut seen = [0; TuViStar::COUNT];
                for palace in &chart.palaces {
                    for placed in palace.main_stars.iter().chain(&palace.minor_stars) {
                        seen[placed.star as usize] += 1;
                    }
                    let stage: TuViStar = palace.life_stage.into();
                    assert!(palace.minor_stars.iter().any(|placed| placed.star == stage));
                }
                assert_eq!(seen, [1; TuViStar::COUNT]);

                let branches: Vec<Branch> = chart.palaces.iter().map(|p| p.branch).collect();
                assert_eq!(branches, Branch::ALL);
                let mut ages: Vec<u8> = chart.palaces.iter().map(|p| p.dai_han_start_age).collect();
                ages.sort();
                let first = chart.bureau.number();
                assert_eq!(
                    ages,
                    core::array::from_fn::<u8, 12, _>(|k| first + 10 * k as u8)
                );
                let mut tieu: Vec<u8> =
                    chart.palaces.iter().map(|p| p.tieu_han_first_age).collect();
                tieu.sort();
                assert_eq!(tieu, core::array::from_fn::<u8, 12, _>(|k| k as u8 + 1));
                assert_eq!(chart.palaces.iter().filter(|p| p.is_than).count(), 1);
                assert_eq!(
                    chart.palace_of_role(TuViPalaceRole::Menh).branch,
                    chart.menh_branch
                );
                assert_eq!(chart.palace(chart.than_branch).role, chart.than_role);
            }
        }
    }

    #[test]
    fn roles_run_clockwise_from_menh() {
        let chart = from_lunar(lunar(1913, 9, 23), Dan, Gender::Male);
        for (k, role) in TuViPalaceRole::ALL.into_iter().enumerate() {
            assert_eq!(chart.palace(shift(Than, k as i64)).role, role);
        }
        assert_eq!(chart.palace(Dau).role, TuViPalaceRole::PhuMau);
        assert_eq!(chart.palace(Mui).role, TuViPalaceRole::HuynhDe);
    }

    #[test]
    fn dai_han_ages_cover_ten_years_each() {
        let chart = from_lunar(lunar(1913, 9, 23), Dan, Gender::Male);
        assert_eq!(chart.bureau.number(), 3);
        assert!(chart.dai_han_palace(2).is_none());
        assert_eq!(chart.dai_han_palace(3).unwrap().branch, Than);
        assert_eq!(chart.dai_han_palace(12).unwrap().branch, Than);
        assert_eq!(chart.dai_han_palace(13).unwrap().branch, Mui);
        assert_eq!(chart.dai_han_palace(122).unwrap().branch, Dau);
        assert!(chart.dai_han_palace(123).is_none());
    }

    #[test]
    fn tieu_han_repeats_every_twelve_years() {
        let chart = from_lunar(lunar(1913, 9, 23), Dan, Gender::Male);
        assert!(chart.tieu_han_palace(0).is_none());
        assert_eq!(chart.tieu_han_palace(1).unwrap().branch, Mui);
        assert_eq!(chart.tieu_han_palace(13).unwrap().branch, Mui);
        assert_eq!(chart.tieu_han_palace(12).unwrap().branch, Ngo);
    }

    #[test]
    fn leap_month_splits_after_the_fifteenth() {
        let leap = |day| LunarDate {
            year: 2020,
            month: 4,
            day,
            is_leap_month: true,
        };
        let rule = LeapMonthRule::SplitAt15th;
        assert_eq!(effective_month(leap(15), rule), 4);
        assert_eq!(effective_month(leap(16), rule), 5);
        assert_eq!(
            effective_month(leap(16), LeapMonthRule::SameAsCurrentMonth),
            4
        );
        assert_eq!(effective_month(lunar(2020, 4, 20), rule), 4);
        // The rule leaves the day and the year alone, and wraps past the twelfth month.
        let chart = from_lunar(leap(16), Ty, Gender::Male);
        assert_eq!(chart.lunar.day, 16);
        assert_eq!(chart.effective_month, 5);
        assert_eq!(chart.year_can_chi, year_can_chi(2020));
        assert_eq!(chart.month_can_chi, month_can_chi(2020, 4));
        let december = LunarDate {
            year: 2020,
            month: 12,
            day: 20,
            is_leap_month: true,
        };
        assert_eq!(effective_month(december, rule), 1);
    }

    #[test]
    fn no_twelfth_leap_month_exists_at_utc_plus_7() {
        for year in 1900..=2100 {
            assert_ne!(lunar_year(year, VN).unwrap().leap_month, Some(12));
        }
    }

    #[test]
    fn rat_hour_convention_sets_the_lunar_day() {
        let birth = date_time(2024, 3, 15, 23, 30);
        let split = tuvi_chart(birth, Gender::Male, &TuViConfig::default()).unwrap();
        let new_day = TuViConfig {
            rat_hour: RatHourConvention::NewDayAtRatHour,
            ..TuViConfig::default()
        };
        let next = tuvi_chart(birth, Gender::Male, &new_day).unwrap();
        assert_eq!(split.hour_branch, Ty);
        assert_eq!(next.hour_branch, Ty);
        assert_eq!(split.lunar.day + 1, next.lunar.day);
        assert_eq!(split.solar_date.day, 15);
        assert_eq!(next.solar_date.day, 16);
        // The Tý hour pillar is that of the next day under both, as in four_pillars.
        assert_eq!(split.hour_can_chi, next.hour_can_chi);
        assert_eq!(split.day_can_chi.stem.index() as i64 + 1, {
            let stem = next.day_can_chi.stem.index() as i64;
            if stem == 0 { 10 } else { stem }
        });
        // 00:30 on the same day is the same day under both.
        let late = date_time(2024, 3, 15, 0, 30);
        let a = tuvi_chart(late, Gender::Male, &TuViConfig::default()).unwrap();
        let b = tuvi_chart(late, Gender::Male, &new_day).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn new_day_at_rat_hour_can_cross_into_the_next_lunar_year() {
        // 09/02/2024 is the last day of Quý Mão; Tết of Giáp Thìn is 10/02/2024.
        let birth = date_time(2024, 2, 9, 23, 30);
        let split = tuvi_chart(birth, Gender::Male, &TuViConfig::default()).unwrap();
        assert_eq!(split.year_can_chi, year_can_chi(2023));
        let new_day = TuViConfig {
            rat_hour: RatHourConvention::NewDayAtRatHour,
            ..TuViConfig::default()
        };
        let next = tuvi_chart(birth, Gender::Male, &new_day).unwrap();
        assert_eq!(next.lunar, lunar(2024, 1, 1));
        assert_eq!(next.year_can_chi, year_can_chi(2024));
    }

    #[test]
    fn year_changes_at_tet_not_at_lap_xuan() {
        // Between Lập Xuân on 04/02/2024 and Tết on 10/02/2024 the year is still Quý Mão.
        let before_tet = date_time(2024, 2, 7, 12, 0);
        let chart = tuvi_chart(before_tet, Gender::Male, &TuViConfig::default()).unwrap();
        assert_eq!(chart.year_can_chi, year_can_chi(2023));
    }

    #[test]
    fn true_solar_time_is_opt_in() {
        // 22:55 at Hồ Chí Minh City on 03/11 is 23:17 by the Sun, so Tý instead of Hợi.
        let birth = date_time(2024, 11, 3, 22, 55);
        let clock = TuViConfig {
            longitude: Some(106.63),
            ..TuViConfig::default()
        };
        let chart = tuvi_chart(birth, Gender::Female, &clock).unwrap();
        assert_eq!(chart.hour_branch, Hoi);
        assert_eq!(chart.effective_time, Some(birth));

        let solar = TuViConfig {
            use_true_solar_time: true,
            ..clock
        };
        let chart = tuvi_chart(birth, Gender::Female, &solar).unwrap();
        assert_eq!(chart.hour_branch, Ty);
        assert_eq!(chart.effective_time.unwrap().hour, 23);

        // Without a longitude only the equation of time applies, at the 105° meridian.
        let meridian = TuViConfig {
            use_true_solar_time: true,
            ..TuViConfig::default()
        };
        let at_meridian = TuViConfig {
            longitude: Some(105.0),
            ..meridian
        };
        assert_eq!(
            tuvi_chart(birth, Gender::Female, &meridian),
            tuvi_chart(birth, Gender::Female, &at_meridian)
        );
    }

    #[test]
    fn invalid_input_is_rejected() {
        let config = TuViConfig::default();
        let birth = date_time(2024, 3, 15, 12, 0);
        let bad_zone = TuViConfig {
            tz_offset: 20.0,
            ..config
        };
        assert!(matches!(
            tuvi_chart(birth, Gender::Male, &bad_zone),
            Err(AstroError::InvalidTimeZone { .. })
        ));
        let bad_longitude = TuViConfig {
            use_true_solar_time: true,
            longitude: Some(200.0),
            ..config
        };
        assert!(matches!(
            tuvi_chart(birth, Gender::Male, &bad_longitude),
            Err(AstroError::InvalidLongitude { .. })
        ));
        let bad_time = SolarDateTime { hour: 25, ..birth };
        assert!(matches!(
            tuvi_chart(bad_time, Gender::Male, &config),
            Err(AstroError::InvalidTime { .. })
        ));
        assert!(matches!(
            tuvi_chart_from_lunar(lunar(2024, 13, 1), Ty, Gender::Male, &config),
            Err(AstroError::InvalidLunarDate { .. })
        ));
        let not_leap = LunarDate {
            is_leap_month: true,
            ..lunar(2024, 3, 1)
        };
        assert!(matches!(
            tuvi_chart_from_lunar(not_leap, Ty, Gender::Male, &config),
            Err(AstroError::InvalidLunarDate { .. })
        ));
        assert!(matches!(
            tuvi_chart_from_lunar(lunar(1500, 1, 1), Ty, Gender::Male, &config),
            Err(AstroError::YearOutOfRange { .. })
        ));
    }

    #[test]
    fn thien_ban_fields_follow_section_5_5() {
        // P. 31: "Sinh năm Tý thuộc Dương, an Mệnh tại cung Dần cũng thuộc Dương là thuận lý."
        assert_eq!(YinYangHarmony::of(Giap, Dan), YinYangHarmony::ThuanLy);
        assert_eq!(YinYangHarmony::of(Giap, Mao), YinYangHarmony::NghichLy);
        assert_eq!(YinYangHarmony::of(At, Mao), YinYangHarmony::ThuanLy);
        assert_eq!(YinYangHarmony::of(At, Dan), YinYangHarmony::NghichLy);

        // P. 31: Kim Mệnh with a Thủy bureau, then with a Thổ bureau.
        use FiveElements::{Hoa, Kim, Moc, Tho, Thuy};
        assert_eq!(MenhCucRelation::of(Kim, Thuy), MenhCucRelation::MenhSinhCuc);
        assert_eq!(MenhCucRelation::of(Kim, Tho), MenhCucRelation::CucSinhMenh);
        assert_eq!(MenhCucRelation::of(Kim, Moc), MenhCucRelation::MenhKhacCuc);
        assert_eq!(MenhCucRelation::of(Kim, Hoa), MenhCucRelation::CucKhacMenh);
        assert_eq!(MenhCucRelation::of(Kim, Kim), MenhCucRelation::BinhHoa);
        for (from, to) in [(Kim, Thuy), (Thuy, Moc), (Moc, Hoa), (Hoa, Tho), (Tho, Kim)] {
            assert_eq!(MenhCucRelation::of(from, to), MenhCucRelation::MenhSinhCuc);
            assert_eq!(MenhCucRelation::of(to, from), MenhCucRelation::CucSinhMenh);
        }
        for (from, to) in [(Kim, Moc), (Moc, Tho), (Tho, Thuy), (Thuy, Hoa), (Hoa, Kim)] {
            assert_eq!(MenhCucRelation::of(from, to), MenhCucRelation::MenhKhacCuc);
            assert_eq!(MenhCucRelation::of(to, from), MenhCucRelation::CucKhacMenh);
        }

        // Sách 1: Tang Đố Mộc with Mộc Tam Cục; Sách 2: Tùng Bách Mộc with Thủy Nhị Cục.
        let first = from_lunar(lunar(1913, 9, 23), Dan, Gender::Male);
        assert_eq!(first.menh_cuc, MenhCucRelation::BinhHoa);
        // Quý is yin, Thân is yang.
        assert_eq!(first.harmony, YinYangHarmony::NghichLy);
        let second = from_lunar(lunar(1891, 6, 6), Mao, Gender::Male);
        assert_eq!(second.menh_cuc, MenhCucRelation::CucSinhMenh);
    }

    #[test]
    fn element_relation_is_the_generic_form_of_menh_cuc_relation() {
        use FiveElements::{Hoa, Kim, Moc, Tho, Thuy};
        assert_eq!(ElementRelation::of(Kim, Thuy), ElementRelation::Sinh);
        assert_eq!(ElementRelation::of(Kim, Tho), ElementRelation::DuocSinh);
        assert_eq!(ElementRelation::of(Kim, Moc), ElementRelation::Khac);
        assert_eq!(ElementRelation::of(Kim, Hoa), ElementRelation::BiKhac);
        assert_eq!(ElementRelation::of(Kim, Kim), ElementRelation::BinhHoa);
        for (menh, cuc) in [
            (Kim, Thuy),
            (Kim, Tho),
            (Kim, Moc),
            (Kim, Hoa),
            (Kim, Kim),
            (Moc, Hoa),
        ] {
            let generic = ElementRelation::of(menh, cuc);
            let specific = MenhCucRelation::of(menh, cuc);
            let same_shape = matches!(
                (generic, specific),
                (ElementRelation::Sinh, MenhCucRelation::MenhSinhCuc)
                    | (ElementRelation::DuocSinh, MenhCucRelation::CucSinhMenh)
                    | (ElementRelation::Khac, MenhCucRelation::MenhKhacCuc)
                    | (ElementRelation::BiKhac, MenhCucRelation::CucKhacMenh)
                    | (ElementRelation::BinhHoa, MenhCucRelation::BinhHoa)
            );
            assert!(same_shape, "{menh:?} vs {cuc:?}: {generic:?}, {specific:?}");
        }
    }

    #[test]
    fn birth_time_of_day_splits_the_twelve_hours_in_two_halves_of_six() {
        for hour in [Mao, Thin, Ti, Ngo, Mui, Than] {
            assert_eq!(BirthTimeOfDay::of(hour), BirthTimeOfDay::Day);
        }
        for hour in [Dau, Tuat, Hoi, Ty, Suu, Dan] {
            assert_eq!(BirthTimeOfDay::of(hour), BirthTimeOfDay::Night);
        }
    }

    #[test]
    fn only_the_fourteen_chinh_tinh_have_an_element() {
        // P. 33: Tử Vi is Dương Thổ, p. 39: Hỏa Tinh carries no Ngũ hành of its own.
        assert_eq!(TuViStar::TuVi.element(), Some(FiveElements::Tho));
        assert_eq!(TuViStar::VuKhuc.element(), Some(FiveElements::Kim));
        assert_eq!(TuViStar::PhaQuan.element(), Some(FiveElements::Thuy));
        assert_eq!(TuViStar::HoaTinh.element(), None);
        assert_eq!(TuViStar::LocTon.element(), None);
        for star in TuViStar::ALL {
            assert_eq!(
                star.element().is_some(),
                star.category() == StarCategory::ChinhTinh
            );
        }
    }

    #[test]
    fn menh_chu_and_than_chu_tables_of_section_5_5() {
        let table = [
            (Ty, "Tham Lang", "Hỏa Tinh"),
            (Suu, "Cự Môn", "Thiên Tướng"),
            (Dan, "Lộc Tồn", "Thiên Lương"),
            (Mao, "Văn Khúc", "Thiên Đồng"),
            (Thin, "Liêm Trinh", "Văn Xương"),
            (Ti, "Vũ Khúc", "Thiên Cơ"),
            (Ngo, "Phá Quân", "Hỏa Tinh"),
            (Mui, "Vũ Khúc", "Thiên Tướng"),
            (Than, "Liêm Trinh", "Thiên Lương"),
            (Dau, "Văn Khúc", "Thiên Đồng"),
            (Tuat, "Lộc Tồn", "Văn Xương"),
            (Hoi, "Cự Môn", "Thiên Cơ"),
        ];
        for (branch, menh, than) in table {
            assert_eq!(menh_chu(branch).name_vi(), menh);
            assert_eq!(than_chu(branch).name_vi(), than);
        }
        // Mệnh Chủ follows the Mệnh palace, Thân Chủ the year.
        let chart = from_lunar(lunar(1913, 9, 23), Dan, Gender::Male);
        assert_eq!(chart.menh_chu, menh_chu(Than));
        assert_eq!(chart.than_chu, than_chu(Suu));
    }

    #[test]
    fn pillars_of_the_centre_agree_with_the_four_pillars() {
        let birth = date_time(2020, 6, 10, 10, 30);
        let chart = tuvi_chart(birth, Gender::Male, &TuViConfig::default()).unwrap();
        let options = FourPillarsOptions {
            boundary: YearMonthBoundary::LunarNewYear,
            ..FourPillarsOptions::default()
        };
        let pillars = four_pillars(birth, &options).unwrap();
        assert_eq!(chart.year_can_chi, pillars.year);
        assert_eq!(chart.month_can_chi, pillars.month);
        assert_eq!(chart.day_can_chi, pillars.day);
        assert_eq!(chart.hour_can_chi, pillars.hour);

        // The same birth by lunar date gives the same palaces.
        let by_date = tuvi_chart_from_lunar(chart.lunar, Ti, Gender::Male, &TuViConfig::default());
        let by_date = by_date.unwrap();
        assert_eq!(by_date.palaces, chart.palaces);
        assert_eq!(by_date.day_can_chi, chart.day_can_chi);
        assert_eq!(by_date.hour_can_chi, chart.hour_can_chi);
        assert_eq!(by_date.effective_time, None);
    }

    #[test]
    fn names_and_numbers() {
        assert_eq!(TuViStar::QuanPhuThaiTue.name_vi(), "Quan Phù");
        assert_eq!(TuViStar::QuanPhuBacSy.name_vi(), "Quan Phủ");
        assert_eq!(TuViStar::from(LifeStage::Suy), TuViStar::Suy);
        assert_eq!(
            TuViStar::from(LifeStage::TruongSinh).name_vi(),
            "Trường Sinh"
        );
        for stage in LifeStage::ALL {
            assert_eq!(TuViStar::from(stage).name_vi(), stage.name_vi());
        }
        let numbers = FiveElementsBureau::ALL.map(FiveElementsBureau::number);
        assert_eq!(numbers, [2, 3, 4, 5, 6]);
        for bureau in FiveElementsBureau::ALL {
            assert_eq!(FiveElementsBureau::from_element(bureau.element()), bureau);
        }
        assert_eq!(FiveElementsBureau::HoaLuc.name_vi(), "Hỏa Lục Cục");
        assert_eq!(TuViPalaceRole::HuynhDe.name_vi(), "Huynh Đệ");
        assert_eq!(StarBrightness::Dac.mark(), "Đ");
        assert_eq!(StarBrightness::Ham.name_vi(), "Hãm");
        assert_eq!(TuViPolarity::AmNu.name_vi(), "Âm Nữ");
        assert!(TuViPolarity::AmNu.is_forward());
        assert!(!TuViPolarity::DuongNu.is_forward());
        assert_eq!(YinYangHarmony::NghichLy.name_vi(), "Nghịch lý");
        assert_eq!(MenhCucRelation::BinhHoa.name_vi(), "Bình hòa");
        assert_eq!(StarNature::Hung.name_vi(), "Hung");
        assert_eq!(FourTransformations::HoaKy.name_vi(), "Hóa Kỵ");
    }

    #[test]
    fn default_config_is_the_documented_one() {
        let config = TuViConfig::default();
        assert_eq!(config.tz_offset, VN);
        assert_eq!(config.leap_month_rule, LeapMonthRule::SplitAt15th);
        assert_eq!(config.rat_hour, RatHourConvention::SplitAtMidnight);
        assert!(!config.use_true_solar_time);
        assert_eq!(config.longitude, None);
    }
}
