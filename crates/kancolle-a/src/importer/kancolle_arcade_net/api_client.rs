//! Module for HTTPS client for https://kancolle-arcade.net/ac/api/ and
//! https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json

// TODO WASI Support (Which means non-blocking, and maybe no cookies?)
use reqwest::{blocking::Client as ReqwestClient, Url};
use reqwest::{cookie::Jar, header::HeaderMap};
use std::{error::Error, io::Read, sync::Arc};

const API_BASE: &str = "https://kancolle-arcade.net/ac/api/";

pub struct ClientBuilder {
    jsessionid: Option<String>,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder { jsessionid: None }
    }

    pub fn build(&self) -> Result<Client, Box<dyn Error>> {
        let mut headers = HeaderMap::default();
        headers.insert("X-Requested-With", "XMLHttpRequest".parse()?);

        let cookies = Jar::default();
        if let Some(ref jsessionid) = self.jsessionid {
            cookies.add_cookie_str(
                &format!("JSESSIONID={}; Path=/; HttpOnly", jsessionid),
                &API_BASE.parse::<Url>()?,
            )
        };

        Ok(Client {
            client: ReqwestClient::builder()
                .cookie_provider(Arc::new(cookies))
                .default_headers(headers)
                .build()?,
        })
    }

    pub fn jsessionid(mut self, jsessionid: String) -> ClientBuilder {
        self.jsessionid = Some(jsessionid);
        self
    }
}

pub enum ApiEndpoint {
    // Global data: no authentication needed, unaffected by auth status
    KanmusuList,
    EventHold,
    EventInfo,
    PlaceDistricts,
    // PlaceExclude,
    PlacePlaces,
    // PlacePlacesFromHere, // Takes query parameters
    // PlaceVerified,
    RankingMonthlyCurrent,
    RankingMonthlyPrev,
    RankingTotal,
    // TcErrorInfo,
    // TcErrorRequest, // POST
    // TcErrorReceive, // POST
    TcErrorDispFlag,

    // Per-user data: Authentication needed, contains personal info
    AimeCampaignHold,
    AimeCampaignInfo,
    AreaCaptureInfo,
    BlueprintListInfo,
    CampaignHistory,
    CampaignInfo,
    CampaignPresent,
    CharacterListInfo,
    CopCheckreward,
    // CopInfo,
    // CopHold,
    EpFesHold,
    EpFesProgress,
    EquipBookInfo,
    EquipListInfo,
    ExerciseInfo,
    // NCampExchange, // POST
    // NCampHistory,
    NCampInfo,
    // NCampJoin,
    // NCampPlace, // Takes query parameters
    PersonalBasicInfo,
    QuestInfo,
    RoomItemListInfo,
    TcBookInfo,

    // TODO: Auth stuff, will need special handling. Maybe not in this enum?
    // AuthAutoLogin, // POST
    // AuthLogin, // POST
    // AuthTokenDelete, // POST
    // AuthLoginState,
    // AuthLogout,
    // AuthAccessCode, // POST
    // SegaIdRegistration, // POST
    // AimeCardRegistration, // POST

    // User-specified
    Other(String),
}

fn url_for_endpoint(endpoint: &ApiEndpoint) -> String {
    // TODO: When these have some value... Currenty just empty JSON arrays.
    // * https://kancolle-arcade.net/ac/resources/place/exclude.json
    // * https://kancolle-arcade.net/ac/resources/place/verified.json
    use ApiEndpoint::*;
    match endpoint {
        KanmusuList => {
            "https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json".to_string()
        }

        AimeCampaignHold => format!("{API_BASE}AimeCampaign/hold"),
        AimeCampaignInfo => format!("{API_BASE}AimeCampaign/info"),
        AreaCaptureInfo => format!("{API_BASE}Area/captureInfo"),
        BlueprintListInfo => format!("{API_BASE}BlueprintList/info"),
        CampaignHistory => format!("{API_BASE}Campaign/history"),
        CampaignInfo => format!("{API_BASE}Campaign/info"),
        CampaignPresent => format!("{API_BASE}Campaign/present"),
        CharacterListInfo => format!("{API_BASE}CharacterList/info"),
        CopCheckreward => format!("{API_BASE}Cop/checkreward"),
        EpFesHold => format!("{API_BASE}EpFes/hold"),
        EpFesProgress => format!("{API_BASE}EpFes/progress"),
        EquipBookInfo => format!("{API_BASE}EquipBook/info"),
        EquipListInfo => format!("{API_BASE}EquipList/info"),
        EventHold => format!("{API_BASE}Event/hold"),
        EventInfo => format!("{API_BASE}Event/info"),
        ExerciseInfo => format!("{API_BASE}Exercise/info"),
        NCampInfo => format!("{API_BASE}NCamp/info"),
        PersonalBasicInfo => format!("{API_BASE}Personal/basicInfo"),
        PlaceDistricts => format!("{API_BASE}Place/districts"),
        PlacePlaces => format!("{API_BASE}Place/places"),
        QuestInfo => format!("{API_BASE}Quest/info"),
        RankingMonthlyCurrent => format!("{API_BASE}Ranking/monthly/current"),
        RankingMonthlyPrev => format!("{API_BASE}Ranking/monthly/prev"),
        RankingTotal => format!("{API_BASE}Ranking/total"),
        RoomItemListInfo => format!("{API_BASE}RoomItemList/info"),
        TcBookInfo => format!("{API_BASE}TcBook/info"),
        TcErrorDispFlag => format!("{API_BASE}TcError/dispFlag"),

        Other(raw_path) => format!("{API_BASE}/{raw_path}"),
    }
}

#[derive(Default)]
pub struct Client {
    client: ReqwestClient,
}

impl Client {
    pub fn fetch(&self, endpoint: &ApiEndpoint) -> Result<Box<dyn Read>, Box<dyn Error>> {
        Ok(Box::new(
            self.client
                .get(url_for_endpoint(endpoint))
                .send()?
                .error_for_status()?,
        ))
    }
}
