use async_trait::async_trait;
use naumachia::{
    ledger_client::LedgerClient,
    logic::{as_lookup_err, SCLogic, SCLogicResult},
    policy_id::PolicyId,
    transaction::TxActions,
    values::Values,
};

#[derive(Debug, Eq, PartialEq)]
pub struct TriremeLogic;

#[derive(Debug, Eq, PartialEq)]
pub enum TriremeLookups {
    LovelaceBalance,
    TotalBalance,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TriremeResponses {
    LovelaceBalance(u64),
    TotalBalance(Vec<(PolicyId, u64)>),
}

#[async_trait]
impl SCLogic for TriremeLogic {
    type Endpoints = ();
    type Lookups = TriremeLookups;
    type LookupResponses = TriremeResponses;
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        _endpoint: Self::Endpoints,
        _ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        todo!()
    }

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponses> {
        match query {
            TriremeLookups::LovelaceBalance => impl_lovelace_balance(ledger_client).await,
            TriremeLookups::TotalBalance => impl_total_balance(ledger_client).await,
        }
    }
}

async fn impl_lovelace_balance<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
) -> SCLogicResult<TriremeResponses> {
    let address = ledger_client
        .signer_base_address()
        .await
        .map_err(as_lookup_err)?;
    let lovelace = ledger_client
        .balance_at_address(&address, &PolicyId::Lovelace)
        .await
        .map_err(as_lookup_err)?;
    let response = TriremeResponses::LovelaceBalance(lovelace);
    Ok(response)
}

async fn impl_total_balance<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
) -> SCLogicResult<TriremeResponses> {
    let address = ledger_client
        .signer_base_address()
        .await
        .map_err(as_lookup_err)?;
    let outputs = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(as_lookup_err)?;

    let total_value = Values::from_outputs(&outputs).vec();
    let response = TriremeResponses::TotalBalance(total_value);
    Ok(response)
}
