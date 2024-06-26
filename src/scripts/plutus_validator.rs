use crate::{
    scripts::{
        as_failed_to_execute,
        context::TxContext,
        plutus_validator::plutus_data::{
            BigInt,
            Constr,
            PlutusData,
        },
        raw_script::{
            PlutusScriptError,
            PlutusScriptFile,
            RawPlutusScriptResult,
            ValidatorBlueprint,
        },
        ScriptError,
        ScriptResult,
        Validator,
    },
    transaction::TransactionVersion,
};
use cardano_multiplatform_lib::{
    address::{
        EnterpriseAddress,
        StakeCredential,
    },
    plutus::{
        PlutusScript,
        PlutusV1Script,
    },
};
use minicbor::{
    Decoder,
    Encoder,
};

use crate::scripts::ExecutionCost;
use cardano_multiplatform_lib::plutus::PlutusV2Script;
use pallas_addresses::{
    Address,
    Network,
};
use pallas_primitives::babbage::Language;
use std::{
    marker::PhantomData,
    rc::Rc,
};
use uplc::{
    ast::{
        Constant,
        FakeNamedDeBruijn,
        NamedDeBruijn,
        Program,
        Term,
    },
    machine::{
        cost_model::ExBudget,
        runtime::convert_constr_to_tag,
    },
    BigInt as AikenBigInt,
    Constr as AikenConstr,
    PlutusData as AikenPlutusData,
};

#[allow(missing_docs)]
pub mod plutus_data;

#[cfg(test)]
mod tests;

// TODO: Maybe make V1 and V2 different types? We want to protect the end user better!
/// Implementatoin of `Validator` for compiled UPLC code
pub struct PlutusValidator<Datum, Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<D, R> PlutusValidator<D, R> {
    /// Create a new V1 `PlutusValidator` from a [`PlutusScriptFile`]
    pub fn new_v1(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v1_policy = PlutusValidator {
            version: TransactionVersion::V1,
            cbor: outer.to_vec(),
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v1_policy)
    }

    /// Create a new V2 `PlutusValidator` from a [`PlutusScriptFile`]
    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = PlutusValidator {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }

    /// Create a new V2 `PlutusValidator` from a [`ValidatorBlueprint`] based on CIP-0057
    pub fn from_blueprint(blueprint: ValidatorBlueprint) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(blueprint.compiled_code())
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = PlutusValidator {
            version: TransactionVersion::V2,
            cbor,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }

    /// Create a new V2 `PlutusValidator` from a CBOR string
    pub fn v2_from_cbor(cbor: String) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(cbor)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = PlutusValidator {
            version: TransactionVersion::V2,
            cbor,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }
}

/// A builder for a `PlutusValidator` that takes one parameter
pub struct OneParamRawValidator<One, Datum, Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _one: PhantomData<One>,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<One: Into<PlutusData>, D, R> OneParamRawValidator<One, D, R> {
    /// Create a new V2 `OneParamRawValidator` from a [`PlutusScriptFile`]
    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_val = OneParamRawValidator {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _one: Default::default(),
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
    }

    /// Create a new V2 `OneParamRawValidator` from a [`ValidatorBlueprint`] based on CIP-0057
    pub fn from_blueprint(blueprint: ValidatorBlueprint) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(blueprint.compiled_code())
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_val = OneParamRawValidator {
            version: TransactionVersion::V2,
            cbor,
            _one: Default::default(),
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
    }

    /// Apply the parameter to the validator to get a full [`PlutusValidator`]
    pub fn apply(&self, one: One) -> RawPlutusScriptResult<PlutusValidator<D, R>> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .unwrap()
                .into();
        let one_data: PlutusData = one.into();
        let one_term = Term::Constant(Rc::new(Constant::Data(one_data.into())));
        let program = program.apply_term(&one_term);
        let fake: Program<FakeNamedDeBruijn> = program.into();
        let new_cbor = fake
            .to_cbor()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let policy = PlutusValidator {
            version: self.version.clone(),
            cbor: new_cbor,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(policy)
    }
}

impl From<PlutusData> for AikenPlutusData {
    fn from(data: PlutusData) -> Self {
        match data {
            PlutusData::Constr(constr) => AikenPlutusData::Constr(constr.into()),
            PlutusData::Map(map) => AikenPlutusData::Map(
                map.into_iter()
                    .map(|(key, value)| {
                        (AikenPlutusData::from(key), AikenPlutusData::from(value))
                    })
                    .collect::<Vec<_>>()
                    .into(),
            ),
            PlutusData::BigInt(big_int) => AikenPlutusData::BigInt(big_int.into()),
            PlutusData::BoundedBytes(bytes) => {
                AikenPlutusData::BoundedBytes(bytes.into())
            }
            PlutusData::Array(data) => {
                AikenPlutusData::Array(data.into_iter().map(Into::into).collect())
            }
        }
    }
}

