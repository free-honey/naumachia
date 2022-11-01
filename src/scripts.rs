use crate::address::Address;
use crate::PolicyId;
use std::fmt::Debug;
use thiserror::Error;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
#[derive(Clone)]
pub struct TxContext {
    pub signer: Address,
}

pub trait ValidatorCode<D, R>: Send + Sync {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> ScriptResult<()>;
    // TODO: Add network param!
    fn address(&self, network: u8) -> ScriptResult<Address>;
    fn script_hex(&self) -> ScriptResult<&str>;
}

pub trait MintingPolicy: Send + Sync {
    fn execute(&self, ctx: TxContext) -> ScriptResult<()>;
    // TODO: Add network param!
    fn id(&self) -> PolicyId;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScriptError {
    #[error("Failed to execute: {0:?}")]
    FailedToExecute(String),
    #[error("Failed to construct: {0:?}")]
    FailedToConstruct(String),
}

pub fn as_failed_to_execute<E: Debug>(e: E) -> ScriptError {
    ScriptError::FailedToExecute(format!("{:?}", e))
}

pub type ScriptResult<T> = Result<T, ScriptError>;
