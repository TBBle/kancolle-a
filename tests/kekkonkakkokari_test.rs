use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use path_macro::path;
// TODO: Wrap a nice API around this.
use kancolle_a::importer::kancolle_arcade_net as kca_net;

#[test]
fn parse_empty_kekkonkakkokari_reader() {
    kca_net::KekkonKakkoKariList::new(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_kekkonkakkokari_vector() {
    let kekkonkakkokari = kca_net::KekkonKakkoKariList::new("[]".as_bytes()).unwrap();
    assert_eq!(kekkonkakkokari.len(), 0);
}

fn validate_kekkonkakkokari_common(_kekkonkakkokari: &kca_net::KekkonKakkoKariList) {
    // TODO: Does this data have any internal consistency to maintain?
}

#[test]
fn parse_fixture_kekkonkakkokari_info_20240623() {
    let manfest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json (Not an API, no auth required)
    let fixture =
        path!(Path::new(&manfest_dir) / "tests" / "fixtures" / "2024-06-23" / "kanmusu_list.json");

    let data = BufReader::new(File::open(fixture).unwrap());
    let kekkonkakkokari = kca_net::KekkonKakkoKariList::new(data).unwrap();

    assert_eq!(kekkonkakkokari.len(), 441);
    validate_kekkonkakkokari_common(&kekkonkakkokari);
}
