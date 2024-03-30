use context::TxContext;
use pallas_addresses::{
    Address,
    Network,
};
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

/// Interface for a script locking UTxOs at a script address
pub trait Validator<D, R>: Send + Sync {
    /// Execute the script with specified datum, redeemer, and tx context
    fn execute(
        &self,
        datum: D,
        redeemer: R,
        ctx: TxContext,
    ) -> ScriptResult<ExecutionCost>;
    /// Address of Outputs locked by this script
    fn address(&self, network: Network) -> ScriptResult<Address>;
    /// Hex bytes of the script
    fn script_hex(&self) -> ScriptResult<String>;
}

/// Interface for a script constraining the minting of tokens
pub trait MintingPolicy<R>: Send + Sync {
    /// Execute the script with specified redeemer and tx context
    fn execute(&self, redeemer: R, ctx: TxContext) -> ScriptResult<ExecutionCost>;
    /// Asset ID for tokens whose minting is constrained by this script
    fn id(&self) -> ScriptResult<String>;
    /// Hex bytes of the script
    fn script_hex(&self) -> ScriptResult<String>;
}

/// Cost of executing a script
#[derive(Clone, Debug)]
pub struct ExecutionCost {
    mem: i64,
    cpu: i64,
}

impl ExecutionCost {
    /// Constructor for an ExecutionCost
    pub fn new(mem: i64, cpu: i64) -> Self {
        ExecutionCost { mem, cpu }
    }

    /// Getter for the memory cost
    pub fn mem(&self) -> i64 {
        self.mem
    }

    /// Getter for the CPU cost
    pub fn cpu(&self) -> i64 {
        self.cpu
    }
}

impl Default for ExecutionCost {
    fn default() -> Self {
        ExecutionCost::new(0, 0)
    }
}

#[allow(missing_docs)]
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

/// Convert a generic error into a [`ScriptError'] `FailedToExecute` variant
pub fn as_failed_to_execute<E: Debug>(e: E) -> ScriptError {
    ScriptError::FailedToExecute(format!("{e:?}"))
}

#[allow(missing_docs)]
pub type ScriptResult<T> = Result<T, ScriptError>;
