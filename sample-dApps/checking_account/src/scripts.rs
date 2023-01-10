use naumachia::address::Address;
use naumachia::scripts::raw_policy_script::OneParamRawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::{ScriptError, ScriptResult, TxContext, ValidatorCode};

pub mod spend_token_policy;

pub struct FakeCheckingAccountValidator;

impl ValidatorCode<(), ()> for FakeCheckingAccountValidator {
    fn execute(&self, _datum: (), _redeemer: (), _ctx: TxContext) -> ScriptResult<()> {
        Ok(())
    }

    fn address(&self, _network: u8) -> ScriptResult<Address> {
        let address = Address::new("fake checking account");
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}
