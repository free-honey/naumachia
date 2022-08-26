use crate::ledger_client::LedgerClient;
use crate::{error::Result, UnBuiltTransaction};

use thiserror::Error;

use std::error;
use std::fmt::Debug;
use std::hash::Hash;

pub trait SCLogic: Send + Sync {
    type Endpoint: Send + Sync;
    type Lookup: Send + Sync;
    type LookupResponse: Send + Sync;
    type Datum: Clone + Eq + Debug + Send + Sync;
    type Redeemer: Clone + PartialEq + Eq + Hash + Send + Sync;

    fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        txo_record: &Record,
    ) -> SCLogicResult<UnBuiltTransaction<Self::Datum, Self::Redeemer>>;

    fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Lookup,
        txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponse>;
}

#[derive(Debug, Error)]
pub enum SCLogicError {
    #[error("Error handling endpoint: {0:?}")]
    Endpoint(Box<dyn error::Error>),
    #[error("Error doing lookup: {0:?}")]
    Lookup(Box<dyn error::Error>),
}

pub type SCLogicResult<T> = Result<T, SCLogicError>;