impl From<Constr<PlutusData>> for AikenConstr<AikenPlutusData> {
    fn from(constr: Constr<PlutusData>) -> Self {
        let tag = convert_constr_to_tag(constr.constr);
        AikenConstr {
            tag: tag.unwrap_or(102),
            any_constructor: match tag {
                Some(_) => None,
                None => Some(constr.constr),
            },
            fields: constr.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BigInt> for AikenBigInt {
    fn from(big_int: BigInt) -> Self {
        match big_int {
            BigInt::Int { neg, val } => {
                let new_val = val as i128;
                let final_val = if neg { -new_val } else { new_val };
                let inner = final_val.try_into().expect(
                    "Since this was converted from a u64, it should always be valid 🤞",
                );
                AikenBigInt::Int(inner)
            }
            BigInt::BigUInt(bytes) => AikenBigInt::BigUInt(bytes.into()),
            BigInt::BigNInt(bytes) => AikenBigInt::BigNInt(bytes.into()),
        }
    }
}

impl<Datum, Redeemer> Validator<Datum, Redeemer> for PlutusValidator<Datum, Redeemer>
where
    Datum: Into<PlutusData> + Send + Sync,
    Redeemer: Into<PlutusData> + Send + Sync,
{
    fn execute(
        &self,
        datum: Datum,
        redeemer: Redeemer,
        ctx: TxContext,
    ) -> ScriptResult<ExecutionCost> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .map_err(as_failed_to_execute)?
                .into();
        let datum_data: PlutusData = datum.into();
        let aiken_datum_data: uplc::PlutusData = datum_data.into();
        let datum_term = Term::Constant(Rc::new(Constant::Data(aiken_datum_data)));
        let program = program.apply_term(&datum_term);
        let redeemer_data: PlutusData = redeemer.into();
        let redeemer_term = Term::Constant(Rc::new(Constant::Data(redeemer_data.into())));
        let program = program.apply_term(&redeemer_term);
        let ctx_data: PlutusData = ctx.into();
        let ctx_term = Term::Constant(Rc::new(Constant::Data(ctx_data.into())));
        let program = program.apply_term(&ctx_term);
        let mut eval_result = match self.version {
            TransactionVersion::V1 => program.eval_version(&Language::PlutusV1),
            TransactionVersion::V2 => program.eval(ExBudget::default()), /* TODO: parameterize */
        };
        let logs = eval_result.logs();
        let cost = eval_result.cost();
        eval_result
            .result()
            .map_err(|e| PlutusScriptError::AikenEval {
                error: format!("{e:?}"),
                logs,
            })
            .map_err(as_failed_to_execute)?;
        Ok(cost.into())
    }

    // TODO: Stop using CML
    fn address(&self, network: Network) -> ScriptResult<Address> {
        let network_index = match network {
            Network::Testnet => 0,
            Network::Mainnet => 1,
            Network::Other(inner) => inner,
        };
        let cbor = self.script_hex().unwrap(); // TODO
        let script = match self.version {
            TransactionVersion::V1 => {
                let script_bytes = hex::decode(&cbor)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                let v1 = PlutusV1Script::from_bytes(script_bytes)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                PlutusScript::from_v1(&v1)
            }
            TransactionVersion::V2 => {
                let script_bytes = hex::decode(&cbor)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                let v2 = PlutusV2Script::from_bytes(script_bytes)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                PlutusScript::from_v2(&v2)
            }
        };
        let script_hash = script.hash();
        let stake_cred = StakeCredential::from_scripthash(&script_hash);
        let enterprise_addr = EnterpriseAddress::new(network_index, &stake_cred);
        let cml_script_address = enterprise_addr.to_address();
        let script_address_str = cml_script_address
            .to_bech32(None)
            .map_err(|e| ScriptError::ScriptHexRetrieval(e.to_string()))?;
        let address = Address::from_bech32(&script_address_str)
            .map_err(|e| ScriptError::ScriptHexRetrieval(e.to_string()))?;
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<String> {
        let wrap = Encoder::new(Vec::new())
            .bytes(&self.cbor)
            .map_err(|e| ScriptError::ScriptHexRetrieval(e.to_string()))?
            .clone()
            .into_writer();

        let hex = hex::encode(wrap);
        Ok(hex)
    }
}
