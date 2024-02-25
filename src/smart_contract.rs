use async_trait::async_trait;
use std::fmt::Debug;

use crate::transaction::TxId;
use crate::{error::Result, ledger_client::LedgerClient, logic::SCLogic};
use crate::ledger_client::LedgerClientResult;

/// Interface defining how to interact with your smart contract
#[async_trait]
pub trait SmartContractTrait {
    /// Represents the domain-specific transactions the consumer of a Smart Contract can submit.
    type Endpoint;

    /// Represents the domain-specific data the consumer of a Smart Contract can query.
    type Lookup;

    /// Responses from the Lookup queries
    type LookupResponse;

    /// Method for hitting specific endpoint
    async fn hit_endpoint(&self, endpoint: Self::Endpoint) -> Result<TxId>;
    /// Method for querying specific data
    async fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse>;
}

/// Standard, concrete implementation of a Smart Contract
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
    /// Constructor for standard SmartContract impl
    pub fn new(offchain_logic: Logic, backend: LC) -> Self {
        SmartContract {
            offchain_logic,
            ledger_client: backend,
        }
    }

    /// Returns reference to LedgerClient used by the SmartContract

    pub fn ledger_client(&self) -> &LC {
        &self.ledger_client
    }

    /// Returns reference to the Smart contract logic used by the SmartContract
    pub fn logic(&self) -> &Logic {
        &self.offchain_logic
    }
}

#[async_trait]
impl<Logic, Record> SmartContractTrait for SmartContract<Logic, Record>
where
    Logic: SCLogic + Eq + Debug + Send + Sync,
    Logic::Endpoints: Debug,
    Logic::Lookups: Debug,
    Record: LedgerClient<Logic::Datums, Logic::Redeemers> + Send + Sync,
{
    type Endpoint = Logic::Endpoints;
    type Lookup = Logic::Lookups;
    type LookupResponse = Logic::LookupResponses;

    async fn hit_endpoint(&self, endpoint: Logic::Endpoints) -> Result<TxId> {
        tracing::info!("Hitting smart contract endpoint: {:?}", &endpoint);
        let tx_actions = Logic::handle_endpoint(endpoint, &self.ledger_client).await?;
        let tx = tx_actions.to_unbuilt_tx()?;
        match self.ledger_client.issue(tx).await {
            Ok(tx_id) => {
                tracing::info!("Successfully submitted transaction with id: {:?}", &tx_id);
                Ok(tx_id)
            }
            Err(err) => {
                tracing::error!("Failed to submit transaction: {:?}", err);
                Err(err.into())
            }
        }
    }

    async fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse> {
        tracing::info!("Looking up smart contract information: {:?}", &lookup);
        Ok(Logic::lookup(lookup, &self.ledger_client).await?)
    }
}
