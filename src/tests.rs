use crate::{
    Address, DataSource, Output, Policy, Transaction, TxBuilder, TxIssuer, UnBuiltTransaction,
};
use std::cell::RefCell;

mod always_mints_contract;
mod escrow_contract;

struct MockBackend {
    me: Address,
    outputs: RefCell<Vec<(Address, Output)>>,
}

impl MockBackend {
    fn balance_at_address(&self, address: &Address, policy: &Policy) -> u64 {
        self.outputs
            .borrow()
            .iter()
            .filter_map(|(a, o)| if a == address { Some(o) } else { None }) // My outputs
            .fold(0, |acc, o| acc + o.value[policy]) // Sum up policy values TODO: Panics
    }

    fn my_balance(&self, policy: &Policy) -> u64 {
        self.balance_at_address(&self.me, policy)
    }
}

impl DataSource for MockBackend {}

impl TxBuilder for MockBackend {
    fn build(&self, unbuilt_tx: UnBuiltTransaction) -> crate::Result<Transaction> {
        let UnBuiltTransaction { inputs, outputs } = unbuilt_tx;
        Ok(Transaction { inputs, outputs })
    }
}

impl TxIssuer for MockBackend {
    fn issue(&self, tx: Transaction) -> crate::Result<()> {
        let mut my_outputs = self.outputs.borrow_mut();
        for tx_o in tx.outputs {
            my_outputs.push((self.me.clone(), tx_o))
        }
        Ok(())
    }
}
