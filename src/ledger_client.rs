use thiserror::Error;

pub mod cml_client;
pub mod in_memory_ledger;
pub mod local_persisted_ledger;
use async_trait::async_trait;

use crate::{
    address::Address,
    output::{Output, OutputId, UnbuiltOutput},
    transaction::TxId,
    transaction::UnbuiltTransaction,
    values::Values,
    PolicyId,
};
use std::error;
use uuid::Uuid;

// TODO: Having this bound to a specific Datum/Redeemer doesn't really make sense at this scope.
//   It's convenient from the backend's perspective, but it's constricting else-wise.
//   https://github.com/MitchTurner/naumachia/issues/38
#[async_trait]
pub trait LedgerClient<Datum, Redeemer>: Send + Sync {
    async fn signer(&self) -> LedgerClientResult<Address>;
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
    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId>; // TODO: Move to other trait
}

#[derive(Debug, Error)]
pub enum LedgerClientError {
    #[error("Couldn't retrieve base address")]
    BaseAddress(Box<dyn error::Error + Send>),
    #[error("Failed to retrieve outputs at {0:?}: {1:?}.")]
    FailedToRetrieveOutputsAt(Address, Box<dyn error::Error + Send>),
    #[error("Failed to retrieve UTXO with ID {0:?}.")]
    FailedToRetrieveOutputWithId(OutputId, Box<dyn error::Error + Send>),
    #[error("Failed to issue transaction: {0:?}")]
    FailedToIssueTx(Box<dyn error::Error + Send>),
    #[error("There isn't a single utxo big enough for collateral")]
    NoBigEnoughCollateralUTxO,
    #[error("The script input you're trying to spend doesn't have a datum")]
    NoDatumOnScriptInput,
}

pub type LedgerClientResult<T> = Result<T, LedgerClientError>;

pub(crate) fn new_wallet_output<Datum>(addr: &Address, vals: &Values) -> Output<Datum> {
    // TODO: Fix to not do tx_hash here maybe
    let tx_hash = Uuid::new_v4().to_string();
    let index = 0;
    Output::new_wallet(tx_hash, index, addr.clone(), vals.clone())
}

pub(crate) fn new_validator_output<Datum>(
    addr: &Address,
    vals: &Values,
    datum: Datum,
) -> Output<Datum> {
    // TODO: Fix to not do tx_hash here maybe
    let tx_hash = Uuid::new_v4().to_string();
    let index = 0;
    Output::new_validator(tx_hash, index, addr.clone(), vals.clone(), datum)
}

fn build_outputs<Datum>(unbuilt_outputs: Vec<UnbuiltOutput<Datum>>) -> Vec<Output<Datum>> {
    unbuilt_outputs
        .into_iter()
        .map(|output| match output {
            UnbuiltOutput::Wallet { owner, values } => new_wallet_output(&owner, &values),
            UnbuiltOutput::Validator {
                script_address: owner,
                values,
                datum,
            } => new_validator_output(&owner, &values, datum),
        })
        .collect()
}
