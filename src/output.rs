use crate::{Address, Policy};
use std::collections::HashMap;

// TODO: Find max size instead of u64. It might not actually matter since we'll never be able to
//       select more than actually exists on chain. But maybe for minting?
#[derive(Clone, PartialEq, Debug)]
pub enum Output<Datum> {
    Wallet {
        owner: Address,
        values: HashMap<Policy, u64>,
    },
    Validator {
        owner: Address,
        values: HashMap<Policy, u64>,
        datum: Datum,
    },
}

pub type Value = (Policy, u64);

impl<Datum> Output<Datum> {
    pub fn wallet(owner: Address, values: HashMap<Policy, u64>) -> Self {
        Output::Wallet { owner, values }
    }

    pub fn owner(&self) -> &Address {
        match self {
            Output::Wallet { owner, .. } => owner,
            Output::Validator { owner, .. } => owner,
        }
    }

    pub fn values(&self) -> &HashMap<Policy, u64> {
        match self {
            Output::Wallet { values, .. } => values,
            Output::Validator { values, .. } => values,
        }
    }
}
