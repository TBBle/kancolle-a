/// Module for importer for https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json
pub mod kanmusu_list {
    use chrono::NaiveDate;
    use derive_getters::Getters;
    use serde::Deserialize;
    use serde_json::Result;
    use std::{io::Read, ops::Deref};

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
}

#[cfg(test)]
mod tests;
