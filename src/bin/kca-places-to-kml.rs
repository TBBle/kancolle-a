use kancolle_a::importer::kancolle_arcade_net::{self, Place};
use kml::{
    types::{Coord, Element, Geometry, Placemark, Point},
    Kml, KmlDocument, KmlVersion, KmlWriter,
};
use std::fs::File;
use std::{collections::HashMap, io::BufReader};
use std::{error::Error, io};

fn place_to_kml(place: &Place) -> Kml<f64> {
    let coord = Coord::<f64> {
        x: place.longitude().parse::<f64>().unwrap(),
        y: place.latitude().parse::<f64>().unwrap(),
        ..Default::default()
    };
    let geometry = Geometry::Point(Point::<f64> {
        coord,
        ..Default::default()
    });
    // TODO: Generate a nice description. We can apparently use HTML here.
    let placemark = Placemark::<f64> {
        name: Some(place.name().clone()),
        geometry: Some(geometry),
        attrs: HashMap::from([(
            // This needs to be NCName per XML Schema, which can't start with a number.
            "id".to_string(),
            "Place_".to_string() + &place.id().to_string(),
        )]),
        // Google My Maps appears to ignore these attributes. Oh well.
        // (Tested that removing the geometry does try to use the address, so the format is correct.)
        children: vec![
            Element {
                name: "address".to_string(),
                content: Some(place.address().clone()),
                ..Default::default()
            },
            Element {
                name: "phoneNumber".to_string(),
                content: Some(place.tel().clone()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    Kml::Placemark(placemark)
}

pub(crate) mod args {
    use std::path::PathBuf;

    use bpaf::*;
    use kancolle_a::cli_helpers;

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) places: PathBuf,
    }

    pub fn options() -> OptionParser<Options> {
        let places = cli_helpers::places_path_parser();
        construct!(Options { places })
            .to_options()
            .descr("A tool to convert the Kancolle Arcade locations list into KML.")
    }

    #[test]
    fn kca_places_to_kml_check_options() {
        options().check_invariants(false)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();
    let places_path = args.places;

    let places_data = BufReader::new(File::open(places_path)?);

    let places = kancolle_arcade_net::read_place_places(places_data)?;

    // Useful structure references:
    // * https://support.vgis.io/hc/en-us/articles/360035415013-KML-File-Structure-Requirements-KB-GI006
    // * https://developers.google.com/kml/documentation/kmlreference?hl=en#feature

    let kml = Kml::KmlDocument(KmlDocument::<f64> {
        version: KmlVersion::V22,
        elements: vec![Kml::Folder {
            elements: places.iter().map(place_to_kml).collect(),
            attrs: HashMap::from([("id".to_string(), "ActivePlaces".to_string())]),
        }],
        attrs: HashMap::from([
            (
                // TODO: Feature request for kml library to do this automatically given the KmlVersion is set?
                "xmlns".to_string(),
                "http://www.opengis.net/kml/2.2".to_string(),
            ),
            (
                "xmlns:xsi".to_string(),
                "http://www.w3.org/2001/XMLSchema-instance".to_string(),
            ),
            (
                "xsi:schemaLocation".to_string(),
                "http://www.opengis.net/kml/2.2 https://schemas.opengis.net/kml/2.2.0/ogckml22.xsd"
                    .to_string(),
            ),
        ]),
        ..Default::default()
    });

    let mut writer = KmlWriter::from_writer(io::stdout());
    writer.write(&kml).unwrap();

    Ok(())
}
