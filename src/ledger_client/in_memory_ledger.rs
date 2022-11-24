use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use crate::ledger_client::{build_outputs, new_wallet_output};
use crate::transaction::TxId;
use crate::{
    backend::Backend,
    ledger_client::LedgerClientError::FailedToIssueTx,
    ledger_client::{LedgerClient, LedgerClientError, LedgerClientResult},
    output::Output,
    values::Values,
    Address, PolicyId, UnbuiltTransaction,
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

    pub fn build_in_memory(
        &self,
    ) -> Backend<Datum, Redeemer, TestLedgerClient<Datum, Redeemer, InMemoryStorage<Datum>>> {
        let ledger_client =
            TestLedgerClient::new_in_memory(self.signer.clone(), self.outputs.clone());
        Backend {
            _datum: PhantomData::default(),
            _redeemer: PhantomData::default(),
            ledger_client,
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
    #[error("The same input is listed twice")]
    DuplicateInput, // TODO: WE don't need this once we dedupe
}

#[async_trait::async_trait]
pub trait TestLedgerStorage<Datum> {
    async fn signer(&self) -> LedgerClientResult<Address>;
    async fn outputs_by_count(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>>;
    async fn all_outputs(&self, address: &Address) -> LedgerClientResult<Vec<Output<Datum>>>;
    async fn remove_output(&self, output: &Output<Datum>) -> LedgerClientResult<()>;
    async fn add_output(&self, output: &Output<Datum>) -> LedgerClientResult<()>;
}

#[derive(Debug)]
pub struct InMemoryStorage<Datum> {
    signer: Address,
    outputs: MutableData<Datum>,
}

#[async_trait::async_trait]
impl<Datum: Send + Sync> TestLedgerStorage<Datum> for InMemoryStorage<Datum> {
    async fn signer(&self) -> LedgerClientResult<Address> {
        todo!()
    }

    async fn outputs_by_count(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        todo!()
    }

    async fn all_outputs(&self, address: &Address) -> LedgerClientResult<Vec<Output<Datum>>> {
        todo!()
    }

    async fn remove_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        todo!()
    }

    async fn add_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        todo!()
    }
}

#[derive(Debug)]
pub struct TestLedgerClient<Datum, Redeemer, Storage: TestLedgerStorage<Datum>> {
    // pub signer: Address,
    // pub outputs: MutableData<Datum>,
    storage: Storage,
    _datum: PhantomData<Datum>, // This is useless but makes calling it's functions easier
    _redeemer: PhantomData<Redeemer>, // This is useless but makes calling it's functions easier
}

impl<Datum: Send + Sync, Redeemer> TestLedgerClient<Datum, Redeemer, InMemoryStorage<Datum>> {
    pub fn new_in_memory(signer: Address, outputs: Vec<(Address, Output<Datum>)>) -> Self {
        let storage = InMemoryStorage {
            signer,
            outputs: Arc::new(Mutex::new(outputs)),
        };
        TestLedgerClient {
            storage,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }
}

#[async_trait]
impl<Datum, Redeemer, Storage> LedgerClient<Datum, Redeemer>
    for TestLedgerClient<Datum, Redeemer, Storage>
where
    Datum: Clone + PartialEq + Debug + Send + Sync,
    Redeemer: Clone + Eq + PartialEq + Debug + Hash + Send + Sync,
    Storage: TestLedgerStorage<Datum> + Send + Sync,
{
    async fn signer(&self) -> LedgerClientResult<Address> {
        self.storage.signer().await
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        // let outputs = self
        //     .outputs
        //     .lock()
        //     .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
        //     .map_err(|e| {
        //         LedgerClientError::FailedToRetrieveOutputsAt(address.clone(), Box::new(e))
        //     })?
        //     .iter()
        //     .cloned()
        //     .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
        //     .take(count)
        //     .collect();
        // Ok(outputs)
        self.storage.outputs_by_count(address, count).await
    }

    async fn all_outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        // let outputs = self
        //     .outputs
        //     .lock()
        //     .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
        //     .map_err(|e| {
        //         LedgerClientError::FailedToRetrieveOutputsAt(address.clone(), Box::new(e))
        //     })?
        //     .iter()
        //     .cloned()
        //     .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
        //     .collect();
        // Ok(outputs)
        self.storage.all_outputs(address).await
    }

    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId> {
        // TODO: Have all matching Tx Id
        let signer = self.signer().await?;
        let mut combined_inputs = self.all_outputs_at_address(&signer).await?;
        tx.script_inputs()
            .iter()
            .for_each(|(input, _, _)| combined_inputs.push(input.clone())); // TODO: Check for dupes

        let mut total_input_value =
            combined_inputs
                .iter()
                .fold(Values::default(), |mut acc, utxo| {
                    acc.add_values(utxo.values());
                    acc
                });

        total_input_value.add_values(&tx.minting);

        let total_output_value =
            tx.unbuilt_outputs()
                .iter()
                .fold(Values::default(), |mut acc, utxo| {
                    acc.add_values(utxo.values());
                    acc
                });
        let maybe_remainder = total_input_value
            .try_subtract(&total_output_value)
            .map_err(|_| InMemoryLCError::NotEnoughInputs)
            .map_err(|e| FailedToIssueTx(Box::new(e)))?;

        // let mut ledger_utxos = self
        //     .outputs
        //     .lock()
        //     .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
        //     .map_err(|e| FailedToIssueTx(Box::new(e)))?;
        // for input in combined_inputs {
        //     let index = ledger_utxos
        //         .iter()
        //         .position(|(_, x)| x == &input)
        //         .ok_or_else(|| {
        //             LedgerClientError::FailedToRetrieveOutputWithId(
        //                 input.id().clone(),
        //                 Box::new(InMemoryLCError::DuplicateInput),
        //             )
        //         })?;
        //     ledger_utxos.remove(index);
        // }

        for input in combined_inputs {
            self.storage.remove_output(&input).await?;
        }

        let mut combined_outputs = Vec::new();
        if let Some(remainder) = maybe_remainder {
            combined_outputs.push(new_wallet_output(&signer, &remainder));
        }

        let built_outputs = build_outputs(tx.unbuilt_outputs);

        combined_outputs.extend(built_outputs);

        for output in combined_outputs {
            // ledger_utxos.push((output.owner().clone(), output.clone()))
            self.storage.add_output(&output).await?;
        }
        Ok(TxId::new("Not a real id"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use crate::ledger_client::local_persisted_ledger::starting_output;
    use crate::output::UnbuiltOutput;
    use tempfile::TempDir;

    #[tokio::test]
    async fn outputs_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        // let record = TestLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount).unwrap();
        let output = starting_output::<()>(&signer, starting_amount);
        let outputs = vec![(signer.clone(), output)];
        let record: TestLedgerClient<(), (), _> =
            TestLedgerClient::new_in_memory(signer.clone(), outputs);
        let mut outputs = record.all_outputs_at_address(&signer).await.unwrap();
        assert_eq!(outputs.len(), 1);
        let first_output = outputs.pop().unwrap();
        let expected = starting_amount;
        let actual = first_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(expected, actual);
    }

    // #[tokio::test]
    // async fn balance_at_address() {
    //     let tmp_dir = TempDir::new().unwrap();
    //     let path = tmp_dir.path().join("data");
    //     let signer = Address::new("alice");
    //     let starting_amount = 10_000_000;
    //     let record =
    //         TestLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount).unwrap();
    //     let expected = starting_amount;
    //     let actual = record
    //         .balance_at_address(&signer, &PolicyId::ADA)
    //         .await
    //         .unwrap();
    //     assert_eq!(expected, actual);
    // }
    //
    // #[tokio::test]
    // async fn issue() {
    //     let tmp_dir = TempDir::new().unwrap();
    //     let path = tmp_dir.path().join("data");
    //     let signer = Address::new("alice");
    //     let starting_amount = 10_000_000;
    //     let record =
    //         TestLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount).unwrap();
    //     let first_output = record
    //         .all_outputs_at_address(&signer)
    //         .await
    //         .unwrap()
    //         .pop()
    //         .unwrap();
    //     let owner = Address::new("bob");
    //     let new_output = UnbuiltOutput::new_wallet(owner.clone(), first_output.values().clone());
    //     let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
    //         script_inputs: vec![],
    //         unbuilt_outputs: vec![new_output],
    //         minting: Default::default(),
    //         policies: Default::default(),
    //     };
    //     record.issue(tx).await.unwrap();
    //     let expected_bob = starting_amount;
    //     let actual_bob = record
    //         .balance_at_address(&owner, &PolicyId::ADA)
    //         .await
    //         .unwrap();
    //     assert_eq!(expected_bob, actual_bob);
    //
    //     let expected_alice = 0;
    //     let actual_alice = record
    //         .balance_at_address(&signer, &PolicyId::ADA)
    //         .await
    //         .unwrap();
    //     assert_eq!(expected_alice, actual_alice)
    // }
}
