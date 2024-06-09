//! Module for importers for https://kancolle-arcade.net/ac/api/ resources

use chrono::{DateTime, Utc};
use derive_getters::Getters;
use serde::Deserialize;
use serde_json::Result;
use std::sync::OnceLock;
use std::{collections::HashMap, io::Read, ops::Deref};

#[derive(Debug, Deserialize)]
pub struct TcBook(Vec<BookShip>);

impl TcBook {
    /// Parses a TcBook from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(tcbook_reader: impl Read) -> Result<TcBook> {
        let result: TcBook = serde_json::from_reader(tcbook_reader)?;
        Ok(result)
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for TcBook {
    type Target = Vec<BookShip>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShip {
    book_no: u16,
    ship_class: Option<String>,
    ship_class_index: Option<i16>,
    ship_type: String,
    ship_model_num: String,
    ship_name: String,
    card_index_img: String,
    card_list: Vec<BookShipCardPage>,
    variation_num: u16,
    acquire_num: u16,
    lv: u16,
    is_married: Option<Vec<bool>>,
    married_img: Option<Vec<String>>, // Probably... No married ships to validate this.
}

// Notes for future functions
// * Card images (s/tc_NO_xxx.jpg) live in https://kancolle-arcade.net/ac/resources/pictureBook/
// * Status images (i/i_xxx.png) live in https://kancolle-arcade.net/ac/resources/chara/
// ** Status images end with _n, _bs, _bm, or _bl. (Not sure if there's one for sunk?)

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BookShipCardPageSource {
    Unknown, // Fallback
    Normal,  // Priority 0 is always this

    // 期間限定ドロップイベント
    DecisiveBattle, // 決戦mode: May 2019 once-off
    Swimsuit,       // 水着mode, 里帰り水着mode, 夏のお嬢さんmode: July 2019 onward
    Christmas,      // クリスマスmode: December 2019 onward
    Halloween,      // ハロウィンmode: Oct 2020 onward
    Valentine,      // バレンタインmode: Feb 2021 onward
    PacificSaury,   // 秋刀魚mode, F作業mode: Oct 2021 onward (鎮守府秋刀魚祭り)
    SundayBest,     // 晴れ着mode: Jan 2022 onward
    RainySeason,    // 梅雨mode: May-June 2022 onward
    Yukata,         // 浴衣mode: Sep 2023 onward

    // オリジナルイラストカード
    OriginalIllustration, // Theory: Last page, may have a different variation_num_in_page
}

static BOOK_SHIP_SOURCES: OnceLock<HashMap<u16, Vec<BookShipCardPageSource>>> = OnceLock::new();

fn init_book_ship_sources() {
    use BookShipCardPageSource::*;
    BOOK_SHIP_SOURCES.get_or_init(|| {
        let mut sources = HashMap::new();
        // 赤城: https://kancolle-a.sega.jp/players/information/211230_2.html
        sources.insert(6, vec![SundayBest]);
        sources
    });
}

impl BookShip {
    /// Reports the event-source for the given page ("priority") of a TcBook entry
    pub fn source(&self, priority: u16) -> BookShipCardPageSource {
        use BookShipCardPageSource::*;
        if priority == 0 {
            return Normal;
        }
        init_book_ship_sources();
        if let Some(sources) = BOOK_SHIP_SOURCES.get().unwrap().get(self.book_no()) {
            sources
                .get((priority - 1) as usize)
                .or(Some(&Unknown))
                .unwrap()
                .to_owned()
        } else {
            Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_ship_source() {
        let ship = BookShip {
            book_no: 6,
            ship_class: None,
            ship_class_index: None,
            ship_type: "".to_string(),
            ship_model_num: "".to_string(),
            ship_name: "".to_string(),
            card_index_img: "".to_string(),
            card_list: vec![],
            variation_num: 6,
            acquire_num: 0,
            lv: 1,
            is_married: None,
            married_img: None,
        };

        use BookShipCardPageSource::*;
        assert_eq!(ship.source(0), Normal);
        assert_eq!(ship.source(1), SundayBest);
        assert_eq!(ship.source(2), Unknown);
    }
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShipCardPage {
    priority: u16,
    card_img_list: Vec<String>,
    status_img: Option<Vec<String>>,
    variation_num_in_page: u16,
    acquire_num_in_page: u16,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintList(Vec<BlueprintShip>);

impl BlueprintList {
    /// Parses a BlueprintList from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(blueprintlist_reader: impl Read) -> Result<BlueprintList> {
        let result: BlueprintList = serde_json::from_reader(blueprintlist_reader)?;
        Ok(result)
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for BlueprintList {
    type Target = Vec<BlueprintShip>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Notes for future functions
// * Status images (i/i_xxx.png) live in https://kancolle-arcade.net/ac/resources/chara/
// ** Status images end with _n, _bs, _bm, or _bl. (Not sure if there's one for sunk?)
// * Expiration date appears to be the 11th of the month of expiry. Not clear why.
// ** True expiration date is 23:59 on the last date of the month.
// ** Or I made a mistake, I guess?

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BlueprintShip {
    ship_class_id: u16,
    ship_class_index: u16,
    ship_sort_no: u16,
    ship_type: String,
    ship_name: String,
    status_img: String,
    blueprint_total_num: u16,
    exists_warning_for_expiration: bool,
    expiration_date_list: Vec<BlueprintExpirationDate>,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BlueprintExpirationDate {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    expiration_date: DateTime<Utc>,
    blueprint_num: u16,
    expire_this_month: bool,
}
