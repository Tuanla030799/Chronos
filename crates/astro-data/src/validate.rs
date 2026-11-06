//! Checks V1 to V8 and V12 of section 10 of the specification. Nothing stops at the first problem:
//! one run reports all of them.

use crate::collections::{COLLECTIONS, Collection, KeyInfo, PASSAGE_COLLECTION, find};
use crate::compile::{Gate, anchors, gate};
use crate::dataset::{Dataset, LoadedFile};
use crate::model::{ActivityNote, Entry, Gap, Passage, SourceRef, SourceType, When};
use crate::reach::{Resolved, expand, impossible, resolve};
use crate::source::words;
use astro_core::tuvi::{StarCategory, TuViStar};
use std::collections::{BTreeMap, BTreeSet};

/// Longest `summary`, in characters (V8).
pub const SUMMARY_LIMIT: usize = 160;
/// Longest phrase of `keywords`, in words.
pub const KEYWORD_WORDS: usize = 15;
/// Shortest run of source words that counts as copying in a paraphrase (V12).
pub const COPIED_WORDS: usize = 12;
/// Longest `quote`, in words: D3 keeps verbatim text to short verses.
pub const QUOTE_WORDS: usize = 40;

/// A problem found in the data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Check, e.g. `V5`.
    pub code: &'static str,
    /// File and item.
    pub place: String,
    /// What is wrong.
    pub message: String,
}

/// An entry that parsed, with its keys.
#[derive(Debug, Clone)]
pub struct ValidEntry {
    /// Collection id.
    pub collection: String,
    /// Batch id.
    pub batch: String,
    /// The keys.
    pub keys: Vec<KeyInfo>,
    /// The entry.
    pub entry: Entry,
}

/// A passage that parsed, with what it compiles to.
#[derive(Debug, Clone)]
pub struct ValidPassage {
    /// Batch id.
    pub batch: String,
    /// The passage.
    pub passage: Passage,
    /// Its anchors.
    pub anchors: Vec<String>,
    /// Its gates.
    pub gates: Vec<Gate>,
}

/// A gap that parsed, with its key.
#[derive(Debug, Clone)]
pub struct ValidGap {
    /// Collection id.
    pub collection: String,
    /// Batch id.
    pub batch: String,
    /// Path of the key.
    pub path: String,
    /// The gap.
    pub gap: Gap,
}

/// What a run of the checks found.
#[derive(Debug, Clone, Default)]
pub struct Validated {
    /// Entries.
    pub entries: Vec<ValidEntry>,
    /// Passages.
    pub passages: Vec<ValidPassage>,
    /// Gaps.
    pub gaps: Vec<ValidGap>,
    /// Problems.
    pub issues: Vec<Issue>,
}

impl Validated {
    fn report(&mut self, code: &'static str, place: &str, message: impl Into<String>) {
        self.issues.push(Issue {
            code,
            place: place.to_string(),
            message: message.into(),
        });
    }
}

fn check_file(file: &LoadedFile, out: &mut Validated) -> Option<&'static Collection> {
    let data = &file.data;
    let Some(collection) = find(&data.collection) else {
        out.report(
            "V1",
            &file.path,
            format!("collection lạ: {}", data.collection),
        );
        return None;
    };
    let short = data.collection.split('.').nth(1).unwrap_or_default();
    let expected = format!(
        "{}/{}.{short}.json",
        collection.module(),
        data.batch.to_lowercase()
    );
    if file.path != expected {
        out.report("V1", &file.path, format!("phải đặt tên {expected}"));
    }
    Some(collection)
}

