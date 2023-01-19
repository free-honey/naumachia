use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use crate::output::UnbuiltOutput;
use crate::scripts::context::{CtxValue, Input, TxContext, ValidRange};
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::{
    backend::Backend,
    ledger_client::{
        test_ledger_client::in_memory_storage::InMemoryStorage, LedgerClient, LedgerClientError,
        LedgerClientResult,
    },
    output::Output,
    transaction::TxId,
    values::Values,
    Address, PolicyId, UnbuiltTransaction,
};
use async_trait::async_trait;
use local_persisted_storage::LocalPersistedStorage;
use rand::Rng;
use serde::{de::DeserializeOwned, Serialize};
use tempfile::TempDir;
use thiserror::Error;

pub mod in_memory_storage;
pub mod local_persisted_storage;

#[cfg(test)]
mod tests;

pub struct TestBackendsBuilder<Datum, Redeemer> {
    signer: Address,
    // TODO: Remove owner
    outputs: Vec<(Address, Output<Datum>)>,
    _redeemer: PhantomData<Redeemer>,
}

impl<Datum, Redeemer> TestBackendsBuilder<Datum, Redeemer>
where
    Datum: Clone + PartialEq + Debug + Send + Sync + Into<PlutusData>,
    Redeemer: Clone + Eq + PartialEq + Debug + Hash + Send + Sync,
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
            datum: None,
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
    datum: Option<Datum>,
}

impl<Datum, Redeemer> OutputBuilder<Datum, Redeemer>
where
    Datum: Clone + PartialEq + Debug + Send + Sync + Into<PlutusData>,
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

    pub fn with_datum(mut self, datum: Datum) -> OutputBuilder<Datum, Redeemer> {
        self.datum = Some(datum);
        self
    }

    pub fn finish_output(self) -> TestBackendsBuilder<Datum, Redeemer> {
        let OutputBuilder {
            mut inner,
            owner,
            values,
            datum,
        } = self;
        let address = owner.clone();
        let tx_hash = arbitrary_tx_id();
        let index = 0;
        let output = if let Some(datum) = datum {
            Output::new_validator(tx_hash, index, address, values, datum)
        } else {
            Output::new_wallet(tx_hash, index, address, values)
        };
        inner.add_output(&owner, output);
        inner
    }
}

#[derive(Debug, Error)]
enum TestLCError {
    #[error("Mutex lock error: {0:?}")]
    Mutex(String),
    #[error("Not enough input value available for outputs")]
    NotEnoughInputs,
    #[error("The same input is listed twice")]
    DuplicateInput,
    #[error("Tx too early")]
    TxTooEarly,
    #[error("Tx too late")]
    TxTooLate,
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
    async fn current_time(&self) -> LedgerClientResult<i64>;
    async fn set_current_time(&mut self, posix_time: i64) -> LedgerClientResult<()>;
}

#[derive(Debug)]
pub struct TestLedgerClient<Datum, Redeemer, Storage: TestLedgerStorage<Datum>> {
    storage: Storage,
    _datum: PhantomData<Datum>, // This is useless but makes calling it's functions easier
    _redeemer: PhantomData<Redeemer>, // This is useless but makes calling it's functions easier
}

impl<Datum, Redeemer> TestLedgerClient<Datum, Redeemer, InMemoryStorage<Datum>>
where
    Datum: Clone + Send + Sync + PartialEq,
{
    pub fn new_in_memory(signer: Address, outputs: Vec<(Address, Output<Datum>)>) -> Self {
        let storage = InMemoryStorage {
            signer,
            outputs: Arc::new(Mutex::new(outputs)),
            current_posix_time: 0,
        };
        TestLedgerClient {
            storage,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }
}
impl<Datum, Redeemer> TestLedgerClient<Datum, Redeemer, LocalPersistedStorage<Datum>>
where
    Datum: Clone + Send + Sync + PartialEq + Serialize + DeserializeOwned,
{
    pub fn new_local_persisted(signer: Address, starting_amount: u64) -> Self {
        let tmp_dir = TempDir::new().unwrap();
        let storage = LocalPersistedStorage::init(tmp_dir, signer, starting_amount);
        let _ = storage.get_data();
        TestLedgerClient {
            storage,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }
}

impl<Datum, Redeemer, Storage> TestLedgerClient<Datum, Redeemer, Storage>
where
    Datum: Clone + Send + Sync + PartialEq,
    Storage: TestLedgerStorage<Datum> + Send + Sync,
{
    pub async fn current_time(&self) -> LedgerClientResult<i64> {
        self.storage.current_time().await
    }

    pub async fn set_current_time(&mut self, posix_time: i64) -> LedgerClientResult<()> {
        self.storage.set_current_time(posix_time).await
    }
}

#[async_trait]
impl<Datum, Redeemer, Storage> LedgerClient<Datum, Redeemer>
    for TestLedgerClient<Datum, Redeemer, Storage>
where
    Datum: Clone + PartialEq + Debug + Send + Sync + Into<PlutusData>,
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
        // Setup
        let valid_range = tx.valid_range;
        let current_time = self.current_time().await?;
        check_time_valid(valid_range, current_time)
            .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;

        let signer = self.signer().await?;

        // TODO: Optimize selection
        let mut combined_inputs = self.all_outputs_at_address(&signer).await?;

        let mut spending_outputs: Vec<Output<_>> = Vec::new();
        for (input, redeemer, script) in tx.script_inputs().iter() {
            if let Some(datum) = input.datum() {
                if !spending_outputs.contains(input) {
                    let ctx = tx_context(&tx, &signer)?;
                    // TODO: Check that the output is at the script address
                    //  https://github.com/MitchTurner/naumachia/issues/86
                    script
                        .execute(datum.to_owned(), redeemer.to_owned(), ctx)
                        .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;
                    combined_inputs.push(input.clone());
                    spending_outputs.push(input.clone());
                } else {
                    return Err(LedgerClientError::FailedToIssueTx(Box::new(
                        TestLCError::DuplicateInput,
                    )));
                }
            }
        }

        let mut total_input_value =
            combined_inputs
                .iter()
                .fold(Values::default(), |mut acc, utxo| {
                    acc.add_values(utxo.values());
                    acc
                });

        let mut construction_ctx = TxIdConstructionCtx::new();

        let mut minted_value = Values::default();

        for (amount, asset_name, redeemer, policy) in tx.minting.iter() {
            let id = policy
                .id()
                .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;
            let policy_id = PolicyId::native_token(&id, asset_name);
            let ctx = tx_context(&tx, &signer)?;
            policy
                .execute(redeemer.to_owned(), ctx)
                .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;
            minted_value.add_one_value(&policy_id, *amount);
        }

        total_input_value.add_values(&minted_value);

        let total_output_value =
            tx.unbuilt_outputs()
                .iter()
                .fold(Values::default(), |mut acc, utxo| {
                    acc.add_values(utxo.values());
                    acc
                });
        let maybe_remainder = total_input_value
            .try_subtract(&total_output_value)
            .map_err(|_| TestLCError::NotEnoughInputs)
            .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;

        for input in combined_inputs {
            self.storage.remove_output(&input).await?;
        }

        let mut combined_outputs = Vec::new();
        if let Some(remainder) = maybe_remainder {
            combined_outputs.push(new_wallet_output(
                &signer,
                &remainder,
                &mut construction_ctx,
            ));
        }

        let built_outputs = build_outputs(tx.unbuilt_outputs, &mut construction_ctx);

        combined_outputs.extend(built_outputs);

        for output in combined_outputs {
            self.storage.add_output(&output).await?;
        }

        Ok(TxId::new("Not a real id"))
    }
}

