use super::*;
use serde::Deserialize;

#[test]
fn test_array_of_simple_objects() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
        str: String,
    }

    let data = r"\
    |~int|~str|h\n\
    |5|hi|\n\
    |10|there|\n\
    |~int|~str|f\n\
    ";

    let expected = vec![
        Test {
            int: 5,
            str: "hi".to_string(),
        },
        Test {
            int: 10,
            str: "there".to_string(),
        },
    ];
    assert_eq!(expected, from_str::<Vec<Test>>(data).unwrap());
}
