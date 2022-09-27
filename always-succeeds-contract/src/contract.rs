use async_trait::async_trait;
use naumachia::output::OutputId;
use naumachia::transaction::TxId;
use naumachia::{
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicResult},
    transaction::TxActions,
};

pub mod script;

pub struct AlwaysSucceedsContract;

pub enum AlwaysSucceedsEndpoints {
    Lock { amount: u64 },
    Claim { output: OutputId },
}

pub enum AlwaysSucceedsLookups {
    ListActiveContracts { count: u64 },
}

#[async_trait]
impl SCLogic for AlwaysSucceedsContract {
    type Endpoint = AlwaysSucceedsEndpoints;
    type Lookup = ();
    type LookupResponse = ();
    type Datum = ();
    type Redeemer = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        txo_record: &Record,
    ) -> SCLogicResult<TxActions<Self::Datum, Self::Redeemer>> {
        todo!()
    }

    async fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Lookup,
        txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponse> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
