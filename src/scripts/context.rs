use crate::{
    output::Output, scripts::plutus_validator::plutus_data::PlutusData, values::Values, PolicyId,
};
use pallas_addresses::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
// TODO: This should be shaped like the real one actually. That will be extra useful because we can
//   expose all the primitives in case people want to use them for params, datums, etc...
/// The context of the transaction that is executing a script
#[derive(Clone, Debug)]
pub struct TxContext {
    /// The purpose of the script
    pub purpose: CtxScriptPurpose,
    /// The signer of the transaction
    pub signer: PubKeyHash,
    /// The valid range of the transaction
    pub range: ValidRange,
    /// The input UTxOs of the transaction
    pub inputs: Vec<Input>,
    /// The output UTxOs of the transaction
    pub outputs: Vec<CtxOutput>,
    /// The extra signatories of the transaction
    pub extra_signatories: Vec<PubKeyHash>,
    /// A map of datum hashes to datums
    pub datums: Vec<(Vec<u8>, PlutusData)>,
}

/// The purpose of the script
#[derive(Clone, Debug)]
pub enum CtxScriptPurpose {
    /// Mint tokens
    Mint(Vec<u8>),
    /// Spend tokens at a script address
    Spend(CtxOutputReference),
    /// Withdraw staked tokens
    WithdrawFrom,
    /// Publish certificate
    Publish,
}

/// Specifies the output that is being spent in the script purpose
#[derive(Clone, Debug)]
pub struct CtxOutputReference {
    pub(crate) transaction_id: Vec<u8>,
    pub(crate) output_index: u64,
}

impl CtxOutputReference {
    /// Constructor for `CtxOutputReference`
    pub fn new(transaction_id: Vec<u8>, output_index: u64) -> Self {
        CtxOutputReference {
            transaction_id,
            output_index,
        }
    }
}

/// The public key hash of the signer
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct PubKeyHash(Vec<u8>);

impl PubKeyHash {
    /// Constructor for `PubKeyHash`
    pub fn new(inner: &[u8]) -> Self {
        PubKeyHash(inner.to_vec())
    }

    /// Getter for inner bytes of `PubKeyHash`
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

// TODO: Remove the inclusive bool. It's not needed.
/// Valid range of tx in milliseconds, and a `bool` specifying inclusive. If `None`, then the range is unbounded.
#[derive(Clone, Debug)]
pub struct ValidRange {
    /// Lower bound of valid range
    pub lower: Option<(i64, bool)>,
    /// Upper bound of valid range
    pub upper: Option<(i64, bool)>,
}

/// [`TxContext`]'s representation of an input UTxO
#[derive(Clone, Debug)]
pub struct Input {
    /// Transaction id
    pub transaction_id: Vec<u8>,
    /// ID of the transaction that outputs this UTxO
    pub output_index: u64,
    /// Owner's address
    pub address: Address,
    /// Value of UTxO
    pub value: CtxValue,
    /// Datum attached to UTxO
    pub datum: CtxDatum,
    /// Script referenced by UTxO
    pub reference_script: Option<Vec<u8>>,
}

/// [`TxContext`]'s representation of an output UTxO
#[derive(Clone, Debug)]
pub struct CtxOutput {
    /// ID of the transaction that outputs this UTxO
    pub address: Address,
    /// Value of UTxO
    pub value: CtxValue,
    /// Datum attached to the UTxO
    pub datum: CtxDatum,
    /// Script referenced by UTxO
    pub reference_script: Option<Vec<u8>>,
}

/// [`TxContext`]'s representation of UTxO values.
#[derive(Clone, Debug)]
pub struct CtxValue {
    /// Inner map of PolicyIds to Asset Names and amount
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

/// [`TxContext`]'s representation of a datum
#[derive(Clone, Debug)]
pub enum CtxDatum {
    /// No datum is attached
    NoDatum,
    /// Includes a reference to a Datum Hash
    DatumHash(Vec<u8>),
    /// Includes the actual datum inline
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

/// Builder for constructing [`TxContext`]s in tests and other mocked environments
///
/// For example, you can construct a context with a "spend" purpose like this:
/// ```
/// let ctx = ContextBuilder::new(signer_pkh)
///             .with_input(&hex::decode("73d65e0b9b68ebf3971b6ccddc75900dd62f9845f5ab972e469c5d803973015b")
///                     .unwrap(),
///                 0,
///                 &signer,
///             )
///             .with_value(&hex::encode(&policy), "", 1)
///             .finish_input()
///             .build_spend(&vec![], 0);
/// ```
pub struct ContextBuilder {
    signer: PubKeyHash,
    range: Option<ValidRange>,
    inputs: Vec<Input>,
    outputs: Vec<CtxOutput>,
    extra_signatories: Vec<PubKeyHash>,
    datums: Vec<(Vec<u8>, PlutusData)>,
}

impl ContextBuilder {
    /// Constructor for `ContextBuilder`
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

    /// Add specific valid range for the `TxContext`
    pub fn with_range(mut self, lower: Option<(i64, bool)>, upper: Option<(i64, bool)>) -> Self {
        let valid_range = ValidRange { lower, upper };
        self.range = Some(valid_range);
        self
    }

