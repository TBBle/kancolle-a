use super::*;

use lazy_static_include::*;

// https://kancolle-arcade.net/ac/api/TcBook/info
lazy_static_include_bytes! {
    TCBOOK_2024_05_28 => "tests/fixtures/2024-05-28/TcBook_info.json",
    TCBOOK_2024_05_30 => "tests/fixtures/2024-05-30/TcBook_info.json",
    TCBOOK_2024_06_09 => "tests/fixtures/2024-06-09/TcBook_info.json",
    TCBOOK_2024_06_10 => "tests/fixtures/2024-06-10/TcBook_info.json",
    TCBOOK_2024_06_20 => "tests/fixtures/2024-06-20/TcBook_info.json",
    TCBOOK_2024_06_23 => "tests/fixtures/2024-06-23/TcBook_info.json",
    TCBOOK_2024_10_06 => "tests/fixtures/2024-10-06/TcBook_info.json",
    TCBOOK_LATEST => "tests/fixtures/latest/TcBook_info.json",
}

#[test]
fn test_book_ship_source() {
    let ship = BookShip {
        book_no: 6,
        ship_class: None,
        ship_class_index: None,
        ship_type: "".to_string(),
        ship_model_num: "".to_string(),
        ship_name: "".to_string(),
        card_index_img: "".to_string(),
        card_list: vec![
            BookShipCardPage {
                priority: 0,
                card_img_list: vec!["".to_string(), "".to_string()],
                status_img: None,
                variation_num_in_page: 3,
                acquire_num_in_page: 0,
            },
            BookShipCardPage {
                priority: 0,
                card_img_list: vec!["".to_string(), "".to_string()],
                status_img: None,
                variation_num_in_page: 3,
                acquire_num_in_page: 0,
            },
        ],
        variation_num: 6,
        acquire_num: 0,
        lv: 1,
        is_married: None,
        married_img: None,
    };

    use BookShipCardPageSource::*;
    assert_eq!(ship.source(0), Normal);
    assert_eq!(ship.source(1), SundayBest);
    assert_eq!(ship.source(2), Unknown);
}

