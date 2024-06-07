use chrono::{Datelike, Utc};
use kancolle_a::importer::kancolle_arcade_net::BlueprintList;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::BufReader;
use std::{env, fs::File};

fn main() -> Result<(), Box<dyn Error>> {
    let bp_path = env::args()
        .nth(1)
        .ok_or("Need an info file to open".to_owned())?;

    let bp_data = BufReader::new(File::open(bp_path)?);

    let bp_list = BlueprintList::new(bp_data)?;

    let mut bp_per_month = BTreeMap::new();

    for bp in bp_list.iter() {
        for entry in bp.expiration_date_list() {
            if !bp_per_month.contains_key(entry.expiration_date()) {
                bp_per_month.insert(entry.expiration_date(), Vec::<(&String, u16)>::new());
            }
            let month_ships = bp_per_month.get_mut(entry.expiration_date()).unwrap();
            month_ships.push((bp.ship_name(), *entry.blueprint_num()));
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
