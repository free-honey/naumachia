use thiserror::Error;

use crate::{address::Address, error::Result};

use std::error;

// TODO: Move
#[derive(Clone)]
pub struct TxContext {
    pub signer: Address,
}

pub trait ValidatorCode<D, R> {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> Result<()>;
    fn address(&self) -> Address;
}

#[derive(Debug, Error)]
pub enum ValidatorCodeError {
    #[error("Failed to execute: {0:?}")]
    FailedToExecute(Box<dyn error::Error>)
}
