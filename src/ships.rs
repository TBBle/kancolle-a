//! The abstract concept of a ship(girl) in Kancolle Arcade

use derive_getters::Getters;
use std::{collections::HashMap, error::Error, io::Read, ops::Deref};

use crate::importer::kancolle_arcade_net::{self, BlueprintShip, BookShip, KekkonKakkoKari};

pub struct Ships(HashMap<String, Ship>);

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for Ships {
    type Target = HashMap<String, Ship>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub enum UserDataSource<'a> {
    #[default]
    None,
    FromReader(&'a mut dyn Read),
    // FromSega // TODO: Authentication data needed.
}

#[derive(Default)]
pub enum GlobalDataSource<'a> {
    #[default]
    Static,
    FromReader(&'a mut dyn Read),
    // FromUpstream // TODO: Authentication data needed in some cases...
}

#[derive(Default)]
pub struct DataSources<'a> {
    pub book: UserDataSource<'a>,
    pub blueprint: UserDataSource<'a>,

    pub kekkon: GlobalDataSource<'a>,
}

/// Determine the blueprint/unmodified ship name for the given ship
fn ship_blueprint_name(ship_name: &str) -> &str {
    // Base case: Ships
    let base_name = if let Some(split_name) = ship_name.split_once("改") {
        split_name.0
    } else {
        ship_name
    };

    // Ships that are renamed in modification, or have non-改 variants.
    match base_name {
        // Tested against data
        "龍鳳" => "大鯨",
        "Верный" => "響",
        "Italia" => "Littorio",
        // Untested against data as I don't own them.
        "呂500" => "U-511",
        "千代田甲" | "千代田航" => "千代田",
        "千歳甲" | "千歳航" => "千歳",
        "Октябрьская революция" => "Гангут",
        "大鷹" => "春日丸",
        _ => base_name,
    }
}

impl Ships {
    /// Import a list of ships from the given datasource
    pub fn new(data_sources: DataSources) -> Result<Self, Box<dyn Error>> {
        let book = match data_sources.book {
            UserDataSource::None => None,
            UserDataSource::FromReader(reader) => {
                let mut book = kancolle_arcade_net::read_tclist(reader)?;
                book.retain(|ship| *ship.acquire_num() > 0);
                Some(book)
            }
        };

        let bplist = match data_sources.blueprint {
            UserDataSource::None => None,
            UserDataSource::FromReader(reader) => {
                Some(kancolle_arcade_net::read_blueprintlist(reader)?)
            }
        };

        let kekkonlist = match data_sources.kekkon {
            GlobalDataSource::Static => None, // TODO.
            GlobalDataSource::FromReader(reader) => {
                Some(kancolle_arcade_net::read_kekkonkakkokarilist(reader)?)
            }
        };

        // TODO: Can we precalculate capacity? What happens if we undershoot by a bit?
        // HACK: 500 is more than the kekkon list, so it'll do for now.
        let mut ships: HashMap<String, Ship> = HashMap::with_capacity(500);

        // Kekkon list is a convenient source of distinct ship names, if we have it.
        // TODO: This should never be None, once Static is implemented.
        if let Some(kekkonlist) = kekkonlist {
            for kekkon in kekkonlist.into_iter() {
                let book_ship = if let Some(book) = book.as_ref() {
                    match book
                        .iter()
                        .position(|book_ship| (*book_ship.book_no() as u32) == kekkon.id)
                    {
                        None => None,
                        Some(index) => {
                            assert!(*book[index].acquire_num() > 0);
                            Some(book[index].clone())
                        }
                    }
                } else {
                    None
                };

                let bp_ship = if let Some(bplist) = bplist.as_ref() {
                    match bplist.iter().position(|bp_ship| {
                        bp_ship.ship_name() == ship_blueprint_name(&kekkon.name)
                    }) {
                        None => None,
                        Some(index) => Some(bplist[index].clone()),
                    }
                } else {
                    None
                };

                match ships.insert(
                    kekkon.name.clone(),
                    Ship::new(kekkon.name.clone(), book_ship, Some(kekkon), bp_ship)?,
                ) {
                    Some(old_ship) => panic!("Duplicate ship {}", old_ship.name()),
                    None => (),
                }
            }
        };

        if let Some(book) = book {
            for book_ship in book.into_iter() {
                let bp_ship = if let Some(bplist) = bplist.as_ref() {
                    match bplist.iter().position(|bp_ship| {
                        bp_ship.ship_name() == ship_blueprint_name(book_ship.ship_name())
                    }) {
                        None => None,
                        Some(index) => Some(bplist[index].clone()),
                    }
                } else {
                    None
                };
                if *book_ship.card_list()[0].variation_num_in_page() == 6 {
                    ships
                        .entry(format!("{}改", book_ship.ship_name()))
                        .or_insert_with_key(|ship_name| {
                            Ship::new(
                                ship_name.clone(),
                                Some(book_ship.clone()),
                                None,
                                bp_ship.clone(),
                            )
                            .unwrap()
                        });
                }
                ships
                    .entry(book_ship.ship_name().clone())
                    .or_insert_with_key(|ship_name| {
                        Ship::new(ship_name.clone(), Some(book_ship), None, bp_ship).unwrap()
                    });
            }
        }

        if let Some(bplist) = bplist {
            for bp_ship in bplist.into_iter() {
                ships
                    .entry(bp_ship.ship_name().clone())
                    .or_insert_with_key(|ship_name| {
                        Ship::new(ship_name.clone(), None, None, Some(bp_ship)).unwrap()
                    });
            }
        }

        Ok(Ships(ships))
    }
}

