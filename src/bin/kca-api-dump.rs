use kancolle_a::importer::kancolle_arcade_net::{ApiEndpoint, ClientBuilder};
use std::io::Read;
use std::{error::Error, fs};

pub(crate) mod args {
    use bpaf::*;

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) jsessionid: String,
        // TODO: Output path?
    }

    pub fn options() -> OptionParser<Options> {
        let jsessionid = long("jsessionid").help("The JSESSIONID cookie from a logged-in session at https://kancolle-arcade.net/ac/api").argument::<String>("JSESSIONID");
        construct!(Options { jsessionid })
            .to_options()
            .descr("A tool to fetch all supported data from https://kancolle-arcade.net/ac/")
    }

    #[test]
    fn kca_api_dump_check_options() {
        options().check_invariants(false)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();

    let client = ClientBuilder::new().jsessionid(args.jsessionid).build()?;

    let mut data = String::new();
    client
        .fetch(ApiEndpoint::KanmusuList)?
        .read_to_string(&mut data)?;
    fs::write("tests/fixtures/latest/kanmusu_list.json", data)?;

    let mut data = String::new();
    client
        .fetch(ApiEndpoint::PlaceDistricts)?
        .read_to_string(&mut data)?;
    fs::write("tests/fixtures/latest/Place_districts.json", data)?;

    let mut data = String::new();
    client
        .fetch(ApiEndpoint::PlacePlaces)?
        .read_to_string(&mut data)?;
    fs::write("tests/fixtures/latest/Place_places.json", data)?;

    // Auth is required for the below

    let mut data = String::new();
    client
        .fetch(ApiEndpoint::TcBookInfo)?
        .read_to_string(&mut data)?;
    fs::write("tests/fixtures/latest/TcBook_info.json", data)?;

    let mut data = String::new();
    client
        .fetch(ApiEndpoint::BlueprintListInfo)?
        .read_to_string(&mut data)?;
    fs::write("tests/fixtures/latest/BlueprintList_info.json", data)?;

    Ok(())
}