    /// Initializes [`CtxInputBuilder`] sub-builder for adding an input to the `TxContext`
    pub fn with_input(
        self,
        transaction_id: &[u8],
        output_index: u64,
        address: &Address,
    ) -> CtxInputBuilder {
        CtxInputBuilder {
            outer: self,
            transaction_id: transaction_id.to_vec(),
            address: address.clone(),
            value: Default::default(),
            datum: CtxDatum::NoDatum,
            reference_script: None,
            output_index,
        }
    }

    /// Add specific [`Input`] UTxO, rather than using `with_input`
    fn add_input(mut self, input: Input) -> ContextBuilder {
        self.inputs.push(input);
        self
    }

    /// Add specific [`Output`] as an input, rather than using `with_input`
    pub fn add_specific_input<D: Clone + Into<PlutusData>>(mut self, input: &Output<D>) -> Self {
        let id = input.id();
        let transaction_id = id.tx_hash().to_vec();
        let output_index = id.index();
        let address = input.owner();
        let value = CtxValue::from(input.values().to_owned());
        let datum = input.typed_datum().into();
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

    /// Initializes [`CtxOutputBuilder`] sub-builder for adding an output to the `TxContext`
    pub fn with_output(self, address: &Address) -> CtxOutputBuilder {
        CtxOutputBuilder {
            outer: self,
            address: address.clone(),
            value: Default::default(),
            datum: CtxDatum::NoDatum,
            reference_script: None,
        }
    }

    /// Add specific [`CtxOutput`] as an output, rather than using `with_output`
    fn add_output(mut self, output: CtxOutput) -> ContextBuilder {
        self.outputs.push(output);
        self
    }

    /// Add specific [`Output`] as an output, rather than using `with_input`
    pub fn add_specific_output<D: Clone + Into<PlutusData>>(mut self, input: &Output<D>) -> Self {
        let address = input.owner();
        let value = CtxValue::from(input.values().to_owned());
        let datum = input.typed_datum().into();
        let ctx_input = CtxOutput {
            address,
            value,
            datum,
            reference_script: None,
        };
        self.outputs.push(ctx_input);
        self
    }

    /// Add specific "extra" signatory
    pub fn add_signatory(mut self, signer: PubKeyHash) -> Self {
        self.extra_signatories.push(signer);
        self
    }

    /// Add specific `Datum`
    pub fn add_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> Self {
        let data = datum.into();
        self.datums.push((data.hash(), data));
        self
    }

    /// Build the context with a "spend" purpose
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

    /// Build the context with a "mint" purpose
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

/// Sub-builder for adding an [`CtxInput`] to a [`TxContext`]
pub struct CtxInputBuilder {
    outer: ContextBuilder,
    transaction_id: Vec<u8>,
    output_index: u64,
    address: Address,
    value: HashMap<String, HashMap<String, u64>>,
    datum: CtxDatum,
    reference_script: Option<Vec<u8>>,
}

impl CtxInputBuilder {
    /// Add single value to the `CtxInput`
    pub fn with_value(mut self, policy_id: &str, asset_name: &str, amt: u64) -> CtxInputBuilder {
        add_to_nested(&mut self.value, policy_id, asset_name, amt);
        self
    }

    /// Add an inline `Datum` to the `CtxInput`. Will override previous value
    pub fn with_inline_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> CtxInputBuilder {
        self.datum = CtxDatum::InlineDatum(datum.into());
        self
    }

    /// Add datum hash to the `CtxInput`. Will override the previous value
    pub fn with_datum_hash(mut self, datum_hash: Vec<u8>) -> CtxInputBuilder {
        self.datum = CtxDatum::DatumHash(datum_hash);
        self
    }

    /// Add a `Datum` that will be stored on the `CtxInput` as a datum hash.
    pub fn with_datum_hash_from_datum<Datum: Into<PlutusData>>(
        mut self,
        datum: Datum,
    ) -> CtxInputBuilder {
        self.datum = CtxDatum::DatumHash(datum.into().hash());
        self
    }

    /// Build the input with the specified values and add it to the [`ContextBuilder`]
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

/// Sub-builder for adding an [`CtxOutput`] to a [`TxContext`]
pub struct CtxOutputBuilder {
    outer: ContextBuilder,
    address: Address,
    value: HashMap<String, HashMap<String, u64>>,
    datum: CtxDatum,
    reference_script: Option<Vec<u8>>,
}

impl CtxOutputBuilder {
    /// Add specific value to the `CtxOutput`
    pub fn with_value(mut self, policy_id: &str, asset_name: &str, amt: u64) -> Self {
        add_to_nested(&mut self.value, policy_id, asset_name, amt);
        self
    }

    /// Add an inline datum to the `CtxOutput`. Will override the previous value
    pub fn with_inline_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> Self {
        self.datum = CtxDatum::InlineDatum(datum.into());
        self
    }

    /// Add a datum has to the `CtxOutput`. Will override the previous value
    pub fn with_datum_hash(mut self, datum_hash: Vec<u8>) -> Self {
        self.datum = CtxDatum::DatumHash(datum_hash);
        self
    }

    /// Add a `Datum` that will be stored on the `CtxOutput` as a datum hash.
    pub fn with_datum_hash_from_datum<Datum: Into<PlutusData>>(mut self, datum: Datum) -> Self {
        self.datum = CtxDatum::DatumHash(datum.into().hash());
        self
    }

    /// Build the output with the specified values and add it to the [`ContextBuilder`]
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
