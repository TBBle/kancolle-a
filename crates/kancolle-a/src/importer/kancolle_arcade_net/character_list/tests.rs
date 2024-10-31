use super::*;
use itertools::izip;

use lazy_static_include::*;

// https://kancolle-arcade.net/ac/api/CharacterList/info
lazy_static_include_bytes! {
    CHARLIST_2024_10_31 => "tests/fixtures/2024-10-31/CharacterList_info.json",
}

#[test]
fn parse_empty_character_list_reader() {
    read_characterlist(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_character_list_vector() {
    let character_list = read_characterlist("[]".as_bytes()).unwrap();
    assert_eq!(character_list.len(), 0);
}

fn validate_character_list_common(character_list: &CharacterList) {
    const CARD_IMAGE_SUFFIX: &str = ".jpg";
    const STATUS_IMAGE_PREFIX: &str = "i/i_";
    const STATUS_IMAGE_SUFFIXES: [&str; 4] = ["_n.png", "_bs.png", "_bm.png", "_bl.png"];
    const EQUIP_IMAGE_PREFIX: &str = "equip_icon_";

    // Validating some dependent values to ensure we are making good assumptions

    for ship in character_list.iter() {
        let _ship_name = &ship.ship_name;
        eprintln!("Ship {_ship_name}");
        // Minimum of four, but some ships have more
        assert_eq!(ship.slot_equip_name.len(), ship.slot_amount.len());
        assert_eq!(ship.slot_equip_name.len(), ship.slot_disp.len());
        assert_eq!(ship.slot_equip_name.len(), ship.slot_img.len());
        assert_eq!(ship.slot_equip_name.len(), ship.slot_extension.len());

        for (equip, amount, display, image, extension) in izip!(
            ship.slot_equip_name[(ship.slot_num as usize)..].iter(),
            ship.slot_amount[(ship.slot_num as usize)..].iter(),
            ship.slot_disp[(ship.slot_num as usize)..].iter(),
            ship.slot_img[(ship.slot_num as usize)..].iter(),
            ship.slot_extension[(ship.slot_num as usize)..].iter()
        ) {
            assert_eq!(equip, "");
            assert_eq!(*amount, 0);
            assert_eq!(display, "NONE");
            assert_eq!(image, "");
            assert!(!extension);
        }
        for (equip, amount, display, image, extension) in izip!(
            ship.slot_equip_name[..(ship.slot_num as usize)].iter(),
            ship.slot_amount[..(ship.slot_num as usize)].iter(),
            ship.slot_disp[..(ship.slot_num as usize)].iter(),
            ship.slot_img[..(ship.slot_num as usize)].iter(),
            ship.slot_extension[..(ship.slot_num as usize)].iter()
        ) {
            if equip.is_empty() {
                assert_eq!(image, "");
                assert!(!extension);
                if *amount == 0 {
                    // TODO: Why can this be NOT_EQUIPPED_AIRCRAFT? See 鳳翔改.
                    // Wiki shows it should be 14-16-12, so data issue? Invisible damage?
                    // Might be a weirdness due to initial loadout zeroing out the slots?
                    assert!(display == "NONE" || display == "NOT_EQUIPPED_AIRCRAFT");
                } else {
                    assert_eq!(display, "NOT_EQUIPPED_AIRCRAFT");
                }
            } else {
                assert_ne!(image, "");
                // TODO: What is this for? New in VERSION E REVISION 2
                assert!(!extension);
                if *amount == 0 {
                    // TODO: Why can this be NOT_EQUIPPED_AIRCRAFT? See 熊野改.
                    // Wiki shows it should be 5-6-5-6, so data issue? In this case she is damaged, but
                    // the other slots are unaffected.
                    // Might be a weirdness due to initial loadout zeroing out the slots?
                    // TODO: 明石 has 0-count aircraft-capable mounts. And I happen to have used them in my data,
                    // so we can have EQUIPPED_AIRCRAFT here too.
                    assert!(
                        display == "NONE"
                            || display == "NOT_EQUIPPED_AIRCRAFT"
                            || display == "EQUIPPED_AIRCRAFT"
                    );
                } else {
                    // Depends on whether the equipped item is an aircraft
                    assert!(display == "EQUIPPED_AIRCRAFT" || display == "NOT_EQUIPPED_AIRCRAFT");
                }
            }
        }
    }

    // Universally-true facts about all ships we have characters for

    for ship in character_list.iter() {
        let _ship_name = &ship.ship_name;
        eprintln!("Ship {_ship_name}");

        assert_ne!(ship.ship_type, "");
        assert_ne!(ship.ship_name, "");

        assert!(ship.status_img.starts_with(STATUS_IMAGE_PREFIX));
        assert!(
            ship.status_img.ends_with(STATUS_IMAGE_SUFFIXES[0])
                || ship.status_img.ends_with(STATUS_IMAGE_SUFFIXES[1])
                || ship.status_img.ends_with(STATUS_IMAGE_SUFFIXES[2])
                || ship.status_img.ends_with(STATUS_IMAGE_SUFFIXES[3])
        );

        let card_image_prefix = format!("s/tc_{0}_", ship.book_no);
        assert!(ship.tc_img.starts_with(&card_image_prefix));
        assert!(ship.tc_img.ends_with(CARD_IMAGE_SUFFIX));

        for image in ship.slot_img.iter().filter(|img| !img.is_empty()) {
            assert!(image.starts_with(EQUIP_IMAGE_PREFIX));
        }

        for extension in ship.slot_extension.iter() {
            assert!(!extension);
        }

        // Breakdown of dispSortNo
        let disp_sort_no = ship.disp_sort_no;
        assert_eq!(ship.remodel_lv, (disp_sort_no % 100) as u16);
        let disp_sort_no = disp_sort_no / 100;
        // dispSortNo is actually more reliable here, as various ships lack this index in their data.
        if let Some(ship_class_index) = ship.ship_class_index {
            assert_eq!(ship_class_index, (disp_sort_no % 1000) as u16);
        } else {
            assert_ne!((disp_sort_no % 1000), 0);
        };
        let disp_sort_no = disp_sort_no / 1000;
        // Not sure what's next. Probably the same as Blueprint shipClassId.
        // TODO: Validate this in Ship tests.
        let disp_sort_no = disp_sort_no / 1000;
        // Useful fact but annoying to test: This is the base ship's book number.
        if ship.remodel_lv == 0 {
            assert_eq!(ship.book_no, disp_sort_no as u16);
        }
    }
}

#[test]
fn parse_fixture_character_list_info_20241031() {
    let character_list = read_characterlist(CHARLIST_2024_10_31.as_ref()).unwrap();

    assert_eq!(character_list.len(), 393);
    validate_character_list_common(&character_list);
}
