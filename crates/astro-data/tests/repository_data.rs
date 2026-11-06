//! Checks on the real `data/interpretations/`: it stays clean, and the checks still catch the
//! mistakes that matter when they are planted in it.

use astro_data::collections::PASSAGE_COLLECTION;
use astro_data::compile::compile;
use astro_data::dataset::{DATA_DIR, Dataset, load};
use astro_data::output;
use astro_data::validate::{Issue, Validated, missing_keys, validate};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT: AtomicU32 = AtomicU32::new(0);

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for file in fs::read_dir(from).unwrap() {
        let file = file.unwrap();
        let target = to.join(file.file_name());
        if file.file_type().unwrap().is_dir() {
            copy_dir(&file.path(), &target);
        } else {
            fs::copy(file.path(), target).unwrap();
        }
    }
}

/// A copy of the repository data, with the source text when this machine has it.
struct Copy {
    root: PathBuf,
}

impl Copy {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("astro-data-repo-{}-{id}", std::process::id()));
        copy_dir(&repository().join(DATA_DIR), &root.join(DATA_DIR));
        let pages = ".ref/sources/tan_bien/pages.txt";
        if repository().join(pages).exists() {
            fs::create_dir_all(root.join(".ref/sources/tan_bien")).unwrap();
            fs::copy(repository().join(pages), root.join(pages)).unwrap();
        }
        Copy { root }
    }

    /// Replaces `from` by `to` in a data file; `from` must occur.
    fn plant(&self, file: &str, from: &str, to: &str) {
        let path = self.root.join(DATA_DIR).join(file);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(from), "{from} is no longer in {file}");
        fs::write(path, text.replacen(from, to, 1)).unwrap();
    }

    fn check(&self) -> (Dataset, Validated) {
        let dataset = load(&self.root).unwrap();
        let validated = validate(&dataset);
        (dataset, validated)
    }

    fn codes(&self) -> Vec<&'static str> {
        self.check()
            .1
            .issues
            .iter()
            .map(|issue| issue.code)
            .collect()
    }
}

impl Drop for Copy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

const PASSAGES: &str = "tuvi/tv-15.passage.json";

fn has_source_text() -> bool {
    repository()
        .join(".ref/sources/tan_bien/pages.txt")
        .exists()
}

#[test]
fn the_repository_data_is_clean() {
    let dataset = load(&repository()).unwrap();
    let validated = validate(&dataset);
    assert_eq!(validated.issues, Vec::<Issue>::new());
}

#[test]
fn quan_loc_has_a_passage_for_each_chinh_tinh() {
    let dataset = load(&repository()).unwrap();
    let validated = validate(&dataset);
    let missing = &missing_keys(&validated)[PASSAGE_COLLECTION];
    assert!(!missing.iter().any(|anchor| anchor.starts_with("QuanLoc/")));
}

#[test]
fn every_brightness_of_the_chinh_tinh_has_text_or_a_gap() {
    let dataset = load(&repository()).unwrap();
    let validated = validate(&dataset);
    assert_eq!(
        missing_keys(&validated)["tuvi.star_brightness"],
        Vec::<String>::new()
    );
}

#[test]
fn the_repository_data_builds_to_equal_formats() {
    let copy = Copy::new();
    let (dataset, validated) = copy.check();
    let compiled = compile(&validated, &dataset.sources, false);
    let dir = copy.root.join("out");
    let manifest = output::write(&compiled, &dir).unwrap();
    assert_eq!(manifest.content_hash, output::content_hash(&compiled));
    assert_eq!(
        output::read_sqlite(&dir.join(output::SQLITE_FILE)).unwrap(),
        compiled
    );
    assert_eq!(output::read_web(&dir, &manifest).unwrap(), compiled);
}

#[test]
fn a_release_keeps_exactly_the_approved_passages() {
    let dataset = load(&repository()).unwrap();
    let validated = validate(&dataset);
    let release = compile(&validated, &dataset.sources, true);
    let approved = validated
        .passages
        .iter()
        .filter(|item| item.passage.status == astro_data::model::Status::Approved);
    assert_eq!(release.passages.len(), approved.count());
}

#[test]
fn the_misprint_of_page_160_is_caught_if_copied_as_printed() {
    let copy = Copy::new();
    let file = copy.root.join(DATA_DIR).join(PASSAGES);
    let text = fs::read_to_string(&file).unwrap();
    let at = text.find(r#""id": "quan-loc/thien-co/03""#).unwrap();
    let rest = text[at..].replacen(r#""Than""#, r#""Thin""#, 1);
    fs::write(file, format!("{}{rest}", &text[..at])).unwrap();
    let codes = copy.codes();
    assert!(
        !codes.is_empty() && codes.iter().all(|&code| code == "V7"),
        "{codes:?}"
    );
}

#[test]
fn an_impossible_pair_is_caught() {
    let copy = Copy::new();
    copy.plant(PASSAGES, r#""PhaToai""#, r#""PhaQuan""#);
    assert!(copy.codes().contains(&"V7"));
}

#[test]
fn a_brightness_the_star_never_has_is_caught() {
    let copy = Copy::new();
    copy.plant(
        "tuvi/tv-01.star_brightness.json",
        r#""key": [
        "TuVi",
        "Binh"
      ]"#,
        r#""key": [
        "TuVi",
        "Ham"
      ]"#,
    );
    assert_eq!(copy.codes(), ["V2"]);
}

#[test]
fn a_misspelt_star_is_caught() {
    let copy = Copy::new();
    copy.plant(PASSAGES, r#""ThienMa""#, r#""ThienMaa""#);
    assert_eq!(copy.codes(), ["V6"]);
}

#[test]
fn an_invented_keyword_is_caught() {
    if !has_source_text() {
        return;
    }
    let copy = Copy::new();
    copy.plant(
        PASSAGES,
        "Sát đồng cung: có uy quyền",
        "Sát đồng cung: đại phú đại quý",
    );
    assert_eq!(copy.codes(), ["V5"]);
}

#[test]
fn a_keyword_on_the_wrong_page_is_caught() {
    if !has_source_text() {
        return;
    }
    let copy = Copy::new();
    copy.plant("tuvi/tv-15.palace.json", "158", "159");
    assert_eq!(copy.codes(), ["V5"]);
}

#[test]
fn copying_the_book_into_a_paraphrase_is_caught() {
    if !has_source_text() {
        return;
    }
    let copy = Copy::new();
    copy.plant(
        PASSAGES,
        "Thành đạt trong binh nghiệp, có uy quyền cả về chính trị, được nhiều người nể trọng.",
        "Võ nghiệp hiến đạt, có uy quyền kiêm nhiếp cả việc chính trị, được nhiều người kính nể.",
    );
    assert_eq!(copy.codes(), ["V12"]);
}

#[test]
fn a_duplicated_id_is_caught() {
    let copy = Copy::new();
    copy.plant(PASSAGES, "quan-loc/tu-vi/02", "quan-loc/tu-vi/01");
    assert_eq!(copy.codes(), ["V3"]);
}

#[test]
fn a_summary_written_for_tu_vi_from_memory_is_refused() {
    let copy = Copy::new();
    copy.plant(
        "tuvi/tv-15.palace.json",
        "\"paraphrase\"",
        "\"canonical_summary\"",
    );
    assert!(copy.codes().contains(&"V4"));
}
