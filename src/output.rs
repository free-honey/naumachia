use crate::{Address, PolicyId};
use serde::{Deserialize, Serialize};

use crate::values::Values;

// TODO: Find max size instead of u64. It might not actually matter since we'll never be able to
//       select more than actually exists on chain. But maybe for minting?
// TODO: We should genericize the id
// TODO: We should genericize the owner
#[serde_with::serde_as]
#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub enum Output<Datum> {
    Wallet {
        id: OutputId,
        owner: Address,
        values: Values,
    },
    Validator {
        id: OutputId,
        owner: Address,
        values: Values,
        datum: Datum,
    },
}

#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub struct OutputId {
    tx_hash: String,
    index: u32,
}

impl OutputId {
    pub fn new(tx_hash: String, index: u32) -> Self {
        OutputId { tx_hash, index }
    }
}

pub type Value = (PolicyId, u64);

impl<Datum> Output<Datum> {
    pub fn new_wallet(tx_hash: String, index: u32, owner: Address, values: Values) -> Self {
        let id = OutputId::new(tx_hash, index);
        Output::Wallet { id, owner, values }
    }

    pub fn id(&self) -> &OutputId {
        match self {
            Output::Wallet { id, .. } => id,
            Output::Validator { id, .. } => id,
        }
    }

    pub fn owner(&self) -> &Address {
        match self {
            Output::Wallet { owner, .. } => owner,
            Output::Validator { owner, .. } => owner,
        }
    }

    pub fn values(&self) -> &Values {
        match self {
            Output::Wallet { values, .. } => values,
            Output::Validator { values, .. } => values,
        }
    }

    pub fn datum(&self) -> Option<&Datum> {
        match self {
            Output::Wallet { .. } => None,
            Output::Validator { datum, .. } => Some(datum),
        }
    }
}
