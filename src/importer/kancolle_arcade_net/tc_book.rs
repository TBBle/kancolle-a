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

#[derive(Debug, Deserialize, Clone)]
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
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/2206_seaarea_event12_detail.html
        sources.insert(7, vec![SundayBest, OriginalIllustration(1)]);
        // 島風, 島風改:
        // * (DecisiveBattle) https://kancolle-a.sega.jp/players/information/190508_1.html
        // * (OriginalIllustration)
        // ** (改) https://kancolle-a.sega.jp/players/information/200901_2.html
        // ** (改) https://kancolle-a.sega.jp/players/information/210409_1.html
        sources.insert(10, vec![DecisiveBattle, OriginalIllustration(2)]);
        // 敷波, 敷波改: (改) https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(18, vec![OriginalIllustration(1)]);
        // 大井: https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(19, vec![RainySeason]);
        // 鳳翔, 鳳翔改: (改) https://kancolle-a.sega.jp/players/information/2303_seaarea_event13_detail.html
        sources.insert(25, vec![OriginalIllustration(1)]);
        // 扶桑, 扶桑改: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(26, vec![RainySeason]);
        // 球磨, 球磨改: https://kancolle-a.sega.jp/players/information/2212_xmas.html
        sources.insert(39, vec![Christmas]);
        // 多摩, 多摩改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(40, vec![PacificSaury]);
        // 由良, 由良改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(45, vec![Yukata]);
        // 川内, 川内改: (改) https://kancolle-a.sega.jp/players/information/2302_card_tsuika.html
        sources.insert(46, vec![OriginalIllustration(1)]);
        // 那珂, 那珂改: (改) https://kancolle-a.sega.jp/players/information/190914_1.html
        sources.insert(48, vec![OriginalIllustration(1)]);
        // 最上:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        sources.insert(51, vec![SurigaoStrait, RainySeason]);
        // 加古, 加古改: https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        sources.insert(53, vec![RainySeason]);
        // 朧, 朧改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/2404_spring_seaarea_detail.html
        sources.insert(67, vec![Fishing, OriginalIllustration(1)]);
        // 曙, 曙改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * https://kancolle-a.sega.jp/players/information/211005_1.html
        // * (OriginalIllustration)
        // ** (改) https://kancolle-a.sega.jp/players/information/211027_1.html
        // ** (改) https://kancolle-a.sega.jp/players/information/210409_1.html
        sources.insert(68, vec![PacificSaury, Fishing, OriginalIllustration(2)]);
        // 漣, 漣改:
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/2404_spring_seaarea_detail.html
        sources.insert(69, vec![PacificSaury, OriginalIllustration(1)]);
        // 潮, 潮改:
        // * https://kancolle-a.sega.jp/players/information/2201_valentine.html
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/211027_1.html
        sources.insert(70, vec![Valentine, PacificSaury, OriginalIllustration(1)]);
        // 暁, 暁改: (改) https://kancolle-a.sega.jp/players/information/210409_1.html
        sources.insert(71, vec![OriginalIllustration(1)]);
        // 雷, 雷改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(73, vec![Yukata]);
        // 電, 電改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(74, vec![Yukata]);
        // 白露, 白露改: (改) https://kancolle-a.sega.jp/players/information/180316_1.html
        sources.insert(79, vec![OriginalIllustration(1)]);
        // 村雨:
        // * (OriginalIllustration)
        // ** (改) https://kancolle-a.sega.jp/players/information/2206_seaarea_event12_detail.html
        // ** (改) https://kancolle-a.sega.jp/players/information/180316_1.html
        sources.insert(81, vec![OriginalIllustration(2)]);
        // 夕立, 夕立改:
        // * https://kancolle-a.sega.jp/players/information/201026_1.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190425_2.html
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
        // 伊勢改: https://kancolle-a.sega.jp/players/information/170420_1.html
        sources.insert(102, vec![OriginalIllustration(1)]);
        // 日向改: https://kancolle-a.sega.jp/players/information/170420_1.html
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
        // 大和:
        // * (SundayBest) https://kancolle-a.sega.jp/players/information/211230_2.html
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/190813_1.html
        sources.insert(131, vec![SundayBest, Swimsuit]);
        // 夕雲, 夕雲改:
        // * https://kancolle-a.sega.jp/players/information/2310_sauryfestival.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(133, vec![PacificSaury, OriginalIllustration(1)]);
        // 巻雲, 巻雲改: (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(134, vec![OriginalIllustration(1)]);
        // 長波, 長波改: (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(135, vec![OriginalIllustration(1)]);
        // 衣笠改二: https://kancolle-a.sega.jp/players/information/210205_1.html
        sources.insert(142, vec![Valentine]);
        // 夕立改二:
        // * https://kancolle-a.sega.jp/players/information/201027_1.html
        // * https://kancolle-a.sega.jp/players/information/2205_rainy_season.html
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/180316_1.html
        // ** https://kancolle-a.sega.jp/players/information/171124_3.html
        sources.insert(144, vec![RainySeason, Halloween, OriginalIllustration(2)]);
        // 時雨改二:
        // * https://kancolle-a.sega.jp/players/information/2401_seaarea_event14_detail_report.html
        // * https://kancolle-a.sega.jp/players/information/190805_1.html
        // * https://kancolle-a.sega.jp/players/information/211005_1.html
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/210409_1.html
        // ** https://kancolle-a.sega.jp/players/information/180316_1.html
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
        // 卯月, 卯月改:
        // * https://kancolle-a.sega.jp/players/information/210205_1.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190425_2.html
        sources.insert(165, vec![Valentine, OriginalIllustration(1)]);
        // 磯風, 磯風改: https://kancolle-a.sega.jp/players/information/211005_1.html
        sources.insert(167, vec![PacificSaury]);
        // 浦風, 浦風改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(168, vec![Yukata]);
        // 浜風, 浜風改: https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        sources.insert(170, vec![Yukata]);
        // 天津風, 天津風改: (改) https://kancolle-a.sega.jp/players/information/200901_2.html
        sources.insert(181, vec![OriginalIllustration(1)]);
        // 大淀, 大淀改:
        // * https://kancolle-a.sega.jp/players/information/210907_1.html
        // * (OriginalIllustration) https://kancolle-a.sega.jp/players/information/190508_1.html
        sources.insert(183, vec![Swimsuit, OriginalIllustration(1)]);
        // 大鯨: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(184, vec![PacificSaury]);
        // 龍鳳: https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(185, vec![PacificSaury]);
        // 春雨, 春雨改:
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/180316_1.html
        // ** (改) https://kancolle-a.sega.jp/players/information/200623_1.html
        sources.insert(205, vec![OriginalIllustration(2)]);
        // 潮改二:
        // * https://kancolle-a.sega.jp/players/information/2201_valentine.html
        // * https://kancolle-a.sega.jp/players/information/2209_sauryfestival.html
        sources.insert(207, vec![Valentine, PacificSaury]);
        // 早霜, 早霜改:
        // * https://kancolle-a.sega.jp/players/information/2406_rainy_season.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190307_2.html
        sources.insert(209, vec![RainySeason, OriginalIllustration(1)]);
        // 清霜, 清霜改:
        // * https://kancolle-a.sega.jp/players/information/2306_rainy_season.html
        // * (OriginalIllustration) (改) https://kancolle-a.sega.jp/players/information/190307_2.html
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
        // * (OriginalIllustration)
        // ** https://kancolle-a.sega.jp/players/information/190914_1.html
        // ** (改) https://kancolle-a.sega.jp/players/information/200623_1.html
        sources.insert(239, vec![SundayBest, OriginalIllustration(2)]);
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
        // 瑞鶴改二: https://kancolle-a.sega.jp/players/information/211214_1.html
        sources.insert(262, vec![Christmas]);
        // 朝潮改二: https://kancolle-a.sega.jp/players/information/201013_1.html
        sources.insert(263, vec![Halloween]);
        // 霞改二: https://kancolle-a.sega.jp/players/information/200805_1.html
        sources.insert(264, vec![Swimsuit]);
        // 鹿島, 鹿島改: https://kancolle-a.sega.jp/players/information/201201_1.html
        sources.insert(265, vec![Christmas]);
        // 朝潮改二丁: https://kancolle-a.sega.jp/players/information/201013_1.html
        sources.insert(268, vec![Halloween]);
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
        // 鈴谷改二: https://kancolle-a.sega.jp/players/information/201201_1.html
        sources.insert(303, vec![Christmas]);
        // Johnston, Johnston改: https://kancolle-a.sega.jp/players/information/2207_join_Johnston_swim.html
        sources.insert(362, vec![Swimsuit]);
        // Gotland: https://kancolle-a.sega.jp/players/information/2307_join_gotland_swim.html
        sources.insert(374, vec![Swimsuit]);
        // 金剛改二丙: https://kancolle-a.sega.jp/players/information/201225_1.html
        sources.insert(391, vec![OriginalIllustration(1)]);
        // Fletcher, Fletcher改: https://kancolle-a.sega.jp/players/information/2407_join_fletcher_swim.html
        sources.insert(396, vec![Swimsuit]);
        // 時雨改三: https://kancolle-a.sega.jp/players/information/2408_join_shigure_swim_start.html
        sources.insert(561, vec![Swimsuit]);

        // These cannot be determined yet, as there's multiple variations I don't
        // have in my current data

        // 大和改:
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/190813_1.html
        // * (SundayBest) https://kancolle-a.sega.jp/players/information/211230_2.html
        sources.insert(136, vec![Unknown, Unknown]);
        // 由良改二:
        // * (Yukata) https://kancolle-a.sega.jp/players/information/2309_yukata_season.html
        // * (Swimsuit) https://kancolle-a.sega.jp/players/information/2307_join_yura-kai2_swim.html
        sources.insert(288, vec![Unknown, Unknown]);

        // Info store for ships I don't have at all.
        // 呂500: https://kancolle-a.sega.jp/players/information/210914_1.html
        // Grecale, Grecale改: https://kancolle-a.sega.jp/players/information/2310_halloween.html
        // 瑞鶴改二甲: https://kancolle-a.sega.jp/players/information/211214_1.html
        // 翔鶴改二: https://kancolle-a.sega.jp/players/information/211214_1.html, https://kancolle-a.sega.jp/players/information/210805_1.html
        // 翔鶴改二甲: https://kancolle-a.sega.jp/players/information/211214_1.html
        // 熊野改二: https://kancolle-a.sega.jp/players/information/201201_1.html
        // Gotland改: https://kancolle-a.sega.jp/players/information/2307_join_gotland_swim.html
        // 明石改: https://kancolle-a.sega.jp/players/information/190914_1.html

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
                .or(Some(&Unknown))
                .unwrap()
                .to_owned()
        } else {
            Unknown
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
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
