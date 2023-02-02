use crate::output::Output;
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::values::Values;
use crate::PolicyId;
use pallas_addresses::Address;
use std::collections::HashMap;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
// TODO: This should be shaped like the real one actually. That will be extra useful because we can
//   expose all the primitives in case people want to use them for params, datums, etc...
#[derive(Clone, Debug)]
pub struct TxContext {
    pub purpose: CtxScriptPurpose,
    pub signer: PubKey,
    pub range: ValidRange,
    pub inputs: Vec<Input>,
    pub outputs: Vec<CtxOutput>,
    pub extra_signatories: Vec<PubKey>,
    pub datums: Vec<(Vec<u8>, PlutusData)>,
}

pub enum CtxScriptPurpose {
    Mint(Vec<u8>),
    Spend(Vec<u8>, u64),
    WithdrawFrom,
    Publish,
}

#[derive(Clone, Debug)]
pub struct PubKey(Vec<u8>);

impl PubKey {
    pub fn new(inner: &[u8]) -> Self {
        PubKey(inner.to_vec())
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.0.to_owned()
    }
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
pub struct CtxOutput {
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
    signer: PubKey,
    range: Option<ValidRange>,
    inputs: Vec<Input>,
    outputs: Vec<CtxOutput>,
    extra_signatories: Vec<PubKey>,
    datums: Vec<(Vec<u8>, PlutusData)>,
}

impl ContextBuilder {
    pub fn new(signer: Address) -> Self {
        // TODO: This is completely wrong, pubkey can't be derived from an address
        let signer = PubKey::new(&signer.to_vec());
        ContextBuilder {
            signer,
            range: None,
            inputs: vec![],
            outputs: vec![],
            extra_signatories: vec![],
            datums: vec![],
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
        address: &[u8],
    ) -> InputBuilder {
        InputBuilder {
            outer: self,
            transaction_id: transaction_id.to_vec(),
            address: address.to_vec(),
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
        let address = input.owner().to_vec();
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

    pub fn build_output(self, address: &[u8]) -> CtxOutputBuilder {
        CtxOutputBuilder {
            outer: self,
            address: address.to_vec(),
            value: Default::default(),
            datum: CtxDatum::NoDatum,
            reference_script: None,
        }
    }

    fn add_output(mut self, output: CtxOutput) -> ContextBuilder {
        self.outputs.push(output);
        self
    }

    pub fn add_specific_output<D: Clone + Into<PlutusData>>(mut self, input: &Output<D>) -> Self {
        let address = input.owner().to_vec();
        let value = CtxValue::from(input.values().to_owned());
        let maybe_datum: Option<D> = input.datum().map(|v| v.to_owned());
        let datum = CtxDatum::from(maybe_datum);
        let ctx_input = CtxOutput {
            address,
            value,
            datum,
            reference_script: None,
        };
        self.outputs.push(ctx_input);
        self
    }

    pub fn add_signatory(mut self, signer: &Address) -> Self {
        let pubkey = PubKey::new(&signer.to_vec());
        self.extra_signatories.push(pubkey);
        self
    }

    pub fn add_datum<Datum: Into<PlutusData>>(mut self, datum_hash: &[u8], datum: Datum) -> Self {
        self.datums.push((datum_hash.to_vec(), datum.into()));
        self
    }

    pub fn build_spend(&self, tx_id: &[u8], index: u64) -> TxContext {
        let range = if let Some(range) = self.range.clone() {
            range
        } else {
            ValidRange {
                lower: None,
                upper: None,
            }
        };
        TxContext {
            purpose: CtxScriptPurpose::Spend(tx_id.to_vec(), index),
            signer: self.signer.clone(),
            range,
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            extra_signatories: self.extra_signatories.clone(),
            datums: self.datums.clone(),
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

pub struct CtxOutputBuilder {
    outer: ContextBuilder,
    address: Vec<u8>,
    value: HashMap<String, HashMap<String, u64>>,
    datum: CtxDatum,
    reference_script: Option<Vec<u8>>,
}

impl CtxOutputBuilder {
    pub fn with_value(mut self, policy_id: &str, asset_name: &str, amt: u64) -> Self {
        add_to_nested(&mut self.value, policy_id, asset_name, amt);
        self
    }

    pub fn with_inline_datum(mut self, plutus_data: PlutusData) -> Self {
        self.datum = CtxDatum::InlineDatum(plutus_data);
        self
    }

    pub fn with_datum_hash(mut self, datum_hash: Vec<u8>) -> Self {
        self.datum = CtxDatum::DatumHash(datum_hash);
        self
    }

    pub fn finish_output(self) -> ContextBuilder {
        let value = CtxValue { inner: self.value };
        let output = CtxOutput {
            address: self.address,
            value,
            datum: self.datum,
            reference_script: self.reference_script,
        };
        self.outer.add_output(output)
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
