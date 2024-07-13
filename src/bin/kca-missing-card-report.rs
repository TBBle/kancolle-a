use kancolle_a::{
    importer::kancolle_arcade_net::BookShipCardPageSourceDiscriminants,
    ships::{DataSources, GlobalDataSource, Ships, UserDataSource},
};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub(crate) mod args {
    use kancolle_a::cli_helpers::{self, ShipSourceDataOptions};
    use kancolle_a::importer::kancolle_arcade_net::BookShipCardPageSourceDiscriminants;

    use bpaf::*;

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) data: ShipSourceDataOptions,
        pub(crate) source: BookShipCardPageSourceDiscriminants,
        pub(crate) skip_unseen: bool,
    }

    pub fn options() -> OptionParser<Options> {
        let data = cli_helpers::ship_file_sources_parser();
        let source = cli_helpers::book_ship_card_page_source_parser();

        let skip_unseen = long("skip-unseen")
            .help("Should we exclude ships not present in our book?")
            .switch()
            .fallback(false)
            .display_fallback();
        construct!(Options {
            data,
            source,
            skip_unseen
        })
        .to_options().descr("A tool to report on missing cards from your collection.").header("For event sources, reports all ships with missing cards.\n For Normal source, reports only cards where you are missing the non-holo version.")
    }

    #[test]
    fn kca_missing_card_reports_check_options() {
        options().check_invariants(false)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();

    let mut tc_reader = BufReader::new(File::open(args.data.tcbook)?);
    let mut bp_reader = BufReader::new(File::open(args.data.bplist)?);
    let mut kk_reader = BufReader::new(File::open(args.data.kekkon)?);

    let data_source = DataSources {
        book: UserDataSource::FromReader(&mut tc_reader),
        blueprint: UserDataSource::FromReader(&mut bp_reader),
        kekkon: GlobalDataSource::FromReader(&mut kk_reader),
    };

    let ships = Ships::new(data_source)?;

    let mut card_status: Vec<(u16, String, bool, bool, bool)> = vec![];

    // TODO: OriginalIllustration needs to be handled specially, card_status assumes badly.
    // Note: It's possible that Original Illustrations are actually Normal, Holo, Damaged too.
    let target_source = args.source;
    let skip_unseen = args.skip_unseen;

    for (ship_name, ship) in ships.iter().filter(|(_, ship)| ship.book().is_some()) {
        let book = ship.book().as_ref().unwrap();
        for page in book.card_list.iter().filter(|page| {
            BookShipCardPageSourceDiscriminants::from(book.source(page.priority)) == target_source
        }) {
            let (row1, row2) = page.card_img_list.split_at(3);
            if !ship.book_secondrow() {
                card_status.push((
                    book.book_no,
                    ship_name.clone(),
                    !row1[0].is_empty(),
                    !row1[1].is_empty(),
                    !row1[2].is_empty(),
                ));
            } else {
                card_status.push((
                    book.book_no,
                    ship_name.clone(),
                    !row2[0].is_empty(),
                    !row2[1].is_empty(),
                    !row2[2].is_empty(),
                ));
            }
        }
    }

    if target_source == BookShipCardPageSourceDiscriminants::Normal {
        card_status.sort_by_key(|(book_no, _, _, _, _)| *book_no);
    } else {
        card_status.sort_by_cached_key(|(_, name, _, _, _)| name.clone());
    }

    println!("Missing ({target_source:?})");
    println!("#\tNHD\tShip");

    for (book_no, ship_name, normal, holo, damaged) in {
        card_status
            .iter()
            .filter(|(_, _, normal, _, _)| {
                target_source != BookShipCardPageSourceDiscriminants::Normal || !normal
            })
            .filter(|(_, _, normal, holo, damaged)| !skip_unseen || (*normal || *holo || *damaged))
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
