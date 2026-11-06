//! Reading a Bát Tự chart: hidden stems (Tàng Can), Ten Gods (Thập Thần), the twelve life stages
//! (Vòng Trường Sinh), chart stars (Thần Sát Mệnh Lý) and the luck cycles (Đại Vận).
//!
//! All tables come from `docs/BAZI_ALMANAC_SPEC.md`, sections 2.1 and 2.3 to 2.6: lunar-typescript
//! for the first three and Đại Vận, the project owner's tables for the chart stars.

use crate::bazi_pillars::{FourPillars, FourPillarsOptions, four_pillars};
use crate::canchi::{Branch, CanChi, NaYin, Stem, YinYang};
use crate::error::AstroError;
use crate::solar_terms::surrounding_sectional_terms;
use crate::time::{SolarDate, SolarDateTime, days_in_month, is_leap_year};

/// The Ten Gods (Thập Thần): the relation of a stem to the Day Master.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TenGod {
    /// Tỷ Kiên 比肩
    TyKien,
    /// Kiếp Tài 劫财
    KiepTai,
    /// Thực Thần 食神
    ThucThan,
    /// Thương Quan 伤官
    ThuongQuan,
    /// Thiên Tài 偏财
    ThienTai,
    /// Chính Tài 正财
    ChinhTai,
    /// Thất Sát 七杀
    ThatSat,
    /// Chính Quan 正官
    ChinhQuan,
    /// Thiên Ấn 偏印
    ThienAn,
    /// Chính Ấn 正印
    ChinhAn,
}

impl TenGod {
    /// All gods, in the order of the enum.
    pub const ALL: [TenGod; 10] = [
        TenGod::TyKien,
        TenGod::KiepTai,
        TenGod::ThucThan,
        TenGod::ThuongQuan,
        TenGod::ThienTai,
        TenGod::ChinhTai,
        TenGod::ThatSat,
        TenGod::ChinhQuan,
        TenGod::ThienAn,
        TenGod::ChinhAn,
    ];

    /// Vietnamese name, e.g. "Tỷ Kiên".
    pub const fn name_vi(self) -> &'static str {
        match self {
            TenGod::TyKien => "Tỷ Kiên",
            TenGod::KiepTai => "Kiếp Tài",
            TenGod::ThucThan => "Thực Thần",
            TenGod::ThuongQuan => "Thương Quan",
            TenGod::ThienTai => "Thiên Tài",
            TenGod::ChinhTai => "Chính Tài",
            TenGod::ThatSat => "Thất Sát",
            TenGod::ChinhQuan => "Chính Quan",
            TenGod::ThienAn => "Thiên Ấn",
            TenGod::ChinhAn => "Chính Ấn",
        }
    }
}

/// Ten Gods by Day Master (row) and the other stem (column), in stem order.
const TEN_GODS: [[TenGod; 10]; 10] = {
    use TenGod::*;
    [
        [
            TyKien, KiepTai, ThucThan, ThuongQuan, ThienTai, ChinhTai, ThatSat, ChinhQuan, ThienAn,
            ChinhAn,
        ],
        [
            KiepTai, TyKien, ThuongQuan, ThucThan, ChinhTai, ThienTai, ChinhQuan, ThatSat, ChinhAn,
            ThienAn,
        ],
        [
            ThienAn, ChinhAn, TyKien, KiepTai, ThucThan, ThuongQuan, ThienTai, ChinhTai, ThatSat,
            ChinhQuan,
        ],
        [
            ChinhAn, ThienAn, KiepTai, TyKien, ThuongQuan, ThucThan, ChinhTai, ThienTai, ChinhQuan,
            ThatSat,
        ],
        [
            ThatSat, ChinhQuan, ThienAn, ChinhAn, TyKien, KiepTai, ThucThan, ThuongQuan, ThienTai,
            ChinhTai,
        ],
        [
            ChinhQuan, ThatSat, ChinhAn, ThienAn, KiepTai, TyKien, ThuongQuan, ThucThan, ChinhTai,
            ThienTai,
        ],
        [
            ThienTai, ChinhTai, ThatSat, ChinhQuan, ThienAn, ChinhAn, TyKien, KiepTai, ThucThan,
            ThuongQuan,
        ],
        [
            ChinhTai, ThienTai, ChinhQuan, ThatSat, ChinhAn, ThienAn, KiepTai, TyKien, ThuongQuan,
            ThucThan,
        ],
        [
            ThucThan, ThuongQuan, ThienTai, ChinhTai, ThatSat, ChinhQuan, ThienAn, ChinhAn, TyKien,
            KiepTai,
        ],
        [
            ThuongQuan, ThucThan, ChinhTai, ThienTai, ChinhQuan, ThatSat, ChinhAn, ThienAn,
            KiepTai, TyKien,
        ],
    ]
};

