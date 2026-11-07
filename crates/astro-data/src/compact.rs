//! The compact form a batch is written in, expanded to the JSON of `data/interpretations/`.
//!
//! One JSON object per line. Writing a batch this way costs about half the output of the full
//! JSON, which repeats the same keys, the source and the indentation on every item. The JSON
//! stays the only source of truth: the compact file is a step of `/ingest-content`, not data.
//!
//! - A line with `collection` starts a collection and sets defaults: `batch`, `source`, and
//!   optionally `type` (default `paraphrase`) and `status` (default `draft`).
//! - A line with `gap` is a gap: `{"gap": <key>, "why": "…", "checked": "…"}`.
//! - Any other line is an item. Short names: `k` key, `ks` keys, `s` summary, `b` body, `p`
//!   pages, `loc` locator, `kw` keywords, `t` source type, `x` sensitivity, `c` correction,
//!   `a` anchor, `w` when, `if` condition text, `sec` section, `v` verse. Other fields pass
//!   through unchanged, and validation refuses those a collection does not have.

use crate::collections::find;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

const RENAMES: [(&str, &str); 12] = [
    ("k", "key"),
    ("ks", "keys"),
    ("s", "summary"),
    ("b", "body"),
    ("x", "sensitivity"),
    ("c", "correction"),
    ("a", "anchor"),
    ("w", "when"),
    ("if", "condition_text"),
    ("sec", "section"),
    ("v", "verse"),
    ("id", "id"),
];

struct Defaults {
    collection: String,
    batch: String,
    source: String,
    source_type: String,
    status: String,
}

#[derive(Default)]
struct Expanded {
    entries: Vec<Value>,
    gaps: Vec<Value>,
}

fn text(object: &Map<String, Value>, name: &str) -> Option<String> {
    object.get(name).and_then(Value::as_str).map(str::to_string)
}

fn header(object: &Map<String, Value>, previous: Option<&Defaults>) -> Result<Defaults, String> {
    let inherited = |name: &str, default: &str| {
        text(object, name)
            .or_else(|| {
                previous.map(|defaults| match name {
                    "batch" => defaults.batch.clone(),
                    "source" => defaults.source.clone(),
                    _ => default.to_string(),
                })
            })
            .unwrap_or_else(|| default.to_string())
    };
    let defaults = Defaults {
        collection: text(object, "collection").unwrap_or_default(),
        batch: inherited("batch", ""),
        source: inherited("source", ""),
        source_type: text(object, "type").unwrap_or_else(|| "paraphrase".to_string()),
        status: text(object, "status").unwrap_or_else(|| "draft".to_string()),
    };
    if find(&defaults.collection).is_none() {
        return Err(format!("collection lạ: {}", defaults.collection));
    }
    if defaults.batch.is_empty() || defaults.source.is_empty() {
        return Err("dòng collection đầu tiên cần batch và source".to_string());
    }
    Ok(defaults)
}

fn item(mut object: Map<String, Value>, defaults: &Defaults) -> Value {
    let mut reference = Map::new();
    reference.insert("source".into(), json!(defaults.source));
    let source_type = object
        .remove("t")
        .unwrap_or_else(|| json!(defaults.source_type));
    reference.insert("source_type".into(), source_type);
    for (short, long) in [("p", "pages"), ("loc", "locator"), ("kw", "keywords")] {
        if let Some(value) = object.remove(short) {
            reference.insert(long.into(), value);
        }
    }
    let mut out = Map::new();
    for (short, long) in RENAMES {
        if let Some(value) = object.remove(short) {
            out.insert(long.into(), value);
        }
    }
    out.insert("source_ref".into(), Value::Object(reference));
    out.insert("status".into(), json!(defaults.status));
    out.extend(object);
    Value::Object(out)
}

fn gap(mut object: Map<String, Value>) -> Value {
    json!({
        "key": object.remove("gap").unwrap_or(Value::Null),
        "gap": object.remove("why").unwrap_or(Value::Null),
        "checked": object.remove("checked").unwrap_or(Value::Null),
    })
}

