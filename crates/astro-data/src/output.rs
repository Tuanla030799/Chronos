//! Writes the build: the SQLite file, the Web files and the manifest (sections 2.7 and 8), and
//! reads them back for check V9.

use crate::compile::{Compiled, EntryRow, PassageRow};
use astro_core::ENGINE_VERSION;
use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Version of the build format.
pub const SCHEMA_VERSION: u32 = 1;
/// `CHRN` as a big-endian integer, the `application_id` of the SQLite file.
pub const APPLICATION_ID: u32 = 1_128_813_134;
/// Name of the SQLite file.
pub const SQLITE_FILE: &str = "chronos_content.sqlite";

const SCHEMA: &str = "
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) WITHOUT ROWID;
CREATE TABLE entry (
  key         TEXT PRIMARY KEY,
  collection  TEXT NOT NULL,
  title       TEXT NOT NULL,
  luck        TEXT,
  summary     TEXT NOT NULL,
  body        TEXT NOT NULL,
  extra       TEXT,
  citation    TEXT NOT NULL,
  sensitivity INTEGER NOT NULL DEFAULT 0,
  draft       INTEGER NOT NULL DEFAULT 0
) WITHOUT ROWID;
CREATE INDEX entry_collection ON entry (collection);
CREATE TABLE passage (
  id             INTEGER PRIMARY KEY,
  stable_id      TEXT NOT NULL UNIQUE,
  section        TEXT,
  verse          TEXT,
  gates          TEXT NOT NULL,
  condition_text TEXT,
  body           TEXT NOT NULL,
  citation       TEXT NOT NULL,
  sensitivity    INTEGER NOT NULL DEFAULT 0,
  draft          INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE passage_anchor (
  anchor     TEXT NOT NULL,
  passage_id INTEGER NOT NULL REFERENCES passage (id),
  PRIMARY KEY (anchor, passage_id)
) WITHOUT ROWID;
";

/// One Web file of the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleFile {
    /// Name under `web/`.
    pub file: String,
    /// Entries in it.
    pub entries: usize,
    /// Passages in it.
    pub passages: usize,
}

