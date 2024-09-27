//! Module for HTTPS client for https://kancolle-arcade.net/ac/api/ and
//! https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json

#[cfg(not(target_family = "wasm"))]
use reqwest::{cookie::Jar, Url};
use reqwest::{
    header::{HeaderMap, USER_AGENT},
    StatusCode,
};
use reqwest::{Client as ReqwestClient, ClientBuilder as ReqwestBuilder};
use serde::{Deserialize, Serialize};
#[cfg(not(target_family = "wasm"))]
use std::sync::Arc;
use std::{collections::VecDeque, error::Error, io::Read};

const API_BASE: &str = "https://kancolle-arcade.net/ac/api/";

pub struct ClientBuilder {
    jsessionid: Option<String>,
    userpass: Option<(String, String)>,
}

#[cfg(not(target_family = "wasm"))]
fn setup_cookies(
    jsessionid: Option<String>,
    builder: ReqwestBuilder,
) -> Result<ReqwestBuilder, Box<dyn Error>> {
    Ok(if let Some(jsessionid) = jsessionid {
        let cookies = Jar::default();
        cookies.add_cookie_str(
            &format!("JSESSIONID={}; Path=/; HttpOnly", jsessionid),
            &API_BASE.parse::<Url>()?,
        );
        builder.cookie_provider(Arc::new(cookies))
    } else {
        builder.cookie_store(true)
    })
}

#[cfg(target_family = "wasm")]
fn setup_cookies(
    jsessionid: Option<String>,
    builder: ReqwestBuilder,
) -> Result<ReqwestBuilder, Box<dyn Error>> {
    // TODO: wasm-cookies-rs could be used in the browser
    assert!(jsessionid.is_none());
    Ok(builder)
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            jsessionid: None,
            userpass: None,
        }
    }

    pub fn build(self) -> Result<Client, Box<dyn Error>> {
        let mut reqwest_builder = ReqwestBuilder::new();

        let mut headers = HeaderMap::default();
        headers.insert("X-Requested-With", "XMLHttpRequest".parse()?);
        reqwest_builder = reqwest_builder.default_headers(headers);

        reqwest_builder = setup_cookies(self.jsessionid, reqwest_builder)?;

        Ok(Client {
            client: reqwest_builder.build()?,
            userpass: self.userpass,
        })
    }

    pub fn jsessionid(mut self, jsessionid: String) -> ClientBuilder {
        self.jsessionid = Some(jsessionid);
        self
    }

    pub fn userpass(mut self, username: String, password: String) -> ClientBuilder {
        self.userpass = Some((username, password));
        self
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
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
    AuthLogin, // POST
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
        AuthLogin => format!("{API_BASE}Auth/login"),
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

        Other(raw_path) => format!("{API_BASE}{raw_path}"),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AuthLoginRequest<'a> {
    id: &'a str,
    password: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AuthLoginResponseAimeCard {
    _accesscode: String,
    _comment: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AuthLoginResponseAimeCardList {
    _card_num: u16,
    _card_list: Vec<AuthLoginResponseAimeCard>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct AuthLoginResponse {
    login: bool,
    login_code: String, // Enum?
    _confirmed: bool,
    _aime_card: AuthLoginResponseAimeCardList,
    _hash_auth_key: Option<String>,
}

#[derive(Default)]
pub struct Client {
    client: ReqwestClient,
    userpass: Option<(String, String)>,
}

impl Client {
    pub async fn fetch(&self, endpoint: &ApiEndpoint) -> Result<Box<dyn Read>, Box<dyn Error>> {
        // TODO: Push the async higher, and return an AsyncReader here, so we don't have to
        // pull the whole response down.
        let mut response = self
            .client
            .get(url_for_endpoint(endpoint))
            .send()
            .await?
            .error_for_status();
        if let Err(error) = &response {
            if let Some(status) = error.status() {
                if status == StatusCode::FORBIDDEN && self.userpass.is_some() {
                    self.authenticate(
                        self.userpass.as_ref().unwrap().0.as_str(),
                        self.userpass.as_ref().unwrap().1.as_str(),
                    )
                    .await?;

                    response = self
                        .client
                        .get(url_for_endpoint(endpoint))
                        .send()
                        .await?
                        .error_for_status();
                }
            }
        }

        let body_text = response?.text().await?;
        Ok(Box::new(VecDeque::from(body_text.into_bytes())))
    }

    async fn authenticate(&self, id: &str, password: &str) -> Result<(), Box<dyn Error>> {
        let body = AuthLoginRequest { id, password };

        let body_response = self
            .client
            .post(url_for_endpoint(&ApiEndpoint::AuthLogin))
            // Some kind of user-agent sniffing going on, without this, _success_ produces a 500 error.
            .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let auth_login_response: AuthLoginResponse = serde_json::from_str(&body_response)?;

        if auth_login_response.login {
            Ok(())
        } else {
            // TODO: We _really_ need a custom error code.
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                auth_login_response.login_code,
            )))
        }
    }
}
