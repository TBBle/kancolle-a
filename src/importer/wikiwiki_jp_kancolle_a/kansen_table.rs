//! Module for importer from https://wikiwiki.jp/kancolle-a/艦船/テーブル or https://wikiwiki.jp/kancolle-a/改造艦船/テーブル

use csv::{ReaderBuilder, StringRecord};
use regex::Regex;
use serde::Deserialize;
use std::error::Error;
use std::io::Read;

use lazy_static_include::*;

lazy_static_include_bytes! {
    // https://wikiwiki.jp/kancolle-a/?cmd=edit&page=艦船%2Fテーブル
    pub(crate) KANSEN => "src/importer/wikiwiki_jp_kancolle_a/kansen_table/艦船_テーブル.txt",
    // https://wikiwiki.jp/kancolle-a/?cmd=edit&page=改造艦船%2Fテーブル
    pub(crate) KAIZOU_KANSEN => "src/importer/wikiwiki_jp_kancolle_a/kansen_table/改造艦船_テーブル.txt",
}

// TODO: Yes, I know "kansen" means "ship". Naming is hard.
type KansenTable = Vec<KansenShip>;

// Notes for implementation:
// The help text appears when you edit a page, I'm not sure if there's a better source.
// Rows ending with "c" are format specifications, ignore them.
// Rows ending with "h" or "f" are headers or footers respectively. Field name source.
// Cells with only ">" or "~" are colspan (right)/rowspan (up) respectively.
// - For rowspan, duplicate previous row's data.
// - No colspan in the current data, thankfully.
// Cells starting with "~" are header cells. Ignore rows with a first cell like this.
// - Used in current data to insert repeats of the table headers.
// There's a bunch of format descriptors that can be used at the start of a cell.
// - A quick glance through didn't reveal any being used in the current data.
// Also, will need to parse links. [[text]] and [[text>page_path]].

/// Clean the given record from wikiwiki format to plain-text CSV
/// Specifially: Drops the first and last cells, and removes any markup.
fn clean_record(original: &StringRecord) -> StringRecord {
    let mut result = StringRecord::with_capacity(original.as_slice().len(), original.len() - 1);
    // TODO: Proper error handling.
    assert_eq!(&original[0], "");
    // Probably not perfect regexes, but they'll hold.
    let simple_link_regex = Regex::new(r"\[\[([^]]*)\]\]").unwrap();
    let complex_link_regex = Regex::new(r"\[\[([^]>]*)>([^]]*)\]\]").unwrap();
    for field in original.iter().take(original.len() - 1).skip(1) {
        let field = field.replace("&br;", " ");
        let field = field.trim();
        let field = field.trim_start_matches('~');
        let field = complex_link_regex.replace_all(&field, "$1");
        let field = simple_link_regex.replace_all(&field, "$1");

        result.push_field(&field);
    }

    result
}

/// Parses a Kansen Table from the provided Wikiwiki table reader.
/// Fails if not given a Wikiwiki table, or expected data structure does not match.
// TODO: Proper error
pub(crate) fn read_kansen_table(reader: impl Read) -> Result<KansenTable, Box<dyn Error>> {
    // Theory: Wikiwiki tables are basically pipe-separator CSV files with some special behaviours
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .from_reader(reader);

    // First special behaviour, the header is not the first row, it's the row with a 'h' in the last column
    // TODO: Do we need to rewind the iterator? Do we care about rows before the header row?
    // TODO: Do we need to worry about the first header being ~-tagged cells instead of being a h-suffixed row?
    for record in rdr.records() {
        let record = record?;
        if record.iter().last().unwrap() == "h" {
            rdr.set_headers(clean_record(&record));
            break;
        }
    }

    let header = rdr.headers()?.clone();

    let mut result = KansenTable::new();

    // Next special behaviour: The first cell is empty, as this format uses | on both ends.
    // We need to skip records with any final character, or with a second cell first character ~
    for record in rdr.records() {
        let record = record?;
        if record.iter().last().unwrap() != "" {
            continue;
        }
        if record[1].starts_with("~") {
            continue;
        }

        // Otherwise, let serde do its job.
        // TODO: Capture previous note to implement rowspan (only in notes so far)
        result.push(clean_record(&record).deserialize(Some(&header))?);
    }

    Ok(result)
}

/// Ship table entries from the kancolle-a wiki. Stats represent level 1 stats.
/// Optional stats are absent if no one has scanned a level 1 card yet, i.e. 初期値未確認.
// TODO: Optional stat fields are actually "Unknown", maybe make that an explicit enum?
// TODO: ship_class_index is "-" for a few ships, should we trim that or make it optional?
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KansenShip {
    #[serde(rename = "No.")]
    pub book_no: u16,
    #[serde(rename = "レア")]
    pub rarity: u16,
    #[serde(rename = "艦名")]
    pub ship_name: String,
    #[serde(rename = "艦型")]
    pub ship_class: String,
    #[serde(rename = "艦番")]
    pub ship_class_index: String,
    #[serde(rename = "艦種")]
    pub ship_type: String,
    #[serde(rename = "耐久")]
    pub endurance: u16,
    #[serde(rename = "火力")]
    pub firepower: Option<u16>,
    #[serde(rename = "装甲")]
    pub armor: Option<u16>,
    #[serde(rename = "雷装")]
    pub torpedo: Option<u16>,
    #[serde(rename = "回避")]
    pub evasion: Option<u16>,
    #[serde(rename = "対空")]
    pub anti_aircraft: Option<u16>,
    #[serde(rename = "搭載")]
    pub aircraft_load: u16,
    #[serde(rename = "対潜")]
    pub anti_submarine: Option<u16>,
    #[serde(rename = "速力")]
    pub speed: String, // Enum
    #[serde(rename = "索敵")]
    pub search: Option<u16>,
    #[serde(rename = "射程")]
    pub range: String, // Enum
    #[serde(rename = "運")]
    pub luck: u16,
    #[serde(rename = "備考")]
    pub notes: String,
}

#[cfg(test)]
mod tests;
