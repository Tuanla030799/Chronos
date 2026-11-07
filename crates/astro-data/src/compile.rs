//! Compiles validated items to the rows of the build (section 8 of the specification).

use crate::model::{Anchor, When, sensitivity_mask};
use crate::model::{SourceRef, SourceType, TuanTriet};
use crate::reach::expand;
use crate::source::Source;
use crate::validate::{ValidEntry, ValidPassage, Validated};
pub use astro_core::content::Gate;
use astro_core::tuvi::TuViStar;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// An item of the build, looked up by key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryRow {
    /// Full key.
    pub key: String,
    /// Collection.
    pub collection: String,
    /// Display name, from `astro-core`.
    pub title: String,
    /// Fortune, from `astro-core`.
    pub luck: Option<String>,
    /// Summary.
    pub summary: String,
    /// Paragraphs.
    pub body: Vec<String>,
    /// Collection-specific fields.
    pub extra: Option<Value>,
    /// Where the text comes from, for display.
    pub citation: String,
    /// Mask of sensitive topics.
    pub sensitivity: u32,
    /// Whether the item is not approved yet.
    pub draft: bool,
}

/// A passage of the build.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassageRow {
    /// Row id, assigned at build time.
    pub id: u32,
    /// Stable id.
    pub stable_id: String,
    /// `Primary` for a firm statement, `More` for one shown as "Nếu …" under "Xem thêm" (D12).
    pub tier: String,
    /// Heading in the book.
    pub section: Option<String>,
    /// Verse.
    pub verse: Option<String>,
    /// Alternatives; none means always.
    pub gates: Vec<Gate>,
    /// Condition shown as "Nếu …".
    pub condition_text: Option<String>,
    /// Paragraphs.
    pub body: Vec<String>,
    /// Where the text comes from, for display.
    pub citation: String,
    /// Mask of sensitive topics.
    pub sensitivity: u32,
    /// Whether the item is not approved yet.
    pub draft: bool,
}

/// The rows of a build.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Compiled {
    /// Items by key, sorted.
    pub entries: Vec<EntryRow>,
    /// Passages in book order.
    pub passages: Vec<PassageRow>,
    /// Passage ids by anchor, sorted.
    pub anchors: BTreeMap<String, Vec<u32>>,
}

/// Citation of an item, for display.
pub fn citation(reference: &SourceRef, source: &Source) -> String {
    if reference.source_type == SourceType::CanonicalSummary {
        return format!("Tổng hợp theo truyền thống {}", source.title);
    }
    let place = match (
        reference.pages.first(),
        reference.pages.last(),
        &reference.locator,
    ) {
        (Some(first), Some(last), _) if first == last => format!("tr. {first}"),
        (Some(_), Some(_), _) => {
            let pages: Vec<String> = reference.pages.iter().map(u16::to_string).collect();
            format!("tr. {}", pages.join(", "))
        }
        (_, _, Some(locator)) => locator.clone(),
        _ => return source.title.clone(),
    };
    format!("{}, {place}", source.title)
}

fn facts<T>(prefix: &str, items: &[T], name: impl Fn(&T) -> String) -> Vec<String> {
    items
        .iter()
        .map(|item| format!("{prefix}:{}", name(item)))
        .collect()
}

fn group(gate: &mut Gate, facts: Vec<String>) {
    if !facts.is_empty() {
        gate.any.push(facts);
    }
}

/// The stars and transformations of `anchor` whose brightness a condition is about.
pub fn bright_subjects(
    anchor: &Anchor,
    groups: &BTreeMap<String, Vec<TuViStar>>,
) -> Result<Vec<String>, String> {
    let stars = expand(&anchor.stars, groups)?;
    let mut subjects: Vec<String> = stars.iter().map(|star| format!("{star:?}")).collect();
    subjects.extend(anchor.transformations.iter().map(|hoa| format!("{hoa:?}")));
    Ok(subjects)
}

