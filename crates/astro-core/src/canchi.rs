//! The sexagenary cycle (Can Chi 干支): ten Heavenly Stems paired with twelve Earthly
//! Branches.
//!
//! Formulas from Hồ Ngọc Đức's lunar calendar algorithm:
//! - the day with Julian Day Number N has stem (N + 9) mod 10 and branch (N + 1) mod 12;
//! - lunar year Y has stem (Y + 6) mod 10 and branch (Y + 8) mod 12;
//! - lunar month M of year Y has stem (12·Y + M + 3) mod 10 and branch (M + 1) mod 12, and a
//!   leap month carries the Can Chi of the month it repeats;
//! - the Tý hour of a day whose stem has index s has stem 2·s mod 10, and each following
//!   double-hour advances stem and branch by one.
//!
//! Elements, Nạp Âm and the relations between stems and branches follow `LunarUtil` of
//! lunar-typescript, as tabulated in `docs/BAZI_ALMANAC_SPEC.md` (sections 1 and 2.2).

use core::fmt;

/// The Five Elements (Ngũ Hành).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FiveElements {
    /// Mộc 木
    Moc,
    /// Hỏa 火
    Hoa,
    /// Thổ 土
    Tho,
    /// Kim 金
    Kim,
    /// Thủy 水
    Thuy,
}

impl FiveElements {
    /// Vietnamese name, e.g. "Mộc".
    pub const fn name_vi(self) -> &'static str {
        match self {
            FiveElements::Moc => "Mộc",
            FiveElements::Hoa => "Hỏa",
            FiveElements::Tho => "Thổ",
            FiveElements::Kim => "Kim",
            FiveElements::Thuy => "Thủy",
        }
    }
}

/// Yin and yang polarity (Âm Dương).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum YinYang {
    /// Dương 陽
    Duong,
    /// Âm 陰
    Am,
}

impl YinYang {
    /// Vietnamese name, "Dương" or "Âm".
    pub const fn name_vi(self) -> &'static str {
        match self {
            YinYang::Duong => "Dương",
            YinYang::Am => "Âm",
        }
    }
}

/// The ten Heavenly Stems (Thiên Can).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stem {
    /// Giáp 甲
    Giap,
    /// Ất 乙
    At,
    /// Bính 丙
    Binh,
    /// Đinh 丁
    Dinh,
    /// Mậu 戊
    Mau,
    /// Kỷ 己
    Ky,
    /// Canh 庚
    Canh,
    /// Tân 辛
    Tan,
    /// Nhâm 壬
    Nham,
    /// Quý 癸
    Quy,
}

impl Stem {
    /// All stems in cycle order, from Giáp.
    pub const ALL: [Stem; 10] = [
        Stem::Giap,
        Stem::At,
        Stem::Binh,
        Stem::Dinh,
        Stem::Mau,
        Stem::Ky,
        Stem::Canh,
        Stem::Tan,
        Stem::Nham,
        Stem::Quy,
    ];

    /// Position in the cycle, from 0 (Giáp) to 9 (Quý).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The stem at cycle position `index`, wrapping modulo 10.
    pub const fn from_index(index: i64) -> Self {
        Self::ALL[index.rem_euclid(10) as usize]
    }

    /// Element of the stem.
    pub const fn element(self) -> FiveElements {
        match self {
            Stem::Giap | Stem::At => FiveElements::Moc,
            Stem::Binh | Stem::Dinh => FiveElements::Hoa,
            Stem::Mau | Stem::Ky => FiveElements::Tho,
            Stem::Canh | Stem::Tan => FiveElements::Kim,
            Stem::Nham | Stem::Quy => FiveElements::Thuy,
        }
    }

    /// Polarity of the stem: Giáp, Bính, Mậu, Canh and Nhâm are yang.
    pub const fn yin_yang(self) -> YinYang {
        if self.index() % 2 == 0 {
            YinYang::Duong
        } else {
            YinYang::Am
        }
    }

