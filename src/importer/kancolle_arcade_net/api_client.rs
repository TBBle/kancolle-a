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
    // Global data: no authentication needed
    KanmusuList,
    PlaceDistricts,
    PlacePlaces,

    // Per-user data: Authentication needed
    TcBookInfo,
    BlueprintListInfo,
    Other(String),
}

fn url_for_endpoint(endpoint: &ApiEndpoint) -> String {
    use ApiEndpoint::*;
    match endpoint {
        KanmusuList => {
            "https://kancolle-a.sega.jp/players/kekkonkakkokari/kanmusu_list.json".to_string()
        }

        TcBookInfo => format!("{API_BASE}TcBook/info"),
        BlueprintListInfo => format!("{API_BASE}BlueprintList/info"),
        PlaceDistricts => format!("{API_BASE}Place/districts"),
        PlacePlaces => format!("{API_BASE}Place/places"),
        Other(raw_path) => format!("{API_BASE}/{raw_path}"),
    }
}

#[derive(Default)]
pub struct Client {
    client: ReqwestClient,
}

impl Client {
    pub fn fetch(&self, endpoint: ApiEndpoint) -> Result<Box<dyn Read>, Box<dyn Error>> {
        Ok(Box::new(
            self.client
                .get(url_for_endpoint(&endpoint))
                .send()?
                .error_for_status()?,
        ))
    }
}
