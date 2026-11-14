//! Check V13: every passage must appear on some real chart.
//!
//! Charts come from `astro-core`, and passages are picked by the same `content` module as the
//! apps, so this also runs the selector end to end. Unlike V7 it sees every star and the year.

use crate::compile::Compiled;
use astro_core::content::{PalaceQuery, gate_matches, tuvi_palace_query};
use astro_core::tuvi::{TuViChart, TuViConfig, TuViPalaceRole, tuvi_chart_from_lunar};
use astro_core::{Branch, Gender, LunarDate};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Range, RangeInclusive};

/// Which charts to go through: every month, double-hour and sex of these years and days.
#[derive(Debug, Clone)]
pub struct ScanRange {
    /// Lunar years.
    pub years: Range<i32>,
    /// Lunar days.
    pub days: RangeInclusive<u8>,
}

impl ScanRange {
    /// One 60-year cycle: 510.432 charts, leap months aside.
    pub fn full() -> Self {
        ScanRange {
            years: 1984..2044,
            days: 1..=30,
        }
    }
}

/// Calls `visit` on each chart until it returns false. The year changes fastest, so that rare
/// combinations of year and placement come up early.
fn each_chart(range: &ScanRange, mut visit: impl FnMut(&TuViChart) -> bool) {
    let config = TuViConfig::default();
    for day in range.days.clone() {
        for month in 1..=12 {
            for hour in Branch::ALL {
                for gender in [Gender::Male, Gender::Female] {
                    for year in range.years.clone() {
                        let lunar = LunarDate {
                            year,
                            month,
                            day,
                            is_leap_month: false,
                        };
                        // Months of 29 days have no day 30.
                        let Ok(chart) = tuvi_chart_from_lunar(lunar, hour, gender, &config) else {
                            continue;
                        };
                        if !visit(&chart) {
                            return;
                        }
                    }
                }
            }
        }
    }
}

/// The passages `query` picks, by row id: what an app shows for one palace.
pub fn select(compiled: &Compiled, query: &PalaceQuery) -> BTreeSet<u32> {
    let mut ids = BTreeSet::new();
    for lookup in &query.lookups {
        for anchor in &lookup.anchors {
            for &id in compiled.anchors.get(anchor).into_iter().flatten() {
                let row = &compiled.passages[id as usize - 1];
                if gate_matches(&row.gates, &lookup.facts) {
                    ids.insert(id);
                }
            }
        }
    }
    ids
}

fn roles_with_passages(compiled: &Compiled) -> Vec<TuViPalaceRole> {
    TuViPalaceRole::ALL
        .into_iter()
        .filter(|role| {
            let prefix = format!("{role:?}/");
            compiled
                .anchors
                .keys()
                .any(|anchor| anchor.starts_with(&prefix))
        })
        .collect()
}

/// Stable ids of the passages no chart of `range` shows. Stops as soon as all have appeared.
pub fn unseen(compiled: &Compiled, range: &ScanRange) -> Vec<String> {
    let mut missing: BTreeSet<u32> = compiled.passages.iter().map(|row| row.id).collect();
    let roles = roles_with_passages(compiled);
    each_chart(range, |chart| {
        for &role in &roles {
            let branch = chart.palace_of_role(role).branch;
            for id in select(compiled, &tuvi_palace_query(chart, branch)) {
                missing.remove(&id);
            }
        }
        !missing.is_empty()
    });
    missing
        .iter()
        .map(|&id| compiled.passages[id as usize - 1].stable_id.clone())
        .collect()
}

/// How many passages one palace shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PalaceLoad {
    /// Charts counted.
    pub charts: u64,
    /// Passages shown, over all charts.
    pub shown: u64,
    /// Those of them that are firm statements.
    pub primary: u64,
    /// Fewest passages on a chart.
    pub min: u64,
    /// Most passages on a chart.
    pub max: u64,
}

/// The load of each palace that has passages, over every chart of `range`.
pub fn load(compiled: &Compiled, range: &ScanRange) -> BTreeMap<String, PalaceLoad> {
    let roles = roles_with_passages(compiled);
    let mut loads: BTreeMap<String, PalaceLoad> = BTreeMap::new();
    each_chart(range, |chart| {
        for &role in &roles {
            let branch = chart.palace_of_role(role).branch;
            let ids = select(compiled, &tuvi_palace_query(chart, branch));
            let primary = ids
                .iter()
                .filter(|&&id| compiled.passages[id as usize - 1].tier == "Primary")
                .count() as u64;
            let shown = ids.len() as u64;
            let entry = loads.entry(format!("{role:?}")).or_default();
            entry.min = if entry.charts == 0 {
                shown
            } else {
                entry.min.min(shown)
            };
            entry.max = entry.max.max(shown);
            entry.charts += 1;
            entry.shown += shown;
            entry.primary += primary;
        }
        true
    });
    loads
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::{Gate, PassageRow};

    fn fact(text: &str) -> Vec<String> {
        vec![text.to_string()]
    }

    fn row(id: u32, name: &str, gate: Gate) -> PassageRow {
        PassageRow {
            id,
            stable_id: name.to_string(),
            tier: "Primary".to_string(),
            section: None,
            verse: None,
            gates: vec![gate],
            condition_text: None,
            correction: None,
            body: vec!["x".to_string()],
            citation: "x".to_string(),
            sensitivity: 0,
            draft: false,
        }
    }

    /// Twelve years for every branch, two days: enough to be quick and still see each star.
    fn small() -> ScanRange {
        ScanRange {
            years: 1984..1996,
            days: 1..=2,
        }
    }

    fn compiled(rows: Vec<PassageRow>) -> Compiled {
        let ids: Vec<u32> = rows.iter().map(|row| row.id).collect();
        Compiled {
            entries: vec![],
            passages: rows,
            anchors: BTreeMap::from([("QuanLoc/~Any".to_string(), ids)]),
        }
    }

    #[test]
    fn a_passage_that_can_appear_is_seen() {
        let compiled = compiled(vec![row(
            1,
            "a",
            Gate {
                all: fact("with:DaoHoa"),
                ..Gate::default()
            },
        )]);
        assert_eq!(unseen(&compiled, &small()), Vec::<String>::new());
    }

    #[test]
    fn the_impossible_age_of_section_8_43_is_caught() {
        // Đào Hoa at Tý belongs to Hợi, Mão, Mùi years only: V7 cannot see this, V13 must.
        let dead = Gate {
            all: vec!["with:DaoHoa".to_string(), "branch:Ty".to_string()],
            any: vec![vec![
                "year_branch:Ty".to_string(),
                "year_branch:Dau".to_string(),
            ]],
            none: vec![],
        };
        let live = Gate {
            all: fact("with:DaoHoa"),
            ..Gate::default()
        };
        let compiled = compiled(vec![row(1, "dead", dead), row(2, "live", live)]);
        assert_eq!(unseen(&compiled, &small()), ["dead"]);
    }

    #[test]
    fn load_counts_firm_statements_apart() {
        let mut more = row(2, "more", Gate::default());
        more.tier = "More".to_string();
        let compiled = compiled(vec![row(1, "firm", Gate::default()), more]);
        let range = ScanRange {
            years: 1984..1985,
            days: 1..=1,
        };
        let load = &load(&compiled, &range)["QuanLoc"];
        assert_eq!(load.min, 2);
        assert_eq!(load.max, 2);
        assert_eq!(load.shown, 2 * load.charts);
        assert_eq!(load.primary, load.charts);
    }
}
