use thiserror::Error;

use crate::address::Policy;
use crate::txorecord::TxORecordError;
use crate::validator::ValidatorCodeError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TxORecord Error: {0}")]
    TxORecord(TxORecordError),
    #[error("ValidatorCode Error: {0}")]
    ValidatorCode(ValidatorCodeError),
    #[error("Error: Insufficient amount of {0:?}.")]
    InsufficientAmountOf(Policy),
}
