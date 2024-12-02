use anyhow::Result;
use kancolle_a::ships::{ShipMod, ShipsBuilder};
use kancolle_a_cli_tools::cli_helpers;

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
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
    enum Status {
        ReadyFor,
        SavingFor,
        Complete,
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
    // I doubt there's anywhere machine-readable for that data.

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
            Some(shipmod) => match ship.shipmod_blueprint_cost(shipmod.remodel_level()) {
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
            },
            None => {
                let last_known_shipmod = ship.mods().last().unwrap();
                // Asserting this because we used to guess for unknown ships, but
                // the current API doesn't support that; also data coverage is now
                // good enough that guessing is more-likely wrong than right.
                assert!(ship
                    .shipmod_blueprint_cost(last_known_shipmod.remodel_level() + 1)
                    .is_none());
                ship_status.push((
                    last_known_shipmod.name(),
                    blueprint.blueprint_total_num,
                    0,
                    Status::Complete,
                ));
                continue;
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
