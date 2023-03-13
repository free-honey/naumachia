use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::PolicyId;
use pallas_addresses::Address;
use serde::{Deserialize, Serialize};

use crate::values::Values;

// TODO: Does this need to be separated?
#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub enum UnbuiltOutput<Datum> {
    Wallet {
        owner: String,
        values: Values,
    },
    Validator {
        script_address: String,
        values: Values,
        datum: Datum,
    },
}

impl<Datum> UnbuiltOutput<Datum> {
    pub fn new_wallet(owner: Address, values: Values) -> Self {
        UnbuiltOutput::Wallet {
            owner: owner.to_bech32().expect("already validated"),
            values,
        }
    }

    pub fn new_validator(script_address: Address, values: Values, datum: Datum) -> Self {
        UnbuiltOutput::Validator {
            script_address: script_address.to_bech32().expect("Already validated"),
            values,
            datum,
        }
    }

    pub fn owner(&self) -> Address {
        match self {
            UnbuiltOutput::Wallet { owner, .. } => {
                Address::from_bech32(owner).expect("Already Validated")
            }
            UnbuiltOutput::Validator { script_address, .. } => {
                Address::from_bech32(script_address).expect("Already Validated")
            }
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

#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub enum DatumKind<Datum> {
    Typed(Datum),
    UnTyped(PlutusData),
    None,
}

impl<Datum> From<DatumKind<Datum>> for Option<Datum> {
    fn from(value: DatumKind<Datum>) -> Self {
        match value {
            DatumKind::Typed(datum) => Some(datum),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub struct Output<Datum> {
    id: OutputId,
    owner: String,
    values: Values,
    datum: DatumKind<Datum>,
}

#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub struct OutputId {
    tx_hash: Vec<u8>,
    index: u64,
}

impl OutputId {
    pub fn new(tx_hash: Vec<u8>, index: u64) -> Self {
        OutputId { tx_hash, index }
    }

    pub fn tx_hash(&self) -> &[u8] {
        &self.tx_hash
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}

pub type Value = (PolicyId, u64);

impl<Datum> Output<Datum> {
    pub fn new_wallet(tx_hash: Vec<u8>, index: u64, owner: Address, values: Values) -> Self {
        let id = OutputId::new(tx_hash, index);
        let addr = owner.to_bech32().expect("Already Validated");
        Output {
            id,
            owner: addr,
            values,
            datum: DatumKind::None,
        }
    }

    pub fn new_validator(
        tx_hash: Vec<u8>,
        index: u64,
        owner: Address,
        values: Values,
        datum: Datum,
    ) -> Self {
        let id = OutputId::new(tx_hash, index);
        let addr = owner.to_bech32().expect("Already Validated");
        Output {
            id,
            owner: addr,
            values,
            datum: DatumKind::Typed(datum),
        }
    }

    pub fn id(&self) -> &OutputId {
        &self.id
    }

    pub fn owner(&self) -> Address {
        Address::from_bech32(&self.owner).expect("Already Validated")
    }

    pub fn values(&self) -> &Values {
        &self.values
    }

    pub fn datum(&self) -> &DatumKind<Datum> {
        &self.datum
    }
}

impl<Datum: Clone> Output<Datum> {
    pub fn typed_datum(&self) -> Option<Datum> {
        match &self.datum {
            DatumKind::Typed(datum) => Some(datum.to_owned()),
            _ => None,
        }
    }
}

impl<Datum: Clone + Into<PlutusData>> Output<Datum> {
    pub fn with_untyped_datum(&self) -> Output<Datum> {
        let new_datum = match &self.datum {
            DatumKind::Typed(datum) => DatumKind::UnTyped(datum.to_owned().into()),
            DatumKind::UnTyped(data) => DatumKind::UnTyped(data.clone()),
            DatumKind::None => DatumKind::None,
        };

        Output {
            id: self.id.clone(),
            owner: self.owner.clone(),
            values: self.values.clone(),
            datum: new_datum,
        }
    }
}

impl<Datum: Clone + TryFrom<PlutusData>> Output<Datum> {
    pub fn with_typed_datum_if_possible(&self) -> Output<Datum> {
        let new_datum = match &self.datum {
            DatumKind::Typed(datum) => DatumKind::Typed(datum.clone()),
            DatumKind::UnTyped(data) => {
                if let Ok(datum) = Datum::try_from(data.clone()) {
                    DatumKind::Typed(datum)
                } else {
                    DatumKind::UnTyped(data.clone())
                }
            }
            DatumKind::None => DatumKind::None,
        };

        Output {
            id: self.id.clone(),
            owner: self.owner.clone(),
            values: self.values.clone(),
            datum: new_datum,
        }
    }
}