/// The Ten God of stem `other` for the Day Master `day_stem`.
pub const fn ten_god(day_stem: Stem, other: Stem) -> TenGod {
    TEN_GODS[day_stem.index() as usize][other.index() as usize]
}

/// Hidden stems of each branch, in branch order: main, then middle, then residual qi.
const HIDDEN_STEMS: [&[Stem]; 12] = [
    &[Stem::Quy],
    &[Stem::Ky, Stem::Quy, Stem::Tan],
    &[Stem::Giap, Stem::Binh, Stem::Mau],
    &[Stem::At],
    &[Stem::Mau, Stem::At, Stem::Quy],
    &[Stem::Binh, Stem::Canh, Stem::Mau],
    &[Stem::Dinh, Stem::Ky],
    &[Stem::Ky, Stem::Dinh, Stem::At],
    &[Stem::Canh, Stem::Nham, Stem::Mau],
    &[Stem::Tan],
    &[Stem::Mau, Stem::Tan, Stem::Dinh],
    &[Stem::Nham, Stem::Giap],
];

/// Stems hidden in `branch`: Bản khí first, then Trung khí and Dư khí where they exist.
pub const fn hidden_stems(branch: Branch) -> &'static [Stem] {
    HIDDEN_STEMS[branch.index() as usize]
}

/// The main hidden stem (Bản khí) of `branch`.
pub const fn main_hidden_stem(branch: Branch) -> Stem {
    hidden_stems(branch)[0]
}

/// The twelve life stages (Vòng Trường Sinh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LifeStage {
    /// Trường Sinh 长生
    TruongSinh,
    /// Mộc Dục 沐浴
    MocDuc,
    /// Quan Đới 冠带
    QuanDoi,
    /// Lâm Quan 临官
    LamQuan,
    /// Đế Vượng 帝旺
    DeVuong,
    /// Suy 衰
    Suy,
    /// Bệnh 病
    Benh,
    /// Tử 死
    Tu,
    /// Mộ 墓
    Mo,
    /// Tuyệt 绝
    Tuyet,
    /// Thai 胎
    Thai,
    /// Dưỡng 养
    Duong,
}

impl LifeStage {
    /// All stages in the order of the cycle.
    pub const ALL: [LifeStage; 12] = [
        LifeStage::TruongSinh,
        LifeStage::MocDuc,
        LifeStage::QuanDoi,
        LifeStage::LamQuan,
        LifeStage::DeVuong,
        LifeStage::Suy,
        LifeStage::Benh,
        LifeStage::Tu,
        LifeStage::Mo,
        LifeStage::Tuyet,
        LifeStage::Thai,
        LifeStage::Duong,
    ];

    /// Vietnamese name, e.g. "Trường Sinh".
    pub const fn name_vi(self) -> &'static str {
        match self {
            LifeStage::TruongSinh => "Trường Sinh",
            LifeStage::MocDuc => "Mộc Dục",
            LifeStage::QuanDoi => "Quan Đới",
            LifeStage::LamQuan => "Lâm Quan",
            LifeStage::DeVuong => "Đế Vượng",
            LifeStage::Suy => "Suy",
            LifeStage::Benh => "Bệnh",
            LifeStage::Tu => "Tử",
            LifeStage::Mo => "Mộ",
            LifeStage::Tuyet => "Tuyệt",
            LifeStage::Thai => "Thai",
            LifeStage::Duong => "Dưỡng",
        }
    }
}

/// Position in the cycle of each stem at the branch Tý, in stem order (`CHANG_SHENG_OFFSET`).
const LIFE_STAGE_OFFSETS: [i64; 10] = [1, 6, 10, 9, 10, 9, 7, 0, 4, 3];

/// Life stage of `stem` at `branch`: yang stems run forward through the branches, yin stems
/// backward.
pub const fn life_stage(stem: Stem, branch: Branch) -> LifeStage {
    let offset = LIFE_STAGE_OFFSETS[stem.index() as usize];
    let step = branch.index() as i64;
    let index = match stem.yin_yang() {
        YinYang::Duong => offset + step,
        YinYang::Am => offset - step,
    };
    LifeStage::ALL[index.rem_euclid(12) as usize]
}

/// The chart stars of section 2.5 of the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BaziStar {
    /// Thiên Ất Quý Nhân
    ThienAtQuyNhan,
    /// Văn Xương
    VanXuong,
    /// Lộc Tồn
    LocTon,
    /// Kình Dương
    KinhDuong,
    /// Đà La
    DaLa,
    /// Dịch Mã
    DichMa,
    /// Đào Hoa (Hàm Trì)
    DaoHoa,
    /// Hoa Cái
    HoaCai,
    /// Tướng Tinh
    TuongTinh,
    /// Kiếp Sát
    KiepSat,
}

