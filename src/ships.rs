//! The abstract concept of a ship(girl) in Kancolle Arcade

use derive_getters::Getters;
use std::{collections::HashMap, error::Error, io::Read, ops::Deref};

use crate::importer::kancolle_arcade_net::{
    self, ApiEndpoint, BlueprintShip, BookShip, Character, ClientBuilder, KekkonKakkoKari, KANMUSU,
};

// Based on https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
pub struct ShipsBuilder {
    book: Option<Box<dyn Read>>,
    blueprint: Option<Box<dyn Read>>,
    character: Option<Box<dyn Read>>,
    kekkon: Option<Box<dyn Read>>,
    api_client_builder: Option<ClientBuilder>,
}

impl Default for ShipsBuilder {
    fn default() -> Self {
        Self::new().static_kekkon()
    }
}

impl ShipsBuilder {
    pub fn new() -> ShipsBuilder {
        ShipsBuilder {
            book: None,
            blueprint: None,
            character: None,
            kekkon: None,
            api_client_builder: None,
        }
    }

    pub fn build(mut self) -> Result<Ships, Box<dyn Error>> {
        if let Some(ref api_client_builder) = self.api_client_builder {
            if self.book.is_none() || self.blueprint.is_none() || self.character.is_none() {
                let client = api_client_builder.build()?;
                if self.book.is_none() {
                    self.book = Some(client.fetch(&ApiEndpoint::TcBookInfo)?)
                };
                if self.blueprint.is_none() {
                    self.blueprint = Some(client.fetch(&ApiEndpoint::BlueprintListInfo)?)
                }
                if self.character.is_none() {
                    self.character = Some(client.fetch(&ApiEndpoint::CharacterListInfo)?)
                }
            }
        }
        Ships::new(self)
    }

    pub fn no_book(mut self) -> ShipsBuilder {
        self.book = None;
        self
    }

    pub fn book_from_reader<R>(mut self, reader: R) -> ShipsBuilder
    where
        R: Read + 'static,
    {
        self.book = Some(Box::new(reader));
        self
    }

    pub fn no_blueprint(mut self) -> ShipsBuilder {
        self.blueprint = None;
        self
    }

    pub fn blueprint_from_reader<R>(mut self, reader: R) -> ShipsBuilder
    where
        R: Read + 'static,
    {
        self.blueprint = Some(Box::new(reader));
        self
    }

    pub fn no_character(mut self) -> ShipsBuilder {
        self.character = None;
        self
    }

    pub fn character_from_reader<R>(mut self, reader: R) -> ShipsBuilder
    where
        R: Read + 'static,
    {
        self.character = Some(Box::new(reader));
        self
    }

    pub fn no_kekkon(mut self) -> ShipsBuilder {
        self.kekkon = None;
        self
    }

    pub fn static_kekkon(self) -> ShipsBuilder {
        self.kekkon_from_reader(KANMUSU.as_ref())
    }

    pub fn kekkon_from_reader<R>(mut self, reader: R) -> ShipsBuilder
    where
        R: Read + 'static,
    {
        self.kekkon = Some(Box::new(reader));
        self
    }

    pub fn jsessionid(mut self, jsessionid: String) -> ShipsBuilder {
        self.api_client_builder = Some(ClientBuilder::new().jsessionid(jsessionid));
        self
    }
}