fn check_time_valid(
    valid_range: (Option<i64>, Option<i64>),
    current_time: i64,
) -> Result<(), TestLCError> {
    if let Some(lower) = valid_range.0 {
        if current_time < lower {
            return Err(TestLCError::TxTooEarly);
        }
    } else if let Some(upper) = valid_range.1 {
        if current_time >= upper {
            return Err(TestLCError::TxTooLate);
        }
    }
    Ok(())
}

struct TxIdConstructionCtx {
    tx_hash: String,
    next_index: u64,
}

impl TxIdConstructionCtx {
    pub fn new() -> Self {
        let tx_hash = arbitrary_tx_id();
        TxIdConstructionCtx {
            tx_hash,
            next_index: 0,
        }
    }

    pub fn tx_hash(&self) -> String {
        self.tx_hash.clone()
    }

    pub fn next_index(&mut self) -> u64 {
        let next_index = self.next_index;
        self.next_index += 1;
        next_index
    }
}

fn new_wallet_output<Datum>(
    addr: &Address,
    vals: &Values,
    construction_ctx: &mut TxIdConstructionCtx,
) -> Output<Datum> {
    let tx_hash = construction_ctx.tx_hash();
    let index = construction_ctx.next_index();
    Output::new_wallet(tx_hash, index, addr.clone(), vals.clone())
}

fn new_validator_output<Datum>(
    addr: &Address,
    vals: &Values,
    datum: Datum,
    construction_ctx: &mut TxIdConstructionCtx,
) -> Output<Datum> {
    let tx_hash = construction_ctx.tx_hash();
    let index = construction_ctx.next_index();
    Output::new_validator(tx_hash, index, addr.clone(), vals.clone(), datum)
}

fn build_outputs<Datum>(
    unbuilt_outputs: Vec<UnbuiltOutput<Datum>>,
    construction_ctx: &mut TxIdConstructionCtx,
) -> Vec<Output<Datum>> {
    unbuilt_outputs
        .into_iter()
        .map(|output| match output {
            UnbuiltOutput::Wallet { owner, values } => {
                new_wallet_output(&owner, &values, construction_ctx)
            }
            UnbuiltOutput::Validator {
                script_address: owner,
                values,
                datum,
            } => new_validator_output(&owner, &values, datum, construction_ctx),
        })
        .collect()
}

fn tx_context<Datum: Into<PlutusData> + Clone, Redeemer>(
    tx: &UnbuiltTransaction<Datum, Redeemer>,
    signer: &Address,
) -> LedgerClientResult<TxContext> {
    let lower = tx.valid_range.0.map(|n| (n, true));
    let upper = tx.valid_range.1.map(|n| (n, false));

    let mut inputs = Vec::new();
    for (utxo, _, _) in tx.script_inputs.iter() {
        let id = utxo.id();
        let value = CtxValue::from(utxo.values().to_owned());
        let datum = utxo.datum().map(|d| d.to_owned()).into();
        let address = utxo
            .owner()
            .bytes()
            .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;
        let input = Input {
            transaction_id: id.tx_hash().to_string(),
            output_index: id.index(),
            address,
            value,
            datum,
            reference_script: None,
        };
        inputs.push(input);
    }

    let range = ValidRange { lower, upper };
    let ctx = TxContext {
        signer: signer.clone(),
        range,
        inputs,
    };
    Ok(ctx)
}

fn arbitrary_tx_id() -> String {
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    hex::encode(random_bytes)
}
