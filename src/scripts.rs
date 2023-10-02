use context::TxContext;
use pallas_addresses::{Address, Network};
use std::fmt::Debug;
use thiserror::Error;

/// Script context types
pub mod context;
/// Adapter code for [`MintingPolicy`]
pub mod plutus_minting_policy;
/// Adapter code for [`Validator`]
pub mod plutus_validator;
/// Raw script types
pub mod raw_script;

pub trait Validator<D, R>: Send + Sync {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> ScriptResult<ExecutionCost>;
    fn address(&self, network: Network) -> ScriptResult<Address>;
    fn script_hex(&self) -> ScriptResult<String>;
}

pub trait MintingPolicy<R>: Send + Sync {
    fn execute(&self, redeemer: R, ctx: TxContext) -> ScriptResult<ExecutionCost>;
    fn id(&self) -> ScriptResult<String>;
    fn script_hex(&self) -> ScriptResult<String>;
}

#[derive(Clone, Debug)]
pub struct ExecutionCost {
    mem: i64,
    cpu: i64,
}

impl ExecutionCost {
    pub fn new(mem: i64, cpu: i64) -> Self {
        ExecutionCost { mem, cpu }
    }

    pub fn mem(&self) -> i64 {
        self.mem
    }

    pub fn cpu(&self) -> i64 {
        self.cpu
    }
}

impl Default for ExecutionCost {
    fn default() -> Self {
        ExecutionCost::new(0, 0)
    }
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
    ScriptError::FailedToExecute(format!("{e:?}"))
}

pub type ScriptResult<T> = Result<T, ScriptError>;
