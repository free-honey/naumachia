use std::{cell::RefCell, fmt::Debug, hash::Hash, marker::PhantomData};
use uuid::Uuid;

use crate::ledger_client::fake_address::FakeAddress;
use crate::values::Values;
use crate::{
    backend::Backend,
    ledger_client::{LedgerClient, LedgerClientError, TxORecordResult},
    output::Output,
    PolicyId, Transaction,
};

pub struct TestBackendsBuilder<Datum, Redeemer> {
    signer: FakeAddress,
    // TODO: Remove owner
    outputs: Vec<(FakeAddress, Output<FakeAddress, Datum>)>,
    _redeemer: PhantomData<Redeemer>,
}

impl<Datum: Clone + PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
    TestBackendsBuilder<Datum, Redeemer>
{
    pub fn new(signer: &FakeAddress) -> TestBackendsBuilder<Datum, Redeemer> {
        TestBackendsBuilder {
            signer: signer.clone(),
            outputs: Vec::new(),
            _redeemer: PhantomData::default(),
        }
    }

    pub fn start_output(self, owner: &FakeAddress) -> OutputBuilder<Datum, Redeemer> {
        OutputBuilder {
            inner: self,
            owner: owner.clone(),
            values: Values::default(),
        }
    }

    fn add_output(&mut self, address: &FakeAddress, output: Output<FakeAddress, Datum>) {
        self.outputs.push((address.clone(), output))
    }

    pub fn build(
        &self,
    ) -> Backend<FakeAddress, Datum, Redeemer, InMemoryLedgerClient<Datum, Redeemer>> {
        let txo_record = InMemoryLedgerClient {
            signer: self.signer.clone(),
            outputs: RefCell::new(self.outputs.clone()),
            _redeemer: Default::default(),
        };
        Backend {
            _address: PhantomData::default(),
            _datum: PhantomData::default(),
            _redeemer: PhantomData::default(),
            ledger_client: txo_record,
        }
    }
}

pub struct OutputBuilder<Datum: PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
{
    inner: TestBackendsBuilder<Datum, Redeemer>,
    owner: FakeAddress,
    values: Values,
}

impl<Datum: Clone + PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
    OutputBuilder<Datum, Redeemer>
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
        let id = Uuid::new_v4().to_string();
        let output = Output::new_wallet(id, address, values);
        inner.add_output(&owner, output);
        inner
    }
}

#[derive(Debug)]
pub struct InMemoryLedgerClient<Datum, Redeemer> {
    pub signer: FakeAddress,
    pub outputs: RefCell<Vec<(FakeAddress, Output<FakeAddress, Datum>)>>,
    _redeemer: PhantomData<Redeemer>, // This is useless but makes calling it's functions easier
}

impl<Datum: Clone + PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
    LedgerClient<Datum, Redeemer> for InMemoryLedgerClient<Datum, Redeemer>
{
    type Address = FakeAddress;

    fn signer(&self) -> &FakeAddress {
        &self.signer
    }

    fn outputs_at_address(&self, address: &FakeAddress) -> Vec<Output<FakeAddress, Datum>> {
        self.outputs
            .borrow()
            .clone()
            .into_iter()
            .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
            .collect()
    }

    fn issue(&self, tx: Transaction<FakeAddress, Datum, Redeemer>) -> TxORecordResult<()> {
        let mut my_outputs = self.outputs.borrow_mut();
        for tx_i in tx.inputs() {
            let index = my_outputs
                .iter()
                .position(|(_, x)| x == tx_i)
                .ok_or_else(|| {
                    LedgerClientError::FailedToRetrieveOutputWithId(tx_i.id().to_string())
                })?;
            my_outputs.remove(index);
        }

        for tx_o in tx.outputs() {
            my_outputs.push((tx_o.owner().clone(), tx_o.clone()))
        }
        Ok(())
    }
}
