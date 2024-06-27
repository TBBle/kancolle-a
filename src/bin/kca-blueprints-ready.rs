use kancolle_a::importer::kancolle_arcade_net::{BlueprintList, TcBook};
use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, error::Error};

/// Determine the blueprint/unmodified ship name for the given ship
/// This should probably become an internal utility function in the library.
fn ship_blueprint_name(ship_name: &str) -> &str {
    // Base case: Ships
    let base_name = if let Some(split_name) = ship_name.split_once("改") {
        split_name.0
    } else {
        ship_name
    };

    // Ships that are renamed in modification, or have non-改 variants.
    match base_name {
        // Tested against data
        "龍鳳" => "大鯨",
        "Верный" => "響",
        "Italia" => "Littorio",
        // Untested against data as I don't own them.
        "呂500" => "U-511",
        "千代田甲" | "千代田航" => "千代田",
        "千歳甲" | "千歳航" => "千歳",
        "Октябрьская революция" => "Гангут",
        "大鷹" => "春日丸",
        _ => base_name,
    }
}

/// Report the number of blueprints and large-scale blueprints needed for each stage.
/// `stage` is 0-indexed, i.e. it's the cost to upgrade _from_ that level.
/// May report 改三 stage for ships that don't have one, i.e. use it only for ships that actually exist.
/// This should probably become an internal utility function in the library.
fn ship_blueprint_costs(ship_name: &str, ship_type: &str, stage: usize) -> Option<(u16, u8)> {
    // Special ships.
    let stage_costs = match ship_name {
        // 千歳型, might be shipClassId 5, but I have no 千代田 blueprints to verify.
        "千歳" | "千代田" => vec![(3, 0), (4, 0), (5, 0), (6, 0), (8, 2)],
        // Called 大鷹型 which is actually the first kai level on the wiki, need to find shipClassId
        // TODO: Confirm that this forms an actual ship series: 春日丸, 大鷹, 大鷹改
        "春日丸" => vec![(3, 0), (5, 0)],
        _ => match ship_type {
            "駆逐艦" | "軽巡洋艦" | "潜水艦" => vec![(3, 0), (6, 1), (6, 3)],
            "重巡洋艦" => vec![(3, 0), (8, 2)],
            "戦艦" | "軽空母" | "正規空母" => vec![(3, 0), (8, 2), (8, 4)],
            _ => vec![(3, 0)],
        },
    };
    return stage_costs
        .get(stage)
        .and_then(|costs| Some((costs.0 as u16, costs.1 as u8)));
}

pub(crate) mod args {
    use std::path::PathBuf;

