use crate::address::Address;

// TODO: Move
pub struct TxContext;

pub trait ValidatorCode {
    fn execute<D, R>(datum: D, redeemer: R, ctx: TxContext) -> bool;
    fn address() -> Address;
}
