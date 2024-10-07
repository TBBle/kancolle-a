use anyhow::Result;
use kancolle_a::{
    importer::kancolle_arcade_net::BookShipCardPageSourceDiscriminants, ships::ShipsBuilder,
};
use kancolle_a_cli_tools::cli_helpers;

pub(crate) mod args {
    use kancolle_a::importer::kancolle_arcade_net::BookShipCardPageSourceDiscriminants;
    use kancolle_a_cli_tools::cli_helpers::{self, ShipSourceDataOptions};

    use bpaf::*;

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) data: ShipSourceDataOptions,
        pub(crate) source: BookShipCardPageSourceDiscriminants,
        pub(crate) skip_unseen: bool,
    }

    pub fn options() -> OptionParser<Options> {
        let data = cli_helpers::ship_source_data_parser();
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
    fn kca_missing_card_report_check_options() {
        options().check_invariants(false)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = args::options().run();

    let ships = cli_helpers::ship_source_data_applier(&args.data, ShipsBuilder::default())?
        .build()
        .await?;

    let mut card_status: Vec<(u16, String, bool, bool, bool)> = vec![];

    // TODO: OriginalIllustration1/2 needs to be handled specially, card_status assumes badly.
    let target_source = args.source;
    let skip_unseen = args.skip_unseen;

    for shipmod in ships
        .shipmod_iter()
        .filter(|shipmod| shipmod.book().is_some())
    {
        let book_ship = shipmod.book().as_ref().unwrap();
        for page in book_ship.card_list.iter().filter(|page| {
            BookShipCardPageSourceDiscriminants::from(book_ship.source(page.priority))
                == target_source
        }) {
            // Sort-of special case: 雪風 has no swimsuits, but 雪風改 does.
            // So when the page was split, 雪風 page 1 ends up with an empty list of cards.
            // TODO: Find a way to remove the page entirely without breaking callers of source()
            if page.card_img_list.is_empty() {
                continue;
            }

            let (row1, row2) = page.card_img_list.split_at(3);
            if !shipmod.book_secondrow() {
                card_status.push((
                    book_ship.book_no,
                    shipmod.name().clone(),
                    !row1[0].is_empty(),
                    !row1[1].is_empty(),
                    !row1[2].is_empty(),
                ));
            } else {
                card_status.push((
                    book_ship.book_no,
                    shipmod.name().clone(),
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
