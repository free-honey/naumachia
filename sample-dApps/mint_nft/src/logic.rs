use crate::logic::script::{get_parameterized_script, OutputReference};
use async_trait::async_trait;
use naumachia::output::Output;
use naumachia::scripts::ScriptError;
use naumachia::{
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    transaction::TxActions,
};
use thiserror::Error;

pub mod script;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MintNFTLogic;

pub enum MintNFTEndpoints {
    Mint,
}

#[derive(Debug, Error)]
pub enum MintNFTError {
    #[error("Could not find any UTxO to use as the input for NFT policy")]
    InputNotFound,
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
            MintNFTEndpoints::Mint => impl_mint(ledger_client).await,
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        _query: Self::Lookups,
        _ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        Ok(())
    }
}

async fn impl_mint<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
) -> SCLogicResult<TxActions<(), ()>> {
    let recipient = ledger_client
        .signer()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let my_input = any_input(ledger_client).await?;
    let param_script = get_parameterized_script().map_err(SCLogicError::PolicyScript)?;
    let script = param_script
        .apply(OutputReference::from(&my_input))
        .map_err(|e| ScriptError::FailedToConstruct(format!("{:?}", e)))
        .map_err(SCLogicError::PolicyScript)?;
    let policy = Box::new(script);
    let actions = TxActions::v2()
        .with_mint(1, Some("OneShot".to_string()), &recipient, (), policy)
        .with_specific_input(my_input);
    Ok(actions)
}

async fn any_input<LC: LedgerClient<(), ()>>(ledger_client: &LC) -> SCLogicResult<Output<()>> {
    let me = ledger_client
        .signer()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let input = ledger_client
        .outputs_at_address(&me, 1)
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .pop()
        .ok_or(SCLogicError::Endpoint(Box::new(
            MintNFTError::InputNotFound,
        )))?;
    println!("input id: {:?}", input.id());
    Ok(input)
}
