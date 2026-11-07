//! # astro-data
//!
//! Build-time tool of the offline interpretation database, specified in
//! `docs/INTERPRETATION_DB_SPEC.md`. It reads the JSON under `data/interpretations/`, checks every
//! key against the real enums of `astro-core`, and compiles the entries to a read-only SQLite file
//! for iOS and JSON files for the Web. The apps never link this crate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod collections;
pub mod compact;
pub mod compile;
pub mod dataset;
pub mod model;
pub mod output;
pub mod reach;
pub mod review;
pub mod scan;
pub mod source;
pub mod stats;
#[cfg(test)]
mod test_support;
pub mod validate;
