use crate::ledger_client::cml_client::error::CMLLCError::JsError;
use crate::ledger_client::cml_client::error::*;
use crate::ledger_client::cml_client::key_manager::TESTNET;
use crate::scripts::{ScriptResult, TxContext, ValidatorCode};
use crate::Address;
use cardano_multiplatform_lib::address::{EnterpriseAddress, StakeCredential};
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script};
use std::marker::PhantomData;

pub struct CMLValidator<Datum, Redeemer> {
    script_hex: String,
    cml_script: PlutusScript,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<D, R> CMLValidator<D, R> {
    pub fn new_v1(script_hex: String) -> Result<Self> {
        let script_bytes = hex::decode(&script_hex).map_err(|e| CMLLCError::Hex(Box::new(e)))?; // TODO
        let v1 = PlutusV1Script::from_bytes(script_bytes).map_err(|e| JsError(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let v1_val = CMLValidator {
            script_hex,
            cml_script,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v1_val)
    }
}

impl<Datum: Send + Sync, Redeemer: Send + Sync> ValidatorCode<Datum, Redeemer>
    for CMLValidator<Datum, Redeemer>
{
    fn execute(&self, _datum: Datum, _redeemer: Redeemer, _ctx: TxContext) -> ScriptResult<()> {
        todo!()
    }

    fn address(&self) -> ScriptResult<Address> {
        let network = TESTNET;
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
