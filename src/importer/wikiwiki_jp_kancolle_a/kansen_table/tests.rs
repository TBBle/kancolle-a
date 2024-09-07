use super::*;

use lazy_static_include::*;

lazy_static_include_bytes! {
// https://wikiwiki.jp/kancolle-a/?cmd=edit&page=艦船%2Fテーブル
    KANSEN_2024_09_02 => "tests/fixtures/2024-09-02/艦船_テーブル.txt",
    KANSEN_2024_09_08 => "tests/fixtures/2024-09-08/艦船_テーブル.txt",
// https://wikiwiki.jp/kancolle-a/?cmd=edit&page=改造艦船%2Fテーブル
    KAIZOU_KANSEN_2024_05_01 => "tests/fixtures/2024-05-01/改造艦船_テーブル.txt",
    KAIZOU_KANSEN_2024_09_08 => "tests/fixtures/2024-09-08/改造艦船_テーブル.txt",
}

#[test]
fn parse_empty_kansen_table_reader() {
    let kansen_table = read_kansen_table(std::io::empty()).unwrap();
    assert_eq!(kansen_table.len(), 0);
}

#[test]
fn parse_empty_kansen_table_vector() {
    let kansen_table = read_kansen_table("[]".as_bytes()).unwrap();
    assert_eq!(kansen_table.len(), 0);
}

#[test]
fn parse_fixture_kansen_2024_09_02() {
    let kansen_table = read_kansen_table(KANSEN_2024_09_02.as_ref()).unwrap();
    // TODO: Six missing versus the tcbook tests (285 entries):
    // https://wikiwiki.jp/kancolle-a/由良改二
    // https://wikiwiki.jp/kancolle-a/満潮改二
    // https://wikiwiki.jp/kancolle-a/最上改二
    // https://wikiwiki.jp/kancolle-a/最上改二特
    // https://wikiwiki.jp/kancolle-a/涼月
    // https://wikiwiki.jp/kancolle-a/涼月改
    assert_eq!(kansen_table.len(), 279);
    let 長門 = KansenShip {
        no: 1,
        rarity: 6,
        name: "長門".to_string(),
        model: "長門型".to_string(),
        ship_no: "1番艦".to_string(),
        ship_class: "戦艦".to_string(),
        endurance: 80,
        firepower: Some(82),
        armor: Some(75),
        torpedo: Some(0),
        evasion: Some(24),
        anti_aircraft: Some(31),
        aircraft_load: 12,
        anti_submarine: Some(0),
        speed: "低".to_string(),
        search: Some(12),
        range: "長".to_string(),
        luck: 20,
        notes: "長門改,長門改二(No.341)".to_string(),
    };
    assert_eq!(kansen_table[0], 長門);
}

#[test]
fn parse_fixture_kaizou_kansen_2024_05_01() {
    // This data was faulty, one of the headers had a full-width space in its name.
    assert!(read_kansen_table(KAIZOU_KANSEN_2024_05_01.as_ref())
        .unwrap_err()
        .to_string()
        .contains("unknown field `艦\u{3000}型"));
}

#[test]
fn parse_fixture_kansen_2024_09_08() {
    let kansen_table = read_kansen_table(KANSEN_2024_09_08.as_ref()).unwrap();
    assert_eq!(kansen_table.len(), 285);
    let 長門 = KansenShip {
        no: 1,
        rarity: 6,
        name: "長門".to_string(),
        model: "長門型".to_string(),
        ship_no: "1番艦".to_string(),
        ship_class: "戦艦".to_string(),
        endurance: 80,
        firepower: Some(82),
        armor: Some(75),
        torpedo: Some(0),
        evasion: Some(24),
        anti_aircraft: Some(31),
        aircraft_load: 12,
        anti_submarine: Some(0),
        speed: "低".to_string(),
        search: Some(12),
        range: "長".to_string(),
        luck: 20,
        notes: "長門改,長門改二(No.341)".to_string(),
    };
    assert_eq!(kansen_table[0], 長門);
}

#[test]
fn parse_fixture_kaizou_kansen_2024_09_08() {
    let kansen_table = read_kansen_table(KAIZOU_KANSEN_2024_09_08.as_ref()).unwrap();
    assert_eq!(kansen_table.len(), 160);
    let 長門改 = KansenShip {
        no: 1,
        rarity: 6,
        name: "長門改".to_string(),
        model: "長門型".to_string(),
        ship_no: "1番艦".to_string(),
        ship_class: "戦艦".to_string(),
        endurance: 90,
        firepower: Some(90),
        armor: Some(85),
        torpedo: Some(0),
        evasion: Some(24),
        anti_aircraft: Some(33),
        aircraft_load: 12,
        anti_submarine: Some(0),
        speed: "低速".to_string(),
        search: Some(15),
        range: "長".to_string(),
        luck: 32,
        notes: "".to_string(),
    };
    assert_eq!(kansen_table[0], 長門改);
}

#[test]
fn test_clean_record() {
    let test_record = StringRecord::from(vec![
        "",
        "Normal",
        "~TildePrefixed",
        "[[SimpleLink]]",
        "[[FancyLink>Target]]",
        "~[[TildePrefixedFancyLink]]",
        "Embedded [[SimpleLink]]",
        "[[Multiple]] [[Simple]] [[Links]]",
        "[[Mixed]] [[Simple]] [[And>Target]] [[Fancy>>Target]] [[Links]]",
        " Leading whitespace",
        "Trailing whitespace ",
        "Trailing newline\n",
        " ", // Whitespace only
        "",
    ]);
    let expected = StringRecord::from(vec![
        "Normal",
        "TildePrefixed",
        "SimpleLink",
        "FancyLink",
        "TildePrefixedFancyLink",
        "Embedded SimpleLink",
        "Multiple Simple Links",
        "Mixed Simple And Fancy Links",
        "Leading whitespace",
        "Trailing whitespace",
        "Trailing newline",
        "",
    ]);
    assert_eq!(clean_record(&test_record), expected);
}
