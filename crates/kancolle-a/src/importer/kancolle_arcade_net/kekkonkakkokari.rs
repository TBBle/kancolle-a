/// Module for importer for https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json
pub mod kanmusu_list {
    use chrono::NaiveDate;
    use serde::Deserialize;
    use serde_json::Result;
    use std::io::Read;

    use lazy_static_include::*;

    lazy_static_include_bytes! {
        pub(crate) KANMUSU => "src/importer/kancolle_arcade_net/kekkonkakkokari/kanmusu_list.json",
    }

    // ケッコンカッコカリ, aka 結婚（仮）
    pub(crate) type KekkonKakkoKariList = Vec<KekkonKakkoKari>;

    /// Parses a PlacePlaces from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub(crate) fn read_kekkonkakkokarilist(reader: impl Read) -> Result<KekkonKakkoKariList> {
        let result: KekkonKakkoKariList = serde_json::from_reader(reader)?;
        Ok(result)
    }

    #[derive(Debug, Deserialize, Clone)]
    #[serde(deny_unknown_fields)]
    pub struct KekkonKakkoKari {
        pub id: u32,
        pub web_id: u32,
        pub name: String,
        pub name_reading: String,
        pub kind: String,
        pub category: String,
        #[serde(with = "kekkonkakkokari_date_format")]
        pub start_time: NaiveDate, // Technically 7am JST on this day, AFAIK.
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
}

#[cfg(test)]
mod tests;
