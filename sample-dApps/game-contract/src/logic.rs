use crate::logic::script::{get_script, ClearString, HashedString};
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
pub struct GameLogic;

pub enum GameEndpoints {
    Lock { amount: u64, secret: String },
    Guess { output_id: OutputId, guess: String },
}

pub enum GameLookups {
    ListActiveContracts { count: usize },
}

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
    type Endpoint = GameEndpoints;
    type Lookup = GameLookups;
    type LookupResponse = GameLookupResponses;
    type Datum = HashedString;
    type Redeemer = ClearString;

    async fn handle_endpoint<LC: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datum, Self::Redeemer>> {
        match endpoint {
            GameEndpoints::Lock { amount, secret } => impl_lock(amount, &secret),
            GameEndpoints::Guess { output_id, guess } => {
                impl_guess(ledger_client, output_id, &guess).await
            }
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datum, Self::Redeemer>>(
        query: Self::Lookup,
        ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponse> {
        match query {
            GameLookups::ListActiveContracts { count } => {
                impl_list_active_contracts(ledger_client, count).await
            }
        }
    }
}

fn impl_lock(amount: u64, secret: &str) -> SCLogicResult<TxActions<HashedString, ClearString>> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let hashed_string = HashedString::new(secret);
    let tx_actions = TxActions::default().with_script_init(hashed_string, values, address);
    Ok(tx_actions)
}

async fn impl_guess<LC: LedgerClient<HashedString, ClearString>>(
    ledger_client: &LC,
    output_id: OutputId,
    guess: &str,
) -> SCLogicResult<TxActions<HashedString, ClearString>> {
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
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
    let tx_actions = TxActions::default().with_script_redeem(output, redeemer, script_box);
    Ok(tx_actions)
}

async fn impl_list_active_contracts<LC: LedgerClient<HashedString, ClearString>>(
    ledger_client: &LC,
    count: usize,
) -> SCLogicResult<GameLookupResponses> {
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let outputs = ledger_client
        .outputs_at_address(&address, count)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?;
    let subset = outputs.into_iter().take(count).collect();
    let res = GameLookupResponses::ActiveContracts(subset);
    Ok(res)
}
