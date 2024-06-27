use std::path::PathBuf;

use crate::importer::kancolle_arcade_net::BookShipCardPageSourceDiscriminants;
use itertools;
use strum::VariantNames;

use bpaf::*;

// Per https://github.com/pacak/bpaf/discussions/197
pub fn book_ship_card_page_source_parser() -> impl Parser<BookShipCardPageSourceDiscriminants> {
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

pub fn tcbook_path_parser() -> impl Parser<PathBuf> {
    long("tcbook")
        .help("A copy of your https://kancolle-arcade.net/ac/api/TcBook/info")
        .argument::<PathBuf>("TCBOOK")
}

pub fn bplist_path_parser() -> impl Parser<PathBuf> {
    long("bplist")
        .help("A copy of your https://kancolle-arcade.net/ac/api/BlueprintList/info")
        .argument::<PathBuf>("BPLIST")
}

pub fn places_path_parser() -> impl Parser<PathBuf> {
    long("places")
        .help("A copy of https://kancolle-arcade.net/ac/api/Place/places")
        .argument::<PathBuf>("PLACES")
}
