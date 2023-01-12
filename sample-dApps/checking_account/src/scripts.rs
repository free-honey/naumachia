use crate::CheckingAccountDatums;
use naumachia::address::Address;
use naumachia::scripts::raw_policy_script::OneParamRawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::{ScriptError, ScriptResult, TxContext, ValidatorCode};

pub mod spend_token_policy;

// TODO: Parameterize by some Policy whose tokens are allowed to access and owner
pub struct FakeCheckingAccountValidator;

impl ValidatorCode<CheckingAccountDatums, ()> for FakeCheckingAccountValidator {
    fn execute(
        &self,
        _datum: CheckingAccountDatums,
        _redeemer: (),
        _ctx: TxContext,
    ) -> ScriptResult<()> {
        Ok(())
    }

    fn address(&self, _network: u8) -> ScriptResult<Address> {
        let address = Address::new("fake checking account script");
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

pub struct FakePullerValidator;

impl ValidatorCode<CheckingAccountDatums, ()> for FakePullerValidator {
    fn execute(
        &self,
        _datum: CheckingAccountDatums,
        _redeemer: (),
        _ctx: TxContext,
    ) -> ScriptResult<()> {
        Ok(())
    }

    fn address(&self, _network: u8) -> ScriptResult<Address> {
        let address = Address::new("fake puller script");
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}
