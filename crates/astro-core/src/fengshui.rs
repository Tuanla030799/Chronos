//! Feng shui (Phong Thủy): the Eight Mansions (Bát Trạch) and the Nine Flying Stars (Cửu Cung Phi
//! Tinh).
//!
//! The rules and tables come from `docs/FENGSHUI_SPEC.md`:
//!
//! - **Eight Mansions.** [`kua_info`] gives a person's trigram (Mệnh Quái) from the lunar birth
//!   year and gender, [`house_trigram`] gives a house's trigram (Trạch Quái) from its sitting
//!   direction (Tọa). The matrix approved from *Bát Trạch Minh Cảnh* is read towards a direction
//!   with [`Trigram::star_at`], towards another trigram with [`Trigram::star_towards`] and
//!   between two people with [`kua_compatibility`].
//! - **Nine Stars.** [`nine_stars`] follows `Lunar.getYearNineStar`, `getMonthNineStar`,
//!   `getDayNineStar` and `getTimeNineStar` of lunar-typescript, and [`van_of_year`] follows
//!   `LunarYear.getYun`. By default the year star, the era and the period change on the day of
//!   Lập Xuân and the month star on the day of each Tiết ([`NineStarBoundary`]), while a person's
//!   trigram changes at Tết.
//! - **Lo Shu grid.** [`generate_loshu_grid`] flies the stars out of the center palace along the
//!   Lạc Thư path. Year and month grids fly forward; day and hour grids take the direction in
//!   [`FlyingStar::forward`].
//!
//! With the default boundary, terms are reckoned on local days at `tz_offset`. From 1900 to 2100,
//! `8.0` then matches lunar-typescript on every day and `7.0` differs from it only on the days
//! listed in section 0.3 of the spec. [`NineStarBoundary`] says how the other boundaries differ.

use crate::almanac::{Luck, solar_month_branch, solar_month_branch_at};
use crate::canchi::{Branch, FiveElements, day_can_chi, year_can_chi};
use crate::error::{AstroError, check_tz_offset, check_year};
use crate::lunar_calendar::solar_to_lunar;
use crate::solar_terms::{SolarTerm, term_instant};
use crate::time::{SolarDate, SolarDateTime, local_jdn};

pub use crate::bazi_analysis::Gender;

/// The eight trigrams of the Later Heaven arrangement (Hậu Thiên Bát Quái), in the order of their
/// Lạc Thư numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Trigram {
    /// Khảm 坎 (1)
    Kham,
    /// Khôn 坤 (2)
    Khon,
    /// Chấn 震 (3)
    Chan,
    /// Tốn 巽 (4)
    Ton,
    /// Càn 乾 (6); `CanKim` because Càn and Cấn collide without diacritics
    CanKim,
    /// Đoài 兑 (7)
    Doai,
    /// Cấn 艮 (8); `CanTho` because Càn and Cấn collide without diacritics
    CanTho,
    /// Ly 离 (9)
    Ly,
}

impl Trigram {
    /// All trigrams in the order of their Lạc Thư numbers, from Khảm.
    pub const ALL: [Trigram; 8] = [
        Trigram::Kham,
        Trigram::Khon,
        Trigram::Chan,
        Trigram::Ton,
        Trigram::CanKim,
        Trigram::Doai,
        Trigram::CanTho,
        Trigram::Ly,
    ];

    /// The trigram with Lạc Thư number `number`; the center (5) and numbers outside 1–9 have none.
    pub const fn from_number(number: u8) -> Option<Self> {
        match number {
            1 => Some(Trigram::Kham),
            2 => Some(Trigram::Khon),
            3 => Some(Trigram::Chan),
            4 => Some(Trigram::Ton),
            6 => Some(Trigram::CanKim),
            7 => Some(Trigram::Doai),
            8 => Some(Trigram::CanTho),
            9 => Some(Trigram::Ly),
            _ => None,
        }
    }

    /// Lạc Thư number: 1–4 or 6–9.
    pub const fn number(self) -> u8 {
        self.palace().number()
    }

    /// Palace of the Lo Shu grid held by this trigram.
    pub const fn palace(self) -> Palace {
        match self {
            Trigram::Kham => Palace::Kham,
            Trigram::Khon => Palace::Khon,
            Trigram::Chan => Palace::Chan,
            Trigram::Ton => Palace::Ton,
            Trigram::CanKim => Palace::CanKim,
            Trigram::Doai => Palace::Doai,
            Trigram::CanTho => Palace::CanTho,
            Trigram::Ly => Palace::Ly,
        }
    }

    /// Compass direction of the trigram.
    pub const fn direction(self) -> CompassDirection {
        match self {
            Trigram::Kham => CompassDirection::Bac,
            Trigram::Khon => CompassDirection::TayNam,
            Trigram::Chan => CompassDirection::Dong,
            Trigram::Ton => CompassDirection::DongNam,
            Trigram::CanKim => CompassDirection::TayBac,
            Trigram::Doai => CompassDirection::Tay,
            Trigram::CanTho => CompassDirection::DongBac,
            Trigram::Ly => CompassDirection::Nam,
        }
    }

    /// Element of the trigram.
    pub const fn element(self) -> FiveElements {
        match self {
            Trigram::Kham => FiveElements::Thuy,
            Trigram::Khon | Trigram::CanTho => FiveElements::Tho,
            Trigram::Chan | Trigram::Ton => FiveElements::Moc,
            Trigram::CanKim | Trigram::Doai => FiveElements::Kim,
            Trigram::Ly => FiveElements::Hoa,
        }
    }

    /// Group of the trigram: Đông Tứ or Tây Tứ.
    pub const fn group(self) -> TrigramGroup {
        match self {
            Trigram::Kham | Trigram::Ly | Trigram::Chan | Trigram::Ton => TrigramGroup::DongTu,
            Trigram::CanKim | Trigram::Khon | Trigram::CanTho | Trigram::Doai => {
                TrigramGroup::TayTu
            }
        }
    }

    /// Star of the Eight Mansions between this trigram and `other`. The matrix is symmetric, so
    /// `a.star_towards(b) == b.star_towards(a)`.
    pub const fn star_towards(self, other: Trigram) -> EightMansionsStar {
        EIGHT_MANSIONS[self as usize][other as usize]
    }

    /// Star of the Eight Mansions in `direction`, for a person of this Mệnh Quái or a house of
    /// this Trạch Quái.
    pub const fn star_at(self, direction: CompassDirection) -> EightMansionsStar {
        self.star_towards(direction.trigram())
    }

    /// Stars of the eight directions, in the order of [`CompassDirection::ALL`].
    pub fn chart(self) -> [DirectionStar; 8] {
        CompassDirection::ALL.map(|direction| DirectionStar {
            direction,
            star: self.star_at(direction),
        })
    }

    /// Vietnamese name, e.g. "Khảm".
    pub const fn name_vi(self) -> &'static str {
        match self {
            Trigram::Kham => "Khảm",
            Trigram::Khon => "Khôn",
            Trigram::Chan => "Chấn",
            Trigram::Ton => "Tốn",
            Trigram::CanKim => "Càn",
            Trigram::Doai => "Đoài",
            Trigram::CanTho => "Cấn",
            Trigram::Ly => "Ly",
        }
    }

    /// Hán character, e.g. "坎".
    pub const fn name_han(self) -> &'static str {
        match self {
            Trigram::Kham => "坎",
            Trigram::Khon => "坤",
            Trigram::Chan => "震",
            Trigram::Ton => "巽",
            Trigram::CanKim => "乾",
            Trigram::Doai => "兑",
            Trigram::CanTho => "艮",
            Trigram::Ly => "离",
        }
    }
}

/// The two groups of trigrams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrigramGroup {
    /// Đông Tứ: Khảm, Ly, Chấn, Tốn
    DongTu,
    /// Tây Tứ: Càn, Khôn, Cấn, Đoài
    TayTu,
}

impl TrigramGroup {
    /// Vietnamese name, "Đông Tứ" or "Tây Tứ".
    pub const fn name_vi(self) -> &'static str {
        match self {
            TrigramGroup::DongTu => "Đông Tứ",
            TrigramGroup::TayTu => "Tây Tứ",
        }
    }
}

/// The nine palaces of the Lo Shu grid (Lạc Thư Cửu Cung), in the order of their numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Palace {
    /// Khảm 坎 (1)
    Kham,
    /// Khôn 坤 (2)
    Khon,
    /// Chấn 震 (3)
    Chan,
    /// Tốn 巽 (4)
    Ton,
    /// Trung Cung 中 (5), the center
    TrungCung,
    /// Càn 乾 (6)
    CanKim,
    /// Đoài 兑 (7)
    Doai,
    /// Cấn 艮 (8)
    CanTho,
    /// Ly 离 (9)
    Ly,
}

impl Palace {
    /// All palaces in the order of their numbers, from Khảm.
    pub const ALL: [Palace; 9] = [
        Palace::Kham,
        Palace::Khon,
        Palace::Chan,
        Palace::Ton,
        Palace::TrungCung,
        Palace::CanKim,
        Palace::Doai,
        Palace::CanTho,
        Palace::Ly,
    ];

    /// Position from 0 (Khảm) to 8 (Ly), the Lạc Thư number minus one.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Lạc Thư number, from 1 (Khảm) to 9 (Ly).
    pub const fn number(self) -> u8 {
        self as u8 + 1
    }

    /// Trigram of the palace; the center has none.
    pub const fn trigram(self) -> Option<Trigram> {
        Trigram::from_number(self.number())
    }

    /// Compass direction of the palace; the center has none.
    pub const fn direction(self) -> Option<CompassDirection> {
        match self.trigram() {
            Some(trigram) => Some(trigram.direction()),
            None => None,
        }
    }

    /// Element of the palace.
    pub const fn element(self) -> FiveElements {
        match self.trigram() {
            Some(trigram) => trigram.element(),
            None => FiveElements::Tho,
        }
    }

    /// Vietnamese name, e.g. "Trung Cung".
    pub const fn name_vi(self) -> &'static str {
        match self.trigram() {
            Some(trigram) => trigram.name_vi(),
            None => "Trung Cung",
        }
    }

    /// Hán character, e.g. "中".
    pub const fn name_han(self) -> &'static str {
        match self.trigram() {
            Some(trigram) => trigram.name_han(),
            None => "中",
        }
    }
}

/// The eight compass directions, clockwise from north.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CompassDirection {
    /// Bắc, north
    Bac,
    /// Đông Bắc, north-east
    DongBac,
    /// Đông, east
    Dong,
    /// Đông Nam, south-east
    DongNam,
    /// Nam, south
    Nam,
    /// Tây Nam, south-west
    TayNam,
    /// Tây, west
    Tay,
    /// Tây Bắc, north-west
    TayBac,
}

impl CompassDirection {
    /// All directions, clockwise from north.
    pub const ALL: [CompassDirection; 8] = [
        CompassDirection::Bac,
        CompassDirection::DongBac,
        CompassDirection::Dong,
        CompassDirection::DongNam,
        CompassDirection::Nam,
        CompassDirection::TayNam,
        CompassDirection::Tay,
        CompassDirection::TayBac,
    ];

    /// Position from 0 (Bắc) to 7 (Tây Bắc).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The opposite direction: a house facing south sits north.
    pub const fn opposite(self) -> CompassDirection {
        Self::ALL[(self.index() as usize + 4) % 8]
    }

    /// Trigram of the direction.
    pub const fn trigram(self) -> Trigram {
        match self {
            CompassDirection::Bac => Trigram::Kham,
            CompassDirection::DongBac => Trigram::CanTho,
            CompassDirection::Dong => Trigram::Chan,
            CompassDirection::DongNam => Trigram::Ton,
            CompassDirection::Nam => Trigram::Ly,
            CompassDirection::TayNam => Trigram::Khon,
            CompassDirection::Tay => Trigram::Doai,
            CompassDirection::TayBac => Trigram::CanKim,
        }
    }

    /// Vietnamese name, e.g. "Đông Bắc".
    pub const fn name_vi(self) -> &'static str {
        match self {
            CompassDirection::Bac => "Bắc",
            CompassDirection::DongBac => "Đông Bắc",
            CompassDirection::Dong => "Đông",
            CompassDirection::DongNam => "Đông Nam",
            CompassDirection::Nam => "Nam",
            CompassDirection::TayNam => "Tây Nam",
            CompassDirection::Tay => "Tây",
            CompassDirection::TayBac => "Tây Bắc",
        }
    }
}