impl BaziStar {
    /// All stars, in the order of the enum.
    pub const ALL: [BaziStar; 10] = [
        BaziStar::ThienAtQuyNhan,
        BaziStar::VanXuong,
        BaziStar::LocTon,
        BaziStar::KinhDuong,
        BaziStar::DaLa,
        BaziStar::DichMa,
        BaziStar::DaoHoa,
        BaziStar::HoaCai,
        BaziStar::TuongTinh,
        BaziStar::KiepSat,
    ];

    /// Vietnamese name, e.g. "Thiên Ất Quý Nhân".
    pub const fn name_vi(self) -> &'static str {
        match self {
            BaziStar::ThienAtQuyNhan => "Thiên Ất Quý Nhân",
            BaziStar::VanXuong => "Văn Xương",
            BaziStar::LocTon => "Lộc Tồn",
            BaziStar::KinhDuong => "Kình Dương",
            BaziStar::DaLa => "Đà La",
            BaziStar::DichMa => "Dịch Mã",
            BaziStar::DaoHoa => "Đào Hoa",
            BaziStar::HoaCai => "Hoa Cái",
            BaziStar::TuongTinh => "Tướng Tinh",
            BaziStar::KiepSat => "Kiếp Sát",
        }
    }
}

/// Table A of section 2.5, in stem order: Thiên Ất Quý Nhân, Văn Xương, Lộc Tồn, Kình Dương, Đà La.
const STEM_STARS: [[&[Branch]; 5]; 10] = {
    use Branch::*;
    [
        [&[Suu, Mui], &[Ti], &[Dan], &[Mao], &[Suu]],
        [&[Ty, Than], &[Ngo], &[Mao], &[Thin], &[Dan]],
        [&[Hoi, Dau], &[Than], &[Ti], &[Ngo], &[Thin]],
        [&[Hoi, Dau], &[Dau], &[Ngo], &[Mui], &[Ti]],
        [&[Suu, Mui], &[Than], &[Ti], &[Ngo], &[Thin]],
        [&[Ty, Than], &[Dau], &[Ngo], &[Mui], &[Ti]],
        [&[Suu, Mui], &[Hoi], &[Than], &[Dau], &[Mui]],
        [&[Dan, Ngo], &[Ty], &[Dau], &[Tuat], &[Than]],
        [&[Mao, Ti], &[Dan], &[Hoi], &[Ty], &[Tuat]],
        [&[Mao, Ti], &[Mao], &[Ty], &[Suu], &[Hoi]],
    ]
};

/// Stars of table A, in the column order of [`STEM_STARS`].
const STEM_STAR_KINDS: [BaziStar; 5] = [
    BaziStar::ThienAtQuyNhan,
    BaziStar::VanXuong,
    BaziStar::LocTon,
    BaziStar::KinhDuong,
    BaziStar::DaLa,
];

/// Table B of section 2.5, by Tam Hợp group: Dịch Mã, Đào Hoa, Hoa Cái, Tướng Tinh, Kiếp Sát.
const BRANCH_STARS: [[Branch; 5]; 4] = {
    use Branch::*;
    [
        [Dan, Dau, Thin, Ty, Ti],
        [Than, Mao, Tuat, Ngo, Hoi],
        [Hoi, Ngo, Suu, Dau, Dan],
        [Ti, Ty, Mui, Mao, Than],
    ]
};

/// Stars of table B, in the column order of [`BRANCH_STARS`].
const BRANCH_STAR_KINDS: [BaziStar; 5] = [
    BaziStar::DichMa,
    BaziStar::DaoHoa,
    BaziStar::HoaCai,
    BaziStar::TuongTinh,
    BaziStar::KiepSat,
];

/// The five stars of table A with the branches where each falls, for `stem`.
pub fn stars_of_stem(stem: Stem) -> [(BaziStar, &'static [Branch]); 5] {
    let row = STEM_STARS[stem.index() as usize];
    core::array::from_fn(|i| (STEM_STAR_KINDS[i], row[i]))
}

/// The five stars of table B with the branch where each falls, for the Tam Hợp group of `branch`.
pub fn stars_of_branch(branch: Branch) -> [(BaziStar, Branch); 5] {
    let row = BRANCH_STARS[branch.trine_group()];
    core::array::from_fn(|i| (BRANCH_STAR_KINDS[i], row[i]))
}

/// The two empty branches (Không Vong) of a ten-day period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KhongVong {
    /// First empty branch.
    pub first: Branch,
    /// Second empty branch.
    pub second: Branch,
}

/// Không Vong of `pillar`: its period is `(branch − stem) mod 12`, which is always even.
pub const fn khong_vong(pillar: CanChi) -> KhongVong {
    let difference = pillar.branch.index() as i8 - pillar.stem.index() as i8;
    let (first, second) = match difference.rem_euclid(12) {
        0 => (Branch::Tuat, Branch::Hoi),
        10 => (Branch::Than, Branch::Dau),
        8 => (Branch::Ngo, Branch::Mui),
        6 => (Branch::Thin, Branch::Ti),
        4 => (Branch::Dan, Branch::Mao),
        _ => (Branch::Ty, Branch::Suu),
    };
    KhongVong { first, second }
}

