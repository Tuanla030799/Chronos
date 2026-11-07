//! Which interpretation texts a result needs (section 9 of `docs/INTERPRETATION_DB_SPEC.md`).
//!
//! This module holds no text and does no I/O. It turns a day, a chart or a Tử Vi palace into the
//! keys of the interpretation database, so that Swift and TypeScript never build a key themselves
//! and never repeat the logic of a chart. Keys use the variant names of the enums of this crate.

use crate::almanac::{AgeRestrictions, AlmanacDay};
use crate::bazi_analysis::BaziAnalysis;
use crate::canchi::{Branch, CanChi};
use crate::fengshui::{EightMansionsStar, KuaCompatibility, KuaInfo, NineStar};
use crate::tuvi::{TuViChart, TuViPalace, TuViPalaceRole, TuViPlacedStar};
use std::collections::BTreeSet;
use std::fmt::Debug;

/// Where a star of the Eight Mansions is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EightUse {
    /// Hướng nhà
    HuongNha,
    /// Bếp
    Bep,
    /// Bàn làm việc
    BanLamViec,
    /// Kết hôn
    KetHon,
}

impl EightUse {
    /// All uses.
    pub const ALL: [EightUse; 4] = [
        EightUse::HuongNha,
        EightUse::Bep,
        EightUse::BanLamViec,
        EightUse::KetHon,
    ];

    /// Vietnamese name, e.g. "Bàn làm việc".
    pub const fn name_vi(self) -> &'static str {
        match self {
            EightUse::HuongNha => "Hướng nhà",
            EightUse::Bep => "Bếp",
            EightUse::BanLamViec => "Bàn làm việc",
            EightUse::KetHon => "Kết hôn",
        }
    }
}

/// One alternative of the condition of a passage: every fact of `all`, one fact of each group of
/// `any`, and no fact of `none`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gate {
    /// Facts that must hold.
    pub all: Vec<String>,
    /// Groups of which one fact must hold.
    pub any: Vec<Vec<String>>,
    /// Facts that must not hold.
    pub none: Vec<String>,
}

impl Gate {
    /// Whether `facts` pass this gate.
    pub fn opens(&self, facts: &[String]) -> bool {
        self.all.iter().all(|fact| facts.contains(fact))
            && self
                .any
                .iter()
                .all(|group| group.iter().any(|fact| facts.contains(fact)))
            && !self.none.iter().any(|fact| facts.contains(fact))
    }
}

/// Whether a passage with alternatives `gates` applies to `facts`: no gate means always.
pub fn gate_matches(gates: &[Gate], facts: &[String]) -> bool {
    gates.is_empty() || gates.iter().any(|gate| gate.opens(facts))
}

fn key(collection: &str, part: impl Debug) -> String {
    format!("{collection}/{part:?}")
}

fn unique(keys: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    keys.into_iter()
        .filter(|key| seen.insert(key.clone()))
        .collect()
}

/// Keys of the Trực, Mansion, path god, almanac gods and taboos of a day.
pub fn almanac_day_keys(day: &AlmanacDay) -> Vec<String> {
    let mut keys = vec![
        key("almanac.truc", day.truc),
        key("almanac.mansion", day.mansion),
        key("almanac.path_god", day.path_god),
    ];
    keys.extend(day.gods.iter().map(|&god| key("almanac.god", god)));
    keys.extend(day.taboos.iter().map(|&taboo| key("almanac.taboo", taboo)));
    keys
}

/// Keys of the age restrictions in force, each after the overview of its kind.
pub fn age_restriction_keys(restrictions: &AgeRestrictions) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(tam_tai) = restrictions.tam_tai {
        keys.push("almanac.restriction/TamTai".to_string());
        keys.push(key("almanac.tam_tai", tam_tai));
    }
    if let Some(kim_lau) = restrictions.kim_lau {
        keys.push("almanac.restriction/KimLau".to_string());
        keys.push(key("almanac.kim_lau", kim_lau));
    }
    if let Some(hoang_oc) = restrictions.hoang_oc {
        keys.push("almanac.restriction/HoangOc".to_string());
        keys.push(key("almanac.hoang_oc", hoang_oc));
    }
    keys
}