pub struct Ships(HashMap<String, Ship>);

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for Ships {
    type Target = HashMap<String, Ship>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
    fn new(builder: ShipsBuilder) -> Result<Self, Box<dyn Error>> {
        let book = match builder.book {
            None => None,
            Some(reader) => {
                let mut book = kancolle_arcade_net::read_tclist(reader)?;
                book.retain(|ship| ship.acquire_num > 0);
                Some(book)
            }
        };

        let bplist = match builder.blueprint {
            None => None,
            Some(reader) => Some(kancolle_arcade_net::read_blueprintlist(reader)?),
        };

        let characters = match builder.character {
            None => None,
            Some(reader) => Some(kancolle_arcade_net::read_characterlist(reader)?),
        };

        let kekkonlist = match builder.kekkon {
            None => None,
            Some(reader) => Some(kancolle_arcade_net::read_kekkonkakkokarilist(reader)?),
        };

        // TODO: Can we precalculate capacity? What happens if we undershoot by a bit?
        // HACK: 500 is more than the kekkon list, so it'll do for now.
        let mut ships: HashMap<String, Ship> = HashMap::with_capacity(500);

        // TODO: Avoid cloning and panic in and_modify, and unwrap() in or_insert_with_key.
        // For the cloning issue, refactor Ship to be dumber, and use or_default instead, I guess.

        // For blueprints and book pages, it'd be nice if Ship could just hold references into the relevant
        // lists rather than moving/cloning, but Rust data model makes that hard.
        // See https://docs.rs/rental/latest/rental/ but I don't know if that crate can solve this instance.

        // Kekkon list is a convenient source of distinct ship names, if we have it.
        if let Some(kekkonlist) = kekkonlist {
            for kekkon in kekkonlist.into_iter() {
                ships
                    .entry(kekkon.name.clone())
                    .and_modify(|ship| match &ship.kekkon {
                        None => ship.kekkon = Some(kekkon.clone()),
                        Some(_) => {
                            panic!("Duplicate kekkon entry for {}", kekkon.name.clone())
                        }
                    })
                    .or_insert_with_key(|ship_name| {
                        Ship::new(ship_name.clone(), None, None, Some(kekkon), None).unwrap()
                    });
            }
        };

        if let Some(characters) = characters {
            for character in characters.into_iter() {
                ships
                    .entry(character.ship_name.clone())
                    .and_modify(|ship| match &ship.character {
                        None => ship.character = Some(character.clone()),
                        Some(_) => panic!(
                            "Duplicate character entry for {}",
                            character.ship_name.clone()
                        ),
                    })
                    .or_insert_with_key(|ship_name| {
                        Ship::new(ship_name.clone(), None, Some(character), None, None).unwrap()
                    });
            }
        }

        if let Some(book) = book {
            for book_ship in book.into_iter() {
                if book_ship.card_list[0].variation_num_in_page == 6 {
                    // Clone book entry for the 改 modification which shares it
                    let book_ship = book_ship.clone();
                    let ship_name = format!("{}改", book_ship.ship_name);

                    ships
                        .entry(ship_name.clone())
                        .and_modify(|ship| match &ship.book {
                            None => {
                                ship.book = Some(book_ship.clone());
                                ship.book_secondrow = true
                            }
                            Some(_) => panic!("Duplicate book entry for {}", ship_name.clone()),
                        })
                        .or_insert_with_key(|ship_name| {
                            Ship::new(ship_name.clone(), Some(book_ship), None, None, None).unwrap()
                        });
                }

                ships
                    .entry(book_ship.ship_name.clone())
                    .and_modify(|ship| match &ship.book {
                        None => {
                            ship.book = Some(book_ship.clone());
                            ship.book_secondrow = false
                        }
                        Some(_) => {
                            panic!("Duplicate book entry for {}", book_ship.ship_name.clone())
                        }
                    })
                    .or_insert_with_key(|ship_name| {
                        Ship::new(ship_name.clone(), Some(book_ship), None, None, None).unwrap()
                    });
            }
        }

        if let Some(bplist) = bplist {
            for bp_ship in bplist.into_iter() {
                // Find existing ships which are modified from this blueprint, and clone the blueprint to them.
                for (_, ship) in ships.iter_mut().filter(|(name, _)| {
                    name != &&bp_ship.ship_name && ship_blueprint_name(name) == bp_ship.ship_name
                }) {
                    match &ship.blueprint {
                        None => ship.blueprint = Some(bp_ship.clone()),
                        Some(_) => panic!(
                            "Duplicate blueprint entry for {}",
                            bp_ship.ship_name.clone()
                        ),
                    }
                }

                ships
                    .entry(bp_ship.ship_name.clone())
                    .and_modify(|ship| match &ship.blueprint {
                        None => ship.blueprint = Some(bp_ship.clone()),
                        Some(_) => panic!(
                            "Duplicate blueprint entry for {}",
                            bp_ship.ship_name.clone()
                        ),
                    })
                    .or_insert_with_key(|ship_name| {
                        Ship::new(ship_name.clone(), None, None, None, Some(bp_ship)).unwrap()
                    });
            }
        }

        for ship in ships.values() {
            ship.validate()?
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

    /// The relevant entry in the player's character list data
    character: Option<Character>,

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
        character: Option<Character>,
        kekkon: Option<KekkonKakkoKari>,
        blueprint: Option<BlueprintShip>,
    ) -> Result<Self, Box<dyn Error>> {
        let book_secondrow = match &book {
            None => false,
            // TODO: We should error here.
            Some(book) => {
                let normal_page = &book.card_list[0];
                // TODO: We should probably error here.
                assert_eq!(
                    normal_page.variation_num_in_page % 3,
                    0,
                    "Unexpected variation count {} on normal page of {}",
                    normal_page.variation_num_in_page,
                    book.book_no
                );
                let row_count = normal_page.variation_num_in_page / 3;
                if row_count > 1 && name.ends_with("改") {
                    true
                } else {
                    false
                }
            }
        };

        Ok(Ship {
            name,
            book,
            book_secondrow,
            character,
            kekkon,
            blueprint,
        })
    }

    /// Validate the various data elements agree when present
    fn validate(&self) -> Result<(), Box<dyn Error>> {
        // TODO: We should error in the failure cases, not panic.

        if let Some(book) = self.book.as_ref() {
            assert_ne!(
                book.acquire_num, 0,
                "Ship {} created from \"Unknown\" book entry {}",
                self.name, book.book_no
            );

            let normal_page = &book.card_list[0];
            assert_eq!(
                normal_page.variation_num_in_page % 3,
                0,
                "Unexpected variation count {} on normal page of {}",
                normal_page.variation_num_in_page,
                book.book_no
            );
            let row_count = normal_page.variation_num_in_page / 3;
            assert_eq!(
                self.book_secondrow,
                if row_count > 1 && self.name.ends_with("改") {
                    true
                } else {
                    false
                }
            );
        }

        if let Some(kekkon) = self.kekkon.as_ref() {
            assert_eq!(self.name, kekkon.name);
        }
        if let Some(character) = self.character.as_ref() {
            assert_eq!(self.name, character.ship_name);
        }

        // TODO: Check consistency across the passed-in items where they overlap, e.g., names, types.

        Ok(())
    }

    // TODO: More APIs, particulary when there's multiple sources of truth, and some are more trustworthy
    // than others.
}
