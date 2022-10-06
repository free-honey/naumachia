use async_trait::async_trait;
use naumachia::address::PolicyId;
use naumachia::ledger_client::LedgerClient;
use naumachia::logic::{as_lookup_err, SCLogic, SCLogicResult};
use naumachia::transaction::TxActions;

#[derive(Debug, Eq, PartialEq)]
pub struct TriremeLogic;

#[derive(Debug, Eq, PartialEq)]
pub enum TriremeLookups {
    LovelaceBalance,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TriremeResponses {
    LovelaceBalance(u64),
}

#[async_trait]
impl SCLogic for TriremeLogic {
    type Endpoint = ();
    type Lookup = TriremeLookups;
    type LookupResponse = TriremeResponses;
    type Datum = ();
    type Redeemer = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Endpoint,
        _ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datum, Self::Redeemer>> {
        todo!()
    }

    async fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        query: Self::Lookup,
        ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponse> {
        match query {
            TriremeLookups::LovelaceBalance => impl_lovelace_balance(ledger_client).await,
        }
    }
}

async fn impl_lovelace_balance<LC: LedgerClient<(), ()>>(
    ledger_client: &LC,
) -> SCLogicResult<TriremeResponses> {
    let address = ledger_client.signer().await.map_err(as_lookup_err)?;
    let lovelace = ledger_client
        .balance_at_address(&address, &PolicyId::ADA)
        .await
        .map_err(as_lookup_err)?;
    let response = TriremeResponses::LovelaceBalance(lovelace);
    Ok(response)
}
