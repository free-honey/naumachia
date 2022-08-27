use std::sync::{Arc, Mutex};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};
use uuid::Uuid;

use crate::values::Values;
use crate::{
    backend::Backend,
    ledger_client::{LedgerClient, LedgerClientError, TxORecordResult},
    output::Output,
    Address, PolicyId, Transaction,
};
use async_trait::async_trait;

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
            txo_record,
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
    fn signer(&self) -> &Address {
        &self.signer
    }

    async fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>> {
        self.outputs
            .lock()
            .unwrap() // TODO: Unwrap
            .iter()
            .cloned()
            .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
            .collect()
    }

    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> TxORecordResult<()> {
        let mut my_outputs = self.outputs.lock().unwrap(); // TODO: Unwrap
        for tx_i in tx.inputs() {
            let index = my_outputs
                .iter()
                .position(|(_, x)| x == tx_i)
                .ok_or_else(|| {
                    LedgerClientError::FailedToRetrieveOutputWithId(tx_i.id().clone())
                })?;
            my_outputs.remove(index);
        }

        for tx_o in tx.outputs() {
            my_outputs.push((tx_o.owner().clone(), tx_o.clone()))
        }
        Ok(())
    }
}