/// The eight stars of the Eight Mansions (Bát Du Niên): four good, then four bad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EightMansionsStar {
    /// Sinh Khí 生气
    SinhKhi,
    /// Thiên Y 天医
    ThienY,
    /// Diên Niên 延年, also called Phước Đức
    DienNien,
    /// Phục Vị 伏位
    PhucVi,
    /// Tuyệt Mệnh 绝命
    TuyetMenh,
    /// Ngũ Quỷ 五鬼
    NguQuy,
    /// Lục Sát 六煞
    LucSat,
    /// Họa Hại 祸害
    HoaHai,
}

impl EightMansionsStar {
    /// All stars, the four good ones first.
    pub const ALL: [EightMansionsStar; 8] = [
        EightMansionsStar::SinhKhi,
        EightMansionsStar::ThienY,
        EightMansionsStar::DienNien,
        EightMansionsStar::PhucVi,
        EightMansionsStar::TuyetMenh,
        EightMansionsStar::NguQuy,
        EightMansionsStar::LucSat,
        EightMansionsStar::HoaHai,
    ];

    /// Position from 0 (Sinh Khí) to 7 (Họa Hại).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Fortune of the star.
    pub const fn luck(self) -> Luck {
        match self {
            EightMansionsStar::SinhKhi
            | EightMansionsStar::ThienY
            | EightMansionsStar::DienNien
            | EightMansionsStar::PhucVi => Luck::Cat,
            EightMansionsStar::TuyetMenh
            | EightMansionsStar::NguQuy
            | EightMansionsStar::LucSat
            | EightMansionsStar::HoaHai => Luck::Hung,
        }
    }

    /// Vietnamese name, e.g. "Sinh Khí".
    pub const fn name_vi(self) -> &'static str {
        match self {
            EightMansionsStar::SinhKhi => "Sinh Khí",
            EightMansionsStar::ThienY => "Thiên Y",
            EightMansionsStar::DienNien => "Diên Niên",
            EightMansionsStar::PhucVi => "Phục Vị",
            EightMansionsStar::TuyetMenh => "Tuyệt Mệnh",
            EightMansionsStar::NguQuy => "Ngũ Quỷ",
            EightMansionsStar::LucSat => "Lục Sát",
            EightMansionsStar::HoaHai => "Họa Hại",
        }
    }

    /// Hán characters, e.g. "生气".
    pub const fn name_han(self) -> &'static str {
        match self {
            EightMansionsStar::SinhKhi => "生气",
            EightMansionsStar::ThienY => "天医",
            EightMansionsStar::DienNien => "延年",
            EightMansionsStar::PhucVi => "伏位",
            EightMansionsStar::TuyetMenh => "绝命",
            EightMansionsStar::NguQuy => "五鬼",
            EightMansionsStar::LucSat => "六煞",
            EightMansionsStar::HoaHai => "祸害",
        }
    }

    /// Name of the corresponding star, e.g. "Tham Lang" for Sinh Khí.
    pub const fn corresponding_star_name(self) -> &'static str {
        match self {
            EightMansionsStar::SinhKhi => "Tham Lang",
            EightMansionsStar::ThienY => "Cự Môn",
            EightMansionsStar::DienNien => "Vũ Khúc",
            EightMansionsStar::PhucVi => "Tả Phù",
            EightMansionsStar::TuyetMenh => "Phá Quân",
            EightMansionsStar::NguQuy => "Liêm Trinh",
            EightMansionsStar::LucSat => "Văn Khúc",
            EightMansionsStar::HoaHai => "Lộc Tồn",
        }
    }
}

/// The Eight Mansions matrix, as in the table of *Bát Trạch Minh Cảnh* approved by the project
/// owner: rows and columns follow [`Trigram::ALL`]. Section 3.2 of `docs/FENGSHUI_SPEC.md` orders
/// its columns by compass instead; `matrix_matches_section_3_2_by_direction` compares all 64 cells.
const EIGHT_MANSIONS: [[EightMansionsStar; 8]; 8] = {
    use EightMansionsStar::{DienNien, HoaHai, LucSat, NguQuy, PhucVi, SinhKhi, ThienY, TuyetMenh};
    [
        [
            PhucVi, TuyetMenh, ThienY, SinhKhi, LucSat, HoaHai, NguQuy, DienNien,
        ],
        [
            TuyetMenh, PhucVi, HoaHai, NguQuy, DienNien, ThienY, SinhKhi, LucSat,
        ],
        [
            ThienY, HoaHai, PhucVi, DienNien, NguQuy, TuyetMenh, LucSat, SinhKhi,
        ],
        [
            SinhKhi, NguQuy, DienNien, PhucVi, HoaHai, LucSat, TuyetMenh, ThienY,
        ],
        [
            LucSat, DienNien, NguQuy, HoaHai, PhucVi, SinhKhi, ThienY, TuyetMenh,
        ],
        [
            HoaHai, ThienY, TuyetMenh, LucSat, SinhKhi, PhucVi, DienNien, NguQuy,
        ],
        [
            NguQuy, SinhKhi, LucSat, TuyetMenh, ThienY, DienNien, PhucVi, HoaHai,
        ],
        [
            DienNien, LucSat, SinhKhi, ThienY, TuyetMenh, NguQuy, HoaHai, PhucVi,
        ],
    ]
};

/// A star of the Eight Mansions in one direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectionStar {
    /// The direction.
    pub direction: CompassDirection,
    /// The star in that direction.
    pub star: EightMansionsStar,
}

/// A person's trigram (Mệnh Quái).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct KuaInfo {
    /// Lunar birth year.
    pub birth_lunar_year: i32,
    /// Gender.
    pub gender: Gender,
    /// The number given by the formula, 1–9, before the rule of five.
    pub raw_number: u8,
    /// Lạc Thư number of the trigram, 1–4 or 6–9.
    pub number: u8,
    /// The trigram.
    pub trigram: Trigram,
    /// Group of the trigram.
    pub group: TrigramGroup,
}

/// Mệnh Quái of a person born in lunar year `birth_lunar_year`.
///
/// With `r` the year modulo 9, the number is `11 − r` for a man and `r + 4` for a woman, modulo 9
/// with 0 read as 9. The center has no trigram, so 5 becomes Khôn (2) for a man and Cấn (8) for a
/// woman. The year is the lunar one, so between Tết and Lập Xuân a man's `raw_number` can differ
/// from the year star of that day.
pub const fn kua_info(birth_lunar_year: i32, gender: Gender) -> KuaInfo {
    let remainder = birth_lunar_year.rem_euclid(9) as i64;
    let raw_number = match gender {
        Gender::Male => NineStar::from_number(11 - remainder).number(),
        Gender::Female => NineStar::from_number(remainder + 4).number(),
    };
    let trigram = match Trigram::from_number(raw_number) {
        Some(trigram) => trigram,
        None => match gender {
            Gender::Male => Trigram::Khon,
            Gender::Female => Trigram::CanTho,
        },
    };
    KuaInfo {
        birth_lunar_year,
        gender,
        raw_number,
        number: trigram.number(),
        trigram,
        group: trigram.group(),
    }
}

/// Trạch Quái of a house sitting (Tọa) towards `sitting`: a house sitting north is a Khảm house.
/// Read its directions with [`Trigram::star_at`].
pub const fn house_trigram(sitting: CompassDirection) -> Trigram {
    sitting.trigram()
}

/// Trạch Quái of a house facing `facing`; it sits the opposite way.
pub const fn house_trigram_from_facing(facing: CompassDirection) -> Trigram {
    house_trigram(facing.opposite())
}

/// Compatibility of two people by the Eight Mansions (tương hợp Bát Biến).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KuaCompatibility {
    /// The first person.
    pub first: KuaInfo,
    /// The second person.
    pub second: KuaInfo,
    /// The star between their trigrams.
    pub star: EightMansionsStar,
    /// Fortune of that star.
    pub luck: Luck,
}

/// Reads the Eight Mansions matrix at the trigrams of two people. The matrix is symmetric, so
/// their order does not matter; two people of the same group always get a good star.
pub const fn kua_compatibility(first: KuaInfo, second: KuaInfo) -> KuaCompatibility {
    let star = first.trigram.star_towards(second.trigram);
    KuaCompatibility {
        first,
        second,
        star,
        luck: star.luck(),
    }
}

/// The Nine Stars (Cửu Tinh), in the order of their numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NineStar {
    /// Nhất Bạch Thủy 一白水
    NhatBach,
    /// Nhị Hắc Thổ 二黑土
    NhiHac,
    /// Tam Bích Mộc 三碧木
    TamBich,
    /// Tứ Lục Mộc 四绿木
    TuLuc,
    /// Ngũ Hoàng Thổ 五黄土
    NguHoang,
    /// Lục Bạch Kim 六白金
    LucBach,
    /// Thất Xích Kim 七赤金
    ThatXich,
    /// Bát Bạch Thổ 八白土
    BatBach,
    /// Cửu Tử Hỏa 九紫火
    CuuTu,
}

impl NineStar {
    /// All stars in the order of their numbers, from Nhất Bạch.
    pub const ALL: [NineStar; 9] = [
        NineStar::NhatBach,
        NineStar::NhiHac,
        NineStar::TamBich,
        NineStar::TuLuc,
        NineStar::NguHoang,
        NineStar::LucBach,
        NineStar::ThatXich,
        NineStar::BatBach,
        NineStar::CuuTu,
    ];

    /// Position from 0 (Nhất Bạch) to 8 (Cửu Tử), the number minus one.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Number of the star, from 1 to 9.
    pub const fn number(self) -> u8 {
        self as u8 + 1
    }

    /// The star numbered `number`, wrapping modulo 9: 0 is Cửu Tử, 10 is Nhất Bạch.
    pub const fn from_number(number: i64) -> Self {
        Self::ALL[((number.rem_euclid(9) + 8) % 9) as usize]
    }

    /// Phase of this star in the period ruled by `period`, as given by [`van_of_year`].
    pub const fn phase_in(self, period: NineStar) -> NineStarPhase {
        let (star, period) = (self.number(), period.number());
        if star == period {
            NineStarPhase::DuongVan
        } else if star == period % 9 + 1 {
            NineStarPhase::SinhKhi
        } else if star == (period + 7) % 9 + 1 {
            NineStarPhase::ThoaiKhi
        } else {
            NineStarPhase::ThatVan
        }
    }

    /// Palace where the star belongs in the original Lạc Thư square.
    pub const fn home_palace(self) -> Palace {
        Palace::ALL[self as usize]
    }

    /// Element of the star.
    pub const fn element(self) -> FiveElements {
        match self {
            NineStar::NhatBach => FiveElements::Thuy,
            NineStar::NhiHac | NineStar::NguHoang | NineStar::BatBach => FiveElements::Tho,
            NineStar::TamBich | NineStar::TuLuc => FiveElements::Moc,
            NineStar::LucBach | NineStar::ThatXich => FiveElements::Kim,
            NineStar::CuuTu => FiveElements::Hoa,
        }
    }

    /// Fortune of the star by `LUCK_XUAN_KONG` of lunar-typescript. It is fixed, not relative to
    /// the current era.
    pub const fn luck(self) -> Luck {
        match self {
            NineStar::NhatBach
            | NineStar::TuLuc
            | NineStar::LucBach
            | NineStar::BatBach
            | NineStar::CuuTu => Luck::Cat,
            NineStar::NhiHac | NineStar::TamBich | NineStar::NguHoang | NineStar::ThatXich => {
                Luck::Hung
            }
        }
    }

