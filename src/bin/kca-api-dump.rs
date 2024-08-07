use jsonxf::Formatter;
use kancolle_a::importer::kancolle_arcade_net::{ApiEndpoint, Client, ClientBuilder};
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

fn fetch_to_fixture(
    client: &Client,
    formatter: &mut Formatter,
    endpoint: &ApiEndpoint,
) -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    let filename = fixture_filename(&endpoint);
    client.fetch(&endpoint)?.read_to_string(&mut data)?;
    let data = formatter.format(&data)?;
    fs::write(
        format!("tests/fixtures/latest/{filename}"),
        // Not sure why there's a leading newline here. jsonxf docs don't show it.
        data.trim_start(),
    )?;
    Ok(())
}

fn fixture_filename(endpoint: &ApiEndpoint) -> String {
    use ApiEndpoint::*;
    match endpoint {
        KanmusuList => "kanmusu_list.json".to_string(),

        AimeCampaignHold => "AimeCampaign_hold.json".to_string(),
        AimeCampaignInfo => "AimeCampaign_info.json".to_string(),
        AreaCaptureInfo => "Area_captureInfo.json".to_string(),
        BlueprintListInfo => "BlueprintList_info.json".to_string(),
        CampaignHistory => "Capmpaign_history.json".to_string(),
        CampaignInfo => "Campaign_info.json".to_string(),
        CampaignPresent => "Campaign_present.json".to_string(),
        CharacterListInfo => "CharacterList_info.json".to_string(),
        CopCheckreward => "Cop_checkreward.json".to_string(),
        EpFesHold => "EpFes_hold.json".to_string(),
        EpFesProgress => "EpFes_progress.json".to_string(),
        EquipBookInfo => "EquipBook_info.json".to_string(),
        EquipListInfo => "EquipList_info.json".to_string(),
        EventHold => "Event_hold.json".to_string(),
        EventInfo => "Event_info.json".to_string(),
        ExerciseInfo => "Exercise_info.json".to_string(),
        NCampInfo => "NCamp_info.json".to_string(),
        PersonalBasicInfo => "Personal_basicInfo.json".to_string(),
        PlaceDistricts => "Place_districts.json".to_string(),
        PlacePlaces => "Place_places.json".to_string(),
        QuestInfo => "Quest_info.json".to_string(),
        RankingMonthlyCurrent => "Ranking_monthly_current.json".to_string(),
        RankingMonthlyPrev => "Ranking_monthly_prev.json".to_string(),
        RankingTotal => "Ranking_total.json".to_string(),
        RoomItemListInfo => "RoomItemList_info.json".to_string(),
        TcBookInfo => "TcBook_info.json".to_string(),
        TcErrorDispFlag => "TcError_dispFlag.json".to_string(),

        // TODO: Turn slash into underscore, and drop prefix
        Other(path) => format!("other_{path}.json"),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();

    let client = ClientBuilder::new().jsessionid(args.jsessionid).build()?;
    let mut formatter = Formatter::pretty_printer();
    formatter.indent = "    ".to_string();
    formatter.trailing_output = "\n".to_string();

    // Auth not required for these

    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::KanmusuList)?;

    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EventHold)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EventInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::PlaceDistricts)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::PlacePlaces)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RankingMonthlyCurrent)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RankingMonthlyPrev)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RankingTotal)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::TcErrorDispFlag)?;

    // Auth is required for the below

    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::AimeCampaignHold)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::AimeCampaignInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::AreaCaptureInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::BlueprintListInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CampaignHistory)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CampaignInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CampaignPresent)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CharacterListInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CopCheckreward)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EpFesHold)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EpFesProgress)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EquipBookInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EquipListInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::ExerciseInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::NCampInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::PersonalBasicInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::QuestInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RoomItemListInfo)?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::TcBookInfo)?;

    Ok(())
}
