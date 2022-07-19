use crate::address::{Address, Policy};
use crate::transaction::Action;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::error::Result;
use crate::output::Output;
use crate::smart_contract::{DataSource, TxBuilder, TxIssuer};
use crate::{Transaction, UnBuiltTransaction};

#[derive(Debug)]
pub struct FakeBackends {
    pub me: Address,
    // TODO: Might make sense to make this swapable to
    pub outputs: RefCell<Vec<(Address, Output)>>,
}

impl FakeBackends {
    pub fn new(me: Address) -> Self {
        FakeBackends {
            me,
            outputs: RefCell::new(vec![]),
        }
    }

    pub fn with_output(&mut self, address: Address, output: Output) {
        self.outputs.borrow_mut().push((address, output));
    }

    fn outputs_at_address(&self, address: &Address) -> Vec<Output> {
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
        self.balance_at_address(&self.me, policy)
    }

    fn handle_actions(&self, actions: Vec<Action>) -> Result<(Vec<Output>, Vec<Output>)> {
        let mut min_input_values: HashMap<Address, RefCell<HashMap<Policy, u64>>> = HashMap::new();
        let mut min_output_values: HashMap<Address, RefCell<HashMap<Policy, u64>>> = HashMap::new();
        for action in actions {
            match action {
                Action::Transfer {
                    amount,
                    recipient,
                    policy,
                } => {
                    // Input
                    add_amount_to_nested_map(&mut min_input_values, amount, &self.me, &policy);

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
            }
        }
        // inputs
        let in_vecs = nested_map_to_vecs(min_input_values);
        let (inputs, remainders) = self.select_inputs_for_all(in_vecs)?;

        // outputs
        remainders.iter().for_each(|(amt, recp, policy)| {
            add_amount_to_nested_map(&mut min_output_values, *amt, recp, policy)
        });

        let out_vecs = nested_map_to_vecs(min_output_values);
        let outputs = self.create_outputs_for(out_vecs)?;

        Ok((inputs, outputs))
    }

    fn select_inputs_for_all(
        &self,
        values: Vec<(Address, Vec<(Policy, u64)>)>,
    ) -> Result<(Vec<Output>, Vec<(u64, Address, Policy)>)> {
        let mut total_inputs = Vec::new();
        let mut total_remainders = Vec::new();
        for (owner, values) in values {
            let (outputs, remainders) = self.select_inputs_for_one(&owner, &values)?;
            total_inputs.extend(outputs);
            total_remainders.extend(remainders)
        }
        Ok((total_inputs, total_remainders))
    }

    // LOL Super Naive Solution, just select ALL inputs!
    // TODO: Use Random Improve prolly: https://cips.cardano.org/cips/cip2/
    //       but this is _good_enough_ for tests.
    fn select_inputs_for_one(
        &self,
        address: &Address,
        values: &Vec<(Policy, u64)>,
    ) -> Result<(Vec<Output>, Vec<(u64, Address, Policy)>)> {
        let mut address_values = HashMap::new();
        let all_address_outputs = self.outputs_at_address(address);
        all_address_outputs
            .clone()
            .into_iter()
            .flat_map(|o| o.values().clone().into_iter().collect::<Vec<_>>())
            .for_each(|(policy, amt)| {
                let mut new_total = amt;
                if let Some(total) = address_values.get(&policy) {
                    new_total += total;
                }
                address_values.insert(policy, new_total);
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
            .map(|(policy, amt)| (amt, address.clone(), policy.clone()))
            .collect();
        remainders.extend(other_remainders);
        Ok((all_address_outputs, remainders))
    }

    fn create_outputs_for(
        &self,
        values: Vec<(Address, Vec<(Policy, u64)>)>,
    ) -> Result<Vec<Output>> {
        let outputs = values
            .into_iter()
            .map(|(owner, val_vec)| {
                let values = val_vec.into_iter().collect();
                Output::new(owner, values)
            })
            .collect();
        Ok(outputs)
    }
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

impl DataSource for FakeBackends {
    fn me(&self) -> &Address {
        &self.me
    }
}

impl TxBuilder for FakeBackends {
    /// No Fees, MinAda, or Collateral
    fn build(&self, unbuilt_tx: UnBuiltTransaction) -> crate::Result<Transaction> {
        let UnBuiltTransaction {
            mut inputs,
            actions,
            ..
        } = unbuilt_tx;
        let mut outputs = Vec::new();
        let combined_inputs = combined_totals(&inputs);

        if combined_inputs.len() > 0 {
            let output = Output::new(self.me.clone(), combined_inputs);
            outputs.push(output);
        }

        let (action_inputs, action_outputs) = self.handle_actions(actions)?;
        inputs.extend(action_inputs);
        outputs.extend(action_outputs);

        Ok(Transaction { inputs, outputs })
    }
}

fn combined_totals(inputs: &Vec<Output>) -> HashMap<Policy, u64> {
    let mut combined_values = HashMap::new();
    for input in inputs {
        for (policy, amount) in input.values().iter() {
            let mut new_total: u64 = *amount;
            if let Some(total) = combined_values.get(policy) {
                new_total += total;
            }
            combined_values.insert(policy.clone(), new_total);
        }
    }
    combined_values
}

impl TxIssuer for FakeBackends {
    fn issue(&self, tx: Transaction) -> Result<()> {
        let mut my_outputs = self.outputs.borrow_mut();
        for tx_i in tx.inputs() {
            let index = tx
                .inputs()
                .iter()
                .position(|x| x == tx_i)
                .ok_or(format!("Input: {:?} doesn't exist", &tx_i))?;
            my_outputs.remove(index);
        }

        for tx_o in tx.outputs() {
            my_outputs.push((tx_o.owner().clone(), tx_o.clone()))
        }
        Ok(())
    }
}