#[test]
fn test_book_split_nokai_20240623() {
    let tcbook = read_tclist(TCBOOK_2024_06_23.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    let 神風改 = &tcbook[248];

    let (神風改_nonkai, 神風改_kai) = 神風改.clone().into_kai_split();
    assert!(神風改_kai.is_none());
    assert_eq!(神風改, &神風改_nonkai);
}

#[test]
fn parse_empty_tcbook_reader() {
    read_tclist(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_tcbook_vector() {
    let tcbook = read_tclist("[]".as_bytes()).unwrap();
    assert_eq!(tcbook.len(), 0);
}

// Split all ships in the given tcbook, and validate that as if we got it from the server.
fn validate_tcbook_split_common(tcbook: &TcBook) {
    // Big enough for everything... Should be no more than 445, see the Ship integration tests.
    let mut split_book: TcBook = Vec::with_capacity(500);
    for book_ship in tcbook.iter().cloned() {
        let (book_ship, kai_ship) = book_ship.into_kai_split();
        split_book.push(book_ship);
        if let Some(kai_ship) = kai_ship {
            split_book.push(kai_ship);
        }
    }
    validate_tcbook_common(&split_book);
}

fn validate_tcbook_common(tcbook: &TcBook) {
    const CARD_IMAGE_SUFFIX: &str = ".jpg";
    const STATUS_IMAGE_PREFIX: &str = "i/i_";
    const STATUS_IMAGE_SUFFIXES: [&str; 4] = ["_n.png", "_bs.png", "_bm.png", "_bl.png"];

    // Validating some dependent values to ensure we are making good assumptions

    for ship in tcbook.iter() {
        let _book_no = ship.book_no;
        eprintln!("Ship {_book_no}");
        let mut acquire_num = 0;
        let mut variation_num = 0;
        let mut normal_variation: u16 = 0;
        for (priority, card_page) in ship.card_list.iter().enumerate() {
            use BookShipCardPageSource::*;
            assert_eq!(card_page.priority as usize, priority);
            // First page is the 3 generic images, and possibly a second row if modified version has the same book number.
            // The other pages are for event variations, 3-image sets and then individual original illustrations last.
            // The image set pages are not always the same, e.g. 雪風 has no swimsuit set, but 雪風改 (same number) does.
            // But we should never see more cards on a page than on the first page.
            match ship.source(card_page.priority) {
                OriginalIllustration1(_) if normal_variation == 6 => {
                    assert_ne!(card_page.priority, 0);
                    assert_eq!(card_page.variation_num_in_page, 1);
                }
                OriginalIllustration1(_) => {
                    assert_ne!(card_page.priority, 0);
                    // <= because it may have been split. Ideally'd have removed this page in that case.
                    // Precisely validating variation_num_in_page of a split ship requires knowing whether
                    // this was the kai side of the original ship.
                    // TODO: Can we make source() return the right values for split ships? Same problem as
                    // removing pages for Swimsuit, really...
                    assert!(card_page.variation_num_in_page <= 1);
                }
                OriginalIllustration2(_, _) if normal_variation == 6 => {
                    assert_ne!(card_page.priority, 0);
                    assert_eq!(card_page.variation_num_in_page, 2);
                }
                OriginalIllustration2(_, _) => {
                    assert_ne!(card_page.priority, 0);
                    // <= because it may have been split. Ideally'd have removed this page in that case.
                    assert!(card_page.variation_num_in_page <= 2);
                }
                Normal => {
                    assert_eq!(card_page.priority, 0);
                    assert!(
                        card_page.variation_num_in_page == 3
                            || card_page.variation_num_in_page == 6
                    );
                    normal_variation = card_page.variation_num_in_page;
                }
                Unknown => {
                    // We can't assume this isn't an unmarked Original Illustration page.
                    assert_ne!(card_page.priority, 0);
                    assert!(card_page.variation_num_in_page <= normal_variation);
                }
                Swimsuit => {
                    // Special case of _: 雪風 after splitting ends up with a Swimsuit page with 0 entries.
                    // TODO: Drop this page or something.
                    assert_ne!(card_page.priority, 0);
                    assert!(
                        card_page.variation_num_in_page == 0
                            || card_page.variation_num_in_page == 3
                            || card_page.variation_num_in_page == 6
                    );
                    assert!(card_page.variation_num_in_page <= normal_variation);
                }
                _ => {
                    assert_ne!(card_page.priority, 0);
                    // let _pagecount = *card_page.variation_num_in_page;
                    // eprintln!("\tPagecount {_pagecount}");
                    assert!(
                        card_page.variation_num_in_page == 3
                            || card_page.variation_num_in_page == 6
                    );
                    assert!(card_page.variation_num_in_page <= normal_variation);
                }
            }

            assert_eq!(
                card_page.variation_num_in_page as usize,
                card_page.card_img_list.len()
            );
            assert_eq!(
                card_page.acquire_num_in_page as usize,
                card_page
                    .card_img_list
                    .iter()
                    .filter(|s| !s.is_empty())
                    .count()
            );
            acquire_num += card_page.acquire_num_in_page;
            variation_num += card_page.variation_num_in_page;
        }
        assert_eq!(ship.acquire_num, acquire_num);
        assert_eq!(ship.variation_num, variation_num);
    }

    // Fixed defaults for ships we haven't seen yet.
    let unowned_ships = tcbook.iter().filter(|ship| ship.ship_name == "未取得");
    for ship in unowned_ships {
        assert_ne!(ship.book_no, 0);
        assert_eq!(ship.ship_class.as_ref().unwrap(), "");
        assert_eq!(ship.ship_class_index.unwrap(), -1);
        assert_eq!(ship.ship_type, "");
        assert_eq!(ship.ship_model_num, "");
        assert_eq!(ship.ship_name, "未取得");
        assert_eq!(ship.card_index_img, "");
        assert_eq!(ship.card_list.len(), 0);
        assert_eq!(ship.variation_num, 0);
        assert_eq!(ship.acquire_num, 0);
        assert_eq!(ship.lv, 0);
        assert!(ship.is_married.is_none());
        assert!(ship.married_img.is_none());
    }

    // Universally-true facts about all ships we've scanned already.

    // ... And exceptions thereof.
    let ship_types_without_class = ["工作艦"]; // Unique repair ship

    // Ships with a model num also don't have an index, e.g., DD-445. Is that just for US ships?
    let ship_classes_without_index = [
        "特種船丙型",
        "三式潜航輸送艇",
        "潜特型(伊400型潜水艦)",
        "UボートIXC型",
        "改伊勢型",
        "呂号潜水艦",
    ];

    let owned_ships = tcbook.iter().filter(|ship| ship.ship_name != "未取得");
    for ship in owned_ships {
        let _book_no = ship.book_no;
        eprintln!("Ship {_book_no}");
        let card_image_prefix = format!("s/tc_{0}_", ship.book_no);
        assert_ne!(ship.book_no, 0);
        assert_ne!(ship.ship_type, ""); // Moved earlier because some things depend on this.

        // Some interactions between ship classes and models, which explains all the Option types here.
        if ship_types_without_class.contains(&&ship.ship_type[..]) {
            assert!(ship.ship_class.is_none());
            assert!(ship.ship_class_index.is_none());
            assert_eq!(ship.ship_model_num, "");
        } else {
            assert_ne!(ship.ship_class.as_ref().unwrap(), "");
            if !ship.ship_model_num.is_empty()
                || ship_classes_without_index.contains(&&ship.ship_class.as_ref().unwrap()[..])
            {
                assert!(ship.ship_class_index.is_none());
            } else {
                assert_ne!(ship.ship_class_index.unwrap(), -1)
            }
        }

        assert_ne!(ship.ship_name, "未取得");
        assert_ne!(ship.card_index_img, "");
        assert!(ship.card_index_img.starts_with(&card_image_prefix));
        assert!(ship.card_index_img.ends_with(CARD_IMAGE_SUFFIX));
        assert_ne!(ship.card_list.len(), 0);
        // Test for overshoot returning Unknown
        assert_eq!(
            ship.source(ship.card_list.len() as u16),
            BookShipCardPageSource::Unknown
        );
        assert_ne!(ship.variation_num, 0);
        // Can't assert this, a split ship may not have both parts acquired.
        // assert_ne!(ship.acquire_num, 0);
        assert_ne!(ship.lv, 0);
        assert!(ship.is_married.is_some());
        assert!(ship.married_img.is_some());
        // The isMarried array is repeated for each page except original illustrations.
        let populated_image_pages = ship
            .card_list
            .iter()
            .filter(|page| {
                !matches!(
                    ship.source(page.priority),
                    BookShipCardPageSource::OriginalIllustration1(_)
                        | BookShipCardPageSource::OriginalIllustration2(_, _)
                )
            })
            .count();
        let expected_married_imgs = ship
            .is_married
            .as_ref()
            .unwrap()
            .iter()
            .filter(|is_married| **is_married)
            .count();
        assert_eq!(expected_married_imgs % populated_image_pages, 0);
        assert_eq!(
            ship.married_img.as_ref().unwrap().len(),
            expected_married_imgs / populated_image_pages
        );
        // Being married makes the level range 100..=175 rather than 1..=99. However,
        // we can't validate that here because marrying any renovation
        // unlocks level 100+ for all renovations, including different book-numbers.

        for card_list_page in ship.card_list.iter() {
            for card_image in card_list_page
                .card_img_list
                .iter()
                .filter(|s| !s.is_empty())
            {
                assert!(card_image.starts_with(&card_image_prefix));
                assert!(card_image.ends_with(CARD_IMAGE_SUFFIX));
            }

            // Empty page: Empty status image list for normal page or Original Illustation, no
            // status image list used otherwise.
            // TODO: Original Illustation status icon list should always match Normal list.
            use BookShipCardPageSource::*;
            if card_list_page.status_img.is_none() {
                assert!(match ship.source(card_list_page.priority) {
                    Unknown => true, // We can't assume anything...
                    Normal | OriginalIllustration1(_) | OriginalIllustration2(_, _) => false,
                    _ => true,
                });
                continue;
            } else if card_list_page.status_img.as_ref().unwrap().is_empty() {
                assert!(match ship.source(card_list_page.priority) {
                    Unknown => true, // We can't assume anything...
                    Normal | OriginalIllustration1(_) | OriginalIllustration2(_, _) => true,
                    _ => false,
                });
                continue;
            }

            for status_image in card_list_page
                .status_img
                .as_ref()
                .unwrap()
                .iter()
                .filter(|s| !s.is_empty())
            {
                assert!(status_image.starts_with(STATUS_IMAGE_PREFIX));
                assert!(
                    status_image.ends_with(STATUS_IMAGE_SUFFIXES[0])
                        || status_image.ends_with(STATUS_IMAGE_SUFFIXES[1])
                        || status_image.ends_with(STATUS_IMAGE_SUFFIXES[2])
                        || status_image.ends_with(STATUS_IMAGE_SUFFIXES[3])
                );
            }
        }
    }
}

#[test]
fn parse_fixture_tcbook_info_20240528() {
    let tcbook = read_tclist(TCBOOK_2024_05_28.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    validate_tcbook_common(&tcbook);

    // Specific interesting ships.

    // I'm sure there's a number of reasons not to name variables like this.
    let 長門 = &tcbook[0];
    assert_eq!(長門.book_no, 1);
    assert_eq!(長門.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門.ship_class_index.unwrap(), 1);
    assert_eq!(長門.ship_type, "戦艦");
    assert_eq!(長門.ship_model_num, "");
    assert_eq!(長門.ship_name, "長門");
    assert_eq!(長門.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門.source(0), Normal);
    }
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 2);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &長門.card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &長門_card_list_0.card_img_list,
        &vec![
            "s/tc_1_d7ju63kolamj.jpg",
            "",
            "",
            "s/tc_1_2wp6daq4fn42.jpg",
            "",
            ""
        ]
    );
    assert_eq!(長門_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        長門_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png", "i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_card_list_0.variation_num_in_page, 6);
    assert_eq!(長門_card_list_0.acquire_num_in_page, 2);

    let 扶桑 = &tcbook[25];
    assert_eq!(扶桑.book_no, 26);
    assert_eq!(扶桑.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑.ship_type, "戦艦");
    assert_eq!(扶桑.ship_model_num, "");
    assert_eq!(扶桑.ship_name, "扶桑");
    assert_eq!(扶桑.card_index_img, "s/tc_26_p9u490qtc1a4.jpg");
    assert_eq!(扶桑.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(扶桑.source(0), Normal);
    }
    assert_eq!(扶桑.variation_num, 6);
    assert_eq!(扶桑.acquire_num, 4);
    assert_eq!(扶桑.lv, 98);
    assert_eq!(扶桑.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(扶桑.married_img.as_ref().unwrap().len(), 0);
    let 扶桑_card_list_0 = &扶桑.card_list[0];
    assert_eq!(扶桑_card_list_0.priority, 0);
    assert_eq!(扶桑_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_0.card_img_list,
        &vec![
            "s/tc_26_p9u490qtc1a4.jpg",
            "",
            "",
            "s/tc_26_fskeangzj9cz.jpg",
            "s/tc_26_z2jfdzutu1j3.jpg",
            "s/tc_26_krjdrps6k23r.jpg"
        ]
    );
    assert_eq!(扶桑_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_0.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_0.acquire_num_in_page, 4);

    // Interesting as it gained a new card page before its last page.
    let 早霜 = &tcbook[198];
    assert_eq!(早霜.book_no, 209);
    assert_eq!(早霜.ship_class.as_ref().unwrap(), "夕雲型");
    assert_eq!(早霜.ship_class_index.unwrap(), 17);
    assert_eq!(早霜.ship_type, "駆逐艦");
    assert_eq!(早霜.ship_model_num, "");
    assert_eq!(早霜.ship_name, "早霜");
    assert_eq!(早霜.card_index_img, "s/tc_209_6uqm0rr6azd9.jpg");
    assert_eq!(早霜.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), Unknown /*OriginalIllustration1(true)*/);
    }
    assert_eq!(早霜.variation_num, 7);
    assert_eq!(早霜.acquire_num, 1);
    assert_eq!(早霜.lv, 1);
    assert_eq!(早霜.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(早霜.married_img.as_ref().unwrap().len(), 0);
    let 早霜_card_list_0 = &早霜.card_list[0];
    assert_eq!(早霜_card_list_0.priority, 0);
    assert_eq!(早霜_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_0.card_img_list,
        &vec!["s/tc_209_6uqm0rr6azd9.jpg", "", "", "", "", "",]
    );
    assert_eq!(早霜_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_0.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_0.acquire_num_in_page, 1);
    let 早霜_card_list_1 = &早霜.card_list[1];
    assert_eq!(早霜_card_list_1.priority, 1);
    assert_eq!(早霜_card_list_1.card_img_list.len(), 1);
    assert_eq!(&早霜_card_list_1.card_img_list, &vec!["",]);
    assert_eq!(早霜_card_list_1.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_1.variation_num_in_page, 1);
    assert_eq!(早霜_card_list_1.acquire_num_in_page, 0);

    let 扶桑改二 = &tcbook[200];
    assert_eq!(扶桑改二.book_no, 211);
    assert_eq!(扶桑改二.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑改二.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑改二.ship_type, "航空戦艦");
    assert_eq!(扶桑改二.ship_model_num, "");
    assert_eq!(扶桑改二.ship_name, "扶桑改二");
    assert_eq!(扶桑改二.card_index_img, "s/tc_211_xkrpspyq72qz.jpg");
    assert_eq!(扶桑改二.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
    }
    assert_eq!(扶桑改二.variation_num, 3);
    assert_eq!(扶桑改二.acquire_num, 1);
    assert_eq!(扶桑改二.lv, 98);
    assert_eq!(扶桑改二.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(扶桑改二.married_img.as_ref().unwrap().len(), 0);
    let 扶桑改二_card_list_0 = &扶桑改二.card_list[0];
    assert_eq!(扶桑改二_card_list_0.priority, 0);
    assert_eq!(扶桑改二_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &扶桑改二_card_list_0.card_img_list,
        &vec!["s/tc_211_xkrpspyq72qz.jpg", "", "",]
    );
    assert_eq!(扶桑改二_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_rpyd1nnecq4w_n.png"]
    );
    assert_eq!(扶桑改二_card_list_0.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_0.acquire_num_in_page, 1);
}

#[test]
fn parse_fixture_tcbook_info_20240530() {
    let tcbook = read_tclist(TCBOOK_2024_05_30.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    validate_tcbook_common(&tcbook);

    // Specific interesting ships.

    // I'm sure there's a number of reasons not to name variables like this.
    let 長門 = &tcbook[0];
    assert_eq!(長門.book_no, 1);
    assert_eq!(長門.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門.ship_class_index.unwrap(), 1);
    assert_eq!(長門.ship_type, "戦艦");
    assert_eq!(長門.ship_model_num, "");
    assert_eq!(長門.ship_name, "長門");
    assert_eq!(長門.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門.source(0), Normal);
    }
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 2);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &長門.card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &長門_card_list_0.card_img_list,
        &vec![
            "s/tc_1_d7ju63kolamj.jpg",
            "",
            "",
            "s/tc_1_2wp6daq4fn42.jpg",
            "",
            ""
        ]
    );
    assert_eq!(長門_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        長門_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png", "i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_card_list_0.variation_num_in_page, 6);
    assert_eq!(長門_card_list_0.acquire_num_in_page, 2);

    // Interesting because I hit level 99 and triggered 扶桑改's ケッコンカッコカリ
    let 扶桑 = &tcbook[25];
    assert_eq!(扶桑.book_no, 26);
    assert_eq!(扶桑.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑.ship_type, "戦艦");
    assert_eq!(扶桑.ship_model_num, "");
    assert_eq!(扶桑.ship_name, "扶桑");
    assert_eq!(扶桑.card_index_img, "s/tc_26_p9u490qtc1a4.jpg");
    assert_eq!(扶桑.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(扶桑.source(0), Normal);
    }
    assert_eq!(扶桑.variation_num, 6);
    assert_eq!(扶桑.acquire_num, 4);
    assert_eq!(扶桑.lv, 100);
    assert_eq!(扶桑.is_married.as_ref().unwrap(), &vec![false, true]);
    assert_eq!(扶桑.married_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑.married_img.as_ref().unwrap(),
        &vec!["s/tc_26_tg21e17c6cre.jpg"]
    );
    let 扶桑_card_list_0 = &扶桑.card_list[0];
    assert_eq!(扶桑_card_list_0.priority, 0);
    assert_eq!(扶桑_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_0.card_img_list,
        &vec![
            "s/tc_26_p9u490qtc1a4.jpg",
            "",
            "",
            "s/tc_26_fskeangzj9cz.jpg",
            "s/tc_26_z2jfdzutu1j3.jpg",
            "s/tc_26_krjdrps6k23r.jpg"
        ]
    );
    assert_eq!(扶桑_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_0.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_0.acquire_num_in_page, 4);

    // Interesting as it gained a new card page before its last page.
    let 早霜 = &tcbook[198];
    assert_eq!(早霜.book_no, 209);
    assert_eq!(早霜.ship_class.as_ref().unwrap(), "夕雲型");
    assert_eq!(早霜.ship_class_index.unwrap(), 17);
    assert_eq!(早霜.ship_type, "駆逐艦");
    assert_eq!(早霜.ship_model_num, "");
    assert_eq!(早霜.ship_name, "早霜");
    assert_eq!(早霜.card_index_img, "s/tc_209_6uqm0rr6azd9.jpg");
    assert_eq!(早霜.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), Unknown /*OriginalIllustration1(true)*/);
    }
    assert_eq!(早霜.variation_num, 7);
    assert_eq!(早霜.acquire_num, 1);
    assert_eq!(早霜.lv, 1);
    assert_eq!(早霜.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(早霜.married_img.as_ref().unwrap().len(), 0);
    let 早霜_card_list_0 = &早霜.card_list[0];
    assert_eq!(早霜_card_list_0.priority, 0);
    assert_eq!(早霜_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_0.card_img_list,
        &vec!["s/tc_209_6uqm0rr6azd9.jpg", "", "", "", "", "",]
    );
    assert_eq!(早霜_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_0.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_0.acquire_num_in_page, 1);
    let 早霜_card_list_1 = &早霜.card_list[1];
    assert_eq!(早霜_card_list_1.priority, 1);
    assert_eq!(早霜_card_list_1.card_img_list.len(), 1);
    assert_eq!(&早霜_card_list_1.card_img_list, &vec!["",]);
    assert_eq!(早霜_card_list_1.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_1.variation_num_in_page, 1);
    assert_eq!(早霜_card_list_1.acquire_num_in_page, 0);

    let 扶桑改二 = &tcbook[200];
    assert_eq!(扶桑改二.book_no, 211);
    assert_eq!(扶桑改二.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑改二.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑改二.ship_type, "航空戦艦");
    assert_eq!(扶桑改二.ship_model_num, "");
    assert_eq!(扶桑改二.ship_name, "扶桑改二");
    assert_eq!(扶桑改二.card_index_img, "s/tc_211_xkrpspyq72qz.jpg");
    assert_eq!(扶桑改二.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
    }
    assert_eq!(扶桑改二.variation_num, 3);
    assert_eq!(扶桑改二.acquire_num, 1);
    assert_eq!(扶桑改二.lv, 100);
    assert_eq!(扶桑改二.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(扶桑改二.married_img.as_ref().unwrap().len(), 0);
    let 扶桑改二_card_list_0 = &扶桑改二.card_list[0];
    assert_eq!(扶桑改二_card_list_0.priority, 0);
    assert_eq!(扶桑改二_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &扶桑改二_card_list_0.card_img_list,
        &vec!["s/tc_211_xkrpspyq72qz.jpg", "", "",]
    );
    assert_eq!(扶桑改二_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_rpyd1nnecq4w_n.png"]
    );
    assert_eq!(扶桑改二_card_list_0.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_0.acquire_num_in_page, 1);
}

#[test]
fn parse_fixture_tcbook_info_20240609() {
    let tcbook = read_tclist(TCBOOK_2024_06_09.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    validate_tcbook_common(&tcbook);

    // Specific interesting ships.

    // I'm sure there's a number of reasons not to name variables like this.
    let 長門 = &tcbook[0];
    assert_eq!(長門.book_no, 1);
    assert_eq!(長門.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門.ship_class_index.unwrap(), 1);
    assert_eq!(長門.ship_type, "戦艦");
    assert_eq!(長門.ship_model_num, "");
    assert_eq!(長門.ship_name, "長門");
    assert_eq!(長門.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門.source(0), Normal);
    }
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 2);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &長門.card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &長門_card_list_0.card_img_list,
        &vec![
            "s/tc_1_d7ju63kolamj.jpg",
            "",
            "",
            "s/tc_1_2wp6daq4fn42.jpg",
            "",
            ""
        ]
    );
    assert_eq!(長門_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        長門_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png", "i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_card_list_0.variation_num_in_page, 6);
    assert_eq!(長門_card_list_0.acquire_num_in_page, 2);

    // Interesting because I hit level 99 and triggered 扶桑改's ケッコンカッコカリ
    let 扶桑 = &tcbook[25];
    assert_eq!(扶桑.book_no, 26);
    assert_eq!(扶桑.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑.ship_type, "戦艦");
    assert_eq!(扶桑.ship_model_num, "");
    assert_eq!(扶桑.ship_name, "扶桑");
    assert_eq!(扶桑.card_index_img, "s/tc_26_p9u490qtc1a4.jpg");
    assert_eq!(扶桑.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(扶桑.source(0), Normal);
        assert_eq!(扶桑.source(1), RainySeason);
    }
    assert_eq!(扶桑.variation_num, 12);
    assert_eq!(扶桑.acquire_num, 5);
    assert_eq!(扶桑.lv, 105);
    assert_eq!(
        扶桑.is_married.as_ref().unwrap(),
        &vec![false, true, false, true]
    );
    assert_eq!(扶桑.married_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑.married_img.as_ref().unwrap(),
        &vec!["s/tc_26_tg21e17c6cre.jpg"]
    );
    let 扶桑_card_list_0 = &扶桑.card_list[0];
    assert_eq!(扶桑_card_list_0.priority, 0);
    assert_eq!(扶桑_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_0.card_img_list,
        &vec![
            "s/tc_26_p9u490qtc1a4.jpg",
            "",
            "",
            "s/tc_26_fskeangzj9cz.jpg",
            "s/tc_26_z2jfdzutu1j3.jpg",
            "s/tc_26_krjdrps6k23r.jpg"
        ]
    );
    assert_eq!(扶桑_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_0.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_0.acquire_num_in_page, 4);
    let 扶桑_card_list_1 = &扶桑.card_list[1];
    assert_eq!(扶桑_card_list_1.priority, 1);
    assert_eq!(扶桑_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_1.card_img_list,
        &vec!["", "", "", "s/tc_26_46s6pg02mm41.jpg", "", ""]
    );
    assert_eq!(扶桑_card_list_1.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_1.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_1.acquire_num_in_page, 1);

    // Interesting as it gained a new card page before its last page.
    let 早霜 = &tcbook[198];
    assert_eq!(早霜.book_no, 209);
    assert_eq!(早霜.ship_class.as_ref().unwrap(), "夕雲型");
    assert_eq!(早霜.ship_class_index.unwrap(), 17);
    assert_eq!(早霜.ship_type, "駆逐艦");
    assert_eq!(早霜.ship_model_num, "");
    assert_eq!(早霜.ship_name, "早霜");
    assert_eq!(早霜.card_index_img, "s/tc_209_6uqm0rr6azd9.jpg");
    assert_eq!(早霜.card_list.len(), 3);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
        assert_eq!(早霜.source(2), OriginalIllustration1(true));
    }
    assert_eq!(早霜.variation_num, 13);
    assert_eq!(早霜.acquire_num, 1);
    assert_eq!(早霜.lv, 1);
    assert_eq!(早霜.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(早霜.married_img.as_ref().unwrap().len(), 0);
    let 早霜_card_list_0 = &早霜.card_list[0];
    assert_eq!(早霜_card_list_0.priority, 0);
    assert_eq!(早霜_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_0.card_img_list,
        &vec!["s/tc_209_6uqm0rr6azd9.jpg", "", "", "", "", "",]
    );
    assert_eq!(早霜_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_0.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_0.acquire_num_in_page, 1);
    let 早霜_card_list_1 = &早霜.card_list[1];
    assert_eq!(早霜_card_list_1.priority, 1);
    assert_eq!(早霜_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_1.card_img_list,
        &vec!["", "", "", "", "", "",]
    );
    assert!(早霜_card_list_1.status_img.is_none());
    assert_eq!(早霜_card_list_1.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_1.acquire_num_in_page, 0);
    let 早霜_card_list_2 = &早霜.card_list[2];
    assert_eq!(早霜_card_list_2.priority, 2);
    assert_eq!(早霜_card_list_2.card_img_list.len(), 1);
    assert_eq!(&早霜_card_list_2.card_img_list, &vec!["",]);
    assert_eq!(早霜_card_list_2.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_2.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_2.variation_num_in_page, 1);
    assert_eq!(早霜_card_list_2.acquire_num_in_page, 0);

    let 扶桑改二 = &tcbook[200];
    assert_eq!(扶桑改二.book_no, 211);
    assert_eq!(扶桑改二.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑改二.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑改二.ship_type, "航空戦艦");
    assert_eq!(扶桑改二.ship_model_num, "");
    assert_eq!(扶桑改二.ship_name, "扶桑改二");
    assert_eq!(扶桑改二.card_index_img, "s/tc_211_xkrpspyq72qz.jpg");
    assert_eq!(扶桑改二.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
    }
    assert_eq!(扶桑改二.variation_num, 6);
    assert_eq!(扶桑改二.acquire_num, 1);
    assert_eq!(扶桑改二.lv, 105);
    assert_eq!(扶桑改二.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(扶桑改二.married_img.as_ref().unwrap().len(), 0);
    let 扶桑改二_card_list_0 = &扶桑改二.card_list[0];
    assert_eq!(扶桑改二_card_list_0.priority, 0);
    assert_eq!(扶桑改二_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &扶桑改二_card_list_0.card_img_list,
        &vec!["s/tc_211_xkrpspyq72qz.jpg", "", "",]
    );
    assert_eq!(扶桑改二_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_rpyd1nnecq4w_n.png"]
    );
    assert_eq!(扶桑改二_card_list_0.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_0.acquire_num_in_page, 1);
    let 扶桑改二_card_list_1 = &扶桑改二.card_list[1];
    assert_eq!(扶桑改二_card_list_1.priority, 1);
    assert_eq!(扶桑改二_card_list_1.card_img_list.len(), 3);
    assert_eq!(&扶桑改二_card_list_1.card_img_list, &vec!["", "", "",]);
    assert!(扶桑改二_card_list_1.status_img.is_none());
    assert_eq!(扶桑改二_card_list_1.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_1.acquire_num_in_page, 0);
}

#[test]
fn parse_fixture_tcbook_info_20240610() {
    let tcbook = read_tclist(TCBOOK_2024_06_10.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    validate_tcbook_common(&tcbook);

    // Specific interesting ships.

    // I'm sure there's a number of reasons not to name variables like this.
    let 長門 = &tcbook[0];
    assert_eq!(長門.book_no, 1);
    assert_eq!(長門.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門.ship_class_index.unwrap(), 1);
    assert_eq!(長門.ship_type, "戦艦");
    assert_eq!(長門.ship_model_num, "");
    assert_eq!(長門.ship_name, "長門");
    assert_eq!(長門.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門.source(0), Normal);
    }
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 2);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &長門.card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &長門_card_list_0.card_img_list,
        &vec![
            "s/tc_1_d7ju63kolamj.jpg",
            "",
            "",
            "s/tc_1_2wp6daq4fn42.jpg",
            "",
            ""
        ]
    );
    assert_eq!(長門_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        長門_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png", "i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_card_list_0.variation_num_in_page, 6);
    assert_eq!(長門_card_list_0.acquire_num_in_page, 2);

    // Interesting because I hit level 99 and triggered 扶桑改's ケッコンカッコカリ
    let 扶桑 = &tcbook[25];
    assert_eq!(扶桑.book_no, 26);
    assert_eq!(扶桑.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑.ship_type, "戦艦");
    assert_eq!(扶桑.ship_model_num, "");
    assert_eq!(扶桑.ship_name, "扶桑");
    assert_eq!(扶桑.card_index_img, "s/tc_26_p9u490qtc1a4.jpg");
    assert_eq!(扶桑.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(扶桑.source(0), Normal);
        assert_eq!(扶桑.source(1), RainySeason);
    }
    assert_eq!(扶桑.variation_num, 12);
    assert_eq!(扶桑.acquire_num, 5);
    assert_eq!(扶桑.lv, 105);
    assert_eq!(
        扶桑.is_married.as_ref().unwrap(),
        &vec![false, true, false, true]
    );
    assert_eq!(扶桑.married_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑.married_img.as_ref().unwrap(),
        &vec!["s/tc_26_tg21e17c6cre.jpg"]
    );
    let 扶桑_card_list_0 = &扶桑.card_list[0];
    assert_eq!(扶桑_card_list_0.priority, 0);
    assert_eq!(扶桑_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_0.card_img_list,
        &vec![
            "s/tc_26_p9u490qtc1a4.jpg",
            "",
            "",
            "s/tc_26_fskeangzj9cz.jpg",
            "s/tc_26_z2jfdzutu1j3.jpg",
            "s/tc_26_krjdrps6k23r.jpg"
        ]
    );
    assert_eq!(扶桑_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_0.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_0.acquire_num_in_page, 4);
    let 扶桑_card_list_1 = &扶桑.card_list[1];
    assert_eq!(扶桑_card_list_1.priority, 1);
    assert_eq!(扶桑_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_1.card_img_list,
        &vec!["", "", "", "s/tc_26_46s6pg02mm41.jpg", "", ""]
    );
    assert_eq!(扶桑_card_list_1.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_1.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_1.acquire_num_in_page, 1);

    // Interesting as it gained a new card page before its last page.
    let 早霜 = &tcbook[198];
    assert_eq!(早霜.book_no, 209);
    assert_eq!(早霜.ship_class.as_ref().unwrap(), "夕雲型");
    assert_eq!(早霜.ship_class_index.unwrap(), 17);
    assert_eq!(早霜.ship_type, "駆逐艦");
    assert_eq!(早霜.ship_model_num, "");
    assert_eq!(早霜.ship_name, "早霜");
    assert_eq!(早霜.card_index_img, "s/tc_209_6uqm0rr6azd9.jpg");
    assert_eq!(早霜.card_list.len(), 3);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
        assert_eq!(早霜.source(2), OriginalIllustration1(true));
    }
    assert_eq!(早霜.variation_num, 13);
    assert_eq!(早霜.acquire_num, 2);
    assert_eq!(早霜.lv, 1);
    assert_eq!(
        早霜.is_married.as_ref().unwrap(),
        &vec![false, false, false, false]
    );
    assert_eq!(早霜.married_img.as_ref().unwrap().len(), 0);
    let 早霜_card_list_0 = &早霜.card_list[0];
    assert_eq!(早霜_card_list_0.priority, 0);
    assert_eq!(早霜_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_0.card_img_list,
        &vec!["s/tc_209_6uqm0rr6azd9.jpg", "", "", "", "", "",]
    );
    assert_eq!(早霜_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_0.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_0.acquire_num_in_page, 1);
    let 早霜_card_list_1 = &早霜.card_list[1];
    assert_eq!(早霜_card_list_1.priority, 1);
    assert_eq!(早霜_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_1.card_img_list,
        &vec!["", "", "", "s/tc_209_qt3tt1rukzxr.jpg", "", "",]
    );
    assert_eq!(早霜_card_list_1.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_zp6ze49mx4qw_n.png"]
    );
    assert_eq!(早霜_card_list_1.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_1.acquire_num_in_page, 1);
    let 早霜_card_list_2 = &早霜.card_list[2];
    assert_eq!(早霜_card_list_2.priority, 2);
    assert_eq!(早霜_card_list_2.card_img_list.len(), 1);
    assert_eq!(&早霜_card_list_2.card_img_list, &vec!["",]);
    assert_eq!(早霜_card_list_2.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_2.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_2.variation_num_in_page, 1);
    assert_eq!(早霜_card_list_2.acquire_num_in_page, 0);

    let 扶桑改二 = &tcbook[200];
    assert_eq!(扶桑改二.book_no, 211);
    assert_eq!(扶桑改二.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑改二.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑改二.ship_type, "航空戦艦");
    assert_eq!(扶桑改二.ship_model_num, "");
    assert_eq!(扶桑改二.ship_name, "扶桑改二");
    assert_eq!(扶桑改二.card_index_img, "s/tc_211_xkrpspyq72qz.jpg");
    assert_eq!(扶桑改二.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
    }
    assert_eq!(扶桑改二.variation_num, 6);
    assert_eq!(扶桑改二.acquire_num, 1);
    assert_eq!(扶桑改二.lv, 105);
    assert_eq!(扶桑改二.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(扶桑改二.married_img.as_ref().unwrap().len(), 0);
    let 扶桑改二_card_list_0 = &扶桑改二.card_list[0];
    assert_eq!(扶桑改二_card_list_0.priority, 0);
    assert_eq!(扶桑改二_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &扶桑改二_card_list_0.card_img_list,
        &vec!["s/tc_211_xkrpspyq72qz.jpg", "", "",]
    );
    assert_eq!(扶桑改二_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_rpyd1nnecq4w_n.png"]
    );
    assert_eq!(扶桑改二_card_list_0.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_0.acquire_num_in_page, 1);
    let 扶桑改二_card_list_1 = &扶桑改二.card_list[1];
    assert_eq!(扶桑改二_card_list_1.priority, 1);
    assert_eq!(扶桑改二_card_list_1.card_img_list.len(), 3);
    assert_eq!(&扶桑改二_card_list_1.card_img_list, &vec!["", "", "",]);
    assert!(扶桑改二_card_list_1.status_img.is_none());
    assert_eq!(扶桑改二_card_list_1.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_1.acquire_num_in_page, 0);
}

