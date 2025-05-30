use kancolle_a::ships::{self, ShipsBuilder};

// This is an integration test, so we're only using it against "current" data.

use lazy_static_include::*;

lazy_static_include_bytes! {
    TCBOOK => "tests/fixtures/latest/TcBook_info.json",
    KANMUSU => "tests/fixtures/latest/kanmusu_list.json",
    BPLIST => "tests/fixtures/latest/BlueprintList_info.json",
    CHARLIST => "tests/fixtures/latest/CharacterList_info.json",
}

// crates\kancolle-a\src\importer\wikiwiki_jp_kancolle_a\kansen_table\艦船_テーブル.txt
// Regex `^\|\d\d\d\|\d\|\[\[[^\]改甲航]*\]\]\|[^|]+\|` gives 198
// Then there's 6 ships that are renamed per `ship_blueprint_name`.
const KANSEN_TABLE_SHIPS: usize = 192;
// Regex `^\|\d\d\d\|\d\|`
const KANSEN_TABLE_COUNT: usize = 290;

// crates\kancolle-a\src\importer\wikiwiki_jp_kancolle_a\kansen_table\改造艦船_テーブル.txt
// Regex `^\|\d\d\d\|\d\|`
const MODIFIED_KANSEN_TABLE_COUNT: usize = 163;

// crates\kancolle-a\tests\fixtures\latest\kanmusu_list.json
// 441 entries
const FIXTURE_KANMUSU_LIST_COUNT: usize = 441;
// Regex `"name": ".*[改甲航].*",` gives 247
// Then there's 6 ships that are renamed per `ship_blueprint_name`.
const FIXTURE_KANMUSU_LIST_SHIPS: usize = FIXTURE_KANMUSU_LIST_COUNT - 247 - 6;

// crates\kancolle-a\tests\fixtures\latest\BlueprintList_info.json
const FIXTURE_BLUEPRINT_LIST_COUNT: usize = 159;

// crates\kancolle-a\tests\fixtures\latest\TcBook_info.json
// 290 entries but 22 are 未取得
const FIXTURE_TCBOOK_KNOWN_COUNT: usize = 291 - 22;
// Regex `"shipName": ".*[改甲航].*",` gives 85 book entries with modified names
// Then there's 6 ships that are renamed per `ship_blueprint_name`, but 2 are not in my data.
const FIXTURE_TCBOOK_KNOWN_SHIPS: usize = FIXTURE_TCBOOK_KNOWN_COUNT - 85 - 4;
// JSON Path query `$..cardList[0].variationNumInPage`, reports 153 two-row (6 per page) ships
const FIXTURE_TCBOOK_KNOWN_SHIPMODS: usize = FIXTURE_TCBOOK_KNOWN_COUNT + 153;

// crates\kancolle-a\tests\fixtures\latest\CharacterList_info.json
const FIXTURE_CHARACTERS_COUNT: usize = 414;
// Regex `"shipName": ".*[改甲航].*",` gives 230 characters with modified names
// Then there's 6 ships that are renamed per `ship_blueprint_name`, but 2 are not in my data.
const FIXTURE_CHARACTERS_SHIPS: usize = FIXTURE_CHARACTERS_COUNT - 230 - 4;

#[tokio::test]
async fn test_ships_null_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .no_book()
        .no_blueprint()
        .build()
        .await
        .unwrap();

    assert_eq!(ships.len(), 0);
}

#[tokio::test]
async fn test_ships_default_import() {
    let ships = ShipsBuilder::default().build().await.unwrap();

    assert_eq!(ships.len(), KANSEN_TABLE_SHIPS);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    assert_eq!(
        ships.shipmod_iter().count(),
        KANSEN_TABLE_COUNT + MODIFIED_KANSEN_TABLE_COUNT
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.kekkon().is_some())
            .count(),
        FIXTURE_KANMUSU_LIST_COUNT
    );
    assert!(ships.shipmod_iter().all(|ship| ship.character().is_none()));
    assert!(ships.shipmod_iter().all(|ship| ship.book().is_none()));
    assert!(ships
        .shipmod_iter()
        .all(|ship| ship.wiki_list_entry().is_some()));
}

