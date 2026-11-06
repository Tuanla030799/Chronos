//! The registry of collections: for each one, the real type of its keys.
//!
//! Keys are written with the variant names of `astro-core`, which are also their serde names, so
//! a variant added to an enum shows up here as a missing key and a removed one as a bad key.

use astro_core::almanac::{
    AlmanacGod, HoangOc, KimLau, Luck, Mansion, PathGod, TamTai, Truc, VietnamTaboo,
};
use astro_core::bazi_analysis::{BaziStar, LifeStage, TenGod};
use astro_core::fengshui::{EightMansionsStar, NineStar, Trigram};
use astro_core::tuvi::{FourTransformations, StarBrightness, StarNature, TuViPalaceRole, TuViStar};
use astro_core::{Branch, CanChi, NaYin, Stem};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Id of the collection of passages, which has no key type of its own.
pub const PASSAGE_COLLECTION: &str = "tuvi.passage";

/// What a key says about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
    /// Path after the collection, e.g. `TuVi/Mieu`.
    pub path: String,
    /// Display name, from `astro-core`.
    pub title: String,
    /// `Cat`, `Trung` or `Hung`, from `astro-core`.
    pub luck: Option<&'static str>,
}

/// A type usable as a key.
pub trait Key: DeserializeOwned + PartialEq + Sized {
    /// Every valid key.
    fn all() -> Vec<Self>;
    /// Path of the key.
    fn path(&self) -> String;
    /// Display name.
    fn title(&self) -> String;
    /// Fortune, when `astro-core` has one.
    fn luck(&self) -> Option<&'static str> {
        None
    }
}

const fn luck_name(luck: Luck) -> &'static str {
    match luck {
        Luck::Cat => "Cat",
        Luck::Hung => "Hung",
    }
}

const fn nature_name(nature: StarNature) -> &'static str {
    match nature {
        StarNature::Cat => "Cat",
        StarNature::Trung => "Trung",
        StarNature::Hung => "Hung",
    }
}

macro_rules! enum_key {
    ($type:ty, $all:expr, $title:expr $(, $luck:expr)?) => {
        impl Key for $type {
            fn all() -> Vec<Self> {
                $all.to_vec()
            }
            fn path(&self) -> String {
                format!("{self:?}")
            }
            fn title(&self) -> String {
                ($title)(*self).to_string()
            }
            $(fn luck(&self) -> Option<&'static str> {
                Some(($luck)(*self))
            })?
        }
    };
}

enum_key!(Truc, Truc::ALL, Truc::name_vi);
enum_key!(Mansion, Mansion::ALL, Mansion::name_vi, |value: Mansion| {
    luck_name(value.luck())
});
enum_key!(PathGod, PathGod::ALL, PathGod::name_vi, |value: PathGod| {
    if value.is_hoang_dao() { "Cat" } else { "Hung" }
});
enum_key!(
    AlmanacGod,
    AlmanacGod::ALL,
    AlmanacGod::name_vi,
    |value: AlmanacGod| luck_name(value.luck())
);
enum_key!(VietnamTaboo, VietnamTaboo::ALL, VietnamTaboo::name_vi);
enum_key!(TamTai, TamTai::ALL, TamTai::name_vi);
enum_key!(KimLau, KimLau::ALL, KimLau::name_vi);
enum_key!(HoangOc, HoangOc::ALL, HoangOc::name_vi, |value: HoangOc| {
    luck_name(value.luck())
});
enum_key!(NaYin, NaYin::ALL, NaYin::name_vi);
enum_key!(Stem, Stem::ALL, Stem::name_vi);
enum_key!(TenGod, TenGod::ALL, TenGod::name_vi);
enum_key!(LifeStage, LifeStage::ALL, LifeStage::name_vi);
enum_key!(Trigram, Trigram::ALL, Trigram::name_vi);
enum_key!(
    EightMansionsStar,
    EightMansionsStar::ALL,
    EightMansionsStar::name_vi,
    |value: EightMansionsStar| luck_name(value.luck())
);
enum_key!(
    NineStar,
    NineStar::ALL,
    NineStar::name_vi,
    |value: NineStar| { luck_name(value.luck()) }
);
enum_key!(
    TuViStar,
    TuViStar::ALL,
    TuViStar::name_vi,
    |value: TuViStar| { nature_name(value.nature()) }
);
enum_key!(
    FourTransformations,
    FourTransformations::ALL,
    FourTransformations::name_vi,
    |value: FourTransformations| nature_name(value.nature())
);
enum_key!(TuViPalaceRole, TuViPalaceRole::ALL, TuViPalaceRole::name_vi);

