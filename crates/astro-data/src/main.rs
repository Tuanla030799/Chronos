//! Command line of the interpretation database tool.
//!
//! ```text
//! cargo run -p astro-data -- validate [--complete]
//! cargo run -p astro-data -- build [--release]
//! cargo run -p astro-data -- stats
//! cargo run -p astro-data -- review --batch TV-15
//! cargo run --release -p astro-data -- scan
//! ```

use astro_data::compile::compile;
use astro_data::dataset::{Dataset, load};
use astro_data::model::{SourceRef, SourceType, Status};
use astro_data::output::{self, SQLITE_FILE};
use astro_data::scan::{self, ScanRange};
use astro_data::source::Rights;
use astro_data::validate::{Validated, missing_keys, validate};
use astro_data::{review, stats};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: astro-data validate [--complete] | build [--release] | stats | scan | review --batch <id>";

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

/// V13. It runs on clean data only: a passage V7 refuses need not be looked for.
fn scan_is_clean(dataset: &Dataset, validated: &Validated) -> bool {
    let compiled = compile(validated, &dataset.sources, false);
    let unseen = scan::unseen(&compiled, &ScanRange::full());
    for id in &unseen {
        println!("V13 {id}: không lá số nào hiện đoạn này");
    }
    unseen.is_empty()
}

fn run_validate(complete: bool) -> Result<bool, String> {
    let dataset = load(&root())?;
    let validated = validate(&dataset);
    print_issues(&dataset, &validated);
    let mut clean = validated.issues.is_empty() && scan_is_clean(&dataset, &validated);
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

/// V11: an approved item may not rest on a source whose rights are unverified, nor quote a
/// source released in paraphrase only.
fn release_blockers(dataset: &Dataset, validated: &Validated) -> Vec<String> {
    let blocked = |reference: &SourceRef| match dataset.sources[&reference.source].rights {
        Rights::Unverified => Some(format!("{}: chưa kiểm bản quyền", reference.source)),
        Rights::Paraphrased if reference.source_type != SourceType::Paraphrase => {
            Some(format!("{}: chỉ được diễn đạt lại", reference.source))
        }
        _ => None,
    };
    let mut reasons: Vec<String> = validated
        .entries
        .iter()
        .filter(|item| item.entry.status == Status::Approved)
        .filter_map(|item| blocked(&item.entry.source_ref))
        .chain(
            validated
                .passages
                .iter()
                .filter(|item| item.passage.status == Status::Approved)
                .filter_map(|item| blocked(&item.passage.source_ref)),
        )
        .collect();
    reasons.sort();
    reasons.dedup();
    reasons
}

fn run_build(release: bool) -> Result<bool, String> {
    let root = root();
    let dataset = load(&root)?;
    let validated = validate(&dataset);
    print_issues(&dataset, &validated);
    if !validated.issues.is_empty() || !scan_is_clean(&dataset, &validated) {
        return Ok(false);
    }
    if release {
        let blocked = release_blockers(&dataset, &validated);
        if !blocked.is_empty() {
            println!("V11 {}", blocked.join("; "));
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

fn run_scan() -> Result<bool, String> {
    let dataset = load(&root())?;
    let validated = validate(&dataset);
    let compiled = compile(&validated, &dataset.sources, false);
    let range = ScanRange::full();
    let unseen = scan::unseen(&compiled, &range);
    println!("đoạn không bao giờ hiện: {}", unseen.len());
    for id in &unseen {
        println!("  {id}");
    }
    println!(
        "{:12}{:>9}{:>8}{:>9}{:>6}{:>6}",
        "cung", "lá số", "đoạn", "chắc chắn", "ít", "nhiều"
    );
    for (role, load) in scan::load(&compiled, &range) {
        let charts = load.charts as f64;
        println!(
            "{role:12}{:>9}{:>8.2}{:>9.2}{:>6}{:>6}",
            load.charts,
            load.shown as f64 / charts,
            load.primary as f64 / charts,
            load.min,
            load.max
        );
    }
    Ok(unseen.is_empty())
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
        ["scan"] => run_scan(),
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
