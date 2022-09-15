use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use crate::ledger_client::new_output;
use crate::{
    backend::Backend,
    ledger_client::minting_to_outputs,
    ledger_client::LedgerClientError::TransactionIssuance,
    ledger_client::{LedgerClient, LedgerClientError, LedgerClientResult},
    output::Output,
    values::Values,
    Address, PolicyId, Transaction,
};
use async_trait::async_trait;
use thiserror::Error;

pub struct TestBackendsBuilder<Datum, Redeemer> {
    signer: Address,
    // TODO: Remove owner
    outputs: Vec<(Address, Output<Datum>)>,
    _redeemer: PhantomData<Redeemer>,
}

impl<
        Datum: Clone + PartialEq + Debug + Send + Sync,
        Redeemer: Clone + Eq + PartialEq + Debug + Hash + Send + Sync,
    > TestBackendsBuilder<Datum, Redeemer>
{
    pub fn new(signer: &Address) -> TestBackendsBuilder<Datum, Redeemer> {
        TestBackendsBuilder {
            signer: signer.clone(),
            outputs: Vec::new(),
            _redeemer: PhantomData::default(),
        }
    }

    pub fn start_output(self, owner: &Address) -> OutputBuilder<Datum, Redeemer> {
        OutputBuilder {
            inner: self,
            owner: owner.clone(),
            values: Values::default(),
        }
    }

    fn add_output(&mut self, address: &Address, output: Output<Datum>) {
        self.outputs.push((address.clone(), output))
    }

    pub fn build(&self) -> Backend<Datum, Redeemer, InMemoryLedgerClient<Datum, Redeemer>> {
        let txo_record = InMemoryLedgerClient {
            signer: self.signer.clone(),
            outputs: Arc::new(Mutex::new(self.outputs.clone())),
            _redeemer: Default::default(),
        };
        Backend {
            _datum: PhantomData::default(),
            _redeemer: PhantomData::default(),
            ledger_client: txo_record,
        }
    }
}

pub struct OutputBuilder<Datum: PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
{
    inner: TestBackendsBuilder<Datum, Redeemer>,
    owner: Address,
    values: Values,
}

impl<Datum, Redeemer> OutputBuilder<Datum, Redeemer>
where
    Datum: Clone + PartialEq + Debug + Send + Sync,
    Redeemer: Clone + Eq + PartialEq + Debug + Hash + Send + Sync,
{
    pub fn with_value(mut self, policy: PolicyId, amount: u64) -> OutputBuilder<Datum, Redeemer> {
        let mut new_total = amount;
        if let Some(total) = self.values.get(&policy) {
            new_total += total;
        }
        self.values.add_one_value(&policy, new_total);
        self
    }

    pub fn finish_output(self) -> TestBackendsBuilder<Datum, Redeemer> {
        let OutputBuilder {
            mut inner,
            owner,
            values,
        } = self;
        let address = owner.clone();
        let tx_hash = Uuid::new_v4().to_string();
        let index = 0;
        let output = Output::new_wallet(tx_hash, index, address, values);
        inner.add_output(&owner, output);
        inner
    }
}

type MutableData<Datum> = Arc<Mutex<Vec<(Address, Output<Datum>)>>>;

#[derive(Debug, Error)]
enum InMemoryLCError {
    #[error("Mutex lock error: {0:?}")]
    Mutex(String),
    #[error("Not enough input value available for outputs")]
    NotEnoughInputs,
}

#[derive(Debug)]
pub struct InMemoryLedgerClient<Datum, Redeemer> {
    pub signer: Address,
    pub outputs: MutableData<Datum>,
    _redeemer: PhantomData<Redeemer>, // This is useless but makes calling it's functions easier
}

#[async_trait]
impl<Datum, Redeemer> LedgerClient<Datum, Redeemer> for InMemoryLedgerClient<Datum, Redeemer>
where
    Datum: Clone + PartialEq + Debug + Send + Sync,
    Redeemer: Clone + Eq + PartialEq + Debug + Hash + Send + Sync,
{
    async fn signer(&self) -> LedgerClientResult<&Address> {
        Ok(&self.signer)
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        let outputs = self
            .outputs
            .lock()
            .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
            .map_err(|e| {
                LedgerClientError::FailedToRetrieveOutputsAt(address.clone(), Box::new(e))
            })?
            .iter()
            .cloned()
            .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
            .collect();
        Ok(outputs)
    }

    async fn issue(&self, tx: Transaction<Datum, Redeemer>) -> LedgerClientResult<()> {
        // TODO: Have all matching Tx Id
        let signer = self.signer().await?;
        let mut combined_inputs = self.outputs_at_address(signer).await?;
        combined_inputs.extend(tx.inputs().clone()); // TODO: Check for dupes

        let total_input_value = combined_inputs
            .iter()
            .fold(Values::default(), |mut acc, utxo| {
                acc.add_values(utxo.values());
                acc
            });
        let total_output_value = tx
            .outputs()
            .iter()
            .fold(Values::default(), |mut acc, utxo| {
                acc.add_values(utxo.values());
                acc
            });
        let maybe_remainder = total_input_value
            .try_subtract(&total_output_value)
            .map_err(|_| InMemoryLCError::NotEnoughInputs)
            .map_err(|e| TransactionIssuance(Box::new(e)))?;

        let mut ledger_utxos = self
            .outputs
            .lock()
            .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
            .map_err(|e| TransactionIssuance(Box::new(e)))?;
        for input in combined_inputs {
            let index = ledger_utxos
                .iter()
                .position(|(_, x)| x == &input)
                .ok_or_else(|| {
                    LedgerClientError::FailedToRetrieveOutputWithId(input.id().clone())
                })?;
            ledger_utxos.remove(index);
        }

        let mut combined_outputs = Vec::new();
        if let Some(remainder) = maybe_remainder {
            combined_outputs.push(new_output(signer, &remainder));
        }

        let minting_outputs = minting_to_outputs::<Datum>(&tx.minting);

        combined_outputs.extend(tx.outputs);
        combined_outputs.extend(minting_outputs);

        for output in combined_outputs {
            ledger_utxos.push((output.owner().clone(), output.clone()))
        }
        Ok(())
    }
}
