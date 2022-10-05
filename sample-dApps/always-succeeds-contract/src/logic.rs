use crate::logic::script::AlwaysSucceedsScript;
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

pub enum AlwaysSucceedsEndpoints {
    Lock { amount: u64 },
    Claim { output_id: OutputId },
}

pub enum AlwaysSucceedsLookups {
    ListActiveContracts { count: usize },
}

pub enum AlwaysSucceedsLookupResponses {
    ActiveContracts(Vec<Output<()>>),
}

#[derive(Debug, Error)]
pub enum AlwaysSucceedsError {
    #[error("Could not find an output with id: {0:?}")]
    OutputNotFound(OutputId),
}

#[async_trait]
impl SCLogic for AlwaysSucceedsLogic {
    type Endpoint = AlwaysSucceedsEndpoints;
    type Lookup = AlwaysSucceedsLookups;
    type LookupResponse = AlwaysSucceedsLookupResponses;
    type Datum = ();
    type Redeemer = ();

    async fn handle_endpoint<LC: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datum, Self::Redeemer>> {
        match endpoint {
            AlwaysSucceedsEndpoints::Lock { amount } => impl_lock(amount),
            AlwaysSucceedsEndpoints::Claim { output_id } => {
                impl_claim(ledger_client, output_id).await
            }
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datum, Self::Redeemer>>(
        query: Self::Lookup,
        ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponse> {
        match query {
            AlwaysSucceedsLookups::ListActiveContracts { count } => {
                impl_list_active_contracts(ledger_client, count).await
            }
        }
    }
}

fn impl_lock(amount: u64) -> SCLogicResult<TxActions<(), ()>> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let script = AlwaysSucceedsScript::try_new().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let tx_actions = TxActions::default().with_script_init((), values, address);
    Ok(tx_actions)
}

async fn impl_claim<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
    output_id: OutputId,
) -> SCLogicResult<TxActions<(), ()>> {
    let script = AlwaysSucceedsScript::try_new().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(AlwaysSucceedsError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let redeemer = ();
    let script_box = Box::new(script);
    let tx_actions = TxActions::default().with_script_redeem(output, redeemer, script_box);
    Ok(tx_actions)
}

async fn impl_list_active_contracts<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
    count: usize,
) -> SCLogicResult<AlwaysSucceedsLookupResponses> {
    let script = AlwaysSucceedsScript::try_new().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let outputs = ledger_client
        .outputs_at_address(&address, count)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?;
    let subset = outputs.into_iter().take(count).collect();
    let res = AlwaysSucceedsLookupResponses::ActiveContracts(subset);
    Ok(res)
}