/// `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Version of the build format.
    pub schema_version: u32,
    /// Engine the data was checked against.
    pub engine_version: String,
    /// Hash of the rows, independent of the file formats.
    pub content_hash: String,
    /// The Web files by module.
    pub modules: BTreeMap<String, ModuleFile>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct WebModule {
    entries: BTreeMap<String, EntryRow>,
    passages: Vec<PassageRow>,
    anchors: BTreeMap<String, Vec<u32>>,
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// Hash of the rows.
pub fn content_hash(compiled: &Compiled) -> String {
    let canonical = serde_json::to_vec(compiled).expect("rows serialize");
    format!("sha256:{}", hex(canonical))
}

fn write_sqlite(compiled: &Compiled, manifest: &Manifest, path: &Path) -> rusqlite::Result<()> {
    let mut connection = Connection::open(path)?;
    connection.execute_batch(&format!(
        "PRAGMA journal_mode = DELETE;
         PRAGMA application_id = {APPLICATION_ID};
         PRAGMA user_version = {SCHEMA_VERSION};
         {SCHEMA}"
    ))?;
    let json = |value: &[String]| serde_json::to_string(value).expect("strings serialize");
    let transaction = connection.transaction()?;
    {
        let mut meta = transaction.prepare("INSERT INTO meta VALUES (?1, ?2)")?;
        meta.execute(params!["schema_version", SCHEMA_VERSION.to_string()])?;
        meta.execute(params!["engine_version", manifest.engine_version])?;
        meta.execute(params!["content_hash", manifest.content_hash])?;
        let mut entry = transaction
            .prepare("INSERT INTO entry VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)")?;
        for row in &compiled.entries {
            entry.execute(params![
                row.key,
                row.collection,
                row.title,
                row.luck,
                row.summary,
                json(&row.body),
                row.extra.as_ref().map(|extra| extra.to_string()),
                row.citation,
                row.sensitivity,
                row.draft,
            ])?;
        }
        let mut passage = transaction
            .prepare("INSERT INTO passage VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)")?;
        for row in &compiled.passages {
            passage.execute(params![
                row.id,
                row.stable_id,
                row.section,
                row.verse,
                serde_json::to_string(&row.gates).expect("gates serialize"),
                row.condition_text,
                json(&row.body),
                row.citation,
                row.sensitivity,
                row.draft,
            ])?;
        }
        let mut anchor = transaction.prepare("INSERT INTO passage_anchor VALUES (?1, ?2)")?;
        for (key, ids) in &compiled.anchors {
            for id in ids {
                anchor.execute(params![key, id])?;
            }
        }
    }
    transaction.commit()?;
    connection.execute_batch("VACUUM;")
}

/// Opens the SQLite file the way the app does.
pub fn open_read_only(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(
        format!("file:{}?immutable=1", path.display()),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
}

/// Reads the rows of the SQLite file.
pub fn read_sqlite(path: &Path) -> Result<Compiled, String> {
    fn strings(json: String) -> rusqlite::Result<Vec<String>> {
        serde_json::from_str(&json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
    }
    let connection = open_read_only(path).map_err(text)?;
    let mut compiled = Compiled::default();
    let mut statement = connection
        .prepare(
            "SELECT key, collection, title, luck, summary, body, extra, citation, sensitivity, draft
             FROM entry ORDER BY key",
        )
        .map_err(text)?;
    compiled.entries = statement
        .query_map([], |row| {
            let extra: Option<String> = row.get(6)?;
            Ok(EntryRow {
                key: row.get(0)?,
                collection: row.get(1)?,
                title: row.get(2)?,
                luck: row.get(3)?,
                summary: row.get(4)?,
                body: strings(row.get(5)?)?,
                extra: extra.map(|extra| serde_json::from_str(&extra).expect("written as JSON")),
                citation: row.get(7)?,
                sensitivity: row.get(8)?,
                draft: row.get(9)?,
            })
        })
        .map_err(text)?
        .collect::<Result<_, _>>()
        .map_err(text)?;
    let mut statement = connection
        .prepare(
            "SELECT id, stable_id, section, verse, gates, condition_text, body, citation,
                    sensitivity, draft
             FROM passage ORDER BY id",
        )
        .map_err(text)?;
    compiled.passages = statement
        .query_map([], |row| {
            let gates: String = row.get(4)?;
            Ok(PassageRow {
                id: row.get(0)?,
                stable_id: row.get(1)?,
                section: row.get(2)?,
                verse: row.get(3)?,
                gates: serde_json::from_str(&gates).expect("written as JSON"),
                condition_text: row.get(5)?,
                body: strings(row.get(6)?)?,
                citation: row.get(7)?,
                sensitivity: row.get(8)?,
                draft: row.get(9)?,
            })
        })
        .map_err(text)?
        .collect::<Result<_, _>>()
        .map_err(text)?;
    let mut statement = connection
        .prepare("SELECT anchor, passage_id FROM passage_anchor ORDER BY anchor, passage_id")
        .map_err(text)?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })
        .map_err(text)?;
    for row in rows {
        let (anchor, id) = row.map_err(text)?;
        compiled.anchors.entry(anchor).or_default().push(id);
    }
    Ok(compiled)
}

fn module_of(key: &str) -> &str {
    key.split('.').next().unwrap_or(key)
}

fn split_by_module(compiled: &Compiled) -> BTreeMap<String, WebModule> {
    let mut modules: BTreeMap<String, WebModule> = BTreeMap::new();
    for row in &compiled.entries {
        let module = modules.entry(module_of(&row.key).to_string()).or_default();
        module.entries.insert(row.key.clone(), row.clone());
    }
    if !compiled.passages.is_empty() {
        let module = modules.entry("tuvi".to_string()).or_default();
        module.passages.clone_from(&compiled.passages);
        module.anchors.clone_from(&compiled.anchors);
    }
    modules
}

/// Reads the rows of the Web files of `manifest`.
pub fn read_web(dir: &Path, manifest: &Manifest) -> Result<Compiled, String> {
    let mut compiled = Compiled::default();
    for module in manifest.modules.values() {
        let text_of = fs::read_to_string(dir.join("web").join(&module.file)).map_err(text)?;
        let module: WebModule = serde_json::from_str(&text_of).map_err(text)?;
        compiled.entries.extend(module.entries.into_values());
        compiled.passages.extend(module.passages);
        compiled.anchors.extend(module.anchors);
    }
    compiled.entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(compiled)
}

/// Writes the build into `dir`, then reads it back to prove both formats hold the same rows (V9).
pub fn write(compiled: &Compiled, dir: &Path) -> Result<Manifest, String> {
    let web = dir.join("web");
    if web.exists() {
        fs::remove_dir_all(&web).map_err(text)?;
    }
    fs::create_dir_all(&web).map_err(text)?;
    let hash = content_hash(compiled);
    let mut modules = BTreeMap::new();
    for (name, module) in split_by_module(compiled) {
        let json = serde_json::to_string(&module).expect("rows serialize");
        let file = format!("{name}.{}.json", &hex(&json)[..8]);
        fs::write(web.join(&file), &json).map_err(text)?;
        let entries = module.entries.len();
        let passages = module.passages.len();
        modules.insert(
            name,
            ModuleFile {
                file,
                entries,
                passages,
            },
        );
    }
    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        engine_version: ENGINE_VERSION.to_string(),
        content_hash: hash.clone(),
        modules,
    };
    let sqlite = dir.join(SQLITE_FILE);
    if sqlite.exists() {
        fs::remove_file(&sqlite).map_err(text)?;
    }
    write_sqlite(compiled, &manifest, &sqlite).map_err(text)?;
    let json = serde_json::to_string_pretty(&manifest).expect("manifest serializes");
    fs::write(dir.join("manifest.json"), json).map_err(text)?;

    for (name, rows) in [
        ("SQLite", read_sqlite(&sqlite)?),
        ("Web", read_web(dir, &manifest)?),
    ] {
        if content_hash(&rows) != hash {
            return Err(format!("V9: {name} khác nội dung đã biên dịch"));
        }
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::{Gate, compile};
    use crate::dataset::load;
    use crate::test_support::TestRoot;
    use crate::validate::validate;

    fn sample(root: &TestRoot, status: &str) -> Compiled {
        let reference = r#""source_ref": { "source": "book", "source_type": "paraphrase", "pages": [158], "keywords": ["có uy quyền"] }"#;
        root.write(
            "data/interpretations/tuvi/tv-01.star.json",
            &format!(
                r#"{{ "collection": "tuvi.star", "batch": "TV-01", "entries": [
                    {{ "key": "TuVi", "summary": "Chủ về uy quyền.", "body": ["Một đoạn."], {reference}, "status": "{status}",
                       "profile": {{ "dau": "NamBacDau", "element": "Dương Thổ", "labels": ["Đế tinh"] }} }},
                    {{ "keys": ["KinhDuong", "DaLa"], "summary": "Chủ sát phạt.", {reference}, "status": "draft", "sensitivity": ["Death", "Legal"] }}] }}"#
            ),
        );
        root.write(
            "data/interpretations/tuvi/tv-15.passage.json",
            &format!(
                r#"{{ "collection": "tuvi.passage", "batch": "TV-15", "entries": [
                    {{ "id": "quan-loc/tu-vi/01", "anchor": {{ "role": "QuanLoc", "stars": ["TuVi", "ThatSat"] }},
                       "when": [{{ "with_all": ["TuVi", "ThatSat"] }}], "body": ["Có thực quyền."], {reference}, "status": "{status}" }}] }}"#
            ),
        );
        let dataset = load(root.path()).unwrap();
        let validated = validate(&dataset);
        assert_eq!(validated.issues, []);
        compile(&validated, &dataset.sources, false)
    }

    #[test]
    fn a_build_holds_the_same_rows_in_both_formats() {
        let root = TestRoot::new();
        let compiled = sample(&root, "draft");
        let dir = root.path().join("out");
        let manifest = write(&compiled, &dir).unwrap();
        assert_eq!(read_sqlite(&dir.join(SQLITE_FILE)).unwrap(), compiled);
        assert_eq!(read_web(&dir, &manifest).unwrap(), compiled);
        assert_eq!(compiled.entries.len(), 3);
        assert_eq!(manifest.modules["tuvi"].passages, 1);
    }

    #[test]
    fn the_sqlite_file_is_ready_for_a_read_only_bundle() {
        let root = TestRoot::new();
        let dir = root.path().join("out");
        write(&sample(&root, "draft"), &dir).unwrap();
        let connection = open_read_only(&dir.join(SQLITE_FILE)).unwrap();
        let pragma = |name: &str| -> String {
            connection
                .query_row(&format!("PRAGMA {name}"), [], |row| {
                    Ok(row.get_ref(0)?.as_i64().map_or_else(
                        |_| row.get::<_, String>(0).unwrap(),
                        |number| number.to_string(),
                    ))
                })
                .unwrap()
        };
        assert_eq!(pragma("user_version"), SCHEMA_VERSION.to_string());
        assert_eq!(pragma("application_id"), APPLICATION_ID.to_string());
        assert_eq!(pragma("journal_mode").to_lowercase(), "delete");
        assert!(connection.execute("DELETE FROM entry", []).is_err());
        assert!(!dir.join(format!("{SQLITE_FILE}-wal")).exists());
    }

    #[test]
    fn a_passage_is_found_by_each_of_its_anchors() {
        let root = TestRoot::new();
        let dir = root.path().join("out");
        write(&sample(&root, "draft"), &dir).unwrap();
        let connection = open_read_only(&dir.join(SQLITE_FILE)).unwrap();
        for anchor in ["QuanLoc/TuVi", "QuanLoc/ThatSat"] {
            let stable_id: String = connection
                .query_row(
                    "SELECT stable_id FROM passage JOIN passage_anchor ON id = passage_id
                     WHERE anchor = ?1",
                    [anchor],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(stable_id, "quan-loc/tu-vi/01");
        }
    }

    #[test]
    fn building_twice_gives_the_same_bytes() {
        let root = TestRoot::new();
        let compiled = sample(&root, "draft");
        let first = root.path().join("first");
        let second = root.path().join("second");
        let manifest = write(&compiled, &first).unwrap();
        assert_eq!(write(&compiled, &second).unwrap(), manifest);
        assert_eq!(
            fs::read(first.join(SQLITE_FILE)).unwrap(),
            fs::read(second.join(SQLITE_FILE)).unwrap()
        );
    }

    #[test]
    fn a_release_keeps_approved_items_only() {
        let root = TestRoot::new();
        let compiled = sample(&root, "approved");
        assert!(compiled.entries.iter().any(|row| row.draft));
        let dataset = load(root.path()).unwrap();
        let release = compile(&validate(&dataset), &dataset.sources, true);
        assert_eq!(release.entries.len(), 1);
        assert_eq!(release.entries[0].key, "tuvi.star/TuVi");
        assert!(!release.entries[0].draft);
        assert_eq!(release.passages.len(), 1);
    }

    #[test]
    fn rows_keep_core_names_shared_text_and_masks() {
        let root = TestRoot::new();
        let compiled = sample(&root, "draft");
        let row = |key: &str| compiled.entries.iter().find(|row| row.key == key).unwrap();
        assert_eq!(row("tuvi.star/TuVi").title, "Tử Vi");
        assert_eq!(row("tuvi.star/TuVi").luck.as_deref(), Some("Cat"));
        assert_eq!(row("tuvi.star/TuVi").citation, "Sách, tr. 158");
        assert!(row("tuvi.star/TuVi").extra.as_ref().unwrap()["profile"]["labels"].is_array());
        assert_eq!(
            row("tuvi.star/KinhDuong").summary,
            row("tuvi.star/DaLa").summary
        );
        assert_eq!(row("tuvi.star/DaLa").sensitivity, 0b101);
        assert_eq!(
            compiled.passages[0].gates,
            [Gate {
                all: vec!["with:TuVi".to_string(), "with:ThatSat".to_string()],
                any: vec![],
                none: vec![],
            }]
        );
    }
}
