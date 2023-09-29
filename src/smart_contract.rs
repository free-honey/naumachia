use async_trait::async_trait;
use std::fmt::Debug;

use crate::{error::Result, ledger_client::LedgerClient, logic::SCLogic};

/// Interface defining how to interact with your smart contract
#[async_trait]
pub trait SmartContractTrait {
    /// Represents the domain-specific transactions the consumer of a Smart Contract can submit.
    ///
    /// For example a NFT auction Smart Contracts might have:
    /// ```
    /// type Endpoint = AuctionEndpoints;
    ///
    /// enum AuctionEndpoints {
    ///     StartAuction {
    ///         nft_id: String,
    ///         start_price: u64,
    ///         auction_end: u64,
    ///     },
    ///     Bid {
    ///         amount: u64,
    ///         auction_id: String,
    ///     },
    ///     ClaimWinnings {
    ///         auction_id: String,
    ///     },
    ///     ClaimNFT {
    ///         auction_id: String
    ///     }
    /// }
    ///
    type Endpoint;

    /// Represents the domain-specific data the consumer of a Smart Contract can query.
    ///
    /// For example a NFT auction Smart Contracts might have:
    /// ```
    /// type Lookup = AuctionLookups;
    ///
    /// enum AuctionLookups {
    ///    ActiveAuctions,
    ///    AuctionDetails {
    ///       auction_id: String
    ///    },
    /// }
    /// ```
    type Lookup;

    /// Responses from the Lookup queries
    ///
    /// For example, the appropriate responses for the above lookups might be:
    /// ```
    /// type LookupResponse = AuctionLookupResponses;
    ///
    /// enum AuctionLookupResponses {
    ///     ActiveAuctions(Vec<String>),
    ///     AuctionDetails {
    ///        nft_id: String,
    ///        current_bid: u64,
    ///        current_bidder: String,
    ///        auction_end: u64,
    ///     },
    /// }
    type LookupResponse;

    /// Method for hitting specific endpoint
    async fn hit_endpoint(&self, endpoint: Self::Endpoint) -> Result<()>;
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