    /// Vietnamese name, e.g. "Nhất Bạch Thủy".
    pub const fn name_vi(self) -> &'static str {
        match self {
            NineStar::NhatBach => "Nhất Bạch Thủy",
            NineStar::NhiHac => "Nhị Hắc Thổ",
            NineStar::TamBich => "Tam Bích Mộc",
            NineStar::TuLuc => "Tứ Lục Mộc",
            NineStar::NguHoang => "Ngũ Hoàng Thổ",
            NineStar::LucBach => "Lục Bạch Kim",
            NineStar::ThatXich => "Thất Xích Kim",
            NineStar::BatBach => "Bát Bạch Thổ",
            NineStar::CuuTu => "Cửu Tử Hỏa",
        }
    }

    /// Hán characters, e.g. "一白水".
    pub const fn name_han(self) -> &'static str {
        match self {
            NineStar::NhatBach => "一白水",
            NineStar::NhiHac => "二黑土",
            NineStar::TamBich => "三碧木",
            NineStar::TuLuc => "四绿木",
            NineStar::NguHoang => "五黄土",
            NineStar::LucBach => "六白金",
            NineStar::ThatXich => "七赤金",
            NineStar::BatBach => "八白土",
            NineStar::CuuTu => "九紫火",
        }
    }

    /// Vietnamese name of the color, e.g. "Xanh biếc".
    pub const fn color_vi(self) -> &'static str {
        match self {
            NineStar::NhatBach | NineStar::LucBach | NineStar::BatBach => "Trắng",
            NineStar::NhiHac => "Đen",
            NineStar::TamBich => "Xanh biếc",
            NineStar::TuLuc => "Xanh lục",
            NineStar::NguHoang => "Vàng",
            NineStar::ThatXich => "Đỏ",
            NineStar::CuuTu => "Tím",
        }
    }

    /// Name in Huyền Không, e.g. "Tham Lang".
    pub const fn xuan_kong_name(self) -> &'static str {
        match self {
            NineStar::NhatBach => "Tham Lang",
            NineStar::NhiHac => "Cự Môn",
            NineStar::TamBich => "Lộc Tồn",
            NineStar::TuLuc => "Văn Khúc",
            NineStar::NguHoang => "Liêm Trinh",
            NineStar::LucBach => "Vũ Khúc",
            NineStar::ThatXich => "Phá Quân",
            NineStar::BatBach => "Tả Phù",
            NineStar::CuuTu => "Hữu Bật",
        }
    }
}

/// Where a star stands in a 20-year period (D6 of `docs/INTERPRETATION_DB_SPEC.md`, after
/// *Thẩm Thị Huyền Không Học*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NineStarPhase {
    /// Đương vận: the ruling star of the period
    DuongVan,
    /// Sinh khí: the star of the next period, and only that one
    SinhKhi,
    /// Thoái khí: the star of the previous period
    ThoaiKhi,
    /// Thất vận: every other star
    ThatVan,
}

impl NineStarPhase {
    /// All phases.
    pub const ALL: [NineStarPhase; 4] = [
        NineStarPhase::DuongVan,
        NineStarPhase::SinhKhi,
        NineStarPhase::ThoaiKhi,
        NineStarPhase::ThatVan,
    ];

    /// Vietnamese name, e.g. "Thoái khí".
    pub const fn name_vi(self) -> &'static str {
        match self {
            NineStarPhase::DuongVan => "Đương vận",
            NineStarPhase::SinhKhi => "Sinh khí",
            NineStarPhase::ThoaiKhi => "Thoái khí",
            NineStarPhase::ThatVan => "Thất vận",
        }
    }

    /// Whether the auspicious text of the star applies: every phase but Thất vận.
    pub const fn is_auspicious(self) -> bool {
        !matches!(self, NineStarPhase::ThatVan)
    }
}

/// The three 60-year eras of a 180-year cycle (Tam Nguyên).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TamNguyen {
    /// Thượng Nguyên, periods 1–3
    ThuongNguyen,
    /// Trung Nguyên, periods 4–6
    TrungNguyen,
    /// Hạ Nguyên, periods 7–9
    HaNguyen,
}

impl TamNguyen {
    /// All eras in cycle order, from Thượng Nguyên.
    pub const ALL: [TamNguyen; 3] = [
        TamNguyen::ThuongNguyen,
        TamNguyen::TrungNguyen,
        TamNguyen::HaNguyen,
    ];

    /// Vietnamese name, e.g. "Hạ Nguyên".
    pub const fn name_vi(self) -> &'static str {
        match self {
            TamNguyen::ThuongNguyen => "Thượng Nguyên",
            TamNguyen::TrungNguyen => "Trung Nguyên",
            TamNguyen::HaNguyen => "Hạ Nguyên",
        }
    }
}

/// First year of the Thượng Nguyên that opens a cycle.
const CYCLE_START_YEAR: i64 = 1864;

/// Era (Nguyên) of year `year`, as `LunarYear.getYuan`: 1984–2043 is Hạ Nguyên.
pub const fn nguyen_of_year(year: i32) -> TamNguyen {
    TamNguyen::ALL[(year as i64 - CYCLE_START_YEAR)
        .div_euclid(60)
        .rem_euclid(3) as usize]
}

/// Ruling star of the 20-year period (Vận) of year `year`, as `LunarYear.getYun`: its number is
/// the number of the period, so 2024–2043 is Cửu Tử, period 9.
pub const fn van_of_year(year: i32) -> NineStar {
    NineStar::from_number((year as i64 - CYCLE_START_YEAR).div_euclid(20) + 1)
}

/// Year star (Niên Tinh) of year `year`: 2024 is Tam Bích. This is the closed form of
/// `_getYearNineStar` of lunar-typescript; `year` is the year chosen by [`NineStarBoundary`].
pub const fn star_of_year(year: i32) -> NineStar {
    NineStar::from_number(11 - year as i64)
}

/// Where the year star, the month star and the era change: the `sect` argument of
/// `Lunar.getYearNineStar` and `Lunar.getMonthNineStar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NineStarBoundary {
    /// `sect = 1`: the year changes at Tết, the month on the day of each Tiết. The year comes from
    /// [`solar_to_lunar`], so the days before Tết 1600 are out of range, and the result differs
    /// from the source wherever the two lunar calendars do, such as on 3 February 1916 at UTC+8.
    LunarNewYear,
    /// `sect = 2`: the year changes on the day of Lập Xuân, the month on the day of each Tiết.
    /// The whole day of a Tiết already belongs to the new month.
    #[default]
    LapXuanDay,
    /// `sect = 3`: the year and the month change at the exact instants of Lập Xuân and each Tiết,
    /// compared unrounded as in [`four_pillars`](crate::four_pillars). The source can differ
    /// within the seconds that separate the two engines' instants, 38 at most from 1900 to 2100.
    LapXuanExact,
}

/// A star entering the center palace, with the direction it flies in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlyingStar {
    /// The star.
    pub star: NineStar,
    /// Whether its sequence ascends (thuận, yang) rather than descends (nghịch, yin).
    pub forward: bool,
}

/// The stars ruling one moment. The year and month stars always fly forward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NineStars {
    /// Era (Nguyên).
    pub nguyen: TamNguyen,
    /// Ruling star of the 20-year period (Vận).
    pub van: NineStar,
    /// Year star (Niên Tinh).
    pub year: NineStar,
    /// Month star (Nguyệt Tinh).
    pub month: NineStar,
    /// Day star (Nhật Tinh).
    pub day: FlyingStar,
    /// Hour star (Thời Tinh).
    pub hour: FlyingStar,
}

/// Era, period and stars of wall-clock time `time` in a zone `tz_offset` hours east of UTC. The
/// era and the period use the same year as the year star. For true solar time, pass it to
/// [`day_nine_star`] and [`hour_nine_star`] only: the other stars follow the clock, like the year
/// and month pillars.
pub fn nine_stars(
    time: SolarDateTime,
    tz_offset: f64,
    boundary: NineStarBoundary,
) -> Result<NineStars, AstroError> {
    check_time(time, tz_offset)?;
    let year = ruling_year(time, tz_offset, boundary)?;
    Ok(NineStars {
        nguyen: nguyen_of_year(year),
        van: van_of_year(year),
        year: star_of_year(year),
        month: month_star(year, ruling_month_branch(time, tz_offset, boundary)),
        day: day_star(time.date(), tz_offset),
        hour: hour_star(time, tz_offset),
    })
}

/// Year star (Niên Tinh) at `time`, as `Lunar.getYearNineStar`.
pub fn year_nine_star(
    time: SolarDateTime,
    tz_offset: f64,
    boundary: NineStarBoundary,
) -> Result<NineStar, AstroError> {
    check_time(time, tz_offset)?;
    Ok(star_of_year(ruling_year(time, tz_offset, boundary)?))
}

/// Month star (Nguyệt Tinh) at `time`, as `Lunar.getMonthNineStar`: the Dần month opens with
/// Bát Bạch in a Tý, Ngọ, Mão or Dậu year, Ngũ Hoàng in a Thìn, Tuất, Sửu or Mùi year and Nhị Hắc
/// in a Dần, Thân, Tỵ or Hợi year, then each month counts down. The Tý and Sửu months keep the
/// year of the preceding Dần month, except that with [`NineStarBoundary::LunarNewYear`] a Sửu month
/// after Tết takes the new lunar year.
pub fn month_nine_star(
    time: SolarDateTime,
    tz_offset: f64,
    boundary: NineStarBoundary,
) -> Result<NineStar, AstroError> {
    check_time(time, tz_offset)?;
    let year = ruling_year(time, tz_offset, boundary)?;
    Ok(month_star(
        year,
        ruling_month_branch(time, tz_offset, boundary),
    ))
}

/// Day star (Nhật Tinh) of `date`, as `Lunar.getDayNineStar`.
///
/// The count turns at the Giáp Tý days nearest the solstices: it rises from Nhất Bạch after the
/// one near Đông Chí and falls from Cửu Tử after the one near Hạ Chí, so the day star can change
/// direction up to 30 days away from the solstice that the hour star turns on.
///
/// The turning points come from the Gregorian year of `date`, a trait of the source that is kept
/// on purpose to stay identical to it. Two turning points are usually 180 days apart, which the
/// nine-day cycle divides, and the count runs on. When the one near Đông Chí falls in January
/// after a gap of 240 days, the days from 1 January up to it count down to Nhất Bạch instead of
/// continuing the old count, so the star jumps on 1 January: 31 December 2008 is 4 and 1 January
/// 2009 is 9, where counting on would give 3. From 1900 to 2100 this happens 12 times and touches
/// 225 days at UTC+8, and 13 times and 241 days at UTC+7, where the turning point of Hạ Chí 2077
/// moves and adds 1 January 2078.
pub fn day_nine_star(date: SolarDate, tz_offset: f64) -> Result<FlyingStar, AstroError> {
    date.validate()?;
    check_year(date.year)?;
    check_tz_offset(tz_offset)?;
    Ok(day_star(date, tz_offset))
}

/// Hour star (Thời Tinh) at `time`, as `Lunar.getTimeNineStar`, not `LunarTime.getNineStar`, which
/// lacks the days from Đông Chí to 31 December.
///
/// The sequence ascends from the day of Đông Chí and descends from the day of Hạ Chí. During giờ
/// Tý Sớm, 23:00–23:59, the hour is Tý but the branch of the day stays that of the civil date,
/// unlike the hour pillar.
pub fn hour_nine_star(time: SolarDateTime, tz_offset: f64) -> Result<FlyingStar, AstroError> {
    check_time(time, tz_offset)?;
    Ok(hour_star(time, tz_offset))
}

/// Stars of the nine palaces of a Lo Shu grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LoShuGrid {
    /// Whether the stars flew forward (thuận) rather than backward (nghịch).
    pub forward: bool,
    /// Star in Khảm, north.
    pub kham: NineStar,
    /// Star in Khôn, south-west.
    pub khon: NineStar,
    /// Star in Chấn, east.
    pub chan: NineStar,
    /// Star in Tốn, south-east.
    pub ton: NineStar,
    /// Star in the center.
    pub trung_cung: NineStar,
    /// Star in Càn, north-west.
    pub can_kim: NineStar,
    /// Star in Đoài, west.
    pub doai: NineStar,
    /// Star in Cấn, north-east.
    pub can_tho: NineStar,
    /// Star in Ly, south.
    pub ly: NineStar,
}

impl LoShuGrid {
    /// Star in `palace`.
    pub const fn star_at(&self, palace: Palace) -> NineStar {
        match palace {
            Palace::Kham => self.kham,
            Palace::Khon => self.khon,
            Palace::Chan => self.chan,
            Palace::Ton => self.ton,
            Palace::TrungCung => self.trung_cung,
            Palace::CanKim => self.can_kim,
            Palace::Doai => self.doai,
            Palace::CanTho => self.can_tho,
            Palace::Ly => self.ly,
        }
    }

    /// The 3×3 layout with south on top: Tốn, Ly, Khôn; Chấn, center, Đoài; Cấn, Khảm, Càn.
    pub const fn rows(&self) -> [[NineStar; 3]; 3] {
        [
            [self.ton, self.ly, self.khon],
            [self.chan, self.trung_cung, self.doai],
            [self.can_tho, self.kham, self.can_kim],
        ]
    }
}

