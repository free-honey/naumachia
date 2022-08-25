use crate::ledger_client::fake_address::FakeAddress;
use crate::PolicyId;
use thiserror::Error;

// TODO: Move
#[derive(Clone)]
pub struct TxContext<Address> {
    pub signer: Address,
}

pub trait ValidatorCode<Address, D, R> {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext<Address>) -> ScriptResult<()>;
    fn address(&self) -> Address;
}

pub trait MintingPolicy<Address> {
    fn execute(&self, ctx: TxContext<Address>) -> ScriptResult<()>;
    fn id(&self) -> PolicyId;
}

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Failed to execute: {0:?}")]
    FailedToExecute(String),
}

pub type ScriptResult<T> = Result<T, ScriptError>;
