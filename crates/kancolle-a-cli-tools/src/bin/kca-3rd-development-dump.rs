use std::{cmp::max, collections::HashMap};

use anyhow::Result;
use kancolle_a::{importer::kancolle_arcade_net::Character, ships::ShipsBuilder};
use kancolle_a_cli_tools::cli_helpers;

pub(crate) mod args {
    use bpaf::*;
    use kancolle_a_cli_tools::cli_helpers::{self, ShipSourceDataOptions};

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) data: ShipSourceDataOptions,
        // pub(crate) insufficient_level: bool,
        // pub(crate) fully_developed: bool,
    }

    pub fn options() -> OptionParser<Options> {
        let data = cli_helpers::ship_source_data_parser();
        // let insufficient_level = long("insufficient-level")
        //     .help("Show equipment where no character has the necessary level")
        //     .switch();
        // let fully_developed = long("fully-developed")
        //     .help("Show equipment where all possible instances have been developed")
        //     .switch();
        construct!(Options {
            data,
            // insufficient_level,
            // fully_developed
        })
        .to_options()
        .descr("A tool to report on 3rd Development equipment.")
    }

    #[test]
    fn kca_3rd_development_dump_check_options() {
        options().check_invariants(false)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = args::options().run();

    let ships = cli_helpers::ship_source_data_applier(&args.data, ShipsBuilder::default())?
        .build()
        .await?;

    let mut equipment_to_character: HashMap<String, Vec<(&Character, usize)>> = HashMap::new();

    for ship_mod in ships
        .shipmod_iter()
        .filter(|ship_mod| ship_mod.character().is_some())
    {
        let character = ship_mod.character().as_ref().unwrap();
        let develop_equipment_list = &character.develop_equipment_list;
        for (index, develop_equip) in develop_equipment_list.iter().enumerate() {
            equipment_to_character
                .entry(develop_equip.develop_equip_img.clone())
                .or_default()
                .push((character, index));
        }
    }

    fn get_development_equipment(
        char: &Character,
        index: usize,
    ) -> &kancolle_a::importer::kancolle_arcade_net::DevelopEquipment {
        &char.develop_equipment_list[index]
    }

    // Sanity check that all properties except requireLv, developCount, and maxDevelopCount are constant
    let mut max_sort_index: u16 = 0;
    for (_, chars) in equipment_to_character.iter() {
        let mut entries = chars.iter();
        let (first_char, first_index) = entries.next().unwrap().to_owned();
        let first_development_equipment = get_development_equipment(first_char, first_index);
        max_sort_index = max(max_sort_index, first_development_equipment.sort_index);
        for (char, index) in entries.map(|x| x.to_owned()) {
            let development_equipment = get_development_equipment(char, index);
            assert_eq!(
                first_development_equipment.plan_kind,
                development_equipment.plan_kind
            );
            assert_eq!(
                first_development_equipment.sort_index,
                development_equipment.sort_index
            );
            assert_eq!(
                first_development_equipment.require_strategy_point,
                development_equipment.require_strategy_point
            );
            assert_eq!(
                first_development_equipment.require_material_medal,
                development_equipment.require_material_medal
            );
            assert_eq!(
                first_development_equipment.develop_equip_img,
                development_equipment.develop_equip_img
            );
        }
    }

    // Well, we only have images to go from, so I guess we're dumping HTML?
    // TODO: Implement EquipBook parser, to lookup details from the image name.
    // TODO: Use a non-HashMap, and sort by sort_index and... equipment name?

    println!(
        "<html><head><base href='https://kancolle-arcade.net/ac/resources/pictureBook/'/></head>"
    );
    println!("<body><table>");

    let mut remaining = equipment_to_character.len();
    let mut current_sort_index: u16 = 0;
    while current_sort_index <= max_sort_index {
        for (develop_equip_img, chars) in equipment_to_character.iter().filter(|(_, chars)| {
            get_development_equipment(chars.first().unwrap().0, chars.first().unwrap().1).sort_index
                == current_sort_index
        }) {
            assert!(remaining > 0);
            remaining -= 1;

            let mut shown_cost = false;

            for (char, index) in chars.iter().copied() {
                print!("<tr>");
                let development_equipment = get_development_equipment(char, index);
                if !shown_cost {
                    let rowspan = chars.len();
                    let sp = development_equipment.require_strategy_point;
                    let medal = development_equipment.require_material_medal;
                    print!("\t<td rowspan={rowspan}><img src='{develop_equip_img}'/></td><td rowspan={rowspan}>{sp}</td><td rowspan={rowspan}>{medal}</td>");
                    shown_cost = true;
                }
                let char_name = &char.ship_name;
                let char_level = char.lv;
                let dev_level = development_equipment.require_lv;
                let dev_count = development_equipment.develop_count;
                let dev_max = development_equipment.max_develop_count;
                print!("<td>{char_name}</td><td>{char_level}</td><td>{dev_level}</td><td>{dev_count}</td><td>{dev_max}</td>");
                println!("</tr>");
            }
        }
        current_sort_index += 1;
    }
    println!("</table></body></html>");

    assert_eq!(remaining, 0);

    Ok(())
}
