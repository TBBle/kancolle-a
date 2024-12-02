//! The abstract concept of a ship(girl) in Kancolle Arcade

use derive_getters::Getters;
use std::{
    collections::{hash_map::Iter as HashMapIter, HashMap},
    io::Read,
    iter::FusedIterator,
    ops::Deref,
};

use crate::importer::{
    kancolle_arcade_net::{
        self, ApiEndpoint, BlueprintShip, BookShip, Character, ClientBuilder, KekkonKakkoKari,
        KANMUSU,
    },
    wikiwiki_jp_kancolle_a::KansenShip,
};
use crate::Result;

use crate::importer::wikiwiki_jp_kancolle_a::{self, KAIZOU_KANSEN, KANSEN};

// Based on https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
pub struct ShipsBuilder {
    book: Option<Box<dyn Read>>,
    blueprint: Option<Box<dyn Read>>,
    character: Option<Box<dyn Read>>,
    kekkon: Option<Box<dyn Read>>,
    wiki_kansen_list: Option<Box<dyn Read>>,
    wiki_kaizou_kansen_list: Option<Box<dyn Read>>,
    api_client_builder: Option<ClientBuilder>,
}

impl Default for ShipsBuilder {
    fn default() -> Self {
        Self::new()
            .static_kekkon()
            .static_wiki_kansen_list()
            .static_wiki_kaizou_kansen_list()
    }
}

impl ShipsBuilder {
    pub fn new() -> ShipsBuilder {
        ShipsBuilder {
            book: None,
            blueprint: None,
            character: None,
            kekkon: None,
            wiki_kansen_list: None,
            wiki_kaizou_kansen_list: None,
            api_client_builder: None,
        }
    }

    pub async fn build(mut self) -> Result<Ships> {
        if let Some(api_client_builder) = self.api_client_builder {
            if self.book.is_none() || self.blueprint.is_none() || self.character.is_none() {
                let client = api_client_builder.build()?;
                if self.book.is_none() {
                    self.book = Some(client.fetch(&ApiEndpoint::TcBookInfo).await?)
                };
                if self.blueprint.is_none() {
                    self.blueprint = Some(client.fetch(&ApiEndpoint::BlueprintListInfo).await?)
                }
                if self.character.is_none() {
                    self.character = Some(client.fetch(&ApiEndpoint::CharacterListInfo).await?)
                }
            }
            self.api_client_builder = None
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

    pub fn no_wiki_kansen_list(mut self) -> ShipsBuilder {
        self.wiki_kansen_list = None;
        self
    }

    pub fn static_wiki_kansen_list(self) -> ShipsBuilder {
        self.wiki_kansen_list_from_reader(KANSEN.as_ref())
    }

    pub fn wiki_kansen_list_from_reader<R>(mut self, reader: R) -> ShipsBuilder
    where
        R: Read + 'static,
    {
        self.wiki_kansen_list = Some(Box::new(reader));
        self
    }

    pub fn no_wiki_kaizou_kansen_list(mut self) -> ShipsBuilder {
        self.wiki_kaizou_kansen_list = None;
        self
    }

    pub fn static_wiki_kaizou_kansen_list(self) -> ShipsBuilder {
        self.wiki_kaizou_kansen_list_from_reader(KAIZOU_KANSEN.as_ref())
    }

    pub fn wiki_kaizou_kansen_list_from_reader<R>(mut self, reader: R) -> ShipsBuilder
    where
        R: Read + 'static,
    {
        self.wiki_kaizou_kansen_list = Some(Box::new(reader));
        self
    }

    pub fn jsessionid(mut self, jsessionid: String) -> ShipsBuilder {
        self.api_client_builder = Some(
            self.api_client_builder
                .unwrap_or_default()
                .jsessionid(jsessionid),
        );
        self
    }

    pub fn userpass(mut self, username: String, password: String) -> ShipsBuilder {
        self.api_client_builder = Some(
            self.api_client_builder
                .unwrap_or_default()
                .userpass(username, password),
        );
        self
    }
}

pub struct Ships(HashMap<String, Ship>);

struct ShipModIter<'a> {
    ship_iter: HashMapIter<'a, String, Ship>,
    shipmod_iter: Option<std::slice::Iter<'a, ShipMod>>,
}

impl FusedIterator for ShipModIter<'_> {}

impl<'a> Iterator for ShipModIter<'a> {
    type Item = &'a ShipMod;
    fn next(&mut self) -> Option<Self::Item> {
        // Already exhausted case, covers "Started empty", and makes us Fused
        self.shipmod_iter.as_ref()?;

        // Current ship has another mod
        let next_mod = self.shipmod_iter.as_mut().unwrap().next();
        if next_mod.is_some() {
            return next_mod;
        }

        // Current ship exhausted, advance to next ship
        self.shipmod_iter = self.ship_iter.next().map(|(_, ship)| ship.mods.iter());

        // Tail-call since we changed ships
        self.next()
    }
}

impl Ships {
    pub fn shipmod_by_name(&self, shipmod_name: &str) -> Option<&ShipMod> {
        let ship_name = ship_blueprint_name(shipmod_name);
        self.0
            .get(ship_name)
            .and_then(|ship| ship.shipmod_by_name(shipmod_name))
    }

