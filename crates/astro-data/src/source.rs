//! `sources.json` and the page-keyed text of each source (section 3.3 of the specification).

use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

/// Rights over a source, checked by V11 when a release is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rights {
    /// Free to reuse.
    Open,
    /// Reused under a licence.
    Licensed,
    /// Released in paraphrase only, by decision of the project owner: no item may quote it.
    Paraphrased,
    /// Not checked yet.
    Unverified,
}

/// One entry of `sources.json`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Id used by `source_ref`.
    pub id: String,
    /// Title, shown in citations.
    pub title: String,
    /// Author.
    pub author: Option<String>,
    /// Edition or version used.
    pub edition: Option<String>,
    /// Rights over the text.
    pub rights: Rights,
    /// Path of the text, from the repository root.
    pub packet: Option<String>,
}

/// Replaces the Cyrillic letters that the text layer of *Tân Biên* uses in place of Latin ones,
/// such as the first letter of "Тhìn": they look the same but never match.
fn fold_homoglyph(letter: char) -> char {
    match letter {
        'А' => 'A',
        'В' => 'B',
        'С' => 'C',
        'Е' => 'E',
        'Н' => 'H',
        'К' => 'K',
        'М' => 'M',
        'О' => 'O',
        'Р' => 'P',
        'Т' => 'T',
        'Х' => 'X',
        'а' => 'a',
        'е' => 'e',
        'о' => 'o',
        'р' => 'p',
        'с' => 'c',
        'у' => 'y',
        'х' => 'x',
        other => other,
    }
}

/// `text` on one line, with homoglyphs folded, so that a phrase matches across a line break.
pub fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(fold_homoglyph)
        .collect()
}

/// The lowercase words of `text`.
pub fn words(text: &str) -> Vec<String> {
    normalize(text)
        .split(|letter: char| !letter.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// The text of a source, by printed page. A file without page marks is page 0.
#[derive(Debug, Clone, Default)]
pub struct Packet {
    pages: BTreeMap<u16, String>,
}

impl Packet {
    /// Reads blocks headed `=== tr. N ===`.
    pub fn parse(text: &str) -> Self {
        let mut pages: BTreeMap<u16, String> = BTreeMap::new();
        let mut page = 0;
        for line in text.lines() {
            let number = line
                .strip_prefix("=== tr. ")
                .and_then(|rest| rest.strip_suffix(" ==="))
                .and_then(|number| number.parse().ok());
            match number {
                Some(number) => page = number,
                None => {
                    let block = pages.entry(page).or_default();
                    block.push_str(line);
                    block.push('\n');
                }
            }
        }
        for block in pages.values_mut() {
            *block = normalize(block);
        }
        Packet { pages }
    }

    /// Whether page `page` exists.
    pub fn has_page(&self, page: u16) -> bool {
        self.pages.contains_key(&page)
    }

    /// Characters of the text of `page`.
    pub fn page_chars(&self, page: u16) -> usize {
        self.pages.get(&page).map_or(0, |text| text.chars().count())
    }

    fn selected<'a>(&'a self, pages: &'a [u16]) -> impl Iterator<Item = &'a String> {
        self.pages
            .iter()
            .filter(move |(page, _)| pages.is_empty() || pages.contains(page))
            .map(|(_, text)| text)
    }

    /// Whether `phrase` appears on one of `pages`, or anywhere when `pages` is empty.
    pub fn contains(&self, pages: &[u16], phrase: &str) -> bool {
        let phrase = normalize(phrase);
        self.selected(pages).any(|text| text.contains(&phrase))
    }

    /// `phrase` with the text around it, for review.
    pub fn context(&self, pages: &[u16], phrase: &str, radius: usize) -> Option<String> {
        let phrase = normalize(phrase);
        self.selected(pages).find_map(|text| {
            let at = text.find(&phrase)?;
            let before = text[..at].chars().rev().take(radius).collect::<Vec<_>>();
            let before: String = before.into_iter().rev().collect();
            let rest = &text[at + phrase.len()..];
            let after: String = rest.chars().take(radius).collect();
            Some(format!("{before}**{phrase}**{after}"))
        })
    }

    /// A run of at least `length` consecutive words of `text` that also stands on `pages`.
    pub fn copied_run(&self, pages: &[u16], text: &str, length: usize) -> Option<String> {
        let page_words: Vec<Vec<String>> = self.selected(pages).map(|page| words(page)).collect();
        let runs: HashSet<&[String]> = page_words
            .iter()
            .flat_map(|page| page.windows(length))
            .collect();
        let text_words = words(text);
        text_words
            .windows(length)
            .find(|window| runs.contains(window))
            .map(|window| window.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "=== tr. 158 ===\n- Sát đồng cung: có uy\nquyền, nên chuyên\n=== tr. 159 ===\nĐơn thủ tại Тhìn\n";

    #[test]
    fn pages_are_split_at_their_marks() {
        let packet = Packet::parse(TEXT);
        assert!(packet.has_page(158) && packet.has_page(159));
        assert!(!packet.has_page(160));
    }

    #[test]
    fn a_phrase_matches_across_a_line_break_on_its_own_page_only() {
        let packet = Packet::parse(TEXT);
        assert!(packet.contains(&[158], "có uy quyền, nên"));
        assert!(!packet.contains(&[159], "có uy quyền, nên"));
        assert!(packet.contains(&[], "có uy quyền, nên"));
    }

    #[test]
    fn cyrillic_lookalikes_match_their_latin_letters() {
        let packet = Packet::parse(TEXT);
        assert!(packet.contains(&[159], "Đơn thủ tại Thìn"));
        assert!(packet.contains(&[159], "Đơn thủ tại Тhìn"));
    }

    #[test]
    fn a_file_without_page_marks_is_one_page() {
        let packet = Packet::parse("Ngày Tam Nương\nKỵ khai trương\n");
        assert!(packet.contains(&[], "Tam Nương Kỵ khai"));
    }

    #[test]
    fn copied_runs_are_found_whatever_the_case_and_punctuation() {
        let packet = Packet::parse("=== tr. 1 ===\nmột hai ba, bốn năm sáu bảy\n");
        assert_eq!(
            packet
                .copied_run(&[1], "Xin chào: Một hai ba bốn NĂM!", 5)
                .as_deref(),
            Some("một hai ba bốn năm")
        );
        assert_eq!(packet.copied_run(&[1], "một hai ba bốn", 5), None);
    }

    #[test]
    fn context_marks_the_phrase() {
        let packet = Packet::parse(TEXT);
        let context = packet.context(&[158], "có uy quyền", 4).unwrap();
        assert_eq!(context, "ng: **có uy quyền**, nê");
    }
}
