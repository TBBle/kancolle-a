//! Module for importer for https://kancolle-arcade.net/ac/api/BlueprintList/info

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Result;
use std::io::Read;

type BlueprintList = Vec<BlueprintShip>;

/// Parses a BlueprintList from the provided JSON reader.
/// Fails if not given a JSON array, or expected data structure does not match.
pub(crate) fn read_blueprintlist(blueprintlist_reader: impl Read) -> Result<BlueprintList> {
    let result: BlueprintList = serde_json::from_reader(blueprintlist_reader)?;
    Ok(result)
}

// Notes for future functions
// * Status images (i/i_xxx.png) live in https://kancolle-arcade.net/ac/resources/chara/
// ** Status images end with _n, _bs, _bm, or _bl. (Not sure if there's one for sunk?)
// * Expiration date appears to be the 11th of the month of expiry. Not clear why.
// ** True expiration date is 23:59 on the last date of the month.
// ** Or I made a mistake, I guess?

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BlueprintShip {
    pub ship_class_id: u16,
    pub ship_class_index: u16,
    pub ship_sort_no: u16,
    pub ship_type: String,
    pub ship_name: String,
    pub status_img: String,
    pub blueprint_total_num: u16,
    pub exists_warning_for_expiration: bool,
    pub expiration_date_list: Vec<BlueprintExpirationDate>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BlueprintExpirationDate {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub expiration_date: DateTime<Utc>,
    pub blueprint_num: u16,
    pub expire_this_month: bool,
}

#[cfg(test)]
mod tests;
