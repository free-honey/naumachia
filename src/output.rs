use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use pallas_addresses::Address;
use serde::{Deserialize, Serialize};

use crate::values::Values;

// TODO: Does this need to be separated? We might be able to just have one variant
/// Representation of an UTxO that does not exist yet. This can be used inside the
/// [`SCLogic`] to represent outputs that will be created when the transaction is submitted.
#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub enum UnbuiltOutput<Datum> {
    /// An output owned by a wallet
    Wallet {
        /// Address of the wallet
        owner: String,
        /// Values of the output
        values: Values,
    },
    /// An output owned by a validator script
    Validator {
        /// Address of the validator script
        script_address: String,
        /// Values of the output
        values: Values,
        /// Datum of the output
        datum: Datum,
    },
}

impl<Datum> UnbuiltOutput<Datum> {
    /// Constructor for wallet output
    pub fn new_wallet(owner: Address, values: Values) -> Self {
        UnbuiltOutput::Wallet {
            owner: owner.to_bech32().expect("already validated"),
            values,
        }
    }

    /// Constructor for validator output
    pub fn new_validator(script_address: Address, values: Values, datum: Datum) -> Self {
        UnbuiltOutput::Validator {
            script_address: script_address.to_bech32().expect("Already validated"),
            values,
            datum,
        }
    }

    /// Getter for owner of output
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

    /// Getter for values of output
    pub fn values(&self) -> &Values {
        match self {
            UnbuiltOutput::Wallet { values, .. } => values,
            UnbuiltOutput::Validator { values, .. } => values,
        }
    }

    /// Getter for (optional) datum of output
    pub fn datum(&self) -> Option<&Datum> {
        match self {
            UnbuiltOutput::Wallet { .. } => None,
            UnbuiltOutput::Validator { datum, .. } => Some(datum),
        }
    }
}

/// Representation of an on-chain datum
#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub enum DatumKind<Datum> {
    /// A typed datum
    Typed(Datum),
    /// An untyped datum
    UnTyped(PlutusData),
    /// No datum
    None,
}

impl<Datum> DatumKind<Datum> {
    /// Unwrap the datum if it is typed
    pub fn unwrap_typed(self) -> Datum {
        match self {
            DatumKind::Typed(datum) => datum,
            _ => panic!("Expected Typed Datum"),
        }
    }

    /// Unwrap the datum if it is untyped
    pub fn unwrap_untyped(self) -> PlutusData {
        match self {
            DatumKind::UnTyped(datum) => datum,
            _ => panic!("Expected Untyped Datum"),
        }
    }
}

impl<Datum> From<DatumKind<Datum>> for Option<Datum> {
    fn from(value: DatumKind<Datum>) -> Self {
        match value {
            DatumKind::Typed(datum) => Some(datum),
            _ => None,
        }
    }
}

/// Domain specific representation of an on-chain UTxO
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Output<Datum> {
    id: OutputId,
    owner: String,
    values: Values,
    datum: DatumKind<Datum>,
}

/// Unique identifier for specific UTxO
#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize)]
pub struct OutputId {
    tx_hash: Vec<u8>,
    index: u64,
}

impl OutputId {
    /// Constructor for OutputId
    pub fn new(tx_hash: Vec<u8>, index: u64) -> Self {
        OutputId { tx_hash, index }
    }

    /// Getter for id's tx_hash
    pub fn tx_hash(&self) -> &[u8] {
        &self.tx_hash
    }

    /// Getter for id's index
    pub fn index(&self) -> u64 {
        self.index
    }
}

impl<Datum> Output<Datum> {
    /// Constructor for wallet output
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

    /// Constructor for validator output
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

    /// Constructor for validator output with untyped datum
    pub fn new_untyped_validator(
        tx_hash: Vec<u8>,
        index: u64,
        owner: Address,
        values: Values,
        datum: PlutusData,
    ) -> Self {
        let id = OutputId::new(tx_hash, index);
        let addr = owner.to_bech32().expect("Already Validated");
        Output {
            id,
            owner: addr,
            values,
            datum: DatumKind::UnTyped(datum),
        }
    }

    /// Getter for Output's id
    pub fn id(&self) -> &OutputId {
        &self.id
    }

    /// Getter for Output's owner address
    pub fn owner(&self) -> Address {
        Address::from_bech32(&self.owner).expect("Already Validated")
    }

    /// Getter for Output's values
    pub fn values(&self) -> &Values {
        &self.values
    }

    /// Getter for Output's datum
    pub fn datum(&self) -> &DatumKind<Datum> {
        &self.datum
    }
}

impl<Datum: Clone> Output<Datum> {
    /// Getter for Output's datum, if it is typed. Returns `None` if datum is untyped or non-existent
    pub fn typed_datum(&self) -> Option<Datum> {
        match &self.datum {
            DatumKind::Typed(datum) => Some(datum.to_owned()),
            _ => None,
        }
    }
}

impl<Datum: Clone + Into<PlutusData>> Output<Datum> {
    /// Converts `Output` to have an untyped datum, if it is typed. Returns the same `Output` if
    /// datum is untyped or non-existent
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

    /// Getter for `Output`'s datum as `PlutusData`, if it is typed. Returns `None` if datum is non-existent
    pub fn datum_plutus_data(&self) -> Option<PlutusData> {
        match &self.datum {
            DatumKind::Typed(datum) => Some(datum.to_owned().into()),
            DatumKind::UnTyped(data) => Some(data.to_owned()),
            DatumKind::None => None,
        }
    }
}

impl<Datum: Clone + TryFrom<PlutusData>> Output<Datum> {
    /// Converts `Output` to have a typed datum, if it is untyped and can be converted. Returns the
    /// same `Output` if datum is typed, it can't convert, or non-existent
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
