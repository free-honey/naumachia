use crate::{
    ledger_client::cml_client::validator_script::error::{
        RawPlutusScrioptError, RawPlutusScriptResult,
    },
    ledger_client::{cml_client::error::CMLLCError::JsError, cml_client::error::*},
    scripts::as_failed_to_execute,
    scripts::{ScriptResult, TxContext, ValidatorCode},
    Address,
};
use cardano_multiplatform_lib::{
    address::{EnterpriseAddress, StakeCredential},
    plutus::{PlutusScript, PlutusV1Script},
};
use minicbor::Decoder;

use crate::ledger_client::cml_client::validator_script::plutus_data::{BigInt, Constr, PlutusData};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};
use uplc::{
    ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term},
    tx::script_context::{
        ScriptContext, ScriptPurpose, TimeRange, TxInInfo, TxInfo, TxInfoV1, TxOut,
    },
    tx::to_plutus_data::{MintValue, ToPlutusData},
    BigInt as AikenBigInt, Constr as AikenConstr, PlutusData as AikenPlutusData,
    PostAlonzoTransactionOutput, TransactionInput, TransactionOutput, Value,
};

pub mod plutus_data {
    use crate::scripts::TxContext;
    use std::collections::HashMap;

    pub enum PlutusData {
        Constr(Constr<PlutusData>),
        Map(HashMap<PlutusData, PlutusData>),
        BigInt(BigInt),
        BoundedBytes(Vec<u8>),
        Array(Vec<PlutusData>),
    }

    pub struct Constr<T> {
        pub tag: u64,
        pub any_constructor: Option<u64>,
        pub fields: Vec<T>,
    }

    pub enum BigInt {
        Int { neg: bool, val: u64 },
        BigUInt(Vec<u8>),
        BigNInt(Vec<u8>),
    }

    impl From<TxContext> for PlutusData {
        fn from(_: TxContext) -> Self {
            todo!()
        }
    }

    impl From<()> for PlutusData {
        fn from(_: ()) -> Self {
            PlutusData::BoundedBytes(Vec::new())
        }
    }

    // impl AikenTermInterop for i64 {
    //     fn to_term(&self) -> RawPlutusScriptResult<Term<NamedDeBruijn>> {
    //         let constr = Constr {
    //             tag: 121,
    //             any_constructor: None,
    //             fields: vec![PlutusData::BigInt(BigInt::Int((*self).into()))],
    //         };
    //         Ok(Term::Constant(Constant::Data(PlutusData::Constr(constr))))
    //     }
    // }

    impl From<i64> for PlutusData {
        fn from(num: i64) -> Self {
            let neg = num.is_negative();
            let val = num.abs() as u64;
            let big_int = BigInt::Int { neg, val };
            PlutusData::BigInt(big_int)
        }
    }
}

#[cfg(test)]
mod tests;

#[allow(non_snake_case)]
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PlutusScriptFile {
    pub r#type: String,
    pub description: String,
    pub cborHex: String,
}

pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum RawPlutusScrioptError {
        #[error("Error in Aiken Apply: {0:?}")]
        AikenApply(String),
        #[error("Error in Aiken Eval: {error:?}, Logs: {logs:?}")]
        AikenEval { error: String, logs: Vec<String> },
    }

    pub type RawPlutusScriptResult<T, E = RawPlutusScrioptError> = Result<T, E>;
}

pub struct RawPlutusValidator<Datum, Redeemer> {
    script_file: PlutusScriptFile,
    cml_script: PlutusScript,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<D, R> RawPlutusValidator<D, R> {
    pub fn new_v1(script_file: PlutusScriptFile) -> Result<Self> {
        let script_bytes =
            hex::decode(&script_file.cborHex).map_err(|e| CMLLCError::Hex(Box::new(e)))?;
        let v1 = PlutusV1Script::from_bytes(script_bytes).map_err(|e| JsError(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let v1_val = RawPlutusValidator {
            script_file,
            cml_script,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v1_val)
    }
}

pub trait AikenTermInterop: Sized {
    fn to_term(&self) -> RawPlutusScriptResult<Term<NamedDeBruijn>>;
}

// TODO: Use real values https://github.com/MitchTurner/naumachia/issues/39
impl AikenTermInterop for TxContext {
    fn to_term(&self) -> RawPlutusScriptResult<Term<NamedDeBruijn>> {
        let fake_tx_input = TransactionInput {
            transaction_id: uplc::Hash::new([4; 32]),
            index: 0,
        };
        let address = vec![0; 57];
        let post_alonzo_txo = PostAlonzoTransactionOutput {
            address: address.into(),
            value: Value::Coin(1),
            datum_option: None,
            script_ref: None,
        };
        let tx_output = TransactionOutput::PostAlonzo(post_alonzo_txo);
        let tx_out = TxOut::V1(tx_output);
        let tx_in_info = TxInInfo {
            out_ref: fake_tx_input.clone(),
            resolved: tx_out.clone(),
        };
        let tx_info_inner = TxInfoV1 {
            inputs: vec![tx_in_info],
            outputs: vec![tx_out],
            fee: Value::Coin(100000),
            mint: MintValue {
                mint_value: Vec::new().into(),
            },
            dcert: Vec::new(),
            wdrl: vec![],
            valid_range: TimeRange {
                lower_bound: None,
                upper_bound: None,
            },
            signatories: vec![],
            data: vec![],
            id: uplc::Hash::new([1; 32]),
        };

        let tx_info = TxInfo::V1(tx_info_inner);
        let script_context = ScriptContext {
            tx_info,
            purpose: ScriptPurpose::Spending(fake_tx_input),
        };
        let plutus_data = script_context.to_plutus_data();
        Ok(Term::Constant(Constant::Data(plutus_data)))
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
                let final_val = if neg { new_val * -1 } else { new_val };
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

impl<Datum: Into<PlutusData> + Send + Sync, Redeemer: Into<PlutusData> + Send + Sync>
    ValidatorCode<Datum, Redeemer> for RawPlutusValidator<Datum, Redeemer>
{
    fn execute(&self, datum: Datum, redeemer: Redeemer, ctx: TxContext) -> ScriptResult<()> {
        let cbor = hex::decode(&self.script_file.cborHex).map_err(as_failed_to_execute)?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder.bytes().map_err(as_failed_to_execute)?;
        let mut flat_decoder = Decoder::new(outer);
        let flat = flat_decoder.bytes().map_err(as_failed_to_execute)?;
        // println!("hex: {:?}", hex::encode(&flat));
        let program: Program<NamedDeBruijn> = Program::<FakeNamedDeBruijn>::from_flat(flat)
            .unwrap()
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
        term.map_err(|e| RawPlutusScrioptError::AikenEval {
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
        let script_address_str = cml_script_address.to_bech32(None).unwrap();
        let address = Address::Script(script_address_str);
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<&str> {
        Ok(&self.script_file.cborHex)
    }
}