/// The data files of a compact text, keyed by their path under `data/interpretations/`.
pub fn expand(compact: &str) -> Result<BTreeMap<String, Value>, String> {
    let mut defaults: Option<Defaults> = None;
    let mut collections: BTreeMap<(String, String), Expanded> = BTreeMap::new();
    for (index, line) in compact.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let place = |message: String| format!("dòng {}: {message}", index + 1);
        let value: Value = serde_json::from_str(line).map_err(|e| place(e.to_string()))?;
        let Value::Object(object) = value else {
            return Err(place("cần một object".to_string()));
        };
        if object.contains_key("collection") {
            defaults = Some(header(&object, defaults.as_ref()).map_err(place)?);
            continue;
        }
        let current = defaults
            .as_ref()
            .ok_or_else(|| place("chưa có dòng collection".to_string()))?;
        let slot = collections
            .entry((current.collection.clone(), current.batch.clone()))
            .or_default();
        if object.contains_key("gap") {
            slot.gaps.push(gap(object));
        } else {
            slot.entries.push(item(object, current));
        }
    }
    Ok(collections
        .into_iter()
        .map(|((collection, batch), expanded)| {
            let (module, short) = collection.split_once('.').expect("ids have a module");
            let path = format!("{module}/{}.{short}.json", batch.to_lowercase());
            let mut file =
                json!({ "collection": collection, "batch": batch, "entries": expanded.entries });
            if !expanded.gaps.is_empty() {
                file["gaps"] = Value::Array(expanded.gaps);
            }
            (path, file)
        })
        .collect())
}

/// A data file as written to disk: two-space indentation and a final newline.
pub fn pretty(file: &Value) -> String {
    let mut text = serde_json::to_string_pretty(file).expect("JSON values serialize");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::load;
    use crate::test_support::TestRoot;
    use crate::validate::validate;

    const SAMPLE: &str = r#"
// A comment.
{"collection":"tuvi.passage","batch":"TV-15","source":"book"}
{"id":"q/1","a":{"role":"QuanLoc","stars":["TuVi"]},"w":[{"with_all":["ThatSat"]}],"b":["Có uy quyền."],"p":[158],"kw":["Sát đồng cung: có uy quyền"]}
{"gap":["NoBoc","TuVi"],"why":"Không chia theo Chính Tinh","checked":"book tr. 168"}
{"collection":"tuvi.palace"}
{"k":"QuanLoc","s":"Cho biết công danh.","p":[158],"kw":["có uy quyền"],"t":"quote","status":"approved"}
"#;

    #[test]
    fn short_names_expand_to_the_full_schema() {
        let files = expand(SAMPLE).unwrap();
        assert_eq!(files.len(), 2);
        let passages = &files["tuvi/tv-15.passage.json"];
        let passage = &passages["entries"][0];
        assert_eq!(passage["id"], "q/1");
        assert_eq!(passage["anchor"]["role"], "QuanLoc");
        assert_eq!(passage["when"][0]["with_all"][0], "ThatSat");
        assert_eq!(passage["source_ref"]["source"], "book");
        assert_eq!(passage["source_ref"]["source_type"], "paraphrase");
        assert_eq!(passage["source_ref"]["pages"][0], 158);
        assert_eq!(passage["status"], "draft");
        assert_eq!(passages["gaps"][0]["gap"], "Không chia theo Chính Tinh");
    }

    #[test]
    fn a_new_collection_keeps_the_batch_and_source_and_lines_may_override() {
        let files = expand(SAMPLE).unwrap();
        let palace = &files["tuvi/tv-15.palace.json"]["entries"][0];
        assert_eq!(palace["key"], "QuanLoc");
        assert_eq!(palace["source_ref"]["source"], "book");
        assert_eq!(palace["source_ref"]["source_type"], "quote");
        assert_eq!(
            palace["status"], "approved",
            "a line may override the default"
        );
    }

    #[test]
    fn expanded_files_pass_validation() {
        let root = TestRoot::new();
        for (path, file) in expand(SAMPLE).unwrap() {
            root.write(&format!("data/interpretations/{path}"), &pretty(&file));
        }
        let issues = validate(&load(root.path()).unwrap()).issues;
        assert_eq!(issues, []);
    }

    #[test]
    fn mistakes_name_their_line() {
        assert!(expand("{\"id\":\"x\"}").unwrap_err().starts_with("dòng 1:"));
        assert!(expand("{\"collection\":\"tuvi.nope\",\"batch\":\"X\",\"source\":\"s\"}").is_err());
        assert!(expand("{\"collection\":\"tuvi.palace\"}").is_err());
        assert!(expand("not json").unwrap_err().starts_with("dòng 1:"));
    }
}
