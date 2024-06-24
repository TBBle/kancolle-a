//! Module for importer for https://kancolle-arcade.net/ac/api/BlueprintList/info

use chrono::{DateTime, Utc};
use derive_getters::Getters;
use serde::Deserialize;
use serde_json::Result;
use std::{io::Read, ops::Deref};

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