    /// Partner in the Five Combinations (Ngũ Hợp): Giáp–Kỷ, Ất–Canh, Bính–Tân, Đinh–Nhâm,
    /// Mậu–Quý.
    pub const fn combination(self) -> Stem {
        Self::from_index(self.index() as i64 + 5)
    }

    /// Stem that clashes with this one (Xung): Giáp–Canh, Ất–Tân, Bính–Nhâm, Đinh–Quý. Mậu and
    /// Kỷ have none.
    pub const fn clash(self) -> Option<Stem> {
        match self {
            Stem::Giap => Some(Stem::Canh),
            Stem::At => Some(Stem::Tan),
            Stem::Binh => Some(Stem::Nham),
            Stem::Dinh => Some(Stem::Quy),
            Stem::Mau | Stem::Ky => None,
            Stem::Canh => Some(Stem::Giap),
            Stem::Tan => Some(Stem::At),
            Stem::Nham => Some(Stem::Binh),
            Stem::Quy => Some(Stem::Dinh),
        }
    }

    /// Vietnamese name, e.g. "Giáp".
    pub const fn name_vi(self) -> &'static str {
        match self {
            Stem::Giap => "Giáp",
            Stem::At => "Ất",
            Stem::Binh => "Bính",
            Stem::Dinh => "Đinh",
            Stem::Mau => "Mậu",
            Stem::Ky => "Kỷ",
            Stem::Canh => "Canh",
            Stem::Tan => "Tân",
            Stem::Nham => "Nhâm",
            Stem::Quy => "Quý",
        }
    }

    /// Hán character, e.g. "甲".
    pub const fn name_han(self) -> &'static str {
        match self {
            Stem::Giap => "甲",
            Stem::At => "乙",
            Stem::Binh => "丙",
            Stem::Dinh => "丁",
            Stem::Mau => "戊",
            Stem::Ky => "己",
            Stem::Canh => "庚",
            Stem::Tan => "辛",
            Stem::Nham => "壬",
            Stem::Quy => "癸",
        }
    }
}

/// The twelve Earthly Branches (Địa Chi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Branch {
    /// Tý 子
    Ty,
    /// Sửu 丑
    Suu,
    /// Dần 寅
    Dan,
    /// Mão 卯
    Mao,
    /// Thìn 辰
    Thin,
    /// Tỵ 巳 (also written Tị)
    Ti,
    /// Ngọ 午
    Ngo,
    /// Mùi 未
    Mui,
    /// Thân 申
    Than,
    /// Dậu 酉
    Dau,
    /// Tuất 戌
    Tuat,
    /// Hợi 亥
    Hoi,
}

impl Branch {
    /// All branches in cycle order, from Tý.
    pub const ALL: [Branch; 12] = [
        Branch::Ty,
        Branch::Suu,
        Branch::Dan,
        Branch::Mao,
        Branch::Thin,
        Branch::Ti,
        Branch::Ngo,
        Branch::Mui,
        Branch::Than,
        Branch::Dau,
        Branch::Tuat,
        Branch::Hoi,
    ];

    /// Position in the cycle, from 0 (Tý) to 11 (Hợi).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The branch at cycle position `index`, wrapping modulo 12.
    pub const fn from_index(index: i64) -> Self {
        Self::ALL[index.rem_euclid(12) as usize]
    }

    /// Element of the branch.
    pub const fn element(self) -> FiveElements {
        match self {
            Branch::Dan | Branch::Mao => FiveElements::Moc,
            Branch::Ti | Branch::Ngo => FiveElements::Hoa,
            Branch::Thin | Branch::Suu | Branch::Tuat | Branch::Mui => FiveElements::Tho,
            Branch::Than | Branch::Dau => FiveElements::Kim,
            Branch::Hoi | Branch::Ty => FiveElements::Thuy,
        }
    }

    /// Branch that clashes with this one (Lục Xung): the one opposite it.
    pub const fn clash(self) -> Branch {
        Self::from_index(self.index() as i64 + 6)
    }

