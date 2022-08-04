use std::hash::Hash;
use std::marker::{PhantomData, PhantomPinned};
use std::{cell::RefCell, collections::HashMap, fmt::Debug};

use crate::{
    address::{Address, Policy},
    error::Result,
    output::Output,
    smart_contract::{DataSource, TxBuilder, TxIssuer},
    transaction::Action,
    Transaction, UnBuiltTransaction,
};

pub struct FakeBackendsBuilder<Datum, Redeemer> {
    signer: Address,
    outputs: Vec<(Address, Output<Datum>)>,
    _redeemer: PhantomData<Redeemer>,
}

impl<Datum: Clone, Redeemer> FakeBackendsBuilder<Datum, Redeemer> {
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

    pub fn build(&self) -> FakeBackends<Datum, Redeemer> {
        FakeBackends {
            signer: self.signer.clone(),
            outputs: RefCell::new(self.outputs.clone()),
            _redeemer: PhantomData::default(),
        }
    }
}

pub struct OutputBuilder<Datum, Redeemer> {
    inner: FakeBackendsBuilder<Datum, Redeemer>,
    owner: Address,
    values: HashMap<Policy, u64>,
}

impl<Datum: Clone, Redeemer> OutputBuilder<Datum, Redeemer> {
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

#[derive(Debug)]
pub struct FakeBackends<Datum, Redeemer> {
    pub signer: Address,
    // TODO: Might make sense to make this swapable to
    pub outputs: RefCell<Vec<(Address, Output<Datum>)>>,
    pub _redeemer: PhantomData<Redeemer>,
}

impl<Datum: Clone, Redeemer> FakeBackends<Datum, Redeemer> {
    pub fn new(signer: Address) -> Self {
        FakeBackends {
            signer,
            outputs: RefCell::new(vec![]),
            _redeemer: PhantomData::default(),
        }
    }

    pub fn change_signer(&mut self, new_signer: Address) {
        self.signer = new_signer;
    }

    pub fn with_output(&mut self, address: Address, output: Output<Datum>) {
        self.outputs.borrow_mut().push((address, output));
    }

    pub(crate) fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>> {
        self.outputs
            .borrow()
            .clone()
            .into_iter()
            .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
            .collect()
    }

