use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use path_macro::path;
// TODO: Wrap a nice API around this.
use kancolle_a::importer::kancolle_arcade_net as kca_net;

#[test]
fn parse_empty_place_districts_reader() {
    kca_net::PlaceDistricts::new(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_place_districts_vector() {
    let place_districts = kca_net::PlaceDistricts::new("[]".as_bytes()).unwrap();
    assert_eq!(place_districts.len(), 0);
}

fn validate_place_districts_common(place_districts: &kca_net::PlaceDistricts) {
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
    let manfest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // https://kancolle-arcade.net/ac/api/BlueprintList/info
    let fixture = path!(
        Path::new(&manfest_dir) / "tests" / "fixtures" / "2024-06-23" / "Place_districts.json"
    );

    let data = BufReader::new(File::open(fixture).unwrap());
    let place_districts = kca_net::PlaceDistricts::new(data).unwrap();

    validate_place_districts_common(&place_districts);
}

#[test]
fn parse_empty_place_paces_reader() {
    kca_net::PlacePlaces::new(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_place_places_vector() {
    let place_places = kca_net::PlacePlaces::new("[]".as_bytes()).unwrap();
    assert_eq!(place_places.len(), 0);
}

fn validate_place_places_common(_place_places: &kca_net::PlacePlaces) {
    // TODO: Does this data have any internal consistency to maintain?
}

#[test]
fn parse_fixture_place_places_info_20240623() {
    let manfest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // https://kancolle-arcade.net/ac/api/BlueprintList/info
    let fixture =
        path!(Path::new(&manfest_dir) / "tests" / "fixtures" / "2024-06-23" / "Place_places.json");

    let data = BufReader::new(File::open(fixture).unwrap());
    let place_places = kca_net::PlacePlaces::new(data).unwrap();

    assert_eq!(place_places.len(), 710);
    validate_place_places_common(&place_places);
}
