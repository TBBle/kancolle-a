use kancolle_a::importer::kancolle_arcade_net::{BookShipCardPageSource, TcBook};
use std::error::Error;
use std::io::BufReader;
use std::{env, fs::File};

fn main() -> Result<(), Box<dyn Error>> {
    let bp_path = env::args()
        .nth(1)
        .ok_or("Need an info file to open".to_owned())?;

    let tc_data = BufReader::new(File::open(bp_path)?);

    let tc_list = TcBook::new(tc_data)?;

    let mut card_status: Vec<(u16, String, bool, bool, bool)> = vec![];

    // TODO: CLI, rather than editing the code.
    use BookShipCardPageSource::*;
    let target_source = Normal;
    let skip_kai = true;

    for ship in tc_list.iter().filter(|ship| *ship.acquire_num() > 0) {
        for page in ship
            .card_list()
            .iter()
            .filter(|page| ship.source(*page.priority()) == target_source)
        {
            let (row1, row2) = page.card_img_list().split_at(3);
            if row1[0].is_empty()
                || row1[1].is_empty() && target_source != Normal
                || row1[2].is_empty() && target_source != Normal
            {
                card_status.push((
                    *ship.book_no(),
                    ship.ship_name().clone(),
                    !row1[0].is_empty(),
                    !row1[1].is_empty(),
                    !row1[2].is_empty(),
                ));
            }
            if row2.is_empty() {
                continue;
            }
            if row2[0].is_empty()
                || row2[1].is_empty() && target_source != Normal
                || row2[2].is_empty() && target_source != Normal
            {
                card_status.push((
                    *ship.book_no(),
                    ship.ship_name().clone() + "改",
                    !row2[0].is_empty(),
                    !row2[1].is_empty(),
                    !row2[2].is_empty(),
                ));
            }
        }
    }

    println!("Missing ({target_source:?})");
    println!("#\tNHD\tShip");

    for (book_no, ship_name, normal, holo, damaged) in {
        card_status
            .iter()
            .filter(|(_, name, _, _, _)| !(skip_kai && name.contains("改")))
    } {
        let status = match (normal, holo, damaged) {
            (false, false, false) => "...",
            (false, false, true) => "..D",
            (false, true, false) => ".H.",
            (false, true, true) => ".HD",
            (true, false, false) => "N..",
            (true, false, true) => "N.D",
            (true, true, false) => "NH.",
            (true, true, true) => "NHD",
        };
        println!("{book_no}\t{status}\t{ship_name}");
    }

    Ok(())
}
