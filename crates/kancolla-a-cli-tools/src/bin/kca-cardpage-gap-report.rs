use anyhow::Result;
use kancolle_a::{importer::kancolle_arcade_net::BookShipCardPageSource, ships::ShipsBuilder};
use kancolle_a_cli_tools::cli_helpers;

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
        .to_options().descr("A tool to report on cardpage data gaps.").header("Please share any reported knowable gaps with the tool author to update the source.")
    }

    #[test]
    fn kca_cardpage_gap_report_check_options() {
        options().check_invariants(false)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = args::options().run();

    let ships = cli_helpers::ship_source_data_applier(&args.data, ShipsBuilder::default())?
        .build()
        .await?;

    let mut unknown_pages: Vec<(u16, &str, Vec<u16>, Vec<u16>)> = vec![];

    for (ship_name, book_ship) in ships
        .shipmod_iter()
        .filter_map(|shipmod| shipmod.book().as_ref().map(|ship| (shipmod.name(), ship)))
    {
        let mut knowable: Vec<u16> = vec![];
        let mut unknown: Vec<u16> = vec![];
        for page in &book_ship.card_list[1..] {
            match book_ship.source(page.priority) {
                BookShipCardPageSource::Normal => panic!("Normal page after page 1"),
                BookShipCardPageSource::Unknown => {
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
            unknown_pages.push((book_ship.book_no, ship_name, knowable, unknown));
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
