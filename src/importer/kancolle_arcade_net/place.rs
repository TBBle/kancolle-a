/// Module for importer for https://kancolle-arcade.net/ac/api/Place/districts
pub mod districts {
    use derive_getters::Getters;
    use serde::Deserialize;
    use serde_json::Result;
    use std::io::Read;

    pub type PlaceDistricts = Vec<PlaceTopRegion>;

    /// Parses a PlaceDistricts from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn read_place_districts(reader: impl Read) -> Result<PlaceDistricts> {
        let result: PlaceDistricts = serde_json::from_reader(reader)?;
        Ok(result)
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
}

/// Module for importer for https://kancolle-arcade.net/ac/api/Place/places
pub mod places {
    use derive_getters::Getters;
    use serde::Deserialize;
    use serde_json::Result;
    use std::io::Read;

    pub type PlacePlaces = Vec<Place>;

    /// Parses a PlacePlaces from the provided JSON reader.
    /// Fails if not given a JSON array, or expected data structure does not match.
    pub fn read_place_places(reader: impl Read) -> Result<PlacePlaces> {
        let result: PlacePlaces = serde_json::from_reader(reader)?;
        Ok(result)
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
}

#[cfg(test)]
mod tests;
