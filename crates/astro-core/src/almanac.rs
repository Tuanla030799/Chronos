//! Day selection (Trạch Nhật): the Twelve Officers (12 Trực), the 28 Mansions, the twelve yellow
//! and black path gods, 15 almanac gods, Vietnamese taboo days and the age restrictions.
//!
//! The tables come from `docs/BAZI_ALMANAC_SPEC.md`: sections 3.1 to 3.4 from lunar-typescript and
//! section 4 from `docs/vietnam_custom_rules.md`. Lookups in section 3 use the solar month of the
//! day ([`solar_month_branch`]); those in section 4 use the lunar month.

use crate::astronomy::get_sun_longitude_deg;
use crate::canchi::{Branch, CanChi, Stem, day_can_chi, year_can_chi};
use crate::error::AstroError;
use crate::lunar_calendar::{LunarDate, lunar_month_length, solar_to_lunar};
use crate::solar_terms::solar_month_of_longitude;
use crate::time::{SolarDate, local_midnight_jd};

/// Good or bad fortune.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Luck {
    /// Cát
    Cat,
    /// Hung
    Hung,
}

/// Branch of the solar month (tháng tiết khí) of local day `jdn`: the whole day on which a Tiết
/// begins already belongs to the new month.
pub fn solar_month_branch(jdn: i32, tz_offset: f64) -> Branch {
    let longitude = get_sun_longitude_deg(local_midnight_jd(jdn + 1, tz_offset));
    // Solar month 1 is the Dần month.
    Branch::from_index(i64::from(solar_month_of_longitude(longitude)) + 1)
}

/// The Twelve Officers (12 Trực).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Truc {
    /// Kiến 建
    Kien,
    /// Trừ 除
    Tru,
    /// Mãn 满
    Man,
    /// Bình 平
    Binh,
    /// Định 定
    Dinh,
    /// Chấp 执
    Chap,
    /// Phá 破
    Pha,
    /// Nguy 危
    Nguy,
    /// Thành 成
    Thanh,
    /// Thu 收
    Thu,
    /// Khai 开
    Khai,
    /// Bế 闭
    Be,
}

impl Truc {
    /// All officers in cycle order, from Kiến.
    pub const ALL: [Truc; 12] = [
        Truc::Kien,
        Truc::Tru,
        Truc::Man,
        Truc::Binh,
        Truc::Dinh,
        Truc::Chap,
        Truc::Pha,
        Truc::Nguy,
        Truc::Thanh,
        Truc::Thu,
        Truc::Khai,
        Truc::Be,
    ];

    /// Vietnamese name, e.g. "Kiến".
    pub const fn name_vi(self) -> &'static str {
        match self {
            Truc::Kien => "Kiến",
            Truc::Tru => "Trừ",
            Truc::Man => "Mãn",
            Truc::Binh => "Bình",
            Truc::Dinh => "Định",
            Truc::Chap => "Chấp",
            Truc::Pha => "Phá",
            Truc::Nguy => "Nguy",
            Truc::Thanh => "Thành",
            Truc::Thu => "Thu",
            Truc::Khai => "Khai",
            Truc::Be => "Bế",
        }
    }
}

/// Trực of a day: its distance from the branch of the solar month.
pub const fn truc(month_branch: Branch, day_branch: Branch) -> Truc {
    let distance = day_branch.index() as i8 - month_branch.index() as i8;
    Truc::ALL[distance.rem_euclid(12) as usize]
}

/// The 28 Mansions (Nhị Thập Bát Tú), in cycle order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mansion {
    /// Giác Mộc Giao
    GiacMocGiao,
    /// Cang Kim Long
    CangKimLong,
    /// Đê Thổ Lạc
    DeThoLac,
    /// Phòng Nhật Thố
    PhongNhatTho,
    /// Tâm Nguyệt Hồ
    TamNguyetHo,
    /// Vĩ Hỏa Hổ
    ViHoaHo,
    /// Cơ Thủy Báo
    CoThuyBao,
    /// Đẩu Mộc Giải
    DauMocGiai,
    /// Ngưu Kim Ngưu
    NguuKimNguu,
    /// Nữ Thổ Bức
    NuThoBuc,
    /// Hư Nhật Thử
    HuNhatThu,
    /// Nguy Nguyệt Yến
    NguyNguyetYen,
    /// Thất Hỏa Trư
    ThatHoaTru,
    /// Bích Thủy Du
    BichThuyDu,
    /// Khuê Mộc Lang
    KhueMocLang,
    /// Lâu Kim Cẩu
    LauKimCau,
    /// Vị Thổ Trĩ
    ViThoTri,
    /// Mão Nhật Kê
    MaoNhatKe,
    /// Tất Nguyệt Ô
    TatNguyetO,
    /// Chủy Hỏa Hầu
    ChuyHoaHau,
    /// Sâm Thủy Viên
    SamThuyVien,
    /// Tỉnh Mộc Hãn
    TinhMocHan,
    /// Quỷ Kim Dương
    QuyKimDuong,
    /// Liễu Thổ Chương
    LieuThoChuong,
    /// Tinh Nhật Mã
    TinhNhatMa,
    /// Trương Nguyệt Lộc
    TruongNguyetLoc,
    /// Dực Hỏa Xà
    DucHoaXa,
    /// Chẩn Thủy Dẫn
    ChanThuyDan,
}

impl Mansion {
    /// All mansions in cycle order, from Giác.
    pub const ALL: [Mansion; 28] = [
        Mansion::GiacMocGiao,
        Mansion::CangKimLong,
        Mansion::DeThoLac,
        Mansion::PhongNhatTho,
        Mansion::TamNguyetHo,
        Mansion::ViHoaHo,
        Mansion::CoThuyBao,
        Mansion::DauMocGiai,
        Mansion::NguuKimNguu,
        Mansion::NuThoBuc,
        Mansion::HuNhatThu,
        Mansion::NguyNguyetYen,
        Mansion::ThatHoaTru,
        Mansion::BichThuyDu,
        Mansion::KhueMocLang,
        Mansion::LauKimCau,
        Mansion::ViThoTri,
        Mansion::MaoNhatKe,
        Mansion::TatNguyetO,
        Mansion::ChuyHoaHau,
        Mansion::SamThuyVien,
        Mansion::TinhMocHan,
        Mansion::QuyKimDuong,
        Mansion::LieuThoChuong,
        Mansion::TinhNhatMa,
        Mansion::TruongNguyetLoc,
        Mansion::DucHoaXa,
        Mansion::ChanThuyDan,
    ];