/// Lo Shu grid with `center_star` in the center. The stars follow the Lạc Thư path, center → Càn
/// → Đoài → Cấn → Ly → Khảm → Khôn → Chấn → Tốn, each one higher when `forward`, each one lower
/// otherwise. Year and month grids pass `true`; day and hour grids pass
/// [`FlyingStar::forward`].
pub const fn generate_loshu_grid(center_star: NineStar, forward: bool) -> LoShuGrid {
    LoShuGrid {
        forward,
        kham: flown_star(center_star, forward, Palace::Kham),
        khon: flown_star(center_star, forward, Palace::Khon),
        chan: flown_star(center_star, forward, Palace::Chan),
        ton: flown_star(center_star, forward, Palace::Ton),
        trung_cung: center_star,
        can_kim: flown_star(center_star, forward, Palace::CanKim),
        doai: flown_star(center_star, forward, Palace::Doai),
        can_tho: flown_star(center_star, forward, Palace::CanTho),
        ly: flown_star(center_star, forward, Palace::Ly),
    }
}

/// Star that reaches `palace` from the center.
const fn flown_star(center_star: NineStar, forward: bool, palace: Palace) -> NineStar {
    let steps = palace.number() as i64 - Palace::TrungCung.number() as i64;
    let center = center_star.number() as i64;
    NineStar::from_number(if forward {
        center + steps
    } else {
        center - steps
    })
}

fn check_time(time: SolarDateTime, tz_offset: f64) -> Result<(), AstroError> {
    time.validate()?;
    check_year(time.year)?;
    check_tz_offset(tz_offset)
}

/// JDN of the local day on which `term` begins in Gregorian year `year`.
fn term_day(term: SolarTerm, year: i32, tz_offset: f64) -> i32 {
    local_jdn(term_instant(term, year), tz_offset)
}

/// Year ruling `time` for the year star, the month star, the era and the period.
fn ruling_year(
    time: SolarDateTime,
    tz_offset: f64,
    boundary: NineStarBoundary,
) -> Result<i32, AstroError> {
    let before_lap_xuan = match boundary {
        NineStarBoundary::LunarNewYear => {
            return Ok(solar_to_lunar(time.day, time.month, time.year, tz_offset)?.year);
        }
        NineStarBoundary::LapXuanDay => {
            time.date().to_jdn() < term_day(SolarTerm::LapXuan, time.year, tz_offset)
        }
        NineStarBoundary::LapXuanExact => {
            time.to_julian_day(tz_offset) < term_instant(SolarTerm::LapXuan, time.year)
        }
    };
    Ok(time.year - i32::from(before_lap_xuan))
}

/// Branch of the solar month ruling `time` for the month star.
fn ruling_month_branch(time: SolarDateTime, tz_offset: f64, boundary: NineStarBoundary) -> Branch {
    match boundary {
        NineStarBoundary::LunarNewYear | NineStarBoundary::LapXuanDay => {
            solar_month_branch(time.date().to_jdn(), tz_offset)
        }
        NineStarBoundary::LapXuanExact => solar_month_branch_at(time.to_julian_day(tz_offset)),
    }
}

/// Month star of the solar month `month_branch` in year `year`.
const fn month_star(year: i32, month_branch: Branch) -> NineStar {
    let dan_month_star = match year_can_chi(year).branch.index() % 3 {
        0 => 8,
        1 => 5,
        _ => 2,
    };
    let months_since_dan =
        (month_branch.index() as i64 - Branch::Dan.index() as i64).rem_euclid(12);
    NineStar::from_number(dan_month_star - months_since_dan)
}

/// Giáp Tý day nearest the local day of `term` in Gregorian year `year`. At 30 days either way
/// the source takes the later one.
fn solstice_anchor(term: SolarTerm, year: i32, tz_offset: f64) -> i32 {
    let solstice = term_day(term, year, tz_offset);
    let cycle_index = i32::from(day_can_chi(solstice).cycle_index());
    if cycle_index > 29 {
        solstice + 60 - cycle_index
    } else {
        solstice - cycle_index
    }
}

/// Turning points of the day stars of Gregorian year `year`: near the last Đông Chí, near Hạ Chí
/// and near the next Đông Chí.
fn turning_points(year: i32, tz_offset: f64) -> [i32; 3] {
    [
        solstice_anchor(SolarTerm::DongChi, year - 1, tz_offset),
        solstice_anchor(SolarTerm::HaChi, year, tz_offset),
        solstice_anchor(SolarTerm::DongChi, year, tz_offset),
    ]
}

/// Unchecked [`day_nine_star`].
fn day_star(date: SolarDate, tz_offset: f64) -> FlyingStar {
    day_star_between(date.to_jdn(), turning_points(date.year, tz_offset))
}

/// Day star of `day`, given the [`turning_points`] of its Gregorian year.
const fn day_star_between(day: i32, [last_winter, summer, next_winter]: [i32; 3]) -> FlyingStar {
    let (number, forward) = if day < last_winter {
        (9 + (last_winter - day), false)
    } else if day < summer {
        (1 + (day - last_winter), true)
    } else if day < next_winter {
        (9 - (day - summer), false)
    } else {
        (1 + (day - next_winter), true)
    };
    FlyingStar {
        star: NineStar::from_number(number as i64),
        forward,
    }
}

/// Unchecked [`hour_nine_star`].
fn hour_star(time: SolarDateTime, tz_offset: f64) -> FlyingStar {
    let day = time.date().to_jdn();
    let forward = hour_stars_rise(
        day,
        term_day(SolarTerm::HaChi, time.year, tz_offset),
        term_day(SolarTerm::DongChi, time.year, tz_offset),
    );
    FlyingStar {
        star: hour_star_of_branches(
            day_can_chi(day).branch,
            Branch::from_hour(time.hour),
            forward,
        ),
        forward,
    }
}

/// Whether the hour stars of `day` ascend, given the local days of Hạ Chí and Đông Chí of its
/// Gregorian year.
const fn hour_stars_rise(day: i32, ha_chi: i32, dong_chi: i32) -> bool {
    day < ha_chi || day >= dong_chi
}

/// Star of the Tý hour by day branch modulo 3, when the hour stars ascend: Tý, Ngọ, Mão and Dậu
/// days, then Thìn, Tuất, Sửu and Mùi days, then Dần, Thân, Tỵ and Hợi days.
const TY_HOUR_STARS_RISING: [i64; 3] = [1, 4, 7];

/// Star of the Tý hour by day branch modulo 3, when the hour stars descend.
const TY_HOUR_STARS_FALLING: [i64; 3] = [9, 6, 3];

/// Hour star of the double-hour `hour_branch` on a day of branch `day_branch`.
const fn hour_star_of_branches(day_branch: Branch, hour_branch: Branch, forward: bool) -> NineStar {
    let group = day_branch.index() as usize % 3;
    let hour = hour_branch.index() as i64;
    NineStar::from_number(if forward {
        TY_HOUR_STARS_RISING[group] + hour
    } else {
        TY_HOUR_STARS_FALLING[group] - hour
    })
}

#[cfg(test)]
mod tests {
    use super::CompassDirection::{Bac, Dong, DongBac, DongNam, Nam, Tay, TayBac, TayNam};
    use super::EightMansionsStar::{
        DienNien, HoaHai, LucSat, NguQuy, PhucVi, SinhKhi, ThienY, TuyetMenh,
    };
    use super::NineStar::{
        BatBach, CuuTu, LucBach, NguHoang, NhatBach, NhiHac, TamBich, ThatXich, TuLuc,
    };
    use super::NineStarBoundary::{LapXuanDay, LapXuanExact, LunarNewYear};
    use super::TamNguyen::{HaNguyen, ThuongNguyen, TrungNguyen};
    use super::Trigram::{CanKim, CanTho, Chan, Doai, Kham, Khon, Ly, Ton};
    use super::*;
    use crate::test_support::{CN, VN, date, date_time};
    use crate::{MAX_YEAR, MIN_YEAR};

    const MALE: Gender = Gender::Male;
    const FEMALE: Gender = Gender::Female;

    /// 1900-02-01 and 2100-12-01, the first and last of the 73 353 days compared with
    /// lunar-typescript. Section 5 of the spec gives the recipe of the digests below.
    const SOURCE_DAYS: (i32, i32) = (2_415_052, 2_488_404);
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;

    /// One FNV-1a step over the number of a star.
    fn fold(digest: u64, star: NineStar) -> u64 {
        (digest ^ u64::from(star.number())).wrapping_mul(FNV_PRIME)
    }

    /// Noon of the local day `jdn`.
    fn noon(jdn: i32) -> SolarDateTime {
        let day = SolarDate::from_jdn(jdn);
        date_time(day.year, day.month, day.day, 12, 0)
    }

    const fn rising(star: NineStar) -> FlyingStar {
        FlyingStar {
            star,
            forward: true,
        }
    }

    const fn falling(star: NineStar) -> FlyingStar {
        FlyingStar {
            star,
            forward: false,
        }
    }

    fn kua(year: i32, gender: Gender) -> (u8, Trigram) {
        let info = kua_info(year, gender);
        (info.raw_number, info.trigram)
    }

    fn numbers(grid: LoShuGrid) -> [[u8; 3]; 3] {
        grid.rows().map(|row| row.map(NineStar::number))
    }

    #[test]
    fn phases_of_period_nine_follow_d6() {
        use super::NineStarPhase::{DuongVan, SinhKhi, ThatVan, ThoaiKhi};
        assert_eq!(CuuTu.phase_in(CuuTu), DuongVan);
        assert_eq!(NhatBach.phase_in(CuuTu), SinhKhi);
        assert_eq!(BatBach.phase_in(CuuTu), ThoaiKhi);
        assert_eq!(NhiHac.phase_in(CuuTu), ThatVan);
        assert_eq!(NguHoang.phase_in(CuuTu), ThatVan);
    }

    #[test]
    fn every_period_has_one_star_of_each_phase_and_six_out_of_time() {
        use super::{NineStar, NineStarPhase};
        for period in NineStar::ALL {
            let count = |phase| {
                NineStar::ALL
                    .iter()
                    .filter(|star| star.phase_in(period) == phase)
                    .count()
            };
            assert_eq!(count(NineStarPhase::DuongVan), 1);
            assert_eq!(count(NineStarPhase::SinhKhi), 1);
            assert_eq!(count(NineStarPhase::ThoaiKhi), 1);
            assert_eq!(count(NineStarPhase::ThatVan), 6);
        }
    }

    #[test]
    fn the_cycle_wraps_at_both_ends() {
        use super::NineStarPhase::{SinhKhi, ThoaiKhi};
        assert_eq!(NhatBach.phase_in(CuuTu), SinhKhi);
        assert_eq!(CuuTu.phase_in(NhatBach), ThoaiKhi);
    }

    #[test]
    fn every_phase_but_that_van_uses_the_auspicious_text() {
        let good = super::NineStarPhase::ALL.map(|phase| phase.is_auspicious());
        assert_eq!(good, [true, true, true, false]);
    }

    #[test]
    fn palaces_follow_the_lo_shu_square() {
        let table = [
            (Palace::Kham, "Khảm", "坎", Some(Bac), FiveElements::Thuy),
            (Palace::Khon, "Khôn", "坤", Some(TayNam), FiveElements::Tho),
            (Palace::Chan, "Chấn", "震", Some(Dong), FiveElements::Moc),
            (Palace::Ton, "Tốn", "巽", Some(DongNam), FiveElements::Moc),
            (
                Palace::TrungCung,
                "Trung Cung",
                "中",
                None,
                FiveElements::Tho,
            ),
            (Palace::CanKim, "Càn", "乾", Some(TayBac), FiveElements::Kim),
            (Palace::Doai, "Đoài", "兑", Some(Tay), FiveElements::Kim),
            (
                Palace::CanTho,
                "Cấn",
                "艮",
                Some(DongBac),
                FiveElements::Tho,
            ),
            (Palace::Ly, "Ly", "离", Some(Nam), FiveElements::Hoa),
        ];
        for (index, (palace, name, han, direction, element)) in table.into_iter().enumerate() {
            assert_eq!(Palace::ALL[index], palace);
            assert_eq!(usize::from(palace.index()), index);
            assert_eq!(usize::from(palace.number()), index + 1);
            assert_eq!(palace.name_vi(), name);
            assert_eq!(palace.name_han(), han);
            assert_eq!(palace.direction(), direction);
            assert_eq!(palace.element(), element);
        }
    }