/// A Kancolle Arcade shipgirl
/// Only the name is reliably unique.
/// Many other fields may either surprisingly overlap, or are optional.
#[derive(Debug, Getters)]
pub struct Ship {
    /// Full ship name
    name: String,

    // Everything below here is still in-flux as I shake out the API.
    // Some of these things might become references to data structures held
    // elsewhere, but I think that gets complicated in Rust? So we'll just
    // copy everything for now.
    /// The relevant entry in the player's picture book data
    book: Option<BookShip>,
    /// Whether this is actually the second-row entry in the BookShip
    book_secondrow: bool,

    /// Any Kekkon Kakko Kari entry for this ship
    kekkon: Option<KekkonKakkoKari>,

    /// The Blueprint data for this ship's base ship
    /// May be empty because the player has no blueprints, or
    /// because the base ship is not identified correctly.
    blueprint: Option<BlueprintShip>,
}

impl Ship {
    /// Instantiate a new ship from various sources of ship data.
    /// TODO: Establish a library-wide error type. Probably using thiserror.
    fn new(
        name: String,
        book: Option<BookShip>,
        kekkon: Option<KekkonKakkoKari>,
        blueprint: Option<BlueprintShip>,
    ) -> Result<Self, Box<dyn Error>> {
        let book_secondrow = match &book {
            None => false,
            // TODO: We should error here.
            Some(book) if *book.acquire_num() == 0 => {
                panic!(
                    "Ship {} created from \"Unknown\" book entry {}",
                    name,
                    book.book_no()
                )
            }
            Some(book) => {
                let normal_page = &book.card_list()[0];
                // TODO: We should probably error here.
                assert_eq!(
                    normal_page.variation_num_in_page() % 3,
                    0,
                    "Unexpected variation count {} on normal page of {}",
                    normal_page.variation_num_in_page(),
                    book.book_no()
                );
                let row_count = normal_page.variation_num_in_page() / 3;
                if row_count > 1 && name.ends_with("改") {
                    true
                } else {
                    false
                }
            }
        };

        // TODO: Check consistency across the passed-in items where they overlap, e.g., names, types.

        Ok(Ship {
            name,
            book,
            book_secondrow,
            kekkon,
            blueprint,
        })
    }

    // TODO: More APIs, particulary when there's multiple sources of truth, and some are more trustworthy
    // than others.
}
