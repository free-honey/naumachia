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
use std::marker::PhantomData;

#[allow(non_snake_case)]
#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct PlutusScriptFile {
    pub r#type: String,
    pub description: String,
    pub cborHex: String,
}

pub struct RawPlutusValidator<Datum, Redeemer> {
    script_hex: String,
    cml_script: PlutusScript,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<D, R> RawPlutusValidator<D, R> {
    pub fn new_v1(script_hex: String) -> Result<Self> {
        let script_bytes = hex::decode(&script_hex).map_err(|e| CMLLCError::Hex(Box::new(e)))?;
        let v1 = PlutusV1Script::from_bytes(script_bytes).map_err(|e| JsError(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let v1_val = RawPlutusValidator {
            script_hex,
            cml_script,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v1_val)
    }
}

impl<Datum: Send + Sync, Redeemer: Send + Sync> ValidatorCode<Datum, Redeemer>
    for RawPlutusValidator<Datum, Redeemer>
{
    fn execute(&self, _datum: Datum, _redeemer: Redeemer, _ctx: TxContext) -> ScriptResult<()> {
        todo!()
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
        Ok(&self.script_hex)
    }
}
