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
    type Endpoint: Send + Sync;
    type Lookup: Send + Sync;
    type LookupResponse: Send + Sync;
    type Datum: Clone + Eq + Debug + Send + Sync;
    type Redeemer: Clone + PartialEq + Eq + Hash + Send + Sync;

    async fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datum, Self::Redeemer>>;

    async fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        query: Self::Lookup,
        ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponse>;
}

#[derive(Debug, Error)]
pub enum SCLogicError {
    #[error("Error handling endpoint: {0:?}")]
    Endpoint(Box<dyn error::Error>),
    #[error("Error doing lookup: {0:?}")]
    Lookup(Box<dyn error::Error>),
    #[error("Error from Validator Script: {0:?}")]
    ValidatorScript(ScriptError),
}

pub type SCLogicResult<T> = Result<T, SCLogicError>;
