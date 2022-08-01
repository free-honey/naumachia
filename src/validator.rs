use crate::address::Address;

// TODO: Move
pub struct TxContext;

pub trait ValidatorCode<D, R> {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> bool;
    fn address(&self) -> Address;
}
