use chrono;
use chrono::Datelike;
use chrono::TimeZone;
use chrono_tz::Asia::Tokyo;

use super::*;

use lazy_static_include::*;

// https://kancolle-arcade.net/ac/api/BlueprintList/info
lazy_static_include_bytes! {
    BPLIST_2024_05_30 => "tests/fixtures/2024-05-30/BlueprintList_info.json",
    BPLIST_2024_06_09 => "tests/fixtures/2024-06-09/BlueprintList_info.json",
    BPLIST_2024_06_10 => "tests/fixtures/2024-06-10/BlueprintList_info.json",
    BPLIST_2024_06_20 => "tests/fixtures/2024-06-20/BlueprintList_info.json",
    BPLIST_2024_06_23 => "tests/fixtures/2024-06-23/BlueprintList_info.json",
}

#[test]
fn parse_empty_blueprint_list_reader() {
    BlueprintList::new(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_blueprint_list_vector() {
    let blueprint_list = BlueprintList::new("[]".as_bytes()).unwrap();
    assert_eq!(blueprint_list.len(), 0);
}

fn validate_blueprint_list_common(blueprint_list: &BlueprintList) {
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
            // They all have the same day. No idea why.
            assert_eq!(expiration_date.expiration_date().day(), 11);
            assert_ne!(*expiration_date.blueprint_num(), 0);
        }
    }
}

#[test]
fn parse_fixture_blueprint_list_info_20240528() {
    let blueprint_list = BlueprintList::new(BPLIST_2024_05_30.as_ref()).unwrap();

    assert_eq!(blueprint_list.len(), 133);
    validate_blueprint_list_common(&blueprint_list);

    // Specific interesting ships.

    let 弥生 = &blueprint_list[0];
    assert_eq!(*弥生.ship_class_id(), 14);
    assert_eq!(*弥生.ship_class_index(), 3);
    assert_eq!(*弥生.ship_sort_no(), 1800);
    assert_eq!(弥生.ship_type(), "駆逐艦");
    assert_eq!(弥生.ship_name(), "弥生");
    assert_eq!(弥生.status_img(), "i/i_4ma06a97am0r_n.png");
    assert_eq!(*弥生.blueprint_total_num(), 2);
    assert_eq!(*弥生.exists_warning_for_expiration(), false);
    assert_eq!(弥生.expiration_date_list().len(), 2);
    let 弥生_0 = &弥生.expiration_date_list()[0];
    // No idea why the expiration date is actually the 11th...
    assert_eq!(
        *弥生_0.expiration_date(),
        Tokyo
            .with_ymd_and_hms(2024, 8, 11, 23, 59, 59)
            .unwrap()
            .to_utc()
    );
    assert_eq!(*弥生_0.blueprint_num(), 1);
    assert_eq!(*弥生_0.expire_this_month(), false);
    let 弥生_1 = &弥生.expiration_date_list()[1];
    assert_eq!(
        *弥生_1.expiration_date(),
        Tokyo
            .with_ymd_and_hms(2024, 10, 11, 23, 59, 59)
            .unwrap()
            .to_utc()
    );
    assert_eq!(*弥生_1.blueprint_num(), 1);
    assert_eq!(*弥生_1.expire_this_month(), false);

    // Expiring blueprint.
    let 卯月 = &blueprint_list[12];
    assert_eq!(*卯月.ship_class_id(), 14);
    assert_eq!(*卯月.ship_class_index(), 4);
    assert_eq!(*卯月.ship_sort_no(), 1800);
    assert_eq!(卯月.ship_type(), "駆逐艦");
    assert_eq!(卯月.ship_name(), "卯月");
    assert_eq!(卯月.status_img(), "i/i_mj1x41twqqw6_n.png");
    assert_eq!(*卯月.blueprint_total_num(), 2);
    assert_eq!(*卯月.exists_warning_for_expiration(), true);
    assert_eq!(卯月.expiration_date_list().len(), 2);
    let 卯月_0 = &卯月.expiration_date_list()[0];
    assert_eq!(
        *卯月_0.expiration_date(),
        Tokyo
            .with_ymd_and_hms(2024, 5, 11, 23, 59, 59)
            .unwrap()
            .to_utc()
    );
    assert_eq!(*卯月_0.blueprint_num(), 1);
    assert_eq!(*卯月_0.expire_this_month(), true);
    let 卯月_1 = &卯月.expiration_date_list()[1];
    assert_eq!(
        *卯月_1.expiration_date(),
        Tokyo
            .with_ymd_and_hms(2024, 9, 11, 23, 59, 59)
            .unwrap()
            .to_utc()
    );
    assert_eq!(*卯月_1.blueprint_num(), 1);
    assert_eq!(*卯月_1.expire_this_month(), false);
}

#[test]
fn parse_fixture_blueprint_list_info_20240609() {
    let blueprint_list = BlueprintList::new(BPLIST_2024_06_09.as_ref()).unwrap();

    assert_eq!(blueprint_list.len(), 136);
    validate_blueprint_list_common(&blueprint_list);
}

#[test]
fn parse_fixture_blueprint_list_info_20240610() {
    let blueprint_list = BlueprintList::new(BPLIST_2024_06_10.as_ref()).unwrap();

    assert_eq!(blueprint_list.len(), 135);
    validate_blueprint_list_common(&blueprint_list);
}

#[test]
fn parse_fixture_blueprint_list_info_20240620() {
    let blueprint_list = BlueprintList::new(BPLIST_2024_06_20.as_ref()).unwrap();

    assert_eq!(blueprint_list.len(), 135);
    validate_blueprint_list_common(&blueprint_list);
}

#[test]
fn parse_fixture_blueprint_list_info_20240623() {
    let blueprint_list = BlueprintList::new(BPLIST_2024_06_23.as_ref()).unwrap();

    assert_eq!(blueprint_list.len(), 133);
    validate_blueprint_list_common(&blueprint_list);
}
