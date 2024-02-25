use crate::logic::script::{get_script, ClearString, HashedString};
use async_trait::async_trait;
use naumachia::logic::error::{SCLogicError, SCLogicResult};
use naumachia::{
    ledger_client::LedgerClient,
    logic::SCLogic,
    output::{Output, OutputId},
    policy_id::PolicyId,
    scripts::Validator,
    transaction::TxActions,
    values::Values,
};
use thiserror::Error;

pub mod script;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GameLogic;

#[derive(Debug)]
pub enum GameEndpoints {
    Lock { amount: u64, secret: String },
    Guess { output_id: OutputId, guess: String },
}

#[derive(Debug)]
pub enum GameLookups {
    ListActiveContracts { count: usize },
}

#[derive(Debug)]
pub enum GameLookupResponses {
    ActiveContracts(Vec<Output<HashedString>>),
}

#[derive(Debug, Error)]
pub enum GameSucceedsError {
    #[error("Could not find an output with id: {0:?}")]
    OutputNotFound(OutputId),
}

#[async_trait]
impl SCLogic for GameLogic {
    type Endpoints = GameEndpoints;
    type Lookups = GameLookups;
    type LookupResponses = GameLookupResponses;
    type Datums = HashedString;
    type Redeemers = ClearString;

    async fn handle_endpoint<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            GameEndpoints::Lock { amount, secret } => {
                impl_lock(ledger_client, amount, &secret).await
            }
            GameEndpoints::Guess { output_id, guess } => {
                impl_guess(ledger_client, output_id, &guess).await
            }
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        match query {
            GameLookups::ListActiveContracts { count } => {
                impl_list_active_contracts(ledger_client, count).await
            }
        }
    }
}

async fn impl_lock<LC: LedgerClient<HashedString, ClearString>>(
    ledger_client: &LC,
    amount: u64,
    secret: &str,
) -> SCLogicResult<TxActions<HashedString, ClearString>> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, amount);
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(network)
        .map_err(SCLogicError::ValidatorScript)?;
    let hashed_string = HashedString::new(secret);
    let tx_actions = TxActions::v2().with_script_init(hashed_string, values, address);
    Ok(tx_actions)
}

async fn impl_guess<LC: LedgerClient<HashedString, ClearString>>(
    ledger_client: &LC,
    output_id: OutputId,
    guess: &str,
) -> SCLogicResult<TxActions<HashedString, ClearString>> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(network)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(GameSucceedsError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let redeemer = ClearString::new(guess);
    let script_box = Box::new(script);
    let tx_actions = TxActions::v2().with_script_redeem(output, redeemer, script_box);
    Ok(tx_actions)
}

async fn impl_list_active_contracts<LC: LedgerClient<HashedString, ClearString>>(
    ledger_client: &LC,
    count: usize,
) -> SCLogicResult<GameLookupResponses> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(network)
        .map_err(SCLogicError::ValidatorScript)?;
    let outputs = ledger_client
        .outputs_at_address(&address, count)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?;
    let subset = outputs.into_iter().take(count).collect();
    let res = GameLookupResponses::ActiveContracts(subset);
    Ok(res)
}
