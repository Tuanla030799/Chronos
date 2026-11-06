//! Loads `data/interpretations/` from disk.

use crate::model::DataFile;
use crate::source::{Packet, Source};
use astro_core::tuvi::TuViStar;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Where the data lives, from the repository root.
pub const DATA_DIR: &str = "data/interpretations";

/// The directories holding data files, one per module.
const MODULES: [&str; 4] = ["almanac", "bazi", "fengshui", "tuvi"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Activity {
    id: String,
    #[allow(dead_code)]
    label: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StarGroup {
    id: String,
    stars: Vec<TuViStar>,
    #[allow(dead_code)]
    source: String,
}

/// A data file with where it came from.
#[derive(Debug, Clone)]
pub struct LoadedFile {
    /// Path from the data directory, e.g. `tuvi/tv-15.passage.json`.
    pub path: String,
    /// The module directory it sits in.
    pub module: String,
    /// Size on disk.
    pub bytes: u64,
    /// Its contents.
    pub data: DataFile,
}

/// Everything under the data directory.
#[derive(Debug, Clone, Default)]
pub struct Dataset {
    /// The sources, by id.
    pub sources: BTreeMap<String, Source>,
    /// Source texts found on this machine, by source id.
    pub packets: BTreeMap<String, Packet>,
    /// Sources whose text is declared but absent here: V5 cannot run on them.
    pub missing_packets: BTreeSet<String>,
    /// Star groups, by id without the `@`.
    pub groups: BTreeMap<String, Vec<TuViStar>>,
    /// Activity ids.
    pub activities: BTreeSet<String>,
    /// The data files, sorted by path.
    pub files: Vec<LoadedFile>,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

/// Reads the data under `root`, the repository root.
pub fn load(root: &Path) -> Result<Dataset, String> {
    let dir: PathBuf = root.join(DATA_DIR);
    let mut dataset = Dataset::default();

    let sources: Vec<Source> = read_json(&dir.join("sources.json"))?;
    for source in sources {
        if let Some(packet) = &source.packet {
            match fs::read_to_string(root.join(packet)) {
                Ok(text) => {
                    dataset
                        .packets
                        .insert(source.id.clone(), Packet::parse(&text));
                }
                Err(_) => {
                    dataset.missing_packets.insert(source.id.clone());
                }
            }
        }
        dataset.sources.insert(source.id.clone(), source);
    }

    let activities: Vec<Activity> = read_json(&dir.join("vocab/activities.json"))?;
    dataset.activities = activities.into_iter().map(|activity| activity.id).collect();
    let groups: Vec<StarGroup> = read_json(&dir.join("vocab/star_groups.json"))?;
    dataset.groups = groups
        .into_iter()
        .map(|group| (group.id, group.stars))
        .collect();

    for module in MODULES {
        let module_dir = dir.join(module);
        if !module_dir.is_dir() {
            continue;
        }
        let mut names: Vec<String> = fs::read_dir(&module_dir)
            .map_err(|error| format!("{}: {error}", module_dir.display()))?
            .filter_map(Result::ok)
            .map(|file| file.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".json"))
            .collect();
        names.sort();
        for name in names {
            let path = module_dir.join(&name);
            let data: DataFile = read_json(&path)?;
            let bytes = fs::metadata(&path).map_or(0, |metadata| metadata.len());
            dataset.files.push(LoadedFile {
                path: format!("{module}/{name}"),
                module: module.to_string(),
                bytes,
                data,
            });
        }
    }
    Ok(dataset)
}
