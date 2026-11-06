//! The review sheet of a batch (`review --batch`): each item next to the words of its source.

use crate::dataset::Dataset;
use crate::model::SourceRef;
use crate::validate::Validated;
use std::fmt::Write;

fn sources(out: &mut String, dataset: &Dataset, reference: &SourceRef) {
    let packet = dataset.packets.get(&reference.source);
    for keyword in &reference.keywords {
        let context = packet.and_then(|packet| packet.context(&reference.pages, keyword, 90));
        let _ = writeln!(out, "> {}", context.as_deref().unwrap_or(keyword));
    }
    let _ = writeln!(
        out,
        "\nNguồn: `{}` tr. {:?}\n",
        reference.source, reference.pages
    );
}

/// The sheet of `batch` as Markdown.
pub fn sheet(dataset: &Dataset, validated: &Validated, batch: &str) -> String {
    let mut out = format!("# Duyệt batch {batch}\n\n");
    for item in validated.entries.iter().filter(|item| item.batch == batch) {
        let paths: Vec<&str> = item.keys.iter().map(|key| key.path.as_str()).collect();
        let _ = writeln!(out, "## {} {}\n", item.collection, paths.join(", "));
        let _ = writeln!(out, "**{}**\n", item.entry.summary);
        for paragraph in &item.entry.body {
            let _ = writeln!(out, "- {paragraph}");
        }
        if let Some(correction) = &item.entry.correction {
            let _ = writeln!(out, "\n⚠️ Đính chính: {correction}");
        }
        out.push('\n');
        sources(&mut out, dataset, &item.entry.source_ref);
    }
    for item in validated.passages.iter().filter(|item| item.batch == batch) {
        let passage = &item.passage;
        let _ = writeln!(out, "## {}\n", passage.id);
        let _ = writeln!(out, "Neo: `{}`\n", item.anchors.join("`, `"));
        for gate in &item.gates {
            let text = serde_json::to_string(gate).expect("gates serialize");
            let _ = writeln!(out, "Cổng: `{text}`\n");
        }
        if let Some(condition) = &passage.condition_text {
            let _ = writeln!(out, "Nếu: {condition}\n");
        }
        for paragraph in &passage.body {
            let _ = writeln!(out, "**{paragraph}**\n");
        }
        if !passage.sensitivity.is_empty() {
            let _ = writeln!(out, "Nhạy cảm: {:?}\n", passage.sensitivity);
        }
        if let Some(correction) = &passage.correction {
            let _ = writeln!(out, "⚠️ Đính chính: {correction}\n");
        }
        sources(&mut out, dataset, &passage.source_ref);
    }
    for gap in validated.gaps.iter().filter(|gap| gap.batch == batch) {
        let _ = writeln!(
            out,
            "## {} {} (gap)\n\n{} ({})\n",
            gap.collection, gap.path, gap.gap.gap, gap.gap.checked
        );
    }
    out
}
