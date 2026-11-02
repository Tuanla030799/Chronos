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

use core::fmt;

/// The ten Heavenly Stems (Thiên Can).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CelestialStem {
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

impl CelestialStem {
    /// All stems in cycle order, from Giáp.
    pub const ALL: [CelestialStem; 10] = [
        CelestialStem::Giap,
        CelestialStem::At,
        CelestialStem::Binh,
        CelestialStem::Dinh,
        CelestialStem::Mau,
        CelestialStem::Ky,
        CelestialStem::Canh,
        CelestialStem::Tan,
        CelestialStem::Nham,
        CelestialStem::Quy,
    ];

    /// Position in the cycle, from 0 (Giáp) to 9 (Quý).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The stem at cycle position `index`, wrapping modulo 10.
    pub const fn from_index(index: i64) -> Self {
        Self::ALL[index.rem_euclid(10) as usize]
    }

    /// Vietnamese name, e.g. "Giáp".
    pub const fn name_vi(self) -> &'static str {
        match self {
            CelestialStem::Giap => "Giáp",
            CelestialStem::At => "Ất",
            CelestialStem::Binh => "Bính",
            CelestialStem::Dinh => "Đinh",
            CelestialStem::Mau => "Mậu",
            CelestialStem::Ky => "Kỷ",
            CelestialStem::Canh => "Canh",
            CelestialStem::Tan => "Tân",
            CelestialStem::Nham => "Nhâm",
            CelestialStem::Quy => "Quý",
        }
    }

    /// Hán character, e.g. "甲".
    pub const fn name_han(self) -> &'static str {
        match self {
            CelestialStem::Giap => "甲",
            CelestialStem::At => "乙",
            CelestialStem::Binh => "丙",
            CelestialStem::Dinh => "丁",
            CelestialStem::Mau => "戊",
            CelestialStem::Ky => "己",
            CelestialStem::Canh => "庚",
            CelestialStem::Tan => "辛",
            CelestialStem::Nham => "壬",
            CelestialStem::Quy => "癸",
        }
    }
}

/// The twelve Earthly Branches (Địa Chi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EarthlyBranch {
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

impl EarthlyBranch {
    /// All branches in cycle order, from Tý.
    pub const ALL: [EarthlyBranch; 12] = [
        EarthlyBranch::Ty,
        EarthlyBranch::Suu,
        EarthlyBranch::Dan,
        EarthlyBranch::Mao,
        EarthlyBranch::Thin,
        EarthlyBranch::Ti,
        EarthlyBranch::Ngo,
        EarthlyBranch::Mui,
        EarthlyBranch::Than,
        EarthlyBranch::Dau,
        EarthlyBranch::Tuat,
        EarthlyBranch::Hoi,
    ];

    /// Position in the cycle, from 0 (Tý) to 11 (Hợi).
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The branch at cycle position `index`, wrapping modulo 12.
    pub const fn from_index(index: i64) -> Self {
        Self::ALL[index.rem_euclid(12) as usize]
    }

    /// Branch of the double-hour (canh giờ) containing `hour` (0–23): Tý is 23:00–00:59,
    /// Sửu 01:00–02:59, and so on to Hợi 21:00–22:59.
    pub const fn from_hour(hour: u8) -> Self {
        Self::from_index((hour as i64 + 1) / 2)
    }

    /// Vietnamese name, e.g. "Tý".
    pub const fn name_vi(self) -> &'static str {
        match self {
            EarthlyBranch::Ty => "Tý",
            EarthlyBranch::Suu => "Sửu",
            EarthlyBranch::Dan => "Dần",
            EarthlyBranch::Mao => "Mão",
            EarthlyBranch::Thin => "Thìn",
            EarthlyBranch::Ti => "Tỵ",
            EarthlyBranch::Ngo => "Ngọ",
            EarthlyBranch::Mui => "Mùi",
            EarthlyBranch::Than => "Thân",
            EarthlyBranch::Dau => "Dậu",
            EarthlyBranch::Tuat => "Tuất",
            EarthlyBranch::Hoi => "Hợi",
        }
    }

    /// Hán character, e.g. "子".
    pub const fn name_han(self) -> &'static str {
        match self {
            EarthlyBranch::Ty => "子",
            EarthlyBranch::Suu => "丑",
            EarthlyBranch::Dan => "寅",
            EarthlyBranch::Mao => "卯",
            EarthlyBranch::Thin => "辰",
            EarthlyBranch::Ti => "巳",
            EarthlyBranch::Ngo => "午",
            EarthlyBranch::Mui => "未",
            EarthlyBranch::Than => "申",
            EarthlyBranch::Dau => "酉",
            EarthlyBranch::Tuat => "戌",
            EarthlyBranch::Hoi => "亥",
        }
    }
}

/// A sexagenary pair. Only the 60 pairs whose stem and branch indices share parity occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CanChi {
    /// Heavenly Stem (Can).
    pub stem: CelestialStem,
    /// Earthly Branch (Chi).
    pub branch: EarthlyBranch,
}

impl CanChi {
    /// Pairs a stem with a branch, or returns `None` for a pair outside the cycle
    /// (e.g. Giáp Sửu).
    pub const fn new(stem: CelestialStem, branch: EarthlyBranch) -> Option<Self> {
        if stem.index() % 2 == branch.index() % 2 {
            Some(Self { stem, branch })
        } else {
            None
        }
    }

    /// The pair at cycle position `index` (0 = Giáp Tý, 59 = Quý Hợi), wrapping modulo 60.
    pub const fn from_cycle_index(index: i64) -> Self {
        Self {
            stem: CelestialStem::from_index(index),
            branch: EarthlyBranch::from_index(index),
        }
    }

    /// Position in the cycle, from 0 (Giáp Tý) to 59 (Quý Hợi).
    pub const fn cycle_index(self) -> u8 {
        // Chinese remainder theorem: i ≡ stem (mod 10) and i ≡ branch (mod 12) give
        // i ≡ 6·stem − 5·branch (mod 60).
        (6 * self.stem.index() as i32 - 5 * self.branch.index() as i32).rem_euclid(60) as u8
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
        stem: CelestialStem::from_index(12 * year as i64 + month as i64 + 3),
        branch: EarthlyBranch::from_index(month as i64 + 1),
    }
}

/// Can Chi of the day with Julian Day Number `jdn`.
pub const fn day_can_chi(jdn: i32) -> CanChi {
    CanChi::from_cycle_index(jdn as i64 + 49)
}

/// Can Chi of the double-hour `branch` on a day whose stem is `day_stem`.
pub const fn hour_can_chi(day_stem: CelestialStem, branch: EarthlyBranch) -> CanChi {
    CanChi {
        stem: CelestialStem::from_index(2 * day_stem.index() as i64 + branch.index() as i64),
        branch,
    }
}

#[cfg(test)]
mod tests {
    use super::CelestialStem::*;
    use super::EarthlyBranch::*;
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
        let pairs = CelestialStem::ALL
            .into_iter()
            .flat_map(|stem| {
                EarthlyBranch::ALL
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
        for (day_stem, stem) in CelestialStem::ALL.into_iter().zip(rat_hour_stems) {
            assert_eq!(
                hour_can_chi(day_stem, Ty),
                CanChi { stem, branch: Ty },
                "{day_stem:?}"
            );
        }
        assert_eq!(hour_can_chi(Giap, Thin).to_string(), "Mậu Thìn");
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
            assert_eq!(EarthlyBranch::from_hour(hour), branch, "{hour}h");
        }
    }
}