    /// Partner in the Six Combinations (Lục Hợp): Tý–Sửu, Dần–Hợi, Mão–Tuất, Thìn–Dậu, Tỵ–Thân,
    /// Ngọ–Mùi.
    pub const fn six_combination(self) -> Branch {
        Self::from_index(13 - self.index() as i64)
    }

    /// The three branches of this branch's Tam Hợp group.
    pub const fn trine(self) -> [Branch; 3] {
        TRINES[self.trine_group()]
    }

    /// Row of this branch's group in [`TRINES`].
    pub(crate) const fn trine_group(self) -> usize {
        match self {
            Branch::Than | Branch::Ty | Branch::Thin => 0,
            Branch::Dan | Branch::Ngo | Branch::Tuat => 1,
            Branch::Ti | Branch::Dau | Branch::Suu => 2,
            Branch::Hoi | Branch::Mao | Branch::Mui => 3,
        }
    }

    /// Partner in the Six Harms (Lục Hại): Tý–Mùi, Sửu–Ngọ, Dần–Tỵ, Mão–Thìn, Thân–Hợi, Dậu–Tuất.
    pub const fn harm(self) -> Branch {
        Self::from_index(7 - self.index() as i64)
    }

    /// Branch this one punishes (Hình): Dần→Tỵ→Thân→Dần, Sửu→Tuất→Mùi→Sửu, Tý↔Mão; Thìn, Ngọ,
    /// Dậu and Hợi punish themselves.
    pub const fn punishment(self) -> Branch {
        match self {
            Branch::Ty => Branch::Mao,
            Branch::Suu => Branch::Tuat,
            Branch::Dan => Branch::Ti,
            Branch::Mao => Branch::Ty,
            Branch::Thin => Branch::Thin,
            Branch::Ti => Branch::Than,
            Branch::Ngo => Branch::Ngo,
            Branch::Mui => Branch::Suu,
            Branch::Than => Branch::Dan,
            Branch::Dau => Branch::Dau,
            Branch::Tuat => Branch::Mui,
            Branch::Hoi => Branch::Hoi,
        }
    }

    /// Partner in the Six Destructions (Lục Phá): Tý–Dậu, Ngọ–Mão, Thìn–Sửu, Tuất–Mùi, Dần–Hợi,
    /// Tỵ–Thân. Table confirmed by the project owner.
    pub const fn destruction(self) -> Branch {
        match self {
            Branch::Ty => Branch::Dau,
            Branch::Suu => Branch::Thin,
            Branch::Dan => Branch::Hoi,
            Branch::Mao => Branch::Ngo,
            Branch::Thin => Branch::Suu,
            Branch::Ti => Branch::Than,
            Branch::Ngo => Branch::Mao,
            Branch::Mui => Branch::Tuat,
            Branch::Than => Branch::Ti,
            Branch::Dau => Branch::Ty,
            Branch::Tuat => Branch::Mui,
            Branch::Hoi => Branch::Dan,
        }
    }

    /// Branch of the double-hour (canh giờ) containing `hour` (0–23): Tý is 23:00–00:59,
    /// Sửu 01:00–02:59, and so on to Hợi 21:00–22:59.
    pub const fn from_hour(hour: u8) -> Self {
        Self::from_index((hour as i64 + 1) / 2)
    }

    /// Vietnamese name, e.g. "Tý".
    pub const fn name_vi(self) -> &'static str {
        match self {
            Branch::Ty => "Tý",
            Branch::Suu => "Sửu",
            Branch::Dan => "Dần",
            Branch::Mao => "Mão",
            Branch::Thin => "Thìn",
            Branch::Ti => "Tỵ",
            Branch::Ngo => "Ngọ",
            Branch::Mui => "Mùi",
            Branch::Than => "Thân",
            Branch::Dau => "Dậu",
            Branch::Tuat => "Tuất",
            Branch::Hoi => "Hợi",
        }
    }

    /// Hán character, e.g. "子".
    pub const fn name_han(self) -> &'static str {
        match self {
            Branch::Ty => "子",
            Branch::Suu => "丑",
            Branch::Dan => "寅",
            Branch::Mao => "卯",
            Branch::Thin => "辰",
            Branch::Ti => "巳",
            Branch::Ngo => "午",
            Branch::Mui => "未",
            Branch::Than => "申",
            Branch::Dau => "酉",
            Branch::Tuat => "戌",
            Branch::Hoi => "亥",
        }
    }
}

