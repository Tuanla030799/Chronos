//! Coverage and size report (`stats`).

use crate::collections::{COLLECTIONS, PASSAGE_COLLECTION};
use crate::dataset::Dataset;
use crate::model::Status;
use crate::source::words;
use crate::validate::{Validated, missing_keys};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Default)]
struct Batch {
    files: usize,
    bytes: u64,
    items: usize,
    gaps: usize,
    chars: usize,
    words: usize,
    corrections: usize,
    conditions: usize,
    sensitive: usize,
    pages: BTreeSet<(String, u16)>,
}

impl Batch {
    fn add_text<'a>(&mut self, texts: impl Iterator<Item = &'a String>) {
        for text in texts {
            self.chars += text.chars().count();
            self.words += words(text).len();
        }
    }
}

fn batches(dataset: &Dataset, validated: &Validated) -> BTreeMap<String, Batch> {
    let mut batches: BTreeMap<String, Batch> = BTreeMap::new();
    for file in &dataset.files {
        let batch = batches.entry(file.data.batch.clone()).or_default();
        batch.files += 1;
        batch.bytes += file.bytes;
        batch.gaps += file.data.gaps.len();
    }
    for item in &validated.entries {
        let batch = batches.entry(item.batch.clone()).or_default();
        batch.items += 1;
        batch.add_text(std::iter::once(&item.entry.summary).chain(&item.entry.body));
        batch.corrections += usize::from(item.entry.correction.is_some());
        batch.sensitive += usize::from(!item.entry.sensitivity.is_empty());
        let reference = &item.entry.source_ref;
        batch.pages.extend(
            reference
                .pages
                .iter()
                .map(|&page| (reference.source.clone(), page)),
        );
    }
    for item in &validated.passages {
        let batch = batches.entry(item.batch.clone()).or_default();
        batch.items += 1;
        batch.add_text(item.passage.body.iter());
        batch.corrections += usize::from(item.passage.correction.is_some());
        batch.conditions += usize::from(item.passage.condition_text.is_some());
        batch.sensitive += usize::from(!item.passage.sensitivity.is_empty());
        let reference = &item.passage.source_ref;
        batch.pages.extend(
            reference
                .pages
                .iter()
                .map(|&page| (reference.source.clone(), page)),
        );
    }
    batches
}

/// The report as text.
pub fn report(dataset: &Dataset, validated: &Validated) -> String {
    let mut out = String::new();
    let missing = missing_keys(validated);
    let mut counts: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for item in &validated.entries {
        let approved = item.entry.status == Status::Approved;
        let row = counts.entry(item.collection.as_str()).or_default();
        if approved {
            row.0 += item.keys.len();
        } else {
            row.1 += item.keys.len();
        }
    }
    let mut gaps: BTreeMap<&str, usize> = BTreeMap::new();
    for gap in &validated.gaps {
        *gaps.entry(gap.collection.as_str()).or_default() += 1;
    }
    let _ = writeln!(
        out,
        "{:26}{:>6}{:>10}{:>7}{:>5}{:>7}",
        "collection", "khóa", "approved", "draft", "gap", "thiếu"
    );
    for collection in COLLECTIONS {
        let (approved, draft) = counts.get(collection.id).copied().unwrap_or_default();
        let _ = writeln!(
            out,
            "{:26}{:>6}{:>10}{:>7}{:>5}{:>7}",
            collection.id,
            (collection.all)().len(),
            approved,
            draft,
            gaps.get(collection.id).copied().unwrap_or_default(),
            missing[collection.id].len(),
        );
    }
    let approved = validated
        .passages
        .iter()
        .filter(|item| item.passage.status == Status::Approved)
        .count();
    let _ = writeln!(
        out,
        "\n{PASSAGE_COLLECTION}: {} đoạn, {approved} approved",
        validated.passages.len()
    );
    let _ = writeln!(
        out,
        "\n{:8}{:>6}{:>9}{:>6}{:>5}{:>9}{:>7}{:>9}{:>8}{:>7}{:>7}{:>6}",
        "batch",
        "file",
        "byte",
        "mục",
        "gap",
        "ký tự",
        "từ",
        "nguồn",
        "tỷ lệ",
        "sửa",
        "nếu",
        "nhạy"
    );
    for (name, batch) in batches(dataset, validated) {
        let source: usize = batch
            .pages
            .iter()
            .filter_map(|(source, page)| Some(dataset.packets.get(source)?.page_chars(*page)))
            .sum();
        let ratio = if source == 0 {
            0.0
        } else {
            batch.chars as f64 / source as f64
        };
        let _ = writeln!(
            out,
            "{name:8}{:>6}{:>9}{:>6}{:>5}{:>9}{:>7}{source:>9}{ratio:>8.2}{:>7}{:>7}{:>6}",
            batch.files,
            batch.bytes,
            batch.items,
            batch.gaps,
            batch.chars,
            batch.words,
            batch.corrections,
            batch.conditions,
            batch.sensitive,
        );
    }
    out
}
