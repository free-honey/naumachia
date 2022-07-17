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

    // TODO: Dedupe
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
                    let owner = self.me.clone();
                    let in_policy = policy.clone();
                    if let Some(h_map) = min_input_values.get(&owner) {
                        let mut inner = h_map.borrow_mut();
                        let mut new_total = amount;
                        if let Some(total) = inner.get(&in_policy) {
                            new_total += total;
                        }
                        inner.insert(in_policy, new_total);
                    } else {
                        let mut new_map = HashMap::new();
                        new_map.insert(in_policy, amount);
                        min_input_values.insert(owner, RefCell::new(new_map));
                    }
                    // Output
                    let owner = recipient;
                    let out_policy = policy.clone();
                    if let Some(h_map) = min_output_values.get(&owner) {
                        let mut inner = h_map.borrow_mut();
                        let mut new_total = amount;
                        if let Some(total) = inner.get(&out_policy) {
                            new_total += total;
                        }
                        inner.insert(out_policy, new_total);
                    } else {
                        let mut new_map = HashMap::new();
                        new_map.insert(out_policy, amount);
                        min_output_values.insert(owner, RefCell::new(new_map));
                    }
                }
            }
        }
        dbg!(min_input_values);
        dbg!(min_output_values);
        todo!()
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
