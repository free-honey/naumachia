use crate::address::Address;
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::UnbuiltTransaction;
use std::fmt::Debug;
use thiserror::Error;

pub mod raw_policy_script;
pub mod raw_script;
pub mod raw_validator_script;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
#[derive(Clone)]
pub struct TxContext {
    pub signer: Address,
    pub range: ValidRange,
}

#[derive(Clone)]
pub struct ValidRange {
    pub lower: Option<(i64, bool)>,
    pub upper: Option<(i64, bool)>,
}

pub struct ContextBuilder {
    signer: Address,
    range: Option<ValidRange>,
}

impl ContextBuilder {
    pub fn new(signer: Address) -> Self {
        ContextBuilder {
            signer,
            range: None,
        }
    }

    pub fn with_range(mut self, lower: Option<(i64, bool)>, upper: Option<(i64, bool)>) -> Self {
        let valid_range = ValidRange { lower, upper };
        self.range = Some(valid_range);
        self
    }

    pub fn build(&self) -> TxContext {
        let range = if let Some(range) = self.range.clone() {
            range
        } else {
            ValidRange {
                lower: None,
                upper: None,
            }
        };
        TxContext {
            signer: self.signer.clone(),
            range,
        }
    }
}

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
