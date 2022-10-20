use crate::scripts::ScriptError;
use crate::{
    ledger_client::{cml_client::error::CMLLCError::JsError, cml_client::error::*},
    scripts::{ScriptResult, TxContext, ValidatorCode},
    Address,
};
use cardano_multiplatform_lib::{
    address::{EnterpriseAddress, StakeCredential},
    plutus::{PlutusScript, PlutusV1Script},
};
use minicbor::Decoder;
use pallas_codec::utils::Bytes;
use pallas_crypto::hash::Hash;
use pallas_primitives::alonzo::Multiasset;
use pallas_primitives::babbage::TransactionOutput;
use pallas_primitives::babbage::Value;
use pallas_primitives::babbage::{PostAlonzoTransactionOutput, TransactionInput};
use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;
use uplc::ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term};
use uplc::tx::script_context::{
    ScriptContext, ScriptPurpose, TimeRange, TxInInfo, TxInfo, TxInfoV1, TxOut,
};
use uplc::tx::to_plutus_data::{MintValue, ToPlutusData};
use uplc::PlutusData;

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
    fn to_term(&self) -> Result<Term<NamedDeBruijn>>;
}

impl AikenTermInterop for () {
    fn to_term(&self) -> Result<Term<NamedDeBruijn>> {
        Ok(Term::Constant(Constant::Unit))
    }
}

impl AikenTermInterop for TxContext {
    fn to_term(&self) -> Result<Term<NamedDeBruijn>> {
        let fake_tx_input = TransactionInput {
            transaction_id: Hash::new([4; 32]),
            index: 3,
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
            fee: Value::Coin(0),
            mint: MintValue {
                mint_value: Vec::new().into(),
            },
            dcert: vec![],
            wdrl: vec![],
            valid_range: TimeRange {
                lower_bound: None,
                upper_bound: None,
            },
            signatories: vec![],
            data: vec![],
            id: Hash::new([1; 32]),
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

impl<Datum: AikenTermInterop + Send + Sync, Redeemer: AikenTermInterop + Send + Sync>
    ValidatorCode<Datum, Redeemer> for RawPlutusValidator<Datum, Redeemer>
{
    fn execute(&self, datum: Datum, redeemer: Redeemer, ctx: TxContext) -> ScriptResult<()> {
        let cbor = hex::decode(&self.script_file.cborHex).unwrap();
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder.bytes().unwrap();
        let mut flat_decoder = Decoder::new(&outer);
        let flat = flat_decoder.bytes().unwrap();
        let program: Program<NamedDeBruijn> = Program::<FakeNamedDeBruijn>::from_flat(&flat)
            .unwrap()
            .try_into()
            .unwrap(); // TODO
                       // println!("{}", &program);
        let datum_term = datum.to_term().unwrap(); // TODO
                                                   // dbg!(&datum_term);
        let program = program.apply_term(&datum_term); // TODO
        let redeemer_term = redeemer.to_term().unwrap(); // TODO
                                                         // dbg!(&redeemer_term);
        let program = program.apply_term(&redeemer_term); // TODO
        let ctx_term = ctx.to_term().unwrap(); // TODO
                                               // dbg!(&ctx_term);
        let program = program.apply_term(&ctx_term); // TODO
        let (term, _cost, logs) = program.eval();
        println!("{:?}", &term);
        println!("{:?}", &logs);
        term.map_err(|e| {
            ScriptError::FailedToExecute(format!("Error: {:?}, Logs: {:?}", e, logs))
        })?; // TODO
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