    /// Vietnamese name, e.g. "Giác Mộc Giao".
    pub const fn name_vi(self) -> &'static str {
        match self {
            Mansion::GiacMocGiao => "Giác Mộc Giao",
            Mansion::CangKimLong => "Cang Kim Long",
            Mansion::DeThoLac => "Đê Thổ Lạc",
            Mansion::PhongNhatTho => "Phòng Nhật Thố",
            Mansion::TamNguyetHo => "Tâm Nguyệt Hồ",
            Mansion::ViHoaHo => "Vĩ Hỏa Hổ",
            Mansion::CoThuyBao => "Cơ Thủy Báo",
            Mansion::DauMocGiai => "Đẩu Mộc Giải",
            Mansion::NguuKimNguu => "Ngưu Kim Ngưu",
            Mansion::NuThoBuc => "Nữ Thổ Bức",
            Mansion::HuNhatThu => "Hư Nhật Thử",
            Mansion::NguyNguyetYen => "Nguy Nguyệt Yến",
            Mansion::ThatHoaTru => "Thất Hỏa Trư",
            Mansion::BichThuyDu => "Bích Thủy Du",
            Mansion::KhueMocLang => "Khuê Mộc Lang",
            Mansion::LauKimCau => "Lâu Kim Cẩu",
            Mansion::ViThoTri => "Vị Thổ Trĩ",
            Mansion::MaoNhatKe => "Mão Nhật Kê",
            Mansion::TatNguyetO => "Tất Nguyệt Ô",
            Mansion::ChuyHoaHau => "Chủy Hỏa Hầu",
            Mansion::SamThuyVien => "Sâm Thủy Viên",
            Mansion::TinhMocHan => "Tỉnh Mộc Hãn",
            Mansion::QuyKimDuong => "Quỷ Kim Dương",
            Mansion::LieuThoChuong => "Liễu Thổ Chương",
            Mansion::TinhNhatMa => "Tinh Nhật Mã",
            Mansion::TruongNguyetLoc => "Trương Nguyệt Lộc",
            Mansion::DucHoaXa => "Dực Hỏa Xà",
            Mansion::ChanThuyDan => "Chẩn Thủy Dẫn",
        }
    }

    /// Fortune of the mansion.
    pub const fn luck(self) -> Luck {
        match self {
            Mansion::GiacMocGiao
            | Mansion::PhongNhatTho
            | Mansion::ViHoaHo
            | Mansion::CoThuyBao
            | Mansion::DauMocGiai
            | Mansion::ThatHoaTru
            | Mansion::BichThuyDu
            | Mansion::LauKimCau
            | Mansion::ViThoTri
            | Mansion::TatNguyetO
            | Mansion::SamThuyVien
            | Mansion::TinhMocHan
            | Mansion::TruongNguyetLoc
            | Mansion::ChanThuyDan => Luck::Cat,
            _ => Luck::Hung,
        }
    }
}

/// Mansion of the day with Julian Day Number `jdn`. Equivalent to the source's 84-cell table by
/// day branch and weekday.
pub const fn mansion(jdn: i32) -> Mansion {
    Mansion::ALL[(jdn as i64 + 11).rem_euclid(28) as usize]
}

/// The twelve path gods (12 Thần Hoàng Đạo, Hắc Đạo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PathGod {
    /// Thanh Long 青龙
    ThanhLong,
    /// Minh Đường 明堂
    MinhDuong,
    /// Thiên Hình 天刑
    ThienHinh,
    /// Chu Tước 朱雀
    ChuTuoc,
    /// Kim Quỹ 金匮
    KimQuy,
    /// Thiên Đức 天德, also called Bảo Quang; not the almanac god [`AlmanacGod::ThienDuc`].
    ThienDuc,
    /// Bạch Hổ 白虎
    BachHo,
    /// Ngọc Đường 玉堂
    NgocDuong,
    /// Thiên Lao 天牢
    ThienLao,
    /// Huyền Vũ 玄武
    HuyenVu,
    /// Tư Mệnh 司命
    TuMenh,
    /// Câu Trần 勾陈
    CauTran,
}

impl PathGod {
    /// All gods in cycle order, from Thanh Long.
    pub const ALL: [PathGod; 12] = [
        PathGod::ThanhLong,
        PathGod::MinhDuong,
        PathGod::ThienHinh,
        PathGod::ChuTuoc,
        PathGod::KimQuy,
        PathGod::ThienDuc,
        PathGod::BachHo,
        PathGod::NgocDuong,
        PathGod::ThienLao,
        PathGod::HuyenVu,
        PathGod::TuMenh,
        PathGod::CauTran,
    ];

    /// Vietnamese name, e.g. "Thanh Long".
    pub const fn name_vi(self) -> &'static str {
        match self {
            PathGod::ThanhLong => "Thanh Long",
            PathGod::MinhDuong => "Minh Đường",
            PathGod::ThienHinh => "Thiên Hình",
            PathGod::ChuTuoc => "Chu Tước",
            PathGod::KimQuy => "Kim Quỹ",
            PathGod::ThienDuc => "Thiên Đức",
            PathGod::BachHo => "Bạch Hổ",
            PathGod::NgocDuong => "Ngọc Đường",
            PathGod::ThienLao => "Thiên Lao",
            PathGod::HuyenVu => "Huyền Vũ",
            PathGod::TuMenh => "Tư Mệnh",
            PathGod::CauTran => "Câu Trần",
        }
    }

    /// Whether this is a yellow path god (Hoàng Đạo), which is good; the others are black path
    /// gods (Hắc Đạo), which are bad.
    pub const fn is_hoang_dao(self) -> bool {
        matches!(
            self,
            PathGod::ThanhLong
                | PathGod::MinhDuong
                | PathGod::KimQuy
                | PathGod::ThienDuc
                | PathGod::NgocDuong
                | PathGod::TuMenh
        )
    }
}

/// Offset of the path god cycle for each key branch, in branch order (`ZHI_TIAN_SHEN_OFFSET`).
const PATH_GOD_OFFSETS: [usize; 12] = [4, 2, 0, 10, 8, 6, 4, 2, 0, 10, 8, 6];

/// Path god of day `day_branch` in a solar month of branch `month_branch`.
pub const fn day_path_god(month_branch: Branch, day_branch: Branch) -> PathGod {
    path_god(month_branch, day_branch)
}

/// Path god of double-hour `hour_branch` on a day of branch `day_branch`. From 23:00 pass the
/// branch of the next day.
pub const fn hour_path_god(day_branch: Branch, hour_branch: Branch) -> PathGod {
    path_god(day_branch, hour_branch)
}

const fn path_god(key: Branch, target: Branch) -> PathGod {
    PathGod::ALL[(target.index() as usize + PATH_GOD_OFFSETS[key.index() as usize]) % 12]
}

/// The six yellow path double-hours (giờ Hoàng Đạo) of a day of branch `day_branch`.
pub fn hoang_dao_hours(day_branch: Branch) -> Vec<Branch> {
    Branch::ALL
        .into_iter()
        .filter(|&hour| hour_path_god(day_branch, hour).is_hoang_dao())
        .collect()
}

/// The 15 almanac gods decoded from `DAY_SHEN_SHA`: eight good, then seven bad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlmanacGod {
    /// Thiên Đức 天德
    ThienDuc,
    /// Nguyệt Đức 月德
    NguyetDuc,
    /// Thiên Xá 天赦
    ThienXa,
    /// Thiên Nguyện 天愿
    ThienNguyen,
    /// Sinh Khí 生气
    SinhKhi,
    /// Thiên Hỷ 天喜
    ThienHy,
    /// Tam Hợp 三合
    TamHop,
    /// Dịch Mã 驿马
    DichMa,
    /// Nguyệt Phá 月破
    NguyetPha,
    /// Thiên Cương 天罡
    ThienCuong,
    /// Hà Khôi 河魁
    HaKhoi,
    /// Nguyệt Sát 月煞
    NguyetSat,
    /// Kiếp Sát 劫煞
    KiepSat,
    /// Tai Sát 灾煞
    TaiSat,
    /// Nguyệt Hình 月刑
    NguyetHinh,
}

