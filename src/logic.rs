use crate::ledger_client::LedgerClient;
use crate::TxActions;

use async_trait::async_trait;
use error::SCLogicResult;
use std::fmt::Debug;
use std::hash::Hash;

#[allow(missing_docs)]
pub mod error;

/// Interface defining the logic of a smart contract
#[async_trait]
pub trait SCLogic: Send + Sync {
    /// Represents the domain-specific transactions the consumer of a Smart Contract can submit.
    ///
    /// For example a NFT auction Smart Contracts might have:
    /// ```
    /// type Endpoints = AuctionEndpoints;
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
    type Endpoints: Send + Sync;

    /// Represents the domain-specific data the consumer of a Smart Contract can query.
    ///
    /// For example a NFT auction Smart Contracts might have:
    /// ```
    /// type Lookups = AuctionLookups;
    ///
    /// enum AuctionLookups {
    ///    ActiveAuctions,
    ///    AuctionDetails {
    ///       auction_id: String
    ///    },
    /// }
    /// ```
    type Lookups: Send + Sync;

    /// Responses from the Lookup queries
    ///
    /// For example, the appropriate responses for the above lookups might be:
    /// ```
    /// type LookupResponses = AuctionLookupResponses;
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
    type LookupResponses: Send + Sync;
    /// Datum types for scripts used by the Smart Contract.
    /// Because each Smart Contract might use multiple scripts, this can be a `enum` of all the
    /// different Datum types.
    /// ```
    /// type Datums = MultiScriptDatums;
    ///
    /// enum MultiScriptDatums {
    ///     Script1Datum(Script1Datum),
    ///     Script2Datum(Script2Datum),
    /// }
    ///
    /// struct Script1Datum {
    ///     foo: String,
    /// }
    ///
    /// struct Script2Datum {
    ///     bar: u64,
    /// }
    /// ```
    type Datums: Clone + Eq + Debug + Send + Sync;
    /// Redeemer types for scripts used by the Smart Contract
    /// Because each Smart Contract might use multiple scripts, this can be a `enum` of all the
    /// different Redeemer types.
    type Redeemers: Clone + PartialEq + Eq + Hash + Send + Sync;

    /// Method for handling specific endpoint
    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>>;

    /// Method for querying specific data, by lookup
    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponses>;
}
