use std::cmp::Ordering;

use anyhow::Result;
use kancolle_a::ships::{self, ShipMod, ShipsBuilder};
use kancolle_a_cli_tools::cli_helpers;

/// Report the number of blueprints and large-scale blueprints needed for each stage.
/// `stage` is 0-indexed, i.e. it's the cost to upgrade _from_ that level.
/// May report 改三 stage for ships that don't have one, i.e. use it only for ships that actually exist.
/// This should probably become an internal utility function in the library.
/// TODO: Move into Ship or ShipMod with a better API.
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
            "戦艦" | "軽空母" | "正規空母" | "重巡洋艦" => {
                vec![(3, 0), (8, 2), (8, 4)]
            }
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
        pub(crate) char_only: bool,
        pub(crate) ship_names: Vec<String>,
    }

    pub fn options() -> OptionParser<Options> {
        let data = cli_helpers::ship_source_data_parser();
        let char_only = long("character-only")
            .help("Don't consider missing normal card as missing")
            .switch();
        let ship_names = positional("SHIP")
            .help("Ships to filter the search by")
            .many();
        construct!(Options {
            data,
            char_only,
            ship_names
        })
        .to_options()
        .descr("A tool to report on renovation priorities for your collection.")
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

    let ship_names: Vec<&str> = args
        .ship_names
        .iter()
        .map(|ship_name| ships::ship_blueprint_name(ship_name))
        .collect();

    // Plan: Report ShipMods in the following states, in order:
    // * Ships with no characters or missing base character: Need drops
    // * Ships with uncollected modified characters, sufficient blueprints to construct
    // * Ships with uncollected modified characters, top mod not yet 5 stars
    // * Ships with uncollected modified characters, top mod 5 stars
    // * Ships with no uncollected modified charactes, top mod not yet 5 stars

    pub enum State<'a> {
        MissingAll(&'a String),
        MissingBase(&'a String),
        Constructable {
            ship_name: &'a String,
            current: &'a ShipMod,
            next: &'a ShipMod,
        },
        UpgradeAvailable {
            ship_name: &'a String,
            current: &'a ShipMod,
            next: &'a ShipMod,
        },
        UpgradeReady {
            ship_name: &'a String,
            current: &'a ShipMod,
            next: &'a ShipMod,
        },
        StarsNeeded {
            ship_name: &'a String,
            current: &'a ShipMod,
        },
    }

    let mut results: Vec<State> = Vec::new();

    // No character entry, no book entry, or Normal page Normal card is missing.
    let missing = if args.char_only {
        |ship_mod: &ShipMod| ship_mod.character().is_none()
    } else {
        |ship_mod: &ShipMod| {
            ship_mod.character().is_none()
                || ship_mod.book().is_none()
                || ship_mod
                    .book()
                    .as_ref()
                    .unwrap()
                    .card_list
                    .first()
                    .unwrap()
                    .card_img_list[0]
                    .is_empty()
        }
    };

    for (ship_name, ship) in ships
        .iter()
        .filter(|(ship_name, _)| ship_names.is_empty() || ship_names.contains(&ship_name.as_str()))
    {
        if ship.mods().is_empty() || ship.mods().iter().all(missing) {
            results.push(State::MissingAll(ship_name));
            continue;
        }

        if missing(ship.mods().first().unwrap()) {
            results.push(State::MissingBase(ship_name));
        }

        for i in 0..ship.mods().len() - 1 {
            let ship_mod_pair = &ship.mods()[i..=i + 1];
            let (current, next) = (&ship_mod_pair[0], &ship_mod_pair[1]);
            if current.character().is_none() || !missing(next) {
                continue;
            }

            if ship.blueprint().is_some()
                && ship.blueprint().as_ref().unwrap().blueprint_total_num
                    >= ship_blueprint_costs(
                        &ship.blueprint().as_ref().unwrap().ship_name,
                        &ship.blueprint().as_ref().unwrap().ship_type,
                        current.remodel_level(),
                    )
                    .unwrap_or((99, 99))
                    .0
            {
                results.push(State::Constructable {
                    ship_name,
                    current,
                    next,
                });
            } else if current.character().as_ref().unwrap().star_num == 5 {
                results.push(State::UpgradeReady {
                    ship_name,
                    current,
                    next,
                });
            } else {
                results.push(State::UpgradeAvailable {
                    ship_name,
                    current,
                    next,
                });
            }
        }

        let last_ship = ship.mods().last().unwrap();

        if last_ship.character().is_some() && last_ship.character().as_ref().unwrap().star_num < 5 {
            results.push(State::StarsNeeded {
                ship_name,
                current: ship.mods().last().unwrap(),
            });
        }
    }

    results.sort_unstable_by(|left, right| match left {
        State::MissingAll(_) => match right {
            State::MissingAll(_) => Ordering::Equal,
            _ => Ordering::Greater,
        },
        State::MissingBase(_) => match right {
            State::MissingAll(_) => Ordering::Less,
            State::MissingBase(_) => Ordering::Equal,
            _ => Ordering::Greater,
        },
        State::Constructable { .. } => match right {
            State::MissingAll(_) | State::MissingBase(_) => Ordering::Less,
            State::Constructable { .. } => Ordering::Equal,
            _ => Ordering::Greater,
        },

        State::UpgradeAvailable {
            current: left_curr, ..
        } => match right {
            State::MissingAll(_) | State::MissingBase(_) | State::Constructable { .. } => {
                Ordering::Less
            }
            State::UpgradeAvailable {
                current: right_curr,
                ..
            } => left_curr
                .character()
                .as_ref()
                .unwrap()
                .star_num
                .cmp(&right_curr.character().as_ref().unwrap().star_num),
            _ => Ordering::Greater,
        },
        State::UpgradeReady { .. } => match right {
            State::MissingAll(_)
            | State::MissingBase(_)
            | State::Constructable { .. }
            | State::UpgradeAvailable { .. } => Ordering::Less,
            State::UpgradeReady { .. } => Ordering::Equal,
            _ => Ordering::Greater,
        },
        State::StarsNeeded {
            current: left_curr, ..
        } => match right {
            State::StarsNeeded {
                current: right_curr,
                ..
            } => left_curr
                .character()
                .as_ref()
                .unwrap()
                .star_num
                .cmp(&right_curr.character().as_ref().unwrap().star_num),
            _ => Ordering::Less,
        },
    });

    for result in results {
        match result {
            State::MissingAll(ship_name) => eprintln!("MISSING ALL\t{ship_name}"),
            State::MissingBase(ship_name) => eprintln!("MISSING BASE\t{ship_name}"),
            State::Constructable {
                ship_name,
                current,
                next,
            } => {
                let current_name = current.name();
                let next_name = next.name();
                let current_stars = current.character().as_ref().unwrap().star_num;
                eprintln!(
                    "CONSTRUCTABLE\t{ship_name}\t{current_name}({current_stars}/5)\t=> {next_name}"
                );
            }
            State::UpgradeAvailable {
                ship_name,
                current,
                next,
            } => {
                let current_name = current.name();
                let next_name = next.name();
                let current_stars = current.character().as_ref().unwrap().star_num;
                eprintln!(
                    "AVAILABLE\t{ship_name}\t{current_name}({current_stars}/5)\t=> {next_name}"
                );
            }
            State::UpgradeReady {
                ship_name,
                current,
                next,
            } => {
                let current_name = current.name();
                let next_name = next.name();
                eprintln!("READY\t\t{ship_name}\t{current_name}      \t=> {next_name}");
            }
            State::StarsNeeded { ship_name, current } => {
                let current_name = current.name();
                let current_stars = current.character().as_ref().unwrap().star_num;
                eprintln!("STARS NEEDED\t{ship_name}\t{current_name}({current_stars}/5)");
            }
        }
    }

    Ok(())
}