impl AlmanacGod {
    /// All gods, good ones first, in the row order of the specification.
    pub const ALL: [AlmanacGod; 15] = [
        AlmanacGod::ThienDuc,
        AlmanacGod::NguyetDuc,
        AlmanacGod::ThienXa,
        AlmanacGod::ThienNguyen,
        AlmanacGod::SinhKhi,
        AlmanacGod::ThienHy,
        AlmanacGod::TamHop,
        AlmanacGod::DichMa,
        AlmanacGod::NguyetPha,
        AlmanacGod::ThienCuong,
        AlmanacGod::HaKhoi,
        AlmanacGod::NguyetSat,
        AlmanacGod::KiepSat,
        AlmanacGod::TaiSat,
        AlmanacGod::NguyetHinh,
    ];

    /// Vietnamese name, e.g. "Thiên Đức".
    pub const fn name_vi(self) -> &'static str {
        match self {
            AlmanacGod::ThienDuc => "Thiên Đức",
            AlmanacGod::NguyetDuc => "Nguyệt Đức",
            AlmanacGod::ThienXa => "Thiên Xá",
            AlmanacGod::ThienNguyen => "Thiên Nguyện",
            AlmanacGod::SinhKhi => "Sinh Khí",
            AlmanacGod::ThienHy => "Thiên Hỷ",
            AlmanacGod::TamHop => "Tam Hợp",
            AlmanacGod::DichMa => "Dịch Mã",
            AlmanacGod::NguyetPha => "Nguyệt Phá",
            AlmanacGod::ThienCuong => "Thiên Cương",
            AlmanacGod::HaKhoi => "Hà Khôi",
            AlmanacGod::NguyetSat => "Nguyệt Sát",
            AlmanacGod::KiepSat => "Kiếp Sát",
            AlmanacGod::TaiSat => "Tai Sát",
            AlmanacGod::NguyetHinh => "Nguyệt Hình",
        }
    }

    /// Fortune of the god.
    pub const fn luck(self) -> Luck {
        match self {
            AlmanacGod::ThienDuc
            | AlmanacGod::NguyetDuc
            | AlmanacGod::ThienXa
            | AlmanacGod::ThienNguyen
            | AlmanacGod::SinhKhi
            | AlmanacGod::ThienHy
            | AlmanacGod::TamHop
            | AlmanacGod::DichMa => Luck::Cat,
            _ => Luck::Hung,
        }
    }
}

