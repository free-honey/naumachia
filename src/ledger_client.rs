use crate::{address::Address, output::Output, transaction::Transaction, PolicyId};
use std::collections::HashMap;

use thiserror::Error;

pub mod cml_client;
pub mod in_memory_ledger;
pub mod local_persisted_ledger;
use async_trait::async_trait;

use crate::output::OutputId;
use crate::values::Values;
use std::error;
use uuid::Uuid;

// TODO: Having this bound to a specific Datum/Redeemer doesn't really make sense at this scope.
//   It's convenient from the backend's perspective, but it's constricting else-wise.
//   https://github.com/MitchTurner/naumachia/issues/38
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
    FailedToRetrieveOutputWithId(OutputId, Box<dyn error::Error + Send>),
    #[error("Failed to issue transaction: {0:?}")]
    TransactionIssuance(Box<dyn error::Error + Send>),
}

pub type LedgerClientResult<T> = Result<T, LedgerClientError>;

pub(crate) fn minting_to_outputs<Datum>(minting: &HashMap<Address, Values>) -> Vec<Output<Datum>> {
    minting
        .iter()
        .map(|(addr, vals)| new_output(addr, vals))
        .collect()
}

pub(crate) fn new_output<Datum>(addr: &Address, vals: &Values) -> Output<Datum> {
    // TODO: Fix to not do tx_hash here maybe
    let tx_hash = Uuid::new_v4().to_string();
    let index = 0;
    Output::new_wallet(tx_hash, index, addr.clone(), vals.clone())
}
