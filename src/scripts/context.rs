use crate::{
    output::Output, scripts::raw_validator_script::plutus_data::PlutusData, values::Values,
    PolicyId,
};
use pallas_addresses::Address;
use std::collections::HashMap;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
// TODO: This should be shaped like the real one actually. That will be extra useful because we can
//   expose all the primitives in case people want to use them for params, datums, etc...
#[derive(Clone, Debug)]
pub struct TxContext {
    pub purpose: CtxScriptPurpose,
    pub signer: PubKeyHash,
    pub range: ValidRange,
    pub inputs: Vec<Input>,
    pub outputs: Vec<CtxOutput>,
    pub extra_signatories: Vec<PubKeyHash>,
    pub datums: Vec<(Vec<u8>, PlutusData)>,
}

#[derive(Clone, Debug)]
pub enum CtxScriptPurpose {
    Mint(Vec<u8>),
    Spend(CtxOutputReference),
    WithdrawFrom,
    Publish,
}

#[derive(Clone, Debug)]
pub struct CtxOutputReference {
    pub(crate) transaction_id: Vec<u8>,
    pub(crate) output_index: u64,
}

impl CtxOutputReference {
    pub fn new(transaction_id: Vec<u8>, output_index: u64) -> Self {
        CtxOutputReference {
            transaction_id,
            output_index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PubKeyHash(Vec<u8>);

impl PubKeyHash {
    pub fn new(inner: &[u8]) -> Self {
        PubKeyHash(inner.to_vec())
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.0.to_owned()
    }
}

/// Retrieves pubkey if Address is a Shelley
pub fn pub_key_hash_from_address_if_available(address: &Address) -> Option<PubKeyHash> {
    match address {
        Address::Shelley(shelley_address) => {
            let hash = shelley_address.payment().as_hash().to_vec();
            let pkh = PubKeyHash::new(&hash);
            Some(pkh)
        }
        _ => None,
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
    pub address: Address,
    pub value: CtxValue,
    pub datum: CtxDatum,
    pub reference_script: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct CtxOutput {
    pub address: Address,
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
                PolicyId::Lovelace => ("", ""),
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
    signer: PubKeyHash,
    range: Option<ValidRange>,
    inputs: Vec<Input>,
    outputs: Vec<CtxOutput>,
    extra_signatories: Vec<PubKeyHash>,
    datums: Vec<(Vec<u8>, PlutusData)>,
}

impl ContextBuilder {
    pub fn new(signer: PubKeyHash) -> Self {
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

    pub fn with_input(
        self,
        transaction_id: &[u8],
        output_index: u64,
        address: &Address,
    ) -> InputBuilder {
        InputBuilder {
            outer: self,
            transaction_id: transaction_id.to_vec(),
            address: address.clone(),
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
        let address = input.owner();
        let value = CtxValue::from(input.values().to_owned());
        let maybe_datum: Option<D> = input.datum().to_owned().into();
        let maybe_datum = maybe_datum.map(|v| v.to_owned());
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

    pub fn with_output(self, address: &Address) -> CtxOutputBuilder {
        CtxOutputBuilder {
            outer: self,
            address: address.clone(),
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
        let address = input.owner();
        let value = CtxValue::from(input.values().to_owned());
        let maybe_datum: Option<D> = input.datum().to_owned().into();
        let maybe_datum = maybe_datum.map(|v| v.to_owned());
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

    pub fn add_signatory(mut self, signer: PubKeyHash) -> Self {
        self.extra_signatories.push(signer);
        self
    }

    pub fn add_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> Self {
        let data = datum.into();
        self.datums.push((data.hash(), data));
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
            purpose: CtxScriptPurpose::Spend(CtxOutputReference::new(tx_id.to_vec(), index)),
            signer: self.signer.clone(),
            range,
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            extra_signatories: self.extra_signatories.clone(),
            datums: self.datums.clone(),
        }
    }

    pub fn build_mint(&self, policy_id: &[u8]) -> TxContext {
        let range = if let Some(range) = self.range.clone() {
            range
        } else {
            ValidRange {
                lower: None,
                upper: None,
            }
        };
        TxContext {
            purpose: CtxScriptPurpose::Mint(policy_id.to_vec()),
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
    address: Address,
    value: HashMap<String, HashMap<String, u64>>,
    datum: CtxDatum,
    reference_script: Option<Vec<u8>>,
}

impl InputBuilder {
    pub fn with_value(mut self, policy_id: &str, asset_name: &str, amt: u64) -> InputBuilder {
        add_to_nested(&mut self.value, policy_id, asset_name, amt);
        self
    }

    pub fn with_inline_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> InputBuilder {
        self.datum = CtxDatum::InlineDatum(datum.into());
        self
    }

    pub fn with_datum_hash(mut self, datum_hash: Vec<u8>) -> InputBuilder {
        self.datum = CtxDatum::DatumHash(datum_hash);
        self
    }

    pub fn with_datum_hash_from_datum<Datum: Into<PlutusData>>(
        mut self,
        datum: Datum,
    ) -> InputBuilder {
        self.datum = CtxDatum::DatumHash(datum.into().hash());
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
    address: Address,
    value: HashMap<String, HashMap<String, u64>>,
    datum: CtxDatum,
    reference_script: Option<Vec<u8>>,
}

impl CtxOutputBuilder {
    pub fn with_value(mut self, policy_id: &str, asset_name: &str, amt: u64) -> Self {
        add_to_nested(&mut self.value, policy_id, asset_name, amt);
        self
    }

    pub fn with_inline_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> Self {
        self.datum = CtxDatum::InlineDatum(datum.into());
        self
    }

    pub fn with_datum_hash(mut self, datum_hash: Vec<u8>) -> Self {
        self.datum = CtxDatum::DatumHash(datum_hash);
        self
    }

    pub fn with_datum_hash_from_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> Self {
        self.datum = CtxDatum::DatumHash(datum.into().hash());
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
