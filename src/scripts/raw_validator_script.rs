use crate::{
    scripts::as_failed_to_execute,
    scripts::{ScriptResult, TxContext, ValidatorCode},
    Address,
};
use cardano_multiplatform_lib::{
    address::{EnterpriseAddress, StakeCredential},
    plutus::{PlutusScript, PlutusV1Script},
};
use minicbor::Decoder;

use crate::scripts::raw_script::{PlutusScriptFile, RawPlutusScriptError, RawPlutusScriptResult};
use crate::scripts::raw_validator_script::plutus_data::{BigInt, Constr, PlutusData};
use crate::transaction::ScriptVersion;
use cardano_multiplatform_lib::plutus::PlutusV2Script;
use std::marker::PhantomData;
use uplc::{
    ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term},
    tx::script_context::{
        ScriptContext, ScriptPurpose, TimeRange, TxInInfo, TxInfo, TxInfoV1, TxOut,
    },
    tx::to_plutus_data::{MintValue, ToPlutusData},
    BigInt as AikenBigInt, Constr as AikenConstr, PlutusData as AikenPlutusData,
    PostAlonzoTransactionOutput, TransactionInput, TransactionOutput, Value,
};

pub mod plutus_data;

#[cfg(test)]
mod tests;

// TODO: Maybe make V1 and V2 different types? We want to protect the end user better!
pub struct RawPlutusValidator<Datum, Redeemer> {
    cbor_hex: String,
    cml_script: PlutusScript,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<D, R> RawPlutusValidator<D, R> {
    pub fn new_v1(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let script_bytes = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let v1 = PlutusV1Script::from_bytes(script_bytes)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let v1_val = RawPlutusValidator {
            cbor_hex: script_file.cborHex,
            cml_script,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v1_val)
    }

    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let script_bytes = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let v2 = PlutusV2Script::from_bytes(script_bytes)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let cml_script = PlutusScript::from_v2(&v2);
        let v2_val = RawPlutusValidator {
            cbor_hex: script_file.cborHex,
            cml_script,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
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
        let cbor = hex::decode(&self.cbor_hex).map_err(as_failed_to_execute)?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder.bytes().map_err(as_failed_to_execute)?;
        let mut flat_decoder = Decoder::new(outer);
        let flat = flat_decoder.bytes().map_err(as_failed_to_execute)?;
        // println!("hex: {:?}", hex::encode(&flat));
        let program: Program<NamedDeBruijn> = Program::<FakeNamedDeBruijn>::from_flat(flat)
            .map_err(as_failed_to_execute)?
            .try_into()
            .map_err(as_failed_to_execute)?;
        // println!("whole: {}", &program);
        let datum_data: PlutusData = datum.into();
        let datum_term = Term::Constant(Constant::Data(datum_data.into()));
        // dbg!(&datum_term);
        let program = program.apply_term(&datum_term);
        // println!("apply datum: {}", &program);
        let redeemer_data: PlutusData = redeemer.into();
        let redeemer_term = Term::Constant(Constant::Data(redeemer_data.into()));
        // dbg!(&redeemer_term);
        let program = program.apply_term(&redeemer_term);
        // println!("apply redeemer: {}", &program);
        let ctx_data: PlutusData = ctx.into();
        let ctx_term = Term::Constant(Constant::Data(ctx_data.into()));
        // dbg!(&ctx_term);
        let program = program.apply_term(&ctx_term);
        // println!("apply ctx: {}", &program);
        let (term, _cost, logs) = program.eval_v1();
        // println!("{:?}", &term);
        // println!("{:?}", &logs);
        term.map_err(|e| RawPlutusScriptError::AikenEval {
            error: format!("{:?}", e),
            logs,
        })
        .map_err(as_failed_to_execute)?;
        Ok(())
    }

    fn address(&self, network: u8) -> ScriptResult<Address> {
        let script_hash = self.cml_script.hash();
        let stake_cred = StakeCredential::from_scripthash(&script_hash);
        let enterprise_addr = EnterpriseAddress::new(network, &stake_cred);
        let cml_script_address = enterprise_addr.to_address();
        let script_address_str = cml_script_address.to_bech32(None).unwrap(); // TODO: unwrap
        let address = Address::Script(script_address_str);
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<&str> {
        Ok(&self.cbor_hex)
    }
}
