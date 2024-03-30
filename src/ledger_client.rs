use thiserror::Error;

/// Test ledger client module
pub mod test_ledger_client;

use async_trait::async_trait;

use crate::{
    output::{
        Output,
        OutputId,
    },
    transaction::{
        TxId,
        UnbuiltTransaction,
    },
    PolicyId,
};
use pallas_addresses::{
    Address,
    Network,
};
use std::error;

/// Interface defining interactions with your specific ledger--AKA the Cardano blockchain. The
/// abstraction allows the concept of fake and mock ledgers to be used in tests and simulations.
// TODO: Having this bound to a specific Datum/Redeemer doesn't really make sense at this scope.
//   It's convenient from the backend's perspective, but it's constricting else-wise.
//   https://github.com/MitchTurner/naumachia/issues/38
// TODO: Add methods for finding specific output by id
//   (getting all is expensive if you just want the output for a specific ID)
#[async_trait]
pub trait LedgerClient<Datum, Redeemer>: Send + Sync {
    /// Get the base address for the signer key owned by instance of the `LedgerClient`
    async fn signer_base_address(&self) -> LedgerClientResult<Address>;

    /// Get list of UTxOs owned by a given address limited by `count`
    async fn outputs_at_address(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>>;

    /// Get complete list of UTxOs owned by a given address
    async fn all_outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>>;

    /// Get the balance for a specific policy at a given address
    async fn balance_at_address(
        &self,
        address: &Address,
        policy: &PolicyId,
    ) -> LedgerClientResult<u64> {
        let bal = self
            .all_outputs_at_address(address)
            .await?
            .iter()
            .fold(0, |acc, o| {
                if let Some(val) = o.values().get(policy) {
                    acc + val
                } else {
                    acc
                }
            });
        Ok(bal)
    }

    /// Issue a transaction to the ledger signed by the signer key owned by the instance of `LedgerClient`
    async fn issue(
        &self,
        tx: UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<TxId>;

    /// Get the network identifier for the ledger
    async fn network(&self) -> LedgerClientResult<Network>;

    /// Get the posix time of the most recent block
    async fn last_block_time_secs(&self) -> LedgerClientResult<i64>;

    /// Get the current time in seconds since the UNIX epoch.
    async fn current_time_secs(&self) -> LedgerClientResult<i64>;
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum LedgerClientError {
    #[error("Couldn't retrieve base address")]
    BaseAddress(Box<dyn error::Error + Send + Sync>),
    #[error("Bad address: {0:?}")]
    BadAddress(Box<dyn error::Error + Send + Sync>),
    #[error("Couldn't convert TxId")]
    BadTxId(Box<dyn error::Error + Send + Sync>),
    #[error("Failed to retrieve outputs at {0:?}: {1:?}.")]
    FailedToRetrieveOutputsAt(Address, Box<dyn error::Error + Send + Sync>),
    #[error("Failed to retrieve UTXO with ID {0:?}.")]
    FailedToRetrieveOutputWithId(OutputId, Box<dyn error::Error + Send + Sync>),
    #[error("Failed to issue transaction: {0:?}")]
    FailedToIssueTx(Box<dyn error::Error + Send + Sync>),
    #[error("There isn't a single utxo big enough for collateral")]
    NoBigEnoughCollateralUTxO,
    #[error("The script input you're trying to spend doesn't have a datum")]
    NoDatumOnScriptInput,
    #[error("The script input you're trying to spend doesn't have a datum")]
    ConfigError(String),
    #[error("While getting current time: {0:?}")]
    CurrentTime(Box<dyn error::Error + Send + Sync>),
    #[error("While setting validity range: {0:?}")]
    ValidityRange(String),
    #[error("While getting last block time: {0:?}")]
    FailedToGetBlockTime(Box<dyn error::Error + Send + Sync>),
}

#[allow(missing_docs)]
pub type LedgerClientResult<T> = Result<T, LedgerClientError>;