fn check_source(
    dataset: &Dataset,
    collection: &str,
    place: &str,
    reference: &SourceRef,
    texts: &[&str],
    out: &mut Validated,
) {
    if !dataset.sources.contains_key(&reference.source) {
        out.report("V4", place, format!("nguồn lạ: {}", reference.source));
        return;
    }
    let summary = reference.source_type == SourceType::CanonicalSummary;
    if summary {
        if !reference.keywords.is_empty() || !reference.pages.is_empty() {
            out.report("V4", place, "canonical_summary không có trang hay từ khóa");
        }
        if collection.starts_with("tuvi.") {
            out.report("V4", place, "D4 không cho phép canonical_summary ở Tử Vi");
        }
        return;
    }
    if reference.keywords.is_empty() {
        out.report("V4", place, "thiếu keywords");
    }
    if reference.pages.is_empty() && reference.locator.is_none() {
        out.report("V4", place, "thiếu pages hoặc locator");
    }
    for keyword in &reference.keywords {
        if words(keyword).len() > KEYWORD_WORDS {
            out.report(
                "V4",
                place,
                format!("từ khóa quá {KEYWORD_WORDS} từ: {keyword}"),
            );
        }
    }
    if reference.source_type == SourceType::Quote {
        for text in texts {
            if words(text).len() > QUOTE_WORDS {
                out.report("V12", place, format!("quote quá {QUOTE_WORDS} từ"));
            }
        }
    }
    let Some(packet) = dataset.packets.get(&reference.source) else {
        return;
    };
    for &page in &reference.pages {
        if !packet.has_page(page) {
            out.report("V5", place, format!("nguồn không có trang {page}"));
        }
    }
    for keyword in &reference.keywords {
        if !packet.contains(&reference.pages, keyword) {
            out.report("V5", place, format!("không thấy từ khóa: {keyword}"));
        }
    }
    if reference.source_type == SourceType::Paraphrase {
        for text in texts {
            if let Some(run) = packet.copied_run(&reference.pages, text, COPIED_WORDS) {
                out.report("V12", place, format!("chép nguyên văn: {run}"));
            }
        }
    }
}

fn check_text(place: &str, parts: &[&str], out: &mut Validated) {
    if parts.is_empty() || parts.iter().any(|text| text.trim().is_empty()) {
        out.report("V8", place, "văn bản rỗng");
    }
}

fn check_activities(
    dataset: &Dataset,
    place: &str,
    notes: Option<&Vec<ActivityNote>>,
    out: &mut Validated,
) {
    for note in notes.into_iter().flatten() {
        if !dataset.activities.contains(&note.activity) {
            out.report("V6", place, format!("việc lạ: {}", note.activity));
        }
    }
}

fn entry_keys(entry: &Entry) -> Result<Vec<&serde_json::Value>, &'static str> {
    match (&entry.key, &entry.keys) {
        (Some(key), None) => Ok(vec![key]),
        (None, Some(keys)) if !keys.is_empty() => Ok(keys.iter().collect()),
        _ => Err("cần đúng một trong key, keys"),
    }
}

fn check_entry(
    dataset: &Dataset,
    file: &LoadedFile,
    collection: &Collection,
    index: usize,
    value: &serde_json::Value,
    seen: &mut BTreeMap<(String, String), String>,
    out: &mut Validated,
) {
    let mut place = format!("{}#{}", file.path, index + 1);
    let entry: Entry = match serde_json::from_value(value.clone()) {
        Ok(entry) => entry,
        Err(error) => {
            out.report("V1", &place, error.to_string());
            return;
        }
    };
    let values = match entry_keys(&entry) {
        Ok(values) => values,
        Err(message) => {
            out.report("V1", &place, message);
            return;
        }
    };
    let mut keys = Vec::new();
    for value in values {
        match (collection.parse)(value) {
            Ok(key) => keys.push(key),
            Err(message) => out.report("V2", &place, message),
        }
    }
    if let Some(key) = keys.first() {
        place = format!("{} {}", file.path, key.path);
    }
    for key in &keys {
        let id = (collection.id.to_string(), key.path.clone());
        if let Some(earlier) = seen.insert(id, place.clone()) {
            out.report(
                "V3",
                &place,
                format!("trùng khóa {} với {earlier}", key.path),
            );
        }
    }
    for name in entry.extra().keys() {
        if !collection.extras.contains(&name.as_str()) {
            out.report(
                "V1",
                &place,
                format!("{} không dùng trường {name}", collection.id),
            );
        }
    }
    if entry.summary.chars().count() > SUMMARY_LIMIT {
        out.report("V8", &place, format!("summary quá {SUMMARY_LIMIT} ký tự"));
    }
    let mut texts = vec![entry.summary.as_str()];
    texts.extend(entry.body.iter().map(String::as_str));
    check_text(&place, &texts, out);
    check_activities(dataset, &place, entry.good_for.as_ref(), out);
    check_activities(dataset, &place, entry.avoid.as_ref(), out);
    check_source(
        dataset,
        collection.id,
        &place,
        &entry.source_ref,
        &texts,
        out,
    );
    out.entries.push(ValidEntry {
        collection: collection.id.to_string(),
        batch: file.data.batch.clone(),
        keys,
        entry,
    });
}

