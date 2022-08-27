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
    fn signer(&self) -> &Address;
    async fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>>;
    async fn balance_at_address(&self, address: &Address, policy: &PolicyId) -> u64 {
        self.outputs_at_address(address)
            .await
            .iter()
            .fold(0, |acc, o| {
                if let Some(val) = o.values().get(policy) {
                    acc + val
                } else {
                    acc
                }
            })
    }
    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> TxORecordResult<()>; // TODO: Move to other trait
}

#[derive(Debug, Error)]
pub enum LedgerClientError {
    #[error("Failed to retrieve outputs at {0:?}: {1:?}.")]
    FailedToRetrieveOutputsAt(Address, Box<dyn error::Error>),
    #[error("Failed to retrieve UTXO with ID {0:?}.")]
    FailedToRetrieveOutputWithId(OutputId),
}

pub type TxORecordResult<T> = Result<T, LedgerClientError>;
