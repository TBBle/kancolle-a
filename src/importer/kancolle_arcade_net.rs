//! Module for importers for https://kancolle-arcade.net/ac/api/ resources

use chrono::{DateTime, NaiveDate, Utc};
use derive_getters::Getters;
use serde::Deserialize;
use serde_json::Result;
use std::sync::OnceLock;
use std::{collections::HashMap, io::Read, ops::Deref};

#[derive(Debug, Deserialize)]
pub struct TcBook(Vec<BookShip>);

impl TcBook {
    /// Parses a TcBook from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(tcbook_reader: impl Read) -> Result<TcBook> {
        let result: TcBook = serde_json::from_reader(tcbook_reader)?;
        Ok(result)
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for TcBook {
    type Target = Vec<BookShip>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShip {
    book_no: u16,
    ship_class: Option<String>,
    ship_class_index: Option<i16>,
    ship_type: String,
    ship_model_num: String,
    ship_name: String,
    card_index_img: String,
    card_list: Vec<BookShipCardPage>,
    variation_num: u16,
    acquire_num: u16,
    lv: u16,
    is_married: Option<Vec<bool>>,
    married_img: Option<Vec<String>>,
}

// Notes for future functions
// * Card images (s/tc_NO_xxx.jpg) live in https://kancolle-arcade.net/ac/resources/pictureBook/
// * Status images (i/i_xxx.png) live in https://kancolle-arcade.net/ac/resources/chara/
// ** Status images end with _n, _bs, _bm, or _bl. (Not sure if there's one for sunk?)

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BookShipCardPageSource {
    Unknown, // Fallback
    Normal,  // Priority 0 is always this

    // 期間限定ドロップイベント
    DecisiveBattle, // 決戦mode: May 2019 once-off
    Swimsuit,       // 水着mode, 里帰り水着mode, 夏のお嬢さんmode: July 2019 onward
    Christmas,      // クリスマスmode: December 2019 onward
    Halloween,      // ハロウィンmode: Oct 2020 onward
    Valentine,      // バレンタインmode: Feb 2021 onward
    PacificSaury,   // 秋刀魚mode: Oct 2021 onward (鎮守府秋刀魚祭り)
    Fishing,        // F作業mode: Oct 2021 onward (鎮守府秋刀魚祭り)
    SundayBest,     // 晴れ着mode: Jan 2022 onward
    RainySeason,    // 梅雨mode: May-June 2022 onward
    Yukata,         // 浴衣mode: Sep 2023 onward
    SurigaoStrait, // スリガオ海峡突入mode: Jan 2024 event (第拾肆回期間限定海域：捷号決戦！邀撃、レイテ沖海戦（前篇）)

    // オリジナルイラストカード
    OriginalIllustration(u8), // Theory: Last page, may have a different variation_num_in_page
}

static BOOK_SHIP_SOURCES: OnceLock<HashMap<u16, Vec<BookShipCardPageSource>>> = OnceLock::new();

fn init_book_ship_sources() {
    use BookShipCardPageSource::*;
    BOOK_SHIP_SOURCES.get_or_init(|| {
        let mut sources = HashMap::new();

        // TODO: A clean way to panic on double-entry. (Or even better data format.)
        // 陸奥, 陸奥改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(2, vec![Yukata]);
        // 雪風改: https://kancolle-a.sega.jp/players/information/200811_1.html
        // XXX: Needs special handling! 改-only. Special 夏のお嬢さんmode.
        sources.insert(5, vec![Swimsuit]);
        // 赤城, 赤城改: https://kancolle-a.sega.jp/players/information/211230_2.html
        sources.insert(6, vec![SundayBest]);
        // 加賀, 加賀改:
        // * https://kancolle-a.sega.jp/players/information/211230_2.html
        sources.insert(7, vec![SundayBest, OriginalIllustration(1)]);
        // 島風, 島風改:
        // * (DecisiveBattle) https://kancolle-a.sega.jp/players/information/190508_1.html
        // * (OriginalIllustration)
        sources.insert(10, vec![DecisiveBattle, OriginalIllustration(2)]);
        // 敷波改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(18, vec![OriginalIllustration(1)]);
        // 大井: https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(19, vec![RainySeason]);
        // 鳳翔:
        sources.insert(25, vec![OriginalIllustration(1)]);
        // 扶桑, 扶桑改: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(26, vec![RainySeason]);
        // 球磨, 球磨改: https://kancolle-a.sega.jp/players/information/2212_xmas.html
        sources.insert(39, vec![Christmas]);
        // 多摩, 多摩改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(40, vec![PacificSaury]);
        // 由良, 由良改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(45, vec![Yukata]);
        sources.insert(46, vec![OriginalIllustration(1)]);
        sources.insert(48, vec![OriginalIllustration(1)]);
        // 最上:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(51, vec![SurigaoStrait, RainySeason]);
        // 加古, 加古改: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(53, vec![RainySeason]);
        // 朧, 朧改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // *
        sources.insert(67, vec![Fishing, OriginalIllustration(1)]);
        // 曙, 曙改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * https://kancolle-a.sega.jp/players/information/211005_1.html
        // *
        sources.insert(68, vec![PacificSaury, Fishing, OriginalIllustration(2)]);
        // 漣:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // *
        sources.insert(69, vec![PacificSaury, OriginalIllustration(1)]);
        // 潮, 潮改:
        // * https://kancolle-a.sega.jp/players/information/2201_valentine.html
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // *
        sources.insert(70, vec![Valentine, PacificSaury, OriginalIllustration(1)]);
        sources.insert(71, vec![OriginalIllustration(1)]);
        // 雷, 雷改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(73, vec![Yukata]);
        // 電, 電改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(74, vec![Yukata]);
        sources.insert(79, vec![OriginalIllustration(1)]);
        sources.insert(81, vec![OriginalIllustration(2)]);
        // 夕立, 夕立改:
        // * https://kancolle-a.sega.jp/players/information/201026_1.html
        // *
        sources.insert(82, vec![Halloween, OriginalIllustration(1)]);
        // 朝潮, 朝潮改: https://kancolle-a.sega.jp/players/information/201013_1.html
        sources.insert(85, vec![Halloween]);
        // 大潮, 大潮改: https://kancolle-a.sega.jp/players/information/2310_sauryfestival.html
        sources.insert(86, vec![Fishing]);
        // 満潮, 満潮改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(87, vec![PacificSaury]);
        // 祥鳳, 祥鳳改:
        // * https://kancolle-a.sega.jp/players/information/2206_rainy_season_addition.html
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(94, vec![RainySeason, PacificSaury]);
        // 大井改: https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(97, vec![RainySeason]);
        // 最上改:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(101, vec![SurigaoStrait, RainySeason]);
        sources.insert(102, vec![OriginalIllustration(1)]);
        sources.insert(103, vec![OriginalIllustration(1)]);
        // 翔鶴, 翔鶴改: https://kancolle-a.sega.jp/players/information/211214_1.html
        sources.insert(106, vec![Christmas]);
        // 瑞鶴: https://kancolle-a.sega.jp/players/information/211214_1.html
        sources.insert(107, vec![Christmas]);
        // 瑞鶴改: https://kancolle-a.sega.jp/players/information/211214_1.html
        sources.insert(108, vec![Christmas]);
        // 夕張, 夕張改: https://kancolle-a.sega.jp/players/information/2205_rainy_season.html
        sources.insert(111, vec![RainySeason]);
        // 大井改二: https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(114, vec![RainySeason]);
        // 鈴谷: https://kancolle-a.sega.jp/players/information/191203_1.html
        sources.insert(124, vec![Christmas]);
        // 熊野: https://kancolle-a.sega.jp/players/information/191203_1.html
        sources.insert(125, vec![Christmas]);
        // 鈴谷改: https://kancolle-a.sega.jp/players/information/191203_1.html
        sources.insert(129, vec![Christmas]);
        // 熊野改: https://kancolle-a.sega.jp/players/information/191203_1.html
        sources.insert(130, vec![Christmas]);
        // 夕雲, 夕雲改:
        // * https://kancolle-a.sega.jp/players/information/2310_sauryfestival.html
        // *
        sources.insert(133, vec![PacificSaury, OriginalIllustration(1)]);
        sources.insert(134, vec![OriginalIllustration(1)]);
        sources.insert(135, vec![OriginalIllustration(1)]);
        // 衣笠改二: https://kancolle-a.sega.jp/players/information/210205_1.html
        sources.insert(142, vec![Valentine]);
        // 夕立改二:
        // * https://kancolle-a.sega.jp/players/information/201027_1.html
        // * https://kancolle-a.sega.jp/players/information/2205_rainy_season.html
        sources.insert(144, vec![RainySeason, Halloween, OriginalIllustration(2)]);
        // 時雨改二:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        // * https://kancolle-a.sega.jp/players/information/190805_1.html
        // * https://kancolle-a.sega.jp/players/information/211005_1.html
        // *
        sources.insert(
            145,
            vec![
                SurigaoStrait,
                Swimsuit,
                PacificSaury,
                OriginalIllustration(2),
            ],
        );
        // 榛名改二: https://kancolle-a.sega.jp/players/information/190722_1.html
        sources.insert(151, vec![Swimsuit]);
        // 卯月:
        // * https://kancolle-a.sega.jp/players/information/210205_1.html
        // *
        sources.insert(165, vec![Valentine, OriginalIllustration(1)]);
        // 磯風, 磯風改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(167, vec![PacificSaury]);
        // 浦風, 浦風改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(168, vec![Yukata]);
        // 浜風, 浜風改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(170, vec![Yukata]);
        sources.insert(181, vec![OriginalIllustration(1)]);
        // 大淀, 大淀改:
        // * https://kancolle-a.sega.jp/players/information/210907_1.html
        // * https://kancolle-a.sega.jp/players/information/190508_1.html
        sources.insert(183, vec![Swimsuit, OriginalIllustration(1)]);
        // 大鯨: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(184, vec![PacificSaury]);
        // 龍鳳: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(185, vec![PacificSaury]);
        sources.insert(205, vec![OriginalIllustration(2)]);
        // 潮改二:
        // * https://kancolle-a.sega.jp/players/information/2201_valentine.html
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(207, vec![Valentine, PacificSaury]);
        // 早霜, 早霜改:
        // * https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        // *
        sources.insert(209, vec![RainySeason, OriginalIllustration(1)]);
        // 清霜, 清霜改:
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        // *
        sources.insert(210, vec![RainySeason, OriginalIllustration(1)]);
        // 扶桑改二: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(211, vec![RainySeason]);
        // 朝雲, 朝雲改: https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        sources.insert(213, vec![SurigaoStrait]);
        // 山雲, 山雲改: https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        sources.insert(214, vec![SurigaoStrait]);
        // 野分, 野分改: https://kancolle-a.sega.jp/players/information/2210_halloween.html
        sources.insert(215, vec![Halloween]);
        // 秋月, 秋月改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(221, vec![Yukata]);
        // 初月, 初月改: https://kancolle-a.sega.jp/players/information/2310_sauryfestival.html
        sources.insert(223, vec![PacificSaury]);
        // 高波, 高波改: https://kancolle-a.sega.jp/players/information/2206_rainy_season_addition.html
        sources.insert(224, vec![RainySeason]);
        // U-511, U-511改: https://kancolle-a.sega.jp/players/information/210914_1.html (里帰り水着mode)
        sources.insert(231, vec![Swimsuit]);
        // Warspite, Warspite改:
        // * https://kancolle-a.sega.jp/players/information/2212_haregimode.html
        // *
        sources.insert(239, vec![SundayBest, OriginalIllustration(2)]);
        // Littorio: https://kancolle-a.sega.jp/players/information/200728_1.html
        sources.insert(241, vec![Swimsuit]);
        // Roma, Roma改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(242, vec![Halloween]);
        // Libeccio, Libeccio改: https://kancolle-a.sega.jp/players/information/190729_1.html
        sources.insert(243, vec![Swimsuit]);
        // Italia: https://kancolle-a.sega.jp/players/information/200728_1.html
        sources.insert(246, vec![Swimsuit]);
        // Zara, Zara改: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(248, vec![PacificSaury]);
        // 風雲, 風雲改: https://kancolle-a.sega.jp/players/information/2208_join_kazagumo_swim.html
        sources.insert(253, vec![Swimsuit]);
        // 山風, 山風改: https://kancolle-a.sega.jp/players/information/210827_2.html
        sources.insert(257, vec![Swimsuit]);
        // 速吸, 速吸改: https://kancolle-a.sega.jp/players/information/2312_xmas.html
        sources.insert(260, vec![Christmas]);
        // 朝潮改二: https://kancolle-a.sega.jp/players/information/201013_1.html
        sources.insert(263, vec![Halloween]);
        // 鹿島, 鹿島改: https://kancolle-a.sega.jp/players/information/201201_1.html
        sources.insert(265, vec![Christmas]);
        // 霞改二乙: https://kancolle-a.sega.jp/players/information/200805_1.html
        sources.insert(270, vec![Swimsuit]);
        // 神風: https://kancolle-a.sega.jp/players/information/2402_valentine.html
        sources.insert(271, vec![Valentine]);
        // 神風改: https://kancolle-a.sega.jp/players/information/2402_valentine.html
        sources.insert(276, vec![Valentine]);
        // 満潮改二:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        // * https://kancolle-a.sega.jp/players/information/2310_sauryfestival.html
        sources.insert(289, vec![SurigaoStrait, PacificSaury]);
        // Richelieu, Richelieu改: https://kancolle-a.sega.jp/players/information/2312_haregimode.html
        sources.insert(292, vec![SundayBest]);
        sources.insert(391, vec![OriginalIllustration(1)]);

        // These cannot be determined yet, as there's multiple variations I don't
        // have in my current data

        // 大和:
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/190813_1.html
        // * (SundayBest) https://kancolle-a.sega.jp/players/information/211230_2.html
        sources.insert(131, vec![Unknown, Unknown]);
        // 大和改:
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/190813_1.html
        // * (SundayBest) https://kancolle-a.sega.jp/players/information/211230_2.html
        sources.insert(136, vec![Unknown, Unknown]);
        // 由良改二:
        // * (Yukata) https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/2307_join_yura-kai2_swim.html
        sources.insert(288, vec![Unknown, Unknown]);

        // Info store for ships I don't have at all.
        // 霞改二: https://kancolle-a.sega.jp/players/information/200805_1.html
        // 朝潮改二, 朝潮改二丁: https://kancolle-a.sega.jp/players/information/201013_1.html
        // 呂500: https://kancolle-a.sega.jp/players/information/210914_1.html
        // Grecale, Grecale改: https://kancolle-a.sega.jp/players/information/2310_halloween.html
        // 瑞鶴改二: https://kancolle-a.sega.jp/players/information/211214_1.html
        // 瑞鶴改二甲: https://kancolle-a.sega.jp/players/information/211214_1.html
        // 翔鶴改二: https://kancolle-a.sega.jp/players/information/211214_1.html
        // 翔鶴改二甲: https://kancolle-a.sega.jp/players/information/211214_1.html
        // 熊野改二: https://kancolle-a.sega.jp/players/information/201201_1.html
        // 鈴谷改二: https://kancolle-a.sega.jp/players/information/201201_1.html
        // 秋津洲, 秋津洲改: https://kancolle-a.sega.jp/players/information/2207_join_akitusima_swim.html
        // Johnston, Johnston改: https://kancolle-a.sega.jp/players/information/2207_join_Johnston_swim.html
        // Gotland, Gotland改: https://kancolle-a.sega.jp/players/information/2307_join_gotland_swim.html

        // TODO: Original Illustration details
        // * https://kancolle-a.sega.jp/players/information/2404_spring_seaarea_detail.html (Plus previous year's event?)

        sources
    });
}

impl BookShip {
    /// Reports the event-source for the given page ("priority") of a TcBook entry
    pub fn source(&self, priority: u16) -> BookShipCardPageSource {
        use BookShipCardPageSource::*;
        if priority == 0 {
            return Normal;
        }
        init_book_ship_sources();
        if let Some(sources) = BOOK_SHIP_SOURCES.get().unwrap().get(self.book_no()) {
            if sources.len() + 1 != self.card_list().len() {
                // Old data. Not sure if there's a good way to handle this; as new events are inserted before
                // Original Illustration Cards, but otherwise in order of addition, it appears.
                // Note that Surigao Strait cards in particular were from an in-person event many
                // years before the actual in-game event in 2024, which is why it's first when there's multiple such events.
                // We _could_ assume the last slot is the original illustration slot if it is the wrong shape, but without
                // particular value in reading old data sets (except for tests) it's not particularly worth it.
                return Unknown;
            };
            sources
                .get((priority - 1) as usize)
                .or(Some(&Unknown))
                .unwrap()
                .to_owned()
        } else {
            Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_ship_source() {
        let ship = BookShip {
            book_no: 6,
            ship_class: None,
            ship_class_index: None,
            ship_type: "".to_string(),
            ship_model_num: "".to_string(),
            ship_name: "".to_string(),
            card_index_img: "".to_string(),
            card_list: vec![
                BookShipCardPage {
                    priority: 0,
                    card_img_list: vec!["".to_string(), "".to_string()],
                    status_img: None,
                    variation_num_in_page: 3,
                    acquire_num_in_page: 0,
                },
                BookShipCardPage {
                    priority: 0,
                    card_img_list: vec!["".to_string(), "".to_string()],
                    status_img: None,
                    variation_num_in_page: 3,
                    acquire_num_in_page: 0,
                },
            ],
            variation_num: 6,
            acquire_num: 0,
            lv: 1,
            is_married: None,
            married_img: None,
        };

        use BookShipCardPageSource::*;
        assert_eq!(ship.source(0), Normal);
        assert_eq!(ship.source(1), SundayBest);
        assert_eq!(ship.source(2), Unknown);
    }
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShipCardPage {
    priority: u16,
    card_img_list: Vec<String>,
    status_img: Option<Vec<String>>,
    variation_num_in_page: u16,
    acquire_num_in_page: u16,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintList(Vec<BlueprintShip>);

impl BlueprintList {
    /// Parses a BlueprintList from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(blueprintlist_reader: impl Read) -> Result<BlueprintList> {
        let result: BlueprintList = serde_json::from_reader(blueprintlist_reader)?;
        Ok(result)
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for BlueprintList {
    type Target = Vec<BlueprintShip>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Notes for future functions
// * Status images (i/i_xxx.png) live in https://kancolle-arcade.net/ac/resources/chara/
// ** Status images end with _n, _bs, _bm, or _bl. (Not sure if there's one for sunk?)
// * Expiration date appears to be the 11th of the month of expiry. Not clear why.
// ** True expiration date is 23:59 on the last date of the month.
// ** Or I made a mistake, I guess?

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BlueprintShip {
    ship_class_id: u16,
    ship_class_index: u16,
    ship_sort_no: u16,
    ship_type: String,
    ship_name: String,
    status_img: String,
    blueprint_total_num: u16,
    exists_warning_for_expiration: bool,
    expiration_date_list: Vec<BlueprintExpirationDate>,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BlueprintExpirationDate {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    expiration_date: DateTime<Utc>,
    blueprint_num: u16,
    expire_this_month: bool,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PlaceTopRegion {
    // If this is actually a standard list, I can't find the source.
    // But these specific divisions (e.g., merged HOKKAIDO_TOHOKU) show
    // up in URLs a lot, so there's presumably some standard data list somewhere, or just something
    // taught in school that no one's actually documented anywhere formal. (Or coincidence/parallel thinking)
    // Notably, the break-down doesn't match any of the ones shown at
    // https://ja.wikipedia.org/wiki/%E6%97%A5%E6%9C%AC%E3%81%AE%E5%9C%B0%E5%9F%9F#%E4%B8%BB%E3%81%AA%E5%9C%B0%E5%9F%9F%E3%83%96%E3%83%AD%E3%83%83%E3%82%AF
    top_region_enum: String,
    name: String,
    prefecture_beans: Vec<PlacePrefectureBean>,
}

#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PlacePrefectureBean {
    region_enum: String,
    name: String,
    /// JIS X 0401 都道府県コード: 01..47 (Also ISO 3166-2:JP)
    jis_code: u8,
}

#[derive(Debug, Deserialize)]
pub struct PlaceDistricts(Vec<PlaceTopRegion>);

impl PlaceDistricts {
    /// Parses a PlaceDistricts from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(reader: impl Read) -> Result<PlaceDistricts> {
        let result: PlaceDistricts = serde_json::from_reader(reader)?;
        Ok(result)
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for PlaceDistricts {
    type Target = Vec<PlaceTopRegion>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
pub struct PlacePlaces(Vec<Place>);

impl PlacePlaces {
    /// Parses a PlacePlaces from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(reader: impl Read) -> Result<PlacePlaces> {
        let result: PlacePlaces = serde_json::from_reader(reader)?;
        Ok(result)
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for PlacePlaces {
    type Target = Vec<Place>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: This struct should also be used for placesFromHere handling, but there's
// a few differences that need to be handled.
#[derive(Debug, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Place {
    id: u32,
    distance: String, // No data in places output.
    name: String,
    tel: String,
    address: String,
    station: String,
    open_time: String,
    close_time: String,
    special_info: String,
    country: String,
    /// Reference to PlaceStructureBean.region_enum
    region_enum: String,
    latitude: String,  // Float-in-string.
    longitude: String, // Float-in-string.
    zoom_level: u8,    // Google Maps API zoom level.
}

// ケッコンカッコカリ, aka 結婚（仮）
#[derive(Debug, Deserialize)]
pub struct KekkonKakkoKariList(Vec<KekkonKakkoKari>);

impl KekkonKakkoKariList {
    /// Parses a PlacePlaces from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn new(reader: impl Read) -> Result<KekkonKakkoKariList> {
        let result: KekkonKakkoKariList = serde_json::from_reader(reader)?;
        Ok(result)
    }
}

// Implementing Deref but not DerefMut so it can't be mutated.
impl Deref for KekkonKakkoKariList {
    type Target = Vec<KekkonKakkoKari>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Getters)]
#[serde(deny_unknown_fields)]
pub struct KekkonKakkoKari {
    id: u32,
    web_id: u32,
    name: String,
    name_reading: String,
    kind: String,
    category: String,
    #[serde(with = "kekkonkakkokari_date_format")]
    start_time: NaiveDate, // Technically 7am JST on this day, AFAIK.
}

mod kekkonkakkokari_date_format {
    // https://serde.rs/custom-date-format.html
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer /* , Serializer*/};

    const FORMAT: &'static str = "%Y/%m/%d";

    // pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    // where
    //     S: Serializer,
    // {
    //     let s = format!("{}", date.format(FORMAT));
    //     serializer.serialize_str(&s)
    // }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(dt)
    }
}
