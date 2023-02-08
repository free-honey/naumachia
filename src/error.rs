use pallas_addresses::Address;
use thiserror::Error;

use crate::scripts::ScriptError;
use crate::{address::PolicyId, ledger_client::LedgerClientError, logic::SCLogicError};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error with Address")]
    Address(String),
    #[error("TxORecord Error: {0}")]
    TxORecord(#[from] LedgerClientError),
    #[error("ValidatorCode Error: {0}")]
    Script(#[from] ScriptError),
    #[error("Smart Contract Logic Error: {0:?}")]
    SCLogic(#[from] SCLogicError),
    #[error("Error: Insufficient amount of {0:?}.")]
    InsufficientAmountOf(PolicyId),
    #[error("Error: Failed to retrieve policy for {0:?}.")]
    FailedToRetrievePolicyFor(PolicyId),
    #[error("Error: Failed to retrieve script for {0:?}.")]
    FailedToRetrieveScriptFor(Address),
    #[error("Error: Failed to retrieve redeemer for {0:?}.")]
    FailedToRetrieveRedeemerFor(Address),
    #[error("Unable to mint ADA/Lovelace")]
    ImpossibleToMintADA,
    #[error("Error with Trireme integration: {0:?}")]
    Trireme(String),
    #[error("Error dealing with TOML files: {0:?}")]
    TOML(Box<dyn std::error::Error + Send + Sync>),
}
