use crate::logic::script::get_script;
use async_trait::async_trait;
use naumachia::output::DatumKind;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimeLockedLogic;

pub enum TimeLockedEndpoints {
    Lock { amount: u64, after_secs: i64 },
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
    #[error("Could not find a datum in output: {0:?}")]
    DatumUnreadable(OutputId),
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
            TimeLockedEndpoints::Lock {
                amount,
                after_secs: timestamp,
            } => impl_lock(ledger_client, amount, timestamp).await,
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

async fn impl_lock<LC: LedgerClient<i64, ()>>(
    ledger_client: &LC,
    amount: u64,
    after_secs: i64,
) -> SCLogicResult<TxActions<i64, ()>> {
    let network = ledger_client.network().await?;
    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, amount);
    let script = get_script()?;
    let address = script.address(network)?;
    let current_time = ledger_client.current_time_secs().await?;
    let lock_timestamp = (current_time + after_secs) * 1000;
    let tx_actions = TxActions::v2().with_script_init(lock_timestamp, values, address);
    Ok(tx_actions)
}

async fn impl_claim<LC: LedgerClient<i64, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
) -> SCLogicResult<TxActions<i64, ()>> {
    let network = ledger_client.network().await?;
    let script = get_script()?;
    let address = script.address(network)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(TimeLockedError::OutputNotFound(output_id.clone()))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let redeemer = ();
    let script_box = Box::new(script);
    let lower_bound = if let DatumKind::Typed(inner) = output.datum().clone() {
        inner / 1000
    } else {
        return Err(SCLogicError::Endpoint(Box::new(
            TimeLockedError::OutputNotFound(output_id),
        )));
    };
    let tx_actions = TxActions::v2()
        .with_script_redeem(output, redeemer, script_box)
        .with_valid_range_secs(Some(lower_bound), None);
    Ok(tx_actions)
}

async fn impl_list_active_contracts<LC: LedgerClient<i64, ()>>(
    ledger_client: &LC,
    count: usize,
) -> SCLogicResult<TimeLockedLookupResponses> {
    let network = ledger_client.network().await?;
    let script = get_script()?;
    let address = script.address(network)?;
    let outputs = ledger_client.outputs_at_address(&address, count).await?;
    let subset = outputs.into_iter().take(count).collect();
    let res = TimeLockedLookupResponses::ActiveContracts(subset);
    Ok(res)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use naumachia::{
        error::Error,
        ledger_client::test_ledger_client::TestBackendsBuilder,
        ledger_client::LedgerClientError,
        smart_contract::{SmartContract, SmartContractTrait},
        Address, Network,
    };

    #[tokio::test]
    async fn lock__creates_new_instance() {
        // given
        let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let start_amount = 100_000_000;
        let start_time = 1;
        let backend = TestBackendsBuilder::new(&me)
            .with_starting_time(start_time)
            .start_output(&me)
            .with_value(PolicyId::Lovelace, start_amount)
            .finish_output()
            .build_in_memory();

        let amount = 10_000_000;
        let after_secs = 2;

        let endpoint = TimeLockedEndpoints::Lock { amount, after_secs };

        let contract = SmartContract::new(TimeLockedLogic, backend);

        // when
        contract.hit_endpoint(endpoint).await.unwrap();

        // then
        let network = contract.backend().ledger_client().network().await.unwrap();
        let script_address = get_script().unwrap().address(network).unwrap();
        let outputs = contract
            .backend()
            .ledger_client()
            .all_outputs_at_address(&script_address)
            .await
            .unwrap();

        let output = outputs.first().unwrap();

        let value = output.values().get(&PolicyId::Lovelace).unwrap();
        assert_eq!(value, amount);
        let datum = output.datum().clone().unwrap_typed();
        assert_eq!(datum, (start_time + after_secs) * 1000);
    }

    #[tokio::test]
    async fn claim__can_claim_output_within_time_range() {
        // given
        let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let start_amount = 100_000_000;
        let start_time = 10_000;

        let script_address = get_script().unwrap().address(Network::Testnet).unwrap();
        let locked_amount = 10_000_000;
        let datum = 5_000;

        let backend = TestBackendsBuilder::new(&me)
            .with_starting_time(start_time)
            .start_output(&me)
            .with_value(PolicyId::Lovelace, start_amount)
            .finish_output()
            .start_output(&script_address)
            .with_value(PolicyId::Lovelace, locked_amount)
            .with_datum(datum)
            .finish_output()
            .build_in_memory();

        let endpoint = TimeLockedEndpoints::Claim {
            output_id: backend
                .ledger_client()
                .outputs_at_address(&script_address, 1)
                .await
                .unwrap()
                .first()
                .unwrap()
                .id()
                .clone(),
        };

        let contract = SmartContract::new(TimeLockedLogic, backend);

        // when
        contract.hit_endpoint(endpoint).await.unwrap();

        // then
        let outputs = contract
            .backend()
            .ledger_client()
            .all_outputs_at_address(&script_address)
            .await
            .unwrap();
        assert!(outputs.is_empty());

        let my_balance = contract
            .backend()
            .ledger_client()
            .balance_at_address(&me, &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(my_balance, start_amount + locked_amount);
    }

    #[tokio::test]
    async fn claim__fails_if_out_of_range() {
        // given
        let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let start_amount = 100_000_000;
        let start_time = 10_000;

        let script_address = get_script().unwrap().address(Network::Testnet).unwrap();
        let locked_amount = 10_000_000;
        let datum = 15_000;

        let backend = TestBackendsBuilder::new(&me)
            .with_starting_time(start_time)
            .start_output(&me)
            .with_value(PolicyId::Lovelace, start_amount)
            .finish_output()
            .start_output(&script_address)
            .with_value(PolicyId::Lovelace, locked_amount)
            .with_datum(datum)
            .finish_output()
            .build_in_memory();

        let endpoint = TimeLockedEndpoints::Claim {
            output_id: backend
                .ledger_client()
                .outputs_at_address(&script_address, 1)
                .await
                .unwrap()
                .first()
                .unwrap()
                .id()
                .clone(),
        };

        let contract = SmartContract::new(TimeLockedLogic, backend);

        // when
        let res = contract.hit_endpoint(endpoint).await;

        // then
        let err = res.unwrap_err();
        assert!(matches!(
            err,
            Error::LedgerClient(LedgerClientError::FailedToIssueTx(_))
        ));
    }
}