    pub fn shipmod_iter(&self) -> impl Iterator<Item = &ShipMod> + '_ {
        let mut ship_iter = self.0.iter();
        let shipmod_iter = ship_iter.next().map(|(_, ship)| ship.mods.iter());
        ShipModIter {
            ship_iter,
            shipmod_iter,
        }
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for Ships {
    type Target = HashMap<String, Ship>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Determine the unmodified (blueprint) ship name for the given ship
pub fn ship_blueprint_name(ship_name: &str) -> &str {
    // Base case: Ships
    let base_name = if let Some(split_name) = ship_name.split_once('改') {
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
        "千代田甲" | "千代田航" => "千代田",
        "千歳甲" | "千歳航" => "千歳",
        "呂500" => "U-511",
        // Untested against data as I don't own them.
        // два is Russian for 二, so it should probably match /Гангут .*/...
        "Октябрьская революция" | "Гангут два" => "Гангут",
        "大鷹" => "春日丸",
        _ => base_name,
    }
}

/// Guess the ship remodel level based on its name
pub fn ship_remodel_level_guess(ship_name: &str) -> u16 {
    let (base_name, kai_name) = match ship_name.find('改') {
        None => (ship_name, ""),
        Some(index) => ship_name.split_at(index),
    };

    let base_level: u16 = match base_name {
        // Tested against data
        "龍鳳" => 1,
        "Верный" => 2,
        "Italia" => 1,
        "千代田甲" | "千歳甲" => 2,
        "千代田航" | "千歳航" => 3,
        "呂500" => 2,
        // Untested against data as I don't own them.
        "Октябрьская революция" => 1,
        "大鷹" => 1,
        _ => 0,
    };

    let kai_level: u16 = match kai_name {
        "" => 0,
        "改" => 1,
        "改二" => 2,
        // TODO: 改二＿ perhaps, for some future-proofing?
        "改三" | "改二甲" | "改二丁" | "改二乙" | "改二特" | "改二丙" => 3,
        _ => panic!("Unknown kai_level {kai_name}"),
    };

    base_level + kai_level
}

/// Report the number of blueprints and large-scale blueprints needed for each stage.
/// `stage` is 0-indexed, i.e. it's the cost to upgrade _from_ that level.
/// Generally based on ship type, so is not aware of which specific ships have 改二 or later mods.
fn ship_blueprint_costs(ship_name: &str, ship_type: &str, stage: u16) -> Option<(u16, u8)> {
    // Special ships.
    let stage_costs = match ship_name {
        // TODO: This is shipClassId 5 (ship class name 千歳型). The id is available in blueprints, and the name
        // is available in Character, Book, and Wiki data. Could we rely on always having at least one of those?
        // Notably, kekkon list does not list ship class information at all.
        // How _reliable_ is the Wiki here? Conversely, since we're matching _blueprint_ names, name-matching
        // is probably safest, but requires maintenance.
        "千歳" | "千代田" => vec![(3, 0), (4, 0), (5, 0), (6, 0), (8, 2)],
        // TODO: Wiki lists base as 春日丸級 and mods as 大鷹型; need to find shipClassId too
        // Confirm that this forms an actual ship series: 春日丸, 大鷹, 大鷹改.
        "春日丸" => vec![(3, 0), (5, 0)],
        _ => match ship_type {
            "駆逐艦" | "軽巡洋艦" | "潜水艦" => vec![(3, 0), (6, 1), (6, 3)],
            "戦艦" | "軽空母" | "正規空母" | "重巡洋艦" => {
                vec![(3, 0), (8, 2), (8, 4)]
            }
            _ => vec![(3, 0)],
        },
    };
    stage_costs
        .get(stage as usize)
        .map(|costs| (costs.0 as u16, costs.1 as u8))
}

impl Ships {
    /// Import a list of ships from the given datasource
    fn new(builder: ShipsBuilder) -> Result<Self> {
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

        let wiki_kansen_list = match builder.wiki_kansen_list {
            None => None,
            Some(reader) => Some(wikiwiki_jp_kancolle_a::read_kansen_table(reader)?),
        };

        let wiki_kaizou_kansen_list = match builder.wiki_kaizou_kansen_list {
            None => None,
            Some(reader) => Some(wikiwiki_jp_kancolle_a::read_kansen_table(reader)?),
        };

        // TODO: Can we precalculate capacity?
        // HACK: 500 is more than the wiki list (445), so it'll do for now. This is a temporary array anyway.
        let mut shipmods: HashMap<String, ShipMod> = HashMap::with_capacity(500);

        // TODO: Don't panic in case of duplicates, return an error.

        // Helper function for use with or_insert_with_key.
        let ship_inserter = |ship_name: &String| Ship::new(ship_name.clone());

        // Helper function for use with or_insert_with_key.
        let shipmod_inserter = |ship_name: &String| ShipMod::new(ship_name.clone());

        // The wiki lists should be complete. And there should be no overlaps.
        let wiki_iter: Option<Box<dyn Iterator<Item = KansenShip>>> =
            match (wiki_kansen_list, wiki_kaizou_kansen_list) {
                (None, None) => None,
                (Some(list), None) => Some(Box::new(list.into_iter())),
                (None, Some(list)) => Some(Box::new(list.into_iter())),
                (Some(left), Some(right)) => Some(Box::new(left.into_iter().chain(right))),
            };

        if let Some(wiki_iter) = wiki_iter {
            for wiki_row in wiki_iter {
                let ship = shipmods
                    .entry(wiki_row.ship_name.clone())
                    .or_insert_with_key(shipmod_inserter);
                match &mut ship.wiki_list_entry {
                    None => ship.wiki_list_entry = Some(wiki_row),
                    Some(_) => {
                        panic!("Duplicate wiki list entry for {}", wiki_row.ship_name)
                    }
                }
            }
        }

        // Kekkon list is a convenient source of distinct ship names, if we have it.
        if let Some(kekkonlist) = kekkonlist {
            for kekkon in kekkonlist.into_iter() {
                let ship = shipmods
                    .entry(kekkon.name.clone())
                    .or_insert_with_key(shipmod_inserter);
                match &mut ship.kekkon {
                    None => ship.kekkon = Some(kekkon),
                    Some(_) => {
                        panic!("Duplicate kekkon entry for {}", kekkon.name)
                    }
                };
            }
        };

        if let Some(characters) = characters {
            for character in characters.into_iter() {
                let ship = shipmods
                    .entry(character.ship_name.clone())
                    .or_insert_with_key(shipmod_inserter);
                match &mut ship.character {
                    None => ship.character = Some(character),
                    Some(_) => {
                        panic!("Duplicate character entry for {}", character.ship_name)
                    }
                };
            }
        }

        if let Some(book) = book {
            for book_ship in book.into_iter() {
                let (book_nonkai, book_kai) = book_ship.into_kai_split();

                if let Some(book_kai) = book_kai {
                    let ship = shipmods
                        .entry(book_kai.ship_name.clone())
                        .or_insert_with_key(shipmod_inserter);
                    match &mut ship.book {
                        None => ship.book = Some(book_kai),
                        Some(_) => {
                            panic!("Duplicate book entry for {}", book_kai.ship_name)
                        }
                    };
                }

                let ship = shipmods
                    .entry(book_nonkai.ship_name.clone())
                    .or_insert_with_key(shipmod_inserter);
                match &mut ship.book {
                    None => ship.book = Some(book_nonkai),
                    Some(_) => {
                        panic!("Duplicate book entry for {}", book_nonkai.ship_name)
                    }
                };
            }
        }

        if let Some(bplist) = bplist.as_ref() {
            for bp_ship in bplist.iter() {
                shipmods
                    .entry(bp_ship.ship_name.clone())
                    .or_insert_with_key(shipmod_inserter);
                // TODO: Fail if we have a duplicate blueprint.
            }
        }

        for ship in shipmods.values() {
            ship.validate()?
        }

        // TODO: Can we precalculate capacity more accurately?
        // HACK: 200 is more than the wiki table count (see test_ships_default_import),
        // so it'll do for now. We'll shrink-to-fit later, so being a little over is fine.
        let mut ships: HashMap<String, Ship> = HashMap::with_capacity(200);

        // First, distribute the blueprints.
        if let Some(bplist) = bplist {
            for bp_ship in bplist.into_iter() {
                let ship = ships
                    .entry(bp_ship.ship_name.clone())
                    .or_insert_with_key(ship_inserter);
                match &mut ship.blueprint {
                    None => ship.blueprint = Some(bp_ship),
                    Some(_) => {
                        panic!("Duplicate blueprint entry for {}", bp_ship.ship_name)
                    }
                }
            }
        }

        // Now distribute the ShipMods.
        for (modname, shipmod) in shipmods.into_iter() {
            let basename = ship_blueprint_name(&modname);
            let ship = ships
                .entry(basename.to_string())
                .or_insert_with_key(ship_inserter);
            ship.mods.push(shipmod);
        }

        // Validate, pack it down, we're done.

        for (_, ship) in ships.iter_mut() {
            ship.sort_ship_mods();
            ship.validate()?;
        }

        ships.shrink_to_fit();

        Ok(Ships(ships))
    }
}

/// A Kancolle Arcade shipgirl, covering all modification stages.
/// Only the name is reliably unique.
/// Many other fields may either surprisingly overlap, or are optional.
#[derive(Debug, Getters)]
pub struct Ship {
    /// Base ship name
    name: String,

    /// The Blueprint data for this Ship, if any blueprints are held.
    blueprint: Option<BlueprintShip>,

    /// The various mod stages of this shipgirl, in order of modification stage.
    /// If there are gaps in our data (which should be unlikely) they will not
    /// be visible here.
    mods: Vec<ShipMod>,
}

impl Ship {
    pub fn shipmod_by_name(&self, shipmod_name: &str) -> Option<&ShipMod> {
        self.mods
            .iter()
            .find(|shipmod| shipmod.name() == shipmod_name)
    }

    /// Reports the cost of buying the given remodel_level with blueprints and
    /// large-scale blueprints.
    /// You cannot buy remodel_level 0 ships, so that will always return None.
    /// Also returns None if the remodel_level is higher than the highest-known
    /// remodel_level for this ship.
    pub fn shipmod_blueprint_cost(&self, remodel_level: u16) -> Option<(u16, u8)> {
        if remodel_level == 0 {
            return None;
        }
        if self.mods.is_empty() {
            return None;
        }
        if self.mods.last().unwrap().remodel_level() < remodel_level {
            return None;
        }
        // Having a blueprint is the best data, otherwise check character data, tc_book data, and finally wiki data.
        // Generally the base_name should equal our ship_name, but not building that assumption in here.
        let (base_name, base_ship_type) = {
            if self.blueprint().is_some() {
                (
                    &self.blueprint().as_ref().unwrap().ship_name,
                    &self.blueprint().as_ref().unwrap().ship_type,
                )
            } else if self.mods()[0].character().is_some()
                && self.mods()[0].character().as_ref().unwrap().remodel_lv == 0
            {
                (
                    &self.mods()[0].character().as_ref().unwrap().ship_name,
                    &self.mods()[0].character().as_ref().unwrap().ship_type,
                )
            } else if self.mods()[0].book().is_some()
                && ship_remodel_level_guess(&self.mods()[0].book().as_ref().unwrap().ship_name) == 0
            {
                (
                    &self.mods()[0].book().as_ref().unwrap().ship_name,
                    &self.mods()[0].book().as_ref().unwrap().ship_type,
                )
            } else if self.mods()[0].wiki_list_entry().is_some()
                && ship_remodel_level_guess(
                    &self.mods()[0].wiki_list_entry().as_ref().unwrap().ship_name,
                ) == 0
            {
                (
                    &self.mods()[0].wiki_list_entry().as_ref().unwrap().ship_name,
                    &self.mods()[0].wiki_list_entry().as_ref().unwrap().ship_type,
                )
            } else {
                return None;
            }
        };
        ship_blueprint_costs(base_name, base_ship_type, remodel_level - 1)
    }

    fn new(name: String) -> Ship {
        Ship {
            name,
            blueprint: None,
            // Big enough for 千代田 and 千歳航, we'll shrink-to-fit when sorting.
            mods: Vec::with_capacity(6),
        }
    }

    /// Validate that the ShipMods match and are sorted correctly.
    fn validate(&self) -> Result<()> {
        if let Some(ref blueprint) = self.blueprint() {
            assert_eq!(self.name(), &blueprint.ship_name);
        }

        let mut last_remodel_level = match self.mods().first() {
            Some(shipmod) => shipmod.remodel_level(),
            None => 0,
        };

        let mut first = true;

        for shipmod in self.mods().iter() {
            assert_eq!(self.name(), ship_blueprint_name(shipmod.name()));
            if !first {
                assert!(shipmod.remodel_level() > last_remodel_level);
                last_remodel_level = shipmod.remodel_level();
            }
            first = false;

            if self.name() == shipmod.name() {
                assert_eq!(shipmod.remodel_level(), 0);
            }
        }

        Ok(())
    }

    /// Sort our ShipMods and pack the storage vector
    fn sort_ship_mods(&mut self) {
        self.mods.sort_by_key(|shipmod| shipmod.remodel_level());
        self.mods.shrink_to_fit();
    }
}

/// A Kancolle Arcade shipgirl at a particular modification stage
/// Only the name is reliably unique.
/// Many other fields may either surprisingly overlap, or are optional.
/// TODO: Replace derive_getters with hand-written getters that return Option<&T>
/// instead of &Option<T> so I can remove all the as_ref calls in callers.
#[derive(Debug, Getters)]
pub struct ShipMod {
    /// Full ship name
    name: String,

    // Everything below here is still in-flux as I shake out the API.
    // Some of these things might become references to data structures held
    // elsewhere, but I think that gets complicated in Rust? So we'll just
    // copy everything for now.
    /// The relevant entry in the player's picture book data
    book: Option<BookShip>,

    /// The relevant entry in the player's character list data
    character: Option<Character>,

    /// Any Kekkon Kakko Kari entry for this ship
    kekkon: Option<KekkonKakkoKari>,

    /// The kansen ship list entry for this ship from
    /// https://wikiwiki.jp/kancolle-a/
    wiki_list_entry: Option<KansenShip>,
}

impl ShipMod {
    pub fn remodel_level(&self) -> u16 {
        // Trivial if we have a character
        if let Some(character) = self.character() {
            return character.remodel_lv;
        }

        // Otherwise, we need to guess.
        ship_remodel_level_guess(self.name())
    }

    // TODO: More APIs, particulary when there's multiple sources of truth, and some are more trustworthy
    // than others.

    fn new(name: String) -> ShipMod {
        ShipMod {
            name,
            book: None,
            character: None,
            kekkon: None,
            wiki_list_entry: None,
        }
    }

    /// Validate the various data elements agree when present
    fn validate(&self) -> Result<()> {
        // TODO: We should error in the failure cases, not panic.

        if let Some(book) = self.book.as_ref() {
            assert_ne!(
                book.variation_num, 0,
                "Ship {} created from \"Unknown\" book entry {}",
                self.name, book.book_no
            );

            let normal_page = &book.card_list[0];
            assert_eq!(
                normal_page.variation_num_in_page, 3,
                "Unexpected variation count {} on normal page of {}",
                normal_page.variation_num_in_page, book.book_no
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
}

#[cfg(test)]
mod tests;
