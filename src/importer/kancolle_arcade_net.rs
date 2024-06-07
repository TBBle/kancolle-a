//! Module for importers for https://kancolle-arcade.net/ac/api/ resources

use chrono::{DateTime, Utc};
use derive_getters::Getters;
use serde::Deserialize;
use serde_json::Result;
use std::{io::Read, ops::Deref};

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
