use crate::output::Output;
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::values::Values;
use crate::{Address, PolicyId};
use std::collections::HashMap;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
// TODO: This should be shaped like the real one actually. That will be extra useful because we can
//   expose all the primitives in case people want to use them for params, datums, etc...
#[derive(Clone, Debug)]
pub struct TxContext {
    pub signer: Address,
    pub range: ValidRange,
    pub inputs: Vec<Input>,
}

#[derive(Clone, Debug)]
pub struct ValidRange {
    pub lower: Option<(i64, bool)>,
    pub upper: Option<(i64, bool)>,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub transaction_id: Vec<u8>,
    pub output_index: u64,
    pub address: Vec<u8>,
    pub value: CtxValue,
    pub datum: CtxDatum,
    pub reference_script: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct CtxValue {
    pub inner: HashMap<String, HashMap<String, u64>>,
}

impl From<Values> for CtxValue {
    fn from(values: Values) -> Self {
        let mut inner = HashMap::new();
        for (policy, amt) in values.as_iter() {
            let (policy_id, asset_name) = match policy {
                PolicyId::ADA => ("", ""),
                PolicyId::NativeToken(policy_id, a) => {
                    if let Some(asset_name) = a {
                        (policy_id.as_str(), asset_name.as_str())
                    } else {
                        (policy_id.as_str(), "")
                    }
                }
            };
            add_to_nested(&mut inner, policy_id, asset_name, *amt);
        }
        CtxValue { inner }
    }
}

#[derive(Clone, Debug)]
pub enum CtxDatum {
    NoDatum,
    DatumHash(Vec<u8>),
    InlineDatum(PlutusData),
}

impl<D: Clone + Into<PlutusData>> From<Option<D>> for CtxDatum {
    fn from(value: Option<D>) -> Self {
        match value {
            None => CtxDatum::NoDatum,
            Some(datum) => CtxDatum::InlineDatum(datum.into()),
        }
    }
}

pub struct ContextBuilder {
    signer: Address,
    range: Option<ValidRange>,
    inputs: Vec<Input>,
}

impl ContextBuilder {
    pub fn new(signer: Address) -> Self {
        ContextBuilder {
            signer,
            range: None,
            inputs: vec![],
        }
    }

    pub fn with_range(mut self, lower: Option<(i64, bool)>, upper: Option<(i64, bool)>) -> Self {
        let valid_range = ValidRange { lower, upper };
        self.range = Some(valid_range);
        self
    }

    pub fn build_input(
        self,
        transaction_id: &[u8],
        output_index: u64,
        address: &str,
    ) -> InputBuilder {
        InputBuilder {
            outer: self,
            transaction_id: transaction_id.to_vec(),
            address: hex::decode(address).unwrap(),
            value: Default::default(),
            datum: CtxDatum::NoDatum,
            reference_script: None,
            output_index,
        }
    }

    fn add_input(mut self, input: Input) -> ContextBuilder {
        self.inputs.push(input);
        self
    }

    pub fn add_specific_input<D: Clone + Into<PlutusData>>(mut self, input: &Output<D>) -> Self {
        let id = input.id();
        let transaction_id = id.tx_hash().to_vec();
        let output_index = id.index();
        let address = input.owner().bytes().unwrap();
        let value = CtxValue::from(input.values().to_owned());
        let maybe_datum: Option<D> = input.datum().map(|v| v.to_owned());
        let datum = CtxDatum::from(maybe_datum);
        let ctx_input = Input {
            transaction_id,
            output_index,
            address,
            value,
            datum,
            reference_script: None,
        };
        self.inputs.push(ctx_input);
        self
    }

    pub fn build(&self) -> TxContext {
        let range = if let Some(range) = self.range.clone() {
            range
        } else {
            ValidRange {
                lower: None,
                upper: None,
            }
        };
        TxContext {
            signer: self.signer.clone(),
            range,
            inputs: self.inputs.clone(),
        }
    }
}

pub struct InputBuilder {
    outer: ContextBuilder,
    transaction_id: Vec<u8>,
    output_index: u64,
    address: Vec<u8>,
    value: HashMap<String, HashMap<String, u64>>,
    datum: CtxDatum,
    reference_script: Option<Vec<u8>>,
}

impl InputBuilder {
    pub fn with_value(mut self, policy_id: &str, asset_name: &str, amt: u64) -> InputBuilder {
        add_to_nested(&mut self.value, policy_id, asset_name, amt);
        self
    }

    pub fn with_inline_datum(mut self, plutus_data: PlutusData) -> InputBuilder {
        self.datum = CtxDatum::InlineDatum(plutus_data);
        self
    }

    pub fn with_datum_hash(mut self, datum_hash: Vec<u8>) -> InputBuilder {
        self.datum = CtxDatum::DatumHash(datum_hash);
        self
    }

    pub fn finish_input(self) -> ContextBuilder {
        let value = CtxValue { inner: self.value };
        let input = Input {
            transaction_id: self.transaction_id,
            output_index: self.output_index,
            address: self.address,
            value,
            datum: self.datum,
            reference_script: self.reference_script,
        };
        self.outer.add_input(input)
    }
}

fn add_to_nested(
    values: &mut HashMap<String, HashMap<String, u64>>,
    policy_id: &str,
    asset_name: &str,
    amt: u64,
) {
    let new_assets = if let Some(mut assets) = values.remove(policy_id) {
        if let Some(mut total_amt) = assets.remove(asset_name) {
            total_amt += amt;
            assets.insert(asset_name.to_string(), total_amt);
        } else {
            assets.insert(asset_name.to_string(), amt);
        }
        assets
    } else {
        let mut assets = HashMap::new();
        assets.insert(asset_name.to_string(), amt);
        assets
    };
    values.insert(policy_id.to_string(), new_assets);
}