/// The four Tam Hợp groups, in the row order of the tables in the specification.
pub(crate) const TRINES: [[Branch; 3]; 4] = [
    [Branch::Than, Branch::Ty, Branch::Thin],
    [Branch::Dan, Branch::Ngo, Branch::Tuat],
    [Branch::Ti, Branch::Dau, Branch::Suu],
    [Branch::Hoi, Branch::Mao, Branch::Mui],
];

/// A sexagenary pair. Only the 60 pairs whose stem and branch indices share parity occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CanChi {
    /// Heavenly Stem (Can).
    pub stem: Stem,
    /// Earthly Branch (Chi).
    pub branch: Branch,
}

impl CanChi {
    /// Pairs a stem with a branch, or returns `None` for a pair outside the cycle
    /// (e.g. Giáp Sửu).
    pub const fn new(stem: Stem, branch: Branch) -> Option<Self> {
        if stem.index() % 2 == branch.index() % 2 {
            Some(Self { stem, branch })
        } else {
            None
        }
    }

    /// The pair at cycle position `index` (0 = Giáp Tý, 59 = Quý Hợi), wrapping modulo 60.
    pub const fn from_cycle_index(index: i64) -> Self {
        Self {
            stem: Stem::from_index(index),
            branch: Branch::from_index(index),
        }
    }

    /// Position in the cycle, from 0 (Giáp Tý) to 59 (Quý Hợi).
    pub const fn cycle_index(self) -> u8 {
        // Chinese remainder theorem: i ≡ stem (mod 10) and i ≡ branch (mod 12) give
        // i ≡ 6·stem − 5·branch (mod 60).
        (6 * self.stem.index() as i32 - 5 * self.branch.index() as i32).rem_euclid(60) as u8
    }

    /// Nạp Âm of the pair; two consecutive pairs of the cycle share one.
    pub const fn na_yin(self) -> NaYin {
        NaYin::ALL[self.cycle_index() as usize / 2]
    }

    /// Vietnamese name, e.g. "Giáp Tý".
    pub fn name_vi(self) -> String {
        self.to_string()
    }

    /// Hán characters, e.g. "甲子".
    pub fn name_han(self) -> String {
        format!("{}{}", self.stem.name_han(), self.branch.name_han())
    }
}

/// The 30 Nạp Âm of the sexagenary cycle, in cycle order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NaYin {
    /// Hải Trung Kim
    HaiTrungKim,
    /// Lư Trung Hỏa
    LuTrungHoa,
    /// Đại Lâm Mộc
    DaiLamMoc,
    /// Lộ Bàng Thổ
    LoBangTho,
    /// Kiếm Phong Kim
    KiemPhongKim,
    /// Sơn Đầu Hỏa
    SonDauHoa,
    /// Giản Hạ Thủy
    GianHaThuy,
    /// Thành Đầu Thổ
    ThanhDauTho,
    /// Bạch Lạp Kim
    BachLapKim,
    /// Dương Liễu Mộc
    DuongLieuMoc,
    /// Tuyền Trung Thủy
    TuyenTrungThuy,
    /// Ốc Thượng Thổ
    OcThuongTho,
    /// Tích Lịch Hỏa
    TichLichHoa,
    /// Tùng Bách Mộc
    TungBachMoc,
    /// Trường Lưu Thủy
    TruongLuuThuy,
    /// Sa Trung Kim
    SaTrungKim,
    /// Sơn Hạ Hỏa
    SonHaHoa,
    /// Bình Địa Mộc
    BinhDiaMoc,
    /// Bích Thượng Thổ
    BichThuongTho,
    /// Kim Bạch Kim
    KimBachKim,
    /// Phú Đăng Hỏa
    PhuDangHoa,
    /// Thiên Hà Thủy
    ThienHaThuy,
    /// Đại Dịch Thổ
    DaiDichTho,
    /// Thoa Xuyến Kim
    ThoaXuyenKim,
    /// Tang Đố Mộc
    TangDoMoc,
    /// Đại Khê Thủy
    DaiKheThuy,
    /// Sa Trung Thổ
    SaTrungTho,
    /// Thiên Thượng Hỏa
    ThienThuongHoa,
    /// Thạch Lựu Mộc
    ThachLuuMoc,
    /// Đại Hải Thủy
    DaiHaiThuy,
}

