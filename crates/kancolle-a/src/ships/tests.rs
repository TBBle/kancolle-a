use super::*;

#[test]
fn test_ship_blueprint_name() {
    // Cases grabbed from the web version's wiki, so includes ships not yet in kca.
    let tests = vec![
        // Degenerate cases
        ("", ""),
        ("", "改"),
        ("", "改二"),
        // Generic case: 鳥海
        ("鳥海", "鳥海"),
        ("鳥海", "鳥海改"),
        ("鳥海", "鳥海改二"),
        // Generic case: 時雨
        ("時雨", "時雨"),
        ("時雨", "時雨改"),
        ("時雨", "時雨改二"),
        ("時雨", "時雨改三"),
        // Specific case: 大鯨
        ("大鯨", "大鯨"),
        ("大鯨", "龍鳳"),
        ("大鯨", "龍鳳改"),
        ("大鯨", "龍鳳改二戊"),
        ("大鯨", "龍鳳改二"),
        // Specific case: 響
        ("響", "響"),
        ("響", "響改"),
        ("響", "Верный"),
        // Specific case: Littorio
        ("Littorio", "Littorio"),
        ("Littorio", "Italia"),
        // Specific case: 千代田
        ("千代田", "千代田"),
        ("千代田", "千代田改"),
        ("千代田", "千代田甲"),
        ("千代田", "千代田航"),
        ("千代田", "千代田航改"),
        ("千代田", "千代田航改二"),
        // Specific case: 千歳
        ("千歳", "千歳"),
        ("千歳", "千歳改"),
        ("千歳", "千歳甲"),
        ("千歳", "千歳航"),
        ("千歳", "千歳航改"),
        ("千歳", "千歳航改二"),
        // Specific case: U-511
        ("U-511", "U-511"),
        ("U-511", "U-511改"),
        ("U-511", "呂500"),
        // Specific case: Гангут
        ("Гангут", "Гангут"),
        ("Гангут", "Октябрьская революция"),
        ("Гангут", "Гангут два"),
        // Specific case: 春日丸
        ("春日丸", "春日丸"),
        ("春日丸", "大鷹"),
        ("春日丸", "大鷹改"),
        ("春日丸", "大鷹改二"),
    ];

    for (expected, input) in tests {
        assert_eq!(expected, ship_blueprint_name(input));
    }
}

#[test]
fn test_ship_blueprint_costs() {
    assert_eq!(
        (3, 0),
        ship_blueprint_costs("Anyname", "Anytype", 0).unwrap()
    );
    assert_eq!(None, ship_blueprint_costs("Anyname", "Anytype", 6));

    // TODO: What's a good way to test this further that isn't just repeating the function?
}
