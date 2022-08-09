use thiserror::Error;

use crate::backend::TxORecordError;
use crate::validator::ValidatorCodeError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TxORecord Error: {0}")]
    TxORecord(TxORecordError),
    #[error("ValidatorCode Error: {0}")]
    ValidatorCode(ValidatorCodeError),
}
