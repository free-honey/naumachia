use async_trait::async_trait;
use std::fmt::Debug;

use crate::{error::Result, ledger_client::LedgerClient, logic::SCLogic};

#[async_trait]
pub trait SmartContractTrait {
    type Endpoint;
    type Lookup;
    type LookupResponse;
    async fn hit_endpoint(&self, endpoint: Self::Endpoint) -> Result<()>;
    async fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse>;
}

#[derive(Debug)]
pub struct SmartContract<Logic, LC>
where
    Logic: SCLogic,
    LC: LedgerClient<Logic::Datums, Logic::Redeemers>,
{
    offchain_logic: Logic,
    ledger_client: LC,
}

impl<Logic, LC> SmartContract<Logic, LC>
where
    Logic: SCLogic,
    LC: LedgerClient<Logic::Datums, Logic::Redeemers>,
{
    pub fn new(offchain_logic: Logic, backend: LC) -> Self {
        SmartContract {
            offchain_logic,
            ledger_client: backend,
        }
    }

    pub fn ledger_client(&self) -> &LC {
        &self.ledger_client
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
        let tx_actions = Logic::handle_endpoint(endpoint, &self.ledger_client).await?;
        let tx = tx_actions.to_unbuilt_tx()?;
        let tx_id = self.ledger_client.issue(tx).await?;
        println!("Transaction Submitted: {:?}", &tx_id);
        Ok(())
    }

    async fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse> {
        Ok(Logic::lookup(lookup, &self.ledger_client).await?)
    }
}
