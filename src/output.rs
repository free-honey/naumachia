use crate::{Address, Policy};
use std::collections::HashMap;

// enum NewOutput<Datum> {
//     Wallet(Address, HashMap<Policy, u64>),
//     Validator(Address, HashMap<Policy, u64>, Datum),
// }

pub type Value = (Policy, u64);

// TODO: Find max size instead of u64. It might not actually matter since we'll never be able to
//       select more than actually exists on chain. But maybe for minting?
#[derive(Clone, PartialEq, Debug)]
pub struct Output {
    owner: Address,
    values: HashMap<Policy, u64>,
}

impl Output {
    pub fn new(owner: Address, values: HashMap<Policy, u64>) -> Self {
        Output { owner, values }
    }

    pub fn owner(&self) -> &Address {
        &self.owner
    }

    pub fn values(&self) -> &HashMap<Policy, u64> {
        &self.values
    }
}