    #[test]
    fn trigrams_palaces_and_directions_correspond() {
        assert_eq!(Palace::TrungCung.trigram(), None);
        for number in [0, 5, 10] {
            assert_eq!(Trigram::from_number(number), None);
        }
        assert_eq!(Trigram::ALL.map(Trigram::number), [1, 2, 3, 4, 6, 7, 8, 9]);
        for trigram in Trigram::ALL {
            assert_eq!(Trigram::from_number(trigram.number()), Some(trigram));
            assert_eq!(trigram.palace().trigram(), Some(trigram));
            assert_eq!(trigram.direction().trigram(), trigram);
            assert_eq!(trigram.palace().element(), trigram.element());
            assert_eq!(trigram.palace().name_vi(), trigram.name_vi());
            assert_eq!(trigram.palace().name_han(), trigram.name_han());
        }
    }

    #[test]
    fn trigrams_fall_into_two_groups_of_four() {
        for trigram in [Kham, Ly, Chan, Ton] {
            assert_eq!(trigram.group(), TrigramGroup::DongTu);
        }
        for trigram in [CanKim, Khon, CanTho, Doai] {
            assert_eq!(trigram.group(), TrigramGroup::TayTu);
        }
        assert_eq!(TrigramGroup::DongTu.name_vi(), "Đông Tứ");
        assert_eq!(TrigramGroup::TayTu.name_vi(), "Tây Tứ");
    }

    #[test]
    fn directions_run_clockwise_and_oppose_in_pairs() {
        assert_eq!(
            CompassDirection::ALL.map(CompassDirection::name_vi),
            [
                "Bắc",
                "Đông Bắc",
                "Đông",
                "Đông Nam",
                "Nam",
                "Tây Nam",
                "Tây",
                "Tây Bắc"
            ]
        );
        for (direction, opposite) in [
            (Bac, Nam),
            (DongBac, TayNam),
            (Dong, Tay),
            (DongNam, TayBac),
        ] {
            assert_eq!(direction.opposite(), opposite);
            assert_eq!(opposite.opposite(), direction);
        }
    }

    #[test]
    fn kua_of_the_landmark_years() {
        assert_eq!(kua(1990, MALE), (1, Kham));
        assert_eq!(kua(1990, FEMALE), (5, CanTho));
        assert_eq!(kua(2000, MALE), (9, Ly));
        assert_eq!(kua(2000, FEMALE), (6, CanKim));
    }

    #[test]
    fn rule_of_five_sends_men_to_khon_and_women_to_can_tho() {
        for year in [1986, 1995, 2004, 2013, 2022] {
            assert_eq!(kua(year, MALE), (5, Khon));
        }
        for year in [1981, 1990, 1999, 2008, 2017] {
            assert_eq!(kua(year, FEMALE), (5, CanTho));
        }
        let info = kua_info(2022, MALE);
        assert_eq!((info.number, info.group), (2, TrigramGroup::TayTu));
    }

    #[test]
    fn kua_by_remainder_of_the_year() {
        let table = [
            ((2, Khon), (4, Ton)),
            ((1, Kham), (5, CanTho)),
            ((9, Ly), (6, CanKim)),
            ((8, CanTho), (7, Doai)),
            ((7, Doai), (8, CanTho)),
            ((6, CanKim), (9, Ly)),
            ((5, Khon), (1, Kham)),
            ((4, Ton), (2, Khon)),
            ((3, Chan), (3, Chan)),
        ];
        for (remainder, (male, female)) in table.into_iter().enumerate() {
            let year = 1800 + remainder as i32;
            assert_eq!(year.rem_euclid(9) as usize, remainder);
            assert_eq!(kua(year, MALE), male);
            assert_eq!(kua(year, FEMALE), female);
        }
    }

    #[test]
    fn kua_agrees_with_the_digit_sums_of_both_centuries() {
        let digit_root = |value: i32| {
            let mut value = value;
            while value > 9 {
                value = value / 10 + value % 10;
            }
            value
        };
        let wrap = |value: i32| ((value - 1).rem_euclid(9) + 1) as u8;
        for year in 1900..=2099 {
            let sum = digit_root(year % 100);
            let (male, female) = if year < 2000 {
                (10 - sum, 5 + sum)
            } else {
                (9 - sum, 6 + sum)
            };
            assert_eq!(kua_info(year, MALE).raw_number, wrap(male), "{year}");
            assert_eq!(kua_info(year, FEMALE).raw_number, wrap(female), "{year}");
        }
    }

    #[test]
    fn a_man_raw_number_is_the_year_star() {
        for year in 1600..=2400 {
            assert_eq!(kua_info(year, MALE).raw_number, star_of_year(year).number());
        }
    }

    #[test]
    fn matrix_matches_section_3_2_by_direction() {
        // Columns: Bắc, Đông Bắc, Đông, Đông Nam, Nam, Tây Nam, Tây, Tây Bắc.
        let table = [
            (
                Kham,
                [
                    PhucVi, NguQuy, ThienY, SinhKhi, DienNien, TuyetMenh, HoaHai, LucSat,
                ],
            ),
            (
                Khon,
                [
                    TuyetMenh, SinhKhi, HoaHai, NguQuy, LucSat, PhucVi, ThienY, DienNien,
                ],
            ),
            (
                Chan,
                [
                    ThienY, LucSat, PhucVi, DienNien, SinhKhi, HoaHai, TuyetMenh, NguQuy,
                ],
            ),
            (
                Ton,
                [
                    SinhKhi, TuyetMenh, DienNien, PhucVi, ThienY, NguQuy, LucSat, HoaHai,
                ],
            ),
            (
                CanKim,
                [
                    LucSat, ThienY, NguQuy, HoaHai, TuyetMenh, DienNien, SinhKhi, PhucVi,
                ],
            ),
            (
                Doai,
                [
                    HoaHai, DienNien, TuyetMenh, LucSat, NguQuy, ThienY, PhucVi, SinhKhi,
                ],
            ),
            (
                CanTho,
                [
                    NguQuy, PhucVi, LucSat, TuyetMenh, HoaHai, SinhKhi, DienNien, ThienY,
                ],
            ),
            (
                Ly,
                [
                    DienNien, HoaHai, SinhKhi, ThienY, PhucVi, LucSat, NguQuy, TuyetMenh,
                ],
            ),
        ];
        for (trigram, stars) in table {
            assert_eq!(CompassDirection::ALL.map(|d| trigram.star_at(d)), stars);
            let chart = trigram.chart();
            assert_eq!(chart.map(|cell| cell.star), stars);
            assert_eq!(chart.map(|cell| cell.direction), CompassDirection::ALL);
        }
    }

    #[test]
    fn matrix_is_symmetric() {
        for a in Trigram::ALL {
            for b in Trigram::ALL {
                assert_eq!(a.star_towards(b), b.star_towards(a), "{a:?} and {b:?}");
            }
        }
    }

    #[test]
    fn every_row_holds_each_star_once() {
        for trigram in Trigram::ALL {
            for star in EightMansionsStar::ALL {
                let count = Trigram::ALL
                    .iter()
                    .filter(|other| trigram.star_towards(**other) == star)
                    .count();
                assert_eq!(count, 1, "{star:?} of {trigram:?}");
            }
        }
    }

    #[test]
    fn matrix_follows_the_changed_lines_of_the_trigrams() {
        // Lines from the bottom, 1 for yang (section 3.3).
        let lines = |trigram: Trigram| match trigram {
            CanKim => 0b111,
            Doai => 0b110,
            Ly => 0b101,
            Chan => 0b100,
            Ton => 0b011,
            Kham => 0b010,
            CanTho => 0b001,
            Khon => 0b000,
        };
        let star_of_changed_lines = |changed: u8| match changed {
            0b001 => SinhKhi,
            0b110 => ThienY,
            0b111 => DienNien,
            0b000 => PhucVi,
            0b010 => TuyetMenh,
            0b011 => NguQuy,
            0b101 => LucSat,
            0b100 => HoaHai,
            _ => unreachable!("three lines"),
        };
        for a in Trigram::ALL {
            for b in Trigram::ALL {
                assert_eq!(
                    a.star_towards(b),
                    star_of_changed_lines(lines(a) ^ lines(b)),
                    "{a:?} and {b:?}"
                );
            }
        }
    }

    #[test]
    fn good_stars_stay_within_the_group() {
        for a in Trigram::ALL {
            for b in Trigram::ALL {
                let same_group = a.group() == b.group();
                assert_eq!(a.star_towards(b).luck() == Luck::Cat, same_group);
            }
        }
    }

    #[test]
    fn star_names_and_fortune() {
        let table = [
            (SinhKhi, "Sinh Khí", "生气", "Tham Lang", Luck::Cat),
            (ThienY, "Thiên Y", "天医", "Cự Môn", Luck::Cat),
            (DienNien, "Diên Niên", "延年", "Vũ Khúc", Luck::Cat),
            (PhucVi, "Phục Vị", "伏位", "Tả Phù", Luck::Cat),
            (TuyetMenh, "Tuyệt Mệnh", "绝命", "Phá Quân", Luck::Hung),
            (NguQuy, "Ngũ Quỷ", "五鬼", "Liêm Trinh", Luck::Hung),
            (LucSat, "Lục Sát", "六煞", "Văn Khúc", Luck::Hung),
            (HoaHai, "Họa Hại", "祸害", "Lộc Tồn", Luck::Hung),
        ];
        for (index, (star, name, han, corresponding, luck)) in table.into_iter().enumerate() {
            assert_eq!(EightMansionsStar::ALL[index], star);
            assert_eq!(usize::from(star.index()), index);
            assert_eq!(star.name_vi(), name);
            assert_eq!(star.name_han(), han);
            assert_eq!(star.corresponding_star_name(), corresponding);
            assert_eq!(star.luck(), luck);
        }
    }

    #[test]
    fn house_trigram_follows_the_sitting_direction() {
        assert_eq!(house_trigram(Bac), Kham);
        assert_eq!(house_trigram_from_facing(Nam), Kham);
        assert_eq!(house_trigram_from_facing(DongNam), CanKim);
        for direction in CompassDirection::ALL {
            assert_eq!(house_trigram(direction), direction.trigram());
            assert_eq!(
                house_trigram_from_facing(direction),
                house_trigram(direction.opposite())
            );
        }
        // A Khảm house has its Phục Vị behind it and its Diên Niên in front of it.
        let kham_house = house_trigram(Bac);
        assert_eq!(kham_house.star_at(Bac), PhucVi);
        assert_eq!(kham_house.star_at(Nam), DienNien);
    }

    #[test]
    fn compatibility_reads_the_matrix_at_two_people() {
        let man_1990 = kua_info(1990, MALE);
        let woman_1990 = kua_info(1990, FEMALE);
        let man_2000 = kua_info(2000, MALE);
        let woman_2000 = kua_info(2000, FEMALE);

        let couple = kua_compatibility(man_1990, woman_1990);
        assert_eq!((couple.star, couple.luck), (NguQuy, Luck::Hung));
        assert_eq!((couple.first, couple.second), (man_1990, woman_1990));

        let couple = kua_compatibility(man_1990, man_2000);
        assert_eq!((couple.star, couple.luck), (DienNien, Luck::Cat));

        let couple = kua_compatibility(woman_1990, woman_2000);
        assert_eq!((couple.star, couple.luck), (ThienY, Luck::Cat));

        let couple = kua_compatibility(man_1990, man_1990);
        assert_eq!((couple.star, couple.luck), (PhucVi, Luck::Cat));

        assert_eq!(
            kua_compatibility(man_2000, woman_2000).star,
            kua_compatibility(woman_2000, man_2000).star
        );
    }

