use crate::error::Result;
use crate::transaction::Action;
use crate::{
    Address, DataSource, Output, Policy, Transaction, TxBuilder, TxIssuer, UnBuiltTransaction,
    Value,
};
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::ops::Deref;

mod always_mints_contract;
mod escrow_contract;
mod transfer;

struct FakeBackends {
    me: Address,
    outputs: RefCell<Vec<(Address, Output)>>,
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

    fn balance_at_address(&self, address: &Address, policy: &Policy) -> u64 {
        self.outputs
            .borrow()
            .iter()
            .filter_map(|(a, o)| if a == address { Some(o) } else { None }) // My outputs
            .fold(0, |acc, o| {
                if let Some(val) = o.values.get(policy) {
                    acc + val
                } else {
                    acc
                }
            }) // Sum up policy values
    }

    fn my_balance(&self, policy: &Policy) -> u64 {
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
            }
        }
        // inputs
        let in_vecs = nested_map_to_vecs(min_input_values);
        dbg!(&in_vecs);
        let (inputs, remainders) = self.select_inputs_for_all(in_vecs)?;

        // outputs
        remainders.iter().for_each(|(amt, recp, policy)| {
            add_amount_to_nested_map(&mut min_output_values, *amt, recp, policy)
        });

        let out_vecs = nested_map_to_vecs(min_output_values);
        dbg!(&out_vecs);
        let outputs = self.create_outputs_for(Vec::new())?;

        Ok((inputs, outputs))
    }

    // Naive
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

    fn select_inputs_for_one(
        &self,
        address: &Address,
        values: &Vec<(Policy, u64)>,
    ) -> Result<(Vec<Output>, Vec<(u64, Address, Policy)>)> {
        let address_inputs = self.outputs_at_address(address);
        todo!();
    }

    fn create_outputs_for(
        &self,
        values: Vec<(Address, Vec<(Policy, u64)>)>,
    ) -> Result<Vec<Output>> {
        // todo!()
        Ok(Vec::new())
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

impl DataSource for FakeBackends {}

impl TxBuilder for FakeBackends {
    /// No Fees, MinAda, or Collateral
    fn build(&self, unbuilt_tx: UnBuiltTransaction) -> crate::Result<Transaction> {
        let UnBuiltTransaction {
            mut inputs,
            output_values,
            actions,
        } = unbuilt_tx;
        let combined_inputs = combined_totals(&inputs);

        let output = Output {
            owner: self.me.clone(),
            values: combined_inputs,
        };
        let mut outputs = Vec::new();
        outputs.push(output);
        let (action_inputs, action_outputs) = self.handle_actions(actions)?;
        inputs.extend(action_inputs);
        outputs.extend(action_outputs);

        Ok(Transaction { inputs, outputs })
    }
}

fn combined_totals(inputs: &Vec<Output>) -> HashMap<Policy, u64> {
    let mut combined_values = HashMap::new();
    for input in inputs {
        for (policy, amount) in input.values.iter() {
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
    fn issue(&self, tx: Transaction) -> crate::Result<()> {
        let mut my_outputs = self.outputs.borrow_mut();
        for tx_o in tx.outputs() {
            my_outputs.push((tx_o.owner.clone(), tx_o.clone()))
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let me = Address("me".to_string());
        let policy = Some(Address("some_coin".to_string()));
        let amount = 420;
        let mut values = HashMap::new();
        values.insert(policy.clone(), amount);
        let input = Output {
            owner: me.clone(),
            values: values.clone(),
        };

        let unbuilt_tx = UnBuiltTransaction::new().with_input(input.clone());

        let new_output = Output {
            owner: me.clone(),
            values,
        };

        let expected_tx = Transaction {
            inputs: vec![input.clone()],
            outputs: vec![new_output],
        };

        let mut backend = FakeBackends::new(me.clone());
        backend.with_output(me, input);

        let actual_tx = backend.build(unbuilt_tx).unwrap();

        assert_eq!(expected_tx, actual_tx);
    }
}
