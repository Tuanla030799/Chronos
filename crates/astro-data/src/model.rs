//! The JSON schema of `data/interpretations/`, section 3 and section 7 of the specification.

use astro_core::almanac::Luck;
use astro_core::tuvi::{FourTransformations, StarBrightness, TuViPalaceRole, TuViStar};
use astro_core::{Branch, Gender, Stem};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Whether the project owner has approved an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Generated, not reviewed. Reaches development builds only.
    Draft,
    /// Reviewed by the project owner.
    Approved,
}

/// How an item relates to its source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Rewritten in modern language, the default (D3).
    Paraphrase,
    /// Verbatim, for short verses only.
    Quote,
    /// Translated from Classical Chinese.
    Translation,
    /// Summarised in the spirit of a canonical book, not taken from a page (D4).
    CanonicalSummary,
}

/// A topic the app may hide behind a tap (D9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Sensitivity {
    /// Death, short life.
    Death,
    /// Illness and disability.
    Illness,
    /// Prison and lawsuits.
    Legal,
    /// Sexual conduct.
    Sexuality,
    /// Loss of relatives.
    FamilyLoss,
}

impl Sensitivity {
    /// Bit of this tag in the `sensitivity` mask of the build.
    pub const fn bit(self) -> u32 {
        1 << self as u32
    }
}

/// Mask of `tags`.
pub fn sensitivity_mask(tags: &[Sensitivity]) -> u32 {
    tags.iter().fold(0, |mask, tag| mask | tag.bit())
}

