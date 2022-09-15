use crate::{address::Address, output::Output, transaction::Transaction, PolicyId};

use thiserror::Error;

pub mod blockfrost_client;
pub mod in_memory_ledger;
pub mod local_persisted_ledger;
use async_trait::async_trait;

use crate::output::OutputId;
use std::error;

#[async_trait]
pub trait LedgerClient<Datum, Redeemer>: Send + Sync {
    async fn signer(&self) -> LedgerClientResult<&Address>;
    async fn outputs_at_address(&self, address: &Address)
        -> LedgerClientResult<Vec<Output<Datum>>>;
    async fn balance_at_address(
        &self,
        address: &Address,
        policy: &PolicyId,
    ) -> LedgerClientResult<u64> {
        let bal = self
            .outputs_at_address(address)
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
    async fn issue(&self, tx: Transaction<Datum, Redeemer>) -> LedgerClientResult<()>; // TODO: Move to other trait
}

#[derive(Debug, Error)]
pub enum LedgerClientError {
    #[error("Failed to retrieve outputs at {0:?}: {1:?}.")]
    FailedToRetrieveOutputsAt(Address, Box<dyn error::Error + Send>),
    #[error("Failed to retrieve UTXO with ID {0:?}.")]
    FailedToRetrieveOutputWithId(OutputId),
    #[error("Failed to issue transaction: {0:?}")]
    TransactionIssuance(Box<dyn error::Error + Send>),
}

pub type LedgerClientResult<T> = Result<T, LedgerClientError>;
