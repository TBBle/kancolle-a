use kancolle_a::ships::ShipsBuilder;
use kancolle_a_cli_tools::cli_helpers;
use std::{collections::HashMap, error::Error};

/// Report the number of blueprints and large-scale blueprints needed for each stage.
/// `stage` is 0-indexed, i.e. it's the cost to upgrade _from_ that level.
/// May report 改三 stage for ships that don't have one, i.e. use it only for ships that actually exist.
/// This should probably become an internal utility function in the library.
fn ship_blueprint_costs(ship_name: &str, ship_type: &str, stage: usize) -> Option<(u16, u8)> {
    // Special ships.
    let stage_costs = match ship_name {
        // TODO: This is shipClassId 5 (千歳型), better than name-matching?
        // That said, the only data we can be fairly sure of having is name and (base?) ship type.
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
        .map(|costs| (costs.0 as u16, costs.1 as u8));
}

pub(crate) mod args {
    use bpaf::*;
    use kancolle_a_cli_tools::cli_helpers::{self, ShipSourceDataOptions};

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) data: ShipSourceDataOptions,
    }

    pub fn options() -> OptionParser<Options> {
        let data = cli_helpers::ship_source_data_parser();
        construct!(Options { data })
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

    let ships =
        cli_helpers::ship_source_data_applier(&args.data, ShipsBuilder::default())?.build()?;

    // Build card chain lists. This should probably be in the library somewhere.

    // The bool is tracking if we actually have the unadorned card for this modification level...
    // This part probably wouldn't go in the library, instead it'd point back to the Book number and row?
    let mut card_chains: HashMap<String, Vec<(String, bool)>> = HashMap::new();

    for (ship_name, ship) in ships.iter() {
        if ship.blueprint().is_none() {
            continue;
        }

        let has_normal_card = match ship.book() {
            None => false,
            Some(bookship) => {
                if *ship.book_secondrow() {
                    !bookship.card_list[0].card_img_list[3].is_empty()
                } else {
                    !bookship.card_list[0].card_img_list[0].is_empty()
                }
            }
        };

        let base_name = ship.blueprint().as_ref().unwrap().ship_name.to_string();

        card_chains
            .entry(base_name)
            .or_default()
            .push((ship_name.clone(), has_normal_card));
    }

    // TODO: Need a better sort algorithm when we move this into the library.
    for (_, chain) in card_chains.iter_mut() {
        chain.sort_unstable_by_key(|(ship_name, _)| match ships[ship_name].book().as_ref() {
            None => 10000, // HACK: Put 'em last, at this point order between them doesn't matter.
            Some(book) => {
                book.book_no * 10
                    + if *ships[ship_name].book_secondrow() {
                        5
                    } else {
                        0
                    }
            }
        });
    }

    // Demutable it.
    let card_chains = card_chains;

    /// Status of the ship chain based on current blueprint inventory
    /// The Maybe values will go away once we have a list of all known ships.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
    enum Status {
        MissingBase, // Base ship somehow not owned.
        ReadyFor,
        MaybeReady, // Could build the next level if one exists (check manually)
        SavingFor,
        MaybeSaving, // Could not build the next level.
        Complete,    // No next level possible (no upgrade cost).
    }

    // Ship name, current count, needed for next level, status
    let mut ship_status: Vec<(&str, u16, u16, Status)> = vec![];

    // Reference: https://wikiwiki.jp/kancolle-a/%E5%BB%BA%E9%80%A0#kaizou
    // Does not take into account ships that have just been released in events and hence are not
    // yet upgradable. (At time of writing, 最上改二/最上改二特 and 武蔵改二 are examples of this.)
    // I doubt there's anywhere useful for that data.
    // BUG: This will prefer filling in a normal card we can afford, over saving for a level we don't have.
    // I actually need to report both, somehow.
    for (base_name, chain) in card_chains.iter() {
        let bp_ship = ships[base_name].blueprint().as_ref().unwrap();

        let first_missing = chain.iter().enumerate().find(|(_, (_, owned))| !*owned);

        if let Some((stage, (ship_name, _))) = first_missing {
            if stage == 0 {
                ship_status.push((
                    ship_name,
                    bp_ship.blueprint_total_num,
                    0,
                    Status::MissingBase,
                ));
            } else {
                let bp_needed = ship_blueprint_costs(base_name, &bp_ship.ship_type, stage - 1)
                    .unwrap()
                    .0;
                ship_status.push((
                    ship_name,
                    bp_ship.blueprint_total_num,
                    bp_needed,
                    if bp_ship.blueprint_total_num >= bp_needed {
                        Status::ReadyFor
                    } else {
                        Status::SavingFor
                    },
                ));
            }
        } else {
            let (last_stage, (last_ship, _)) = chain.iter().enumerate().last().unwrap();
            if let Some((bp_needed, _)) =
                ship_blueprint_costs(base_name, &bp_ship.ship_type, last_stage)
            {
                ship_status.push((
                    last_ship,
                    bp_ship.blueprint_total_num,
                    bp_needed,
                    if bp_ship.blueprint_total_num >= bp_needed {
                        Status::MaybeReady
                    } else {
                        Status::MaybeSaving
                    },
                ));
            } else {
                ship_status.push((
                    last_ship,
                    bp_ship.blueprint_total_num,
                    ship_blueprint_costs(&bp_ship.ship_name, &bp_ship.ship_type, last_stage - 1)
                        .unwrap_or((0, 0))
                        .0,
                    Status::Complete,
                ));
            }
        }
    }

    ship_status.sort_by_key(|(_, _, _, status)| *status);

    for (ship_name, bp_count, bp_needed, status) in ship_status {
        println!("{status:?}:\t{ship_name}\t{bp_count}/{bp_needed}");
    }

    Ok(())
}
