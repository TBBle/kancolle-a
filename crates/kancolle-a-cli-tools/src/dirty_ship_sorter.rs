// A quick-and-dirty ship sorter, based on names and hand-maintained from
// https://wikiwiki.jp/kancolle-a/%E8%89%A6%E5%A8%98%E3%82%AB%E3%83%BC%E3%83%89%E4%B8%80%E8%A6%A72
// Parsed out of the source with gvim; not scripted, just lots of %s and %g commands.

use std::cmp::Ordering;

// TODO: There must be a better way to do this? Or at least somewhat better?
// IDEA: Teach the importer to generate this list. We could add the wiki sort order to
// the ships list during import, and then it won't be so string-comparisony.
// Note that this is here because this is the I use to sort my card collection.
// 192 is consistent with KANSEN_TABLE_SHIPS from the integration tests as you'd expect.
const WIKI_SHIP_ORDER: [&str; 192] = [
    // 戦艦
    "金剛",
    "比叡",
    "榛名",
    "霧島",
    "扶桑",
    "山城",
    "伊勢",
    "日向",
    "長門",
    "陸奥",
    "大和",
    "武蔵",
    "Bismarck",
    "Littorio",
    // "Italia",
    "Roma",
    "Warspite",
    "Гангут",
    // "Октябрьская революция",
    "Richelieu",
    "Iowa",
    // 正規空母
    "赤城",
    "加賀",
    "蒼龍",
    "飛龍",
    "翔鶴",
    "瑞鶴",
    "雲龍",
    "Saratoga",
    "Hornet",
    "Ark Royal",
    "Ranger",
    // 装甲空母
    "大鳳",
    // 軽空母
    "鳳翔",
    "龍驤",
    //"龍鳳",
    "祥鳳",
    "瑞鳳",
    "飛鷹",
    "隼鷹",
    "春日丸",
    // "大鷹",
    "Gambier Bay",
    // 水上機母艦
    "千歳",
    "千代田",
    "瑞穂",
    "秋津洲",
    "Commandant Teste",
    // 重巡洋艦
    "古鷹",
    "加古",
    "青葉",
    "衣笠",
    "妙高",
    "那智",
    "足柄",
    "羽黒",
    "高雄",
    "愛宕",
    "摩耶",
    "鳥海",
    "最上",
    "三隈",
    "鈴谷",
    "熊野",
    "利根",
    "筑摩",
    "Prinz Eugen",
    "Zara",
    "Pola",
    // 軽巡洋艦
    "天龍",
    "龍田",
    "球磨",
    "多摩",
    "北上",
    "大井",
    "木曾",
    "長良",
    "五十鈴",
    "名取",
    "由良",
    "鬼怒",
    "阿武隈",
    "夕張",
    "川内",
    "神通",
    "那珂",
    "阿賀野",
    "能代",
    "矢矧",
    "酒匂",
    "大淀",
    // 練習巡洋艦
    "香取",
    "鹿島",
    // 駆逐艦
    "神風",
    "春風",
    "睦月",
    "如月",
    "弥生",
    "卯月",
    "皐月",
    "文月",
    "長月",
    "菊月",
    "三日月",
    "望月",
    "吹雪",
    "白雪",
    "初雪",
    "深雪",
    "叢雲",
    "磯波",
    "綾波",
    "敷波",
    "朧",
    "曙",
    "漣",
    "潮",
    "暁",
    "響",
    // "Верный",
    "雷",
    "電",
    "初春",
    "子日",
    "若葉",
    "初霜",
    "白露",
    "時雨",
    "村雨",
    "夕立",
    "春雨",
    "五月雨",
    "海風",
    "山風",
    "江風",
    "涼風",
    "朝潮",
    "大潮",
    "満潮",
    "荒潮",
    "朝雲",
    "山雲",
    "霰",
    "霞",
    "陽炎",
    "不知火",
    "黒潮",
    "初風",
    "雪風",
    "天津風",
    "時津風",
    "浦風",
    "磯風",
    "浜風",
    "谷風",
    "野分",
    "嵐",
    "萩風",
    "舞風",
    "秋雲",
    "夕雲",
    "巻雲",
    "風雲",
    "長波",
    "高波",
    "朝霜",
    "早霜",
    "清霜",
    "秋月",
    "照月",
    "涼月",
    "初月",
    "島風",
    "Z1",
    "Z3",
    "Grecale",
    "Libeccio",
    "Fletcher",
    "Johnston",
    "Ташкент",
    "Janus",
    // 潜水艦
    "伊168",
    "伊8",
    "伊19",
    "伊58",
    "U-511",
    //"呂500",
    "まるゆ",
    // 潜水空母
    "伊401",
    "伊13",
    "伊14",
    // 補給艦
    "神威",
    "速吸",
    // 軽(航空)巡洋艦
    "Gotland",
    // 防空巡洋艦
    "Atlanta",
    // 潜水母艦
    "大鯨",
    // 揚陸艦
    "あきつ丸",
    // 工作艦
    "明石",
];

pub fn dirty_ship_wiki_cmp(left: &str, right: &str) -> Ordering {
    let left_index = WIKI_SHIP_ORDER.iter().position(|&element| element == left);
    let right_index = WIKI_SHIP_ORDER.iter().position(|&element| element == right);
    match (left_index, right_index) {
        // String-comparison fallback for two unknown ships.
        (None, None) => left.cmp(right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(left_index), Some(right_index)) => left_index.cmp(&right_index),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kancolle_a::ships::ShipsBuilder;
    #[tokio::test]
    async fn wiki_ship_order_comprehensivity() {
        for ship_name in ShipsBuilder::default()
            .build()
            .await
            .unwrap()
            .iter()
            .map(|ship_entry| ship_entry.0)
        {
            assert!(WIKI_SHIP_ORDER.iter().any(|&element| element == ship_name));
        }
    }

    #[tokio::test]
    async fn wiki_ship_order_wastefulness() {
        let ship_names: Vec<String> = ShipsBuilder::default()
            .build()
            .await
            .unwrap()
            .iter()
            .map(|ship_entry| ship_entry.0.to_owned())
            .collect();
        for sort_name in WIKI_SHIP_ORDER {
            assert!(
                ship_names.iter().any(|element| element == sort_name),
                "{sort_name}"
            )
        }
    }
}
