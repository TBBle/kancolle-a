use bpaf::*;
use itertools;
use kancolle_a::{
    importer::kancolle_arcade_net::BookShipCardPageSourceDiscriminants, ships::ShipsBuilder,
};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use strum::VariantNames;

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

fn jsessionid_parser() -> impl Parser<Option<String>> {
    long("jsessionid")
        .help("The JSESSIONID cookie value from https://kancolle-arcade.net/ac/")
        .argument("JSESSIONID")
        .optional()
}

fn tcbook_path_parser() -> impl Parser<Option<PathBuf>> {
    long("tcbook")
        .help("A copy of your https://kancolle-arcade.net/ac/api/TcBook/info")
        .argument::<PathBuf>("TCBOOK")
        .optional()
}

fn bplist_path_parser() -> impl Parser<Option<PathBuf>> {
    long("bplist")
        .help("A copy of your https://kancolle-arcade.net/ac/api/BlueprintList/info")
        .argument::<PathBuf>("BPLIST")
        .optional()
}

fn charlist_path_parser() -> impl Parser<Option<PathBuf>> {
    long("charlist")
        .help("A copy of your https://kancolle-arcade.net/ac/api/CharacterList/info")
        .argument::<PathBuf>("CHARLIST")
        .optional()
}

pub fn places_path_parser() -> impl Parser<PathBuf> {
    long("places")
        .help("A copy of https://kancolle-arcade.net/ac/api/Place/places")
        .argument::<PathBuf>("PLACES")
}

fn kekkon_path_parser() -> impl Parser<Option<PathBuf>> {
    long("kekkon")
        .help("An optional copy of https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json to override the builtin data")
        .argument::<PathBuf>("KEKKON")
        .optional()
}

/// A common CLI parser for getting the data needed to populate ships::ShipsBuilder
#[derive(Debug, Clone)]
pub struct ShipSourceDataOptions {
    pub tcbook: Option<PathBuf>,
    pub bplist: Option<PathBuf>,
    pub charlist: Option<PathBuf>,
    pub kekkon: Option<PathBuf>,
    pub jsessionid: Option<String>,
}

pub fn ship_source_data_parser() -> impl Parser<ShipSourceDataOptions> {
    let jsessionid = jsessionid_parser();
    let tcbook = tcbook_path_parser();
    let bplist = bplist_path_parser();
    let charlist = charlist_path_parser();
    let kekkon = kekkon_path_parser();
    construct!(ShipSourceDataOptions {
        jsessionid,
        tcbook,
        bplist,
        charlist,
        kekkon
    })
}

pub fn ship_source_data_applier(
    args: &ShipSourceDataOptions,
    mut builder: ShipsBuilder,
) -> Result<ShipsBuilder, Box<dyn Error>> {
    if let Some(tcbook) = &args.tcbook {
        builder = builder.book_from_reader(BufReader::new(File::open(tcbook)?));
    }
    if let Some(bplist) = &args.bplist {
        builder = builder.blueprint_from_reader(BufReader::new(File::open(bplist)?));
    }
    if let Some(charlist) = &args.charlist {
        builder = builder.character_from_reader(BufReader::new(File::open(charlist)?));
    }
    if let Some(kekkon) = &args.kekkon {
        builder = builder.kekkon_from_reader(BufReader::new(File::open(kekkon)?));
    }
    if let Some(jsessionid) = &args.jsessionid {
        builder = builder.jsessionid(jsessionid.clone());
    }

    Ok(builder)
}
