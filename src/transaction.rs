use crate::address::{Address, Policy};
use crate::output::Output;
use std::collections::HashMap;

pub enum Action<Datum> {
    Transfer {
        amount: u64,
        recipient: Address,
        policy: Policy,
    },
    Mint {
        amount: u64,
        recipient: Address,
        policy: Policy,
    },
    InitScript {
        datum: Datum,
        values: HashMap<Policy, u64>,
        address: Address,
    },
}

pub struct UnBuiltTransaction<Datum> {
    pub actions: Vec<Action<Datum>>,
}

impl<Datum> Default for UnBuiltTransaction<Datum> {
    fn default() -> Self {
        UnBuiltTransaction {
            actions: Vec::new(),
        }
    }
}

impl<Datum> UnBuiltTransaction<Datum> {
    pub fn with_transfer(mut self, amount: u64, recipient: Address, policy: Policy) -> Self {
        let action = Action::Transfer {
            amount,
            recipient,
            policy,
        };
        self.actions.push(action);
        self
    }

    pub fn with_mint(mut self, amount: u64, recipient: Address, policy: Policy) -> Self {
        let action = Action::Mint {
            amount,
            recipient,
            policy,
        };
        self.actions.push(action);
        self
    }

    pub fn with_script_init(
        mut self,
        datum: Datum,
        values: HashMap<Policy, u64>,
        address: Address,
    ) -> Self {
        let action = Action::InitScript {
            datum,
            values,
            address,
        };
        self.actions.push(action);
        self
    }
}

#[derive(PartialEq, Debug)]
pub struct Transaction<Datum> {
    pub inputs: Vec<Output<Datum>>,
    pub outputs: Vec<Output<Datum>>,
}

impl<Datum> Transaction<Datum> {
    pub fn outputs(&self) -> &Vec<Output<Datum>> {
        &self.outputs
    }

    pub fn inputs(&self) -> &Vec<Output<Datum>> {
        &self.inputs
    }
}