fn is_main(star: TuViStar) -> bool {
    star.category() == StarCategory::ChinhTinh
}

fn check_conditions(
    dataset: &Dataset,
    place: &str,
    passage: &Passage,
    anchor_stars: &[TuViStar],
    out: &mut Validated,
) -> Option<Vec<Gate>> {
    let mut resolved: Vec<Resolved> = Vec::new();
    let mut gates = Vec::new();
    let mut misplaced_brightness = false;
    for when in &passage.when {
        let when: &When = when;
        if !when.brightness.is_empty()
            && (anchor_stars.is_empty() || !anchor_stars.iter().copied().all(is_main))
        {
            out.report(
                "V7",
                place,
                "brightness chỉ dùng cho Chính Tinh; phụ tinh để condition_text (D8)",
            );
            misplaced_brightness = true;
        }
        match (
            resolve(when, &dataset.groups),
            gate(when, anchor_stars, &dataset.groups),
        ) {
            (Ok(when), Ok(compiled)) => {
                resolved.push(when);
                gates.push(compiled);
            }
            (Err(message), _) | (_, Err(message)) => {
                out.report("V6", place, message);
                return None;
            }
        }
    }
    if !misplaced_brightness {
        let main_anchors: Vec<TuViStar> = anchor_stars
            .iter()
            .copied()
            .filter(|&star| is_main(star))
            .collect();
        for problem in impossible(&main_anchors, passage.anchor.no_main_star, &resolved) {
            out.report("V7", place, problem);
        }
    }
    Some(gates)
}

fn check_passage(
    dataset: &Dataset,
    file: &LoadedFile,
    index: usize,
    value: &serde_json::Value,
    seen: &mut BTreeMap<(String, String), String>,
    out: &mut Validated,
) {
    let mut place = format!("{}#{}", file.path, index + 1);
    let passage: Passage = match serde_json::from_value(value.clone()) {
        Ok(passage) => passage,
        Err(error) => {
            out.report("V1", &place, error.to_string());
            return;
        }
    };
    place = format!("{} {}", file.path, passage.id);
    let id = (PASSAGE_COLLECTION.to_string(), passage.id.clone());
    if let Some(earlier) = seen.insert(id, place.clone()) {
        out.report("V3", &place, format!("trùng id với {earlier}"));
    }
    let anchor = &passage.anchor;
    let kinds = usize::from(!anchor.stars.is_empty())
        + usize::from(!anchor.transformations.is_empty())
        + usize::from(anchor.no_main_star);
    if kinds > 1 {
        out.report(
            "V1",
            &place,
            "anchor chỉ dùng một trong stars, transformations, no_main_star",
        );
    }
    let anchor_stars = match expand(&anchor.stars, &dataset.groups) {
        Ok(stars) => stars,
        Err(message) => {
            out.report("V6", &place, message);
            return;
        }
    };
    let Some(gates) = check_conditions(dataset, &place, &passage, &anchor_stars, out) else {
        return;
    };
    let mut texts: Vec<&str> = passage.body.iter().map(String::as_str).collect();
    check_text(&place, &texts, out);
    if let Some(condition) = &passage.condition_text {
        check_text(&place, &[condition], out);
    }
    if let Some(verse) = &passage.verse {
        texts.push(verse);
    }
    check_source(
        dataset,
        PASSAGE_COLLECTION,
        &place,
        &passage.source_ref,
        &texts,
        out,
    );
    let Ok(anchors) = anchors(anchor, &dataset.groups) else {
        return;
    };
    out.passages.push(ValidPassage {
        batch: file.data.batch.clone(),
        passage,
        anchors,
        gates,
    });
}

