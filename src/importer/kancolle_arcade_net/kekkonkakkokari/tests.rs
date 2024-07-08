use super::kanmusu_list::*;

use chrono::NaiveDate;

use lazy_static_include::*;

// https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json (Not an API, no auth required)
lazy_static_include_bytes! {
    KANMUSU_2024_06_23 => "tests/fixtures/2024-06-23/kanmusu_list.json",
}

#[test]
fn parse_empty_kekkonkakkokari_reader() {
    read_kekkonkakkokarilist(std::io::empty()).unwrap_err();
}

#[test]
fn parse_empty_kekkonkakkokari_vector() {
    let kekkonkakkokari = read_kekkonkakkokarilist("[]".as_bytes()).unwrap();
    assert_eq!(kekkonkakkokari.len(), 0);
}

fn validate_kekkonkakkokari_common(_kekkonkakkokari: &KekkonKakkoKariList) {
    // TODO: The list appears date-ordered, maybe validate that? Not really using it as a precondition though.
}

#[test]
fn parse_fixture_kekkonkakkokari_info_20240623() {
    let kekkonkakkokari = read_kekkonkakkokarilist(KANMUSU_2024_06_23.as_ref()).unwrap();

    assert_eq!(kekkonkakkokari.len(), 441);
    validate_kekkonkakkokari_common(&kekkonkakkokari);

    assert_eq!(
        *kekkonkakkokari[0].start_time(),
        NaiveDate::from_ymd_opt(2018, 2, 16).unwrap()
    );
    assert_eq!(
        *kekkonkakkokari[440].start_time(),
        NaiveDate::from_ymd_opt(2024, 6, 13).unwrap()
    );
}
