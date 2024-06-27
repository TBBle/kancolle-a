use super::districts::*;
use super::places::*;

use lazy_static_include::*;

// https://kancolle-arcade.net/ac/api/Place/districts
// https://kancolle-arcade.net/ac/api/Place/places
lazy_static_include_bytes! {
    DISTRICTS_2024_06_23 => "tests/fixtures/2024-06-23/Place_districts.json",
    PLACES_2024_06_23 => "tests/fixtures/2024-06-23/Place_places.json",
}

#[test]
fn parse_empty_place_districts_reader() {
    PlaceDistricts::new(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_place_districts_vector() {
    let place_districts = PlaceDistricts::new("[]".as_bytes()).unwrap();
    assert_eq!(place_districts.len(), 0);
}

fn validate_place_districts_common(place_districts: &PlaceDistricts) {
    // This data is pretty-much fixed, so barring any major geopolitical changes in
    // Japan or expansion into other countries, we can just validate it hard.
    // TODO: We could burn this into the compile if we wanted... Save a fetch?
    assert_eq!(place_districts.len(), 6);
    let 北海道_東北 = &place_districts[0];
    assert_eq!(北海道_東北.top_region_enum(), "HOKKAIDO_TOHOKU");
    assert_eq!(北海道_東北.name(), "北海道・東北");
    assert_eq!(北海道_東北.prefecture_beans().len(), 7);
    let 北海道 = &北海道_東北.prefecture_beans()[0];
    assert_eq!(北海道.region_enum(), "HOKKAIDO");
    assert_eq!(北海道.name(), "北海道");
    assert_eq!(*北海道.jis_code(), 1);
}

#[test]
fn parse_fixture_place_districts_info_20240623() {
    let place_districts = PlaceDistricts::new(DISTRICTS_2024_06_23.as_ref()).unwrap();

    validate_place_districts_common(&place_districts);
}

#[test]
fn parse_empty_place_paces_reader() {
    PlacePlaces::new(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_place_places_vector() {
    let place_places = PlacePlaces::new("[]".as_bytes()).unwrap();
    assert_eq!(place_places.len(), 0);
}

fn validate_place_places_common(_place_places: &PlacePlaces) {
    // TODO: Does this data have any internal consistency to maintain?
}

#[test]
fn parse_fixture_place_places_info_20240623() {
    let place_places = PlacePlaces::new(PLACES_2024_06_23.as_ref()).unwrap();

    assert_eq!(place_places.len(), 710);
    validate_place_places_common(&place_places);
}
