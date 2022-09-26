use std::io;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("url Error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Config field not found: {0:?}")]
    Config(String),
    #[error("Error while reading file: {0:?}")]
    FileRead(io::Error),
    #[error("Error while parsing Toml: {0:?}")]
    Toml(toml::de::Error),
    #[error("EvaluateTxResult malformed: {0:?}")]
    EvaluateTxResult(Box<dyn std::error::Error + Send>),
}