/// Which days of a solar month carry an almanac god.
#[derive(Debug, Clone, Copy)]
enum DayRule {
    Never,
    OnStem(Stem),
    Branches(&'static [Branch]),
    Pair(Stem, Branch),
}

impl DayRule {
    fn matches(self, day: CanChi) -> bool {
        match self {
            DayRule::Never => false,
            DayRule::OnStem(stem) => day.stem == stem,
            DayRule::Branches(branches) => branches.contains(&day.branch),
            DayRule::Pair(stem, branch) => day.stem == stem && day.branch == branch,
        }
    }
}

/// Rule of each god (rows, in [`AlmanacGod::ALL`] order) for each solar month from Dần to Sửu.
const GOD_RULES: [[DayRule; 12]; 15] = {
    use Branch::*;
    use DayRule::{Branches, Never, OnStem, Pair};
    use Stem::{At, Binh, Canh, Dinh, Giap, Ky, Mau, Nham, Quy, Tan};
    [
        // Thiên Đức
        [
            OnStem(Dinh),
            Never,
            OnStem(Nham),
            OnStem(Tan),
            Never,
            OnStem(Giap),
            OnStem(Quy),
            Never,
            OnStem(Binh),
            OnStem(At),
            Never,
            OnStem(Canh),
        ],
        // Nguyệt Đức
        [
            OnStem(Binh),
            OnStem(Giap),
            OnStem(Nham),
            OnStem(Canh),
            OnStem(Binh),
            OnStem(Giap),
            OnStem(Nham),
            OnStem(Canh),
            OnStem(Binh),
            OnStem(Giap),
            OnStem(Nham),
            OnStem(Canh),
        ],
        // Thiên Xá
        [
            Pair(Mau, Dan),
            Pair(Mau, Dan),
            Pair(Mau, Dan),
            Pair(Giap, Ngo),
            Pair(Giap, Ngo),
            Pair(Giap, Ngo),
            Pair(Mau, Than),
            Pair(Mau, Than),
            Pair(Mau, Than),
            Pair(Giap, Ty),
            Pair(Giap, Ty),
            Pair(Giap, Ty),
        ],
        // Thiên Nguyện
        [
            Pair(At, Hoi),
            Pair(Giap, Tuat),
            Pair(At, Dau),
            Pair(Binh, Than),
            Pair(Dinh, Mui),
            Pair(Mau, Ngo),
            Pair(Ky, Ti),
            Pair(Canh, Thin),
            Pair(Tan, Mao),
            Pair(Nham, Dan),
            Pair(Quy, Suu),
            Pair(Giap, Ty),
        ],
        // Sinh Khí
        [
            Branches(&[Ty]),
            Branches(&[Suu]),
            Branches(&[Dan]),
            Branches(&[Mao]),
            Branches(&[Thin]),
            Branches(&[Ti]),
            Branches(&[Ngo]),
            Branches(&[Mui]),
            Branches(&[Than]),
            Branches(&[Dau]),
            Branches(&[Tuat]),
            Branches(&[Hoi]),
        ],
        // Thiên Hỷ
        [
            Branches(&[Tuat]),
            Branches(&[Hoi]),
            Branches(&[Ty]),
            Branches(&[Suu]),
            Branches(&[Dan]),
            Branches(&[Mao]),
            Branches(&[Thin]),
            Branches(&[Ti]),
            Branches(&[Ngo]),
            Branches(&[Mui]),
            Branches(&[Than]),
            Branches(&[Dau]),
        ],
        // Tam Hợp
        [
            Branches(&[Ngo, Tuat]),
            Branches(&[Mui, Hoi]),
            Branches(&[Ty, Than]),
            Branches(&[Suu, Dau]),
            Branches(&[Dan, Tuat]),
            Branches(&[Mao, Hoi]),
            Branches(&[Ty, Thin]),
            Branches(&[Suu, Ti]),
            Branches(&[Dan, Ngo]),
            Branches(&[Mao, Mui]),
            Branches(&[Thin, Than]),
            Branches(&[Ti, Dau]),
        ],
        // Dịch Mã
        [
            Branches(&[Than]),
            Branches(&[Ti]),
            Branches(&[Dan]),
            Branches(&[Hoi]),
            Branches(&[Than]),
            Branches(&[Ti]),
            Branches(&[Dan]),
            Branches(&[Hoi]),
            Branches(&[Than]),
            Branches(&[Ti]),
            Branches(&[Dan]),
            Branches(&[Hoi]),
        ],
        // Nguyệt Phá
        [
            Branches(&[Than]),
            Branches(&[Dau]),
            Branches(&[Tuat]),
            Branches(&[Hoi]),
            Branches(&[Ty]),
            Branches(&[Suu]),
            Branches(&[Dan]),
            Branches(&[Mao]),
            Branches(&[Thin]),
            Branches(&[Ti]),
            Branches(&[Ngo]),
            Branches(&[Mui]),
        ],
        // Thiên Cương
        [
            Branches(&[Ti]),
            Branches(&[Ty]),
            Branches(&[Mui]),
            Branches(&[Dan]),
            Branches(&[Dau]),
            Branches(&[Thin]),
            Branches(&[Hoi]),
            Branches(&[Ngo]),
            Branches(&[Suu]),
            Branches(&[Than]),
            Branches(&[Mao]),
            Branches(&[Tuat]),
        ],
        // Hà Khôi
        [
            Branches(&[Hoi]),
            Branches(&[Ngo]),
            Branches(&[Suu]),
            Branches(&[Than]),
            Branches(&[Mao]),
            Branches(&[Tuat]),
            Branches(&[Ti]),
            Branches(&[Ty]),
            Branches(&[Mui]),
            Branches(&[Dan]),
            Branches(&[Dau]),
            Branches(&[Thin]),
        ],
        // Nguyệt Sát
        [
            Branches(&[Suu]),
            Branches(&[Tuat]),
            Branches(&[Mui]),
            Branches(&[Thin]),
            Branches(&[Suu]),
            Branches(&[Tuat]),
            Branches(&[Mui]),
            Branches(&[Thin]),
            Branches(&[Suu]),
            Branches(&[Tuat]),
            Branches(&[Mui]),
            Branches(&[Thin]),
        ],
        // Kiếp Sát
        [
            Branches(&[Hoi]),
            Branches(&[Than]),
            Branches(&[Ti]),
            Branches(&[Dan]),
            Branches(&[Hoi]),
            Branches(&[Than]),
            Branches(&[Ti]),
            Branches(&[Dan]),
            Branches(&[Hoi]),
            Branches(&[Than]),
            Branches(&[Ti]),
            Branches(&[Dan]),
        ],
        // Tai Sát
        [
            Branches(&[Ty]),
            Branches(&[Dau]),
            Branches(&[Ngo]),
            Branches(&[Mao]),
            Branches(&[Ty]),
            Branches(&[Dau]),
            Branches(&[Ngo]),
            Branches(&[Mao]),
            Branches(&[Ty]),
            Branches(&[Dau]),
            Branches(&[Ngo]),
            Branches(&[Mao]),
        ],
        // Nguyệt Hình
        [
            Branches(&[Ti]),
            Branches(&[Ty]),
            Branches(&[Thin]),
            Branches(&[Than]),
            Branches(&[Ngo]),
            Branches(&[Suu]),
            Branches(&[Dan]),
            Branches(&[Dau]),
            Branches(&[Mui]),
            Branches(&[Hoi]),
            Branches(&[Mao]),
            Branches(&[Tuat]),
        ],
    ]
};

/// Cells of `DAY_SHEN_SHA` that break the rules above, kept so results match the source exactly:
/// god, solar month (0 is Dần), cycle index of the day, and whether the god is added or removed.
const GOD_EXCEPTIONS: [(AlmanacGod, usize, u8, bool); 15] = [
    (AlmanacGod::ThienDuc, 2, 38, false),
    (AlmanacGod::ThienDuc, 6, 18, true),
    (AlmanacGod::ThienDuc, 9, 15, true),
    (AlmanacGod::NguyetDuc, 4, 54, true),
    (AlmanacGod::NguyetDuc, 8, 46, true),
    (AlmanacGod::NguyetDuc, 9, 11, true),
    (AlmanacGod::NguyetDuc, 9, 45, true),
    (AlmanacGod::NguyetDuc, 11, 2, true),
    (AlmanacGod::SinhKhi, 11, 20, true),
    (AlmanacGod::HaKhoi, 9, 25, true),
    (AlmanacGod::KiepSat, 5, 9, true),
    (AlmanacGod::TaiSat, 5, 9, false),
    (AlmanacGod::TaiSat, 6, 54, false),
    (AlmanacGod::NguyetHinh, 7, 45, false),
    (AlmanacGod::NguyetHinh, 10, 3, false),
];

/// Almanac gods of day `day` in a solar month of branch `month_branch`.
pub fn almanac_gods(month_branch: Branch, day: CanChi) -> Vec<AlmanacGod> {
    let month = (i64::from(month_branch.index()) - 2).rem_euclid(12) as usize;
    AlmanacGod::ALL
        .into_iter()
        .enumerate()
        .filter(|&(row, god)| {
            GOD_EXCEPTIONS
                .iter()
                .find(|&&(kind, m, index, _)| {
                    kind == god && m == month && index == day.cycle_index()
                })
                .map_or(GOD_RULES[row][month].matches(day), |exception| exception.3)
        })
        .map(|(_, god)| god)
        .collect()
}

/// A Vietnamese taboo day (section 4 of the specification).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VietnamTaboo {
    /// Tam Nương Sát
    TamNuongSat,
    /// Nguyệt Kỵ
    NguyetKy,
    /// Nguyệt Tận
    NguyetTan,
    /// Dương Công Kỵ Nhật
    DuongCongKyNhat,
    /// Thụ Tử
    ThuTu,
    /// Sát Chủ Dương
    SatChuDuong,
    /// Sát Chủ Âm
    SatChuAm,
    /// Vãng Vong / Lục Bại
    VangVong,
    /// Thiên Địa Chuyển Sát
    ThienDiaChuyenSat,
}

impl VietnamTaboo {
    /// Vietnamese name, e.g. "Tam Nương Sát".
    pub const fn name_vi(self) -> &'static str {
        match self {
            VietnamTaboo::TamNuongSat => "Tam Nương Sát",
            VietnamTaboo::NguyetKy => "Nguyệt Kỵ",
            VietnamTaboo::NguyetTan => "Nguyệt Tận",
            VietnamTaboo::DuongCongKyNhat => "Dương Công Kỵ Nhật",
            VietnamTaboo::ThuTu => "Thụ Tử",
            VietnamTaboo::SatChuDuong => "Sát Chủ Dương",
            VietnamTaboo::SatChuAm => "Sát Chủ Âm",
            VietnamTaboo::VangVong => "Vãng Vong",
            VietnamTaboo::ThienDiaChuyenSat => "Thiên Địa Chuyển Sát",
        }
    }
}

const TAM_NUONG_DAYS: [u8; 6] = [3, 7, 13, 18, 22, 27];
const NGUYET_KY_DAYS: [u8; 3] = [5, 14, 23];

/// Dương Công Kỵ Nhật: days of each lunar month from 1 to 12.
const DUONG_CONG_DAYS: [&[u8]; 12] = [
    &[13],
    &[11],
    &[9],
    &[7],
    &[5],
    &[3],
    &[8, 29],
    &[27],
    &[25],
    &[23],
    &[21],
    &[19],
];