/// Whether a day pillar is one of the four Khôi Canh: Canh Thìn, Canh Tuất, Nhâm Thìn, Mậu Tuất.
pub const fn is_khoi_canh(day: CanChi) -> bool {
    matches!(
        (day.stem, day.branch),
        (Stem::Canh, Branch::Thin | Branch::Tuat)
            | (Stem::Nham, Branch::Thin)
            | (Stem::Mau, Branch::Tuat)
    )
}

/// Sex of a person: together with the year stem it sets the direction of Đại Vận, and it selects
/// the formula of the Mệnh Quái.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Gender {
    /// Nam
    Male,
    /// Nữ
    Female,
}

/// One luck cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LuckCycle {
    /// Number of the cycle; 0 is the period before luck starts.
    pub index: u8,
    /// Can Chi of the cycle; `None` for cycle 0.
    pub can_chi: Option<CanChi>,
    /// First Gregorian year of the cycle.
    pub start_year: i32,
    /// Last Gregorian year of the cycle.
    pub end_year: i32,
    /// Age in the first year, counting the birth year as 1.
    pub start_age: i32,
    /// Age in the last year.
    pub end_age: i32,
}

/// The luck cycles (Đại Vận) of a chart.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DaiVan {
    /// Whether the cycles run forward through the sexagenary cycle.
    pub forward: bool,
    /// Whole years from birth to the start of luck.
    pub start_years: u32,
    /// Further months.
    pub start_months: u32,
    /// Further days.
    pub start_days: u32,
    /// Further hours.
    pub start_hours: u32,
    /// Date and time luck starts, on the birth clock.
    pub start_time: SolarDateTime,
    /// Cycle 0, then the nine cycles that follow it.
    pub cycles: Vec<LuckCycle>,
}

/// Number of cycles reported, including cycle 0 (the default of `Yun.getDaYun`).
const LUCK_CYCLE_COUNT: u8 = 10;

/// Luck cycles of a birth at wall-clock time `birth`, whose chart is `pillars`.
///
/// The cycles follow the year stem and month pillar of `pillars`. The distance is always measured
/// to the astronomical sectional terms, in whole minutes: 4320 minutes count as a year, 360 as a
/// month, 12 as a day and one as two hours (`Yun` with `sect = 2`).
pub fn dai_van(
    birth: SolarDateTime,
    tz_offset: f64,
    pillars: &FourPillars,
    gender: Gender,
) -> DaiVan {
    let yang_year = matches!(pillars.year.stem.yin_yang(), YinYang::Duong);
    let forward = yang_year == matches!(gender, Gender::Male);

    let (previous_term, next_term) = surrounding_sectional_terms(birth.to_julian_day(tz_offset));
    let minutes = if forward {
        whole_minutes(SolarDateTime::from_julian_day(next_term, tz_offset)) - whole_minutes(birth)
    } else {
        whole_minutes(birth)
            - whole_minutes(SolarDateTime::from_julian_day(previous_term, tz_offset))
    };
    let start_years = (minutes / 4320) as u32;
    let start_months = (minutes % 4320 / 360) as u32;
    let start_days = (minutes % 360 / 12) as u32;
    let start_hours = (minutes % 12 * 2) as u32;
    let start_time = add_time_span(birth, start_years, start_months, start_days, start_hours);

    let cycles = (0..LUCK_CYCLE_COUNT)
        .map(|index| luck_cycle(index, birth.year, start_time.year, pillars.month, forward))
        .collect();
    DaiVan {
        forward,
        start_years,
        start_months,
        start_days,
        start_hours,
        start_time,
        cycles,
    }
}

fn luck_cycle(
    index: u8,
    birth_year: i32,
    start_year: i32,
    month: CanChi,
    forward: bool,
) -> LuckCycle {
    if index == 0 {
        return LuckCycle {
            index,
            can_chi: None,
            start_year: birth_year,
            end_year: start_year - 1,
            start_age: 1,
            end_age: start_year - birth_year,
        };
    }
    let step = i64::from(index);
    let position = i64::from(month.cycle_index()) + if forward { step } else { -step };
    let first_year = start_year + (i32::from(index) - 1) * 10;
    let first_age = first_year - birth_year + 1;
    LuckCycle {
        index,
        can_chi: Some(CanChi::from_cycle_index(position)),
        start_year: first_year,
        end_year: first_year + 9,
        start_age: first_age,
        end_age: first_age + 9,
    }
}

/// Minutes since the Julian Day Number epoch, ignoring seconds as `Solar.subtractMinute` does.
fn whole_minutes(time: SolarDateTime) -> i64 {
    i64::from(time.date().to_jdn()) * 1440 + i64::from(time.hour) * 60 + i64::from(time.minute)
}