fn can_chi_key(can_chi: CanChi) -> String {
    format!("bazi.can_chi/{:?}/{:?}", can_chi.stem, can_chi.branch)
}

/// Keys of a Bát Tự reading: the Day Master, then each pillar, then the Thần Sát.
pub fn bazi_keys(analysis: &BaziAnalysis) -> Vec<String> {
    let mut keys = vec![key("bazi.day_master", analysis.day.can_chi.stem)];
    for pillar in [
        &analysis.year,
        &analysis.month,
        &analysis.day,
        &analysis.hour,
    ] {
        keys.push(key("bazi.nayin", pillar.na_yin));
        keys.push(can_chi_key(pillar.can_chi));
        keys.extend(pillar.ten_god.map(|god| key("bazi.ten_god", god)));
        keys.push(key("bazi.life_stage", pillar.life_stage));
    }
    keys.extend(
        analysis
            .stars
            .iter()
            .map(|placement| key("bazi.shen_sha", placement.star)),
    );
    keys.push("bazi.shen_sha/KhongVong".to_string());
    if analysis.is_khoi_canh {
        keys.push("bazi.shen_sha/KhoiCanh".to_string());
    }
    unique(keys)
}

/// Key of a person's trigram.
pub fn kua_key(kua: &KuaInfo) -> String {
    key("fengshui.kua", kua.trigram)
}

/// Key of a star of the Eight Mansions.
pub fn eight_star_key(star: EightMansionsStar) -> String {
    key("fengshui.eight_star", star)
}

/// Key of a star of the Eight Mansions in one use.
pub fn eight_star_use_key(star: EightMansionsStar, usage: EightUse) -> String {
    format!("fengshui.eight_star_use/{star:?}/{usage:?}")
}

/// Keys of the compatibility of two people: the star between them, then its text for marriage.
pub fn kua_compatibility_keys(compatibility: &KuaCompatibility) -> Vec<String> {
    vec![
        eight_star_key(compatibility.star),
        eight_star_use_key(compatibility.star, EightUse::KetHon),
    ]
}

/// Key of a Nine Star. [`NineStar::phase_in`] tells which of its two texts applies.
pub fn nine_star_key(star: NineStar) -> String {
    key("fengshui.nine_star", star)
}

/// A search for passages: the anchors to fetch, and the facts to test them against.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PassageLookup {
    /// Anchors, e.g. `QuanLoc/TuVi`.
    pub anchors: Vec<String>,
    /// Sorted facts of section 9 of the specification.
    pub facts: Vec<String>,
}

/// What the interpretation of one palace needs. A passage found by several anchors or lookups
/// counts once; each one is tested against the facts of the lookup that found it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PalaceQuery {
    /// Keys of the palace, its stars, their brightness and its transformations.
    pub keys: Vec<String>,
    /// The lookups: the palace itself, then the opposite Chính Tinh of a palace without any.
    pub lookups: Vec<PassageLookup>,
}

fn stars(palace: &TuViPalace) -> impl Iterator<Item = &TuViPlacedStar> {
    palace.main_stars.iter().chain(&palace.minor_stars)
}

/// Whether the book says to read the Chính Tinh of the opposite palace as sitting in a palace
/// without any: pp. 131, 142, 166, 171, 180, 197, 207 and 224. Mệnh and Điền Trạch have their own
/// text instead, Tật Ách and Thê Thiếp have nothing on this.
const fn borrows_opposite(role: TuViPalaceRole) -> bool {
    matches!(
        role,
        TuViPalaceRole::PhuMau
            | TuViPalaceRole::PhucDuc
            | TuViPalaceRole::QuanLoc
            | TuViPalaceRole::NoBoc
            | TuViPalaceRole::ThienDi
            | TuViPalaceRole::TaiBach
            | TuViPalaceRole::TuTuc
            | TuViPalaceRole::HuynhDe
    )
}

/// The palaces that meet `palace`: itself, its trine and its opposite (D7 of the specification).
fn meeting_branches(palace: &TuViPalace) -> [Branch; 4] {
    let [first, second, third] = palace.branch.trine();
    [first, second, third, palace.branch.clash()]
}