    pub fn balance_at_address(&self, address: &Address, policy: &Policy) -> u64 {
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

    pub fn my_balance(&self, policy: &Policy) -> u64 {
        self.balance_at_address(&self.signer, policy)
    }

    // TODO: Remove allow
    #[allow(clippy::type_complexity)]
    fn handle_actions(
        &self,
        actions: Vec<Action<Datum, Redeemer>>,
    ) -> Result<(Vec<Output<Datum>>, Vec<Output<Datum>>)> {
        let mut min_input_values: HashMap<Policy, u64> = HashMap::new();
        let mut min_output_values: HashMap<Address, RefCell<HashMap<Policy, u64>>> = HashMap::new();
        let mut script_inputs: Vec<Output<Datum>> = Vec::new();
        let mut specific_outputs: Vec<Output<Datum>> = Vec::new();
        for action in actions {
            match action {
                Action::Transfer {
                    amount,
                    recipient,
                    policy,
                } => {
                    // Input
                    add_to_map(&mut min_input_values, policy.clone(), amount);

                    // Output
                    add_amount_to_nested_map(&mut min_output_values, amount, &recipient, &policy);
                }
                Action::Mint {
                    amount,
                    recipient,
                    policy,
                } => {
                    add_amount_to_nested_map(&mut min_output_values, amount, &recipient, &policy);
                }
                Action::InitScript {
                    datum,
                    values,
                    address,
                } => {
                    for (policy, amount) in values.iter() {
                        add_to_map(&mut min_input_values, policy.clone(), *amount);
                    }
                    let output = Output::Validator {
                        owner: address,
                        values,
                        datum,
                    };
                    specific_outputs.push(output);
                }
                Action::RedeemScriptOutput {
                    output,
                    redeemer,
                    script,
                } => {
                    script_inputs.push(output);
                }
            }
        }
        // inputs
        let (mut inputs, remainders) =
            self.select_inputs_for_one(&self.signer, &min_input_values, script_inputs)?;

        // outputs
        remainders.iter().for_each(|(amt, recp, policy)| {
            add_amount_to_nested_map(&mut min_output_values, *amt, recp, policy)
        });

        let out_vecs = nested_map_to_vecs(min_output_values);
        let mut outputs = self.create_outputs_for(out_vecs)?;
        outputs.extend(specific_outputs);

        Ok((inputs, outputs))
    }

    // LOL Super Naive Solution, just select ALL inputs!
    // TODO: Use Random Improve prolly: https://cips.cardano.org/cips/cip2/
    //       but this is _good_enough_ for tests.
    // TODO: Remove allow
    #[allow(clippy::type_complexity)]
    fn select_inputs_for_one(
        &self,
        address: &Address,
        values: &HashMap<Policy, u64>,
        script_inputs: Vec<Output<Datum>>,
    ) -> Result<(Vec<Output<Datum>>, Vec<(u64, Address, Policy)>)> {
        let mut address_values = HashMap::new();
        let mut all_available_outputs = self.outputs_at_address(address);
        all_available_outputs.extend(script_inputs);
        all_available_outputs
            .clone()
            .into_iter()
            .flat_map(|o| o.values().clone().into_iter().collect::<Vec<_>>())
            .for_each(|(policy, amount)| {
                add_to_map(&mut address_values, policy, amount);
            });
        let mut remainders = Vec::new();

        // TODO: REfactor :(
        for (policy, amt) in values.iter() {
            if let Some(available) = address_values.remove(policy) {
                if amt <= &available {
                    let remaining = available - amt;
                    remainders.push((remaining, address.clone(), policy.clone()));
                } else {
                    return Err(format!("Not enough {:?}", policy));
                }
            } else {
                return Err(format!("Not enough {:?}", policy));
            }
        }
        let other_remainders: Vec<_> = address_values
            .into_iter()
            .map(|(policy, amt)| (amt, address.clone(), policy))
            .collect();
        remainders.extend(other_remainders);
        Ok((all_available_outputs, remainders))
    }

    fn create_outputs_for(
        &self,
        values: Vec<(Address, Vec<(Policy, u64)>)>,
    ) -> Result<Vec<Output<Datum>>> {
        let outputs = values
            .into_iter()
            .map(|(owner, val_vec)| {
                let values = val_vec.into_iter().collect();
                Output::wallet(owner, values)
            })
            .collect();
        Ok(outputs)
    }
}

fn add_to_map(h_map: &mut HashMap<Policy, u64>, policy: Policy, amount: u64) {
    let mut new_total = amount;
    if let Some(total) = h_map.get(&policy) {
        new_total += total;
    }
    h_map.insert(policy.clone(), new_total);
}

fn nested_map_to_vecs(
    nested_map: HashMap<Address, RefCell<HashMap<Policy, u64>>>,
) -> Vec<(Address, Vec<(Policy, u64)>)> {
    nested_map
        .into_iter()
        .map(|(addr, h_map)| (addr, h_map.into_inner().into_iter().collect()))
        .collect()
}

fn add_amount_to_nested_map(
    output_map: &mut HashMap<Address, RefCell<HashMap<Policy, u64>>>,
    amount: u64,
    owner: &Address,
    policy: &Policy,
) {
    if let Some(h_map) = output_map.get(owner) {
        let mut inner = h_map.borrow_mut();
        let mut new_total = amount;
        if let Some(total) = inner.get(policy) {
            new_total += total;
        }
        inner.insert(policy.clone(), new_total);
    } else {
        let mut new_map = HashMap::new();
        new_map.insert(policy.clone(), amount);
        output_map.insert(owner.clone(), RefCell::new(new_map));
    }
}

impl<Datum, Redeemer> DataSource for FakeBackends<Datum, Redeemer> {
    fn me(&self) -> &Address {
        &self.signer
    }
}

impl<Datum: Clone, Redeemer: Clone + PartialEq + Eq + Hash> TxBuilder<Datum, Redeemer>
    for FakeBackends<Datum, Redeemer>
{
    /// No Fees, MinAda, or Collateral
    fn build(
        &self,
        unbuilt_tx: UnBuiltTransaction<Datum, Redeemer>,
    ) -> crate::Result<Transaction<Datum, Redeemer>> {
        let UnBuiltTransaction { actions } = unbuilt_tx;
        let (inputs, outputs) = self.handle_actions(actions)?;
        let redeemers = Vec::new();

        Ok(Transaction {
            inputs,
            outputs,
            redeemers,
        })
    }
}

impl<Datum, Redeemer> TxIssuer<Datum, Redeemer> for FakeBackends<Datum, Redeemer>
where
    Datum: Clone + PartialEq + Debug,
    Redeemer: Clone + PartialEq + Eq + Hash,
{
    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> Result<()> {
        let mut my_outputs = self.outputs.borrow_mut();
        for tx_i in tx.inputs() {
            let index = my_outputs
                .iter()
                .position(|(addr, x)| x == tx_i)
                .ok_or(format!("Input: {:?} doesn't exist", &tx_i))?;
            my_outputs.remove(index);
        }

        for tx_o in tx.outputs() {
            my_outputs.push((tx_o.owner().clone(), tx_o.clone()))
        }
        Ok(())
    }
}