fn check_gap(
    file: &LoadedFile,
    collection: &Collection,
    gap: &Gap,
    seen: &mut BTreeMap<(String, String), String>,
    out: &mut Validated,
) {
    let place = format!("{} gap", file.path);
    match (collection.parse)(&gap.key) {
        Ok(key) => {
            let id = (format!("gap:{}", collection.id), key.path.clone());
            if let Some(earlier) = seen.insert(id, place.clone()) {
                out.report(
                    "V3",
                    &place,
                    format!("trùng gap {} với {earlier}", key.path),
                );
            }
            if seen.contains_key(&(collection.id.to_string(), key.path.clone())) {
                out.report("V3", &place, format!("{} vừa có mục vừa có gap", key.path));
            }
            out.gaps.push(ValidGap {
                collection: collection.id.to_string(),
                batch: file.data.batch.clone(),
                path: key.path,
                gap: gap.clone(),
            });
        }
        Err(message) => out.report("V2", &place, message),
    }
}

/// Runs the checks on `dataset`.
pub fn validate(dataset: &Dataset) -> Validated {
    let mut out = Validated::default();
    let mut seen = BTreeMap::new();
    for file in &dataset.files {
        let Some(collection) = check_file(file, &mut out) else {
            continue;
        };
        for (index, value) in file.data.entries.iter().enumerate() {
            if collection.id == PASSAGE_COLLECTION {
                check_passage(dataset, file, index, value, &mut seen, &mut out);
            } else {
                check_entry(dataset, file, collection, index, value, &mut seen, &mut out);
            }
        }
    }
    for file in &dataset.files {
        if let Some(collection) = find(&file.data.collection) {
            for gap in &file.data.gaps {
                check_gap(file, collection, gap, &mut seen, &mut out);
            }
        }
    }
    out
}