    use bpaf::*;
    use kancolle_a::cli_helpers;

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) tcbook: PathBuf,
        pub(crate) bplist: PathBuf,
    }

    pub fn options() -> OptionParser<Options> {
        let tcbook = cli_helpers::tcbook_path_parser();
        let bplist = cli_helpers::bplist_path_parser();
        construct!(Options { tcbook, bplist })
            .to_options()
            .descr("A tool to report on blueprint status for your collection.")
    }

    #[test]
    fn kca_blueprints_ready_check_options() {
        options().check_invariants(false)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();
    let tc_path = args.tcbook;

    let tc_data = BufReader::new(File::open(tc_path)?);

    let tc_list = TcBook::new(tc_data)?;

    let bp_path = args.bplist;

    let bp_data = BufReader::new(File::open(bp_path)?);

    let bp_list = BlueprintList::new(bp_data)?;

    // Build card chain lists. This should probably be in the library somewhere, but
    // waiting until I am parsing all known ships off the Wiki data, rather than just
    // your unlocks in the TCBook.

    // The bool is tracking if we actually have the unadorned card for this modification level...
    // This part probably wouldn't go in the library, instead it'd point back to the Book number and row?
    let mut card_chains: HashMap<String, Vec<(String, bool)>> = HashMap::new();

    // TODO: Owned ships iterator? This seems like a common pattern.
    // Actually, to deal with the two-row ships, perhaps a flattening iterator?
    // Again, that will make more sense when parsing data from the wiki as our primary ship list source.
    for ship in tc_list.iter().filter(|ship| *ship.acquire_num() > 0) {
        let normal_page = &ship.card_list()[0];
        let base_name = ship_blueprint_name(ship.ship_name());
        let ships = if normal_page.card_img_list().len() == 3 {
            vec![(
                ship.ship_name().clone(),
                !normal_page.card_img_list()[0].is_empty(),
            )]
        } else {
            // AFAIK _this_ claim should always be true, renames have a new card number.
            // Once reading from the Wiki, this can be validated from
            // https://wikiwiki.jp/kancolle-a/%E6%94%B9%E9%80%A0%E8%89%A6%E8%88%B9#table
            // Manual inspection at time-of-writing says it's true.
            vec![
                (
                    ship.ship_name().clone(),
                    !normal_page.card_img_list()[0].is_empty(),
                ),
                (
                    ship.ship_name().clone() + "改",
                    !normal_page.card_img_list()[3].is_empty(),
                ),
            ]
        };

        card_chains
            .entry(base_name.to_string())
            .or_insert(vec![])
            .extend(ships);
    }

    // Demutable it.
    let card_chains = card_chains;

    /// Status of the ship chain based on current blueprint inventory
    /// The Maybe values will go away once we have a list of all known ships.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
    enum Status {
        MissingBase, // Base ship somehow not owned.
        Ready,
        MaybeReady, // Could build the next level if one exists (check manually)
        Saving,
        MaybeSaving, // Could not build the next level.
        Complete,    // No next level possible (no upgrade cost).
    }

    let mut ship_status: Vec<(&str, u16, Status)> = vec![];

    // Reference: https://wikiwiki.jp/kancolle-a/%E5%BB%BA%E9%80%A0#kaizou
    // Does not take into account ships that have just been released in events and hence are not
    // yet upgradable. (At time of writing, 最上改二/最上改二特 and 武蔵改二 are examples of this.)
    // I doubt there's anywhere useful for that data.
    // BUG: This will prefer filling in a normal card we can afford, over saving for a level we don't have.
    for bp_ship in bp_list.iter() {
        let chain = &card_chains[bp_ship.ship_name()];

        let first_missing = chain.iter().enumerate().find(|(_, (_, owned))| !*owned);

        if let Some((stage, (ship_name, _))) = first_missing {
            if stage == 0 {
                ship_status.push((
                    ship_name,
                    *bp_ship.blueprint_total_num(),
                    Status::MissingBase,
                ));
            } else {
                let bp_needed =
                    ship_blueprint_costs(bp_ship.ship_name(), bp_ship.ship_type(), stage - 1)
                        .unwrap()
                        .0;
                ship_status.push((
                    ship_name,
                    *bp_ship.blueprint_total_num(),
                    if *bp_ship.blueprint_total_num() >= bp_needed {
                        Status::Ready
                    } else {
                        Status::Saving
                    },
                ));
            }
        } else {
            let (last_stage, (last_ship, _)) = chain.iter().enumerate().last().unwrap();
            if let Some((bp_needed, _)) =
                ship_blueprint_costs(bp_ship.ship_name(), bp_ship.ship_type(), last_stage)
            {
                ship_status.push((
                    last_ship,
                    *bp_ship.blueprint_total_num(),
                    if *bp_ship.blueprint_total_num() >= bp_needed {
                        Status::MaybeReady
                    } else {
                        Status::MaybeSaving
                    },
                ));
            } else {
                ship_status.push((last_ship, *bp_ship.blueprint_total_num(), Status::Complete));
            }
        }
    }

    ship_status.sort_by_key(|(_, _, status)| *status);

    for (ship_name, bp_count, status) in ship_status {
        println!("{status:?}:\t{ship_name}\t{bp_count}");
    }

    Ok(())
}