#[test]
fn parse_fixture_tcbook_info_20240620() {
    let tcbook = read_tclist(TCBOOK_2024_06_20.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    validate_tcbook_common(&tcbook);

    // Specific interesting ships.

    // I'm sure there's a number of reasons not to name variables like this.
    let 長門 = &tcbook[0];
    assert_eq!(長門.book_no, 1);
    assert_eq!(長門.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門.ship_class_index.unwrap(), 1);
    assert_eq!(長門.ship_type, "戦艦");
    assert_eq!(長門.ship_model_num, "");
    assert_eq!(長門.ship_name, "長門");
    assert_eq!(長門.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門.source(0), Normal);
    }
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 2);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &長門.card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &長門_card_list_0.card_img_list,
        &vec![
            "s/tc_1_d7ju63kolamj.jpg",
            "",
            "",
            "s/tc_1_2wp6daq4fn42.jpg",
            "",
            ""
        ]
    );
    assert_eq!(長門_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        長門_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png", "i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_card_list_0.variation_num_in_page, 6);
    assert_eq!(長門_card_list_0.acquire_num_in_page, 2);

    // Interesting because I hit level 99 and triggered 扶桑改's ケッコンカッコカリ
    let 扶桑 = &tcbook[25];
    assert_eq!(扶桑.book_no, 26);
    assert_eq!(扶桑.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑.ship_type, "戦艦");
    assert_eq!(扶桑.ship_model_num, "");
    assert_eq!(扶桑.ship_name, "扶桑");
    assert_eq!(扶桑.card_index_img, "s/tc_26_p9u490qtc1a4.jpg");
    assert_eq!(扶桑.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(扶桑.source(0), Normal);
        assert_eq!(扶桑.source(1), RainySeason);
    }
    assert_eq!(扶桑.variation_num, 12);
    assert_eq!(扶桑.acquire_num, 6);
    assert_eq!(扶桑.lv, 105);
    assert_eq!(
        扶桑.is_married.as_ref().unwrap(),
        &vec![true, true, true, true]
    );
    assert_eq!(扶桑.married_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑.married_img.as_ref().unwrap(),
        &vec!["s/tc_26_ktke7njfxcnx.jpg", "s/tc_26_tg21e17c6cre.jpg"]
    );
    let 扶桑_card_list_0 = &扶桑.card_list[0];
    assert_eq!(扶桑_card_list_0.priority, 0);
    assert_eq!(扶桑_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_0.card_img_list,
        &vec![
            "s/tc_26_p9u490qtc1a4.jpg",
            "",
            "",
            "s/tc_26_fskeangzj9cz.jpg",
            "s/tc_26_z2jfdzutu1j3.jpg",
            "s/tc_26_krjdrps6k23r.jpg"
        ]
    );
    assert_eq!(扶桑_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_0.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_0.acquire_num_in_page, 4);
    let 扶桑_card_list_1 = &扶桑.card_list[1];
    assert_eq!(扶桑_card_list_1.priority, 1);
    assert_eq!(扶桑_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_1.card_img_list,
        &vec![
            "",
            "s/tc_26_pkx4u0xfe2ga.jpg",
            "",
            "s/tc_26_46s6pg02mm41.jpg",
            "",
            ""
        ]
    );
    assert_eq!(扶桑_card_list_1.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_1.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_1.acquire_num_in_page, 2);

    // Interesting as it gained a new card page before its last page.
    let 早霜 = &tcbook[198];
    assert_eq!(早霜.book_no, 209);
    assert_eq!(早霜.ship_class.as_ref().unwrap(), "夕雲型");
    assert_eq!(早霜.ship_class_index.unwrap(), 17);
    assert_eq!(早霜.ship_type, "駆逐艦");
    assert_eq!(早霜.ship_model_num, "");
    assert_eq!(早霜.ship_name, "早霜");
    assert_eq!(早霜.card_index_img, "s/tc_209_6uqm0rr6azd9.jpg");
    assert_eq!(早霜.card_list.len(), 3);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
        assert_eq!(早霜.source(2), OriginalIllustration1(true));
    }
    assert_eq!(早霜.variation_num, 13);
    assert_eq!(早霜.acquire_num, 3);
    assert_eq!(早霜.lv, 1);
    assert_eq!(
        早霜.is_married.as_ref().unwrap(),
        &vec![false, false, false, false]
    );
    assert_eq!(早霜.married_img.as_ref().unwrap().len(), 0);
    let 早霜_card_list_0 = &早霜.card_list[0];
    assert_eq!(早霜_card_list_0.priority, 0);
    assert_eq!(早霜_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_0.card_img_list,
        &vec!["s/tc_209_6uqm0rr6azd9.jpg", "", "", "", "", "",]
    );
    assert_eq!(早霜_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_0.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_0.acquire_num_in_page, 1);
    let 早霜_card_list_1 = &早霜.card_list[1];
    assert_eq!(早霜_card_list_1.priority, 1);
    assert_eq!(早霜_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_1.card_img_list,
        &vec![
            "s/tc_209_7tm6p0fd3r7n.jpg",
            "",
            "",
            "s/tc_209_qt3tt1rukzxr.jpg",
            "",
            "",
        ]
    );
    assert_eq!(早霜_card_list_1.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        早霜_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png", "i/i_zp6ze49mx4qw_n.png"]
    );
    assert_eq!(早霜_card_list_1.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_1.acquire_num_in_page, 2);
    let 早霜_card_list_2 = &早霜.card_list[2];
    assert_eq!(早霜_card_list_2.priority, 2);
    assert_eq!(早霜_card_list_2.card_img_list.len(), 1);
    assert_eq!(&早霜_card_list_2.card_img_list, &vec!["",]);
    assert_eq!(早霜_card_list_2.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_2.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_2.variation_num_in_page, 1);
    assert_eq!(早霜_card_list_2.acquire_num_in_page, 0);

    let 扶桑改二 = &tcbook[200];
    assert_eq!(扶桑改二.book_no, 211);
    assert_eq!(扶桑改二.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑改二.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑改二.ship_type, "航空戦艦");
    assert_eq!(扶桑改二.ship_model_num, "");
    assert_eq!(扶桑改二.ship_name, "扶桑改二");
    assert_eq!(扶桑改二.card_index_img, "s/tc_211_xkrpspyq72qz.jpg");
    assert_eq!(扶桑改二.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
    }
    assert_eq!(扶桑改二.variation_num, 6);
    assert_eq!(扶桑改二.acquire_num, 1);
    assert_eq!(扶桑改二.lv, 105);
    assert_eq!(扶桑改二.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(扶桑改二.married_img.as_ref().unwrap().len(), 0);
    let 扶桑改二_card_list_0 = &扶桑改二.card_list[0];
    assert_eq!(扶桑改二_card_list_0.priority, 0);
    assert_eq!(扶桑改二_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &扶桑改二_card_list_0.card_img_list,
        &vec!["s/tc_211_xkrpspyq72qz.jpg", "", "",]
    );
    assert_eq!(扶桑改二_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_rpyd1nnecq4w_n.png"]
    );
    assert_eq!(扶桑改二_card_list_0.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_0.acquire_num_in_page, 1);
    let 扶桑改二_card_list_1 = &扶桑改二.card_list[1];
    assert_eq!(扶桑改二_card_list_1.priority, 1);
    assert_eq!(扶桑改二_card_list_1.card_img_list.len(), 3);
    assert_eq!(&扶桑改二_card_list_1.card_img_list, &vec!["", "", "",]);
    assert!(扶桑改二_card_list_1.status_img.is_none());
    assert_eq!(扶桑改二_card_list_1.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_1.acquire_num_in_page, 0);
}

