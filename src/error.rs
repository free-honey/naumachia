use thiserror::Error;

use crate::scripts::ScriptError;
use crate::{address::Address, address::PolicyId, logic::SCLogicError, txorecord::TxORecordError};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TxORecord Error: {0}")]
    TxORecord(#[from] TxORecordError),
    #[error("ValidatorCode Error: {0}")]
    ValidatorCode(#[from] ScriptError),
    #[error("Smart Contract Logic Error: {0:?}")]
    SCLogic(#[from] SCLogicError),
    #[error("Error: Insufficient amount of {0:?}.")]
    InsufficientAmountOf(PolicyId),
    #[error("Error: Failed to retrieve script for {0:?}.")]
    FailedToRetrieveScriptFor(Address),
    #[error("Error: Failed to retrieve redeemer for {0:?}.")]
    FailedToRetrieveRedeemerFor(Address),
    #[error("Unable to mint ADA/Lovelace")]
    ImpossibleToMintADA,
}
