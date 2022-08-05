use crate::backend::{can_spend_inputs, Backend, TxORecord};
use crate::error::Result;
use crate::output::Output;
use crate::{Address, Policy, Transaction};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct FakeBackendsBuilder<Datum, Redeemer> {
    signer: Address,
    outputs: Vec<(Address, Output<Datum>)>,
    _redeemer: PhantomData<Redeemer>,
}

impl<Datum: Clone + PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
    FakeBackendsBuilder<Datum, Redeemer>
{
    pub fn new(signer: Address) -> FakeBackendsBuilder<Datum, Redeemer> {
        FakeBackendsBuilder {
            signer,
            outputs: Vec::new(),
            _redeemer: PhantomData::default(),
        }
    }

    pub fn start_output(self, owner: Address) -> OutputBuilder<Datum, Redeemer> {
        OutputBuilder {
            inner: self,
            owner,
            values: HashMap::new(),
        }
    }

    fn add_output(&mut self, address: Address, output: Output<Datum>) {
        self.outputs.push((address, output))
    }

    pub fn build(&self) -> Backend<Datum, Redeemer, FakeRecord<Datum>> {
        let txo_record = FakeRecord {
            signer: self.signer.clone(),
            outputs: RefCell::new(self.outputs.clone()),
        };
        Backend {
            _datum: PhantomData::default(),
            _redeemer: PhantomData::default(),
            txo_record,
        }
    }
}

pub struct FakeRecord<Datum> {
    pub signer: Address,
    pub outputs: RefCell<Vec<(Address, Output<Datum>)>>,
}

impl<Datum: Clone + PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
    TxORecord<Datum, Redeemer> for FakeRecord<Datum>
{
    fn signer(&self) -> &Address {
        &self.signer
    }

    fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>> {
        self.outputs
            .borrow()
            .clone()
            .into_iter()
            .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
            .collect()
    }

    fn balance_at_address(&self, address: &Address, policy: &Policy) -> u64 {
        self.outputs
            .borrow()
            .iter()
            .filter_map(|(a, o)| if a == address { Some(o) } else { None }) // My outputs
            .fold(0, |acc, o| {
                if let Some(val) = o.values().get(policy) {
                    acc + val
                } else {
                    acc
                }
            }) // Sum up policy values
    }

    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> Result<()> {
        can_spend_inputs(&tx, self.signer.clone())?;
        let mut my_outputs = self.outputs.borrow_mut();
        for tx_i in tx.inputs() {
            let index = my_outputs
                .iter()
                .position(|(_, x)| x == tx_i)
                .ok_or(format!("Input: {:?} doesn't exist", &tx_i))?;
            my_outputs.remove(index);
        }

        for tx_o in tx.outputs() {
            my_outputs.push((tx_o.owner().clone(), tx_o.clone()))
        }
        Ok(())
    }
}

pub struct OutputBuilder<Datum: PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
{
    inner: FakeBackendsBuilder<Datum, Redeemer>,
    owner: Address,
    values: HashMap<Policy, u64>,
}

impl<Datum: Clone + PartialEq + Debug, Redeemer: Clone + Eq + PartialEq + Debug + Hash>
    OutputBuilder<Datum, Redeemer>
{
    pub fn with_value(mut self, policy: Policy, amount: u64) -> OutputBuilder<Datum, Redeemer> {
        let mut new_total = amount;
        if let Some(total) = self.values.get(&policy) {
            new_total += total;
        }
        self.values.insert(policy, new_total);
        self
    }

    pub fn finish_output(self) -> FakeBackendsBuilder<Datum, Redeemer> {
        let OutputBuilder {
            mut inner,
            owner,
            values,
        } = self;
        let address = owner.clone();
        let output = Output::wallet(address, values);
        inner.add_output(owner, output);
        inner
    }
}
