use crate::address::Address;

use crate::error::Result;

// TODO: Move
#[derive(Clone)]
pub struct TxContext {
    pub signer: Address,
}

pub trait ValidatorCode<D, R> {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> Result<()>;
    fn address(&self) -> Address;
}
