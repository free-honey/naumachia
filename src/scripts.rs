use crate::address::Address;
use crate::PolicyId;
use thiserror::Error;

// TODO: Move
#[derive(Clone)]
pub struct TxContext {
    pub signer: Address,
}

pub trait ValidatorCode<D, R>: Send + Sync {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> ScriptResult<()>;
    fn address(&self) -> Address;
}

pub trait MintingPolicy: Send + Sync {
    fn execute(&self, ctx: TxContext) -> ScriptResult<()>;
    fn id(&self) -> PolicyId;
}

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Failed to execute: {0:?}")]
    FailedToExecute(String),
}

pub type ScriptResult<T> = Result<T, ScriptError>;
