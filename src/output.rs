use crate::{Address, PolicyId};
use serde::{Deserialize, Serialize};

use crate::values::Values;

#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub enum UnbuiltOutput<Datum> {
    Wallet {
        owner: Address,
        values: Values,
    },
    Validator {
        owner: Address,
        values: Values,
        datum: Datum,
    },
}

impl<Datum> UnbuiltOutput<Datum> {
    pub fn new_wallet(owner: Address, values: Values) -> Self {
        UnbuiltOutput::Wallet { owner, values }
    }

    pub fn owner(&self) -> &Address {
        match self {
            UnbuiltOutput::Wallet { owner, .. } => owner,
            UnbuiltOutput::Validator { owner, .. } => owner,
        }
    }

    pub fn values(&self) -> &Values {
        match self {
            UnbuiltOutput::Wallet { values, .. } => values,
            UnbuiltOutput::Validator { values, .. } => values,
        }
    }

    pub fn datum(&self) -> Option<&Datum> {
        match self {
            UnbuiltOutput::Wallet { .. } => None,
            UnbuiltOutput::Validator { datum, .. } => Some(datum),
        }
    }
}

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
    index: u64,
}

impl OutputId {
    pub fn new(tx_hash: String, index: u64) -> Self {
        OutputId { tx_hash, index }
    }
}

pub type Value = (PolicyId, u64);

impl<Datum> Output<Datum> {
    pub fn new_wallet(tx_hash: String, index: u64, owner: Address, values: Values) -> Self {
        let id = OutputId::new(tx_hash, index);
        Output::Wallet { id, owner, values }
    }

    pub fn new_validator(
        tx_hash: String,
        index: u64,
        owner: Address,
        values: Values,
        datum: Datum,
    ) -> Self {
        let id = OutputId::new(tx_hash, index);
        Output::Validator {
            id,
            owner,
            values,
            datum,
        }
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
