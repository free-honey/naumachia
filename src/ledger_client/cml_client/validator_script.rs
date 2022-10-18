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
use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;
use uplc::ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term};
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
        let some_string = "some_string".to_string();
        let some_bytes = some_string.as_bytes().to_vec();
        let data = PlutusData::BoundedBytes(some_bytes.into());
        let constr = pallas_primitives::alonzo::Constr {
            tag: 0,
            any_constructor: None,
            fields: vec![data],
        };
        Ok(Term::Constant(Constant::Data(PlutusData::Constr(constr))))
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
        let program = program.apply_term(&datum_term); // TODO
        let redeemer_term = redeemer.to_term().unwrap(); // TODO
        let program = program.apply_term(&redeemer_term); // TODO
        let ctx_term = ctx.to_term().unwrap(); // TODO
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
