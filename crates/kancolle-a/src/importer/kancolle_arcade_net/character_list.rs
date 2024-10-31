//! Module for importer for https://kancolle-arcade.net/ac/api/CharacterList/info

use serde::Deserialize;
use serde_json::Result;
use std::io::Read;

type CharacterList = Vec<Character>;

/// Parses a CharacterList from the provided JSON reader.
/// Fails if not given a JSON array, or expected data structure does not match.
pub(crate) fn read_characterlist(characterlist_reader: impl Read) -> Result<CharacterList> {
    let result: CharacterList = serde_json::from_reader(characterlist_reader)?;
    Ok(result)
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub book_no: u16,
    pub lv: u16,
    pub ship_type: String,
    pub ship_sort_no: u16,
    pub remodel_lv: u16,
    pub ship_name: String,
    pub status_img: String,
    pub star_num: u16,
    pub ship_class: Option<String>,
    pub ship_class_index: Option<u16>,
    pub tc_img: String,
    pub exp_percent: u16,
    pub max_hp: u16,
    pub real_hp: u16,
    pub damage_status: String, // How do I enum this?
    pub slot_num: u16,
    pub slot_equip_name: Vec<String>,
    pub slot_amount: Vec<u16>,
    pub slot_disp: Vec<String>, // How do I enum this?
    pub slot_img: Vec<String>,
    pub slot_extension: Vec<bool>,
    pub blueprint_total_num: u16,
    pub married: bool,
    pub disp_sort_no: u64, // This is really an encoded structure...
    pub develop_equipment_list: Vec<DevelopEquipment>,
    pub ship_model_num: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DevelopEquipment {
    pub plan_kind: u16,
    pub sort_index: u16,
    pub require_lv: u16,
    pub require_strategy_point: u16,
    pub require_material_medal: u16,
    pub develop_count: u16,
    pub max_develop_count: u16,
    pub develop_equip_img: String,
}

#[cfg(test)]
mod tests;