/// Day branches of the taboos of section 4.3, for lunar months 1 to 12.
const THU_TU: [Branch; 12] = {
    use Branch::*;
    [Tuat, Thin, Hoi, Ti, Ty, Ngo, Suu, Mui, Dan, Than, Mao, Dau]
};
const SAT_CHU_DUONG: [Branch; 12] = {
    use Branch::*;
    [
        Ty, Suu, Suu, Tuat, Thin, Thin, Suu, Thin, Suu, Thin, Mui, Thin,
    ]
};
const SAT_CHU_AM: [Branch; 12] = {
    use Branch::*;
    [Ti, Ty, Mui, Mao, Than, Tuat, Hoi, Suu, Ngo, Dau, Dan, Thin]
};
/// Months 9 and 10 follow `docs/vietnam_custom_rules.md`, as in common Vietnamese almanacs;
/// 6tail and the Hiệp Kỷ have them the other way round (month 9 Thìn, month 10 Mùi).
const VANG_VONG: [Branch; 12] = {
    use Branch::*;
    [Dan, Ti, Than, Hoi, Mao, Ngo, Dau, Ty, Mui, Thin, Tuat, Suu]
};
const THIEN_DIA_CHUYEN_SAT: [Branch; 12] = {
    use Branch::*;
    [Mao, Mao, Mao, Ngo, Ngo, Ngo, Dau, Dau, Dau, Ty, Ty, Ty]
};

/// Vietnamese taboos of a lunar day. `month_length` is the length of its lunar month and
/// `day_branch` the branch of the day. A leap month counts as the month it repeats.
pub fn vietnam_taboos(lunar: LunarDate, month_length: u8, day_branch: Branch) -> Vec<VietnamTaboo> {
    let mut taboos = Vec::new();
    if TAM_NUONG_DAYS.contains(&lunar.day) {
        taboos.push(VietnamTaboo::TamNuongSat);
    }
    if NGUYET_KY_DAYS.contains(&lunar.day) {
        taboos.push(VietnamTaboo::NguyetKy);
    }
    if lunar.day == month_length {
        taboos.push(VietnamTaboo::NguyetTan);
    }
    let Some(month) = (lunar.month as usize).checked_sub(1).filter(|&m| m < 12) else {
        return taboos;
    };
    if DUONG_CONG_DAYS[month].contains(&lunar.day) {
        taboos.push(VietnamTaboo::DuongCongKyNhat);
    }
    for (branches, taboo) in [
        (&THU_TU, VietnamTaboo::ThuTu),
        (&SAT_CHU_DUONG, VietnamTaboo::SatChuDuong),
        (&SAT_CHU_AM, VietnamTaboo::SatChuAm),
        (&VANG_VONG, VietnamTaboo::VangVong),
        (&THIEN_DIA_CHUYEN_SAT, VietnamTaboo::ThienDiaChuyenSat),
    ] {
        if branches[month] == day_branch {
            taboos.push(taboo);
        }
    }
    taboos
}

/// Everything day selection shows for one Gregorian date.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct AlmanacDay {
    /// The Gregorian date.
    pub solar: SolarDate,
    /// The same day in the lunar calendar.
    pub lunar: LunarDate,
    /// Can Chi of the day.
    pub day_can_chi: CanChi,
    /// Branch of the solar month.
    pub month_branch: Branch,
    /// Trực of the day.
    pub truc: Truc,
    /// Mansion of the day.
    pub mansion: Mansion,
    /// Path god of the day.
    pub path_god: PathGod,
    /// The six yellow path double-hours.
    pub hoang_dao_hours: Vec<Branch>,
    /// Almanac gods of the day.
    pub gods: Vec<AlmanacGod>,
    /// Vietnamese taboos of the day.
    pub taboos: Vec<VietnamTaboo>,
}

/// Day selection data for `date` in a zone `tz_offset` hours east of UTC.
pub fn almanac_day(date: SolarDate, tz_offset: f64) -> Result<AlmanacDay, AstroError> {
    let lunar = solar_to_lunar(date.day, date.month, date.year, tz_offset)?;
    let month_length = lunar_month_length(lunar, tz_offset)?;
    let jdn = date.to_jdn();
    let day = day_can_chi(jdn);
    let month_branch = solar_month_branch(jdn, tz_offset);
    Ok(AlmanacDay {
        solar: date,
        lunar,
        day_can_chi: day,
        month_branch,
        truc: truc(month_branch, day.branch),
        mansion: mansion(jdn),
        path_god: day_path_god(month_branch, day.branch),
        hoang_dao_hours: hoang_dao_hours(day.branch),
        gods: almanac_gods(month_branch, day),
        taboos: vietnam_taboos(lunar, month_length, day.branch),
    })
}

/// Age in lunar reckoning (tuổi mụ): the birth year counts as 1.
pub const fn lunar_age(birth_lunar_year: i32, target_lunar_year: i32) -> i32 {
    target_lunar_year - birth_lunar_year + 1
}

/// The three years of Tam Tai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TamTai {
    /// Năm đầu, Nhập
    Nhap,
    /// Năm giữa, Cư, nặng nhất
    Cu,
    /// Năm cuối, Xuất
    Xuat,
}

impl TamTai {
    /// Vietnamese name, e.g. "Nhập".
    pub const fn name_vi(self) -> &'static str {
        match self {
            TamTai::Nhap => "Nhập",
            TamTai::Cu => "Cư",
            TamTai::Xuat => "Xuất",
        }
    }
}

/// Tam Tai years by Tam Hợp group of the birth branch, in the row order of the specification.
const TAM_TAI_YEARS: [[Branch; 3]; 4] = {
    use Branch::*;
    [
        [Dan, Mao, Thin],
        [Than, Dau, Tuat],
        [Hoi, Ty, Suu],
        [Ti, Ngo, Mui],
    ]
};

/// Tam Tai of a person born in a year of `birth_branch`, in a year of `target_branch`.
pub fn tam_tai(birth_branch: Branch, target_branch: Branch) -> Option<TamTai> {
    let years = TAM_TAI_YEARS[birth_branch.trine_group()];
    [TamTai::Nhap, TamTai::Cu, TamTai::Xuat]
        .into_iter()
        .zip(years)
        .find(|&(_, year)| year == target_branch)
        .map(|(phase, _)| phase)
}

/// The four Kim Lâu restrictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KimLau {
    /// Kim Lâu Thân: kỵ bản thân người đứng tuổi
    Than,
    /// Kim Lâu Thê: gây họa cho người phối ngẫu
    The,
    /// Kim Lâu Tử: gây họa cho con cái
    Tu,
    /// Kim Lâu Súc: hại vật nuôi, ảnh hưởng kinh tế gia đình
    Suc,
}

impl KimLau {
    /// Vietnamese name, e.g. "Kim Lâu Thân".
    pub const fn name_vi(self) -> &'static str {
        match self {
            KimLau::Than => "Kim Lâu Thân",
            KimLau::The => "Kim Lâu Thê",
            KimLau::Tu => "Kim Lâu Tử",
            KimLau::Suc => "Kim Lâu Súc",
        }
    }
}

