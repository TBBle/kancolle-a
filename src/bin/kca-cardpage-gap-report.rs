use kancolle_a::{
    cli_helpers, importer::kancolle_arcade_net::BookShipCardPageSource, ships::ShipsBuilder,
};
use std::error::Error;

pub(crate) mod args {
    use bpaf::*;
    use kancolle_a::cli_helpers::{self, ShipSourceDataOptions};

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) data: ShipSourceDataOptions,
    }

    pub fn options() -> OptionParser<Options> {
        let data = cli_helpers::ship_source_data_parser();
        construct!(Options { data })
        .to_options().descr("A tool to report on cardpage data gaps.").header("Please share any reported knowable gaps with the tool author to update the source.")
    }

    #[test]
    fn kca_cardpage_gap_report_check_options() {
        options().check_invariants(false)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();

    let ships =
        cli_helpers::ship_source_data_applier(&args.data, ShipsBuilder::default())?.build()?;

    let mut unknown_pages: Vec<(u16, &str, Vec<u16>, Vec<u16>)> = vec![];

    for (ship_name, ship) in ships
        .iter()
        .filter_map(|(ship_name, ship)| ship.book().as_ref().map(|ship| (ship_name, ship)))
    {
        let mut knowable: Vec<u16> = vec![];
        let mut unknown: Vec<u16> = vec![];
        for page in &ship.card_list[1..] {
            use BookShipCardPageSource::*;
            match ship.source(page.priority) {
                Normal => panic!("Normal page after page 1"),
                Unknown => {
                    if page.acquire_num_in_page > 0 {
                        &mut knowable
                    } else {
                        &mut unknown
                    }
                }
                .push(page.priority),
                _ => (),
            }
        }
        if !knowable.is_empty() || !unknown.is_empty() {
            unknown_pages.push((ship.book_no, ship_name, knowable, unknown));
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
