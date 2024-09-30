//! Helpers for configuring cookie support in wasm32 builds
//! Necessary because reqwest's cookie support is excluded in wasm32 builds.

use crate::Result;

use reqwest::ClientBuilder as ReqwestBuilder;

pub(super) fn setup_cookies(
    jsessionid: Option<String>,
    builder: ReqwestBuilder,
) -> Result<ReqwestBuilder> {
    // TODO: wasm-cookies-rs could be used in the browser
    assert!(jsessionid.is_none());
    Ok(builder)
}