impl NaYin {
    /// All Nạp Âm in cycle order, from Hải Trung Kim (Giáp Tý, Ất Sửu).
    pub const ALL: [NaYin; 30] = [
        NaYin::HaiTrungKim,
        NaYin::LuTrungHoa,
        NaYin::DaiLamMoc,
        NaYin::LoBangTho,
        NaYin::KiemPhongKim,
        NaYin::SonDauHoa,
        NaYin::GianHaThuy,
        NaYin::ThanhDauTho,
        NaYin::BachLapKim,
        NaYin::DuongLieuMoc,
        NaYin::TuyenTrungThuy,
        NaYin::OcThuongTho,
        NaYin::TichLichHoa,
        NaYin::TungBachMoc,
        NaYin::TruongLuuThuy,
        NaYin::SaTrungKim,
        NaYin::SonHaHoa,
        NaYin::BinhDiaMoc,
        NaYin::BichThuongTho,
        NaYin::KimBachKim,
        NaYin::PhuDangHoa,
        NaYin::ThienHaThuy,
        NaYin::DaiDichTho,
        NaYin::ThoaXuyenKim,
        NaYin::TangDoMoc,
        NaYin::DaiKheThuy,
        NaYin::SaTrungTho,
        NaYin::ThienThuongHoa,
        NaYin::ThachLuuMoc,
        NaYin::DaiHaiThuy,
    ];

    /// Vietnamese name, e.g. "Hải Trung Kim".
    pub const fn name_vi(self) -> &'static str {
        match self {
            NaYin::HaiTrungKim => "Hải Trung Kim",
            NaYin::LuTrungHoa => "Lư Trung Hỏa",
            NaYin::DaiLamMoc => "Đại Lâm Mộc",
            NaYin::LoBangTho => "Lộ Bàng Thổ",
            NaYin::KiemPhongKim => "Kiếm Phong Kim",
            NaYin::SonDauHoa => "Sơn Đầu Hỏa",
            NaYin::GianHaThuy => "Giản Hạ Thủy",
            NaYin::ThanhDauTho => "Thành Đầu Thổ",
            NaYin::BachLapKim => "Bạch Lạp Kim",
            NaYin::DuongLieuMoc => "Dương Liễu Mộc",
            NaYin::TuyenTrungThuy => "Tuyền Trung Thủy",
            NaYin::OcThuongTho => "Ốc Thượng Thổ",
            NaYin::TichLichHoa => "Tích Lịch Hỏa",
            NaYin::TungBachMoc => "Tùng Bách Mộc",
            NaYin::TruongLuuThuy => "Trường Lưu Thủy",
            NaYin::SaTrungKim => "Sa Trung Kim",
            NaYin::SonHaHoa => "Sơn Hạ Hỏa",
            NaYin::BinhDiaMoc => "Bình Địa Mộc",
            NaYin::BichThuongTho => "Bích Thượng Thổ",
            NaYin::KimBachKim => "Kim Bạch Kim",
            NaYin::PhuDangHoa => "Phú Đăng Hỏa",
            NaYin::ThienHaThuy => "Thiên Hà Thủy",
            NaYin::DaiDichTho => "Đại Dịch Thổ",
            NaYin::ThoaXuyenKim => "Thoa Xuyến Kim",
            NaYin::TangDoMoc => "Tang Đố Mộc",
            NaYin::DaiKheThuy => "Đại Khê Thủy",
            NaYin::SaTrungTho => "Sa Trung Thổ",
            NaYin::ThienThuongHoa => "Thiên Thượng Hỏa",
            NaYin::ThachLuuMoc => "Thạch Lựu Mộc",
            NaYin::DaiHaiThuy => "Đại Hải Thủy",
        }
    }

    /// Element of the Nạp Âm.
    pub const fn element(self) -> FiveElements {
        match self {
            NaYin::HaiTrungKim
            | NaYin::KiemPhongKim
            | NaYin::BachLapKim
            | NaYin::SaTrungKim
            | NaYin::KimBachKim
            | NaYin::ThoaXuyenKim => FiveElements::Kim,
            NaYin::LuTrungHoa
            | NaYin::SonDauHoa
            | NaYin::TichLichHoa
            | NaYin::SonHaHoa
            | NaYin::PhuDangHoa
            | NaYin::ThienThuongHoa => FiveElements::Hoa,
            NaYin::DaiLamMoc
            | NaYin::DuongLieuMoc
            | NaYin::TungBachMoc
            | NaYin::BinhDiaMoc
            | NaYin::TangDoMoc
            | NaYin::ThachLuuMoc => FiveElements::Moc,
            NaYin::LoBangTho
            | NaYin::ThanhDauTho
            | NaYin::OcThuongTho
            | NaYin::BichThuongTho
            | NaYin::DaiDichTho
            | NaYin::SaTrungTho => FiveElements::Tho,
            NaYin::GianHaThuy
            | NaYin::TuyenTrungThuy
            | NaYin::TruongLuuThuy
            | NaYin::ThienHaThuy
            | NaYin::DaiKheThuy
            | NaYin::DaiHaiThuy => FiveElements::Thuy,
        }
    }
}

