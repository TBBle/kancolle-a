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

fn fetch_to_fixture(client: &Client, endpoint: &ApiEndpoint) -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    let filename = fixture_filename(&endpoint);
    client.fetch(&endpoint)?.read_to_string(&mut data)?;
    fs::write(format!("tests/fixtures/latest/{filename}"), data)?;
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

    // Auth not required for these

    fetch_to_fixture(&client, &ApiEndpoint::KanmusuList)?;

    fetch_to_fixture(&client, &ApiEndpoint::EventHold)?;
    fetch_to_fixture(&client, &ApiEndpoint::EventInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::PlaceDistricts)?;
    fetch_to_fixture(&client, &ApiEndpoint::PlacePlaces)?;
    fetch_to_fixture(&client, &ApiEndpoint::RankingMonthlyCurrent)?;
    fetch_to_fixture(&client, &ApiEndpoint::RankingMonthlyPrev)?;
    fetch_to_fixture(&client, &ApiEndpoint::RankingTotal)?;
    fetch_to_fixture(&client, &ApiEndpoint::TcErrorDispFlag)?;

    // Auth is required for the below

    fetch_to_fixture(&client, &ApiEndpoint::AimeCampaignHold)?;
    fetch_to_fixture(&client, &ApiEndpoint::AimeCampaignInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::AreaCaptureInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::BlueprintListInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::CampaignHistory)?;
    fetch_to_fixture(&client, &ApiEndpoint::CampaignInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::CampaignPresent)?;
    fetch_to_fixture(&client, &ApiEndpoint::CharacterListInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::CopCheckreward)?;
    fetch_to_fixture(&client, &ApiEndpoint::EpFesHold)?;
    fetch_to_fixture(&client, &ApiEndpoint::EpFesProgress)?;
    fetch_to_fixture(&client, &ApiEndpoint::EquipBookInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::EquipListInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::ExerciseInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::NCampInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::PersonalBasicInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::QuestInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::RoomItemListInfo)?;
    fetch_to_fixture(&client, &ApiEndpoint::TcBookInfo)?;

    Ok(())
}
