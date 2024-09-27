use jsonxf::Formatter;
use kancolle_a::importer::kancolle_arcade_net::{ApiEndpoint, Client, ClientBuilder};
use std::{error::Error, fs};

pub(crate) mod args {
    use bpaf::*;

    #[derive(Debug, Clone)]
    pub(crate) struct Options {
        pub(crate) jsessionid: Option<String>,
        pub(crate) username: Option<String>,
        // TODO: Output path?
    }

    pub fn options() -> OptionParser<Options> {
        let jsessionid = long("jsessionid")
            .help("The JSESSIONID cookie from a logged-in session at https://kancolle-arcade.net/ac/api")
            .argument::<String>("JSESSIONID")
            .optional();
        let username = long("username")
            .help("The USERNAME to log into https://kancolle-arcade.net/ac/")
            .argument("USERNAME")
            .optional();
        construct!(Options {
            jsessionid,
            username
        })
        .to_options()
        .descr("A tool to fetch all supported data from https://kancolle-arcade.net/ac/")
    }

    #[test]
    fn kca_api_dump_check_options() {
        options().check_invariants(false)
    }
}

async fn fetch_to_fixture(
    client: &Client,
    formatter: &mut Formatter,
    endpoint: &ApiEndpoint,
) -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    let filename = fixture_filename(endpoint);
    client.fetch(endpoint).await?.read_to_string(&mut data)?;
    let data = formatter.format(&data)?;
    fs::write(
        format!("crates/kancolle-a/tests/fixtures/latest/{filename}"),
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

        AuthLogin => panic!("Unsupported data fixture"),

        // TODO: Turn slash into underscore, and drop prefix
        Other(path) => format!("other_{path}.json"),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::options().run();

    let mut client_builder = ClientBuilder::new();
    if let Some(jsessionid) = args.jsessionid {
        client_builder = client_builder.jsessionid(jsessionid);
    }
    if let Some(username) = args.username {
        let prompt = format!("Enter the password for {}:", username);
        let password = rpassword::prompt_password(prompt)?;
        client_builder = client_builder.userpass(username, password);
    }
    let client = client_builder.build()?;
    let mut formatter = Formatter::pretty_printer();
    formatter.indent = "    ".to_string();
    formatter.trailing_output = "\n".to_string();

    // Auth not required for these

    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::KanmusuList).await?;

    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EventHold).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EventInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::PlaceDistricts).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::PlacePlaces).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RankingMonthlyCurrent).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RankingMonthlyPrev).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RankingTotal).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::TcErrorDispFlag).await?;

    // Auth is required for the below

    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::AimeCampaignHold).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::AimeCampaignInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::AreaCaptureInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::BlueprintListInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CampaignHistory).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CampaignInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CampaignPresent).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CharacterListInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::CopCheckreward).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EpFesHold).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EpFesProgress).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EquipBookInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::EquipListInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::ExerciseInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::NCampInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::PersonalBasicInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::QuestInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::RoomItemListInfo).await?;
    fetch_to_fixture(&client, &mut formatter, &ApiEndpoint::TcBookInfo).await?;

    Ok(())
}