#[test]
fn parse_fixture_tcbook_info_20240623() {
    let tcbook = read_tclist(TCBOOK_2024_06_23.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    validate_tcbook_common(&tcbook);

    // Specific interesting ships.

    // I'm sure there's a number of reasons not to name variables like this.
    let 長門 = &tcbook[0];
    assert_eq!(長門.book_no, 1);
    assert_eq!(長門.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門.ship_class_index.unwrap(), 1);
    assert_eq!(長門.ship_type, "戦艦");
    assert_eq!(長門.ship_model_num, "");
    assert_eq!(長門.ship_name, "長門");
    assert_eq!(長門.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門.source(0), Normal);
    }
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 2);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &長門.card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &長門_card_list_0.card_img_list,
        &vec![
            "s/tc_1_d7ju63kolamj.jpg",
            "",
            "",
            "s/tc_1_2wp6daq4fn42.jpg",
            "",
            ""
        ]
    );
    assert_eq!(長門_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        長門_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png", "i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_card_list_0.variation_num_in_page, 6);
    assert_eq!(長門_card_list_0.acquire_num_in_page, 2);

    // Interesting because I hit level 99 and triggered 扶桑改's ケッコンカッコカリ
    let 扶桑 = &tcbook[25];
    assert_eq!(扶桑.book_no, 26);
    assert_eq!(扶桑.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑.ship_type, "戦艦");
    assert_eq!(扶桑.ship_model_num, "");
    assert_eq!(扶桑.ship_name, "扶桑");
    assert_eq!(扶桑.card_index_img, "s/tc_26_p9u490qtc1a4.jpg");
    assert_eq!(扶桑.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(扶桑.source(0), Normal);
        assert_eq!(扶桑.source(1), RainySeason);
    }
    assert_eq!(扶桑.variation_num, 12);
    assert_eq!(扶桑.acquire_num, 6);
    assert_eq!(扶桑.lv, 105);
    assert_eq!(
        扶桑.is_married.as_ref().unwrap(),
        &vec![true, true, true, true]
    );
    assert_eq!(扶桑.married_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑.married_img.as_ref().unwrap(),
        &vec!["s/tc_26_ktke7njfxcnx.jpg", "s/tc_26_tg21e17c6cre.jpg"]
    );
    let 扶桑_card_list_0 = &扶桑.card_list[0];
    assert_eq!(扶桑_card_list_0.priority, 0);
    assert_eq!(扶桑_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_0.card_img_list,
        &vec![
            "s/tc_26_p9u490qtc1a4.jpg",
            "",
            "",
            "s/tc_26_fskeangzj9cz.jpg",
            "s/tc_26_z2jfdzutu1j3.jpg",
            "s/tc_26_krjdrps6k23r.jpg"
        ]
    );
    assert_eq!(扶桑_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_0.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_0.acquire_num_in_page, 4);
    let 扶桑_card_list_1 = &扶桑.card_list[1];
    assert_eq!(扶桑_card_list_1.priority, 1);
    assert_eq!(扶桑_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_1.card_img_list,
        &vec![
            "",
            "s/tc_26_pkx4u0xfe2ga.jpg",
            "",
            "s/tc_26_46s6pg02mm41.jpg",
            "",
            ""
        ]
    );
    assert_eq!(扶桑_card_list_1.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_1.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_1.acquire_num_in_page, 2);

    // Interesting as it gained a new card page before its last page.
    let 早霜 = &tcbook[198];
    assert_eq!(早霜.book_no, 209);
    assert_eq!(早霜.ship_class.as_ref().unwrap(), "夕雲型");
    assert_eq!(早霜.ship_class_index.unwrap(), 17);
    assert_eq!(早霜.ship_type, "駆逐艦");
    assert_eq!(早霜.ship_model_num, "");
    assert_eq!(早霜.ship_name, "早霜");
    assert_eq!(早霜.card_index_img, "s/tc_209_6uqm0rr6azd9.jpg");
    assert_eq!(早霜.card_list.len(), 3);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
        assert_eq!(早霜.source(2), OriginalIllustration1(true));
    }
    assert_eq!(早霜.variation_num, 13);
    assert_eq!(早霜.acquire_num, 3);
    assert_eq!(早霜.lv, 1);
    assert_eq!(
        早霜.is_married.as_ref().unwrap(),
        &vec![false, false, false, false]
    );
    assert_eq!(早霜.married_img.as_ref().unwrap().len(), 0);
    let 早霜_card_list_0 = &早霜.card_list[0];
    assert_eq!(早霜_card_list_0.priority, 0);
    assert_eq!(早霜_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_0.card_img_list,
        &vec!["s/tc_209_6uqm0rr6azd9.jpg", "", "", "", "", "",]
    );
    assert_eq!(早霜_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_0.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_0.acquire_num_in_page, 1);
    let 早霜_card_list_1 = &早霜.card_list[1];
    assert_eq!(早霜_card_list_1.priority, 1);
    assert_eq!(早霜_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_1.card_img_list,
        &vec![
            "s/tc_209_7tm6p0fd3r7n.jpg",
            "",
            "",
            "s/tc_209_qt3tt1rukzxr.jpg",
            "",
            "",
        ]
    );
    assert_eq!(早霜_card_list_1.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        早霜_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png", "i/i_zp6ze49mx4qw_n.png"]
    );
    assert_eq!(早霜_card_list_1.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_1.acquire_num_in_page, 2);
    let 早霜_card_list_2 = &早霜.card_list[2];
    assert_eq!(早霜_card_list_2.priority, 2);
    assert_eq!(早霜_card_list_2.card_img_list.len(), 1);
    assert_eq!(&早霜_card_list_2.card_img_list, &vec!["",]);
    assert_eq!(早霜_card_list_2.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_2.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_2.variation_num_in_page, 1);
    assert_eq!(早霜_card_list_2.acquire_num_in_page, 0);

    let 扶桑改二 = &tcbook[200];
    assert_eq!(扶桑改二.book_no, 211);
    assert_eq!(扶桑改二.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑改二.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑改二.ship_type, "航空戦艦");
    assert_eq!(扶桑改二.ship_model_num, "");
    assert_eq!(扶桑改二.ship_name, "扶桑改二");
    assert_eq!(扶桑改二.card_index_img, "s/tc_211_xkrpspyq72qz.jpg");
    assert_eq!(扶桑改二.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
    }
    assert_eq!(扶桑改二.variation_num, 6);
    assert_eq!(扶桑改二.acquire_num, 1);
    assert_eq!(扶桑改二.lv, 105);
    assert_eq!(扶桑改二.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(扶桑改二.married_img.as_ref().unwrap().len(), 0);
    let 扶桑改二_card_list_0 = &扶桑改二.card_list[0];
    assert_eq!(扶桑改二_card_list_0.priority, 0);
    assert_eq!(扶桑改二_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &扶桑改二_card_list_0.card_img_list,
        &vec!["s/tc_211_xkrpspyq72qz.jpg", "", "",]
    );
    assert_eq!(扶桑改二_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_rpyd1nnecq4w_n.png"]
    );
    assert_eq!(扶桑改二_card_list_0.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_0.acquire_num_in_page, 1);
    let 扶桑改二_card_list_1 = &扶桑改二.card_list[1];
    assert_eq!(扶桑改二_card_list_1.priority, 1);
    assert_eq!(扶桑改二_card_list_1.card_img_list.len(), 3);
    assert_eq!(&扶桑改二_card_list_1.card_img_list, &vec!["", "", "",]);
    assert!(扶桑改二_card_list_1.status_img.is_none());
    assert_eq!(扶桑改二_card_list_1.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_1.acquire_num_in_page, 0);

    // Interesting because it has a variation only in its kai form.
    let 雪風 = &tcbook[4];
    assert_eq!(雪風.book_no, 5);
    assert_eq!(雪風.ship_class.as_ref().unwrap(), "陽炎型");
    assert_eq!(雪風.ship_class_index.unwrap(), 8);
    assert_eq!(雪風.ship_type, "駆逐艦");
    assert_eq!(雪風.ship_model_num, "");
    assert_eq!(雪風.ship_name, "雪風");
    assert_eq!(雪風.card_index_img, "s/tc_5_gc3ynk3f42p4.jpg");
    assert_eq!(雪風.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(雪風.source(0), Normal);
        assert_eq!(雪風.source(1), Swimsuit);
    }
    assert_eq!(雪風.variation_num, 9);
    assert_eq!(雪風.acquire_num, 2);
    assert_eq!(雪風.lv, 1);
    assert_eq!(雪風.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(雪風.married_img.as_ref().unwrap().len(), 0);
    let 雪風_card_list_0 = &雪風.card_list[0];
    assert_eq!(雪風_card_list_0.priority, 0);
    assert_eq!(雪風_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &雪風_card_list_0.card_img_list,
        &vec![
            "s/tc_5_gc3ynk3f42p4.jpg",
            "s/tc_5_grzytq71dazm.jpg",
            "",
            "",
            "",
            "",
        ]
    );
    assert_eq!(雪風_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        雪風_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_gc3ynk3f42p4_n.png"]
    );
    assert_eq!(雪風_card_list_0.variation_num_in_page, 6);
    assert_eq!(雪風_card_list_0.acquire_num_in_page, 2);
    let 雪風_card_list_1 = &雪風.card_list[1];
    assert_eq!(雪風_card_list_1.priority, 1);
    assert_eq!(雪風_card_list_1.card_img_list.len(), 3);
    assert_eq!(&雪風_card_list_1.card_img_list, &vec!["", "", "",]);
    assert!(雪風_card_list_1.status_img.is_none());
    assert_eq!(雪風_card_list_1.variation_num_in_page, 3);
    assert_eq!(雪風_card_list_1.acquire_num_in_page, 0);
}

#[test]
fn parse_fixture_tcbook_info_20241006() {
    let tcbook = read_tclist(TCBOOK_2024_10_06.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 285);

    validate_tcbook_common(&tcbook);

    // Specific interesting ships.

    // I'm sure there's a number of reasons not to name variables like this.
    let 長門 = &tcbook[0];
    assert_eq!(長門.book_no, 1);
    assert_eq!(長門.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門.ship_class_index.unwrap(), 1);
    assert_eq!(長門.ship_type, "戦艦");
    assert_eq!(長門.ship_model_num, "");
    assert_eq!(長門.ship_name, "長門");
    assert_eq!(長門.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門.source(0), Normal);
    }
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 3);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &長門.card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &長門_card_list_0.card_img_list,
        &vec![
            "s/tc_1_d7ju63kolamj.jpg",
            "s/tc_1_kgsz396y04p3.jpg",
            "",
            "s/tc_1_2wp6daq4fn42.jpg",
            "",
            ""
        ]
    );
    assert_eq!(長門_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        長門_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png", "i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_card_list_0.variation_num_in_page, 6);
    assert_eq!(長門_card_list_0.acquire_num_in_page, 3);

    // Interesting because I hit level 99 and triggered 扶桑改's ケッコンカッコカリ
    let 扶桑 = &tcbook[25];
    assert_eq!(扶桑.book_no, 26);
    assert_eq!(扶桑.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑.ship_type, "戦艦");
    assert_eq!(扶桑.ship_model_num, "");
    assert_eq!(扶桑.ship_name, "扶桑");
    assert_eq!(扶桑.card_index_img, "s/tc_26_p9u490qtc1a4.jpg");
    assert_eq!(扶桑.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(扶桑.source(0), Normal);
        assert_eq!(扶桑.source(1), RainySeason);
    }
    assert_eq!(扶桑.variation_num, 12);
    assert_eq!(扶桑.acquire_num, 7);
    assert_eq!(扶桑.lv, 105);
    assert_eq!(
        扶桑.is_married.as_ref().unwrap(),
        &vec![true, true, true, true]
    );
    assert_eq!(扶桑.married_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑.married_img.as_ref().unwrap(),
        &vec!["s/tc_26_ktke7njfxcnx.jpg", "s/tc_26_tg21e17c6cre.jpg"]
    );
    let 扶桑_card_list_0 = &扶桑.card_list[0];
    assert_eq!(扶桑_card_list_0.priority, 0);
    assert_eq!(扶桑_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_0.card_img_list,
        &vec![
            "s/tc_26_p9u490qtc1a4.jpg",
            "",
            "",
            "s/tc_26_fskeangzj9cz.jpg",
            "s/tc_26_z2jfdzutu1j3.jpg",
            "s/tc_26_krjdrps6k23r.jpg"
        ]
    );
    assert_eq!(扶桑_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_0.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_0.acquire_num_in_page, 4);
    let 扶桑_card_list_1 = &扶桑.card_list[1];
    assert_eq!(扶桑_card_list_1.priority, 1);
    assert_eq!(扶桑_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &扶桑_card_list_1.card_img_list,
        &vec![
            "s/tc_26_eexkxxukp10d.jpg",
            "s/tc_26_pkx4u0xfe2ga.jpg",
            "",
            "s/tc_26_46s6pg02mm41.jpg",
            "",
            ""
        ]
    );
    assert_eq!(扶桑_card_list_1.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        扶桑_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_p9u490qtc1a4_n.png", "i/i_fskeangzj9cz_n.png"]
    );
    assert_eq!(扶桑_card_list_1.variation_num_in_page, 6);
    assert_eq!(扶桑_card_list_1.acquire_num_in_page, 3);

    // Interesting as it gained a new card page before its last page.
    let 早霜 = &tcbook[198];
    assert_eq!(早霜.book_no, 209);
    assert_eq!(早霜.ship_class.as_ref().unwrap(), "夕雲型");
    assert_eq!(早霜.ship_class_index.unwrap(), 17);
    assert_eq!(早霜.ship_type, "駆逐艦");
    assert_eq!(早霜.ship_model_num, "");
    assert_eq!(早霜.ship_name, "早霜");
    assert_eq!(早霜.card_index_img, "s/tc_209_6uqm0rr6azd9.jpg");
    assert_eq!(早霜.card_list.len(), 3);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
        assert_eq!(早霜.source(2), OriginalIllustration1(true));
    }
    assert_eq!(早霜.variation_num, 13);
    assert_eq!(早霜.acquire_num, 3);
    assert_eq!(早霜.lv, 1);
    assert_eq!(
        早霜.is_married.as_ref().unwrap(),
        &vec![false, false, false, false]
    );
    assert_eq!(早霜.married_img.as_ref().unwrap().len(), 0);
    let 早霜_card_list_0 = &早霜.card_list[0];
    assert_eq!(早霜_card_list_0.priority, 0);
    assert_eq!(早霜_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_0.card_img_list,
        &vec!["s/tc_209_6uqm0rr6azd9.jpg", "", "", "", "", "",]
    );
    assert_eq!(早霜_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_0.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_0.acquire_num_in_page, 1);
    let 早霜_card_list_1 = &早霜.card_list[1];
    assert_eq!(早霜_card_list_1.priority, 1);
    assert_eq!(早霜_card_list_1.card_img_list.len(), 6);
    assert_eq!(
        &早霜_card_list_1.card_img_list,
        &vec![
            "s/tc_209_7tm6p0fd3r7n.jpg",
            "",
            "",
            "s/tc_209_qt3tt1rukzxr.jpg",
            "",
            "",
        ]
    );
    assert_eq!(早霜_card_list_1.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        早霜_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png", "i/i_zp6ze49mx4qw_n.png"]
    );
    assert_eq!(早霜_card_list_1.variation_num_in_page, 6);
    assert_eq!(早霜_card_list_1.acquire_num_in_page, 2);
    let 早霜_card_list_2 = &早霜.card_list[2];
    assert_eq!(早霜_card_list_2.priority, 2);
    assert_eq!(早霜_card_list_2.card_img_list.len(), 1);
    assert_eq!(&早霜_card_list_2.card_img_list, &vec!["",]);
    assert_eq!(早霜_card_list_2.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        早霜_card_list_2.status_img.as_ref().unwrap(),
        &vec!["i/i_fm346tmmjnkp_n.png"]
    );
    assert_eq!(早霜_card_list_2.variation_num_in_page, 1);
    assert_eq!(早霜_card_list_2.acquire_num_in_page, 0);

    let 扶桑改二 = &tcbook[200];
    assert_eq!(扶桑改二.book_no, 211);
    assert_eq!(扶桑改二.ship_class.as_ref().unwrap(), "扶桑型");
    assert_eq!(扶桑改二.ship_class_index.unwrap(), 1);
    assert_eq!(扶桑改二.ship_type, "航空戦艦");
    assert_eq!(扶桑改二.ship_model_num, "");
    assert_eq!(扶桑改二.ship_name, "扶桑改二");
    assert_eq!(扶桑改二.card_index_img, "s/tc_211_xkrpspyq72qz.jpg");
    assert_eq!(扶桑改二.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(早霜.source(0), Normal);
        assert_eq!(早霜.source(1), RainySeason);
    }
    assert_eq!(扶桑改二.variation_num, 6);
    assert_eq!(扶桑改二.acquire_num, 1);
    assert_eq!(扶桑改二.lv, 105);
    assert_eq!(扶桑改二.is_married.as_ref().unwrap(), &vec![true, true]);
    assert_eq!(扶桑改二.married_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二.married_img.as_ref().unwrap(),
        &vec!["s/tc_211_61n9a2tn6jtt.jpg"]
    );
    let 扶桑改二_card_list_0 = &扶桑改二.card_list[0];
    assert_eq!(扶桑改二_card_list_0.priority, 0);
    assert_eq!(扶桑改二_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &扶桑改二_card_list_0.card_img_list,
        &vec!["s/tc_211_xkrpspyq72qz.jpg", "", "",]
    );
    assert_eq!(扶桑改二_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        扶桑改二_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_rpyd1nnecq4w_n.png"]
    );
    assert_eq!(扶桑改二_card_list_0.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_0.acquire_num_in_page, 1);
    let 扶桑改二_card_list_1 = &扶桑改二.card_list[1];
    assert_eq!(扶桑改二_card_list_1.priority, 1);
    assert_eq!(扶桑改二_card_list_1.card_img_list.len(), 3);
    assert_eq!(&扶桑改二_card_list_1.card_img_list, &vec!["", "", "",]);
    assert!(扶桑改二_card_list_1.status_img.is_none());
    assert_eq!(扶桑改二_card_list_1.variation_num_in_page, 3);
    assert_eq!(扶桑改二_card_list_1.acquire_num_in_page, 0);

    // Interesting because it has a variation only in its kai form.
    let 雪風 = &tcbook[4];
    assert_eq!(雪風.book_no, 5);
    assert_eq!(雪風.ship_class.as_ref().unwrap(), "陽炎型");
    assert_eq!(雪風.ship_class_index.unwrap(), 8);
    assert_eq!(雪風.ship_type, "駆逐艦");
    assert_eq!(雪風.ship_model_num, "");
    assert_eq!(雪風.ship_name, "雪風");
    assert_eq!(雪風.card_index_img, "s/tc_5_gc3ynk3f42p4.jpg");
    assert_eq!(雪風.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(雪風.source(0), Normal);
        assert_eq!(雪風.source(1), Swimsuit);
    }
    assert_eq!(雪風.variation_num, 9);
    assert_eq!(雪風.acquire_num, 4);
    assert_eq!(雪風.lv, 41);
    assert_eq!(
        雪風.is_married.as_ref().unwrap(),
        &vec![false, false, false]
    );
    assert_eq!(雪風.married_img.as_ref().unwrap().len(), 0);
    let 雪風_card_list_0 = &雪風.card_list[0];
    assert_eq!(雪風_card_list_0.priority, 0);
    assert_eq!(雪風_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        &雪風_card_list_0.card_img_list,
        &vec![
            "s/tc_5_gc3ynk3f42p4.jpg",
            "s/tc_5_grzytq71dazm.jpg",
            "",
            "",
            "s/tc_5_20f4czkuk3uq.jpg",
            "",
        ]
    );
    assert_eq!(雪風_card_list_0.status_img.as_ref().unwrap().len(), 2);
    assert_eq!(
        雪風_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_gc3ynk3f42p4_n.png", "i/i_7sy2x2xkfurn_n.png"]
    );
    assert_eq!(雪風_card_list_0.variation_num_in_page, 6);
    assert_eq!(雪風_card_list_0.acquire_num_in_page, 3);
    let 雪風_card_list_1 = &雪風.card_list[1];
    assert_eq!(雪風_card_list_1.priority, 1);
    assert_eq!(雪風_card_list_1.card_img_list.len(), 3);
    assert_eq!(
        &雪風_card_list_1.card_img_list,
        &vec!["s/tc_5_nfwyfjkqnmwp.jpg", "", "",]
    );
    assert_eq!(雪風_card_list_1.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        雪風_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_mqxamkacq4qm_n.png"]
    );
    assert_eq!(雪風_card_list_1.variation_num_in_page, 3);
    assert_eq!(雪風_card_list_1.acquire_num_in_page, 1);
}