impl fmt::Display for CanChi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.stem.name_vi(), self.branch.name_vi())
    }
}

/// Can Chi of lunar year `year`, e.g. 2024 → Giáp Thìn.
pub const fn year_can_chi(year: i32) -> CanChi {
    CanChi::from_cycle_index(year as i64 - 4)
}

/// Can Chi of lunar month `month` (1–12) of lunar year `year`; a leap month shares the Can
/// Chi of the month it repeats.
pub const fn month_can_chi(year: i32, month: u8) -> CanChi {
    CanChi {
        stem: Stem::from_index(12 * year as i64 + month as i64 + 3),
        branch: Branch::from_index(month as i64 + 1),
    }
}

/// Can Chi of the day with Julian Day Number `jdn`.
pub const fn day_can_chi(jdn: i32) -> CanChi {
    CanChi::from_cycle_index(jdn as i64 + 49)
}

/// Can Chi of the double-hour `branch` on a day whose stem is `day_stem`.
pub const fn hour_can_chi(day_stem: Stem, branch: Branch) -> CanChi {
    CanChi {
        stem: Stem::from_index(2 * day_stem.index() as i64 + branch.index() as i64),
        branch,
    }
}

#[cfg(test)]
mod tests {
    use super::Branch::*;
    use super::Stem::*;
    use super::*;

    #[test]
    fn cycle_positions_round_trip() {
        for index in 0..60 {
            let pair = CanChi::from_cycle_index(index);
            assert_eq!(i64::from(pair.cycle_index()), index);
            assert_eq!(CanChi::new(pair.stem, pair.branch), Some(pair));
        }
        assert_eq!(CanChi::from_cycle_index(60), CanChi::from_cycle_index(0));
        assert_eq!(CanChi::from_cycle_index(-1).to_string(), "Quý Hợi");
    }

    #[test]
    fn only_same_parity_pairs_exist() {
        assert_eq!(CanChi::new(Giap, Suu), None);
        let pairs = Stem::ALL
            .into_iter()
            .flat_map(|stem| {
                Branch::ALL
                    .into_iter()
                    .filter_map(move |branch| CanChi::new(stem, branch))
            })
            .count();
        assert_eq!(pairs, 60);
    }

