use crate::logic::script::{get_parameterized_script, OutputReference};
use async_trait::async_trait;
use naumachia::address::PolicyId;
use naumachia::output::Output;
use naumachia::scripts::ScriptError;
use naumachia::{
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    transaction::TxActions,
};
use thiserror::Error;

pub mod script;

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
    let my_input = select_any_above_min(ledger_client).await?;
    let param_script = get_parameterized_script().map_err(SCLogicError::PolicyScript)?;
    let script = param_script
        .apply(OutputReference::from(&my_input))
        .map_err(|e| ScriptError::FailedToConstruct(format!("{:?}", e)))
        .map_err(SCLogicError::PolicyScript)?;
    let policy = Box::new(script);
    let actions = TxActions::v2()
        .with_mint(1, Some("OneShot".to_string()), (), policy)
        .with_specific_input(my_input);
    Ok(actions)
}

// This is a workaround. It shouldn't matter the size of the input, but there seems to be a bug
// in CML: https://github.com/MitchTurner/naumachia/issues/73
// Not happy about this leaking into my top level code.
async fn select_any_above_min<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
) -> SCLogicResult<Output<()>> {
    const MIN_LOVELACE: u64 = 5_000_000;
    let me = ledger_client
        .signer()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let selected = ledger_client
        .all_outputs_at_address(&me)
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .iter()
        .filter_map(|input| {
            if let Some(ada_value) = input.values().get(&PolicyId::ADA) {
                if ada_value > MIN_LOVELACE {
                    Some(input)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .pop()
        .ok_or(SCLogicError::Endpoint(Box::new(
            MintNFTError::InputNotFound,
        )))?
        .to_owned();
    println!("input id: {:?}", selected.id());
    Ok(selected)
}
