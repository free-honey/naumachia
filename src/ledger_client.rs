use crate::{output::Output, transaction::Transaction, PolicyId};

use thiserror::Error;

pub mod blockfrost_client;
pub mod in_memory_ledger;
pub mod local_persisted_ledger;

pub mod fake_address;

use crate::address::ValidAddress;
use std::error;

pub trait LedgerClient<Datum, Redeemer> {
    type Address: ValidAddress;

    fn signer(&self) -> &Self::Address;
    fn outputs_at_address(&self, address: &Self::Address) -> Vec<Output<Self::Address, Datum>>;
    fn balance_at_address(&self, address: &Self::Address, policy: &PolicyId) -> u64 {
        self.outputs_at_address(address).iter().fold(0, |acc, o| {
            if let Some(val) = o.values().get(policy) {
                acc + val
            } else {
                acc
            }
        })
    }
    fn issue(&self, tx: Transaction<Self::Address, Datum, Redeemer>) -> TxORecordResult<()>; // TODO: Move to other trait
}

#[derive(Debug, Error)]
pub enum LedgerClientError {
    #[error("Failed to retrieve outputs at {0:?}: {1:?}.")]
    FailedToRetrieveOutputsAt(String, Box<dyn error::Error>),
    #[error("Failed to retrieve UTXO with ID {0:?}.")]
    FailedToRetrieveOutputWithId(String),
}

pub type TxORecordResult<T> = Result<T, LedgerClientError>;
