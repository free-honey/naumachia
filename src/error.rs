use thiserror::Error;

use crate::{
    address::Address, address::Policy, logic::SCLogicError, txorecord::TxORecordError,
    validator::ValidatorCodeError,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TxORecord Error: {0}")]
    TxORecord(#[from] TxORecordError),
    #[error("ValidatorCode Error: {0}")]
    ValidatorCode(#[from] ValidatorCodeError),
    #[error("Smart Contract Logic Error: {0:?}")]
    SCLogic(#[from] SCLogicError),
    #[error("Error: Insufficient amount of {0:?}.")]
    InsufficientAmountOf(Policy),
    #[error("Error: Failed to retrieve script for {0:?}.")]
    FailedToRetrieveScriptFor(Address),
    #[error("Error: Failed to retrieve redeemer for {0:?}.")]
    FailedToRetrieveRedeemerFor(Address),
}
