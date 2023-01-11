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

// TODO: Pass through someplace, do not hardcode!
const NETWORK: u8 = 0;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimeLockedLogic;

pub enum TimeLockedEndpoints {
    Lock { amount: u64, timestamp: i64 },
    Claim { output_id: OutputId },
}

pub enum TimeLockedLookups {
    ListActiveContracts { count: usize },
}

pub enum TimeLockedLookupResponses {
    ActiveContracts(Vec<Output<i64>>),
}

#[derive(Debug, Error)]
pub enum TimeLockedError {
    #[error("Could not find an output with id: {0:?}")]
    OutputNotFound(OutputId),
}

#[async_trait]
impl SCLogic for TimeLockedLogic {
    type Endpoints = TimeLockedEndpoints;
    type Lookups = TimeLockedLookups;
    type LookupResponses = TimeLockedLookupResponses;
    type Datums = i64;
    type Redeemers = ();

    async fn handle_endpoint<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            TimeLockedEndpoints::Lock { amount, timestamp } => impl_lock(amount, timestamp),
            TimeLockedEndpoints::Claim { output_id } => impl_claim(ledger_client, output_id).await,
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        match query {
            TimeLockedLookups::ListActiveContracts { count } => {
                impl_list_active_contracts(ledger_client, count).await
            }
        }
    }
}

fn impl_lock(amount: u64, timestamp: i64) -> SCLogicResult<TxActions<i64, ()>> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let tx_actions = TxActions::v2().with_script_init(timestamp, values, address);
    Ok(tx_actions)
}

async fn impl_claim<LC: LedgerClient<i64, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
) -> SCLogicResult<TxActions<i64, ()>> {
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
        .ok_or(TimeLockedError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let redeemer = ();
    let script_box = Box::new(script);
    let tx_actions = TxActions::v2()
        .with_script_redeem(output, redeemer, script_box)
        .with_valid_range(Some(1595967616), None);
    Ok(tx_actions)
}

async fn impl_list_active_contracts<LC: LedgerClient<i64, ()>>(
    ledger_client: &LC,
    count: usize,
) -> SCLogicResult<TimeLockedLookupResponses> {
    let script = get_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let outputs = ledger_client
        .outputs_at_address(&address, count)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?;
    let subset = outputs.into_iter().take(count).collect();
    let res = TimeLockedLookupResponses::ActiveContracts(subset);
    Ok(res)
}