/// The gate of one alternative; `subjects` are what its brightness is about.
pub fn gate(
    when: &When,
    subjects: &[String],
    groups: &BTreeMap<String, Vec<TuViStar>>,
) -> Result<Gate, String> {
    let debug = |star: &TuViStar| format!("{star:?}");
    let mut gate = Gate::default();
    group(
        &mut gate,
        facts("branch", &when.branches, |b| format!("{b:?}")),
    );
    let mut bright = Vec::new();
    for subject in subjects {
        for level in &when.brightness {
            bright.push(format!("bright:{subject}:{level:?}"));
        }
    }
    group(&mut gate, bright);
    match when.solo {
        Some(true) => gate.all.push("solo".to_string()),
        Some(false) => gate.none.push("solo".to_string()),
        None => {}
    }
    gate.all
        .extend(facts("with", &expand(&when.with_all, groups)?, debug));
    gate.none
        .extend(facts("with", &expand(&when.not_with, groups)?, debug));
    group(
        &mut gate,
        facts("meet", &expand(&when.meets_any, groups)?, debug),
    );
    gate.all
        .extend(facts("meet", &expand(&when.meets_all, groups)?, debug));
    gate.none
        .extend(facts("meet", &expand(&when.not_meets, groups)?, debug));
    gate.all
        .extend(facts("pair", &expand(&when.paired_with, groups)?, debug));
    gate.all
        .extend(facts("flank", &expand(&when.flanked_by, groups)?, debug));
    group(
        &mut gate,
        facts("hoa", &when.transformations, |t| format!("{t:?}")),
    );
    gate.all
        .extend(facts("hoa", &when.transformations_all, |t| {
            format!("{t:?}")
        }));
    group(
        &mut gate,
        facts("hoa_with", &when.with_transformations, |t| format!("{t:?}")),
    );
    match when.tuan_triet {
        Some(TuanTriet::Tuan) => gate.all.push("tuan".to_string()),
        Some(TuanTriet::Triet) => gate.all.push("triet".to_string()),
        Some(TuanTriet::Either) => gate.any.push(vec!["tuan".to_string(), "triet".to_string()]),
        Some(TuanTriet::Neither) => {
            gate.none.extend(["tuan".to_string(), "triet".to_string()]);
        }
        None => {}
    }
    match when.is_menh {
        Some(true) => gate.all.push("is_menh".to_string()),
        Some(false) => gate.none.push("is_menh".to_string()),
        None => {}
    }
    match when.than_here {
        Some(true) => gate.all.push("than".to_string()),
        Some(false) => gate.none.push("than".to_string()),
        None => {}
    }
    if let Some(gender) = when.gender {
        gate.all.push(format!("gender:{gender:?}"));
    }
    group(
        &mut gate,
        facts("polarity", &when.polarity, |p| format!("{p:?}")),
    );
    if let Some(harmony) = when.harmony {
        gate.all.push(format!("harmony:{harmony:?}"));
    }
    group(
        &mut gate,
        facts("ban_menh", &when.ban_menh, |e| format!("{e:?}")),
    );
    group(
        &mut gate,
        facts("year_stem", &when.year_stems, |s| format!("{s:?}")),
    );
    group(
        &mut gate,
        facts("year_branch", &when.year_branches, |b| format!("{b:?}")),
    );
    group(
        &mut gate,
        facts("menh", &expand(&when.menh_any, groups)?, debug),
    );
    gate.all
        .extend(facts("menh", &expand(&when.menh_all, groups)?, debug));
    Ok(gate)
}

/// The anchors of a passage: `QuanLoc/TuVi`, `QuanLoc/HoaKy`, `QuanLoc/~NoMain`, `QuanLoc/~Any`.
pub fn anchors(
    anchor: &Anchor,
    groups: &BTreeMap<String, Vec<TuViStar>>,
) -> Result<Vec<String>, String> {
    let role = anchor.role;
    let stars = expand(&anchor.stars, groups)?;
    let mut keys: Vec<String> = stars
        .iter()
        .map(|star| format!("{role:?}/{star:?}"))
        .collect();
    keys.extend(
        anchor
            .transformations
            .iter()
            .map(|hoa| format!("{role:?}/{hoa:?}")),
    );
    if anchor.no_main_star {
        keys.push(format!("{role:?}/~NoMain"));
    }
    if keys.is_empty() {
        keys.push(format!("{role:?}/~Any"));
    }
    Ok(keys)
}