#[test]
fn test_book_split_haskai_20240623() {
    let tcbook = read_tclist(TCBOOK_2024_06_23.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 284);

    validate_tcbook_split_common(&tcbook);

    let 長門 = &tcbook[0];

    let (長門_nonkai, 長門_kai) = 長門.clone().into_kai_split();
    let 長門_kai = 長門_kai.unwrap();

    assert_eq!(長門_nonkai.book_no, 1);
    assert_eq!(長門_nonkai.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門_nonkai.ship_class_index.unwrap(), 1);
    assert_eq!(長門_nonkai.ship_type, "戦艦");
    assert_eq!(長門_nonkai.ship_model_num, "");
    assert_eq!(長門_nonkai.ship_name, "長門");
    assert_eq!(長門_nonkai.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門_nonkai.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門_nonkai.source(0), Normal);
    }
    assert_eq!(長門_nonkai.variation_num, 3);
    assert_eq!(長門_nonkai.acquire_num, 1);
    assert_eq!(長門_nonkai.lv, 56);
    assert_eq!(長門_nonkai.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(長門_nonkai.married_img.as_ref().unwrap().len(), 0);
    let 長門_nonkai_card_list_0 = &長門_nonkai.card_list[0];
    assert_eq!(長門_nonkai_card_list_0.priority, 0);
    assert_eq!(長門_nonkai_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &長門_nonkai_card_list_0.card_img_list,
        &vec!["s/tc_1_d7ju63kolamj.jpg", "", "",]
    );
    assert_eq!(
        長門_nonkai_card_list_0.status_img.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        長門_nonkai_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png"]
    );
    assert_eq!(長門_nonkai_card_list_0.variation_num_in_page, 3);
    assert_eq!(長門_nonkai_card_list_0.acquire_num_in_page, 1);

    assert_eq!(長門_kai.book_no, 1);
    assert_eq!(長門_kai.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門_kai.ship_class_index.unwrap(), 1);
    assert_eq!(長門_kai.ship_type, "戦艦");
    assert_eq!(長門_kai.ship_model_num, "");
    assert_eq!(長門_kai.ship_name, "長門改");
    assert_eq!(長門_kai.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門_kai.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門_kai.source(0), Normal);
    }
    assert_eq!(長門_kai.variation_num, 3);
    assert_eq!(長門_kai.acquire_num, 1);
    assert_eq!(長門_kai.lv, 56);
    assert_eq!(長門_kai.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(長門_kai.married_img.as_ref().unwrap().len(), 0);
    let 長門_kai_card_list_0 = &長門_kai.card_list[0];
    assert_eq!(長門_kai_card_list_0.priority, 0);
    assert_eq!(長門_kai_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &長門_kai_card_list_0.card_img_list,
        &vec!["s/tc_1_2wp6daq4fn42.jpg", "", ""]
    );
    assert_eq!(長門_kai_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        長門_kai_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_kai_card_list_0.variation_num_in_page, 3);
    assert_eq!(長門_kai_card_list_0.acquire_num_in_page, 1);

    // Interesting because it has a variation only in its kai form.
    let 雪風 = &tcbook[4];

    let (雪風_nonkai, 雪風_kai) = 雪風.clone().into_kai_split();
    let 雪風_kai = 雪風_kai.unwrap();

    // TODO: acquire_num == 0 basically never appears in real data. What do we do?
    // Also, source data is iffy. See TODO elsewhere about that.
    assert_eq!(雪風_nonkai.book_no, 5);
    assert_eq!(雪風_nonkai.ship_class.as_ref().unwrap(), "陽炎型");
    assert_eq!(雪風_nonkai.ship_class_index.unwrap(), 8);
    assert_eq!(雪風_nonkai.ship_type, "駆逐艦");
    assert_eq!(雪風_nonkai.ship_model_num, "");
    assert_eq!(雪風_nonkai.ship_name, "雪風");
    assert_eq!(雪風_nonkai.card_index_img, "s/tc_5_gc3ynk3f42p4.jpg");
    assert_eq!(雪風_nonkai.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(雪風_nonkai.source(0), Normal);
        assert_eq!(雪風_nonkai.source(1), Swimsuit);
    }
    assert_eq!(雪風_nonkai.variation_num, 3);
    assert_eq!(雪風_nonkai.acquire_num, 2);
    assert_eq!(雪風_nonkai.lv, 1);
    assert_eq!(雪風_nonkai.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(雪風_nonkai.married_img.as_ref().unwrap().len(), 0);
    let 雪風_nonkai_card_list_0 = &雪風_nonkai.card_list[0];
    assert_eq!(雪風_nonkai_card_list_0.priority, 0);
    assert_eq!(雪風_nonkai_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &雪風_nonkai_card_list_0.card_img_list,
        &vec!["s/tc_5_gc3ynk3f42p4.jpg", "s/tc_5_grzytq71dazm.jpg", "",]
    );
    assert_eq!(
        雪風_nonkai_card_list_0.status_img.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        雪風_nonkai_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_gc3ynk3f42p4_n.png"]
    );
    assert_eq!(雪風_nonkai_card_list_0.variation_num_in_page, 3);
    assert_eq!(雪風_nonkai_card_list_0.acquire_num_in_page, 2);
    let 雪風_nonkai_card_list_1 = &雪風_nonkai.card_list[1];
    assert_eq!(雪風_nonkai_card_list_1.priority, 1);
    // NOTE: This breaks the assumption that all pages are 3 or 6, compared to upstream data.
    assert!(雪風_nonkai_card_list_1.card_img_list.is_empty());
    assert!(雪風_nonkai_card_list_1.status_img.is_none());
    assert_eq!(雪風_nonkai_card_list_1.variation_num_in_page, 0);
    assert_eq!(雪風_nonkai_card_list_1.acquire_num_in_page, 0);

    assert_eq!(雪風_kai.book_no, 5);
    assert_eq!(雪風_kai.ship_class.as_ref().unwrap(), "陽炎型");
    assert_eq!(雪風_kai.ship_class_index.unwrap(), 8);
    assert_eq!(雪風_kai.ship_type, "駆逐艦");
    assert_eq!(雪風_kai.ship_model_num, "");
    assert_eq!(雪風_kai.ship_name, "雪風改");
    assert_eq!(雪風_kai.card_index_img, "s/tc_5_gc3ynk3f42p4.jpg");
    assert_eq!(雪風_kai.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(雪風_kai.source(0), Normal);
        assert_eq!(雪風_kai.source(1), Swimsuit);
    }
    assert_eq!(雪風_kai.variation_num, 6);
    assert_eq!(雪風_kai.acquire_num, 0);
    assert_eq!(雪風_kai.lv, 1);
    assert_eq!(雪風_kai.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(雪風_kai.married_img.as_ref().unwrap().len(), 0);
    let 雪風_kai_card_list_0 = &雪風_kai.card_list[0];
    assert_eq!(雪風_kai_card_list_0.priority, 0);
    assert_eq!(雪風_kai_card_list_0.card_img_list.len(), 3);
    assert_eq!(&雪風_kai_card_list_0.card_img_list, &vec!["", "", "",]);
    assert!(雪風_kai_card_list_0.status_img.as_ref().unwrap().is_empty());
    assert_eq!(雪風_kai_card_list_0.variation_num_in_page, 3);
    assert_eq!(雪風_kai_card_list_0.acquire_num_in_page, 0);
    let 雪風_kai_card_list_1 = &雪風_kai.card_list[1];
    assert_eq!(雪風_kai_card_list_1.priority, 1);
    assert_eq!(雪風_kai_card_list_1.card_img_list.len(), 3);
    assert_eq!(&雪風_kai_card_list_1.card_img_list, &vec!["", "", "",]);
    assert!(雪風_kai_card_list_1.status_img.is_none());
    assert_eq!(雪風_kai_card_list_1.variation_num_in_page, 3);
    assert_eq!(雪風_kai_card_list_1.acquire_num_in_page, 0);
}

#[test]
fn test_book_split_haskai_20241006() {
    let tcbook = read_tclist(TCBOOK_2024_10_06.as_ref()).unwrap();
    assert_eq!(tcbook.len(), 285);

    validate_tcbook_split_common(&tcbook);

    let 長門 = &tcbook[0];

    let (長門_nonkai, 長門_kai) = 長門.clone().into_kai_split();
    let 長門_kai = 長門_kai.unwrap();

    assert_eq!(長門_nonkai.book_no, 1);
    assert_eq!(長門_nonkai.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門_nonkai.ship_class_index.unwrap(), 1);
    assert_eq!(長門_nonkai.ship_type, "戦艦");
    assert_eq!(長門_nonkai.ship_model_num, "");
    assert_eq!(長門_nonkai.ship_name, "長門");
    assert_eq!(長門_nonkai.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門_nonkai.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門_nonkai.source(0), Normal);
    }
    assert_eq!(長門_nonkai.variation_num, 3);
    assert_eq!(長門_nonkai.acquire_num, 2);
    assert_eq!(長門_nonkai.lv, 56);
    assert_eq!(長門_nonkai.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(長門_nonkai.married_img.as_ref().unwrap().len(), 0);
    let 長門_nonkai_card_list_0 = &長門_nonkai.card_list[0];
    assert_eq!(長門_nonkai_card_list_0.priority, 0);
    assert_eq!(長門_nonkai_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &長門_nonkai_card_list_0.card_img_list,
        &vec!["s/tc_1_d7ju63kolamj.jpg", "s/tc_1_kgsz396y04p3.jpg", "",]
    );
    assert_eq!(
        長門_nonkai_card_list_0.status_img.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        長門_nonkai_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_d7ju63kolamj_n.png"]
    );
    assert_eq!(長門_nonkai_card_list_0.variation_num_in_page, 3);
    assert_eq!(長門_nonkai_card_list_0.acquire_num_in_page, 2);

    assert_eq!(長門_kai.book_no, 1);
    assert_eq!(長門_kai.ship_class.as_ref().unwrap(), "長門型");
    assert_eq!(長門_kai.ship_class_index.unwrap(), 1);
    assert_eq!(長門_kai.ship_type, "戦艦");
    assert_eq!(長門_kai.ship_model_num, "");
    assert_eq!(長門_kai.ship_name, "長門改");
    assert_eq!(長門_kai.card_index_img, "s/tc_1_d7ju63kolamj.jpg");
    assert_eq!(長門_kai.card_list.len(), 1);
    {
        use BookShipCardPageSource::*;
        assert_eq!(長門_kai.source(0), Normal);
    }
    assert_eq!(長門_kai.variation_num, 3);
    assert_eq!(長門_kai.acquire_num, 1);
    assert_eq!(長門_kai.lv, 56);
    assert_eq!(長門_kai.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(長門_kai.married_img.as_ref().unwrap().len(), 0);
    let 長門_kai_card_list_0 = &長門_kai.card_list[0];
    assert_eq!(長門_kai_card_list_0.priority, 0);
    assert_eq!(長門_kai_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &長門_kai_card_list_0.card_img_list,
        &vec!["s/tc_1_2wp6daq4fn42.jpg", "", ""]
    );
    assert_eq!(長門_kai_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        長門_kai_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_2wp6daq4fn42_n.png"]
    );
    assert_eq!(長門_kai_card_list_0.variation_num_in_page, 3);
    assert_eq!(長門_kai_card_list_0.acquire_num_in_page, 1);

    // Interesting because it has a variation only in its kai form.
    let 雪風 = &tcbook[4];

    let (雪風_nonkai, 雪風_kai) = 雪風.clone().into_kai_split();
    let 雪風_kai = 雪風_kai.unwrap();

    // TODO: acquire_num == 0 basically never appears in real data. What do we do?
    // Also, source data is iffy. See TODO elsewhere about that.
    assert_eq!(雪風_nonkai.book_no, 5);
    assert_eq!(雪風_nonkai.ship_class.as_ref().unwrap(), "陽炎型");
    assert_eq!(雪風_nonkai.ship_class_index.unwrap(), 8);
    assert_eq!(雪風_nonkai.ship_type, "駆逐艦");
    assert_eq!(雪風_nonkai.ship_model_num, "");
    assert_eq!(雪風_nonkai.ship_name, "雪風");
    assert_eq!(雪風_nonkai.card_index_img, "s/tc_5_gc3ynk3f42p4.jpg");
    assert_eq!(雪風_nonkai.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(雪風_nonkai.source(0), Normal);
        assert_eq!(雪風_nonkai.source(1), Swimsuit);
    }
    assert_eq!(雪風_nonkai.variation_num, 3);
    assert_eq!(雪風_nonkai.acquire_num, 2);
    assert_eq!(雪風_nonkai.lv, 41);
    assert_eq!(雪風_nonkai.is_married.as_ref().unwrap(), &vec![false]);
    assert_eq!(雪風_nonkai.married_img.as_ref().unwrap().len(), 0);
    let 雪風_nonkai_card_list_0 = &雪風_nonkai.card_list[0];
    assert_eq!(雪風_nonkai_card_list_0.priority, 0);
    assert_eq!(雪風_nonkai_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &雪風_nonkai_card_list_0.card_img_list,
        &vec!["s/tc_5_gc3ynk3f42p4.jpg", "s/tc_5_grzytq71dazm.jpg", "",]
    );
    assert_eq!(
        雪風_nonkai_card_list_0.status_img.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        雪風_nonkai_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_gc3ynk3f42p4_n.png"]
    );
    assert_eq!(雪風_nonkai_card_list_0.variation_num_in_page, 3);
    assert_eq!(雪風_nonkai_card_list_0.acquire_num_in_page, 2);
    let 雪風_nonkai_card_list_1 = &雪風_nonkai.card_list[1];
    assert_eq!(雪風_nonkai_card_list_1.priority, 1);
    // NOTE: This breaks the assumption that all pages are 3 or 6, compared to upstream data.
    assert!(雪風_nonkai_card_list_1.card_img_list.is_empty());
    assert!(雪風_nonkai_card_list_1.status_img.is_none());
    assert_eq!(雪風_nonkai_card_list_1.variation_num_in_page, 0);
    assert_eq!(雪風_nonkai_card_list_1.acquire_num_in_page, 0);

    assert_eq!(雪風_kai.book_no, 5);
    assert_eq!(雪風_kai.ship_class.as_ref().unwrap(), "陽炎型");
    assert_eq!(雪風_kai.ship_class_index.unwrap(), 8);
    assert_eq!(雪風_kai.ship_type, "駆逐艦");
    assert_eq!(雪風_kai.ship_model_num, "");
    assert_eq!(雪風_kai.ship_name, "雪風改");
    assert_eq!(雪風_kai.card_index_img, "s/tc_5_gc3ynk3f42p4.jpg");
    assert_eq!(雪風_kai.card_list.len(), 2);
    {
        use BookShipCardPageSource::*;
        assert_eq!(雪風_kai.source(0), Normal);
        assert_eq!(雪風_kai.source(1), Swimsuit);
    }
    assert_eq!(雪風_kai.variation_num, 6);
    assert_eq!(雪風_kai.acquire_num, 2);
    assert_eq!(雪風_kai.lv, 41);
    assert_eq!(雪風_kai.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(雪風_kai.married_img.as_ref().unwrap().len(), 0);
    let 雪風_kai_card_list_0 = &雪風_kai.card_list[0];
    assert_eq!(雪風_kai_card_list_0.priority, 0);
    assert_eq!(雪風_kai_card_list_0.card_img_list.len(), 3);
    assert_eq!(
        &雪風_kai_card_list_0.card_img_list,
        &vec!["", "s/tc_5_20f4czkuk3uq.jpg", "",]
    );
    assert_eq!(雪風_kai_card_list_0.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        雪風_kai_card_list_0.status_img.as_ref().unwrap(),
        &vec!["i/i_7sy2x2xkfurn_n.png"]
    );
    assert_eq!(雪風_kai_card_list_0.variation_num_in_page, 3);
    assert_eq!(雪風_kai_card_list_0.acquire_num_in_page, 1);
    let 雪風_kai_card_list_1 = &雪風_kai.card_list[1];
    assert_eq!(雪風_kai_card_list_1.priority, 1);
    assert_eq!(雪風_kai_card_list_1.card_img_list.len(), 3);
    assert_eq!(
        &雪風_kai_card_list_1.card_img_list,
        &vec!["s/tc_5_nfwyfjkqnmwp.jpg", "", "",]
    );
    assert_eq!(雪風_kai_card_list_1.status_img.as_ref().unwrap().len(), 1);
    assert_eq!(
        雪風_kai_card_list_1.status_img.as_ref().unwrap(),
        &vec!["i/i_mqxamkacq4qm_n.png"]
    );
    assert_eq!(雪風_kai_card_list_1.variation_num_in_page, 3);
    assert_eq!(雪風_kai_card_list_1.acquire_num_in_page, 1);
}

#[test]
fn test_sources_against_latest() {
    let mut tcbook = read_tclist(TCBOOK_LATEST.as_ref()).unwrap();
    tcbook.retain(|ship| ship.acquire_num > 0);
    init_book_ship_sources();

    for book_ship in tcbook.iter() {
        let book_no = book_ship.book_no;
        let source = BOOK_SHIP_SOURCES.get().unwrap().get(&book_no);
        let expected_len = source.map_or_else(|| 0, |s| s.len()) + 1;
        assert_eq!(book_ship.card_list.len(), expected_len, "{book_no}");
    }
}