/// Adds years, months, days and hours in that order; a day past the end of the month becomes its
/// last day (`Yun.getStartSolar`).
fn add_time_span(
    time: SolarDateTime,
    years: u32,
    months: u32,
    days: u32,
    hours: u32,
) -> SolarDateTime {
    let year = time.year + years as i32;
    let mut day = time.day;
    if time.month == 2 && day > 28 && !is_leap_year(year) {
        day = 28;
    }
    let month_count = year * 12 + i32::from(time.month) - 1 + months as i32;
    let (year, month) = (
        month_count.div_euclid(12),
        month_count.rem_euclid(12) as u8 + 1,
    );
    if let Some(last_day) = days_in_month(year, month) {
        day = day.min(last_day);
    }
    let total_hours = u32::from(time.hour) + hours;
    let jdn = SolarDate { year, month, day }.to_jdn() + (days + total_hours / 24) as i32;
    let date = SolarDate::from_jdn(jdn);
    SolarDateTime {
        year: date.year,
        month: date.month,
        day: date.day,
        hour: (total_hours % 24) as u8,
        minute: time.minute,
        second: time.second,
    }
}

/// A hidden stem with its Ten God.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HiddenStem {
    /// The stem.
    pub stem: Stem,
    /// Its relation to the Day Master.
    pub ten_god: TenGod,
}

/// One pillar of a chart, read against the Day Master.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PillarReading {
    /// The pillar.
    pub can_chi: CanChi,
    /// Its Nạp Âm.
    pub na_yin: NaYin,
    /// Ten God of the pillar's stem; `None` for the day pillar, whose stem is the Day Master.
    pub ten_god: Option<TenGod>,
    /// Hidden stems of the pillar's branch: Bản khí, then Trung khí and Dư khí where they exist.
    pub hidden_stems: Vec<HiddenStem>,
    /// Life stage of the Day Master at the pillar's branch.
    pub life_stage: LifeStage,
}

/// A pillar of a chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PillarPosition {
    /// Trụ năm
    Year,
    /// Trụ tháng
    Month,
    /// Trụ ngày
    Day,
    /// Trụ giờ
    Hour,
}

/// What a chart star was looked up from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StarBasis {
    /// Can Ngày
    DayStem,
    /// Can Năm
    YearStem,
    /// Chi Ngày
    DayBranch,
    /// Chi Năm
    YearBranch,
}

/// A chart star falling on a pillar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StarPlacement {
    /// The star.
    pub star: BaziStar,
    /// What it was looked up from.
    pub basis: StarBasis,
    /// The pillar whose branch it falls on.
    pub pillar: PillarPosition,
}

/// A full reading of a Bát Tự chart.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BaziAnalysis {
    /// The Four Pillars.
    pub pillars: FourPillars,
    /// Reading of the year pillar.
    pub year: PillarReading,
    /// Reading of the month pillar.
    pub month: PillarReading,
    /// Reading of the day pillar.
    pub day: PillarReading,
    /// Reading of the hour pillar.
    pub hour: PillarReading,
    /// Every chart star found. The tables do not say whether the pillar a star was looked up
    /// from counts, so nothing is filtered out.
    pub stars: Vec<StarPlacement>,
    /// Không Vong from the day pillar.
    pub khong_vong_day: KhongVong,
    /// Không Vong from the year pillar.
    pub khong_vong_year: KhongVong,
    /// Whether the day pillar is Khôi Canh.
    pub is_khoi_canh: bool,
    /// The luck cycles.
    pub dai_van: DaiVan,
}

/// Reads the chart of a birth at wall-clock time `birth`.
pub fn analyze_bazi(
    birth: SolarDateTime,
    options: &FourPillarsOptions,
    gender: Gender,
) -> Result<BaziAnalysis, AstroError> {
    let pillars = four_pillars(birth, options)?;
    let day_stem = pillars.day.stem;
    Ok(BaziAnalysis {
        pillars,
        year: read_pillar(
            pillars.year,
            day_stem,
            Some(ten_god(day_stem, pillars.year.stem)),
        ),
        month: read_pillar(
            pillars.month,
            day_stem,
            Some(ten_god(day_stem, pillars.month.stem)),
        ),
        day: read_pillar(pillars.day, day_stem, None),
        hour: read_pillar(
            pillars.hour,
            day_stem,
            Some(ten_god(day_stem, pillars.hour.stem)),
        ),
        stars: place_stars(&pillars),
        khong_vong_day: khong_vong(pillars.day),
        khong_vong_year: khong_vong(pillars.year),
        is_khoi_canh: is_khoi_canh(pillars.day),
        dai_van: dai_van(birth, options.tz_offset, &pillars, gender),
    })
}

