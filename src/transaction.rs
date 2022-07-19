use crate::{Address, Output, Policy, Value};

pub enum Action {
    Transfer {
        amount: u64,
        recipient: Address,
        policy: Policy,
    },
}

pub struct UnBuiltTransaction {
    pub inputs: Vec<Output>,
    pub output_values: Vec<(Address, Value)>,
    pub actions: Vec<Action>,
}

impl UnBuiltTransaction {
    pub fn new() -> Self {
        UnBuiltTransaction {
            inputs: vec![],
            output_values: vec![],
            actions: vec![],
        }
    }

    pub fn with_input(mut self, input: Output) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn with_output_value(mut self, output_value: (Address, Value)) -> Self {
        self.output_values.push(output_value);
        self
    }

    pub fn with_transfer(mut self, amount: u64, recipient: Address, policy: Policy) -> Self {
        let action = Action::Transfer {
            amount,
            recipient,
            policy,
        };
        self.actions.push(action);
        self
    }
}

#[derive(PartialEq, Debug)]
pub struct Transaction {
    pub inputs: Vec<Output>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    pub fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }

    pub fn inputs(&self) -> &Vec<Output> {
        &self.inputs
    }
}
