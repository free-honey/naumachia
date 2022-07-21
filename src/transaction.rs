use crate::address::{Address, Policy};
use crate::output::{Output, Value};

pub enum Action {
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
}

#[derive(Default)]
pub struct UnBuiltTransaction<Datum> {
    pub inputs: Vec<Output<Datum>>,
    pub output_values: Vec<(Address, Value)>,
    pub actions: Vec<Action>,
}

impl<Datum> UnBuiltTransaction<Datum> {
    // pub fn with_input(mut self, input: Output) -> Self {
    //     self.inputs.push(input);
    //     self
    // }
    //
    // pub fn with_output_value(mut self, output_value: (Address, Value)) -> Self {
    //     self.output_values.push(output_value);
    //     self
    // }

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
