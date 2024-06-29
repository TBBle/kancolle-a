use kancolle_a::ships::{DataSources, GlobalDataSource, Ships, UserDataSource};

// This is an integration test, so we're only using it against "current" data.

use lazy_static_include::*;

lazy_static_include_bytes! {
    TCBOOK => "tests/fixtures/2024-06-30/TcBook_info.json",
    KANMUSU => "tests/fixtures/2024-06-30/kanmusu_list.json",
    BPLIST => "tests/fixtures/2024-06-30/BlueprintList_info.json",
}

#[test]
fn test_ships_null_import() {
    let data_sources = DataSources {
        book: UserDataSource::None,
        blueprint: UserDataSource::None,
        kekkon: GlobalDataSource::Static,
    };
    let ships = Ships::new(data_sources).unwrap();

    assert_eq!(ships.len(), 0);
}

#[test]
fn test_ships_kekkon_only_import() {
    let mut kanmusu_bytes = KANMUSU.as_ref();
    let data_sources = DataSources {
        book: UserDataSource::None,
        blueprint: UserDataSource::None,
        kekkon: GlobalDataSource::FromReader(&mut kanmusu_bytes),
    };
    let ships = Ships::new(data_sources).unwrap();

    assert_eq!(ships.len(), 441);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_blueprint_only_import() {
    let mut blueprint_bytes = BPLIST.as_ref();
    let data_sources = DataSources {
        book: UserDataSource::None,
        blueprint: UserDataSource::FromReader(&mut blueprint_bytes),
        kekkon: GlobalDataSource::Static,
    };
    let ships = Ships::new(data_sources).unwrap();

    assert_eq!(ships.len(), 131);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
}

#[test]
fn test_ships_book_only_import() {
    let mut book_bytes = TCBOOK.as_ref();
    let data_sources = DataSources {
        book: UserDataSource::FromReader(&mut book_bytes),
        blueprint: UserDataSource::None,
        kekkon: GlobalDataSource::Static,
    };
    let ships = Ships::new(data_sources).unwrap();

    // 284 entries, 64 未取得, and of the remaining 220, 147 have two rows.
    assert_eq!(ships.len(), 220 + 147);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_some()));

    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| !*ship.book_secondrow())
            .count(),
        220
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| *ship.book_secondrow())
            .count(),
        147
    );
}

#[test]
fn test_ships_full_import() {
    let mut kanmusu_bytes = KANMUSU.as_ref();
    let mut blueprint_bytes = BPLIST.as_ref();
    let mut book_bytes = TCBOOK.as_ref();
    let data_sources = DataSources {
        book: UserDataSource::FromReader(&mut book_bytes),
        blueprint: UserDataSource::FromReader(&mut blueprint_bytes),
        kekkon: GlobalDataSource::FromReader(&mut kanmusu_bytes),
    };
    let ships = Ships::new(data_sources).unwrap();

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
        318
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && !*ship.book_secondrow())
            .count(),
        220
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && *ship.book_secondrow())
            .count(),
        147
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
