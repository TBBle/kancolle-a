//! Helpers for configuring cookie support (except in wasm32 builds)
//! Necessary because reqwest's cookie support is excluded in wasm32 builds.

use crate::Result;

use reqwest::cookie::Jar;
use reqwest::ClientBuilder as ReqwestBuilder;
use std::sync::Arc;
use url::Url;

pub(super) fn setup_cookies(
    jsessionid: Option<String>,
    builder: ReqwestBuilder,
) -> Result<ReqwestBuilder> {
    Ok(if let Some(jsessionid) = jsessionid {
        let cookies = Jar::default();
        cookies.add_cookie_str(
            &format!("JSESSIONID={}; Path=/; HttpOnly", jsessionid),
            &super::API_BASE.parse::<Url>()?,
        );
        builder.cookie_provider(Arc::new(cookies))
    } else {
        builder.cookie_store(true)
    })
}