fn read_pillar(can_chi: CanChi, day_stem: Stem, ten_god_of_stem: Option<TenGod>) -> PillarReading {
    PillarReading {
        can_chi,
        na_yin: can_chi.na_yin(),
        ten_god: ten_god_of_stem,
        hidden_stems: hidden_stems(can_chi.branch)
            .iter()
            .map(|&stem| HiddenStem {
                stem,
                ten_god: ten_god(day_stem, stem),
            })
            .collect(),
        life_stage: life_stage(day_stem, can_chi.branch),
    }
}

fn place_stars(pillars: &FourPillars) -> Vec<StarPlacement> {
    let positions = [
        (PillarPosition::Year, pillars.year.branch),
        (PillarPosition::Month, pillars.month.branch),
        (PillarPosition::Day, pillars.day.branch),
        (PillarPosition::Hour, pillars.hour.branch),
    ];
    let mut placements = Vec::new();
    for (basis, stem) in [
        (StarBasis::DayStem, pillars.day.stem),
        (StarBasis::YearStem, pillars.year.stem),
    ] {
        for (star, branches) in stars_of_stem(stem) {
            for (pillar, branch) in positions {
                if branches.contains(&branch) {
                    placements.push(StarPlacement {
                        star,
                        basis,
                        pillar,
                    });
                }
            }
        }
    }
    for (basis, source) in [
        (StarBasis::DayBranch, pillars.day.branch),
        (StarBasis::YearBranch, pillars.year.branch),
    ] {
        for (star, target) in stars_of_branch(source) {
            for (pillar, branch) in positions {
                if branch == target {
                    placements.push(StarPlacement {
                        star,
                        basis,
                        pillar,
                    });
                }
            }
        }
    }
    placements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_gods_and_stars_are_complete_and_in_order() {
        for (index, god) in TenGod::ALL.into_iter().enumerate() {
            assert_eq!(god as usize, index);
        }
        for (index, star) in BaziStar::ALL.into_iter().enumerate() {
            assert_eq!(star as usize, index);
        }
        let mut names: Vec<&str> = TenGod::ALL.iter().map(|god| god.name_vi()).collect();
        names.extend(BaziStar::ALL.iter().map(|star| star.name_vi()));
        names.sort();
        names.dedup();
        assert_eq!(names.len(), TenGod::ALL.len() + BaziStar::ALL.len());
    }
    use crate::bazi_pillars::{FourPillarsOptions, YearMonthBoundary};
    use crate::canchi::Branch::{Dan, Dau, Hoi, Mao, Mui, Ngo, Suu, Than, Thin, Ti, Tuat, Ty};
    use crate::canchi::Stem::{At, Binh, Canh, Dinh, Giap, Ky, Mau, Nham, Quy, Tan};
    use crate::test_support::date_time;

    fn pair(stem: Stem, branch: Branch) -> CanChi {
        CanChi::new(stem, branch).expect("test pairs are valid")
    }

    #[test]
    fn ten_gods_by_day_master() {
        assert_eq!(ten_god(Giap, Giap), TenGod::TyKien);
        assert_eq!(ten_god(Giap, At), TenGod::KiepTai);
        assert_eq!(ten_god(Giap, Canh), TenGod::ThatSat);
        assert_eq!(ten_god(Giap, Tan), TenGod::ChinhQuan);
        assert_eq!(ten_god(At, Giap), TenGod::KiepTai);
        assert_eq!(ten_god(Binh, Quy), TenGod::ChinhQuan);
        assert_eq!(ten_god(Quy, Mau), TenGod::ChinhQuan);
        assert_eq!(ten_god(Mau, Binh), TenGod::ThienAn);
        assert_eq!(ten_god(Ky, Binh), TenGod::ChinhAn);
        for stem in Stem::ALL {
            assert_eq!(ten_god(stem, stem), TenGod::TyKien, "{stem:?}");
        }
    }

    #[test]
    fn main_hidden_stem_of_each_branch() {
        let expected = [Quy, Ky, Giap, At, Mau, Binh, Dinh, Ky, Canh, Tan, Mau, Nham];
        for (branch, stem) in Branch::ALL.into_iter().zip(expected) {
            assert_eq!(main_hidden_stem(branch), stem, "{branch:?}");
        }
    }

    #[test]
    fn hidden_stems_of_branches() {
        assert_eq!(hidden_stems(Ty), &[Quy]);
        assert_eq!(hidden_stems(Suu), &[Ky, Quy, Tan]);
        assert_eq!(hidden_stems(Dan), &[Giap, Binh, Mau]);
        assert_eq!(hidden_stems(Ngo), &[Dinh, Ky]);
        assert_eq!(hidden_stems(Ti), &[Binh, Canh, Mau]);
        assert_eq!(hidden_stems(Hoi), &[Nham, Giap]);
    }

    #[test]
    fn every_stem_is_born_at_its_branch() {
        for (stem, branch) in [
            (Giap, Hoi),
            (At, Ngo),
            (Binh, Dan),
            (Dinh, Dau),
            (Mau, Dan),
            (Ky, Dau),
            (Canh, Ti),
            (Tan, Ty),
            (Nham, Than),
            (Quy, Mao),
        ] {
            assert_eq!(life_stage(stem, branch), LifeStage::TruongSinh, "{stem:?}");
        }
        // Yang stems advance, yin stems retreat.
        assert_eq!(life_stage(Giap, Ty), LifeStage::MocDuc);
        assert_eq!(life_stage(At, Ti), LifeStage::MocDuc);
        assert_eq!(life_stage(Giap, Mao), LifeStage::DeVuong);
    }

    #[test]
    fn stars_of_stems_follow_table_a() {
        let of = |stem, star| {
            stars_of_stem(stem)
                .into_iter()
                .find(|&(kind, _)| kind == star)
                .map(|(_, branches)| branches)
                .unwrap()
        };
        assert_eq!(of(Giap, BaziStar::ThienAtQuyNhan), &[Suu, Mui]);
        assert_eq!(of(Tan, BaziStar::ThienAtQuyNhan), &[Dan, Ngo]);
        assert_eq!(of(Nham, BaziStar::ThienAtQuyNhan), &[Mao, Ti]);
        assert_eq!(of(Canh, BaziStar::VanXuong), &[Hoi]);
        assert_eq!(of(Dinh, BaziStar::LocTon), &[Ngo]);
        assert_eq!(of(Quy, BaziStar::KinhDuong), &[Suu]);
        assert_eq!(of(At, BaziStar::DaLa), &[Dan]);
    }

    #[test]
    fn stars_of_branches_follow_table_b() {
        assert_eq!(
            stars_of_branch(Ty),
            [
                (BaziStar::DichMa, Dan),
                (BaziStar::DaoHoa, Dau),
                (BaziStar::HoaCai, Thin),
                (BaziStar::TuongTinh, Ty),
                (BaziStar::KiepSat, Ti),
            ]
        );
        assert_eq!(stars_of_branch(Tuat)[0], (BaziStar::DichMa, Than));
        assert_eq!(stars_of_branch(Suu)[2], (BaziStar::HoaCai, Suu));
        assert_eq!(stars_of_branch(Mui)[4], (BaziStar::KiepSat, Than));
    }

    #[test]
    fn khong_vong_of_each_ten_day_period() {
        let empty = [
            (Tuat, Hoi),
            (Than, Dau),
            (Ngo, Mui),
            (Thin, Ti),
            (Dan, Mao),
            (Ty, Suu),
        ];
        for index in 0..60 {
            let result = khong_vong(CanChi::from_cycle_index(index));
            let (first, second) = empty[index as usize / 10];
            assert_eq!(result, KhongVong { first, second }, "{index}");
        }
    }

    #[test]
    fn khoi_canh_days() {
        for day in [
            pair(Canh, Thin),
            pair(Canh, Tuat),
            pair(Nham, Thin),
            pair(Mau, Tuat),
        ] {
            assert!(is_khoi_canh(day), "{day}");
        }
        assert!(!is_khoi_canh(pair(Nham, Tuat)));
        assert!(!is_khoi_canh(pair(Mau, Thin)));
    }

    fn cycles_for(gender: Gender) -> DaiVan {
        let birth = date_time(2024, 2, 10, 8, 0);
        let options = FourPillarsOptions::default();
        dai_van(birth, 7.0, &four_pillars(birth, &options).unwrap(), gender)
    }

    #[test]
    fn dai_van_runs_forward_for_a_man_born_in_a_yang_year() {
        // Giáp Thìn year; Kinh Trập fell about 24 days and 1 h 23 min after this birth.
        let dai_van = cycles_for(Gender::Male);
        assert!(dai_van.forward);
        assert_eq!((dai_van.start_years, dai_van.start_months), (8, 0));
        assert!(matches!(dai_van.start_days, 6 | 7), "{dai_van:?}");
        assert_eq!(dai_van.start_time.year, 2032);
        assert_eq!(dai_van.cycles.len(), 10);
        assert_eq!(
            dai_van.cycles[0],
            LuckCycle {
                index: 0,
                can_chi: None,
                start_year: 2024,
                end_year: 2031,
                start_age: 1,
                end_age: 8
            }
        );
        assert_eq!(
            dai_van.cycles[1],
            LuckCycle {
                index: 1,
                can_chi: Some(pair(Dinh, Mao)),
                start_year: 2032,
                end_year: 2041,
                start_age: 9,
                end_age: 18
            }
        );
        assert_eq!(dai_van.cycles[2].can_chi, Some(pair(Mau, Thin)));
    }

    #[test]
    fn dai_van_runs_backward_for_a_woman_born_in_a_yang_year() {
        // Lập Xuân fell about 5 days and 16 h 33 min before this birth.
        let dai_van = cycles_for(Gender::Female);
        assert!(!dai_van.forward);
        assert_eq!((dai_van.start_years, dai_van.start_months), (1, 10));
        // 1 year, 10 months and 22 days after 10 February 2024 is already January 2026.
        assert_eq!(dai_van.start_time.year, 2026);
        assert_eq!(dai_van.cycles[1].can_chi, Some(pair(At, Suu)));
        assert_eq!(dai_van.cycles[1].start_year, 2026);
        assert_eq!(dai_van.cycles[1].start_age, 3);
    }

    #[test]
    fn dai_van_direction_flips_in_a_yin_year() {
        let birth = date_time(2025, 3, 1, 8, 0);
        let pillars = four_pillars(birth, &FourPillarsOptions::default()).unwrap();
        assert_eq!(pillars.year, pair(At, Ti));
        assert!(!dai_van(birth, 7.0, &pillars, Gender::Male).forward);
        assert!(dai_van(birth, 7.0, &pillars, Gender::Female).forward);
    }

    #[test]
    fn dai_van_follows_the_chart_in_lunar_new_year_mode() {
        // After Lập Xuân, before Tết: the year stem is Quý (yin) by Tết, Giáp (yang) by Lập Xuân.
        let birth = date_time(2024, 2, 5, 10, 0);
        let options = FourPillarsOptions {
            boundary: YearMonthBoundary::LunarNewYear,
            ..FourPillarsOptions::default()
        };
        let pillars = four_pillars(birth, &options).unwrap();
        let dai_van = dai_van(birth, 7.0, &pillars, Gender::Male);
        assert!(!dai_van.forward);
        assert_eq!(dai_van.cycles[1].can_chi, Some(pair(Giap, Ty)));
    }

    #[test]
    fn time_span_clamps_to_the_end_of_the_month() {
        let leap_day = date_time(2024, 2, 29, 23, 30);
        let later = add_time_span(leap_day, 1, 0, 0, 0);
        assert_eq!((later.year, later.month, later.day), (2025, 2, 28));
        let end_of_january = date_time(2023, 1, 31, 10, 0);
        let later = add_time_span(end_of_january, 0, 1, 0, 0);
        assert_eq!((later.month, later.day), (2, 28));
        let later = add_time_span(date_time(2023, 12, 31, 22, 15), 0, 1, 2, 4);
        assert_eq!(
            (later.year, later.month, later.day, later.hour, later.minute),
            (2024, 2, 3, 2, 15)
        );
    }

    #[test]
    fn full_reading_of_a_chart() {
        // 08:00 on Tết Giáp Thìn: Giáp Thìn year, Bính Dần month, Giáp Thìn day, Mậu Thìn hour.
        let analysis = analyze_bazi(
            date_time(2024, 2, 10, 8, 0),
            &FourPillarsOptions::default(),
            Gender::Male,
        )
        .unwrap();
        assert_eq!(analysis.day.ten_god, None);
        assert_eq!(analysis.year.ten_god, Some(TenGod::TyKien));
        assert_eq!(analysis.month.ten_god, Some(TenGod::ThucThan));
        assert_eq!(analysis.hour.ten_god, Some(TenGod::ThienTai));
        assert_eq!(analysis.day.na_yin, NaYin::PhuDangHoa);
        assert_eq!(analysis.month.hidden_stems[0].stem, Giap);
        assert_eq!(analysis.month.hidden_stems[0].ten_god, TenGod::TyKien);
        assert_eq!(analysis.day.hidden_stems.len(), 3);
        assert_eq!(analysis.month.life_stage, LifeStage::LamQuan);
        assert_eq!(
            analysis.khong_vong_day,
            KhongVong {
                first: Dan,
                second: Mao
            }
        );
        assert!(!analysis.is_khoi_canh);
        // Lộc Tồn of Giáp falls on Dần, the month branch, from both the day and the year stem.
        for basis in [StarBasis::DayStem, StarBasis::YearStem] {
            assert!(analysis.stars.contains(&StarPlacement {
                star: BaziStar::LocTon,
                basis,
                pillar: PillarPosition::Month,
            }));
        }
        // Hoa Cái of the Thân–Tý–Thìn group falls on Thìn.
        assert!(analysis.stars.contains(&StarPlacement {
            star: BaziStar::HoaCai,
            basis: StarBasis::DayBranch,
            pillar: PillarPosition::Hour,
        }));
    }

    #[test]
    fn propagates_validation_errors() {
        let birth = SolarDateTime {
            hour: 25,
            ..date_time(2024, 2, 10, 8, 0)
        };
        assert!(analyze_bazi(birth, &FourPillarsOptions::default(), Gender::Male).is_err());
    }

    #[test]
    fn names() {
        assert_eq!(TenGod::ThienAn.name_vi(), "Thiên Ấn");
        assert_eq!(LifeStage::DeVuong.name_vi(), "Đế Vượng");
        assert_eq!(BaziStar::DaoHoa.name_vi(), "Đào Hoa");
    }
}
