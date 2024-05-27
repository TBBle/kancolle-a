//! Module for importers for https://kancolle-arcade.net/ac/api/ resources

use serde::Deserialize;
use serde_json::Result;
use std::ops::Deref;

#[derive(Debug, Deserialize)]
pub struct TcBook(Vec<BookShip>);

// TODO: What's the easy way to make this read-only outside this module?
impl TcBook {
    /// Parses a TcBook from the provided JSON string.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(tcbook_json: &str) -> Result<TcBook> {
        let result: TcBook = serde_json::from_str(tcbook_json)?;
        Ok(result)
    }
}

impl Deref for TcBook {
    type Target = Vec<BookShip>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShip {
    pub book_no: u16,
    pub ship_class: Option<String>,
    pub ship_class_index: Option<i16>,
    pub ship_type: String,
    pub ship_model_num: String,
    pub ship_name: String,
    pub card_index_img: String,
    pub card_list: Vec<BookShipCardPage>,
    pub variation_num: u16,
    pub acquire_num: u16,
    pub lv: u16,
    pub is_married: Option<Vec<bool>>,
    pub married_img: Option<Vec<String>>, // Probably... No married ships
}

// Notes for future functions
// * Card images (s/tc_NO_xxx.jpg) live in https://kancolle-arcade.net/ac/resources/pictureBook/
// * Status images (i/i_xxx.png) live in https://kancolle-arcade.net/ac/resources/chara/
// ** Status images end with _n, _

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShipCardPage {
    pub priority: u16,
    pub card_img_list: Vec<String>,
    pub status_img: Option<Vec<String>>,
    pub variation_num_in_page: u16,
    pub acquire_num_in_page: u16,
}
