use crate::{
    scripts::as_failed_to_execute,
    scripts::{ScriptResult, ValidatorCode},
    Address,
};
use cardano_multiplatform_lib::{
    address::{EnterpriseAddress, StakeCredential},
    plutus::{PlutusScript, PlutusV1Script},
};
use minicbor::{Decoder, Encoder};

use crate::scripts::context::TxContext;
use crate::scripts::raw_script::{PlutusScriptFile, RawPlutusScriptError, RawPlutusScriptResult};
use crate::scripts::raw_validator_script::plutus_data::{BigInt, Constr, PlutusData};
use crate::scripts::ScriptError;
use crate::transaction::TransactionVersion;
use cardano_multiplatform_lib::plutus::PlutusV2Script;
use std::marker::PhantomData;
use uplc::machine::cost_model::ExBudget;
use uplc::{
    ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term},
    BigInt as AikenBigInt, Constr as AikenConstr, PlutusData as AikenPlutusData,
};

pub mod plutus_data;

#[cfg(test)]
mod tests;

// TODO: Maybe make V1 and V2 different types? We want to protect the end user better!
pub struct RawPlutusValidator<Datum, Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<D, R> RawPlutusValidator<D, R> {
    pub fn new_v1(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let v1_policy = RawPlutusValidator {
            version: TransactionVersion::V1,
            cbor: outer.to_vec(),
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v1_policy)
    }

    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = RawPlutusValidator {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }
}

pub struct OneParamRawValidator<One, Datum, Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _one: PhantomData<One>,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<One: Into<PlutusData>, D, R> OneParamRawValidator<One, D, R> {
    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let v2_val = OneParamRawValidator {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _one: Default::default(),
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
    }

    pub fn apply(&self, one: One) -> RawPlutusScriptResult<RawPlutusValidator<D, R>> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .unwrap()
                .into();
        let one_data: PlutusData = one.into();
        let one_term = Term::Constant(Constant::Data(one_data.into()));
        let program = program.apply_term(&one_term);
        let fake: Program<FakeNamedDeBruijn> = program.into();
        let new_cbor = fake
            .to_cbor()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let policy = RawPlutusValidator {
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
                    .map(|(key, value)| (AikenPlutusData::from(key), AikenPlutusData::from(value)))
                    .collect::<Vec<_>>()
                    .into(),
            ),
            PlutusData::BigInt(big_int) => AikenPlutusData::BigInt(big_int.into()),
            PlutusData::BoundedBytes(bytes) => AikenPlutusData::BoundedBytes(bytes.into()),
            PlutusData::Array(data) => {
                AikenPlutusData::Array(data.into_iter().map(Into::into).collect())
            }
        }
    }
}

impl From<Constr<PlutusData>> for AikenConstr<AikenPlutusData> {
    fn from(constr: Constr<PlutusData>) -> Self {
        AikenConstr {
            tag: constr.tag,
            any_constructor: constr.any_constructor,
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
                let inner = final_val
                    .try_into()
                    .expect("Since this was converted from a u64, it should always be valid 🤞");
                AikenBigInt::Int(inner)
            }
            BigInt::BigUInt(bytes) => AikenBigInt::BigUInt(bytes.into()),
            BigInt::BigNInt(bytes) => AikenBigInt::BigNInt(bytes.into()),
        }
    }
}

impl<Datum, Redeemer> ValidatorCode<Datum, Redeemer> for RawPlutusValidator<Datum, Redeemer>
where
    Datum: Into<PlutusData> + Send + Sync,
    Redeemer: Into<PlutusData> + Send + Sync,
{
    fn execute(&self, datum: Datum, redeemer: Redeemer, ctx: TxContext) -> ScriptResult<()> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .map_err(as_failed_to_execute)?
                .into();
        let datum_data: PlutusData = datum.into();
        let aiken_datum_data: uplc::PlutusData = datum_data.into();
        let datum_term = Term::Constant(Constant::Data(aiken_datum_data));
        let program = program.apply_term(&datum_term);
        let redeemer_data: PlutusData = redeemer.into();
        let redeemer_term = Term::Constant(Constant::Data(redeemer_data.into()));
        let program = program.apply_term(&redeemer_term);
        let ctx_data: PlutusData = ctx.into();
        let ctx_term = Term::Constant(Constant::Data(ctx_data.into()));
        let program = program.apply_term(&ctx_term);
        let (term, _cost, logs) = match self.version {
            TransactionVersion::V1 => program.eval_v1(),
            TransactionVersion::V2 => program.eval(ExBudget::default()), // TODO: parameterize
        };
        term.map_err(|e| RawPlutusScriptError::AikenEval {
            error: format!("{:?}", e),
            logs,
        })
        .map_err(as_failed_to_execute)?;
        Ok(())
    }

    fn address(&self, network: u8) -> ScriptResult<Address> {
        let cbor = self.script_hex().unwrap();
        let script = match self.version {
            TransactionVersion::V1 => {
                let script_bytes =
                    hex::decode(&cbor).map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                let v1 = PlutusV1Script::from_bytes(script_bytes)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                PlutusScript::from_v1(&v1)
            }
            TransactionVersion::V2 => {
                let script_bytes =
                    hex::decode(&cbor).map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                let v2 = PlutusV2Script::from_bytes(script_bytes)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                PlutusScript::from_v2(&v2)
            }
        };
        let script_hash = script.hash();
        let stake_cred = StakeCredential::from_scripthash(&script_hash);
        let enterprise_addr = EnterpriseAddress::new(network, &stake_cred);
        let cml_script_address = enterprise_addr.to_address();
        let script_address_str = cml_script_address
            .to_bech32(None)
            .map_err(|e| ScriptError::ScriptHexRetrieval(e.to_string()))?; // TODO: unwrap
        let address = Address::Script(script_address_str);
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
