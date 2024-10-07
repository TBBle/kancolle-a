use anyhow::Result;
use kancolle_a::ships::{ShipMod, ShipsBuilder};
use kancolle_a_cli_tools::cli_helpers;

/// Report the number of blueprints and large-scale blueprints needed for each stage.
/// `stage` is 0-indexed, i.e. it's the cost to upgrade _from_ that level.
/// May report 改三 stage for ships that don't have one, i.e. use it only for ships that actually exist.
/// This should probably become an internal utility function in the library.
fn ship_blueprint_costs(ship_name: &str, ship_type: &str, stage: u16) -> Option<(u16, u8)> {
    // Special ships.
    let stage_costs = match ship_name {
        // TODO: This is shipClassId 5 (ship class name 千歳型). The id is available in blueprints, and the name
        // is available in Character, Book, and Wiki data. Could we rely on always having at least one?
        // Notably, kekkon list does not list ship class information at all; can we assume kekkon is a subset of the wiki?
        // How _reliable_ is the Wiki here? Conversely, since we're matching _blueprint_ names, name-matching
        // is probably safest, but requires maintenance.
        "千歳" | "千代田" => vec![(3, 0), (4, 0), (5, 0), (6, 0), (8, 2)],
        // TODO: Wiki lists base as 春日丸級 and mods as 大鷹型; need to find shipClassId to
        // Confirm that this forms an actual ship series: 春日丸, 大鷹, 大鷹改.
        "春日丸" => vec![(3, 0), (5, 0)],
        _ => match ship_type {
            "駆逐艦" | "軽巡洋艦" | "潜水艦" => vec![(3, 0), (6, 1), (6, 3)],
            // TODO: As of 2024/9/26, 最上改二 and 最上改二特 are buildable, so need to get their numbers.
            "重巡洋艦" => vec![(3, 0), (8, 2)],
            "戦艦" | "軽空母" | "正規空母" => vec![(3, 0), (8, 2), (8, 4)],
            _ => vec![(3, 0)],
        },
    };
    return stage_costs
        .get(stage as usize)
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = args::options().run();

    let ships = cli_helpers::ship_source_data_applier(&args.data, ShipsBuilder::default())?
        .build()
        .await?;

    /// Status of the ship chain based on current blueprint inventory
    /// The Maybe values will go away once we have a list of all known ships.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
    enum Status {
        ReadyFor,
        MaybeReady, // Could build the next level if one exists (check manually)
        SavingFor,
        MaybeSaving, // Could not build the next level.
        Complete,    // No next level possible (no upgrade cost).
    }

    // Ship name, current count, needed for next level, status
    let mut ship_status: Vec<(&str, u16, u16, Status)> = vec![];

    let has_normal_card = |shipmod: &ShipMod| match shipmod.book() {
        None => false,
        Some(book_ship) => !book_ship.card_list[0].card_img_list[0].is_empty(),
    };

    // Reference: https://wikiwiki.jp/kancolle-a/%E5%BB%BA%E9%80%A0#kaizou
    // Does not take into account ships that have just been released in events and hence are not
    // yet upgradable. (At time of writing, 最上改二/最上改二特 and 武蔵改二 are examples of this.)
    // I doubt there's anywhere useful for that data.

    // NOTE: This loop will prefer to save for a level we don't have a card for, versus
    // backfilling a level we can afford now. That's usually what you want, but sometimes
    // you want to know what you can do _now_, e.g., when blueprints are expiring.
    // TODO: Represent both cases somehow. I could include any such match in the output, rather than
    // only taking one-per-Ship.
    for (ship, blueprint) in ships
        .iter()
        .filter_map(|(_, ship)| ship.blueprint().as_ref().map(|blueprint| (ship, blueprint)))
    {
        match ship
            .mods()
            .iter()
            .rev()
            .find(|shipmod| shipmod.remodel_level() > 0 && !has_normal_card(shipmod))
        {
            Some(shipmod) => {
                match ship_blueprint_costs(
                    &blueprint.ship_name,
                    &blueprint.ship_type,
                    shipmod.remodel_level() - 1,
                ) {
                    None => {
                        ship_status.push((
                            shipmod.name(),
                            blueprint.blueprint_total_num,
                            0,
                            Status::Complete,
                        ));
                        continue;
                    }
                    Some((bp_cost, _)) if bp_cost > blueprint.blueprint_total_num => {
                        ship_status.push((
                            shipmod.name(),
                            blueprint.blueprint_total_num,
                            bp_cost,
                            Status::SavingFor,
                        ));
                        continue;
                    }
                    Some((bp_cost, _)) => {
                        ship_status.push((
                            shipmod.name(),
                            blueprint.blueprint_total_num,
                            bp_cost,
                            Status::ReadyFor,
                        ));
                        continue;
                    }
                }
            }
            None => {
                let last_known_shipmod = ship.mods().last().unwrap();
                match ship_blueprint_costs(
                    &blueprint.ship_name,
                    &blueprint.ship_type,
                    last_known_shipmod.remodel_level(),
                ) {
                    None => {
                        ship_status.push((
                            last_known_shipmod.name(),
                            blueprint.blueprint_total_num,
                            0,
                            Status::Complete,
                        ));
                        continue;
                    }
                    Some((bp_cost, _)) if bp_cost > blueprint.blueprint_total_num => {
                        ship_status.push((
                            last_known_shipmod.name(),
                            blueprint.blueprint_total_num,
                            bp_cost,
                            Status::MaybeSaving,
                        ));
                        continue;
                    }
                    Some((bp_cost, _)) => {
                        ship_status.push((
                            last_known_shipmod.name(),
                            blueprint.blueprint_total_num,
                            bp_cost,
                            Status::MaybeReady,
                        ));
                        continue;
                    }
                }
            }
        }
    }

    // TODO: Ships and ship_mods can supply a sort key if they have a character.
    // For now, sort by name and then by status.
    ship_status.sort_by_key(|(ship_name, _, _, _)| *ship_name);

    ship_status.sort_by_key(|(_, _, _, status)| *status);

    for (ship_name, bp_count, bp_needed, status) in ship_status {
        println!("{status:?}:\t{ship_name}\t{bp_count}/{bp_needed}");
    }

    Ok(())
}
