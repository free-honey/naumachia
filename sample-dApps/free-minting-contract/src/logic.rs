use async_trait::async_trait;
use naumachia::{
    address::PolicyId,
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    output::{Output, OutputId},
    scripts::ValidatorCode,
    transaction::TxActions,
    values::Values,
};
use thiserror::Error;

pub mod script;
#[cfg(test)]
mod tests;

// TODO: Pass through someplace, do not hardcode!
const NETWORK: u8 = 0;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlwaysSucceedsLogic;

pub enum FreeMintingEndpoints {
    Mint { amount: u64 },
}

#[async_trait]
impl SCLogic for AlwaysSucceedsLogic {
    type Endpoints = FreeMintingEndpoints;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        todo!()
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        unimplemented!()
    }
}
