use kancolle_a::ships::ShipsBuilder;

// This is an integration test, so we're only using it against "current" data.

use lazy_static_include::*;

lazy_static_include_bytes! {
    TCBOOK => "tests/fixtures/latest/TcBook_info.json",
    KANMUSU => "tests/fixtures/latest/kanmusu_list.json",
    BPLIST => "tests/fixtures/latest/BlueprintList_info.json",
    CHARLIST => "tests/fixtures/latest/CharacterList_info.json",
}

#[test]
fn test_ships_null_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .no_book()
        .no_blueprint()
        .build()
        .unwrap();

    assert_eq!(ships.len(), 0);
}

#[test]
fn test_ships_default_import() {
    let ships = ShipsBuilder::default().build().unwrap();

    assert_eq!(ships.len(), 441);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_kekkon_only_import() {
    let ships = ShipsBuilder::new()
        .kekkon_from_reader(KANMUSU.as_ref())
        .no_book()
        .no_character()
        .no_blueprint()
        .build()
        .unwrap();

    assert_eq!(ships.len(), 441);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_blueprint_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .no_book()
        .no_character()
        .blueprint_from_reader(BPLIST.as_ref())
        .build()
        .unwrap();

    assert_eq!(ships.len(), 133);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_book_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .book_from_reader(TCBOOK.as_ref())
        .no_character()
        .no_blueprint()
        .build()
        .unwrap();

    // 284 entries, 50 未取得, and of the remaining 234, 149 have two rows.
    assert_eq!(ships.len(), 234 + 149);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_some()));

    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| !*ship.book_secondrow())
            .count(),
        234
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| *ship.book_secondrow())
            .count(),
        149
    );
}

#[test]
fn test_ships_characters_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .no_book()
        .character_from_reader(CHARLIST.as_ref())
        .no_blueprint()
        .build()
        .unwrap();

    assert_eq!(ships.len(), 354);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_full_import() {
    let ships = ShipsBuilder::new()
        .kekkon_from_reader(KANMUSU.as_ref())
        .book_from_reader(TCBOOK.as_ref())
        .character_from_reader(CHARLIST.as_ref())
        .blueprint_from_reader(BPLIST.as_ref())
        .build()
        .unwrap();

    assert_eq!(ships.len(), 444);
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.kekkon().is_some())
            .count(),
        441
    );
    // TODO: Independently verify this... It's not much of a test if I just change the
    // value to be the failure case.
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.blueprint().is_some())
            .count(),
        324
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && !*ship.book_secondrow())
            .count(),
        234
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && *ship.book_secondrow())
            .count(),
        149
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.character().is_some())
            .count(),
        354
    );

    let non_kekkon_ships: Vec<&str> = ships
        .iter()
        .filter(|(_, ship)| ship.kekkon().is_none())
        .map(|(name, _)| name.as_ref())
        .collect();
    // I happen to have these ships from the most-recent event.
    assert_eq!(non_kekkon_ships.len(), 3);
    assert!(non_kekkon_ships.contains(&"Ranger"));
    assert!(non_kekkon_ships.contains(&"武蔵改二"));
    assert!(non_kekkon_ships.contains(&"Ranger改"));
}