fn insert_placed(facts: &mut BTreeSet<String>, palace: &TuViPalace) {
    if palace.main_stars.len() == 1 {
        facts.insert("solo".to_string());
    }
    for placed in stars(palace) {
        let star = placed.star;
        facts.insert(format!("with:{star:?}"));
        if let Some(level) = placed.brightness {
            facts.insert(format!("bright:{star:?}:{level:?}"));
        }
    }
}

fn palace_facts(chart: &TuViChart, palace: &TuViPalace) -> BTreeSet<String> {
    let mut facts = BTreeSet::new();
    facts.insert(format!("branch:{:?}", palace.branch));
    insert_placed(&mut facts, palace);
    for (present, fact) in [
        (palace.role == TuViPalaceRole::Menh, "is_menh"),
        (palace.main_stars.is_empty(), "no_main"),
        (palace.is_than, "than"),
        (palace.has_tuan, "tuan"),
        (palace.has_triet, "triet"),
    ] {
        if present {
            facts.insert(fact.to_string());
        }
    }
    for hoa in stars(palace).filter_map(|placed| placed.transformation) {
        facts.insert(format!("hoa_with:{hoa:?}"));
        if let Some(level) = hoa.brightness_at(palace.branch) {
            facts.insert(format!("bright:{hoa:?}:{level:?}"));
        }
    }
    for branch in meeting_branches(palace) {
        for placed in stars(chart.palace(branch)) {
            facts.insert(format!("meet:{:?}", placed.star));
            if let Some(hoa) = placed.transformation {
                facts.insert(format!("hoa:{hoa:?}"));
            }
        }
    }
    for placed in stars(chart.palace(palace.branch.six_combination())) {
        facts.insert(format!("pair:{:?}", placed.star));
    }
    let index = palace.branch.index() as i64;
    for neighbour in [index - 1, index + 1].map(Branch::from_index) {
        for placed in stars(chart.palace(neighbour)) {
            facts.insert(format!("flank:{:?}", placed.star));
        }
    }
    facts.insert(format!("gender:{:?}", chart.gender));
    facts.insert(format!("polarity:{:?}", chart.polarity));
    facts.insert(format!("harmony:{:?}", chart.harmony));
    facts.insert(format!("ban_menh:{:?}", chart.ban_menh.element()));
    facts.insert(format!("year_stem:{:?}", chart.year_can_chi.stem));
    facts.insert(format!("year_branch:{:?}", chart.year_can_chi.branch));
    let menh = chart.palace_of_role(TuViPalaceRole::Menh);
    for placed in stars(menh) {
        facts.insert(format!("menh:{:?}", placed.star));
    }
    facts
}

/// The roles whose passages apply: the palace, and Mệnh on the palace of Thân (section 4.2 of the
/// book is about "cung Mệnh - Thân").
fn roles_of(palace: &TuViPalace) -> Vec<TuViPalaceRole> {
    let mut roles = vec![palace.role];
    if palace.is_than && palace.role != TuViPalaceRole::Menh {
        roles.push(TuViPalaceRole::Menh);
    }
    roles
}

fn own_anchors(palace: &TuViPalace) -> Vec<String> {
    let mut anchors = Vec::new();
    for role in roles_of(palace) {
        anchors.push(format!("{role:?}/~Any"));
        if palace.main_stars.is_empty() {
            anchors.push(format!("{role:?}/~NoMain"));
        }
        for placed in stars(palace) {
            anchors.push(format!("{role:?}/{:?}", placed.star));
            if let Some(hoa) = placed.transformation {
                anchors.push(format!("{role:?}/{hoa:?}"));
            }
        }
    }
    anchors
}

/// The lookup of the Chính Tinh of `opposite` read as sitting in `palace`: they keep their branch
/// and their brightness, the rest stays what `palace` sees. A palace without Chính Tinh always
/// faces two of them, so such a lookup is never "đơn thủ".
fn borrowed_lookup(
    palace: &TuViPalace,
    opposite: &TuViPalace,
    own: &BTreeSet<String>,
) -> PassageLookup {
    let mut facts = own.clone();
    facts.remove(&format!("branch:{:?}", palace.branch));
    facts.remove("no_main");
    facts.insert(format!("branch:{:?}", opposite.branch));
    for placed in &opposite.main_stars {
        facts.insert(format!("with:{:?}", placed.star));
        if let Some(level) = placed.brightness {
            facts.insert(format!("bright:{:?}:{level:?}", placed.star));
        }
    }
    PassageLookup {
        anchors: opposite
            .main_stars
            .iter()
            .map(|placed| format!("{:?}/{:?}", palace.role, placed.star))
            .collect(),
        facts: facts.into_iter().collect(),
    }
}

