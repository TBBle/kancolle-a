//! Module for importer for https://kancolle-arcade.net/ac/api/TcBook/info

use serde::Deserialize;
use serde_json::Result;
use std::sync::OnceLock;
use std::{collections::HashMap, io::Read};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumString, VariantNames};

pub type TcBook = Vec<BookShip>;

/// Parses a TcBook from the provided JSON reader.
/// Fails if not given a JSON array, or expected data structure does not match.
pub(crate) fn read_tclist(tcbook_reader: impl Read) -> Result<TcBook> {
    let result: TcBook = serde_json::from_reader(tcbook_reader)?;
    Ok(result)
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShip {
    pub book_no: u16,
    pub ship_class: Option<String>,
    pub ship_class_index: Option<i16>,
    pub ship_type: String,
    pub ship_model_num: String,
    pub ship_name: String,
    pub card_index_img: String,
    pub card_list: Vec<BookShipCardPage>,
    pub variation_num: u16,
    pub acquire_num: u16,
    pub lv: u16,
    pub is_married: Option<Vec<bool>>,
    pub married_img: Option<Vec<String>>,
}

// Notes for future functions
// * Card images (s/tc_NO_xxx.jpg) live in https://kancolle-arcade.net/ac/resources/pictureBook/
// * Status images (i/i_xxx.png) live in https://kancolle-arcade.net/ac/resources/chara/
// ** Status images end with _n, _bs, _bm, or _bl. (Not sure if there's one for sunk?)

#[derive(Debug, PartialEq, Eq, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumString, Display, AsRefStr, VariantNames))]
pub enum BookShipCardPageSource {
    Unknown, // Fallback
    Normal,  // Priority 0 is always this

    // 期間限定ドロップイベント
    DecisiveBattle, // 決戦mode, エンガノ岬決戦mode, スリガオ海峡突入mode: Once-off events. See https://kancolle-a.sega.jp/players/information/2410_seaarea_event15_start.html
    Swimsuit,       // 水着mode, 里帰り水着mode, 夏のお嬢さんmode: July 2019 onward
    Christmas,      // クリスマスmode: December 2019 onward
    Halloween,      // ハロウィンmode: Oct 2020 onward
    Valentine,      // バレンタインmode: Feb 2021 onward
    PacificSaury,   // 秋刀魚mode: Oct 2021 onward (鎮守府秋刀魚祭り)
    Fishing,        // F作業mode: Oct 2021 onward (鎮守府秋刀魚祭り)
    SundayBest,     // 晴れ着mode: Jan 2022 onward
    RainySeason,    // 梅雨mode: May-June 2022 onward
    Yukata,         // 浴衣mode: Sep 2023 onward

    // オリジナルイラストカード
    // Should be the last page, if present; not enough data to know if kai/non-kai are grouped.
    // True if the card belongs to the second row, i.e. 改
    OriginalIllustration1(bool),
    OriginalIllustration2(bool, bool),
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
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/2206_seaarea_event12_detail.html
        sources.insert(7, vec![SundayBest, OriginalIllustration1(true)]);
        // 島風, 島風改:
        // * (DecisiveBattle) https://kancolle-a.sega.jp/players/information/190508_1.html
        // * (OriginalIllustration)
        // ** (改) https://kancolle-a.sega.jp/players/information/200901_2.html
        // ** (改) https://kancolle-a.sega.jp/players/information/210409_1.html
        sources.insert(10, vec![DecisiveBattle, OriginalIllustration2(true, true)]);
        // 敷波, 敷波改: (改) https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(18, vec![OriginalIllustration1(true)]);
        // 大井: https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(19, vec![RainySeason]);
        // 鳳翔, 鳳翔改: (改) https://kancolle-a.sega.jp/players/information/2303_seaarea_event13_detail.html
        sources.insert(25, vec![OriginalIllustration1(true)]);
        // 扶桑, 扶桑改: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(26, vec![RainySeason]);
        // 球磨, 球磨改: https://kancolle-a.sega.jp/players/information/2212_xmas.html
        sources.insert(39, vec![Christmas]);
        // 多摩, 多摩改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(40, vec![PacificSaury]);
        // 由良, 由良改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(45, vec![Yukata]);
        // 川内, 川内改: (改) https://kancolle-a.sega.jp/players/information/2302_card_tsuika.html
        sources.insert(46, vec![OriginalIllustration1(true)]);
        // 那珂, 那珂改: (改) https://kancolle-a.sega.jp/players/information/190914_1.html
        sources.insert(48, vec![OriginalIllustration1(true)]);
        // 最上:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html (スリガオ海峡突入mode)
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(51, vec![DecisiveBattle, RainySeason]);
        // 加古, 加古改: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(53, vec![RainySeason]);
        // 朧, 朧改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/2404_spring_seaarea_detail.html
        sources.insert(67, vec![Fishing, OriginalIllustration1(true)]);
        // 曙, 曙改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * https://kancolle-a.sega.jp/players/information/211005_1.html
        // * (OriginalIllustration)
        // ** (改) https://kancolle-a.sega.jp/players/information/211027_1.html
        // ** (改) https://kancolle-a.sega.jp/players/information/210409_1.html
        sources.insert(
            68,
            vec![PacificSaury, Fishing, OriginalIllustration2(true, true)],
        );
        // 漣, 漣改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/2404_spring_seaarea_detail.html
        sources.insert(69, vec![PacificSaury, OriginalIllustration1(true)]);
        // 潮, 潮改:
        // * https://kancolle-a.sega.jp/players/information/2201_valentine.html
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/211027_1.html
        sources.insert(
            70,
            vec![Valentine, PacificSaury, OriginalIllustration1(true)],
        );
        // 暁, 暁改: (改) https://kancolle-a.sega.jp/players/information/210409_1.html
        sources.insert(71, vec![OriginalIllustration1(true)]);
        // 雷, 雷改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(73, vec![Yukata]);
        // 電, 電改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(74, vec![Yukata]);
        // 白露, 白露改: (改) https://kancolle-a.sega.jp/players/information/180316_1.html
        sources.insert(79, vec![OriginalIllustration1(true)]);
        // 村雨:
        // * (OriginalIllustration)
        // ** (改) https://kancolle-a.sega.jp/players/information/2206_seaarea_event12_detail.html
        // ** (改) https://kancolle-a.sega.jp/players/information/180316_1.html
        sources.insert(81, vec![OriginalIllustration2(true, true)]);
        // 夕立, 夕立改:
        // * https://kancolle-a.sega.jp/players/information/201026_1.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190425_2.html
        sources.insert(82, vec![Halloween, OriginalIllustration1(true)]);
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
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html (スリガオ海峡突入mode)
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(101, vec![DecisiveBattle, RainySeason]);
        // 伊勢改: https://kancolle-a.sega.jp/players/information/170420_1.html
        sources.insert(102, vec![OriginalIllustration1(false)]);
        // 日向改: https://kancolle-a.sega.jp/players/information/170420_1.html
        sources.insert(103, vec![OriginalIllustration1(false)]);
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
        // 大和:
        // * (SundayBest) https://kancolle-a.sega.jp/players/information/211230_2.html
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/190813_1.html
        sources.insert(131, vec![SundayBest, Swimsuit]);
        // 夕雲, 夕雲改:
        // * https://kancolle-a.sega.jp/players/information/2310_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(133, vec![PacificSaury, OriginalIllustration1(true)]);
        // 巻雲, 巻雲改: (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(134, vec![OriginalIllustration1(true)]);
        // 長波, 長波改: (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(135, vec![OriginalIllustration1(true)]);
        // 大和改:
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/190813_1.html
        // * (SundayBest) https://kancolle-a.sega.jp/players/information/211230_2.html
        sources.insert(136, vec![SundayBest, Swimsuit]);
        // 衣笠改二: https://kancolle-a.sega.jp/players/information/210205_1.html
        sources.insert(142, vec![Valentine]);
        // 夕立改二:
        // * https://kancolle-a.sega.jp/players/information/201027_1.html
        // * https://kancolle-a.sega.jp/players/information/2205_rainy_season.html
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/180316_1.html
        // ** https://kancolle-a.sega.jp/players/information/171124_3.html
        sources.insert(
            144,
            vec![RainySeason, Halloween, OriginalIllustration2(false, false)],
        );
        // 時雨改二:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html (スリガオ海峡突入mode)
        // * https://kancolle-a.sega.jp/players/information/190805_1.html
        // * https://kancolle-a.sega.jp/players/information/211005_1.html
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/210409_1.html
        // ** https://kancolle-a.sega.jp/players/information/180316_1.html
        sources.insert(
            145,
            vec![
                DecisiveBattle,
                Swimsuit,
                PacificSaury,
                OriginalIllustration2(false, false),
            ],
        );
        // 榛名改二: https://kancolle-a.sega.jp/players/information/190722_1.html
        sources.insert(151, vec![Swimsuit]);
        // 卯月, 卯月改:
        // * https://kancolle-a.sega.jp/players/information/210205_1.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190425_2.html
        sources.insert(165, vec![Valentine, OriginalIllustration1(true)]);
        // 磯風, 磯風改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(167, vec![PacificSaury]);
        // 浦風, 浦風改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(168, vec![Yukata]);
        // 浜風, 浜風改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(170, vec![Yukata]);
        // 天津風, 天津風改: (改) https://kancolle-a.sega.jp/players/information/200901_2.html
        sources.insert(181, vec![OriginalIllustration1(true)]);
        // 大淀, 大淀改:
        // * https://kancolle-a.sega.jp/players/information/210907_1.html
        // * (OriginalIllustration) https://kancolle-a.sega.jp/players/information/190508_1.html
        sources.insert(183, vec![Swimsuit, OriginalIllustration1(false)]);
        // 大鯨: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(184, vec![PacificSaury]);
        // 龍鳳: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(185, vec![PacificSaury]);
        // 明石改: https://kancolle-a.sega.jp/players/information/190914_1.html
        sources.insert(187, vec![OriginalIllustration1(false)]);
        // 春雨, 春雨改:
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/180316_1.html
        // ** (改) https://kancolle-a.sega.jp/players/information/200623_1.html
        sources.insert(205, vec![OriginalIllustration2(false, true)]);
        // 潮改二:
        // * https://kancolle-a.sega.jp/players/information/2201_valentine.html
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(207, vec![Valentine, PacificSaury]);
        // 早霜, 早霜改:
        // * https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(209, vec![RainySeason, OriginalIllustration1(true)]);
        // 清霜, 清霜改:
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(210, vec![RainySeason, OriginalIllustration1(true)]);
        // 扶桑改二: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(211, vec![RainySeason]);
        // 朝雲, 朝雲改: https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html (スリガオ海峡突入mode)
        sources.insert(213, vec![DecisiveBattle]);
        // 山雲, 山雲改: https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html (スリガオ海峡突入mode)
        sources.insert(214, vec![DecisiveBattle]);
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
        // 呂500: https://kancolle-a.sega.jp/players/information/210914_1.html
        sources.insert(236, vec![Swimsuit]);
        // Warspite, Warspite改:
        // * https://kancolle-a.sega.jp/players/information/2212_haregimode.html
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/190914_1.html
        // ** (改) https://kancolle-a.sega.jp/players/information/200623_1.html
        sources.insert(239, vec![SundayBest, OriginalIllustration2(false, true)]);
        // Littorio: https://kancolle-a.sega.jp/players/information/200728_1.html
        sources.insert(241, vec![Swimsuit]);
        // Roma, Roma改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(242, vec![Halloween]);
        // Libeccio, Libeccio改: https://kancolle-a.sega.jp/players/information/190729_1.html
        sources.insert(243, vec![Swimsuit]);
        // 秋津洲: https://kancolle-a.sega.jp/players/information/2207_join_akitusima_swim.html
        sources.insert(245, vec![Swimsuit]);
        // Italia: https://kancolle-a.sega.jp/players/information/200728_1.html
        sources.insert(246, vec![Swimsuit]);
        // Zara, Zara改: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(248, vec![PacificSaury]);
        // 秋津洲改: https://kancolle-a.sega.jp/players/information/2207_join_akitusima_swim.html
        sources.insert(250, vec![Swimsuit]);
        // 風雲, 風雲改: https://kancolle-a.sega.jp/players/information/2208_join_kazagumo_swim.html
        sources.insert(253, vec![Swimsuit]);
        // 山風, 山風改: https://kancolle-a.sega.jp/players/information/210827_2.html
        sources.insert(257, vec![Swimsuit]);
        // 速吸, 速吸改: https://kancolle-a.sega.jp/players/information/2312_xmas.html
        sources.insert(260, vec![Christmas]);
        // 翔鶴改二:
        // * https://kancolle-a.sega.jp/players/information/211214_1.html
        // * (OriginalIllustration) https://kancolle-a.sega.jp/players/information/210805_1.html
        sources.insert(261, vec![Christmas, OriginalIllustration1(false)]);
        // 瑞鶴改二: https://kancolle-a.sega.jp/players/information/211214_1.html
        sources.insert(262, vec![Christmas]);
        // 朝潮改二: https://kancolle-a.sega.jp/players/information/201013_1.html
        sources.insert(263, vec![Halloween]);
        // 霞改二: https://kancolle-a.sega.jp/players/information/200805_1.html
        sources.insert(264, vec![Swimsuit]);
        // 鹿島, 鹿島改: https://kancolle-a.sega.jp/players/information/201201_1.html
        sources.insert(265, vec![Christmas]);
        // 翔鶴改二甲: https://kancolle-a.sega.jp/players/information/211214_1.html
        sources.insert(266, vec![Christmas]);
        // 瑞鶴改二甲:
        // * (Christmas) https://kancolle-a.sega.jp/players/information/211214_1.html
        // * (DecisiveBattle) https://kancolle-a.sega.jp/players/information/2409_join_zuikaku_kai_2_engano.html (エンガノ岬決戦mode)
        sources.insert(267, vec![DecisiveBattle, Christmas]);
        // 朝潮改二丁: https://kancolle-a.sega.jp/players/information/201013_1.html
        sources.insert(268, vec![Halloween]);
        // 霞改二乙: https://kancolle-a.sega.jp/players/information/200805_1.html
        sources.insert(270, vec![Swimsuit]);
        // 神風: https://kancolle-a.sega.jp/players/information/2402_valentine.html
        sources.insert(271, vec![Valentine]);
        // 神風改: https://kancolle-a.sega.jp/players/information/2402_valentine.html
        sources.insert(276, vec![Valentine]);
        // 満潮改二:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html (スリガオ海峡突入mode)
        // * https://kancolle-a.sega.jp/players/information/2310_sauryfestival.html
        sources.insert(289, vec![DecisiveBattle, PacificSaury]);
        // Richelieu, Richelieu改: https://kancolle-a.sega.jp/players/information/2312_haregimode.html
        sources.insert(292, vec![SundayBest]);
        // 鈴谷改二: https://kancolle-a.sega.jp/players/information/201201_1.html
        sources.insert(303, vec![Christmas]);
        // 熊野改二: https://kancolle-a.sega.jp/players/information/201201_1.html
        sources.insert(304, vec![Christmas]);
        // Johnston, Johnston改: https://kancolle-a.sega.jp/players/information/2207_join_Johnston_swim.html
        sources.insert(362, vec![Swimsuit]);
        // Gotland: https://kancolle-a.sega.jp/players/information/2307_join_gotland_swim.html
        sources.insert(374, vec![Swimsuit]);
        // 金剛改二丙: https://kancolle-a.sega.jp/players/information/201225_1.html
        sources.insert(391, vec![OriginalIllustration1(false)]);
        // Fletcher, Fletcher改: https://kancolle-a.sega.jp/players/information/2407_join_fletcher_swim.html
        sources.insert(396, vec![Swimsuit]);
        // Grecale: https://kancolle-a.sega.jp/players/information/2310_halloween.html
        sources.insert(414, vec![Halloween]);
        // 時雨改三: https://kancolle-a.sega.jp/players/information/2408_join_shigure_swim_start.html
        sources.insert(561, vec![Swimsuit]);

        // These cannot be determined yet, as there's multiple variations I don't
        // have in my current data

        // 由良改二:
        // * (Yukata) https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/2307_join_yura-kai2_swim.html
        sources.insert(288, vec![Unknown, Unknown]);

        // Info store for ships I don't have at all.
        // Grecale改: https://kancolle-a.sega.jp/players/information/2310_halloween.html
        // Gotland改: https://kancolle-a.sega.jp/players/information/2307_join_gotland_swim.html

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
        if let Some(sources) = BOOK_SHIP_SOURCES.get().unwrap().get(&self.book_no) {
            if sources.len() + 1 != self.card_list.len() {
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
                .unwrap_or(&Unknown)
                .to_owned()
        } else {
            Unknown
        }
    }

    /// Split ourselves into a non-kai and optional kai BookShips.
    pub(crate) fn into_kai_split(mut self) -> (BookShip, Option<BookShip>) {
        if self.card_list.is_empty() || self.card_list[0].variation_num_in_page == 3 {
            return (self, None);
        }

        let mut kai = self.clone();
        kai.ship_name += "改";

        let mut variation_count = 0u16;
        let mut owned_count = 0u16;
        let mut self_non_original_pages = 0;
        let mut self_normal_status_img: Option<Vec<String>> = None;
        for card_page in self.card_list.as_mut_slice() {
            use BookShipCardPageSource::*;
            // Cheating using kai because the borrow checker won't let us use self due to
            // the mut borrow to get our iterator.
            match kai.source(card_page.priority) {
                // 雪風 has no swimsuits, but 雪風改 does. And they share a book entry. >_<
                Swimsuit if self.ship_name == "雪風" => {
                    // Drop this page.
                    card_page.acquire_num_in_page = 0;
                    card_page.variation_num_in_page = 0;
                    // TODO: We aren't actually dropping this page, so for sanity, clear the array.
                    card_page.card_img_list.clear();
                    card_page.status_img = None;
                }
                OriginalIllustration1(false) | OriginalIllustration2(false, false) => {
                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(self_normal_status_img.is_some());
                    card_page.status_img = self_normal_status_img.clone();
                }
                OriginalIllustration1(true) | OriginalIllustration2(true, true) => {
                    // Drop this page.
                    card_page.acquire_num_in_page = 0;
                    card_page.variation_num_in_page = 0;
                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(self_normal_status_img.is_some());
                    card_page.status_img = self_normal_status_img.clone();
                    // TODO: We aren't actually dropping this page, so for sanity, clear the array.
                    card_page.card_img_list.clear();
                }
                OriginalIllustration2(false, true) => {
                    // Take the first one only
                    assert_eq!(card_page.variation_num_in_page, 2);
                    assert_eq!(card_page.card_img_list.len(), 2);
                    card_page.card_img_list.remove(1);
                    card_page.variation_num_in_page = 1;

                    // Fix counts
                    card_page.acquire_num_in_page = card_page
                        .card_img_list
                        .iter()
                        .filter(|s| !s.is_empty())
                        .count() as u16;

                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(self_normal_status_img.is_some());
                    card_page.status_img = self_normal_status_img.clone();

                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                }
                OriginalIllustration2(true, false) => {
                    // Take the second one only
                    assert_eq!(card_page.variation_num_in_page, 2);
                    assert_eq!(card_page.card_img_list.len(), 2);
                    card_page.card_img_list.remove(0);
                    card_page.variation_num_in_page = 1;

                    // Fix counts
                    card_page.acquire_num_in_page = card_page
                        .card_img_list
                        .iter()
                        .filter(|s| !s.is_empty())
                        .count() as u16;

                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(self_normal_status_img.is_some());
                    card_page.status_img = self_normal_status_img.clone();

                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                }
                source => {
                    // Split, take the first half
                    assert_eq!(card_page.variation_num_in_page, 6);
                    assert_eq!(card_page.card_img_list.len(), 6);
                    card_page.card_img_list.truncate(3);
                    card_page.variation_num_in_page = 3;

                    // Fix counts
                    card_page.acquire_num_in_page = card_page
                        .card_img_list
                        .iter()
                        .filter(|s| !s.is_empty())
                        .count() as u16;

                    // Fix status images
                    if let Some(status_image) = card_page.status_img.as_mut() {
                        if card_page.acquire_num_in_page > 0 {
                            status_image.truncate(1);
                        } else {
                            status_image.clear();
                        }
                    }

                    if matches!(source, Normal) {
                        // Store a copy of the normal status page for OriginalIllustrations to use
                        assert!(self_normal_status_img.is_none());
                        assert!(card_page.status_img.is_some());
                        self_normal_status_img = card_page.status_img.clone();
                    } else if card_page.status_img.as_ref().is_some_and(|s| s.is_empty()) {
                        // If we emptied the status image array, None it instead.
                        card_page.status_img = None;
                    }

                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;

                    self_non_original_pages += 1;
                }
            }
        }

        self.variation_num = variation_count;
        self.acquire_num = owned_count;

        // NOTE: We cannot remove the 0-variation pages from self yet, as we need source() to
        // return the correct value until we finish processing the kai data.
        // In fact, we may not be able to remove them at all, due to source() validating that.
        // TODO: What about a function to wrap walking the pages with their source?
        // We may need to remove source() from the public API since splitting the book like this
        // will break it.
        // IDEA: Push source() up into Ship, and massage things there? Still messy, but only
        // internally-messy.

        let mut variation_count = 0u16;
        let mut owned_count = 0u16;
        let mut kai_non_original_pages = 0;
        let mut kai_normal_status_img: Option<Vec<String>> = None;
        for card_page in kai.card_list.as_mut_slice() {
            use BookShipCardPageSource::*;
            // Cheating using self because the borrow checker won't let us use kai due to
            // the mut borrow to get our iterator.
            match self.source(card_page.priority) {
                // 雪風 has no swimsuits, but 雪風改 does. And they share a book entry. >_<
                Swimsuit if kai.ship_name == "雪風改" => {
                    // Keep this page.
                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                    kai_non_original_pages += 1;
                }
                OriginalIllustration1(true) | OriginalIllustration2(true, true) => {
                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(kai_normal_status_img.is_some());
                    card_page.status_img = kai_normal_status_img.clone();
                }
                OriginalIllustration1(false) | OriginalIllustration2(false, false) => {
                    // Drop this page.
                    card_page.acquire_num_in_page = 0;
                    card_page.variation_num_in_page = 0;
                    // TODO: We aren't actually dropping this page, so for sanity, clear the array.
                    card_page.card_img_list.clear();
                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(kai_normal_status_img.is_some());
                    card_page.status_img = kai_normal_status_img.clone();
                }
                OriginalIllustration2(true, false) => {
                    // Take the first one only
                    assert_eq!(card_page.variation_num_in_page, 2);
                    assert_eq!(card_page.card_img_list.len(), 2);
                    card_page.card_img_list.remove(1);
                    card_page.variation_num_in_page = 1;

                    // Fix counts
                    card_page.acquire_num_in_page = card_page
                        .card_img_list
                        .iter()
                        .filter(|s| !s.is_empty())
                        .count() as u16;

                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(kai_normal_status_img.is_some());
                    card_page.status_img = kai_normal_status_img.clone();

                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                }
                OriginalIllustration2(false, true) => {
                    // Take the second one only
                    assert_eq!(card_page.variation_num_in_page, 2);
                    assert_eq!(card_page.card_img_list.len(), 2);
                    card_page.card_img_list.remove(0);
                    card_page.variation_num_in_page = 1;

                    // Fix counts
                    card_page.acquire_num_in_page = card_page
                        .card_img_list
                        .iter()
                        .filter(|s| !s.is_empty())
                        .count() as u16;

                    // Original Illustrations should have the same status icons as the Normal page
                    assert!(kai_normal_status_img.is_some());
                    card_page.status_img = kai_normal_status_img.clone();

                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                }
                source => {
                    // Split, take the second half
                    assert_eq!(card_page.variation_num_in_page, 6);
                    assert_eq!(card_page.card_img_list.len(), 6);
                    drop(card_page.card_img_list.drain(0..3));
                    card_page.variation_num_in_page = 3;

                    // Fix counts
                    card_page.acquire_num_in_page = card_page
                        .card_img_list
                        .iter()
                        .filter(|s| !s.is_empty())
                        .count() as u16;

                    // Fix status images
                    if let Some(status_img) = card_page.status_img.as_mut() {
                        if card_page.acquire_num_in_page == 0 {
                            status_img.clear();
                        } else if status_img.len() == 2 {
                            status_img.remove(0);
                        }
                    }

                    if matches!(source, Normal) {
                        // Store a copy of the normal status page for OriginalIllustrations to use
                        assert!(kai_normal_status_img.is_none());
                        assert!(card_page.status_img.is_some());
                        kai_normal_status_img = card_page.status_img.clone();
                    } else if card_page.status_img.as_ref().is_some_and(|s| s.is_empty()) {
                        // If we emptied the status image array, None it instead.
                        card_page.status_img = None;
                    }

                    variation_count += card_page.variation_num_in_page;
                    owned_count += card_page.acquire_num_in_page;
                    kai_non_original_pages += 1;
                }
            }
        }

        kai.variation_num = variation_count;
        kai.acquire_num = owned_count;

        // Fixup isMarried and marriedImg
        if let Some(married_vec) = self.is_married.as_ref() {
            assert!(self.married_img.is_some());
            let self_married = if self.acquire_num > 0 {
                *married_vec.first().unwrap()
            } else {
                false
            };
            let kai_married = if kai.acquire_num > 0 {
                *married_vec.last().unwrap()
            } else {
                false
            };
            self.is_married = Some([self_married].repeat(self_non_original_pages));
            kai.is_married = Some([kai_married].repeat(kai_non_original_pages));
            if !self_married {
                self.married_img.as_mut().unwrap().clear();
            } else if self.married_img.as_ref().unwrap().len() == 2 {
                self.married_img.as_mut().unwrap().remove(0);
            }

            if !kai_married {
                kai.married_img.as_mut().unwrap().clear();
            } else if kai.married_img.as_ref().unwrap().len() == 2 {
                kai.married_img.as_mut().unwrap().remove(1);
            }
        }

        // TODO: Shrink-to-fit across all the arrays we messed with?
        // card_page.card_img_list, card_page.status_img, married_img?
        // If we were going to do that, we could have just created new Vectors anyway...
        // At this point, I wonder if rather than clone-and-edit, we could have just created two
        // new BookShips, populated them, and dropped self in the end.

        (self, Some(kai))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BookShipCardPage {
    pub priority: u16,
    pub card_img_list: Vec<String>,
    pub status_img: Option<Vec<String>>,
    pub variation_num_in_page: u16,
    pub acquire_num_in_page: u16,
}

#[cfg(test)]
mod tests;
