use crate::{
    Address, DataSource, Output, Policy, Transaction, TxBuilder, TxIssuer, UnBuiltTransaction,
    Value,
};
use std::cell::RefCell;
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
            }) // Sum up policy values TODO: Panics
    }

    fn my_balance(&self, policy: &Policy) -> u64 {
        self.balance_at_address(&self.me, policy)
    }
}

impl DataSource for FakeBackends {}

impl TxBuilder for FakeBackends {
    /// No Fees, MinAda, or Collateral
    fn build(&self, unbuilt_tx: UnBuiltTransaction) -> crate::Result<Transaction> {
        let UnBuiltTransaction {
            inputs,
            output_values,
        } = unbuilt_tx;
        let combined_inputs = combined_totals(&inputs);

        let output = Output {
            owner: self.me.clone(),
            values: combined_inputs,
        };
        Ok(Transaction {
            inputs,
            outputs: vec![output],
        })
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
