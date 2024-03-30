use crate::{
    ledger_client::LedgerClientError,
    scripts::ScriptError,
};
use std::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SCLogicError {
    #[error("Error handling endpoint: {0:?}")]
    Endpoint(Box<dyn error::Error + Send + Sync>),
    #[error("Error doing lookup: {0:?}")]
    Lookup(Box<dyn error::Error + Send + Sync>),
    #[error("Error from Validator Script: {0:?}")]
    ValidatorScript(ScriptError),
    #[error("Error from Policy Script: {0:?}")]
    PolicyScript(ScriptError),
    #[error("From LedgerClient: {0:?}")]
    LedgerClient(#[from] LedgerClientError),
    #[error("Error from Script: {0:?}")]
    ScriptError(#[from] ScriptError),
}

pub type SCLogicResult<T> = crate::error::Result<T, SCLogicError>;

pub fn as_endpoint_err<E: error::Error + Send + Sync + 'static>(
    error: E,
) -> SCLogicError {
    SCLogicError::Endpoint(Box::new(error))
}

pub fn as_lookup_err<E: error::Error + Send + Sync + 'static>(error: E) -> SCLogicError {
    SCLogicError::Lookup(Box::new(error))
}