/// The three age restrictions as topics of `almanac.restriction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum AgeTopic {
    /// Tam Tai
    TamTai,
    /// Kim Lâu
    KimLau,
    /// Hoang Ốc
    HoangOc,
}

impl AgeTopic {
    const ALL: [AgeTopic; 3] = [AgeTopic::TamTai, AgeTopic::KimLau, AgeTopic::HoangOc];

    const fn name_vi(self) -> &'static str {
        match self {
            AgeTopic::TamTai => "Tam Tai",
            AgeTopic::KimLau => "Kim Lâu",
            AgeTopic::HoangOc => "Hoang Ốc",
        }
    }
}

enum_key!(AgeTopic, AgeTopic::ALL, AgeTopic::name_vi);

/// Where a star of the Eight Mansions is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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
    const ALL: [EightUse; 4] = [
        EightUse::HuongNha,
        EightUse::Bep,
        EightUse::BanLamViec,
        EightUse::KetHon,
    ];

    const fn name_vi(self) -> &'static str {
        match self {
            EightUse::HuongNha => "Hướng nhà",
            EightUse::Bep => "Bếp",
            EightUse::BanLamViec => "Bàn làm việc",
            EightUse::KetHon => "Kết hôn",
        }
    }
}

/// A Thần Sát of the chart: the ten [`BaziStar`] and two fields of `BaziAnalysis`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ShenSha {
    /// One of the stars.
    Star(BaziStar),
    /// Không Vong or Khôi Canh.
    Marker(ShenShaMarker),
}

/// The two Thần Sát that are not a [`BaziStar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ShenShaMarker {
    /// Không Vong
    KhongVong,
    /// Khôi Canh
    KhoiCanh,
}

impl Key for ShenSha {
    fn all() -> Vec<Self> {
        BaziStar::ALL
            .into_iter()
            .map(ShenSha::Star)
            .chain([
                ShenSha::Marker(ShenShaMarker::KhongVong),
                ShenSha::Marker(ShenShaMarker::KhoiCanh),
            ])
            .collect()
    }
    fn path(&self) -> String {
        match self {
            ShenSha::Star(star) => format!("{star:?}"),
            ShenSha::Marker(marker) => format!("{marker:?}"),
        }
    }
    fn title(&self) -> String {
        match self {
            ShenSha::Star(star) => star.name_vi().to_string(),
            ShenSha::Marker(ShenShaMarker::KhongVong) => "Không Vong".to_string(),
            ShenSha::Marker(ShenShaMarker::KhoiCanh) => "Khôi Canh".to_string(),
        }
    }
}

/// A Can Chi of the 60-year cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct CycleKey(pub Stem, pub Branch);

impl Key for CycleKey {
    fn all() -> Vec<Self> {
        (0..60)
            .map(|index| {
                let can_chi = CanChi::from_cycle_index(index);
                CycleKey(can_chi.stem, can_chi.branch)
            })
            .collect()
    }
    fn path(&self) -> String {
        format!("{:?}/{:?}", self.0, self.1)
    }
    fn title(&self) -> String {
        CanChi {
            stem: self.0,
            branch: self.1,
        }
        .name_vi()
    }
}

/// A star of the Eight Mansions in one use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct EightStarUse(pub EightMansionsStar, pub EightUse);

impl Key for EightStarUse {
    fn all() -> Vec<Self> {
        EightMansionsStar::ALL
            .into_iter()
            .flat_map(|star| EightUse::ALL.map(|usage| EightStarUse(star, usage)))
            .collect()
    }
    fn path(&self) -> String {
        format!("{:?}/{:?}", self.0, self.1)
    }
    fn title(&self) -> String {
        format!("{} · {}", self.0.name_vi(), self.1.name_vi())
    }
}

/// A Chính Tinh at a brightness it really reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct StarBright(pub TuViStar, pub StarBrightness);

impl Key for StarBright {
    fn all() -> Vec<Self> {
        TuViStar::ALL
            .into_iter()
            .flat_map(|star| {
                StarBrightness::ALL.into_iter().filter_map(move |level| {
                    Branch::ALL
                        .into_iter()
                        .any(|branch| star.brightness_at(branch) == Some(level))
                        .then_some(StarBright(star, level))
                })
            })
            .collect()
    }
    fn path(&self) -> String {
        format!("{:?}/{:?}", self.0, self.1)
    }
    fn title(&self) -> String {
        format!("{} · {}", self.0.name_vi(), self.1.name_vi())
    }
}

