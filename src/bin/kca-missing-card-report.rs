use kancolle_a::importer::kancolle_arcade_net::{BookShipCardPageSourceDiscriminants, TcBook};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub(crate) mod args {
    use itertools;
    use kancolle_a::importer::kancolle_arcade_net::BookShipCardPageSourceDiscriminants;
    use std::path::PathBuf;
    use strum::VariantNames;

    use bpaf::*;

    // Per https://github.com/pacak/bpaf/discussions/197
    fn source() -> impl Parser<BookShipCardPageSourceDiscriminants> {
        const DEFAULT: BookShipCardPageSourceDiscriminants =
            BookShipCardPageSourceDiscriminants::Normal;

        let mut help_msg = Doc::from("The event source to show cards for.\n One of ");

        // TODO: intersperse will move into the core library at some point.
        // https://github.com/rust-lang/rust/issues/79524
        for (index, &text) in
            itertools::intersperse(BookShipCardPageSourceDiscriminants::VARIANTS, &", ").enumerate()
        {
            if index % 2 == 0 {
                help_msg.literal(text)
            } else {
                help_msg.text(text)
            }
        }

        long("source")
            .help(help_msg)
            .argument::<String>("SOURCE")
            .parse(|x| x.parse())
            .fallback(DEFAULT)
            .display_fallback()
    }

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) tcbook: PathBuf,
        pub(crate) source: BookShipCardPageSourceDiscriminants,
        pub(crate) skip_kai: bool,
    }

    pub fn options() -> OptionParser<Options> {
        let tcbook = long("tcbook")
            .help("A copy of your https://kancolle-arcade.net/ac/api/TcBook/info")
            .argument::<PathBuf>("TCBOOK");
        let source = source();

        // TODO: What did I want this flag for? I _might_ have wanted to show only droppable
        // ships, or maybe it was actually for Blueprint checks?
        // In the latter case, kca-blueprints-ready will supplant it.
        let skip_kai = long("skip-kai")
            .help("Should we exclude ships with 改 in their name?")
            .switch()
            .fallback(false)
            .display_fallback();
        construct!(Options {
            tcbook,
            source,
            skip_kai
        })
        .to_options().descr("A tool to report on missing cards from your collection.").header("For event sources, reports all ships with missing cards.\n For Normal source, reports only cards where you are missing the non-holo version.")
    }

    #[test]
    fn check_options() {
        options().check_invariants(false)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();
    let tc_path = args.tcbook;

    let tc_data = BufReader::new(File::open(tc_path)?);

    let tc_list = TcBook::new(tc_data)?;

    let mut card_status: Vec<(u16, String, bool, bool, bool)> = vec![];

    // TODO: OriginalIllustration needs to be handled specially, card_status assumes badly.
    // Note: It's possible that Original Illustrations are actually Normal, Holo, Damaged too.
    let target_source = args.source;
    let skip_kai = args.skip_kai;

    for ship in tc_list.iter().filter(|ship| *ship.acquire_num() > 0) {
        for page in ship.card_list().iter().filter(|page| {
            BookShipCardPageSourceDiscriminants::from(ship.source(*page.priority()))
                == target_source
        }) {
            let (row1, row2) = page.card_img_list().split_at(3);
            if row1[0].is_empty()
                || row1[1].is_empty()
                    && target_source != BookShipCardPageSourceDiscriminants::Normal
                || row1[2].is_empty()
                    && target_source != BookShipCardPageSourceDiscriminants::Normal
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
                || row2[1].is_empty()
                    && target_source != BookShipCardPageSourceDiscriminants::Normal
                || row2[2].is_empty()
                    && target_source != BookShipCardPageSourceDiscriminants::Normal
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