    #[test]
    fn nine_star_table() {
        let table = [
            (
                NhatBach,
                "Nhất Bạch Thủy",
                "一白水",
                FiveElements::Thuy,
                "Trắng",
                Palace::Kham,
                "Tham Lang",
                Luck::Cat,
            ),
            (
                NhiHac,
                "Nhị Hắc Thổ",
                "二黑土",
                FiveElements::Tho,
                "Đen",
                Palace::Khon,
                "Cự Môn",
                Luck::Hung,
            ),
            (
                TamBich,
                "Tam Bích Mộc",
                "三碧木",
                FiveElements::Moc,
                "Xanh biếc",
                Palace::Chan,
                "Lộc Tồn",
                Luck::Hung,
            ),
            (
                TuLuc,
                "Tứ Lục Mộc",
                "四绿木",
                FiveElements::Moc,
                "Xanh lục",
                Palace::Ton,
                "Văn Khúc",
                Luck::Cat,
            ),
            (
                NguHoang,
                "Ngũ Hoàng Thổ",
                "五黄土",
                FiveElements::Tho,
                "Vàng",
                Palace::TrungCung,
                "Liêm Trinh",
                Luck::Hung,
            ),
            (
                LucBach,
                "Lục Bạch Kim",
                "六白金",
                FiveElements::Kim,
                "Trắng",
                Palace::CanKim,
                "Vũ Khúc",
                Luck::Cat,
            ),
            (
                ThatXich,
                "Thất Xích Kim",
                "七赤金",
                FiveElements::Kim,
                "Đỏ",
                Palace::Doai,
                "Phá Quân",
                Luck::Hung,
            ),
            (
                BatBach,
                "Bát Bạch Thổ",
                "八白土",
                FiveElements::Tho,
                "Trắng",
                Palace::CanTho,
                "Tả Phù",
                Luck::Cat,
            ),
            (
                CuuTu,
                "Cửu Tử Hỏa",
                "九紫火",
                FiveElements::Hoa,
                "Tím",
                Palace::Ly,
                "Hữu Bật",
                Luck::Cat,
            ),
        ];
        for (index, (star, name, han, element, color, palace, xuan_kong, luck)) in
            table.into_iter().enumerate()
        {
            assert_eq!(NineStar::ALL[index], star);
            assert_eq!(usize::from(star.index()), index);
            assert_eq!(usize::from(star.number()), index + 1);
            assert_eq!(star.name_vi(), name);
            assert_eq!(star.name_han(), han);
            assert_eq!(star.element(), element);
            assert_eq!(star.color_vi(), color);
            assert_eq!(star.home_palace(), palace);
            assert_eq!(star.xuan_kong_name(), xuan_kong);
            assert_eq!(star.luck(), luck);
        }
    }

    #[test]
    fn star_numbers_wrap_modulo_nine() {
        assert_eq!(NineStar::from_number(1), NhatBach);
        assert_eq!(NineStar::from_number(9), CuuTu);
        assert_eq!(NineStar::from_number(0), CuuTu);
        assert_eq!(NineStar::from_number(10), NhatBach);
        assert_eq!(NineStar::from_number(-2), ThatXich);
        assert_eq!(NineStar::from_number(i64::MIN), NhatBach);
        assert_eq!(NineStar::from_number(i64::MAX), ThatXich);
    }

    #[test]
    fn eras_and_periods() {
        let table = [
            (1864, ThuongNguyen, NhatBach),
            (1884, ThuongNguyen, NhiHac),
            (1904, ThuongNguyen, TamBich),
            (1924, TrungNguyen, TuLuc),
            (1944, TrungNguyen, NguHoang),
            (1964, TrungNguyen, LucBach),
            (1984, HaNguyen, ThatXich),
            (2004, HaNguyen, BatBach),
            (2024, HaNguyen, CuuTu),
        ];
        for (first_year, nguyen, van) in table {
            for year in [first_year, first_year + 19] {
                assert_eq!(nguyen_of_year(year), nguyen, "{year}");
                assert_eq!(van_of_year(year), van, "{year}");
            }
            // One 180-year cycle earlier and later.
            assert_eq!(van_of_year(first_year - 180), van);
            assert_eq!(nguyen_of_year(first_year + 180), nguyen);
        }
        assert_eq!(van_of_year(2044), NhatBach);
        assert_eq!(nguyen_of_year(2044), ThuongNguyen);
        assert_eq!(van_of_year(1863), CuuTu);
        assert_eq!(nguyen_of_year(1863), HaNguyen);
        assert_eq!(HaNguyen.name_vi(), "Hạ Nguyên");
    }

    #[test]
    fn year_star_of_the_landmark_years() {
        assert_eq!(star_of_year(1984), ThatXich);
        assert_eq!(star_of_year(2017), NhatBach);
        assert_eq!(star_of_year(2024), TamBich);
    }

    #[test]
    fn year_star_matches_the_tests_of_the_source() {
        let at = |year, month, day| date_time(year, month, day, 12, 0);
        // NineStarTest.ts: Lunar 2022-1-1 is 2022-02-01, Lunar 2033-1-1 is 2033-01-31.
        let cases = [
            (at(1985, 2, 19), LucBach),
            (at(2022, 2, 1), LucBach),
            (at(2033, 1, 31), TuLuc),
        ];
        for (time, star) in cases {
            assert_eq!(year_nine_star(time, CN, LapXuanDay), Ok(star));
        }
    }

    #[test]
    fn boundaries_match_each_sect_of_the_source() {
        // (time, year stars of sect 1, 2, 3, month stars of sect 1, 2, 3) from lunar-typescript.
        let table = [
            (date_time(2024, 2, 3, 12, 0), [4, 4, 4], [6, 6, 6]),
            (date_time(2024, 2, 4, 8, 0), [4, 3, 4], [8, 5, 6]),
            (date_time(2024, 2, 4, 18, 0), [4, 3, 3], [8, 5, 5]),
            (date_time(2024, 2, 9, 12, 0), [4, 3, 3], [8, 5, 5]),
            (date_time(2024, 2, 10, 12, 0), [3, 3, 3], [5, 5, 5]),
            (date_time(2022, 2, 2, 8, 0), [5, 6, 6], [9, 3, 3]),
            (date_time(2022, 2, 4, 8, 0), [5, 5, 5], [2, 2, 2]),
        ];
        for (time, years, months) in table {
            let boundaries = [LunarNewYear, LapXuanDay, LapXuanExact];
            let year_stars =
                boundaries.map(|boundary| year_nine_star(time, CN, boundary).unwrap().number());
            let month_stars =
                boundaries.map(|boundary| month_nine_star(time, CN, boundary).unwrap().number());
            assert_eq!(year_stars, years, "{time:?}");
            assert_eq!(month_stars, months, "{time:?}");
        }
    }

    #[test]
    fn the_period_changes_with_the_year_star() {
        // Between Lập Xuân and Tết of 2024: the lunar year is still 2023, the period 8.
        let time = date_time(2024, 2, 8, 8, 0);
        let by_lap_xuan = nine_stars(time, VN, LapXuanDay).unwrap();
        assert_eq!((by_lap_xuan.van, by_lap_xuan.year), (CuuTu, TamBich));
        let by_tet = nine_stars(time, VN, LunarNewYear).unwrap();
        assert_eq!((by_tet.van, by_tet.year), (BatBach, TuLuc));
    }

    #[test]
    fn lap_xuan_day_is_reckoned_locally() {
        // Lập Xuân 2009 fell at 00:49 on 4 February in China, 23:49 on 3 February in Vietnam.
        let time = date_time(2009, 2, 3, 12, 0);
        assert_eq!(year_nine_star(time, CN, LapXuanDay), Ok(star_of_year(2008)));
        assert_eq!(year_nine_star(time, VN, LapXuanDay), Ok(star_of_year(2009)));
    }