/// A palace and a Chính Tinh, the anchors counted by section 7.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PalaceStar(pub TuViPalaceRole, pub TuViStar);

impl Key for PalaceStar {
    fn all() -> Vec<Self> {
        TuViPalaceRole::ALL
            .into_iter()
            .flat_map(|role| {
                TuViStar::ALL[..14]
                    .iter()
                    .map(move |&star| PalaceStar(role, star))
            })
            .collect()
    }
    fn path(&self) -> String {
        format!("{:?}/{:?}", self.0, self.1)
    }
    fn title(&self) -> String {
        format!("{} · {}", self.1.name_vi(), self.0.name_vi())
    }
}

/// A collection: its id, the fields of [`crate::model::Entry`] it may use, and its keys.
#[derive(Clone, Copy)]
pub struct Collection {
    /// Id, e.g. `almanac.truc`.
    pub id: &'static str,
    /// Collection-specific fields allowed in an entry.
    pub extras: &'static [&'static str],
    /// Reads a key from JSON.
    pub parse: fn(&Value) -> Result<KeyInfo, String>,
    /// Every valid key, as paths.
    pub all: fn() -> Vec<String>,
}

impl Collection {
    /// The module of the collection, e.g. `almanac`.
    pub fn module(&self) -> &'static str {
        self.id.split('.').next().unwrap_or(self.id)
    }
}

fn parse_key<K: Key>(value: &Value) -> Result<KeyInfo, String> {
    let key: K =
        serde_json::from_value(value.clone()).map_err(|error| format!("{value}: {error}"))?;
    if !K::all().contains(&key) {
        return Err(format!("{value} không tồn tại trong astro-core"));
    }
    Ok(KeyInfo {
        path: key.path(),
        title: key.title(),
        luck: key.luck(),
    })
}

fn all_paths<K: Key>() -> Vec<String> {
    K::all().iter().map(Key::path).collect()
}

