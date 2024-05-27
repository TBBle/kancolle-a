use std::env;
use std::fs;
use std::path::Path;

use path_macro::path;
// TODO: Wrap a nice API around this.
use kancolle_a::importer::kancolle_arcade_net as kca_net;

#[test]
fn parse_empty_string() {
    kca_net::TcBook::new("").unwrap_err();
}

#[test]
fn parse_empty_vector() {
    let tcbook = kca_net::TcBook::new("[]").unwrap();
    assert_eq!(tcbook.len(), 0);
}

#[test]
fn parse_fixture_tcbook_info() {
    let manfest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // https://kancolle-arcade.net/ac/api/TcBook/info
    let fixture = path!(Path::new(&manfest_dir) / "tests" / "fixtures" / "TcBook_info.json");

    let data = fs::read_to_string(fixture).unwrap();
    let tcbook = kca_net::TcBook::new(&data).unwrap();
    assert_eq!(tcbook.len(), 284);

    const CARD_IMAGE_SUFFIX: &str = ".jpg";
    const STATUS_IMAGE_PREFIX: &str = "i/i_";
    const STATUS_IMAGE_SUFFIXES: [&'static str; 4] = ["_n.png", "_bs.png", "_bm.png", "_bl.png"];

    // Validating some dependent values to ensure we are making good assumptions

    for ship in tcbook.iter() {
        let mut acquire_num = 0;
        let mut variation_num = 0;
        for (priority, card_page) in ship.card_list.iter().enumerate() {
            assert_eq!(card_page.priority as usize, priority);
            // First page is the 3 generic images, and possibly a second row if modified version has the same book number.
            // The other pages are for event variations, 3-image sets and then individual original illustrations.
            // No real way to tell them apart at this time though, e.g. 雪風 has no swimsuit set, but 雪風改 (same number) does.
            // So the best we can do is verify that the first page is either 3 or 6 cards.
            if priority == 0 {
                assert!(
                    card_page.variation_num_in_page == 3 || card_page.variation_num_in_page == 6
                );
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
    let ship_types_without_class = vec!["工作艦"]; // Unique repair ship

    // Ships with a model num also don't have an index, e.g., DD-445. Just for US ships?
    let ship_classes_without_index =
        vec!["特種船丙型", "三式潜航輸送艇", "UボートIXC型", "改伊勢型"];

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
            if ship.ship_model_num != ""
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
        assert_ne!(ship.variation_num, 0);
        assert_ne!(ship.acquire_num, 0);
        assert_ne!(ship.lv, 0);
        assert!(ship.is_married.is_some());
        assert!(ship.married_img.is_some());

        for card_list_page in ship.card_list.iter() {
            for card_image in card_list_page
                .card_img_list
                .iter()
                .filter(|s| !s.is_empty())
            {
                assert!(card_image.starts_with(&card_image_prefix));
                assert!(card_image.ends_with(CARD_IMAGE_SUFFIX));
            }
            if card_list_page.status_img.is_none() {
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

    // Validate that I have no data, for later updates when I have data.
    let has_married_img_ships = tcbook.iter().filter(|ship| ship.married_img.is_some());
    for ship in has_married_img_ships {
        // No currently-married examples in my data
        assert_eq!(ship.married_img.as_ref().unwrap().len(), 0);
    }

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
    assert_eq!(長門.variation_num, 6);
    assert_eq!(長門.acquire_num, 2);
    assert_eq!(長門.lv, 56);
    assert_eq!(長門.is_married.as_ref().unwrap(), &vec![false, false]);
    assert_eq!(長門.married_img.as_ref().unwrap().len(), 0);
    let 長門_card_list_0 = &tcbook[0].card_list[0];
    assert_eq!(長門_card_list_0.priority, 0);
    // This is the number of rows on this page. TODO: Rename the field?
    assert_eq!(長門_card_list_0.card_img_list.len(), 6);
    assert_eq!(
        長門_card_list_0.card_img_list,
        vec![
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
}
