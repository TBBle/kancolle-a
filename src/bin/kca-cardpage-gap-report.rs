use kancolle_a::importer::kancolle_arcade_net::{BookShipCardPageSource, TcBook};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub(crate) mod args {
    use std::path::PathBuf;

    use bpaf::*;
    use kancolle_a::cli_helpers;

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) tcbook: PathBuf,
    }

    pub fn options() -> OptionParser<Options> {
        let tcbook = cli_helpers::tcbook_path_parser();
        construct!(Options {
            tcbook,
        })
        .to_options().descr("A tool to report on cardpage data gaps.").header("Please share any reported knowable gaps with the tool author to update the source.")
    }

    #[test]
    fn kca_cardpage_gap_report_check_options() {
        options().check_invariants(false)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();
    let tc_path = args.tcbook;

    let tc_data = BufReader::new(File::open(tc_path)?);

    let tc_list = TcBook::new(tc_data)?;

    let mut unknown_pages: Vec<(u16, &str, Vec<u16>, Vec<u16>)> = vec![];

    for ship in tc_list.iter().filter(|ship| *ship.acquire_num() > 0) {
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
