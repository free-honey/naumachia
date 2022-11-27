use crate::ledger_client::LedgerClient;
use crate::{error::Result, TxActions};

use thiserror::Error;

use crate::scripts::ScriptError;
use async_trait::async_trait;
use std::error;
use std::fmt::Debug;
use std::hash::Hash;

#[async_trait]
pub trait SCLogic: Send + Sync {
    type Endpoints: Send + Sync;
    type Lookups: Send + Sync;
    type LookupResponses: Send + Sync;
    type Datums: Clone + Eq + Debug + Send + Sync;
    type Redeemers: Clone + PartialEq + Eq + Hash + Send + Sync;

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>>;

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponses>;
}

#[derive(Debug, Error)]
pub enum SCLogicError {
    #[error("Error handling endpoint: {0:?}")]
    Endpoint(Box<dyn error::Error + Send + Sync>),
    #[error("Error doing lookup: {0:?}")]
    Lookup(Box<dyn error::Error + Send + Sync>),
    #[error("Error from Validator Script: {0:?}")]
    ValidatorScript(ScriptError),
}

pub type SCLogicResult<T> = Result<T, SCLogicError>;

pub fn as_endpoint_err<E: error::Error + Send + Sync + 'static>(error: E) -> SCLogicError {
    SCLogicError::Endpoint(Box::new(error))
}

pub fn as_lookup_err<E: error::Error + Send + Sync + 'static>(error: E) -> SCLogicError {
    SCLogicError::Lookup(Box::new(error))
}