/// Kim Lâu of lunar age `lunar_age`, or `None` when it does not apply.
pub const fn kim_lau(lunar_age: i32) -> Option<KimLau> {
    match lunar_age.rem_euclid(9) {
        1 => Some(KimLau::Than),
        3 => Some(KimLau::The),
        6 => Some(KimLau::Tu),
        8 => Some(KimLau::Suc),
        _ => None,
    }
}

/// The six palaces of Hoang Ốc, from `hoang_oc_index` 1 to 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HoangOc {
    /// Nhất Cát
    NhatCat,
    /// Nhì Nghi
    NhiNghi,
    /// Tam Địa Sát
    TamDiaSat,
    /// Tứ Tấn Tài
    TuTanTai,
    /// Ngũ Thọ Tử
    NguThoTu,
    /// Lục Hoang Ốc
    LucHoangOc,
}

impl HoangOc {
    /// Vietnamese name, e.g. "Nhất Cát".
    pub const fn name_vi(self) -> &'static str {
        match self {
            HoangOc::NhatCat => "Nhất Cát",
            HoangOc::NhiNghi => "Nhì Nghi",
            HoangOc::TamDiaSat => "Tam Địa Sát",
            HoangOc::TuTanTai => "Tứ Tấn Tài",
            HoangOc::NguThoTu => "Ngũ Thọ Tử",
            HoangOc::LucHoangOc => "Lục Hoang Ốc",
        }
    }

    /// Fortune of the palace.
    pub const fn luck(self) -> Luck {
        match self {
            HoangOc::NhatCat | HoangOc::NhiNghi | HoangOc::TuTanTai => Luck::Cat,
            HoangOc::TamDiaSat | HoangOc::NguThoTu | HoangOc::LucHoangOc => Luck::Hung,
        }
    }
}

/// Hoang Ốc palace of lunar age `lunar_age`; the rule applies from age 10.
pub const fn hoang_oc(lunar_age: i32) -> Option<HoangOc> {
    if lunar_age < 10 {
        return None;
    }
    let (tens, units) = (lunar_age / 10, lunar_age % 10);
    Some(match (tens + units - 1) % 6 {
        0 => HoangOc::NhatCat,
        1 => HoangOc::NhiNghi,
        2 => HoangOc::TamDiaSat,
        3 => HoangOc::TuTanTai,
        4 => HoangOc::NguThoTu,
        _ => HoangOc::LucHoangOc,
    })
}

/// Age restrictions of a person in a target year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct AgeRestrictions {
    /// Age in lunar reckoning.
    pub lunar_age: i32,
    /// Tam Tai of the target year, if any.
    pub tam_tai: Option<TamTai>,
    /// Kim Lâu of the age, if any.
    pub kim_lau: Option<KimLau>,
    /// Hoang Ốc palace of the age; `None` below age 10.
    pub hoang_oc: Option<HoangOc>,
}

