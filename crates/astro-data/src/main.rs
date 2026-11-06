//! Command line of the interpretation database tool.
//!
//! ```text
//! cargo run -p astro-data -- validate [--complete]
//! cargo run -p astro-data -- build [--release]
//! cargo run -p astro-data -- stats
//! cargo run -p astro-data -- review --batch TV-15
//! ```

use astro_data::compile::compile;
use astro_data::dataset::{Dataset, load};
use astro_data::model::Status;
use astro_data::output::{self, SQLITE_FILE};
use astro_data::source::Rights;
use astro_data::validate::{Validated, missing_keys, validate};
use astro_data::{review, stats};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str =
    "usage: astro-data validate [--complete] | build [--release] | stats | review --batch <id>";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn print_issues(dataset: &Dataset, validated: &Validated) {
    for issue in &validated.issues {
        println!("{} {}: {}", issue.code, issue.place, issue.message);
    }
    for source in &dataset.missing_packets {
        println!("V5 bỏ qua: không có văn bản nguồn của `{source}` trên máy này");
    }
    println!(
        "{} mục, {} đoạn, {} gap, {} lỗi",
        validated.entries.len(),
        validated.passages.len(),
        validated.gaps.len(),
        validated.issues.len()
    );
}

fn run_validate(complete: bool) -> Result<bool, String> {
    let dataset = load(&root())?;
    let validated = validate(&dataset);
    print_issues(&dataset, &validated);
    let mut clean = validated.issues.is_empty();
    if complete {
        for (collection, missing) in missing_keys(&validated) {
            if !missing.is_empty() {
                println!("V3 {collection}: thiếu {} khóa", missing.len());
                clean = false;
            }
        }
    }
    Ok(clean)
}

/// V11: approved items may not rest on a source whose rights are unverified.
fn release_blockers(dataset: &Dataset, validated: &Validated) -> Vec<String> {
    let unverified = |source: &str| dataset.sources[source].rights == Rights::Unverified;
    let mut blocked: Vec<String> = validated
        .entries
        .iter()
        .filter(|item| item.entry.status == Status::Approved)
        .filter(|item| unverified(&item.entry.source_ref.source))
        .map(|item| item.entry.source_ref.source.clone())
        .chain(
            validated
                .passages
                .iter()
                .filter(|item| item.passage.status == Status::Approved)
                .filter(|item| unverified(&item.passage.source_ref.source))
                .map(|item| item.passage.source_ref.source.clone()),
        )
        .collect();
    blocked.sort();
    blocked.dedup();
    blocked
}

fn run_build(release: bool) -> Result<bool, String> {
    let root = root();
    let dataset = load(&root)?;
    let validated = validate(&dataset);
    print_issues(&dataset, &validated);
    if !validated.issues.is_empty() {
        return Ok(false);
    }
    if release {
        let blocked = release_blockers(&dataset, &validated);
        if !blocked.is_empty() {
            println!("V11 nguồn chưa kiểm bản quyền: {}", blocked.join(", "));
            return Ok(false);
        }
    }
    let compiled = compile(&validated, &dataset.sources, release);
    let dir = root.join("build/content");
    let manifest = output::write(&compiled, &dir)?;
    let size = |path: PathBuf| fs::metadata(path).map_or(0, |metadata| metadata.len());
    println!("{}: {} byte", SQLITE_FILE, size(dir.join(SQLITE_FILE)));
    for module in manifest.modules.values() {
        println!(
            "web/{}: {} byte, {} mục, {} đoạn",
            module.file,
            size(dir.join("web").join(&module.file)),
            module.entries,
            module.passages
        );
    }
    println!("{}", manifest.content_hash);
    Ok(true)
}

fn run_stats() -> Result<bool, String> {
    let dataset = load(&root())?;
    let validated = validate(&dataset);
    print!("{}", stats::report(&dataset, &validated));
    Ok(validated.issues.is_empty())
}

fn run_review(batch: &str) -> Result<bool, String> {
    let root = root();
    let dataset = load(&root)?;
    let validated = validate(&dataset);
    let dir = root.join("build/content/review");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!("{batch}.md"));
    fs::write(&path, review::sheet(&dataset, &validated, batch))
        .map_err(|error| error.to_string())?;
    println!("{}", path.display());
    Ok(true)
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let result = match arguments.as_slice() {
        ["validate"] => run_validate(false),
        ["validate", "--complete"] => run_validate(true),
        ["build"] => run_build(false),
        ["build", "--release"] => run_build(true),
        ["stats"] => run_stats(),
        ["review", "--batch", batch] => run_review(batch),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
