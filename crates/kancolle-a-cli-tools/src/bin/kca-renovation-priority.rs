use std::cmp::Ordering;

use anyhow::Result;
use kancolle_a::ships::{self, ShipMod, ShipsBuilder};
use kancolle_a_cli_tools::cli_helpers;

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
        Kekkonable {
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
        if ship.mods().is_empty()
            || ship
                .mods()
                .iter()
                .all(|ship_mod| ship_mod.character().is_none())
        {
            results.push(State::MissingAll(ship_name));
            continue;
        }

        if missing(ship.mods().first().unwrap()) {
            results.push(State::MissingBase(ship_name));
        }

        for ship_mod in ship.mods().iter().filter(|&ship_mod| {
            ship_mod.kekkon().is_some()
                && ship_mod.character().is_some()
                && !ship_mod.character().as_ref().unwrap().married
                && ship_mod.character().as_ref().unwrap().lv >= 99
        }) {
            results.push(State::Kekkonable {
                ship_name,
                current: ship_mod,
            });
        }

        for i in 0..ship.mods().len() - 1 {
            let ship_mod_pair = &ship.mods()[i..=i + 1];
            let (current, next) = (&ship_mod_pair[0], &ship_mod_pair[1]);
            if current.character().is_none() || !missing(next) {
                continue;
            }

            if ship.blueprint().is_some()
                && ship.blueprint().as_ref().unwrap().blueprint_total_num
                    >= ship
                        .shipmod_blueprint_cost(next.remodel_level())
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
        State::Kekkonable { .. } => match right {
            State::MissingAll(_)
            | State::MissingBase(_)
            | State::Constructable { .. }
            | State::UpgradeAvailable { .. }
            | State::UpgradeReady { .. } => Ordering::Less,
            State::Kekkonable { .. } => Ordering::Equal,
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
                let next_priority = if next.character().is_none() {
                    "※ "
                } else {
                    ""
                };
                let current_name = current.name();
                let next_name = next.name();
                let current_stars = current.character().as_ref().unwrap().star_num;
                eprintln!(
                    "{next_priority}CONSTRUCTABLE\t{ship_name}\t{current_name}({current_stars}/5)\t=> {next_name}"
                );
            }
            State::UpgradeAvailable {
                ship_name,
                current,
                next,
            } => {
                let next_priority = if next.character().is_none() {
                    "※ "
                } else {
                    ""
                };
                let current_name = current.name();
                let next_name = next.name();
                let current_stars = current.character().as_ref().unwrap().star_num;
                eprintln!(
                    "{next_priority}AVAILABLE\t{ship_name}\t{current_name}({current_stars}/5)\t=> {next_name}"
                );
            }
            State::UpgradeReady {
                ship_name,
                current,
                next,
            } => {
                let next_priority = if next.character().is_none() {
                    "※ "
                } else {
                    ""
                };
                let current_name = current.name();
                let next_name = next.name();
                eprintln!(
                    "{next_priority}READY\t\t{ship_name}\t{current_name}      \t=> {next_name}"
                );
            }
            State::StarsNeeded { ship_name, current } => {
                let current_name = current.name();
                let current_stars = current.character().as_ref().unwrap().star_num;
                eprintln!("STARS NEEDED\t{ship_name}\t{current_name}({current_stars}/5)");
            }
            State::Kekkonable { ship_name, current } => {
                let current_name = current.name();
                eprintln!("KEKKONABLE\t{ship_name}\t{current_name}");
            }
        }
    }

    Ok(())
}
