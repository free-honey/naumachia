use crate::{Address, Output, Value};

pub struct UnBuiltTransaction {
    pub inputs: Vec<Output>,
    pub output_values: Vec<(Address, Value)>,
}

impl UnBuiltTransaction {
    pub fn new() -> Self {
        UnBuiltTransaction {
            inputs: vec![],
            output_values: vec![],
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
}
