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
    Record: LedgerClient<Logic::Datum, Logic::Redeemer>,
{
    pub smart_contract: &'a Logic,
    pub backend: &'a Backend<Logic::Datum, Logic::Redeemer, Record>,
}

impl<'a, Logic, Record> SmartContract<'a, Logic, Record>
where
    Logic: SCLogic,
    Record: LedgerClient<Logic::Datum, Logic::Redeemer>,
{
    pub fn new(
        smart_contract: &'a Logic,
        backend: &'a Backend<Logic::Datum, Logic::Redeemer, Record>,
    ) -> Self {
        SmartContract {
            smart_contract,
            backend,
        }
    }
}

#[async_trait]
impl<'a, Logic, Record> SmartContractTrait for SmartContract<'a, Logic, Record>
where
    Logic: SCLogic + Eq + Debug + Send + Sync,
    Record: LedgerClient<Logic::Datum, Logic::Redeemer> + Send + Sync,
{
    type Endpoint = Logic::Endpoint;
    type Lookup = Logic::Lookup;
    type LookupResponse = Logic::LookupResponse;

    async fn hit_endpoint(&self, endpoint: Logic::Endpoint) -> Result<()> {
        let tx_actions = Logic::handle_endpoint(endpoint, self.backend.ledger_client()).await?;
        self.backend.process(tx_actions).await?;
        Ok(())
    }

    async fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse> {
        Ok(Logic::lookup(lookup, self.backend.ledger_client()).await?)
    }
}