const fn collection<K: Key>(id: &'static str, extras: &'static [&'static str]) -> Collection {
    Collection {
        id,
        extras,
        parse: parse_key::<K>,
        all: all_paths::<K>,
    }
}

const DAY_ITEM: &[&str] = &["good_for", "avoid"];
const ADVICE: &[&str] = &["advice"];

/// Every collection of sections 4 to 7, passages included, whose keys are their anchors.
pub const COLLECTIONS: &[Collection] = &[
    collection::<Truc>("almanac.truc", DAY_ITEM),
    collection::<Mansion>("almanac.mansion", &["good_for", "avoid", "verse"]),
    collection::<PathGod>("almanac.path_god", DAY_ITEM),
    collection::<AlmanacGod>("almanac.god", DAY_ITEM),
    collection::<VietnamTaboo>("almanac.taboo", &["avoid", "verse"]),
    collection::<AgeTopic>("almanac.restriction", ADVICE),
    collection::<TamTai>("almanac.tam_tai", ADVICE),
    collection::<KimLau>("almanac.kim_lau", ADVICE),
    collection::<HoangOc>("almanac.hoang_oc", ADVICE),
    collection::<NaYin>("bazi.nayin", &["image", "traits", "fortune"]),
    collection::<CycleKey>("bazi.can_chi", &["traits", "fortune"]),
    collection::<Stem>("bazi.day_master", &["traits"]),
    collection::<TenGod>("bazi.ten_god", &[]),
    collection::<LifeStage>("bazi.life_stage", &[]),
    collection::<ShenSha>("bazi.shen_sha", &[]),
    collection::<Trigram>("fengshui.kua", &[]),
    collection::<EightMansionsStar>("fengshui.eight_star", &[]),
    collection::<EightStarUse>("fengshui.eight_star_use", &["verdict"]),
    collection::<NineStar>(
        "fengshui.nine_star",
        &["auspicious_meaning", "inauspicious_meaning"],
    ),
    collection::<TuViStar>("tuvi.star", &["profile"]),
    collection::<StarBright>("tuvi.star_brightness", &[]),
    collection::<FourTransformations>("tuvi.transformation", &[]),
    collection::<TuViPalaceRole>("tuvi.palace", &[]),
    collection::<PalaceStar>(PASSAGE_COLLECTION, &[]),
];

/// The collection `id`.
pub fn find(id: &str) -> Option<&'static Collection> {
    COLLECTIONS.iter().find(|collection| collection.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn count(id: &str) -> usize {
        (find(id).unwrap().all)().len()
    }

    #[test]
    fn collections_have_the_sizes_of_the_specification() {
        let sizes = [
            ("almanac.truc", 12),
            ("almanac.mansion", 28),
            ("almanac.path_god", 12),
            ("almanac.god", 15),
            ("almanac.taboo", 9),
            ("almanac.restriction", 3),
            ("almanac.tam_tai", 3),
            ("almanac.kim_lau", 4),
            ("almanac.hoang_oc", 6),
            ("bazi.nayin", 30),
            ("bazi.can_chi", 60),
            ("bazi.day_master", 10),
            ("bazi.ten_god", 10),
            ("bazi.life_stage", 12),
            ("bazi.shen_sha", 12),
            ("fengshui.kua", 8),
            ("fengshui.eight_star", 8),
            ("fengshui.eight_star_use", 32),
            ("fengshui.nine_star", 9),
            ("tuvi.star", 105),
            ("tuvi.star_brightness", 56),
            ("tuvi.transformation", 4),
            ("tuvi.palace", 12),
            (PASSAGE_COLLECTION, 168),
        ];
        assert_eq!(sizes.len(), COLLECTIONS.len());
        for (id, size) in sizes {
            assert_eq!(count(id), size, "{id}");
        }
        let keyed: usize = sizes
            .iter()
            .filter(|(id, _)| *id != PASSAGE_COLLECTION)
            .map(|(_, size)| size)
            .sum();
        assert_eq!(keyed, 460);
    }

    #[test]
    fn every_path_reads_back_as_the_same_key() {
        for collection in COLLECTIONS {
            for path in (collection.all)() {
                let parts: Vec<&str> = path.split('/').collect();
                let value = if parts.len() == 1 {
                    json!(parts[0])
                } else {
                    json!(parts)
                };
                let info = (collection.parse)(&value).unwrap();
                assert_eq!(info.path, path, "{}", collection.id);
                assert!(!info.title.is_empty());
            }
        }
    }

    #[test]
    fn keys_outside_the_enums_are_rejected() {
        let truc = find("almanac.truc").unwrap();
        assert!((truc.parse)(&json!("Kien")).is_ok());
        assert!((truc.parse)(&json!("Kiên")).is_err());
        assert!((truc.parse)(&json!("kien")).is_err());
    }

    #[test]
    fn combinations_the_engine_cannot_produce_are_rejected() {
        let brightness = find("tuvi.star_brightness").unwrap();
        assert!((brightness.parse)(&json!(["TuVi", "Binh"])).is_ok());
        assert!((brightness.parse)(&json!(["TuVi", "Ham"])).is_err());
        assert!((brightness.parse)(&json!(["LiemTrinh", "Binh"])).is_err());
        assert!((brightness.parse)(&json!(["LocTon", "Dac"])).is_err());
        let can_chi = find("bazi.can_chi").unwrap();
        assert!((can_chi.parse)(&json!(["Giap", "Ty"])).is_ok());
        assert!((can_chi.parse)(&json!(["Giap", "Suu"])).is_err());
    }

    #[test]
    fn names_shared_by_several_enums_stay_apart_by_collection() {
        let god = find("almanac.god").unwrap();
        let path_god = find("almanac.path_god").unwrap();
        let star = find("tuvi.star").unwrap();
        for collection in [god, path_god, star] {
            assert_eq!(
                (collection.parse)(&json!("ThienDuc")).unwrap().path,
                "ThienDuc"
            );
        }
        assert_eq!((god.parse)(&json!("ThienDuc")).unwrap().luck, Some("Cat"));
    }

    #[test]
    fn shen_sha_holds_the_stars_and_the_two_markers() {
        let shen_sha = find("bazi.shen_sha").unwrap();
        assert_eq!((shen_sha.parse)(&json!("DaoHoa")).unwrap().title, "Đào Hoa");
        assert_eq!(
            (shen_sha.parse)(&json!("KhoiCanh")).unwrap().title,
            "Khôi Canh"
        );
    }

    #[test]
    fn luck_comes_from_the_engine() {
        let path_god = find("almanac.path_god").unwrap();
        assert_eq!(
            (path_god.parse)(&json!("ThanhLong")).unwrap().luck,
            Some("Cat")
        );
        assert_eq!(
            (path_god.parse)(&json!("BachHo")).unwrap().luck,
            Some("Hung")
        );
        let star = find("tuvi.star").unwrap();
        assert_eq!((star.parse)(&json!("ThatSat")).unwrap().luck, Some("Hung"));
    }
}