/// The keys each collection still lacks: neither an entry, nor a passage anchor, nor a gap.
pub fn missing_keys(validated: &Validated) -> BTreeMap<&'static str, Vec<String>> {
    let mut present: BTreeSet<(String, String)> = BTreeSet::new();
    for item in &validated.entries {
        for key in &item.keys {
            present.insert((item.collection.clone(), key.path.clone()));
        }
    }
    for item in &validated.passages {
        for anchor in &item.anchors {
            present.insert((PASSAGE_COLLECTION.to_string(), anchor.clone()));
        }
    }
    for gap in &validated.gaps {
        present.insert((gap.collection.clone(), gap.path.clone()));
    }
    COLLECTIONS
        .iter()
        .map(|collection| {
            let missing = (collection.all)()
                .into_iter()
                .filter(|path| !present.contains(&(collection.id.to_string(), path.clone())))
                .collect();
            (collection.id, missing)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::load;
    use crate::test_support::TestRoot;

    const PARAPHRASE: &str = r#""source_ref": { "source": "book", "source_type": "paraphrase", "pages": [158], "keywords": ["Sát đồng cung: có uy quyền"] }"#;

    fn check(path: &str, file: &str) -> Vec<Issue> {
        let root = TestRoot::new();
        root.write(&format!("data/interpretations/{path}"), file);
        validate(&load(root.path()).unwrap()).issues
    }

    fn entries(body: &str) -> Vec<Issue> {
        let file =
            format!(r#"{{ "collection": "almanac.truc", "batch": "AL-03", "entries": [{body}] }}"#);
        check("almanac/al-03.truc.json", &file)
    }

    fn passages(body: &str) -> Vec<Issue> {
        let file =
            format!(r#"{{ "collection": "tuvi.passage", "batch": "TV-15", "entries": [{body}] }}"#);
        check("tuvi/tv-15.passage.json", &file)
    }

    fn passage(anchor: &str, when: &str) -> String {
        format!(
            r#"{{ "id": "x/1", "anchor": {anchor}, "when": {when}, "body": ["Có thực quyền."], {PARAPHRASE}, "status": "draft" }}"#
        )
    }

    fn codes(issues: &[Issue]) -> Vec<&'static str> {
        issues.iter().map(|issue| issue.code).collect()
    }

    fn entry(key: &str) -> String {
        format!(
            r#"{{ "key": {key}, "summary": "Có thực quyền.", {PARAPHRASE}, "status": "draft" }}"#
        )
    }

    #[test]
    fn a_clean_entry_and_a_clean_passage_pass() {
        assert_eq!(entries(&entry(r#""Kien""#)), []);
        let good = passage(
            r#"{ "role": "QuanLoc", "stars": ["TuVi"] }"#,
            r#"[{ "with_all": ["ThatSat"] }]"#,
        );
        assert_eq!(passages(&good), []);
    }

    #[test]
    fn v1_rejects_unknown_fields_and_fields_of_other_collections() {
        let unknown = r#"{ "key": "Kien", "summary": "x", "colour": "red", "status": "draft" }"#;
        assert_eq!(codes(&entries(unknown)), ["V1"]);
        let foreign = format!(
            r#"{{ "key": "Kien", "summary": "Có thực quyền.", "verse": "x", {PARAPHRASE}, "status": "draft" }}"#
        );
        assert_eq!(codes(&entries(&foreign)), ["V1"]);
        let both = format!(
            r#"{{ "key": "Kien", "keys": ["Tru"], "summary": "Có thực quyền.", {PARAPHRASE}, "status": "draft" }}"#
        );
        assert_eq!(codes(&entries(&both)), ["V1"]);
    }

    #[test]
    fn v1_rejects_a_misnamed_file_and_an_unknown_collection() {
        let file = r#"{ "collection": "almanac.truc", "batch": "AL-03" }"#;
        assert_eq!(codes(&check("almanac/truc.json", file)), ["V1"]);
        let file = r#"{ "collection": "almanac.nope", "batch": "AL-03" }"#;
        assert_eq!(codes(&check("almanac/al-03.nope.json", file)), ["V1"]);
    }

    #[test]
    fn v2_rejects_keys_outside_the_enum() {
        assert_eq!(codes(&entries(&entry(r#""Kiến""#))), ["V2"]);
    }

    #[test]
    fn v3_rejects_duplicates_and_an_entry_that_is_also_a_gap() {
        let twice = format!("{},{}", entry(r#""Kien""#), entry(r#""Kien""#));
        assert_eq!(codes(&entries(&twice)), ["V3"]);
        let file = format!(
            r#"{{ "collection": "almanac.truc", "batch": "AL-03", "entries": [{}],
                 "gaps": [{{ "key": "Kien", "gap": "x", "checked": "y" }}] }}"#,
            entry(r#""Kien""#)
        );
        assert_eq!(codes(&check("almanac/al-03.truc.json", &file)), ["V3"]);
    }

    #[test]
    fn v4_enforces_the_rules_of_each_source_type() {
        let reference = |text: &str| {
            format!(
                r#"{{ "key": "Kien", "summary": "Có thực quyền.", "source_ref": {text}, "status": "draft" }}"#
            )
        };
        let unknown = reference(r#"{ "source": "nope", "source_type": "paraphrase" }"#);
        assert_eq!(codes(&entries(&unknown)), ["V4"]);
        let bare = reference(r#"{ "source": "book", "source_type": "paraphrase" }"#);
        assert_eq!(codes(&entries(&bare)), ["V4", "V4"]);
        let summary = reference(r#"{ "source": "closed", "source_type": "canonical_summary" }"#);
        assert_eq!(entries(&summary), []);
        let summary_with_page = reference(
            r#"{ "source": "closed", "source_type": "canonical_summary", "pages": [1] }"#,
        );
        assert_eq!(codes(&entries(&summary_with_page)), ["V4"]);
    }

    #[test]
    fn v4_keeps_canonical_summaries_out_of_tu_vi() {
        let file = r#"{ "collection": "tuvi.star", "batch": "TV-01", "entries": [
            { "key": "TuVi", "summary": "x", "status": "draft",
              "source_ref": { "source": "closed", "source_type": "canonical_summary" } }] }"#;
        assert_eq!(codes(&check("tuvi/tv-01.star.json", file)), ["V4"]);
    }

    #[test]
    fn v5_finds_invented_keywords_and_wrong_pages() {
        let keyword = |pages: &str, keyword: &str| {
            format!(
                r#"{{ "key": "Kien", "summary": "Có thực quyền.", "status": "draft",
                     "source_ref": {{ "source": "book", "source_type": "paraphrase",
                                      "pages": [{pages}], "keywords": ["{keyword}"] }} }}"#
            )
        };
        assert_eq!(entries(&keyword("158", "có uy quyền")), []);
        assert_eq!(
            codes(&entries(&keyword("158", "câu không có trong sách"))),
            ["V5"]
        );
        assert_eq!(codes(&entries(&keyword("159", "có uy quyền"))), ["V5"]);
        assert_eq!(
            codes(&entries(&keyword("999", "có uy quyền"))),
            ["V5", "V5"]
        );
    }

    #[test]
    fn v5_is_skipped_only_for_a_text_absent_from_this_machine() {
        let file = r#"{ "key": "Kien", "summary": "Có thực quyền.", "status": "draft",
            "source_ref": { "source": "absent", "source_type": "paraphrase", "pages": [1],
                            "keywords": ["bất kỳ"] } }"#;
        assert_eq!(entries(file), []);
        let root = TestRoot::new();
        assert!(
            load(root.path())
                .unwrap()
                .missing_packets
                .contains("absent")
        );
    }

    #[test]
    fn v6_rejects_unknown_activities_and_groups() {
        let unknown = format!(
            r#"{{ "key": "Kien", "summary": "Có thực quyền.", "avoid": [{{ "activity": "Bay" }}], {PARAPHRASE}, "status": "draft" }}"#
        );
        assert_eq!(codes(&entries(&unknown)), ["V6"]);
        let bad = passage(r#"{ "role": "QuanLoc", "stars": ["@Khong"] }"#, "[]");
        assert_eq!(codes(&passages(&bad)), ["V6"]);
        let misspelt = passage(
            r#"{ "role": "QuanLoc" }"#,
            r#"[{ "meets_all": ["TuVii"] }]"#,
        );
        assert_eq!(codes(&passages(&misspelt)), ["V6"]);
        let group = passage(
            r#"{ "role": "QuanLoc" }"#,
            r#"[{ "meets_any": ["@KinhDa"] }]"#,
        );
        assert_eq!(passages(&group), []);
    }

    #[test]
    fn v7_rejects_conditions_that_never_hold() {
        let anchor = r#"{ "role": "QuanLoc", "stars": ["TuVi"] }"#;
        let merged = passage(
            anchor,
            r#"[{ "solo": true, "branches": ["Ngo"], "with_all": ["ThienPhu"] }]"#,
        );
        assert!(codes(&passages(&merged)).contains(&"V7"));
        let split = passage(
            anchor,
            r#"[{ "solo": true, "branches": ["Ngo"] }, { "with_all": ["ThienPhu"] }]"#,
        );
        assert_eq!(passages(&split), []);
        let ham = passage(anchor, r#"[{ "brightness": ["Ham"] }]"#);
        assert!(codes(&passages(&ham)).contains(&"V7"));
        let misprint = passage(
            r#"{ "role": "QuanLoc", "stars": ["ThienCo", "ThaiAm"] }"#,
            r#"[{ "with_all": ["ThienCo", "ThaiAm"], "branches": ["Thin"] }]"#,
        );
        assert!(codes(&passages(&misprint)).contains(&"V7"));
    }

    #[test]
    fn v7_sends_the_brightness_of_minor_stars_to_condition_text() {
        let minor = passage(
            r#"{ "role": "QuanLoc", "stars": ["VanXuong"] }"#,
            r#"[{ "brightness": ["Dac"] }]"#,
        );
        assert_eq!(codes(&passages(&minor)), ["V7"]);
    }

    #[test]
    fn v8_limits_the_summary_and_refuses_empty_text() {
        let long = "x".repeat(SUMMARY_LIMIT + 1);
        let too_long =
            format!(r#"{{ "key": "Kien", "summary": "{long}", {PARAPHRASE}, "status": "draft" }}"#);
        assert_eq!(codes(&entries(&too_long)), ["V8"]);
        let empty =
            format!(r#"{{ "key": "Kien", "summary": " ", {PARAPHRASE}, "status": "draft" }}"#);
        assert_eq!(codes(&entries(&empty)), ["V8"]);
    }

    #[test]
    fn v12_catches_a_paraphrase_that_copies_the_page() {
        let copied = r#"{ "key": "Kien", "summary": "một hai ba bốn năm sáu bảy tám chín mười mười một mười hai", "status": "draft",
            "source_ref": { "source": "book", "source_type": "paraphrase", "pages": [158], "keywords": ["một hai ba"] } }"#;
        assert_eq!(codes(&entries(copied)), ["V12"]);
        let quoted = copied.replace("\"paraphrase\"", "\"quote\"");
        assert_eq!(entries(&quoted), []);
    }

    #[test]
    fn v12_keeps_quotes_short() {
        let long = "chữ ".repeat(QUOTE_WORDS + 1);
        let quote = format!(
            r#"{{ "key": "Kien", "summary": "x", "body": ["{long}"], "status": "draft",
                 "source_ref": {{ "source": "book", "source_type": "quote", "pages": [158], "keywords": ["có uy quyền"] }} }}"#
        );
        assert_eq!(codes(&entries(&quote)), ["V12"]);
    }

    #[test]
    fn missing_keys_count_entries_anchors_and_gaps() {
        let root = TestRoot::new();
        root.write(
            "data/interpretations/almanac/al-03.truc.json",
            &format!(
                r#"{{ "collection": "almanac.truc", "batch": "AL-03", "entries": [{}],
                     "gaps": [{{ "key": "Tru", "gap": "x", "checked": "y" }}] }}"#,
                entry(r#""Kien""#)
            ),
        );
        root.write(
            "data/interpretations/tuvi/tv-15.passage.json",
            &format!(
                r#"{{ "collection": "tuvi.passage", "batch": "TV-15", "entries": [{}] }}"#,
                passage(r#"{ "role": "QuanLoc", "stars": ["TuVi"] }"#, "[]")
            ),
        );
        let validated = validate(&load(root.path()).unwrap());
        assert_eq!(validated.issues, []);
        let missing = missing_keys(&validated);
        assert_eq!(missing["almanac.truc"].len(), 10);
        assert_eq!(missing[PASSAGE_COLLECTION].len(), 167);
        assert!(!missing[PASSAGE_COLLECTION].contains(&"QuanLoc/TuVi".to_string()));
    }
}
