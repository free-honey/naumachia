use async_trait::async_trait;
use std::fmt::Debug;

use crate::{backend::Backend, error::Result, ledger_client::LedgerClient, logic::SCLogic};

#[async_trait]
pub trait SmartContractTrait {
    type Endpoint;
    type Lookup;
    type LookupResponse;
    async fn hit_endpoint(&self, endpoint: Self::Endpoint) -> Result<()>;
    async fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse>;
}

#[derive(Debug)]
pub struct SmartContract<'a, Logic, Record>
where
    Logic: SCLogic,
    Record: LedgerClient<Logic::Datums, Logic::Redeemers>,
{
    pub offchain_logic: &'a Logic,
    pub backend: &'a Backend<Logic::Datums, Logic::Redeemers, Record>,
}

impl<'a, Logic, Record> SmartContract<'a, Logic, Record>
where
    Logic: SCLogic,
    Record: LedgerClient<Logic::Datums, Logic::Redeemers>,
{
    pub fn new(
        offchain_logic: &'a Logic,
        backend: &'a Backend<Logic::Datums, Logic::Redeemers, Record>,
    ) -> Self {
        SmartContract {
            offchain_logic,
            backend,
        }
    }
}

#[async_trait]
impl<'a, Logic, Record> SmartContractTrait for SmartContract<'a, Logic, Record>
where
    Logic: SCLogic + Eq + Debug + Send + Sync,
    Record: LedgerClient<Logic::Datums, Logic::Redeemers> + Send + Sync,
{
    type Endpoint = Logic::Endpoints;
    type Lookup = Logic::Lookups;
    type LookupResponse = Logic::LookupResponses;

    async fn hit_endpoint(&self, endpoint: Logic::Endpoints) -> Result<()> {
        let tx_actions = Logic::handle_endpoint(endpoint, self.backend.ledger_client()).await?;
        self.backend.process(tx_actions).await?;
        Ok(())
    }

    async fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse> {
        Ok(Logic::lookup(lookup, self.backend.ledger_client()).await?)
    }
}
