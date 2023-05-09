use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    backend::Backend,
    ledger_client::{
        test_ledger_client::in_memory_storage::InMemoryStorage, LedgerClient, LedgerClientError,
        LedgerClientResult,
    },
    output::{DatumKind, Output, UnbuiltOutput},
    scripts::context::{
        pub_key_hash_from_address_if_available, CtxDatum, CtxOutput, CtxOutputReference,
    },
    scripts::{
        context::{CtxScriptPurpose, CtxValue, Input, TxContext, ValidRange},
        raw_validator_script::plutus_data::PlutusData,
    },
    transaction::TxId,
    values::Values,
    PolicyId, UnbuiltTransaction,
};
use async_trait::async_trait;
use local_persisted_storage::LocalPersistedStorage;
use pallas_addresses::{Address, Network};
use rand::Rng;
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
        let block_length = 1000;
        let ledger_client = TestLedgerClient::new_in_memory(
            self.signer.clone(),
            self.outputs.clone(),
            block_length,
        );
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
        let tx_hash = arbitrary_tx_id().to_vec();
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
    #[error("Can't read Datum")]
    WrongDatum,
    #[error("Tx too early")]
    TxTooEarly,
    #[error("Tx too late")]
    TxTooLate,
    #[error("Not a valid signer address")]
    InvalidAddress,
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
    async fn set_current_time(&self, posix_time: i64) -> LedgerClientResult<()>;
    async fn get_block_length(&self) -> LedgerClientResult<i64>;
    async fn network(&self) -> LedgerClientResult<Network>;
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
    pub fn new_in_memory(
        signer: Address,
        outputs: Vec<(Address, Output<Datum>)>,
        block_length: i64,
    ) -> Self {
        let storage = InMemoryStorage {
            signer,
            outputs: Arc::new(Mutex::new(outputs)),
            current_posix_time: Arc::new(Mutex::new(0)),
            block_length,
        };
        TestLedgerClient {
            storage,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }
}
impl<T, Datum, Redeemer> TestLedgerClient<Datum, Redeemer, LocalPersistedStorage<T, Datum>>
where
    Datum: Clone + Send + Sync + PartialEq + Into<PlutusData> + TryFrom<PlutusData>,
    T: AsRef<Path> + Send + Sync,
{
    pub fn new_local_persisted(dir: T, signer: &Address, starting_amount: u64) -> Self {
        let signer_name = "Alice";
        let block_length = 1000;
        let storage =
            LocalPersistedStorage::init(dir, signer_name, signer, starting_amount, block_length);
        let _ = storage.get_data();
        TestLedgerClient {
            storage,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }

    pub fn load_local_persisted(dir: T) -> Self {
        let storage = LocalPersistedStorage::load(dir);
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

    pub async fn set_current_time(&self, posix_time: i64) -> LedgerClientResult<()> {
        self.storage.set_current_time(posix_time).await
    }

    pub async fn advance_time_one_block(&self) -> LedgerClientResult<()> {
        let block_length = self.storage.get_block_length().await?;
        let current_time = self.storage.current_time().await?;
        let new_time = block_length + current_time;
        self.storage.set_current_time(new_time).await
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
    async fn signer_base_address(&self) -> LedgerClientResult<Address> {
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

        let signer = self.signer_base_address().await?;

        // TODO: Optimize selection
        let mut combined_inputs = self.all_outputs_at_address(&signer).await?;

        let mut spending_outputs: Vec<Output<_>> = Vec::new();
        for (input, redeemer, script) in tx.script_inputs().iter() {
            if let DatumKind::Typed(datum) = input.datum() {
                if !spending_outputs.contains(input) {
                    let ctx = spend_tx_context(&tx, &signer, input)?;
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
            } else {
                return Err(LedgerClientError::FailedToIssueTx(Box::new(
                    TestLCError::WrongDatum,
                )));
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
            let ctx = mint_tx_context(&tx, &signer, &id)?;
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

        self.advance_time_one_block().await?;

        Ok(TxId::new(&hex::encode(construction_ctx.tx_hash())))
    }

    async fn network(&self) -> LedgerClientResult<Network> {
        self.storage.network().await
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
    tx_hash: Vec<u8>,
    next_index: u64,
}

impl TxIdConstructionCtx {
    pub fn new() -> Self {
        let tx_hash = arbitrary_tx_id().to_vec();
        TxIdConstructionCtx {
            tx_hash,
            next_index: 0,
        }
    }

    pub fn tx_hash(&self) -> Vec<u8> {
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
                let addr = Address::from_bech32(&owner).expect("Already validated");
                new_wallet_output(&addr, &values, construction_ctx)
            }
            UnbuiltOutput::Validator {
                script_address: owner,
                values,
                datum,
            } => {
                let addr = Address::from_bech32(&owner).expect("Already validated");
                new_validator_output(&addr, &values, datum, construction_ctx)
            }
        })
        .collect()
}

fn spend_tx_context<Datum: Into<PlutusData> + Clone, Redeemer>(
    tx: &UnbuiltTransaction<Datum, Redeemer>,
    signer_address: &Address,
    output: &Output<Datum>,
) -> LedgerClientResult<TxContext> {
    let id = output.id();
    let out_ref = CtxOutputReference::new(id.tx_hash().to_vec(), id.index());
    let purpose = CtxScriptPurpose::Spend(out_ref);
    tx_context(tx, signer_address, purpose)
}

fn mint_tx_context<Datum: Into<PlutusData> + Clone, Redeemer>(
    tx: &UnbuiltTransaction<Datum, Redeemer>,
    signer_address: &Address,
    policy_id: &str,
) -> LedgerClientResult<TxContext> {
    let id = hex::decode(policy_id).map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;
    let purpose = CtxScriptPurpose::Mint(id);
    tx_context(tx, signer_address, purpose)
}

fn tx_context<Datum: Into<PlutusData> + Clone, Redeemer>(
    tx: &UnbuiltTransaction<Datum, Redeemer>,
    signer_address: &Address,
    purpose: CtxScriptPurpose,
) -> LedgerClientResult<TxContext> {
    let lower = tx.valid_range.0.map(|n| (n, true));
    let upper = tx.valid_range.1.map(|n| (n, false));

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for (utxo, _, _) in tx.script_inputs.iter() {
        let id = utxo.id();
        let value = CtxValue::from(utxo.values().to_owned());
        let datum = utxo.typed_datum().into();
        let address = utxo.owner();
        let transaction_id = id.tx_hash().to_vec();
        let input = Input {
            transaction_id,
            output_index: id.index(),
            address,
            value,
            datum,
            reference_script: None,
        };
        inputs.push(input);
    }

    for input in &tx.specific_wallet_inputs {
        let id = input.id();
        let transaction_id = id.tx_hash().to_vec();
        let output_index = id.index();
        let address = input.owner();
        let value = CtxValue::from(input.values().to_owned());

        let new_input = Input {
            transaction_id,
            output_index,
            address,
            value,
            datum: CtxDatum::NoDatum,
            reference_script: None,
        };
        inputs.push(new_input)
    }

    for output in tx.unbuilt_outputs.iter() {
        let new_output = match output {
            UnbuiltOutput::Wallet { owner, values } => {
                let address = Address::from_bech32(owner)
                    .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;
                let value = CtxValue::from(values.to_owned());
                CtxOutput {
                    address,
                    value,
                    datum: CtxDatum::NoDatum,
                    reference_script: None,
                }
            }
            UnbuiltOutput::Validator {
                script_address,
                values,
                datum,
            } => {
                let address = Address::from_bech32(script_address)
                    .map_err(|e| LedgerClientError::FailedToIssueTx(Box::new(e)))?;
                let value = CtxValue::from(values.to_owned());
                let datum = CtxDatum::InlineDatum(datum.to_owned().into());
                CtxOutput {
                    address,
                    value,
                    datum,
                    reference_script: None,
                }
            }
        };
        outputs.push(new_output)
    }

    let signer = pub_key_hash_from_address_if_available(signer_address).ok_or(
        LedgerClientError::FailedToIssueTx(Box::new(TestLCError::InvalidAddress)),
    )?;
    let range = ValidRange { lower, upper };

    // TODO: Extra Signatories, and Datums (they are already included in CTX Builder)
    let ctx = TxContext {
        purpose,
        signer,
        range,
        inputs,
        outputs,
        extra_signatories: vec![],
        datums: vec![],
    };
    Ok(ctx)
}

fn arbitrary_tx_id() -> [u8; 32] {
    rand::thread_rng().gen()
}
