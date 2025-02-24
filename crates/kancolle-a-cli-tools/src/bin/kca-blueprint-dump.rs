use anyhow::Result;
use itertools::Itertools;
use kancolle_a::ships::ShipsBuilder;
use kancolle_a_cli_tools::{cli_helpers, dirty_ship_sorter};
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

    let mut expiring_this_month = 0u16;

    for bp in ships
        .iter()
        .filter(|(_, ship)| ship.blueprint().is_some())
        .sorted_by(|&left, &right| dirty_ship_sorter::dirty_ship_wiki_cmp(left.0, right.0))
        .map(|(_, ship)| ship.blueprint().as_ref().unwrap())
    {
        for entry in &bp.expiration_date_list {
            bp_per_month
                .entry(entry.expiration_date)
                .or_insert_with(Vec::<(&String, u16)>::new);
            let month_ships = bp_per_month.get_mut(&entry.expiration_date).unwrap();
            month_ships.push((&bp.ship_name, entry.blueprint_num));
            if entry.expire_this_month {
                expiring_this_month += entry.blueprint_num;
            }
        }
    }

    if expiring_this_month > 0 {
        println!("{expiring_this_month} blueprints expiring this month!")
    }

    for (month, blueprints) in bp_per_month {
        let month_name = month.format("%Y %B");
        println!("{month_name}");
        for (name, count) in blueprints {
            println!("  {name}\t{count}");
        }
        println!();
    }

    Ok(())
}
