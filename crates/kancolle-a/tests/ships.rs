use kancolle_a::ships::{self, ShipsBuilder};

// This is an integration test, so we're only using it against "current" data.

use lazy_static_include::*;

lazy_static_include_bytes! {
    TCBOOK => "tests/fixtures/latest/TcBook_info.json",
    KANMUSU => "tests/fixtures/latest/kanmusu_list.json",
    BPLIST => "tests/fixtures/latest/BlueprintList_info.json",
    CHARLIST => "tests/fixtures/latest/CharacterList_info.json",
}

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

    // Regex against the ships table `^\|\d\d\d\|\d\|\[\[[^\]改甲航]*\]\]\|[^|]+\|` gives 197
    // Then there's 6 ships that are renamed per `ship_blueprint_name`.
    assert_eq!(ships.len(), 191);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    // Per the wiki ship lists `^\|\d\d\d\|\d\|`
    assert_eq!(ships.shipmod_iter().count(), 287 + 162);
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.kekkon().is_some())
            .count(),
        441
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

    // 441 entries in kanmusu_list.json, Regex `"name": ".*[改甲航].*",` hits 247, plus 6 renames
    assert_eq!(ships.len(), 441 - 247 - 6);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    assert_eq!(ships.shipmod_iter().count(), 441);
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

    assert_eq!(ships.len(), 159);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.mods().len() == 1));

    assert_eq!(ships.shipmod_iter().count(), 159);
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

    // Regex `"shipName": ".*[改甲航].*",` gives 82 book entries with modified names
    // Then there's 6 ships that are renamed per `ship_blueprint_name`, but 2 are not in my data.
    assert_eq!(ships.len(), 266 - 82 - 4);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    // 287 entries, 21 未取得, and of the remaining 266, per JSON Path query `$..cardList[0].variationNumInPage`, 153 have two rows.
    assert_eq!(ships.shipmod_iter().count(), 266 + 153);
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

    // Regex `"shipName": ".*[改甲航].*",` gives 219 characters with modified names
    // Then there's 6 ships that are renamed per `ship_blueprint_name`, but 2 are not in my data.
    assert_eq!(ships.len(), 403 - 219 - 4);
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));

    assert_eq!(ships.shipmod_iter().count(), 403);
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

    assert_eq!(ships.len(), 191);
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.blueprint().is_some())
            .count(),
        159
    );

    // Per the wiki ship lists `^\|\d\d\d\|\d\|`
    assert_eq!(ships.shipmod_iter().count(), 287 + 162);
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.kekkon().is_some())
            .count(),
        441
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.book().is_some())
            .count(),
        266 + 153
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.character().is_some())
            .count(),
        403
    );
    assert_eq!(
        ships
            .shipmod_iter()
            .filter(|ship| ship.wiki_list_entry().is_some())
            .count(),
        287 + 162
    );

    let non_kekkon_ships: Vec<&str> = ships
        .shipmod_iter()
        .filter(|ship| ship.kekkon().is_none())
        .map(|ship| ship.name().as_ref())
        .collect();

    assert_eq!(non_kekkon_ships.len(), 8);
    assert!(non_kekkon_ships.contains(&"Ranger"));
    assert!(non_kekkon_ships.contains(&"武蔵改二"));
    assert!(non_kekkon_ships.contains(&"Ranger改"));
    assert!(non_kekkon_ships.contains(&"時雨改三"));
    assert!(non_kekkon_ships.contains(&"Gambier Bay"));
    assert!(non_kekkon_ships.contains(&"Gambier Bay改"));
    assert!(non_kekkon_ships.contains(&"Iowa"));
    assert!(non_kekkon_ships.contains(&"Iowa改"));

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
    assert_eq!(unowned_ships.len(), 11);
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