    #[test]
    fn names() {
        assert_eq!(CanChi::from_cycle_index(0).to_string(), "Giáp Tý");
        assert_eq!(CanChi::from_cycle_index(0).name_han(), "甲子");
        assert_eq!(CanChi::from_cycle_index(59).name_vi(), "Quý Hợi");
        assert_eq!(Ti.name_vi(), "Tỵ");
        assert_eq!(Mau.name_han(), "戊");
    }

    #[test]
    fn years() {
        for (year, name) in [
            (1945, "Ất Dậu"),
            (1968, "Mậu Thân"),
            (1975, "Ất Mão"),
            (1984, "Giáp Tý"),
            (2000, "Canh Thìn"),
            (2024, "Giáp Thìn"),
            (2025, "Ất Tỵ"),
            (2026, "Bính Ngọ"),
        ] {
            assert_eq!(year_can_chi(year).to_string(), name, "{year}");
        }
    }

    #[test]
    fn days() {
        // 1 January 2000 was a Mậu Ngọ day and Tết Giáp Thìn (10 February 2024) a Giáp Thìn day.
        assert_eq!(day_can_chi(2_451_545).to_string(), "Mậu Ngọ");
        assert_eq!(day_can_chi(2_460_351).to_string(), "Giáp Thìn");
        assert_eq!(day_can_chi(2_460_352).to_string(), "Ất Tỵ");
    }

    #[test]
    fn first_month_stem_follows_the_year_stem() {
        // Ngũ Hổ Độn: Giáp/Kỷ years open with Bính Dần, Ất/Canh with Mậu Dần, Bính/Tân with
        // Canh Dần, Đinh/Nhâm with Nhâm Dần, Mậu/Quý with Giáp Dần (1984 is a Giáp year).
        let first_month_stems = [Binh, Mau, Canh, Nham, Giap, Binh, Mau, Canh, Nham, Giap];
        for (year, stem) in (1984..).zip(first_month_stems) {
            assert_eq!(
                month_can_chi(year, 1),
                CanChi { stem, branch: Dan },
                "{year}"
            );
        }
        assert_eq!(month_can_chi(2023, 11).to_string(), "Giáp Tý");
        assert_eq!(month_can_chi(2023, 12).to_string(), "Ất Sửu");
    }

    #[test]
    fn rat_hour_stem_follows_the_day_stem() {
        // Ngũ Thử Độn: Giáp/Kỷ days start with Giáp Tý, Ất/Canh with Bính Tý, Bính/Tân with
        // Mậu Tý, Đinh/Nhâm with Canh Tý, Mậu/Quý with Nhâm Tý.
        let rat_hour_stems = [Giap, Binh, Mau, Canh, Nham, Giap, Binh, Mau, Canh, Nham];
        for (day_stem, stem) in Stem::ALL.into_iter().zip(rat_hour_stems) {
            assert_eq!(
                hour_can_chi(day_stem, Ty),
                CanChi { stem, branch: Ty },
                "{day_stem:?}"
            );
        }
        assert_eq!(hour_can_chi(Giap, Thin).to_string(), "Mậu Thìn");
    }

    #[test]
    fn elements_and_polarity() {
        assert_eq!(Giap.element(), FiveElements::Moc);
        assert_eq!(Ky.element(), FiveElements::Tho);
        assert_eq!(Quy.element(), FiveElements::Thuy);
        assert_eq!(Ty.element(), FiveElements::Thuy);
        assert_eq!(Mui.element(), FiveElements::Tho);
        assert_eq!(Dau.element(), FiveElements::Kim);
        assert_eq!(Giap.yin_yang(), YinYang::Duong);
        assert_eq!(Ky.yin_yang(), YinYang::Am);
        assert_eq!(FiveElements::Hoa.name_vi(), "Hỏa");
        assert_eq!(YinYang::Am.name_vi(), "Âm");
    }