fn palace_keys(palace: &TuViPalace) -> Vec<String> {
    let mut keys = vec![key("tuvi.palace", palace.role)];
    for placed in stars(palace) {
        keys.push(key("tuvi.star", placed.star));
        if let Some(level) = placed.brightness {
            keys.push(format!("tuvi.star_brightness/{:?}/{level:?}", placed.star));
        }
        keys.extend(
            placed
                .transformation
                .map(|hoa| key("tuvi.transformation", hoa)),
        );
    }
    keys
}

/// What the interpretation of the palace at `branch` needs.
pub fn tuvi_palace_query(chart: &TuViChart, branch: Branch) -> PalaceQuery {
    let palace = chart.palace(branch);
    let facts = palace_facts(chart, palace);
    let mut lookups = Vec::new();
    if palace.main_stars.is_empty() && borrows_opposite(palace.role) {
        lookups.push(borrowed_lookup(
            palace,
            chart.palace(branch.clash()),
            &facts,
        ));
    }
    lookups.insert(
        0,
        PassageLookup {
            anchors: own_anchors(palace),
            facts: facts.into_iter().collect(),
        },
    );
    PalaceQuery {
        keys: palace_keys(palace),
        lookups,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::almanac::{age_restrictions, almanac_day};
    use crate::bazi_analysis::{Gender, analyze_bazi};
    use crate::bazi_pillars::FourPillarsOptions;
    use crate::fengshui::{kua_compatibility, kua_info};
    use crate::lunar_calendar::LunarDate;
    use crate::test_support::{VN, date};
    use crate::time::SolarDateTime;
    use crate::tuvi::{
        FourTransformations, StarBrightness, TuViConfig, TuViStar, chinh_tinh_branches,
        tuvi_chart_from_lunar,
    };

    fn facts(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    fn gate(all: &[&str], any: &[&[&str]], none: &[&str]) -> Gate {
        Gate {
            all: facts(all),
            any: any.iter().map(|group| facts(group)).collect(),
            none: facts(none),
        }
    }

    fn chart(year: i32, month: u8, day: u8, hour: Branch, gender: Gender) -> TuViChart {
        let lunar = LunarDate {
            year,
            month,
            day,
            is_leap_month: false,
        };
        tuvi_chart_from_lunar(lunar, hour, gender, &TuViConfig::default()).unwrap()
    }

    /// The first chart of the 2024 cycle whose palace of `role` passes `test`.
    fn find_palace(
        role: TuViPalaceRole,
        test: impl Fn(&TuViChart, &TuViPalace) -> bool,
    ) -> (TuViChart, Branch) {
        for day in 1..=29 {
            for month in 1..=12 {
                for hour in Branch::ALL {
                    let chart = chart(2024, month, day, hour, Gender::Male);
                    let branch = chart.palace_of_role(role).branch;
                    if test(&chart, chart.palace(branch)) {
                        return (chart, branch);
                    }
                }
            }
        }
        panic!("no such chart");
    }

    #[test]
    fn a_gate_needs_all_of_all_one_of_each_group_and_none_of_none() {
        let gate = gate(&["a"], &[&["b", "c"]], &["d"]);
        assert!(gate.opens(&facts(&["a", "c"])));
        assert!(!gate.opens(&facts(&["c"])));
        assert!(!gate.opens(&facts(&["a"])));
        assert!(!gate.opens(&facts(&["a", "b", "d"])));
    }

    #[test]
    fn no_gate_means_always_and_one_open_gate_is_enough() {
        assert!(gate_matches(&[], &[]));
        let gates = [gate(&["a"], &[], &[]), gate(&["b"], &[], &[])];
        assert!(gate_matches(&gates, &facts(&["b"])));
        assert!(!gate_matches(&gates, &facts(&["c"])));
    }

    #[test]
    fn a_day_lists_its_five_kinds_of_key() {
        let day = almanac_day(date(2024, 2, 10), VN).unwrap();
        let keys = almanac_day_keys(&day);
        assert_eq!(keys[0], format!("almanac.truc/{:?}", day.truc));
        assert_eq!(keys[1], format!("almanac.mansion/{:?}", day.mansion));
        assert_eq!(keys[2], format!("almanac.path_god/{:?}", day.path_god));
        assert_eq!(keys.len(), 3 + day.gods.len() + day.taboos.len());
    }

    #[test]
    fn age_restrictions_put_the_overview_before_the_case() {
        // Born 1990, in 2026: age 37, Tam Tai no, Kim Lâu no (37 % 9 = 1 is Thân), Tứ Tấn Tài.
        let keys = age_restriction_keys(&age_restrictions(1990, 2026));
        assert_eq!(
            keys,
            [
                "almanac.restriction/KimLau",
                "almanac.kim_lau/Than",
                "almanac.restriction/HoangOc",
                "almanac.hoang_oc/TuTanTai"
            ]
        );
    }

    #[test]
    fn a_young_person_out_of_every_restriction_needs_nothing() {
        // Age 2 is below Hoang Ốc and not Kim Lâu; a Tý year person has no Tam Tai in a Sửu year.
        let restrictions = age_restrictions(2020, 2021);
        assert_eq!(restrictions.hoang_oc, None);
        assert_eq!(restrictions.kim_lau, None);
        assert_eq!(age_restriction_keys(&restrictions).len(), 0);
    }

    #[test]
    fn bazi_keys_start_with_the_day_master_and_never_repeat() {
        let birth = SolarDateTime {
            year: 1990,
            month: 5,
            day: 15,
            hour: 10,
            minute: 30,
            second: 0,
        };
        let analysis = analyze_bazi(birth, &FourPillarsOptions::default(), Gender::Male).unwrap();
        let keys = bazi_keys(&analysis);
        assert_eq!(
            keys[0],
            format!("bazi.day_master/{:?}", analysis.day.can_chi.stem)
        );
        assert!(keys.contains(&"bazi.shen_sha/KhongVong".to_string()));
        assert_eq!(keys.len(), keys.iter().collect::<BTreeSet<_>>().len());
    }

    #[test]
    fn feng_shui_keys_name_their_collection() {
        use crate::fengshui::EightMansionsStar::SinhKhi;
        assert_eq!(eight_star_key(SinhKhi), "fengshui.eight_star/SinhKhi");
        assert_eq!(
            eight_star_use_key(SinhKhi, EightUse::Bep),
            "fengshui.eight_star_use/SinhKhi/Bep"
        );
        assert_eq!(nine_star_key(NineStar::CuuTu), "fengshui.nine_star/CuuTu");
        let kua = kua_info(1990, Gender::Male);
        assert_eq!(kua_key(&kua), format!("fengshui.kua/{:?}", kua.trigram));
        let keys = kua_compatibility_keys(&kua_compatibility(kua, kua));
        assert_eq!(
            keys,
            [
                "fengshui.eight_star/PhucVi",
                "fengshui.eight_star_use/PhucVi/KetHon"
            ]
        );
    }

    #[test]
    fn a_palace_query_lists_its_stars_and_brightness() {
        let (chart, branch) = find_palace(TuViPalaceRole::QuanLoc, |_, p| p.main_stars.len() == 1);
        let query = tuvi_palace_query(&chart, branch);
        let palace = chart.palace(branch);
        let main = palace.main_stars[0];
        assert_eq!(query.keys[0], "tuvi.palace/QuanLoc");
        assert!(query.keys.contains(&format!("tuvi.star/{:?}", main.star)));
        let lookup = &query.lookups[0];
        assert!(lookup.anchors.contains(&format!("QuanLoc/{:?}", main.star)));
        assert!(lookup.anchors.contains(&"QuanLoc/~Any".to_string()));
        assert!(lookup.facts.contains(&"solo".to_string()));
        assert!(lookup.facts.contains(&format!("branch:{branch:?}")));
        let level = main.brightness.unwrap();
        assert!(
            lookup
                .facts
                .contains(&format!("bright:{:?}:{level:?}", main.star))
        );
        assert!(lookup.facts.is_sorted());
    }

    #[test]
    fn meeting_covers_the_trine_and_the_opposite_but_not_the_nhi_hop() {
        let (chart, branch) = find_palace(TuViPalaceRole::QuanLoc, |_, p| !p.main_stars.is_empty());
        let facts = &tuvi_palace_query(&chart, branch).lookups[0].facts;
        for star in stars(chart.palace(branch.clash())) {
            assert!(facts.contains(&format!("meet:{:?}", star.star)));
        }
        for at in branch.trine() {
            for star in stars(chart.palace(at)) {
                assert!(facts.contains(&format!("meet:{:?}", star.star)));
            }
        }
        for star in stars(chart.palace(branch.six_combination())) {
            assert!(facts.contains(&format!("pair:{:?}", star.star)));
        }
        let met: BTreeSet<_> = meeting_branches(chart.palace(branch))
            .iter()
            .flat_map(|&b| stars(chart.palace(b)).map(|s| s.star))
            .collect();
        for star in stars(chart.palace(branch.six_combination())) {
            let fact = format!("meet:{:?}", star.star);
            assert_eq!(facts.contains(&fact), met.contains(&star.star));
        }
    }

    #[test]
    fn a_transformation_gives_an_anchor_and_a_fact() {
        let (chart, branch) = find_palace(TuViPalaceRole::Menh, |_, p| {
            stars(p).any(|s| s.transformation == Some(FourTransformations::HoaKy))
        });
        let query = tuvi_palace_query(&chart, branch);
        assert!(
            query
                .keys
                .contains(&"tuvi.transformation/HoaKy".to_string())
        );
        assert!(query.lookups[0].anchors.contains(&"Menh/HoaKy".to_string()));
        assert!(
            query.lookups[0]
                .facts
                .contains(&"hoa_with:HoaKy".to_string())
        );
        let level = FourTransformations::HoaKy.brightness_at(branch).unwrap();
        assert!(
            query.lookups[0]
                .facts
                .contains(&format!("bright:HoaKy:{level:?}"))
        );
    }

    #[test]
    fn minor_stars_bring_their_brightness() {
        let (chart, branch) = find_palace(TuViPalaceRole::QuanLoc, |_, p| {
            stars(p).any(|s| s.star == TuViStar::KinhDuong)
        });
        let query = tuvi_palace_query(&chart, branch);
        let level = TuViStar::KinhDuong.brightness_at(branch).unwrap();
        assert!(
            query.lookups[0]
                .facts
                .contains(&format!("bright:KinhDuong:{level:?}"))
        );
        let brightness_key = format!("tuvi.star_brightness/KinhDuong/{level:?}");
        assert!(query.keys.contains(&brightness_key));
        assert!(matches!(level, StarBrightness::Dac | StarBrightness::Ham));
    }

    #[test]
    fn an_empty_quan_loc_borrows_the_opposite_chinh_tinh() {
        let (chart, branch) = find_palace(TuViPalaceRole::QuanLoc, |_, palace| {
            palace.main_stars.is_empty()
        });
        let query = tuvi_palace_query(&chart, branch);
        assert_eq!(query.lookups.len(), 2);
        let own = &query.lookups[0];
        assert!(own.anchors.contains(&"QuanLoc/~NoMain".to_string()));
        assert!(own.facts.contains(&"no_main".to_string()));
        let borrowed = &query.lookups[1];
        let opposite = chart.palace(branch.clash());
        assert_eq!(borrowed.anchors.len(), 2);
        for placed in &opposite.main_stars {
            assert!(
                borrowed
                    .anchors
                    .contains(&format!("QuanLoc/{:?}", placed.star))
            );
            assert!(borrowed.facts.contains(&format!("with:{:?}", placed.star)));
            let level = placed.brightness.unwrap();
            assert!(
                borrowed
                    .facts
                    .contains(&format!("bright:{:?}:{level:?}", placed.star))
            );
        }
        assert!(
            borrowed
                .facts
                .contains(&format!("branch:{:?}", opposite.branch))
        );
        assert!(!borrowed.facts.contains(&format!("branch:{branch:?}")));
        assert!(!borrowed.facts.contains(&"no_main".to_string()));
        assert!(!borrowed.facts.contains(&"solo".to_string()));
    }

    #[test]
    fn a_palace_without_chinh_tinh_always_faces_two() {
        for tu_vi in Branch::ALL {
            let stars = chinh_tinh_branches(tu_vi);
            let count = |at: Branch| stars.iter().filter(|(_, branch)| *branch == at).count();
            for palace in Branch::ALL.into_iter().filter(|&b| count(b) == 0) {
                assert_eq!(count(palace.clash()), 2, "Tử Vi at {tu_vi:?}");
            }
        }
    }

    #[test]
    fn only_the_eight_palaces_of_the_book_borrow() {
        for role in [
            TuViPalaceRole::Menh,
            TuViPalaceRole::DienTrach,
            TuViPalaceRole::TatAch,
            TuViPalaceRole::PhuThe,
        ] {
            let (chart, branch) = find_palace(role, |_, palace| palace.main_stars.is_empty());
            assert_eq!(
                tuvi_palace_query(&chart, branch).lookups.len(),
                1,
                "{role:?}"
            );
        }
        assert_eq!(
            TuViPalaceRole::ALL
                .iter()
                .filter(|&&r| borrows_opposite(r))
                .count(),
            8
        );
    }

    #[test]
    fn the_palace_of_than_also_reads_the_passages_of_menh() {
        let (chart, branch) = find_palace(TuViPalaceRole::QuanLoc, |_, p| {
            p.is_than && !p.main_stars.is_empty()
        });
        let query = tuvi_palace_query(&chart, branch);
        let star = chart.palace(branch).main_stars[0].star;
        assert!(
            query.lookups[0]
                .anchors
                .contains(&format!("QuanLoc/{star:?}"))
        );
        assert!(query.lookups[0].anchors.contains(&format!("Menh/{star:?}")));
        assert!(query.lookups[0].facts.contains(&"than".to_string()));
        assert!(!query.lookups[0].facts.contains(&"is_menh".to_string()));
    }

    #[test]
    fn only_the_menh_palace_itself_is_menh() {
        let chart = chart(2024, 3, 12, Branch::Ngo, Gender::Male);
        for palace in &chart.palaces {
            let facts = &tuvi_palace_query(&chart, palace.branch).lookups[0].facts;
            let is_menh = facts.contains(&"is_menh".to_string());
            assert_eq!(
                is_menh,
                palace.role == TuViPalaceRole::Menh,
                "{:?}",
                palace.role
            );
        }
    }

    #[test]
    fn menh_with_than_does_not_repeat_its_anchors() {
        let (chart, branch) = find_palace(TuViPalaceRole::Menh, |_, p| p.is_than);
        let anchors = &tuvi_palace_query(&chart, branch).lookups[0].anchors;
        assert_eq!(anchors.len(), anchors.iter().collect::<BTreeSet<_>>().len());
    }

    #[test]
    fn the_person_and_the_menh_palace_are_facts_of_every_palace() {
        let chart = chart(2024, 3, 12, Branch::Ngo, Gender::Female);
        let branch = chart.palace_of_role(TuViPalaceRole::TaiBach).branch;
        let facts = &tuvi_palace_query(&chart, branch).lookups[0].facts;
        assert!(facts.contains(&"gender:Female".to_string()));
        assert!(facts.contains(&format!("polarity:{:?}", chart.polarity)));
        assert!(facts.contains(&format!("harmony:{:?}", chart.harmony)));
        assert!(facts.contains(&format!("ban_menh:{:?}", chart.ban_menh.element())));
        assert!(facts.contains(&format!("year_stem:{:?}", chart.year_can_chi.stem)));
        assert!(facts.contains(&format!("year_branch:{:?}", chart.year_can_chi.branch)));
        for star in stars(chart.palace_of_role(TuViPalaceRole::Menh)) {
            assert!(facts.contains(&format!("menh:{:?}", star.star)));
        }
    }
}