/// Tam Tai, Kim Lâu and Hoang Ốc of a person born in lunar year `birth_lunar_year`, in lunar year
/// `target_lunar_year`.
pub fn age_restrictions(birth_lunar_year: i32, target_lunar_year: i32) -> AgeRestrictions {
    let age = lunar_age(birth_lunar_year, target_lunar_year);
    AgeRestrictions {
        lunar_age: age,
        tam_tai: tam_tai(
            year_can_chi(birth_lunar_year).branch,
            year_can_chi(target_lunar_year).branch,
        ),
        kim_lau: kim_lau(age),
        hoang_oc: hoang_oc(age),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canchi::Branch::{Dan, Dau, Hoi, Mao, Mui, Ngo, Suu, Than, Thin, Ti, Tuat, Ty};
    use crate::test_support::{VN, date};

    /// Days of each god by solar month, one bit per cycle index, decoded directly from
    /// `DAY_SHEN_SHA` of lunar-typescript.
    const SOURCE_DAYS: [[u64; 12]; 15] = [
        [
            0x020080200802008,
            0x000000000000000,
            0x401000010040100,
            0x200802008020080,
            0x000000000000000,
            0x004010040100401,
            0x8020080200c0200,
            0x000000000000000,
            0x010040100401004,
            0x008020080208802,
            0x000000000000000,
            0x100401004010040,
        ],
        [
            0x010040100401004,
            0x004010040100401,
            0x401004010040100,
            0x100401004010040,
            0x050040100401004,
            0x004010040100401,
            0x401004010040100,
            0x100401004010040,
            0x010440100401004,
            0x004210040100c01,
            0x401004010040100,
            0x100401004010044,
        ],
        [
            0x000000000004000,
            0x000000000004000,
            0x000000000004000,
            0x000000040000000,
            0x000000040000000,
            0x000000040000000,
            0x000100000000000,
            0x000100000000000,
            0x000100000000000,
            0x000000000000001,
            0x000000000000001,
            0x000000000000001,
        ],
        [
            0x000000000000800,
            0x000000000000400,
            0x000000000200000,
            0x000000100000000,
            0x000080000000000,
            0x040000000000000,
            0x000000000000020,
            0x000000000010000,
            0x000000008000000,
            0x000004000000000,
            0x002000000000000,
            0x000000000000001,
        ],
        [
            0x001001001001001,
            0x002002002002002,
            0x004004004004004,
            0x008008008008008,
            0x010010010010010,
            0x020020020020020,
            0x040040040040040,
            0x080080080080080,
            0x100100100100100,
            0x200200200200200,
            0x400400400400400,
            0x800800800900800,
        ],
        [
            0x400400400400400,
            0x800800800800800,
            0x001001001001001,
            0x002002002002002,
            0x004004004004004,
            0x008008008008008,
            0x010010010010010,
            0x020020020020020,
            0x040040040040040,
            0x080080080080080,
            0x100100100100100,
            0x200200200200200,
        ],
        [
            0x440440440440440,
            0x880880880880880,
            0x101101101101101,
            0x202202202202202,
            0x404404404404404,
            0x808808808808808,
            0x011011011011011,
            0x022022022022022,
            0x044044044044044,
            0x088088088088088,
            0x110110110110110,
            0x220220220220220,
        ],
        [
            0x100100100100100,
            0x020020020020020,
            0x004004004004004,
            0x800800800800800,
            0x100100100100100,
            0x020020020020020,
            0x004004004004004,
            0x800800800800800,
            0x100100100100100,
            0x020020020020020,
            0x004004004004004,
            0x800800800800800,
        ],
        [
            0x100100100100100,
            0x200200200200200,
            0x400400400400400,
            0x800800800800800,
            0x001001001001001,
            0x002002002002002,
            0x004004004004004,
            0x008008008008008,
            0x010010010010010,
            0x020020020020020,
            0x040040040040040,
            0x080080080080080,
        ],
        [
            0x020020020020020,
            0x001001001001001,
            0x080080080080080,
            0x004004004004004,
            0x200200200200200,
            0x010010010010010,
            0x800800800800800,
            0x040040040040040,
            0x002002002002002,
            0x100100100100100,
            0x008008008008008,
            0x400400400400400,
        ],
        [
            0x800800800800800,
            0x040040040040040,
            0x002002002002002,
            0x100100100100100,
            0x008008008008008,
            0x400400400400400,
            0x020020020020020,
            0x001001001001001,
            0x080080080080080,
            0x004004006004004,
            0x200200200200200,
            0x010010010010010,
        ],
        [
            0x002002002002002,
            0x400400400400400,
            0x080080080080080,
            0x010010010010010,
            0x002002002002002,
            0x400400400400400,
            0x080080080080080,
            0x010010010010010,
            0x002002002002002,
            0x400400400400400,
            0x080080080080080,
            0x010010010010010,
        ],
        [
            0x800800800800800,
            0x100100100100100,
            0x020020020020020,
            0x004004004004004,
            0x800800800800800,
            0x100100100100300,
            0x020020020020020,
            0x004004004004004,
            0x800800800800800,
            0x100100100100100,
            0x020020020020020,
            0x004004004004004,
        ],
        [
            0x001001001001001,
            0x200200200200200,
            0x040040040040040,
            0x008008008008008,
            0x001001001001001,
            0x200200200200000,
            0x000040040040040,
            0x008008008008008,
            0x001001001001001,
            0x200200200200200,
            0x040040040040040,
            0x008008008008008,
        ],
        [
            0x020020020020020,
            0x001001001001001,
            0x010010010010010,
            0x100100100100100,
            0x040040040040040,
            0x002002002002002,
            0x004004004004004,
            0x200000200200200,
            0x080080080080080,
            0x800800800800800,
            0x008008008008000,
            0x400400400400400,
        ],
    ];

    #[test]
    fn gods_match_the_source_in_every_cell() {
        for (god, months) in AlmanacGod::ALL.into_iter().zip(SOURCE_DAYS) {
            for (month, days) in months.into_iter().enumerate() {
                let month_branch = Branch::from_index(month as i64 + 2);
                for index in 0..60 {
                    let day = CanChi::from_cycle_index(index);
                    let expected = days >> index & 1 == 1;
                    let found = almanac_gods(month_branch, day).contains(&god);
                    assert_eq!(found, expected, "{god:?} {month} {day}");
                }
            }
        }
    }

    #[test]
    fn gods_good_and_bad() {
        assert_eq!(AlmanacGod::ThienXa.luck(), Luck::Cat);
        assert_eq!(AlmanacGod::DichMa.luck(), Luck::Cat);
        assert_eq!(AlmanacGod::NguyetPha.luck(), Luck::Hung);
        assert_eq!(AlmanacGod::ThienDuc.name_vi(), "Thiên Đức");
    }

    #[test]
    fn truc_counts_from_the_month_branch() {
        assert_eq!(truc(Dan, Dan), Truc::Kien);
        assert_eq!(truc(Dan, Thin), Truc::Man);
        assert_eq!(truc(Dan, Ty), Truc::Khai);
        assert_eq!(truc(Ty, Hoi), Truc::Be);
        assert_eq!(truc(Mui, Mui), Truc::Kien);
    }

    #[test]
    fn solar_month_changes_on_the_day_of_the_tiet() {
        // Lập Xuân 2024 fell at 15:27 on 4 February in Vietnam: the whole day is the Dần month.
        assert_eq!(solar_month_branch(date(2024, 2, 3).to_jdn(), VN), Suu);
        assert_eq!(solar_month_branch(date(2024, 2, 4).to_jdn(), VN), Dan);
        assert_eq!(solar_month_branch(date(2024, 3, 4).to_jdn(), VN), Dan);
        assert_eq!(solar_month_branch(date(2024, 3, 5).to_jdn(), VN), Mao);
    }

    #[test]
    fn mansion_cycle() {
        assert_eq!(mansion(2_460_351), Mansion::DeThoLac);
        assert_eq!(mansion(2_460_351 + 28), Mansion::DeThoLac);
        assert_eq!(mansion(2_460_349), Mansion::GiacMocGiao);
        assert_eq!(Mansion::GiacMocGiao.luck(), Luck::Cat);
        assert_eq!(Mansion::CangKimLong.luck(), Luck::Hung);
        assert_eq!(Mansion::ViThoTri.name_vi(), "Vị Thổ Trĩ");
    }

    #[test]
    fn path_gods() {
        // On a Tý day Thanh Long rules the Thân hour.
        assert_eq!(hour_path_god(Ty, Than), PathGod::ThanhLong);
        assert_eq!(day_path_god(Dan, Ty), PathGod::ThanhLong);
        assert!(PathGod::KimQuy.is_hoang_dao());
        assert!(!PathGod::HuyenVu.is_hoang_dao());
    }

    #[test]
    fn hoang_dao_hours_by_day_branch() {
        assert_eq!(hoang_dao_hours(Ty), vec![Ty, Suu, Mao, Ngo, Than, Dau]);
        assert_eq!(hoang_dao_hours(Ngo), hoang_dao_hours(Ty));
        for branch in Branch::ALL {
            assert_eq!(hoang_dao_hours(branch).len(), 6);
        }
    }

    fn lunar(month: u8, day: u8, is_leap_month: bool) -> LunarDate {
        LunarDate {
            year: 2025,
            month,
            day,
            is_leap_month,
        }
    }

    #[test]
    fn fixed_lunar_day_taboos() {
        assert_eq!(
            vietnam_taboos(lunar(1, 3, false), 30, Ngo),
            vec![VietnamTaboo::TamNuongSat]
        );
        assert_eq!(
            vietnam_taboos(lunar(1, 14, false), 30, Ngo),
            vec![VietnamTaboo::NguyetKy]
        );
        assert_eq!(
            vietnam_taboos(lunar(4, 29, false), 29, Suu),
            vec![VietnamTaboo::NguyetTan]
        );
        assert!(vietnam_taboos(lunar(4, 29, false), 30, Suu).is_empty());
        assert_eq!(
            vietnam_taboos(lunar(4, 30, false), 30, Suu),
            vec![VietnamTaboo::NguyetTan]
        );
    }

    #[test]
    fn duong_cong_applies_to_leap_months_too() {
        for leap in [false, true] {
            assert!(
                vietnam_taboos(lunar(6, 3, leap), 30, Ngo).contains(&VietnamTaboo::DuongCongKyNhat)
            );
            assert!(
                vietnam_taboos(lunar(7, 29, leap), 30, Ngo)
                    .contains(&VietnamTaboo::DuongCongKyNhat)
            );
            assert!(
                vietnam_taboos(lunar(7, 8, leap), 30, Ngo).contains(&VietnamTaboo::DuongCongKyNhat)
            );
        }
        assert!(
            !vietnam_taboos(lunar(7, 9, false), 30, Ngo).contains(&VietnamTaboo::DuongCongKyNhat)
        );
    }

    #[test]
    fn branch_taboos_follow_the_lunar_month() {
        let has = |month, branch, taboo| {
            vietnam_taboos(lunar(month, 2, false), 30, branch).contains(&taboo)
        };
        assert!(has(1, Tuat, VietnamTaboo::ThuTu));
        assert!(has(12, Dau, VietnamTaboo::ThuTu));
        assert!(has(4, Tuat, VietnamTaboo::SatChuDuong));
        assert!(has(11, Mui, VietnamTaboo::SatChuDuong));
        assert!(has(1, Ti, VietnamTaboo::SatChuAm));
        assert!(has(12, Thin, VietnamTaboo::SatChuAm));
        assert!(has(1, Dan, VietnamTaboo::VangVong));
        assert!(has(6, Ngo, VietnamTaboo::VangVong));
        assert!(has(3, Mao, VietnamTaboo::ThienDiaChuyenSat));
        assert!(has(12, Ty, VietnamTaboo::ThienDiaChuyenSat));
    }

    #[test]
    fn vang_vong_keeps_the_vietnamese_order_in_months_9_and_10() {
        let has = |month, branch| {
            vietnam_taboos(lunar(month, 2, false), 30, branch).contains(&VietnamTaboo::VangVong)
        };
        assert!(has(9, Mui) && has(10, Thin));
        assert!(!has(9, Thin) && !has(10, Mui));
    }

    #[test]
    fn leap_months_use_the_branch_table_of_the_month_they_repeat() {
        let taboos = vietnam_taboos(lunar(6, 2, true), 29, Ngo);
        assert!(taboos.contains(&VietnamTaboo::ThuTu));
    }

    #[test]
    fn test_vector_10_february_2024() {
        let day = almanac_day(date(2024, 2, 10), VN).unwrap();
        assert_eq!(day.solar.to_jdn(), 2_460_351);
        assert_eq!(day.day_can_chi.to_string(), "Giáp Thìn");
        assert_eq!(day.day_can_chi.na_yin().name_vi(), "Phú Đăng Hỏa");
        assert_eq!((day.lunar.day, day.lunar.month), (1, 1));
        assert_eq!(day.month_branch, Dan);
        assert_eq!(day.truc, Truc::Man);
        assert_eq!(day.mansion, Mansion::DeThoLac);
        assert_eq!(day.path_god, PathGod::KimQuy);
        assert!(day.path_god.is_hoang_dao());
        assert!(day.taboos.is_empty());
    }

    #[test]
    fn test_vector_16_february_2024() {
        let day = almanac_day(date(2024, 2, 16), VN).unwrap();
        assert_eq!(day.solar.to_jdn(), 2_460_357);
        assert_eq!(day.day_can_chi.to_string(), "Canh Tuất");
        assert_eq!(day.day_can_chi.na_yin().name_vi(), "Thoa Xuyến Kim");
        assert_eq!((day.lunar.day, day.lunar.month), (7, 1));
        assert_eq!(day.month_branch, Dan);
        assert_eq!(day.truc, Truc::Thanh);
        assert_eq!(day.mansion, Mansion::NguuKimNguu);
        assert_eq!(day.path_god, PathGod::TuMenh);
        assert_eq!(
            day.taboos,
            vec![VietnamTaboo::TamNuongSat, VietnamTaboo::ThuTu]
        );
    }

    #[test]
    fn test_vector_25_july_2025() {
        let day = almanac_day(date(2025, 7, 25), VN).unwrap();
        assert_eq!(day.solar.to_jdn(), 2_460_882);
        assert_eq!(day.day_can_chi.to_string(), "Ất Mùi");
        assert_eq!(day.day_can_chi.na_yin().name_vi(), "Sa Trung Kim");
        assert_eq!(
            (day.lunar.day, day.lunar.month, day.lunar.is_leap_month),
            (1, 6, true)
        );
        assert_eq!(day.month_branch, Mui);
        assert_eq!(day.truc, Truc::Kien);
        assert_eq!(day.mansion, Mansion::CangKimLong);
        assert_eq!(day.path_god, PathGod::HuyenVu);
        assert!(!day.path_god.is_hoang_dao());
        assert!(day.taboos.is_empty());
    }

    #[test]
    fn nguyet_tan_uses_the_real_month_length() {
        // 9 February 2024 was the 30th of a full month; 28 January 2025 the 29th of a short one.
        for (solar, lunar_day) in [(date(2024, 2, 9), 30), (date(2025, 1, 28), 29)] {
            let day = almanac_day(solar, VN).unwrap();
            assert_eq!(day.lunar.day, lunar_day);
            assert!(day.taboos.contains(&VietnamTaboo::NguyetTan));
        }
    }

    #[test]
    fn almanac_day_rejects_invalid_input() {
        assert!(
            almanac_day(
                SolarDate {
                    year: 2024,
                    month: 2,
                    day: 30
                },
                VN
            )
            .is_err()
        );
        assert!(almanac_day(date(2024, 2, 10), 99.0).is_err());
    }

    #[test]
    fn lunar_age_of_the_specification_example() {
        assert_eq!(lunar_age(1990, 2026), 37);
    }

    #[test]
    fn tam_tai_by_birth_group() {
        assert_eq!(tam_tai(Ty, Dan), Some(TamTai::Nhap));
        assert_eq!(tam_tai(Than, Mao), Some(TamTai::Cu));
        assert_eq!(tam_tai(Thin, Thin), Some(TamTai::Xuat));
        assert_eq!(tam_tai(Ngo, Than), Some(TamTai::Nhap));
        assert_eq!(tam_tai(Dau, Ty), Some(TamTai::Cu));
        assert_eq!(tam_tai(Mao, Mui), Some(TamTai::Xuat));
        assert_eq!(tam_tai(Hoi, Ngo), Some(TamTai::Cu));
        assert_eq!(tam_tai(Suu, Hoi), Some(TamTai::Nhap));
        assert_eq!(tam_tai(Ty, Ngo), None);
    }

    #[test]
    fn kim_lau_by_remainder() {
        assert_eq!(kim_lau(37), Some(KimLau::Than));
        assert_eq!(kim_lau(30), Some(KimLau::The));
        assert_eq!(kim_lau(33), Some(KimLau::Tu));
        assert_eq!(kim_lau(26), Some(KimLau::Suc));
        for age in [27, 29, 31, 32, 34] {
            assert_eq!(kim_lau(age), None, "{age}");
        }
    }

    #[test]
    fn hoang_oc_examples_of_the_specification() {
        assert_eq!(hoang_oc(10), Some(HoangOc::NhatCat));
        assert_eq!(hoang_oc(20), Some(HoangOc::NhiNghi));
        assert_eq!(hoang_oc(37), Some(HoangOc::TuTanTai));
        assert_eq!(hoang_oc(30), Some(HoangOc::TamDiaSat));
        assert_eq!(hoang_oc(9), None);
        assert_eq!(HoangOc::TuTanTai.luck(), Luck::Cat);
        assert_eq!(HoangOc::NguThoTu.luck(), Luck::Hung);
    }

    #[test]
    fn age_restrictions_of_the_specification_example() {
        // Born 1990 (Canh Ngọ), looking at 2026 (Bính Ngọ).
        assert_eq!(
            age_restrictions(1990, 2026),
            AgeRestrictions {
                lunar_age: 37,
                tam_tai: None,
                kim_lau: Some(KimLau::Than),
                hoang_oc: Some(HoangOc::TuTanTai),
            }
        );
    }
}
