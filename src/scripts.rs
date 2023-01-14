use crate::address::Address;
use context::TxContext;
use std::fmt::Debug;
use thiserror::Error;

pub mod raw_policy_script;
pub mod raw_script;
pub mod raw_validator_script;

pub mod context;

pub trait ValidatorCode<D, R>: Send + Sync {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> ScriptResult<()>;
    fn address(&self, network: u8) -> ScriptResult<Address>;
    fn script_hex(&self) -> ScriptResult<String>;
}

pub trait MintingPolicy<R>: Send + Sync {
    fn execute(&self, redeemer: R, ctx: TxContext) -> ScriptResult<()>;
    fn id(&self) -> ScriptResult<String>;
    fn script_hex(&self) -> ScriptResult<String>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScriptError {
    #[error("Failed to execute: {0:?}")]
    FailedToExecute(String),
    #[error("Failed to construct: {0:?}")]
    FailedToConstruct(String),
    #[error("Failed to deserialize Datum")]
    DatumDeserialization(String),
    #[error("Failed to deserialize Redeemer")]
    RedeemerDeserialization(String),
    #[error("Failed to retrieve script ID")]
    IdRetrieval(String),
    #[error("Failed to retrieve script Cbor Hex")]
    ScriptHexRetrieval(String),
}

pub fn as_failed_to_execute<E: Debug>(e: E) -> ScriptError {
    ScriptError::FailedToExecute(format!("{:?}", e))
}

pub type ScriptResult<T> = Result<T, ScriptError>;
