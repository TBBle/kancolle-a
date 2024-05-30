use std::env;
use std::fs;
use std::path::Path;

use path_macro::path;
// TODO: Wrap a nice API around this.
use kancolle_a::importer::kancolle_arcade_net as kca_net;

#[test]
fn parse_empty_string() {
    kca_net::BlueprintList::new("").unwrap_err();
}

#[test]
fn parse_empty_vector() {
    let blueprint_list = kca_net::BlueprintList::new("[]").unwrap();
    assert_eq!(blueprint_list.len(), 0);
}

#[test]
fn parse_fixture_tcbook_info_20240528() {
    let manfest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // https://kancolle-arcade.net/ac/api/BlueprintList/info
    let fixture = path!(
        Path::new(&manfest_dir) / "tests" / "fixtures" / "2024-05-30" / "BlueprintList_info.json"
    );

    let data = fs::read_to_string(fixture).unwrap();
    let blueprint_list = kca_net::BlueprintList::new(&data).unwrap();

    assert_eq!(blueprint_list.len(), 133);

    const STATUS_IMAGE_PREFIX: &str = "i/i_";
    const STATUS_IMAGE_SUFFIXES: [&'static str; 4] = ["_n.png", "_bs.png", "_bm.png", "_bl.png"];

    // Validating some dependent values to ensure we are making good assumptions

    for ship in blueprint_list.iter() {
        let mut blueprint_total_num = 0;
        let mut expected_warning: bool = false;
        for card_page in ship.expiration_date_list().iter() {
            // TODO: Validate expiration date for each data against "this month"
            expected_warning |= card_page.expire_this_month();
            blueprint_total_num += card_page.blueprint_num();
        }
        assert_eq!(*ship.blueprint_total_num(), blueprint_total_num);
        assert_eq!(*ship.exists_warning_for_expiration(), expected_warning);
    }

    // Universally-true facts about all ships we have blueprints for

    for ship in blueprint_list.iter() {
        let _ship_name = ship.ship_name();
        eprintln!("Ship {_ship_name}");

        //assert_ne!(*ship.ship_class_id(), 0); // Can be zero, is this an index into something?
        assert_ne!(*ship.ship_class_index(), 0);
        assert_ne!(ship.ship_type(), "");
        assert_ne!(ship.ship_name(), "");

        assert!(ship.status_img().starts_with(STATUS_IMAGE_PREFIX));
        assert!(
            ship.status_img().ends_with(STATUS_IMAGE_SUFFIXES[0])
                || ship.status_img().ends_with(STATUS_IMAGE_SUFFIXES[1])
                || ship.status_img().ends_with(STATUS_IMAGE_SUFFIXES[2])
                || ship.status_img().ends_with(STATUS_IMAGE_SUFFIXES[3])
        );

        assert_ne!(*ship.blueprint_total_num(), 0);
        assert!(!ship.expiration_date_list().is_empty());

        for expiration_date in ship.expiration_date_list().iter() {
            assert_ne!(*expiration_date.expiration_date(), 0);
            assert_ne!(*expiration_date.blueprint_num(), 0);
        }
    }
}
