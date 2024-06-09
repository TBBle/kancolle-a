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

    let mut unknown_pages: Vec<(u16, &str, Vec<u16>, Vec<u16>)> = vec![];

    for ship in tc_list.iter().filter(|ship| ship.card_list().len() > 1) {
        let mut knowable: Vec<u16> = vec![];
        let mut unknown: Vec<u16> = vec![];
        for page in &ship.card_list()[1..] {
            use BookShipCardPageSource::*;
            match ship.source(*page.priority()) {
                Normal => panic!("Normal page after page 1"),
                Unknown => {
                    if *page.acquire_num_in_page() > 0 {
                        &mut knowable
                    } else {
                        &mut unknown
                    }
                }
                .push(*page.priority()),
                _ => (),
            }
        }
        if !knowable.is_empty() || !unknown.is_empty() {
            unknown_pages.push((*ship.book_no(), ship.ship_name(), knowable, unknown));
        }
    }

    if unknown_pages.is_empty() {
        println!("No unidentified pages!");
        return Ok(());
    }

    println!("Knowable");
    println!("#\tSome\tNone\tShip");

    for (book_no, ship_name, knowable, unknown) in unknown_pages
        .iter()
        .filter(|(_, _, _, unknown)| unknown.len() <= 1)
    {
        println!("{book_no}\t{knowable:?}\t{unknown:?}\t{ship_name}");
    }

    println!("\nUnknowable");
    println!("#\tSome\tNone\tShip");
    for (book_no, ship_name, knowable, unknown) in unknown_pages
        .iter()
        .filter(|(_, _, _, unknown)| unknown.len() > 1)
    {
        println!("{book_no}\t{knowable:?}\t{unknown:?}\t{ship_name}");
    }

    Ok(())
}
