use anyhow::Result;
use chrono::{Datelike, Utc};
use kancolle_a::ships::ShipsBuilder;
use kancolle_a_cli_tools::cli_helpers;
use std::collections::BTreeMap;

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
            .descr("A tool to dump your current blueprint inventory.")
            .header("Output is grouped by expiry month.")
    }

    #[test]
    fn kca_blueprint_dump_check_options() {
        options().check_invariants(false)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = args::options().run();

    let ships = cli_helpers::ship_source_data_applier(&args.data, ShipsBuilder::default())?
        .build()
        .await?;

    let mut bp_per_month = BTreeMap::new();

    // TODO: The Ships abstraction makes this a little messy, since Blueprint data is duplicated.
    // A "base ship" or "blueprint ship" flag might improve the aesthetics.
    for bp in ships
        .iter()
        .filter(|(ship_name, ship)| {
            ship.blueprint().is_some()
                && ship.blueprint().as_ref().unwrap().ship_name == **ship_name
        })
        .map(|(_, ship)| ship.blueprint().as_ref().unwrap())
    {
        for entry in &bp.expiration_date_list {
            bp_per_month
                .entry(entry.expiration_date)
                .or_insert_with(Vec::<(&String, u16)>::new);
            let month_ships = bp_per_month.get_mut(&entry.expiration_date).unwrap();
            month_ships.push((&bp.ship_name, entry.blueprint_num));
        }
    }

    let today = Utc::now();
    let mut expiring_this_month = 0u16;

    for (month, blueprints) in bp_per_month {
        let is_this_month = month.year() == today.year() && month.month() == today.month();
        let month_name = month.format("%Y %B");
        println!("{month_name}");
        for (name, count) in blueprints {
            println!("  {name}\t{count}");
            if is_this_month {
                expiring_this_month += count;
            }
        }
        println!();
    }

    if expiring_this_month > 0 {
        println!("{expiring_this_month} blueprints expiring this month!")
    }

    Ok(())
}
