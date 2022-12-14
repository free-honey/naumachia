use crate::logic::script::get_script;
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
pub struct MintNFTLogic;

pub enum MintNFTEndpoints {
    Mint,
}

#[derive(Debug, Error)]
pub enum AlwaysSucceedsError {
    #[error("Could not find an output with id: {0:?}")]
    OutputNotFound(OutputId),
}

#[async_trait]
impl SCLogic for MintNFTLogic {
    type Endpoints = MintNFTEndpoints;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<LC: LedgerClient<(), ()>>(
        endpoint: Self::Endpoints,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            MintNFTEndpoints::Mint => impl_mint(ledger_client),
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        _query: Self::Lookups,
        _ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        Ok(())
    }
}

fn impl_mint<LC: LedgerClient<(), ()>>(ledger_client: &LC) -> SCLogicResult<TxActions<(), ()>> {
    todo!()
}
