use kancolle_a::ships::ShipsBuilder;

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

    // Per the wiki ship lists
    assert_eq!(ships.len(), 285 + 160);
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.kekkon().is_some())
            .count(),
        441
    );
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
    assert!(ships
        .iter()
        .all(|(_, ship)| ship.wiki_list_entry().is_some()));
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

    assert_eq!(ships.len(), 441);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
    assert!(ships
        .iter()
        .all(|(_, ship)| ship.wiki_list_entry().is_none()));
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

    assert_eq!(ships.len(), 135);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
    assert!(ships
        .iter()
        .all(|(_, ship)| ship.wiki_list_entry().is_none()));
    assert!(ships
        .iter()
        .all(|(_, ship)| ship.wiki_list_entry().is_none()));
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

    // 285 entries, 37 未取得, and of the remaining 248, 151 have two rows.
    assert_eq!(ships.len(), 248 + 151);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_some()));
    assert!(ships
        .iter()
        .all(|(_, ship)| ship.wiki_list_entry().is_none()));

    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| !*ship.book_secondrow())
            .count(),
        248
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| *ship.book_secondrow())
            .count(),
        151
    );
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

    assert_eq!(ships.len(), 377);
    assert!(ships.iter().all(|(_, ship)| ship.kekkon().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.blueprint().is_none()));
    assert!(ships.iter().all(|(_, ship)| ship.character().is_some()));
    assert!(ships.iter().all(|(_, ship)| ship.book().is_none()));
    assert!(ships
        .iter()
        .all(|(_, ship)| ship.wiki_list_entry().is_none()));
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

    // Per the wiki ship lists
    assert_eq!(ships.len(), 285 + 160);
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
        331
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && !*ship.book_secondrow())
            .count(),
        248
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.book().is_some() && *ship.book_secondrow())
            .count(),
        151
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.character().is_some())
            .count(),
        377
    );
    assert_eq!(
        ships
            .iter()
            .filter(|(_, ship)| ship.wiki_list_entry().is_some())
            .count(),
        285 + 160
    );

    let non_kekkon_ships: Vec<&str> = ships
        .iter()
        .filter(|(_, ship)| ship.kekkon().is_none())
        .map(|(name, _)| name.as_ref())
        .collect();

    assert_eq!(non_kekkon_ships.len(), 4);
    assert!(non_kekkon_ships.contains(&"Ranger"));
    assert!(non_kekkon_ships.contains(&"武蔵改二"));
    assert!(non_kekkon_ships.contains(&"Ranger改"));
    assert!(non_kekkon_ships.contains(&"時雨改三"));
}
