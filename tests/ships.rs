use kancolle_a::ships::ShipsBuilder;

// This is an integration test, so we're only using it against "current" data.

use lazy_static_include::*;

lazy_static_include_bytes! {
    TCBOOK => "tests/fixtures/latest/TcBook_info.json",
    KANMUSU => "tests/fixtures/latest/kanmusu_list.json",
    BPLIST => "tests/fixtures/latest/BlueprintList_info.json",
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
fn test_ships_kekkon_only_import() {
    let ships = ShipsBuilder::new()
        .kekkon_from_reader(KANMUSU.as_ref())
        .no_book()
        .no_blueprint()
        .build()
        .unwrap();

    assert_eq!(ships.len(), 441);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_blueprint_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .no_book()
        .blueprint_from_reader(BPLIST.as_ref())
        .build()
        .unwrap();

    assert_eq!(ships.len(), 133);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_book_only_import() {
    let ships = ShipsBuilder::new()
        .no_kekkon()
        .book_from_reader(TCBOOK.as_ref())
        .no_blueprint()
        .build()
        .unwrap();

    // 284 entries, 59 未取得, and of the remaining 225, 148 have two rows.
    assert_eq!(ships.len(), 225 + 148);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_some()));

    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| !*ship.book_secondrow())
            .count(),
        225
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| *ship.book_secondrow())
            .count(),
        148
    );
}

#[test]
fn test_ships_full_import() {
    let ships = ShipsBuilder::new()
        .kekkon_from_reader(KANMUSU.as_ref())
        .book_from_reader(TCBOOK.as_ref())
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
        322
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && !*ship.book_secondrow())
            .count(),
        225
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && *ship.book_secondrow())
            .count(),
        148
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
