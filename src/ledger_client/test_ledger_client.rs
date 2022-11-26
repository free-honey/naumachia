use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use crate::ledger_client::test_ledger_client::in_memory_storage::InMemoryStorage;
use crate::{
    backend::Backend,
    ledger_client::LedgerClientError::FailedToIssueTx,
    ledger_client::{build_outputs, new_wallet_output},
    ledger_client::{LedgerClient, LedgerClientResult},
    output::Output,
    transaction::TxId,
    values::Values,
    Address, PolicyId, UnbuiltTransaction,
};
use async_trait::async_trait;
use local_persisted_storage::LocalPersistedStorage;
use serde::{de::DeserializeOwned, Serialize};
use tempfile::TempDir;
use thiserror::Error;

pub mod in_memory_storage;
pub mod local_persisted_storage;

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
pub struct TestLedgerClient<Datum, Redeemer, Storage: TestLedgerStorage<Datum>> {
    // pub signer: Address,
    // pub outputs: MutableData<Datum>,
    storage: Storage,
    _datum: PhantomData<Datum>, // This is useless but makes calling it's functions easier
    _redeemer: PhantomData<Redeemer>, // This is useless but makes calling it's functions easier
}

impl<Datum: Clone + Send + Sync + PartialEq, Redeemer>
    TestLedgerClient<Datum, Redeemer, InMemoryStorage<Datum>>
{
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
impl<Datum: Clone + Send + Sync + PartialEq + Serialize + DeserializeOwned, Redeemer>
    TestLedgerClient<Datum, Redeemer, LocalPersistedStorage<Datum>>
{
    pub fn new_local_persisted(signer: Address, starting_amount: u64) -> Self {
        let tmp_dir = TempDir::new().unwrap();
        let storage = LocalPersistedStorage::init(tmp_dir, signer, starting_amount);
        let _ = storage.get_data();
        let client = TestLedgerClient {
            storage,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        client
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
        self.storage.outputs_by_count(address, count).await
    }

    async fn all_outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
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
            self.storage.add_output(&output).await?;
        }
        Ok(TxId::new("Not a real id"))
    }
}