    #[test]
    fn month_star_table() {
        // Rows: year branches; columns: month branches from Dần to Sửu (section 4.4).
        let table = [
            (
                [Branch::Ty, Branch::Ngo, Branch::Mao, Branch::Dau],
                [8, 7, 6, 5, 4, 3, 2, 1, 9, 8, 7, 6],
            ),
            (
                [Branch::Thin, Branch::Tuat, Branch::Suu, Branch::Mui],
                [5, 4, 3, 2, 1, 9, 8, 7, 6, 5, 4, 3],
            ),
            (
                [Branch::Dan, Branch::Than, Branch::Ti, Branch::Hoi],
                [2, 1, 9, 8, 7, 6, 5, 4, 3, 2, 1, 9],
            ),
        ];
        for (year_branches, stars) in table {
            for year_branch in year_branches {
                let year = (2020..2032)
                    .find(|year| year_can_chi(*year).branch == year_branch)
                    .expect("twelve years hold every branch");
                for (months_since_dan, star) in stars.into_iter().enumerate() {
                    let month_branch = Branch::from_index(months_since_dan as i64 + 2);
                    assert_eq!(
                        month_star(year, month_branch).number(),
                        star,
                        "{year_branch:?} {month_branch:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn day_star_on_each_branch_of_the_source() {
        let table = [
            // Rising from the turning point near Đông Chí 2023.
            (date(2024, 1, 1), rising(NhatBach)),
            (date(2024, 3, 1), rising(ThatXich)),
            (date(2024, 6, 28), rising(CuuTu)),
            // Falling from the turning point near Hạ Chí 2024.
            (date(2024, 6, 29), falling(CuuTu)),
            (date(2024, 8, 1), falling(TamBich)),
            (date(2024, 12, 25), falling(NhatBach)),
            // Rising again from the turning point near Đông Chí 2024.
            (date(2024, 12, 26), rising(NhatBach)),
            (date(2024, 12, 30), rising(NguHoang)),
            // Before the turning point near Đông Chí 2008.
            (date(2009, 1, 10), falling(CuuTu)),
            (date(2009, 1, 18), falling(NhatBach)),
            (date(2009, 1, 19), rising(NhatBach)),
        ];
        for (day, star) in table {
            assert_eq!(day_nine_star(day, CN), Ok(star), "{day:?}");
        }
    }

    #[test]
    fn turning_points_take_the_nearest_giap_ty() {
        // Đông Chí 1939 is 30 days from either Giáp Tý: the later one wins.
        assert_eq!(
            solstice_anchor(SolarTerm::DongChi, 1939, CN),
            date(1940, 1, 22).to_jdn()
        );
        // Hạ Chí 1928 is 29 days after Giáp Tý.
        assert_eq!(
            solstice_anchor(SolarTerm::HaChi, 1928, CN),
            date(1928, 5, 24).to_jdn()
        );
        for (term, year) in [(SolarTerm::DongChi, 2024), (SolarTerm::HaChi, 2024)] {
            let anchor = solstice_anchor(term, year, VN);
            assert_eq!(day_can_chi(anchor).cycle_index(), 0);
        }
    }

    #[test]
    fn ha_chi_2077_moves_its_turning_point_in_vietnam() {
        // It falls at 00:23 on 21 June in China, 23:23 on 20 June in Vietnam, across the tie.
        assert_eq!(
            solstice_anchor(SolarTerm::HaChi, 2077, CN),
            date(2077, 7, 21).to_jdn()
        );
        assert_eq!(
            solstice_anchor(SolarTerm::HaChi, 2077, VN),
            date(2077, 5, 22).to_jdn()
        );
    }

    #[test]
    fn day_star_jumps_on_1_january_after_a_240_day_gap() {
        // Section 4.5: (year, turning point, star of 31 December, star of 1 January).
        let table = [
            (1906, date(1906, 1, 20), 5, 1),
            (1929, date(1929, 1, 19), 4, 9),
            (1940, date(1940, 1, 22), 7, 3),
            (1952, date(1952, 1, 19), 4, 9),
            (1963, date(1963, 1, 21), 6, 2),
            (1986, date(1986, 1, 20), 5, 1),
            (2009, date(2009, 1, 19), 4, 9),
            (2032, date(2032, 1, 19), 4, 9),
            (2043, date(2043, 1, 21), 6, 2),
            (2055, date(2055, 1, 18), 3, 8),
            (2066, date(2066, 1, 20), 5, 1),
            (2089, date(2089, 1, 19), 4, 9),
        ];
        for (year, turning_point, last_star, first_star) in table {
            let star = |day: SolarDate| day_nine_star(day, CN).unwrap();
            assert_eq!(star(date(year - 1, 12, 31)).star.number(), last_star);
            assert_eq!(
                star(date(year, 1, 1)),
                falling(NineStar::from_number(first_star))
            );
            assert_eq!(star(turning_point), rising(NhatBach));
            let before = SolarDate::from_jdn(turning_point.to_jdn() - 1);
            assert_eq!(star(before), falling(NhatBach));
        }
    }

    #[test]
    fn day_star_jumps_on_1_january_2078_only_in_vietnam() {
        // The turning point of Hạ Chí 2077 moves to 22 May in Vietnam, which leaves 240 days up
        // to the next one; in China the gap of 240 days ends at that turning point instead.
        assert_eq!(day_nine_star(date(2077, 12, 31), VN), Ok(falling(NhiHac)));
        assert_eq!(day_nine_star(date(2078, 1, 1), VN), Ok(falling(ThatXich)));
        assert_eq!(day_nine_star(date(2078, 1, 16), VN), Ok(falling(NhatBach)));
        assert_eq!(day_nine_star(date(2078, 1, 17), VN), Ok(rising(NhatBach)));
        assert_eq!(day_nine_star(date(2077, 12, 31), CN), Ok(falling(BatBach)));
        assert_eq!(day_nine_star(date(2078, 1, 1), CN), Ok(falling(ThatXich)));
    }

    #[test]
    fn day_star_matches_the_source_on_every_day() {
        const SOURCE_DIGEST: u64 = 0x31a5_8f3e_adc6_b0f6;
        let (first, last) = SOURCE_DAYS;
        let mut digest = FNV_OFFSET;
        let mut year = 0;
        let mut points = [0; 3];
        for day in first..=last {
            let solar_year = SolarDate::from_jdn(day).year;
            if solar_year != year {
                year = solar_year;
                points = turning_points(year, CN);
            }
            digest = fold(digest, day_star_between(day, points).star);
        }
        assert_eq!(digest, SOURCE_DIGEST);
    }

    #[test]
    fn hour_star_matches_the_source_on_every_day() {
        const SOURCE_DIGEST: u64 = 0x7d18_f00b_f1de_91c9;
        let (first, last) = SOURCE_DAYS;
        let mut digest = FNV_OFFSET;
        let mut year = 0;
        let mut solstices = (0, 0);
        for day in first..=last {
            let solar_year = SolarDate::from_jdn(day).year;
            if solar_year != year {
                year = solar_year;
                solstices = (
                    term_day(SolarTerm::HaChi, year, CN),
                    term_day(SolarTerm::DongChi, year, CN),
                );
            }
            let forward = hour_stars_rise(day, solstices.0, solstices.1);
            for hour_branch in Branch::ALL {
                let branch = day_can_chi(day).branch;
                digest = fold(digest, hour_star_of_branches(branch, hour_branch, forward));
            }
        }
        assert_eq!(digest, SOURCE_DIGEST);
    }

    #[test]
    fn hour_star_matches_the_source_around_every_solstice() {
        // Through `hour_nine_star`, so this also covers the direction taken by `hour_star`.
        const SOURCE_DIGEST: u64 = 0x8258_aec4_ee04_030b;
        let mut digest = FNV_OFFSET;
        for year in 1901..=2099 {
            for term in [SolarTerm::HaChi, SolarTerm::DongChi] {
                let solstice = term_day(term, year, CN);
                for day in [solstice - 1, solstice] {
                    digest = fold(digest, hour_nine_star(noon(day), CN).unwrap().star);
                }
            }
        }
        assert_eq!(digest, SOURCE_DIGEST);
    }

    #[test]
    fn year_and_month_stars_match_the_source_around_every_tiet() {
        // These are the only days on which the two stars can change with the default boundary.
        const SOURCE_DIGEST: u64 = 0x5750_b5fc_d7ff_0650;
        let mut digest = FNV_OFFSET;
        for year in 1901..=2099 {
            for term in SolarTerm::ALL
                .into_iter()
                .filter(|term| !term.is_principal())
            {
                let tiet = term_day(term, year, CN);
                for day in [tiet - 1, tiet] {
                    let time = noon(day);
                    digest = fold(digest, year_nine_star(time, CN, LapXuanDay).unwrap());
                    digest = fold(digest, month_nine_star(time, CN, LapXuanDay).unwrap());
                }
            }
        }
        assert_eq!(digest, SOURCE_DIGEST);
    }

    #[test]
    fn hour_star_table() {
        // Section 4.6: columns are the double-hours from Tý to Hợi.
        let table = [
            (
                true,
                [Branch::Ty, Branch::Ngo, Branch::Mao, Branch::Dau],
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3],
            ),
            (
                true,
                [Branch::Thin, Branch::Tuat, Branch::Suu, Branch::Mui],
                [4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6],
            ),
            (
                true,
                [Branch::Dan, Branch::Than, Branch::Ti, Branch::Hoi],
                [7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            ),
            (
                false,
                [Branch::Ty, Branch::Ngo, Branch::Mao, Branch::Dau],
                [9, 8, 7, 6, 5, 4, 3, 2, 1, 9, 8, 7],
            ),
            (
                false,
                [Branch::Thin, Branch::Tuat, Branch::Suu, Branch::Mui],
                [6, 5, 4, 3, 2, 1, 9, 8, 7, 6, 5, 4],
            ),
            (
                false,
                [Branch::Dan, Branch::Than, Branch::Ti, Branch::Hoi],
                [3, 2, 1, 9, 8, 7, 6, 5, 4, 3, 2, 1],
            ),
        ];
        for (forward, day_branches, stars) in table {
            for day_branch in day_branches {
                let row = Branch::ALL
                    .map(|hour| hour_star_of_branches(day_branch, hour, forward).number());
                assert_eq!(row, stars, "{day_branch:?}");
            }
        }
    }

    #[test]
    fn hour_star_turns_on_the_days_of_the_solstices() {
        let table = [
            (date_time(2024, 6, 20, 12, 0), rising(ThatXich)),
            (date_time(2024, 6, 21, 12, 0), falling(CuuTu)),
            (date_time(2024, 12, 20, 12, 0), falling(TamBich)),
            (date_time(2024, 12, 21, 12, 0), rising(NhatBach)),
            // `LunarTime.getNineStar` wrongly falls here: it would give Lục Bạch.
            (date_time(2024, 12, 25, 12, 0), rising(TuLuc)),
        ];
        for (time, star) in table {
            assert_eq!(hour_nine_star(time, CN), Ok(star), "{time:?}");
        }
    }

    #[test]
    fn ty_som_keeps_the_branch_of_the_civil_day() {
        // 23:30 on a Dậu day, then 00:30 on the following Tuất day.
        assert_eq!(
            hour_nine_star(date_time(1990, 5, 20, 23, 30), CN),
            Ok(rising(NhatBach))
        );
        assert_eq!(
            hour_nine_star(date_time(1990, 5, 21, 0, 30), CN),
            Ok(rising(TuLuc))
        );
        // 00:30 and 23:30 of one civil day share the star.
        assert_eq!(
            hour_nine_star(date_time(1990, 5, 20, 0, 30), CN),
            hour_nine_star(date_time(1990, 5, 20, 23, 30), CN)
        );
    }

    #[test]
    fn grids_of_the_lo_shu_square() {
        assert_eq!(
            numbers(generate_loshu_grid(NguHoang, true)),
            [[4, 9, 2], [3, 5, 7], [8, 1, 6]]
        );
        assert_eq!(
            numbers(generate_loshu_grid(NguHoang, false)),
            [[6, 1, 8], [7, 5, 3], [2, 9, 4]]
        );
        assert_eq!(
            numbers(generate_loshu_grid(TamBich, true)),
            [[2, 7, 9], [1, 3, 5], [6, 8, 4]]
        );
        assert_eq!(
            numbers(generate_loshu_grid(TamBich, false)),
            [[4, 8, 6], [5, 3, 1], [9, 7, 2]]
        );
    }

    #[test]
    fn stars_step_along_the_flight_path() {
        let path = [
            Palace::TrungCung,
            Palace::CanKim,
            Palace::Doai,
            Palace::CanTho,
            Palace::Ly,
            Palace::Kham,
            Palace::Khon,
            Palace::Chan,
            Palace::Ton,
        ];
        for center in NineStar::ALL {
            let center_number = i64::from(center.number());
            for (forward, step) in [(true, 1), (false, -1)] {
                let grid = generate_loshu_grid(center, forward);
                assert_eq!(grid.forward, forward);
                assert_eq!(grid.trung_cung, center);
                for (steps, palace) in path.into_iter().enumerate() {
                    assert_eq!(
                        grid.star_at(palace),
                        NineStar::from_number(center_number + step * steps as i64),
                        "{center:?} {palace:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_grid_holds_each_star_once() {
        for center in NineStar::ALL {
            for forward in [true, false] {
                let grid = generate_loshu_grid(center, forward);
                let mut seen = Palace::ALL.map(|palace| grid.star_at(palace).number());
                seen.sort_unstable();
                assert_eq!(seen, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
            }
        }
    }

    #[test]
    fn five_yellow_of_2024_sits_in_the_west() {
        let grid = generate_loshu_grid(star_of_year(2024), true);
        assert_eq!(grid.doai, NguHoang);
    }

    #[track_caller]
    fn assert_vector(
        time: SolarDateTime,
        era: (TamNguyen, NineStar),
        year_and_month: (NineStar, NineStar),
        day: FlyingStar,
        hour: FlyingStar,
    ) {
        let stars = nine_stars(time, VN, NineStarBoundary::default()).unwrap();
        assert_eq!((stars.nguyen, stars.van), era);
        assert_eq!((stars.year, stars.month), year_and_month);
        assert_eq!((stars.day, stars.hour), (day, hour));
        assert_eq!(year_nine_star(time, VN, LapXuanDay), Ok(stars.year));
        assert_eq!(month_nine_star(time, VN, LapXuanDay), Ok(stars.month));
        assert_eq!(day_nine_star(time.date(), VN), Ok(stars.day));
        assert_eq!(hour_nine_star(time, VN), Ok(stars.hour));
    }

    #[test]
    fn vector_1_ordinary_day() {
        assert_eq!(kua(1990, MALE), (1, Kham));
        assert_eq!(kua(1990, FEMALE), (5, CanTho));
        assert_vector(
            date_time(1990, 5, 20, 10, 30),
            (HaNguyen, ThatXich),
            (NhatBach, NguHoang),
            rising(ThatXich),
            rising(LucBach),
        );
        assert_eq!(
            numbers(generate_loshu_grid(NhatBach, true)),
            [[9, 5, 7], [8, 1, 3], [4, 6, 2]]
        );
    }

    #[test]
    fn vector_2_ty_som_between_the_solstice_and_its_turning_point() {
        assert_eq!(kua(2000, MALE), (9, Ly));
        assert_eq!(kua(2000, FEMALE), (6, CanKim));
        assert_vector(
            date_time(2000, 6, 25, 23, 30),
            (HaNguyen, ThatXich),
            (CuuTu, NhatBach),
            rising(CuuTu),
            falling(TamBich),
        );
        assert_eq!(
            numbers(generate_loshu_grid(CuuTu, true)),
            [[8, 4, 6], [7, 9, 2], [3, 5, 1]]
        );
    }

    #[test]
    fn vector_3_between_tet_and_lap_xuan() {
        assert_eq!(kua(2022, MALE), (5, Khon));
        assert_eq!(kua(2022, FEMALE), (1, Kham));
        assert_vector(
            date_time(2022, 2, 2, 8, 0),
            (HaNguyen, BatBach),
            (LucBach, TamBich),
            rising(NguHoang),
            rising(BatBach),
        );
        assert_eq!(
            numbers(generate_loshu_grid(LucBach, true)),
            [[5, 1, 3], [4, 6, 8], [9, 2, 7]]
        );
    }

    #[test]
    fn the_whole_year_range_is_accepted() {
        for boundary in [LapXuanDay, LapXuanExact] {
            assert!(nine_stars(date_time(MIN_YEAR, 1, 1, 0, 0), VN, boundary).is_ok());
            assert!(nine_stars(date_time(MAX_YEAR, 12, 31, 23, 59), VN, boundary).is_ok());
        }
        assert!(day_nine_star(date(MIN_YEAR, 1, 1), VN).is_ok());
        assert!(day_nine_star(date(MAX_YEAR, 12, 31), VN).is_ok());
    }

    #[test]
    fn the_lunar_boundary_starts_at_tet_1600() {
        // Those first days belong to lunar year 1599, which the lunar calendar does not cover.
        assert_eq!(
            nine_stars(date_time(MIN_YEAR, 1, 1, 0, 0), VN, LunarNewYear),
            Err(AstroError::YearOutOfRange { year: MIN_YEAR - 1 })
        );
        assert!(nine_stars(date_time(MIN_YEAR, 2, 20, 0, 0), VN, LunarNewYear).is_ok());
        assert!(nine_stars(date_time(MAX_YEAR, 12, 31, 23, 59), VN, LunarNewYear).is_ok());
    }

    #[test]
    fn rejects_invalid_input() {
        let time = date_time(2024, 2, 10, 8, 0);
        let bad_date = SolarDateTime { month: 13, ..time };
        let bad_time = SolarDateTime { hour: 24, ..time };
        let bad_year = SolarDateTime {
            year: MIN_YEAR - 1,
            ..time
        };
        for boundary in [LunarNewYear, LapXuanDay, LapXuanExact] {
            assert!(matches!(
                nine_stars(bad_date, VN, boundary),
                Err(AstroError::InvalidSolarDate { .. })
            ));
            assert!(matches!(
                year_nine_star(bad_time, VN, boundary),
                Err(AstroError::InvalidTime { .. })
            ));
            assert!(matches!(
                month_nine_star(bad_year, VN, boundary),
                Err(AstroError::YearOutOfRange { .. })
            ));
            assert!(matches!(
                nine_stars(time, 99.0, boundary),
                Err(AstroError::InvalidTimeZone { .. })
            ));
        }
        assert!(matches!(
            hour_nine_star(bad_time, VN),
            Err(AstroError::InvalidTime { .. })
        ));
        assert!(matches!(
            day_nine_star(bad_date.date(), VN),
            Err(AstroError::InvalidSolarDate { .. })
        ));
        assert!(matches!(
            day_nine_star(bad_year.date(), VN),
            Err(AstroError::YearOutOfRange { .. })
        ));
        assert!(matches!(
            day_nine_star(time.date(), f64::NAN),
            Err(AstroError::InvalidTimeZone { .. })
        ));
    }

    #[test]
    fn era_names_in_cycle_order() {
        assert_eq!(
            TamNguyen::ALL.map(TamNguyen::name_vi),
            ["Thượng Nguyên", "Trung Nguyên", "Hạ Nguyên"]
        );
    }

    #[test]
    fn direction_indices_run_from_zero_to_seven() {
        for (index, direction) in CompassDirection::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(direction.index()), index);
        }
    }

    #[test]
    fn kua_info_carries_every_field() {
        // Sections 2.2 and 2.3.
        let table = [
            (1990, MALE, 1, 1, Kham, TrigramGroup::DongTu),
            (1990, FEMALE, 5, 8, CanTho, TrigramGroup::TayTu),
            (2000, MALE, 9, 9, Ly, TrigramGroup::DongTu),
            (2000, FEMALE, 6, 6, CanKim, TrigramGroup::TayTu),
            (2022, MALE, 5, 2, Khon, TrigramGroup::TayTu),
        ];
        for (birth_lunar_year, gender, raw_number, number, trigram, group) in table {
            assert_eq!(
                kua_info(birth_lunar_year, gender),
                KuaInfo {
                    birth_lunar_year,
                    gender,
                    raw_number,
                    number,
                    trigram,
                    group
                }
            );
        }
    }

    #[test]
    fn default_boundary_is_the_day_of_lap_xuan() {
        assert_eq!(NineStarBoundary::default(), LapXuanDay);
    }

    #[test]
    fn nine_stars_applies_the_boundary_to_the_year_and_the_month() {
        // One row of `boundaries_match_each_sect_of_the_source`, read through `nine_stars`.
        let time = date_time(2024, 2, 4, 8, 0);
        let table = [
            (LunarNewYear, TuLuc, BatBach),
            (LapXuanDay, TamBich, NguHoang),
            (LapXuanExact, TuLuc, LucBach),
        ];
        for (boundary, year, month) in table {
            let stars = nine_stars(time, CN, boundary).unwrap();
            assert_eq!((stars.year, stars.month), (year, month), "{boundary:?}");
        }
    }

    #[test]
    fn the_era_changes_with_the_year_star() {
        // Lập Xuân 2044 falls on 4 February; `LunarYear.getYuan`, `getYun` of 2043 and 2044.
        let before = nine_stars(date_time(2044, 1, 15, 12, 0), VN, LapXuanDay).unwrap();
        assert_eq!(
            (before.nguyen, before.van, before.year),
            (HaNguyen, CuuTu, NhiHac)
        );
        let after = nine_stars(date_time(2044, 2, 10, 12, 0), VN, LapXuanDay).unwrap();
        assert_eq!(
            (after.nguyen, after.van, after.year),
            (ThuongNguyen, NhatBach, NhatBach)
        );
    }

    #[test]
    fn month_star_of_lap_xuan_day_is_reckoned_locally() {
        // The UTC+8 values are `getMonthNineStar` of 3 and 4 February 2009.
        let time = date_time(2009, 2, 3, 12, 0);
        assert_eq!(month_nine_star(time, CN, LapXuanDay), Ok(LucBach));
        assert_eq!(month_nine_star(time, VN, LapXuanDay), Ok(NguHoang));
    }

    #[test]
    fn exact_lap_xuan_is_reckoned_in_the_zone() {
        // Lập Xuân 2024 is 16:27 at UTC+8, 15:27 at UTC+7. 16:00 at UTC+7 is the instant of 17:00
        // at UTC+8, which sect 3 of the source reads as 3 and 5; 16:00 at UTC+8 reads as 4 and 6.
        let time = date_time(2024, 2, 4, 16, 0);
        assert_eq!(year_nine_star(time, CN, LapXuanExact), Ok(TuLuc));
        assert_eq!(month_nine_star(time, CN, LapXuanExact), Ok(LucBach));
        assert_eq!(year_nine_star(time, VN, LapXuanExact), Ok(TamBich));
        assert_eq!(month_nine_star(time, VN, LapXuanExact), Ok(NguHoang));
    }

    #[test]
    fn tet_is_reckoned_in_the_zone() {
        // Tết 2007 is 18 February in China, 17 February in Vietnam. Sect 1 of the source gives 3
        // and 5 on 17 February, 2 and 2 on 18 February.
        let time = date_time(2007, 2, 17, 12, 0);
        assert_eq!(year_nine_star(time, CN, LunarNewYear), Ok(TamBich));
        assert_eq!(month_nine_star(time, CN, LunarNewYear), Ok(NguHoang));
        assert_eq!(year_nine_star(time, VN, LunarNewYear), Ok(NhiHac));
        assert_eq!(month_nine_star(time, VN, LunarNewYear), Ok(NhiHac));
    }

    #[test]
    fn day_star_uses_the_turning_point_of_the_zone() {
        // Section 0.3: Hạ Chí 2077 moves its turning point from 21 July to 22 May in Vietnam.
        let day = date(2077, 6, 1);
        assert_eq!(day_nine_star(day, CN), Ok(rising(NhiHac)));
        assert_eq!(day_nine_star(day, VN), Ok(falling(BatBach)));
    }

    #[test]
    fn hour_star_turns_on_the_local_day_of_the_solstice() {
        // Section 0.3: Hạ Chí 2015 and Đông Chí 2017 fall at 23:xx in Vietnam, 00:xx in China.
        let summer = date_time(2015, 6, 21, 12, 0);
        assert_eq!(hour_nine_star(summer, CN), Ok(rising(NhatBach)));
        assert_eq!(hour_nine_star(summer, VN), Ok(falling(CuuTu)));
        let winter = date_time(2017, 12, 21, 12, 0);
        assert_eq!(hour_nine_star(winter, CN), Ok(falling(TamBich)));
        assert_eq!(hour_nine_star(winter, VN), Ok(rising(ThatXich)));
    }

    #[test]
    fn vectors_on_the_days_vietnam_differs_from_the_source() {
        // Section 0.3. Each row sits on a day where one lookup moves at UTC+7.
        assert_vector(
            date_time(2009, 2, 3, 12, 0),
            (HaNguyen, BatBach),
            (CuuTu, NguHoang),
            rising(ThatXich),
            rising(ThatXich),
        );
        assert_vector(
            date_time(2015, 6, 21, 12, 0),
            (HaNguyen, BatBach),
            (TamBich, NhatBach),
            falling(NguHoang),
            falling(CuuTu),
        );
        assert_vector(
            date_time(2017, 12, 21, 12, 0),
            (HaNguyen, BatBach),
            (NhatBach, ThatXich),
            rising(NhatBach),
            rising(ThatXich),
        );
        assert_vector(
            date_time(2077, 6, 1, 12, 0),
            (ThuongNguyen, NhiHac),
            (TuLuc, NguHoang),
            falling(BatBach),
            rising(NhatBach),
        );
    }

    #[test]
    fn month_star_of_january_keeps_the_year_of_the_dan_month() {
        // `getMonthNineStar` of 15 January 2044: the Sửu month of 2043, which opens from Nhị Hắc.
        let time = date_time(2044, 1, 15, 12, 0);
        assert_eq!(month_nine_star(time, CN, LapXuanDay), Ok(CuuTu));
        assert_eq!(nine_stars(time, CN, LapXuanDay).unwrap().month, CuuTu);
    }

    #[test]
    fn the_exact_instant_of_lap_xuan_belongs_to_the_new_year() {
        // `solarYmdHms < liChunYmdHms` in the source. No quarter-hour zone puts a whole second
        // exactly on an instant from 1600 to 2400, so the zone here is built for the tie.
        let instant = term_instant(SolarTerm::LapXuan, 2024);
        let time = date_time(2024, 2, 4, 0, 0);
        let tz_offset = (f64::from(time.date().to_jdn()) - 0.5 - instant) * 24.0;
        assert_eq!(time.to_julian_day(tz_offset), instant);
        assert_eq!(year_nine_star(time, tz_offset, LapXuanExact), Ok(TamBich));
    }

    #[test]
    fn day_star_uses_the_winter_turning_point_of_the_zone() {
        // Đông Chí 2042 is 18:04 on 21 December at UTC, 02:04 on 22 December at UTC+8, across the
        // tie of section 4.5: its turning point moves from 21 January 2043 to 22 November 2042.
        // No Đông Chí crosses the tie between UTC+7 and UTC+8 from 1600 to 2400.
        const UTC: f64 = 0.0;
        assert_eq!(day_nine_star(date(2042, 12, 1), CN), Ok(falling(CuuTu)));
        assert_eq!(day_nine_star(date(2042, 12, 1), UTC), Ok(rising(NhatBach)));
        assert_eq!(day_nine_star(date(2043, 1, 1), CN), Ok(falling(NhiHac)));
        assert_eq!(day_nine_star(date(2043, 1, 1), UTC), Ok(rising(NguHoang)));
    }

    #[test]
    fn nine_stars_passes_the_zone_to_every_star() {
        // The rows of `vectors_on_the_days_vietnam_differs_from_the_source`, read at UTC+8.
        let stars = nine_stars(date_time(2009, 2, 3, 12, 0), CN, LapXuanDay).unwrap();
        assert_eq!((stars.year, stars.month), (NhatBach, LucBach));
        let stars = nine_stars(date_time(2077, 6, 1, 12, 0), CN, LapXuanDay).unwrap();
        assert_eq!(stars.day, rising(NhiHac));
        let stars = nine_stars(date_time(2015, 6, 21, 12, 0), CN, LapXuanDay).unwrap();
        assert_eq!(stars.hour, rising(NhatBach));
    }

    #[test]
    fn term_days_change_at_midnight_not_at_the_ty_hour() {
        // lunar-typescript: 23:30 stays on the eve of a term, 00:30 already counts as its day.
        let table = [
            // Lập Xuân, 4 February 2024.
            (date_time(2024, 2, 3, 23, 30), TuLuc, LucBach),
            (date_time(2024, 2, 4, 0, 30), TamBich, NguHoang),
            // Kinh Trập, 5 March 2024.
            (date_time(2024, 3, 4, 23, 30), TamBich, NguHoang),
            (date_time(2024, 3, 5, 0, 30), TamBich, TuLuc),
        ];
        for (time, year, month) in table {
            assert_eq!(year_nine_star(time, CN, LapXuanDay), Ok(year), "{time:?}");
            assert_eq!(month_nine_star(time, CN, LapXuanDay), Ok(month), "{time:?}");
        }
        let table = [
            // Hạ Chí, 21 June 2024.
            (date_time(2024, 6, 20, 23, 30), rising(NhatBach)),
            (date_time(2024, 6, 21, 0, 30), falling(LucBach)),
            // Đông Chí, 21 December 2024.
            (date_time(2024, 12, 20, 23, 30), falling(CuuTu)),
            (date_time(2024, 12, 21, 0, 30), rising(TuLuc)),
        ];
        for (time, star) in table {
            assert_eq!(hour_nine_star(time, CN), Ok(star), "{time:?}");
        }
    }

    #[test]
    fn lap_xuan_exact_compares_instants_without_rounding() {
        // A term shown rounded down to the second has not begun yet at that second.
        let shown =
            |year: i32| SolarDateTime::from_julian_day(term_instant(SolarTerm::LapXuan, year), CN);
        let year = (1900..=2100)
            .find(|year| shown(*year).to_julian_day(CN) < term_instant(SolarTerm::LapXuan, *year))
            .expect("about half of the instants round down");
        assert_eq!(
            year_nine_star(shown(year), CN, LapXuanExact),
            Ok(star_of_year(year - 1))
        );
    }
}
