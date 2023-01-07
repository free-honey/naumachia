use async_trait::async_trait;
use naumachia::ledger_client::LedgerClient;
use naumachia::logic::{SCLogic, SCLogicResult};
use naumachia::transaction::TxActions;

pub mod scripts;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimeLockedLogic;

pub enum PullEndpoints {
    Init,
    Fund,
    Pull,
}

pub struct CheckingAccount;

#[async_trait]
impl SCLogic for CheckingAccount {
    type Endpoints = PullEndpoints;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        todo!()
    }

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponses> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn init_creates_instance() {
        todo!()
    }
}