#[tokio::test]
async fn test_ships_kekkon_only_import() {
    let ships = ShipsBuilder::new()
        .kekkon_from_reader(KANMUSU.as_ref())
        .no_book()
        .no_character()
        .no_blueprint()
        .build()
        .await
        .unwrap();

    assert_eq!(ships.len(), FIXTURE_KANMUSU_LIST_SHIPS);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    assert_eq!(ships.shipmod_iter().count(), FIXTURE_KANMUSU_LIST_COUNT);
    assert!(ships.shipmod_iter().all(|ship| ship.kekkon().is_some()));
    assert!(ships.shipmod_iter().all(|ship| ship.character().is_none()));
    assert!(ships.shipmod_iter().all(|ship| ship.book().is_none()));
    assert!(ships
        .shipmod_iter()
        .all(|ship| ship.wiki_list_entry().is_none()));
}

#[tokio::test]
async fn test_ships_blueprint_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .no_book()
        .no_character()
        .blueprint_from_reader(BPLIST.as_ref())
        .build()
        .await
        .unwrap();

    assert_eq!(ships.len(), FIXTURE_BLUEPRINT_LIST_COUNT);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.mods().len() == 1));

    assert_eq!(ships.shipmod_iter().count(), FIXTURE_BLUEPRINT_LIST_COUNT);
    assert!(ships.shipmod_iter().all(|ship| ship.kekkon().is_none()));
    assert!(ships.shipmod_iter().all(|ship| ship.character().is_none()));
    assert!(ships.shipmod_iter().all(|ship| ship.book().is_none()));
    assert!(ships
        .shipmod_iter()
        .all(|ship| ship.wiki_list_entry().is_none()));
}

#[tokio::test]
async fn test_ships_book_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .book_from_reader(TCBOOK.as_ref())
        .no_character()
        .no_blueprint()
        .build()
        .await
        .unwrap();

    assert_eq!(ships.len(), FIXTURE_TCBOOK_KNOWN_SHIPS);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    assert_eq!(ships.shipmod_iter().count(), FIXTURE_TCBOOK_KNOWN_SHIPMODS);
    assert!(ships.shipmod_iter().all(|ship| ship.kekkon().is_none()));
    assert!(ships.shipmod_iter().all(|ship| ship.character().is_none()));
    assert!(ships.shipmod_iter().all(|ship| ship.book().is_some()));
    assert!(ships
        .shipmod_iter()
        .all(|ship| ship.wiki_list_entry().is_none()));
}

#[tokio::test]
async fn test_ships_characters_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .no_book()
        .character_from_reader(CHARLIST.as_ref())
        .no_blueprint()
        .build()
        .await
        .unwrap();

    assert_eq!(ships.len(), FIXTURE_CHARACTERS_SHIPS);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    assert_eq!(ships.shipmod_iter().count(), FIXTURE_CHARACTERS_COUNT);
    assert!(ships.shipmod_iter().all(|ship| ship.kekkon().is_none()));
    assert!(ships.shipmod_iter().all(|ship| ship.character().is_some()));
    assert!(ships.shipmod_iter().all(|ship| ship.book().is_none()));
    assert!(ships
        .shipmod_iter()
        .all(|ship| ship.wiki_list_entry().is_none()));

    // Opportunistic test for ship_remodel_level_guess
    assert!(ships
        .shipmod_iter()
        .all(|ship| ships::ship_remodel_level_guess(ship.name())
            == ship.character().as_ref().unwrap().remodel_lv));
}