    #[test]
    fn stem_relations() {
        for (a, b) in [
            (Giap, Ky),
            (At, Canh),
            (Binh, Tan),
            (Dinh, Nham),
            (Mau, Quy),
        ] {
            assert_eq!((a.combination(), b.combination()), (b, a));
        }
        for (a, b) in [(Giap, Canh), (At, Tan), (Binh, Nham), (Dinh, Quy)] {
            assert_eq!((a.clash(), b.clash()), (Some(b), Some(a)));
        }
        assert_eq!((Mau.clash(), Ky.clash()), (None, None));
    }

    #[test]
    fn branch_pair_relations_are_symmetric() {
        for (a, b) in [
            (Ty, Ngo),
            (Suu, Mui),
            (Dan, Than),
            (Mao, Dau),
            (Thin, Tuat),
            (Ti, Hoi),
        ] {
            assert_eq!((a.clash(), b.clash()), (b, a));
        }
        for (a, b) in [
            (Ty, Suu),
            (Dan, Hoi),
            (Mao, Tuat),
            (Thin, Dau),
            (Ti, Than),
            (Ngo, Mui),
        ] {
            assert_eq!((a.six_combination(), b.six_combination()), (b, a));
        }
        for (a, b) in [
            (Ty, Mui),
            (Suu, Ngo),
            (Dan, Ti),
            (Mao, Thin),
            (Than, Hoi),
            (Dau, Tuat),
        ] {
            assert_eq!((a.harm(), b.harm()), (b, a));
        }
    }

    #[test]
    fn luc_pha_is_the_six_confirmed_pairs() {
        let pairs = [
            (Ty, Dau),
            (Ngo, Mao),
            (Thin, Suu),
            (Tuat, Mui),
            (Dan, Hoi),
            (Ti, Than),
        ];
        for (a, b) in pairs {
            assert_eq!((a.destruction(), b.destruction()), (b, a));
        }
        // The six pairs use each branch exactly once.
        let covered: std::collections::BTreeSet<_> =
            pairs.into_iter().flat_map(|(a, b)| [a, b]).collect();
        assert_eq!(covered.len(), 12);
    }

    #[test]
    fn punishment_cycles() {
        assert_eq!(
            (Dan.punishment(), Ti.punishment(), Than.punishment()),
            (Ti, Than, Dan)
        );
        assert_eq!(
            (Suu.punishment(), Tuat.punishment(), Mui.punishment()),
            (Tuat, Mui, Suu)
        );
        assert_eq!((Ty.punishment(), Mao.punishment()), (Mao, Ty));
        for branch in [Thin, Ngo, Dau, Hoi] {
            assert_eq!(branch.punishment(), branch);
        }
    }

    #[test]
    fn trines_cover_every_branch_once() {
        assert_eq!(Ty.trine(), [Than, Ty, Thin]);
        assert_eq!(Tuat.trine(), [Dan, Ngo, Tuat]);
        assert_eq!(Suu.trine(), [Ti, Dau, Suu]);
        assert_eq!(Mao.trine(), [Hoi, Mao, Mui]);
        for branch in Branch::ALL {
            assert!(branch.trine().contains(&branch), "{branch:?}");
        }
    }

    #[test]
    fn na_yin_of_the_cycle() {
        let of = |index| CanChi::from_cycle_index(index).na_yin();
        assert_eq!(of(0), NaYin::HaiTrungKim);
        assert_eq!(of(1), NaYin::HaiTrungKim);
        assert_eq!(of(40).name_vi(), "Phú Đăng Hỏa");
        assert_eq!(of(59), NaYin::DaiHaiThuy);
        assert_eq!(of(59).element(), FiveElements::Thuy);
        assert_eq!(of(24).element(), FiveElements::Hoa);
    }

    #[test]
    fn double_hours() {
        for (hour, branch) in [
            (23, Ty),
            (0, Ty),
            (1, Suu),
            (2, Suu),
            (3, Dan),
            (11, Ngo),
            (12, Ngo),
            (13, Mui),
            (21, Hoi),
            (22, Hoi),
        ] {
            assert_eq!(Branch::from_hour(hour), branch, "{hour}h");
        }
    }
}