fn entry_rows(item: &ValidEntry, source: &Source, draft: bool) -> Vec<EntryRow> {
    let extra = item.entry.extra();
    item.keys
        .iter()
        .map(|key| EntryRow {
            key: format!("{}/{}", item.collection, key.path),
            collection: item.collection.clone(),
            title: key.title.clone(),
            luck: key.luck.map(str::to_string),
            summary: item.entry.summary.clone(),
            body: item.entry.body.clone(),
            extra: (!extra.is_empty()).then(|| Value::Object(extra.clone())),
            citation: citation(&item.entry.source_ref, source),
            sensitivity: sensitivity_mask(&item.entry.sensitivity),
            draft,
        })
        .collect()
}

fn passage_row(id: u32, item: &ValidPassage, source: &Source, draft: bool) -> PassageRow {
    let passage = &item.passage;
    PassageRow {
        id,
        stable_id: passage.id.clone(),
        tier: if passage.condition_text.is_some() {
            "More"
        } else {
            "Primary"
        }
        .to_string(),
        section: passage.section.map(|section| format!("{section:?}")),
        verse: passage.verse.clone(),
        gates: item.gates.clone(),
        condition_text: passage.condition_text.clone(),
        body: passage.body.clone(),
        citation: citation(&passage.source_ref, source),
        sensitivity: sensitivity_mask(&passage.sensitivity),
        draft,
    }
}