#[tokio::test]
async fn test_ships_full_import() {
    let ships = ShipsBuilder::new()
        .kekkon_from_reader(KANMUSU.as_ref())
        .book_from_reader(TCBOOK.as_ref())
        .character_from_reader(CHARLIST.as_ref())
        .blueprint_from_reader(BPLIST.as_ref())
        .static_wiki_kansen_list()
        .static_wiki_kaizou_kansen_list()
        .build()
        .await
        .unwrap();

    assert_eq!(ships.len(), KANSEN_TABLE_SHIPS);
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.blueprint().is_some())
            .count(),
        FIXTURE_BLUEPRINT_LIST_COUNT
    );

    assert_eq!(
        ships.shipmod_iter().count(),
        KANSEN_TABLE_COUNT + MODIFIED_KANSEN_TABLE_COUNT
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.kekkon().is_some())
            .count(),
        FIXTURE_KANMUSU_LIST_COUNT
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.book().is_some())
            .count(),
        FIXTURE_TCBOOK_KNOWN_SHIPMODS
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.character().is_some())
            .count(),
        FIXTURE_CHARACTERS_COUNT
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.wiki_list_entry().is_some())
            .count(),
        KANSEN_TABLE_COUNT + MODIFIED_KANSEN_TABLE_COUNT
    );

    let non_kekkon_ships: Vec<&str> = ships
        .shipmod_iter()
        .filter(|ship| ship.kekkon().is_none())
        .map(|ship| ship.name().as_ref())
        .collect();

    assert_eq!(non_kekkon_ships.len(), 12);
    assert!(non_kekkon_ships.contains(&"Ranger"));
    assert!(non_kekkon_ships.contains(&"武蔵改二"));
    assert!(non_kekkon_ships.contains(&"Ranger改"));
    assert!(non_kekkon_ships.contains(&"時雨改三"));
    assert!(non_kekkon_ships.contains(&"Gambier Bay"));
    assert!(non_kekkon_ships.contains(&"Gambier Bay改"));
    assert!(non_kekkon_ships.contains(&"Iowa"));
    assert!(non_kekkon_ships.contains(&"Iowa改"));
    assert!(non_kekkon_ships.contains(&"矢矧改二"));
    assert!(non_kekkon_ships.contains(&"雪風改二"));
    assert!(non_kekkon_ships.contains(&"Janus"));
    assert!(non_kekkon_ships.contains(&"Janus改"));

    // Not really a test, more a record of the data in the integration tests.
    let unowned_ships: Vec<&str> = ships
        .iter()
        .filter_map(|(_, ship)| {
            if ship
                .mods()
                .iter()
                .all(|shipmod| shipmod.book().is_none() && shipmod.character().is_none())
            {
                Some(ship.name().as_ref())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(unowned_ships.len(), 12);
    assert!(unowned_ships.contains(&"Ark Royal"));
    assert!(unowned_ships.contains(&"Hornet"));
    assert!(unowned_ships.contains(&"伊14"));
    assert!(unowned_ships.contains(&"Ташкент"));
    assert!(unowned_ships.contains(&"神威"));
    assert!(unowned_ships.contains(&"Saratoga"));
    assert!(unowned_ships.contains(&"Commandant Teste"));
    assert!(unowned_ships.contains(&"伊13"));
    assert!(unowned_ships.contains(&"Atlanta"));
    assert!(unowned_ships.contains(&"Гангут"));
    assert!(unowned_ships.contains(&"Z3"));
    assert!(unowned_ships.contains(&"Janus"));

    // Validate our assumption that the Wiki ship_type for the mod-level 0 ShipMod
    // matches the ship_type in the Blueprint data.
    let blueprint_and_wiki_ships = ships.iter().filter(|(_, ship)| {
        ship.blueprint().is_some()
            && !ship.mods().is_empty()
            && ship.mods()[0].wiki_list_entry().is_some()
            && ship.mods()[0].remodel_level() == 0
    });
    for (_, ship) in blueprint_and_wiki_ships {
        let bp_ship = ship.blueprint().as_ref().unwrap();
        let base_wiki_ship = ship.mods()[0].wiki_list_entry().as_ref().unwrap();
        assert_eq!(bp_ship.ship_name, base_wiki_ship.ship_name);
        assert_eq!(bp_ship.ship_type, base_wiki_ship.ship_type);
    }
}