/// Where an item comes from (section 3.3).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    /// Id in `sources.json`.
    pub source: String,
    /// How the item relates to the source.
    pub source_type: SourceType,
    /// Printed pages holding the original words.
    #[serde(default)]
    pub pages: Vec<u16>,
    /// Place in a source without pages.
    pub locator: Option<String>,
    /// Original phrases, typos included, that must appear on those pages.
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Something to do or avoid, from `vocab/activities.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivityNote {
    /// Id of the activity.
    pub activity: String,
    /// Optional remark.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Which Dipper a star belongs to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Dau {
    /// Nam Đẩu
    NamDau,
    /// Bắc Đẩu
    BacDau,
    /// Nam Bắc Đẩu, Tử Vi only
    NamBacDau,
}

/// The heading of a star in the book.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StarProfile {
    /// Dipper.
    pub dau: Dau,
    /// Element as printed, e.g. "Dương Thổ".
    pub element: String,
    /// Labels as printed, e.g. "Đế tinh".
    pub labels: Vec<String>,
}

/// An item looked up by key. Fields below `status` belong to some collections only.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// The key.
    pub key: Option<Value>,
    /// The keys, when one text serves several.
    pub keys: Option<Vec<Value>>,
    /// One sentence for a card or a calendar cell.
    pub summary: String,
    /// Paragraphs.
    #[serde(default)]
    pub body: Vec<String>,
    /// Other names.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Source.
    pub source_ref: SourceRef,
    /// Sensitive topics.
    #[serde(default)]
    pub sensitivity: Vec<Sensitivity>,
    /// Review state.
    pub status: Status,
    /// How the text departs from a misprint of the source. Review only, never built.
    pub correction: Option<String>,
    /// Good for.
    pub good_for: Option<Vec<ActivityNote>>,
    /// To avoid.
    pub avoid: Option<Vec<ActivityNote>>,
    /// Verse.
    pub verse: Option<String>,
    /// Practical folk advice, never a ritual (D5).
    pub advice: Option<String>,
    /// Image of a Nạp Âm.
    pub image: Option<String>,
    /// Character.
    pub traits: Option<Vec<String>>,
    /// General fortune.
    pub fortune: Option<Vec<String>>,
    /// Fortune in one use of a star, when it differs from `luck()`.
    pub verdict: Option<Luck>,
    /// Meaning of a Nine Star in its time (D6).
    pub auspicious_meaning: Option<String>,
    /// Meaning of a Nine Star out of its time (D6).
    pub inauspicious_meaning: Option<String>,
    /// Heading of a star.
    pub profile: Option<StarProfile>,
}

impl Entry {
    /// The collection-specific fields in use, as a JSON object.
    pub fn extra(&self) -> Map<String, Value> {
        fn put<T: Serialize>(map: &mut Map<String, Value>, name: &str, value: &Option<T>) {
            if let Some(value) = value {
                let value = serde_json::to_value(value).expect("schema types serialize");
                map.insert(name.to_string(), value);
            }
        }
        let mut map = Map::new();
        put(&mut map, "good_for", &self.good_for);
        put(&mut map, "avoid", &self.avoid);
        put(&mut map, "verse", &self.verse);
        put(&mut map, "advice", &self.advice);
        put(&mut map, "image", &self.image);
        put(&mut map, "traits", &self.traits);
        put(&mut map, "fortune", &self.fortune);
        put(&mut map, "verdict", &self.verdict);
        put(&mut map, "auspicious_meaning", &self.auspicious_meaning);
        put(&mut map, "inauspicious_meaning", &self.inauspicious_meaning);
        put(&mut map, "profile", &self.profile);
        map
    }
}

/// A star, or a group of `vocab/star_groups.json` written `@Name`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StarRef {
    /// One star.
    Star(TuViStar),
    /// A group, or a misspelt star: validation tells them apart.
    Group(String),
}

/// The heading a passage belongs to in the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Section {
    /// Đại cương
    DaiCuong,
    /// Nữ mệnh
    NuMenh,
    /// Phú giải
    PhuGiai,
}

/// How to find a passage. No `stars`, `transformations` or `no_main_star` means the whole palace.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    /// Palace.
    pub role: TuViPalaceRole,
    /// The passage applies when one of these stars sits in the palace.
    #[serde(default)]
    pub stars: Vec<StarRef>,
    /// The passage applies when one of these sits in the palace.
    #[serde(default)]
    pub transformations: Vec<FourTransformations>,
    /// The passage applies to a palace without Chính Tinh.
    #[serde(default)]
    pub no_main_star: bool,
}

/// Tuần or Triệt on a palace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TuanTriet {
    /// Tuần
    Tuan,
    /// Triệt
    Triet,
    /// Tuần or Triệt
    Either,
    /// Neither of them
    Neither,
}

/// One alternative of a condition: all its fields must hold (section 7.3).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct When {
    /// The palace is at one of these branches.
    #[serde(default)]
    pub branches: Vec<Branch>,
    /// The anchor star has one of these brightness levels.
    #[serde(default)]
    pub brightness: Vec<StarBrightness>,
    /// Whether the palace has exactly one Chính Tinh.
    pub solo: Option<bool>,
    /// All these stars sit in the palace; anchor stars may be listed.
    #[serde(default)]
    pub with_all: Vec<StarRef>,
    /// None of these stars sits in the palace.
    #[serde(default)]
    pub not_with: Vec<StarRef>,
    /// One of these stars meets the palace.
    #[serde(default)]
    pub meets_any: Vec<StarRef>,
    /// All these stars meet the palace.
    #[serde(default)]
    pub meets_all: Vec<StarRef>,
    /// None of these stars meets the palace.
    #[serde(default)]
    pub not_meets: Vec<StarRef>,
    /// All these stars sit in the Nhị Hợp palace.
    #[serde(default)]
    pub paired_with: Vec<StarRef>,
    /// All these stars sit in the two neighbouring palaces.
    #[serde(default)]
    pub flanked_by: Vec<StarRef>,
    /// One of these transformations meets the palace.
    #[serde(default)]
    pub transformations: Vec<FourTransformations>,
    /// All these transformations meet the palace.
    #[serde(default)]
    pub transformations_all: Vec<FourTransformations>,
    /// One of these transformations sits in the palace.
    #[serde(default)]
    pub with_transformations: Vec<FourTransformations>,
    /// Tuần, Triệt on the palace.
    pub tuan_triet: Option<TuanTriet>,
    /// Whether the Thân palace falls here.
    pub than_here: Option<bool>,
    /// Sex of the person.
    pub gender: Option<Gender>,
    /// Stem of the birth year.
    #[serde(default)]
    pub year_stems: Vec<Stem>,
    /// Branch of the birth year.
    #[serde(default)]
    pub year_branches: Vec<Branch>,
    /// One of these stars sits in the Mệnh palace.
    #[serde(default)]
    pub menh_any: Vec<StarRef>,
    /// All these stars sit in the Mệnh palace.
    #[serde(default)]
    pub menh_all: Vec<StarRef>,
}

/// A passage of `tuvi.passage` (section 7.2).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Passage {
    /// Stable id, `<palace>/<star>/<number>`.
    pub id: String,
    /// How to find it.
    pub anchor: Anchor,
    /// Heading in the book.
    pub section: Option<Section>,
    /// Verse of a Phú giải block.
    pub verse: Option<String>,
    /// Alternatives: one must hold. Empty means always.
    #[serde(default)]
    pub when: Vec<When>,
    /// The part of the condition no machine can check; the app shows it as "Nếu …".
    pub condition_text: Option<String>,
    /// Paragraphs.
    pub body: Vec<String>,
    /// Source.
    pub source_ref: SourceRef,
    /// Sensitive topics.
    #[serde(default)]
    pub sensitivity: Vec<Sensitivity>,
    /// Review state.
    pub status: Status,
    /// How the data departs from a misprint of the source. Review only, never built.
    pub correction: Option<String>,
}

/// A key the source does not cover, checked and recorded (section 3.5).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    /// The key.
    pub key: Value,
    /// Why there is no text.
    pub gap: String,
    /// What was read to find out.
    pub checked: String,
}

/// One file under `data/interpretations/<module>/`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataFile {
    /// Collection id.
    pub collection: String,
    /// Batch id.
    pub batch: String,
    /// Items, as parsed later against the collection.
    #[serde(default)]
    pub entries: Vec<Value>,
    /// Recorded gaps.
    #[serde(default)]
    pub gaps: Vec<Gap>,
}