/// The rows of `validated`. A release keeps approved items only.
pub fn compile(
    validated: &Validated,
    sources: &BTreeMap<String, Source>,
    release: bool,
) -> Compiled {
    use crate::model::Status::{Approved, Draft};
    let mut compiled = Compiled::default();
    for item in &validated.entries {
        if release && item.entry.status != Approved {
            continue;
        }
        let source = &sources[&item.entry.source_ref.source];
        compiled
            .entries
            .extend(entry_rows(item, source, item.entry.status == Draft));
    }
    compiled.entries.sort_by(|a, b| a.key.cmp(&b.key));
    for item in &validated.passages {
        if release && item.passage.status != Approved {
            continue;
        }
        let id = compiled.passages.len() as u32 + 1;
        let source = &sources[&item.passage.source_ref.source];
        compiled
            .passages
            .push(passage_row(id, item, source, item.passage.status == Draft));
        for anchor in &item.anchors {
            compiled.anchors.entry(anchor.clone()).or_default().push(id);
        }
    }
    compiled
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SourceType, StarRef};
    use crate::source::Rights;
    use astro_core::Branch::{Dan, Ngo};
    use astro_core::Gender;
    use astro_core::tuvi::FourTransformations::HoaQuyen;
    use astro_core::tuvi::StarBrightness::{Dac, Mieu};
    use astro_core::tuvi::TuViPalaceRole::QuanLoc;
    use astro_core::tuvi::TuViStar::{KinhDuong, ThatSat, TuVi};

    fn no_groups() -> BTreeMap<String, Vec<TuViStar>> {
        BTreeMap::new()
    }

    #[test]
    fn a_gate_gathers_every_kind_of_condition() {
        let when = When {
            branches: vec![Dan, Ngo],
            brightness: vec![Mieu, Dac],
            solo: Some(true),
            with_all: vec![StarRef::Star(ThatSat)],
            meets_any: vec![StarRef::Star(KinhDuong)],
            not_meets: vec![StarRef::Star(ThatSat)],
            with_transformations: vec![HoaQuyen],
            tuan_triet: Some(TuanTriet::Either),
            gender: Some(Gender::Female),
            ..When::default()
        };
        let gate = gate(&when, &["TuVi".to_string()], &no_groups()).unwrap();
        assert_eq!(gate.all, ["solo", "with:ThatSat", "gender:Female"]);
        assert_eq!(
            gate.any,
            [
                vec!["branch:Dan", "branch:Ngo"],
                vec!["bright:TuVi:Mieu", "bright:TuVi:Dac"],
                vec!["meet:KinhDuong"],
                vec!["hoa_with:HoaQuyen"],
                vec!["tuan", "triet"],
            ]
        );
        assert_eq!(gate.none, ["meet:ThatSat"]);
    }

    #[test]
    fn exclusions_and_full_sets_of_transformations_compile() {
        let when = When {
            not_with: vec![StarRef::Star(ThatSat)],
            transformations_all: vec![HoaQuyen],
            ..When::default()
        };
        let gate = gate(&when, &[], &no_groups()).unwrap();
        assert_eq!(gate.none, ["with:ThatSat"]);
        assert_eq!(gate.all, ["hoa:HoaQuyen"]);
    }

    #[test]
    fn neither_tuan_nor_triet_excludes_both() {
        let when = When {
            tuan_triet: Some(TuanTriet::Neither),
            ..When::default()
        };
        let gate = gate(&when, &[], &no_groups()).unwrap();
        assert_eq!(gate.none, ["tuan", "triet"]);
    }

    #[test]
    fn groups_expand_inside_gates_and_anchors() {
        let groups = BTreeMap::from([("Bo".to_string(), vec![TuVi, ThatSat])]);
        let reference = vec![StarRef::Group("@Bo".to_string())];
        let when = When {
            meets_all: reference.clone(),
            ..When::default()
        };
        assert_eq!(
            gate(&when, &[], &groups).unwrap().all,
            ["meet:TuVi", "meet:ThatSat"]
        );
        let anchor = Anchor {
            role: QuanLoc,
            stars: reference,
            transformations: vec![],
            no_main_star: false,
        };
        assert_eq!(
            anchors(&anchor, &groups).unwrap(),
            ["QuanLoc/TuVi", "QuanLoc/ThatSat"]
        );
    }

    #[test]
    fn the_four_kinds_of_anchor() {
        let base = Anchor {
            role: QuanLoc,
            stars: vec![],
            transformations: vec![],
            no_main_star: false,
        };
        assert_eq!(anchors(&base, &no_groups()).unwrap(), ["QuanLoc/~Any"]);
        let empty = Anchor {
            no_main_star: true,
            ..base.clone()
        };
        assert_eq!(anchors(&empty, &no_groups()).unwrap(), ["QuanLoc/~NoMain"]);
        let hoa = Anchor {
            transformations: vec![HoaQuyen],
            ..base
        };
        assert_eq!(anchors(&hoa, &no_groups()).unwrap(), ["QuanLoc/HoaQuyen"]);
    }

    #[test]
    fn brightness_is_about_the_anchor_stars_or_transformations() {
        let groups = BTreeMap::from([("Bo".to_string(), vec![KinhDuong, TuVi])]);
        let base = Anchor {
            role: QuanLoc,
            stars: vec![StarRef::Group("@Bo".to_string())],
            transformations: vec![],
            no_main_star: false,
        };
        let subjects = bright_subjects(&base, &groups).unwrap();
        assert_eq!(subjects, ["KinhDuong", "TuVi"]);
        let hoa = Anchor {
            stars: vec![],
            transformations: vec![HoaQuyen],
            ..base
        };
        assert_eq!(bright_subjects(&hoa, &groups).unwrap(), ["HoaQuyen"]);
        let when = When {
            brightness: vec![Dac],
            ..When::default()
        };
        let compiled = gate(&when, &subjects, &groups).unwrap();
        assert_eq!(
            compiled.any,
            [vec!["bright:KinhDuong:Dac", "bright:TuVi:Dac"]]
        );
    }

    #[test]
    fn citations_never_give_a_page_for_a_summary() {
        let source = Source {
            id: "s".to_string(),
            title: "Tam Mệnh Thông Hội".to_string(),
            author: None,
            edition: None,
            rights: Rights::Unverified,
            packet: None,
        };
        let reference = |source_type, pages: Vec<u16>| SourceRef {
            source: "s".to_string(),
            source_type,
            pages,
            locator: None,
            keywords: vec![],
        };
        assert_eq!(
            citation(&reference(SourceType::CanonicalSummary, vec![]), &source),
            "Tổng hợp theo truyền thống Tam Mệnh Thông Hội"
        );
        assert_eq!(
            citation(&reference(SourceType::Paraphrase, vec![158]), &source),
            "Tam Mệnh Thông Hội, tr. 158"
        );
        assert_eq!(
            citation(&reference(SourceType::Paraphrase, vec![158, 162]), &source),
            "Tam Mệnh Thông Hội, tr. 158, 162"
        );
    }
}
