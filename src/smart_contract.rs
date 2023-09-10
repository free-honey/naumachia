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
pub struct SmartContract<Logic, Record>
where
    Logic: SCLogic,
    Record: LedgerClient<Logic::Datums, Logic::Redeemers>,
{
    offchain_logic: Logic,
    backend: Backend<Logic::Datums, Logic::Redeemers, Record>,
}

impl<Logic, Record> SmartContract<Logic, Record>
where
    Logic: SCLogic,
    Record: LedgerClient<Logic::Datums, Logic::Redeemers>,
{
    pub fn new(
        offchain_logic: Logic,
        backend: Backend<Logic::Datums, Logic::Redeemers, Record>,
    ) -> Self {
        SmartContract {
            offchain_logic,
            backend,
        }
    }

    pub fn backend(&self) -> &Backend<Logic::Datums, Logic::Redeemers, Record> {
        &self.backend
    }

    pub fn offchain_logic(&self) -> &Logic {
        &self.offchain_logic
    }
}

#[async_trait]
impl<Logic, Record> SmartContractTrait for SmartContract<Logic, Record>
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
