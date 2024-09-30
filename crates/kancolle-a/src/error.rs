//! Error and failure handing types

use thiserror::Error;

use csv::Error as CSVError;
use reqwest::header::InvalidHeaderValue;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use url::ParseError;

/// A `Result` alias where the `Err` case is `kancolle-a::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// The Errors that may occur in the kancolle-a crate APIs.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Authentication failed, login_code {0}")]
    AuthenticationFailed(String),

    // Passthroughs from other libraries
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error(transparent)]
    ReqwestError(#[from] ReqwestError),
    #[error(transparent)]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error(transparent)]
    CSVError(#[from] CSVError),
}
