use crate::{
    ledger_client::{cml_client::error::CMLLCError::JsError, cml_client::error::*},
    scripts::{ScriptResult, TxContext, ValidatorCode},
    Address,
};
use cardano_multiplatform_lib::{
    address::{EnterpriseAddress, StakeCredential},
    plutus::{PlutusScript, PlutusV1Script},
};
use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;
use uplc::ast::{Constant, DeBruijn, FakeNamedDeBruijn, Name, NamedDeBruijn, Program, Term};
use uplc::parser;

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
    fn from_term(term: Term<DeBruijn>) -> Result<Self>;
}

impl AikenTermInterop for () {
    fn to_term(&self) -> Result<Term<NamedDeBruijn>> {
        Ok(Term::Constant(Constant::Unit))
    }

    fn from_term(_term: Term<DeBruijn>) -> Result<Self> {
        Ok(())
    }
}

impl AikenTermInterop for TxContext {
    fn to_term(&self) -> Result<Term<NamedDeBruijn>> {
        Ok(Term::Constant(Constant::Unit))
    }

    fn from_term(_term: Term<DeBruijn>) -> Result<Self> {
        // TODO: This might be enough reason to split this into two traits or just use `From`
        unimplemented!("When would we use this")
    }
}

impl<Datum: AikenTermInterop + Send + Sync, Redeemer: AikenTermInterop + Send + Sync>
    ValidatorCode<Datum, Redeemer> for RawPlutusValidator<Datum, Redeemer>
{
    fn execute(&self, datum: Datum, redeemer: Redeemer, ctx: TxContext) -> ScriptResult<()> {
        let flat = hex::decode(&self.script_file.cborHex).unwrap();
        let program: Program<NamedDeBruijn> = Program::<FakeNamedDeBruijn>::from_flat(&flat)
            .unwrap()
            .try_into()
            .unwrap(); // TODO
        dbg!(&program);
        let datum_term = datum.to_term().unwrap(); // TODO
        let program = program.apply_term(&datum_term); // TODO
        dbg!(&program);
        let redeemer_term = redeemer.to_term().unwrap(); // TODO
        let program = program.apply_term(&redeemer_term); // TODO
        dbg!(&program);
        let ctx_term = ctx.to_term().unwrap(); // TODO
        let program = program.apply_term(&ctx_term); // TODO
        dbg!(&program);
        let (term, _cost, _logs) = program.eval();
        term.unwrap(); // TODO
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute() {
        let script_file = PlutusScriptFile {
            r#type: "PlutusScriptV1".to_string(),
            description: "".to_string(),
            cborHex: "4e4d01000033222220051200120011".to_string(),
        };
        let script = RawPlutusValidator::new_v1(script_file).unwrap();

        let datum = ();
        let redeemer = ();
        let ctx = TxContext {
            signer: Address::Raw("placeholder".to_string()),
        };

        let res = script.execute(datum, redeemer, ctx).unwrap();
    }
}
